//! E-11 catalog+recommender slice -- map ProjectClass + brief text to a
//! [`SaasArchetype`] recommendation with calibrated confidence.
//!
//! Design constraints:
//!   * The recommender is **deterministic** -- same `(class, brief)`
//!     input always produces the same recommendation. The contract test
//!     `archetype_recommender_deterministic` enforces this.
//!   * Confidence is a 3-arm [`RecommendationConfidence`] enum, NOT an
//!     `f32`. GAP-3 from the E-11 integration-explorer report --
//!     mirrors the E-19 Pattern-2 + ProbeKind discriminator precedent
//!     and keeps the SSB payload discriminant scheme simple
//!     ("high" / "medium" / "low" wire labels).
//!   * The recommender ALWAYS emits a primary archetype. The
//!     low-confidence branch produces >= 3 clarifying questions drawn
//!     from the archetype's `scope_clarity_profile` so the call site
//!     (idea-framer / per-level critic) can pivot the user into a
//!     framing-stage HITL gate without losing the recommendation.
//!   * No new `ProjectClass` variants are added here. The input enum
//!     is FROZEN at 9 variants per e12.A guard 4.
//!   * The 5-keyword-class brief heuristic (GAP-4) is encoded as a
//!     `const` table -- editing it is the single canonical knob for
//!     biasing ambiguous classes.

use super::prior_art::{PriorArtRef, PriorArtRegistry};
use super::saas_archetype::SaasArchetype;
use crate::idea_framing::ProjectClass;

/// Calibrated 3-arm confidence for a recommendation. Mirror of the E-19
/// Pattern-2 + ProbeKind discriminator shape -- snake_case wire labels
/// `"high"` / `"medium"` / `"low"` are SSB-payload-compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecommendationConfidence {
    /// `(class, brief)` maps to a single archetype with no realistic
    /// ambiguity. No clarifying questions required.
    High,
    /// Recommendation is the best guess but >= 1 plausible alternative
    /// exists. Emit at least 2 clarifying questions.
    Medium,
    /// Recommendation is a heuristic guess; >= 3 clarifying questions
    /// MUST be emitted. The call site should NOT auto-advance into the
    /// archetype-specific code-gen path without user confirmation.
    Low,
}

impl RecommendationConfidence {
    /// Canonical snake_case wire label. SSB-payload-compatible.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    /// Inverse of [`RecommendationConfidence::as_snake_case`]. Returns
    /// `None` on unknown input so the SSB read path can fail closed.
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            _ => None,
        }
    }

    /// Minimum number of clarifying questions the recommender MUST
    /// surface at this confidence level. Enforced by the
    /// `archetype_recommender_low_confidence_emits_three_questions`
    /// contract test.
    #[must_use]
    pub fn min_clarifying_questions(self) -> usize {
        match self {
            Self::High => 0,
            Self::Medium => 2,
            Self::Low => 3,
        }
    }
}

/// Recommender output. Carries the primary archetype, the confidence
/// arm, the clarifying questions (may be empty at High), and the
/// prior-art entries the call site can surface alongside the
/// recommendation.
#[derive(Debug, Clone)]
pub struct ArchetypeRecommendation {
    /// Best-guess primary archetype. ALWAYS populated (never None).
    pub primary: SaasArchetype,
    /// Calibrated confidence in `primary`. Drives downstream gating.
    pub confidence: RecommendationConfidence,
    /// Clarifying questions surfaced to the user when confidence is
    /// less than [`RecommendationConfidence::High`]. Drawn from the
    /// archetype's `scope_clarity_profile` (catalog.rs).
    pub clarifying_questions: Vec<String>,
    /// Prior-art entries the call site can render alongside the
    /// recommendation. Tagged-by-archetype lookup against the
    /// `PriorArtRegistry`.
    pub prior_art_pointers: Vec<&'static PriorArtRef>,
}

// ---------------------------------------------------------------------------
// 5-keyword-class brief heuristic (GAP-4). Editing this table is the
// single canonical knob for biasing ambiguous classes.
// ---------------------------------------------------------------------------

/// Keyword class -> archetype bias rule. Each row is a (keyword,
/// archetype) pair; multiple keywords can point at the same archetype.
const KEYWORD_BIAS_TABLE: &[(&str, SaasArchetype)] = &[
    // AUTH class -> SaasDashboard (most common destination for "auth"
    // ideas where the team already has a CRUD product in mind).
    ("auth", SaasArchetype::SaasDashboard),
    ("signin", SaasArchetype::SaasDashboard),
    ("oauth", SaasArchetype::SaasDashboard),
    ("login", SaasArchetype::SaasDashboard),
    ("sso", SaasArchetype::SaasDashboard),
    // BILLING class -> B2bMarketplace (the highest-risk archetype for
    // billing; if the brief mentions billing, default to the strict
    // checklist).
    ("billing", SaasArchetype::B2bMarketplace),
    ("stripe", SaasArchetype::B2bMarketplace),
    ("subscription", SaasArchetype::B2bMarketplace),
    ("invoice", SaasArchetype::B2bMarketplace),
    ("checkout", SaasArchetype::B2bMarketplace),
    // TENANT class -> B2bMarketplace (multi-tenant is the marker).
    ("tenant", SaasArchetype::B2bMarketplace),
    ("workspace", SaasArchetype::B2bMarketplace),
    ("organization", SaasArchetype::B2bMarketplace),
    ("multi-tenant", SaasArchetype::B2bMarketplace),
    // API class -> DeveloperApi.
    ("api", SaasArchetype::DeveloperApi),
    ("endpoint", SaasArchetype::DeveloperApi),
    ("sdk", SaasArchetype::DeveloperApi),
    ("openapi", SaasArchetype::DeveloperApi),
    // FEED class -> ConsumerSocial.
    ("feed", SaasArchetype::ConsumerSocial),
    ("post", SaasArchetype::ConsumerSocial),
    ("share", SaasArchetype::ConsumerSocial),
    ("social", SaasArchetype::ConsumerSocial),
    ("follower", SaasArchetype::ConsumerSocial),
    // INTERNAL class -> InternalTooling.
    ("internal", SaasArchetype::InternalTooling),
    ("admin", SaasArchetype::InternalTooling),
    ("ops console", SaasArchetype::InternalTooling),
];

/// Apply the 5-keyword-class heuristic to a brief and return the
/// most-biased archetype, if any. Tie-break: first keyword hit wins
/// (deterministic).
fn keyword_bias(brief: &str) -> Option<SaasArchetype> {
    let lower = brief.to_ascii_lowercase();
    for (kw, archetype) in KEYWORD_BIAS_TABLE {
        if lower.contains(kw) {
            return Some(*archetype);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Recommender entry point.
// ---------------------------------------------------------------------------

/// Map an idea-framing [`ProjectClass`] + free-text brief to an
/// archetype recommendation.
///
/// Deterministic for any `(class, brief)` input. Calls the keyword-bias
/// table for ambiguous `ProjectClass` variants (`WebApp`, `DesktopApp`,
/// `GameMod`, `Other`) and falls back to the per-class default mapping
/// otherwise.
///
/// The clarifying-question count is selected by the confidence arm's
/// [`RecommendationConfidence::min_clarifying_questions`] minimum. The
/// catalog's `scope_clarity_profile` provides the question text.
#[must_use]
pub fn recommend_archetype(class: ProjectClass, brief: &str) -> ArchetypeRecommendation {
    let (primary, confidence) = match class {
        // High-confidence direct mappings.
        ProjectClass::SaaSDashboard => {
            (SaasArchetype::SaasDashboard, RecommendationConfidence::High)
        }
        ProjectClass::LibraryOrSDK | ProjectClass::CLITool => {
            (SaasArchetype::DeveloperApi, RecommendationConfidence::High)
        }
        // Medium-confidence per-class defaults.
        ProjectClass::ResearchTool => (
            SaasArchetype::InternalTooling,
            RecommendationConfidence::Medium,
        ),
        ProjectClass::ConsumerMobileApp => (
            SaasArchetype::ConsumerSocial,
            RecommendationConfidence::Medium,
        ),
        // Ambiguous classes: lean on keyword-bias before falling back to
        // best-guess + Low confidence.
        ProjectClass::WebApp => match keyword_bias(brief) {
            Some(arch) => (arch, RecommendationConfidence::Medium),
            None => (
                SaasArchetype::SaasDashboard,
                RecommendationConfidence::Low,
            ),
        },
        ProjectClass::DesktopApp => match keyword_bias(brief) {
            Some(arch) => (arch, RecommendationConfidence::Low),
            None => (
                SaasArchetype::InternalTooling,
                RecommendationConfidence::Low,
            ),
        },
        ProjectClass::GameMod | ProjectClass::Other => match keyword_bias(brief) {
            Some(arch) => (arch, RecommendationConfidence::Low),
            None => (
                SaasArchetype::ConsumerSocial,
                RecommendationConfidence::Low,
            ),
        },
    };

    let entry = super::catalog::ArchetypeCatalog::entry_for(primary);
    let min_q = confidence.min_clarifying_questions();
    let clarifying_questions: Vec<String> = entry
        .scope_clarity_profile
        .iter()
        .take(min_q.max(0))
        .map(|s| (*s).to_owned())
        .collect();

    let prior_art_pointers = PriorArtRegistry::entries_for_archetype(primary);

    ArchetypeRecommendation {
        primary,
        confidence,
        clarifying_questions,
        prior_art_pointers,
    }
}

// ---------------------------------------------------------------------------
// Tranche B-7 triple E-50 -- archetype-selected SSB emit surface.
// ---------------------------------------------------------------------------

/// Tranche B-7 triple (SCHEMA_VERSION 195) -- selection-source discriminator
/// recorded on the [`crate::ssb::SaasArchPatternEventKind::SaasArchetypeSelected`]
/// arm. Closed enum mirroring [`RecommendationConfidence`] plus the two
/// out-of-recommender sources (user override + default fallback when no
/// brief is available). Snake-case wire labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchetypeSelectionSource {
    /// Selection came from the recommender at [`RecommendationConfidence::High`].
    RecommenderHigh,
    /// Selection came from the recommender at [`RecommendationConfidence::Medium`].
    RecommenderMedium,
    /// Selection came from the recommender at [`RecommendationConfidence::Low`].
    RecommenderLow,
    /// Operator override (e.g. CLI `--archetype` flag, settings panel
    /// selection, idea-framer manual pick).
    UserOverride,
    /// No brief was available; the selection fell back to the default
    /// per-class map.
    Default,
}

impl ArchetypeSelectionSource {
    /// Canonical snake_case wire label. SSB-payload-compatible.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::RecommenderHigh => "recommender_high",
            Self::RecommenderMedium => "recommender_medium",
            Self::RecommenderLow => "recommender_low",
            Self::UserOverride => "user_override",
            Self::Default => "default",
        }
    }

    /// Inverse of [`Self::as_snake_case`]. Returns `None` on unknown
    /// input so the SSB read path can fail closed.
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "recommender_high" => Some(Self::RecommenderHigh),
            "recommender_medium" => Some(Self::RecommenderMedium),
            "recommender_low" => Some(Self::RecommenderLow),
            "user_override" => Some(Self::UserOverride),
            "default" => Some(Self::Default),
            _ => None,
        }
    }

    /// Map a [`RecommendationConfidence`] to its matching
    /// `recommender_*` source arm. Helper for the canonical
    /// recommender-emit path.
    #[must_use]
    pub fn from_confidence(c: RecommendationConfidence) -> Self {
        match c {
            RecommendationConfidence::High => Self::RecommenderHigh,
            RecommendationConfidence::Medium => Self::RecommenderMedium,
            RecommendationConfidence::Low => Self::RecommenderLow,
        }
    }
}

/// Tranche B-7 triple (SCHEMA_VERSION 195) -- env-gate predicate for the
/// E-50 archetype-selected emit surface. Default OFF. Wired through the
/// canonical [`crate::ssb::env_gate_truthy`] helper.
#[must_use]
pub fn archetype_selected_emit_enabled() -> bool {
    crate::ssb::env_gate_truthy(
        crate::ssb::ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV,
    )
}

/// Tranche B-7 triple (SCHEMA_VERSION 195) -- build the
/// [`crate::ssb::SaasArchPatternEventPayload`] for the
/// `SaasArchetypeSelected` arm. Pure function (no env-gate check, no IO)
/// so callers compose with the canonical SSB emit pipeline (e.g.
/// `crate::ssb::Ssb::write_payload`).
///
/// `confidence_bps` is the basis-points encoding of the recommender's
/// confidence (0..=10_000). High=10_000, Medium=6_000, Low=2_500;
/// `UserOverride` and `Default` carry the supplied operator scalar.
///
/// `selected_at_step` is the idea-framing step ordinal the selection was
/// emitted at; callers that emit out-of-band may pass `0`.
#[must_use]
pub fn build_archetype_selected_payload(
    rec: &ArchetypeRecommendation,
    source: ArchetypeSelectionSource,
    confidence_bps: u16,
    selected_at_step: u32,
    session_id: Option<String>,
    brief_hash: u64,
    ts_ms: i64,
) -> crate::ssb::SaasArchPatternEventPayload {
    crate::ssb::SaasArchPatternEventPayload {
        kind: crate::ssb::SaasArchPatternEventKind::SaasArchetypeSelected,
        session_id,
        archetype: rec.primary.as_snake_case().to_owned(),
        details_json: serde_json::json!({
            "source": source.as_snake_case(),
            "confidence_bps": confidence_bps,
            "selected_at_step": selected_at_step,
            "brief_hash": format!("0x{:016x}", brief_hash),
            "caller_site": "recommend_archetype_with_emit"
        }),
        ts_ms,
    }
}

/// Tranche B-7 triple (SCHEMA_VERSION 195) -- recommender + canonical
/// confidence-to-bps mapping. Mirrors [`recommend_archetype`] but
/// additionally returns the payload the caller can hand to the SSB
/// writer when [`archetype_selected_emit_enabled`] is `true`.
///
/// The function does NOT itself touch the SSB -- composition with a
/// concrete `Ssb` instance is the caller's responsibility (the
/// orchestrator owns the writer, the architecture subtree stays
/// SSB-agnostic to preserve the pure-fn surface).
///
/// Returns the recommendation AND an `Option<SaasArchPatternEventPayload>`
/// (None when the env gate is off, Some otherwise). The Pattern-2
/// envelope means the caller pays exactly one SSB row per selection.
#[must_use]
pub fn recommend_archetype_with_emit(
    class: ProjectClass,
    brief: &str,
    session_id: Option<String>,
    selected_at_step: u32,
    ts_ms: i64,
) -> (
    ArchetypeRecommendation,
    Option<crate::ssb::SaasArchPatternEventPayload>,
) {
    let rec = recommend_archetype(class, brief);
    if !archetype_selected_emit_enabled() {
        return (rec, None);
    }
    let source = ArchetypeSelectionSource::from_confidence(rec.confidence);
    let confidence_bps: u16 = match rec.confidence {
        RecommendationConfidence::High => 10_000,
        RecommendationConfidence::Medium => 6_000,
        RecommendationConfidence::Low => 2_500,
    };
    let brief_hash = crate::ssb::blake3_label_prefix(brief);
    let payload = build_archetype_selected_payload(
        &rec,
        source,
        confidence_bps,
        selected_at_step,
        session_id,
        brief_hash,
        ts_ms,
    );
    (rec, Some(payload))
}

#[cfg(test)]
mod b7_archetype_selected_tests {
    use super::*;

    #[test]
    fn source_round_trip_snake_case() {
        for s in &[
            ArchetypeSelectionSource::RecommenderHigh,
            ArchetypeSelectionSource::RecommenderMedium,
            ArchetypeSelectionSource::RecommenderLow,
            ArchetypeSelectionSource::UserOverride,
            ArchetypeSelectionSource::Default,
        ] {
            let label = s.as_snake_case();
            let parsed = ArchetypeSelectionSource::from_snake_case(label)
                .expect("round trip");
            assert_eq!(parsed, *s);
        }
    }

    #[test]
    fn confidence_to_source_mapping_stable() {
        assert_eq!(
            ArchetypeSelectionSource::from_confidence(RecommendationConfidence::High),
            ArchetypeSelectionSource::RecommenderHigh
        );
        assert_eq!(
            ArchetypeSelectionSource::from_confidence(RecommendationConfidence::Medium),
            ArchetypeSelectionSource::RecommenderMedium
        );
        assert_eq!(
            ArchetypeSelectionSource::from_confidence(RecommendationConfidence::Low),
            ArchetypeSelectionSource::RecommenderLow
        );
    }

    #[test]
    fn build_payload_shape_matches_details_contract() {
        let rec = recommend_archetype(ProjectClass::SaaSDashboard, "auth + billing");
        let payload = build_archetype_selected_payload(
            &rec,
            ArchetypeSelectionSource::RecommenderHigh,
            10_000,
            3,
            Some("s-b7-test".to_owned()),
            0xDEAD_BEEF_CAFE_BABE,
            1_700_000_195_001,
        );
        assert_eq!(
            payload.kind,
            crate::ssb::SaasArchPatternEventKind::SaasArchetypeSelected
        );
        assert_eq!(payload.archetype, "saas_dashboard");
        let details = &payload.details_json;
        assert_eq!(details["source"], "recommender_high");
        assert_eq!(details["confidence_bps"], 10_000);
        assert_eq!(details["selected_at_step"], 3);
    }

    #[test]
    fn recommend_with_emit_returns_none_when_gate_off() {
        std::env::remove_var(crate::ssb::ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV);
        let (_rec, payload) = recommend_archetype_with_emit(
            ProjectClass::WebApp,
            "auth dashboard",
            None,
            0,
            1_700_000_195_002,
        );
        assert!(payload.is_none());
    }
}
