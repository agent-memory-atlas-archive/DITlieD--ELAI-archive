mod bootstrap;
pub mod clock;
pub mod config;
pub mod entropy;
pub mod json;
pub mod sandbox;
mod usage;

pub use bootstrap::{BootstrapPhase, BootstrapPlan};
pub use config::{
    ConfigEntry, ConfigError, ConfigLoader, ConfigSource, McpConfigCollection,
    McpManagedProxyServerConfig, McpOAuthConfig, McpRemoteServerConfig, McpSdkServerConfig,
    McpServerConfig, McpStdioServerConfig, McpTransport, McpWebSocketServerConfig, OAuthConfig,
    ResolvedPermissionMode, RuntimeConfig, RuntimeFeatureConfig, RuntimeHookConfig,
    RuntimePermissionRuleConfig, RuntimePluginConfig, ScopedMcpServerConfig,
    CLAW_SETTINGS_SCHEMA_NAME,
};
pub use json::JsonValue;
pub use sandbox::{
    build_linux_sandbox_command, detect_container_environment, detect_container_environment_from,
    resolve_sandbox_status, resolve_sandbox_status_for_request, ContainerEnvironment,
    FilesystemIsolationMode, LinuxSandboxCommand, SandboxConfig, SandboxDetectionInputs,
    SandboxRequest, SandboxStatus,
};
pub use usage::{
    format_usd, pricing_for_model, ModelPricing, TokenUsage, UsageCostEstimate, UsageTracker,
};

// --- Entropy quarantine: all raw time/entropy access goes through these ---
pub use clock::{ClockProvider, LogicalInstant, OsClock, SeededClock};
pub use entropy::{EntropySource, OsEntropy, SeededRng};

use std::sync::{Arc, Mutex};

/// Process-wide context carrying deterministic time and entropy.
/// Created once at startup; passed via Arc clone.
#[derive(Clone)]
pub struct RuntimeContext {
    clock: Arc<dyn ClockProvider>,
    entropy: Arc<Mutex<dyn EntropySource>>,
}

impl RuntimeContext {
    pub fn new(
        clock: Arc<dyn ClockProvider>,
        entropy: Arc<Mutex<dyn EntropySource>>,
    ) -> Self {
        Self { clock, entropy }
    }

    /// Production default: OsClock + OsEntropy.
    pub fn os_default() -> Self {
        Self {
            clock: Arc::new(OsClock),
            entropy: Arc::new(Mutex::new(OsEntropy)),
        }
    }

    /// Build from environment. If ELAI_SEED is set to a valid u64, use SeededClock + SeededRng.
    /// If ELAI_SEED is set but invalid, logs a warning and falls back to os_default().
    /// If ELAI_SEED is unset, falls back to os_default().
    pub fn from_env() -> Self {
        match std::env::var("ELAI_SEED") {
            Ok(val) if !val.is_empty() => match val.parse::<u64>() {
                Ok(seed) => Self {
                    clock: Arc::new(SeededClock::new()),
                    entropy: Arc::new(Mutex::new(SeededRng::new(seed))),
                },
                Err(e) => {
                    eprintln!("WARN: ELAI_SEED={val:?} is not a valid u64 ({e}), falling back to OS providers");
                    Self::os_default()
                }
            },
            _ => Self::os_default(),
        }
    }

    pub fn clock(&self) -> &dyn ClockProvider {
        &*self.clock
    }

    pub fn entropy(&self) -> &Arc<Mutex<dyn EntropySource>> {
        &self.entropy
    }
}
