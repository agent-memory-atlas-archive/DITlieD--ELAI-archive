//! E-12 Tier-A — production fullstack codegen regression contracts.
//!
//! Four negative-space guards + walker sanity. First contract added AFTER
//! the shared-walker refactor in E-12.A; zero edits to sibling contracts
//! needed. Adding a 6th contract will also be zero-edit thanks to the
//! `e<N>_*_contracts.rs` auto-skip pattern in tests/common/mod.rs.
//!
//! Locks invariants for the E-12 fullstack codegen path (extends E-11.A's
//! idea-framing + SaaS-archetype scaffolding into actual emitted code):
//!   - One canonical FrameworkAdapter trait (no parallel emitters).
//!   - No template-engine forking — scaffold byte-substitute pattern stays.
//!   - No write-files-without-checklist (the production checklist gate must
//!     run for every emitted file scaffold).
//!   - No untracked framework version literals (pin in elai_cc.toml).
//!
//! Tier-B will land: FrameworkAdapter trait + 5 concrete adapters
//! (NextAppRouter / SvelteKit / FastAPI / Axum / Tauri-Hono) +
//! production_checklist/ runner + SERVER_ENTRY_SEEDS const map +
//! AuditorMode::ProductionGrade hook + dashboard ScaffoldEmitterPanel. See
//! workflow/reasoning/E-12-decisions.md.

mod common;

use common::{collect_hits, format_hits, workspace_rs_files};

#[test]
fn no_parallel_framework_adapter_trait() {
    // FrameworkAdapter (Tier-B) will live in crates/orchestrator/src/
    // framework_adapter.rs. A parallel trait fragments per-stack emit logic
    // and means the production checklist runner cannot enforce a single
    // gate.
    let banned = [
        "trait FrameworkAdapterV2",
        "trait BespokeFrameworkAdapter",
        "trait FullstackEmitterV2",
        "struct AdHocScaffoldEmitter",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel FrameworkAdapter traits banned — extend the canonical trait when Tier-B lands.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_template_engine_dep_pollution() {
    // ELAI's scaffold crate uses byte-level `{{ident}}` substitution
    // (scaffold/substitute.rs). Adding tera / handlebars / askama /
    // minijinja / liquid as a dep is a workspace-policy decision that
    // requires explicit user confirmation per CLAUDE.md "No new
    // dependencies without confirmation". Catch silent additions.
    let banned = [
        "use tera::",
        "use handlebars::",
        "use askama::",
        "use minijinja::",
        "use liquid::",
        "tera::Tera::new",
        "handlebars::Handlebars::new",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Template-engine deps require explicit user confirmation. scaffold byte-substitute is the path.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_emit_without_production_checklist_gate() {
    // E-11 + E-12 architectural invariant: every emitted file scaffold
    // must pass the production checklist (auth / rate-limit / input-
    // validation / migration / observability / etc). A helper that writes
    // files without the gate breaks the demo-vs-deploy guard E-11 ships.
    let banned = [
        "fn emit_scaffold_no_checklist",
        "fn write_files_unaudited",
        "fn skip_production_gate",
        "fn unchecked_codegen",
        "fn raw_emit_no_audit",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Emit-without-checklist helpers banned — production gate must run.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_hardcoded_unpinned_framework_versions_in_source() {
    // Framework version pins belong in elai_cc.toml / workflow/archetypes/
    // templates/ — NOT inline in Rust source. A literal like
    // `"next@15.0.0"` baked into source means version bumps require code
    // edits + risk silent mismatch with the actual template files.
    let banned = [
        "next@15.0.0",
        "next@15.1.0",
        "next@15.2.0",
        "sveltekit@2.20",
        "fastapi==0.115",
        "axum = \"0.8.0\"",
        "axum = \"0.9.0\"",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Hard-coded framework version literals in source banned — pin in elai_cc.toml / templates.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let files = workspace_rs_files();
    assert!(
        !files.is_empty(),
        "e12 contract walker found zero .rs files — the four anti-pattern asserts cannot be trusted",
    );
}
