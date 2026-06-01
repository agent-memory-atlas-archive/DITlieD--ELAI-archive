//! E-25 Tranche C (C.4) -- edit_checkpoints regression contracts.
//!
//! Four guards covering the snapshot + rewind path: snapshot creates
//! file at expected location; rewind restores file content; rewind
//! emits SSB payload with `rewind_event=true` marker (9-arm Kind
//! enum LOCKED -- no 10th arm); unknown seq returns error not panic.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use elai_orchestrator::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
};
use elai_orchestrator::architecture::edit_checkpoints::{
    build_snapshot_payload, rewind_to, snapshot_pre_edit, EditCheckpointError,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fresh_test_root() -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut root = std::env::temp_dir();
    root.push(format!(
        "elai-e25-edit-checkpoints-contract-{}-{}-{}",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&root).expect("create test root");
    root
}

#[test]
fn snapshot_creates_file_at_expected_path() {
    let root = fresh_test_root();
    let src = root.join("alpha.txt");
    let cp = snapshot_pre_edit(&root, "s-c4", &src, b"original-content")
        .expect("snapshot");
    assert_eq!(cp.seq, 0);
    assert!(cp.snapshot_path.exists(), "snapshot file should exist on disk");
    let body = fs::read(&cp.snapshot_path).expect("read snapshot");
    assert_eq!(body, b"original-content");
    // Path shape: root/s-c4/000000/<blake3 prefix>/<encoded src>
    let s = cp.snapshot_path.to_string_lossy();
    assert!(s.contains("s-c4"), "{}", s);
    assert!(s.contains("000000"), "{}", s);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rewind_restores_file_content() {
    let root = fresh_test_root();
    let target = root.join("beta").join("file.txt");
    let original = b"v0";
    let _ = snapshot_pre_edit(&root, "s-c4", &target, original)
        .expect("snapshot");
    // Simulate post-edit divergence on disk.
    if let Some(p) = target.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&target, b"v1-mutated").unwrap();
    let outcome = rewind_to(&root, "s-c4", 0).expect("rewind");
    assert_eq!(outcome.target_seq, 0);
    let restored = fs::read(&target).unwrap();
    assert_eq!(restored.as_slice(), original);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rewind_emits_ssb_row_with_rewind_event_marker() {
    let root = fresh_test_root();
    let src = root.join("gamma.txt");
    let cp = snapshot_pre_edit(&root, "s-c4", &src, b"x").expect("snapshot");
    let snap = build_snapshot_payload(&cp, false);
    assert_eq!(
        snap.kind,
        ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted
    );
    assert_eq!(snap.primitive, ClaudeCodeArchPrimitive::EditCheckpoints);
    assert_eq!(
        snap.details_json
            .get("rewind_event")
            .and_then(|v| v.as_bool()),
        Some(false)
    );
    let rew = build_snapshot_payload(&cp, true);
    assert_eq!(
        rew.kind,
        ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted
    );
    assert_eq!(
        rew.details_json
            .get("rewind_event")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unknown_seq_returns_error_not_panic() {
    let root = fresh_test_root();
    // No snapshots yet -> NoSnapshots, not a panic.
    let err = rewind_to(&root, "s-c4", 0).unwrap_err();
    assert!(matches!(err, EditCheckpointError::NoSnapshots { .. }));
    // Snapshot then ask for a seq that does not exist -> SeqNotFound.
    let p = root.join("delta.txt");
    let _ = snapshot_pre_edit(&root, "s-c4", &p, b"q").unwrap();
    let err = rewind_to(&root, "s-c4", 42).unwrap_err();
    assert!(matches!(err, EditCheckpointError::SeqNotFound { .. }));
    let _ = fs::remove_dir_all(&root);
}
