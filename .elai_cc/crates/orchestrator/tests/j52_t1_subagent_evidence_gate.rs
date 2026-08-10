//! J-52 T1 contract tests. Part B is the falsification lever for WIRING:
//! deleting the gate call at `crates/tools/src/lib.rs` `fn run_agent_job` turns
//! Part B RED while Part A stays GREEN. The off-agent
//! `.claude/tools/production_caller_check.py` is the second lever. These tests
//! do not claim coverage beyond the cases below.

use elai_orchestrator::subagent_evidence_gate::{
    evaluate_subagent_return, EvidenceVerdict,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn blocked_reason(final_text: &str, evidence_root: &std::path::Path) -> String {
    match evaluate_subagent_return(final_text, evidence_root) {
        EvidenceVerdict::Blocked { reason } => reason,
        other => panic!("expected Blocked, got {other:?}"),
    }
}

#[test]
fn receipt_present_is_accepted() {
    let root = TempDir::new().unwrap();
    let receipt = root.path().join("r.md");
    fs::write(&receipt, b"evidence").unwrap();

    let result = evaluate_subagent_return(
        &format!("finished\nEVIDENCE_RECORDED: {}", receipt.display()),
        root.path(),
    );
    assert!(
        matches!(result, EvidenceVerdict::Present { .. }),
        "J-52 T1: production call site removed would not affect this Part A receipt acceptance case"
    );
}

#[test]
fn missing_receipt_line_is_blocked() {
    let root = TempDir::new().unwrap();
    let reason = blocked_reason("Done, everything passes.", root.path());
    assert!(
        reason.contains("trust"),
        "J-52 T1: production call site removed; missing receipt must retain trust-gate classification"
    );
}

#[test]
fn empty_receipt_file_is_blocked() {
    let root = TempDir::new().unwrap();
    let receipt = root.path().join("empty.md");
    fs::write(&receipt, []).unwrap();
    let reason = blocked_reason(
        &format!("EVIDENCE_RECORDED: {}", receipt.display()),
        root.path(),
    );
    assert!(
        reason.contains("trust"),
        "J-52 T1: production call site removed; empty receipt must retain trust-gate classification"
    );
}

#[cfg(unix)]
#[test]
fn symlink_receipt_is_blocked() {
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let target = root.path().join("target.md");
    let receipt = root.path().join("link.md");
    fs::write(&target, b"evidence").unwrap();
    symlink(&target, &receipt).unwrap();
    let reason = blocked_reason(
        &format!("EVIDENCE_RECORDED: {}", receipt.display()),
        root.path(),
    );
    assert!(
        reason.contains("trust"),
        "J-52 T1: production call site removed; symlink receipt must retain trust-gate classification"
    );
}

#[test]
fn parent_traversal_receipt_is_blocked() {
    let root = TempDir::new().unwrap();
    let reason = blocked_reason("EVIDENCE_RECORDED: ../outside.md", root.path());
    assert!(
        reason.contains("trust"),
        "J-52 T1: production call site removed; parent traversal must retain trust-gate classification"
    );
}

#[test]
fn absolute_receipt_outside_root_is_blocked() {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let receipt = outside.path().join("outside.md");
    fs::write(&receipt, b"evidence").unwrap();
    let reason = blocked_reason(
        &format!("EVIDENCE_RECORDED: {}", receipt.display()),
        root.path(),
    );
    assert!(
        reason.contains("trust"),
        "J-52 T1: production call site removed; outside absolute receipt must retain trust-gate classification"
    );
}

#[test]
fn receipt_line_must_be_last_non_empty_line() {
    let root = TempDir::new().unwrap();
    let receipt = root.path().join("r.md");
    fs::write(&receipt, b"evidence").unwrap();
    let result = evaluate_subagent_return(
        &format!("finished\nEVIDENCE_RECORDED: {}\n\n", receipt.display()),
        root.path(),
    );
    assert!(
        matches!(result, EvidenceVerdict::Present { .. }),
        "J-52 T1: production call site removed would not affect trailing-blank receipt acceptance"
    );
}

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/orchestrator has a parent")
        .to_path_buf()
}

#[test]
fn run_agent_job_calls_the_evidence_gate() {
    let source_path = crates_dir().join("tools/src/lib.rs");
    let source = fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", source_path.display()));
    let start = source
        .find("fn run_agent_job(")
        .expect("J-52 T1: production call site removed; run_agent_job must remain discoverable");
    let body = source[start..]
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with("fn "))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        body.contains("evaluate_subagent_return"),
        "J-52 T1 wiring regression: production call site removed from fn run_agent_job"
    );
    assert!(
        body.contains("EvidenceVerdict::Blocked"),
        "J-52 T1 wiring regression: production call site removed from fn run_agent_job"
    );
    assert!(
        body.contains("\"blocked\""),
        "J-52 T1 wiring regression: production call site removed from fn run_agent_job"
    );
    assert!(
        body.contains("EvidenceVerdict::Present"),
        "J-52 T1 wiring regression: production call site removed; completed must be a Present match arm"
    );
}
