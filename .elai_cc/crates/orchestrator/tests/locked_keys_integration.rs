//! D-9 — LockedKeysAuditor end-to-end integration test.
//!
//! Pins down the registry-file -> auditor -> SSB-emission path that the
//! pre-commit hook + CI workflow rely on. Complements the in-memory
//! observable_smoke test by exercising:
//!
//!   1. TOML registry parse from a temp file with a populated
//!      `[[locked_keys]]` entry.
//!   2. Audit a write that overlaps the locked path -> exactly one
//!      LockedKeysAuditRun row + one LockedKeysViolationDetected row.
//!   3. Audit a write to a non-locked path -> one LockedKeysAuditRun row,
//!      zero violation rows.
//!   4. Mutate the registry on disk between two loads and confirm the
//!      `registry_blake3` hash on the LockedKeysAuditRun row differs.
//!      That's the chain-hash drift signal dashboard consumers rely on
//!      to detect mid-session registry tampering.
//!
//! Grounding:
//!   * docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md
//!   * crates/orchestrator/src/locked_keys_auditor.rs
//!   * crates/orchestrator/tests/locked_keys_observable_smoke.rs
//!     (in-memory analogue; this test adds the file-system round trip).
//!   * arxiv:1908.04734 (Everitt current-RF)

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_orchestrator::locked_keys_auditor::{LockedKeysAuditor, LockedKeysSink};
use elai_orchestrator::ssb::SsbPayload;
use tempfile::TempDir;

/// Write a TOML registry with `entries.len()` populated rows. Each entry
/// is whole-file (`line_range = [0, u32::MAX]`) so the auditor matches
/// on path equality alone, which is the shipped MVP surface.
fn write_registry(tmp: &TempDir, entries: &[(&str, &str)]) -> PathBuf {
    let path = tmp.path().join("locked_keys_registry.toml");
    let mut body = String::new();
    for (locked_path, reason) in entries {
        body.push_str("[[locked_keys]]\n");
        body.push_str(&format!("path = \"{}\"\n", locked_path));
        body.push_str("line_range = [0, 4294967295]\n");
        body.push_str(&format!("reason = \"{}\"\n", reason));
        body.push_str("owner_session = \"integration-test\"\n");
        body.push_str("freeze_ts_ms = 0\n");
        body.push_str("signature_blake3 = \"\"\n\n");
    }
    fs::write(&path, body).expect("write registry");
    path
}

/// Construct a capture sink + a handle to the captured payload buffer.
fn capture_sink() -> (LockedKeysSink, Arc<Mutex<Vec<SsbPayload>>>) {
    let buf = Arc::new(Mutex::new(Vec::<SsbPayload>::new()));
    (LockedKeysSink::Capture(buf.clone()), buf)
}

#[test]
fn integration_violating_write_fires_violation_row() {
    let tmp = TempDir::new().expect("tempdir");
    let registry_path = write_registry(
        &tmp,
        &[
            (
                "crates/orchestrator/src/tripwire.rs",
                "tripwire detection per arxiv:1908.04734",
            ),
            (
                "crates/orchestrator/src/locked_keys_auditor.rs",
                "the auditor itself is L3-locked",
            ),
        ],
    );

    let (sink, buf) = capture_sink();
    let auditor = LockedKeysAuditor::load(&registry_path, Some(sink))
        .expect("registry parses");

    let staged = vec![PathBuf::from("crates/orchestrator/src/tripwire.rs")];
    let result = auditor.audit_diff(&staged);

    assert_eq!(result.violations.len(), 1, "must flag the locked write");
    assert!(result.allowed_edits.is_empty());

    let captured = buf.lock().expect("capture lock");
    let n_runs = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysAuditRun(_)))
        .count();
    let n_viol = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysViolationDetected(_)))
        .count();
    assert_eq!(n_runs, 1, "exactly 1 audit-run row per audit_diff call");
    assert_eq!(n_viol, 1, "exactly 1 violation row for 1 overlap");
}

#[test]
fn integration_unrelated_write_passes_audit_and_emits_only_run_row() {
    let tmp = TempDir::new().expect("tempdir");
    let registry_path = write_registry(
        &tmp,
        &[(
            "crates/orchestrator/src/tripwire.rs",
            "tripwire detection per arxiv:1908.04734",
        )],
    );

    let (sink, buf) = capture_sink();
    let auditor = LockedKeysAuditor::load(&registry_path, Some(sink))
        .expect("registry parses");

    // A path that is not under any locked entry.
    let staged = vec![PathBuf::from("crates/dashboard-v2/src/App.svelte")];
    let result = auditor.audit_diff(&staged);

    assert!(result.violations.is_empty(), "no violation expected");
    assert_eq!(result.allowed_edits.len(), 1);

    let captured = buf.lock().expect("capture lock");
    assert_eq!(
        captured.len(),
        1,
        "audit-run row only; no violation row on a clean diff"
    );
    assert!(matches!(captured[0], SsbPayload::LockedKeysAuditRun(_)));
}

#[test]
fn integration_registry_mutation_changes_run_row_blake3() {
    // Two consecutive audits with a registry mutation between them must
    // produce different `registry_blake3` values on the audit-run rows.
    // That's the drift-detection signal documented in
    // locked_keys_auditor.rs::content_blake3.
    let tmp = TempDir::new().expect("tempdir");
    let registry_path = write_registry(
        &tmp,
        &[(
            "crates/orchestrator/src/tripwire.rs",
            "tripwire detection per arxiv:1908.04734",
        )],
    );

    let staged = vec![PathBuf::from("crates/orchestrator/src/tripwire.rs")];

    // First audit pass.
    let (sink1, buf1) = capture_sink();
    {
        let auditor =
            LockedKeysAuditor::load(&registry_path, Some(sink1)).expect("first load");
        let _ = auditor.audit_diff(&staged);
    }
    let first_hash = {
        let captured = buf1.lock().expect("buf1");
        captured
            .iter()
            .find_map(|p| match p {
                SsbPayload::LockedKeysAuditRun(r) => Some(r.registry_blake3.clone()),
                _ => None,
            })
            .expect("first audit-run row")
    };

    // Mutate the registry: add a second locked path. This re-writes the
    // file with two entries instead of one.
    let _new_path = write_registry(
        &tmp,
        &[
            (
                "crates/orchestrator/src/tripwire.rs",
                "tripwire detection per arxiv:1908.04734",
            ),
            (
                "crates/orchestrator/src/locked_keys_auditor.rs",
                "the auditor itself is L3-locked",
            ),
        ],
    );

    // Second audit pass — reloads from disk so the new content is hashed.
    let (sink2, buf2) = capture_sink();
    {
        let auditor =
            LockedKeysAuditor::load(&registry_path, Some(sink2)).expect("second load");
        let _ = auditor.audit_diff(&staged);
    }
    let second_hash = {
        let captured = buf2.lock().expect("buf2");
        captured
            .iter()
            .find_map(|p| match p {
                SsbPayload::LockedKeysAuditRun(r) => Some(r.registry_blake3.clone()),
                _ => None,
            })
            .expect("second audit-run row")
    };

    assert_ne!(
        first_hash, second_hash,
        "registry mutation must change registry_blake3"
    );
    assert_eq!(first_hash.len(), 64, "blake3 hex32 = 64 chars");
    assert_eq!(second_hash.len(), 64);
}

#[test]
fn integration_unreadable_registry_fails_closed() {
    // Pointing at a missing registry file must produce a structured
    // RegistryUnreadable error rather than silently constructing an
    // empty auditor — that's the WIRING_CONTRACT.stub_ban contract.
    let tmp = TempDir::new().expect("tempdir");
    let missing = tmp.path().join("no-such-registry.toml");
    let outcome = LockedKeysAuditor::load(&missing, None);
    assert!(
        outcome.is_err(),
        "missing registry must fail-closed, not return an empty auditor"
    );
}
