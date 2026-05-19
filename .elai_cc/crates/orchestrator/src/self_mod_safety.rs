//! B-23 Phase 4H Group D — 4-layer self-modification safety (4H.6).
//!
//! Implements the in-orchestrator pieces of the spec's defence-in-depth
//! design. The four enforcement layers are:
//!   * **Dispatch** — `is_locked` rejects any proposal targeting a key in
//!     [`LOCKED_KEYS`]. Called by the dashboard / staged-eval gate before
//!     any IPC round-trip.
//!   * **IPC** — orchestrator's task_runner / dispatcher refuses write
//!     requests to locked paths. Implemented in batch-3C; not extended here.
//!   * **Filesystem** — sandbox mount excludes locked paths. Lives outside
//!     the orchestrator crate (workflow/sandbox setup); covered by
//!     integration tests, not contract tests.
//!   * **Git** — pre-commit hook rejects changes to locked files. Lives in
//!     `.git/hooks/pre-commit` and the workflow harness; not in this file.
//!
//! Plus:
//!   * Layer 3 [`StructuralChange`] gate — `apply_structural_change` only
//!     proceeds when `human_approved=true`.
//!   * Auto-revert helper — `should_auto_revert` flags >10% degradation
//!     across N consecutive pipelines.
//!
//! Spec: docs/specs/guide/steps/phase-4H.txt section 4H.6
//! (lines 655-732) + spec lines 201-204 LOCKED key list.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::self_mod_types::{ConfigVariant, FullPipelineOutcome, ScoreWeights, StructuralChange};

// ---------------------------------------------------------------------------
// Locked key set (spec phase-4H.txt:201-204)
// ---------------------------------------------------------------------------

/// Spec-canonical list of Class B parameter keys the self-mod agent MAY
/// modify via staged evaluation (4H.5). Any `AllowedConfigKey::Parameter`
/// proposal whose key string is NOT in this set requires a Layer 3
/// human-approved [`StructuralChange`] before it can be applied
/// (spec phase-4H.txt:177-193 + audit BLOCK-2: closed enumeration).
///
/// C-21 Task 7: this `&[&str]` is preserved alongside [`ParameterKey`] for
/// backward-compat consumers that build proposals from JSON / TOML where
/// the wire form is a string. The closed-enum [`ParameterKey`] is the
/// canonical form for code paths that construct proposals statically.
pub const KNOWN_PARAMETER_KEYS: &[&str] = &[
    // Inference / ETC.
    "etc_threshold",
    // Freshness model.
    "freshness_half_life",
    "freshness_doc_half_life",
    "freshness_boost_norm",
    // Baby-step LOC limit (per VRAM tier; tier identifier in proposal payload).
    "baby_step_max_loc",
    // Context budget slot weights (slot identifier in proposal payload).
    "context_slot_weight",
    // Retry / branching.
    "retry_budget",
    "ab_mcts_max_branches",
    "ab_mcts_temperatures",
    // Reasoning subsystem.
    "micro_reflection_max_tokens",
    "steering_vector_coefficient",
    // Self-mod cadence.
    "self_mod_frequency",
];

// ---------------------------------------------------------------------------
// C-21 Task 7 — ParameterKey closed enum (12 variants matching KNOWN_PARAMETER_KEYS)
// ---------------------------------------------------------------------------

/// Closed enumeration of the 12 Class B parameter keys the self-mod agent
/// MAY modify via staged evaluation. Replaces ad-hoc
/// `AllowedConfigKey::Parameter(String)` constructions with type-safe
/// variants so the compiler rejects unknown keys at construction time.
///
/// Spec: phase-4H.txt:177-193 + audit BLOCK-2 (closed enumeration). The
/// canonical wire-form string for each variant lives in [`Self::as_str`]
/// and round-trips with [`Self::from_str`].
///
/// Variants that carry a numeric discriminant (e.g. per-VRAM-tier LOC
/// limits, per-slot weights) embed the discriminant inline so a single
/// `ParameterKey::BabyStepMaxLoc { vram_tier: 8 }` is distinguishable from
/// `ParameterKey::BabyStepMaxLoc { vram_tier: 24 }` at proposal level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterKey {
    EtcThreshold,
    FreshnessHalfLife,
    FreshnessDocHalfLife,
    FreshnessBoostNorm,
    BabyStepMaxLoc { vram_tier: u8 },
    ContextSlotWeight { slot: u8 },
    RetryBudget,
    AbMctsMaxBranches,
    AbMctsTemperatures,
    MicroReflectionMaxTokens,
    SteeringVectorCoefficient,
    SelfModFrequency,
}

impl ParameterKey {
    /// Canonical wire-form string. Discriminant-carrying variants expose
    /// the base key (the discriminant lives in proposal payload, not the
    /// key string), matching the [`KNOWN_PARAMETER_KEYS`] entries.
    pub fn as_str(&self) -> &'static str {
        match self {
            ParameterKey::EtcThreshold => "etc_threshold",
            ParameterKey::FreshnessHalfLife => "freshness_half_life",
            ParameterKey::FreshnessDocHalfLife => "freshness_doc_half_life",
            ParameterKey::FreshnessBoostNorm => "freshness_boost_norm",
            ParameterKey::BabyStepMaxLoc { .. } => "baby_step_max_loc",
            ParameterKey::ContextSlotWeight { .. } => "context_slot_weight",
            ParameterKey::RetryBudget => "retry_budget",
            ParameterKey::AbMctsMaxBranches => "ab_mcts_max_branches",
            ParameterKey::AbMctsTemperatures => "ab_mcts_temperatures",
            ParameterKey::MicroReflectionMaxTokens => "micro_reflection_max_tokens",
            ParameterKey::SteeringVectorCoefficient => "steering_vector_coefficient",
            ParameterKey::SelfModFrequency => "self_mod_frequency",
        }
    }

    /// Parse the canonical wire-form string. Discriminant-carrying
    /// variants default their discriminant to `0` because the wire-form
    /// key carries no payload — callers reconstructing a typed proposal
    /// from a serialised string MUST re-apply the discriminant from the
    /// proposal payload before dispatch.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "etc_threshold" => Some(ParameterKey::EtcThreshold),
            "freshness_half_life" => Some(ParameterKey::FreshnessHalfLife),
            "freshness_doc_half_life" => Some(ParameterKey::FreshnessDocHalfLife),
            "freshness_boost_norm" => Some(ParameterKey::FreshnessBoostNorm),
            "baby_step_max_loc" => Some(ParameterKey::BabyStepMaxLoc { vram_tier: 0 }),
            "context_slot_weight" => Some(ParameterKey::ContextSlotWeight { slot: 0 }),
            "retry_budget" => Some(ParameterKey::RetryBudget),
            "ab_mcts_max_branches" => Some(ParameterKey::AbMctsMaxBranches),
            "ab_mcts_temperatures" => Some(ParameterKey::AbMctsTemperatures),
            "micro_reflection_max_tokens" => Some(ParameterKey::MicroReflectionMaxTokens),
            "steering_vector_coefficient" => Some(ParameterKey::SteeringVectorCoefficient),
            "self_mod_frequency" => Some(ParameterKey::SelfModFrequency),
            _ => None,
        }
    }
}

/// Returns true when `key` is in [`KNOWN_PARAMETER_KEYS`]. The closed
/// enumeration prevents arbitrary string keys from sneaking past the
/// staged-eval gate (audit BLOCK-2 fix).
pub fn is_known_parameter(key: &str) -> bool {
    KNOWN_PARAMETER_KEYS.iter().any(|k| *k == key)
}

/// Spec-canonical list of config keys the self-mod agent MAY NEVER modify
/// — not even via Layer 3 human approval. Any proposal naming one of these
/// must be rejected at dispatch.
pub const LOCKED_KEYS: &[&str] = &[
    // Score normalisation (audit BLOCK-1) — drift here would break MSI.
    "score_weights",
    // Gate definitions and auditor config — modifying them is a Goodhart's
    // law vector (DGM Appendix H).
    "gate_definitions",
    "auditor_config",
    // Benchmark harness + test suite — agent could fake passes.
    "benchmark_harness",
    "test_suite",
    // Schema invariants — DB layout never modifiable from inside the agent.
    "config_variants_schema",
    // Safety self-config (the lock list itself).
    "safety_locks",
    "lock_list",
    // Cloud spend rails — unbounded budget = unbounded blast radius.
    "cloud_monthly_budget",
    "cloud_opus_allowed",
    "cloud_escalation_threshold",
];

/// Default auto-revert threshold — drop >10% of baseline score for the
/// `DEFAULT_AUTO_REVERT_WINDOW` consecutive pipelines and the active
/// variant is reverted (spec phase-4H.txt:725).
pub const DEFAULT_AUTO_REVERT_FRACTION: f32 = 0.10;
/// Spec phase-4H.txt:725 "auto-revert: metrics degrade >10% over 3 pipelines".
pub const DEFAULT_AUTO_REVERT_WINDOW: usize = 3;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SafetyError {
    #[error("locked key rejected at dispatch: {0}")]
    LockedKey(String),
    #[error("Layer 3 structural change requires human_approved=true (got {change_type})")]
    NotHumanApproved { change_type: String },
    /// A `Parameter(_)` proposal named a key that is NOT in
    /// [`KNOWN_PARAMETER_KEYS`]. Adding new entries to that list is itself
    /// a Layer 3 [`StructuralChange`] — see audit BLOCK-2 fix.
    #[error("unknown parameter key (Layer 3 approval required to add): {0}")]
    UnknownParameterKey(String),
}

// ---------------------------------------------------------------------------
// Layer enforcement helpers
// ---------------------------------------------------------------------------

/// Returns true when `key` is in the spec's locked set. Used by both the
/// staged-eval entry guard and the dashboard `apply_proposal_from_user`
/// command.
pub fn is_locked(key: &str) -> bool {
    locked_set().contains(key)
}

/// Cached singleton — built once on first call. Group D audit WARN-1 fix:
/// previously rebuilt the HashSet on every dispatch call.
fn locked_set() -> &'static HashSet<&'static str> {
    use std::sync::OnceLock;
    static LOCKED: OnceLock<HashSet<&'static str>> = OnceLock::new();
    LOCKED.get_or_init(|| LOCKED_KEYS.iter().copied().collect())
}

/// Dispatch-level layer check. Returns `Err(SafetyError::LockedKey)` when
/// `key` is locked.
pub fn check_locked_key(key: &str) -> Result<(), SafetyError> {
    if is_locked(key) {
        Err(SafetyError::LockedKey(key.to_string()))
    } else {
        Ok(())
    }
}

/// Defensive helper for the spec's "self-mod agent cannot READ the lock
/// list" rule. The agent has no API for the lock list, but if someone
/// adds one in a future patch, this is the centralised refusal point.
/// Returns a generic "access denied" message — never reveals the contents.
pub fn check_lock_list_read() -> Result<(), SafetyError> {
    Err(SafetyError::LockedKey("lock_list".to_string()))
}

// ---------------------------------------------------------------------------
// Layer 3 — StructuralChange gate (spec phase-4H.txt:712-716)
// ---------------------------------------------------------------------------

/// Apply a Layer 3 [`StructuralChange`]. Always returns `Err` unless
/// `change.human_approved == true`. Returns `Ok(())` on approval; the
/// caller is responsible for executing the actual structural change
/// (FSM state addition, agent-role edit, etc.) — this gate only verifies
/// approval state.
pub fn apply_structural_change(change: &StructuralChange) -> Result<(), SafetyError> {
    if !change.human_approved {
        return Err(SafetyError::NotHumanApproved {
            change_type: change.change_type.as_str().to_string(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Auto-revert (spec phase-4H.txt:725-730)
// ---------------------------------------------------------------------------

/// Returns true when the latest `window` outcomes show consistent
/// >`fraction` degradation against `baseline_score` (the score of the
/// currently-active variant captured at promotion time).
///
/// The default fraction is [`DEFAULT_AUTO_REVERT_FRACTION`] (10%) and
/// default window is [`DEFAULT_AUTO_REVERT_WINDOW`] (3). The caller
/// supplies a slice of recent [`FullPipelineOutcome`] in chronological
/// order; only the last `window` are considered.
pub fn should_auto_revert(
    history: &[FullPipelineOutcome],
    baseline: &ConfigVariant,
    weights: &ScoreWeights,
    token_budget: u64,
    max_interventions: usize,
    fraction: f32,
    window: usize,
) -> bool {
    if window == 0 || history.len() < window {
        return false;
    }
    let cutoff = baseline.score * (1.0 - fraction);
    let tail = &history[history.len() - window..];
    tail.iter()
        .all(|o| o.normalised_score(weights, token_budget, max_interventions) < cutoff)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_mod_layer1a::PipelineMode;
    use crate::self_mod_types::{
        ConfigVariant, DEFAULT_SCORE_WEIGHTS, FullPipelineOutcome, StructuralChangeType,
        TestOutcomeReport, VariantStatus,
    };

    fn outcome(coverage: f32, first_try: usize, total: usize) -> FullPipelineOutcome {
        FullPipelineOutcome {
            template_id: "t".into(),
            project_id: "p".into(),
            pipeline_mode: PipelineMode::Surgical,
            phases_total: total,
            phases_passed_first_try: first_try,
            audit_findings_total: 0,
            audit_findings_block: 0,
            test_coverage_final: coverage,
            time_to_complete_seconds: 0,
            tokens_consumed_total: 0,
            human_interventions: 0,
            rules_triggered: vec![],
            test_outcomes: TestOutcomeReport::default(),
            etc_trigger_events: 0,
            stale_api_catches: vec![],
            context_budget_truncations: vec![],
            failure_taxonomy_misses: vec![],
        }
    }

    fn baseline(score: f32) -> ConfigVariant {
        ConfigVariant {
            variant_id: "active".into(),
            parent_id: None,
            score,
            children_count: 0,
            config_blob: vec![],
            created_at: 0,
            pipeline_outcomes: vec![],
            pipeline_mode: PipelineMode::Surgical,
            status: VariantStatus::Active,
        }
    }

    #[test]
    fn locked_key_rejected() {
        assert!(is_locked("cloud_monthly_budget"));
        assert!(is_locked("score_weights"));
        assert!(is_locked("auditor_config"));
        assert!(!is_locked("etc_threshold"));
        assert!(!is_locked("freshness_half_life"));
    }

    #[test]
    fn check_locked_key_returns_err_for_locked() {
        let err = check_locked_key("cloud_monthly_budget").unwrap_err();
        assert!(matches!(err, SafetyError::LockedKey(_)));
    }

    #[test]
    fn check_locked_key_passes_for_safe_key() {
        check_locked_key("etc_threshold").expect("non-locked passes");
    }

    #[test]
    fn lock_list_read_always_fails() {
        let err = check_lock_list_read().unwrap_err();
        assert!(matches!(err, SafetyError::LockedKey(_)));
    }

    #[test]
    fn structural_change_requires_approval() {
        let unapproved = StructuralChange {
            change_type: StructuralChangeType::AllowedConfigKeyAddition,
            description: "add new key".into(),
            human_approved: false,
        };
        let err = apply_structural_change(&unapproved).unwrap_err();
        assert!(matches!(err, SafetyError::NotHumanApproved { .. }));

        let approved = StructuralChange {
            change_type: StructuralChangeType::AllowedConfigKeyAddition,
            description: "add new key".into(),
            human_approved: true,
        };
        apply_structural_change(&approved).expect("approved passes");
    }

    #[test]
    fn auto_revert_triggers_on_three_consecutive_drops() {
        // Baseline score 0.8; cutoff at 0.72. Three pipelines all below.
        // Score is dominated by first_try_rate (weight 0.30) and intervention
        // (weight 0.15 with max=1) — push first_try to 1/10 + interventions to 1
        // to land below the cutoff comfortably.
        let history: Vec<FullPipelineOutcome> = (0..3)
            .map(|_| {
                let mut o = outcome(0.0, 1, 10); // first_try=1, coverage=0
                o.human_interventions = 1;
                o.audit_findings_block = 5;
                o
            })
            .collect();
        let revert = should_auto_revert(
            &history,
            &baseline(0.8),
            &DEFAULT_SCORE_WEIGHTS,
            10_000,
            1,
            DEFAULT_AUTO_REVERT_FRACTION,
            DEFAULT_AUTO_REVERT_WINDOW,
        );
        assert!(revert);
    }

    #[test]
    fn auto_revert_skips_when_history_short() {
        let history = vec![outcome(0.0, 1, 10)];
        let revert = should_auto_revert(
            &history,
            &baseline(0.8),
            &DEFAULT_SCORE_WEIGHTS,
            10_000,
            5,
            DEFAULT_AUTO_REVERT_FRACTION,
            DEFAULT_AUTO_REVERT_WINDOW,
        );
        assert!(!revert);
    }

    #[test]
    fn auto_revert_does_not_trigger_when_one_recovers() {
        // Two bad + one good. Window=3, so the "all" check fails on the good one.
        let mut history: Vec<FullPipelineOutcome> = (0..2)
            .map(|_| outcome(0.0, 1, 10))
            .collect();
        history.push(outcome(0.95, 10, 10)); // recovery
        let revert = should_auto_revert(
            &history,
            &baseline(0.8),
            &DEFAULT_SCORE_WEIGHTS,
            10_000,
            5,
            DEFAULT_AUTO_REVERT_FRACTION,
            DEFAULT_AUTO_REVERT_WINDOW,
        );
        assert!(!revert);
    }
}
