//! Spectrum-based fault localization (SBFL) -- Ochiai compute.
//!
//! Per AutoCodeRover (arXiv:2404.05427, ISSTA 2024) the Ochiai score
//! ranks suspicious source lines from a per-test coverage matrix. The
//! AutoCodeRover paper reports 37.3% SWE-lite pass@1 with SBFL as the
//! first-stage candidate-line selector.
//!
//! Ochiai formula (verified against the AutoCodeRover paper; the
//! E-21 plan body had the numerator transposed and is corrected here
//! per workflow/reasoning/E-21-decisions.md D-B-3):
//!
//! ```text
//!     Ochiai(line) = e_fail(line) / sqrt(F_total * e_total(line))
//! ```
//!
//! where
//!   - `e_fail(line)`   = number of FAILING tests that executed `line`
//!   - `F_total`        = total number of failing tests in the run
//!   - `e_total(line)`  = number of tests (passing + failing) that
//!                        executed `line`
//!
//! When `F_total * e_total(line)` is zero the score is defined as 0.0
//! (a line never executed by any test cannot be suspicious; a run with
//! no failing tests has no suspects).
//!
//! This module is a PURE COMPUTE surface. It does NOT shell out to
//! `cargo-tarpaulin` / `cargo-llvm-cov` / any coverage harness; the
//! caller assembles the [`CoverageReport`] from whatever source they
//! have available (per workflow/reasoning/E-21-decisions.md D-B-3, no
//! new dep is added in this pass). Once a coverage harness lands as
//! a separate (user-confirmed) E-N, its output becomes the input to
//! [`compute_ochiai`].

use std::collections::BTreeMap;

/// One source line identified by (file path, 1-based line number).
///
/// `PartialOrd` / `Ord` are derived so [`CoverageReport`] can use a
/// `BTreeMap` keyed on [`LineId`] for deterministic output ordering
/// (important so dashboard tile + auditor consumer see the same rank
/// order on repeated runs over the same coverage data).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineId {
    /// Source file path, normalized to forward slashes by the caller
    /// (no path normalization is performed here -- garbage in, garbage
    /// out; the caller's coverage harness owns the canonical form).
    pub path: String,
    /// 1-based line number. Zero is rejected by [`compute_ochiai`]
    /// (returns an error variant).
    pub line: u32,
}

/// Per-line execution counts for one full test run (passing + failing
/// tests aggregated).
///
/// Construction contract: the caller is responsible for ensuring
/// `e_total >= e_fail` (a line cannot be executed by more failing
/// tests than total tests). [`compute_ochiai`] validates this and
/// returns [`SbflError::Inconsistent`] when violated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCoverage {
    /// Total tests (passing + failing) that executed this line.
    pub e_total: u32,
    /// Number of FAILING tests that executed this line.
    pub e_fail: u32,
}

/// Per-run coverage snapshot: total failing-test count + per-line
/// execution counts.
///
/// Pure data; no I/O. Build it from a coverage harness output (e.g.
/// `cargo tarpaulin --out Json`, `cargo llvm-cov --json`, or an in-
/// process counter) on the caller side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageReport {
    /// Total failing-test count in the run (denominator of Ochiai).
    pub f_total: u32,
    /// Per-line execution stats. Ordered map for deterministic
    /// output rank.
    pub lines: BTreeMap<LineId, LineCoverage>,
}

impl CoverageReport {
    /// Convenience constructor. Callers that build the report
    /// piecewise can use [`Self::insert`] to add lines one at a time.
    #[must_use]
    pub fn new(f_total: u32) -> Self {
        Self {
            f_total,
            lines: BTreeMap::new(),
        }
    }

    /// Insert / overwrite a single line's coverage counts. Returns
    /// the previous value if the line was already present (useful for
    /// callers that aggregate over multiple coverage shards).
    pub fn insert(
        &mut self,
        id: LineId,
        cov: LineCoverage,
    ) -> Option<LineCoverage> {
        self.lines.insert(id, cov)
    }

    /// Total number of lines tracked.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// One ranked suspicious line emitted by [`compute_ochiai`].
///
/// Output ordering: descending by `score`, then ascending by
/// `(path, line)` for stable tie-breaking. Consumers that wire SBFL
/// into the fixer/B-23 self-mod pipeline can truncate the head of the
/// sorted vector (typical top-K = 20 per AutoCodeRover).
#[derive(Debug, Clone, PartialEq)]
pub struct SuspiciousLine {
    /// Source file path (mirrors the input [`LineId::path`]).
    pub path: String,
    /// 1-based line number.
    pub line: u32,
    /// Ochiai score in [0.0, 1.0]. Higher = more suspicious. A score
    /// of exactly 0.0 means either (a) the line was never executed by
    /// any failing test, or (b) the run had zero failing tests.
    pub score: f64,
    /// Number of failing tests that executed this line (mirrors the
    /// input `e_fail` so consumers can audit the raw counts).
    pub e_fail: u32,
    /// Total tests that executed this line (mirrors the input
    /// `e_total`).
    pub e_total: u32,
}

/// SBFL compute errors.
#[derive(Debug, thiserror::Error)]
pub enum SbflError {
    /// A [`LineCoverage`] entry had `e_fail > e_total`, which is
    /// physically impossible (failing tests are a subset of all
    /// tests). Indicates an upstream coverage-harness bug.
    #[error(
        "inconsistent coverage for {path}:{line} (e_fail={e_fail} > e_total={e_total})"
    )]
    Inconsistent {
        /// Source file path.
        path: String,
        /// 1-based line number.
        line: u32,
        /// Failing-test executions reported.
        e_fail: u32,
        /// Total executions reported.
        e_total: u32,
    },
    /// A [`LineId`] had a `line` field of zero (line numbers are
    /// 1-based per the [`LineId`] docstring).
    #[error("zero line number for {path} (LineId is 1-based)")]
    ZeroLine {
        /// Source file path with the offending zero line.
        path: String,
    },
}

/// Compute Ochiai suspicion scores for every line in `report` and
/// return them sorted by descending score (then by path/line for
/// deterministic tie-breaking).
///
/// Returns an empty vector when `report.f_total == 0` (no failing
/// tests -> no suspects). Returns [`SbflError::Inconsistent`] /
/// [`SbflError::ZeroLine`] when the input is malformed.
///
/// Complexity: O(n log n) where n = `report.line_count()` (the sort
/// dominates the per-line score arithmetic).
pub fn compute_ochiai(
    report: &CoverageReport,
) -> Result<Vec<SuspiciousLine>, SbflError> {
    // Short-circuit: no failing tests -> empty suspect list.
    if report.f_total == 0 {
        return Ok(Vec::new());
    }

    let mut out: Vec<SuspiciousLine> = Vec::with_capacity(report.lines.len());

    for (id, cov) in &report.lines {
        if id.line == 0 {
            return Err(SbflError::ZeroLine {
                path: id.path.clone(),
            });
        }
        if cov.e_fail > cov.e_total {
            return Err(SbflError::Inconsistent {
                path: id.path.clone(),
                line: id.line,
                e_fail: cov.e_fail,
                e_total: cov.e_total,
            });
        }

        // Ochiai = e_fail / sqrt(f_total * e_total).
        // When the denominator is zero (e_total == 0; f_total > 0
        // already checked above), the line was never executed so
        // score = 0.0 (the safe interpretation; cannot blame an
        // un-executed line).
        let score: f64 = if cov.e_total == 0 {
            0.0
        } else {
            let denom = ((report.f_total as f64) * (cov.e_total as f64)).sqrt();
            (cov.e_fail as f64) / denom
        };

        out.push(SuspiciousLine {
            path: id.path.clone(),
            line: id.line,
            score,
            e_fail: cov.e_fail,
            e_total: cov.e_total,
        });
    }

    // Sort: descending score, then ascending path, then ascending
    // line. partial_cmp on f64 is safe here because score >= 0.0
    // (no negative values produced by the Ochiai branch) and we
    // explicitly handle the NaN-impossible domain (sqrt of a
    // non-negative real, no division-by-zero reaches this point).
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.line.cmp(&b.line))
    });

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lid(path: &str, line: u32) -> LineId {
        LineId {
            path: path.to_string(),
            line,
        }
    }

    fn cov(e_total: u32, e_fail: u32) -> LineCoverage {
        LineCoverage { e_total, e_fail }
    }

    #[test]
    fn empty_report_returns_empty() {
        let report = CoverageReport::new(0);
        let out = compute_ochiai(&report).expect("compute");
        assert!(out.is_empty(), "no failing tests -> no suspects");
    }

    #[test]
    fn synthetic_three_line_table_matches_hand_compute() {
        // Hand-computed expected scores for f_total=4:
        //   L1: e_total=4, e_fail=4 -> 4 / sqrt(16) = 4/4 = 1.0
        //   L2: e_total=4, e_fail=2 -> 2 / sqrt(16) = 2/4 = 0.5
        //   L3: e_total=2, e_fail=0 -> 0 / sqrt(8)  = 0.0
        let mut report = CoverageReport::new(4);
        report.insert(lid("src/a.rs", 10), cov(4, 4));
        report.insert(lid("src/b.rs", 20), cov(4, 2));
        report.insert(lid("src/c.rs", 30), cov(2, 0));

        let out = compute_ochiai(&report).expect("compute");
        assert_eq!(out.len(), 3);

        // Descending by score: L1 (1.0) > L2 (0.5) > L3 (0.0).
        assert_eq!(out[0].path, "src/a.rs");
        assert!((out[0].score - 1.0).abs() < 1e-9);
        assert_eq!(out[1].path, "src/b.rs");
        assert!((out[1].score - 0.5).abs() < 1e-9);
        assert_eq!(out[2].path, "src/c.rs");
        assert!(out[2].score.abs() < 1e-9);
    }

    #[test]
    fn inconsistent_coverage_returns_error() {
        // e_fail > e_total is physically impossible.
        let mut report = CoverageReport::new(2);
        report.insert(lid("src/x.rs", 1), cov(1, 3));

        let err = compute_ochiai(&report).expect_err("must reject");
        assert!(matches!(err, SbflError::Inconsistent { .. }));
    }

    #[test]
    fn zero_line_returns_error() {
        let mut report = CoverageReport::new(1);
        report.insert(lid("src/x.rs", 0), cov(1, 1));

        let err = compute_ochiai(&report).expect_err("must reject");
        assert!(matches!(err, SbflError::ZeroLine { .. }));
    }

    #[test]
    fn ties_break_by_path_then_line() {
        let mut report = CoverageReport::new(4);
        // Same score (0.5 each): identical (e_total, e_fail) for two
        // different lines. Stable tie-break: src/a.rs:10 before
        // src/b.rs:20 (path ascending).
        report.insert(lid("src/b.rs", 20), cov(4, 2));
        report.insert(lid("src/a.rs", 10), cov(4, 2));
        // Same path, two lines: line ascending breaks the tie.
        report.insert(lid("src/a.rs", 5), cov(4, 2));

        let out = compute_ochiai(&report).expect("compute");
        assert_eq!(out.len(), 3);
        assert_eq!((out[0].path.as_str(), out[0].line), ("src/a.rs", 5));
        assert_eq!((out[1].path.as_str(), out[1].line), ("src/a.rs", 10));
        assert_eq!((out[2].path.as_str(), out[2].line), ("src/b.rs", 20));
    }

    #[test]
    fn never_executed_line_scores_zero() {
        // e_total = 0: line is not in the coverage path of any test.
        // Score must be 0.0, not NaN (sqrt(0) division guard).
        let mut report = CoverageReport::new(3);
        report.insert(lid("src/dead.rs", 1), cov(0, 0));

        let out = compute_ochiai(&report).expect("compute");
        assert_eq!(out.len(), 1);
        assert!(out[0].score.abs() < 1e-9);
        assert!(out[0].score.is_finite(), "must not be NaN/inf");
    }
}
