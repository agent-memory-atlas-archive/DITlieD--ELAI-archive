// MCP Lifecycle - error recovery, heartbeat monitoring, graceful degradation
// Based on spec from phase-3J.txt step 9 and mcp_lifecycle_hardened.rs reference

use crate::mcp_bridge::{McpBridge, McpConnection, McpError};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use elai_core::{ClockProvider as _, LogicalInstant, OsClock};

/// Connection retry policy with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Calculate backoff duration for a given attempt number
    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        // NARROWING-OK: attempt is bounded by max_attempts (default 3); powi requires i32 with no TryFrom<u32>.
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let backoff_secs = self.initial_backoff.as_secs_f64() * multiplier;
        Duration::from_secs_f64(backoff_secs.min(60.0)) // Cap at 60s
    }

    /// Get the sequence of backoff durations for all attempts
    pub fn backoff_sequence(&self) -> Vec<Duration> {
        (0..self.max_attempts)
            .map(|i| self.backoff_for_attempt(i))
            .collect()
    }
}

/// Heartbeat monitor for server crash detection
#[derive(Debug, Clone)]
pub struct HeartbeatMonitor {
    /// Timeout after which server is considered crashed
    pub timeout: Duration,
    /// Last heartbeat timestamp
    last_heartbeat: Arc<RwLock<Option<LogicalInstant>>>,
}

impl HeartbeatMonitor {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            last_heartbeat: Arc::new(RwLock::new(None)),
        }
    }

    /// Update the last heartbeat timestamp
    pub async fn record_heartbeat(&self) {
        let mut last = self.last_heartbeat.write().await;
        *last = Some(OsClock.now());
    }

    /// Check if the server is alive (heartbeat within timeout)
    pub async fn is_alive(&self) -> bool {
        let last = self.last_heartbeat.read().await;
        match *last {
            Some(mark) => OsClock.elapsed_since(mark) < self.timeout,
            None => false, // No heartbeat recorded yet
        }
    }

    /// Get time since last heartbeat
    pub async fn time_since_last_heartbeat(&self) -> Option<Duration> {
        let last = self.last_heartbeat.read().await;
        last.map(|mark| OsClock.elapsed_since(mark))
    }
}

/// MCP connection manager with lifecycle management
#[derive(Debug)]
pub struct McpConnectionManager<B: McpBridge> {
    /// The underlying MCP bridge
    bridge: B,
    /// Retry policy for connection attempts
    retry_policy: RetryPolicy,
    /// Heartbeat monitor
    heartbeat: HeartbeatMonitor,
    /// Current connection state
    connection: Arc<RwLock<Option<McpConnection>>>,
}

impl<B: McpBridge> McpConnectionManager<B> {
    pub fn new(bridge: B, retry_policy: RetryPolicy, heartbeat_timeout: Duration) -> Self {
        let manager = Self {
            bridge,
            retry_policy,
            heartbeat: HeartbeatMonitor::new(heartbeat_timeout),
            connection: Arc::new(RwLock::new(None)),
        };

        // Start heartbeat watchdog
        manager.start_watchdog();

        manager
    }

    /// Start heartbeat watchdog thread to detect hung connections
    fn start_watchdog(&self) {
        let heartbeat = self.heartbeat.clone();
        let connection = self.connection.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;

                // Check if heartbeat has timed out
                if !heartbeat.is_alive().await {
                    // Heartbeat timeout - disconnect
                    let mut conn = connection.write().await;
                    if conn.is_some() {
                        tracing::warn!("MCP heartbeat timeout, disconnecting");
                        *conn = None;
                    }
                }
            }
        });
    }

    /// Connect to MCP server with retry logic
    pub async fn connect_with_retry(&self, server_url: &str) -> Result<McpConnection, McpError> {
        let mut last_error = None;

        for attempt in 0..self.retry_policy.max_attempts {
            match self.bridge.connect(server_url).await {
                Ok(conn) => {
                    // Record successful connection
                    let mut connection = self.connection.write().await;
                    *connection = Some(conn.clone());
                    self.heartbeat.record_heartbeat().await;
                    return Ok(conn);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.retry_policy.max_attempts - 1 {
                        let backoff = self.retry_policy.backoff_for_attempt(attempt);
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            McpError::ConnectionFailed("Max retries exceeded".to_string())
        }))
    }

    /// Execute a tool call with error recovery
    pub async fn call_tool_with_recovery(
        &self,
        tool: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        // Check if server is alive via heartbeat
        if !self.heartbeat.is_alive().await {
            return Err(McpError::ServerUnavailable(
                "Heartbeat timeout - server may have crashed".to_string(),
            ));
        }

        // Get current connection
        let conn = {
            let connection = self.connection.read().await;
            connection
                .as_ref()
                .cloned()
                .ok_or_else(|| McpError::ConnectionFailed("Not connected".to_string()))?
        };

        // Attempt tool call
        match self.bridge.call_tool(&conn, tool, args.clone()).await {
            Ok(result) => {
                self.heartbeat.record_heartbeat().await;
                Ok(result)
            }
            Err(e) => {
                // If tool call fails, try to reconnect and retry once
                let server_url = conn.server_url.clone();
                drop(conn);

                // Attempt reconnection
                if let Ok(new_conn) = self.connect_with_retry(&server_url).await {
                    // Retry the tool call with new connection
                    self.bridge.call_tool(&new_conn, tool, args).await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Graceful degradation: return error but don't crash if server unavailable
    pub async fn call_tool_graceful(
        &self,
        tool: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        match self.call_tool_with_recovery(tool, args).await {
            Ok(result) => Ok(result),
            Err(e) => {
                // Log the error but return a structured error response
                // This allows the orchestrator to continue without crashing
                Err(McpError::ServerUnavailable(format!(
                    "Tool '{}' unavailable: {}",
                    tool, e
                )))
            }
        }
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> Result<(), McpError> {
        let conn = {
            let mut connection = self.connection.write().await;
            connection.take()
        };

        if let Some(c) = conn {
            self.bridge.disconnect(c).await?;
        }

        Ok(())
    }

    /// Check if currently connected
    pub async fn is_connected(&self) -> bool {
        let connection = self.connection.read().await;
        connection.is_some()
    }

    /// Get heartbeat status
    pub fn heartbeat(&self) -> &HeartbeatMonitor {
        &self.heartbeat
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_bridge::DefaultMcpBridge;

    #[test]
    fn test_retry_policy_backoff_sequence() {
        let policy = RetryPolicy::default();
        let sequence = policy.backoff_sequence();
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence[0], Duration::from_secs(1));
        assert_eq!(sequence[1], Duration::from_secs(2));
        assert_eq!(sequence[2], Duration::from_secs(4));
    }

    #[test]
    fn test_retry_policy_backoff_capped() {
        let policy = RetryPolicy {
            max_attempts: 10,
            initial_backoff: Duration::from_secs(10),
            backoff_multiplier: 10.0,
        };
        let backoff = policy.backoff_for_attempt(5);
        assert!(backoff <= Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_heartbeat_monitor_alive() {
        let monitor = HeartbeatMonitor::new(Duration::from_secs(30));
        assert!(!monitor.is_alive().await); // No heartbeat yet

        monitor.record_heartbeat().await;
        assert!(monitor.is_alive().await); // Just recorded
    }

    #[tokio::test]
    async fn test_heartbeat_monitor_timeout() {
        let monitor = HeartbeatMonitor::new(Duration::from_millis(100));
        monitor.record_heartbeat().await;
        assert!(monitor.is_alive().await);

        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!monitor.is_alive().await); // Timed out
    }

    #[tokio::test]
    async fn test_connection_manager_initial_state() {
        let bridge = DefaultMcpBridge::default();
        let policy = RetryPolicy::default();
        let manager = McpConnectionManager::new(bridge, policy, Duration::from_secs(30));

        assert!(!manager.is_connected().await);
        assert!(!manager.heartbeat().is_alive().await);
    }

    // Adversarial tests for timeout handling
    #[tokio::test]
    async fn test_heartbeat_timeout_detection() {
        let monitor = HeartbeatMonitor::new(Duration::from_millis(50));
        
        monitor.record_heartbeat().await;
        assert!(monitor.is_alive().await);
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!monitor.is_alive().await);
    }

    #[tokio::test]
    async fn test_connection_manager_timeout_handling() {
        let bridge = DefaultMcpBridge::default();
        let policy = RetryPolicy::default();
        let manager = McpConnectionManager::new(
            bridge,
            policy,
            Duration::from_millis(50), // Short timeout for testing
        );

        // Test that tool call fails when heartbeat times out
        let result = manager.call_tool_with_recovery("test_tool", serde_json::json!({"arg": "value"})).await;
        
        // Should fail with ServerUnavailable due to heartbeat timeout
        assert!(result.is_err());
        match result {
            Err(McpError::ServerUnavailable(msg)) => {
                assert!(msg.contains("Heartbeat timeout"));
            }
            _ => panic!("Expected ServerUnavailable error"),
        }
    }

    #[tokio::test]
    async fn test_graceful_degradation_on_server_failure() {
        let bridge = DefaultMcpBridge::default();
        let policy = RetryPolicy::default();
        let manager = McpConnectionManager::new(
            bridge,
            policy,
            Duration::from_secs(30),
        );

        // Graceful degradation should return error but not panic
        let result = manager.call_tool_graceful("unavailable_tool", serde_json::json!({})).await;
        
        // Should return error, not panic
        assert!(result.is_err());
        // The error should indicate the tool is unavailable
        match result {
            Err(McpError::ServerUnavailable(msg)) => {
                assert!(msg.contains("unavailable_tool"));
            }
            _ => {} // Other error types are acceptable for graceful degradation
        }
    }
}
