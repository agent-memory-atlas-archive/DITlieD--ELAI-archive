// Contract tests for A-36: PI Tier 3 + Telescope Completions.
// Tests written BEFORE implementation (TDD).

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

use elai_core::project_profile::*;
use elai_knowledge::telescope::assemble_telescope;
use elai_knowledge::{DkgEngine, DkgNodeInfo};

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
            distribution: HashMap::new(),
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
                env: HashMap::new(),
                timeout_s: 120,
                scope: CommandScope::Full,
                verified: false,
            }],
            test: vec![BuildCommand {
                cmd: "cargo test".to_owned(),
                working_dir: PathBuf::from("."),
                env: HashMap::new(),
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
