use std::collections::HashMap;

// ---------------------------------------------------------------------------
// GrcState: the 7 states in a Grounded Reasoning Chain
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrcState {
    Observe,
    Hypothesize,
    PlanAction,
    Act,
    Verify,
    Converge,
    Status,
}

impl GrcState {
    /// Valid next states from this state.
    ///
    /// Normal chain: Observe -> Hypothesize -> PlanAction -> Act -> Verify.
    /// Verify can loop back to Observe (new chain) or Hypothesize (refine).
    /// Converge is allowed from any state (forced convergence path).
    /// Status is allowed from any state (free echo, does not advance chain).
    pub fn allowed_next(&self) -> &[GrcState] {
        use GrcState::*;
        match self {
            Observe => &[Hypothesize, Converge, Status],
            Hypothesize => &[PlanAction, Converge, Status],
            PlanAction => &[Act, Converge, Status],
            Act => &[Verify, Converge, Status],
            Verify => &[Observe, Hypothesize, Converge, Status],
            Converge => &[Observe, Status],
            Status => &[Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status],
        }
    }

    /// Whether this step type requires grounding citations.
    /// Observe, Hypothesize, Verify: yes. PlanAction, Act: no. Status: no.
    pub fn requires_grounding(&self) -> bool {
        matches!(self, GrcState::Observe | GrcState::Hypothesize | GrcState::Verify)
    }

    /// Whether this step consumes budget.
    /// Status is free. All others consume budget.
    pub fn consumes_budget(&self) -> bool {
        !matches!(self, GrcState::Status)
    }
}

// ---------------------------------------------------------------------------
// ThinkStep: a single reasoning step submitted to the GRC engine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ThinkStep {
    pub step_type: GrcState,
    pub content: String,
    pub topic: String,
    pub grounding: Option<String>,
    pub conclusion: Option<String>,
    /// When AgentGrcConfig.require_scope_enumeration = true, the Observe step
    /// must enumerate all task components/modalities before proceeding.
    /// Orchestrator validates: every scope item appears in subsequent PlanAction.
    /// Unaddressed items -> WARN injected into agent context.
    pub scope: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// GrcBudget: per-topic call and character limits
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrcBudget {
    pub max_calls: u32,
    pub max_chars: u32,
    pub calls_used: u32,
    pub chars_used: u32,
}

impl Default for GrcBudget {
    /// Default: moderate budget (5 calls, 800 chars) per reasoning.txt section 3.
    fn default() -> Self {
        Self {
            max_calls: 5,
            max_chars: 800,
            calls_used: 0,
            chars_used: 0,
        }
    }
}

impl GrcBudget {
    pub fn new(max_calls: u32, max_chars: u32) -> Self {
        Self {
            max_calls,
            max_chars,
            calls_used: 0,
            chars_used: 0,
        }
    }

    pub fn remaining_calls(&self) -> u32 {
        self.max_calls.saturating_sub(self.calls_used)
    }

    pub fn remaining_chars(&self) -> u32 {
        self.max_chars.saturating_sub(self.chars_used)
    }

    pub fn is_exhausted(&self) -> bool {
        self.calls_used >= self.max_calls
    }

    /// When remaining_calls <= 1, the engine forces a convergence step.
    pub fn must_converge(&self) -> bool {
        self.remaining_calls() <= 1
    }
}

// ---------------------------------------------------------------------------
// GroundingLevel: strength of evidence behind a claim
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum GroundingLevel {
    /// From training, not verified. Must be resolved before acting.
    Uncertain,
    /// Code just read in this session.
    High,
    /// Tested, searched, or read from authoritative source.
    Verified,
}

// ---------------------------------------------------------------------------
// GrcError: typed errors for all rejection modes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, thiserror::Error)]
pub enum GrcError {
    #[error("ungrounded step: {step_type:?} on topic '{topic}' requires grounding")]
    UngroundedStep { step_type: GrcState, topic: String },

    #[error("budget exhausted for topic '{topic}': {calls_used}/{max_calls} calls used")]
    BudgetExhausted {
        topic: String,
        calls_used: u32,
        max_calls: u32,
    },

    #[error("invalid transition from {from:?} to {to:?} (allowed: {allowed:?})")]
    InvalidTransition {
        from: Option<GrcState>,
        to: GrcState,
        allowed: Vec<GrcState>,
    },

    #[error("malformed grounding: '{raw}' — {reason}")]
    MalformedGrounding { raw: String, reason: String },

    #[error("verification failed: {method:?} on '{claim}' — {reason}")]
    VerificationError {
        method: VerificationMethod,
        claim: String,
        reason: String,
    },

    #[error("insufficient findings for role '{role}': expected >= {expected_min}")]
    InsufficientFindings { role: String, expected_min: usize },

    #[error("config validation failed for role '{role}': {reason}")]
    ConfigValidation { role: String, reason: String },
}

// ---------------------------------------------------------------------------
// ThinkResult: returned on successful think() call
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ThinkResult {
    pub step: ThinkStep,
    pub budget_remaining: GrcBudget,
    pub grounding_level: Option<GroundingLevel>,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// AgentGrcConfig: per-agent GRC configuration (phase 2C)
// 8 fields: role, budget, required_steps, grounding_level, csq_required,
// allowed_methods, transition_overrides, require_scope_enumeration.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentGrcConfig {
    /// Agent role name (e.g., "executor", "auditor", "context_compiler").
    /// Validated against known roles at load time (String, not enum — avoids
    /// cross-crate coupling with orchestrator's role definitions).
    pub role: String,
    /// Per-agent budget (overrides global default when set).
    pub budget: GrcBudget,
    /// Required step sequence for this role's GRC chain.
    /// Must be non-empty. Must form a valid chain through default + overridden transitions.
    pub required_steps: Vec<GrcState>,
    /// Minimum accepted grounding level. Steps with weaker grounding are rejected.
    /// Ordering: Uncertain < High < Verified.
    pub grounding_level: GroundingLevel,
    /// Whether counterfactual safety questions are required before finalizing.
    /// If true, required_steps must include at least one Verify step.
    pub csq_required: bool,
    /// Policy restriction: which verification methods this role may request.
    /// Enforcement is in-process (GRC engine rejects disallowed methods).
    pub allowed_methods: Vec<VerificationMethod>,
    /// Additive transition overrides merged with GrcState::allowed_next().
    /// Cannot remove default transitions, only add new ones.
    /// Example: auditor needs {Verify: [Verify, Observe]} for double-verify.
    pub transition_overrides: HashMap<GrcState, Vec<GrcState>>,
    /// When true, Observe step's scope field must enumerate all task components.
    /// Orchestrator validates every scope item appears in subsequent PlanAction.
    /// Default: true for executor/designer, false for memory_writer.
    pub require_scope_enumeration: bool,
}

impl Default for AgentGrcConfig {
    fn default() -> Self {
        Self {
            role: String::new(),
            budget: GrcBudget::default(),
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Act,
                GrcState::Verify,
            ],
            grounding_level: GroundingLevel::Uncertain,
            csq_required: false,
            allowed_methods: vec![
                VerificationMethod::FileCheck,
                VerificationMethod::SourceCheck,
                VerificationMethod::TestCheck,
                VerificationMethod::WebCheck,
                VerificationMethod::ApiCheck,
                VerificationMethod::ApiRegistryCheck,
            ],
            transition_overrides: HashMap::new(),
            require_scope_enumeration: false,
        }
    }
}

// ---------------------------------------------------------------------------
// GrcDefaults: global GRC defaults loaded from [grc.defaults] in elai_cc.toml
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GrcDefaults {
    pub budget: GrcBudget,
    pub grounding_level: GroundingLevel,
    pub csq_required: bool,
    pub allowed_methods: Vec<VerificationMethod>,
}

impl Default for GrcDefaults {
    fn default() -> Self {
        Self {
            budget: GrcBudget::default(),
            grounding_level: GroundingLevel::Uncertain,
            csq_required: false,
            allowed_methods: vec![
                VerificationMethod::FileCheck,
                VerificationMethod::SourceCheck,
                VerificationMethod::TestCheck,
                VerificationMethod::WebCheck,
                VerificationMethod::ApiCheck,
                VerificationMethod::ApiRegistryCheck,
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// TopicState: internal per-topic tracking in BasicGrcEngine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TopicState {
    pub budget: GrcBudget,
    pub last_state: Option<GrcState>,
    pub steps: Vec<ThinkStep>,
    pub grounding_log: Vec<(String, GroundingLevel)>,
    /// Scope items declared in the first Observe step (if scope enumeration required).
    pub declared_scope: Option<Vec<String>>,
}

impl TopicState {
    pub fn new(budget: GrcBudget) -> Self {
        Self {
            budget,
            last_state: None,
            steps: Vec::new(),
            grounding_log: Vec::new(),
            declared_scope: None,
        }
    }
}

// ---------------------------------------------------------------------------
// VerificationMethod: the 6 verification strategies (phase 2B)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VerificationMethod {
    FileCheck,
    SourceCheck,
    TestCheck,
    WebCheck,
    ApiCheck,
    ApiRegistryCheck,
}

impl VerificationMethod {
    /// Methods requiring network access (cannot run in offline/sandboxed mode).
    pub fn requires_network(&self) -> bool {
        matches!(
            self,
            VerificationMethod::WebCheck
                | VerificationMethod::ApiCheck
                | VerificationMethod::ApiRegistryCheck
        )
    }

    /// Methods requiring tools/ crate file operations.
    pub fn requires_tools(&self) -> bool {
        matches!(
            self,
            VerificationMethod::FileCheck
                | VerificationMethod::SourceCheck
                | VerificationMethod::TestCheck
        )
    }
}

// ---------------------------------------------------------------------------
// VerificationResult: outcome of a single claim verification (phase 2B)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerificationResult {
    pub claim: String,
    pub passes: bool,
    pub evidence: String,
    pub method_used: VerificationMethod,
}

// ---------------------------------------------------------------------------
// GroundingResult: claim + method + level + source + timestamp (phase 2B)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroundingResult {
    pub claim: String,
    pub method: VerificationMethod,
    pub level: GroundingLevel,
    pub source: String,
    /// Unix timestamp from TimeSource (u64). Injected via entropy-quarantined
    /// trait when TimeSource is added to core. For now, caller provides directly.
    pub verified_at: u64,
}

// ---------------------------------------------------------------------------
// PlanGroundingResult: aggregate of all claims in a plan sweep (phase 2B)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanGroundingResult {
    pub claims: Vec<GroundingResult>,
    /// True iff ungrounded.is_empty().
    pub all_grounded: bool,
    /// Claims that remain ungrounded after verification attempts.
    pub ungrounded: Vec<String>,
    /// Claims that were "training:" and could NOT be auto-resolved.
    /// Subset of ungrounded.
    pub verify_needed: Vec<String>,
}

// ---------------------------------------------------------------------------
// TestRunResult: outcome of running a single test (phase 2B)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestRunResult {
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}
