// Rust guideline compliant 2026-11-18

//! Persistent undo ledger and content-addressed snapshot storage.

use anyhow::{Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

use crate::{
    storage::AtomicFile,
    undo::types::{CompensableUsage, UndoEntry, UndoLedger},
};

/// Stores JSON snapshots under their SHA-256 content hash.
#[derive(Clone, Debug)]
pub struct SnapshotStore {
    directory: PathBuf,
}

impl SnapshotStore {
    /// Opens a snapshot store at `directory`, creating it with private permissions when needed.
    pub fn open(directory: impl AsRef<Path>) -> Result<Self> {
        let directory = directory.as_ref().to_path_buf();
        fs::create_dir_all(&directory).with_context(|| {
            format!(
                "failed to create snapshot directory {}",
                directory.display()
            )
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
        }
        Ok(Self { directory })
    }

    /// Saves a JSON value once and returns its content-addressed artifact hash.
    pub fn put(&self, value: &Value) -> Result<String> {
        let bytes = serde_json::to_vec(value).context("failed to encode snapshot JSON")?;
        let hash = hex::encode(Sha256::digest(&bytes));
        let path = self.directory.join(format!("art_{}.json", hash));
        if path.exists() {
            let existing = fs::read(&path).with_context(|| {
                format!("failed to verify snapshot artifact {}", path.display())
            })?;
            anyhow::ensure!(
                hex::encode(Sha256::digest(&existing)) == hash,
                "existing snapshot artifact hash mismatch"
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            }
        } else {
            let temporary =
                self.directory
                    .join(format!(".art_{}.{}.tmp", hash, rand::random::<u64>()));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary).with_context(|| {
                format!("failed to create snapshot artifact {}", temporary.display())
            })?;
            file.write_all(&bytes).with_context(|| {
                format!("failed to write snapshot artifact {}", temporary.display())
            })?;
            file.sync_all()
                .context("failed to sync snapshot artifact")?;
            drop(file);
            match fs::rename(&temporary, &path) {
                Ok(()) => {}
                Err(error) if path.exists() => {
                    let _ = fs::remove_file(&temporary);
                    let _ = error;
                }
                Err(error) => return Err(error).context("failed to commit snapshot artifact"),
            }
            fs::File::open(&self.directory)
                .and_then(|directory| directory.sync_all())
                .context("failed to sync snapshot directory")?;
        }
        Ok(hash)
    }

    /// Loads and verifies a snapshot by its SHA-256 artifact hash.
    pub fn get(&self, hash: &str) -> Result<Value> {
        anyhow::ensure!(
            hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "invalid snapshot hash"
        );
        let path = self.directory.join(format!("art_{}.json", hash));
        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read snapshot artifact {}", path.display()))?;
        let actual = hex::encode(Sha256::digest(&bytes));
        anyhow::ensure!(actual == hash, "snapshot artifact hash mismatch");
        serde_json::from_slice(&bytes).context("failed to decode snapshot JSON")
    }
}

/// Thread-safe persistent manager for registered undo actions.
#[derive(Clone)]
pub struct UndoLedgerStore {
    ledger: Arc<Mutex<UndoLedger>>,
    storage: AtomicFile<UndoLedger>,
}

/// Result of atomically claiming an undo entry for operator execution.
#[derive(Clone, Debug, PartialEq)]
pub enum UndoClaimResult {
    /// The entry is durably marked `Executing` and may be dispatched.
    Claimed(UndoEntry),
    /// The undo window expired and the entry is durably marked `Expired`.
    Expired(UndoEntry),
    /// The entry is terminal, executing, missing, or otherwise unavailable.
    Unavailable(Option<UndoEntry>),
}

/// Values that identify and authorize a compensable invocation reservation.
pub struct CompensableReservation<'a> {
    /// Tenant that owns the invocation.
    pub tenant_id: &'a str,
    /// Active profile, if profile scoped.
    pub profile_id: Option<&'a str>,
    /// Capability being invoked.
    pub capability: &'a str,
    /// Maximum non-approved invocations in the rolling 24-hour window.
    pub daily_limit: u64,
    /// Idempotency key used to avoid counting a replay twice.
    pub idempotency_key: Option<&'a str>,
    /// Whether an operator approved an invocation after reaching the cap.
    pub approved_over_limit: bool,
    /// Current Unix timestamp in milliseconds.
    pub now_ms: u64,
}

impl UndoLedgerStore {
    /// Opens or creates an undo ledger backed by an atomic JSON file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let storage = AtomicFile::new(path);
        let ledger = storage.load_or_default(UndoLedger::default())?;
        let mut undo_ids = HashSet::with_capacity(ledger.entries.len());
        anyhow::ensure!(
            ledger
                .entries
                .iter()
                .all(|entry| undo_ids.insert(entry.undo_id.as_str())),
            "undo ledger contains duplicate entry IDs"
        );
        Ok(Self {
            ledger: Arc::new(Mutex::new(ledger)),
            storage,
        })
    }

    /// Appends an undo entry and persists the complete ledger before returning.
    pub async fn register(&self, entry: UndoEntry) -> Result<()> {
        let mut guard = self.ledger.lock().await;
        anyhow::ensure!(
            !guard
                .entries
                .iter()
                .any(|registered| registered.undo_id == entry.undo_id),
            "undo entry ID '{}' is already registered",
            entry.undo_id
        );
        guard.entries.push(entry);
        if let Err(error) = self.storage.save(&guard) {
            guard.entries.pop();
            return Err(error).context("failed to persist undo ledger");
        }
        Ok(())
    }

    /// Returns a snapshot of all ledger entries.
    pub async fn entries(&self) -> Vec<UndoEntry> {
        self.ledger.lock().await.entries.clone()
    }

    /// Returns one ledger entry by its stable identifier.
    pub async fn get(&self, undo_id: &str) -> Option<UndoEntry> {
        self.ledger
            .lock()
            .await
            .entries
            .iter()
            .find(|entry| entry.undo_id == undo_id)
            .cloned()
    }

    /// Returns entries belonging to one tenant, newest first, with optional work-item/status filters.
    pub async fn list_for_tenant(
        &self,
        tenant_id: &str,
        work_item_id: Option<&str>,
        status: Option<&crate::undo::UndoEntryStatus>,
    ) -> Vec<UndoEntry> {
        let mut entries = self
            .ledger
            .lock()
            .await
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.tenant_id == tenant_id
                    && work_item_id.is_none_or(|id| entry.work_item_id.as_deref() == Some(id))
                    && status.is_none_or(|wanted| &entry.status == wanted)
            })
            .map(|(index, entry)| (index, entry.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|(left_index, left), (right_index, right)| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right_index.cmp(left_index))
        });
        entries.into_iter().map(|(_, entry)| entry).collect()
    }

    /// Claims an entry for an operator, optionally forcing an expired entry after authorization.
    pub async fn claim_for_operator(
        &self,
        undo_id: &str,
        now_ms: u64,
        force_expired: bool,
    ) -> Result<UndoClaimResult> {
        let mut guard = self.ledger.lock().await;
        let Some(index) = guard
            .entries
            .iter()
            .position(|entry| entry.undo_id == undo_id)
        else {
            return Ok(UndoClaimResult::Unavailable(None));
        };
        let entry = &guard.entries[index];
        let expired = entry.expires_at_ms <= now_ms;
        if entry.status == crate::undo::UndoEntryStatus::Armed && expired && !force_expired {
            let previous = entry.status.clone();
            guard.entries[index].status = crate::undo::UndoEntryStatus::Expired;
            if let Err(error) = self.storage.save(&guard) {
                guard.entries[index].status = previous;
                return Err(error).context("failed to persist expired undo status");
            }
            return Ok(UndoClaimResult::Expired(guard.entries[index].clone()));
        }
        let forceable = force_expired
            && expired
            && matches!(
                entry.status,
                crate::undo::UndoEntryStatus::Armed | crate::undo::UndoEntryStatus::Expired
            );
        if entry.status != crate::undo::UndoEntryStatus::Armed && !forceable {
            return Ok(UndoClaimResult::Unavailable(Some(entry.clone())));
        }
        guard.entries[index].status = crate::undo::UndoEntryStatus::Executing;
        let claimed = guard.entries[index].clone();
        if let Err(error) = self.storage.save(&guard) {
            guard.entries[index].status = if expired {
                crate::undo::UndoEntryStatus::Expired
            } else {
                crate::undo::UndoEntryStatus::Armed
            };
            return Err(error).context("failed to persist operator undo claim");
        }
        Ok(UndoClaimResult::Claimed(claimed))
    }

    /// Claims an armed, unexpired entry for execution and persists the claim.
    ///
    /// The transition is serialized with other ledger operations. A missing, expired, or
    /// non-armed entry is left unchanged, which makes concurrent retries safe to reject.
    ///
    /// # Arguments
    /// * `undo_id` - Stable identifier of the entry to claim.
    /// * `now_ms` - Current Unix timestamp in milliseconds.
    ///
    /// # Returns
    /// The entry in `Executing` state when the claim succeeds, or `None` when it is unavailable.
    ///
    /// # Errors
    /// Returns an error if persisting the claim fails; the in-memory status is restored.
    pub async fn claim(&self, undo_id: &str, now_ms: u64) -> Result<Option<UndoEntry>> {
        let mut guard = self.ledger.lock().await;
        let Some(entry) = guard
            .entries
            .iter_mut()
            .find(|entry| entry.undo_id == undo_id)
        else {
            return Ok(None);
        };
        if entry.status != crate::undo::UndoEntryStatus::Armed || entry.expires_at_ms <= now_ms {
            return Ok(None);
        }
        entry.status = crate::undo::UndoEntryStatus::Executing;
        let claimed = entry.clone();
        if let Err(error) = self.storage.save(&guard) {
            if let Some(entry) = guard
                .entries
                .iter_mut()
                .find(|entry| entry.undo_id == undo_id)
            {
                entry.status = crate::undo::UndoEntryStatus::Armed;
            }
            return Err(error).context("failed to persist undo claim");
        }
        Ok(Some(claimed))
    }

    /// Persists a terminal status for an executing entry.
    ///
    /// Only `Reverted`, `Failed`, and `Expired` are accepted. If persistence fails, the entry
    /// remains `Executing` so a restart will not automatically redispatch an uncertain inverse.
    ///
    /// # Arguments
    /// * `undo_id` - Stable identifier of the claimed entry.
    /// * `status` - Terminal outcome to persist.
    ///
    /// # Errors
    /// Returns an error when the entry is missing, not executing, the status is not terminal,
    /// or the updated ledger cannot be persisted.
    pub async fn finish(&self, undo_id: &str, status: crate::undo::UndoEntryStatus) -> Result<()> {
        anyhow::ensure!(
            matches!(
                status,
                crate::undo::UndoEntryStatus::Reverted
                    | crate::undo::UndoEntryStatus::Failed
                    | crate::undo::UndoEntryStatus::Expired
            ),
            "undo completion status must be terminal"
        );
        let mut guard = self.ledger.lock().await;
        let entry = guard
            .entries
            .iter_mut()
            .find(|entry| entry.undo_id == undo_id)
            .context("undo entry not found")?;
        anyhow::ensure!(
            entry.status == crate::undo::UndoEntryStatus::Executing,
            "undo entry is not executing"
        );
        let previous = entry.status.clone();
        entry.status = status;
        if let Err(error) = self.storage.save(&guard) {
            if let Some(entry) = guard
                .entries
                .iter_mut()
                .find(|entry| entry.undo_id == undo_id)
            {
                entry.status = previous;
            }
            return Err(error).context("failed to persist undo completion");
        }
        Ok(())
    }

    /// Releases a claimed undo entry back to `Armed` when a pre-dispatch safety check blocks it.
    ///
    /// # Arguments
    /// * `undo_id` - Stable undo entry identifier whose claim should be released.
    ///
    /// # Errors
    /// Returns an error if the entry is not executing or the armed transition cannot be persisted.
    pub async fn release_operator_claim(&self, undo_id: &str) -> Result<()> {
        let mut guard = self.ledger.lock().await;
        let entry = guard
            .entries
            .iter_mut()
            .find(|entry| entry.undo_id == undo_id)
            .context("undo entry not found")?;
        anyhow::ensure!(
            entry.status == crate::undo::UndoEntryStatus::Executing,
            "undo entry is not executing"
        );
        let previous = entry.status.clone();
        entry.status = crate::undo::UndoEntryStatus::Armed;
        if let Err(error) = self.storage.save(&guard) {
            if let Some(entry) = guard
                .entries
                .iter_mut()
                .find(|entry| entry.undo_id == undo_id)
            {
                entry.status = previous;
            }
            return Err(error).context("failed to release undo claim");
        }
        Ok(())
    }

    /// Reserves a compensable invocation or reports that operator approval is required.
    pub async fn reserve_compensable(
        &self,
        reservation: CompensableReservation<'_>,
    ) -> Result<bool> {
        let CompensableReservation {
            tenant_id,
            profile_id,
            capability,
            daily_limit,
            idempotency_key,
            approved_over_limit,
            now_ms,
        } = reservation;
        let mut guard = self.ledger.lock().await;
        guard
            .compensable_usage
            .retain(|usage| now_ms.saturating_sub(usage.timestamp_ms) < 86_400_000);
        if let Some(key) = idempotency_key {
            if guard.compensable_usage.iter().any(|usage| {
                usage.idempotency_key.as_deref() == Some(key)
                    && usage.tenant_id == tenant_id
                    && usage.profile_id.as_deref() == profile_id
                    && usage.capability == capability
            }) {
                return Ok(true);
            }
        }
        let count = guard
            .compensable_usage
            .iter()
            .filter(|usage| {
                usage.tenant_id == tenant_id
                    && usage.profile_id.as_deref() == profile_id
                    && usage.capability == capability
            })
            .count() as u64;
        if count >= daily_limit && !approved_over_limit {
            return Ok(false);
        }
        guard.compensable_usage.push(CompensableUsage {
            tenant_id: tenant_id.to_string(),
            profile_id: profile_id.map(ToString::to_string),
            capability: capability.to_string(),
            idempotency_key: idempotency_key.map(ToString::to_string),
            timestamp_ms: now_ms,
        });
        if let Err(error) = self.storage.save(&guard) {
            guard.compensable_usage.pop();
            return Err(error).context("failed to persist compensable invocation reservation");
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::{CompensableReservation, SnapshotStore, UndoLedgerStore};
    use crate::undo::{UndoEntry, UndoEntryStatus};
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn snapshot_store_deduplicates_and_verifies_content() {
        let directory = tempdir().unwrap();
        let store = SnapshotStore::open(directory.path().join("artifacts")).unwrap();
        let snapshot = json!({"priority": "high", "version": 3});
        let first_hash = store.put(&snapshot).unwrap();
        let second_hash = store.put(&snapshot).unwrap();

        assert_eq!(first_hash, second_hash);
        assert_eq!(store.get(&first_hash).unwrap(), snapshot);
        assert!(store.get("../outside").is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let artifact = directory
                .path()
                .join("artifacts")
                .join(format!("art_{first_hash}.json"));
            assert_eq!(
                std::fs::metadata(artifact).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                std::fs::metadata(directory.path().join("artifacts"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }
    }

    #[test]
    fn snapshot_store_rejects_corrupt_existing_content_addressed_artifact() {
        let directory = tempdir().unwrap();
        let store = SnapshotStore::open(directory.path().join("artifacts")).unwrap();
        let snapshot = json!({"priority": "high"});
        let hash = store.put(&snapshot).unwrap();
        let artifact = directory
            .path()
            .join("artifacts")
            .join(format!("art_{hash}.json"));
        std::fs::write(&artifact, b"{\"priority\":\"low\"}").unwrap();

        assert!(store.put(&snapshot).is_err());
        assert!(store.get(&hash).is_err());
    }

    #[tokio::test]
    async fn undo_ledger_persists_registered_entries() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("undo_ledger.json");
        let store = UndoLedgerStore::open(&path).unwrap();
        let entry = UndoEntry {
            undo_id: "undo-test-1".to_string(),
            operation_id: Some("op-1".to_string()),
            work_item_id: None,
            batch_id: None,
            tenant_id: "tenant-a".to_string(),
            profile_id: Some("coding".to_string()),
            actor_id: Some("actor-a".to_string()),
            forward_capability: "tickets.set_priority".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "linear.update_issue".to_string(),
            resolved_inverse_args: json!({"id": "ISSUE-1", "priority": "low"}),
            change_preview: Vec::new(),
            snapshot_artifact_hash: Some("a".repeat(64)),
            pre_undo_check: None,
            expected_post_snapshot_hash: None,
            created_at_ms: 100,
            expires_at_ms: 200,
            status: UndoEntryStatus::Armed,
        };
        store.register(entry.clone()).await.unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }

        let reopened = UndoLedgerStore::open(path).unwrap();
        assert_eq!(reopened.entries().await, vec![entry]);
    }

    #[tokio::test]
    async fn operator_listing_is_tenant_scoped_and_expiry_override_is_durable() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("undo_operator.json");
        let store = UndoLedgerStore::open(&path).unwrap();
        let make_entry = |undo_id: &str, tenant_id: &str| UndoEntry {
            undo_id: undo_id.to_string(),
            operation_id: None,
            work_item_id: Some("work-1".to_string()),
            batch_id: None,
            tenant_id: tenant_id.to_string(),
            profile_id: None,
            actor_id: None,
            forward_capability: "forward.call".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "inverse.call".to_string(),
            resolved_inverse_args: json!({"secret": "hidden"}),
            change_preview: Vec::new(),
            snapshot_artifact_hash: None,
            pre_undo_check: None,
            expected_post_snapshot_hash: None,
            created_at_ms: 1,
            expires_at_ms: 10,
            status: UndoEntryStatus::Armed,
        };
        store.register(make_entry("a", "tenant-a")).await.unwrap();
        store.register(make_entry("b", "tenant-b")).await.unwrap();

        let visible = store
            .list_for_tenant("tenant-a", Some("work-1"), None)
            .await;
        assert_eq!(
            visible
                .iter()
                .map(|entry| entry.undo_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a"]
        );
        assert!(matches!(
            store.claim_for_operator("a", 10, false).await.unwrap(),
            crate::undo::UndoClaimResult::Expired(_)
        ));
        assert!(matches!(
            store.claim_for_operator("a", 10, true).await.unwrap(),
            crate::undo::UndoClaimResult::Claimed(_)
        ));

        let reopened = UndoLedgerStore::open(path).unwrap();
        assert_eq!(
            reopened.get("a").await.unwrap().status,
            UndoEntryStatus::Executing
        );
        assert_eq!(
            reopened.get("b").await.unwrap().status,
            UndoEntryStatus::Armed
        );
    }

    #[tokio::test]
    async fn undo_claim_and_completion_are_persisted_and_terminal_entries_cannot_reclaim() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("undo_ledger.json");
        let store = UndoLedgerStore::open(&path).unwrap();
        let entry = UndoEntry {
            undo_id: "undo-transition".to_string(),
            operation_id: None,
            work_item_id: Some("work-1".to_string()),
            batch_id: Some("batch-1".to_string()),
            tenant_id: "tenant-a".to_string(),
            profile_id: None,
            actor_id: None,
            forward_capability: "mock.mutate".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "mock.restore".to_string(),
            resolved_inverse_args: json!({"value": 0}),
            change_preview: Vec::new(),
            snapshot_artifact_hash: None,
            pre_undo_check: None,
            expected_post_snapshot_hash: None,
            created_at_ms: 100,
            expires_at_ms: 200,
            status: UndoEntryStatus::Armed,
        };
        store.register(entry).await.unwrap();
        assert_eq!(
            store
                .claim("undo-transition", 150)
                .await
                .unwrap()
                .unwrap()
                .status,
            UndoEntryStatus::Executing
        );
        let recovered = UndoLedgerStore::open(&path).unwrap();
        assert_eq!(
            recovered.entries().await[0].status,
            UndoEntryStatus::Executing
        );
        assert!(recovered
            .claim("undo-transition", 150)
            .await
            .unwrap()
            .is_none());
        recovered
            .finish("undo-transition", UndoEntryStatus::Reverted)
            .await
            .unwrap();
        assert!(recovered
            .claim("undo-transition", 150)
            .await
            .unwrap()
            .is_none());
        let reopened = UndoLedgerStore::open(path).unwrap();
        assert_eq!(
            reopened.entries().await[0].status,
            UndoEntryStatus::Reverted
        );
    }

    #[tokio::test]
    async fn undo_ledger_rejects_duplicate_ids_without_mutating_the_ledger() {
        let directory = tempdir().unwrap();
        let store = UndoLedgerStore::open(directory.path().join("undo_ledger.json")).unwrap();
        let entry = UndoEntry {
            undo_id: "undo-duplicate".to_string(),
            operation_id: None,
            work_item_id: None,
            batch_id: None,
            tenant_id: "tenant-a".to_string(),
            profile_id: None,
            actor_id: None,
            forward_capability: "tickets.set_priority".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "tickets.set_priority".to_string(),
            resolved_inverse_args: json!({"id": "ISSUE-1", "priority": "low"}),
            change_preview: Vec::new(),
            snapshot_artifact_hash: None,
            pre_undo_check: None,
            expected_post_snapshot_hash: None,
            created_at_ms: 100,
            expires_at_ms: 200,
            status: UndoEntryStatus::Armed,
        };

        store.register(entry.clone()).await.unwrap();
        assert!(store.register(entry.clone()).await.is_err());
        assert_eq!(store.entries().await, vec![entry]);
    }

    #[test]
    fn undo_ledger_rejects_duplicate_ids_loaded_from_disk() {
        use crate::{storage::AtomicFile, undo::UndoLedger};

        let directory = tempdir().unwrap();
        let path = directory.path().join("undo_ledger.json");
        let entry = UndoEntry {
            undo_id: "undo-duplicate-on-disk".to_string(),
            operation_id: None,
            work_item_id: None,
            batch_id: None,
            tenant_id: "tenant-a".to_string(),
            profile_id: None,
            actor_id: None,
            forward_capability: "tickets.set_priority".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "tickets.set_priority".to_string(),
            resolved_inverse_args: json!({"id": "ISSUE-1", "priority": "low"}),
            change_preview: Vec::new(),
            snapshot_artifact_hash: None,
            pre_undo_check: None,
            expected_post_snapshot_hash: None,
            created_at_ms: 100,
            expires_at_ms: 200,
            status: UndoEntryStatus::Armed,
        };
        AtomicFile::new(&path)
            .save(&UndoLedger {
                entries: vec![entry.clone(), entry],
                compensable_usage: Vec::new(),
            })
            .unwrap();

        assert!(UndoLedgerStore::open(path).is_err());
    }

    #[tokio::test]
    async fn compensable_reservations_enforce_rolling_daily_scope() {
        let directory = tempdir().unwrap();
        let store = UndoLedgerStore::open(directory.path().join("undo_ledger.json")).unwrap();
        assert!(store
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("coding"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("k1"),
                approved_over_limit: false,
                now_ms: 10_000
            })
            .await
            .unwrap());
        assert!(store
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("coding"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("k1"),
                approved_over_limit: false,
                now_ms: 15_000
            })
            .await
            .unwrap());
        assert!(!store
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("coding"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("k2"),
                approved_over_limit: false,
                now_ms: 20_000
            })
            .await
            .unwrap());
        assert!(store
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("coding"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("k2"),
                approved_over_limit: true,
                now_ms: 20_000
            })
            .await
            .unwrap());
        assert!(store
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("support"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("k3"),
                approved_over_limit: false,
                now_ms: 20_001
            })
            .await
            .unwrap());
        assert!(store
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("coding"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("k4"),
                approved_over_limit: false,
                now_ms: 86_420_001
            })
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn compensable_usage_survives_ledger_reopen() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("undo_ledger.json");
        let store = UndoLedgerStore::open(&path).unwrap();
        assert!(store
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("coding"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("first-attempt"),
                approved_over_limit: false,
                now_ms: 10_000,
            })
            .await
            .unwrap());
        drop(store);

        let reopened = UndoLedgerStore::open(path).unwrap();
        assert!(!reopened
            .reserve_compensable(CompensableReservation {
                tenant_id: "tenant-a",
                profile_id: Some("coding"),
                capability: "billing.refund",
                daily_limit: 1,
                idempotency_key: Some("second-attempt"),
                approved_over_limit: false,
                now_ms: 20_000,
            })
            .await
            .unwrap());
    }
}
