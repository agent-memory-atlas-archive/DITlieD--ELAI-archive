// cross-lang-skip: rust-py-sidecar
//! H-21 (SCHEMA_VERSION 226): runtime look-ahead / causality DIFFERENTIAL gate.
//!
//! The Tier-1 fail-closed runtime backstop for look-ahead bias (the un-gameable
//! complement to the H-20 static leakage COMPOSITE, R5). It is a PURE perturb-
//! and-diff: given two already-computed feature column-sets — one produced on
//! the FULL data, one on the FUTURE-TRUNCATED data — it diffs them column by
//! column. Any column whose values change when the future is removed PROVES the
//! pipeline read the future (a look-ahead leak) => BLOCK. Columns that diff to
//! zero are causal (backward-looking only) => ALLOW.
//!
//! # Tier boundary (cross-lang-skip rationale)
//! Tier-1 (THIS gate) consumes two column-sets as PLAIN DATA — it runs no
//! pipeline. WHO produces the two column-sets is the DEFERRED Tier-2 Python
//! feature-runner (`run_pipeline_twice`, re-executes a real strategy on full vs
//! future-truncated data). That sidecar is RunPod-pending this run (cloud-only
//! override); no stub is shipped. The `// cross-lang-skip: rust-py-sidecar`
//! header opts this file out of the Rust<->Python pair drift check until the
//! Tier-2 runner ships. The Tier-1 done-gate is exercisable WITHOUT a sidecar
//! via the `elai lookahead-check --demo-*` fixtures.
//!
//! # Method grounding (Freqtrade look-ahead-analysis, verified 2026-06-05)
//! Freqtrade's `freqtrade/optimize/analysis/lookahead.py` runs a strategy twice
//! (full vs time-truncated) and flags any indicator column that differs as a
//! look-ahead bias. H-21 is a sound EXTENSION of that idea, NOT a port:
//! Freqtrade uses exact `DataFrame.compare()` (any bit-difference), which yields
//! cross-hardware FALSE POSITIVES on benign FMA/SIMD reassociation. H-21 instead
//! uses a relative+absolute tolerance band (its OWN choice — TD-5 honest
//! residue, not borrowed from Freqtrade) tight enough to catch a real O(price)
//! leak yet loose enough to absorb last-digit cross-hardware FP noise. The diff
//! is NEVER asserted bitwise-exact across hardware.
//!
//! # FAIL-CLOSED (R1/R2/W-1 — "cannot prove causality" => Block)
//! Three conditions short-circuit to a hard `FutureLeak`/`Unprovable` Block with
//! an emitted SSB row, NEVER a silent pass:
//!   1. the truncated column-set is missing (`None`) or empty,
//!   2. the two sets have mismatched shapes (different row counts),
//!   3. the column SETS mismatch (a column present in one but not the other).
//! Absence of the future-truncated re-run is proof the gate did not run.
//!
//! # SSB emission (off-agent artifact, R4)
//! Emits one [`crate::ssb::SsbPayload::LookaheadDifferential`] row on EVERY
//! decision (Causal / FutureLeak). Even an ALLOW row is evidence.
//!
//! # HARD RULES
//!   - No `.unwrap()`/`.expect()` outside `#[cfg(test)]` (CLAUDE.md).
//!   - blake3 only for hashing (CLAUDE.md anti-stale).
//!   - No new Rust crate; named constants (no magic float literals).
//!
//! References:
//!   - crates/orchestrator/src/leakage_gate.rs (H-20 sibling shape).
//!   - crates/orchestrator/src/determinism.rs:364 (DriftComparator first-
//!     divergence-index shape — reused, NOT called; that compares strings).
//!   - workflow/reasoning/H-21-decisions.md.
//!   - github.com/freqtrade/freqtrade freqtrade/optimize/analysis/lookahead.py.

use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::ssb::{LookaheadDifferentialKind, LookaheadDifferentialPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// LOCKED named constants (frozen in gate-manifest; the tolerance IS the gate)
// ---------------------------------------------------------------------------

/// Canonical rule id for the look-ahead differential gate (documentary parity
/// with the sibling gates' `*_RULE_ID`; H-21 seeds NO rules.db row — it is a
/// pure diff with no DKG predicate).
pub const LOOKAHEAD_RULE_ID: &str = "FIN-LOOKAHEAD-001";

/// Relative tolerance for the column-value comparator.
///
/// `|a-b| <= max(ABS, REL*max(|a|,|b|))`. `1e-9` is ~30 bits of mantissa
/// agreement — the standard "near-equal double" relative band (numpy.isclose's
/// default rtol is the looser `1e-5`; we tighten to `1e-9` because the seed is
/// fixed (ELAI_SEED, same-env per GD-2), so cross-RUN noise is near-zero and the
/// only residue we must absorb is cross-HARDWARE FMA/SIMD reassociation).
///
/// This is LOOSER than `f64::EPSILON` (~2.2e-16, which is 1 ulp at 1.0 and
/// guarantees false positives on benign reassociation) but TIGHTER than a real
/// leak: a column that read `iloc[t+1]` changes by O(price-magnitude) under
/// truncation, a relative delta `>> 1e-9`. This is H-21's OWN choice (TD-5),
/// NOT borrowed from Freqtrade (which uses exact compare). The GD-4 bound on the
/// false-positive rate over >=3 real causal pipelines is a Tier-2 follow-up
/// (no real pipelines this run); do NOT claim a validated FP rate.
pub const DEFAULT_RELATIVE_TOLERANCE: f64 = 1e-9;

/// Absolute tolerance floor for values near zero, where the relative test
/// degenerates (`rel*max(|a|,|b|)` -> 0). `1e-12` is well below any meaningful
/// financial signal magnitude yet above f64 round-off near zero.
pub const DEFAULT_ABSOLUTE_TOLERANCE: f64 = 1e-12;

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// Floating-point comparator tolerance: a value passes when
/// `|a-b| <= max(abs, rel*max(|a|,|b|))`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FpTolerance {
    /// Relative tolerance (fraction of the larger magnitude).
    pub rel: f64,
    /// Absolute tolerance floor (for values near zero).
    pub abs: f64,
}

impl Default for FpTolerance {
    fn default() -> Self {
        FpTolerance {
            rel: DEFAULT_RELATIVE_TOLERANCE,
            abs: DEFAULT_ABSOLUTE_TOLERANCE,
        }
    }
}

impl FpTolerance {
    /// True iff `a` and `b` are equal within this tolerance.
    ///
    /// NaN/inf handling is EXPLICIT (a financial gate must not panic on bad
    /// data, and a NaN is itself a divergence the caller must see):
    ///   - two bit-identical values (incl. `0.0 == 0.0`, `inf == inf` of the
    ///     same sign) are equal,
    ///   - a NaN on either side is NEVER close (NaN != NaN is itself a flagged
    ///     divergence, not a panic),
    ///   - mismatched infinities (`+inf` vs `-inf`, or `inf` vs finite) are a
    ///     divergence,
    ///   - otherwise the mixed absolute-OR-relative band applies.
    pub fn close(&self, a: f64, b: f64) -> bool {
        if a == b {
            // Covers exact equality incl. same-sign infinities and 0.0/-0.0.
            return true;
        }
        if a.is_nan() || b.is_nan() {
            // A NaN is a flagged divergence, never "close".
            return false;
        }
        if a.is_infinite() || b.is_infinite() {
            // One side infinite and not bit-equal to the other => divergence.
            return false;
        }
        let diff = (a - b).abs();
        let scale = a.abs().max(b.abs());
        diff <= self.abs.max(self.rel * scale)
    }
}

/// One named feature/signal column over the overlapping (pre-truncation) region.
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureColumn {
    /// Column name (must match across the full and truncated runs).
    pub name: String,
    /// Column values on the overlapping region.
    pub values: Vec<f64>,
}

/// An ordered, named set of feature columns (a minimal column-oriented frame;
/// orchestrator has no dataframe type, so H-21 builds this minimal shape).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColumnSet {
    /// Columns in declaration order.
    pub columns: Vec<FeatureColumn>,
}

impl ColumnSet {
    /// Construct from `(name, values)` pairs.
    pub fn from_pairs(pairs: Vec<(String, Vec<f64>)>) -> Self {
        ColumnSet {
            columns: pairs
                .into_iter()
                .map(|(name, values)| FeatureColumn { name, values })
                .collect(),
        }
    }

    /// Number of columns.
    pub fn width(&self) -> usize {
        self.columns.len()
    }

    /// Number of rows (max column length; an empty set has 0 rows). Used only
    /// for the shape guard — the per-column diff walks each column's own length.
    pub fn row_count(&self) -> usize {
        self.columns.iter().map(|c| c.values.len()).max().unwrap_or(0)
    }
}

/// The first column+index whose value diverged under future-truncation.
///
/// Mirrors [`crate::determinism::DeterminismDriftPayload::first_divergence_index`]
/// reporting, lifted to (column, index) over f64 values.
#[derive(Debug, Clone, PartialEq)]
pub struct FirstChange {
    /// Name of the first divergent column.
    pub column: String,
    /// 0-based row index of the first divergence within that column.
    pub index: u32,
    /// Value on the full run.
    pub full_value: f64,
    /// Value on the future-truncated run.
    pub truncated_value: f64,
    /// `|full - truncated|`.
    pub abs_diff: f64,
    /// Relative diff `|full - truncated| / max(|full|,|truncated|)` (0 if both 0).
    pub rel_diff: f64,
}

/// Verdict kind returned by [`diff_future_truncation`].
#[derive(Debug, Clone, PartialEq)]
pub enum LookaheadVerdictKind {
    /// All columns diff to zero within tolerance: no future used. ALLOW.
    Causal,
    /// >=1 column changed under future-truncation: a look-ahead leak. BLOCK.
    FutureLeak(FirstChange),
    /// Fail-closed: the future-truncated run was missing/empty, or the two sets
    /// had mismatched shapes / column sets, so causality could not be proven.
    /// Hard Block (R1/R2: missing evidence = FAIL, never ALLOW).
    Unprovable,
}

/// Full verdict from [`diff_future_truncation`].
#[derive(Debug, Clone, PartialEq)]
pub struct LookaheadVerdict {
    /// Causal / FutureLeak / Unprovable.
    pub kind: LookaheadVerdictKind,
    /// Machine-readable reason code
    /// ("causal" | "future_leak" | "missing_truncated" | "shape_mismatch" |
    /// "column_set_mismatch").
    pub reason: String,
    /// The first divergent column+index (Some only for FutureLeak).
    pub first_change: Option<FirstChange>,
    /// The relative tolerance applied (for the dashboard / audit).
    pub tolerance: f64,
    /// Free-text evidence string.
    pub evidence: String,
}

/// Input bundle for [`diff_future_truncation`].
#[derive(Debug, Clone)]
pub struct LookaheadInput {
    /// The feature/signal pipeline under test (repo-relative path or id).
    pub strategy_path: String,
    /// Pipeline output on the FULL data.
    pub full_run: ColumnSet,
    /// Pipeline output on the FUTURE-TRUNCATED data. `None` => fail-closed
    /// `Unprovable` Block (the re-run never happened).
    pub truncated_run: Option<ColumnSet>,
    /// Comparator tolerance (relative + absolute).
    pub tolerance: FpTolerance,
    /// ELAI_SEED for GD-2 same-env reproducibility / corpus pinning.
    pub seed: u64,
}

/// Errors from [`diff_future_truncation`]. Note: a missing/empty truncated run
/// and shape/column-set mismatches are NOT errors — they are fail-closed
/// `Unprovable` VERDICTS (each emits an SSB row). Only an SSB write failure is
/// an error here.
#[derive(Debug, Error)]
pub enum LookaheadError {
    /// Writing the off-agent LookaheadDifferential SSB row failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// Main gate: diff_future_truncation
// ---------------------------------------------------------------------------

/// Diff the full vs future-truncated feature column-sets (Tier-1 runtime
/// look-ahead backstop). Emits exactly one [`SsbPayload::LookaheadDifferential`]
/// row on every decision (the off-agent artifact, R4).
///
/// # Order of checks (FAIL-CLOSED first)
///   1. `truncated_run.is_none()` or empty => `Unprovable` Block
///      (reason `missing_truncated`): the re-run never happened, so causality
///      cannot be proven. NEVER "no diff -> Allow".
///   2. Column-SET guard: the two sets must have the SAME column names in the
///      SAME order. A name present in one but absent in the other (or a width
///      mismatch) => `Unprovable` Block (reason `column_set_mismatch`) — the
///      missing column IS a divergence, not silently skipped.
///   3. Shape guard: a column whose full vs truncated lengths differ over the
///      overlapping region => `Unprovable` Block (reason `shape_mismatch`).
///   4. Per-column value diff (reusing the first-divergence-index shape): walk
///      columns in order, within each walk indices, and on the FIRST value whose
///      tolerance check fails record a [`FirstChange`] and STOP =>
///      `FutureLeak` Block.
///   5. All columns diff to zero within tolerance => `Causal` Allow.
pub async fn diff_future_truncation(
    ssb: &Arc<Ssb>,
    input: LookaheadInput,
) -> Result<LookaheadVerdict, LookaheadError> {
    let tol = input.tolerance;

    // -- Step 1: FAIL-CLOSED on missing/empty truncated run. ----------------
    let truncated = match &input.truncated_run {
        Some(set) if set.width() > 0 => set,
        _ => {
            warn!(
                strategy = %input.strategy_path,
                "lookahead_differential: truncated run missing/empty -> Unprovable Block \
                 (fail-closed, R1/R2)"
            );
            let verdict = unprovable_verdict(
                tol.rel,
                "missing_truncated",
                "The future-truncated re-run is absent/empty: causality cannot be proven. \
                 Fail-closed Block (R1/R2 missing-evidence=FAIL).",
            );
            emit_ssb(ssb, build_payload(&input, &verdict)).await?;
            return Ok(verdict);
        }
    };
    let full = &input.full_run;

    // -- Step 2: column-SET guard (same names, same order, same width). -----
    if full.width() != truncated.width() {
        let verdict = unprovable_verdict(
            tol.rel,
            "column_set_mismatch",
            &format!(
                "Column-set width mismatch: full has {} columns, truncated has {}. \
                 A missing/extra column IS a divergence; fail-closed Block.",
                full.width(),
                truncated.width()
            ),
        );
        emit_ssb(ssb, build_payload(&input, &verdict)).await?;
        return Ok(verdict);
    }
    for (f_col, t_col) in full.columns.iter().zip(truncated.columns.iter()) {
        if f_col.name != t_col.name {
            let verdict = unprovable_verdict(
                tol.rel,
                "column_set_mismatch",
                &format!(
                    "Column-set name mismatch at the same position: full '{}' vs truncated \
                     '{}'. Fail-closed Block.",
                    f_col.name, t_col.name
                ),
            );
            emit_ssb(ssb, build_payload(&input, &verdict)).await?;
            return Ok(verdict);
        }
    }

    // -- Step 3: shape guard (per-column row counts must match). ------------
    for (f_col, t_col) in full.columns.iter().zip(truncated.columns.iter()) {
        if f_col.values.len() != t_col.values.len() {
            let verdict = unprovable_verdict(
                tol.rel,
                "shape_mismatch",
                &format!(
                    "Row-count mismatch in column '{}': full has {} rows, truncated has {}. \
                     The diff is over the overlapping region; an unestablishable overlap is \
                     fail-closed Block.",
                    f_col.name,
                    f_col.values.len(),
                    t_col.values.len()
                ),
            );
            emit_ssb(ssb, build_payload(&input, &verdict)).await?;
            return Ok(verdict);
        }
    }

    // -- Step 4: per-column value diff (first-divergence-index shape). -------
    if let Some(first_change) = first_divergence(full, truncated, &tol) {
        warn!(
            strategy = %input.strategy_path,
            column = %first_change.column,
            index = first_change.index,
            "lookahead_differential: column changed under future-truncation -> FutureLeak Block"
        );
        let evidence = format!(
            "Look-ahead leak: column '{}' value changed at row {} when the future was \
             truncated (full={}, truncated={}, abs_diff={}, rel_diff={}). The pipeline read \
             the future. Fail-closed Block.",
            first_change.column,
            first_change.index,
            first_change.full_value,
            first_change.truncated_value,
            first_change.abs_diff,
            first_change.rel_diff,
        );
        let verdict = LookaheadVerdict {
            kind: LookaheadVerdictKind::FutureLeak(first_change.clone()),
            reason: "future_leak".to_string(),
            first_change: Some(first_change),
            tolerance: tol.rel,
            evidence,
        };
        emit_ssb(ssb, build_payload(&input, &verdict)).await?;
        return Ok(verdict);
    }

    // -- Step 5: Causal Allow (all columns diff to zero within tolerance). --
    debug!(
        strategy = %input.strategy_path,
        "lookahead_differential: all columns stable under truncation -> Causal Allow"
    );
    let verdict = LookaheadVerdict {
        kind: LookaheadVerdictKind::Causal,
        reason: "causal".to_string(),
        first_change: None,
        tolerance: tol.rel,
        evidence: format!(
            "Causal: all {} columns diff to zero within tolerance (rel={}, abs={}) when the \
             future was truncated. No future used.",
            full.width(),
            tol.rel,
            tol.abs
        ),
    };
    emit_ssb(ssb, build_payload(&input, &verdict)).await?;
    Ok(verdict)
}

// ---------------------------------------------------------------------------
// Diff helpers
// ---------------------------------------------------------------------------

/// Walk columns in order, within each walk indices, returning the FIRST
/// (column, index) whose values are not close within `tol`. Mirrors
/// [`crate::determinism::DriftComparator::compare`]'s first-divergence-index
/// shape (one report at the first mismatch; identical => `None`), lifted to f64
/// columns with a relative-tolerance comparator instead of `!=`.
///
/// Precondition: `full` and `truncated` have matching column sets AND matching
/// per-column lengths (guaranteed by steps 2-3 of the caller).
fn first_divergence(
    full: &ColumnSet,
    truncated: &ColumnSet,
    tol: &FpTolerance,
) -> Option<FirstChange> {
    for (f_col, t_col) in full.columns.iter().zip(truncated.columns.iter()) {
        for (i, (&a, &b)) in f_col.values.iter().zip(t_col.values.iter()).enumerate() {
            if !tol.close(a, b) {
                let abs_diff = (a - b).abs();
                let scale = a.abs().max(b.abs());
                let rel_diff = if scale == 0.0 { 0.0 } else { abs_diff / scale };
                // u32 index (named narrowing via try_into per CLAUDE.md). A
                // pathological >4G-row column saturates to u32::MAX rather than
                // panicking; the column name + values still identify the leak.
                let index = u32::try_from(i).unwrap_or(u32::MAX);
                return Some(FirstChange {
                    column: f_col.name.clone(),
                    index,
                    full_value: a,
                    truncated_value: b,
                    abs_diff,
                    rel_diff,
                });
            }
        }
    }
    None
}

/// Build a fail-closed `Unprovable` verdict with the given reason + evidence.
fn unprovable_verdict(tolerance: f64, reason: &str, evidence: &str) -> LookaheadVerdict {
    LookaheadVerdict {
        kind: LookaheadVerdictKind::Unprovable,
        reason: reason.to_string(),
        first_change: None,
        tolerance,
        evidence: evidence.to_string(),
    }
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

async fn emit_ssb(
    ssb: &Arc<Ssb>,
    payload: LookaheadDifferentialPayload,
) -> Result<(), LookaheadError> {
    ssb.write_payload(
        "lookahead_differential",
        0,
        SsbPayload::LookaheadDifferential(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| LookaheadError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Build the off-agent [`LookaheadDifferentialPayload`] from inputs + verdict.
///
/// `corpus_blake3` pins the exact inputs so the verdict is recomputable (GD-2).
/// blake3 ONLY (CLAUDE.md HARD RULE).
fn build_payload(
    input: &LookaheadInput,
    verdict: &LookaheadVerdict,
) -> LookaheadDifferentialPayload {
    let kind = match &verdict.kind {
        // Causal => no leak. Unprovable (fail-closed) is carried under the
        // FutureLeak Kind because it BLOCKS (the gate refused to allow); the
        // exact reason ("missing_truncated"/"shape_mismatch"/etc.) is in
        // `verdict`. Only a genuine Causal pass maps to the Causal Kind.
        LookaheadVerdictKind::Causal => LookaheadDifferentialKind::Causal,
        LookaheadVerdictKind::FutureLeak(_) => LookaheadDifferentialKind::FutureLeak,
        LookaheadVerdictKind::Unprovable => LookaheadDifferentialKind::FutureLeak,
    };

    let (first_change_column, first_change_index, abs_diff, rel_diff) = match &verdict.first_change {
        Some(fc) => (fc.column.clone(), fc.index, fc.abs_diff, fc.rel_diff),
        None => (String::new(), 0, 0.0, 0.0),
    };

    let corpus_key = format!(
        "{}:{}:{}:{}",
        input.strategy_path, verdict.reason, verdict.tolerance, input.seed
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    LookaheadDifferentialPayload {
        kind,
        strategy_path: input.strategy_path.clone(),
        first_change_column,
        first_change_index,
        abs_diff,
        rel_diff,
        tolerance: verdict.tolerance,
        verdict: verdict.reason.clone(),
        seed: input.seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Unit tests (comparator NaN/inf + tolerance boundary + fail-closed guards)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;

    fn test_ssb() -> Arc<Ssb> {
        let dir = std::env::temp_dir().join(format!("h21-unit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = SsbConfig {
            db_path: dir.join(format!("ssb-{}.db", rand_suffix())),
            pool_size: Some(2),
            project_id: "h21-unit".to_string(),
            session_id: "h21-unit-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
        Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
    }

    fn rand_suffix() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now() // entropy-allow: test-only tempdir db filename suffix, never byte-compared by replay
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn cs(pairs: Vec<(&str, Vec<f64>)>) -> ColumnSet {
        ColumnSet::from_pairs(
            pairs
                .into_iter()
                .map(|(n, v)| (n.to_string(), v))
                .collect(),
        )
    }

    fn input(full: ColumnSet, truncated: Option<ColumnSet>) -> LookaheadInput {
        LookaheadInput {
            strategy_path: "strat.py".to_string(),
            full_run: full,
            truncated_run: truncated,
            tolerance: FpTolerance::default(),
            seed: 42,
        }
    }

    // -- comparator unit tests ------------------------------------------------

    #[test]
    fn close_exact_equal_including_zero_and_inf() {
        let t = FpTolerance::default();
        assert!(t.close(0.0, 0.0));
        assert!(t.close(0.0, -0.0));
        assert!(t.close(1.5, 1.5));
        assert!(t.close(f64::INFINITY, f64::INFINITY));
        assert!(t.close(f64::NEG_INFINITY, f64::NEG_INFINITY));
    }

    #[test]
    fn close_nan_is_never_close() {
        let t = FpTolerance::default();
        assert!(!t.close(f64::NAN, f64::NAN), "NaN!=NaN is a flagged divergence");
        assert!(!t.close(f64::NAN, 1.0));
        assert!(!t.close(1.0, f64::NAN));
    }

    #[test]
    fn close_mismatched_infinities_diverge() {
        let t = FpTolerance::default();
        assert!(!t.close(f64::INFINITY, f64::NEG_INFINITY));
        assert!(!t.close(f64::INFINITY, 1e308));
    }

    #[test]
    fn close_relative_boundary() {
        let t = FpTolerance::default();
        // 1.0 vs 1.0 + 0.5e-9: within rel 1e-9 (band is 1e-9*1.0 = 1e-9).
        assert!(t.close(1.0, 1.0 + 0.5e-9));
        // 1.0 vs 1.0 + 1e-6: a real divergence (>> 1e-9).
        assert!(!t.close(1.0, 1.0 + 1e-6));
    }

    #[test]
    fn close_absolute_floor_near_zero() {
        let t = FpTolerance::default();
        // Near zero, the relative band degenerates; the absolute floor applies.
        assert!(t.close(0.0, 0.5e-12));
        assert!(!t.close(0.0, 1e-9));
    }

    // -- gate fail-closed + diff tests ---------------------------------------

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn missing_truncated_is_fail_closed_block() {
        let ssb = test_ssb();
        let full = cs(vec![("rsi", vec![1.0, 2.0, 3.0])]);
        let verdict = diff_future_truncation(&ssb, input(full, None))
            .await
            .expect("diff");
        assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
        assert_eq!(verdict.reason, "missing_truncated");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_truncated_is_fail_closed_block() {
        let ssb = test_ssb();
        let full = cs(vec![("rsi", vec![1.0, 2.0])]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(ColumnSet::default())))
            .await
            .expect("diff");
        assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
        assert_eq!(verdict.reason, "missing_truncated");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn column_set_width_mismatch_is_block() {
        let ssb = test_ssb();
        let full = cs(vec![("a", vec![1.0]), ("b", vec![2.0])]);
        let truncated = cs(vec![("a", vec![1.0])]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
            .await
            .expect("diff");
        assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
        assert_eq!(verdict.reason, "column_set_mismatch");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn column_set_name_mismatch_is_block() {
        let ssb = test_ssb();
        let full = cs(vec![("a", vec![1.0])]);
        let truncated = cs(vec![("z", vec![1.0])]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
            .await
            .expect("diff");
        assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
        assert_eq!(verdict.reason, "column_set_mismatch");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shape_mismatch_is_block() {
        let ssb = test_ssb();
        let full = cs(vec![("a", vec![1.0, 2.0, 3.0])]);
        let truncated = cs(vec![("a", vec![1.0, 2.0])]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
            .await
            .expect("diff");
        assert_eq!(verdict.kind, LookaheadVerdictKind::Unprovable);
        assert_eq!(verdict.reason, "shape_mismatch");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn future_leak_is_blocked_with_first_change() {
        let ssb = test_ssb();
        // 'leaky' reads iloc[t+1]: index 2 changes when the tail is removed.
        let full = cs(vec![
            ("causal", vec![10.0, 11.0, 12.0]),
            ("leaky", vec![1.0, 2.0, 99.0]),
        ]);
        let truncated = cs(vec![
            ("causal", vec![10.0, 11.0, 12.0]),
            ("leaky", vec![1.0, 2.0, 3.0]),
        ]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
            .await
            .expect("diff");
        match verdict.kind {
            LookaheadVerdictKind::FutureLeak(fc) => {
                assert_eq!(fc.column, "leaky");
                assert_eq!(fc.index, 2);
            }
            other => panic!("expected FutureLeak, got {other:?}"),
        }
        assert_eq!(verdict.reason, "future_leak");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn causal_passes() {
        let ssb = test_ssb();
        let full = cs(vec![("sma", vec![1.0, 2.0, 3.0])]);
        let truncated = cs(vec![("sma", vec![1.0, 2.0, 3.0])]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
            .await
            .expect("diff");
        assert_eq!(verdict.kind, LookaheadVerdictKind::Causal);
        assert_eq!(verdict.reason, "causal");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fp_noise_within_tolerance_does_not_block() {
        let ssb = test_ssb();
        // Sub-1e-9 relative jitter (cross-hardware FP residue) must NOT block.
        let full = cs(vec![("x", vec![1.0, 100.0, 1000.0])]);
        let truncated = cs(vec![(
            "x",
            vec![1.0 + 0.5e-9, 100.0 + 5e-8, 1000.0 + 5e-7],
        )]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
            .await
            .expect("diff");
        assert_eq!(
            verdict.kind,
            LookaheadVerdictKind::Causal,
            "sub-tolerance FP noise must pass"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn nan_in_column_is_a_divergence_block() {
        let ssb = test_ssb();
        let full = cs(vec![("x", vec![1.0, f64::NAN, 3.0])]);
        let truncated = cs(vec![("x", vec![1.0, f64::NAN, 3.0])]);
        let verdict = diff_future_truncation(&ssb, input(full, Some(truncated)))
            .await
            .expect("diff");
        // NaN != NaN: a NaN column is flagged as a leak (never silently equal).
        match verdict.kind {
            LookaheadVerdictKind::FutureLeak(fc) => {
                assert_eq!(fc.column, "x");
                assert_eq!(fc.index, 1);
            }
            other => panic!("expected FutureLeak on NaN, got {other:?}"),
        }
    }
}
