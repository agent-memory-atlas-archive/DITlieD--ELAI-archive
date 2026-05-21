//! E-50 Tier-A — SaaS architecture pattern catalog regression contracts.
//!
//! Sixth sibling negative-space contract. Thanks to the shared-walker
//! refactor in E-12.A (tests/common/mod.rs with is_sibling_negative_space_contract
//! auto-skip), adding this contract required ZERO edits to the existing
//! siblings (e11 / e12 / e25 / e38 / e48). The N*N edit cost is gone.
//!
//! Locks invariants the E-50 catalog will need when Tier-B lands:
//!   - One canonical SaasArchetype taxonomy — no parallel enum, no
//!     fabricated archetype variants beyond what E-11.A workflow/archetypes/
//!     README declares (SaaSDashboard / B2BMarketplace / InternalTooling /
//!     DeveloperAPI / ConsumerSocial).
//!   - No silent multi-tenancy bypass — every emitted DB query must go
//!     through the RLS / tenant-scoped path.
//!   - No raw `DROP DATABASE` / `DROP TABLE` in templates (E-50 templates
//!     would be reused across tenants; destructive DDL must require
//!     explicit per-template opt-in via review).
//!   - No bespoke billing integration parallel to the canonical
//!     billing adapter (Tier-B will land that adapter; this contract
//!     pre-positions the invariant).

mod common;

use common::{collect_hits, format_hits, workspace_rs_files};

#[test]
fn no_parallel_saas_archetype_enum() {
    // E-11.A workflow/archetypes/README.md declares the canonical archetype
    // taxonomy: SaaSDashboard / B2BMarketplace / InternalTooling /
    // DeveloperAPI / ConsumerSocial. Parallel enums (e.g. SaasArchetypeV2
    // with B2C/B2B/ApiFirst/AiApp as the E-50 plan mapping suggested)
    // fragment the catalog.
    let banned = [
        "enum SaasArchetypeV2",
        "enum SaaSArchetypeNext",
        "enum ProductArchetype",
        "enum BespokeArchetype",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel SaaS archetype enums banned — extend the canonical taxonomy in workflow/archetypes/.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_silent_multi_tenancy_bypass() {
    // Every emitted DB query must go through the RLS / tenant-scoped path.
    // crates/rules/src/contract_tests.rs:1603-1649 already enforces this
    // (app.5/app.6 rules: SET app.tenant_id ALLOW + DISCARD ALL on pooler
    // reuse + RLS WHERE-only BLOCK). E-50 templates must not introduce
    // helpers that skip the gate.
    let banned = [
        "fn raw_db_query_no_tenant",
        "fn bypass_rls_policy",
        "fn skip_tenant_scope",
        "fn untenanted_query",
        "fn admin_skip_rls",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Multi-tenancy bypass helpers banned — every query goes through RLS / tenant scope.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_destructive_ddl_helpers_in_codegen() {
    // E-50 templates will be reused across tenants. A helper that emits
    // DROP DATABASE / DROP TABLE / TRUNCATE TABLE / DELETE FROM into
    // template files is one silent-merge away from a tenant-wide data
    // disaster. Ban helpers that produce destructive DDL programmatically.
    let banned = [
        "fn emit_drop_database",
        "fn emit_drop_table",
        "fn emit_silent_table_clear",
        "fn unscoped_delete_all",
        "fn generate_destructive_migration",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Destructive DDL emitters banned in codegen — require explicit per-template review.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_bespoke_billing_adapter_parallel_to_canonical() {
    // Tier-B will land a canonical billing adapter (Stripe / Lemonsqueezy
    // / Paddle dispatch behind one trait). Bespoke parallel adapters
    // fragment the cost-cap + tenant-scope + cost-attribution policy.
    let banned = [
        "trait BespokeBillingAdapter",
        "struct CustomBillingV2",
        "trait QuickPaymentAdapter",
        "fn skip_billing_quota_check",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel billing adapters banned — extend the canonical adapter when Tier-B lands.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let files = workspace_rs_files();
    assert!(
        !files.is_empty(),
        "e50 contract walker found zero .rs files — the four anti-pattern asserts cannot be trusted",
    );
}
