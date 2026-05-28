//! E-11 catalog+recommender slice -- canonical SaaS archetype taxonomy.
//!
//! The 5-archetype taxonomy is the single source of truth for which
//! production-checklist subset applies to a given idea-framing output.
//! Variant names + snake_case discriminants are LOCKED by the E-50.A
//! negative-space contract guard (forbidden tokens: `SaasArchetypeV2`,
//! `SaaSArchetypeNext`, `ProductArchetype`, `BespokeArchetype`).
//!
//! The 5-archetype set is declared in `workflow/archetypes/README.md`.
//! Adding a 6th archetype REQUIRES bumping the README first AND adding a
//! matching entry to [`ArchetypeCatalog`] (catalog.rs) -- the
//! `saas_archetype_taxonomy_exhaustive` test in
//! `orchestrator/tests/saas_archetype_taxonomy.rs` fires otherwise.
//!
//! `Saas` casing (single uppercase S) is Rust-idiomatic and avoids the
//! `SaaSArchetype` lexical ambiguity. The snake_case serde discriminant
//! `"saas_dashboard"` MATCHES `ProjectClass::SaaSDashboard`'s existing
//! serde discriminant verbatim, which keeps cross-enum round-trips clean
//! when a [`crate::idea_framing::ProjectClass::SaaSDashboard`] is forwarded
//! into the recommender input path.

use serde::{Deserialize, Serialize};

/// Canonical SaaS archetype taxonomy. Exactly 5 variants per
/// `workflow/archetypes/README.md` + E-50.A negative-space contract.
///
/// snake_case wire discriminants:
///   - `saas_dashboard` -- B2B/B2C dashboard with auth + multi-tenant data
///   - `b2b_marketplace` -- two-sided marketplace, multi-tenant + billing
///   - `internal_tooling` -- single-tenant ops console, no public surface
///   - `developer_api` -- API-first product, SDK + OpenAPI surface
///   - `consumer_social` -- consumer feed / social, single-tenant per user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaasArchetype {
    /// B2B / B2C dashboard with first-party auth + multi-tenant data.
    /// Default-profile lock: RLS + secrets-scan + OWASP because Tenzai
    /// CSA Dec 2025 found 100% of vibe-coded dashboards missing CSRF.
    SaasDashboard,
    /// Two-sided marketplace: supply-side + demand-side accounts plus
    /// payments + tenant isolation. Strictest production checklist
    /// because Escape.tech 2025 found 400+ secrets exposed in
    /// marketplace builds.
    B2bMarketplace,
    /// Single-tenant internal-only tool (no public surface). Skips
    /// public-surface checks (Lighthouse / axe) but keeps secrets-scan
    /// + dependency-audit + lint pass.
    InternalTooling,
    /// API-first product: SDK + OpenAPI surface, no first-party UI.
    /// Substitutes Lighthouse with OTel/Sentry coverage probes plus
    /// LSOF port-binding sanity.
    DeveloperApi,
    /// Consumer feed / social-graph product, single-tenant per user.
    /// Includes Lighthouse (consumer UX matters) but typically does NOT
    /// require RLS (no cross-tenant data sharing).
    ConsumerSocial,
}

/// All 5 [`SaasArchetype`] discriminants in declaration order. Iteration
/// surface for tests + the catalog `iter()` traversal.
pub const ALL_SAAS_ARCHETYPES: &[SaasArchetype] = &[
    SaasArchetype::SaasDashboard,
    SaasArchetype::B2bMarketplace,
    SaasArchetype::InternalTooling,
    SaasArchetype::DeveloperApi,
    SaasArchetype::ConsumerSocial,
];

impl SaasArchetype {
    /// Canonical snake_case wire label. Stable across versions; used in
    /// SSB payloads + workflow/archetypes/<archetype>.toml manifest stems
    /// (when a per-archetype manifest exists).
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::SaasDashboard => "saas_dashboard",
            Self::B2bMarketplace => "b2b_marketplace",
            Self::InternalTooling => "internal_tooling",
            Self::DeveloperApi => "developer_api",
            Self::ConsumerSocial => "consumer_social",
        }
    }

    /// Inverse of [`SaasArchetype::as_snake_case`]. Returns `None` for an
    /// unknown discriminant so callers can fail closed on bad SSB input.
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "saas_dashboard" => Some(Self::SaasDashboard),
            "b2b_marketplace" => Some(Self::B2bMarketplace),
            "internal_tooling" => Some(Self::InternalTooling),
            "developer_api" => Some(Self::DeveloperApi),
            "consumer_social" => Some(Self::ConsumerSocial),
            _ => None,
        }
    }

    /// Human-readable display label for dashboard / CLI surfaces. Stable
    /// because the dashboard hand-authored TS mirror keys on these.
    #[must_use]
    pub fn display_label(self) -> &'static str {
        match self {
            Self::SaasDashboard => "SaaS Dashboard",
            Self::B2bMarketplace => "B2B Marketplace",
            Self::InternalTooling => "Internal Tooling",
            Self::DeveloperApi => "Developer API",
            Self::ConsumerSocial => "Consumer Social",
        }
    }
}
