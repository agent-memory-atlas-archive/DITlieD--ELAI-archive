//! B-23 Phase 4H — shared types for the Self-Modification system.
//!
//! Spec: docs/specs/guide/steps/phase-4H.txt section 4H.1 SKELETON.
//!
//! This module is the surface area for groups B..G of the phase. Algorithms
//! live in their own modules (config_archive, msi, osc, regression,
//! evolution_gate); shared types stay here so the modules can interop without
//! a circular import.
//!
//! Existing types (PipelineMode, AllowedConfigKey, DataTable, MsiStatus,
//! PipelineOutcome) are intentionally NOT redefined here — they live in
//! [`crate::self_mod_layer1a`] from batch-3C and are imported as needed.
//! The richer 17-field outcome from spec phase-4H.txt:86-103 is named
//! [`FullPipelineOutcome`] to avoid colliding with the MVP six-field type.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::self_mod_layer1a::{AllowedConfigKey, PipelineMode};

// ---------------------------------------------------------------------------
// Type aliases (newtypes deferred until a behaviour clearly requires them)
// ---------------------------------------------------------------------------

pub type TestId = String;
pub type TemplateId = String;
pub type ProjectId = String;
pub type RuleId = String;
pub type TriggerCount = u32;
pub type CatchRate = f32;
pub type Slot = String;
pub type VariantId = String;
pub type PipelineOutcomeId = String;
pub type SsbLogEntry = String;
pub type AgentId = String;
pub type PhaseId = String;
pub type GrcStepType = String;
pub type Language = String;
pub type BabyStepId = String;
pub type PipelineId = String;

// ---------------------------------------------------------------------------
// Score normalization weights (spec phase-4H.txt:131-139)
// ---------------------------------------------------------------------------

/// Normalised score component weights. Sum to 1.0 by construction. Mirrors
/// the `[self_mod.score_weights]` block in elai_cc.toml. All components are
/// already in [0,1]; the weighted sum stays in [0,1].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub first_try_rate_weight: f32,
    pub coverage_weight: f32,
    pub efficiency_weight: f32,
    pub intervention_weight: f32,
    pub findings_weight: f32,
}

/// Default weights per spec phase-4H.txt:135-139.
pub const DEFAULT_SCORE_WEIGHTS: ScoreWeights = ScoreWeights {
    first_try_rate_weight: 0.30,
    coverage_weight: 0.20,
    efficiency_weight: 0.20,
    intervention_weight: 0.15,
    findings_weight: 0.15,
};

impl ScoreWeights {
    /// Weights must sum to 1.0 within rounding tolerance.
    pub fn invariant_sum(&self) -> f32 {
        self.first_try_rate_weight
            + self.coverage_weight
            + self.efficiency_weight
            + self.intervention_weight
            + self.findings_weight
    }
}

// ---------------------------------------------------------------------------
// SWE-bench grading (spec phase-4H.txt:125-129; SWE-World 2602.03419)
// ---------------------------------------------------------------------------

/// SWE-bench 4-category test grading. `pass_to_fail` is the regression bucket
/// — any non-empty entry triggers immediate rejection at the staged-eval gate
/// (phase-4H.txt:606-608, 4H.5 TASK).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TestOutcomeReport {
    pub fail_to_pass: Vec<TestId>,
    pub pass_to_pass: Vec<TestId>,
    pub fail_to_fail: Vec<TestId>,
    pub pass_to_fail: Vec<TestId>,
}

impl TestOutcomeReport {
    /// True when at least one previously passing test is now failing.
    /// Spec gate: PASS_TO_FAIL → immediate reject (no further stages).
    pub fn has_regression(&self) -> bool {
        !self.pass_to_fail.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Anti-staleness wiring (spec phase-4H.txt:110-117)
// ---------------------------------------------------------------------------

/// Where the auditor / hook / ETC subsystem caught a deprecated API usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatchLayer {
    T1Hook,
    T2Hook,
    Auditor,
    Etc,
    /// C-21 Task 4 — manifest-level catch from the dep-audit CLI subcommand
    /// (cli::dep_audit). Distinct from the in-process Auditor / Etc layers
    /// because the catch surface is the dependency manifest, not source
    /// code; aids dashboard attribution + SSB action_type filtering.
    DepAudit,
}

impl CatchLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            CatchLayer::T1Hook => "t1_hook",
            CatchLayer::T2Hook => "t2_hook",
            CatchLayer::Auditor => "auditor",
            CatchLayer::Etc => "etc",
            CatchLayer::DepAudit => "dep_audit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaleApiCatch {
    pub file_path: PathBuf,
    pub deprecated_name: String,
    pub replacement_name: String,
    pub ecosystem: String,
    pub caught_by: CatchLayer,
}

// ---------------------------------------------------------------------------
// Reasoning architecture wiring (spec phase-4H.txt:118-123, 231-236)
// ---------------------------------------------------------------------------

/// An error Layer 1 of the reasoning architecture could not classify. After
/// `occurrence_count >= DEFAULT_MIN_OCCURRENCES` (phase-3F self-discovery
/// rule), Layer 1A auto-proposes a new FailureTaxonomy category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnclassifiedError {
    pub error_text: String,
    pub file_path: PathBuf,
    pub baby_step_id: BabyStepId,
    pub occurrence_count: usize,
}

/// Pointer into a persisted reasoning trace. Identifies WHERE in the GRC
/// chain a phase failed — not just THAT it failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTraceRef {
    pub agent_id: AgentId,
    pub phase: PhaseId,
    pub grc_step_index: usize,
    pub step_type: GrcStepType,
    pub failure_mode: String,
}

// ---------------------------------------------------------------------------
// Full PipelineOutcome (spec phase-4H.txt:86-103)
// ---------------------------------------------------------------------------

/// Spec-rich pipeline outcome consumed by Layer 2 / staged eval / log-driven
/// proposals. Distinct from the Layer 1A six-field
/// [`crate::self_mod_layer1a::PipelineOutcome`] which exists for backward
/// compatibility with batch-3C.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FullPipelineOutcome {
    pub template_id: TemplateId,
    pub project_id: ProjectId,
    pub pipeline_mode: PipelineMode,
    pub phases_total: usize,
    pub phases_passed_first_try: usize,
    pub audit_findings_total: usize,
    pub audit_findings_block: usize,
    pub test_coverage_final: f32,
    pub time_to_complete_seconds: u64,
    pub tokens_consumed_total: u64,
    pub human_interventions: usize,
    pub rules_triggered: Vec<(RuleId, TriggerCount, CatchRate)>,
    pub test_outcomes: TestOutcomeReport,
    pub etc_trigger_events: usize,
    pub stale_api_catches: Vec<StaleApiCatch>,
    pub context_budget_truncations: Vec<(Slot, usize)>,
    pub failure_taxonomy_misses: Vec<UnclassifiedError>,
}

impl FullPipelineOutcome {
    /// Compute the normalised score in [0,1] from `weights`. All component
    /// fractions are clamped to [0,1] before mixing, so the result is also
    /// guaranteed to be in [0,1] (spec audit BLOCK-1).
    pub fn normalised_score(&self, weights: &ScoreWeights, token_budget: u64, max_interventions: usize) -> f32 {
        let first_try_rate = if self.phases_total == 0 {
            0.0
        } else {
            (self.phases_passed_first_try as f32 / self.phases_total as f32).clamp(0.0, 1.0)
        };
        let coverage = self.test_coverage_final.clamp(0.0, 1.0);
        let efficiency = if token_budget == 0 {
            0.0
        } else {
            (1.0 - self.tokens_consumed_total as f32 / token_budget as f32).clamp(0.0, 1.0)
        };
        let intervention = if max_interventions == 0 {
            1.0
        } else {
            (1.0 - self.human_interventions as f32 / max_interventions as f32).clamp(0.0, 1.0)
        };
        let findings = if self.phases_total == 0 {
            1.0
        } else {
            (1.0 - self.audit_findings_block as f32 / self.phases_total as f32).clamp(0.0, 1.0)
        };
        (weights.first_try_rate_weight * first_try_rate
            + weights.coverage_weight * coverage
            + weights.efficiency_weight * efficiency
            + weights.intervention_weight * intervention
            + weights.findings_weight * findings)
            .clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// ConfigVariant + status (spec phase-4H.txt:154-163)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantStatus {
    Active,
    Archived,
    Reverted,
}

impl VariantStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VariantStatus::Active => "active",
            VariantStatus::Archived => "archived",
            VariantStatus::Reverted => "reverted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigVariant {
    pub variant_id: VariantId,
    pub parent_id: Option<VariantId>,
    pub score: f32,
    pub children_count: usize,
    pub config_blob: Vec<u8>,
    pub created_at: u64,
    pub pipeline_outcomes: Vec<PipelineOutcomeId>,
    pub pipeline_mode: PipelineMode,
    pub status: VariantStatus,
}

// ---------------------------------------------------------------------------
// Proposal types (spec phase-4H.txt:209-228)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProposedChange {
    /// Class A — append to a Layer 1A data table (validated by FPR check).
    InsertRow { table: String, row: Vec<u8> },
    /// Class B — adjust a parameter key (validated by staged eval 10/50/200).
    UpdateKey {
        key: String,
        old_value: String,
        new_value: String,
    },
}

/// Filter for cross-ecosystem / cross-mode contamination. A scoped proposal
/// only applies when the active project matches every Some(_) field.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectScope {
    pub ecosystem: Option<String>,
    pub language: Option<Language>,
    pub pipeline_mode: Option<PipelineMode>,
}

/// Spec-canonical proposal carried through Layer 2 / staged eval. Distinct
/// from [`crate::self_mod_layer1a::DataTableProposal`] (Layer 1A specific).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelfModProposal {
    pub variant_id: VariantId,
    pub parent_id: VariantId,
    pub target_config_key: AllowedConfigKey,
    pub proposed_change: ProposedChange,
    pub ssb_evidence: Vec<SsbLogEntry>,
    pub reasoning_trace_evidence: Vec<ReasoningTraceRef>,
    pub rationale: String,
    pub scope: Option<ProjectScope>,
}

// ---------------------------------------------------------------------------
// Staged evaluation (spec phase-4H.txt:238)
// ---------------------------------------------------------------------------

/// Stage progression for Class B (parameter) proposals only. Class A
/// (data-table) proposals are validated by [`DataTableValidation`] FPR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "stage", content = "tasks", rename_all = "snake_case")]
pub enum StagedEvaluation {
    Stage1(u32),
    Stage2(u32),
    Stage3(u32),
}

impl StagedEvaluation {
    pub fn task_count(&self) -> u32 {
        match self {
            StagedEvaluation::Stage1(n)
            | StagedEvaluation::Stage2(n)
            | StagedEvaluation::Stage3(n) => *n,
        }
    }
}

/// Spec defaults: 10 / 50 / 200 (phase-4H.txt:600-603).
pub const STAGE1_DEFAULT_TASKS: u32 = 10;
pub const STAGE2_DEFAULT_TASKS: u32 = 50;
pub const STAGE3_DEFAULT_TASKS: u32 = 200;
/// Stage-3 promotion gate (DGM section 2.5): >40% on 60-task subset AND top 2.
pub const STAGE3_PROMOTION_THRESHOLD: f32 = 0.40;
/// Default 60-task subset size used when ranking variants pre-Stage-3.
pub const STAGE3_RANKING_SUBSET: u32 = 60;

// ---------------------------------------------------------------------------
// Layer 1A FPR validation (spec phase-4H.txt:241-249)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataTableVerdict {
    Promote,
    Reject,
    InsufficientData,
}

impl DataTableVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataTableVerdict::Promote => "promote",
            DataTableVerdict::Reject => "reject",
            DataTableVerdict::InsufficientData => "insufficient_data",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTableValidation {
    pub false_positive_count: usize,
    pub total_evaluations: usize,
    pub false_positive_rate: f32,
    pub threshold: f32,
    pub window: usize,
    pub verdict: DataTableVerdict,
}

impl DataTableValidation {
    /// Compute the verdict from the running counts. Only PROMOTE when the
    /// rolling window has at least `window` evaluations AND the FPR is below
    /// the threshold.
    pub fn verdict_from_counts(false_positive: usize, total: usize, threshold: f32, window: usize) -> DataTableVerdict {
        if total < window {
            DataTableVerdict::InsufficientData
        } else {
            let fpr = false_positive as f32 / total as f32;
            if fpr < threshold {
                DataTableVerdict::Promote
            } else {
                DataTableVerdict::Reject
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Layer 3 stub (spec phase-4H.txt:306-309)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralChangeType {
    FsmState,
    AgentRole,
    GrcProtocol,
    EvidenceGate,
    PipelineControl,
    AllowedConfigKeyAddition,
}

impl StructuralChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StructuralChangeType::FsmState => "fsm_state",
            StructuralChangeType::AgentRole => "agent_role",
            StructuralChangeType::GrcProtocol => "grc_protocol",
            StructuralChangeType::EvidenceGate => "evidence_gate",
            StructuralChangeType::PipelineControl => "pipeline_control",
            StructuralChangeType::AllowedConfigKeyAddition => "allowed_config_key_addition",
        }
    }
}

/// Layer 3 placeholder. Required to land in Group A so SSB rows + dashboard
/// surfaces can reference the type. Actual L3 execution is HUMAN-ONLY and
/// requires `human_approved=true`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralChange {
    pub change_type: StructuralChangeType,
    pub description: String,
    pub human_approved: bool,
}

// ---------------------------------------------------------------------------
// OSC — Ontology-Similarity Convergence (spec phase-4H.txt:256-301)
// ---------------------------------------------------------------------------

/// Weights from ouroboros src/ouroboros/core/lineage.py OntologyDelta.compute.
/// Locked: 0.5 / 0.3 / 0.2 (do NOT alter without spec change).
pub const SIM_WEIGHT_KEY_OVERLAP: f32 = 0.5;
pub const SIM_WEIGHT_TYPE_MATCH: f32 = 0.3;
pub const SIM_WEIGHT_EXACT_MATCH: f32 = 0.2;

/// Threshold for stagnation / oscillation detection. ouroboros default 0.95.
pub const SIM_STAGNATION_THRESHOLD: f32 = 0.95;
/// Default stagnation window (consecutive variants).
pub const SIM_STAGNATION_WINDOW: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConfigVariantSimilarity {
    pub key_overlap: f32,
    pub type_match: f32,
    pub exact_match: f32,
    pub similarity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum OscVerdict {
    Stagnant { window: usize, similarity: f32 },
    Oscillating { period: usize },
    Evolving,
    Inconclusive { reason: String },
}

impl OscVerdict {
    pub fn label(&self) -> &'static str {
        match self {
            OscVerdict::Stagnant { .. } => "stagnant",
            OscVerdict::Oscillating { .. } => "oscillating",
            OscVerdict::Evolving => "evolving",
            OscVerdict::Inconclusive { .. } => "inconclusive",
        }
    }
}

// ---------------------------------------------------------------------------
// REG — Cross-Pipeline AC Regression (spec phase-4H.txt:271-295)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcRegression {
    pub ac_index: usize,
    pub ac_text: String,
    pub passed_in_pipeline: PipelineId,
    pub failed_in_pipeline: PipelineId,
    pub consecutive_failures: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RegressionReport {
    pub regressions: Vec<AcRegression>,
    pub has_regressions: bool,
    pub regressed_ac_indices: Vec<usize>,
}

impl RegressionReport {
    /// Constructor that recomputes the boolean flag + index list from the
    /// regressions vector so the three fields stay consistent.
    pub fn from_regressions(regressions: Vec<AcRegression>) -> Self {
        let regressed_ac_indices: Vec<usize> = regressions.iter().map(|r| r.ac_index).collect();
        let has_regressions = !regressed_ac_indices.is_empty();
        Self {
            regressions,
            has_regressions,
            regressed_ac_indices,
        }
    }
}

// ---------------------------------------------------------------------------
// EVO — Evolution Gate (spec phase-4H.txt:292-301)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum EvolutionGateResult {
    Allow {
        evolved_count: usize,
    },
    Withhold {
        reason: String,
        identical_streak: usize,
    },
}

impl EvolutionGateResult {
    pub fn label(&self) -> &'static str {
        match self {
            EvolutionGateResult::Allow { .. } => "allow",
            EvolutionGateResult::Withhold { .. } => "withhold",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_score_weights_sum_to_one() {
        let s = DEFAULT_SCORE_WEIGHTS.invariant_sum();
        assert!((s - 1.0).abs() < 1e-6, "weights must sum to 1.0, got {s}");
    }

    #[test]
    fn pass_to_fail_marks_regression() {
        let mut r = TestOutcomeReport::default();
        assert!(!r.has_regression());
        r.pass_to_fail.push("t".into());
        assert!(r.has_regression());
    }

    #[test]
    fn full_pipeline_outcome_score_in_unit_interval() {
        let outcome = FullPipelineOutcome {
            template_id: "t".into(),
            project_id: "p".into(),
            pipeline_mode: PipelineMode::Surgical,
            phases_total: 10,
            phases_passed_first_try: 7,
            audit_findings_total: 3,
            audit_findings_block: 1,
            test_coverage_final: 0.85,
            time_to_complete_seconds: 600,
            tokens_consumed_total: 50_000,
            human_interventions: 1,
            rules_triggered: vec![],
            test_outcomes: TestOutcomeReport::default(),
            etc_trigger_events: 0,
            stale_api_catches: vec![],
            context_budget_truncations: vec![],
            failure_taxonomy_misses: vec![],
        };
        let score = outcome.normalised_score(&DEFAULT_SCORE_WEIGHTS, 100_000, 5);
        assert!((0.0..=1.0).contains(&score), "score {score} not in [0,1]");
    }

    #[test]
    fn full_pipeline_outcome_perfect_caps_at_one() {
        let outcome = FullPipelineOutcome {
            template_id: "t".into(),
            project_id: "p".into(),
            pipeline_mode: PipelineMode::Greenfield,
            phases_total: 10,
            phases_passed_first_try: 10,
            audit_findings_total: 0,
            audit_findings_block: 0,
            test_coverage_final: 1.0,
            time_to_complete_seconds: 0,
            tokens_consumed_total: 0,
            human_interventions: 0,
            rules_triggered: vec![],
            test_outcomes: TestOutcomeReport::default(),
            etc_trigger_events: 0,
            stale_api_catches: vec![],
            context_budget_truncations: vec![],
            failure_taxonomy_misses: vec![],
        };
        let score = outcome.normalised_score(&DEFAULT_SCORE_WEIGHTS, 100_000, 5);
        assert!((score - 1.0).abs() < 1e-5, "perfect outcome should hit 1.0, got {score}");
    }

    #[test]
    fn full_pipeline_outcome_zero_phases_handled() {
        let outcome = FullPipelineOutcome {
            template_id: "t".into(),
            project_id: "p".into(),
            pipeline_mode: PipelineMode::Surgical,
            phases_total: 0,
            phases_passed_first_try: 0,
            audit_findings_total: 0,
            audit_findings_block: 0,
            test_coverage_final: 0.0,
            time_to_complete_seconds: 0,
            tokens_consumed_total: 0,
            human_interventions: 0,
            rules_triggered: vec![],
            test_outcomes: TestOutcomeReport::default(),
            etc_trigger_events: 0,
            stale_api_catches: vec![],
            context_budget_truncations: vec![],
            failure_taxonomy_misses: vec![],
        };
        // No divisions-by-zero, score returns a finite value in [0,1].
        let score = outcome.normalised_score(&DEFAULT_SCORE_WEIGHTS, 100_000, 5);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn fpr_verdict_promotes_below_threshold() {
        // 1 false positive in 20 evaluations -> 5% FPR; threshold 6% -> PROMOTE.
        let v = DataTableValidation::verdict_from_counts(1, 20, 0.06, 10);
        assert_eq!(v, DataTableVerdict::Promote);
    }

    #[test]
    fn fpr_verdict_rejects_at_threshold() {
        // FPR exactly equals threshold -> REJECT (strict less-than).
        let v = DataTableValidation::verdict_from_counts(1, 20, 0.05, 10);
        assert_eq!(v, DataTableVerdict::Reject);
    }

    #[test]
    fn fpr_verdict_insufficient_data_when_window_unmet() {
        let v = DataTableValidation::verdict_from_counts(0, 5, 0.05, 10);
        assert_eq!(v, DataTableVerdict::InsufficientData);
    }

    #[test]
    fn regression_report_from_regressions_is_consistent() {
        let r = RegressionReport::from_regressions(vec![AcRegression {
            ac_index: 7,
            ac_text: "ac".into(),
            passed_in_pipeline: "p1".into(),
            failed_in_pipeline: "p5".into(),
            consecutive_failures: 1,
        }]);
        assert!(r.has_regressions);
        assert_eq!(r.regressed_ac_indices, vec![7]);
        assert_eq!(r.regressions.len(), 1);
    }

    #[test]
    fn empty_regression_report_default() {
        let r = RegressionReport::default();
        assert!(!r.has_regressions);
        assert!(r.regressions.is_empty());
        assert!(r.regressed_ac_indices.is_empty());
    }

    #[test]
    fn osc_verdict_label_strings() {
        assert_eq!(OscVerdict::Stagnant { window: 3, similarity: 0.96 }.label(), "stagnant");
        assert_eq!(OscVerdict::Oscillating { period: 2 }.label(), "oscillating");
        assert_eq!(OscVerdict::Evolving.label(), "evolving");
        assert_eq!(OscVerdict::Inconclusive { reason: "x".into() }.label(), "inconclusive");
    }

    #[test]
    fn evolution_gate_labels() {
        assert_eq!(EvolutionGateResult::Allow { evolved_count: 4 }.label(), "allow");
        assert_eq!(
            EvolutionGateResult::Withhold {
                reason: "ontology never evolved".into(),
                identical_streak: 5,
            }
            .label(),
            "withhold"
        );
    }

    #[test]
    fn similarity_weights_locked() {
        // Spec weights MUST stay 0.5/0.3/0.2 (ouroboros source, MIT).
        assert!((SIM_WEIGHT_KEY_OVERLAP - 0.5).abs() < 1e-6);
        assert!((SIM_WEIGHT_TYPE_MATCH - 0.3).abs() < 1e-6);
        assert!((SIM_WEIGHT_EXACT_MATCH - 0.2).abs() < 1e-6);
        let total = SIM_WEIGHT_KEY_OVERLAP + SIM_WEIGHT_TYPE_MATCH + SIM_WEIGHT_EXACT_MATCH;
        assert!((total - 1.0).abs() < 1e-6);
    }

    #[test]
    fn staged_evaluation_task_counts() {
        assert_eq!(StagedEvaluation::Stage1(STAGE1_DEFAULT_TASKS).task_count(), 10);
        assert_eq!(StagedEvaluation::Stage2(STAGE2_DEFAULT_TASKS).task_count(), 50);
        assert_eq!(StagedEvaluation::Stage3(STAGE3_DEFAULT_TASKS).task_count(), 200);
    }
}
