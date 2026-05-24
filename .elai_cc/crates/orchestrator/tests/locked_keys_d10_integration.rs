//! D-10 closure GAP-6 — L3 lock enforcement on the mode-routing
//! predicate surface.
//!
//! The D-9 [`LockedKeysAuditor`] gates edits to L3-locked surfaces.
//! D-10 is the FIRST non-empty entry in the bootstrap registry
//! (see workflow/reasoning/D-10-decisions.md DECISION-7 +
//! .elai_cc/scripts/locked_keys_registry.toml). This test pins down
//! the regression case: synthetic edits to
//!   - crates/orchestrator/src/grounding_mode.rs (the predicate file)
//!   - crates/orchestrator/src/claim.rs (the ClaimType enum file)
//! MUST fire [`SsbPayload::LockedKeysViolationDetected`].
//!
//! Grounding:
//!   * docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md §7
//!     (L3 locking task — "agent cannot edit routing logic").
//!   * workflow/papers/everitt-reward-tampering-1908.04734-interface.md
//!     (current-RF principle — agent must not edit grading fn).
//!   * workflow/papers/cohen-boxed-myopic-interface.md (Cohen BoMAI —
//!     immutable reward record).
//!   * .elai_cc/crates/orchestrator/tests/locked_keys_integration.rs
//!     (D-9 sibling integration test; same pattern + helpers).

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_orchestrator::locked_keys_auditor::{LockedKeysAuditor, LockedKeysSink};
use elai_orchestrator::ssb::SsbPayload;
use tempfile::TempDir;

/// Construct a capture sink + a handle to the captured payload buffer.
fn capture_sink() -> (LockedKeysSink, Arc<Mutex<Vec<SsbPayload>>>) {
    let buf = Arc::new(Mutex::new(Vec::<SsbPayload>::new()));
    (LockedKeysSink::Capture(buf.clone()), buf)
}

/// Write a TOML registry that mirrors the live D-10 entries (the two
/// mode-routing-predicate surface paths). Mirrors the
/// `write_registry` helper in locked_keys_integration.rs but
/// hardcodes the D-10 paths so the regression is self-contained
/// (no read from .elai_cc/scripts/locked_keys_registry.toml, which
/// would couple the unit test to repo layout).
fn write_d10_registry(tmp: &TempDir) -> PathBuf {
    let path = tmp.path().join("locked_keys_d10.toml");
    let body = r#"[[locked_keys]]
path = "crates/orchestrator/src/grounding_mode.rs"
line_range = [0, 4294967295]
reason = "select_grounding_mode + GroundingMode closed enum per D-10 §3.2; agent cannot alter routing logic (arxiv:1908.04734)"
owner_session = "d-10-integration-test"
freeze_ts_ms = 0
signature_blake3 = ""

[[locked_keys]]
path = "crates/orchestrator/src/claim.rs"
line_range = [0, 4294967295]
reason = "ClaimType + EvidenceKind closed enums per D-10 companion §4; agent cannot add new claim types"
owner_session = "d-10-integration-test"
freeze_ts_ms = 0
signature_blake3 = ""
"#;
    fs::write(&path, body).expect("write d-10 registry");
    path
}

#[test]
fn tampering_with_select_grounding_mode_fires_locked_keys_violation() {
    let tmp = TempDir::new().expect("tempdir");
    let registry_path = write_d10_registry(&tmp);

    let (sink, buf) = capture_sink();
    let auditor =
        LockedKeysAuditor::load(&registry_path, Some(sink)).expect("D-10 registry parses");

    // Synthetic staged edit on the predicate file. The auditor must
    // flag it because the registry locks the file at whole-file
    // line_range = [0, u32::MAX].
    let staged = vec![PathBuf::from(
        "crates/orchestrator/src/grounding_mode.rs",
    )];
    let result = auditor.audit_diff(&staged);

    assert_eq!(
        result.violations.len(),
        1,
        "edit to grounding_mode.rs must fire exactly 1 LockedKeysViolationDetected"
    );
    assert!(result.allowed_edits.is_empty());

    let captured = buf.lock().expect("capture lock");
    let n_runs = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysAuditRun(_)))
        .count();
    let n_violations = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysViolationDetected(_)))
        .count();
    assert_eq!(n_runs, 1, "exactly 1 audit-run row per audit_diff call");
    assert_eq!(
        n_violations, 1,
        "exactly 1 violation row for 1 overlap (grounding_mode.rs)"
    );

    // The violation's `locked_key_path` must reference the predicate
    // file by its posix-form path (the auditor normalises Windows
    // backslashes).
    let violation_paths: Vec<String> = captured
        .iter()
        .filter_map(|p| match p {
            SsbPayload::LockedKeysViolationDetected(v) => Some(v.locked_key_path.clone()),
            _ => None,
        })
        .collect();
    assert!(
        violation_paths
            .iter()
            .any(|p| p == "crates/orchestrator/src/grounding_mode.rs"),
        "violation must cite grounding_mode.rs (got {violation_paths:?})"
    );
}

#[test]
fn tampering_with_claim_type_fires_locked_keys_violation() {
    // Companion case: edits to crates/orchestrator/src/claim.rs (the
    // ClaimType enum file) must also fire a violation. Ensures the
    // ClaimType lock is live, not just the predicate lock.
    let tmp = TempDir::new().expect("tempdir");
    let registry_path = write_d10_registry(&tmp);

    let (sink, buf) = capture_sink();
    let auditor =
        LockedKeysAuditor::load(&registry_path, Some(sink)).expect("D-10 registry parses");

    let staged = vec![PathBuf::from("crates/orchestrator/src/claim.rs")];
    let result = auditor.audit_diff(&staged);

    assert_eq!(
        result.violations.len(),
        1,
        "edit to claim.rs must fire exactly 1 LockedKeysViolationDetected"
    );
    let captured = buf.lock().expect("capture lock");
    let n_runs = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysAuditRun(_)))
        .count();
    let n_violations = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysViolationDetected(_)))
        .count();
    assert_eq!(n_runs, 1);
    assert_eq!(n_violations, 1);
}

#[test]
fn unrelated_edit_does_not_fire_d10_violation() {
    // A staged edit on a path NOT in the D-10 locked set must pass
    // the audit (one audit-run row, zero violation rows). Confirms
    // the locks are scoped to the predicate + ClaimType surface only.
    let tmp = TempDir::new().expect("tempdir");
    let registry_path = write_d10_registry(&tmp);

    let (sink, buf) = capture_sink();
    let auditor =
        LockedKeysAuditor::load(&registry_path, Some(sink)).expect("D-10 registry parses");

    // A path that is not under any D-10 locked entry.
    let staged = vec![PathBuf::from(
        "crates/orchestrator/src/agent_pool.rs",
    )];
    let result = auditor.audit_diff(&staged);

    assert!(
        result.violations.is_empty(),
        "agent_pool.rs is NOT locked (it hosts the route_claim producer; the predicate fn itself is locked)"
    );
    assert_eq!(result.allowed_edits.len(), 1);

    let captured = buf.lock().expect("capture lock");
    assert_eq!(captured.len(), 1, "only the audit-run row; no violation");
    assert!(matches!(captured[0], SsbPayload::LockedKeysAuditRun(_)));
}

#[test]
fn windows_backslash_staged_path_still_matches_d10_predicate_lock() {
    // Cross-platform regression: when git emits Windows backslash
    // paths the auditor's normalisation must still hit the locked
    // entry. Mirrors the D-9 integration test's same check.
    let tmp = TempDir::new().expect("tempdir");
    let registry_path = write_d10_registry(&tmp);

    let auditor = LockedKeysAuditor::load(&registry_path, None).expect("D-10 registry parses");

    let staged = vec![PathBuf::from(
        "crates\\orchestrator\\src\\grounding_mode.rs",
    )];
    let result = auditor.audit_diff(&staged);
    assert_eq!(
        result.violations.len(),
        1,
        "Windows-form path must normalise to posix and still match the D-10 lock"
    );
}
