use rustc_hash::FxHashMap;

// ---------------------------------------------------------------------------
// GrcState: the 7 states in a Grounded Reasoning Chain
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
// AuditorBudgetReserve: J-70 E2a — auditor financial-independence budget
// reservation. The orchestrator cannot raid this pool for executor work
// (NASA-STD financial independence: the auditor's budget is HARD-reserved).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditorBudgetReserve {
    pub max_calls: u32,
    pub max_chars: u32,
    pub calls_used: u32,
    pub chars_used: u32,
}

impl Default for AuditorBudgetReserve {
    fn default() -> Self {
        Self {
            max_calls: 10,
            max_chars: 1200,
            calls_used: 0,
            chars_used: 0,
        }
    }
}

impl AuditorBudgetReserve {
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

    pub fn charge(&mut self, calls: u32, chars: u32) -> Result<(), &str> {
        if self.calls_used.saturating_add(calls) > self.max_calls
            || self.chars_used.saturating_add(chars) > self.max_chars
        {
            return Err("auditor budget reserve exceeded");
        }
        self.calls_used = self.calls_used.saturating_add(calls);
        self.chars_used = self.chars_used.saturating_add(chars);
        Ok(())
    }

    pub fn refuse_raid(&self, calls: u32, chars: u32) -> Result<(), &str> {
        if calls == 0 && chars == 0 {
            Ok(())
        } else {
            Err("auditor financial independence: reserve cannot be raided by the orchestrator")
        }
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
        // Boxed to keep the Err variant small — a Vec<GrcState> (24 bytes on
        // 64-bit) inflates every Result<GrcError> return; Box<[GrcState]> is
        // a single pointer + length (16 bytes) and allocates only when the
        // error is actually constructed.
        allowed: Box<[GrcState]>,
    },

    #[error("malformed grounding: '{raw}' — {reason}")]
    MalformedGrounding { raw: String, reason: String },

    #[error("verification failed: {method:?} on '{claim}' — {reason}")]
    VerificationError {
        // Boxed to keep the Err variant small — VerificationMethod itself
        // carries up to 3 Strings (VisualArtifactCheck/CosmosCheck), making
        // the variant ~128 bytes unboxed. Box reduces it to a single pointer.
        method: Box<VerificationMethod>,
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
    pub transition_overrides: FxHashMap<GrcState, Vec<GrcState>>,
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
                // ArtifactCheck: wildcard entry (empty strings) — the gate uses
                // discriminant comparison, not PartialEq field matching, so any
                // ArtifactCheck variant is permitted when this entry is present.
                VerificationMethod::ArtifactCheck {
                    artifact_name: String::new(),
                    query: String::new(),
                },
                // E-36: VisualArtifactCheck wildcard. Same discriminant-only
                // gating pattern as ArtifactCheck; routing is by claim
                // shape (visual_spatial claim_category) rather than per-
                // artifact match. Wire-dark unless
                // ELAI_DOCUMENT_VISION_RAG_ENABLE=1.
                VerificationMethod::VisualArtifactCheck {
                    artifact_name: String::new(),
                    query: String::new(),
                    claim_category: String::new(),
                },
            ],
            transition_overrides: FxHashMap::default(),
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
                // ArtifactCheck: wildcard entry (empty strings) — the gate uses
                // discriminant comparison, not PartialEq field matching, so any
                // ArtifactCheck variant is permitted when this entry is present.
                VerificationMethod::ArtifactCheck {
                    artifact_name: String::new(),
                    query: String::new(),
                },
                // E-36: VisualArtifactCheck wildcard entry. Same
                // discriminant-only gating. Wire-dark unless
                // ELAI_DOCUMENT_VISION_RAG_ENABLE=1.
                VerificationMethod::VisualArtifactCheck {
                    artifact_name: String::new(),
                    query: String::new(),
                    claim_category: String::new(),
                },
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

/// Note: `Copy` is intentionally NOT derived on this enum.  The `ArtifactCheck`
/// variant holds `String` fields (runtime artifact name + query), which are not
/// `Copy`.  All existing callers construct variants inline and do not rely on
/// implicit copy semantics; any site that previously relied on copy should
/// use `.clone()` instead.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VerificationMethod {
    FileCheck,
    SourceCheck,
    TestCheck,
    WebCheck,
    ApiCheck,
    ApiRegistryCheck,
    /// Verify a claim by searching the context artifact store (B-35 Phase B).
    ///
    /// `artifact_name` scopes the search to one manifest entry (or all when
    /// the verifier's `artifact_lookup` closure is provided with an empty string
    /// and the closure ignores it). `query` is the search string.
    ArtifactCheck {
        artifact_name: String,
        query: String,
    },
    /// Verify a claim against the cosmos artifact store (B-42 Phase 2).
    ///
    /// Cosmos artifacts are cross-project, user-global Markdown briefs
    /// (`~/.elai/cosmos/`) refreshed on cadence via FlashResearch. Used for
    /// design / IP-boundary / monetization / app_store / marketing_pattern /
    /// ux_convention claim categories (per B-42 plan §6).
    ///
    /// `artifact_name` is one manifest entry name (e.g. "ios_hig_summary").
    /// `query` is the search string. `claim_category` is the routing label
    /// retained for tracing / dashboard observability.
    CosmosCheck {
        artifact_name: String,
        query: String,
        claim_category: String,
    },
    /// Verify a claim against the Empire event stream (B-25 Phase 4J).
    ///
    /// Empire events are real-time factory-brain bus messages (deploy,
    /// neuron-registration, repair-proposal, etc.) the bridge persists to
    /// SSB via the 5 `empire_*` action types. The verifier resolves a
    /// claim like "Empire registered a CircularDep repair against bus.py"
    /// by querying SSB rows of type `empire_event_received` /
    /// `empire_repair_proposed` filtered by `source` + `event_type`.
    ///
    /// `source` is the originating bus (e.g. `"factory"` / `"engine"`);
    /// `event_type` is the Empire-side event label. Both are matched
    /// case-sensitively against the SSB payload columns. EMPIRE-PENDING:
    /// the actual SSB query path activates when the bridge starts writing
    /// rows (post-EMPIRE-PENDING flip).
    EmpireEventCheck {
        source: String,
        event_type: String,
    },
    /// Verify a visual-spatial claim against the vision-RAG index
    /// (E-36 — document + vision RAG path; 10th VerificationMethod).
    ///
    /// Visual-spatial claims are statements about visually-rich content
    /// that text retrieval alone cannot ground: "the auth flow has 3
    /// redirects", "X sits between Y and Z in the ERD", "the dashboard
    /// shows a histogram". The verifier composes:
    ///   * the ColModernVBERT / ColQwen2.5-v0.2 page-image retriever
    ///     (top-k page candidates by vision multi-vector similarity)
    ///   * the sidecar VLM (Qwen2.5-VL) via PerRoleRuntime, returning a
    ///     `VisualArtifactVerdict` + `ArtifactReliability` tier.
    ///
    /// HARD RULE per plan task 5 + the ELAI source-grounding rule:
    /// verdicts whose reliability is `VlmCaption` carry LOW reliability
    /// and downstream gates MUST downweight. Producers route claims via
    /// the `claim_category` field — reusing the existing
    /// `CosmosCheck::claim_category` taxonomy instead of introducing a
    /// parallel `claim_class` field per workflow/reasoning/E-36-
    /// decisions.md GAP-7.
    ///
    /// Wire-dark default: dispatch fires only when
    /// `ELAI_DOCUMENT_VISION_RAG_ENABLE=1`.
    VisualArtifactCheck {
        artifact_name: String,
        query: String,
        claim_category: String,
    },
}

// ---------------------------------------------------------------------------
// E-36 — visual-artifact verdict + reliability tier (used by
// VerificationMethod::VisualArtifactCheck dispatch).
// ---------------------------------------------------------------------------

/// Outcome of a single VisualArtifactCheck dispatch. Closed enum, 3 arms
/// matching the existing CosmosCheck verdict shape (Confirm/Refute/
/// Indeterminate).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum VisualArtifactVerdict {
    /// The visual claim is supported by at least one retrieved page.
    Confirm,
    /// The visual claim contradicts at least one retrieved page.
    Refute,
    /// Retrieval returned candidates but the VLM could not commit a
    /// verdict — caller treats this as "no evidence" rather than refute.
    Indeterminate,
}

impl VisualArtifactVerdict {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Confirm => "confirm",
            Self::Refute => "refute",
            Self::Indeterminate => "indeterminate",
        }
    }
}

/// HARD reliability tier for VisualArtifactCheck verdicts. The 3 arms
/// correspond to the three retrieval modalities the verifier can route
/// through; downstream gates MUST downweight `VlmCaption` per the
/// ELAI source-grounding rule (decompiled / generated content = LOW
/// reliability).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactReliability {
    /// LOW — the verdict was sourced from a VLM-emitted caption only.
    /// Downstream gates (auditor, scope-planner) MUST downweight.
    VlmCaption,
    /// MEDIUM — the verdict was sourced from OCR text extracted via
    /// PaddleOCR-VL / Dolphin / MinerU. Higher trust than a free-form
    /// caption but still derived from a parser pass.
    OcrParse,
    /// HIGH — the verdict was sourced from a diagram source file
    /// (Mermaid / PlantUML / D2 / Excalidraw text). Ground-truth tier.
    DiagramSource,
}

impl ArtifactReliability {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::VlmCaption => "vlm_caption",
            Self::OcrParse => "ocr_parse",
            Self::DiagramSource => "diagram_source",
        }
    }

    /// True when the reliability tier is LOW and downstream gates MUST
    /// downweight the carrying verdict.
    #[must_use]
    pub const fn is_low(&self) -> bool {
        matches!(self, Self::VlmCaption)
    }
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

    /// Methods requiring access to the artifact store (B-35 Phase B).
    pub fn requires_artifact_store(&self) -> bool {
        matches!(self, VerificationMethod::ArtifactCheck { .. })
    }

    /// Methods requiring access to the cosmos artifact store (B-42 Phase 2).
    pub fn requires_cosmos_store(&self) -> bool {
        matches!(self, VerificationMethod::CosmosCheck { .. })
    }

    /// Methods requiring access to the Empire event SSB rows (B-25 Phase 4J).
    /// `EmpireEventCheck` resolves a claim by reading SSB rows of type
    /// `empire_event_received` / `empire_repair_proposed` filtered by
    /// `source` + `event_type`. EMPIRE-PENDING activation: the verifier
    /// returns "no rows match" until the bridge starts writing them.
    pub fn requires_empire_bridge(&self) -> bool {
        matches!(self, VerificationMethod::EmpireEventCheck { .. })
    }

    /// E-36: methods requiring access to the document+vision RAG index.
    /// `VisualArtifactCheck` resolves visual-spatial claims via the
    /// ColModernVBERT / ColQwen2.5-v0.2 page-image retriever + the
    /// sidecar VLM. Wire-dark default: dispatch fires only when
    /// `ELAI_DOCUMENT_VISION_RAG_ENABLE=1`.
    pub fn requires_vision_index(&self) -> bool {
        matches!(self, VerificationMethod::VisualArtifactCheck { .. })
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
    /// Unix timestamp (u64) from entropy-quarantined TimeSource.
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

// ---------------------------------------------------------------------------
// CSQ types: Counterfactual Safety Questions (phase 2D)
// ---------------------------------------------------------------------------

/// Severity of a counterfactual safety check.
/// Critical: data_loss, security_breach, financial_error, auth_bypass — guard MANDATORY.
/// Warning: incorrect_output, performance_degradation — guard recommended, does not block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsqSeverity {
    Critical,
    Warning,
}

/// A single counterfactual safety question check.
/// "If [assumption] is wrong, [consequence] happens."
/// guard_added must be explicitly set to true by the caller after adding guard code.
#[derive(Debug, Clone)]
pub struct CsqCheck {
    pub assumption: String,
    pub consequence: String,
    pub severity: CsqSeverity,
    pub guard_added: bool,
}

/// Aggregate CSQ result for a single function.
/// passed() is computed dynamically — no stored field.
#[derive(Debug, Clone)]
pub struct CsqResult {
    pub function_id: String,
    pub checks: Vec<CsqCheck>,
}

impl CsqResult {
    /// True iff all CRITICAL checks have guard_added == true.
    /// Empty checks -> true (no sensitive ops = no CSQ needed).
    pub fn passed(&self) -> bool {
        self.checks
            .iter()
            .all(|c| c.severity != CsqSeverity::Critical || c.guard_added)
    }
}

// ---------------------------------------------------------------------------
// Inline feasibility types (phase 2D)
// ---------------------------------------------------------------------------

/// Outcome of an inline feasibility check during EXECUTING.
/// Pass: claim verified, proceed. Fail: use original plan. Blocked: emit ReplanRequest(MAJOR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeasibilityOutcome {
    Pass,
    Fail,
    Blocked,
}

/// Method used for inline feasibility verification.
/// Phase 2D: SourceMatch only. Phase 4F adds WebSearch, Grep, ApiRegistryCheck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeasibilityMethod {
    SourceMatch,
}

/// Result of a single inline feasibility check.
#[derive(Debug, Clone)]
pub struct InlineFeasibilityResult {
    pub claim: String,
    pub method: FeasibilityMethod,
    pub outcome: FeasibilityOutcome,
}

/// Tracks inline feasibility check usage within an EXECUTING phase.
/// max_checks is hardcoded to 2 (reasoning.txt s8). Not publicly constructible
/// with arbitrary max_checks — use Default or reset().
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlineFeasibilityState {
    pub checks_used: u32,
    max_checks: u32,
}

impl Default for InlineFeasibilityState {
    fn default() -> Self {
        Self {
            checks_used: 0,
            max_checks: 2,
        }
    }
}

impl InlineFeasibilityState {
    pub fn is_exhausted(&self) -> bool {
        self.checks_used >= self.max_checks
    }

    pub fn record_check(&mut self) {
        self.checks_used = self.checks_used.saturating_add(1);
    }

    pub fn reset(&mut self) {
        self.checks_used = 0;
    }

    pub fn max_checks(&self) -> u32 {
        self.max_checks
    }
}
