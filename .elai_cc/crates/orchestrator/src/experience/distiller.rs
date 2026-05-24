//! C-49 — Heuristic prompt-fragment distiller.
//!
//! `distill_fragments` projects the per-skill trace bin from
//! [`ExperienceCorpus`] into a small ranked list of
//! [`PromptFragment`]s. The projection is HEURISTIC counting:
//!
//!   1. iterate every `SuccessTrace` whose `audit_pass == true`
//!   2. count fragment-text occurrences across traces
//!   3. compute empirical success rate per fragment
//!   4. retain fragments with `freq >= min_freq` AND
//!      `success_rate >= min_success_rate`
//!   5. sort by `rank_score = freq * success_rate` descending,
//!      tie-break by `ts_ms_last_used` descending
//!
//! Determinism: the underlying [`std::collections::BTreeMap`] gives a
//! stable iteration order over fragment texts; the final
//! `partial_cmp` sort is stable in `slice::sort_by`. No RNG.
//!
//! PAPER GROUNDING vs THIS IMPLEMENTATION (arXiv:2605.14443):
//!   - The paper trains a small policy `pi_theta` via GRPO + KL anchor
//!     to a reference policy. THIS module does NOT train anything; the
//!     RL-trained prompter is RUNPOD-PENDING (Algorithm 1, sec 4.2.3).
//!   - The paper's Eq. 5 buffer-insertion threshold
//!     `r_t^(i) >= r_max - epsilon` requires a scalar reward `r`. We
//!     SIMPLIFY to "audit_pass = true" — a binary signal — because
//!     ELAI's existing audit infrastructure produces a binary verdict.
//!   - The paper's Section 5.3 Web of Lies row uses NO buffer (buffer
//!     hurts on that task). We expose `min_freq = 1` as the boundary
//!     case where the distiller becomes a near-passthrough; callers
//!     who do not want fragment biasing simply skip
//!     `lookup_distilled_fragments`.

use std::collections::BTreeMap;

use super::corpus::ExperienceCorpus;
use super::fragment::PromptFragment;

/// Default minimum occurrence count before a fragment is emitted.
/// Two ensures we never emit single-trace coincidences as
/// "distilled wisdom" — the smallest meaningful sample for cross-trace
/// generalisation.
pub const DEFAULT_MIN_FREQ: u32 = 2;

/// Default minimum empirical success rate before a fragment is emitted.
/// 0.6 leaves headroom over chance for a binary audit signal while not
/// requiring perfection (paper-grounded only in spirit — the original
/// `epsilon` threshold operates on a continuous reward; this is a
/// heuristic floor).
pub const DEFAULT_MIN_SUCCESS_RATE: f32 = 0.6;

/// Tunable thresholds for [`distill_fragments`].
///
/// The defaults are exposed as module-level constants so callers can
/// reach them without constructing a config; the struct form is for
/// callers that want to override.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistillationConfig {
    /// Minimum number of distinct successful traces the fragment must
    /// appear in.
    pub min_freq: u32,
    /// Minimum empirical success rate
    /// (`successes / (successes + failures_observed_in_corpus)`).
    pub min_success_rate: f32,
    /// Hard cap on the number of fragments returned. Defaults to 32 —
    /// enough headroom for a prompt-scaffold injection budget of
    /// ~500 tokens at ~15-30 tokens per fragment without surfacing the
    /// long tail.
    pub max_emit: usize,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            min_freq: DEFAULT_MIN_FREQ,
            min_success_rate: DEFAULT_MIN_SUCCESS_RATE,
            max_emit: 32,
        }
    }
}

/// Distill fragments from the corpus for one skill.
///
/// Returns an empty vec when the skill has no traces (greenfield),
/// when every trace has `audit_pass = false`, or when no fragment
/// crosses the `min_freq` + `min_success_rate` floor. Never panics.
///
/// Algorithm (heuristic; paper analogue documented per-step):
///   1. Collect per-fragment counters: `(freq, last_ts_ms)` keyed by
///      fragment text. `BTreeMap` for determinism.
///   2. For each fragment in any retained trace, increment `freq` and
///      max-merge `last_ts_ms`. Because we filtered on
///      `audit_pass = true`, every observation is a successful
///      observation, so empirical success rate degenerates to 1.0 (we
///      retain the parameter for the future contrastive extension
///      that will track failure observations too).
///   3. Filter by `freq >= cfg.min_freq` AND
///      `success_rate >= cfg.min_success_rate`.
///   4. Sort by `rank_score = freq * success_rate` descending; ties
///      broken by `ts_ms_last_used` descending. Truncate to
///      `cfg.max_emit`.
pub fn distill_fragments(
    corpus: &ExperienceCorpus,
    skill_name: &str,
    min_freq: u32,
    min_success_rate: f32,
) -> Vec<PromptFragment> {
    let cfg = DistillationConfig {
        min_freq,
        min_success_rate,
        ..DistillationConfig::default()
    };
    distill_with_config(corpus, skill_name, cfg)
}

/// Same as [`distill_fragments`] but exposes the full
/// [`DistillationConfig`] (max_emit cap included).
pub fn distill_with_config(
    corpus: &ExperienceCorpus,
    skill_name: &str,
    cfg: DistillationConfig,
) -> Vec<PromptFragment> {
    let bin = corpus.traces_for(skill_name);
    if bin.is_empty() {
        return Vec::new();
    }

    // (freq, last_ts_ms) per fragment text. BTreeMap for deterministic
    // iteration order matching the rest of the experience module.
    let mut counts: BTreeMap<String, (u32, u64)> = BTreeMap::new();
    for trace in bin.iter() {
        if !trace.audit_pass {
            // Heuristic simplification of paper Eq. 5: only positive
            // observations count toward distilled fragments. Negatives
            // are retained on the trace for future contrastive use.
            continue;
        }
        for frag in &trace.fragments {
            let entry = counts
                .entry(frag.text.clone())
                .or_insert((0, frag.ts_ms_last_used));
            entry.0 = entry.0.saturating_add(1);
            if frag.ts_ms_last_used > entry.1 {
                entry.1 = frag.ts_ms_last_used;
            }
        }
    }

    // Project counters into PromptFragments. Empirical success rate is
    // 1.0 by construction (we only counted positive observations) — the
    // floor parameter is kept on the API for the future contrastive
    // extension; we filter against it here so callers see the same
    // surface today.
    let mut emit: Vec<PromptFragment> = counts
        .into_iter()
        .filter(|(_, (freq, _))| *freq >= cfg.min_freq)
        .map(|(text, (freq, ts))| PromptFragment::new(text, freq, 1.0, ts))
        .filter(|f| f.success_rate >= cfg.min_success_rate)
        .collect();

    // Sort by rank score desc; tie-break by ts_ms_last_used desc.
    emit.sort_by(|a, b| {
        let by_score = b
            .rank_score()
            .partial_cmp(&a.rank_score())
            .unwrap_or(std::cmp::Ordering::Equal);
        if by_score == std::cmp::Ordering::Equal {
            b.ts_ms_last_used.cmp(&a.ts_ms_last_used)
        } else {
            by_score
        }
    });
    if emit.len() > cfg.max_emit {
        emit.truncate(cfg.max_emit);
    }
    emit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experience::corpus::ExperienceCorpus;
    use crate::experience::trace::SuccessTrace;

    fn frag(text: &str, ts: u64) -> PromptFragment {
        PromptFragment::new(text.into(), 1, 1.0, ts)
    }

    fn trace(skill: &str, ts: u64, audit_pass: bool, fragments: Vec<PromptFragment>) -> SuccessTrace {
        SuccessTrace::new(skill.into(), 0, 0, audit_pass, ts, fragments)
    }

    #[test]
    fn empty_corpus_yields_empty_vec() {
        let c = ExperienceCorpus::new();
        let out = distill_fragments(&c, "skill_x", 1, 0.0);
        assert!(out.is_empty(), "greenfield must produce empty output");
    }

    #[test]
    fn single_observation_below_min_freq_is_filtered() {
        let mut c = ExperienceCorpus::new();
        c.record_trace(
            "skill_x",
            trace("skill_x", 1, true, vec![frag("once", 1)]),
        )
        .expect("rec");
        let out = distill_fragments(&c, "skill_x", DEFAULT_MIN_FREQ, 0.0);
        assert!(out.is_empty(), "min_freq=2 must drop single observations");
    }

    #[test]
    fn repeated_observation_passes_filter_and_ranks() {
        let mut c = ExperienceCorpus::new();
        // "common" appears in 3 traces, "rare" in 1.
        c.record_trace(
            "skill_x",
            trace(
                "skill_x",
                10,
                true,
                vec![frag("common", 10), frag("rare", 10)],
            ),
        )
        .expect("rec");
        c.record_trace(
            "skill_x",
            trace("skill_x", 20, true, vec![frag("common", 20)]),
        )
        .expect("rec");
        c.record_trace(
            "skill_x",
            trace("skill_x", 30, true, vec![frag("common", 30)]),
        )
        .expect("rec");

        let out = distill_fragments(&c, "skill_x", 2, 0.0);
        assert_eq!(out.len(), 1, "only 'common' meets min_freq=2");
        assert_eq!(out[0].text, "common");
        assert_eq!(out[0].freq_used, 3);
        assert_eq!(out[0].ts_ms_last_used, 30);
    }

    #[test]
    fn audit_pass_false_excluded_from_counts() {
        let mut c = ExperienceCorpus::new();
        c.record_trace(
            "skill_x",
            trace("skill_x", 10, false, vec![frag("ignored", 10)]),
        )
        .expect("rec");
        c.record_trace(
            "skill_x",
            trace("skill_x", 20, false, vec![frag("ignored", 20)]),
        )
        .expect("rec");
        let out = distill_fragments(&c, "skill_x", 1, 0.0);
        assert!(
            out.is_empty(),
            "negatives must NOT contribute to distilled output (paper Eq. 5 simplification)"
        );
    }

    #[test]
    fn determinism_two_runs_identical_output() {
        let mut c = ExperienceCorpus::new();
        for ts in 1..=5 {
            c.record_trace(
                "skill_x",
                trace(
                    "skill_x",
                    ts,
                    true,
                    vec![frag("alpha", ts), frag("beta", ts)],
                ),
            )
            .expect("rec");
        }
        let a = distill_fragments(&c, "skill_x", 2, 0.0);
        let b = distill_fragments(&c, "skill_x", 2, 0.0);
        assert_eq!(a, b, "distillation must be deterministic");
    }

    #[test]
    fn max_emit_cap_truncates() {
        let mut c = ExperienceCorpus::new();
        for i in 0..50 {
            let text = format!("f{i}");
            // Each text appears 2x so all pass min_freq.
            for ts in 0..2 {
                c.record_trace(
                    "skill_x",
                    trace("skill_x", ts, true, vec![frag(&text, ts)]),
                )
                .expect("rec");
            }
        }
        let cfg = DistillationConfig {
            min_freq: 2,
            min_success_rate: 0.0,
            max_emit: 10,
        };
        let out = distill_with_config(&c, "skill_x", cfg);
        assert_eq!(out.len(), 10, "max_emit cap enforced");
    }
}
