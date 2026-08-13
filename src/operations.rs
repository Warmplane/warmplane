// Rust guideline compliant 2026-08-13

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// In-flight async operation registry providing token cancellation tracking.
#[derive(Clone, Default)]
pub struct OperationRegistry {
    /// Map of active request IDs to their cancellation tokens.
    pub active_tokens: Arc<RwLock<HashMap<String, CancellationToken>>>,
}

impl OperationRegistry {
    /// Creates a new empty `OperationRegistry`.
    ///
    /// # Returns
    /// An empty `OperationRegistry` instance.
    pub fn new() -> Self {
        Self {
            active_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new active request ID and returns a `CancellationToken`.
    ///
    /// # Arguments
    /// * `request_id` - Unique request identifier string.
    ///
    /// # Returns
    /// A new `CancellationToken` linked to the registered request.
    pub async fn register(&self, request_id: impl AsRef<str>) -> CancellationToken {
        let token = CancellationToken::new();
        let mut map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, CancellationToken>> =
            self.active_tokens.write().await;
        map.insert(request_id.as_ref().to_string(), token.clone());
        token
    }

    /// Unregisters an active request ID after operation completion.
    ///
    /// # Arguments
    /// * `request_id` - Request identifier string.
    pub async fn unregister(&self, request_id: impl AsRef<str>) {
        let mut map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, CancellationToken>> =
            self.active_tokens.write().await;
        map.remove(request_id.as_ref());
    }

    /// Cancels an in-flight operation by request ID if present.
    ///
    /// # Arguments
    /// * `request_id` - Request identifier string to cancel.
    ///
    /// # Returns
    /// `true` if an active token was found and cancelled, `false` otherwise.
    pub async fn cancel(&self, request_id: impl AsRef<str>) -> bool {
        let mut map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, CancellationToken>> =
            self.active_tokens.write().await;
        if let Some(token) = map.remove(request_id.as_ref()) {
            token.cancel();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_operation_cancellation() {
        let registry = OperationRegistry::new();
        let req_id = "req-test-cancel";

        let token = registry.register(req_id).await;
        assert!(!token.is_cancelled());

        let cancelled = registry.cancel(req_id).await;
        assert!(cancelled);
        assert!(token.is_cancelled());

        let cancel_again = registry.cancel(req_id).await;
        assert!(!cancel_again);
    }
}
