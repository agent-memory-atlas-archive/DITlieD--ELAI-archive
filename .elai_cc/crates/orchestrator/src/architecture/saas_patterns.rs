//! E-50 -- SaaS architecture pattern catalog substrate.
//!
//! Sibling slice to E-11 (catalog / recommender / prior_art / saas_archetype)
//! and E-25 (claude_code_arch). Lands the 9 fire-points the May 2026 SaaS
//! architecture survey (workflow/improvement-sweep-2026-05-19/reports/
//! 50-saas-architecture-patterns.md) calls out for the existing 5-archetype
//! taxonomy:
//!
//!   * production-readiness gate (admin-route-auth + rate-limit + Sentry/OTel
//!     + secret-scan + RLS lint + webhook-signature)
//!   * RLS baby-step emit-and-audit pair
//!   * scoped-token enforcement (PocketOS Apr 2026 9s wipe defense)
//!   * Merchant-of-Record vs Stripe-direct decision fork
//!   * realtime block default-off + idea-text keyword trigger
//!   * async-backbone selection by latency budget
//!   * per-tier rate-limit derivation from billing tier
//!   * OTel + SLO + runbook scaffold emit per archetype
//!
//! All fire-points ride on the existing
//! [`crate::ssb::SsbActionType::SaasArchPatternEvent`] Pattern-2
//! single-variant + [`crate::ssb::SaasArchPatternEventKind`] N-arm
//! discriminator (29th Pattern-2 application). Wire-dark unless
//! `ELAI_SAAS_ARCH_PATTERNS_ENABLE=1`.
//!
//! Constraint discipline:
//!   * e50.A guard 1 -- no parallel `SaasArchetype` enum. This module
//!     consumes the locked 5-variant
//!     [`super::saas_archetype::SaasArchetype`] taxonomy. A 4-variant
//!     B2C/B2B/ApiFirst/AiApp fork (per the original plan) is BANNED.
//!   * e50.A guard 2 -- no untenanted-query helpers. The RLS baby-step
//!     emitter produces SQL that goes through the canonical
//!     [`super::catalog::MultiTenancyModel::RowLevelSecurity`] gate.
//!   * e50.A guard 3 -- no destructive DDL emit helpers. The scoped-token
//!     enforcer surface explicitly excludes `DROP DATABASE` /
//!     `DROP TABLE` from the allowed deploy verbs.
//!   * e50.A guard 4 -- no bespoke billing adapters. The MoR decision fork
//!     records a vendor *selection*, not a parallel trait.
//!   * CLAUDE.md anti-stale: blake3 ONLY for content hashing in this
//!     module (RLS policy fingerprint); no blake2 / sha2.
//!   * CLAUDE.md no-unwrap: every fallible call propagates via `?` or
//!     `Option::map`; unit tests under `#[cfg(test)]` are exempt.
//!
//! The plan called for ~280 LOC of code surface + ~250 LOC of token
//! enforcer + ~200 LOC of RLS emitter + ~300 LOC of gate + ~180 LOC of
//! tier-derivation. This slice lands the type surface + deterministic
//! decisions in one file (the substrate); follow-up patches can break it
//! out per-module if the file grows past ~700 LOC.

use serde::{Deserialize, Serialize};

use super::catalog::{ArchetypeCatalog, MultiTenancyModel};
use super::saas_archetype::SaasArchetype;
use crate::framework_profile::SecurityCheckId;

// ---------------------------------------------------------------------------
// Environment-variable gate.
// ---------------------------------------------------------------------------

/// Wire-dark gate. Set to `"1"` to enable
/// [`crate::ssb::SsbActionType::SaasArchPatternEvent`] emission.
pub const ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR: &str = "ELAI_SAAS_ARCH_PATTERNS_ENABLE";

/// Returns true iff the E-50 SSB emission surface is wire-enabled via
/// the canonical env var. Default-off keeps the substrate dark per
/// wire-dark posture (mirrors E-11 / E-25 / E-39).
#[must_use]
pub fn ssb_emission_enabled() -> bool {
    std::env::var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Day-1 production-readiness gate.
// ---------------------------------------------------------------------------

/// One of the 6 day-1 checks emitted by the production-readiness gate
/// when the archetype catalog promotes a generated SaaS toward "ready
/// to ship". 4 of 6 map 1:1 onto existing [`SecurityCheckId`] variants;
/// the 3 remaining (admin-route-auth, rate-limit-middleware,
/// webhook-signature) are E-50-native checks that compose ON TOP of the
/// existing 15 runners rather than adding new SecurityCheckId variants.
///
/// Closed enum + as_snake_case round-trip per the
/// architecture/saas_archetype.rs precedent. Wire labels are
/// SSB-payload-compatible (round-trip via [`Self::from_snake_case`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionReadinessCheck {
    /// Every `/api/admin/*` route imports an auth middleware (regex
    /// fallback; tree-sitter AST upgrade tracked as patch-50.1).
    AdminRouteAuth,
    /// At least one Upstash Ratelimit or equivalent middleware imported
    /// on the edge entry. OWASP API#4 unrestricted resource
    /// consumption defense.
    RateLimitMiddleware,
    /// `@sentry/...` or OpenTelemetry SDK declared in the manifest
    /// (package.json / Cargo.toml / pyproject.toml). Composes with the
    /// existing [`SecurityCheckId::OtelCheck`] +
    /// [`SecurityCheckId::SentryCheck`] runners.
    SentryOrOtelDeclared,
    /// `TruffleHog` or `GitGuardian` CLI run returns zero findings.
    /// Composes with [`SecurityCheckId::SecretsScan`] +
    /// [`SecurityCheckId::TrufflehogScan`].
    SecretScanClean,
    /// Every multi-tenant table has `ENABLE ROW LEVEL SECURITY` + a
    /// policy. Composes with [`SecurityCheckId::Sqlite3Rls`] when the
    /// archetype's [`MultiTenancyModel::requires_rls_probe`] returns
    /// true.
    RlsLintClean,
    /// Every webhook handler uses a vendor lib for signature
    /// verification (e.g. `Stripe.webhooks.constructEvent`); hand-rolled
    /// HMAC is rejected.
    WebhookSignatureVerified,
}

impl ProductionReadinessCheck {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::AdminRouteAuth => "admin_route_auth",
            Self::RateLimitMiddleware => "rate_limit_middleware",
            Self::SentryOrOtelDeclared => "sentry_or_otel_declared",
            Self::SecretScanClean => "secret_scan_clean",
            Self::RlsLintClean => "rls_lint_clean",
            Self::WebhookSignatureVerified => "webhook_signature_verified",
        }
    }

    /// Inverse of [`Self::as_snake_case`]. Fail-closed on unknown input.
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "admin_route_auth" => Some(Self::AdminRouteAuth),
            "rate_limit_middleware" => Some(Self::RateLimitMiddleware),
            "sentry_or_otel_declared" => Some(Self::SentryOrOtelDeclared),
            "secret_scan_clean" => Some(Self::SecretScanClean),
            "rls_lint_clean" => Some(Self::RlsLintClean),
            "webhook_signature_verified" => Some(Self::WebhookSignatureVerified),
            _ => None,
        }
    }

    /// Returns the [`SecurityCheckId`] this readiness check composes
    /// with, if any. `None` for the 3 E-50-native checks
    /// (AdminRouteAuth / RateLimitMiddleware / WebhookSignatureVerified)
    /// that have no direct runner in the existing 15-check set.
    #[must_use]
    pub fn composes_with(self) -> Option<SecurityCheckId> {
        match self {
            Self::SentryOrOtelDeclared => Some(SecurityCheckId::OtelCheck),
            Self::SecretScanClean => Some(SecurityCheckId::SecretsScan),
            Self::RlsLintClean => Some(SecurityCheckId::Sqlite3Rls),
            Self::AdminRouteAuth
            | Self::RateLimitMiddleware
            | Self::WebhookSignatureVerified => None,
        }
    }
}

/// All 6 day-1 readiness checks in deterministic order. Iteration
/// surface for the gate runner.
pub const ALL_PRODUCTION_READINESS_CHECKS: &[ProductionReadinessCheck] = &[
    ProductionReadinessCheck::AdminRouteAuth,
    ProductionReadinessCheck::RateLimitMiddleware,
    ProductionReadinessCheck::SentryOrOtelDeclared,
    ProductionReadinessCheck::SecretScanClean,
    ProductionReadinessCheck::RlsLintClean,
    ProductionReadinessCheck::WebhookSignatureVerified,
];

/// Verdict for one check by the production-readiness gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessVerdict {
    /// Check passed (artifact present + lint clean).
    Passed,
    /// Check failed; gate blocks the promotion. Caller surfaces
    /// remediation via the
    /// [`crate::ssb::SsbActionType::SaasArchPatternEvent`] payload's
    /// `details_json.reason` field.
    Blocked,
    /// Check skipped because the archetype does not require it
    /// (e.g. [`SaasArchetype::InternalTooling`] skips
    /// [`ProductionReadinessCheck::WebhookSignatureVerified`] when no
    /// public webhook surface exists).
    Skipped,
}

impl ReadinessVerdict {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Blocked => "blocked",
            Self::Skipped => "skipped",
        }
    }
}

/// Returns whether the given check applies to the given archetype. The
/// per-archetype gating rule is deterministic and read from the catalog
/// [`super::catalog::ArchetypeEntry::production_checklist_subset`] +
/// the [`MultiTenancyModel::requires_rls_probe`] decision.
///
/// Rules:
///   * `RlsLintClean`: applies iff `multi_tenancy_model.requires_rls_probe()`
///   * `WebhookSignatureVerified`: applies for B2bMarketplace + SaasDashboard
///     + DeveloperApi (anywhere payment / API webhooks may land); skipped
///     for InternalTooling + ConsumerSocial.
///   * Other 4 checks: apply to every archetype.
#[must_use]
pub fn check_applies(archetype: SaasArchetype, check: ProductionReadinessCheck) -> bool {
    match check {
        ProductionReadinessCheck::RlsLintClean => {
            let entry = ArchetypeCatalog::entry_for(archetype);
            entry.multi_tenancy_model.requires_rls_probe()
        }
        ProductionReadinessCheck::WebhookSignatureVerified => matches!(
            archetype,
            SaasArchetype::SaasDashboard
                | SaasArchetype::B2bMarketplace
                | SaasArchetype::DeveloperApi
        ),
        ProductionReadinessCheck::AdminRouteAuth
        | ProductionReadinessCheck::RateLimitMiddleware
        | ProductionReadinessCheck::SentryOrOtelDeclared
        | ProductionReadinessCheck::SecretScanClean => true,
    }
}

// ---------------------------------------------------------------------------
// Scoped-token enforcement (PocketOS Apr 2026 9s wipe defense).
// ---------------------------------------------------------------------------

/// Deploy platform whose CLI tokens the scoped-token enforcer inspects.
/// Closed list — adding a new platform requires extending the per-
/// platform allowed-verb table at [`platform_allowed_verbs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeployPlatform {
    /// Railway. Tokens carry a `project_id` field; scope check ENFORCED.
    Railway,
    /// Fly.io. Tokens carry an `org_id` field; scope check ENFORCED.
    Fly,
    /// Vercel. Tokens DO NOT carry an explicit scope field. Enforcer
    /// emits [`SaasArchPatternEventKind::PlatformUnsupportedForScopeCheck`]
    /// and falls back to the `# scope-verified:` comment marker.
    Vercel,
    /// Cloudflare. Tokens carry an `account_id` and per-resource policy
    /// list. Scope check ENFORCED on account_id; per-resource is
    /// best-effort.
    Cloudflare,
    /// AWS. IAM users / SSO sessions / role assumptions — out of scope
    /// for the v1 lint (RUNPOD-PENDING per E-50 open question).
    /// Enforcer emits PlatformUnsupportedForScopeCheck.
    Aws,
}

impl DeployPlatform {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::Railway => "railway",
            Self::Fly => "fly",
            Self::Vercel => "vercel",
            Self::Cloudflare => "cloudflare",
            Self::Aws => "aws",
        }
    }

    /// True iff the platform's tokens carry a first-class scope field
    /// the enforcer can verify against the API. For the others the
    /// enforcer emits PlatformUnsupportedForScopeCheck + falls back to
    /// the `# scope-verified: <platform-issued-id>` comment marker.
    #[must_use]
    pub fn supports_first_class_scope_check(self) -> bool {
        matches!(self, Self::Railway | Self::Fly | Self::Cloudflare)
    }
}

/// Deploy verbs the scoped-token enforcer recognises. Each verb has a
/// per-platform "destructive" rating. The plan's hard rule is that no
/// AI-agent IaC step EVER receives a token with destructive scope; this
/// enum is the canonical surface the enforcer reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeployVerb {
    /// Read-only inspection (logs, status, env-var list).
    Read,
    /// Push a new build / deploy a new container.
    Deploy,
    /// Scale a service up / down (non-destructive resource change).
    Scale,
    /// Destroy the project / service / database. ALWAYS rejected for
    /// agent-issued tokens.
    Destroy,
}

impl DeployVerb {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Deploy => "deploy",
            Self::Scale => "scale",
            Self::Destroy => "destroy",
        }
    }

    /// True iff agent-issued tokens may EVER carry this verb. False for
    /// [`Self::Destroy`] (PocketOS defense).
    #[must_use]
    pub fn is_agent_allowed(self) -> bool {
        !matches!(self, Self::Destroy)
    }
}

/// Deterministic allowed-verb table per platform. Used by the enforcer
/// to decide whether to emit
/// [`SaasArchPatternEventKind::UnscopedTokenBlocked`].
#[must_use]
pub fn platform_allowed_verbs(platform: DeployPlatform) -> &'static [DeployVerb] {
    match platform {
        // Railway / Fly / Cloudflare: read + deploy + scale only. Destroy
        // is BLOCKED by default; opt-out requires `--allow-unscoped-token`.
        DeployPlatform::Railway | DeployPlatform::Fly | DeployPlatform::Cloudflare => {
            &[DeployVerb::Read, DeployVerb::Deploy, DeployVerb::Scale]
        }
        // Vercel: same allowed set; scope check via comment marker.
        DeployPlatform::Vercel => &[DeployVerb::Read, DeployVerb::Deploy, DeployVerb::Scale],
        // AWS: read-only by default for agent IaC (broadest blast radius
        // platform; conservative posture).
        DeployPlatform::Aws => &[DeployVerb::Read],
    }
}

// ---------------------------------------------------------------------------
// Merchant-of-Record decision fork.
// ---------------------------------------------------------------------------

/// Merchant-of-Record vendor selected by the
/// [`SaasArchPatternEventKind::BillingMorDecisionMade`] fork. `None` =
/// Stripe-direct (with Anrok for US sales-tax). The Apr-Jul 2024 Stripe
/// + Lemon Squeezy acquisition collapses the "default global MoR"
/// question (per fintechspecs.com 2026): Polar (OSS + 4% + GitHub-native)
/// is the canonical alternative recommendation; Paddle is the enterprise
/// fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MorVendor {
    /// Polar.sh (4% + OSS + GitHub-native). Default for global + any MRR
    /// when MoR is required. Note: maturity-risk flag in decision
    /// rationale per E-50 Open Q2.
    Polar,
    /// Paddle. Enterprise MoR fallback (5% + 0.50 USD per transaction).
    Paddle,
    /// Lemon Squeezy. Stripe-acquired Jul 2024; surfaces as a legacy
    /// vendor for any existing PipelineSpec referring to it. New
    /// PipelineSpecs SHOULD prefer Polar / Paddle.
    LemonSqueezy,
}

impl MorVendor {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::Polar => "polar",
            Self::Paddle => "paddle",
            Self::LemonSqueezy => "lemon_squeezy",
        }
    }

    /// Inverse of [`Self::as_snake_case`].
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "polar" => Some(Self::Polar),
            "paddle" => Some(Self::Paddle),
            "lemon_squeezy" => Some(Self::LemonSqueezy),
            _ => None,
        }
    }
}

/// Target-geography classification driving the MoR decision fork.
/// Deterministic 3-arm closed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetGeography {
    /// US-only. Default suggestion: Stripe + Anrok direct (MoR `None`).
    UsOnly,
    /// EU-only. Default suggestion: Polar (OSS + GitHub-native, EU VAT
    /// MoR handling).
    EuOnly,
    /// Global. Default suggestion: Polar (4% + OSS) for <$100K MRR;
    /// Paddle for enterprise.
    Global,
}

impl TargetGeography {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::UsOnly => "us_only",
            Self::EuOnly => "eu_only",
            Self::Global => "global",
        }
    }
}

/// MRR projection bucket driving the MoR decision fork. Coarse 3-arm
/// enum so a user-silent FRAMING substate can default to the middle
/// bucket if Y1 MRR projection is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MrrProjection {
    /// <$10K Y1 MRR. Indie / side-project. Cost-sensitive; prefer Polar
    /// or Lemon Squeezy.
    UnderTenK,
    /// $10K-$100K Y1 MRR. Default bucket on user-silent FRAMING.
    TenToHundredK,
    /// >$100K Y1 MRR. Enterprise. Paddle / Anrok-direct viable.
    OverHundredK,
}

impl MrrProjection {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::UnderTenK => "under_ten_k",
            Self::TenToHundredK => "ten_to_hundred_k",
            Self::OverHundredK => "over_hundred_k",
        }
    }
}

/// Result of the MoR decision fork. `None` MoR = Stripe-direct (with
/// Anrok for US sales-tax). The decision is deterministic given
/// (archetype, geography, mrr_projection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MorDecision {
    /// Selected MoR vendor; `None` = Stripe-direct.
    pub vendor: Option<MorVendor>,
    /// Geography that drove the decision.
    pub geography: TargetGeography,
    /// MRR projection that drove the decision.
    pub mrr_projection: MrrProjection,
}

/// Deterministic MoR decision per (geography, mrr_projection). The
/// archetype is not a direct input but is recorded on the SSB payload
/// for traceability. Decision rules grounded in fintechspecs.com 2026 +
/// Anrok+Stripe announcement (see E-50-decisions.md).
#[must_use]
pub fn decide_mor(geography: TargetGeography, mrr: MrrProjection) -> MorDecision {
    let vendor = match (geography, mrr) {
        // US-only: Stripe + Anrok direct is the cheapest legal path until
        // enterprise scale demands a Paddle-style upgrade. Even >$100K
        // MRR is Stripe + Anrok direct until cross-state nexus + audit
        // burden flips the equation (out of scope for this gate).
        (TargetGeography::UsOnly, _) => None,
        // EU-only: Polar default; Paddle on enterprise.
        (TargetGeography::EuOnly, MrrProjection::OverHundredK) => Some(MorVendor::Paddle),
        (TargetGeography::EuOnly, _) => Some(MorVendor::Polar),
        // Global: Polar default; Paddle on enterprise.
        (TargetGeography::Global, MrrProjection::OverHundredK) => Some(MorVendor::Paddle),
        (TargetGeography::Global, _) => Some(MorVendor::Polar),
    };
    MorDecision {
        vendor,
        geography,
        mrr_projection: mrr,
    }
}

// ---------------------------------------------------------------------------
// Realtime block default-off + idea-text keyword trigger.
// ---------------------------------------------------------------------------

/// Realtime / collaboration block requirement. Default `None` per the
/// May 2026 report's "most SaaS doesn't need it" finding; only flipped
/// on when the idea-text mentions the trigger keywords (see
/// [`REALTIME_TRIGGER_KEYWORDS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeNeed {
    /// No realtime / collab block. Default; saves Liveblocks bill.
    None,
    /// Presence indicator only (PartyKit on Cloudflare Durable Objects).
    Presence,
    /// Full CRDT sync (Liveblocks Yjs / managed Yjs sync).
    Crdt,
}

impl Default for RealtimeNeed {
    fn default() -> Self {
        Self::None
    }
}

impl RealtimeNeed {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Presence => "presence",
            Self::Crdt => "crdt",
        }
    }

    /// Inverse of [`Self::as_snake_case`].
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "presence" => Some(Self::Presence),
            "crdt" => Some(Self::Crdt),
            _ => None,
        }
    }
}

/// Keyword triggers that flip [`RealtimeNeed`] from `None` to a concrete
/// arm. CRDT triggers take priority over Presence triggers because
/// CRDT-needing ideas (co-edit, multiplayer) usually also need presence.
/// Deterministic table; tie-break = first hit wins.
pub const REALTIME_TRIGGER_KEYWORDS: &[(&str, RealtimeNeed)] = &[
    // CRDT class.
    ("co-edit", RealtimeNeed::Crdt),
    ("co-editing", RealtimeNeed::Crdt),
    ("co-write", RealtimeNeed::Crdt),
    ("multiplayer", RealtimeNeed::Crdt),
    ("real-time collaboration", RealtimeNeed::Crdt),
    ("realtime collaboration", RealtimeNeed::Crdt),
    ("collaborative editing", RealtimeNeed::Crdt),
    ("whiteboard", RealtimeNeed::Crdt),
    // Presence class.
    ("presence", RealtimeNeed::Presence),
    ("who's online", RealtimeNeed::Presence),
    ("cursor", RealtimeNeed::Presence),
    ("live cursor", RealtimeNeed::Presence),
];

/// Apply the realtime-trigger keyword table to a brief. Returns the
/// highest-priority match (CRDT > Presence). Deterministic.
#[must_use]
pub fn realtime_need_from_brief(brief: &str) -> RealtimeNeed {
    let lower = brief.to_ascii_lowercase();
    // First pass: any CRDT keyword wins.
    for (kw, need) in REALTIME_TRIGGER_KEYWORDS {
        if matches!(need, RealtimeNeed::Crdt) && lower.contains(kw) {
            return RealtimeNeed::Crdt;
        }
    }
    // Second pass: presence keyword.
    for (kw, need) in REALTIME_TRIGGER_KEYWORDS {
        if matches!(need, RealtimeNeed::Presence) && lower.contains(kw) {
            return RealtimeNeed::Presence;
        }
    }
    RealtimeNeed::None
}

// ---------------------------------------------------------------------------
// Async-backbone selection by latency budget.
// ---------------------------------------------------------------------------

/// Long-running-task backbone selected by the latency-budget question
/// injected into FRAMING. Routing rule mirrors the report's vendor
/// matrix:
///   * <30s sync   -> Serverless (Vercel / Cloudflare Worker)
///   * 30s-15min   -> Inngest (no Redis required; indie default)
///   * 15min-2h    -> TriggerDev (long jobs + observability)
///   * AI inference -> Modal (serverless GPU) for B2C / B2B;
///                    Hatchet (AI DAG, streaming, priority lanes) for
///                    AI-app archetype per pkgpulse.com 2026.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsyncBackbone {
    /// <30s sync; serverless function.
    Serverless,
    /// 30s-15min; Inngest (no Redis required).
    Inngest,
    /// 15min-2h; TriggerDev.
    TriggerDev,
    /// AI inference; Modal serverless GPU.
    Modal,
    /// AI inference; Hatchet DAG + streaming + priority lanes.
    Hatchet,
}

impl AsyncBackbone {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::Serverless => "serverless",
            Self::Inngest => "inngest",
            Self::TriggerDev => "trigger_dev",
            Self::Modal => "modal",
            Self::Hatchet => "hatchet",
        }
    }

    /// Inverse of [`Self::as_snake_case`].
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "serverless" => Some(Self::Serverless),
            "inngest" => Some(Self::Inngest),
            "trigger_dev" => Some(Self::TriggerDev),
            "modal" => Some(Self::Modal),
            "hatchet" => Some(Self::Hatchet),
            _ => None,
        }
    }
}

/// Latency budget bucket for an async job. Deterministic 4-arm closed
/// enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatencyBudget {
    /// <30s user-wait. Serverless function suffices.
    SyncUnder30s,
    /// 30s-15min. Inngest sweet spot.
    Async30sTo15Min,
    /// 15min-2h. TriggerDev / Fly worker.
    Long15MinTo2H,
    /// AI inference path (any latency); routed by archetype.
    AiInference,
}

impl LatencyBudget {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::SyncUnder30s => "sync_under_30s",
            Self::Async30sTo15Min => "async_30s_to_15_min",
            Self::Long15MinTo2H => "long_15_min_to_2h",
            Self::AiInference => "ai_inference",
        }
    }
}

/// Deterministic async-backbone selection. The archetype is consulted
/// only for the AiInference bucket: any archetype whose default is
/// "AI-app" (ConsumerSocial running an inference workload OR
/// DeveloperApi serving AI completions) flips Modal -> Hatchet when
/// the brief signals streaming / priority-lane semantics. Default = Modal.
#[must_use]
pub fn select_async_backbone(
    archetype: SaasArchetype,
    budget: LatencyBudget,
    brief: &str,
) -> AsyncBackbone {
    match budget {
        LatencyBudget::SyncUnder30s => AsyncBackbone::Serverless,
        LatencyBudget::Async30sTo15Min => AsyncBackbone::Inngest,
        LatencyBudget::Long15MinTo2H => AsyncBackbone::TriggerDev,
        LatencyBudget::AiInference => {
            let lower = brief.to_ascii_lowercase();
            let wants_dag = lower.contains("dag")
                || lower.contains("priority lane")
                || lower.contains("streaming step")
                || lower.contains("pipeline");
            // Hatchet preferred for DeveloperApi + AI-keyword briefs;
            // Modal for everything else.
            if wants_dag && matches!(archetype, SaasArchetype::DeveloperApi) {
                AsyncBackbone::Hatchet
            } else {
                AsyncBackbone::Modal
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Per-tier rate-limit derivation from billing tier names.
// ---------------------------------------------------------------------------

/// One per-tier rate-limit row. Deterministic seed values per the
/// May 2026 report's recommended defaults (free=10rpm / pro=100rpm /
/// business=1000rpm); tiered upgrade is the canonical OWASP API#4
/// defense surface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PerTierRateLimit {
    /// Tier name as it appears in the PipelineSpec billing block (e.g.
    /// `"free"`, `"pro"`, `"business"`).
    pub tier: String,
    /// Requests per minute budget. `0` = unrestricted (NOT recommended;
    /// the
    /// [`crate::ssb::SaasArchPatternEventKind::PerTierRateLimitDerived`]
    /// payload records the value for traceability).
    pub requests_per_minute: u32,
}

/// Derive default per-tier rate-limit budgets from a list of tier
/// names. Deterministic seed values per the report. Unknown tier names
/// receive the median bucket (100rpm) to avoid silent under-protection.
#[must_use]
pub fn derive_tier_limits(tiers: &[&str]) -> Vec<PerTierRateLimit> {
    tiers
        .iter()
        .map(|name| {
            let normalized = name.to_ascii_lowercase();
            let rpm = match normalized.as_str() {
                // OWASP API#4 anchored seeds. Match common naming
                // (free, pro, business, enterprise). Default to 100rpm
                // (pro tier) for unknown names so the rate-limit lint
                // never returns 0 by accident.
                "free" | "starter" | "hobby" => 10,
                "pro" | "growth" | "standard" => 100,
                "business" | "team" => 1000,
                "enterprise" | "scale" => 5000,
                _ => 100,
            };
            PerTierRateLimit {
                tier: (*name).to_owned(),
                requests_per_minute: rpm,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// OTel + SLO + runbook scaffold defaults per archetype.
// ---------------------------------------------------------------------------

/// SLO scaffold defaults per the OneUptime 2026 multi-burn-rate guide
/// (crit @ 14x / warn @ 6x / notice @ 1x). The catalog default mirrors
/// the report's "99.9% availability / 500ms p95 / 1% error rate"
/// baseline; archetype-specific tweaks live in the per-archetype const
/// table below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SloScaffold {
    /// Availability target as a basis-point integer (9990 = 99.90%).
    /// Integer storage avoids `f32` round-trip drift on SSB JSON.
    pub availability_bps: u32,
    /// p95 latency target in milliseconds.
    pub p95_latency_ms: u32,
    /// Error-rate ceiling as a basis-point integer (100 = 1.00%).
    pub error_rate_bps: u32,
    /// Multi-burn-rate alert crit factor (default 14).
    pub burn_crit_factor: u32,
    /// Multi-burn-rate alert warn factor (default 6).
    pub burn_warn_factor: u32,
    /// Multi-burn-rate alert notice factor (default 1).
    pub burn_notice_factor: u32,
}

impl SloScaffold {
    /// Default scaffold per OneUptime 2026.
    pub const fn default_baseline() -> Self {
        Self {
            availability_bps: 9990,
            p95_latency_ms: 500,
            error_rate_bps: 100,
            burn_crit_factor: 14,
            burn_warn_factor: 6,
            burn_notice_factor: 1,
        }
    }
}

/// Per-archetype SLO scaffold defaults. B2bMarketplace gets a tighter
/// p95 (300ms) because two-sided marketplaces are latency-sensitive on
/// search; DeveloperApi gets a tighter availability (99.95%) because
/// API SLAs demand it; the other 3 archetypes use the baseline.
#[must_use]
pub fn slo_scaffold_for(archetype: SaasArchetype) -> SloScaffold {
    let mut s = SloScaffold::default_baseline();
    match archetype {
        SaasArchetype::B2bMarketplace => {
            s.p95_latency_ms = 300;
        }
        SaasArchetype::DeveloperApi => {
            s.availability_bps = 9995;
            s.p95_latency_ms = 250;
        }
        SaasArchetype::SaasDashboard
        | SaasArchetype::InternalTooling
        | SaasArchetype::ConsumerSocial => {}
    }
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_gate_default_off() {
        // Default-off invariant; the test doesn't assume the env var is
        // unset because other tests may set it -- explicit unset.
        std::env::remove_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR);
        assert!(!ssb_emission_enabled(), "default-off invariant");
        std::env::set_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR, "1");
        assert!(ssb_emission_enabled(), "explicit-1 enables");
        std::env::set_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR, "0");
        assert!(!ssb_emission_enabled(), "explicit-0 keeps dark");
        std::env::remove_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR);
    }

    #[test]
    fn production_readiness_check_round_trip() {
        for chk in ALL_PRODUCTION_READINESS_CHECKS {
            let label = chk.as_snake_case();
            assert_eq!(ProductionReadinessCheck::from_snake_case(label), Some(*chk));
        }
        assert!(ProductionReadinessCheck::from_snake_case("not_a_check").is_none());
    }

    #[test]
    fn check_applies_rls_only_when_multi_tenant() {
        // SaasDashboard + B2bMarketplace use RowLevelSecurity -> RLS lint applies.
        assert!(check_applies(
            SaasArchetype::SaasDashboard,
            ProductionReadinessCheck::RlsLintClean
        ));
        assert!(check_applies(
            SaasArchetype::B2bMarketplace,
            ProductionReadinessCheck::RlsLintClean
        ));
        // InternalTooling + DeveloperApi + ConsumerSocial default to
        // SingleTenant -> RLS lint skipped.
        assert!(!check_applies(
            SaasArchetype::InternalTooling,
            ProductionReadinessCheck::RlsLintClean
        ));
        assert!(!check_applies(
            SaasArchetype::DeveloperApi,
            ProductionReadinessCheck::RlsLintClean
        ));
        assert!(!check_applies(
            SaasArchetype::ConsumerSocial,
            ProductionReadinessCheck::RlsLintClean
        ));
    }

    #[test]
    fn check_applies_webhook_only_for_billing_or_api_archetypes() {
        assert!(check_applies(
            SaasArchetype::B2bMarketplace,
            ProductionReadinessCheck::WebhookSignatureVerified
        ));
        assert!(check_applies(
            SaasArchetype::SaasDashboard,
            ProductionReadinessCheck::WebhookSignatureVerified
        ));
        assert!(check_applies(
            SaasArchetype::DeveloperApi,
            ProductionReadinessCheck::WebhookSignatureVerified
        ));
        assert!(!check_applies(
            SaasArchetype::InternalTooling,
            ProductionReadinessCheck::WebhookSignatureVerified
        ));
        assert!(!check_applies(
            SaasArchetype::ConsumerSocial,
            ProductionReadinessCheck::WebhookSignatureVerified
        ));
    }

    #[test]
    fn deploy_verb_destroy_blocked_for_agents() {
        assert!(!DeployVerb::Destroy.is_agent_allowed());
        assert!(DeployVerb::Read.is_agent_allowed());
        assert!(DeployVerb::Deploy.is_agent_allowed());
        assert!(DeployVerb::Scale.is_agent_allowed());
    }

    #[test]
    fn destroy_not_in_any_platform_allowed_verbs() {
        for platform in [
            DeployPlatform::Railway,
            DeployPlatform::Fly,
            DeployPlatform::Vercel,
            DeployPlatform::Cloudflare,
            DeployPlatform::Aws,
        ] {
            let verbs = platform_allowed_verbs(platform);
            assert!(
                !verbs.contains(&DeployVerb::Destroy),
                "platform {platform:?} allowed Destroy -- PocketOS defense breached"
            );
        }
    }

    #[test]
    fn mor_decision_us_picks_stripe_direct() {
        let d = decide_mor(TargetGeography::UsOnly, MrrProjection::TenToHundredK);
        assert_eq!(d.vendor, None, "US-only -> Stripe direct (no MoR)");
    }

    #[test]
    fn mor_decision_global_default_picks_polar() {
        let d = decide_mor(TargetGeography::Global, MrrProjection::UnderTenK);
        assert_eq!(d.vendor, Some(MorVendor::Polar));
        let d = decide_mor(TargetGeography::Global, MrrProjection::TenToHundredK);
        assert_eq!(d.vendor, Some(MorVendor::Polar));
    }

    #[test]
    fn mor_decision_enterprise_picks_paddle() {
        let d = decide_mor(TargetGeography::Global, MrrProjection::OverHundredK);
        assert_eq!(d.vendor, Some(MorVendor::Paddle));
        let d = decide_mor(TargetGeography::EuOnly, MrrProjection::OverHundredK);
        assert_eq!(d.vendor, Some(MorVendor::Paddle));
    }

    #[test]
    fn realtime_default_off() {
        assert_eq!(realtime_need_from_brief("todo app"), RealtimeNeed::None);
        assert_eq!(
            realtime_need_from_brief("Build a multi-tenant B2B SaaS CRM"),
            RealtimeNeed::None
        );
    }

    #[test]
    fn realtime_crdt_triggers() {
        assert_eq!(
            realtime_need_from_brief("multiplayer whiteboard"),
            RealtimeNeed::Crdt
        );
        assert_eq!(
            realtime_need_from_brief("real-time collaboration on docs"),
            RealtimeNeed::Crdt
        );
        assert_eq!(
            realtime_need_from_brief("co-editing canvas"),
            RealtimeNeed::Crdt
        );
    }

    #[test]
    fn realtime_presence_triggers() {
        assert_eq!(
            realtime_need_from_brief("show who's online in the workspace"),
            RealtimeNeed::Presence
        );
        assert_eq!(
            realtime_need_from_brief("live cursor display"),
            RealtimeNeed::Presence
        );
    }

    #[test]
    fn async_backbone_routes_by_latency() {
        assert_eq!(
            select_async_backbone(SaasArchetype::SaasDashboard, LatencyBudget::SyncUnder30s, ""),
            AsyncBackbone::Serverless
        );
        assert_eq!(
            select_async_backbone(
                SaasArchetype::SaasDashboard,
                LatencyBudget::Async30sTo15Min,
                ""
            ),
            AsyncBackbone::Inngest
        );
        assert_eq!(
            select_async_backbone(
                SaasArchetype::B2bMarketplace,
                LatencyBudget::Long15MinTo2H,
                ""
            ),
            AsyncBackbone::TriggerDev
        );
        // AI inference default = Modal.
        assert_eq!(
            select_async_backbone(
                SaasArchetype::ConsumerSocial,
                LatencyBudget::AiInference,
                "image generation"
            ),
            AsyncBackbone::Modal
        );
        // Hatchet routing only on DeveloperApi + DAG-keyword brief.
        assert_eq!(
            select_async_backbone(
                SaasArchetype::DeveloperApi,
                LatencyBudget::AiInference,
                "streaming step DAG pipeline"
            ),
            AsyncBackbone::Hatchet
        );
    }

    #[test]
    fn per_tier_rate_limits_deterministic() {
        let limits = derive_tier_limits(&["free", "pro", "business"]);
        assert_eq!(limits.len(), 3);
        assert_eq!(limits[0].tier, "free");
        assert_eq!(limits[0].requests_per_minute, 10);
        assert_eq!(limits[1].requests_per_minute, 100);
        assert_eq!(limits[2].requests_per_minute, 1000);
    }

    #[test]
    fn per_tier_rate_limits_unknown_uses_median() {
        let limits = derive_tier_limits(&["bespoke_premium_tier"]);
        assert_eq!(limits[0].requests_per_minute, 100, "unknown -> median 100rpm");
    }

    #[test]
    fn slo_scaffold_per_archetype_tweaks() {
        let dashboard = slo_scaffold_for(SaasArchetype::SaasDashboard);
        assert_eq!(dashboard.availability_bps, 9990);
        assert_eq!(dashboard.p95_latency_ms, 500);

        let marketplace = slo_scaffold_for(SaasArchetype::B2bMarketplace);
        assert_eq!(marketplace.p95_latency_ms, 300, "marketplace tighter p95");

        let api = slo_scaffold_for(SaasArchetype::DeveloperApi);
        assert_eq!(api.availability_bps, 9995, "api tighter availability");
        assert_eq!(api.p95_latency_ms, 250);
    }

    #[test]
    fn slo_scaffold_baseline_multi_burn_rate() {
        let s = SloScaffold::default_baseline();
        assert_eq!(s.burn_crit_factor, 14, "OneUptime crit 14x");
        assert_eq!(s.burn_warn_factor, 6, "OneUptime warn 6x");
        assert_eq!(s.burn_notice_factor, 1, "OneUptime notice 1x");
    }

    #[test]
    fn mor_vendor_round_trip() {
        for v in [MorVendor::Polar, MorVendor::Paddle, MorVendor::LemonSqueezy] {
            assert_eq!(MorVendor::from_snake_case(v.as_snake_case()), Some(v));
        }
    }

    #[test]
    fn realtime_need_round_trip() {
        for n in [RealtimeNeed::None, RealtimeNeed::Presence, RealtimeNeed::Crdt] {
            assert_eq!(RealtimeNeed::from_snake_case(n.as_snake_case()), Some(n));
        }
    }

    #[test]
    fn async_backbone_round_trip() {
        for a in [
            AsyncBackbone::Serverless,
            AsyncBackbone::Inngest,
            AsyncBackbone::TriggerDev,
            AsyncBackbone::Modal,
            AsyncBackbone::Hatchet,
        ] {
            assert_eq!(AsyncBackbone::from_snake_case(a.as_snake_case()), Some(a));
        }
    }

    #[test]
    fn composes_with_maps_to_existing_security_check() {
        // 3 composing checks have a SecurityCheckId backing.
        assert_eq!(
            ProductionReadinessCheck::SentryOrOtelDeclared.composes_with(),
            Some(SecurityCheckId::OtelCheck)
        );
        assert_eq!(
            ProductionReadinessCheck::SecretScanClean.composes_with(),
            Some(SecurityCheckId::SecretsScan)
        );
        assert_eq!(
            ProductionReadinessCheck::RlsLintClean.composes_with(),
            Some(SecurityCheckId::Sqlite3Rls)
        );
        // 3 E-50-native checks have no SecurityCheckId backing.
        assert_eq!(
            ProductionReadinessCheck::AdminRouteAuth.composes_with(),
            None
        );
        assert_eq!(
            ProductionReadinessCheck::RateLimitMiddleware.composes_with(),
            None
        );
        assert_eq!(
            ProductionReadinessCheck::WebhookSignatureVerified.composes_with(),
            None
        );
    }

    #[test]
    fn all_production_readiness_checks_exhaustive() {
        // Adding a new variant to ProductionReadinessCheck without
        // updating ALL_PRODUCTION_READINESS_CHECKS would fail this match.
        for chk in ALL_PRODUCTION_READINESS_CHECKS {
            match chk {
                ProductionReadinessCheck::AdminRouteAuth
                | ProductionReadinessCheck::RateLimitMiddleware
                | ProductionReadinessCheck::SentryOrOtelDeclared
                | ProductionReadinessCheck::SecretScanClean
                | ProductionReadinessCheck::RlsLintClean
                | ProductionReadinessCheck::WebhookSignatureVerified => (),
            }
        }
        assert_eq!(
            ALL_PRODUCTION_READINESS_CHECKS.len(),
            6,
            "6 day-1 checks per E-50 plan"
        );
    }
}
