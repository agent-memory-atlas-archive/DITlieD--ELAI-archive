//! E-11 catalog+recommender slice -- per-archetype catalog of production
//! defaults, multi-tenancy model, framework profile pointers, and the
//! [`SecurityCheckId`] bitset that the production-grade auditor applies
//! when this archetype is the catalog-selected one.
//!
//! Constraint discipline:
//!   * e12.A guard 4: NO inline framework version pins. The
//!     `default_framework_profile` field carries a `'static` TOML stem
//!     (e.g. `"nextjs15"`) that resolves at runtime to
//!     `workflow/archetypes/<stem>.toml`. Version pins live in those
//!     TOML manifests, never inline here.
//!   * e50.A guard 2-4: no destructive DDL emitters, no untenanted
//!     queries, no bespoke billing adapters in catalog data. The
//!     `multi_tenancy_model` field is enum-only -- raw SQL strings
//!     would violate the guard.
//!
//! Per-archetype `production_checklist_subset` mapping (GAP-2 from
//! E-11 integration-explorer report):
//!
//!   * `SaasDashboard`      -> 13/15 (omit Lighthouse if no public UI, omit
//!                                    CargoMachete by default)
//!   * `B2bMarketplace`     -> 15/15 (strictest, multi-tenant + billing)
//!   * `InternalTooling`    -> 11/15 (omit Lighthouse + Sqlite3Rls + OTel
//!                                    + Sentry if single-tenant ops console)
//!   * `DeveloperApi`       -> 13/15 (omit Lighthouse if API-only, omit
//!                                    CargoMachete)
//!   * `ConsumerSocial`     -> 11/15 (Lighthouse YES, Sqlite3Rls NO since
//!                                    single-tenant per user, omit
//!                                    CargoAudit + CargoMachete + Clippy if
//!                                    Node-only)
//!
//! The exact bitset for each archetype is the canonical source-of-truth
//! and is contract-tested in
//! `crates/contracts/tests/e11_b_saas_prior_art_catalog_contract.rs`.

use super::saas_archetype::SaasArchetype;
use crate::framework_profile::SecurityCheckId;

/// Authentication scheme baked into the per-archetype catalog default.
/// Concrete adapters live downstream (sibling slices); catalog only
/// records the canonical choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthScheme {
    /// First-party Google OAuth (default for SaasDashboard / ConsumerSocial).
    OauthGoogle,
    /// First-party GitHub OAuth (default for DeveloperApi).
    OauthGithub,
    /// OIDC via WorkOS / Auth0 (default for B2bMarketplace).
    OidcWorkOs,
    /// Email magic-link (default for InternalTooling -- minimum-blast
    /// internal-only flow).
    EmailMagicLink,
    /// API-key only (DeveloperApi alternate; rarely the catalog default).
    ApiKey,
}

impl AuthScheme {
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::OauthGoogle => "oauth_google",
            Self::OauthGithub => "oauth_github",
            Self::OidcWorkOs => "oidc_workos",
            Self::EmailMagicLink => "email_magic_link",
            Self::ApiKey => "api_key",
        }
    }
}

/// Multi-tenancy isolation model. Enum-only by design -- a raw SQL
/// shape here would violate e50.A guard 2 (`raw_db_query_no_tenant`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MultiTenancyModel {
    /// One tenant per deployment. InternalTooling default.
    SingleTenant,
    /// One Postgres schema per tenant. Strong isolation, harder to
    /// migrate -- B2bMarketplace alternate.
    SchemaPerTenant,
    /// Single schema, every tenanted table gates on Postgres RLS
    /// policy. SaasDashboard + B2bMarketplace default.
    RowLevelSecurity,
    /// Per-tenant database (rare; for compliance-heavy customers).
    SeparateDatabase,
}

impl MultiTenancyModel {
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::SingleTenant => "single_tenant",
            Self::SchemaPerTenant => "schema_per_tenant",
            Self::RowLevelSecurity => "row_level_security",
            Self::SeparateDatabase => "separate_database",
        }
    }

    /// Returns true when the model requires the RLS policy probe in the
    /// production-grade checklist (Sqlite3Rls runner gates on this).
    #[must_use]
    pub fn requires_rls_probe(self) -> bool {
        matches!(self, Self::RowLevelSecurity | Self::SchemaPerTenant)
    }
}

/// One [`SaasArchetype`]'s catalog entry. All fields are `'static`
/// because the entire catalog table is a `const` match arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchetypeEntry {
    /// Which archetype this entry describes.
    pub archetype: SaasArchetype,
    /// Default first-party auth scheme.
    pub default_auth_scheme: AuthScheme,
    /// Default multi-tenancy isolation model.
    pub multi_tenancy_model: MultiTenancyModel,
    /// Stem of the canonical framework manifest under
    /// `workflow/archetypes/<stem>.toml`. NO version literal here --
    /// the TOML carries pins.
    pub default_framework_profile: &'static str,
    /// Production-grade checklist subset applied by the
    /// [`crate::auditor_mode::AuditorMode::ProductionGrade`] runner
    /// when this archetype is selected.
    pub production_checklist_subset: ProductionChecklistSubset,
    /// Scope-clarity rubric questions surfaced by the idea-critic when
    /// the recommender returns this archetype with low confidence.
    /// Exactly 4 entries per the report's 4-point rubric.
    pub scope_clarity_profile: &'static [&'static str],
}

/// Bitset-style wrapper over which of the 15 [`SecurityCheckId`]
/// variants apply to a given archetype. Stored as a `&'static [SecurityCheckId]`
/// to keep ordering deterministic for tests and for the dashboard
/// rendering of the per-archetype subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductionChecklistSubset {
    /// The subset of [`SecurityCheckId`] variants the archetype-aware
    /// production-grade auditor runs. Order matches the
    /// production_checklist::run_all dispatch order.
    pub checks: &'static [SecurityCheckId],
}

impl ProductionChecklistSubset {
    /// Number of checks in the subset.
    #[must_use]
    pub fn len(&self) -> usize {
        self.checks.len()
    }

    /// True iff the subset is empty (no archetype ships with an empty
    /// subset; this is a sanity-check method).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }

    /// True iff the given check is in this archetype's subset.
    #[must_use]
    pub fn contains(&self, check: SecurityCheckId) -> bool {
        self.checks.iter().any(|c| *c == check)
    }

    /// Returns the subset as a fresh `Vec<SecurityCheckId>` for the
    /// `production_checklist::run_all(target, &subset.to_vec())`
    /// consumption point.
    #[must_use]
    pub fn to_vec(&self) -> Vec<SecurityCheckId> {
        self.checks.to_vec()
    }
}

// ---------------------------------------------------------------------------
// Per-archetype const subsets (GAP-2 deterministic mapping).
// ---------------------------------------------------------------------------

/// SaasDashboard: 13/15. Omits Lighthouse (rendered separately by
/// dashboard-v2 e2e if needed) and CargoMachete (Rust-side only).
const SAAS_DASHBOARD_CHECKS: &[SecurityCheckId] = &[
    SecurityCheckId::SecretsScan,
    SecurityCheckId::TrufflehogScan,
    SecurityCheckId::OwaspStatic,
    SecurityCheckId::NpmAudit,
    SecurityCheckId::PipAudit,
    SecurityCheckId::CargoAudit,
    SecurityCheckId::Biome,
    SecurityCheckId::Ruff,
    SecurityCheckId::Clippy,
    SecurityCheckId::Sqlite3Rls,
    SecurityCheckId::OtelCheck,
    SecurityCheckId::SentryCheck,
    SecurityCheckId::LsofPort,
];

/// B2bMarketplace: 15/15. Multi-tenant + billing demands every check.
const B2B_MARKETPLACE_CHECKS: &[SecurityCheckId] = &[
    SecurityCheckId::SecretsScan,
    SecurityCheckId::TrufflehogScan,
    SecurityCheckId::OwaspStatic,
    SecurityCheckId::NpmAudit,
    SecurityCheckId::PipAudit,
    SecurityCheckId::CargoAudit,
    SecurityCheckId::Lighthouse,
    SecurityCheckId::Biome,
    SecurityCheckId::Ruff,
    SecurityCheckId::Clippy,
    SecurityCheckId::Sqlite3Rls,
    SecurityCheckId::CargoMachete,
    SecurityCheckId::OtelCheck,
    SecurityCheckId::SentryCheck,
    SecurityCheckId::LsofPort,
];

/// InternalTooling: 11/15. No public UI -> no Lighthouse. Single-tenant
/// -> no Sqlite3Rls. No external customer telemetry -> no OTel/Sentry.
const INTERNAL_TOOLING_CHECKS: &[SecurityCheckId] = &[
    SecurityCheckId::SecretsScan,
    SecurityCheckId::TrufflehogScan,
    SecurityCheckId::OwaspStatic,
    SecurityCheckId::NpmAudit,
    SecurityCheckId::PipAudit,
    SecurityCheckId::CargoAudit,
    SecurityCheckId::Biome,
    SecurityCheckId::Ruff,
    SecurityCheckId::Clippy,
    SecurityCheckId::CargoMachete,
    SecurityCheckId::LsofPort,
];

/// DeveloperApi: 13/15. API-only -> no Lighthouse. Adds OTel/Sentry for
/// SLA observability. Omits CargoMachete (treated as soft-finding).
const DEVELOPER_API_CHECKS: &[SecurityCheckId] = &[
    SecurityCheckId::SecretsScan,
    SecurityCheckId::TrufflehogScan,
    SecurityCheckId::OwaspStatic,
    SecurityCheckId::NpmAudit,
    SecurityCheckId::PipAudit,
    SecurityCheckId::CargoAudit,
    SecurityCheckId::Biome,
    SecurityCheckId::Ruff,
    SecurityCheckId::Clippy,
    SecurityCheckId::CargoMachete,
    SecurityCheckId::OtelCheck,
    SecurityCheckId::SentryCheck,
    SecurityCheckId::LsofPort,
];

/// ConsumerSocial: 11/15. Lighthouse YES (consumer UX matters). No RLS
/// since single-tenant per user. Trim Rust-side runners since stack is
/// typically JS-only.
const CONSUMER_SOCIAL_CHECKS: &[SecurityCheckId] = &[
    SecurityCheckId::SecretsScan,
    SecurityCheckId::TrufflehogScan,
    SecurityCheckId::OwaspStatic,
    SecurityCheckId::NpmAudit,
    SecurityCheckId::PipAudit,
    SecurityCheckId::Lighthouse,
    SecurityCheckId::Biome,
    SecurityCheckId::Ruff,
    SecurityCheckId::OtelCheck,
    SecurityCheckId::SentryCheck,
    SecurityCheckId::LsofPort,
];

// ---------------------------------------------------------------------------
// Per-archetype scope-clarity profiles (4-point rubric per the survey).
// ---------------------------------------------------------------------------

const SAAS_DASHBOARD_CLARITY: &[&str] = &[
    "Which user persona owns this dashboard, and what one decision does it drive?",
    "What is the acceptance criteria for a successful first session (measurable)?",
    "Which tenancy model: single-tenant per customer, or RLS-per-row?",
    "Which third-party integrations are in-scope vs explicitly deferred?",
];

const B2B_MARKETPLACE_CLARITY: &[&str] = &[
    "Who are the two sides of the marketplace, and what is the take rate?",
    "What is the acceptance criteria for a first transaction (measurable)?",
    "Which payment processor and which dispute / refund policy?",
    "What is the tenant-isolation contract for marketplace data?",
];

const INTERNAL_TOOLING_CLARITY: &[&str] = &[
    "Which internal team owns this tool, and what does success look like for them?",
    "What is the SLA expectation (24x7 or business-hours)?",
    "Is single-sign-on required (Okta / Google Workspace)?",
    "What is the data-retention contract for tool-stored records?",
];

const DEVELOPER_API_CLARITY: &[&str] = &[
    "Which developer persona is the target, and which SDK languages?",
    "What is the rate-limit and quota model?",
    "What is the auth model: API-key only, OAuth, or both?",
    "What is the SLA + on-call expectation for the API uptime?",
];

const CONSUMER_SOCIAL_CLARITY: &[&str] = &[
    "Who is the target user, and what is the daily-active engagement goal?",
    "What is the content moderation policy at launch (manual or automated)?",
    "What is the data-retention and right-to-delete contract?",
    "Which content / social-graph features ship at MVP vs post-MVP?",
];

// ---------------------------------------------------------------------------
// Per-archetype catalog entries (const, deterministic).
// ---------------------------------------------------------------------------

const SAAS_DASHBOARD_ENTRY: ArchetypeEntry = ArchetypeEntry {
    archetype: SaasArchetype::SaasDashboard,
    default_auth_scheme: AuthScheme::OauthGoogle,
    multi_tenancy_model: MultiTenancyModel::RowLevelSecurity,
    default_framework_profile: "nextjs15",
    production_checklist_subset: ProductionChecklistSubset {
        checks: SAAS_DASHBOARD_CHECKS,
    },
    scope_clarity_profile: SAAS_DASHBOARD_CLARITY,
};

const B2B_MARKETPLACE_ENTRY: ArchetypeEntry = ArchetypeEntry {
    archetype: SaasArchetype::B2bMarketplace,
    default_auth_scheme: AuthScheme::OidcWorkOs,
    multi_tenancy_model: MultiTenancyModel::RowLevelSecurity,
    default_framework_profile: "nextjs15",
    production_checklist_subset: ProductionChecklistSubset {
        checks: B2B_MARKETPLACE_CHECKS,
    },
    scope_clarity_profile: B2B_MARKETPLACE_CLARITY,
};

const INTERNAL_TOOLING_ENTRY: ArchetypeEntry = ArchetypeEntry {
    archetype: SaasArchetype::InternalTooling,
    default_auth_scheme: AuthScheme::EmailMagicLink,
    multi_tenancy_model: MultiTenancyModel::SingleTenant,
    default_framework_profile: "svelte5",
    production_checklist_subset: ProductionChecklistSubset {
        checks: INTERNAL_TOOLING_CHECKS,
    },
    scope_clarity_profile: INTERNAL_TOOLING_CLARITY,
};

const DEVELOPER_API_ENTRY: ArchetypeEntry = ArchetypeEntry {
    archetype: SaasArchetype::DeveloperApi,
    default_auth_scheme: AuthScheme::OauthGithub,
    multi_tenancy_model: MultiTenancyModel::SingleTenant,
    default_framework_profile: "fastapi-modern",
    production_checklist_subset: ProductionChecklistSubset {
        checks: DEVELOPER_API_CHECKS,
    },
    scope_clarity_profile: DEVELOPER_API_CLARITY,
};

const CONSUMER_SOCIAL_ENTRY: ArchetypeEntry = ArchetypeEntry {
    archetype: SaasArchetype::ConsumerSocial,
    default_auth_scheme: AuthScheme::OauthGoogle,
    multi_tenancy_model: MultiTenancyModel::SingleTenant,
    default_framework_profile: "nextjs15",
    production_checklist_subset: ProductionChecklistSubset {
        checks: CONSUMER_SOCIAL_CHECKS,
    },
    scope_clarity_profile: CONSUMER_SOCIAL_CLARITY,
};

const ALL_ENTRIES: &[&ArchetypeEntry] = &[
    &SAAS_DASHBOARD_ENTRY,
    &B2B_MARKETPLACE_ENTRY,
    &INTERNAL_TOOLING_ENTRY,
    &DEVELOPER_API_ENTRY,
    &CONSUMER_SOCIAL_ENTRY,
];

/// Read-only catalog surface. Unit type — the underlying data is global
/// `const`. All lookups are O(1) match arms.
#[derive(Debug, Default, Clone, Copy)]
pub struct ArchetypeCatalog;

impl ArchetypeCatalog {
    /// Returns the catalog entry for the given archetype.
    #[must_use]
    pub fn entry_for(archetype: SaasArchetype) -> &'static ArchetypeEntry {
        match archetype {
            SaasArchetype::SaasDashboard => &SAAS_DASHBOARD_ENTRY,
            SaasArchetype::B2bMarketplace => &B2B_MARKETPLACE_ENTRY,
            SaasArchetype::InternalTooling => &INTERNAL_TOOLING_ENTRY,
            SaasArchetype::DeveloperApi => &DEVELOPER_API_ENTRY,
            SaasArchetype::ConsumerSocial => &CONSUMER_SOCIAL_ENTRY,
        }
    }

    /// Iterate over every catalog entry in declaration order.
    pub fn iter() -> impl Iterator<Item = &'static ArchetypeEntry> {
        ALL_ENTRIES.iter().copied()
    }

    /// Returns the production-grade checklist subset for the given
    /// archetype. Consumed by the
    /// [`crate::auditor_mode::AuditorMode::ProductionGrade`] dispatch
    /// site at the per-archetype-aware wiring callsite (sibling slice).
    #[must_use]
    pub fn production_checklist_subset_for(
        archetype: SaasArchetype,
    ) -> &'static ProductionChecklistSubset {
        &Self::entry_for(archetype).production_checklist_subset
    }

    /// Total number of catalog entries. Always 5 (locked by the
    /// `saas_archetype_taxonomy_exhaustive` contract test).
    #[must_use]
    pub fn len() -> usize {
        ALL_ENTRIES.len()
    }
}
