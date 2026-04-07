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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
// AgentGrcConfig: per-agent GRC configuration (phase 2C full impl, 2A for scope)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentGrcConfig {
    pub require_scope_enumeration: bool,
}

impl Default for AgentGrcConfig {
    fn default() -> Self {
        Self {
            require_scope_enumeration: false,
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
// Phase 2B stub types (forward references, #[allow(dead_code)] until 2B)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub claim: String,
    pub passes: bool,
    pub evidence: String,
    pub method_used: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroundingResult {
    pub claim: String,
    pub method: String,
    pub level: GroundingLevel,
    pub source: String,
}
