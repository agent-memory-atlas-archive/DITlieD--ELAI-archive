//! E-11 catalog+recommender slice -- SaaS archetype catalog +
//! ProjectClass-aware recommender + 2024-2026 prior-art registry.
//!
//! This subtree is the **substrate** the wider E-11 plan's downstream
//! consumers (production-grade auditor per-archetype subset selection,
//! package_verifier per-manifest scan choice, idea-critic scope_clarity
//! signal) ride on top of. Sibling slices (scope_clarity_score,
//! package_verifier, anti-spiral hard-abort, edit_format) are NOT in
//! this subtree -- they extend `crate::idea_framing`,
//! `crate::autonomous_gate`, and new sibling modules.
//!
//! Wire-dark posture: the SSB emission surface
//! [`crate::ssb::SsbActionType::SaasPriorArtEvent`] is gated by the
//! `ELAI_SAAS_PRIOR_ART_ENABLE=1` environment variable. The
//! recommender + catalog + registry themselves are always available
//! (in-process callers can consume them without touching the SSB).
//!
//! Public surface:
//!   * [`SaasArchetype`] -- the 5-archetype taxonomy
//!   * [`ALL_SAAS_ARCHETYPES`] -- canonical iteration order
//!   * [`ArchetypeCatalog`] / [`ArchetypeEntry`] -- per-archetype defaults
//!   * [`ProductionChecklistSubset`] -- SecurityCheckId subset per archetype
//!   * [`AuthScheme`] / [`MultiTenancyModel`] -- catalog field enums
//!   * [`recommend_archetype`] -- the recommender entry point
//!   * [`ArchetypeRecommendation`] / [`RecommendationConfidence`]
//!   * [`PriorArtRef`] / [`PriorArtRegistry`] / [`PRIOR_ART`]

pub mod catalog;
pub mod claude_code_arch;
pub mod prior_art;
pub mod recommender;
pub mod saas_archetype;
pub mod saas_patterns;
// E-25 Tranche C (C.1): anti-pattern guards (Bash complexity / approval
// fatigue / sandbox template). Each guard reuses the SHIPPED Pattern-2
// ClaudeCodeArchitectureEvent single-variant + an already-shipped Kind
// arm; ZERO new SsbActionType and ZERO schema bump per session-4
// DECISION-2 + DECISION-5 9-arm lock. See
// workflow/reasoning/E-25-trancheC-decisions.md.
pub mod bash_complexity_guard;
pub mod approval_fatigue_guard;
pub mod sandbox_template_guard;
// E-25 Tranche C (C.2): LLM-decision hook handler. NEW PromptDecision
// enum {Allow, Deny(reason), Ask(reason)} -- distinct from
// HookVerdictKind {Allow, Warn, Block} and PermissionPromptDecision
// {Allow, Deny}. Reuses the SHIPPED ClaudeCodeArchitectureEventKind::
// PromptDecisionMade arm via Pattern-2 single-variant SSB cascade.
// See workflow/reasoning/E-25-trancheC-decisions.md.
pub mod hook_prompt;
// E-25 Tranche C (C.4): edit_checkpoints (Primitive 6b). Pre-edit
// blake3 snapshot store + rewind path. On-disk content-addressed
// layout under .elai_cc/.session-checkpoints/{session_id}/{seq:06}/
// {blake3_prefix:8}/{path}. Reuses the SHIPPED
// ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted arm;
// rewind events ride on the same arm via
// details_json.rewind_event=true (9-arm Kind enum LOCKED per
// session-4 DECISION-5). See workflow/reasoning/E-25-trancheC-
// decisions.md.
pub mod edit_checkpoints;
// H-49 (SCHEMA_VERSION 226): reversible LLM edits — CAS object store +
// undo/redo/branch engine over the SHIPPED E-25 edit_checkpoints primitives.
// Kill-switch: ELAI_TIMETRAVEL_DISABLE (default-ON; no wire-dark-allowlist row).
pub mod timetravel;

pub use catalog::{
    ArchetypeCatalog, ArchetypeEntry, AuthScheme, MultiTenancyModel, ProductionChecklistSubset,
};
pub use claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload, ClaudeCodeLadderResult, CompactionTierId,
    ALL_CLAUDE_CODE_PRIMITIVES, ALL_COMPACTION_TIERS, ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR,
};
pub use prior_art::{PriorArtRef, PriorArtRegistry, PRIOR_ART};
pub use recommender::{recommend_archetype, ArchetypeRecommendation, RecommendationConfidence};
pub use saas_archetype::{SaasArchetype, ALL_SAAS_ARCHETYPES};
pub use saas_patterns::{
    check_applies, decide_mor, derive_tier_limits, platform_allowed_verbs, realtime_need_from_brief,
    select_async_backbone, slo_scaffold_for, ssb_emission_enabled as saas_patterns_ssb_emission_enabled,
    AsyncBackbone, DeployPlatform, DeployVerb, LatencyBudget, MorDecision, MorVendor,
    MrrProjection, PerTierRateLimit, ProductionReadinessCheck, ReadinessVerdict, RealtimeNeed,
    SloScaffold, TargetGeography, ALL_PRODUCTION_READINESS_CHECKS,
    ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR,
};

/// Environment-variable name controlling wire-dark SSB emission for
/// E-11 prior-art / archetype-recommendation rows. Set to `1` to
/// enable.
pub const ELAI_SAAS_PRIOR_ART_ENABLE_VAR: &str = "ELAI_SAAS_PRIOR_ART_ENABLE";

/// Returns true iff the E-11 SSB emission surface is wire-enabled via
/// the canonical env var. Default-off keeps the surface dark per
/// wire-dark posture.
#[must_use]
pub fn ssb_emission_enabled() -> bool {
    std::env::var(ELAI_SAAS_PRIOR_ART_ENABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// E-25 -- alias for the claude-code-architecture wire-dark gate. Keeps
/// the per-substrate naming convention consistent for downstream
/// consumers that read `architecture::claude_code_arch_ssb_emission_enabled()`.
#[must_use]
pub fn claude_code_arch_ssb_emission_enabled() -> bool {
    claude_code_arch::ssb_emission_enabled()
}
