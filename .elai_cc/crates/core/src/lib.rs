mod bootstrap;
pub mod bounded_string;
pub mod clock;
pub mod config;
pub mod entropy;
pub mod file_plan;
pub mod interrogation_tree;
pub mod micro_reflection;
pub mod mistake_book;
pub mod json;
pub mod paths;
pub mod project_id;
pub mod project_profile;
pub mod reasoning;
pub mod sandbox;
pub mod telescope;
mod usage;
pub mod research_types;
pub mod source_tier;
pub mod tool_dispatcher;
pub mod vram_tier;

pub use bootstrap::{BootstrapPhase, BootstrapPlan};
pub use config::{
    ConfigEntry, ConfigError, ConfigLoader, ConfigSource, McpConfigCollection,
    McpManagedProxyServerConfig, McpOAuthConfig, McpRemoteServerConfig, McpSdkServerConfig,
    McpServerConfig, McpStdioServerConfig, McpTransport, McpWebSocketServerConfig, OAuthConfig,
    ResolvedPermissionMode, RuntimeConfig, RuntimeFeatureConfig, RuntimeHookConfig,
    RuntimePermissionRuleConfig, RuntimePluginConfig, ScopedMcpServerConfig,
    ELAI_SETTINGS_SCHEMA_NAME,
};
pub use json::JsonValue;
pub use sandbox::{
    build_linux_sandbox_command, detect_container_environment, detect_container_environment_from,
    resolve_sandbox_status, resolve_sandbox_status_for_request, ContainerEnvironment,
    FilesystemIsolationMode, LinuxSandboxCommand, SandboxConfig, SandboxDetectionInputs,
    SandboxRequest, SandboxStatus,
};
// C-14 — Phase 0 debt strip: pricing-adjacent symbols moved to the providers
// crate (see `elai_providers::pricing`). Only the raw counter primitives
// remain in core.
pub use usage::{TokenUsage, UsageTracker};

pub use project_id::{
    project_id_from_cwd, project_id_from_path, PROJECT_ID_HEX_LEN, PROJECT_ID_PREFIX,
};

pub use project_profile::{
    ArchitecturePattern, BuildCommand, BuildCommandRegistry, BuildSystem, BuildSystemProfile,
    CiProfile, CiSystem, CommandScope, Convention, EntryPoint, EntryPointKind, ErrorHandlingProfile,
    ErrorPattern, FrameworkCategory, FrameworkDetection, Language, LanguageProfile, LintProfile,
    ModuleOrganization, NamingConventions, OrgPattern, ProjectProfile, ProjectProfileError,
    PropagationStyle, TestLocationPattern, TestProfile, WorkspaceMember,
    DEFAULT_COMMAND_TIMEOUT_S, MIN_DETECTION_CONFIDENCE, PROFILE_SCHEMA_VERSION,
};

pub use mistake_book::{
    contains_diff, format_constraints, insert_with_eviction, DiffHash, FailureCategory,
    MistakeEntry, DIFF_HASH_SHORT_LEN, MAX_ENTRIES_PER_STEP, MAX_INJECTION_TOKENS,
};

// C-31 — Layer 1 FilePlan primitives.
pub use file_plan::{
    FileModSpec, FilePlan, FilePlanError, FileSpec, FunctionSpec, LanguageLimitsMap,
    DEFAULT_LANGUAGE_HARD_BLOCK, MAX_FUNCTIONS_PER_FILE,
};

pub use micro_reflection::{
    parse_response as parse_reflection_response, MicroReflectionRequest, MicroReflectionResponse,
    ReflectionError, CHARS_PER_TOKEN, CODE_BUDGET_TOKENS, CONTEXT_BUDGET_TOKENS,
    ERROR_BUDGET_TOKENS, GOAL_BUDGET_TOKENS, MAX_ACTION_WORDS, MAX_OUTPUT_TOKENS,
    MAX_ROOT_CAUSE_WORDS, REFLECTION_SCHEMA_VERSION, TYPES_BUDGET_TOKENS,
};

pub use interrogation_tree::{
    Choice, ChoiceId, FixTemplate, InterrogationTree, QuestionNode, TraversalOutcome,
    TraversalState, MAX_CHOICES, MAX_TRAVERSAL_TOKENS, MAX_TREE_DEPTH, MIN_CHOICES,
    TOKENS_PER_QUESTION, TREE_SCHEMA_VERSION,
};

pub use source_tier::{
    classify_source, load_source_tier_config, parse_source_tier_config,
    ClassificationMethod, SourceClassification, SourceTier, SourceTierConfig, SourceTierError,
};

pub use research_types::{
    generate_fact_id, AnswerConfidence, ExtractedFact, ResearchAnswer, ResearchPipelineError,
    ResearchQuery, VerificationLevel, VerifiedClaim,
    DEFAULT_MAX_SOURCES, FACT_ID_HEX_LEN, MAX_RESYNTHESIS_RETRIES, MAX_SEARCH_QUERIES,
    MAX_UNCITED_SENTENCES, MIN_SEARCH_QUERIES, SUPPORTED_MIN_SOURCES, VERIFIED_MIN_SOURCES,
};

pub use vram_tier::{
    CloudImportance, ReasoningLayer, SwapStrategy, VramTier, VramTierConfig,
    ALL_LAYERS, MAX_MCTS_BRANCHES_8GB, MAX_MCTS_BRANCHES_12GB, MAX_MCTS_BRANCHES_16GB_PLUS,
};

// Batch 2B — ToolDispatcher trait and value types.
pub use tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditMatchTier, EditOutcome, EditRequest, FailureInput,
    FileView, GitBranchPolicy, GitCommitRequest, IncrementalOutcome, IncrementalRequest,
    ToolDispatchError, ToolDispatcher, ToolFailureCategory, ValidateRequest, ValidationTier,
    ValidationTierResult,
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
