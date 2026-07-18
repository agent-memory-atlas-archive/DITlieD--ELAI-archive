//! E-52 T2 U1 contract tests for the RustAnalyzerParser pure-function
//! surface. Pins two contracts the plan's U1 VERIFY specifies:
//!   (a) parse_rust_analyzer_stdout_extracts_error -- feeds the GROUNDED
//!       fixture stdout (a single `at crate ... Error RustcHardError("E0308")
//!       ...` line) to the parser, asserts error_count >= 1 +
//!       diagnostics[0].code == Some("E0308").
//!   (b) parse_rust_analyzer_stdout_empty_returns_clean -- feeds empty
//!       stdout, asserts is_clean() + run_status == Completed.
//!
//! The parser is a PURE FUNCTION of (stdout, stderr, req): the test does NOT
//! spawn rust-analyzer (CI-stable). The subprocess invocation lives in
//! `elai_orchestrator::diagnostic_probe::dispatch_diagnostic_probe` and is
//! exercised by the integration smoke, not this contract test.
//!
//! Plan ref: conductor/opus_walk/evidence/E-52/T2-execution-plan.md U1 VERIFY.
//!
//! GROUNDED fixture: the stdout literal below mirrors the observed
//! rust-analyzer 1.97.1 output on a `let x: String = 5;` fixture at
//! /tmp/ra-fixture/src/lib.rs line 6 (recorded in the plan's GROUNDING
//! block). The path prefix is normalized against req.file_path.parent().

use elai_knowledge::diagnostic::{DiagnosticProbeRunStatus, DiagnosticSeverity, ProbeKind};
use elai_orchestrator::diagnostic_probe::diagnostic_probe_parsers::test_support::{
    fixture_req, parse_rust_analyzer_stdout,
};

const GROUNDED_STDOUT: &str = "1/2 50% processing src/lib.rs\n\
at crate ra_fixture, file /tmp/ra-fixture/src/lib.rs: Error RustcHardError(\"E0308\") from LineCol { line: 6, col: 12 } to LineCol { line: 6, col: 13 }: mismatched types: expected `String`, found integer\n\
2/2 100% processing src/lib.rs\n\
diagnostic scan complete\n";

#[test]
fn parse_rust_analyzer_stdout_extracts_error() {
    let req = fixture_req("/tmp/ra-fixture/src/lib.rs");
    let out = parse_rust_analyzer_stdout(GROUNDED_STDOUT, "", &req);
    assert_eq!(
        out.run_status,
        DiagnosticProbeRunStatus::Completed,
        "CONTRACT: parser MUST return Completed (rust-analyzer exits 0 even with errors)"
    );
    assert!(
        out.error_count >= 1,
        "CONTRACT: grounded fixture has 1 Error line; error_count >= 1, got {}",
        out.error_count
    );
    assert!(
        !out.is_clean(),
        "CONTRACT: is_clean() MUST be false when error_count >= 1"
    );
    assert_eq!(
        out.diagnostics.len(),
        1,
        "CONTRACT: exactly one diagnostic line in the fixture (progress + complete lines skipped)"
    );
    let d = &out.diagnostics[0];
    assert_eq!(
        d.severity,
        DiagnosticSeverity::Error,
        "CONTRACT: severity MUST map to Error"
    );
    assert_eq!(
        d.code.as_deref(),
        Some("E0308"),
        "CONTRACT: RustcHardError(\"E0308\") wrapper MUST be stripped to Some(\"E0308\")"
    );
    assert_eq!(d.line, 6, "CONTRACT: line MUST be 6 (fixture line)");
    assert_eq!(d.column, 12, "CONTRACT: column MUST be 12 (fixture col)");
    assert_eq!(
        d.path, "src/lib.rs",
        "CONTRACT: path MUST be normalized relative to req.file_path.parent()"
    );
    assert_eq!(
        out.kind,
        ProbeKind::RustAnalyzer,
        "CONTRACT: outcome.kind MUST be RustAnalyzer"
    );
}

#[test]
fn parse_rust_analyzer_stdout_empty_returns_clean() {
    let req = fixture_req("/tmp/ra-fixture/src/lib.rs");
    let out = parse_rust_analyzer_stdout("", "", &req);
    assert_eq!(
        out.run_status,
        DiagnosticProbeRunStatus::Completed,
        "CONTRACT: empty stdout MUST return Completed"
    );
    assert!(
        out.is_clean(),
        "CONTRACT: empty stdout MUST be clean (zero errors)"
    );
    assert_eq!(
        out.diagnostics.len(),
        0,
        "CONTRACT: empty stdout MUST produce zero diagnostics"
    );
    assert_eq!(out.error_count, 0);
    assert_eq!(out.warning_count, 0);
    assert_eq!(out.kind, ProbeKind::RustAnalyzer);
}
