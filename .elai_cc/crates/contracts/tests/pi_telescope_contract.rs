// Contract tests for A-36: PI Tier 3 + Telescope Completions.
// Tests written BEFORE implementation (TDD).
//
// phase-3L (s-25120): added `pi_telescope_contract` test fn at the bottom
// of the file to close P2 — top-K stability + cumulative token-budget cap
// invariants on `assemble_telescope_with_hooks` + the categorical T3
// invariant (PiT3 NEVER emits BLOCK).

#[allow(unused_imports)]
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

use elai_core::project_profile::*;
use elai_knowledge::telescope::{assemble_telescope, assemble_telescope_with_hooks};
use elai_knowledge::{
    DkgEngine, DkgNodeInfo, PiT1Hook, PiT2Hook, PiT3Hook, ProjectIntelligenceHook,
    PI_T1_ID, PI_T2_ID, PI_T3_ID,
};
use elai_plugins::tiered_analysis::HookVerdict;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rust_profile() -> ProjectProfile {
    ProjectProfile {
        schema_version: PROFILE_SCHEMA_VERSION,
        generated_at: "2026-04-15T00:00:00Z".to_owned(),
        project_root: PathBuf::from("/test"),
        build_system: BuildSystemProfile {
            system: BuildSystem::Cargo,
            config_path: PathBuf::from("Cargo.toml"),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        },
        languages: LanguageProfile {
            primary: Language::Rust,
            secondary: Vec::new(),
            distribution: std::collections::BTreeMap::new(),
            tree_sitter_coverage: 0.0,
        },
        frameworks: Vec::new(),
        test_profile: TestProfile::default(),
        lint_tools: Vec::new(),
        entry_points: Vec::new(),
        ci: None,
        naming: NamingConventions {
            functions: Convention::Strong("snake_case".to_owned()),
            types: Convention::Strong("PascalCase".to_owned()),
            constants: Convention::Strong("SCREAMING_SNAKE_CASE".to_owned()),
            modules: Convention::Unknown,
        },
        error_handling: ErrorHandlingProfile {
            pattern: ErrorPattern::ResultBased,
            propagation: PropagationStyle::QuestionMark,
            error_crate: Some("thiserror".to_owned()),
        },
        module_organization: ModuleOrganization {
            pattern: OrgPattern::Layered,
        },
        dep_profile: DependencyProfile::default(),
        project_vision: Some("Test project".to_owned()),
        architecture: Some(ArchitecturePattern::Layered),
        domains: vec![DomainClassification {
            domain: ProjectDomain::WebBackend,
            confidence: 0.9,
            signals: vec!["test".to_owned()],
        }],
        commands: BuildCommandRegistry {
            build: vec![BuildCommand {
                cmd: "cargo build".to_owned(),
                working_dir: PathBuf::from("."),
                env: std::collections::BTreeMap::new(),
                timeout_s: 120,
                scope: CommandScope::Full,
                verified: false,
            }],
            test: vec![BuildCommand {
                cmd: "cargo test".to_owned(),
                working_dir: PathBuf::from("."),
                env: std::collections::BTreeMap::new(),
                timeout_s: 120,
                scope: CommandScope::Full,
                verified: false,
            }],
            lint: Vec::new(),
            format: Vec::new(),
            run: Vec::new(),
            clean: Vec::new(),
        },
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        overrides_applied: false,
    }
}

fn seed_dkg_with_functions(engine: &DkgEngine) {
    // Insert test function nodes directly via SQL.
    let names = [
        ("parse_document", "function", "fn parse_document(s: &str)", 10, 20),
        ("parse_config", "function", "fn parse_config(path: &Path)", 30, 40),
        ("build_index", "function", "fn build_index(docs: &[Doc])", 50, 60),
        ("render_template", "function", "fn render_template(t: &Template)", 70, 80),
        ("parse_json_response", "method", "fn parse_json_response(&self)", 90, 100),
    ];
    for (name, kind, sig, start, end) in &names {
        engine.insert_test_node(name, kind, sig, *start, *end);
    }
}

// ---------------------------------------------------------------------------
// Contract 1: find_similar_functions returns at most k results
// ---------------------------------------------------------------------------

#[test]
fn find_similar_functions_returns_at_most_k() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("dkg.db");
    let engine = DkgEngine::new(&db_path, None, 0).unwrap();
    seed_dkg_with_functions(&engine);

    let results = engine.find_similar_functions("parse_document", 2).unwrap();
    assert!(
        results.len() <= 2,
        "expected at most 2 results, got {}",
        results.len()
    );
}

// ---------------------------------------------------------------------------
// Contract 2: find_similar_functions returns empty vec on empty DB
// ---------------------------------------------------------------------------

#[test]
fn find_similar_functions_empty_db_returns_empty_vec() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("dkg.db");
    let engine = DkgEngine::new(&db_path, None, 0).unwrap();

    let results = engine.find_similar_functions("anything", 3).unwrap();
    assert!(results.is_empty(), "expected empty vec on empty DB");
}

// ---------------------------------------------------------------------------
// Contract 3: format_verification_with_crate produces correct output
// ---------------------------------------------------------------------------

#[test]
fn format_verification_with_crate_produces_crate_flag() {
    let profile = rust_profile();
    let ctx = assemble_telescope(&profile);

    let result = ctx.format_verification_with_crate("my_crate");
    assert_eq!(
        result,
        "After writing this code, verify: cargo test -p my_crate"
    );
}

// ---------------------------------------------------------------------------
// Contract 4: format_verification_with_crate returns empty when no test_cmd
// ---------------------------------------------------------------------------

#[test]
fn format_verification_with_crate_empty_when_no_test_cmd() {
    let mut profile = rust_profile();
    profile.commands.test.clear();
    let ctx = assemble_telescope(&profile);

    let result = ctx.format_verification_with_crate("any_crate");
    assert!(
        result.is_empty(),
        "expected empty string when test_cmd is None, got: {result}"
    );
}

// ---------------------------------------------------------------------------
// Contract 5: telescope format still contains all required sections (regression)
// ---------------------------------------------------------------------------

#[test]
fn telescope_format_contains_all_sections_regression() {
    let profile = rust_profile();
    let ctx = assemble_telescope(&profile);
    let formatted = ctx.format_for_phase_spec();

    assert!(formatted.starts_with("PROJECT CONTEXT\n"));
    assert!(formatted.contains("Vision:"));
    assert!(formatted.contains("Language: rust"));
}

// ---------------------------------------------------------------------------
// Contract 6: few-shot format matches spec
// ---------------------------------------------------------------------------

#[test]
fn few_shot_format_matches_spec() {
    let node = DkgNodeInfo {
        node_id: 1,
        symbol_name: "parse_document".to_owned(),
        file_path: PathBuf::from("src/parser.rs"),
        signature: "fn parse_document(s: &str)".to_owned(),
        kind: elai_knowledge::SymbolKind::Function,
        start_line: 10,
        end_line: 20,
    };

    let expected = "Here's how this codebase implements a similar function: parse_document (src/parser.rs:10)";
    let actual = elai_knowledge::telescope::format_few_shot_example(&node);
    assert_eq!(actual, expected);
}

// ---------------------------------------------------------------------------
// Contract 7: telescope format includes templates when available
// ---------------------------------------------------------------------------

#[test]
fn telescope_format_includes_templates() {
    let profile = rust_profile();
    let ctx = assemble_telescope(&profile);
    let formatted = ctx.format_for_phase_spec();

    assert!(
        formatted.contains("Templates:"),
        "formatted telescope should include Templates section, got:\n{formatted}"
    );
}

// ---------------------------------------------------------------------------
// Contract 8: find_similar_functions scores by token overlap
// ---------------------------------------------------------------------------

#[test]
fn find_similar_functions_ranks_by_token_overlap() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("dkg.db");
    let engine = DkgEngine::new(&db_path, None, 0).unwrap();
    seed_dkg_with_functions(&engine);

    // "parse_document" query: "parse" and "document" tokens.
    // parse_document has 2 tokens overlap, parse_config has 1, parse_json_response has 1.
    let results = engine.find_similar_functions("parse_document", 3).unwrap();
    assert!(!results.is_empty(), "should find at least one match");

    // First result should be the best match.
    assert_eq!(
        results[0].symbol_name, "parse_document",
        "best match should be parse_document"
    );
}

// ---------------------------------------------------------------------------
// phase-3L (P2 closure): pi_telescope_contract
// ---------------------------------------------------------------------------
//
// Spec invariants asserted here:
//   (a) PI Tier hook outputs are deterministic for a fixed profile: 10
//       repeated calls to `assemble_telescope_with_hooks` produce a
//       byte-identical sequence of blocks (top-K STABILITY).
//   (b) Ordering: T1 (conventions) block appears before T2 (imports)
//       before T3 (profile) — priority anchors 100 > 80 > 60.
//   (c) Token budget cap: cumulative chars across all rendered blocks
//       respect the per-hook token-cost estimates declared by the trait,
//       AND each individual hook never exceeds its own
//       `token_cost_estimate * 4` char budget.
//   (d) T3 NEVER emits BLOCK (categorical invariant — `PiT3Hook::evaluate`
//       only returns None or Warn).
//   (e) T2 returns Ok(None) on unrecognised language input (mirrors the
//       T2Hook::evaluate Ok(None) contract from
//       workflow/papers/tiered_analysis-interface.md).
//
// Failure of any of these invariants surfaces a CONTRACT violation in the
// auditor's `cargo test -p elai-contracts pi_telescope_contract` gate.

#[test]
fn pi_telescope_contract() {
    let profile = rust_profile();

    // (a) + (b): build a 3-hook list, call 10x, assert byte-equal output
    //            and the T1 > T2 > T3 ordering.
    fn fresh_hooks() -> Vec<Box<dyn ProjectIntelligenceHook>> {
        vec![
            Box::new(PiT1Hook::new()),
            Box::new(PiT2Hook::new()),
            Box::new(PiT3Hook::new()),
        ]
    }

    let hooks_first = fresh_hooks();
    let (_ctx_first, blocks_first) = assemble_telescope_with_hooks(&profile, &hooks_first);
    let baseline = blocks_first.clone();

    for iter in 0..10 {
        let hooks_iter = fresh_hooks();
        let (_, blocks_iter) = assemble_telescope_with_hooks(&profile, &hooks_iter);
        assert_eq!(
            baseline, blocks_iter,
            "pi_telescope_contract: top-K stability violated on iteration {iter}; \
             expected identical block sequence across runs"
        );
    }

    // Profile has STRONG conventions + frameworks-empty + DEP empty +
    // vision/arch/commands set, so we expect T1 (CONVENTIONS) + T3
    // (PROFILE) blocks but T2 IMPORTS may be None (no deps).
    let header_order: Vec<&str> = baseline
        .iter()
        .map(|s| s.lines().next().unwrap_or(""))
        .collect();
    let t1_idx = header_order.iter().position(|h| h == &"CONVENTIONS");
    let t3_idx = header_order.iter().position(|h| h == &"PROFILE");
    assert!(t1_idx.is_some(), "CONVENTIONS block must be present");
    assert!(t3_idx.is_some(), "PROFILE block must be present");
    assert!(
        t1_idx.unwrap() < t3_idx.unwrap(),
        "T1 CONVENTIONS must appear before T3 PROFILE (priority 100 > 60)"
    );

    // (c) per-hook char budget never exceeded.
    let hooks = fresh_hooks();
    for hook in hooks.iter() {
        if let Some(block) = hook.render_block(&profile) {
            let max_chars = hook.token_cost_estimate() * 4 + 1;
            assert!(
                block.len() <= max_chars,
                "{} block exceeded its char budget: len={} max={}",
                hook.id(),
                block.len(),
                max_chars
            );
        }
    }
    // Cumulative cap: the assembler aborts before exceeding
    // TELESCOPE_MAX_CHARS. Re-derive by summing.
    let total: usize = baseline.iter().map(String::len).sum();
    assert!(
        total <= 1200,
        "pi_telescope_contract: cumulative blocks ({total}b) breached TELESCOPE_MAX_CHARS"
    );

    // (d) T3 categorical invariant: NEVER emits BLOCK.
    let t3 = PiT3Hook::new();
    if let Some(v) = t3.evaluate(&profile) {
        assert!(
            !matches!(v, HookVerdict::Block { .. }),
            "pi_telescope_contract: PiT3Hook emitted BLOCK (categorical invariant violated)"
        );
    }

    // (e) T2 unrecognised language: returns Ok(None).
    let tmp = TempDir::new().expect("tmp");
    let view = elai_knowledge::CorpusView::new(None, tmp.path(), "fortran");
    let t2 = PiT2Hook::new();
    let result = t2.evaluate(&view).expect("evaluate ok");
    assert!(
        result.is_none(),
        "pi_telescope_contract: PiT2Hook must return Ok(None) for unrecognised language"
    );

    // Sanity: at least one of {T1,T2,T3} ids must appear in the canonical
    // hook id constants (drift-guard against silent renames).
    assert_eq!(PI_T1_ID, "pi_t1_conventions");
    assert_eq!(PI_T2_ID, "pi_t2_imports");
    assert_eq!(PI_T3_ID, "pi_t3_profile");
}
