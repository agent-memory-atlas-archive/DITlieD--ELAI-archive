//! E-48 Task 7 — never-silently-resolved Contradiction surfacing.
//!
//! Provides the host-side scaffolding for the LLM-backed contradiction
//! pair-judge. The live sidecar call is RUNPOD-PENDING -- the
//! [`Classifier`] trait abstracts the backend so a test stub can be
//! injected for contract tests without standing up a sidecar.
//!
//! Materialization sets `prune_protected: true` on the produced
//! [`crate::research_tree::Contradiction`] so the speculative pruner
//! refuses to drop the parent node (enforced by the
//! [`super::speculative_pruner::evaluate`] predicate).
//!
//! Renderer obligation (the "never silently resolved" canon from
//! arXiv:2511.06668 healthcare RAG + arXiv:2510.03418 LegalWiz) lives
//! in the downstream report renderer; the
//! [`render_contradiction_line`] helper here produces the canonical
//! "Source A says X; Source B says ~X; reconciliation: [unresolved]"
//! line that the renderer MUST surface verbatim.
//!
//! O(n^2) pairwise budget: at most [`CONTRADICTION_PAIR_BUDGET`] pairs
//! are considered per REFLECT cycle. When the source list exceeds the
//! cap, the most recent rows (rightmost slice) are kept and the
//! producer-side emit fn records a `contradiction_check_budget_clamped`
//! sentinel inside the regular `contradiction_node_materialized` row
//! (the `details_json.budget_clamped: true` field).

use serde::{Deserialize, Serialize};

use crate::research_tree::{Contradiction, SourceCitation};

/// Maximum number of evidence rows considered in the O(n^2) pairwise
/// contradiction check per REFLECT cycle. Plan spec caps at 50 to
/// bound cost; deeper trees can re-run the classifier on subsequent
/// cycles so unbounded growth is not required.
pub const CONTRADICTION_PAIR_BUDGET: usize = 50;

/// Maximum length of the `classification_reason` field on a
/// materialized Contradiction row. Mirrors the SSB row's
/// `classification_reason` truncation cap so the on-disk Contradiction
/// and the SSB telemetry stay in sync.
pub const CLASSIFICATION_REASON_CHAR_CAP: usize = 200;

/// LLM verdict for a single source-pair judgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairVerdict {
    /// Sources agree on the claim.
    Agree,
    /// Sources disagree on the claim. Materializes a Contradiction.
    Disagree,
    /// Sources address orthogonal aspects; no contradiction to surface.
    Orthogonal,
}

/// One LLM pair-judge result, returned by [`Classifier::classify_pair`].
#[derive(Debug, Clone, PartialEq)]
pub struct PairJudgment {
    pub verdict: PairVerdict,
    pub reason: String,
}

/// Pluggable backend for the pair-judge. The default
/// [`SidecarClassifier`] hits the local sidecar; tests inject a
/// [`StubClassifier`] with a hard-coded verdict table so the substrate
/// can be unit-tested without RUNPOD.
pub trait Classifier: Send + Sync {
    /// Judge whether `claim_a` (from `source_a`) and `claim_b` (from
    /// `source_b`) agree, disagree, or are orthogonal. Returns the
    /// verdict plus a short rationale.
    fn classify_pair(
        &self,
        claim_a: &str,
        source_a: &SourceCitation,
        claim_b: &str,
        source_b: &SourceCitation,
    ) -> PairJudgment;
}

/// Production sidecar-backed classifier.
///
/// RUNPOD-PENDING: live wiring requires the sidecar `/contradiction`
/// endpoint that takes a strict-JSON `{source_a, source_b}` payload and
/// returns `{classification, reason}`. The endpoint does not exist
/// today (per the plan's open question on sidecar /rerank parity); the
/// implementation here returns `Orthogonal` until the endpoint is
/// wired so wire-dark callers see no spurious contradictions.
pub struct SidecarClassifier {
    /// Sidecar base URL (e.g. `http://127.0.0.1:8100`). Held for the
    /// follow-up patch that wires the live HTTP call.
    pub sidecar_url: String,
}

impl SidecarClassifier {
    /// Construct a sidecar-backed classifier. Live calls are
    /// RUNPOD-PENDING; the default implementation returns
    /// [`PairVerdict::Orthogonal`] until wiring lands.
    #[must_use]
    pub fn new(sidecar_url: impl Into<String>) -> Self {
        Self {
            sidecar_url: sidecar_url.into(),
        }
    }
}

impl Classifier for SidecarClassifier {
    fn classify_pair(
        &self,
        _claim_a: &str,
        _source_a: &SourceCitation,
        _claim_b: &str,
        _source_b: &SourceCitation,
    ) -> PairJudgment {
        // RUNPOD-PENDING: sidecar /contradiction endpoint wiring.
        PairJudgment {
            verdict: PairVerdict::Orthogonal,
            reason: "sidecar /contradiction endpoint pending (E-48 RUNPOD-PENDING)".to_owned(),
        }
    }
}

/// Test-only stub classifier with a hard-coded verdict and reason.
/// Lives outside `#[cfg(test)]` so contract tests in sibling crates
/// can use it.
pub struct StubClassifier {
    pub forced_verdict: PairVerdict,
    pub forced_reason: String,
}

impl StubClassifier {
    /// Build a stub returning the supplied verdict + reason for every
    /// pair.
    #[must_use]
    pub fn new(verdict: PairVerdict, reason: impl Into<String>) -> Self {
        Self {
            forced_verdict: verdict,
            forced_reason: reason.into(),
        }
    }
}

impl Classifier for StubClassifier {
    fn classify_pair(
        &self,
        _claim_a: &str,
        _source_a: &SourceCitation,
        _claim_b: &str,
        _source_b: &SourceCitation,
    ) -> PairJudgment {
        PairJudgment {
            verdict: self.forced_verdict,
            reason: self.forced_reason.clone(),
        }
    }
}

/// Outcome of one O(n^2) pairwise contradiction scan.
///
/// `PartialEq` deliberately NOT derived: [`Contradiction`] embeds
/// [`SourceCitation`] which does not derive PartialEq (the
/// `SourceQuality` arm is a float bucket and fuzzy equality is not
/// useful here). Tests assert per-field equality on the contained rows.
#[derive(Debug, Clone)]
pub struct ScanOutcome {
    pub contradictions: Vec<Contradiction>,
    /// True when the scan was clamped to the last
    /// [`CONTRADICTION_PAIR_BUDGET`] sources because the input slice
    /// exceeded the cap.
    pub budget_clamped: bool,
}

/// Run an O(n^2) pairwise contradiction scan over a slice of
/// `(claim, source)` evidence rows. Materializes one
/// [`Contradiction`] per Disagree verdict; Agree / Orthogonal verdicts
/// short-circuit.
///
/// `surfaced_at_unix_ms` is stamped onto every materialized row.
///
/// The scan is deterministic for a given input + classifier (the stub
/// classifier yields predictable verdicts in tests).
pub fn scan_pairs<C: Classifier>(
    classifier: &C,
    evidence: &[(String, SourceCitation)],
    surfaced_at_unix_ms: i64,
) -> ScanOutcome {
    let (slice, budget_clamped) = if evidence.len() > CONTRADICTION_PAIR_BUDGET {
        (
            &evidence[evidence.len() - CONTRADICTION_PAIR_BUDGET..],
            true,
        )
    } else {
        (evidence, false)
    };

    let mut contradictions = Vec::new();
    for i in 0..slice.len() {
        for j in (i + 1)..slice.len() {
            let (claim_a, source_a) = &slice[i];
            let (claim_b, source_b) = &slice[j];
            let judgment = classifier.classify_pair(claim_a, source_a, claim_b, source_b);
            if matches!(judgment.verdict, PairVerdict::Disagree) {
                contradictions.push(Contradiction {
                    claim_a: claim_a.clone(),
                    source_a: source_a.clone(),
                    claim_b: claim_b.clone(),
                    source_b: source_b.clone(),
                    resolution: None,
                    prune_protected: true,
                    surfaced_at_unix_ms,
                    classification_reason: truncate(
                        &judgment.reason,
                        CLASSIFICATION_REASON_CHAR_CAP,
                    ),
                });
            }
        }
    }

    ScanOutcome {
        contradictions,
        budget_clamped,
    }
}

/// Canonical never-silently-resolved renderer line.
///
/// Produces the EXACT string the report renderer MUST surface
/// (verbatim) per the plan's WIRING_CONTRACT::stub_ban rule:
///
///     "Source A (<title> / <url>) says X; Source B (<title> / <url>)
///      says ~X; reconciliation: [unresolved]"
///
/// `X` is the first 80 chars of `claim_a`; `~X` is the first 80 chars
/// of `claim_b`. The reconciliation marker is the literal
/// `"[unresolved]"` -- callers MUST NOT substitute a chosen resolution.
#[must_use]
pub fn render_contradiction_line(contradiction: &Contradiction) -> String {
    let a_title = &contradiction.source_a.title;
    let a_url = &contradiction.source_a.url;
    let b_title = &contradiction.source_b.title;
    let b_url = &contradiction.source_b.url;
    let claim_a = truncate(&contradiction.claim_a, 80);
    let claim_b = truncate(&contradiction.claim_b, 80);
    format!(
        "Source A ({a_title} / {a_url}) says {claim_a}; Source B ({b_title} / {b_url}) says ~{claim_b}; reconciliation: [unresolved]",
    )
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    s.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research_tree::{EvidenceLevel, SourceQuality};

    fn dummy_citation(url: &str, title: &str) -> SourceCitation {
        SourceCitation {
            url: url.to_owned(),
            title: title.to_owned(),
            quality: SourceQuality::Tier2,
            evidence_level: EvidenceLevel::SingleSource,
            excerpt: "excerpt".to_owned(),
        }
    }

    #[test]
    fn disagree_materializes_prune_protected_contradiction() {
        let stub = StubClassifier::new(PairVerdict::Disagree, "FastAPI vs Litestar throughput");
        let evidence = vec![
            (
                "FastAPI fastest".to_owned(),
                dummy_citation("https://a.example.com", "Source A"),
            ),
            (
                "Litestar fastest".to_owned(),
                dummy_citation("https://b.example.com", "Source B"),
            ),
        ];
        let out = scan_pairs(&stub, &evidence, 1_700_000_000_000);
        assert_eq!(out.contradictions.len(), 1);
        assert!(out.contradictions[0].prune_protected, "Disagree MUST set prune_protected");
        assert_eq!(out.contradictions[0].surfaced_at_unix_ms, 1_700_000_000_000);
        assert!(!out.budget_clamped);
    }

    #[test]
    fn agree_yields_no_contradiction() {
        let stub = StubClassifier::new(PairVerdict::Agree, "both agree");
        let evidence = vec![
            ("c1".to_owned(), dummy_citation("https://a", "A")),
            ("c2".to_owned(), dummy_citation("https://b", "B")),
        ];
        let out = scan_pairs(&stub, &evidence, 0);
        assert!(out.contradictions.is_empty());
    }

    #[test]
    fn orthogonal_yields_no_contradiction() {
        let stub = StubClassifier::new(PairVerdict::Orthogonal, "different topics");
        let evidence = vec![
            ("c1".to_owned(), dummy_citation("https://a", "A")),
            ("c2".to_owned(), dummy_citation("https://b", "B")),
        ];
        let out = scan_pairs(&stub, &evidence, 0);
        assert!(out.contradictions.is_empty());
    }

    #[test]
    fn budget_clamps_at_50_evidence_rows() {
        let stub = StubClassifier::new(PairVerdict::Orthogonal, "stub");
        let evidence: Vec<(String, SourceCitation)> = (0..(CONTRADICTION_PAIR_BUDGET + 5))
            .map(|i| (format!("c{i}"), dummy_citation(&format!("https://{i}"), &format!("S{i}"))))
            .collect();
        let out = scan_pairs(&stub, &evidence, 0);
        assert!(out.budget_clamped);
    }

    #[test]
    fn classification_reason_truncated() {
        let big = "x".repeat(CLASSIFICATION_REASON_CHAR_CAP * 2);
        let stub = StubClassifier::new(PairVerdict::Disagree, big);
        let evidence = vec![
            ("c1".to_owned(), dummy_citation("https://a", "A")),
            ("c2".to_owned(), dummy_citation("https://b", "B")),
        ];
        let out = scan_pairs(&stub, &evidence, 0);
        assert_eq!(
            out.contradictions[0].classification_reason.chars().count(),
            CLASSIFICATION_REASON_CHAR_CAP
        );
    }

    #[test]
    fn render_contradiction_line_matches_canonical_pattern() {
        let stub = StubClassifier::new(PairVerdict::Disagree, "disagree");
        let evidence = vec![
            (
                "FastAPI fastest".to_owned(),
                dummy_citation("https://a.example.com", "Bench"),
            ),
            (
                "Litestar fastest".to_owned(),
                dummy_citation("https://b.example.com", "Counter"),
            ),
        ];
        let out = scan_pairs(&stub, &evidence, 0);
        let line = render_contradiction_line(&out.contradictions[0]);
        assert!(line.contains("Source A"));
        assert!(line.contains("Source B"));
        assert!(line.contains("reconciliation: [unresolved]"));
    }

    #[test]
    fn sidecar_classifier_returns_orthogonal_pending_wiring() {
        let sc = SidecarClassifier::new("http://127.0.0.1:8100");
        let j = sc.classify_pair(
            "a",
            &dummy_citation("u1", "t1"),
            "b",
            &dummy_citation("u2", "t2"),
        );
        assert_eq!(j.verdict, PairVerdict::Orthogonal);
    }
}
