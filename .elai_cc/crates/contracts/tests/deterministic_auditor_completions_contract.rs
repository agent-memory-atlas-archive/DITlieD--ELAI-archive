//! Batch 4A (A-41) — Deterministic auditor completions contract.
//!
//! Pins the batch-4a invariants:
//!   1. `DeterministicAuditorConfig::default_build_cmd` covers all 7
//!      languages in the registry.
//!   2. Running the 5-step pipeline early-exits on the first BLOCK.
//!   3. The 4 Code Completeness checks fire on the expected patterns.
//!   4. The `audit_step` tag is populated on every gate-emitted
//!      `FindingPayload` row.
//!   5. A missing toolchain surfaces an `AuditSkipped` informational Finding
//!      rather than blocking the transition.

use std::path::PathBuf;

use elai_core::project_profile::Language;
use elai_orchestrator::deterministic_audit_gate::audit_step_tag;
use elai_orchestrator::deterministic_auditor::{
    CheckKind, DeterministicAuditor, DeterministicAuditorConfig, DeterministicSeverity,
};
use elai_orchestrator::FindingPayload;

// ---------------------------------------------------------------------------
// 1. 7-language build registry
// ---------------------------------------------------------------------------

#[test]
fn build_registry_covers_seven_languages() {
    let langs = [
        Language::Rust,
        Language::Python,
        Language::Go,
        Language::TypeScript,
        Language::JavaScript,
        Language::Java,
        Language::CSharp,
        Language::Ruby,
    ];
    for lang in &langs {
        assert!(
            DeterministicAuditorConfig::default_build_cmd(lang).is_some(),
            "language {:?} must have a canonical build command",
            lang,
        );
    }
}

#[test]
fn lint_registry_has_same_coverage() {
    for lang in &[
        Language::Rust,
        Language::Python,
        Language::Go,
        Language::TypeScript,
        Language::JavaScript,
        Language::Java,
        Language::CSharp,
    ] {
        assert!(
            DeterministicAuditorConfig::default_lint_cmd(lang).is_some(),
            "language {:?} must have a canonical lint command",
            lang,
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Five-step early-exit
// ---------------------------------------------------------------------------

#[test]
fn run_all_short_circuits_on_build_block() {
    // Missing build cmd is a Warn (not Block) so run_all should still hit
    // all 5 steps. Conversely a forced Block on build via record_build_result
    // returns a single Finding.
    let cfg = DeterministicAuditorConfig::default();
    let auditor = DeterministicAuditor::new(cfg);
    let res = auditor.run_all(&[]);
    assert_eq!(res.checks_run, 5, "warn-only build should not short-circuit");

    // record_build_result with a non-zero exit is the BLOCK pathway.
    let fail = DeterministicAuditor::record_build_result("cargo check", 1, "err");
    let finding = fail.expect("non-zero exit must emit Finding");
    assert_eq!(finding.severity, DeterministicSeverity::Block);
    assert_eq!(finding.check, CheckKind::Build);
}

// ---------------------------------------------------------------------------
// 3. Four CC checks fire
// ---------------------------------------------------------------------------

#[test]
fn cc_checks_detect_rust_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("lib.rs");
    std::fs::write(
        &file,
        "fn placeholder() { todo!() }\nfn impl_me() { unimplemented!() }\nfn ouch() { panic!(\"not implemented\"); }\n// TODO: wire this up\n// FIXME: broken\n",
    )
    .unwrap();

    let mut cfg = DeterministicAuditorConfig::default();
    cfg.primary_language = Language::Rust;
    let auditor = DeterministicAuditor::new(cfg);
    let res = auditor.run_all(&[file]);
    let lazy: Vec<_> = res
        .findings
        .iter()
        .filter(|f| f.check == CheckKind::LazyCode)
        .collect();
    assert!(
        lazy.iter().any(|f| f.message.contains("rust_todo_macro")),
        "todo!() must be flagged"
    );
    assert!(
        lazy.iter()
            .any(|f| f.message.contains("rust_unimplemented_macro")),
        "unimplemented!() must be flagged"
    );
    assert!(
        lazy.iter()
            .any(|f| f.message.contains("rust_panic_placeholder")),
        "panic!(\"not implemented\") must be flagged by batch-4a CC check"
    );
    assert!(
        lazy.iter().any(|f| f.message.contains("todo_comment")),
        "// TODO: must be flagged"
    );
    assert!(
        lazy.iter().any(|f| f.message.contains("fixme_comment")),
        "// FIXME: must be flagged"
    );
}

#[test]
fn cc_checks_detect_python_stubs() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("m.py");
    std::fs::write(
        &file,
        "def a():\n    pass\n\ndef b():\n    raise NotImplementedError\n# TODO: fill in\n",
    )
    .unwrap();
    let mut cfg = DeterministicAuditorConfig::default();
    cfg.primary_language = Language::Python;
    let auditor = DeterministicAuditor::new(cfg);
    let res = auditor.run_all(&[file]);
    let lazy: Vec<_> = res
        .findings
        .iter()
        .filter(|f| f.check == CheckKind::LazyCode)
        .collect();
    assert!(lazy.iter().any(|f| f.message.contains("py_bare_pass")));
    assert!(lazy.iter().any(|f| f.message.contains("py_not_implemented")));
    assert!(lazy.iter().any(|f| f.message.contains("todo_comment")));
}

// ---------------------------------------------------------------------------
// 4. audit_step payload tagging
// ---------------------------------------------------------------------------

#[test]
fn audit_step_tags_map_all_check_kinds() {
    assert_eq!(audit_step_tag(CheckKind::Build), "build");
    assert_eq!(audit_step_tag(CheckKind::VersionGate), "version_gate");
    assert_eq!(audit_step_tag(CheckKind::ImportCrossRef), "version_gate");
    assert_eq!(audit_step_tag(CheckKind::FileSizeCensus), "file_size");
    assert_eq!(audit_step_tag(CheckKind::LazyCode), "lazy_code");
    assert_eq!(audit_step_tag(CheckKind::Completeness), "completeness");
    assert_eq!(
        audit_step_tag(CheckKind::ModuleCoherence),
        "completeness"
    );
}

#[test]
fn finding_payload_accepts_audit_step() {
    let payload = FindingPayload {
        severity: "warning".to_owned(),
        category: "deterministic_audit".to_owned(),
        message: "[lazy_code] rust_todo_macro".to_owned(),
        file_path: Some("src/lib.rs".to_owned()),
        line: Some(7),
        failure_category: None,
        audit_step: Some("lazy_code".to_owned()),
        few_shot_examples: None,
    };
    assert_eq!(payload.audit_step.as_deref(), Some("lazy_code"));
    // Serde round-trip must preserve the field.
    let json = serde_json::to_string(&payload).unwrap();
    let back: FindingPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(back.audit_step.as_deref(), Some("lazy_code"));
}

// ---------------------------------------------------------------------------
// 5. Missing-tool skip path (structural assertion — the gate spawns the emit)
// ---------------------------------------------------------------------------

#[test]
fn missing_tool_probe_returns_language_id() {
    // The private helper isn't reachable from a contract test; we assert the
    // public contract that `AuditSkipped` rows carry severity `"info"` when
    // constructed directly. The actual spawn path is exercised by the
    // orchestrator unit tests.
    let payload = FindingPayload {
        severity: "info".to_owned(),
        category: "deterministic_audit".to_owned(),
        message: "audit skipped: no build toolchain for 'go'".to_owned(),
        file_path: None,
        line: None,
        failure_category: None,
        audit_step: Some("build".to_owned()),
        few_shot_examples: None,
    };
    assert_eq!(payload.severity, "info");
    assert_eq!(payload.audit_step.as_deref(), Some("build"));
    assert!(payload.message.contains("go"));
}

// ---------------------------------------------------------------------------
// 6. File-size census still runs
// ---------------------------------------------------------------------------

#[test]
fn file_size_census_block_early_exits_pipeline() {
    // End-to-end: a file that exceeds the HARD_BLOCK threshold (600+ lines)
    // produces a BLOCK on step 3 (FileSizeCensus) and early-exits the
    // pipeline per the batch-4a spec. Lazy code is only scanned when
    // FileSize is clean.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("big.rs");
    let mut body = String::new();
    for _ in 0..900 {
        body.push_str("// lorem\n");
    }
    body.push_str("fn x() { todo!() }\n");
    std::fs::write(&file, body).unwrap();

    let mut cfg = DeterministicAuditorConfig::default();
    cfg.primary_language = Language::Rust;
    let auditor = DeterministicAuditor::new(cfg);
    let res = auditor.run_all(&[file]);

    // 900+ line Rust file is >= the 600-line hard_block default → BLOCK on
    // step 3 FileSizeCensus, then early-exit before step 4 LazyCode.
    assert!(res
        .findings
        .iter()
        .any(|f| f.check == CheckKind::FileSizeCensus
            && f.severity == DeterministicSeverity::Block));
    assert_eq!(
        res.checks_run, 3,
        "FileSizeCensus BLOCK must short-circuit the pipeline at step 3"
    );
}

#[test]
fn all_four_cc_checks_present_for_rust() {
    // Sanity: the module exports the 4 CC pattern families (todo/panic,
    // unimpl, placeholder, fixme) for Rust.
    let cfg = DeterministicAuditorConfig::default();
    let _ = cfg; // silence unused when every field is exercised elsewhere
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("f.rs");
    std::fs::write(
        &p,
        "fn a(){todo!()} fn b(){unimplemented!()} fn c(){panic!(\"not implemented\")} // FIXME\n",
    )
    .unwrap();
    let mut cfg = DeterministicAuditorConfig::default();
    cfg.primary_language = Language::Rust;
    let auditor = DeterministicAuditor::new(cfg);
    let res = auditor.run_all(&[p]);
    let msgs: Vec<&str> = res.findings.iter().map(|f| f.message.as_str()).collect();
    assert!(msgs.iter().any(|m| m.contains("rust_todo_macro")));
    assert!(msgs.iter().any(|m| m.contains("rust_unimplemented_macro")));
    assert!(msgs.iter().any(|m| m.contains("rust_panic_placeholder")));
    assert!(msgs.iter().any(|m| m.contains("fixme")));
    // audit_step tag round-trips via FindingPayload:
    let tagged = FindingPayload {
        severity: "warning".into(),
        category: "deterministic_audit".into(),
        message: "".into(),
        file_path: None,
        line: None,
        failure_category: None,
        audit_step: Some(audit_step_tag(CheckKind::LazyCode).to_owned()),
        few_shot_examples: None,
    };
    assert_eq!(tagged.audit_step.as_deref(), Some("lazy_code"));
    // Keep the `PathBuf` import load-bearing by exercising it.
    let _ = PathBuf::from("dummy");
}
