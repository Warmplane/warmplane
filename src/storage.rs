// Rust guideline compliant 2026-11-18
//! Atomic filesystem storage engine for persistent application state.
//!
//! Provides thread-safe, crash-safe atomic JSON file persistence using temporary file
//! writes and POSIX atomic renames (`M-CANONICAL-DOCS`).

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use tracing::debug;

/// The destination was replaced, but syncing its parent directory failed.
#[derive(Debug)]
pub struct ReplacementDurabilityUnknown(std::io::Error);

impl std::fmt::Display for ReplacementDurabilityUnknown {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "replacement is visible, but directory sync failed: {}",
            self.0
        )
    }
}

impl std::error::Error for ReplacementDurabilityUnknown {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Thread-safe manager for an atomically persisted JSON file on disk.
#[derive(Debug, Clone)]
pub struct AtomicFile<T> {
    path: PathBuf,
    _phantom: std::marker::PhantomData<T>,
    #[cfg(all(test, unix))]
    fail_next_directory_sync: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl<T> AtomicFile<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a new `AtomicFile` handler for the specified path.
    ///
    /// # Arguments
    /// * `path` - Destination file path on disk.
    ///
    /// # Returns
    /// An `AtomicFile<T>` instance managing the target file.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            _phantom: std::marker::PhantomData,
            #[cfg(all(test, unix))]
            fail_next_directory_sync: std::sync::Arc::default(),
        }
    }

    /// Returns the target file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads and deserializes data from disk, returning `Ok(None)` if the file does not exist.
    ///
    /// # Returns
    /// `Ok(Some(T))` if the file exists and is valid JSON, `Ok(None)` if file is not found.
    ///
    /// # Errors
    /// Returns an error if reading the file fails or JSON deserialization fails.
    pub fn load_opt(&self) -> Result<Option<T>> {
        match fs::read_to_string(&self.path) {
            Ok(content) => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    return Ok(None);
                }
                let data: T = serde_json::from_str(trimmed).with_context(|| {
                    format!(
                        "Failed to deserialize persistent state from {}",
                        self.path.display()
                    )
                })?;
                debug!(path = %self.path.display(), "loaded persistent state");
                Ok(Some(data))
            }
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err).with_context(|| {
                format!(
                    "Failed to read persistent state file from {}",
                    self.path.display()
                )
            }),
        }
    }

    /// Loads data from disk or returns the provided default value if the file is missing or empty.
    ///
    /// # Arguments
    /// * `default` - Fallback value if file is not present.
    ///
    /// # Returns
    /// Loaded or default value of `T`.
    ///
    /// # Errors
    /// Returns an error if the file exists but contains invalid JSON.
    pub fn load_or_default(&self, default: T) -> Result<T> {
        Ok(self.load_opt()?.unwrap_or(default))
    }

    /// Atomically saves data to disk by writing to a temporary file and renaming it.
    ///
    /// Temporary files have independently generated names in the destination directory and are
    /// automatically removed if writing or persistence fails. On Unix, saved files have mode
    /// `0o600`, regardless of the destination filename.
    ///
    /// # Arguments
    /// * `data` - Reference to data structure implementing `Serialize`.
    ///
    /// # Errors
    /// Returns an error if preparing or replacing the file fails. A
    /// [`ReplacementDurabilityUnknown`] error means the new file is already visible.
    pub fn save(&self, data: &T) -> Result<()> {
        if let Some(directory) = self.replace(data)? {
            #[cfg(all(test, unix))]
            if self
                .fail_next_directory_sync
                .swap(false, std::sync::atomic::Ordering::Relaxed)
            {
                return sync_replacement(
                    Err(std::io::Error::other("injected sync failure")),
                    &self.path,
                );
            }
            sync_replacement(directory.sync_all(), &self.path)?;
        }
        debug!(path = %self.path.display(), "atomically saved persistent state");
        Ok(())
    }

    #[cfg(all(test, unix))]
    pub(crate) fn fail_next_directory_sync(&self) {
        self.fail_next_directory_sync
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn replace(&self, data: &T) -> Result<Option<fs::File>> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create persistent state directory {}",
                        parent.display()
                    )
                })?;
            }
        }

        let json_bytes = serde_json::to_string_pretty(data)
            .context("Failed to serialize state data to JSON format")?;

        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = fs::canonicalize(parent)
            .with_context(|| format!("Failed to resolve state directory {}", parent.display()))?;
        #[cfg(unix)]
        let directory = {
            use rustix::fs::{Mode, OFlags};

            // Retain a directory-only descriptor through replacement and sync. Resolve configured
            // symlinks above, then refuse a final symlink if the path changes before opening it.
            let descriptor = rustix::fs::open(
                &parent,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .with_context(|| format!("Failed to open state directory {}", parent.display()))?;
            fs::File::from(descriptor)
        };

        // Keep the temporary file beside the destination for an atomic rename. Its generated
        // basename is independent of the configured target, and ownership handles cleanup.
        let mut temporary = tempfile::NamedTempFile::new_in(&parent).with_context(|| {
            format!(
                "Failed to create temporary state file in {}",
                parent.display()
            )
        })?;
        temporary
            .write_all(format!("{}\n", json_bytes).as_bytes())
            .context("Failed to write temporary state file")?;
        temporary
            .as_file()
            .sync_all()
            .context("Failed to sync temporary state file")?;
        temporary
            .persist(&self.path)
            .map_err(|error| error.error)
            .with_context(|| {
                format!(
                    "Failed to atomically replace state file {}",
                    self.path.display()
                )
            })?;

        #[cfg(unix)]
        return Ok(Some(directory));
        #[cfg(not(unix))]
        Ok(None)
    }

    /// Saves state and logs any persistence failure using the specified error message context.
    pub fn save_logged(&self, data: &T, failure_msg: &str) {
        if let Err(e) = self.save(data) {
            tracing::error!(error = %e, path = %self.path.display(), "{}", failure_msg);
        }
    }

    /// Deletes the state file if it exists on disk.
    ///
    /// # Errors
    /// Returns an error if file exists but removal fails.
    pub fn remove(&self) -> Result<bool> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err).with_context(|| {
                format!(
                    "Failed to delete persistent state file {}",
                    self.path.display()
                )
            }),
        }
    }
}

fn sync_replacement(result: std::io::Result<()>, path: &Path) -> Result<()> {
    result
        .map_err(ReplacementDurabilityUnknown)
        .with_context(|| format!("Failed to sync state directory for {}", path.display()))
}

/// Helper to ensure the base state directory exists and return paths for state components.
#[derive(Debug, Clone)]
pub struct StateDirectory {
    base_dir: PathBuf,
}

impl StateDirectory {
    /// Creates a new `StateDirectory` instance.
    ///
    /// # Arguments
    /// * `base_dir` - Directory path where state files will reside.
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Ensures the state directory exists on disk.
    ///
    /// # Errors
    /// Returns an error if directory creation fails.
    pub fn ensure_exists(&self) -> Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir).with_context(|| {
                format!(
                    "Failed to create state directory at {}",
                    self.base_dir.display()
                )
            })?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.base_dir, fs::Permissions::from_mode(0o700)).with_context(
                || {
                    format!(
                        "Failed to set private permissions on state directory {}",
                        self.base_dir.display()
                    )
                },
            )?;
        }

        Ok(())
    }

    /// Returns the base directory path.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Returns the file path for persistent HITL approvals.
    pub fn approvals_file(&self) -> PathBuf {
        self.base_dir.join("approvals.json")
    }

    /// Returns the file path for persistent idempotency cache.
    pub fn idempotency_file(&self) -> PathBuf {
        self.base_dir.join("idempotency.json")
    }

    /// Returns the file path for persistent OAuth2 tokens.
    pub fn oauth_tokens_file(&self) -> PathBuf {
        self.base_dir.join("oauth_tokens.json")
    }

    /// Returns the file path for persistent catalog events.
    pub fn catalog_events_file(&self) -> PathBuf {
        self.base_dir.join("catalog_events.json")
    }

    /// Returns the file path for persistent sampling delegation requests.
    pub fn sampling_file(&self) -> PathBuf {
        self.base_dir.join("sampling.json")
    }

    /// Returns the file path for persistent SEP-2663 tasks.
    pub fn tasks_file(&self) -> PathBuf {
        self.base_dir.join("tasks.json")
    }

    /// Returns the file path for the persistent undo ledger.
    pub fn undo_ledger_file(&self) -> PathBuf {
        self.base_dir.join("undo_ledger.json")
    }

    /// Returns the directory path for content-addressed snapshot artifacts.
    pub fn artifacts_dir(&self) -> PathBuf {
        self.base_dir.join("artifacts")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct SampleState {
        counter: u64,
        tags: Vec<String>,
    }

    #[cfg(unix)]
    #[test]
    fn directory_sync_failure_reports_a_visible_replacement() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.json");
        let store = AtomicFile::new(&path);
        store.save(&1_u64).unwrap();

        store.fail_next_directory_sync();
        let error = store.save(&2).unwrap_err();

        assert!(error.is::<ReplacementDurabilityUnknown>());
        assert_eq!(store.load_opt().unwrap(), Some(2));
    }

    #[test]
    fn test_atomic_file_save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_state.json");
        let store = AtomicFile::<SampleState>::new(&file_path);

        assert!(store.load_opt().unwrap().is_none());

        let state = SampleState {
            counter: 42,
            tags: vec!["mcp".to_string(), "warmplane".to_string()],
        };

        store.save(&state).unwrap();
        assert!(file_path.exists());

        let loaded = store.load_opt().unwrap().expect("state should exist");
        assert_eq!(loaded, state);

        let default_state = SampleState {
            counter: 0,
            tags: vec![],
        };
        let loaded_or_default = store.load_or_default(default_state.clone()).unwrap();
        assert_eq!(loaded_or_default, state);

        let removed = store.remove().unwrap();
        assert!(removed);
        assert!(!file_path.exists());

        let loaded_after_remove = store.load_or_default(default_state.clone()).unwrap();
        assert_eq!(loaded_after_remove, default_state);
    }

    #[test]
    fn failed_atomic_replace_cleans_temporary_file_and_preserves_destination() {
        let directory = tempdir().unwrap();
        let destination = directory.path().join("occupied.json");
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("original"), b"unchanged").unwrap();
        let store = AtomicFile::<SampleState>::new(&destination);
        let error = store
            .save(&SampleState {
                counter: 1,
                tags: vec![],
            })
            .unwrap_err();
        assert!(error.to_string().contains("atomically replace"));
        assert_eq!(
            fs::read(destination.join("original")).unwrap(),
            b"unchanged"
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_replace_keeps_arbitrary_filenames_private_and_replaces_symlinks() {
        use std::os::unix::{fs::symlink, fs::PermissionsExt};

        let directory = tempdir().unwrap();
        let original = directory.path().join("outside.json");
        fs::write(&original, b"untouched").unwrap();
        let destination = directory.path().join("customer-state.json");
        symlink(&original, &destination).unwrap();
        let store = AtomicFile::<SampleState>::new(&destination);
        let initial = SampleState {
            counter: 1,
            tags: vec!["first".to_string()],
        };
        store.save(&initial).unwrap();
        assert_eq!(fs::read(&original).unwrap(), b"untouched");
        assert!(!fs::symlink_metadata(&destination)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            fs::metadata(&destination).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let updated = SampleState {
            counter: 2,
            tags: vec![],
        };
        store.save(&updated).unwrap();
        assert_eq!(store.load_opt().unwrap(), Some(updated));
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_save_supports_a_configured_symlink_to_the_state_directory() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().unwrap();
        let actual = directory.path().join("actual");
        fs::create_dir(&actual).unwrap();
        let configured = directory.path().join("configured");
        symlink(&actual, &configured).unwrap();

        let store = AtomicFile::<SampleState>::new(configured.join("state.json"));
        let state = SampleState {
            counter: 3,
            tags: vec![],
        };
        store.save(&state).unwrap();
        assert_eq!(store.load_opt().unwrap(), Some(state));
        assert!(actual.join("state.json").is_file());
        assert_eq!(fs::read_dir(actual).unwrap().count(), 1);
    }

    #[test]
    fn test_state_directory_paths_and_creation() {
        let dir = tempdir().unwrap();
        let state_dir_path = dir.path().join("nested").join(".warmplane_state");
        let state_dir = StateDirectory::new(&state_dir_path);

        assert!(!state_dir_path.exists());
        state_dir.ensure_exists().unwrap();
        assert!(state_dir_path.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&state_dir_path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }

        assert_eq!(
            state_dir.approvals_file(),
            state_dir_path.join("approvals.json")
        );
        assert_eq!(
            state_dir.idempotency_file(),
            state_dir_path.join("idempotency.json")
        );
        assert_eq!(
            state_dir.oauth_tokens_file(),
            state_dir_path.join("oauth_tokens.json")
        );
        assert_eq!(
            state_dir.catalog_events_file(),
            state_dir_path.join("catalog_events.json")
        );
    }
}
