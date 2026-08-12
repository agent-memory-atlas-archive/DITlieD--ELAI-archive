//! P-19 Slice 1 — focused contract tests for `ConfigLifecycle::guard_write`.
//!
//! These tests exercise the guard through the public orchestrator API
//! (no internal helpers) and prove the shrink-floor is load-bearing by
//! comparing the guarded path with the raw atomic helper.

use elai_orchestrator::config_lifecycle::{ConfigLifecycle, ConfigWriteOutcome};
use elai_orchestrator::util::atomic_io::atomic_write_sync;

fn tmp_dir() -> tempfile::TempDir {
    tempfile::TempDir::new().expect("tempdir")
}

/// A >50% shrink rejects, preserves the original active file, and
/// preserves the rejected bytes as a `.rejected.<timestamp>` sibling.
#[test]
fn shrink_below_half_rejects_and_preserves_original_and_rejected() {
    let dir = tmp_dir();
    let path = dir.path().join("elai_cc.toml");
    let original = b"[agents.executor.runtime]\nbackend = \"sidecar\"\n# padding padding padding\n";
    std::fs::write(&path, original).expect("write original");
    let proposed = b"X"; // 1 byte, existing 66 -> 1*2=2 < 66

    let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("guard");
    match outcome {
        ConfigWriteOutcome::Rejected {
            existing_len,
            proposed_len,
        } => {
            assert_eq!(existing_len, original.len());
            assert_eq!(proposed_len, 1);
        }
        ConfigWriteOutcome::Accepted => panic!("should have rejected the >50% shrink"),
    }

    // Active file unchanged.
    let active = std::fs::read(&path).expect("read active");
    assert_eq!(active, original);

    // Rejected bytes preserved in exactly one .rejected.<ts> sibling.
    let rejected: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .contains(".rejected.")
        })
        .collect();
    assert_eq!(rejected.len(), 1, "exactly one rejected sibling");
    let rejected_content = std::fs::read(rejected[0].path()).expect("read rejected");
    assert_eq!(rejected_content, proposed);
}

/// A normal write succeeds and preserves the prior content in the LKG
/// (last-known-good) sibling.
#[test]
fn normal_write_preserves_prior_content_as_lkg() {
    let dir = tmp_dir();
    let path = dir.path().join("elai_cc.toml");
    let original = b"[agents.executor.runtime]\nbackend = \"sidecar\"\n";
    std::fs::write(&path, original).expect("write original");
    let proposed = b"[agents.executor.runtime]\nbackend = \"anthropic_api\"\n";

    let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("guard");
    assert!(matches!(outcome, ConfigWriteOutcome::Accepted));

    // New content is active.
    let active = std::fs::read(&path).expect("read active");
    assert_eq!(active, proposed);

    // Prior content preserved as .lkg.
    let lkg_path = dir.path().join("elai_cc.toml.lkg");
    let lkg = std::fs::read(&lkg_path).expect("read lkg");
    assert_eq!(lkg, original);
}

/// Negative test: proves the guard is load-bearing by showing the raw
/// atomic helper would overwrite the >50% shrink that the guard rejects.
#[test]
fn raw_atomic_helper_overwrites_shrink_that_guard_rejects() {
    let dir_guard = tmp_dir();
    let dir_raw = tmp_dir();
    let path_guard = dir_guard.path().join("elai_cc.toml");
    let path_raw = dir_raw.path().join("elai_cc.toml");
    let original = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"; // 32 bytes
    std::fs::write(&path_guard, original).expect("write original guard");
    std::fs::write(&path_raw, original).expect("write original raw");
    let proposed = b"X"; // 1 byte, 1*2=2 < 32

    // Guard rejects.
    let outcome = ConfigLifecycle::guard_write(&path_guard, proposed).expect("guard");
    assert!(matches!(outcome, ConfigWriteOutcome::Rejected { .. }));
    let guard_active = std::fs::read(&path_guard).expect("read guard active");
    assert_eq!(guard_active, original, "guard preserved the original");

    // Raw helper overwrites — proving the guard is load-bearing.
    atomic_write_sync(&path_raw, proposed).expect("raw write");
    let raw_active = std::fs::read(&path_raw).expect("read raw active");
    assert_eq!(raw_active, proposed, "raw helper overwrote without guard");
}

/// Missing original is allowed (no shrink floor, no LKG).
#[test]
fn missing_original_is_allowed() {
    let dir = tmp_dir();
    let path = dir.path().join("elai_cc.toml");
    let proposed = b"[test]\nkey = 1\n";
    let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("guard");
    assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
    let active = std::fs::read(&path).expect("read active");
    assert_eq!(active, proposed);
    // No LKG because no prior content.
    assert!(!dir.path().join("elai_cc.toml.lkg").exists());
}

/// Zero-length original is allowed (no shrink floor, no LKG).
#[test]
fn zero_length_original_is_allowed() {
    let dir = tmp_dir();
    let path = dir.path().join("elai_cc.toml");
    std::fs::write(&path, b"").expect("write empty");
    let proposed = b"[test]\nkey = 1\n";
    let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("guard");
    assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
    let active = std::fs::read(&path).expect("read active");
    assert_eq!(active, proposed);
    // No LKG because prior content was empty.
    assert!(!dir.path().join("elai_cc.toml.lkg").exists());
}

/// Exact-half boundary (proposed == existing/2) is accepted, not rejected.
#[test]
fn exact_half_boundary_is_accepted() {
    let dir = tmp_dir();
    let path = dir.path().join("elai_cc.toml");
    let original = b"ABCD"; // 4 bytes
    std::fs::write(&path, original).expect("write original");
    let proposed = b"XY"; // 2 bytes, 2*2=4 == 4, NOT strictly below
    let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("guard");
    assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
    let active = std::fs::read(&path).expect("read active");
    assert_eq!(active, proposed);
    let lkg = std::fs::read(dir.path().join("elai_cc.toml.lkg")).expect("read lkg");
    assert_eq!(lkg, original);
}

/// A write that is above half (e.g. 75% of original) is accepted.
#[test]
fn write_above_half_is_accepted() {
    let dir = tmp_dir();
    let path = dir.path().join("elai_cc.toml");
    let original = b"AAAAAAAAAAAAAAAA"; // 16 bytes
    std::fs::write(&path, original).expect("write original");
    let proposed = b"BBBBBBBBBBBB"; // 12 bytes, 12*2=24 > 16
    let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("guard");
    assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
    let active = std::fs::read(&path).expect("read active");
    assert_eq!(active, proposed);
}
