// Rust guideline compliant 2026-08-18

//! Atomic filesystem storage engine for persistent application state.
//!
//! Provides thread-safe, crash-safe atomic JSON file persistence using temporary file
//! writes and POSIX atomic renames (`M-CANONICAL-DOCS`).

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tracing::debug;

/// Thread-safe manager for an atomically persisted JSON file on disk.
#[derive(Debug, Clone)]
pub struct AtomicFile<T> {
    path: PathBuf,
    _phantom: std::marker::PhantomData<T>,
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
    /// # Arguments
    /// * `data` - Reference to data structure implementing `Serialize`.
    ///
    /// # Errors
    /// Returns an error if serialization, directory creation, writing, or renaming fails.
    pub fn save(&self, data: &T) -> Result<()> {
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

        let tmp_path = self
            .path
            .with_extension(format!("tmp.{}", std::process::id()));

        fs::write(&tmp_path, format!("{}\n", json_bytes)).with_context(|| {
            format!(
                "Failed to write temporary state file at {}",
                tmp_path.display()
            )
        })?;

        fs::rename(&tmp_path, &self.path).with_context(|| {
            // Attempt cleanup of temp file if rename fails
            let _ = fs::remove_file(&tmp_path);
            format!(
                "Failed to atomically replace target state file {} with {}",
                self.path.display(),
                tmp_path.display()
            )
        })?;

        debug!(path = %self.path.display(), "atomically saved persistent state");
        Ok(())
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
    fn test_state_directory_paths_and_creation() {
        let dir = tempdir().unwrap();
        let state_dir_path = dir.path().join("nested").join(".warmplane_state");
        let state_dir = StateDirectory::new(&state_dir_path);

        assert!(!state_dir_path.exists());
        state_dir.ensure_exists().unwrap();
        assert!(state_dir_path.exists());

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
