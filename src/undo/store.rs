// Rust guideline compliant 2026-11-16

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
            tenant_id: "tenant-a".to_string(),
            profile_id: Some("coding".to_string()),
            actor_id: Some("actor-a".to_string()),
            forward_capability: "tickets.set_priority".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "linear.update_issue".to_string(),
            resolved_inverse_args: json!({"id": "ISSUE-1", "priority": "low"}),
            snapshot_artifact_hash: Some("a".repeat(64)),
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
    async fn undo_ledger_rejects_duplicate_ids_without_mutating_the_ledger() {
        let directory = tempdir().unwrap();
        let store = UndoLedgerStore::open(directory.path().join("undo_ledger.json")).unwrap();
        let entry = UndoEntry {
            undo_id: "undo-duplicate".to_string(),
            operation_id: None,
            work_item_id: None,
            tenant_id: "tenant-a".to_string(),
            profile_id: None,
            actor_id: None,
            forward_capability: "tickets.set_priority".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "tickets.set_priority".to_string(),
            resolved_inverse_args: json!({"id": "ISSUE-1", "priority": "low"}),
            snapshot_artifact_hash: None,
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
            tenant_id: "tenant-a".to_string(),
            profile_id: None,
            actor_id: None,
            forward_capability: "tickets.set_priority".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "tickets.set_priority".to_string(),
            resolved_inverse_args: json!({"id": "ISSUE-1", "priority": "low"}),
            snapshot_artifact_hash: None,
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
