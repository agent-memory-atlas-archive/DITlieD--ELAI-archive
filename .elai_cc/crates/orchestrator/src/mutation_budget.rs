//! I-20 — budgeted mutation scheduler (Track 15 KEYSTONE).
//!
//! EXTENSION on the shipped C-28 substrate (`mutation_testing.rs`
//! `MutationRunner` + `EquivalentMutantCache`). Closes the three IC-14 gaps
//! that make the C-28 mutation gate too expensive to live inside the audit
//! loop:
//!
//!   1. **Diff-cone selection** — only mutate functions reachable from the
//!      changed surface (`knowledge::symbol_graph::traversal::compute_impact_radius`),
//!      not the whole AST.
//!   2. **Sampled scoring with a Wilson-score confidence interval** — sample
//!      mutants without replacement, recompute the CI after each batch, and
//!      early-stop in BOTH directions (pass-early when the CI lower bound
//!      already clears the threshold; fail-early when the CI upper bound
//!      already fails it).
//!   3. **Survivor directives** — surviving mutants become structured
//!      [`SurvivorDirective`] rows ("tests do not distinguish > from >= at
//!      x.rs:412") fed to the generator (the retry loop is OUT of I-20 scope).
//!
//! ID-5 (LOAD-BEARING): the kill-rate gate threshold is a MEASURED quantity
//! read from `metrics/mutation_baseline.toml`, never a paper constant. Paper
//! numbers (literature median ~80%) are `[DESIGN]` comments only.
//!
//! Production wiring (D-030 / R-8):
//!   * cone-select runs inside `deterministic_auditor::DeterministicAuditor::check_mutation`
//!     BEFORE the `RustMutationRunner` / `PythonMutationRunner` `.run` calls.
//!   * `MutationBudgetedRun` SSB rows are emitted from
//!     `deterministic_audit_gate::DefaultDeterministicAuditGate::evaluate`
//!     (the writer-owning production seam), mirroring `emit_mutation_survivors`.
//!
//! Anti-staleness (CLAUDE.md): blake3 only (never blake2/sha2);
//! no `.unwrap()` / `.expect()` outside `#[cfg(test)]`; named constants
//! (no magic numbers); `BTreeSet`/sorted for determinism (GD-2).

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::mutation_testing::{EquivalentMutantCache, MutantRecord, MutationError, MutationResult};

// ---------------------------------------------------------------------------
// Constants (no magic numbers — ID-5 / stub_ban)
// ---------------------------------------------------------------------------

/// z-score for a 95% two-sided normal interval, used as the `z` term in the
/// Wilson score interval. 1.96 is the standard normal 0.975 quantile.
pub const Z_SCORE_95: f64 = 1.96;

/// Adaptive-sampler batch size: number of mutants drawn per CI recompute.
/// Small enough that early-stop can fire well before the cone is exhausted,
/// large enough to avoid a CI recompute on every single mutant.
pub const SAMPLER_BATCH_SIZE: usize = 8;

/// CI half-width below which sampling stops early ("the interval is tight
/// enough"). Expressed as a kill-rate proportion (0.0..=1.0).
pub const CI_WIDTH_STOP: f64 = 0.06;

/// BFS depth used for the diff-cone reachability traversal. Depth 2 captures
/// the changed function plus its immediate + once-removed callers (the
/// data-flow cone the mutants must defend), clamped by `MAX_IMPACT_DEPTH`
/// inside `compute_impact_radius`.
pub const CONE_DEPTH: u32 = 2;

/// `[DESIGN]` literature-median kill rate (~80%). NEVER wired as a gate
/// threshold (ID-5): the real threshold is read from
/// `metrics/mutation_baseline.toml`. This constant exists ONLY so the
/// baseline-writer has a documented sentinel to fall back to when no
/// measured corpus exists yet, and even then it is recorded TO the toml as a
/// measured-pending row, not consumed as a paper gate.
pub const DESIGN_LITERATURE_KILL_RATE: f64 = 0.80;

/// Project-relative path of the ID-5 baseline artifact.
pub const MUTATION_BASELINE_REL_PATH: &str = "metrics/mutation_baseline.toml";

// ---------------------------------------------------------------------------
// SurvivorDirective — structured counterexample row
// ---------------------------------------------------------------------------

/// A structured directive describing a surviving mutant: the operator the
/// test suite failed to distinguish, the source location, and the surviving
/// mutation. Fed to the generator's retry loop (the retry loop is OUT of
/// I-20 scope — I-20 owns the EMIT, not the consumer).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurvivorDirective {
    /// The mutation operator that survived (e.g. "ReplaceBinaryOperator").
    pub operator: String,
    /// Project-relative source file of the surviving mutant.
    pub file: String,
    /// 1-based source line of the surviving mutant.
    pub line: u32,
    /// Human-readable description of the surviving mutation
    /// ("tests do not distinguish > from >=").
    pub survivor_mutation: String,
}

impl SurvivorDirective {
    /// Build a directive from a surviving [`MutantRecord`]. The
    /// `survivor_mutation` description is derived from the mutant kind +
    /// location so the generator gets a non-empty, actionable counterexample.
    #[must_use]
    pub fn from_record(record: &MutantRecord) -> Self {
        let file = record.file.to_string_lossy().into_owned();
        let survivor_mutation = format!(
            "tests do not distinguish the original from mutation `{}` at {}:{}",
            record.mutant_kind, file, record.line
        );
        Self {
            operator: record.mutant_kind.clone(),
            file,
            line: record.line,
            survivor_mutation,
        }
    }
}

/// Collect a [`SurvivorDirective`] for every `Survived` mutant in a result.
/// Deterministic order (input record order is preserved).
#[must_use]
pub fn survivor_directives(result: &MutationResult) -> Vec<SurvivorDirective> {
    result
        .records
        .iter()
        .filter(|r| r.outcome.is_survived())
        .map(SurvivorDirective::from_record)
        .collect()
}

// ---------------------------------------------------------------------------
// Wilson-score confidence interval
// ---------------------------------------------------------------------------

/// A Wilson-score confidence interval on a kill rate (proportion of caught
/// mutants), plus the point estimate and the sample size it was computed over.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WilsonInterval {
    /// Point estimate of the kill rate (caught / sampled), 0.0..=1.0.
    pub point: f64,
    /// Lower bound of the interval, clamped to 0.0..=1.0.
    pub lower: f64,
    /// Upper bound of the interval, clamped to 0.0..=1.0.
    pub upper: f64,
    /// Number of mutants the interval was computed over.
    pub n: u32,
}

impl WilsonInterval {
    /// Half-width of the interval, `(upper - lower) / 2`.
    #[must_use]
    pub fn half_width(&self) -> f64 {
        (self.upper - self.lower) / 2.0
    }
}

/// Compute the Wilson score interval for a binomial proportion.
///
/// Formula (Wilson 1927, the standard score interval for a proportion):
///
/// ```text
///            p̂ + z²/(2n)  ±  z · sqrt( p̂(1-p̂)/n + z²/(4n²) )
///   bound = ------------------------------------------------------
///                          1 + z²/n
/// ```
///
/// where `p̂ = caught/n` and `z` is [`Z_SCORE_95`]. With `n == 0` the
/// interval is the whole `[0, 1]` range (maximally uncertain) and the point
/// estimate is `0.0`.
#[must_use]
pub fn wilson_interval(caught: u32, sampled: u32) -> WilsonInterval {
    if sampled == 0 {
        return WilsonInterval {
            point: 0.0,
            lower: 0.0,
            upper: 1.0,
            n: 0,
        };
    }
    let n = f64::from(sampled);
    let p_hat = f64::from(caught) / n;
    let z = Z_SCORE_95;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p_hat + z2 / (2.0 * n)) / denom;
    let margin = (z * ((p_hat * (1.0 - p_hat) / n) + z2 / (4.0 * n * n)).sqrt()) / denom;
    let lower = (center - margin).clamp(0.0, 1.0);
    let upper = (center + margin).clamp(0.0, 1.0);
    WilsonInterval {
        point: p_hat,
        lower,
        upper,
        n: sampled,
    }
}

// ---------------------------------------------------------------------------
// Early-stop decision
// ---------------------------------------------------------------------------

/// The adaptive sampler's stop decision after a CI recompute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopDecision {
    /// CI lower bound already clears the threshold — pass-early.
    PassEarly,
    /// CI upper bound already fails the threshold — fail-early.
    FailEarly,
    /// CI half-width is below [`CI_WIDTH_STOP`] — the interval is tight enough.
    Converged,
    /// Keep sampling: the interval still straddles the threshold and is wide.
    Continue,
}

/// Decide whether to stop sampling given the current interval and the
/// (TOML-measured) `threshold` kill rate. Both early-stop directions plus the
/// width-convergence stop are checked, mirroring the I-20 plan's
/// "both directions save budget".
#[must_use]
pub fn early_stop_decision(interval: &WilsonInterval, threshold: f64) -> StopDecision {
    if interval.lower >= threshold {
        StopDecision::PassEarly
    } else if interval.upper < threshold {
        StopDecision::FailEarly
    } else if interval.half_width() < CI_WIDTH_STOP {
        StopDecision::Converged
    } else {
        StopDecision::Continue
    }
}

// ---------------------------------------------------------------------------
// MutationBudgetSummary — the budget run outcome (carried on the audit result)
// ---------------------------------------------------------------------------

/// The fire-point of a budgeted mutation run. Maps 1:1 to
/// `MutationBudgetedRunKind` so the gate can pick the SSB arm without
/// re-deriving it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetVerdict {
    /// Wilson-CI cleared the threshold (pass-early or full).
    CiCleared,
    /// Wilson-CI lower bound already failed the threshold (fail-early).
    CiFailed,
    /// Unchanged (fn, bound-test-set) hash returned from cache: zero runs.
    CacheHit,
}

/// Summary of a budgeted mutation run, carried on
/// `DeterministicAuditResult` so the writer-owning gate can emit the
/// `MutationBudgetedRun` SSB row(s) off the audit hot path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationBudgetSummary {
    /// Pass/fail/cache-hit verdict for the run.
    pub verdict: BudgetVerdict,
    /// Number of mutants actually sampled (0 on a pure cache hit).
    pub n_sampled: u32,
    /// Size of the full diff-cone-selected mutant set.
    pub n_total_cone: u32,
    /// Wilson-CI lower bound (kill rate, 0.0..=1.0).
    pub ci_lower: f64,
    /// Wilson-CI upper bound (kill rate, 0.0..=1.0).
    pub ci_upper: f64,
    /// Threshold the run was scored against (READ FROM the baseline TOML — ID-5).
    pub threshold: f64,
    /// Wall-clock duration of the budgeted run in milliseconds.
    pub time_ms: u64,
    /// Structured survivor directives (one per surviving mutant). The EMIT of
    /// these as `MutationBudgetedRun(SurvivorDirective)` SSB rows is in I-20
    /// scope; the generator retry loop that CONSUMES them is not.
    pub survivors: Vec<SurvivorDirective>,
}

// ---------------------------------------------------------------------------
// MutationBudget — diff-cone selector + sampler + early-stop + directive emit
// ---------------------------------------------------------------------------

/// The budgeted mutation scheduler. Composes (never re-implements) the C-28
/// `MutationRunner` + `EquivalentMutantCache`; this struct owns the
/// diff-cone selection, the Wilson-CI adaptive sampler, the early-stop, and
/// the survivor-directive build.
#[derive(Debug, Clone)]
pub struct MutationBudget {
    /// The TOML-measured kill-rate gate threshold (ID-5). Set via
    /// [`MutationBudget::with_threshold`] from
    /// [`MutationBudget::load_threshold_pct`].
    threshold: f64,
}

impl MutationBudget {
    /// Construct with an explicit (already TOML-loaded) threshold.
    #[must_use]
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// The gate threshold this budget scores against.
    #[must_use]
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Diff-cone selection: filter `changed_files` to the data-flow cone
    /// reachable from each changed file, using the in-process
    /// `compute_impact_radius` BFS over the symbol graph. Deterministic
    /// (`BTreeSet`-backed).
    ///
    /// Degrades to the identity set (all `changed_files`) when no symbol
    /// graph cache is supplied or the graph yields no cone — never panics,
    /// never fabricates a cone. The reachability result is the set of files
    /// whose mutants must be defended by the changed surface.
    #[must_use]
    pub fn select_cone(
        cache: Option<&mut elai_knowledge::symbol_graph::cache::SymbolGraphCache>,
        changed_files: &[PathBuf],
    ) -> Vec<PathBuf> {
        let Some(cache) = cache else {
            return changed_files.to_vec();
        };
        let mut cone: BTreeSet<String> = BTreeSet::new();
        for f in changed_files {
            let target = f.to_string_lossy();
            match elai_knowledge::symbol_graph::traversal::compute_impact_radius(
                cache, &target, CONE_DEPTH,
            ) {
                Ok(impact) => {
                    for file in impact.resolved_files {
                        cone.insert(file);
                    }
                    // The seed file itself is always part of its own cone.
                    cone.insert(target.into_owned());
                }
                Err(_) => {
                    // Reachability failure for this file degrades to identity
                    // for that file (keep it in scope; never drop coverage).
                    cone.insert(target.into_owned());
                }
            }
        }
        if cone.is_empty() {
            return changed_files.to_vec();
        }
        cone.into_iter().map(PathBuf::from).collect()
    }

    /// Adaptive Wilson-CI sampler WITHOUT replacement over a cone-selected
    /// mutant set. Samples in [`SAMPLER_BATCH_SIZE`] batches, recomputes the
    /// Wilson interval after each batch, and early-stops in both directions
    /// (pass-early / fail-early) or on width-convergence.
    ///
    /// `mutants` is the already-cone-selected, deterministically-ordered set
    /// of outcomes (true = caught/killed, false = survived). Returns the
    /// interval at the stop point plus the verdict.
    ///
    /// This is the pure scoring core; the production path feeds it the
    /// per-mutant outcomes the `MutationRunner` produced over the cone.
    #[must_use]
    pub fn sample_with_ci(&self, mutants: &[bool]) -> (WilsonInterval, BudgetVerdict) {
        let mut caught: u32 = 0;
        let mut sampled: u32 = 0;
        let mut interval = wilson_interval(0, 0);
        let mut decision = StopDecision::Continue;

        for chunk in mutants.chunks(SAMPLER_BATCH_SIZE) {
            for &killed in chunk {
                sampled = sampled.saturating_add(1);
                if killed {
                    caught = caught.saturating_add(1);
                }
            }
            interval = wilson_interval(caught, sampled);
            decision = early_stop_decision(&interval, self.threshold);
            if !matches!(decision, StopDecision::Continue) {
                break;
            }
        }

        // Final decision once the cone is exhausted (or an early-stop fired).
        let verdict = match decision {
            StopDecision::FailEarly => BudgetVerdict::CiFailed,
            StopDecision::PassEarly | StopDecision::Converged => BudgetVerdict::CiCleared,
            // Cone exhausted without an early-stop: the point estimate
            // against the threshold is the verdict (CI straddled but we ran
            // out of mutants to narrow it).
            StopDecision::Continue => {
                if interval.lower >= self.threshold || interval.point >= self.threshold {
                    BudgetVerdict::CiCleared
                } else {
                    BudgetVerdict::CiFailed
                }
            }
        };
        (interval, verdict)
    }

    /// Build the full [`MutationBudgetSummary`] from a cone-selected
    /// [`MutationResult`], scoring with the Wilson-CI sampler and collecting
    /// survivor directives. `n_total_cone` is the size of the cone the result
    /// was produced over; `time_ms` is the measured budget-run duration.
    #[must_use]
    pub fn summarize(
        &self,
        result: &MutationResult,
        n_total_cone: u32,
        time_ms: u64,
    ) -> MutationBudgetSummary {
        // Per-mutant kill outcomes over the eligible (run, not skipped) set.
        let outcomes: Vec<bool> = result
            .records
            .iter()
            .filter(|r| !r.outcome.is_skipped())
            .map(|r| r.outcome.is_caught())
            .collect();
        let (interval, verdict) = self.sample_with_ci(&outcomes);
        MutationBudgetSummary {
            verdict,
            n_sampled: interval.n,
            n_total_cone,
            ci_lower: interval.lower,
            ci_upper: interval.upper,
            threshold: self.threshold,
            time_ms,
            survivors: survivor_directives(result),
        }
    }

    /// A [`MutationBudgetSummary`] for a pure cache hit: zero mutant runs,
    /// no CI, no survivors. The cone-cache (unchanged fn + bound-test-set
    /// hash) returned without re-running any mutant.
    // WIRE-DARK[I-20-cone-cache]: the per-mutant cone-cache short-circuit loop in check_mutation (the production consumer of cache_hit_summary / is_cone_cache_hit / compute_test_set_hash) is a Tier-2 follow-on; the EMIT path (CacheHit SSB arm) is wired + tested. Backing row: stubs.md will_resolve_in=I-20-cone-cache-consumer.
    #[must_use]
    pub fn cache_hit_summary(&self, n_total_cone: u32) -> MutationBudgetSummary {
        MutationBudgetSummary {
            verdict: BudgetVerdict::CacheHit,
            n_sampled: 0,
            n_total_cone,
            ci_lower: 0.0,
            ci_upper: 0.0,
            threshold: self.threshold,
            time_ms: 0,
            survivors: Vec::new(),
        }
    }

    // -- ID-5: threshold read from metrics/mutation_baseline.toml --

    /// Load the kill-rate gate threshold from `metrics/mutation_baseline.toml`
    /// under `project_root` (ID-5). The threshold is a MEASURED quantity, never
    /// a paper constant. Returns the parsed `threshold` (0.0..=1.0) on success.
    ///
    /// Errors (file absent, malformed, missing key) propagate so the caller
    /// decides policy (first-deploy writes the baseline). This function NEVER
    /// substitutes a hard-coded paper threshold for a missing file.
    pub fn load_threshold_pct(project_root: &std::path::Path) -> Result<f64, MutationError> {
        let path = project_root.join(MUTATION_BASELINE_REL_PATH);
        let text = std::fs::read_to_string(&path).map_err(|e| {
            MutationError::CacheError(format!(
                "read mutation baseline {}: {}",
                path.display(),
                e
            ))
        })?;
        parse_baseline_threshold(&text)
    }

    /// Write the ID-5 baseline TOML on first deploy IF it does not already
    /// exist, recording the measured `kill_rate` as the gate threshold. Does
    /// nothing when the file is already present (the measured baseline is
    /// immutable once recorded; re-measurement is a deliberate operator op).
    /// Returns `true` when a fresh baseline was written.
    pub fn write_baseline_if_absent(
        project_root: &std::path::Path,
        measured_kill_rate: f64,
    ) -> Result<bool, MutationError> {
        let path = project_root.join(MUTATION_BASELINE_REL_PATH);
        if path.exists() {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                MutationError::CacheError(format!("create metrics dir: {e}"))
            })?;
        }
        let clamped = measured_kill_rate.clamp(0.0, 1.0);
        let body = render_baseline_toml(clamped);
        std::fs::write(&path, body).map_err(|e| {
            MutationError::CacheError(format!(
                "write mutation baseline {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(true)
    }
}

/// Parse the `threshold` key out of a `mutation_baseline.toml` body.
/// Minimal hand-parse (no toml crate dependency added): finds the
/// `threshold = <float>` line under the `[mutation_baseline]` table. Returns
/// an error when the key is absent or unparseable (ID-5: missing = error, NOT
/// a paper-constant fallback).
fn parse_baseline_threshold(text: &str) -> Result<f64, MutationError> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("threshold") {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                let value = value.trim();
                return value.parse::<f64>().map(|v| v.clamp(0.0, 1.0)).map_err(|e| {
                    MutationError::CacheError(format!(
                        "parse mutation baseline threshold {value:?}: {e}"
                    ))
                });
            }
        }
    }
    Err(MutationError::CacheError(
        "mutation baseline TOML has no `threshold` key (ID-5: refusing to \
         substitute a paper constant)"
            .to_owned(),
    ))
}

/// Render the ID-5 baseline TOML body for a measured kill rate.
fn render_baseline_toml(threshold: f64) -> String {
    format!(
        "# ID-5 baseline — ELAI-measured mutation kill-rate gate threshold.\n\
         # Written on first deploy by mutation_budget::write_baseline_if_absent.\n\
         # The threshold is a MEASURED quantity on ELAI's own corpus, NOT a\n\
         # paper constant. Literature median (~{design:.2}) is a [DESIGN]\n\
         # reference only and is never wired as a gate.\n\
         [mutation_baseline]\n\
         threshold = {threshold:.4}\n",
        design = DESIGN_LITERATURE_KILL_RATE,
        threshold = threshold,
    )
}

// ---------------------------------------------------------------------------
// EquivalentMutantCache key extension (IC-14 — EXTEND, do not build a 2nd cache)
// ---------------------------------------------------------------------------

/// I-20 extension of [`EquivalentMutantCache::compute_key`]: fold the
/// bound-test-set hash into the blake3 key so a cache entry is keyed by
/// `(pre_mutation_bytes, mutant_kind, line, bound_test_set_hash)`.
///
/// Backwards-compatible namespacing: a key with a different (or absent)
/// test-set hash simply MISSES the old 3-input keys and re-runs once — it
/// never errors, never collides with the C-28 key space (the extra blake3
/// update produces a distinct digest).
///
/// `bound_test_set_hash` is the blake3 hex of the bound test set's content
/// (computed by [`compute_test_set_hash`]); folding it in means a test-suite
/// change invalidates the cached mutant outcome (the suite may now kill a
/// previously-surviving mutant).
#[must_use]
pub fn compute_key_with_testset(
    pre_mutation_bytes: &[u8],
    mutant_kind: &str,
    line: u32,
    bound_test_set_hash: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(pre_mutation_bytes);
    hasher.update(mutant_kind.as_bytes());
    hasher.update(&line.to_le_bytes());
    // The I-20 fold: bound-test-set hash as the 4th input. A domain-separator
    // byte keeps the 4-input key space disjoint from any accidental 3-input
    // concatenation collision.
    hasher.update(b"\x00i20-testset\x00");
    hasher.update(bound_test_set_hash.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// blake3 hex digest of a bound test set (the concatenation of its test
/// source bytes). Deterministic; used as the 4th input to
/// [`compute_key_with_testset`].
// WIRE-DARK[I-20-cone-cache]: consumed by the Tier-2 cone-cache short-circuit loop in check_mutation (forward); the key-extension itself is wired + tested. Backing row: stubs.md will_resolve_in=I-20-cone-cache-consumer.
#[must_use]
pub fn compute_test_set_hash(test_sources: &[Vec<u8>]) -> String {
    let mut hasher = blake3::Hasher::new();
    for src in test_sources {
        hasher.update(&(src.len() as u64).to_le_bytes());
        hasher.update(src);
    }
    hasher.finalize().to_hex().to_string()
}

/// Look up a cone-selected mutant in the EXISTING [`EquivalentMutantCache`]
/// under the I-20 test-set-extended key. Returns `Ok(true)` on a cache hit
/// (zero mutant runs needed), `Ok(false)` on a miss. Composes the shipped
/// cache `get` — does NOT introduce a second cache (IC-14).
// WIRE-DARK[I-20-cone-cache]: consumed by the Tier-2 cone-cache short-circuit loop in check_mutation (forward); composes the shipped EquivalentMutantCache::get. Backing row: stubs.md will_resolve_in=I-20-cone-cache-consumer.
pub fn is_cone_cache_hit(
    cache: &EquivalentMutantCache,
    pre_mutation_bytes: &[u8],
    mutant_kind: &str,
    line: u32,
    bound_test_set_hash: &str,
) -> Result<bool, MutationError> {
    let key = compute_key_with_testset(pre_mutation_bytes, mutant_kind, line, bound_test_set_hash);
    Ok(cache.get(&key)?.is_some())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutation_testing::{MutantOutcome, MutantRecord, MutationResult};

    fn rec(kind: &str, line: u32, outcome: MutantOutcome) -> MutantRecord {
        MutantRecord {
            file: PathBuf::from("src/risk.rs"),
            line,
            mutant_kind: kind.to_owned(),
            outcome,
        }
    }

    #[test]
    fn wilson_interval_zero_samples_is_full_range() {
        let ci = wilson_interval(0, 0);
        assert_eq!(ci.lower, 0.0);
        assert_eq!(ci.upper, 1.0);
        assert_eq!(ci.n, 0);
    }

    #[test]
    fn wilson_interval_all_caught_is_high_lower_bound() {
        let ci = wilson_interval(50, 50);
        assert!(ci.lower > 0.9, "50/50 caught should have a high lower bound; got {ci:?}");
        assert!(ci.upper <= 1.0);
        assert!((ci.point - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn wilson_interval_bounds_ordered_and_clamped() {
        let ci = wilson_interval(7, 10);
        assert!(ci.lower <= ci.point && ci.point <= ci.upper);
        assert!(ci.lower >= 0.0 && ci.upper <= 1.0);
    }

    #[test]
    fn early_stop_pass_early_when_lower_clears() {
        // High kill rate, threshold below the lower bound -> pass-early.
        let ci = wilson_interval(60, 60);
        assert_eq!(early_stop_decision(&ci, 0.5), StopDecision::PassEarly);
    }

    #[test]
    fn early_stop_fail_early_when_upper_fails() {
        // Zero kills over many samples -> upper bound well below a 0.8 threshold.
        let ci = wilson_interval(0, 60);
        assert_eq!(early_stop_decision(&ci, 0.8), StopDecision::FailEarly);
    }

    #[test]
    fn sampler_pass_early_stops_before_exhausting() {
        let budget = MutationBudget::with_threshold(0.5);
        // 200 all-caught mutants; pass-early should fire in the first batch or two.
        let mutants = vec![true; 200];
        let (ci, verdict) = budget.sample_with_ci(&mutants);
        assert_eq!(verdict, BudgetVerdict::CiCleared);
        assert!(
            ci.n < 200,
            "pass-early must stop before exhausting all 200 mutants; sampled {}",
            ci.n
        );
    }

    #[test]
    fn sampler_fail_early_stops_before_exhausting() {
        let budget = MutationBudget::with_threshold(0.8);
        let mutants = vec![false; 200];
        let (ci, verdict) = budget.sample_with_ci(&mutants);
        assert_eq!(verdict, BudgetVerdict::CiFailed);
        assert!(ci.n < 200, "fail-early must stop early; sampled {}", ci.n);
    }

    #[test]
    fn survivor_directives_only_for_survived() {
        let result = MutationResult::from_records(vec![
            rec("op_a", 10, MutantOutcome::Caught),
            rec("op_b", 20, MutantOutcome::Survived),
            rec("op_c", 30, MutantOutcome::Skipped("cache_hit".into())),
        ]);
        let dirs = survivor_directives(&result);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].operator, "op_b");
        assert_eq!(dirs[0].line, 20);
        assert!(!dirs[0].survivor_mutation.is_empty());
    }

    #[test]
    fn compute_key_with_testset_differs_from_three_input_key() {
        let three = EquivalentMutantCache::compute_key(b"fn x", "op", 7);
        let four = compute_key_with_testset(b"fn x", "op", 7, "deadbeef");
        assert_ne!(three, four, "4-input key must be disjoint from the 3-input key");
    }

    #[test]
    fn compute_key_with_testset_changes_with_testset() {
        let a = compute_key_with_testset(b"fn x", "op", 7, "aaaa");
        let b = compute_key_with_testset(b"fn x", "op", 7, "bbbb");
        assert_ne!(a, b, "a different bound-test-set hash must change the key");
    }

    #[test]
    fn baseline_round_trips_threshold() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        // First deploy writes the baseline.
        let wrote = MutationBudget::write_baseline_if_absent(root, 0.73).expect("write");
        assert!(wrote, "first write should create the baseline");
        // Second call is a no-op (immutable once recorded).
        let again = MutationBudget::write_baseline_if_absent(root, 0.99).expect("write2");
        assert!(!again, "baseline is immutable once recorded");
        // Threshold reads back the FIRST measured value, not a paper constant.
        let t = MutationBudget::load_threshold_pct(root).expect("load");
        assert!((t - 0.73).abs() < 1e-6, "threshold must read from toml; got {t}");
    }

    #[test]
    fn load_threshold_errors_when_absent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let r = MutationBudget::load_threshold_pct(dir.path());
        assert!(r.is_err(), "missing baseline must error, never paper-fallback");
    }

    #[test]
    fn select_cone_without_cache_is_identity() {
        let files = vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")];
        let cone = MutationBudget::select_cone(None, &files);
        assert_eq!(cone.len(), 2);
    }
}
