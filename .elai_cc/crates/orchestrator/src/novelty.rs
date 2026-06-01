//! D-15 — Discovery-vs-search labeling (`NoveltyClass`).
//!
//! Every retained reasoning-template result (Mode A or Mode D output)
//! gets one of three closed labels:
//!
//!   * [`NoveltyClass::Search`]                — high overlap with the
//!     DKG and the archive's near-duplicate fingerprint; "we have seen
//!     this template before, modulo whitespace".
//!   * [`NoveltyClass::DistantSearch`]         — composition of known
//!     primitives but no near-duplicate match; novel arrangement of
//!     things already in the knowledge graph.
//!   * [`NoveltyClass::DiscoveryCandidate`]    — references primitives
//!     not present in the DKG and far from every archived template;
//!     flagged for human review per the real-problem §1 Mode C honesty
//!     layer.
//!
//! The classifier is a **pure deterministic predicate** over
//! [`NoveltyEvidence`] — no I/O, no clock, no LLM call. Thresholds live
//! in [`NoveltyConfig`] so the production caller can tune the
//! search/distant/discovery cut-off points without touching the
//! predicate logic.
//!
//! Grounding sources (per CLAUDE.md grounded-by-default rule):
//!   * arXiv:2406.05761 — *LLM Novelty Detection*: three-bucket
//!     classification (familiar / recombination / out-of-distribution)
//!     is the empirically motivated coarseness for human routing; finer
//!     splits did not improve reviewer agreement.
//!   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
//!     §1 Mode C (lines 68-89) + §4.8 (lines 310-328) — the
//!     `NoveltyClass` taxonomy and the "system never claims discovery
//!     autonomously; surfaces candidates for human ratification" rule.
//!   * docs/thefinalglance/D-plan/D-15_novelty_class_labeling.md (the
//!     full plan; this module ships the orchestrator-scoped MVP).
//!
//! Scope (this module ships the orchestrator-scoped MVP per the
//! ORCHESTRATE brief):
//!   * pure types + pure [`classify`] predicate;
//!   * an SSB-emitting producer ([`emit_novelty_classified`]);
//!   * an in-process [`VecNoveltyClassifiedEmitter`] used by tests +
//!     the integration suite;
//!   * a typed projection of the recent SSB rows for the dashboard.
//!
//! Out-of-scope (deferred to a later D-15 follow-up; see the plan):
//!   * embedding-based [`near_duplicate_check`] against the live DKG;
//!     the MVP accepts a pre-computed `near_duplicate_threshold` scalar
//!     in [`NoveltyEvidence`] supplied by the caller.
//!   * `workflow/sessions/{id}/discovery-queue/*.toml` artifact
//!     emission. The MVP only emits the SSB row; the queue-file writer
//!     is a follow-up.
//!   * GroundingMode dispatch_by_mode wiring (D-10 integration).
//!
//! No new external dep. blake3, tracing, thiserror, native async-fn in
//! trait — per the CLAUDE.md anti-stale-training rule.

use std::pin::Pin;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::archive_chain::ArchiveChain;
use crate::reasoning_template::ReasoningTemplate;
use crate::ssb::NoveltyClassifiedPayload;

// ---------------------------------------------------------------------------
// Closed taxonomy
// ---------------------------------------------------------------------------

/// Closed three-bucket novelty taxonomy. L3-frozen — adding a variant is
/// a plan-level schema change (a fourth bucket would also need a
/// matching arXiv:2406.05761-style empirical motivation; the MVP keeps
/// the published count at three).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoveltyClass {
    /// High DKG overlap AND near-duplicate fingerprint above the
    /// `near_duplicate_high` threshold. The result is "we have already
    /// done this; the template-archive already contains it modulo
    /// whitespace".
    Search,
    /// Some DKG matches but below the `search_dkg_min` floor, OR
    /// near-duplicate fingerprint between `near_duplicate_low` and
    /// `near_duplicate_high`. Composition of known primitives without
    /// a near-duplicate template.
    DistantSearch,
    /// Zero DKG matches AND near-duplicate fingerprint below the
    /// `near_duplicate_low` threshold. The system never claims
    /// discovery autonomously — these candidates surface for human
    /// ratification per real-problem §1 Mode C.
    DiscoveryCandidate,
}

impl NoveltyClass {
    /// Stable snake-case wire label (matches the serde tag).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::DistantSearch => "distant_search",
            Self::DiscoveryCandidate => "discovery_candidate",
        }
    }

    /// Every variant in declaration order. Used by the dashboard panel
    /// + the histogram to render the class list without hard-coding.
    #[must_use]
    pub const fn all() -> [NoveltyClass; 3] {
        [
            Self::Search,
            Self::DistantSearch,
            Self::DiscoveryCandidate,
        ]
    }
}

// ---------------------------------------------------------------------------
// Evidence + Config
// ---------------------------------------------------------------------------

/// Caller-supplied evidence bundle. All fields are scalars the caller
/// has already computed against the live DKG / ArchiveChain; the MVP
/// classifier does NOT walk the graph itself (the embedding-vs-DKG
/// query is deferred per the module-level out-of-scope list).
///
/// Field semantics:
///   * `dkg_match_count` — number of DKG symbols the template body
///     references that also exist in the active symbol graph. Zero =
///     references a primitive the graph does not know.
///   * `near_duplicate_threshold` — cosine similarity in `[0.0, 1.0]`
///     against the closest archived template body. `1.0` = literal
///     match; `0.0` = completely orthogonal.
///   * `dist_to_archive_centroid` — Euclidean distance from the
///     template embedding to the archive's centroid. Larger = farther
///     from the "typical" archived template. Currently informational
///     only (the predicate does not use it directly); reserved for the
///     follow-up where the centroid distance becomes part of the
///     decision boundary.
///   * `sigterm_overlap` — Jaccard overlap of significant terms (after
///     stopword removal) between the new template body and the
///     archive's term frequency profile. Again informational at MVP;
///     reserved for the follow-up.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoveltyEvidence {
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub dist_to_archive_centroid: f64,
    pub sigterm_overlap: f64,
}

impl NoveltyEvidence {
    /// Construct an evidence bundle, clamping the unit-interval fields
    /// to `[0.0, 1.0]`. Non-finite values collapse to `0.0`
    /// (maximally novel — the predicate routes to `DiscoveryCandidate`
    /// when the cosine similarity is zero).
    #[must_use]
    pub fn new(
        dkg_match_count: u32,
        near_duplicate_threshold: f64,
        dist_to_archive_centroid: f64,
        sigterm_overlap: f64,
    ) -> Self {
        Self {
            dkg_match_count,
            near_duplicate_threshold: clamp_unit(near_duplicate_threshold),
            dist_to_archive_centroid: if dist_to_archive_centroid.is_finite()
                && dist_to_archive_centroid >= 0.0
            {
                dist_to_archive_centroid
            } else {
                0.0
            },
            sigterm_overlap: clamp_unit(sigterm_overlap),
        }
    }
}

fn clamp_unit(x: f64) -> f64 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// CheckerResult -> NoveltyEvidence projection
// ---------------------------------------------------------------------------

/// D-15 closure: deterministic projection from a D-11
/// [`crate::checker_registry::CheckerResult`] into the shipped
/// [`NoveltyEvidence`] shape. Closes the "missing projection caller"
/// gap reported in
/// `workflow/sessions/s-25120/cross-check-recheck/D-15_novelty_class_labeling-report.md`
/// — the D-11 verifier substrate produces `CheckerResult`s; the D-15
/// classifier consumes `NoveltyEvidence`. This function is the bridge.
///
/// The mapping is intentionally coarse: `CheckerResult` is a closed
/// 4-variant enum with no numeric payload, so the projection emits a
/// fixed scalar per variant that the predicate then routes through
/// the standard 3-bucket logic.
///
///   * `Pass`          -> `(dkg_match_count = 8, near_duplicate = 0.9)`
///                        — a passing checker is strong evidence the
///                        template recapitulates known content, which
///                        the predicate routes to `Search` under the
///                        default `NoveltyConfig`.
///   * `Fail`          -> `(dkg_match_count = 3, near_duplicate = 0.5)`
///                        — a failing checker says the template
///                        references something the checker knew but
///                        the candidate got wrong, which the
///                        predicate routes to `DistantSearch`.
///   * `Inapplicable`  -> `(dkg_match_count = 0, near_duplicate = 0.1)`
///                        — the checker did not know the references,
///                        so the predicate routes to
///                        `DiscoveryCandidate`.
///   * `Error`         -> `(0, 0.0, 0.0, 0.0)` — infrastructure
///                        failure; the predicate routes to
///                        `DiscoveryCandidate` as the conservative
///                        default (no evidence the result is known).
///
/// Pure function — no I/O, no clock, no allocation. The scalars are
/// caller-tunable in a follow-up only by changing this function (the
/// predicate is invariant). See workflow/reasoning/D-15-decisions.md
/// DECISION-4 (signature preserved; projection added separately).
#[must_use]
pub fn novelty_evidence_from_checker_result(
    result: &crate::checker_registry::CheckerResult,
) -> NoveltyEvidence {
    use crate::checker_registry::CheckerResult;
    match result {
        CheckerResult::Pass => NoveltyEvidence::new(8, 0.9, 0.0, 0.9),
        CheckerResult::Fail { .. } => NoveltyEvidence::new(3, 0.5, 0.0, 0.5),
        CheckerResult::Inapplicable { .. } => {
            NoveltyEvidence::new(0, 0.1, 0.0, 0.1)
        }
        CheckerResult::Error { .. } => NoveltyEvidence::new(0, 0.0, 0.0, 0.0),
    }
}

/// Caller-tunable thresholds for the [`classify`] predicate. Defaults
/// follow the D-15 plan (§4.8 lines 310-328) — five DKG matches with a
/// 0.8 near-duplicate score routes to [`NoveltyClass::Search`]; zero
/// matches with a sub-0.4 score routes to
/// [`NoveltyClass::DiscoveryCandidate`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoveltyConfig {
    /// Inclusive lower bound on `dkg_match_count` for the
    /// [`NoveltyClass::Search`] arm.
    pub search_dkg_min: u32,
    /// Inclusive lower bound on `near_duplicate_threshold` for the
    /// [`NoveltyClass::Search`] arm.
    pub near_duplicate_high: f64,
    /// Strict upper bound on `near_duplicate_threshold` for the
    /// [`NoveltyClass::DiscoveryCandidate`] arm (and the lower edge of
    /// [`NoveltyClass::DistantSearch`]'s near-duplicate band).
    pub near_duplicate_low: f64,
}

impl Default for NoveltyConfig {
    fn default() -> Self {
        Self {
            search_dkg_min: 5,
            near_duplicate_high: 0.8,
            near_duplicate_low: 0.4,
        }
    }
}

impl NoveltyConfig {
    /// Validate the thresholds. Returns the same value on Ok; this is
    /// the cheap precondition runtime callers can use before holding
    /// the config across a long-running classification batch.
    pub fn validated(self) -> Result<Self, NoveltyConfigError> {
        if !(0.0..=1.0).contains(&self.near_duplicate_high)
            || !(0.0..=1.0).contains(&self.near_duplicate_low)
        {
            return Err(NoveltyConfigError::OutOfRangeThreshold);
        }
        if self.near_duplicate_low > self.near_duplicate_high {
            return Err(NoveltyConfigError::InvertedThresholds);
        }
        Ok(self)
    }
}

/// Errors raised when a [`NoveltyConfig`] is rejected.
#[derive(Debug, thiserror::Error)]
pub enum NoveltyConfigError {
    #[error(
        "near_duplicate_high / near_duplicate_low must lie in [0.0, 1.0]"
    )]
    OutOfRangeThreshold,
    #[error(
        "near_duplicate_low must be <= near_duplicate_high (got low > high)"
    )]
    InvertedThresholds,
}

// ---------------------------------------------------------------------------
// Pure predicate
// ---------------------------------------------------------------------------

/// Classify `evidence` against `config`. Pure function — no I/O, no
/// clock, no allocation.
///
/// Decision rules (per the D-15 plan task 2):
///   * `dkg_match_count >= search_dkg_min` AND
///     `near_duplicate_threshold >= near_duplicate_high`
///       -> [`NoveltyClass::Search`]
///   * `dkg_match_count == 0` AND
///     `near_duplicate_threshold < near_duplicate_low`
///       -> [`NoveltyClass::DiscoveryCandidate`]
///   * otherwise -> [`NoveltyClass::DistantSearch`]
///
/// The `template` + `archive` arguments are accepted for ABI stability
/// — production callers will use them when the centroid-distance arm
/// lands. The MVP predicate does not read them; the tests verify this
/// by passing in synthetic templates that would route incorrectly if
/// the predicate accidentally walked the chain.
#[must_use]
pub fn classify(
    _template: &ReasoningTemplate,
    _archive: &ArchiveChain,
    evidence: NoveltyEvidence,
    config: NoveltyConfig,
) -> NoveltyClass {
    if evidence.dkg_match_count >= config.search_dkg_min
        && evidence.near_duplicate_threshold >= config.near_duplicate_high
    {
        return NoveltyClass::Search;
    }
    if evidence.dkg_match_count == 0
        && evidence.near_duplicate_threshold < config.near_duplicate_low
    {
        return NoveltyClass::DiscoveryCandidate;
    }
    NoveltyClass::DistantSearch
}

// ---------------------------------------------------------------------------
// SSB producer surface
// ---------------------------------------------------------------------------

/// Producer-side SSB emitter for `novelty_classified` rows. Mirrors
/// [`crate::conformal_calibration::ConformalRefusalDecidedEmitter`] in
/// shape so production callers can share a single `Ssb`-writer adapter
/// under both traits.
///
/// Native async-fn-in-trait via boxed-future erasure (the trait must
/// remain `dyn`-compatible because the producer is held behind a
/// reference in [`emit_novelty_classified`]).
pub trait NoveltyClassifiedEmitter: Send + Sync {
    fn emit(
        &self,
        payload: NoveltyClassifiedPayload,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that records every `novelty_classified` payload
/// behind a `tokio::sync::Mutex` (no `std::sync::Mutex` held across
/// `.await` per the CLAUDE.md anti-stale-training rule).
#[derive(Default)]
pub struct VecNoveltyClassifiedEmitter {
    rows: tokio::sync::Mutex<Vec<NoveltyClassifiedPayload>>,
}

impl VecNoveltyClassifiedEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<NoveltyClassifiedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl NoveltyClassifiedEmitter for VecNoveltyClassifiedEmitter {
    fn emit(
        &self,
        payload: NoveltyClassifiedPayload,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// Classify `template` against `archive` + `evidence`, emit one
/// `novelty_classified` SSB row, and return the typed
/// [`NoveltyClass`].
///
/// `now_ms` is injected (no clock read inside the producer) per the
/// entropy-quarantine rule — matches
/// [`crate::conformal_calibration::compute_tau_and_emit`] and the rest
/// of the D-plan producers.
///
/// A failed SSB emission is logged via `tracing::warn` but does NOT
/// override the classification result — the predicate verdict is the
/// load-bearing piece; the row is the durable audit signal.
pub async fn emit_novelty_classified<E: NoveltyClassifiedEmitter + ?Sized>(
    template: &ReasoningTemplate,
    archive: &ArchiveChain,
    evidence: NoveltyEvidence,
    config: NoveltyConfig,
    emitter: &E,
    now_ms: i64,
) -> NoveltyClass {
    let class = classify(template, archive, evidence, config);
    // FINAL TRANCHE B4 / D-15 -- DiscoveryCandidate classifications enter
    // the lifecycle in `Queued` state; the D-13 falsifiability follow-up
    // emits a sibling row tagged with the same template_id and either
    // `Ratified` or `Rejected`. Non-DiscoveryCandidate classes carry
    // `None` (lifecycle does not apply to Search / DistantSearch).
    let candidate_status = match class {
        NoveltyClass::DiscoveryCandidate => {
            Some(crate::ssb::DiscoveryCandidateStatus::Queued)
        }
        NoveltyClass::Search | NoveltyClass::DistantSearch => None,
    };
    let payload = NoveltyClassifiedPayload {
        template_id: template.id.as_hex(),
        class: class.as_str().to_string(),
        dkg_match_count: evidence.dkg_match_count,
        near_duplicate_threshold: evidence.near_duplicate_threshold,
        ts_ms: now_ms,
        candidate_status,
    };
    if let Err(e) = emitter.emit(payload).await {
        warn!(
            template_id = %template.id.as_hex(),
            class = class.as_str(),
            error = %e,
            "emit_novelty_classified: SSB emission failed (classification still applies)"
        );
    }
    class
}

// ---------------------------------------------------------------------------
// Read-surface projection — typed rows the dashboard consumes
// ---------------------------------------------------------------------------

/// Typed projection of a single `novelty_classified` SSB row.
///
/// FINAL TRANCHE B4 / D-15 -- `candidate_status` is the optional lifecycle
/// stage of a discovery candidate; carries snake-case wire labels
/// (`"queued" | "ratified" | "rejected"`). `None` for legacy rows
/// pre-dating the lifecycle field + for non-DiscoveryCandidate classes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyClassifiedRow {
    pub template_id: String,
    pub class: String,
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub ts_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_status: Option<String>,
}

/// D-15 read surface — the most recent N `novelty_classified` SSB rows
/// the dashboard `NoveltyPanel` renders.
#[must_use]
pub fn recent_novelty_rows(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<NoveltyClassifiedRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("novelty_classified", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            warn!(
                action_type = "novelty_classified",
                error = %e,
                "recent_novelty_rows: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

/// D-15 read surface — per-class counts across the most recent N
/// `novelty_classified` rows. Used by the dashboard to render the
/// distribution pie chart without re-walking the row list in JS.
#[must_use]
pub fn novelty_distribution(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> NoveltyDistribution {
    let rows = recent_novelty_rows(viewer, limit);
    let mut search = 0_u32;
    let mut distant = 0_u32;
    let mut discovery = 0_u32;
    for r in &rows {
        match r.class.as_str() {
            "search" => search = search.saturating_add(1),
            "distant_search" => distant = distant.saturating_add(1),
            "discovery_candidate" => discovery = discovery.saturating_add(1),
            _ => {}
        }
    }
    NoveltyDistribution {
        total: u32::try_from(rows.len()).unwrap_or(u32::MAX),
        search,
        distant_search: distant,
        discovery_candidate: discovery,
    }
}

/// Per-class count snapshot for the dashboard distribution chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyDistribution {
    pub total: u32,
    pub search: u32,
    pub distant_search: u32,
    pub discovery_candidate: u32,
}

fn detail_to_row(d: crate::ssb_viewer::SsbActionDetail) -> Option<NoveltyClassifiedRow> {
    if d.action_type.as_str() != "novelty_classified" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer.get("NoveltyClassified").and_then(|v| v.as_object()) {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(NoveltyClassifiedRow {
        template_id: obj
            .get("template_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        class: obj
            .get("class")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        dkg_match_count: obj
            .get("dkg_match_count")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        near_duplicate_threshold: obj
            .get("near_duplicate_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        candidate_status: obj
            .get("candidate_status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reasoning_template::{
        compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate, ReasoningTemplate,
        TemplateBody, TemplateId, VerifierKind,
    };
    use std::path::PathBuf;

    fn make_template(body: &str) -> ReasoningTemplate {
        let bounded = TemplateBody::try_new(body).expect("body fits TEMPLATE_BODY_BOUND");
        let hash = compute_content_hash(bounded.as_str(), None);
        ReasoningTemplate {
            id: TemplateId::new_v4(),
            domain: Domain::Code,
            pattern_kind: PatternKind::Decomposition,
            template_body: bounded,
            eligibility_predicate: Predicate::Always,
            verifier_required: VerifierKind::Refuse,
            content_hash: hash,
            created_at_ms: 1_700_000_000_000,
            parent_id: None,
            archive_status: ArchiveStatus::Promoted,
        }
    }

    fn empty_archive() -> ArchiveChain {
        ArchiveChain::new(PathBuf::from("test-archive.jsonl"))
    }

    #[test]
    fn evidence_clamps_unit_intervals_and_distance() {
        let e = NoveltyEvidence::new(3, 1.5, -2.0, f64::NAN);
        assert_eq!(e.dkg_match_count, 3);
        assert_eq!(e.near_duplicate_threshold, 1.0);
        assert_eq!(e.dist_to_archive_centroid, 0.0);
        assert_eq!(e.sigterm_overlap, 0.0);

        let e2 = NoveltyEvidence::new(0, f64::INFINITY, 7.5, -0.3);
        assert_eq!(e2.near_duplicate_threshold, 0.0);
        assert!((e2.dist_to_archive_centroid - 7.5).abs() < 1e-9);
        assert_eq!(e2.sigterm_overlap, 0.0);
    }

    #[test]
    fn config_default_is_search_5_high_0p8_low_0p4() {
        let c = NoveltyConfig::default();
        assert_eq!(c.search_dkg_min, 5);
        assert!((c.near_duplicate_high - 0.8).abs() < 1e-9);
        assert!((c.near_duplicate_low - 0.4).abs() < 1e-9);
        c.validated().expect("default config is valid");
    }

    #[test]
    fn config_validation_rejects_inverted_thresholds() {
        let bad = NoveltyConfig {
            search_dkg_min: 5,
            near_duplicate_high: 0.3,
            near_duplicate_low: 0.7,
        };
        let err = bad.validated().expect_err("inverted must fail");
        assert!(matches!(err, NoveltyConfigError::InvertedThresholds));
    }

    #[test]
    fn config_validation_rejects_out_of_range() {
        let bad = NoveltyConfig {
            search_dkg_min: 5,
            near_duplicate_high: 1.5,
            near_duplicate_low: 0.4,
        };
        let err = bad.validated().expect_err("out-of-range must fail");
        assert!(matches!(err, NoveltyConfigError::OutOfRangeThreshold));
    }

    #[test]
    fn high_match_high_duplicate_classifies_as_search() {
        let t = make_template("body-search");
        let a = empty_archive();
        let e = NoveltyEvidence::new(10, 0.95, 0.1, 0.9);
        let class = classify(&t, &a, e, NoveltyConfig::default());
        assert_eq!(class, NoveltyClass::Search);
    }

    #[test]
    fn medium_match_medium_duplicate_classifies_as_distant_search() {
        let t = make_template("body-distant");
        let a = empty_archive();
        let e = NoveltyEvidence::new(3, 0.6, 0.5, 0.5);
        let class = classify(&t, &a, e, NoveltyConfig::default());
        assert_eq!(class, NoveltyClass::DistantSearch);
    }

    #[test]
    fn zero_match_low_duplicate_classifies_as_discovery_candidate() {
        let t = make_template("body-discovery");
        let a = empty_archive();
        let e = NoveltyEvidence::new(0, 0.1, 1.4, 0.05);
        let class = classify(&t, &a, e, NoveltyConfig::default());
        assert_eq!(class, NoveltyClass::DiscoveryCandidate);
    }

    #[test]
    fn boundary_high_threshold_with_low_match_routes_to_distant() {
        // 4 DKG matches with a 0.95 near-duplicate score: passes the
        // duplicate floor but fails the dkg_min, so the Search arm
        // does not fire. Also fails the DiscoveryCandidate arm because
        // dkg_match_count != 0. Falls through to DistantSearch.
        let t = make_template("body-boundary");
        let a = empty_archive();
        let e = NoveltyEvidence::new(4, 0.95, 0.0, 0.0);
        let class = classify(&t, &a, e, NoveltyConfig::default());
        assert_eq!(class, NoveltyClass::DistantSearch);
    }

    #[test]
    fn boundary_zero_match_at_low_threshold_routes_to_distant() {
        // 0 DKG matches with near_duplicate_threshold == near_duplicate_low
        // (the strict `<` boundary in the DiscoveryCandidate arm): falls
        // through to DistantSearch.
        let t = make_template("body-edge");
        let a = empty_archive();
        let e = NoveltyEvidence::new(0, 0.4, 0.0, 0.0);
        let class = classify(&t, &a, e, NoveltyConfig::default());
        assert_eq!(class, NoveltyClass::DistantSearch);
    }

    #[test]
    fn class_as_str_round_trips_and_all_lists_three() {
        let all = NoveltyClass::all();
        assert_eq!(all.len(), 3);
        assert_eq!(NoveltyClass::Search.as_str(), "search");
        assert_eq!(NoveltyClass::DistantSearch.as_str(), "distant_search");
        assert_eq!(
            NoveltyClass::DiscoveryCandidate.as_str(),
            "discovery_candidate"
        );
    }

    #[test]
    fn novelty_evidence_from_checker_result_routes_each_variant() {
        use crate::checker_registry::CheckerResult;

        let t = make_template("body-projection");
        let a = empty_archive();
        let cfg = NoveltyConfig::default();

        // Pass -> Search
        let e_pass =
            novelty_evidence_from_checker_result(&CheckerResult::Pass);
        assert_eq!(e_pass.dkg_match_count, 8);
        assert!((e_pass.near_duplicate_threshold - 0.9).abs() < 1e-9);
        assert_eq!(
            classify(&t, &a, e_pass, cfg),
            NoveltyClass::Search
        );

        // Fail -> DistantSearch
        let e_fail = novelty_evidence_from_checker_result(&CheckerResult::Fail {
            reason: "checker rejected".into(),
        });
        assert_eq!(e_fail.dkg_match_count, 3);
        assert!((e_fail.near_duplicate_threshold - 0.5).abs() < 1e-9);
        assert_eq!(
            classify(&t, &a, e_fail, cfg),
            NoveltyClass::DistantSearch
        );

        // Inapplicable -> DiscoveryCandidate (zero matches, sub-0.4
        // duplicate).
        let e_inappl = novelty_evidence_from_checker_result(
            &CheckerResult::Inapplicable {
                reason: "wrong claim type".into(),
            },
        );
        assert_eq!(e_inappl.dkg_match_count, 0);
        assert!((e_inappl.near_duplicate_threshold - 0.1).abs() < 1e-9);
        assert_eq!(
            classify(&t, &a, e_inappl, cfg),
            NoveltyClass::DiscoveryCandidate
        );

        // Error -> DiscoveryCandidate (conservative — zero floor).
        let e_err =
            novelty_evidence_from_checker_result(&CheckerResult::Error {
                source: "subprocess timed out".into(),
            });
        assert_eq!(e_err.dkg_match_count, 0);
        assert!(e_err.near_duplicate_threshold.abs() < 1e-9);
        assert_eq!(
            classify(&t, &a, e_err, cfg),
            NoveltyClass::DiscoveryCandidate
        );
    }

    #[test]
    fn novelty_evidence_from_checker_result_is_deterministic() {
        use crate::checker_registry::CheckerResult;
        // Pure function — same input must yield identical evidence on
        // repeat calls (no clock, no PRNG, no I/O).
        let r = CheckerResult::Fail {
            reason: "x".into(),
        };
        let a = novelty_evidence_from_checker_result(&r);
        let b = novelty_evidence_from_checker_result(&r);
        assert_eq!(a, b);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_novelty_classified_records_payload() {
        let t = make_template("body-emit");
        let a = empty_archive();
        let e = NoveltyEvidence::new(7, 0.9, 0.0, 0.0);
        let emitter = VecNoveltyClassifiedEmitter::new();
        let class = emit_novelty_classified(
            &t,
            &a,
            e,
            NoveltyConfig::default(),
            &emitter,
            1_700_000_900_000,
        )
        .await;
        assert_eq!(class, NoveltyClass::Search);
        assert_eq!(emitter.len().await, 1);
        let rows = emitter.rows().await;
        assert_eq!(rows[0].class, "search");
        assert_eq!(rows[0].dkg_match_count, 7);
        assert!((rows[0].near_duplicate_threshold - 0.9).abs() < 1e-9);
        assert_eq!(rows[0].ts_ms, 1_700_000_900_000);
    }
}
