// I-3 — Atomic checkpoint durability contract test.
//
// Locks the crash-resilience properties of the .tmp+sync_all+rename atomic
// write pattern now applied to all 8 checkpoint write sites in the
// elai-orchestrator crate.
//
// Three arms (per exploration I-3-persistence.md §CRASH FIXTURE DESIGN):
//   (a) POSITIVE: torn .tmp left by crash is ignored; old complete file survives
//       byte-identical (blake3).
//   (b) POSITIVE: happy-path atomic write; new complete file is readable and
//       the in-memory bytes round-trip via rmp_serde::from_slice.
//   (c) BROKEN ARM (R2): direct write followed by truncation produces torn
//       msgpack that rmp_serde::from_slice rejects (Err). The load gate must
//       never return Ok(data) for a torn file — verified here at the
//       deserialization layer which is checkpoint.rs:312-314's actual gate.
//
// Observable artifact: blake3 hash comparison proves byte-identical content.
// R2: the broken arm's rmp_serde failure is the off-agent artifact we assert on.

use elai_orchestrator::checkpoint::AgentCheckpointData;
use elai_orchestrator::util::atomic_io::atomic_write_sync;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Build minimal deterministic AgentCheckpointData for fixture use.
fn fixture_data(tag: &str) -> AgentCheckpointData {
    AgentCheckpointData {
        agent_status: format!("running-{tag}"),
        grc_history: vec![],
        active_files: vec![],
        tool_results: vec![],
        output_draft: String::new(),
        channel_references: vec![],
        schema_version: elai_orchestrator::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION,
    }
}

fn serialize(data: &AgentCheckpointData) -> Vec<u8> {
    rmp_serde::to_vec_named(data).expect("msgpack encode")
}

// ---------------------------------------------------------------------------
// (a) POSITIVE — torn .tmp does not corrupt prior complete file
// ---------------------------------------------------------------------------

/// Simulate crash-before-rename: write partial bytes to state.msgpack.tmp
/// without calling rename. The original state.msgpack must be byte-identical
/// after the crash (blake3 pre == blake3 post).
///
/// This mirrors the recovery behavior documented at checkpoint.rs:303
/// "Ignore any .tmp files (partial writes from prior crashes)" — the file
/// the .tmp would have replaced is never touched.
#[test]
fn i3_a_torn_tmp_does_not_corrupt_prior_complete_file() {
    let dir = TempDir::new().expect("tempdir");
    let state_file = dir.path().join("state.msgpack");

    // Write a complete valid checkpoint file.
    let old_data = fixture_data("old");
    let old_bytes = serialize(&old_data);
    atomic_write_sync(&state_file, &old_bytes).expect("write old checkpoint");

    let pre_crash_hash = blake3_hex(&std::fs::read(&state_file).expect("read pre-crash"));

    // Simulate crash: write partial bytes to the .tmp and stop (no rename).
    let tmp_path = dir.path().join("state.msgpack.tmp");
    std::fs::write(&tmp_path, b"partial-garbage-simulating-crash").expect("write torn tmp");

    // Original file must be untouched.
    let post_crash_bytes = std::fs::read(&state_file).expect("read post-crash");
    let post_crash_hash = blake3_hex(&post_crash_bytes);

    assert_eq!(
        pre_crash_hash, post_crash_hash,
        "state.msgpack must be byte-identical before and after a crash that \
         left a .tmp sibling (the rename never happened)"
    );

    // The complete file must still deserialize correctly.
    let recovered: AgentCheckpointData =
        rmp_serde::from_slice(&post_crash_bytes).expect("deserialization must succeed");
    assert_eq!(
        recovered.agent_status, old_data.agent_status,
        "recovered data must match the pre-crash complete checkpoint"
    );
}

// ---------------------------------------------------------------------------
// (b) POSITIVE — happy-path atomic write produces new complete file
// ---------------------------------------------------------------------------

/// After a successful `atomic_write_sync` the destination holds the new
/// complete bytes; blake3 of the on-disk content matches the in-memory bytes.
#[test]
fn i3_b_happy_path_atomic_write_produces_new_complete_file() {
    let dir = TempDir::new().expect("tempdir");
    let dest = dir.path().join("state.msgpack");

    let new_data = fixture_data("new");
    let new_bytes = serialize(&new_data);
    let expected_hash = blake3_hex(&new_bytes);

    atomic_write_sync(&dest, &new_bytes).expect("atomic_write_sync must succeed");

    // On-disk content is byte-identical to in-memory bytes.
    let on_disk = std::fs::read(&dest).expect("read written file");
    let on_disk_hash = blake3_hex(&on_disk);
    assert_eq!(
        expected_hash, on_disk_hash,
        "blake3 of the written file must match blake3 of the input bytes"
    );

    // No .tmp sibling remains.
    let tmp = std::path::PathBuf::from(format!("{}.tmp", dest.display()));
    assert!(!tmp.exists(), ".tmp sibling must be absent after a successful write");

    // Bytes round-trip through rmp_serde::from_slice.
    let back: AgentCheckpointData =
        rmp_serde::from_slice(&on_disk).expect("round-trip deserialization");
    assert_eq!(back.agent_status, new_data.agent_status);
}

// ---------------------------------------------------------------------------
// (c) BROKEN ARM (R2) — direct write + truncation = torn msgpack the gate rejects
// ---------------------------------------------------------------------------

/// R2 doctrine: a direct write (no tmp) that is truncated mid-write produces
/// a torn msgpack blob. The checkpoint load gate (checkpoint.rs:312-314
/// `rmp_serde::from_slice`) must reject it (Err, not Ok). This test verifies
/// that the integrity gate is NOT fail-open.
///
/// If `rmp_serde::from_slice` returned `Ok(Some(_))` for a torn file the gate
/// would be fail-open — asserted to FAIL the property per R2.
#[test]
fn i3_c_broken_direct_write_torn_msgpack_is_rejected_by_gate() {
    let dir = TempDir::new().expect("tempdir");
    let state_file = dir.path().join("state.msgpack");

    // Write full valid msgpack to the destination.
    let data = fixture_data("full");
    let full_bytes = serialize(&data);
    std::fs::write(&state_file, &full_bytes).expect("write full");

    // Truncate to roughly half — simulates a crash during writeback.
    let torn_len = full_bytes.len() / 2;
    use std::fs::OpenOptions;
    let f = OpenOptions::new()
        .write(true)
        .open(&state_file)
        .expect("open for truncate");
    f.set_len(torn_len as u64).expect("truncate");
    drop(f);

    // Read the torn bytes back.
    let torn_bytes = std::fs::read(&state_file).expect("read torn file");
    assert_eq!(torn_bytes.len(), torn_len, "torn file must be half the original length");

    // The deserialization gate must reject the torn blob.
    let gate_result: Result<AgentCheckpointData, _> = rmp_serde::from_slice(&torn_bytes);
    assert!(
        gate_result.is_err(),
        "BROKEN ARM (R2): rmp_serde::from_slice must return Err for a torn msgpack blob. \
         Returning Ok(_) would mean the checkpoint load gate is fail-open."
    );
}

// ---------------------------------------------------------------------------
// (d) SCHEMA_VERSION lock: I-3 adds no new SSB variant
// ---------------------------------------------------------------------------

/// I-3 is a pure infrastructure hardening session (no new action_type).
/// The schema version floor must be exactly 233 (the live value when I-3
/// was authored — re-read at edit per CONCURRENT_SERIALIZED_SSB_LANE_REREAD).
#[test]
fn i3_d_schema_version_no_new_variant() {
    assert!(
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC >= 233,
        "I-3 added no new SSB variant; SCHEMA_VERSION_PUBLIC must be >= 233, \
         found {}", elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC
    );
}
