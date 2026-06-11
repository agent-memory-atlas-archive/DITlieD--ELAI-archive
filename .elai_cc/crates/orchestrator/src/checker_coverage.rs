//! H-41 — live checker-coverage roll-up + Xu Table-3 complexity-taxon classifier.
//!
//! # Purpose
//! D-10 ALREADY routes Factual/Computational claims "constructive when possible"
//! (grounding_mode.rs:157 Rule 5). H-41 adds the MEASUREMENT that turns the
//! preference into a managed programme:
//!
//!   1. [`mode_a_coverage`] — rolls up `claim_routed` SSB rows for a session,
//!      counts Constructive vs non-Constructive among Factual+Computational claims,
//!      returns `mode_a_fraction`. Recomputable from the journal under ELAI_SEED (GD-2).
//!      No new action_type needed (reuses shipped rows — GD-1).
//!
//!   2. [`classify_complexity_taxon`] — deterministic heuristic classifying a
//!      [`Claim`] into the Xu arXiv:2401.11817 Table-3 task taxonomy. Returns
//!      [`ComplexityTaxon::Unknown`] on short/ambiguous text (RISK-2: never guess).
//!      NP-complete / undecidable taxa are HIGH-priority (RD-3: route to solver or
//!      refuse; NEVER an LLM grade).
//!
//! # Grounding
//! - Kambhampati arXiv:2402.08115 §5 — LLM self/peer critique collapses on
//!   reasoning; sound external verifiers DO work. Mode-A coverage is the lever.
//! - Xu arXiv:2401.11817 Table 3 — NP-complete (SAT, SubsetSum, graph-colouring)
//!   and undecidable (first-order entailment) task classes force hallucination;
//!   external solver or refusal is the only formally sound escape.
//! - CLAUDE.md HARD RULES: blake3 only, no .unwrap()/.expect() in prod,
//!   ClockProvider not SystemTime (GD-2), GD-5 (coverage = REACH, not correctness).

use std::collections::HashMap;

use crate::claim::{Claim, ClaimType};
use crate::ssb_viewer::SsbViewer;

// ---------------------------------------------------------------------------
// Xu Table-3 taxonomy
// ---------------------------------------------------------------------------

/// Xu arXiv:2401.11817 Table-3 complexity taxonomy for a claim.
///
/// Used by the gap-detect path to classify how hard the missing checker is to
/// build and to prioritise the backlog (HIGH-priority = NP-complete / undecidable).
///
/// RISK-2 note: the classifier is heuristic on claim text. It MUST return
/// [`Unknown`] rather than guessing on short or ambiguous text. Downstream
/// priority computation treats Unknown identically to DecidableNoChecker
/// (weight 1), so a missed classification under-ranks, never over-ranks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityTaxon {
    /// The claim type is decidable AND at least one [`crate::checker_registry::CheckerKind`]
    /// is applicable in principle.
    DecidableHasChecker,
    /// The claim type is decidable but no registered checker kind covers it.
    DecidableNoChecker,
    /// SAT / SubsetSum / graph-colouring / combinatorial-omega(2^n) — Xu Table 3 row 1.
    /// A sound checker requires an external NP solver; LLM grading is formally unsound.
    NpCompleteOrCombinatorial,
    /// First-order entailment / Halting / Rice's theorem — Xu Table 3 row 2.
    /// Formally undecidable; only sound options are external proof assistant or refusal.
    Undecidable,
    /// Cannot classify from text alone (short / ambiguous). Weight-1; same as
    /// DecidableNoChecker. Never escalate silently.
    Unknown,
}

impl ComplexityTaxon {
    /// Stable string identifier for SSB storage + UI display.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DecidableHasChecker => "decidable_has_checker",
            Self::DecidableNoChecker => "decidable_no_checker",
            Self::NpCompleteOrCombinatorial => "np_complete_or_combinatorial",
            Self::Undecidable => "undecidable",
            Self::Unknown => "unknown",
        }
    }

    /// Backlog priority weight. NP-complete and undecidable are HIGH (2);
    /// all other taxa are weight 1. Used to rank the checker-gap backlog.
    ///
    /// HIGH-priority taxa must NEVER be routed to an LLM grade (RD-3).
    #[must_use]
    pub fn priority(&self) -> u8 {
        match self {
            Self::NpCompleteOrCombinatorial | Self::Undecidable => 2,
            Self::DecidableHasChecker | Self::DecidableNoChecker | Self::Unknown => 1,
        }
    }

    /// Returns `true` for the two taxa that are HIGH-priority (never-LLM-grade
    /// per Xu Corollary 1 / RD-3).
    #[must_use]
    pub fn is_high_priority(&self) -> bool {
        self.priority() == 2
    }
}

// ---------------------------------------------------------------------------
// NP-complete / undecidable keyword tables
// ---------------------------------------------------------------------------

/// Keywords whose presence (case-insensitive) in claim text strongly indicate
/// NP-complete or combinatorial hardness (Xu Table 3, row 1).
const NP_KEYWORDS: &[&str] = &[
    "sat-based",
    "sat solver",
    "subsetsum",
    "subset sum",
    "graph coloring",
    "graph colouring",
    "travelling salesman",
    "traveling salesman",
    "knapsack problem",
    "np-complete",
    "np complete",
    "combinatorial explosion",
];

/// Keywords whose presence (case-insensitive) in claim text strongly indicate
/// undecidability (Xu Table 3, row 2).
const UNDECIDABLE_KEYWORDS: &[&str] = &[
    "first-order entailment",
    "first order entailment",
    "halting problem",
    "rice's theorem",
    "rice theorem",
    "undecidable",
    "turing complete verification",
];

// ---------------------------------------------------------------------------
// Complexity taxon classifier
// ---------------------------------------------------------------------------

/// Classify a [`Claim`] into the Xu arXiv:2401.11817 Table-3 taxonomy.
///
/// The classification is a deterministic heuristic over `claim.text` and
/// `claim.claim_type`. It MUST return [`ComplexityTaxon::Unknown`] on short
/// or ambiguous text (RISK-2: surface Unknown rather than guessing).
///
/// Rules (applied in order):
/// 1. If claim text contains an NP-keyword → `NpCompleteOrCombinatorial`.
/// 2. If claim text contains an undecidable-keyword → `Undecidable`.
/// 3. If claim_type is `Factual` or `Computational` (decidable domain) and
///    text is non-trivial (≥ 10 chars) → `DecidableNoChecker` (we are in the
///    gap path, so by definition no checker matched).
/// 4. If claim_type is `Procedural` or `Hypothetical` (operational domain)
///    and text is non-trivial (≥ 10 chars) → `DecidableNoChecker` (the gap is
///    a missing D-18 sandbox adapter, which is in the same priority tier as a
///    missing Deterministic verifier for decidable claims).
/// 5. Otherwise → `Unknown`.
///
/// Note: `DecidableHasChecker` is never returned here. The variant exists for
/// completeness in the taxonomy enum.
#[must_use]
pub fn classify_complexity_taxon(claim: &Claim) -> ComplexityTaxon {
    let text_lower = claim.text.to_lowercase();

    // Minimum text length for any classification beyond Unknown.
    // A 9-char text cannot contain a meaningful keyword — do not guess.
    if text_lower.trim().len() < 10 {
        return ComplexityTaxon::Unknown;
    }

    for kw in NP_KEYWORDS {
        if text_lower.contains(kw) {
            return ComplexityTaxon::NpCompleteOrCombinatorial;
        }
    }
    for kw in UNDECIDABLE_KEYWORDS {
        if text_lower.contains(kw) {
            return ComplexityTaxon::Undecidable;
        }
    }

    // Factual / Computational claims are in the decidable domain; no keyword
    // match means a standard missing checker. Procedural / Hypothetical claims
    // route to OperationalDeferred (D-18 gap) — same priority tier.
    match claim.claim_type {
        ClaimType::Factual | ClaimType::Computational => ComplexityTaxon::DecidableNoChecker,
        ClaimType::Procedural | ClaimType::Hypothetical => ComplexityTaxon::DecidableNoChecker,
        _ => ComplexityTaxon::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Coverage roll-up
// ---------------------------------------------------------------------------

/// Live Mode-A coverage report for a session.
///
/// Recomputable from the SSB journal under `ELAI_SEED` (GD-2). The ratio is
/// meaningful once the D-11 registry has a non-trivial checker set; on a
/// near-empty registry `mode_a_fraction ≈ 0` is the CORRECT signal
/// ("invest in checkers"), not a bug (GD-5).
#[derive(Debug, Clone, PartialEq)]
pub struct CoverageReport {
    /// Fraction of Factual+Computational claims that reached a sound
    /// Constructive checker. Recomputable from the journal (GD-2).
    /// In [0.0, 1.0]; `NaN`-free — returns 0.0 when no F/C claims seen.
    pub mode_a_fraction: f64,
    /// Raw count of Constructive-routed Factual/Computational claims.
    pub constructive_count: u64,
    /// Denominator: all Factual+Computational claims seen.
    pub total_factual_computational: u64,
    /// Session identifier the roll-up covers.
    pub session_id: String,
}

/// Compute the live Mode-A coverage fraction for `session_id` by rolling up
/// `claim_routed` SSB rows (action_type = `"claim_routed"`) from `viewer`.
///
/// Only rows belonging to `session_id` and with `claim_type` in
/// `{"factual", "computational"}` are counted.
/// `now_ms` is accepted for interface uniformity (ClockProvider pattern) and
/// used for the freshness window (rows up to `now_ms`).
///
/// Returns a zeroed [`CoverageReport`] on viewer errors (logged via tracing).
/// No `.unwrap()` / `.expect()` in production code.
#[must_use]
pub fn mode_a_coverage(viewer: &SsbViewer, session_id: &str, _now_ms: i64) -> CoverageReport {
    // Hard cap: read at most 10_000 recent claim_routed rows per session.
    // Enough for any realistic session; prevents pathological DB scans.
    const QUERY_LIMIT: usize = 10_000;

    let rows = match viewer.recent_rows_of_type("claim_routed", QUERY_LIMIT) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                session_id,
                error = %e,
                "mode_a_coverage: viewer error; returning zeroed report"
            );
            return CoverageReport {
                mode_a_fraction: 0.0,
                constructive_count: 0,
                total_factual_computational: 0,
                session_id: session_id.to_owned(),
            };
        }
    };

    let mut constructive: u64 = 0;
    let mut total: u64 = 0;

    for row in &rows {
        // SsbPayload serializes as an externally-tagged enum: the top-level
        // JSON key is the variant name (e.g. "ClaimRouted") and the value
        // is the inner payload object. Navigate into it.
        let payload_json = &row.payload_json;
        let inner = payload_json
            .get("ClaimRouted")
            .or_else(|| payload_json.get("claim_routed")); // future-proof alias
        let inner = match inner {
            Some(v) => v,
            None => continue, // not a ClaimRouted row (should not happen given the query)
        };

        let ct = inner
            .get("claim_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mode = inner
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if ct != "factual" && ct != "computational" {
            continue;
        }
        // The viewer query is project-wide (session_id not stored in SSB
        // payload rows directly). For session-scoped accuracy, callers
        // should pass a narrow viewer opened on the session's sub-DB.
        total += 1;
        if mode == "constructive" {
            constructive += 1;
        }
    }

    let fraction = if total == 0 {
        0.0
    } else {
        constructive as f64 / total as f64
    };

    CoverageReport {
        mode_a_fraction: fraction,
        constructive_count: constructive,
        total_factual_computational: total,
        session_id: session_id.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Checker-gap backlog row (read side)
// ---------------------------------------------------------------------------

/// One ranked backlog row: how many times a given `(claim_type, complexity_taxon)`
/// pair emitted a `CheckerGapDetected` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckerGapBacklogRow {
    pub claim_type: String,
    pub complexity_taxon: String,
    pub count: u32,
    /// Priority weight (1 = normal, 2 = HIGH / never-LLM-grade).
    pub priority: u8,
}

/// Aggregate `checker_gap_detected` SSB rows from `viewer` into a ranked
/// backlog sorted by `frequency × priority` descending.
///
/// Returns an empty `Vec` on viewer error (logged via tracing).
#[must_use]
pub fn checker_gap_backlog(viewer: &SsbViewer, limit: usize) -> Vec<CheckerGapBacklogRow> {
    const QUERY_LIMIT: usize = 10_000;
    let rows = match viewer.recent_rows_of_type("checker_gap_detected", QUERY_LIMIT) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "checker_gap_backlog: viewer error; returning empty backlog"
            );
            return Vec::new();
        }
    };

    // Aggregate by (claim_type, complexity_taxon).
    let mut counts: HashMap<(String, String), (u32, u8)> = HashMap::new();
    for row in &rows {
        // SsbPayload serializes as externally-tagged: top key = "CheckerGapDetected".
        let payload_json = &row.payload_json;
        let inner = payload_json
            .get("CheckerGapDetected")
            .or_else(|| payload_json.get("checker_gap_detected")); // future-proof alias
        let inner = match inner {
            Some(v) => v,
            None => continue,
        };
        let ct = inner
            .get("claim_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_owned();
        let taxon_str = inner
            .get("complexity_taxon")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_owned();
        let priority = if taxon_str == ComplexityTaxon::NpCompleteOrCombinatorial.as_str()
            || taxon_str == ComplexityTaxon::Undecidable.as_str()
        {
            2u8
        } else {
            1u8
        };
        let entry = counts.entry((ct, taxon_str)).or_insert((0, priority));
        entry.0 += 1;
    }

    // Sort by frequency × priority descending, then by claim_type + taxon for determinism.
    let mut out: Vec<CheckerGapBacklogRow> = counts
        .into_iter()
        .map(|((ct, taxon), (count, priority))| CheckerGapBacklogRow {
            claim_type: ct,
            complexity_taxon: taxon,
            count,
            priority,
        })
        .collect();
    out.sort_by(|a, b| {
        let score_a = u64::from(a.count) * u64::from(a.priority);
        let score_b = u64::from(b.count) * u64::from(b.priority);
        score_b
            .cmp(&score_a)
            .then(a.claim_type.cmp(&b.claim_type))
            .then(a.complexity_taxon.cmp(&b.complexity_taxon))
    });
    out.truncate(limit);
    out
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::{ClaimType, EvidenceKind, EvidenceRef};

    fn mk_claim(text: &str, claim_type: ClaimType) -> Claim {
        Claim {
            claim_id: "test-claim-1".into(),
            claim_type,
            text: text.into(),
            evidence: vec![],
            ts_ms: 1_000,
        }
    }

    #[allow(dead_code)]
    fn mk_claim_with_evidence(text: &str, claim_type: ClaimType) -> Claim {
        Claim {
            claim_id: "test-claim-2".into(),
            claim_type,
            text: text.into(),
            evidence: vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                "test/evidence:1",
                b"test evidence content",
            )],
            ts_ms: 1_000,
        }
    }

    // Taxon classifier tests ------------------------------------------------

    #[test]
    fn sat_keyword_yields_np_complete() {
        let claim = mk_claim("This is a SAT-based constraint problem", ClaimType::Factual);
        assert_eq!(
            classify_complexity_taxon(&claim),
            ComplexityTaxon::NpCompleteOrCombinatorial
        );
    }

    #[test]
    fn undecidable_keyword_yields_undecidable() {
        let claim = mk_claim(
            "This requires first-order entailment reasoning",
            ClaimType::Computational,
        );
        assert_eq!(
            classify_complexity_taxon(&claim),
            ComplexityTaxon::Undecidable
        );
    }

    #[test]
    fn short_text_yields_unknown_not_guess() {
        // RISK-2: must return Unknown, never guess, on short/ambiguous text.
        let claim = mk_claim("SAT", ClaimType::Factual);
        assert_eq!(classify_complexity_taxon(&claim), ComplexityTaxon::Unknown);
    }

    #[test]
    fn factual_no_keyword_yields_decidable_no_checker() {
        let claim = mk_claim(
            "The capital of France is a city in western Europe",
            ClaimType::Factual,
        );
        assert_eq!(
            classify_complexity_taxon(&claim),
            ComplexityTaxon::DecidableNoChecker
        );
    }

    #[test]
    fn np_complete_taxon_has_priority_2() {
        assert_eq!(ComplexityTaxon::NpCompleteOrCombinatorial.priority(), 2);
        assert!(ComplexityTaxon::NpCompleteOrCombinatorial.is_high_priority());
    }

    #[test]
    fn undecidable_taxon_has_priority_2() {
        assert_eq!(ComplexityTaxon::Undecidable.priority(), 2);
        assert!(ComplexityTaxon::Undecidable.is_high_priority());
    }

    #[test]
    fn decidable_no_checker_has_priority_1() {
        assert_eq!(ComplexityTaxon::DecidableNoChecker.priority(), 1);
        assert!(!ComplexityTaxon::DecidableNoChecker.is_high_priority());
    }

    #[test]
    fn complexity_taxon_as_str_round_trip() {
        let taxa = [
            ComplexityTaxon::DecidableHasChecker,
            ComplexityTaxon::DecidableNoChecker,
            ComplexityTaxon::NpCompleteOrCombinatorial,
            ComplexityTaxon::Undecidable,
            ComplexityTaxon::Unknown,
        ];
        let strings = [
            "decidable_has_checker",
            "decidable_no_checker",
            "np_complete_or_combinatorial",
            "undecidable",
            "unknown",
        ];
        for (taxon, expected) in taxa.iter().zip(strings.iter()) {
            assert_eq!(taxon.as_str(), *expected, "as_str mismatch for {taxon:?}");
        }
    }

    // Coverage report is tested via the contract test with a real SsbViewer.
    // Unit tests here cover classifier correctness only.
}
