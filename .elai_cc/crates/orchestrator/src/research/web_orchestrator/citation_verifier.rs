//! E-48 Tasks 8 + 9 — citation verification + sub-agent attribution lock.
//!
//! Four-stage cheapest-first citation verifier:
//!
//!   (a) URL-liveness -- HEAD request, 5s timeout. 404 / timeout /
//!       connection-error => `dead_url` outcome + dropped from final
//!       report.
//!   (b) arXiv-ID regex match `arxiv:\d{4}\.\d{4,5}` + HEAD against
//!       `arxiv.org/abs/{id}`. 404 => `invalid_arxiv` outcome +
//!       dropped.
//!   (c) CrossRef DOI regex match `10\.\d+/.+` + HEAD against
//!       `doi.org/{doi}`. 404 => `invalid_doi` outcome + dropped.
//!   (d) SemanticCite-style faithfulness -- sidecar pair-judge call
//!       "Given claim X and source-extract Y, does Y SUPPORT X?".
//!       Mismatch => `unfaithful` outcome but NOT dropped (user-visible
//!       flag instead).
//!
//! Sub-agent attribution lock (Task 9): citations are
//! `(agent_id, fetched_url, fetched_at_unix, source_tier)` tuples. The
//! lead / orchestrator may NOT invent a citation -- only the
//! SEARCH-stage producer may attach one. Enforced by the
//! [`AttributionSet`] type: lead-side `lookup` returns
//! [`AttributionLookup::Blocked`] for any url not in the producer set
//! and the producer-side emit fn records the
//! `citation_attribution_violation_blocked` SSB row.
//!
//! Compile-time invariant ([`VerifiedCitation`]): the newtype wraps a
//! [`crate::research_tree::SourceCitation`] and is UNCONSTRUCTABLE
//! outside this module (`new` is `pub(super)` so the
//! `web_orchestrator` substrate can mint them but external callers
//! cannot). Final-report code MUST accept only `VerifiedCitation` --
//! the type system enforces "no citation in final report without
//! verification."
//!
//! Stages (a) / (b) / (c) are deterministic HTTP HEAD + regex. Stage (d)
//! is sidecar-backed and RUNPOD-PENDING -- the [`FaithfulnessJudge`]
//! trait abstracts the backend so a stub can be injected for tests.
//!
//! Grounding:
//!   * arXiv:2602.06718 -- GhostCite (14-95% LLM citation hallucination
//!     baseline; URL/arXiv/DOI HEAD highest-yield defense).
//!   * arXiv:2604.03173 -- CITEVERIFIER / SemanticCite (84% acc) /
//!     CiteGuard (68%) -- the documented detectors.
//!   * arXiv:2601.18724 -- HalluCitation Matters (arxiv-ID HEAD is the
//!     highest-yield cheap defense -- grounds stage-b primacy).
//!   * simonwillison.net/2025/Jun/14 -- Anthropic multi-agent system
//!     (lead-never-cites attribution lock -- grounds Task 9).

use std::collections::HashSet;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::research_tree::SourceCitation;

/// arXiv-ID pattern: `arxiv:NNNN.NNNNN` (4 digits + dot + 4 or 5 digits).
/// Case-insensitive prefix; the digits are anchored.
pub const ARXIV_ID_PATTERN: &str = r"(?i)arxiv[:/]?\s*(\d{4})\.(\d{4,5})";

/// CrossRef DOI pattern. Permissive on the suffix so DOIs with slashes
/// and dashes are matched.
pub const DOI_PATTERN: &str = r"(?i)\b10\.\d+/[^\s)]+";

/// Faithfulness verdict from the SemanticCite-style pair-judge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaithfulnessDecision {
    /// The source extract SUPPORTS the claim.
    Support,
    /// The source extract partially supports the claim (cite-keep + flag).
    Partial,
    /// The source extract CONTRADICTS the claim (cite-keep + flag).
    Contradict,
    /// The source extract is orthogonal -- citation does not support
    /// the claim (cite-keep + flag).
    Orthogonal,
}

impl FaithfulnessDecision {
    /// Snake-case wire label for SSB telemetry.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Support => "support",
            Self::Partial => "partial",
            Self::Contradict => "contradict",
            Self::Orthogonal => "orthogonal",
        }
    }
}

/// Outcome of one 4-stage verification pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationOutcome {
    /// Stage (a) -- URL-liveness HEAD verdict. False == dead URL.
    pub url_live: bool,
    /// Stage (b) -- arXiv-ID validity verdict. `None` when no arXiv-ID
    /// was detected in the citation URL.
    pub arxiv_valid: Option<bool>,
    /// Stage (c) -- CrossRef DOI validity verdict. `None` when no DOI
    /// was detected in the citation URL.
    pub doi_valid: Option<bool>,
    /// Stage (d) -- SemanticCite-style faithfulness verdict.
    /// `None` when the faithfulness check was skipped (sidecar
    /// unavailable / RUNPOD-PENDING).
    pub faithful: Option<FaithfulnessDecision>,
}

impl VerificationOutcome {
    /// Returns `true` iff the citation should be DROPPED from the final
    /// report. Stages (a) / (b) / (c) drop; stage (d) only flags
    /// (per the plan task spec).
    #[must_use]
    pub fn must_drop(&self) -> bool {
        !self.url_live
            || matches!(self.arxiv_valid, Some(false))
            || matches!(self.doi_valid, Some(false))
    }

    /// Returns a canonical outcome label for the SSB telemetry row.
    /// Discrimination cascade: dead_url > invalid_arxiv > invalid_doi
    /// > unfaithful > pass.
    #[must_use]
    pub fn ssb_outcome_label(&self) -> &'static str {
        if !self.url_live {
            return "dead_url";
        }
        if matches!(self.arxiv_valid, Some(false)) {
            return "invalid_arxiv";
        }
        if matches!(self.doi_valid, Some(false)) {
            return "invalid_doi";
        }
        match self.faithful {
            Some(FaithfulnessDecision::Support) | None => "pass",
            Some(_) => "unfaithful",
        }
    }
}

/// Compile-time-enforced verified-citation newtype.
///
/// Constructor is `pub(super)` so only modules inside the
/// `web_orchestrator` subtree can mint instances. External callers
/// accept `VerifiedCitation` arguments and cannot fabricate them --
/// "no citation in final report without verification" is enforced by
/// Rust's privacy rules.
///
/// `PartialEq` / `Eq` deliberately NOT derived: the inner
/// [`SourceCitation`] does not derive them today (per the existing
/// research_tree shape — equality is fuzzy for SourceQuality scores).
/// Tests use field-level assertions on the public accessors instead.
#[derive(Debug, Clone)]
pub struct VerifiedCitation {
    inner: SourceCitation,
    outcome: VerificationOutcome,
    agent_id: String,
    fetched_at_unix_ms: i64,
}

impl VerifiedCitation {
    /// Construct a verified citation. `pub(super)` -- only modules
    /// inside the `web_orchestrator` subtree may call this.
    pub(super) fn new(
        inner: SourceCitation,
        outcome: VerificationOutcome,
        agent_id: impl Into<String>,
        fetched_at_unix_ms: i64,
    ) -> Self {
        Self {
            inner,
            outcome,
            agent_id: agent_id.into(),
            fetched_at_unix_ms,
        }
    }

    /// Borrow the inner [`SourceCitation`] for the final-report renderer.
    #[must_use]
    pub fn citation(&self) -> &SourceCitation {
        &self.inner
    }

    /// Borrow the verification outcome.
    #[must_use]
    pub fn outcome(&self) -> &VerificationOutcome {
        &self.outcome
    }

    /// The id of the SEARCH-stage producer that fetched this citation.
    #[must_use]
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// When the citation was fetched (Unix millis).
    #[must_use]
    pub fn fetched_at_unix_ms(&self) -> i64 {
        self.fetched_at_unix_ms
    }
}

/// Sub-agent attribution set per Task 9.
///
/// The orchestrator records the `(agent_id, fetched_url)` tuple for
/// every SEARCH-stage-produced evidence row. The lead / final report
/// renderer queries [`Self::lookup`] before emitting a citation; any
/// url not in the set is `Blocked`.
#[derive(Debug, Default, Clone)]
pub struct AttributionSet {
    /// (agent_id, url) tuples recorded at SEARCH-stage producer time.
    entries: HashSet<(String, String)>,
}

/// Lookup result from [`AttributionSet::lookup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributionLookup {
    /// The url has at least one SEARCH-stage producer in the set.
    Allowed,
    /// No SEARCH-stage producer recorded this url -- citation refused.
    Blocked,
}

impl AttributionSet {
    /// Construct an empty attribution set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a SEARCH-stage producer for a fetched url.
    pub fn record(&mut self, agent_id: impl Into<String>, url: impl Into<String>) {
        self.entries.insert((agent_id.into(), url.into()));
    }

    /// Check whether any SEARCH-stage producer has fetched the url.
    #[must_use]
    pub fn lookup(&self, url: &str) -> AttributionLookup {
        if self.entries.iter().any(|(_, recorded)| recorded == url) {
            AttributionLookup::Allowed
        } else {
            AttributionLookup::Blocked
        }
    }

    /// Number of (producer, url) tuples in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Plug-in SemanticCite-style faithfulness judge.
///
/// Default implementation [`PendingJudge`] returns
/// [`FaithfulnessDecision::Support`] without performing a live call --
/// the sidecar `/faithfulness` endpoint is RUNPOD-PENDING.
pub trait FaithfulnessJudge: Send + Sync {
    fn judge(&self, claim: &str, source_extract: &str) -> FaithfulnessDecision;
}

/// RUNPOD-PENDING default judge. Returns `Support` until the live
/// sidecar endpoint is wired so wire-dark callers do not see spurious
/// `unfaithful` flags.
pub struct PendingJudge;

impl FaithfulnessJudge for PendingJudge {
    fn judge(&self, _claim: &str, _source_extract: &str) -> FaithfulnessDecision {
        FaithfulnessDecision::Support
    }
}

/// Result of the cheap (stages a-c) URL-shape inspection, before any
/// HTTP call fires. Useful for tests that want to assert the regex
/// arms in isolation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlShapeInspection {
    pub has_arxiv_id: bool,
    pub has_doi: bool,
    pub arxiv_id: Option<String>,
    pub doi: Option<String>,
}

/// Inspect a url for arXiv-ID and DOI shape without any HTTP call.
#[must_use]
pub fn inspect_url_shape(url: &str) -> UrlShapeInspection {
    let arxiv_re = Regex::new(ARXIV_ID_PATTERN).expect("ARXIV_ID_PATTERN compiles");
    let doi_re = Regex::new(DOI_PATTERN).expect("DOI_PATTERN compiles");
    let arxiv_match = arxiv_re.captures(url);
    let arxiv_id = arxiv_match.map(|c| format!("{}.{}", &c[1], &c[2]));
    let doi_match = doi_re.find(url);
    let doi = doi_match.map(|m| m.as_str().to_owned());
    UrlShapeInspection {
        has_arxiv_id: arxiv_id.is_some(),
        has_doi: doi.is_some(),
        arxiv_id,
        doi,
    }
}

/// Builds a [`VerificationOutcome`] from already-computed stage
/// verdicts. The HTTP HEAD calls are caller-side (so the verifier
/// stays test-injectable); this fn collates the inputs into the
/// canonical outcome shape.
#[must_use]
pub fn build_outcome(
    url_live: bool,
    arxiv_valid: Option<bool>,
    doi_valid: Option<bool>,
    faithful: Option<FaithfulnessDecision>,
) -> VerificationOutcome {
    VerificationOutcome {
        url_live,
        arxiv_valid,
        doi_valid,
        faithful,
    }
}

/// Mint a verified citation -- restricted to the `web_orchestrator`
/// subtree per the compile-time invariant. External callers go
/// through [`crate::research::web_orchestrator::producers`] which
/// internally invokes this fn.
pub(super) fn mint_verified(
    citation: SourceCitation,
    outcome: VerificationOutcome,
    agent_id: impl Into<String>,
    fetched_at_unix_ms: i64,
) -> VerifiedCitation {
    VerifiedCitation::new(citation, outcome, agent_id, fetched_at_unix_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research_tree::{EvidenceLevel, SourceQuality};

    fn dummy_citation(url: &str) -> SourceCitation {
        SourceCitation {
            url: url.to_owned(),
            title: "title".to_owned(),
            quality: SourceQuality::Tier2,
            evidence_level: EvidenceLevel::SingleSource,
            excerpt: "excerpt".to_owned(),
        }
    }

    #[test]
    fn arxiv_id_pattern_extracts_canonical_form() {
        let s = inspect_url_shape("https://arxiv.org/abs/2510.05145");
        assert!(s.has_arxiv_id);
        assert_eq!(s.arxiv_id.as_deref(), Some("2510.05145"));
    }

    #[test]
    fn arxiv_id_pattern_extracts_alternative_form() {
        let s = inspect_url_shape("citing arxiv:2510.05145 inline");
        assert!(s.has_arxiv_id);
        assert_eq!(s.arxiv_id.as_deref(), Some("2510.05145"));
    }

    #[test]
    fn arxiv_id_pattern_misses_non_arxiv_urls() {
        let s = inspect_url_shape("https://docs.rs/serde");
        assert!(!s.has_arxiv_id);
    }

    #[test]
    fn doi_pattern_matches_canonical_form() {
        let s = inspect_url_shape("https://doi.org/10.1145/3641289.3641430");
        assert!(s.has_doi);
        assert!(s.doi.as_deref().expect("doi present").starts_with("10.1145/"));
    }

    #[test]
    fn must_drop_when_url_dead() {
        let o = build_outcome(false, None, None, None);
        assert!(o.must_drop());
        assert_eq!(o.ssb_outcome_label(), "dead_url");
    }

    #[test]
    fn must_drop_when_invalid_arxiv() {
        let o = build_outcome(true, Some(false), None, None);
        assert!(o.must_drop());
        assert_eq!(o.ssb_outcome_label(), "invalid_arxiv");
    }

    #[test]
    fn must_drop_when_invalid_doi() {
        let o = build_outcome(true, None, Some(false), None);
        assert!(o.must_drop());
        assert_eq!(o.ssb_outcome_label(), "invalid_doi");
    }

    #[test]
    fn unfaithful_flags_but_does_not_drop() {
        let o = build_outcome(
            true,
            Some(true),
            None,
            Some(FaithfulnessDecision::Contradict),
        );
        assert!(!o.must_drop(), "unfaithful must NOT drop -- only flag");
        assert_eq!(o.ssb_outcome_label(), "unfaithful");
    }

    #[test]
    fn pass_when_all_stages_clean() {
        let o = build_outcome(
            true,
            Some(true),
            Some(true),
            Some(FaithfulnessDecision::Support),
        );
        assert!(!o.must_drop());
        assert_eq!(o.ssb_outcome_label(), "pass");
    }

    #[test]
    fn attribution_set_blocks_uncited_urls() {
        let mut set = AttributionSet::new();
        set.record("search_agent_1", "https://arxiv.org/abs/2510.05145");
        assert_eq!(
            set.lookup("https://arxiv.org/abs/2510.05145"),
            AttributionLookup::Allowed
        );
        assert_eq!(
            set.lookup("https://attacker.example.com/fake"),
            AttributionLookup::Blocked
        );
    }

    #[test]
    fn pending_judge_returns_support() {
        let j = PendingJudge;
        assert_eq!(j.judge("claim", "extract"), FaithfulnessDecision::Support);
    }

    #[test]
    fn verified_citation_round_trips_inner() {
        let c = dummy_citation("https://arxiv.org/abs/2510.05145");
        let o = build_outcome(true, Some(true), None, Some(FaithfulnessDecision::Support));
        let v = mint_verified(c.clone(), o.clone(), "search_agent_1", 1_700_000_000_000);
        assert_eq!(v.citation().url, c.url);
        assert!(v.outcome().url_live);
        assert_eq!(v.outcome().arxiv_valid, Some(true));
        assert_eq!(v.outcome().doi_valid, None);
        assert_eq!(v.outcome().faithful, Some(FaithfulnessDecision::Support));
        assert_eq!(v.agent_id(), "search_agent_1");
        assert_eq!(v.fetched_at_unix_ms(), 1_700_000_000_000);
    }
}
