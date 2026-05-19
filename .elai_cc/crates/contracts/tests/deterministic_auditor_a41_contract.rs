//! A-41 contract tests: deterministic auditor completions.
//!
//! Validates:
//!   - Early-exit on BLOCK at any step (not just Build)
//!   - default_lint_cmd() coverage for 7 languages
//!   - AuditorConfig defaults

use std::path::PathBuf;

use elai_core::project_profile::{BuildCommand, CommandScope, Language};
use elai_orchestrator::{
    CheckKind, DeterministicAuditor, DeterministicAuditorConfig,
    DeterministicSeverity,
};
use elai_rules::schema::DeprecatedPatternEntry;

// ---------------------------------------------------------------------------
// Helper: build a default config with a passing build command
// ---------------------------------------------------------------------------

fn config_with_passing_build() -> DeterministicAuditorConfig {
    // C-28: disable Step 6 (mutation) for these legacy a41 contracts so
    // their checks_run / warn_count assertions stay stable. Mutation
    // testing has its own contract surface in c28_mutation_testing_contract.
    let mut mutation = elai_orchestrator::MutationConfig::default();
    mutation.enabled = false;
    // C-29: disable Step 7 (metamorphic) for the same reason — assertions
    // pin checks_run before C-29 landed.
    let mut metamorphic = elai_orchestrator::MetamorphicConfig::default();
    metamorphic.enabled = false;
    DeterministicAuditorConfig {
        enabled: true,
        build_cmds: vec![BuildCommand {
            cmd: "echo ok".into(),
            working_dir: PathBuf::from("."),
            env: Default::default(),
            timeout_s: 120,
            scope: CommandScope::Full,
            verified: true,
        }],
        lint_cmds: Vec::new(),
        deprecated_patterns: Vec::new(),
        primary_language: Language::Rust,
        project_root: PathBuf::from("."),
        file_discipline: Default::default(),
        mutation,
        metamorphic,
    }
}

// ---------------------------------------------------------------------------
// Contract: early exit on VersionGate BLOCK
// ---------------------------------------------------------------------------

#[test]
fn early_exit_on_version_gate_block() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("lib.rs");
    std::fs::write(&file, "use blake2::Digest;\n").unwrap();

    let mut config = config_with_passing_build();
    config.deprecated_patterns = vec![DeprecatedPatternEntry {
        deprecated_name: "blake2".into(),
        replacement_name: "blake3".into(),
        severity: "BLOCK".into(),
    }];
    let auditor = DeterministicAuditor::new(config);
    let result = auditor.run_all(&[file]);

    assert!(!result.passed, "should fail on VersionGate BLOCK");
    assert!(
        result.checks_run <= 2,
        "should early-exit after step 2 (VersionGate), got checks_run={}",
        result.checks_run
    );
    assert!(
        result.findings.iter().any(|f| f.check == CheckKind::VersionGate
            && f.severity == DeterministicSeverity::Block),
        "must contain a VersionGate BLOCK finding"
    );
}

// ---------------------------------------------------------------------------
// Contract: early exit on FileSizeCensus BLOCK
// ---------------------------------------------------------------------------

#[test]
fn early_exit_on_file_size_census_block() {
    let dir = tempfile::tempdir().unwrap();
    // Create an oversized file (>600 lines default hard_block)
    let file = dir.path().join("huge.rs");
    let content: String = (0..700).map(|i| format!("fn f_{i}() {{}}\n")).collect();
    std::fs::write(&file, content).unwrap();

    let config = config_with_passing_build();
    let auditor = DeterministicAuditor::new(config);
    let result = auditor.run_all(&[file]);

    assert!(!result.passed, "should fail on FileSizeCensus BLOCK");
    assert!(
        result.checks_run <= 3,
        "should early-exit after step 3 (FileSizeCensus), got checks_run={}",
        result.checks_run
    );
    assert!(
        result.findings.iter().any(|f| f.check == CheckKind::FileSizeCensus
            && f.severity == DeterministicSeverity::Block),
        "must contain a FileSizeCensus BLOCK finding"
    );
}

// ---------------------------------------------------------------------------
// Contract: full pass runs all 5 checks
// ---------------------------------------------------------------------------

#[test]
fn full_pass_runs_all_checks() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("clean.rs");
    std::fs::write(
        &file,
        "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    let config = config_with_passing_build();
    let auditor = DeterministicAuditor::new(config);
    let result = auditor.run_all(&[file]);

    assert!(result.passed, "clean file should pass all checks");
    assert_eq!(
        result.checks_run, 5,
        "all 5 checks must run on a clean pass"
    );
    assert_eq!(result.block_count(), 0);
}

// ---------------------------------------------------------------------------
// Contract: default_lint_cmd covers Rust
// ---------------------------------------------------------------------------

#[test]
fn default_lint_cmd_rust_returns_some() {
    let cmd = DeterministicAuditorConfig::default_lint_cmd(&Language::Rust);
    assert!(cmd.is_some(), "Rust must have a default lint command");
    assert!(
        cmd.unwrap().contains("clippy"),
        "Rust lint command must include clippy"
    );
}

// ---------------------------------------------------------------------------
// Contract: default_lint_cmd covers all 7 languages
// ---------------------------------------------------------------------------

#[test]
fn default_lint_cmd_covers_seven_languages() {
    let languages = [
        Language::Rust,
        Language::Python,
        Language::Go,
        Language::TypeScript,
        Language::Java,
        Language::CSharp,
        Language::Ruby,
    ];
    for lang in &languages {
        let cmd = DeterministicAuditorConfig::default_lint_cmd(lang);
        assert!(
            cmd.is_some(),
            "default_lint_cmd must return Some for {:?}",
            lang
        );
        assert!(
            !cmd.unwrap().is_empty(),
            "default_lint_cmd must return non-empty for {:?}",
            lang
        );
    }
}

// ---------------------------------------------------------------------------
// Contract: unknown language returns None for lint cmd
// ---------------------------------------------------------------------------

#[test]
fn default_lint_cmd_unknown_returns_none() {
    let cmd = DeterministicAuditorConfig::default_lint_cmd(&Language::Unknown);
    assert!(cmd.is_none(), "Unknown language should return None");
}
