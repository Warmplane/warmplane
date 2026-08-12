use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Default)]
pub struct OperationRegistry {
    pub active_tokens: Arc<RwLock<HashMap<String, CancellationToken>>>,
}

impl OperationRegistry {
    pub fn new() -> Self {
        Self {
            active_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, request_id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        let mut map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, CancellationToken>> =
            self.active_tokens.write().await;
        map.insert(request_id.to_string(), token.clone());
        token
    }

    pub async fn unregister(&self, request_id: &str) {
        let mut map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, CancellationToken>> =
            self.active_tokens.write().await;
        map.remove(request_id);
    }

    pub async fn cancel(&self, request_id: &str) -> bool {
        let mut map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, CancellationToken>> =
            self.active_tokens.write().await;
        if let Some(token) = map.remove(request_id) {
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
