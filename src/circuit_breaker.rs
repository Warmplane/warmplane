// Rust guideline compliant 2026-08-17

//! Circuit breaker pattern implementation for upstream MCP server fault tolerance (`M-CANONICAL-DOCS`).

use serde::{Deserialize, Serialize};
use std::{
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;

/// Configuration options for circuit breaker fault tolerance.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ResilienceConfig {
    /// Consecutive failure threshold before tripping circuit to Open (default: 3).
    #[serde(
        default = "default_failure_threshold",
        rename = "failureThreshold",
        alias = "failure_threshold"
    )]
    pub failure_threshold: u32,
    /// Cooldown duration in milliseconds before testing recovery in HalfOpen state (default: 30,000ms).
    #[serde(
        default = "default_cooldown_ms",
        rename = "cooldownMs",
        alias = "cooldown_ms"
    )]
    pub cooldown_ms: u64,
    /// Consecutive successful probe calls required in HalfOpen to reset circuit to Closed (default: 2).
    #[serde(
        default = "default_consecutive_successes",
        rename = "consecutiveSuccesses",
        alias = "consecutive_successes"
    )]
    pub consecutive_successes: u32,
    /// Automatically restart crashed stdio child processes (default: true).
    #[serde(
        default = "default_true",
        rename = "autoRestart",
        alias = "auto_restart"
    )]
    pub auto_restart: bool,
    /// Maximum restart attempts before giving up (default: 5).
    #[serde(
        default = "default_max_restarts",
        rename = "maxRestarts",
        alias = "max_restarts"
    )]
    pub max_restarts: u32,
    /// Periodic health check interval in seconds (optional).
    #[serde(
        default,
        rename = "healthCheckIntervalSecs",
        alias = "health_check_interval_secs",
        skip_serializing_if = "Option::is_none"
    )]
    pub health_check_interval_secs: Option<u64>,
}

fn default_true() -> bool {
    true
}

fn default_max_restarts() -> u32 {
    5
}

fn default_failure_threshold() -> u32 {
    3
}

fn default_cooldown_ms() -> u64 {
    30_000
}

fn default_consecutive_successes() -> u32 {
    2
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            cooldown_ms: default_cooldown_ms(),
            consecutive_successes: default_consecutive_successes(),
            auto_restart: default_true(),
            max_restarts: default_max_restarts(),
            health_check_interval_secs: None,
        }
    }
}

/// Operational state of a circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    /// Normal operation: requests flow directly to upstream server.
    Closed,
    /// Fault state: requests fast-fail immediately without invoking upstream.
    Open,
    /// Recovery probing: single test executions allowed through to probe health.
    HalfOpen,
}

/// Circuit breaker tracking failure counts, state transitions, and cooldowns for an upstream server.
#[derive(Debug)]
pub struct CircuitBreaker {
    server_id: String,
    config: ResilienceConfig,
    state: RwLock<CircuitState>,
    consecutive_failures: AtomicU32,
    consecutive_successes: AtomicU32,
    half_open_probes: AtomicU32,
    last_state_change_ms: AtomicU64,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker for a named upstream server.
    ///
    /// # Arguments
    /// * `server_id` - Identifier of upstream server.
    /// * `config` - Resilience and threshold parameters.
    pub fn new(server_id: impl Into<String>, config: ResilienceConfig) -> Self {
        let now_ms = current_time_ms();
        Self {
            server_id: server_id.into(),
            config,
            state: RwLock::new(CircuitState::Closed),
            consecutive_failures: AtomicU32::new(0),
            consecutive_successes: AtomicU32::new(0),
            half_open_probes: AtomicU32::new(0),
            last_state_change_ms: AtomicU64::new(now_ms),
        }
    }

    /// Checks if a request is permitted to proceed or should fast-fail.
    ///
    /// # Returns
    /// * `Ok(())` if request can proceed.
    /// * `Err(CircuitOpenError)` with remaining cooldown details if circuit is open.
    pub async fn check_permission(&self) -> Result<(), CircuitOpenError> {
        let mut state_guard = self.state.write().await;
        let now = current_time_ms();

        match *state_guard {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                let opened_at = self.last_state_change_ms.load(Ordering::Relaxed);
                let elapsed = now.saturating_sub(opened_at);

                if elapsed >= self.config.cooldown_ms {
                    // Transition to HalfOpen to probe recovery
                    *state_guard = CircuitState::HalfOpen;
                    self.last_state_change_ms.store(now, Ordering::Relaxed);
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                    self.half_open_probes.store(1, Ordering::SeqCst);
                    Ok(())
                } else {
                    let remaining_ms = self.config.cooldown_ms.saturating_sub(elapsed);
                    Err(CircuitOpenError {
                        server_id: self.server_id.clone(),
                        remaining_cooldown_ms: remaining_ms,
                        consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed),
                    })
                }
            }
            CircuitState::HalfOpen => {
                let in_flight = self.half_open_probes.fetch_add(1, Ordering::SeqCst);
                if in_flight > 0 {
                    self.half_open_probes.fetch_sub(1, Ordering::SeqCst);
                    Err(CircuitOpenError {
                        server_id: self.server_id.clone(),
                        remaining_cooldown_ms: 50,
                        consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Resets the circuit breaker back to Closed state (e.g. upon supervisor reconnection).
    pub async fn reset(&self) {
        let mut state_guard = self.state.write().await;
        *state_guard = CircuitState::Closed;
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        self.half_open_probes.store(0, Ordering::Relaxed);
        self.last_state_change_ms
            .store(current_time_ms(), Ordering::Relaxed);
    }

    /// Records a successful upstream call.
    pub async fn record_success(&self) {
        let mut state_guard = self.state.write().await;
        match *state_guard {
            CircuitState::Closed => {
                self.consecutive_failures.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                self.half_open_probes.store(0, Ordering::Relaxed);
                let successes = self.consecutive_successes.fetch_add(1, Ordering::Relaxed) + 1;
                if successes >= self.config.consecutive_successes {
                    *state_guard = CircuitState::Closed;
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                    self.last_state_change_ms
                        .store(current_time_ms(), Ordering::Relaxed);
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Records an upstream execution failure (e.g. timeout or unrecoverable crash).
    pub async fn record_failure(&self) {
        let mut state_guard = self.state.write().await;
        let now = current_time_ms();

        match *state_guard {
            CircuitState::Closed => {
                let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                if failures >= self.config.failure_threshold {
                    *state_guard = CircuitState::Open;
                    self.last_state_change_ms.store(now, Ordering::Relaxed);
                }
            }
            CircuitState::HalfOpen => {
                self.half_open_probes.store(0, Ordering::Relaxed);
                // Any failure in HalfOpen immediately trips back to Open
                *state_guard = CircuitState::Open;
                self.last_state_change_ms.store(now, Ordering::Relaxed);
                self.consecutive_successes.store(0, Ordering::Relaxed);
            }
            CircuitState::Open => {}
        }
    }

    /// Gets current circuit status snapshot.
    pub async fn status(&self) -> CircuitStatusSnapshot {
        let state = *self.state.read().await;
        let opened_at = self.last_state_change_ms.load(Ordering::Relaxed);
        let now = current_time_ms();
        let remaining_cooldown_ms = if state == CircuitState::Open {
            self.config
                .cooldown_ms
                .saturating_sub(now.saturating_sub(opened_at))
        } else {
            0
        };

        CircuitStatusSnapshot {
            server_id: self.server_id.clone(),
            state,
            consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed),
            consecutive_successes: self.consecutive_successes.load(Ordering::Relaxed),
            remaining_cooldown_ms,
        }
    }
}

/// Error returned when an invocation is rejected due to an open circuit breaker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitOpenError {
    /// Server identifier with tripped circuit.
    pub server_id: String,
    /// Cooldown milliseconds remaining before trial invocations resume.
    pub remaining_cooldown_ms: u64,
    /// Number of consecutive failures that triggered the circuit opening.
    pub consecutive_failures: u32,
}

impl std::fmt::Display for CircuitOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Circuit breaker for server '{}' is OPEN ({} consecutive failures). Retry in {}ms.",
            self.server_id, self.consecutive_failures, self.remaining_cooldown_ms
        )
    }
}

impl std::error::Error for CircuitOpenError {}

/// Serializable snapshot of a circuit breaker's current operational status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitStatusSnapshot {
    /// Upstream server identifier.
    pub server_id: String,
    /// Active circuit state.
    pub state: CircuitState,
    /// Number of active consecutive failures.
    pub consecutive_failures: u32,
    /// Number of consecutive successes in half-open state.
    pub consecutive_successes: u32,
    /// Remaining cooldown in milliseconds if open.
    pub remaining_cooldown_ms: u64,
}

/// Thread-safe registry holding circuit breakers for all configured upstream servers.
#[derive(Clone, Default)]
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    /// Retrieves or instantiates a circuit breaker for an upstream server.
    pub async fn get_or_create(
        &self,
        server_id: &str,
        config: ResilienceConfig,
    ) -> Arc<CircuitBreaker> {
        let mut map = self.breakers.write().await;
        if let Some(cb) = map.get(server_id) {
            cb.clone()
        } else {
            let cb = Arc::new(CircuitBreaker::new(server_id, config));
            map.insert(server_id.to_string(), cb.clone());
            cb
        }
    }

    /// Checks if call is permitted to proceed for the given server.
    pub async fn check_permission(&self, server_id: &str) -> Result<(), CircuitOpenError> {
        let cb = {
            let map = self.breakers.read().await;
            map.get(server_id).cloned()
        };

        if let Some(cb) = cb {
            cb.check_permission().await
        } else {
            Ok(())
        }
    }

    /// Records success for server.
    pub async fn record_success(&self, server_id: &str) {
        let cb = {
            let map = self.breakers.read().await;
            map.get(server_id).cloned()
        };
        if let Some(cb) = cb {
            cb.record_success().await;
        }
    }

    /// Resets the circuit breaker state for a server to Closed.
    pub async fn reset(&self, server_id: &str) {
        let cb = {
            let map = self.breakers.read().await;
            map.get(server_id).cloned()
        };
        if let Some(cb) = cb {
            cb.reset().await;
        }
    }

    /// Removes a circuit breaker from the registry (e.g. when unmounted).
    pub async fn remove(&self, server_id: &str) {
        let mut map = self.breakers.write().await;
        map.remove(server_id);
    }

    /// Records failure for server.
    pub async fn record_failure(&self, server_id: &str) {
        let cb = {
            let map = self.breakers.read().await;
            map.get(server_id).cloned()
        };
        if let Some(cb) = cb {
            cb.record_failure().await;
        }
    }

    /// Returns a list of all circuit breaker status snapshots.
    pub async fn all_statuses(&self) -> Vec<CircuitStatusSnapshot> {
        let map = self.breakers.read().await;
        let mut statuses = Vec::new();
        for cb in map.values() {
            statuses.push(cb.status().await);
        }
        statuses
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_trip_and_cooldown() {
        let config = ResilienceConfig {
            failure_threshold: 2,
            cooldown_ms: 100,
            consecutive_successes: 1,
            ..Default::default()
        };
        let cb = CircuitBreaker::new("test_server", config);

        // Initially closed
        assert_eq!(cb.status().await.state, CircuitState::Closed);
        assert!(cb.check_permission().await.is_ok());

        // 1st failure - still closed
        cb.record_failure().await;
        assert_eq!(cb.status().await.state, CircuitState::Closed);
        assert!(cb.check_permission().await.is_ok());

        // 2nd failure - trips to Open
        cb.record_failure().await;
        assert_eq!(cb.status().await.state, CircuitState::Open);
        let err = cb.check_permission().await.unwrap_err();
        assert_eq!(err.server_id, "test_server");
        assert_eq!(err.consecutive_failures, 2);

        // Wait for cooldown
        tokio::time::sleep(Duration::from_millis(110)).await;

        // Transition to HalfOpen on next check
        assert!(cb.check_permission().await.is_ok());
        assert_eq!(cb.status().await.state, CircuitState::HalfOpen);

        // Success resets to Closed
        cb.record_success().await;
        assert_eq!(cb.status().await.state, CircuitState::Closed);
        assert_eq!(cb.status().await.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_failure_reopen() {
        let config = ResilienceConfig {
            failure_threshold: 1,
            cooldown_ms: 50,
            consecutive_successes: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new("flaky", config);

        cb.record_failure().await;
        assert_eq!(cb.status().await.state, CircuitState::Open);

        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(cb.check_permission().await.is_ok()); // transitions to HalfOpen

        // Failure during HalfOpen immediately trips back to Open
        cb.record_failure().await;
        assert_eq!(cb.status().await.state, CircuitState::Open);
    }
}
