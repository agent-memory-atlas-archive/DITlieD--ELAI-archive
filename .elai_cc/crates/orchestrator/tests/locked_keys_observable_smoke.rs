//! D-9 — observable_effect smoke test for LockedKeysAuditor.
//!
//! Implements the WIRING_CONTRACT.observable_effect declared in
//! docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md:
//!
//!   trigger: dev stages an edit overlapping a locked path
//!   expect_ssb: 1 LockedKeysAuditRun row + 1 LockedKeysViolationDetected row
//!   expect_no_ssb_on_clean: 1 LockedKeysAuditRun row + 0 violation rows
//!
//! Uses an in-memory capture sink — no SQLite, no IPC, no git shell-
//! out. The CLI driver's git path is exercised separately under
//! manual / CI smoke; the orchestrator-side semantics are what this
//! test pins down.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_orchestrator::locked_keys_auditor::{
    LockedKeysAuditor, LockedKeysRegistry, LockedKeysSink, LOCKED_KEY_FULL_FILE,
};
use elai_orchestrator::ssb::SsbPayload;
use elai_orchestrator::LockedKey;
use tempfile::TempDir;

fn capture() -> (LockedKeysSink, Arc<Mutex<Vec<SsbPayload>>>) {
    let buf = Arc::new(Mutex::new(Vec::<SsbPayload>::new()));
    (LockedKeysSink::Capture(buf.clone()), buf)
}

fn registry_with_one_locked_path(tmp: &TempDir) -> LockedKeysRegistry {
    let mut entry = LockedKey {
        path: PathBuf::from("crates/orchestrator/src/tripwire.rs"),
        line_range: LOCKED_KEY_FULL_FILE,
        reason: "tripwire detection code per arxiv:1908.04734".into(),
        owner_session: "smoke-test".into(),
        freeze_ts_ms: 0,
        signature_blake3: [0u8; 32],
    };
    entry.signature_blake3 = entry.compute_signature();
    LockedKeysRegistry {
        entries: vec![entry],
        registry_path: tmp.path().join("locked_keys_registry.toml"),
        loaded_ts_ms: 1,
    }
}

#[test]
fn observable_smoke_violation_emits_audit_run_plus_violation_rows() {
    let tmp = TempDir::new().expect("tempdir");
    let registry = registry_with_one_locked_path(&tmp);
    let (sink, buf) = capture();
    let auditor = LockedKeysAuditor::new(registry, Some(sink));
    let staged = vec![PathBuf::from("crates/orchestrator/src/tripwire.rs")];
    let result = auditor.audit_diff(&staged);
    assert_eq!(result.violations.len(), 1, "1 violation expected");

    let captured = buf.lock().expect("lock");
    assert_eq!(
        captured.len(),
        2,
        "1 audit_run row + 1 violation row = 2 total"
    );
    let n_runs = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysAuditRun(_)))
        .count();
    let n_viol = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysViolationDetected(_)))
        .count();
    assert_eq!(n_runs, 1);
    assert_eq!(n_viol, 1);
}

#[test]
fn observable_smoke_clean_emits_audit_run_only() {
    let tmp = TempDir::new().expect("tempdir");
    let registry = registry_with_one_locked_path(&tmp);
    let (sink, buf) = capture();
    let auditor = LockedKeysAuditor::new(registry, Some(sink));
    let staged = vec![PathBuf::from("crates/dashboard-v2/src/App.svelte")];
    let result = auditor.audit_diff(&staged);
    assert!(result.violations.is_empty());

    let captured = buf.lock().expect("lock");
    assert_eq!(captured.len(), 1, "audit_run row only");
    assert!(matches!(captured[0], SsbPayload::LockedKeysAuditRun(_)));
}

#[test]
fn observable_smoke_empty_registry_emits_audit_run_only() {
    let tmp = TempDir::new().expect("tempdir");
    let registry = LockedKeysRegistry::empty(tmp.path().join("empty.toml"));
    let (sink, buf) = capture();
    let auditor = LockedKeysAuditor::new(registry, Some(sink));
    let staged = vec![PathBuf::from("crates/orchestrator/src/tripwire.rs")];
    let result = auditor.audit_diff(&staged);
    assert!(
        result.violations.is_empty(),
        "empty registry must permit any edit"
    );
    let captured = buf.lock().expect("lock");
    assert_eq!(captured.len(), 1);
    assert!(matches!(captured[0], SsbPayload::LockedKeysAuditRun(_)));
}
