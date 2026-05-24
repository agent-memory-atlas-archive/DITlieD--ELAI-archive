//! D-7: Cross-domain transfer canary — 5 orthogonal task families.
//!
//! Companion §3.5 (lines 268-276) + NanoFlux (arxiv:2509.23252, T1):
//! improvement on a target task family can silently regress adjacent
//! families if the verifier is single-domain. Defense: every L2 mutation
//! candidate is evaluated across ALL 5 families; promotion is blocked when
//! any non-target family regresses meaningfully.
//!
//! Surfaces (all closed, L3-frozen — extensions require an additive enum
//! bump):
//!   - [`CanaryFamily`] — the 5 orthogonal task families.
//!   - [`Domain`] mapping — see [`CanaryFamily::domain`] for the 1:1
//!     bijection between families and the L3-frozen `Domain` enum
//!     extended for D-7.
//!   - [`FamilyEvalResult`] — per-family pass/fail counters + transfer
//!     score (candidate pass-rate minus baseline pass-rate) + the
//!     baseline reference score itself.
//!   - [`CanaryReport`] — aggregate report with one
//!     [`FamilyEvalResult`] per evaluated family + an overall transfer
//!     score + a [`CanaryDecision`].
//!   - [`CanaryDecision`] — closed sum: `Pass` / `MarginalPass` /
//!     `Fail { dominant_family }`.
//!
//! Wiring contract (HARD):
//!   1. [`run_canary`] computes the [`CanaryReport`] from a per-family
//!      bundle of held-out tasks + a candidate evaluator + a baseline
//!      evaluator.
//!   2. [`emit_canary_run`] writes one [`SsbActionType::CanaryRunCompleted`]
//!      row + one [`SsbActionType::CanaryFamilyEvaluated`] row per family
//!      that was evaluated. Matched on `report_id` (blake3 of the
//!      canonical report serialization).
//!   3. The BAI promotion gate / dashboard consumer reads recent canary
//!      reports via [`crate::agent_pool::recent_canary_status`].
//!
//! Anti-stale: blake3 (not sha2 / blake2) for the `report_id`. The
//! evaluator surface uses a native `async fn` trait method so async
//! verifier consensus (D-3) can be hooked in without the `async-trait`
//! crate. `thiserror` for the public error type.
//!
//! Reads:
//!   - workflow/reasoning/D-7-decisions.md
//!   - docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md
//!     §3.5 cross-domain canary.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::held_out::HeldOutTask;
use crate::reasoning_template::Domain;
use crate::ssb::{
    CanaryFamilyEvaluatedPayload, CanaryFamilyPassPayload, CanaryInsufficientDataPayload,
    CanaryRunCompletedPayload, CrossDomainRegressionPayload, Ssb, SsbError, SsbPayload,
};

// ---------------------------------------------------------------------------
// Closed enums (L3-frozen)
// ---------------------------------------------------------------------------

/// Variant count for [`CanaryFamily`]. Bumping requires an additive enum
/// extension + a structural change to every consumer.
pub const CANARY_FAMILY_VARIANT_COUNT: usize = 5;

/// The 5 orthogonal task families per the D-7 plan. NanoFlux
/// (arxiv:2509.23252, T1) reports 5 as the literature sweet spot — 3 too
/// few (misses dominant regressions); 12 too many (signal washes out).
/// See workflow/reasoning/D-7-decisions.md decision (a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanaryFamily {
    /// rust/python compile + unit-test pass; fixtures from
    /// exercism-public + leetcode-public.
    CodeFix,
    /// SymPy-checkable algebraic identities + step-by-step proofs;
    /// fixtures from PRMBench math subset (D-19 ingests).
    MathProof,
    /// Natural-language inference + chain-of-reasoning; fixtures from
    /// HotpotQA + AdversarialQA.
    ProseReason,
    /// Multi-step decomposition + dependency order; fixtures from
    /// PlanBench + AlfWorld.
    PlanningDecomposition,
    /// Tool-grounded task families (web/api/tool-call benchmarks).
    /// Replaces the plan-doc's KnowledgeQa per the orchestrator brief
    /// since D-7's BAI gate already consumes PrmBench knowledge rows
    /// from D-19 and the ToolUse family covers a strictly orthogonal
    /// axis (interaction vs recall).
    ToolUse,
}

impl CanaryFamily {
    /// Snake-case wire label. Used by the SSB payload + the dashboard
    /// panel so consumers group without re-deserializing the enum.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodeFix => "code_fix",
            Self::MathProof => "math_proof",
            Self::ProseReason => "prose_reason",
            Self::PlanningDecomposition => "planning_decomposition",
            Self::ToolUse => "tool_use",
        }
    }

    /// 1:1 mapping to the L3-frozen [`Domain`] enum.
    #[must_use]
    pub fn domain(&self) -> Domain {
        match self {
            Self::CodeFix => Domain::Code,
            Self::MathProof => Domain::Math,
            Self::ProseReason => Domain::Prose,
            Self::PlanningDecomposition => Domain::Planning,
            Self::ToolUse => Domain::ToolUse,
        }
    }

    /// The 5 canonical families in declaration order. Mirrors the
    /// enum's variant list so the canary run order is deterministic
    /// across processes / sessions.
    #[must_use]
    pub fn all() -> [CanaryFamily; CANARY_FAMILY_VARIANT_COUNT] {
        [
            CanaryFamily::CodeFix,
            CanaryFamily::MathProof,
            CanaryFamily::ProseReason,
            CanaryFamily::PlanningDecomposition,
            CanaryFamily::ToolUse,
        ]
    }
}

/// Decision returned by [`run_canary`]. Closed sum so the BAI promotion
/// gate's dispatch on it is exhaustive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanaryDecision {
    /// Every family within the tolerance band; transfer is healthy.
    /// Overall transfer score is positive OR zero AND no per-family
    /// delta is below `-pass_band`.
    Pass,
    /// Overall transfer is positive but at least one family
    /// regressed within the tolerance band (above `-fail_band` but
    /// below `-pass_band`). The candidate may still promote but the
    /// dashboard flags the cross-domain concern.
    MarginalPass,
    /// A family regressed beyond the fail band. Carries the dominant
    /// (largest-magnitude) regressing family so the BAI gate's row
    /// surfaces a single named family.
    Fail {
        /// The family with the most negative `transfer_score` on the
        /// run.
        dominant_family: CanaryFamily,
    },
}

impl CanaryDecision {
    /// Snake-case `kind` label for the SSB payload.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::MarginalPass => "marginal_pass",
            Self::Fail { .. } => "fail",
        }
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Per-family evaluator output. Held inside [`CanaryReport::per_family`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FamilyEvalResult {
    pub family: CanaryFamily,
    /// How many tasks were drawn from the family. Capped to
    /// `min_per_family` (caller-supplied) — the per-family bundle MAY
    /// hold more, but we evaluate only the first `min_per_family`.
    pub n_evaluated: u32,
    /// Candidate pass count.
    pub pass_count: u32,
    /// Candidate fail count. `pass_count + fail_count == n_evaluated`.
    pub fail_count: u32,
    /// Candidate pass-rate minus baseline pass-rate. Range
    /// `[-1.0, +1.0]`.
    pub transfer_score: f64,
    /// Reference pass-rate (the baseline template's pass-rate on the
    /// same held-out subset). Stored alongside the delta so the
    /// dashboard renders both bars without recomputing.
    pub baseline_score: f64,
}

/// The aggregate canary report. One produced per [`run_canary`] call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryReport {
    /// Blake3 (64-char hex) of the canonical serialization of this
    /// report. Stable across processes — the BAI gate / dashboard
    /// matches per-family rows back to this run via this id.
    pub report_id: String,
    /// Wall-clock UNIX ms when the run completed (caller-supplied for
    /// determinism in tests).
    pub ts_ms: i64,
    /// One result per evaluated family. Stable order:
    /// [`CanaryFamily::all`].
    pub per_family: Vec<FamilyEvalResult>,
    /// Mean of `per_family[*].transfer_score`. Range `[-1.0, +1.0]`.
    pub overall_transfer_score: f64,
    /// Final decision after the band thresholds were applied.
    pub canary_decision: CanaryDecision,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CanaryError {
    #[error("insufficient samples for family {family:?}: have {have}, need {need}")]
    InsufficientSamples {
        family: CanaryFamily,
        have: usize,
        need: usize,
    },
    #[error("evaluator error: {0}")]
    Evaluator(String),
    #[error("ssb error: {0}")]
    Ssb(#[from] SsbError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Tunables
// ---------------------------------------------------------------------------

/// Per-family `transfer_score` band beyond which a regression promotes
/// to `Fail`. Companion §3.5 suggested 5% as the noise-balanced
/// operating point (≥10 samples at p≈0.7 gives std ≈ 3%). D-7 plan
/// decision (b).
pub const DEFAULT_FAIL_BAND: f64 = 0.05;

/// Per-family `transfer_score` band beyond which a regression is
/// flagged as marginal but does not promote to `Fail`. Half the fail
/// band by default so a 2-3% per-family regression still reaches the
/// dashboard but does not auto-block promotion.
pub const DEFAULT_MARGINAL_BAND: f64 = 0.025;

/// Minimum held-out tasks per family. Below this, the family's
/// `FamilyEvalResult` is omitted from the report and the run is
/// rejected with [`CanaryError::InsufficientSamples`].
pub const DEFAULT_MIN_PER_FAMILY: usize = 5;

// ---------------------------------------------------------------------------
// Evaluator surface (native async-fn trait — no async-trait crate)
// ---------------------------------------------------------------------------

/// Pluggable Pass/Fail evaluator for a single held-out task. Used by
/// [`run_canary`] to score both the candidate and the baseline
/// templates against the same per-family task bundles.
///
/// Native `async fn` in trait (Rust 1.75+) — no `async-trait` crate per
/// the anti-stale-training rule. Implementors return `Ok(true)` on
/// pass, `Ok(false)` on fail; errors propagate as
/// [`CanaryError::Evaluator`].
pub trait CanaryEvaluator: Send + Sync {
    /// Score a single held-out task. Implementors MUST run the same
    /// task body through whichever verifier consensus surface the
    /// orchestrator currently routes through (D-3 multi-verifier /
    /// D-2 PPM) — the canary itself does NOT introduce a new verifier.
    fn evaluate(
        &self,
        task: &HeldOutTask,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send;
}

// ---------------------------------------------------------------------------
// Test-only deterministic evaluator
// ---------------------------------------------------------------------------

/// Deterministic evaluator that returns `Ok(true)` for tasks whose
/// `task_id` contains the marker string. Used by the unit / smoke tests
/// to drive `run_canary` without a live verifier. Public-`pub(crate)`
/// so the observable-effect smoke test in
/// `tests/canary_observable_smoke.rs` can construct it.
pub struct MarkerEvaluator {
    /// Substring search — tasks whose `task_id.as_str()` contains
    /// `pass_marker` count as pass.
    pub pass_marker: String,
}

impl CanaryEvaluator for MarkerEvaluator {
    fn evaluate(
        &self,
        task: &HeldOutTask,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send {
        let pass = task.task_id.as_str().contains(self.pass_marker.as_str());
        async move { Ok(pass) }
    }
}

// ---------------------------------------------------------------------------
// run_canary
// ---------------------------------------------------------------------------

/// Per-family bundle handed to [`run_canary`]. One entry per family the
/// caller wants evaluated. Families absent from the slice are simply
/// skipped — the report's `per_family` will not include them.
#[derive(Debug, Clone)]
pub struct CanaryFamilyBundle {
    pub family: CanaryFamily,
    /// Held-out tasks for this family. MUST hold at least
    /// `min_per_family` entries or [`run_canary`] returns
    /// [`CanaryError::InsufficientSamples`].
    pub tasks: Vec<HeldOutTask>,
}

/// Run the cross-domain canary against the supplied per-family
/// bundles. Each bundle is scored twice — once with the `candidate`
/// evaluator, once with the `baseline` evaluator — and a
/// `transfer_score = candidate_pass_rate - baseline_pass_rate` is
/// recorded. The overall decision applies [`DEFAULT_FAIL_BAND`] /
/// [`DEFAULT_MARGINAL_BAND`] over the resulting per-family deltas.
///
/// `ts_ms` is caller-supplied so tests can pin a deterministic
/// timestamp; production callers should source it from the orchestrator
/// `ClockProvider`.
pub async fn run_canary(
    bundles: &[CanaryFamilyBundle],
    candidate: &impl CanaryEvaluator,
    baseline: &impl CanaryEvaluator,
    min_per_family: usize,
    ts_ms: i64,
) -> Result<CanaryReport, CanaryError> {
    if min_per_family == 0 {
        // A zero floor would mean "evaluate zero tasks per family,"
        // which produces a degenerate report. The DEFAULT is 5 and
        // production callers MUST set a non-zero floor — refusing
        // outright keeps the code path honest.
        return Err(CanaryError::InsufficientSamples {
            family: CanaryFamily::CodeFix,
            have: 0,
            need: 1,
        });
    }

    let mut per_family: Vec<FamilyEvalResult> = Vec::with_capacity(bundles.len());

    for bundle in bundles {
        if bundle.tasks.len() < min_per_family {
            return Err(CanaryError::InsufficientSamples {
                family: bundle.family,
                have: bundle.tasks.len(),
                need: min_per_family,
            });
        }
        // Take exactly `min_per_family` tasks from the bundle —
        // truncation is deterministic (first-N) so two runs with
        // identical inputs produce identical reports.
        let slice = &bundle.tasks[..min_per_family];

        let mut cand_pass: u32 = 0;
        let mut base_pass: u32 = 0;
        for task in slice {
            let c = candidate
                .evaluate(task)
                .await
                .map_err(CanaryError::Evaluator)?;
            let b = baseline
                .evaluate(task)
                .await
                .map_err(CanaryError::Evaluator)?;
            if c {
                cand_pass += 1;
            }
            if b {
                base_pass += 1;
            }
        }
        let n = min_per_family as u32;
        let cand_rate = f64::from(cand_pass) / f64::from(n);
        let base_rate = f64::from(base_pass) / f64::from(n);
        let transfer = cand_rate - base_rate;
        per_family.push(FamilyEvalResult {
            family: bundle.family,
            n_evaluated: n,
            pass_count: cand_pass,
            fail_count: n - cand_pass,
            transfer_score: transfer,
            baseline_score: base_rate,
        });
    }

    // Overall transfer = mean of per-family deltas. Guard against an
    // empty bundles slice (caller passed zero families).
    let overall = if per_family.is_empty() {
        0.0
    } else {
        let sum: f64 = per_family.iter().map(|r| r.transfer_score).sum();
        sum / per_family.len() as f64
    };

    let canary_decision = decide(&per_family, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND);

    // Compute the report_id BEFORE assembling the final struct so the
    // blake3 input excludes report_id itself (otherwise the id would
    // depend on itself). Canonical serialization: JSON of (ts_ms,
    // per_family, overall_transfer_score, canary_decision).
    let pre = serde_json::json!({
        "ts_ms": ts_ms,
        "per_family": per_family,
        "overall_transfer_score": overall,
        "canary_decision": canary_decision,
    });
    let pre_bytes = serde_json::to_vec(&pre)?;
    let report_id = blake3::hash(&pre_bytes).to_hex().to_string();

    Ok(CanaryReport {
        report_id,
        ts_ms,
        per_family,
        overall_transfer_score: overall,
        canary_decision,
    })
}

/// Apply the band thresholds to a per-family delta vec. Pure function,
/// public for unit-test isolation.
#[must_use]
pub fn decide(
    per_family: &[FamilyEvalResult],
    fail_band: f64,
    marginal_band: f64,
) -> CanaryDecision {
    let mut dominant: Option<&FamilyEvalResult> = None;
    let mut marginal_seen = false;
    for r in per_family {
        if r.transfer_score <= -fail_band {
            dominant = Some(match dominant {
                None => r,
                Some(prev) => {
                    if r.transfer_score < prev.transfer_score {
                        r
                    } else {
                        prev
                    }
                }
            });
        } else if r.transfer_score <= -marginal_band {
            marginal_seen = true;
        }
    }
    match dominant {
        Some(r) => CanaryDecision::Fail {
            dominant_family: r.family,
        },
        None if marginal_seen => CanaryDecision::MarginalPass,
        None => CanaryDecision::Pass,
    }
}

// ---------------------------------------------------------------------------
// SSB emit
// ---------------------------------------------------------------------------

/// Emit one [`SsbActionType::CanaryRunCompleted`] row + one
/// [`SsbActionType::CanaryFamilyEvaluated`] row per family. Failures to
/// write are logged at `warn` level — they do NOT propagate, matching
/// the held_out / bai pattern: the canary's decision logic must not
/// depend on whether the dashboard can persist the SSB row.
///
/// D-7 closure (SCHEMA_VERSION 114): alongside the existing two variants
/// this also emits per-family observation rows pinned to the three new
/// [`SsbActionType::CrossDomainRegression`] / [`SsbActionType::CanaryFamilyPass`]
/// / [`SsbActionType::CanaryInsufficientData`] variants. ELAI design
/// hypothesis — NanoFlux (arXiv:2509.23252) provides empirical motivation
/// only; no canary mechanism in the paper itself.
///
/// `source_domain` is the candidate's target-family domain (the one where
/// the win was observed); it carries forward into every emitted
/// `CrossDomainRegression` payload so the dashboard can render the source
/// -> regressed-family edge without re-joining tables. Pass
/// [`Domain::Mixed`] when the caller has no opinion (e.g. evaluation runs
/// not tied to a specific target family).
pub async fn emit_canary_run(
    report: &CanaryReport,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
) {
    emit_canary_run_with_source(report, ssb, agent_id, superstep, Domain::Mixed, DEFAULT_FAIL_BAND)
        .await;
}

/// D-7 closure variant of [`emit_canary_run`] that emits the three new
/// SCHEMA 114 observation rows alongside the two shipped variants.
/// `source_domain` + `fail_band` are caller-supplied so the BAI promotion
/// gate can attribute regressions to the candidate's target family and
/// pin the threshold to the loaded `[canary]` toml stanza.
///
/// ELAI design hypothesis — NanoFlux (arXiv:2509.23252) provides empirical
/// motivation only; no canary mechanism in the paper itself.
pub async fn emit_canary_run_with_source(
    report: &CanaryReport,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    source_domain: Domain,
    fail_band: f64,
) {
    let dominant_fail_family = match &report.canary_decision {
        CanaryDecision::Fail { dominant_family } => dominant_family.as_str().to_string(),
        _ => String::new(),
    };
    let run_payload = SsbPayload::CanaryRunCompleted(CanaryRunCompletedPayload {
        report_id: report.report_id.clone(),
        ts_ms: report.ts_ms,
        overall_transfer_score: report.overall_transfer_score,
        decision_kind: report.canary_decision.kind().to_string(),
        dominant_fail_family,
    });
    if let Err(e) = ssb
        .write_payload(agent_id, superstep, run_payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            report_id = %report.report_id,
            "canary::emit_canary_run: CanaryRunCompleted write failed"
        );
    }
    for fam in &report.per_family {
        let payload = SsbPayload::CanaryFamilyEvaluated(CanaryFamilyEvaluatedPayload {
            report_id: report.report_id.clone(),
            family: fam.family.as_str().to_string(),
            n_evaluated: fam.n_evaluated,
            pass_count: fam.pass_count,
            fail_count: fam.fail_count,
            transfer_score: fam.transfer_score,
            baseline_score: fam.baseline_score,
        });
        if let Err(e) = ssb
            .write_payload(agent_id, superstep, payload, Vec::new())
            .await
        {
            tracing::warn!(
                error = %e,
                report_id = %report.report_id,
                family = fam.family.as_str(),
                "canary::emit_canary_run: CanaryFamilyEvaluated write failed"
            );
        }

        // D-7 closure SCHEMA 114: emit the per-family observation row.
        // Exactly one of the three new variants fires per family.
        emit_per_family_observation(
            ssb,
            agent_id,
            superstep,
            &report.report_id,
            fam,
            source_domain,
            fail_band,
            report.ts_ms,
        )
        .await;
    }
}

/// D-7 closure (SCHEMA_VERSION 114): emit exactly one of the three new
/// per-family observation rows for a single [`FamilyEvalResult`]:
///   - [`SsbActionType::CrossDomainRegression`] when transfer_score is
///     below `-fail_band` (the family regressed beyond the band).
///   - [`SsbActionType::CanaryInsufficientData`] when `n_evaluated <
///     min_required` (the family was under-sampled).
///   - [`SsbActionType::CanaryFamilyPass`] otherwise (clean pass at or
///     above the marginal-band tolerance).
///
/// `min_required` is the in-code default [`DEFAULT_MIN_PER_FAMILY`]; for
/// non-default deployments the caller's toml-loaded
/// `CanaryRuntimeConfig::min_per_family` should be used instead. This
/// helper is `async` so the SSB write surface stays uniform with
/// [`emit_canary_run`].
async fn emit_per_family_observation(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    report_id: &str,
    fam: &FamilyEvalResult,
    source_domain: Domain,
    fail_band: f64,
    ts_ms: i64,
) {
    let min_required = DEFAULT_MIN_PER_FAMILY as u32;
    if fam.n_evaluated < min_required {
        let payload = SsbPayload::CanaryInsufficientData(CanaryInsufficientDataPayload {
            family: fam.family.as_str().to_string(),
            sample_count: fam.n_evaluated,
            min_required,
            ts_ms,
        });
        if let Err(e) = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await {
            tracing::warn!(
                error = %e,
                report_id = report_id,
                family = fam.family.as_str(),
                "canary::emit_canary_run: CanaryInsufficientData write failed"
            );
        }
        return;
    }

    if fam.transfer_score <= -fail_band {
        // Regression beyond the fail-band — emit CrossDomainRegression with
        // the absolute magnitude in percent.
        let delta_pct = (-fam.transfer_score) * 100.0;
        let threshold_pct = fail_band * 100.0;
        let payload = SsbPayload::CrossDomainRegression(CrossDomainRegressionPayload {
            source_domain,
            regressed_domain: fam.family.domain(),
            delta_pct,
            threshold_pct,
            ts_ms,
        });
        if let Err(e) = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await {
            tracing::warn!(
                error = %e,
                report_id = report_id,
                family = fam.family.as_str(),
                "canary::emit_canary_run: CrossDomainRegression write failed"
            );
        }
        return;
    }

    // Clean pass — candidate met or beat baseline (transfer_score above the
    // negative fail-band). Emit CanaryFamilyPass with the candidate's
    // pass-rate in percent.
    let candidate_rate = if fam.n_evaluated == 0 {
        0.0
    } else {
        f64::from(fam.pass_count) / f64::from(fam.n_evaluated)
    };
    let score_pct = candidate_rate * 100.0;
    let payload = SsbPayload::CanaryFamilyPass(CanaryFamilyPassPayload {
        family: fam.family.as_str().to_string(),
        score_pct,
        sample_count: fam.n_evaluated,
        ts_ms,
    });
    if let Err(e) = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await {
        tracing::warn!(
            error = %e,
            report_id = report_id,
            family = fam.family.as_str(),
            "canary::emit_canary_run: CanaryFamilyPass write failed"
        );
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::held_out::{build_task, ExpectedOutcome, TaskSource, DEFAULT_NGRAM_SIZE};
    use crate::ssb::SsbReader;

    fn task_for_family(family: CanaryFamily, seed: u64, pass_marker: bool) -> HeldOutTask {
        // Encode the pass-marker into the body so MarkerEvaluator's
        // `task_id.contains(marker)` check on the encoded body works.
        // We piggyback on the body since MarkerEvaluator inspects
        // task_id; here we override task_id after construction.
        let body = format!(
            "canary family {} seed {} body padding alpha beta gamma delta epsilon",
            family.as_str(),
            seed
        );
        let outcome = match family {
            CanaryFamily::CodeFix => ExpectedOutcome::Code("ok".into()),
            CanaryFamily::MathProof => ExpectedOutcome::Math("ok".into()),
            CanaryFamily::ProseReason => ExpectedOutcome::Prose("ok".into()),
            CanaryFamily::PlanningDecomposition => ExpectedOutcome::Prose("ok".into()),
            CanaryFamily::ToolUse => ExpectedOutcome::Prose("ok".into()),
        };
        let mut task = build_task(
            body,
            TaskSource::Synthetic { seed },
            outcome,
            1_000 + seed as i64,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        // Embed the marker token in task_id so MarkerEvaluator's
        // contains() check fires deterministically.
        if pass_marker {
            task.task_id = crate::held_out::TaskId(format!("pass-{}", task.task_id.0));
        } else {
            task.task_id = crate::held_out::TaskId(format!("fail-{}", task.task_id.0));
        }
        task
    }

    fn bundle(family: CanaryFamily, pass_count: u32, total: u32) -> CanaryFamilyBundle {
        let mut tasks = Vec::with_capacity(total as usize);
        for i in 0..total {
            tasks.push(task_for_family(family, u64::from(i), i < pass_count));
        }
        CanaryFamilyBundle { family, tasks }
    }

    #[test]
    fn family_domain_mapping_is_bijection() {
        // Every family maps to a distinct Domain; the inverse is
        // partial (Mixed has no family) which is fine — the bijection
        // is family -> domain, not domain -> family.
        let mut seen = std::collections::HashSet::new();
        for fam in CanaryFamily::all() {
            let d = fam.domain();
            assert!(seen.insert(d), "domain collision for family {fam:?}");
        }
        assert_eq!(seen.len(), CANARY_FAMILY_VARIANT_COUNT);
    }

    #[test]
    fn family_as_str_round_trips_label() {
        assert_eq!(CanaryFamily::CodeFix.as_str(), "code_fix");
        assert_eq!(CanaryFamily::MathProof.as_str(), "math_proof");
        assert_eq!(CanaryFamily::ProseReason.as_str(), "prose_reason");
        assert_eq!(
            CanaryFamily::PlanningDecomposition.as_str(),
            "planning_decomposition"
        );
        assert_eq!(CanaryFamily::ToolUse.as_str(), "tool_use");
    }

    #[test]
    fn decide_pass_when_all_within_band() {
        let per_family = vec![
            FamilyEvalResult {
                family: CanaryFamily::CodeFix,
                n_evaluated: 5,
                pass_count: 4,
                fail_count: 1,
                transfer_score: 0.10,
                baseline_score: 0.70,
            },
            FamilyEvalResult {
                family: CanaryFamily::MathProof,
                n_evaluated: 5,
                pass_count: 3,
                fail_count: 2,
                transfer_score: 0.00,
                baseline_score: 0.60,
            },
        ];
        let d = decide(&per_family, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND);
        assert!(matches!(d, CanaryDecision::Pass));
    }

    #[test]
    fn decide_marginal_when_one_family_in_band() {
        // -0.03 is below the marginal band (0.025) but above the fail
        // band (0.05) — MarginalPass.
        let per_family = vec![
            FamilyEvalResult {
                family: CanaryFamily::CodeFix,
                n_evaluated: 5,
                pass_count: 5,
                fail_count: 0,
                transfer_score: 0.20,
                baseline_score: 0.60,
            },
            FamilyEvalResult {
                family: CanaryFamily::ProseReason,
                n_evaluated: 5,
                pass_count: 2,
                fail_count: 3,
                transfer_score: -0.03,
                baseline_score: 0.70,
            },
        ];
        let d = decide(&per_family, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND);
        assert!(matches!(d, CanaryDecision::MarginalPass));
    }

    #[test]
    fn decide_fail_when_one_family_below_fail_band() {
        let per_family = vec![
            FamilyEvalResult {
                family: CanaryFamily::CodeFix,
                n_evaluated: 5,
                pass_count: 5,
                fail_count: 0,
                transfer_score: 0.20,
                baseline_score: 0.60,
            },
            FamilyEvalResult {
                family: CanaryFamily::ProseReason,
                n_evaluated: 5,
                pass_count: 1,
                fail_count: 4,
                transfer_score: -0.08,
                baseline_score: 0.70,
            },
        ];
        let d = decide(&per_family, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND);
        match d {
            CanaryDecision::Fail { dominant_family } => {
                assert_eq!(dominant_family, CanaryFamily::ProseReason);
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn decide_fail_picks_largest_magnitude_regressor() {
        let per_family = vec![
            FamilyEvalResult {
                family: CanaryFamily::CodeFix,
                n_evaluated: 5,
                pass_count: 2,
                fail_count: 3,
                transfer_score: -0.10,
                baseline_score: 0.80,
            },
            FamilyEvalResult {
                family: CanaryFamily::ProseReason,
                n_evaluated: 5,
                pass_count: 1,
                fail_count: 4,
                transfer_score: -0.30,
                baseline_score: 0.70,
            },
        ];
        let d = decide(&per_family, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND);
        match d {
            CanaryDecision::Fail { dominant_family } => {
                assert_eq!(dominant_family, CanaryFamily::ProseReason);
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_canary_insufficient_samples_errors() {
        let bundles = vec![bundle(CanaryFamily::CodeFix, 3, 4)];
        let cand = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let base = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let err = run_canary(&bundles, &cand, &base, 5, 1_700_000_000_000)
            .await
            .expect_err("must error when bundle short of min_per_family");
        match err {
            CanaryError::InsufficientSamples {
                family,
                have,
                need,
            } => {
                assert_eq!(family, CanaryFamily::CodeFix);
                assert_eq!(have, 4);
                assert_eq!(need, 5);
            }
            other => panic!("expected InsufficientSamples, got {other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_canary_pass_path() {
        // Candidate passes everything; baseline passes nothing. Every
        // family delta = +1.0, overall = +1.0 — Pass.
        let bundles: Vec<CanaryFamilyBundle> = CanaryFamily::all()
            .iter()
            .map(|&f| bundle(f, 5, 5))
            .collect();
        let candidate = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let baseline = MarkerEvaluator {
            pass_marker: "never-match-this-token-xyzzy".into(),
        };
        let report = run_canary(&bundles, &candidate, &baseline, 5, 1_700_000_000_000)
            .await
            .expect("run");
        assert_eq!(report.per_family.len(), CANARY_FAMILY_VARIANT_COUNT);
        for r in &report.per_family {
            assert_eq!(r.n_evaluated, 5);
            assert_eq!(r.pass_count, 5);
            assert!((r.transfer_score - 1.0).abs() < 1e-9);
        }
        assert!((report.overall_transfer_score - 1.0).abs() < 1e-9);
        assert!(matches!(report.canary_decision, CanaryDecision::Pass));
        assert_eq!(report.report_id.len(), 64);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_canary_fail_path_carries_dominant_family() {
        // Code-fix: candidate +10%; baseline 5/5 (rate 1.0), candidate
        // 5/5 (rate 1.0) → delta 0.0. To stage a regression on
        // ProseReason: bundle 5 tasks all `fail-`-prefixed for the
        // candidate but `pass-` for the baseline. We can't do that with
        // a single MarkerEvaluator pair, so we construct two separate
        // evaluators with disjoint markers.
        let mut bundles: Vec<CanaryFamilyBundle> = Vec::new();
        for fam in CanaryFamily::all() {
            // 5 tasks per family; 4 pass-marker, 1 fail-marker for
            // family != ProseReason. For ProseReason, all 5 are
            // fail-marker.
            let pass_count = if fam == CanaryFamily::ProseReason {
                0
            } else {
                4
            };
            bundles.push(bundle(fam, pass_count, 5));
        }
        // Candidate respects pass-marker; baseline marks everything as
        // pass — so for ProseReason, candidate gets 0/5 vs baseline
        // 5/5 = -1.0 delta. Fail.
        let candidate = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let baseline = MarkerEvaluator {
            pass_marker: "-".into(), // dash appears in every UUID
        };
        let report = run_canary(&bundles, &candidate, &baseline, 5, 1_700_000_000_000)
            .await
            .expect("run");
        match &report.canary_decision {
            CanaryDecision::Fail { dominant_family } => {
                assert_eq!(*dominant_family, CanaryFamily::ProseReason);
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_canary_marker_truncates_to_min_per_family() {
        // Pass 10 tasks per bundle but min_per_family=5. n_evaluated
        // must be exactly 5 — confirms deterministic first-N
        // truncation.
        let bundles: Vec<CanaryFamilyBundle> = CanaryFamily::all()
            .iter()
            .map(|&f| bundle(f, 10, 10))
            .collect();
        let cand = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let base = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let report = run_canary(&bundles, &cand, &base, 5, 1_700_000_000_000)
            .await
            .expect("run");
        for r in &report.per_family {
            assert_eq!(r.n_evaluated, 5);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_canary_run_writes_run_plus_per_family_rows() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let cfg = crate::ssb::SsbConfig {
            db_path: dir.path().join("ssb_canary.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        };
        let clock: std::sync::Arc<dyn elai_core::ClockProvider> =
            std::sync::Arc::new(elai_core::clock::SeededClock::new());
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));

        let bundles: Vec<CanaryFamilyBundle> = CanaryFamily::all()
            .iter()
            .map(|&f| bundle(f, 5, 5))
            .collect();
        let cand = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let base = MarkerEvaluator {
            pass_marker: "pass".into(),
        };
        let report = run_canary(&bundles, &cand, &base, 5, 1_700_000_000_000)
            .await
            .expect("run");
        emit_canary_run(&report, &ssb, "test", 0).await;

        let run_rows = ssb
            .query_by_type(crate::ssb::SsbActionType::CanaryRunCompleted)
            .expect("query run rows");
        assert_eq!(run_rows.len(), 1);
        let fam_rows = ssb
            .query_by_type(crate::ssb::SsbActionType::CanaryFamilyEvaluated)
            .expect("query family rows");
        assert_eq!(fam_rows.len(), CANARY_FAMILY_VARIANT_COUNT);
    }
}
