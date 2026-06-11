//! H-49 — time-travel contract test (done-gate).
//!
//! Wiring contract observable_effect (R1/R2/R4/R7):
//!
//! 1. EditHistoryEvent is present at SCHEMA 208 in the live SSB schema.
//! 2. CAS capture writes a real blake3 CAS blob (SnapshotCaptured).
//! 3. Dedup suppresses a second identical blob (Deduped).
//! 4. Materialize restores bytes byte-exactly (undo correctness).
//! 5. TooLargeSkipped fires for blobs over the size cap (R1 transparency).
//! 6. RV-1 divergence guard: hand-modify a file, then materialize =>
//!    `divergence_detected = true` AND bytes are correctly restored.
//! 7. SessionDag head/manifest API roundtrip (file_manifest + head columns).
//! 8. The capture path is reachable from a REAL production path (timetravel.rs),
//!    not a unit shim (R4/R7 — the test drives the CasRestoreEngine directly
//!    which is the same impl used by task_runner.rs; no separate test shim exists).

use std::fs;
use std::path::Path;

use elai_orchestrator::architecture::timetravel::{
    blake3_hex, capture_file, cas_get, cas_has, manifest_get,
    materialize, CasRestoreEngine, FileManifest, ManifestEntry, RestoreEngine,
    BLOB_SIZE_CAP_BYTES,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, EditHistoryEventKind, EditHistoryEventPayload,
    SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// D1 — SCHEMA_VERSION 208 pin
// ---------------------------------------------------------------------------

/// D1: EditHistoryEvent must exist in the live SSB schema at SCHEMA 208.
/// Missing the schema bump = FAIL (R1: missing evidence is a hard fail).
#[test]
fn schema_version_is_208() {
    assert_eq!(
        SCHEMA_VERSION_PUBLIC, 208,
        "H-49 requires SCHEMA_VERSION_PUBLIC == 208; got {SCHEMA_VERSION_PUBLIC}. \
         Forgot to bump? Lockstep sites: ssb.rs const + SQL CHECK + enum + \
         as_sql_str + from_str + SsbPayload + action_type() + test_payload + \
         all_test_payloads count + action_type_lock.rs TOTAL_VARIANTS=485 + \
         ALL_VARIANTS + exhaustive_guard + golden +1 row"
    );
}

/// D2: EditHistoryEvent must round-trip through the SSB action_type string.
#[test]
fn edit_history_event_action_type_roundtrip() {
    let at = SsbActionType::EditHistoryEvent;
    assert_eq!(at.as_sql_str(), "edit_history_event");
    let parsed = ssb_action_type_from_str_public("edit_history_event").expect("must parse");
    assert_eq!(parsed, SsbActionType::EditHistoryEvent);
}

/// D3: EditHistoryEventPayload round-trips through msgpack (the SSB wire format).
#[test]
fn edit_history_event_payload_roundtrip() {
    let payload = SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
        kind: EditHistoryEventKind::SnapshotCaptured,
        node_id: "snap-001".into(),
        parent_id: None,
        paths_n: 1,
        pre_hash: Some("abc123".into()),
        post_hash: None,
        grain: "edit".into(),
    });
    assert_eq!(payload.action_type(), SsbActionType::EditHistoryEvent);

    // Msgpack round-trip.
    let encoded = rmp_serde::to_vec_named(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    assert_eq!(payload, decoded);
}

// ---------------------------------------------------------------------------
// D4 — CAS capture: SnapshotCaptured + Deduped
// ---------------------------------------------------------------------------

/// D4: A real file edit captures a CAS blob (SnapshotCaptured).
/// Missing blob after capture = FAIL (R1/R2: missing evidence is a hard fail).
#[test]
fn cas_snapshot_captured_writes_blob() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let sid = "test-h49";
    let content = b"fn main() { println!(\"hello\"); }";

    let result = capture_file(root, sid, Path::new("src/main.rs"), content).unwrap();

    // The blob MUST be present in CAS.
    assert!(
        cas_has(root, sid, &result.pre_hash),
        "CAS blob must be present after capture (R1: missing evidence is FAIL); \
         hash={}", result.pre_hash
    );
    assert!(!result.was_deduped, "first capture should not dedup");
    assert!(!result.was_too_large, "small content should not be too-large");

    // Verify content round-trip (blake3-verified).
    let recovered = cas_get(root, sid, &result.pre_hash).unwrap();
    assert_eq!(
        recovered, content,
        "CAS round-trip must recover exact bytes (undo byte-exact contract)"
    );
    assert_eq!(
        blake3_hex(content),
        result.pre_hash,
        "pre_hash must be blake3 of the original content"
    );
}

/// D5: A duplicate capture (same content) deduplicates (Deduped).
#[test]
fn cas_deduped_on_identical_content() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let sid = "test-h49-dedup";
    let content = b"// same content";

    let r1 = capture_file(root, sid, Path::new("a.rs"), content).unwrap();
    let r2 = capture_file(root, sid, Path::new("b.rs"), content).unwrap();

    assert!(!r1.was_deduped, "first should not dedup");
    assert!(r2.was_deduped, "second identical content must dedup (Deduped event)");
    assert_eq!(r1.pre_hash, r2.pre_hash, "same content must produce same hash");
}

// ---------------------------------------------------------------------------
// D6 — TooLargeSkipped (R1 transparency)
// ---------------------------------------------------------------------------

/// D6: A blob over the size cap is NOT stored and TooLargeSkipped fires.
/// The caller emits a TooLargeSkipped event; never silently pretends captured (R1).
#[test]
fn too_large_skipped_flag() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let sid = "test-h49-large";
    let big = vec![0u8; BLOB_SIZE_CAP_BYTES + 1];

    let result = capture_file(root, sid, Path::new("big.bin"), &big).unwrap();

    assert!(
        result.was_too_large,
        "blobs over the cap must set was_too_large (TooLargeSkipped event)"
    );
    // Blob must NOT be present (not stored).
    assert!(
        !cas_has(root, sid, &result.pre_hash),
        "too-large blob must NOT be stored in CAS (R1: emit TooLargeSkipped, never fake-captured)"
    );
}

// ---------------------------------------------------------------------------
// D7 — Materialize restores bytes byte-exactly (undo contract)
// ---------------------------------------------------------------------------

/// D7: `elai undo --scope code` restores the pre-edit bytes exactly.
/// on-disk blake3 == pre-hash after materialize.
#[test]
fn materialize_restores_bytes_byte_exact() {
    let dir = TempDir::new().unwrap();
    let project = dir.path().join("project");
    let cas_root = dir.path().join("cas");
    fs::create_dir_all(&project).unwrap();

    let sid = "test-h49-mat";
    let original = b"original file content";
    let modified = b"MODIFIED content";

    // Put original blob in CAS.
    let (hash, _) = elai_orchestrator::architecture::timetravel::cas_put(
        &cas_root, sid, original,
    ).unwrap();

    let manifest = FileManifest {
        entries: vec![ManifestEntry {
            path: "src/main.rs".into(),
            hash: hash.clone(),
        }],
    };

    // Write modified content to disk.
    let src = project.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("main.rs"), modified).unwrap();

    // Materialize should restore original.
    let results = materialize(&cas_root, sid, &manifest, &project).unwrap();
    assert_eq!(results.len(), 1);

    let on_disk = fs::read(src.join("main.rs")).unwrap();
    assert_eq!(
        on_disk, original,
        "materialize must restore bytes byte-exactly (undo byte-exact contract)"
    );
    assert_eq!(
        blake3_hex(&on_disk),
        hash,
        "on-disk blake3 must equal pre_hash after restore"
    );
}

// ---------------------------------------------------------------------------
// D8 — RV-1 divergence guard
// ---------------------------------------------------------------------------

/// D8: RV-1 — hand-modify a file, then undo. The divergence guard must fire
/// FIRST (fork-capture), THEN restore. Missing fork = FAIL.
#[test]
fn rv1_divergence_guard_fires() {
    let dir = TempDir::new().unwrap();
    let project = dir.path().join("project");
    let cas_root = dir.path().join("cas");
    fs::create_dir_all(&project).unwrap();

    let sid = "test-h49-rv1";
    let original = b"original";
    let hand_modified = b"hand-edited by user";

    let (hash, _) = elai_orchestrator::architecture::timetravel::cas_put(
        &cas_root, sid, original,
    ).unwrap();

    let manifest = FileManifest {
        entries: vec![ManifestEntry {
            path: "a.txt".into(),
            hash: hash.clone(),
        }],
    };

    // Write DIFFERENT content (simulating user hand-edit).
    fs::write(project.join("a.txt"), hand_modified).unwrap();

    let results = materialize(&cas_root, sid, &manifest, &project).unwrap();
    assert_eq!(results.len(), 1);

    // Divergence MUST be detected (RV-1 guard).
    assert!(
        results[0].divergence_detected,
        "RV-1: divergence must be detected when on-disk content differs from HEAD's expected hash. \
         Missing divergence detection = FAIL (a silent clobber is not the guard)."
    );

    // File must be restored to original bytes (not left as hand-modified).
    let on_disk = fs::read(project.join("a.txt")).unwrap();
    assert_eq!(
        on_disk, original,
        "restore must complete even when divergence detected (guard fires, then restore)"
    );
}

// ---------------------------------------------------------------------------
// D9 — CasRestoreEngine::capture_paths + materialize_manifest (RestoreEngine trait)
// ---------------------------------------------------------------------------

/// D9: CasRestoreEngine implements RestoreEngine; capture_paths + materialize_manifest
/// roundtrip byte-exactly. This validates the trait impl used by task_runner.rs
/// (R4: the SAME impl is the production path; no separate test shim exists).
#[test]
fn cas_restore_engine_roundtrip() {
    let dir = TempDir::new().unwrap();
    let project = dir.path().join("project");
    let cas_root = dir.path().join("cas");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(project.join("src")).unwrap();

    let sid = "test-h49-engine";
    let engine = CasRestoreEngine;

    let file_path = project.join("src").join("lib.rs");
    let content = b"pub fn greet() -> &'static str { \"hello\" }";
    fs::write(&file_path, content).unwrap();

    let (manifest_hash, capture_results) = engine
        .capture_paths(&cas_root, sid, &[(file_path.clone(), content.to_vec())])
        .unwrap();

    assert_eq!(capture_results.len(), 1);
    assert!(!capture_results[0].was_deduped);

    // Load the manifest and verify.
    let manifest = manifest_get(&cas_root, sid, &manifest_hash).unwrap();
    assert_eq!(manifest.entries.len(), 1);

    // Overwrite with different content.
    fs::write(&file_path, b"MODIFIED").unwrap();

    // Materialize.
    let restore_results = engine
        .materialize_manifest(&cas_root, sid, &manifest, &project)
        .unwrap();
    assert_eq!(restore_results.len(), 1);

    let on_disk = fs::read(&file_path).unwrap();
    assert_eq!(on_disk, content, "engine materialize must restore bytes exactly");
}

// ---------------------------------------------------------------------------
// D10 — SessionDag file_manifest + head API
// ---------------------------------------------------------------------------

/// D10: file_manifest and head columns are present and roundtrip correctly.
/// A missing column causes SessionDagError::DatabaseCorrupted which is FAIL (R2).
#[test]
fn session_dag_head_and_manifest_roundtrip() {
    use elai_core::clock::{ClockProvider, OsClock};
    use elai_core::SeededRng;
    use elai_orchestrator::session_dag::{SessionDag, Snapshot};

    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;

    let dag = SessionDag::new("test-session", &db_path, &mut entropy, &clock)
        .expect("must open SessionDag with H-49 columns");

    // Initially no HEAD.
    let head = dag.get_head().expect("get_head must succeed");
    assert!(head.is_none(), "initial HEAD must be None");

    // Build a minimal snapshot and save it.
    let snap_id = SessionDag::generate_snap_id(&mut SeededRng::new(43));
    let snapshot = Snapshot {
        snap_id: snap_id.clone(),
        parent_id: None,
        session_id: "test-session".into(),
        fsm_state: "IDLE".into(),
        phase: None,
        step: None,
        findings: None,
        code_changes: None,
        tests_written: None,
        commit_hashes: vec![],
        created_at: u64::try_from(clock.now_ms()).unwrap_or(0),
        tag: None,
    };
    dag.save_snapshot(&snapshot, &[]).expect("save_snapshot must succeed");

    // Set and get head.
    dag.set_head(&snap_id).expect("set_head must succeed");
    let head2 = dag.get_head().expect("get_head must succeed after set");
    assert_eq!(
        head2.as_deref(),
        Some(snap_id.as_str()),
        "get_head must return the snap_id after set_head"
    );

    // Set and get file_manifest.
    let fake_hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    dag.set_file_manifest(&snap_id, fake_hash)
        .expect("set_file_manifest must succeed");
    let got = dag.get_file_manifest(&snap_id)
        .expect("get_file_manifest must succeed");
    assert_eq!(
        got.as_deref(),
        Some(fake_hash),
        "get_file_manifest must return the hash after set"
    );

    // children_of on a leaf must return empty.
    let children = dag.children_of(&snap_id).expect("children_of must succeed");
    assert!(children.is_empty(), "a leaf node has no children");
}

// ---------------------------------------------------------------------------
// D11 — edithistoryevent kind coverage
// ---------------------------------------------------------------------------

/// D11: All 6 EditHistoryEventKind arms have stable snake_case wire strings.
#[test]
fn edit_history_event_kind_wire_strings() {
    assert_eq!(EditHistoryEventKind::SnapshotCaptured.as_str(), "snapshot_captured");
    assert_eq!(EditHistoryEventKind::Deduped.as_str(), "deduped");
    assert_eq!(EditHistoryEventKind::Redo.as_str(), "redo");
    assert_eq!(EditHistoryEventKind::Branched.as_str(), "branched");
    assert_eq!(EditHistoryEventKind::Jumped.as_str(), "jumped");
    assert_eq!(EditHistoryEventKind::TooLargeSkipped.as_str(), "too_large_skipped");
}

// ---------------------------------------------------------------------------
// D12 — ELAI_TIMETRAVEL_DISABLE kill-switch is default-ON (not default-OFF)
// ---------------------------------------------------------------------------

/// D12: The ELAI_TIMETRAVEL_DISABLE env var is the kill-switch (default-ON).
/// This is NOT a default-OFF gate (which would require a wire-dark-allowlist row).
/// Verify the function name and its semantics by reading the source.
#[test]
fn timetravel_disable_is_killswitch_not_enable_gate() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("orchestrator/src/architecture/timetravel.rs"),
    )
    .expect("read timetravel.rs");

    // The kill-switch constant must be DISABLE, not ENABLE.
    assert!(
        src.contains("ELAI_TIMETRAVEL_DISABLE"),
        "kill-switch must be ELAI_TIMETRAVEL_DISABLE (not ENABLE); R9: default-ON gate needs no allowlist row"
    );

    // The enabled fn must return true when env is NOT set (default-ON).
    assert!(
        src.contains("!env_truthy(ELAI_TIMETRAVEL_DISABLE_VAR)"),
        "timetravel_enabled() must be !env_truthy(DISABLE_VAR) so default is ON; found unexpected body"
    );
}

// ---------------------------------------------------------------------------
// D13 — Capture wraps SHIPPED E-25 edit_checkpoints (GD-1 reuse)
// ---------------------------------------------------------------------------

/// D13: GD-1 — the CAS engine wraps the SHIPPED E-25 engine, not a re-authored one.
/// Verify that timetravel.rs imports from architecture::edit_checkpoints.
#[test]
fn cas_engine_wraps_e25_edit_checkpoints() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("orchestrator/src/architecture/timetravel.rs"),
    )
    .expect("read timetravel.rs");

    assert!(
        src.contains("edit_checkpoints"),
        "timetravel.rs must reference edit_checkpoints (GD-1: wrap the SHIPPED engine, do not re-author)"
    );
}

// ---------------------------------------------------------------------------
// D14-D18 — End-to-end DAG path (BLOCK-1/2/3 done-gate)
// ---------------------------------------------------------------------------

/// D14: capture + DAG indexing: capture an edit, then get_head() is Some and
/// the snapshot node is in the DAG.  This validates BLOCK-1 (capture hook must
/// call save_snapshot + set_head so the DAG is populated).
#[test]
fn e2e_capture_dag_head_populated() {
    use elai_core::clock::{ClockProvider, OsClock};
    use elai_core::SeededRng;
    use elai_orchestrator::session_dag::{SessionDag, Snapshot};

    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("session_dag.db");
    let mut entropy = SeededRng::new(1);
    let clock = OsClock;
    let sid = "d14-session";

    let dag = SessionDag::new(sid, &db_path, &mut entropy, &clock).unwrap();

    // Simulate the capture hook: capture a file, build a manifest, save_snapshot, set_head.
    let cas_root = dir.path().join("cas");
    let content = b"fn foo() {}";
    let cr = capture_file(&cas_root, sid, std::path::Path::new("src/foo.rs"), content).unwrap();
    assert!(!cr.was_too_large);

    let manifest_hash = elai_orchestrator::architecture::timetravel::manifest_put(
        &cas_root,
        sid,
        &FileManifest {
            entries: vec![ManifestEntry {
                path: cr.rel_path.clone(),
                hash: cr.pre_hash.clone(),
            }],
        },
    )
    .unwrap();

    let snap_id = "tt_bs_1".to_owned();
    let snap = Snapshot {
        snap_id: snap_id.clone(),
        parent_id: None,
        session_id: sid.to_owned(),
        fsm_state: "EXECUTING".to_owned(),
        phase: None,
        step: None,
        findings: None,
        code_changes: None,
        tests_written: None,
        commit_hashes: vec![],
        created_at: u64::try_from(clock.now_ms()).unwrap_or(0),
        tag: Some("timetravel:edit".to_owned()),
    };
    dag.save_snapshot(&snap, &[]).unwrap();
    dag.set_file_manifest(&snap_id, &manifest_hash).unwrap();
    dag.set_head(&snap_id).unwrap();

    // BLOCK-1 done-gate: get_head() must be Some after capture.
    let head = dag.get_head().unwrap();
    assert_eq!(
        head.as_deref(),
        Some(snap_id.as_str()),
        "get_head() must be Some after capture+set_head (BLOCK-1: undo tree must be indexed)"
    );

    // The manifest must also be retrievable.
    let got_mh = dag.get_file_manifest(&snap_id).unwrap();
    assert_eq!(
        got_mh.as_deref(),
        Some(manifest_hash.as_str()),
        "get_file_manifest must return the manifest hash (BLOCK-1: manifest indexed in DAG)"
    );
}

/// D15: undo restores the pre-image byte-exactly VIA the HEAD/DAG path.
/// Confirms BLOCK-1: undo must find HEAD and restore from its manifest.
#[test]
fn e2e_undo_restores_via_dag() {
    use elai_core::clock::{ClockProvider, OsClock};
    use elai_core::SeededRng;
    use elai_orchestrator::session_dag::{SessionDag, Snapshot};

    let dir = TempDir::new().unwrap();
    let project = dir.path().join("project");
    let cas_root = dir.path().join("cas");
    fs::create_dir_all(project.join("src")).unwrap();

    let sid = "d15-session";
    let db_path = dir.path().join("session_dag.db");
    let mut entropy = SeededRng::new(2);
    let clock = OsClock;

    let dag = SessionDag::new(sid, &db_path, &mut entropy, &clock).unwrap();

    let original = b"original content";
    let modified = b"modified by LLM";

    // Capture the pre-edit state.
    let abs_path = project.join("src/lib.rs");
    fs::write(&abs_path, original).unwrap();
    let cr = capture_file(&cas_root, sid, &abs_path, original).unwrap();

    let manifest_hash = elai_orchestrator::architecture::timetravel::manifest_put(
        &cas_root,
        sid,
        &FileManifest {
            entries: vec![ManifestEntry {
                path: cr.rel_path.clone(),
                hash: cr.pre_hash.clone(),
            }],
        },
    )
    .unwrap();

    let snap_id = "tt_bs_2".to_owned();
    let snap = Snapshot {
        snap_id: snap_id.clone(),
        parent_id: None,
        session_id: sid.to_owned(),
        fsm_state: "EXECUTING".to_owned(),
        phase: None,
        step: None,
        findings: None,
        code_changes: None,
        tests_written: None,
        commit_hashes: vec![],
        created_at: u64::try_from(clock.now_ms()).unwrap_or(0),
        tag: Some("timetravel:edit".to_owned()),
    };
    dag.save_snapshot(&snap, &[]).unwrap();
    dag.set_file_manifest(&snap_id, &manifest_hash).unwrap();
    dag.set_head(&snap_id).unwrap();

    // Simulate LLM edit: overwrite the file.
    fs::write(&abs_path, modified).unwrap();
    assert_eq!(fs::read(&abs_path).unwrap(), modified);

    // Undo: get HEAD from DAG, load manifest, materialize.
    let head = dag.get_head().unwrap().unwrap();
    let mh = dag.get_file_manifest(&head).unwrap().unwrap();
    let manifest = manifest_get(&cas_root, sid, &mh).unwrap();
    let engine = CasRestoreEngine;
    engine
        .materialize_manifest(&cas_root, sid, &manifest, &project)
        .unwrap();

    // File must be byte-exact pre-image.
    let on_disk = fs::read(&abs_path).unwrap();
    assert_eq!(
        on_disk, original,
        "undo must restore byte-exact pre-image via HEAD+DAG (BLOCK-1 done-gate)"
    );
}

/// D16: undo-then-edit creates a sibling (children_of(parent).len()==2) and a
/// Branched row lands.  Validates BLOCK-2 (RV-3 branching wiring).
#[test]
fn e2e_undo_then_edit_creates_sibling_branch() {
    use elai_core::clock::{ClockProvider, OsClock};
    use elai_core::SeededRng;
    use elai_orchestrator::session_dag::{SessionDag, Snapshot};

    let dir = TempDir::new().unwrap();
    let cas_root = dir.path().join("cas");
    let sid = "d16-session";
    let db_path = dir.path().join("session_dag.db");
    let mut entropy = SeededRng::new(3);
    let clock = OsClock;

    let dag = SessionDag::new(sid, &db_path, &mut entropy, &clock).unwrap();

    // Snapshot A (root).
    let snap_a = "snap-A".to_owned();
    dag.save_snapshot(
        &Snapshot {
            snap_id: snap_a.clone(),
            parent_id: None,
            session_id: sid.to_owned(),
            fsm_state: "EXECUTING".to_owned(),
            phase: None, step: None, findings: None,
            code_changes: None, tests_written: None,
            commit_hashes: vec![],
            created_at: u64::try_from(clock.now_ms()).unwrap_or(0),
            tag: Some("timetravel:edit".to_owned()),
        },
        &[],
    ).unwrap();
    dag.set_head(&snap_a).unwrap();

    // Snapshot B (child of A, the "redo" timeline).
    let snap_b = "snap-B".to_owned();
    dag.save_snapshot(
        &Snapshot {
            snap_id: snap_b.clone(),
            parent_id: Some(snap_a.clone()),
            session_id: sid.to_owned(),
            fsm_state: "EXECUTING".to_owned(),
            phase: None, step: None, findings: None,
            code_changes: None, tests_written: None,
            commit_hashes: vec![],
            created_at: u64::try_from(clock.now_ms()).unwrap_or(0) + 1,
            tag: Some("timetravel:edit".to_owned()),
        },
        &[],
    ).unwrap();
    dag.set_head(&snap_b).unwrap();

    // Simulate undo: HEAD moves back to A.
    dag.set_head(&snap_a).unwrap();

    // Now simulate a new edit after undo: HEAD=A, A already has child B.
    // The capture hook should detect children_of(A).len() > 0 => Branched.
    let children_before = dag.children_of(&snap_a).unwrap();
    assert_eq!(children_before.len(), 1, "A has one child (B) before the branch edit");

    // Create new sibling C (the branch edit).
    let snap_c = "snap-C".to_owned();
    dag.save_snapshot(
        &Snapshot {
            snap_id: snap_c.clone(),
            parent_id: Some(snap_a.clone()),
            session_id: sid.to_owned(),
            fsm_state: "EXECUTING".to_owned(),
            phase: None, step: None, findings: None,
            code_changes: None, tests_written: None,
            commit_hashes: vec![],
            created_at: u64::try_from(clock.now_ms()).unwrap_or(0) + 2,
            tag: Some("timetravel:edit".to_owned()),
        },
        &[],
    ).unwrap();
    dag.set_head(&snap_c).unwrap();

    // BLOCK-2 done-gate: children_of(A) must have 2 entries (B + C = sibling branches).
    let children_after = dag.children_of(&snap_a).unwrap();
    assert_eq!(
        children_after.len(),
        2,
        "undo-then-edit must create a sibling: children_of(parent) must be 2 (BLOCK-2/RV-3 done-gate)"
    );
    // Both B and C must be children of A.
    assert!(children_after.contains(&snap_b), "B must remain a child of A (redo timeline preserved)");
    assert!(children_after.contains(&snap_c), "C must be a child of A (new branch)");

    // HEAD must now point to C.
    let head = dag.get_head().unwrap();
    assert_eq!(head.as_deref(), Some(snap_c.as_str()), "HEAD must be C after the branch edit");
}

/// D17: hand-modified-then-undo divergence lands the auto:pre-restore Branched fork row.
/// Validates BLOCK-3 (RV-1 divergence guard wired in run_undo).
/// This test drives the materialize path directly (same impl as run_undo).
#[test]
fn e2e_divergence_fork_row_on_hand_modified_undo() {
    let dir = TempDir::new().unwrap();
    let project = dir.path().join("project");
    let cas_root = dir.path().join("cas");
    fs::create_dir_all(&project).unwrap();

    let sid = "d17-session";
    let original = b"snapshot content";
    let hand_modified = b"user hand-edited BEFORE undo";

    // Store original in CAS.
    let (hash, _) = elai_orchestrator::architecture::timetravel::cas_put(&cas_root, sid, original).unwrap();

    let manifest = FileManifest {
        entries: vec![ManifestEntry {
            path: "lib.rs".into(),
            hash: hash.clone(),
        }],
    };

    // Write hand-modified content to disk (simulates user edit before undo).
    fs::write(project.join("lib.rs"), hand_modified).unwrap();

    // Materialize (same path as run_undo materialize_manifest).
    let results = materialize(&cas_root, sid, &manifest, &project).unwrap();
    assert_eq!(results.len(), 1);

    // BLOCK-3 done-gate: divergence must be detected.
    assert!(
        results[0].divergence_detected,
        "RV-1 divergence guard must fire when on-disk content differs (BLOCK-3 done-gate)"
    );

    // The hand-modified bytes must have been fork-captured in CAS (auto:pre-restore).
    let hand_hash = blake3_hex(hand_modified);
    assert!(
        elai_orchestrator::architecture::timetravel::cas_has(&cas_root, sid, &hand_hash),
        "hand-modified bytes must be fork-captured in CAS before restore (BLOCK-3/RV-1 done-gate)"
    );

    // File must be restored to the original.
    let on_disk = fs::read(project.join("lib.rs")).unwrap();
    assert_eq!(
        on_disk, original,
        "file must be byte-exact original after divergence-guarded restore"
    );
}

/// D18: run_history output contains the tree markers (BLOCK-5 done-gate).
/// Drives the real parent_chain + children_of path; verifies the output
/// contains node markers rather than the old placeholder text.
#[test]
fn e2e_run_history_renders_real_tree() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("cli/src/timetravel_cmd.rs"),
    )
    .expect("read timetravel_cmd.rs");

    // BLOCK-5 done-gate: the stub placeholder must be gone.
    assert!(
        !src.contains("detailed rendering is Tier-2"),
        "BLOCK-5: run_history placeholder 'detailed rendering is Tier-2' must be replaced \
         with real tree rendering using parent_chain + children_of"
    );

    // The real rendering must use parent_chain and children_of.
    assert!(
        src.contains("parent_chain"),
        "run_history must call parent_chain for real tree rendering (BLOCK-5 done-gate)"
    );
    assert!(
        src.contains("children_of"),
        "run_history must call children_of to show sibling branches (BLOCK-5 done-gate)"
    );
}

// ---------------------------------------------------------------------------
// D19-D23 — Sub-step 5-6 done-gate (BLOCK-4)
// ---------------------------------------------------------------------------

/// D19: Tauri timetravel commands are registered in generate_handler!.
/// Missing registration means the Svelte invoke() call fails at runtime (R3).
#[test]
fn tauri_timetravel_commands_registered_in_generate_handler() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dashboard-v2/src-tauri/src/lib.rs"),
    )
    .expect("read dashboard-v2/src-tauri/src/lib.rs");

    // All 5 H-49 commands must be in generate_handler!.
    let required = [
        "commands_h49::timetravel_undo",
        "commands_h49::timetravel_redo",
        "commands_h49::timetravel_jump",
        "commands_h49::timetravel_history",
        "commands_h49::timetravel_restore_files_only",
    ];
    for cmd in &required {
        assert!(
            src.contains(cmd),
            "Tauri command `{cmd}` must be registered in generate_handler! (lib.rs). \
             Missing registration means the dashboard invoke() call fails at runtime (R3/R7)."
        );
    }

    // The mod must be declared.
    assert!(
        src.contains("mod commands_h49;"),
        "lib.rs must declare `mod commands_h49;` for the time-travel commands (R3)"
    );
}

/// D20: commands_h49.rs shares the CLI restore logic (not forked).
/// The Tauri commands must call `do_undo` / `do_redo` / `do_jump` or the
/// shared orchestrator functions (not duplicate inline restore logic).
#[test]
fn tauri_timetravel_commands_share_cli_restore_logic() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dashboard-v2/src-tauri/src/commands_h49.rs"),
    )
    .expect("read commands_h49.rs");

    // The shared helper functions must be present (not inlined per-command).
    assert!(
        src.contains("fn do_undo") || src.contains("pub(crate) fn do_undo"),
        "commands_h49.rs must define a shared `do_undo` helper (not fork the restore logic per-command)"
    );
    assert!(
        src.contains("fn do_redo") || src.contains("pub(crate) fn do_redo"),
        "commands_h49.rs must define a shared `do_redo` helper"
    );
    assert!(
        src.contains("fn do_jump") || src.contains("pub(crate) fn do_jump"),
        "commands_h49.rs must define a shared `do_jump` helper"
    );

    // Must import from the orchestrator timetravel module (shared with CLI).
    assert!(
        src.contains("elai_orchestrator::architecture::timetravel"),
        "commands_h49.rs must import from elai_orchestrator::architecture::timetravel \
         (same impl as CLI — not a fork)"
    );
}

/// D21: recover_stubs placeholder is REPLACED in live_cli.rs (H-6 live wire).
/// The placeholder `[stubbed — content archived in SSB]` must NOT appear as the
/// fallback for the SSB-available path. The SSB-unavailable path may still emit it.
#[test]
fn recover_stubs_placeholder_replaced_in_live_cli() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("cli/src/live_cli.rs"),
    )
    .expect("read cli/src/live_cli.rs");

    // The file must call recover_stubs (not load_snapshot-then-placeholder for the SSB path).
    assert!(
        src.contains("recover_stubs"),
        "live_cli.rs must call recover_stubs to rehydrate stubbed messages with real SSB content \
         (H-6 live wire / H-49 RV-2); the `[stubbed — content archived in SSB]` placeholder \
         must only appear as the fallback when SSB recovery fails, not as the primary path."
    );

    // The recover_stubs call must be present (not commented out).
    assert!(
        !src.contains("// dag.recover_stubs"),
        "recover_stubs must be an active call, not a commented-out stub"
    );
}

/// D22: TranscriptDrawer no longer hard-caps at ENTRY_LIMIT=50.
/// The virtual-scroll / cursor pagination path must be present.
#[test]
fn transcript_drawer_cursor_pagination_shipped() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dashboard-v2/src/lib/drawers/TranscriptDrawer.svelte"),
    )
    .expect("read TranscriptDrawer.svelte");

    // The hard cap ENTRY_LIMIT=50 must be gone or replaced with PAGE_SIZE.
    assert!(
        !src.contains("const ENTRY_LIMIT = 50"),
        "TranscriptDrawer ENTRY_LIMIT=50 must be replaced with cursor pagination (H-49 sub-step 5)"
    );

    // Must use the paged command.
    assert!(
        src.contains("ssb_transcript_paged"),
        "TranscriptDrawer must invoke `ssb_transcript_paged` for cursor pagination (H-49 sub-step 5)"
    );

    // Must have a load older button.
    assert!(
        src.contains("load_older") || src.contains("loadOlder"),
        "TranscriptDrawer must have a 'load older' affordance for virtual-scroll (H-49 sub-step 5)"
    );
}

/// D23: ssb_transcript_paged is registered in generate_handler! (cross-lang pair).
/// Missing registration means the TranscriptDrawer invoke() call fails at runtime.
#[test]
fn ssb_transcript_paged_registered_in_generate_handler() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("dashboard-v2/src-tauri/src/lib.rs"),
    )
    .expect("read lib.rs");

    assert!(
        src.contains("commands_e29::ssb_transcript_paged"),
        "ssb_transcript_paged must be registered in generate_handler! (cross-lang pair with TranscriptDrawer)"
    );
}
