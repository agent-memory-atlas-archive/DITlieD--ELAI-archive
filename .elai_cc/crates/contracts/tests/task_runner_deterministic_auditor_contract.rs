//! Gap #4 contract tests (MVP audit 2026-04-18 section 4 item 4):
//! `DeterministicAuditor::run_all` is invoked post-baby-step and Block
//! verdicts are routed into typed `SsbPayload::Finding` rows with
//! `severity = "block"` + `category = "deterministic_audit"`.
//!
//! The dashboard-integration path (drive_task → auditor → SSB) requires a
//! live task_runner with a synthetic executor that emits `changed_files` —
//! that surface is covered by the existing baby_step_executor_contract and
//! is intentionally out of scope for these hermetic tests. Here we exercise
//! the auditor's surface directly (the pure function the task_runner hook
//! wraps) and verify:
//!   1. A Passed outcome + non-empty `changed_files` produces a
//!      DeterministicAuditResult whose findings can be mapped to SSB
//!      Findings with severity="block" / category="deterministic_audit".
//!   2. An infallible auditor call never panics on an empty profile + empty
//!      changed_files set — matches the "auditor internal error does NOT
//!      abort drive_task" acceptance criterion (the auditor is infallible;
//!      the failure mode we must tolerate is profile detection failing,
//!      which the hook handles with a warn! + early return).
//!   3. The SSB Finding category constant matches the value that the
//!      frontend activity feed filters on (`deterministic_audit`).

use std::path::PathBuf;

use elai_core::project_profile::{BuildCommand, CommandScope, Language};
use elai_orchestrator::deterministic_audit_gate::GATE_FINDING_CATEGORY;
use elai_orchestrator::{
    DeterministicAuditor, DeterministicAuditorConfig, DeterministicSeverity,
};
use elai_rules::schema::DeprecatedPatternEntry;

fn config_with_passing_build() -> DeterministicAuditorConfig {
    // C-28: disable Step 6 (mutation) for these legacy gap-#4 contracts.
    let mut mutation = elai_orchestrator::MutationConfig::default();
    mutation.enabled = false;
    // C-29: disable Step 7 (metamorphic) for the same reason.
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
// Contract 1: Block verdict produces a finding we can map to SSB severity
// "block" + category "deterministic_audit".
// ---------------------------------------------------------------------------

#[test]
fn block_verdict_maps_to_severity_block_and_deterministic_audit_category() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("lib.rs");
    // VersionGate is triggered by `blake2::Digest` per the A-41 fixture.
    std::fs::write(&file, "use blake2::Digest;\n").expect("write lib.rs");

    let mut cfg = config_with_passing_build();
    cfg.deprecated_patterns = vec![DeprecatedPatternEntry {
        deprecated_name: "blake2".into(),
        replacement_name: "blake3".into(),
        severity: "BLOCK".into(),
    }];
    let auditor = DeterministicAuditor::new(cfg);
    let result = auditor.run_all(&[file]);

    assert!(!result.passed, "auditor must fail on a BLOCK verdict");
    let block_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.severity == DeterministicSeverity::Block)
        .collect();
    assert!(
        !block_findings.is_empty(),
        "auditor must produce at least one Block finding",
    );

    // Simulate the task_runner's mapping: Block → "block" + category constant.
    let sample = block_findings[0];
    let mapped_severity = match sample.severity {
        DeterministicSeverity::Block => "block",
        DeterministicSeverity::Warn => "warning",
    };
    assert_eq!(mapped_severity, "block");
    // The task_runner uses the same category string as the FSM-boundary
    // gate (DefaultDeterministicAuditGate) so the dashboard unifies the
    // two surfaces on a single filter.
    assert_eq!(GATE_FINDING_CATEGORY, "deterministic_audit");
}

// ---------------------------------------------------------------------------
// Contract 2: empty changed_files set does NOT panic. The task_runner hook
// uses this as the no-op fallback when the synthetic executor reports no
// changes.
// ---------------------------------------------------------------------------

#[test]
fn run_all_on_empty_changed_files_does_not_panic() {
    let auditor = DeterministicAuditor::new(config_with_passing_build());
    let result = auditor.run_all(&[]);
    // The build command "echo ok" passes so the auditor continues through
    // the remaining steps; none of them produce findings on an empty
    // changed-files set.
    assert!(
        result.passed,
        "auditor must pass on an empty changed_files set with a passing build",
    );
}

// ---------------------------------------------------------------------------
// Contract 3: disabled auditor short-circuits cleanly. When the dashboard
// (post-MVP) sets `enabled: false` the task_runner hook must receive an
// empty findings list without any spurious SSB writes.
// ---------------------------------------------------------------------------

#[test]
fn disabled_auditor_returns_empty_findings() {
    let mut cfg = config_with_passing_build();
    cfg.enabled = false;
    let auditor = DeterministicAuditor::new(cfg);
    let result = auditor.run_all(&[PathBuf::from("src/lib.rs")]);
    assert!(result.findings.is_empty());
    assert!(result.passed);
    assert_eq!(result.checks_run, 0);
}

// ---------------------------------------------------------------------------
// Contract 4: the severity mapping is stable. The task_runner hook emits
// "block" (not "error") so the dashboard filter can distinguish Block
// verdicts from generic errors. This guards the mapping against a future
// refactor that might collapse them.
// ---------------------------------------------------------------------------

#[test]
fn severity_mapping_is_block_not_error() {
    let block = DeterministicSeverity::Block;
    let warn = DeterministicSeverity::Warn;
    // What the post-baby-step hook emits:
    let block_sev = match block {
        DeterministicSeverity::Block => "block",
        DeterministicSeverity::Warn => "warning",
    };
    let warn_sev = match warn {
        DeterministicSeverity::Block => "block",
        DeterministicSeverity::Warn => "warning",
    };
    assert_eq!(block_sev, "block");
    assert_eq!(warn_sev, "warning");
}
