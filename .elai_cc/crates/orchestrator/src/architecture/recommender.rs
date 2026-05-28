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
