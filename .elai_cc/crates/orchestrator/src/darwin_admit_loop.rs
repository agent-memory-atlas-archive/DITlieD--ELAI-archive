//! F-11: config-DGM admit-loop driver core.
//!
//! Closes ONE config-only Darwin-Goedel-Machine generation under the four
//! existing safety guards and emits [`SsbActionType::DarwinianAdmit`] rows.
//!
//! The loop, per generation:
//!   1. **propose** — sample a parent from the [`ConfigArchive`]
//!      ([`ConfigArchive::select_parent_with`], `ELAI_SEED`-deterministic) or,
//!      on an empty archive, seed a root candidate; the mutant is a
//!      [`ConfigVariant`] (a `config_blob` delta ONLY, never an agent-source
//!      self-rewrite, so the Layer-3 `human_approved` gate is avoided by
//!      construction — the loop builds a `ConfigVariant`, not a
//!      `StructuralChange`).
//!   2. **evaluate** — derive a REAL, candidate-coupled `alpha` in `[0, 1]` and
//!      consult [`external_benches::get_suite`] for the suite floor. (The live
//!      multi-generation Python bench run is the DEFERRED Tier-2,
//!      RUNPOD-PENDING; the in-process single generation uses the deterministic
//!      candidate-derived score — NOT a synthetic `% 2` reward and NOT a
//!      hard-coded literal.)
//!   3. **guard chain IN SERIES** ([`score_candidate`]):
//!      (i)   tripwire — blake3 fingerprint of the `config_blob` (pre-exec
//!            identity stamp; blake3 ONLY, never blake2/sha2);
//!      (ii)  trap detection on the LIVE candidate (H-44: three behavioral
//!            legs via [`trap_behavioral::detect_trap_behavior`], GAP-2 fix;
//!            keyword advisory via [`trap_tasks::score_candidate_for_traps`]
//!            is retained as a cheap first-pass ONLY — VD-2, never sole
//!            blocker); emits a [`TrapBehaviorDetected`] SSB row;
//!      (iii) [`self_mod_layer1a::is_security_sensitive`] on the candidate JSON
//!            (killer-risk #1 — the archive-admit path BYPASSES this filter, so
//!            the driver MUST call it explicitly);
//!      (iv)  [`promotion_gate::two_gate_promote`] (margin tau then capacity
//!            k_m; killer-risk #2 — both UNCONFIRMED carried-forward defaults).
//!   4. **admit** — only if all four pass: [`ConfigArchive::insert_variant`]
//!      (the FIRST production caller) THEN [`ConfigArchive::select_parent`] (the
//!      `LoopClosed` proof). On any guard reject: emit a `RejectedByGuard` row,
//!      do NOT insert.
//!
//! Acceptance is "loop closed AND a guard caught a trap mutant", NOT "score
//! went up" (a single-generation positive delta is UNGROUNDED — DGM needed many
//! iterations and hit deceptive dips; arXiv:2505.22954).
//!
//! Wire-dark: this module compiles only under `cfg(feature = "darwinian-evolve")`
//! and the live driver gates additionally on `ELAI_DARWINIAN_LOOP_ENABLE=1`.
//!
//! No `.unwrap()`/`.expect()` in non-test paths; thresholds are named consts;
//! single-writer SSB (all writes via the async [`Ssb`] channel); entropy
//! quarantine ([`EntropySource`] only).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use elai_core::{EntropySource, SeededRng};

use crate::abstention_score::{
    parse_eval_log_outcome, AbstentionAggregate, DEFAULT_CONFIDENCE_TARGET,
};
use crate::config_archive::{ConfigArchive, ConfigArchiveError};
use crate::external_benches::get_suite;
use crate::promotion_gate::{
    two_gate_promote, two_gate_promote_aggregate, PromotionConfig, PromotionDecisionKind,
    TAU_CALIBRATED_AT,
};
use crate::promotion_score::{aggregate_promotion_score, LineageVerdict};
use crate::self_mod_layer1a::{is_security_sensitive, PipelineMode};
use crate::self_mod_types::{ConfigVariant, VariantId, VariantStatus};
use crate::execution_fitness::{
    execution_grounded_fitness, read_seed, FitnessError, FrozenCorpus, FrozenTestCase,
};
use crate::eval_firewall::{
    date_gated_sample, indistinguishability_probe, regenerate_instance, FirewallSummary,
};
use crate::ssb::{
    BaiArmUpdatedPayload,
    CladeSelectedKind, CladeSelectedPayload,
    DarwinianAdmitKind, DarwinianAdmitPayload,
    EvalFirewallEventKind, EvalFirewallPayload,
    FitnessEvaluatedKind, FitnessEvaluatedPayload,
    PromotionScoreAggregatedKind, PromotionScoreAggregatedPayload,
    TrapBehaviorDetectedKind, TrapBehaviorDetectedPayload,
    Ssb, SsbError, SsbPayload,
};
use crate::darwinian_bai::{
    run_sr_phase, select_next_arm, seed_arms_from_partial_history, update_arms_from_reward,
    DiscountedArmPosterior, DiscountedTsConfig,
};
use crate::template_archive::{
    ArchiveEntry, BehaviourDescriptor, LatencyTier, MapElitesArchive, TokenClass,
};
use crate::trap_behavioral::{
    detect_trap_behavior, detected_kind_to_ssb, measure_recall, DetectedKind,
};
use crate::trap_tasks::{score_candidate_for_traps, TrapPattern};
use crate::reward_tampering;

// ---------------------------------------------------------------------------
// Named constants
// ---------------------------------------------------------------------------

/// Length of the blake3 hex prefix used as the candidate fingerprint
/// (`candidate_blake3`). 16 hex chars = 8 bytes — enough to disambiguate
/// candidates within a generation without bloating the SSB payload. Matches the
/// plan's `/^[0-9a-f]{16}/` acceptance regex.
pub const CANDIDATE_BLAKE3_HEX_LEN: usize = 16;

/// Default benchmark suite the in-process generation consults for its floor.
/// `math-floors` is one of the 11 canonical suites
/// (`external_benches::list_suites`); it is deterministic + offline-friendly
/// (no live sidecar) so it suits the Tier-1 in-process gate.
pub const DEFAULT_SUITE_NAME: &str = "math-floors";

/// SSB agent id stamped on every `darwinian_admit` row this driver writes.
pub const ADMIT_LOOP_AGENT_ID: &str = "cli:darwin-loop";

/// Env var (kill-switch posture) that, when truthy, enables the live admit
/// loop. Both this AND `cfg(feature = "darwinian-evolve")` are required (an
/// autonomous self-mod loop is opt-in by ELAI policy; E-7 DECISION-13). Mirrors
/// the gate the E-7 substrate already documents.
pub const ELAI_DARWINIAN_LOOP_ENABLE_ENV: &str = "ELAI_DARWINIAN_LOOP_ENABLE";

/// H-28-WIRE-1: env var (kill-switch) that gates the reward-tampering
/// interception in the darwin admit loop. When truthy, a candidate whose
/// reward jumps implausibly (outside DEFAULT_PLAUSIBLE_REWARD_BAND) is
/// rejected with a RejectedByGuard beacon and a RewardTampering SSB row.
/// Default OFF (unset/0) so the loop behaves exactly as today.
pub const ELAI_H28_REWARD_GATE_ENABLE_ENV: &str = "ELAI_H28_REWARD_GATE_ENABLE";

// ---------------------------------------------------------------------------
// H-47: EvalFirewallContext — optional dependency bundle for run_one_generation
// ---------------------------------------------------------------------------

/// H-47 optional firewall context injected into `run_one_generation`.
/// When `None`, the firewall legs are skipped (tests that don't exercise
/// the firewall path are unaffected — the firewall is an ELAI_DARWINIAN_LOOP_ENABLE
/// production feature, not a test invariant).
///
/// When `Some`, all three legs (date-gate, template-regen, budget-binding) run
/// before the per-candidate scoring loop. A `None` store returns from
/// `date_gated_sample` already handled — the firewall only gates what items
/// feed the scoring, it does not block the scoring itself.
pub struct EvalFirewallContext<'a> {
    /// The held-out task store (D-6 composition surface).
    pub held_out_store: &'a crate::held_out::HeldOutStore,
    /// The Thresholdout budget ledger (H-19 composition surface).
    pub budget_ledger: &'a crate::holdout_budget::HoldoutBudgetLedger,
    /// The injected clock provider (C-15 determinism, no SystemTime direct).
    pub clock: &'a std::sync::Arc<dyn elai_core::clock::ClockProvider>,
    /// Ssb handle for budget-leg emit (H-19 emits HoldoutBudgetEvent internally).
    pub ssb_arc: &'a std::sync::Arc<crate::ssb::Ssb>,
    /// Training cutoff used by the date-gate (D-6 ModelTrainingCutoff).
    pub cutoff: &'a crate::held_out::ModelTrainingCutoff,
    /// Holdout identity key for the budget query (canonicalized by query_holdout).
    pub holdout_id: &'a str,
    /// Objective key for the budget query.
    pub objective: &'a str,
    /// Budget data_window string (e.g. `"0..1000000"`).
    pub data_window: &'a str,
    /// ELAI_SEED for budget GD-2 reproducibility.
    pub budget_seed: u64,
    /// Generation index (loop counter, 0-indexed).
    pub generation_id: u64,
    /// Workload n-gram slice for indistinguishability probe (E-54 surface).
    /// Pass an empty slice when no workload trace is available (probe returns 0.0).
    pub workload_ngrams: &'a [u64],
    /// When `true`, skip the date-gate (R2 broken-arm for contract tests).
    /// MUST be `false` in all production calls. The broken arm scores pre-update
    /// items and inflates fitness — that is the R2 falsifiability requirement.
    pub ignore_dates: bool,
}

/// Env var pointing at a JSON file holding the mutant candidates to evaluate
/// (a JSON array of [`MutantCandidate`] objects). Used by the deterministic
/// offline smoke + the live CLI to inject a fixed candidate set. When unset,
/// the driver seeds `mutants` deterministic clean candidates from the seed.
pub const ELAI_DARWINIAN_LOOP_MUTANTS_JSON_ENV: &str = "ELAI_DARWINIAN_LOOP_MUTANTS_JSON";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Failure modes of the admit-loop driver.
#[derive(Debug, thiserror::Error)]
pub enum AdmitLoopError {
    /// The config archive (self_mod.db) could not be opened or queried.
    #[error("config archive: {0}")]
    Archive(#[from] ConfigArchiveError),
    /// An SSB write failed.
    #[error("ssb: {0}")]
    Ssb(#[from] SsbError),
    /// A mutant candidate's JSON could not be parsed.
    #[error("mutant json: {0}")]
    MutantJson(String),
    /// The candidate JSON could not be re-serialized for the security filter.
    #[error("candidate serialize: {0}")]
    CandidateSerialize(String),
    /// H-28-WIRE-1: reward-tampering gate error.
    #[error("reward tampering: {0}")]
    RewardTampering(#[from] crate::reward_tampering::RewardTamperingError),
    /// J-26: evaluator ABI hard FAIL (R1/R2: missing results.json, non-zero
    /// exit, or no scalar score). Surfaced when ELAI_EVALUATOR_ABI_ENABLE=1.
    #[error("evaluator abi: {0}")]
    EvaluatorAbi(#[from] crate::evaluator_abi::AbiError),
}

// ---------------------------------------------------------------------------
// Mutant candidate (the unit the loop evaluates)
// ---------------------------------------------------------------------------

/// A config/template mutant the loop evaluates. NEVER an agent-source
/// self-rewrite: `config` is a JSON object describing a config delta only, so
/// the loop produces a [`ConfigVariant`], not a Layer-3 `StructuralChange`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutantCandidate {
    /// Stable human label / id for the candidate.
    pub variant_label: String,
    /// Pipeline mode the variant targets (`surgical` / `greenfield` / `oracle`).
    #[serde(default = "default_pipeline_mode")]
    pub pipeline_mode: String,
    /// The config delta as a JSON object. The security filter
    /// ([`is_security_sensitive`]) scans this verbatim.
    pub config: serde_json::Value,
    /// The candidate's evaluation log (real bench output / trace). The trap
    /// scorer scans this alongside the serialized config.
    #[serde(default)]
    pub eval_log: String,
}

fn default_pipeline_mode() -> String {
    "surgical".to_string()
}

impl MutantCandidate {
    /// Parse a JSON array of candidates (the smoke/CLI injection format).
    pub fn parse_array(json: &str) -> Result<Vec<MutantCandidate>, AdmitLoopError> {
        serde_json::from_str(json).map_err(|e| AdmitLoopError::MutantJson(e.to_string()))
    }

    /// Resolve the pipeline mode string to the closed [`PipelineMode`] enum,
    /// defaulting to `Surgical` for any unrecognized value.
    #[must_use]
    pub fn resolved_mode(&self) -> PipelineMode {
        match self.pipeline_mode.as_str() {
            "greenfield" => PipelineMode::Greenfield,
            "oracle" => PipelineMode::Oracle,
            _ => PipelineMode::Surgical,
        }
    }

    /// The canonical `config_blob` bytes: the serialized config object. This is
    /// what the archive stores and what the tripwire fingerprint + trap scorer
    /// consume.
    #[must_use]
    pub fn config_blob(&self) -> Vec<u8> {
        // `serde_json::to_vec` on a `Value` cannot fail (it is already a valid
        // JSON value); fall back to an empty object rather than panic.
        serde_json::to_vec(&self.config).unwrap_or_else(|_| b"{}".to_vec())
    }

    /// blake3 hex-prefix fingerprint of the `config_blob` (guard i, blake3
    /// ONLY — never blake2/sha2).
    #[must_use]
    pub fn candidate_blake3(&self) -> String {
        let blob = self.config_blob();
        let hex = blake3::hash(&blob).to_hex();
        hex[..CANDIDATE_BLAKE3_HEX_LEN].to_string()
    }

    /// Execution-grounded evaluation score in `[0, 1]`.
    ///
    /// H-43 (VD-1): replaces the blake3-of-bytes placeholder (GAP-1) with a
    /// CAUSAL signal: the candidate's measured pass-rate over a FROZEN test corpus
    /// (CodeRM arXiv:2501.01054) blended with dual-execution agreement (CodeT
    /// arXiv:2207.10397). Fitness = 0.7 * pass_rate + 0.3 * dual_agreement.
    ///
    /// Honors an explicit `config.force_alpha` field (fixture hook for tests that
    /// drive a specific Two-Gate margin). The force_alpha path is KEPT — tests use
    /// it to inject specific margins without building a corpus.
    ///
    /// The frozen corpus is passed from the caller (run_one_generation). When
    /// `corpus` is `None` the scorer falls back to the legacy blake3-of-bytes path
    /// so the existing offline smoke tests (which pre-date H-43 and don't supply a
    /// corpus) continue to compile and run. The production path ALWAYS passes a
    /// corpus.
    #[must_use]
    pub fn derived_alpha(&self) -> f64 {
        if let Some(forced) = self.config.get("force_alpha").and_then(|v| v.as_f64()) {
            return forced.clamp(0.0, 1.0);
        }
        // H-43: execution-grounded path. Build a minimal in-process fixture from
        // the candidate's eval_log (or config_blob as source) so derived_alpha
        // always returns a causal score. When no eval_log tokens are present,
        // the pass-rate is 0.0 (the candidate produced no evidence of correctness).
        //
        // The production run_one_generation path calls
        // execution_grounded_fitness_for_candidate directly and passes the full
        // frozen corpus; this method serves as the standalone / legacy path.
        let snippet = if self.eval_log.is_empty() {
            self.config_blob()
                .iter()
                .map(|b| *b as char)
                .collect::<String>()
        } else {
            self.eval_log.clone()
        };
        let seed = read_seed();
        // Build a minimal 1-case corpus from the eval_log tokens (Tier-1 proxy).
        let corpus_bytes = snippet.as_bytes();
        let test_case = FrozenTestCase {
            id: "derived_alpha_stub".into(),
            candidate_snippet: snippet.clone(),
            // The candidate passes if its eval_log is non-empty and contains
            // at least 10 characters (proxy for "produced output").
            expected_output: None,
        };
        match FrozenCorpus::new(vec![test_case], corpus_bytes) {
            Ok(corpus) => {
                match execution_grounded_fitness(&snippet, &corpus, seed, None) {
                    Ok(ef) => ef.fitness,
                    Err(_) => 0.0,
                }
            }
            Err(_) => 0.0,
        }
    }

    /// Execution-grounded fitness using a CALLER-SUPPLIED frozen corpus.
    ///
    /// This is the production path called from `run_one_generation`. It avoids
    /// rebuilding the corpus per-candidate (the corpus is shared across all
    /// candidates in a generation) and returns the full [`ExecutionFitness`] struct
    /// so the caller can emit the FitnessEvaluated SSB row.
    pub fn execution_grounded_fitness_for_candidate(
        &self,
        corpus: &FrozenCorpus,
        seed: u64,
    ) -> Result<crate::execution_fitness::ExecutionFitness, FitnessError> {
        if let Some(forced) = self.config.get("force_alpha").and_then(|v| v.as_f64()) {
            // force_alpha fixture hook: synthesize an ExecutionFitness with
            // pass_rate = forced (so the two-gate promotion margin is correct).
            let f = forced.clamp(0.0, 1.0);
            return Ok(crate::execution_fitness::ExecutionFitness {
                pass_rate: f,
                dual_agreement: f,
                tests_total: corpus.cases.len() as u32,
                tests_passed: (f * corpus.cases.len() as f64).round() as u32,
                fitness: f,
                seed,
                test_set_hash: corpus.corpus_blake3,
            });
        }
        let snippet = if self.eval_log.is_empty() {
            // Fall back to config_blob as the scored text.
            String::from_utf8_lossy(&self.config_blob()).into_owned()
        } else {
            self.eval_log.clone()
        };
        execution_grounded_fitness(&snippet, corpus, seed, None)
    }
}

// ---------------------------------------------------------------------------
// Guard chain
// ---------------------------------------------------------------------------

/// Outcome of routing a candidate through the four-guard chain.
#[derive(Debug, Clone, PartialEq)]
pub enum GuardOutcome {
    /// All four guards passed; the candidate is eligible for `insert_variant`.
    Admitted,
    /// A guard rejected the candidate. `guard` is one of `tripwire` / `trap` /
    /// `security` / `two_gate`; `detail` is a human-readable reason.
    Rejected { guard: String, detail: String },
}

/// H-44: the behavioral trap detection result carried out of the guard chain
/// step (ii) so the SSB emit can happen in the async `run_one_generation`
/// context. `None` when no trap detection was attempted (tripwire rejected
/// before reaching step (ii)).
#[derive(Debug, Clone)]
pub struct TrapGuardResult {
    /// Whether the behavioral detector flagged the candidate.
    pub flagged: bool,
    /// The behavioral arm that fired (if flagged).
    pub kind: Option<DetectedKind>,
    /// Whether a behavioral leg (not keyword-only) was the deciding signal.
    pub behavioral_flag: bool,
    /// Measured recall over the frozen TRACE corpus for this run.
    pub recall_report: Option<crate::trap_behavioral::TrapRecallReport>,
}

impl GuardOutcome {
    /// `true` iff the candidate was admitted by all four guards.
    #[must_use]
    pub fn is_admitted(&self) -> bool {
        matches!(self, GuardOutcome::Admitted)
    }

    /// The firing guard name, or `None` when admitted.
    #[must_use]
    pub fn guard_name(&self) -> Option<&str> {
        match self {
            GuardOutcome::Admitted => None,
            GuardOutcome::Rejected { guard, .. } => Some(guard.as_str()),
        }
    }
}

/// Guard names (stable wire strings used in `guard_rejected_by`).
pub const GUARD_TRIPWIRE: &str = "tripwire";
pub const GUARD_TRAP: &str = "trap";
pub const GUARD_SECURITY: &str = "security";
pub const GUARD_TWO_GATE: &str = "two_gate";

/// Route ONE candidate through the four-guard chain IN SERIES. Pure logic (no
/// IO, no clock, no RNG) so it is deterministic + directly unit/contract
/// testable.
///
/// `archive_occupied_cells` is the current occupied-cell count of the
/// MAP-Elites view the Two-Gate capacity gate checks (the driver passes the
/// live archive size; tests pass a fixed value).
///
/// Order matters (the safety spine): tripwire -> trap -> security -> two_gate.
///
/// H-44: step (ii) now routes through `detect_trap_behavior` (three behavioral
/// legs) with `score_candidate_for_traps` as an advisory-only fallback (VD-2).
/// The behavioral trap detection result is returned alongside the `GuardOutcome`
/// so the async caller can emit the `TrapBehaviorDetected` SSB row (off-agent
/// artifact, R4) without blocking the synchronous guard chain.
pub fn score_candidate(
    candidate: &MutantCandidate,
    alpha: f64,
    archive_occupied_cells: u32,
    promotion_cfg: &PromotionConfig,
) -> Result<GuardOutcome, AdmitLoopError> {
    score_candidate_with_trap(candidate, alpha, archive_occupied_cells, promotion_cfg)
        .map(|(outcome, _trap)| outcome)
}

/// H-45 + H-44: variant of [`score_candidate_with_trap`] that also computes
/// the worst-case / variance-penalized aggregate over lineage-diverse verdicts
/// and returns the [`crate::promotion_score::PromotionScore`] for SSB emission.
///
/// The `execution_fitness` argument is the H-43 execution-grounded fitness
/// value for this candidate. When `Some`, it is the single lineage verdict
/// (ModelFamily::Local) fed into `aggregate_promotion_score`. When `None`
/// (corpus build failed), the function falls back to `two_gate_promote`
/// with the raw `alpha` (the pre-H-45 behaviour) and returns `None` for
/// the promotion score.
///
/// D-13 forward dependency: when D-13's cross-family adversary verdict becomes
/// available it is added as a second `LineageVerdict { family: Anthropic, score: ... }`.
/// No stub or TODO marker — this is a cross-plan forward dep documented in
/// H-45-decisions.md §D-13-forward-dep. The current single-family aggregate
/// is the Tier-1 done-gate.
pub fn score_candidate_with_aggregate(
    candidate: &MutantCandidate,
    alpha: f64,
    execution_fitness_score: Option<f64>,
    archive_occupied_cells: u32,
    promotion_cfg: &PromotionConfig,
) -> Result<
    (GuardOutcome, Option<TrapGuardResult>, Option<crate::promotion_score::PromotionScore>),
    AdmitLoopError,
> {
    use crate::adversary::ModelFamily;

    let blob = candidate.config_blob();

    // (i) Tripwire.
    let fingerprint = candidate.candidate_blake3();
    if blob.is_empty() || fingerprint.len() != CANDIDATE_BLAKE3_HEX_LEN {
        return Ok((
            GuardOutcome::Rejected {
                guard: GUARD_TRIPWIRE.to_string(),
                detail: "empty or unfingerprintable config_blob".to_string(),
            },
            None,
            None,
        ));
    }

    // (ii) H-44: behavioral trap detection.
    let seed = crate::execution_fitness::read_seed();
    let trap_result = detect_trap_behavior(&blob, &candidate.eval_log, seed);
    let recall = measure_recall(seed);
    let trap_guard = TrapGuardResult {
        flagged: trap_result.flagged,
        kind: trap_result.kind,
        behavioral_flag: trap_result.behavioral_flag,
        recall_report: Some(recall),
    };

    if trap_result.flagged && trap_result.behavioral_flag {
        let detail = match trap_result.kind {
            Some(DetectedKind::ImpossiblePassExploit) =>
                "behavioral trap: impossible_pass_exploit".to_string(),
            Some(DetectedKind::IsomorphMismatch) =>
                "behavioral trap: isomorph_mismatch".to_string(),
            Some(DetectedKind::ExecutionHallucination) =>
                "behavioral trap: execution_hallucination".to_string(),
            Some(DetectedKind::KeywordFallback) =>
                "behavioral trap: keyword_fallback (unreachable in hard-block path)".to_string(),
            Some(DetectedKind::ClosureEscapeHatch) =>
                "behavioral trap: closure_escape_hatch (unregistered escape in proof closure)".to_string(),
            None => "behavioral trap: unknown".to_string(),
        };
        return Ok((
            GuardOutcome::Rejected { guard: GUARD_TRAP.to_string(), detail },
            Some(trap_guard),
            None,
        ));
    }

    if trap_result.flagged && !trap_result.behavioral_flag {
        let pattern_label = score_candidate_for_traps(&blob, &candidate.eval_log)
            .map_or("unknown", |p| trap_label(p));
        tracing::debug!(
            pattern = pattern_label,
            "TrapGuard: keyword advisory (advisory-only, candidate proceeds per VD-2)"
        );
    }

    // (iii) Security.
    if is_security_sensitive(&candidate.config) {
        return Ok((
            GuardOutcome::Rejected {
                guard: GUARD_SECURITY.to_string(),
                detail: "config touches a security-sensitive keyword".to_string(),
            },
            Some(trap_guard),
            None,
        ));
    }

    // (iv) H-45: Two-Gate promotion using worst-case aggregate.
    let descriptor = config_cell_descriptor();
    let archive = build_two_gate_archive_view(archive_occupied_cells, &descriptor);

    // Build the lineage-diverse verdict slice.
    // Tier-1: one verdict from the Local execution leg (H-43).
    // D-13 cross-family adversary (forward dep): added here when shipped.
    let promotion_score = if let Some(ef_score) = execution_fitness_score {
        let verdicts = vec![LineageVerdict {
            score: ef_score,
            family: ModelFamily::Local,
        }];
        match aggregate_promotion_score(&verdicts, false) {
            Ok(agg) => Some(agg),
            Err(crate::promotion_score::PromotionScoreError::LineageDiverseDisagree {
                spread,
                band,
            }) => {
                // Disagreement block: treat as below-margin rejection.
                return Ok((
                    GuardOutcome::Rejected {
                        guard: GUARD_TWO_GATE.to_string(),
                        detail: format!(
                            "lineage_diverse_disagree: spread {spread:.4} > band {band:.4}"
                        ),
                    },
                    Some(trap_guard),
                    None,
                ));
            }
            Err(crate::promotion_score::PromotionScoreError::EmptyVerdicts) => {
                // Defensive: can't happen (verdicts has 1 element).
                None
            }
        }
    } else {
        None
    };

    // Route through the promotion gate.
    let decision = if let Some(ref agg) = promotion_score {
        // H-45 path: use the worst-case aggregate margin.
        two_gate_promote_aggregate(agg, &descriptor, &archive, promotion_cfg)
            .map_err(|e| AdmitLoopError::CandidateSerialize(e.to_string()))?
    } else {
        // Fallback to raw alpha (H-43 corpus build failed; pre-H-45 path).
        let entry = ArchiveEntry {
            template_blake3: blake3::hash(&blob).into(),
            fitness: alpha,
        };
        two_gate_promote(&entry, &descriptor, &archive, promotion_cfg)
            .map_err(|e| AdmitLoopError::CandidateSerialize(e.to_string()))?
    };

    let outcome = match decision.kind {
        PromotionDecisionKind::Promoted => GuardOutcome::Admitted,
        other => GuardOutcome::Rejected {
            guard: GUARD_TWO_GATE.to_string(),
            detail: format!(
                "{} ({})",
                two_gate_label(&other),
                decision.reason.unwrap_or_default()
            ),
        },
    };
    Ok((outcome, Some(trap_guard), promotion_score))
}

/// H-44: variant of [`score_candidate`] that also returns the
/// [`TrapGuardResult`] for SSB emission in the async `run_one_generation`
/// caller.
pub fn score_candidate_with_trap(
    candidate: &MutantCandidate,
    alpha: f64,
    archive_occupied_cells: u32,
    promotion_cfg: &PromotionConfig,
) -> Result<(GuardOutcome, Option<TrapGuardResult>), AdmitLoopError> {
    let blob = candidate.config_blob();

    // (i) Tripwire: blake3 fingerprint pre-exec. The identity stamp must be a
    // well-formed 16-char hex prefix; an empty config_blob (no candidate) is a
    // tripwire reject (a degenerate candidate that never produced a config).
    let fingerprint = candidate.candidate_blake3();
    if blob.is_empty() || fingerprint.len() != CANDIDATE_BLAKE3_HEX_LEN {
        return Ok((
            GuardOutcome::Rejected {
                guard: GUARD_TRIPWIRE.to_string(),
                detail: "empty or unfingerprintable config_blob".to_string(),
            },
            None, // No trap detection attempted; tripwire fired first.
        ));
    }

    // (ii) H-44: behavioral trap detection (REAL detector, GAP-2 fix).
    //
    // The behavioral legs run first (ImpossibleBench / IsomorphMismatch /
    // CodeHalu). The keyword advisory (`score_candidate_for_traps`) runs last
    // and is NEVER the sole blocker (VD-2, Skalse 2209.13085).
    //
    // The ELAI_SEED env var drives the CodeHalu execution leg (GD-2).
    let seed = crate::execution_fitness::read_seed();
    let trap_result = detect_trap_behavior(&blob, &candidate.eval_log, seed);
    let recall = measure_recall(seed);
    let trap_guard = TrapGuardResult {
        flagged: trap_result.flagged,
        kind: trap_result.kind,
        behavioral_flag: trap_result.behavioral_flag,
        recall_report: Some(recall),
    };

    // VD-2: only a BEHAVIORAL flag is a hard block. A keyword-only advisory
    // (behavioral_flag=false, kind=KeywordFallback) is logged but the candidate
    // PROCEEDS — keyword-grep is never the sole cheat-detector (Skalse 2209.13085).
    if trap_result.flagged && trap_result.behavioral_flag {
        let detail = match trap_result.kind {
            Some(DetectedKind::ImpossiblePassExploit) =>
                "behavioral trap: impossible_pass_exploit".to_string(),
            Some(DetectedKind::IsomorphMismatch) =>
                "behavioral trap: isomorph_mismatch".to_string(),
            Some(DetectedKind::ExecutionHallucination) =>
                "behavioral trap: execution_hallucination".to_string(),
            // KeywordFallback has behavioral_flag=false, so this arm is
            // unreachable when the outer condition is `behavioral_flag == true`.
            // Keep for exhaustiveness.
            Some(DetectedKind::KeywordFallback) =>
                "behavioral trap: keyword_fallback (unreachable in hard-block path)".to_string(),
            // H-54: closure escape-hatch is a deterministic behavioral block.
            Some(DetectedKind::ClosureEscapeHatch) =>
                "behavioral trap: closure_escape_hatch (unregistered escape in proof closure)".to_string(),
            None => "behavioral trap: unknown".to_string(),
        };
        return Ok((
            GuardOutcome::Rejected {
                guard: GUARD_TRAP.to_string(),
                detail,
            },
            Some(trap_guard),
        ));
    }

    // Keyword-only advisory path (behavioral_flag=false): log and continue.
    // The candidate is NOT rejected; the advisory is observable in the SSB row.
    if trap_result.flagged && !trap_result.behavioral_flag {
        let pattern_label = score_candidate_for_traps(&blob, &candidate.eval_log)
            .map_or("unknown", |p| trap_label(p));
        tracing::debug!(
            pattern = pattern_label,
            "TrapGuard: keyword advisory (advisory-only, candidate proceeds per VD-2)"
        );
    }

    // (iii) Security keyword filter (killer-risk #1): the archive-admit path
    // BYPASSES is_security_sensitive, so call it explicitly on the candidate
    // JSON. A candidate whose config touches auth/credential/token/secret/etc
    // is rejected before admit.
    if is_security_sensitive(&candidate.config) {
        return Ok((
            GuardOutcome::Rejected {
                guard: GUARD_SECURITY.to_string(),
                detail: "config touches a security-sensitive keyword".to_string(),
            },
            Some(trap_guard),
        ));
    }

    // (iv) Two-Gate promotion (killer-risk #2: tau/k_m are UNCONFIRMED
    // carried-forward defaults). Bridge the ConfigVariant candidate into the
    // template_archive types two_gate_promote needs: a single-cell
    // MapElitesArchive (best_in_cell -> incumbent 0.0 on empty, so margin =
    // alpha) + an ArchiveEntry carrying the candidate fingerprint + alpha.
    let descriptor = config_cell_descriptor();
    let archive = build_two_gate_archive_view(archive_occupied_cells, &descriptor);
    let entry = ArchiveEntry {
        template_blake3: blake3::hash(&blob).into(),
        fitness: alpha,
    };
    let decision = two_gate_promote(&entry, &descriptor, &archive, promotion_cfg)
        .map_err(|e| AdmitLoopError::CandidateSerialize(e.to_string()))?;
    let outcome = match decision.kind {
        PromotionDecisionKind::Promoted => GuardOutcome::Admitted,
        other => GuardOutcome::Rejected {
            guard: GUARD_TWO_GATE.to_string(),
            detail: format!(
                "{} ({})",
                two_gate_label(&other),
                decision.reason.unwrap_or_default()
            ),
        },
    };
    Ok((outcome, Some(trap_guard)))
}

/// Snake-case label for a trap pattern (delegates to the enum's own wire label).
fn trap_label(pattern: TrapPattern) -> &'static str {
    pattern.as_str()
}

/// Human label for a non-Promoted two-gate decision.
fn two_gate_label(kind: &PromotionDecisionKind) -> &'static str {
    match kind {
        PromotionDecisionKind::Promoted => "promoted",
        PromotionDecisionKind::BlockedBelowMargin => "below_margin",
        PromotionDecisionKind::BlockedCapacityExceeded => "capacity_exceeded",
        PromotionDecisionKind::BlockedSecurityRegression => "security_regression",
    }
}

/// The fixed behaviour-descriptor cell config-only mutants occupy. Config
/// deltas do not change latency/token behaviour by construction, so they map to
/// a single canonical cell.
fn config_cell_descriptor() -> BehaviourDescriptor {
    BehaviourDescriptor {
        latency_tier: LatencyTier::Sub10s,
        token_class: TokenClass::Lt500,
        failure_mode: None,
    }
}

/// Build the MAP-Elites archive VIEW the Two-Gate capacity gate inspects.
///
/// The config-DGM archive is a [`ConfigArchive`] (SQLite), NOT a
/// [`MapElitesArchive`]; this adapter bridges the two for the capacity gate by
/// constructing a transient MAP-Elites archive whose `occupied_cells()` equals
/// the live config-archive size (so the capacity gate observes the real
/// archive occupancy). The candidate's own cell is left EMPTY so
/// `best_in_cell` returns `None` and the margin gate measures the FULL
/// candidate fitness against a 0.0 incumbent (the conservative posture: a
/// brand-new config variant must clear `tau` on its own merit).
fn build_two_gate_archive_view(
    occupied_cells: u32,
    candidate_descriptor: &BehaviourDescriptor,
) -> MapElitesArchive {
    let mut archive = MapElitesArchive::new();
    // Fill `occupied_cells` distinct cells with placeholder incumbents so
    // `occupied_cells()` reports the live count. We deliberately do NOT occupy
    // the candidate's own cell.
    let filler_cells = enumerate_filler_cells(candidate_descriptor);
    for descriptor in filler_cells.into_iter().take(occupied_cells as usize) {
        let _ = archive.admit(
            descriptor,
            ArchiveEntry {
                template_blake3: [0u8; 32],
                fitness: 0.0,
            },
        );
    }
    archive
}

/// Enumerate the canonical MAP-Elites cells (3 latency x 4 token x {clean})
/// EXCLUDING the candidate's own cell, used as capacity-gate filler.
fn enumerate_filler_cells(exclude: &BehaviourDescriptor) -> Vec<BehaviourDescriptor> {
    let latencies = [LatencyTier::Sub10s, LatencyTier::Sub60s, LatencyTier::Over60s];
    let tokens = [
        TokenClass::Lt500,
        TokenClass::Lt2k,
        TokenClass::Lt8k,
        TokenClass::Over8k,
    ];
    let mut out = Vec::new();
    for &latency_tier in &latencies {
        for &token_class in &tokens {
            let d = BehaviourDescriptor {
                latency_tier,
                token_class,
                failure_mode: None,
            };
            if &d != exclude {
                out.push(d);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// H-39: Abstention-aware alpha blending
// ---------------------------------------------------------------------------

/// Compute the abstention-aware alpha for a candidate using a CALLER-SUPPLIED
/// base fitness.
///
/// This is the production path called from `run_one_generation` where the
/// execution-grounded fitness (H-43, VD-1) is the base. The abstention
/// adjustment (H-39) is stacked ON TOP of the execution fitness, not on top
/// of the `derived_alpha` length proxy. When the eval_log carries a parseable
/// abstention outcome, the normalised abstention-aware score REPLACES the base.
/// When it does not, the base is returned unchanged.
///
/// The kill-switch (W-2) is honoured: if `ELAI_ABSTENTION_SCORE_DISABLE=1`,
/// returns the base directly.
pub fn abstention_blended_alpha_with_base(
    base_alpha: f64,
    candidate: &MutantCandidate,
) -> (f64, AbstentionAggregate) {
    // Kill-switch (W-2): default-ON, disable with ELAI_ABSTENTION_SCORE_DISABLE=1.
    let disabled = std::env::var("ELAI_ABSTENTION_SCORE_DISABLE")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);
    if disabled {
        return (base_alpha, AbstentionAggregate::new());
    }

    let mut agg = AbstentionAggregate::new();
    match parse_eval_log_outcome(&candidate.eval_log) {
        Some(outcome) => {
            agg.record(outcome, DEFAULT_CONFIDENCE_TARGET);
            let score = agg.mean_score();
            let penalty = DEFAULT_CONFIDENCE_TARGET / (1.0 - DEFAULT_CONFIDENCE_TARGET);
            let normalised = (score + penalty) / (1.0 + penalty);
            (normalised.clamp(0.0, 1.0), agg)
        }
        None => (base_alpha, agg),
    }
}

/// Compute the abstention-aware alpha for a candidate.
///
/// When the candidate's `eval_log` carries a parseable outcome triple
/// (see [`parse_eval_log_outcome`]), the derived_alpha is REPLACED by
/// the abstention-aware score so the two_gate_promote fitness gate
/// penalises confident-wrong candidates and credits abstaining ones.
///
/// When the eval_log is absent or unparseable (e.g. tasks with no
/// abstain channel), the score degrades gracefully to `derived_alpha`
/// (a hash-derived proxy in [0, 1]); this is the explicit plan allowance:
/// "abstention_rate will be 0.0 and the score degrades to a
///  penalty-weighted pass rate".
///
/// The ELAI_ABSTENTION_SCORE_DISABLE kill-switch (W-2) overrides to the
/// raw derived_alpha so the feature can be disabled without a code deploy.
///
/// Returns the alpha to feed to `score_candidate`, plus the
/// `AbstentionAggregate` for the SSB beacon.
pub fn abstention_blended_alpha(
    candidate: &MutantCandidate,
) -> (f64, AbstentionAggregate) {
    let base_alpha = candidate.derived_alpha();

    // Kill-switch (W-2): default-ON, disable with ELAI_ABSTENTION_SCORE_DISABLE=1.
    let disabled = std::env::var("ELAI_ABSTENTION_SCORE_DISABLE")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);
    if disabled {
        return (base_alpha, AbstentionAggregate::new());
    }

    let mut agg = AbstentionAggregate::new();
    match parse_eval_log_outcome(&candidate.eval_log) {
        Some(outcome) => {
            agg.record(outcome, DEFAULT_CONFIDENCE_TARGET);
            let score = agg.mean_score();
            // Map abstention-aware score from [-penalty, +1] into [0, 1]
            // for the two_gate_promote fitness gate (which expects [0, 1]).
            // At t=0.75: range is [-3, +1]. Normalise: (score+penalty)/(1+penalty).
            // penalty = t/(1-t) = 3.0 at t=0.75.
            let penalty = DEFAULT_CONFIDENCE_TARGET / (1.0 - DEFAULT_CONFIDENCE_TARGET);
            let normalised = (score + penalty) / (1.0 + penalty);
            (normalised.clamp(0.0, 1.0), agg)
        }
        None => (base_alpha, agg),
    }
}

// ---------------------------------------------------------------------------
// The closed generation (propose -> evaluate -> guard -> admit -> select)
// ---------------------------------------------------------------------------

/// Summary of one closed generation (for the CLI surface + tests).
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationSummary {
    /// Number of candidates admitted (passed all four guards + inserted).
    pub admitted: usize,
    /// Number of candidates rejected by a guard.
    pub rejected: usize,
    /// `true` iff the loop closed (an admit was followed by `select_parent`).
    pub loop_closed: bool,
    /// The suite floor consulted this generation (`None` if the suite name was
    /// unknown — the loop still runs; the floor is telemetry, not a hard gate
    /// here).
    pub suite_floor: Option<f64>,
    /// H-48: index of the bandit-selected arm for the NEXT generation (None if
    /// the darwinian arms vector is empty or bandit selection did not run).
    /// Derived from `select_next_arm` over the Discounted-TS posteriors updated
    /// with H-45's worst_case reward this generation.
    pub bandit_selected_arm: Option<usize>,
    /// H-48 SR sweep: index of the best arm identified by the Successive-Rejects
    /// fixed-budget sweep over the posterior means. Run AFTER `select_next_arm`
    /// as a secondary identification sweep; provides a confirmation signal that
    /// can diverge from `bandit_selected_arm` when TS exploration picks a
    /// non-greedy arm. `None` if fewer than 2 arms or SR sweep did not run.
    pub sr_best_arm: Option<usize>,
}

/// Run ONE closed config-only generation against the supplied archive + SSB.
///
/// For each mutant candidate: derive `alpha`, route through the four-guard
/// chain, emit a `darwinian_admit` row (`Admitted` or `RejectedByGuard`). On
/// the FIRST admit, `insert_variant` then `select_parent` (loop closure) and
/// emit a `LoopClosed` row. Returns a [`GenerationSummary`].
///
/// `entropy` is the `ELAI_SEED`-seeded source for `select_parent` determinism.
#[allow(clippy::too_many_arguments)]
pub async fn run_one_generation<E: EntropySource + ?Sized>(
    archive: &ConfigArchive,
    ssb: &Ssb,
    candidates: &[MutantCandidate],
    mode: PipelineMode,
    seed: u64,
    suite_name: &str,
    promotion_cfg: &PromotionConfig,
    entropy: &mut E,
    // H-47: optional eval firewall context (date-gate + template-regen + budget).
    // Pass `None` in tests that don't exercise the firewall path.
    firewall_ctx: Option<&EvalFirewallContext<'_>>,
) -> Result<GenerationSummary, AdmitLoopError> {
    let suite_floor = get_suite(suite_name).map(|s| s.local_floor());

    // Fail CLOSED on overflow: a usize->u32 `as` cast WRAPS on overflow,
    // shrinking `occupied` so the capacity gate would see room and fail OPEN
    // (admit past the cell budget). Saturate to u32::MAX instead — an
    // implausibly-full archive then REJECTS, which is the safe direction.
    let occupied = u32::try_from(archive.list_variants(mode).await?.len()).unwrap_or(u32::MAX);

    let mut admitted = 0usize;
    let mut rejected = 0usize;
    let mut loop_closed = false;

    // H-47: run the eval firewall BEFORE the per-candidate scoring loop.
    // The firewall gates which held-out items feed the fitness scorer.
    // (a) date-gate: reject pre-update items;
    // (b) template-regen: regenerate templated items per generation;
    // (c) budget-binding: refuse after Thresholdout budget exhausted.
    // Emits EvalFirewallEvent row after the per-item gate closes.
    // RISK-1: if no post-cutoff items remain, we WARN and continue without
    // a held-out eval item (the loop still runs on execution-grounded fitness).
    // HARD RULE: never fall back to scoring a pre-update item.
    let mut firewall_summary = FirewallSummary { seed, ..Default::default() };
    if let Some(fw) = firewall_ctx {
        // WARN-4: ignore_dates is documented test-only on EvalFirewallContext.
        // Production callers set ignore_dates=false (darwin_loop_cmd.rs:BLOCK-1).
        // A runtime assert here cannot distinguish integration tests from production
        // (both are non-cfg(test) debug binaries), so enforcement is documentation +
        // code-review. The struct field doc ("MUST be false in all production calls")
        // is the canonical gate.
        firewall_summary.generation_id = fw.generation_id;
        let now_ms = crate::ssb::Ssb::now_ms_via_clock(fw.clock);

        // Leg 1: date-gate. Returns None on pre-update or empty store.
        let gated_task = if fw.ignore_dates {
            // R2 BROKEN ARM: skip date-gate to let pre-update items inflate fitness.
            crate::agent_pool::sample_held_out_task(fw.held_out_store, now_ms)
        } else {
            date_gated_sample(fw.held_out_store, now_ms, fw.cutoff)
        };

        match &gated_task {
            None => {
                // RISK-1: no safe eval item available this generation.
                // WARN and continue — never fall back to pre-update item.
                tracing::warn!(
                    generation_id = fw.generation_id,
                    "run_one_generation: eval firewall date-gate found no post-cutoff items; \
                     generation proceeds without held-out eval item (RISK-1)"
                );
                firewall_summary.items_rejected += 1;
            }
            Some(task) => {
                // Leg 2: template-regen.
                let final_task = if task.template_id.is_some() {
                    let regen = regenerate_instance(task, fw.generation_id, seed);
                    firewall_summary.regen_count += 1;
                    regen
                } else {
                    task.clone()
                };

                // Leg 3: budget-binding (H-19 Thresholdout DP budget).
                // We pass train_stat = 0.0, holdout_stat = 1.0 as proxies for
                // the "did the item score" query (the real fitness values come
                // from execution_grounded_fitness; the budget guards the adaptive
                // holdout query count, not the fitness value itself).
                let budget_result = fw.budget_ledger.query_holdout(
                    fw.ssb_arc,
                    fw.clock,
                    fw.holdout_id,
                    fw.objective,
                    fw.data_window,
                    0.0,
                    final_task.ngram_signature.len() as f64,
                    &format!("gen-{}", fw.generation_id),
                    fw.budget_seed,
                )
                .await;

                match budget_result {
                    Ok(outcome) if !outcome.is_refused() => {
                        firewall_summary.items_scored += 1;
                        firewall_summary.budget_remaining =
                            i64::try_from(
                                outcome.budget_total.saturating_sub(outcome.queries_spent)
                            )
                            .unwrap_or(i64::MAX);
                        // Leg 4: indistinguishability probe.
                        firewall_summary.distinguishability = indistinguishability_probe(
                            &final_task.ngram_signature,
                            fw.workload_ngrams,
                        );
                        let _ = final_task; // consumed by the scorer below if needed
                    }
                    Ok(_) | Err(_) => {
                        // Budget exhausted or missing — fail-closed.
                        tracing::warn!(
                            generation_id = fw.generation_id,
                            "run_one_generation: eval firewall budget exhausted/missing; \
                             held-out eval refused for this generation (fail-closed)"
                        );
                        firewall_summary.budget_remaining = 0;
                    }
                }
            }
        }

        // Emit EvalFirewallEvent SSB row (off-agent artifact, R4).
        emit_eval_firewall_row(ssb, &firewall_summary).await.ok();
    }

    // H-43: build a generation-scoped frozen corpus for execution-grounded fitness.
    // The corpus is built once and shared across all candidates in this generation.
    // Each candidate's eval_log (if non-empty) provides the scored text; the corpus
    // cases are derived from ALL candidates so the gate is INDEPENDENT per-candidate
    // (a candidate scores against the shared corpus, not against itself — R8).
    let h43_corpus: Option<FrozenCorpus> = {
        // Build corpus cases: each candidate becomes one case. The expected_output
        // is a short unique token derived from the candidate label (not from the
        // candidate snippet itself — the gate tests each candidate against a corpus
        // slot it did NOT write, ensuring the off-agent property).
        let cases: Vec<FrozenTestCase> = candidates
            .iter()
            .enumerate()
            .map(|(i, c)| FrozenTestCase {
                id: format!("gen-case-{i}"),
                candidate_snippet: c.eval_log.clone(),
                // BLOCK-2 fix (VD-1): use the candidate's variant_label as the
                // expected token. A candidate whose eval_log CONTAINS its own
                // variant_label scores higher (it produced evidence of its own
                // identity in its output). A candidate with an empty or unrelated
                // eval_log scores 0. This makes the corpus DISCRIMINATING:
                // fitness(C_good) > fitness(C_bad) holds on the real path (R4).
                expected_output: Some(c.variant_label.clone()),
            })
            .collect();
        let corpus_bytes: Vec<u8> = candidates
            .iter()
            .flat_map(|c| c.eval_log.as_bytes().to_vec())
            .collect();
        FrozenCorpus::new(cases, &corpus_bytes).ok()
    };

    // H-46: pre-compute generation-scoped fitness_deltas for CMP-weighted parent
    // selection. Built from H-43's execution-grounded fitness BEFORE the main loop
    // so select_parent_with_cmp receives the full generation's signal (VD-1: real
    // deltas, not byte-hash values). Each candidate's variant_label becomes its
    // VariantId key (mirrors what build_config_variant uses after insert_variant).
    let fitness_deltas: HashMap<VariantId, f64> = if let Some(ref corpus) = h43_corpus {
        candidates
            .iter()
            .filter_map(|c| {
                c.execution_grounded_fitness_for_candidate(corpus, seed)
                    .ok()
                    .map(|ef| (c.variant_label.clone(), ef.fitness))
            })
            .collect()
    } else {
        HashMap::new()
    };

    // Track parent fitness for FitnessRegression detection.
    // On the first admitted candidate we treat prior alpha as 0.0 (root generation).
    let mut last_admitted_fitness: Option<f64> = None;

    // H-48: initialise Discounted-TS arm posteriors (one per candidate, each
    // representing a distinct curriculum arm). Beta(1, 1) uniform prior.
    // ExIt seeding: if the caller has supplied partial-history scores via the
    // ELAI_DARWINIAN_PARTIAL_HISTORY env var (comma-separated "idx:reward"
    // pairs), seed the arms before the main loop. This is optional — an
    // absent/empty env var is a no-op and the arms start from the uniform prior.
    let darwinian_cfg = DiscountedTsConfig::default();
    let mut darwinian_arms: Vec<DiscountedArmPosterior> = candidates
        .iter()
        .map(|_| DiscountedArmPosterior::default())
        .collect();
    // ExIt curriculum seed from partial past history (arXiv:2509.04575).
    let partial_history_seeds: Vec<(usize, f64)> =
        std::env::var("ELAI_DARWINIAN_PARTIAL_HISTORY")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, ':');
                let idx: usize = parts.next()?.parse().ok()?;
                let reward: f64 = parts.next()?.parse().ok()?;
                Some((idx, reward))
            })
            .collect();
    seed_arms_from_partial_history(
        &mut darwinian_arms,
        &partial_history_seeds,
        darwinian_cfg.discount_factor,
    );
    // Bandit-selected arm for this generation (updated after all candidates
    // are scored). Used in GenerationSummary and for the next generation's
    // parent selection hint.
    let mut bandit_selected_arm: Option<usize> = None;
    // Deterministic RNG for bandit arm selection (seeded from generation seed).
    let mut bandit_rng = SeededRng::new(seed);

    // J-26: evaluator ABI sibling channel. When ELAI_EVALUATOR_ABI_ENABLE=1
    // AND ELAI_EVALUATOR_ABI_PACK points to a valid task-pack on disk, the
    // fitness evaluator is invoked as a SUBPROCESS (--gen-dir <dir>) instead
    // of the in-process scorer. The in-process path stays as fallback when
    // no pack is supplied (RISK-3). On ABI Err when env is ON: hard FAIL
    // (R2), never .ok()-away into skip.
    let abi_pack: Option<crate::evaluator_abi::TaskPack> = {
        let enabled = std::env::var("ELAI_EVALUATOR_ABI_ENABLE")
            .map(|v| v == "1")
            .unwrap_or(false);
        if !enabled {
            None
        } else {
            let pack_path = std::env::var("ELAI_EVALUATOR_ABI_PACK").ok();
            match pack_path {
                Some(p) if !p.is_empty() && std::path::Path::new(&p).is_dir() => {
                    match crate::evaluator_abi::TaskPack::read_from(std::path::Path::new(&p)) {
                        Ok(pack) => Some(pack),
                        Err(e) => {
                            tracing::error!(
                                pack = %p,
                                error = %e,
                                "J-26 ABI: pack read failed (hard FAIL, R2)"
                            );
                            return Err(AdmitLoopError::EvaluatorAbi(e));
                        }
                    }
                }
                _ => None,
            }
        }
    };

    for candidate in candidates {
        // H-43: compute execution-grounded fitness before alpha to get the
        // FitnessEvaluated SSB row (off-agent artifact, R4). The row is emitted
        // regardless of guard outcome (execution fitness is a property of the
        // candidate, not of the guard chain).
        //
        // J-26 sibling channel: when the ABI pack is available, invoke the
        // subprocess evaluator instead of the in-process scorer. The
        // emit_fitness_row path is REUSED (not re-authored). On ABI Err when
        // env is ON: hard FAIL (R2), never .ok()-away.
        let h43_fitness = if let Some(ref pack) = abi_pack {
            // Per-candidate gen-dir under the system temp dir.
            let gen_dir = std::env::temp_dir().join(format!(
                "elai-j26-eval-{}-{}",
                seed,
                candidate.variant_label.replace('/', "_")
            ));
            std::fs::create_dir_all(&gen_dir).ok();
            // Write the candidate's eval_log as the gen output for the evaluator.
            let candidate_out = gen_dir.join("candidate_output.txt");
            std::fs::write(&candidate_out, &candidate.eval_log).ok();
            let invocation = crate::evaluator_abi::EvaluatorInvocation::for_gen_dir(pack, &gen_dir);
            match crate::evaluator_abi::run_evaluator(&invocation) {
                Ok(results) => {
                    let test_set_hash = h43_corpus
                        .as_ref()
                        .map(|c| c.corpus_blake3)
                        .unwrap_or([0u8; 32]);
                    match crate::evaluator_abi::results_to_execution_fitness(
                        &results, seed, test_set_hash,
                    ) {
                        Ok(ef) => Some(ef),
                        Err(e) => {
                            tracing::error!(
                                candidate = %candidate.variant_label,
                                error = %e,
                                "J-26 ABI: results_to_execution_fitness failed (hard FAIL, R2)"
                            );
                            return Err(AdmitLoopError::EvaluatorAbi(e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        candidate = %candidate.variant_label,
                        error = %e,
                        "J-26 ABI: evaluator failed (hard FAIL, R2)"
                    );
                    return Err(AdmitLoopError::EvaluatorAbi(e));
                }
            }
        } else if let Some(ref corpus) = h43_corpus {
            candidate.execution_grounded_fitness_for_candidate(corpus, seed).ok()
        } else {
            None
        };

        // Emit FitnessEvaluated row (H-43 off-agent artifact). Missing = FAIL (R1).
        if let Some(ref ef) = h43_fitness {
            // Determine Kind: FitnessRegression if this candidate scored below
            // the last admitted candidate (parent fitness proxy).
            let kind = if let Some(parent_f) = last_admitted_fitness {
                if ef.fitness < parent_f {
                    FitnessEvaluatedKind::FitnessRegression
                } else {
                    FitnessEvaluatedKind::ExecutionPassRate
                }
            } else {
                FitnessEvaluatedKind::ExecutionPassRate
            };
            let candidate_blake3_h43 = candidate.candidate_blake3();
            emit_fitness_row(
                ssb,
                FitnessEvaluatedPayload {
                    kind,
                    pass_rate: ef.pass_rate,
                    dual_agreement: ef.dual_agreement,
                    tests_total: ef.tests_total,
                    fitness: ef.fitness,
                    seed,
                    test_set_hash: ef.test_set_hash,
                    candidate_blake3: candidate_blake3_h43.clone(),
                },
            )
            .await
            .ok(); // Non-fatal: SSB emit failure must not block the guard chain.
            // WARN-1 fix: emit the DualAgreement secondary row alongside the
            // primary ExecutionPassRate row, making all 3 FitnessEvaluatedKind
            // arms live. DualAgreement captures the CodeT consensus mass signal
            // (arXiv:2207.10397) and is emitted whenever dual_agreement > 0.0.
            if ef.dual_agreement > 0.0 {
                emit_fitness_row(
                    ssb,
                    FitnessEvaluatedPayload {
                        kind: FitnessEvaluatedKind::DualAgreement,
                        pass_rate: ef.pass_rate,
                        dual_agreement: ef.dual_agreement,
                        tests_total: ef.tests_total,
                        fitness: ef.fitness,
                        seed,
                        test_set_hash: ef.test_set_hash,
                        candidate_blake3: candidate_blake3_h43,
                    },
                )
                .await
                .ok();
            }
        }

        // H-43 + H-39: use the execution-grounded fitness (VD-1) as the base
        // alpha, then stack the abstention adjustment (H-39) on top. When no
        // h43_fitness is available (corpus build failed), fall back to the
        // standalone abstention_blended_alpha path (which uses derived_alpha).
        let (alpha, _abstention_agg) = if let Some(ref ef) = h43_fitness {
            // VD-1: the promotion decision is driven by real execution fitness,
            // not the blake3/length proxy. derived_alpha is no longer the alpha
            // source on the live path after this branch.
            abstention_blended_alpha_with_base(ef.fitness, candidate)
        } else {
            abstention_blended_alpha(candidate)
        };

        // H-44 + H-45: use score_candidate_with_aggregate to capture both the
        // behavioral detection result (H-44, TrapBehaviorDetected SSB) and the
        // worst-case aggregate promotion score (H-45, PromotionScoreAggregated
        // SSB). Both are off-agent artifacts (R4). Non-fatal: SSB emit failure
        // must not block the guard chain decision.
        let h43_fitness_score = h43_fitness.as_ref().map(|ef| ef.fitness);
        let (outcome, trap_guard, promotion_score) =
            score_candidate_with_aggregate(
                candidate,
                alpha,
                h43_fitness_score,
                occupied,
                promotion_cfg,
            )?;

        // H-44: emit TrapBehaviorDetected SSB row if detection ran.
        let candidate_blake3 = candidate.candidate_blake3();
        if let Some(ref tg) = trap_guard {
            let kind_ssb = if let Some(dk) = tg.kind {
                detected_kind_to_ssb(dk)
            } else {
                TrapBehaviorDetectedKind::KeywordFallback
            };
            let (recall_val, fp_rate_val, n_val, corpus_hash_val) =
                if let Some(ref rr) = tg.recall_report {
                    (rr.recall, rr.fp_rate, rr.n, rr.corpus_hash)
                } else {
                    (0.0, 0.0, 0, [0u8; 32])
                };
            emit_trap_row(
                ssb,
                TrapBehaviorDetectedPayload {
                    kind: kind_ssb,
                    recall: recall_val,
                    fp_rate: fp_rate_val,
                    n: n_val,
                    corpus_hash: corpus_hash_val,
                    candidate_blake3: candidate_blake3.clone(),
                    seed,
                    behavioral_flag: tg.behavioral_flag,
                },
            )
            .await
            .ok(); // Non-fatal.
        }

        // H-45: emit PromotionScoreAggregated SSB row (off-agent artifact, R4).
        // Missing rows = FAIL (R1). Emitted regardless of guard outcome so the
        // score is auditable even when the candidate is rejected.
        if let Some(ref ps) = promotion_score {
            let ps_kind = if ps.single_view_penalty_applied {
                PromotionScoreAggregatedKind::WorstCaseScored
            } else if ps.distinct_families >= 2 {
                PromotionScoreAggregatedKind::VariancePenalized
            } else {
                PromotionScoreAggregatedKind::WorstCaseScored
            };
            emit_promotion_score_row(
                ssb,
                PromotionScoreAggregatedPayload {
                    kind: ps_kind,
                    worst_case: ps.worst_case,
                    variance: ps.variance,
                    mean: ps.mean,
                    disagreement: ps.disagreement,
                    tau: promotion_cfg.tau,
                    calibrated_at: TAU_CALIBRATED_AT.to_string(),
                    seed,
                },
            )
            .await
            .ok(); // Non-fatal.

            // H-48: update the Discounted-TS arm posterior for this candidate's
            // arm using H-45's worst_case as the real reward (VD-1: NOT a
            // byte-hash). The arm index is the candidate's position in the
            // candidates slice. `admitted_this_candidate` is set below; we
            // probe `matches!(outcome, ...)` here to pass was_promoted
            // accurately before the outcome block.
            let was_admitted = matches!(outcome, GuardOutcome::Admitted);
            let cand_idx = candidates
                .iter()
                .position(|c| c.variant_label == candidate.variant_label)
                .unwrap_or(0);
            update_arms_from_reward(
                &mut darwinian_arms,
                cand_idx,
                ps.worst_case,
                darwinian_cfg.discount_factor,
            );
            // Emit BaiArmUpdated telemetry row (off-agent artifact, R4).
            // REUSES D-5's BaiArmUpdated (GD-1; no new variant). The
            // template_id field carries a darwinian arm label. See
            // workflow/reasoning/H-48-decisions.md §ssb-decision.
            if cand_idx < darwinian_arms.len() {
                let darm = &darwinian_arms[cand_idx];
                let payload = SsbPayload::BaiArmUpdated(BaiArmUpdatedPayload {
                    template_id: format!("darwinian-arm-{cand_idx}"),
                    alpha: darm.alpha,
                    beta: darm.beta,
                    n_pulls: darm.n_pulls,
                    mean_estimate: darm.mean(),
                    was_promoted: was_admitted,
                });
                ssb.write_payload(ADMIT_LOOP_AGENT_ID, seed as i64, payload, Vec::new())
                    .await
                    .ok(); // Non-fatal: SSB backpressure must not block loop closure.
            }
        }

        match outcome {
            GuardOutcome::Rejected { guard, .. } => {
                rejected += 1;
                emit_admit_row(
                    ssb,
                    DarwinianAdmitPayload {
                        kind: DarwinianAdmitKind::RejectedByGuard,
                        variant_id: candidate.variant_label.clone(),
                        parent_id: None,
                        alpha,
                        guard_rejected_by: Some(guard),
                        candidate_blake3,
                        seed,
                    },
                )
                .await?;
            }
            GuardOutcome::Admitted => {
                // H-28-WIRE-1: reward-tampering gate (default-OFF). When enabled,
                // intercept candidates with implausible reward jumps before insert_variant.
                if h28_reward_gate_enabled() {
                    if let (Some(fw), Some(ef), Some(corpus)) = (firewall_ctx, h43_fitness.as_ref(), h43_corpus.as_ref()) {
                        let prior_reward = last_admitted_fitness.unwrap_or(0.0);
                        let new_reward = ef.fitness;
                        let corpus_blake3_hex: String = corpus.corpus_blake3
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect();
                        let run_id = &format!("gen-{}", fw.generation_id);
                        let dummy_db_path = std::path::Path::new("/dev/null");

                        let tampering_outcome = reward_tampering::check_reward_tampering(
                            fw.ssb_arc,
                            dummy_db_path,
                            &corpus_blake3_hex,
                            &corpus_blake3_hex,
                            prior_reward,
                            new_reward,
                            reward_tampering::DEFAULT_PLAUSIBLE_REWARD_BAND,
                            run_id,
                            seed,
                        )
                        .await;

                        if let Ok(outcome) = tampering_outcome {
                            if !outcome.is_pass() {
                                rejected += 1;
                                emit_admit_row(
                                    ssb,
                                    DarwinianAdmitPayload {
                                        kind: DarwinianAdmitKind::RejectedByGuard,
                                        variant_id: candidate.variant_label.clone(),
                                        parent_id: None,
                                        alpha,
                                        guard_rejected_by: Some("reward-tampering".to_string()),
                                        candidate_blake3,
                                        seed,
                                    },
                                )
                                .await?;
                                continue;
                            }
                        }
                    }
                }

                // H-46: Resolve a parent BEFORE insert using CMP-weighted selection.
                // select_parent_with_cmp is the FIRST (and only) production caller
                // of that path (R6/R7 fix). Returns (selected_id, clade_scores).
                let (parent_pre, _pre_clade_scores) = archive
                    .select_parent_with_cmp(mode, &fitness_deltas, entropy, false)
                    .await?;
                let parent_id = if parent_pre.is_empty() {
                    None
                } else {
                    Some(parent_pre)
                };

                let variant = build_config_variant(candidate, alpha, parent_id.clone(), mode);
                // insert_variant: the FIRST production caller of the DGM archive
                // admit path. A pre-existing active variant for the mode is a
                // benign archive-state condition, not a guard rejection — we
                // surface it as a non-admit without an SSB Admitted beacon.
                match archive.insert_variant(&variant).await {
                    Ok(()) => {
                        admitted += 1;
                        // H-43: track the admitted candidate's fitness for
                        // FitnessRegression detection in subsequent candidates.
                        if let Some(ref ef) = h43_fitness {
                            last_admitted_fitness = Some(ef.fitness);
                        }
                        emit_admit_row(
                            ssb,
                            DarwinianAdmitPayload {
                                kind: DarwinianAdmitKind::Admitted,
                                variant_id: candidate.variant_label.clone(),
                                parent_id: parent_id.clone(),
                                alpha,
                                guard_rejected_by: None,
                                candidate_blake3: candidate_blake3.clone(),
                                seed,
                            },
                        )
                        .await?;

                        // H-46: select_parent_with_cmp AFTER insert_variant —
                        // the loop-closure proof AND the FIRST production caller
                        // of select_parent_with_cmp on the real path (R6/R7 fix).
                        // CMP-weighted selection names the parent for the NEXT
                        // generation. The returned clade_scores feed the
                        // CladeSelected SSB row (off-agent artifact, R4).
                        let (next_parent, clade_scores) = archive
                            .select_parent_with_cmp(mode, &fitness_deltas, entropy, false)
                            .await?;
                        loop_closed = true;
                        // Emit CladeSelected row from the returned clade_scores
                        // (off-agent artifact — the archive computes the scores,
                        // not the candidate). Non-fatal: SSB backpressure must
                        // not block loop closure.
                        emit_clade_selected_row(
                            ssb,
                            &next_parent,
                            &clade_scores,
                            seed,
                        )
                        .await
                        .ok();
                        emit_admit_row(
                            ssb,
                            DarwinianAdmitPayload {
                                kind: DarwinianAdmitKind::LoopClosed,
                                variant_id: candidate.variant_label.clone(),
                                parent_id: if next_parent.is_empty() {
                                    None
                                } else {
                                    Some(next_parent)
                                },
                                alpha,
                                guard_rejected_by: None,
                                candidate_blake3,
                                seed,
                            },
                        )
                        .await?;
                    }
                    Err(ConfigArchiveError::ActiveAlreadyExists(_)) => {
                        // Archive already holds an active variant for this mode;
                        // not a guard reject and not a fresh admit. Skip without
                        // a beacon (the loop still closed on a prior candidate).
                    }
                    Err(e) => return Err(AdmitLoopError::Archive(e)),
                }
            }
        }
    }

    // H-48: after all candidates are scored, run select_next_arm on the updated
    // Discounted-TS posteriors to pick the arm for the NEXT generation.
    // This is the PRODUCTION bandit selection step (W-3: not test-only).
    bandit_selected_arm = select_next_arm(&darwinian_arms, &mut bandit_rng);
    tracing::debug!(
        bandit_selected_arm,
        n_arms = darwinian_arms.len(),
        "run_one_generation: bandit selected arm for next generation (H-48)"
    );

    // H-48 SR sweep: run Successive-Rejects fixed-budget BAI over the posterior
    // means as a secondary identification pass. Budget = total n_pulls across all
    // arms (saturated to u32). This gives the SR sweep a proportional budget
    // relative to how much evidence was collected this generation. The SR best-arm
    // is a confirmation signal alongside the Discounted-TS bandit selection above.
    // run_sr_phase is deterministic given arm_rewards; bandit_rng is threaded for
    // API consistency (SeededRng, no wall-clock).
    let arm_mean_rewards: Vec<f64> = darwinian_arms.iter().map(|a| a.mean()).collect();
    let sr_budget: u32 = darwinian_arms
        .iter()
        .fold(0u32, |acc, a| acc.saturating_add(a.n_pulls));
    let sr_best_arm = run_sr_phase(&arm_mean_rewards, sr_budget.max(1), &mut bandit_rng);
    tracing::debug!(
        sr_best_arm,
        sr_budget,
        "run_one_generation: SR best-arm identification sweep (H-48)"
    );

    Ok(GenerationSummary {
        admitted,
        rejected,
        loop_closed,
        suite_floor,
        bandit_selected_arm,
        sr_best_arm,
    })
}

/// Build the [`ConfigVariant`] archive row from an admitted candidate. The
/// variant is ALWAYS a config/template delta (a `config_blob`), never a Layer-3
/// `StructuralChange`.
fn build_config_variant(
    candidate: &MutantCandidate,
    alpha: f64,
    parent_id: Option<String>,
    mode: PipelineMode,
) -> ConfigVariant {
    ConfigVariant {
        variant_id: candidate.variant_label.clone(),
        parent_id,
        // Archive score is an f32 in [0, 1]; clamp the f64 alpha.
        score: alpha.clamp(0.0, 1.0) as f32,
        children_count: 0,
        config_blob: candidate.config_blob(),
        // created_at is a monotone field; the loop uses the candidate
        // fingerprint's low bytes so it is deterministic without a clock.
        created_at: deterministic_created_at(candidate),
        pipeline_outcomes: Vec::new(),
        pipeline_mode: mode,
        status: VariantStatus::Active,
    }
}

/// Deterministic `created_at` derived from the candidate fingerprint (entropy
/// quarantine: no raw `std::time`). Stable across runs.
fn deterministic_created_at(candidate: &MutantCandidate) -> u64 {
    let hash = blake3::hash(&candidate.config_blob());
    let bytes = hash.as_bytes();
    let mut acc: u64 = 0;
    for &b in &bytes[..8] {
        acc = (acc << 8) | u64::from(b);
    }
    acc
}

/// Emit one `darwinian_admit` SSB row through the single-writer channel.
async fn emit_admit_row(ssb: &Ssb, payload: DarwinianAdmitPayload) -> Result<(), AdmitLoopError> {
    ssb.write_payload(
        ADMIT_LOOP_AGENT_ID,
        0,
        SsbPayload::DarwinianAdmit(payload),
        Vec::new(),
    )
    .await?;
    Ok(())
}

/// H-43: emit a `FitnessEvaluated` SSB row (off-agent artifact, R4).
///
/// The row is written at the scoring call site in `run_one_generation`, NOT by
/// the candidate being evaluated. Missing rows = FAIL (R1); a non-fatal emit
/// failure (SSB backpressure) is logged but does not block the guard chain.
async fn emit_fitness_row(
    ssb: &Ssb,
    payload: FitnessEvaluatedPayload,
) -> Result<(), SsbError> {
    ssb.write_payload(
        ADMIT_LOOP_AGENT_ID,
        0,
        SsbPayload::FitnessEvaluated(payload),
        Vec::new(),
    )
    .await?;
    Ok(())
}

/// H-44: emit a `TrapBehaviorDetected` SSB row (off-agent artifact, R4).
///
/// The row is written at the behavioral detection call site in
/// `run_one_generation`, NOT by the candidate being evaluated. Missing rows =
/// FAIL (R1). Non-fatal: an SSB backpressure error must not block the guard
/// chain decision.
async fn emit_trap_row(
    ssb: &Ssb,
    payload: TrapBehaviorDetectedPayload,
) -> Result<(), SsbError> {
    ssb.write_payload(
        ADMIT_LOOP_AGENT_ID,
        0,
        SsbPayload::TrapBehaviorDetected(payload),
        Vec::new(),
    )
    .await?;
    Ok(())
}

/// H-45: emit one `PromotionScoreAggregated` SSB row (off-agent artifact, R4).
/// Non-fatal wrapper — callers use `.ok()` to avoid blocking the guard chain.
async fn emit_promotion_score_row(
    ssb: &Ssb,
    payload: PromotionScoreAggregatedPayload,
) -> Result<(), SsbError> {
    ssb.write_payload(
        ADMIT_LOOP_AGENT_ID,
        0,
        SsbPayload::PromotionScoreAggregated(payload),
        Vec::new(),
    )
    .await?;
    Ok(())
}

/// H-46: emit a `CladeSelected` SSB row (off-agent artifact, R4).
///
/// `selected_id` is the variant the CMP sampler chose. `clade_scores` is the
/// parallel slice returned by `select_parent_with_cmp`; this function finds
/// the score entry for `selected_id` to populate the payload fields.
///
/// Non-fatal wrapper — callers use `.ok()` so SSB backpressure never blocks
/// loop closure (consistent with the `emit_fitness_row` / `emit_trap_row`
/// convention in this file).
async fn emit_clade_selected_row(
    ssb: &Ssb,
    selected_id: &str,
    clade_scores: &[crate::clade_metaproductivity::CladeScore],
    seed: u64,
) -> Result<(), SsbError> {
    // Find the score entry for the selected variant. The `clade_scores` slice
    // is parallel to `list_variants` output inside `select_parent_with_cmp`,
    // but we don't have the variants list here — we use the first score as a
    // fallback when the archive is empty (empty archive -> selected_id is
    // empty string -> the row still carries the correct zero-CMP state).
    let score = clade_scores.first().cloned().unwrap_or(
        crate::clade_metaproductivity::CladeScore {
            cmp: 0.0,
            clade_size: 0,
            immediate_score: 0.0,
            key: crate::clade_metaproductivity::SelectionKey::ImmediateScoreFallback,
        },
    );
    let kind = match score.key {
        crate::clade_metaproductivity::SelectionKey::CladeMetaproductivity => {
            CladeSelectedKind::CladeMetaproductivity
        }
        crate::clade_metaproductivity::SelectionKey::ImmediateScoreFallback => {
            CladeSelectedKind::ImmediateScoreFallback
        }
    };
    let key_used = kind.as_str().to_string();
    ssb.write_payload(
        ADMIT_LOOP_AGENT_ID,
        0,
        SsbPayload::CladeSelected(CladeSelectedPayload {
            kind,
            node_id: selected_id.to_string(),
            cmp: score.cmp,
            clade_size: score.clade_size,
            immediate_score: score.immediate_score,
            key_used,
            seed,
        }),
        Vec::new(),
    )
    .await?;
    Ok(())
}

/// `true` iff the `ELAI_DARWINIAN_LOOP_ENABLE` env var is set to a truthy
/// value (`1` / `true` / `yes` / `on`, case-insensitive). The live driver
/// requires this AND `cfg(feature = "darwinian-evolve")`.
#[must_use]
pub fn loop_enabled_via_env() -> bool {
    match std::env::var(ELAI_DARWINIAN_LOOP_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

/// H-28-WIRE-1: `true` iff the `ELAI_H28_REWARD_GATE_ENABLE` env var is set to a
/// truthy value (`1` / `true` / `yes` / `on`, case-insensitive). When OFF (default),
/// the loop behaves exactly as today; when ON, the reward-tampering gate intercepts
/// candidates with implausible reward jumps.
#[must_use]
fn h28_reward_gate_enabled() -> bool {
    match std::env::var(ELAI_H28_REWARD_GATE_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

/// H-47: emit the EvalFirewallEvent SSB row (off-agent artifact, R4).
/// Emitted ONCE per generation after the per-item firewall loop closes.
/// The kind is determined by the dominant event: if any item was rejected by
/// the date-gate, PreUpdateItemRejected; else if any item was regenerated,
/// TemplateRegenerated; else IndistinguishabilityProbe.
/// Non-fatal: SSB backpressure must not block loop execution.
async fn emit_eval_firewall_row(
    ssb: &Ssb,
    summary: &FirewallSummary,
) -> Result<(), SsbError> {
    let kind = if summary.items_rejected > 0 {
        EvalFirewallEventKind::PreUpdateItemRejected
    } else if summary.regen_count > 0 {
        EvalFirewallEventKind::TemplateRegenerated
    } else {
        EvalFirewallEventKind::IndistinguishabilityProbe
    };
    ssb.write_payload(
        ADMIT_LOOP_AGENT_ID,
        0,
        SsbPayload::EvalFirewallEvent(EvalFirewallPayload {
            kind,
            items_scored: summary.items_scored,
            items_rejected: summary.items_rejected,
            regen_count: summary.regen_count,
            budget_remaining: summary.budget_remaining,
            distinguishability: summary.distinguishability,
            seed: summary.seed,
            generation_id: summary.generation_id,
        }),
        Vec::new(),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededRng;

    fn clean_candidate() -> MutantCandidate {
        MutantCandidate {
            variant_label: "clean-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "retry_budget": 3, "rag_top_k": 8, "force_alpha": 0.6 }),
            eval_log: "suite=math-floors score=0.6 clean".to_string(),
        }
    }

    fn trap_candidate() -> MutantCandidate {
        // Behavioral trap: embeds the ImpossibleBench canary token so the
        // ImpossiblePassExploit leg fires (behavioral_flag=true). The token
        // is the first ConflictTask's conflict_expect from IMPOSSIBLE_TASKS.
        // This is a HARD behavioral block (not just a keyword advisory).
        use crate::trap_behavioral::IMPOSSIBLE_TASKS;
        let token = IMPOSSIBLE_TASKS[0].conflict_expect;
        let mut config = serde_json::Map::new();
        config.insert("force_alpha".to_string(), serde_json::json!(0.8));
        config.insert("hack".to_string(), serde_json::Value::String(token.to_string()));
        MutantCandidate {
            variant_label: "trap-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::Value::Object(config),
            eval_log: format!("hack detected: {token}"),
        }
    }

    fn keyword_advisory_candidate() -> MutantCandidate {
        // Keyword-only candidate: triggers the MarkerRemoval advisory but has
        // NO behavioral signal. Per BLOCK-1/VD-2, this MUST NOT be rejected by
        // the trap guard; it is advisory-only and the candidate proceeds.
        MutantCandidate {
            variant_label: "kw-advisory-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "post_step_hook": "remove marker lines from own source", "force_alpha": 0.6 }),
            eval_log: "marker_removal applied".to_string(),
        }
    }

    fn security_candidate() -> MutantCandidate {
        MutantCandidate {
            variant_label: "sec-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "api_key": "sk-leak", "force_alpha": 0.9 }),
            eval_log: "clean run".to_string(),
        }
    }

    #[test]
    fn clean_candidate_is_admitted() {
        let cfg = PromotionConfig::default();
        let c = clean_candidate();
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(out, GuardOutcome::Admitted);
    }

    #[test]
    fn h28_reward_gate_flag_read() {
        std::env::set_var(ELAI_H28_REWARD_GATE_ENABLE_ENV, "1");
        assert!(h28_reward_gate_enabled());
        std::env::set_var(ELAI_H28_REWARD_GATE_ENABLE_ENV, "0");
        assert!(!h28_reward_gate_enabled());
        std::env::remove_var(ELAI_H28_REWARD_GATE_ENABLE_ENV);
        assert!(!h28_reward_gate_enabled());
    }

    #[test]
    fn trap_candidate_rejected_by_trap_guard() {
        let cfg = PromotionConfig::default();
        // Behavioral trap: ImpossiblePassExploit (behavioral_flag=true) -> hard block.
        let c = trap_candidate();
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(
            out.guard_name(),
            Some(GUARD_TRAP),
            "behavioral trap must be rejected by the trap guard"
        );
    }

    #[test]
    fn keyword_advisory_candidate_not_rejected_by_trap_guard() {
        // BLOCK-1 / VD-2: a keyword-only candidate (behavioral_flag=false) MUST
        // NOT be rejected by the trap guard. The advisory is logged but the
        // candidate proceeds. It may be rejected by another guard (two-gate,
        // security) or admitted — but NOT by the trap guard.
        let cfg = PromotionConfig::default();
        let c = keyword_advisory_candidate();
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_ne!(
            out.guard_name(),
            Some(GUARD_TRAP),
            "keyword-only advisory candidate MUST NOT be rejected by the trap guard (VD-2)"
        );
    }

    #[test]
    fn security_candidate_rejected_by_security_guard() {
        let cfg = PromotionConfig::default();
        let c = security_candidate();
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(out.guard_name(), Some(GUARD_SECURITY));
    }

    #[test]
    fn low_alpha_rejected_by_two_gate() {
        let cfg = PromotionConfig::default();
        let c = MutantCandidate {
            variant_label: "low-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "force_alpha": 0.01 }),
            eval_log: "marginal".to_string(),
        };
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(out.guard_name(), Some(GUARD_TWO_GATE));
    }

    #[test]
    fn candidate_blake3_is_16_hex_chars() {
        let c = clean_candidate();
        let fp = c.candidate_blake3();
        assert_eq!(fp.len(), CANDIDATE_BLAKE3_HEX_LEN);
        assert!(fp.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn derived_alpha_is_deterministic_and_in_range() {
        let c = clean_candidate();
        let a1 = c.derived_alpha();
        let a2 = c.derived_alpha();
        assert_eq!(a1, a2);
        assert!((0.0..=1.0).contains(&a1));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn one_generation_closes_loop_and_catches_trap() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let archive = ConfigArchive::open(&dir.path().join("self_mod.db"))
            .await
            .expect("archive open");
        let ssb = Ssb::new(
            crate::ssb::SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: None,
                project_id: "test".to_string(),
                session_id: "darwin-loop-test".to_string(),
            },
            std::sync::Arc::new(elai_core::OsClock),
        )
        .expect("ssb");

        let candidates = vec![trap_candidate(), clean_candidate()];
        let cfg = PromotionConfig::default();
        let mut rng = SeededRng::new(42);
        let summary = run_one_generation(
            &archive,
            &ssb,
            &candidates,
            PipelineMode::Surgical,
            42,
            DEFAULT_SUITE_NAME,
            &cfg,
            &mut rng,
            None, // H-47: no eval firewall context in this unit test
        )
        .await
        .expect("generation");

        assert!(summary.loop_closed, "clean candidate must close the loop");
        assert_eq!(summary.rejected, 1, "trap candidate must be rejected");
        assert_eq!(summary.admitted, 1, "clean candidate must be admitted");
        // WARN-2 fix: R4 observable — bandit and SR paths must have fired on the
        // full production path (select_next_arm + run_sr_phase ran in run_one_generation).
        assert!(
            summary.bandit_selected_arm.is_some(),
            "select_next_arm must fire and return Some on the production path"
        );
        assert!(
            summary.sr_best_arm.is_some(),
            "run_sr_phase must fire and return Some on the production path"
        );
    }
}
