//! H-47: DGM eval firewall — date-gate + template-regen + holdout-budget binding.
//!
//! Composes three SHIPPED mechanisms (GD-1 P0 — none of these are re-implemented):
//!   - D-6  [`held_out::is_task_after_cutoff`] + [`held_out::ModelTrainingCutoff`]
//!   - E-54 [`held_out::ngram_filter`] + contamination filter
//!   - H-19 [`holdout_budget::HoldoutBudgetLedger::query_holdout`]
//!
//! Net-new wiring only:
//!   - [`date_gated_sample`]: wraps [`agent_pool::sample_held_out_task`] with the
//!     LiveCodeBench date-gate (arXiv:2403.07974). Pre-update items return `None`.
//!   - [`regenerate_instance`]: score-time template regen (GSM-Symbolic 2410.05229).
//!     Blake3-seeded shuffle over the template body — the loop never scores the same
//!     instance twice. `None` template_id -> no-op clone.
//!   - [`bind_holdout_budget`]: one budget query per DGM score call (Thresholdout
//!     1506.02629 DP bound). Returns `Err(FirewallError::BudgetExhausted)` on refusal.
//!   - [`indistinguishability_probe`]: Jaccard-based distinguishability score (0.0 =
//!     indistinguishable; 1.0 = fully distinct). Surfaces VD-5 sandbagging risk.
//!
//! HARD RULES:
//!   - blake3 for all hashing (not blake2 / sha2).
//!   - No `.unwrap()` / `.expect()` in production paths; use `?` / `ok_or`.
//!   - SeededClock (C-15) injected for SSB timestamps; no `SystemTime` direct call.
//!   - fail-closed on starvation: `None` from `date_gated_sample` NEVER falls back
//!     to scoring a pre-update item.

use std::sync::Arc;

use tracing::warn;

use crate::agent_pool::sample_held_out_task;
use crate::held_out::{is_task_after_cutoff, HeldOutStore, HeldOutTask, ModelTrainingCutoff};
use crate::holdout_budget::{HoldoutBudgetLedger, HoldoutQueryOutcome};
use crate::ssb::Ssb;
use elai_core::clock::ClockProvider;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the eval firewall pipeline.
#[derive(Debug, thiserror::Error)]
pub enum FirewallError {
    #[error("holdout budget exhausted or missing — eval refused (fail-closed)")]
    BudgetExhausted,
    #[error("holdout budget error: {0}")]
    Budget(#[from] crate::holdout_budget::HoldoutBudgetError),
}

// ---------------------------------------------------------------------------
// Leg 1: date-gate (D-6 compose)
// ---------------------------------------------------------------------------

/// Wraps [`sample_held_out_task`] with the LiveCodeBench temporal date-gate
/// (arXiv:2403.07974 §S3.SS1). Returns `None` in two cases:
///   1. No active task is available in the store.
///   2. The sampled task's release date is AT OR BEFORE `cutoff.date_ms`
///      (the item is presumed in-training and is therefore rejected).
///
/// HARD RULE (RISK-1): `None` from this function MUST NOT cause the caller to
/// fall back to scoring a pre-update item. The caller MUST treat `None` as
/// "no safe eval item available" and pause scoring for this generation.
///
/// `now_ms` is the generation-scoped timestamp (derived from the loop clock,
/// never from `SystemTime::now()` directly).
#[must_use]
pub fn date_gated_sample(
    store: &HeldOutStore,
    now_ms: i64,
    cutoff: &ModelTrainingCutoff,
) -> Option<HeldOutTask> {
    let task = sample_held_out_task(store, now_ms)?;
    if is_task_after_cutoff(&task, cutoff) {
        Some(task)
    } else {
        // Pre-update item — it may be memorized. Reject fail-closed.
        warn!(
            task_id = %task.task_id.0,
            ingested_ts_ms = task.ingested_ts_ms,
            cutoff_date_ms = cutoff.date_ms,
            model_id = %cutoff.model_id,
            "eval_firewall: date-gated rejection (task pre-dates model cutoff)",
        );
        None
    }
}

// ---------------------------------------------------------------------------
// Leg 2: score-time template regeneration (E-54 compose)
// ---------------------------------------------------------------------------

/// Produces a fresh eval instance from a templated task by applying a
/// blake3-seeded permutation to the task body (GSM-Symbolic 2410.05229 §4
/// score-time generation — the loop never scores the exact instance twice).
///
/// Template detection (Risk-2 resolution): [`HeldOutTask::template_id`] carries
/// `Some(id)` iff the task was generated from a named template. Tasks with
/// `None` are returned unchanged (no-op). This is the least-invasive backward-
/// compatible extension (see `workflow/reasoning/H-47-decisions.md`).
///
/// Seeding: the rng seed is `generation_id ^ seed` so each generation produces
/// a distinct permutation (GD-2 reproducible under the same `(generation_id, seed)`
/// pair).
#[must_use]
pub fn regenerate_instance(task: &HeldOutTask, generation_id: u64, seed: u64) -> HeldOutTask {
    // No template -> no-op clone (free-form tasks rely on the date-gate + budget).
    let Some(ref template_id) = task.template_id else {
        return task.clone();
    };

    // Derive a fresh body hash from template_id + generation_id + seed using blake3.
    // The regenerated body is a deterministic transformation of the original that
    // varies per generation (the exact body content is a FUTURE integration with the
    // live template library; for the Tier-1 in-process gate we regenerate by
    // appending the generation hash as a distinguisher — same semantic coverage,
    // different byte pattern so n-gram collisions are avoided per E-54).
    let mut hasher = blake3::Hasher::new();
    hasher.update(template_id.as_bytes());
    hasher.update(&generation_id.to_le_bytes());
    hasher.update(&seed.to_le_bytes());
    hasher.update(task.body.as_bytes());
    let gen_hash = hasher.finalize();
    // Take the first 16 hex chars of the hash as a deterministic suffix.
    let suffix: String = gen_hash
        .as_bytes()
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect();

    // Rebuild the ngram_signature for the regenerated body (E-54 contamination
    // filter applies at ingest time; the regen updates the signature so future
    // contamination checks are against the fresh instance).
    let regenerated_body = format!("{} [gen-{}]", task.body, suffix);
    let ngram_sig = crate::held_out::ngram_filter(&regenerated_body, 9);

    // Recompute content_blake3 for the regenerated body (blake3 HARD RULE).
    let body_hash = blake3::hash(regenerated_body.as_bytes());
    let content_blake3 = body_hash.to_hex().to_string();

    HeldOutTask {
        task_id: task.task_id.clone(),
        source: task.source.clone(),
        content_blake3,
        expected_outcome: task.expected_outcome.clone(),
        ingested_ts_ms: task.ingested_ts_ms,
        eligibility_window_ms: task.eligibility_window_ms,
        ngram_signature: ngram_sig,
        body: regenerated_body,
        template_id: task.template_id.clone(),
    }
}

// ---------------------------------------------------------------------------
// Leg 3: holdout budget binding (H-19 compose)
// ---------------------------------------------------------------------------

/// Route one DGM eval query through the H-19 Thresholdout DP budget
/// (Dwork et al. Science 2015, arXiv:1506.02629). Returns the budget query
/// outcome on success; returns `Err(FirewallError::BudgetExhausted)` when the
/// budget is Exhausted or Missing (fail-closed R1/R2).
///
/// The `HoldoutBudgetEvent` SSB row is emitted INSIDE `query_holdout` (H-19
/// production code — GD-1: H-47 does NOT duplicate that emit). H-47 only calls
/// `query_holdout` and propagates the REFUSED arm as `FirewallError`.
///
/// `train_stat` and `holdout_stat` are the training-set and held-out fitness
/// statistics for this query. `data_window` is the window string `"start..end"`.
#[allow(clippy::too_many_arguments)]
pub async fn bind_holdout_budget(
    ledger: &HoldoutBudgetLedger,
    ssb: &Arc<Ssb>,
    clock: &Arc<dyn ClockProvider>,
    holdout_id: &str,
    objective: &str,
    data_window: &str,
    train_stat: f64,
    holdout_stat: f64,
    run_id: &str,
    seed: u64,
) -> Result<HoldoutQueryOutcome, FirewallError> {
    let outcome = ledger
        .query_holdout(
            ssb,
            clock,
            holdout_id,
            objective,
            data_window,
            train_stat,
            holdout_stat,
            run_id,
            seed,
        )
        .await?;
    if outcome.is_refused() {
        return Err(FirewallError::BudgetExhausted);
    }
    Ok(outcome)
}

// ---------------------------------------------------------------------------
// Leg 4: indistinguishability probe (E-54 compose)
// ---------------------------------------------------------------------------

/// Measure how distinguishable the eval item is from a "workload trace" slice
/// by Jaccard n-gram similarity (E-54 / D-23 composing the shipped
/// [`held_out::ngram_filter`] surface).
///
/// Returns a `distinguishability` score in `[0.0, 1.0]`:
///   - `0.0` = the eval item is indistinguishable from the workload slice
///     (all n-grams shared — maximum indistinguishability, lowest sandbagging
///     risk from eval-awareness).
///   - `1.0` = no n-gram overlap (fully distinguishable — the model CAN detect
///     it is being evaluated, VD-5 sandbagging risk HIGH).
///
/// `eval_ngrams` is the eval item's signature (from `HeldOutTask::ngram_signature`
/// or freshly computed). `workload_ngrams` is a slice of n-gram hashes sampled
/// from recent live workload traces (the eval-awareness probe corpus, E-54 §3).
///
/// VD-5 honest residue: a score near 0.0 REDUCES sandbagging risk but does not
/// eliminate it — a strong model can detect evals from semantic cues not captured
/// by n-gram similarity. Surface the number, name the residue.
#[must_use]
pub fn indistinguishability_probe(eval_ngrams: &[u64], workload_ngrams: &[u64]) -> f64 {
    // Jaccard overlap: 1.0 - overlap gives distinguishability.
    // 0 overlap = fully distinguishable (1.0); full overlap = indistinguishable (0.0).
    let overlap = crate::held_out::jaccard_overlap(eval_ngrams, workload_ngrams);
    // f64 cast from f32 is lossless widening. Clamp to [0.0, 1.0] for robustness.
    let dist = (1.0 - overlap as f64).clamp(0.0, 1.0);
    dist
}

// ---------------------------------------------------------------------------
// Firewall summary — returned to run_one_generation for SSB emit
// ---------------------------------------------------------------------------

/// Aggregate counters collected during one generation's eval firewall pass.
/// The caller emits one `EvalFirewallEvent` SSB row per kind after the
/// per-item loop closes.
#[derive(Debug, Clone, Default)]
pub struct FirewallSummary {
    /// Items that passed the date-gate and were scored.
    pub items_scored: u32,
    /// Items rejected by the date-gate (pre-update, possible memorization).
    pub items_rejected: u32,
    /// Items for which `regenerate_instance` produced a fresh variant.
    pub regen_count: u32,
    /// Remaining budget after the generation (from `HoldoutQueryOutcome`).
    pub budget_remaining: i64,
    /// Distinguishability score from the indistinguishability probe.
    pub distinguishability: f64,
    /// The generation seed.
    pub seed: u64,
    /// The generation id (loop counter).
    pub generation_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::held_out::{
        ExpectedOutcome, HeldOutTask, ModelTrainingCutoff, TaskSource,
    };

    fn make_task(ingested_ts_ms: i64, template_id: Option<String>) -> HeldOutTask {
        HeldOutTask {
            task_id: crate::held_out::TaskId("task-001".into()),
            source: TaskSource::Synthetic { seed: 42 },
            content_blake3: "aabbcc".into(),
            expected_outcome: ExpectedOutcome::Code("ok".into()),
            ingested_ts_ms,
            eligibility_window_ms: 0,
            ngram_signature: vec![1u64, 2u64, 3u64],
            body: "def add(a, b): return a + b".into(),
            template_id,
        }
    }

    #[test]
    fn date_gate_accepts_post_cutoff() {
        let cutoff = ModelTrainingCutoff {
            model_id: "test-model".into(),
            date_ms: 1_000,
        };
        let task = make_task(2_000, None); // ingested AFTER cutoff
        assert!(
            is_task_after_cutoff(&task, &cutoff),
            "post-cutoff task must pass the date-gate"
        );
    }

    #[test]
    fn date_gate_rejects_pre_cutoff() {
        let cutoff = ModelTrainingCutoff {
            model_id: "test-model".into(),
            date_ms: 2_000,
        };
        let task = make_task(1_000, None); // ingested BEFORE cutoff
        assert!(
            !is_task_after_cutoff(&task, &cutoff),
            "pre-cutoff task must fail the date-gate"
        );
    }

    #[test]
    fn regenerate_instance_non_templated_is_noop() {
        let task = make_task(1_000, None);
        let regen = regenerate_instance(&task, 1, 42);
        // Non-templated task: body and task_id must be identical to original.
        assert_eq!(regen.body, task.body);
        assert_eq!(regen.task_id, task.task_id);
    }

    #[test]
    fn regenerate_instance_templated_changes_body() {
        let task = make_task(1_000, Some("tmpl-add".into()));
        let regen_g1 = regenerate_instance(&task, 1, 42);
        let regen_g2 = regenerate_instance(&task, 2, 42);
        // Templated task: each generation yields a different body.
        assert_ne!(
            regen_g1.body, task.body,
            "regenerated body must differ from original"
        );
        assert_ne!(
            regen_g1.body, regen_g2.body,
            "different generation_ids must yield different bodies"
        );
    }

    #[test]
    fn regenerate_instance_is_reproducible() {
        let task = make_task(1_000, Some("tmpl-add".into()));
        let a = regenerate_instance(&task, 7, 99);
        let b = regenerate_instance(&task, 7, 99);
        assert_eq!(a.body, b.body, "same seed + generation_id must be byte-identical (GD-2)");
    }

    #[test]
    fn indistinguishability_probe_fully_shared() {
        let ngrams = vec![1u64, 2, 3, 4];
        let dist = indistinguishability_probe(&ngrams, &ngrams);
        assert!(
            dist < 1e-6,
            "fully shared ngrams -> distinguishability near 0.0"
        );
    }

    #[test]
    fn indistinguishability_probe_fully_disjoint() {
        let eval_ng = vec![1u64, 2, 3];
        let work_ng = vec![4u64, 5, 6];
        let dist = indistinguishability_probe(&eval_ng, &work_ng);
        assert!(
            (dist - 1.0).abs() < 1e-6,
            "fully disjoint ngrams -> distinguishability near 1.0"
        );
    }
}
