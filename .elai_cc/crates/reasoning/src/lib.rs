pub mod ambiguity;
pub mod chat_memory_reflect;
pub mod chat_reflect;
pub mod config;
pub mod engine;
pub mod layer_routing;
// phase-3L Step 4 / M10: SYSTEM-prompt assembler that injects PI
// CONVENTIONS blocks produced by elai_knowledge::PiT1Hook.
pub mod prompt_builder;
pub mod traversal;
pub mod types;
pub mod verification;

pub use ambiguity::{
    clamp01, round_to_4dp, AmbiguityError, AmbiguityScore, AmbiguityScorer, ComponentScore,
    Milestone, ScoreBreakdown, AMBIGUITY_THRESHOLD, AUTO_COMPLETE_STREAK_REQUIRED,
    BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT, BROWNFIELD_CONTEXT_CLARITY_FLOOR,
    BROWNFIELD_CONTEXT_CLARITY_WEIGHT, BROWNFIELD_GOAL_CLARITY_WEIGHT,
    BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT, CONSTRAINT_CLARITY_FLOOR,
    CONSTRAINT_CLARITY_WEIGHT, GOAL_CLARITY_FLOOR, GOAL_CLARITY_WEIGHT, SCORING_TEMPERATURE,
    SUCCESS_CRITERIA_CLARITY_FLOOR, SUCCESS_CRITERIA_CLARITY_WEIGHT,
};

pub use chat_memory_reflect::{
    reflect as chat_memory_reflect_run, should_reflect as should_chat_memory_reflect,
    summarize_window as chat_memory_summarize_window,
    ReflectConfig as ChatMemoryReflectConfig, ReflectError as ChatMemoryReflectError,
    ReflectResult as ChatMemoryReflectResult, DEFAULT_REFLECT_EVERY_N_TURNS,
};
pub use chat_reflect::{
    ChatFuture, ChatInferenceClient, ChatReflectConfig, ChatReflectPipeline, ReflectError,
    ReflectEvent, ReflectEventSink, ReflectOutput, ReflectTokenSummary, TurnClass, N_CANDIDATES,
    REFINE_CAP,
};
pub use config::{agent_grc_config_for_role, AgentGrcConfigLoader, TomlConfigLoader};
pub use engine::{
    extract_intrinsic_signals, BasicGrcEngine, ExtractedIntrinsicSignals, GrcEngine, InferenceOutput,
};
pub use types::{
    AgentGrcConfig, CsqCheck, CsqResult, CsqSeverity, FeasibilityMethod, FeasibilityOutcome,
    GrcBudget, GrcDefaults, GrcError, GrcState, GroundingLevel, GroundingResult,
    InlineFeasibilityResult, InlineFeasibilityState, PlanGroundingResult, TestRunResult,
    ThinkResult, ThinkStep, TopicState, VerificationMethod, VerificationResult,
};
pub use layer_routing::configure_layers_for_tier;
pub use prompt_builder::{
    build_with_conventions, CONVENTIONS_BLOCK_HEADER, CONVENTIONS_PROMPT_BUDGET_TOKENS,
};
pub use traversal::{default_answers_for_category, traverse_with_answers, TreeTraverser};
pub use verification::{
    FileReader, LocalVerifier, MockFileReader, MockTestRunner, NetworkVerifier, NoopTestRunner,
    StdFileReader, TestRunner, Verifier,
};
