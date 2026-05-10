//! B-23 Phase 4H Group F — top-level self-modification orchestration (4H.8).
//!
//! Glue module that ties together every Group A..E surface into a single
//! pipeline-outcome processor. The orchestrator owns:
//!   * an [`MsiTracker`] for per-PipelineMode trajectory
//!   * a working memory of recent [`ConfigVariant`]s for OSC + EVO
//!   * a working memory of recent [`FullPipelineOutcome`]s for REG
//!
//! Per spec phase-4H.txt section 4H.8, ONE call to
//! [`SelfModOrchestrator::process_outcome`] runs:
//!   1. Bridge — generate Class A proposals from `stale_api_catches` and
//!      `failure_taxonomy_misses` (Group C `proposals_from_full_outcome`).
//!   2. Score normalisation against the active variant's score → MSI delta.
//!   3. OSC — [`compute_osc_verdict`] over the recent variant window.
//!   4. EVO — [`compute_evolution_gate`] over the same window. On Withhold,
//!      [`MsiTracker::force_stabilizing`] is invoked for the affected mode.
//!   5. REG — [`crate::regression::detect`] over the recent outcome window.
//!   6. Auto-revert — [`should_auto_revert`] over the latest outcomes.
//!
//! Returns a [`SelfModUpdate`] the dashboard / Tauri commands can render
//! without re-executing the algorithms.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config_archive::ConfigArchiveError;
use crate::evolution_gate::compute_evolution_gate;
use crate::msi::{MsiSnapshot, MsiTracker};
use crate::osc::compute_osc_verdict;
use crate::regression::detect as detect_regressions;
use crate::self_mod_bridge::{proposals_from_full_outcome, regression_pre_screen};
use crate::self_mod_layer1a::{DataTableProposal, PipelineMode};
use crate::self_mod_safety::{should_auto_revert, DEFAULT_AUTO_REVERT_FRACTION, DEFAULT_AUTO_REVERT_WINDOW};
use crate::self_mod_types::{
    ConfigVariant, EvolutionGateResult, FullPipelineOutcome, OscVerdict, RegressionReport,
    ScoreWeights, DEFAULT_SCORE_WEIGHTS,
};

const HISTORY_WINDOW: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelfModUpdate {
    pub pipeline_mode: String,
    pub msi: MsiSnapshot,
    pub osc: OscVerdict,
    pub evo: EvolutionGateResult,
    pub regression: RegressionReport,
    pub regression_blocks_promotion: bool,
    pub should_revert: bool,
    pub generated_proposals: Vec<DataTableProposal>,
}

#[derive(Debug, thiserror::Error)]
pub enum SelfModOrchestratorError {
    #[error("config archive: {0}")]
    Archive(#[from] ConfigArchiveError),
}

pub struct SelfModOrchestrator {
    msi: MsiTracker,
    /// Per-mode rolling window of recent ConfigVariants (oldest first).
    variants: BTreeMap<PipelineMode, Vec<ConfigVariant>>,
    /// Per-mode rolling window of recent FullPipelineOutcomes (oldest first).
    outcomes: BTreeMap<PipelineMode, Vec<FullPipelineOutcome>>,
    /// Active variant per mode (for delta_score + auto-revert baseline).
    active: BTreeMap<PipelineMode, ConfigVariant>,
    score_weights: ScoreWeights,
    token_budget: u64,
    max_interventions: usize,
    min_occurrences: u32,
    history_window: usize,
}

impl SelfModOrchestrator {
    pub fn new() -> Self {
        Self {
            msi: MsiTracker::new(),
            variants: BTreeMap::new(),
            outcomes: BTreeMap::new(),
            active: BTreeMap::new(),
            score_weights: DEFAULT_SCORE_WEIGHTS,
            token_budget: 100_000,
            max_interventions: 5,
            min_occurrences: crate::self_mod_layer1a::DEFAULT_MIN_OCCURRENCES,
            history_window: HISTORY_WINDOW,
        }
    }

    pub fn with_score_weights(mut self, weights: ScoreWeights) -> Self {
        self.score_weights = weights;
        self
    }

    pub fn with_token_budget(mut self, budget: u64) -> Self {
        self.token_budget = budget.max(1);
        self
    }

    pub fn with_max_interventions(mut self, max: usize) -> Self {
        self.max_interventions = max.max(1);
        self
    }

    pub fn with_min_occurrences(mut self, n: u32) -> Self {
        self.min_occurrences = n.max(1);
        self
    }

    pub fn with_history_window(mut self, n: usize) -> Self {
        self.history_window = n.max(1);
        self
    }

    /// Observe a new ConfigVariant for `mode`. Used by callers to seed the
    /// orchestrator's working memory before processing outcomes.
    pub fn observe_variant(&mut self, variant: ConfigVariant) {
        let mode = variant.pipeline_mode;
        if matches!(variant.status, crate::self_mod_types::VariantStatus::Active) {
            self.active.insert(mode, variant.clone());
        }
        let window = self.variants.entry(mode).or_default();
        window.push(variant);
        let cap = self.history_window;
        if window.len() > cap {
            let drop = window.len() - cap;
            window.drain(0..drop);
        }
    }

    pub fn msi_snapshot(&self, mode: PipelineMode) -> MsiSnapshot {
        self.msi.snapshot(mode)
    }

    pub fn all_msi_snapshots(&self) -> Vec<MsiSnapshot> {
        self.msi.all_snapshots()
    }

    /// Top-level processor — call once per completed pipeline.
    pub fn process_outcome(
        &mut self,
        outcome: FullPipelineOutcome,
    ) -> Result<SelfModUpdate, SelfModOrchestratorError> {
        let mode = outcome.pipeline_mode;

        // 1. Generate Class A proposals from the rich outcome.
        let proposals = proposals_from_full_outcome(&outcome, self.min_occurrences);

        // 2. MSI delta_score against active variant (0.0 baseline if none).
        let observed_score =
            outcome.normalised_score(&self.score_weights, self.token_budget, self.max_interventions);
        let baseline_score = self.active.get(&mode).map(|v| v.score).unwrap_or(0.0);
        let delta = (observed_score - baseline_score).clamp(-1.0, 1.0);
        let variant_id_hint = self
            .active
            .get(&mode)
            .map(|v| v.variant_id.clone())
            .unwrap_or_else(|| format!("synthetic-{}", outcome.template_id));
        self.msi.record(mode, variant_id_hint, delta);

        // 3. Outcome history (per-mode, capped).
        let outcomes_window = self.outcomes.entry(mode).or_default();
        outcomes_window.push(outcome);
        if outcomes_window.len() > self.history_window {
            let drop = outcomes_window.len() - self.history_window;
            outcomes_window.drain(0..drop);
        }

        // 4. OSC + EVO over the variant window.
        let variant_window: Vec<ConfigVariant> = self
            .variants
            .get(&mode)
            .cloned()
            .unwrap_or_default();
        let osc = compute_osc_verdict(&variant_window);
        let evo = compute_evolution_gate(&variant_window);
        if matches!(evo, EvolutionGateResult::Withhold { .. }) {
            self.msi.force_stabilizing(mode);
        } else if self.msi.is_forced_stabilizing(mode) {
            self.msi.clear_force_stabilizing(mode);
        }

        // 5. REG over the outcome window.
        let regression = detect_regressions(outcomes_window);
        let regression_blocks_promotion = !regression_pre_screen(&regression);

        // 6. Auto-revert decision.
        let should_revert = self
            .active
            .get(&mode)
            .map(|baseline| {
                should_auto_revert(
                    outcomes_window,
                    baseline,
                    &self.score_weights,
                    self.token_budget,
                    self.max_interventions,
                    DEFAULT_AUTO_REVERT_FRACTION,
                    DEFAULT_AUTO_REVERT_WINDOW,
                )
            })
            .unwrap_or(false);

        let msi = self.msi.snapshot(mode);
        Ok(SelfModUpdate {
            pipeline_mode: mode.as_str().to_string(),
            msi,
            osc,
            evo,
            regression,
            regression_blocks_promotion,
            should_revert,
            generated_proposals: proposals,
        })
    }
}

impl Default for SelfModOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msi::MsiVerdict;
    use crate::self_mod_types::{
        CatchLayer, StaleApiCatch, TestOutcomeReport, VariantStatus,
    };
    use std::path::PathBuf;

    fn variant(id: &str, score: f32, status: VariantStatus, blob: serde_json::Value) -> ConfigVariant {
        ConfigVariant {
            variant_id: id.into(),
            parent_id: None,
            score,
            children_count: 0,
            config_blob: serde_json::to_vec(&blob).unwrap_or_default(),
            created_at: 0,
            pipeline_outcomes: vec![],
            pipeline_mode: PipelineMode::Surgical,
            status,
        }
    }

    fn outcome(catches: Vec<StaleApiCatch>) -> FullPipelineOutcome {
        FullPipelineOutcome {
            template_id: "t".into(),
            project_id: "p".into(),
            pipeline_mode: PipelineMode::Surgical,
            phases_total: 10,
            phases_passed_first_try: 7,
            audit_findings_total: 0,
            audit_findings_block: 0,
            test_coverage_final: 0.85,
            time_to_complete_seconds: 60,
            tokens_consumed_total: 50_000,
            human_interventions: 0,
            rules_triggered: vec![],
            test_outcomes: TestOutcomeReport::default(),
            etc_trigger_events: 0,
            stale_api_catches: catches,
            context_budget_truncations: vec![],
            failure_taxonomy_misses: vec![],
        }
    }

    #[test]
    fn process_outcome_surfaces_proposals_from_stale_api_catches() {
        let mut orch = SelfModOrchestrator::new().with_min_occurrences(3);
        let catches: Vec<_> = (0..3)
            .map(|_| StaleApiCatch {
                file_path: PathBuf::from("src/lib.rs"),
                deprecated_name: "blake2".into(),
                replacement_name: "blake3".into(),
                ecosystem: "cargo".into(),
                caught_by: CatchLayer::Auditor,
            })
            .collect();
        let update = orch.process_outcome(outcome(catches)).unwrap();
        assert_eq!(update.generated_proposals.len(), 1);
        assert_eq!(update.pipeline_mode, "surgical");
    }

    #[test]
    fn evo_withhold_forces_stabilizing_in_msi() {
        let mut orch = SelfModOrchestrator::new();
        // Seed 5 byte-identical variants — EVO should Withhold.
        let blob = serde_json::json!({"k": 1, "j": "x"});
        for i in 0..5 {
            orch.observe_variant(variant(
                &format!("v{i}"),
                0.6 + (i as f32) * 0.05,
                if i == 4 { VariantStatus::Active } else { VariantStatus::Archived },
                blob.clone(),
            ));
        }
        // Process several outcomes that would push MSI toward IMPROVING.
        for _ in 0..5 {
            let _ = orch.process_outcome(outcome(vec![]));
        }
        let snap = orch.msi_snapshot(PipelineMode::Surgical);
        // EVO Withhold should pin verdict to STABILIZING regardless of trend.
        assert_eq!(snap.verdict, MsiVerdict::Stabilizing);
    }

    #[test]
    fn regression_blocks_promotion_when_ac_regressed() {
        let mut orch = SelfModOrchestrator::new();
        // First outcome — AC t1 passed.
        let mut o1 = outcome(vec![]);
        o1.test_outcomes.pass_to_pass = vec!["t1".into()];
        orch.process_outcome(o1).unwrap();
        // Second outcome — AC t1 failed.
        let mut o2 = outcome(vec![]);
        o2.test_outcomes.fail_to_fail = vec!["t1".into()];
        let update = orch.process_outcome(o2).unwrap();
        assert!(update.regression.has_regressions);
        assert!(update.regression_blocks_promotion);
    }

    #[test]
    fn auto_revert_triggers_after_three_bad_pipelines() {
        let mut orch = SelfModOrchestrator::new()
            .with_max_interventions(1)
            .with_token_budget(10_000);
        // Active variant with baseline score = 0.8.
        orch.observe_variant(variant(
            "active",
            0.8,
            VariantStatus::Active,
            serde_json::json!({"k": 1}),
        ));
        // Three terrible outcomes.
        for _ in 0..3 {
            let mut o = outcome(vec![]);
            o.test_coverage_final = 0.0;
            o.phases_passed_first_try = 0;
            o.human_interventions = 1;
            o.audit_findings_block = 5;
            orch.process_outcome(o).unwrap();
        }
        let last = orch
            .process_outcome({
                let mut o = outcome(vec![]);
                o.test_coverage_final = 0.0;
                o.phases_passed_first_try = 0;
                o.human_interventions = 1;
                o.audit_findings_block = 5;
                o
            })
            .unwrap();
        assert!(last.should_revert);
    }
}
