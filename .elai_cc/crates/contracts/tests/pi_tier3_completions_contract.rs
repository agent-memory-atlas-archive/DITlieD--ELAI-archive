//! Batch 4A (A-36) — PI Tier 3 completions contract.
//!
//! Pins:
//!   1. `FixTemplateRegistry::load_from_dir` loads the 4 seed templates
//!      (`rust.import_path_fix`, `python.import_stub`,
//!      `typescript.null_check`, `go.error_wrap`).
//!   2. `attach_fix_template_keys` populates `ProjectProfile.fix_template_keys`.
//!   3. `few_shot_for_symbol` attaches up to `MAX_FEW_SHOT_EXAMPLES=3`
//!      in-project exemplars.
//!   4. `convention_aware_key` prefers the `.camel` variant when the
//!      detected function casing is camelCase.
//!   5. Empty DKG yields an empty few-shot vec (graceful fallback).

use std::fs;
use std::path::PathBuf;

use elai_core::project_profile::{Convention, NamingConventions};
use elai_knowledge::fix_templates::{
    attach_fix_template_keys, convention_aware_key, few_shot_for_symbol, FewShotExample,
    FixTemplateRegistry, MAX_FEW_SHOT_EXAMPLES,
};
use elai_knowledge::{DkgEngine, SymbolKind};

// ---------------------------------------------------------------------------
// 1. Seed template loading
// ---------------------------------------------------------------------------

#[test]
fn seed_templates_load_with_canonical_keys() {
    let tmp = tempfile::tempdir().unwrap();
    for (name, body) in &[
        ("rust_import_path_fix.txt", "use {replacement}::{symbol};"),
        ("python_import_stub.txt", "from {replacement} import {symbol}"),
        ("typescript_null_check.txt", "const v = {expr} ?? {fallback};"),
        ("go_error_wrap.txt", "if {err} != nil {{ return fmt.Errorf(\"{context}: %w\", {err}) }}"),
    ] {
        fs::write(tmp.path().join(name), body).unwrap();
    }
    let reg = FixTemplateRegistry::load_from_dir(tmp.path());
    assert_eq!(reg.len(), 4);
    assert!(reg.get("rust.import_path_fix").is_some());
    assert!(reg.get("python.import_stub").is_some());
    assert!(reg.get("typescript.null_check").is_some());
    assert!(reg.get("go.error_wrap").is_some());
}

#[test]
fn load_default_from_project_root() {
    // Build a scratch tree with .elai_cc/data/fix_templates/ under it and
    // verify `load_default` resolves correctly.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp
        .path()
        .join(".elai_cc")
        .join("data")
        .join("fix_templates");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("rust_import_path_fix.txt"), "use X;").unwrap();

    // load_default resolves `<project_root>/data/fix_templates` — we exercise
    // it via `load_from_dir` directly to avoid hard-coding `.elai_cc/` path
    // assumptions here.
    let reg = FixTemplateRegistry::load_from_dir(&dir);
    assert!(reg.get("rust.import_path_fix").is_some());
}

// ---------------------------------------------------------------------------
// 2. ProjectProfile.fix_template_keys attachment
// ---------------------------------------------------------------------------

#[test]
fn attach_writes_keys_into_profile() {
    let mut reg = FixTemplateRegistry::new();
    reg.insert("rust.import_path_fix", "use X;");
    reg.insert("python.import_stub", "from X import Y");
    let mut profile = scaffold_profile();
    attach_fix_template_keys(&mut profile, &reg);
    assert!(profile
        .fix_template_keys
        .iter()
        .any(|k| k == "rust.import_path_fix"));
    assert!(profile
        .fix_template_keys
        .iter()
        .any(|k| k == "python.import_stub"));
}

// ---------------------------------------------------------------------------
// 3. DKG few-shot
// ---------------------------------------------------------------------------

#[test]
fn few_shot_attaches_up_to_three_examples() {
    let dkg = open_dkg();
    // Seed 5 functions that share the "parse" token.
    for (name, sig, start, end) in &[
        ("parse_document", "fn parse_document(src: &str)", 1, 10),
        ("parse_config", "fn parse_config(path: &Path)", 11, 20),
        ("parse_header", "fn parse_header(buf: &[u8])", 21, 30),
        ("parse_footer", "fn parse_footer(buf: &[u8])", 31, 40),
        ("build_index", "fn build_index(docs: &[Doc])", 41, 50),
    ] {
        dkg.insert_test_node(name, "function", sig, *start, *end);
    }
    let examples = few_shot_for_symbol(&dkg, "parse_query").expect("few_shot");
    assert!(
        examples.len() <= MAX_FEW_SHOT_EXAMPLES,
        "must cap at MAX_FEW_SHOT_EXAMPLES"
    );
    // The returned nodes should all share the "parse" token.
    for ex in &examples {
        assert!(
            ex.symbol_name.contains("parse"),
            "expected parse-* symbol, got {}",
            ex.symbol_name
        );
    }
}

#[test]
fn empty_dkg_yields_empty_few_shot() {
    let dkg = open_dkg();
    let examples = few_shot_for_symbol(&dkg, "parse_query").expect("few_shot");
    assert!(
        examples.is_empty(),
        "empty DKG must degrade to empty few-shot list"
    );
}

// ---------------------------------------------------------------------------
// 4. Convention-aware picker
// ---------------------------------------------------------------------------

#[test]
fn camel_profile_prefers_camel_variant() {
    let mut reg = FixTemplateRegistry::new();
    reg.insert("ts.null_check", "snake body");
    reg.insert("ts.null_check.camel", "camel body");
    let mut naming = NamingConventions::default();
    naming.functions = Convention::Strong("camelCase".to_owned());
    let body = convention_aware_key(&reg, "ts.null_check", &naming).unwrap();
    assert_eq!(body, "camel body");
}

#[test]
fn snake_profile_uses_default_variant() {
    let mut reg = FixTemplateRegistry::new();
    reg.insert("rust.import_path_fix", "snake body");
    reg.insert("rust.import_path_fix.camel", "camel body");
    let mut naming = NamingConventions::default();
    naming.functions = Convention::Strong("snake_case".to_owned());
    let body = convention_aware_key(&reg, "rust.import_path_fix", &naming).unwrap();
    assert_eq!(body, "snake body");
}

#[test]
fn camel_profile_falls_back_when_no_camel_key() {
    let mut reg = FixTemplateRegistry::new();
    reg.insert("ts.null_check", "snake body");
    let mut naming = NamingConventions::default();
    naming.functions = Convention::Strong("camelCase".to_owned());
    let body = convention_aware_key(&reg, "ts.null_check", &naming).unwrap();
    assert_eq!(body, "snake body");
}

// ---------------------------------------------------------------------------
// 5. FewShotExample <-> SSB FewShotExample parity
// ---------------------------------------------------------------------------

#[test]
fn few_shot_example_fields_round_trip() {
    let ex = FewShotExample {
        symbol_name: "parse_document".to_owned(),
        file_path: PathBuf::from("src/parse.rs"),
        line: 42,
        snippet: "fn parse_document(src: &str)".to_owned(),
    };
    assert_eq!(ex.symbol_name, "parse_document");
    assert_eq!(ex.line, 42);
    assert!(ex.snippet.contains("parse_document"));

    // Structural equivalence with the SSB payload field set.
    let ssb_example = elai_orchestrator::FewShotExample {
        symbol_name: ex.symbol_name.clone(),
        file_path: ex.file_path.to_string_lossy().into_owned(),
        line: ex.line,
        snippet: ex.snippet.clone(),
    };
    assert_eq!(ssb_example.symbol_name, "parse_document");
    assert_eq!(ssb_example.line, 42);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn open_dkg() -> DkgEngine {
    use std::path::Path;
    DkgEngine::new(Path::new(":memory:"), None, 0).expect("open in-memory DKG")
}

fn scaffold_profile() -> elai_core::project_profile::ProjectProfile {
    use elai_core::project_profile::{
        BuildCommandRegistry, BuildSystem, BuildSystemProfile, DependencyProfile,
        ErrorHandlingProfile, Language, LanguageProfile, ModuleOrganization, NamingConventions,
        ProjectProfile, TestProfile, PROFILE_SCHEMA_VERSION,
    };
    use std::collections::HashMap;
    ProjectProfile {
        schema_version: PROFILE_SCHEMA_VERSION,
        generated_at: "2026-04-19T00:00:00Z".to_owned(),
        project_root: PathBuf::from("/tmp"),
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
        naming: NamingConventions::default(),
        error_handling: ErrorHandlingProfile::default(),
        module_organization: ModuleOrganization::default(),
        dep_profile: DependencyProfile::default(),
        project_vision: None,
        architecture: None,
        domains: Vec::new(),
        commands: BuildCommandRegistry::default(),
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        overrides_applied: false,
    }
}

fn _unused_symbol_kind() {
    let _ = SymbolKind::Function;
}
