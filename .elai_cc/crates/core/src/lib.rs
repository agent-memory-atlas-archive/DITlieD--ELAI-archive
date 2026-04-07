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

/// Process-wide context carrying deterministic time and entropy.
/// Static dispatch: zero vtable overhead, no Mutex on entropy.
pub struct RuntimeContext<C, E> {
    clock: C,
    entropy: E,
}

impl<C: Clone, E: Clone> Clone for RuntimeContext<C, E> {
    fn clone(&self) -> Self {
        Self {
            clock: self.clock.clone(),
            entropy: self.entropy.clone(),
        }
    }
}

/// Production: OsClock + OsEntropy.
pub type ProductionContext = RuntimeContext<OsClock, OsEntropy>;
/// Deterministic tests: SeededClock + SeededRng.
pub type TestContext = RuntimeContext<SeededClock, SeededRng>;

impl<C: ClockProvider, E: EntropySource> RuntimeContext<C, E> {
    pub fn new(clock: C, entropy: E) -> Self {
        Self { clock, entropy }
    }

    pub fn clock(&self) -> &C {
        &self.clock
    }

    pub fn entropy_mut(&mut self) -> &mut E {
        &mut self.entropy
    }
}

impl ProductionContext {
    /// Production default: OsClock + OsEntropy.
    pub fn os_default() -> Self {
        Self {
            clock: OsClock,
            entropy: OsEntropy,
        }
    }

    /// Build from environment. Returns ProductionContext (os_default).
    /// For seeded/test contexts, use `TestContext::from_seed(seed)` directly.
    pub fn from_env() -> Self {
        Self::os_default()
    }
}

impl TestContext {
    /// Build a deterministic test context from a u64 seed.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            clock: SeededClock::new(),
            entropy: SeededRng::new(seed),
        }
    }
}
