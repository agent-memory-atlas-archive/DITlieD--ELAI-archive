pub mod ambiguity;
pub mod chat_reflect;
pub mod config;
pub mod engine;
pub mod layer_routing;
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

pub use chat_reflect::{
    ChatFuture, ChatInferenceClient, ChatReflectConfig, ChatReflectPipeline, ReflectError,
    ReflectEvent, ReflectEventSink, ReflectOutput, ReflectTokenSummary, TurnClass, N_CANDIDATES,
    REFINE_CAP,
};
pub use config::{agent_grc_config_for_role, AgentGrcConfigLoader, TomlConfigLoader};
pub use engine::{BasicGrcEngine, GrcEngine};
pub use types::{
    AgentGrcConfig, CsqCheck, CsqResult, CsqSeverity, FeasibilityMethod, FeasibilityOutcome,
    GrcBudget, GrcDefaults, GrcError, GrcState, GroundingLevel, GroundingResult,
    InlineFeasibilityResult, InlineFeasibilityState, PlanGroundingResult, TestRunResult,
    ThinkResult, ThinkStep, TopicState, VerificationMethod, VerificationResult,
};
pub use layer_routing::configure_layers_for_tier;
pub use traversal::{default_answers_for_category, traverse_with_answers, TreeTraverser};
pub use verification::{
    FileReader, LocalVerifier, MockFileReader, MockTestRunner, NetworkVerifier, NoopTestRunner,
    StdFileReader, TestRunner, Verifier,
};
