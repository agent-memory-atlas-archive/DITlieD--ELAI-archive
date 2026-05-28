//! E-11 catalog+recommender slice -- prior-art registry contract.
//!
//! Verifies the 15-entry registry (9 tools + 6 reports) ships intact,
//! each entry has a non-empty name/url/gap/report_finding, archetype
//! tag lookup is non-empty for each archetype, and the catalog entry
//! count matches the 5-archetype taxonomy.

use elai_orchestrator::architecture::{
    ALL_SAAS_ARCHETYPES, ArchetypeCatalog, PRIOR_ART, PriorArtRegistry, SaasArchetype,
};

#[test]
fn registry_has_at_least_eight_entries() {
    // The 2024-2026 survey converges on >=8 prior-art product / report
    // citations. Trimming below 8 requires updating the
    // workflow/improvement-sweep-2026-05-19/reports/
    // 11-idea-to-saas-prior-art.md citation set first.
    assert!(
        PriorArtRegistry::len() >= 8,
        "PRIOR_ART must carry >=8 entries; current length: {}",
        PriorArtRegistry::len()
    );
}

#[test]
fn registry_has_exactly_fifteen_entries() {
    // Locked at 9 products + 6 reports = 15. Bumping requires a
    // matching report-citation update.
    assert_eq!(
        PRIOR_ART.len(),
        15,
        "PRIOR_ART carries exactly 15 entries (9 tools + 6 reports). \
         Adjusting requires bumping the canonical survey citation set."
    );
}

#[test]
fn every_entry_has_nonempty_fields() {
    for entry in PriorArtRegistry::iter() {
        assert!(!entry.name.is_empty(), "name empty: {:?}", entry);
        assert!(!entry.url.is_empty(), "url empty: {:?}", entry);
        assert!(!entry.gap.is_empty(), "gap empty: {:?}", entry);
        assert!(
            !entry.report_finding.is_empty(),
            "report_finding empty: {:?}",
            entry
        );
        assert!(
            !entry.archetypes.is_empty(),
            "archetypes empty for: {}",
            entry.name
        );
    }
}

#[test]
fn every_archetype_has_at_least_one_tagged_entry() {
    for arch in ALL_SAAS_ARCHETYPES {
        let entries = PriorArtRegistry::entries_for_archetype(*arch);
        assert!(
            !entries.is_empty(),
            "SaasArchetype::{:?} has zero prior-art entries -- registry \
             must tag every archetype with at least one entry",
            arch
        );
    }
}

#[test]
fn catalog_has_one_entry_per_archetype() {
    assert_eq!(
        ArchetypeCatalog::len(),
        ALL_SAAS_ARCHETYPES.len(),
        "ArchetypeCatalog must have one entry per SaasArchetype; \
         mismatch would mean a variant was added without a catalog entry."
    );

    for arch in ALL_SAAS_ARCHETYPES {
        let entry = ArchetypeCatalog::entry_for(*arch);
        assert_eq!(entry.archetype, *arch);
        // Each archetype catalog entry has exactly 4 scope-clarity
        // questions (the report's 4-point rubric).
        assert_eq!(
            entry.scope_clarity_profile.len(),
            4,
            "scope_clarity_profile must have exactly 4 questions per \
             the report's 4-point rubric; archetype: {:?}",
            arch
        );
    }
}

#[test]
fn catalog_iter_returns_five_entries() {
    let count = ArchetypeCatalog::iter().count();
    assert_eq!(count, 5);
}

#[test]
fn catalog_production_checklist_subsets_are_nonempty() {
    for arch in ALL_SAAS_ARCHETYPES {
        let subset =
            ArchetypeCatalog::production_checklist_subset_for(*arch);
        assert!(
            !subset.is_empty(),
            "production checklist subset for {:?} must be non-empty",
            arch
        );
    }
}

#[test]
fn b2b_marketplace_runs_all_fifteen_checks() {
    let subset = ArchetypeCatalog::production_checklist_subset_for(
        SaasArchetype::B2bMarketplace,
    );
    assert_eq!(
        subset.len(),
        15,
        "B2bMarketplace is the strictest archetype -- must run all 15 \
         production-grade checks. Per-archetype subset table at \
         architecture::catalog GAP-2."
    );
}

#[test]
fn internal_tooling_excludes_lighthouse_and_rls() {
    use elai_orchestrator::framework_profile::SecurityCheckId;
    let subset = ArchetypeCatalog::production_checklist_subset_for(
        SaasArchetype::InternalTooling,
    );
    assert!(
        !subset.contains(SecurityCheckId::Lighthouse),
        "InternalTooling has no public UI -- Lighthouse must NOT be in subset"
    );
    assert!(
        !subset.contains(SecurityCheckId::Sqlite3Rls),
        "InternalTooling is single-tenant -- Sqlite3Rls must NOT be in subset"
    );
}
