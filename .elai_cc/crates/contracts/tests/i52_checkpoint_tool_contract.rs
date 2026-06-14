//! I-52 (Track 20 Wave 3) — agent-callable checkpoint tool contract.
//!
//! Black-box done-gate for the 3-arm save/rewind/diff checkpoint tool over the
//! SHIPPED H-49 time-travel CAS substrate. Pins:
//!   (C1) SSB bijection: CheckpointToolEvent's 4 wire tags round-trip
//!        from_str <-> as_sql_str; SCHEMA_VERSION_PUBLIC >= 259.
//!   (C2) REAL-DB production path: drive translate_tool_request_for_test (the
//!        live agent dispatch chokepoint) save -> diff -> rewind; a covered file
//!        is edited speculatively BEFORE the rewind (ordinary divergence inside
//!        the checkpoint set). Assert the Saved + Rewound rows land in a TempDir
//!        SQLite SSB, the restored bytes blake3-match the checkpoint pre-images
//!        (non-destructive restore), AND the divergent bytes survive as a
//!        fork-captured redo target in the CAS (divergence preserved, not lost).
//!   (C3) UNRECOVERABLE REFUSAL (the load-bearing safety observable, R1): save,
//!        then FORCE a genuinely unrecoverable condition -- the checkpoint's CAS
//!        blob is removed so the bytes cannot be restored -- and rewind. Assert
//!        the rewind is REFUSED (not silently applied): a RefusedDiverged row
//!        lands naming the offending path, AND the on-disk bytes are NOT
//!        overwritten (the refusal is real, never a silent partial restore).
//!
//! Wiring: the tool is reachable ONLY via the production agent dispatch
//! (translate_tool_request); these contract callers drive that exact entrypoint
//! through the public test helper (a contracts-only caller does NOT satisfy R7,
//! but the helper forwards into the real translate_tool_request match — the same
//! code path a live worker hits).
//!
//! Files under test:
//!   - crates/orchestrator/src/production_agent_rpc.rs (checkpoint_* dispatch arms)
//!   - crates/orchestrator/src/architecture/checkpoint_tool.rs (save/rewind/diff core)
//!   - crates/orchestrator/src/architecture/timetravel.rs (CAS + materialize, reused)
//!   - crates/orchestrator/src/ssb/{action_type,payloads,schema}.rs (CheckpointToolEvent)

use std::pin::Pin;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTierResult,
};
use std::path::PathBuf;

use elai_ipc_types::{TOOL_CHECKPOINT_DIFF, TOOL_CHECKPOINT_REWIND, TOOL_CHECKPOINT_SAVE};
use elai_orchestrator::architecture::timetravel;
use elai_orchestrator::production_agent_rpc::translate_tool_request_for_test;
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, CheckpointToolEventKind, Ssb,
    SsbActionType, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const CHECKPOINT_SESSION_ID_ENV: &str = "ELAI_AGENT_SESSION_ID";
const CHECKPOINT_ROOT_ENV: &str = "ELAI_AGENT_CHECKPOINT_ROOT";
const PROJECT_ROOT_ENV: &str = "ELAI_PROJECT_ROOT";

// Serialize the env-mutating REAL-DB tests (C2/C3) to prevent a process-global
// ELAI_PROJECT_ROOT / ELAI_AGENT_* race under the default parallel runner. Plain
// std::sync::Mutex so no external dep is required (mirrors the
// feasibility_probe_contract.rs ENV_MUTEX idiom).
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i52-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb open"))
}

/// Set the run-context env scope the checkpoint dispatch arms read. session_id +
/// checkpoint-root + project-root are run-context-supplied (NEVER agent-supplied);
/// here the test plays the run context.
fn set_checkpoint_env(session_id: &str, cas_root: &std::path::Path, project_root: &std::path::Path) {
    std::env::set_var(CHECKPOINT_SESSION_ID_ENV, session_id);
    std::env::set_var(CHECKPOINT_ROOT_ENV, cas_root);
    std::env::set_var(PROJECT_ROOT_ENV, project_root);
}

fn clear_checkpoint_env() {
    std::env::remove_var(CHECKPOINT_SESSION_ID_ENV);
    std::env::remove_var(CHECKPOINT_ROOT_ENV);
    std::env::remove_var(PROJECT_ROOT_ENV);
}

// PanicDispatcher: the checkpoint arms dispatch via the JSON path and never
// delegate to the ToolDispatcher; reaching it is a wiring bug.
struct PanicDispatcher;

impl ToolDispatcher for PanicDispatcher {
    fn dispatch_edit(
        &self,
        _req: EditRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>>
    {
        panic!("PanicDispatcher::dispatch_edit reached — checkpoint arm must not delegate")
    }

    fn dispatch_validate(
        &self,
        _req: ValidateRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async { None })
    }

    fn dispatch_git_commit(
        &self,
        _req: GitCommitRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>>
    {
        panic!("PanicDispatcher::dispatch_git_commit reached — checkpoint arm must not delegate")
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>>
    {
        Box::pin(async {
            Ok(FileView {
                outline_only: false,
                signatures: Vec::new(),
                imports: Vec::new(),
                full_content: Some(String::new()),
                total_lines: 0,
            })
        })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

// ---------------------------------------------------------------------------
// (C1) SSB bijection: the 4 CheckpointToolEvent wire tags + SCHEMA floor.
// ---------------------------------------------------------------------------

#[test]
fn i52_c1_checkpoint_tool_event_bijection() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 259,
        "I-52 requires SCHEMA_VERSION >= 259, got {SCHEMA_VERSION_PUBLIC}"
    );

    let cases = [
        ("checkpoint_tool_saved", CheckpointToolEventKind::Saved),
        ("checkpoint_tool_rewound", CheckpointToolEventKind::Rewound),
        ("checkpoint_tool_diffed", CheckpointToolEventKind::Diffed),
        (
            "checkpoint_tool_refused_diverged",
            CheckpointToolEventKind::RefusedDiverged,
        ),
    ];
    for (tag, kind) in cases {
        let at = ssb_action_type_from_str_public(tag)
            .unwrap_or_else(|_| panic!("from_str must accept '{tag}'"));
        assert_eq!(
            at,
            SsbActionType::CheckpointToolEvent(kind),
            "from_str('{tag}') must map to CheckpointToolEvent({kind:?})"
        );
        assert_eq!(
            at.as_sql_str(),
            tag,
            "as_sql_str round-trip for CheckpointToolEvent({kind:?})"
        );
    }
}

// ---------------------------------------------------------------------------
// (C2) REAL-DB happy path: save -> diff(clean) -> rewind, rows land, bytes match.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i52_c2_save_diff_rewind_via_production_dispatch() {
    let _env = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    let cas_dir = TempDir::new().expect("cas tempdir");
    let proj_dir = TempDir::new().expect("project tempdir");
    let ssb_dir = TempDir::new().expect("ssb tempdir");
    let session = "i52-c2-session";
    set_checkpoint_env(session, cas_dir.path(), proj_dir.path());

    let ssb = test_ssb(&ssb_dir, session);
    let dispatcher = PanicDispatcher;
    let clock = test_clock();

    // Two project files with known pre-edit content.
    let original_a = b"fn a() -> i32 { 1 }\n";
    let original_b = b"fn b() -> i32 { 2 }\n";
    std::fs::write(proj_dir.path().join("a.rs"), original_a).expect("write a.rs");
    std::fs::write(proj_dir.path().join("b.rs"), original_b).expect("write b.rs");

    // SAVE checkpoint "A" over both files (JSON-path dispatch).
    let save_args = serde_json::json!({ "label": "A", "paths": ["a.rs", "b.rs"] });
    let save_bytes = serde_json::to_vec(&save_args).expect("encode save args");
    let save_res = translate_tool_request_for_test(
        TOOL_CHECKPOINT_SAVE,
        &save_bytes,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("checkpoint_save must succeed");
    let save_json: serde_json::Value =
        serde_json::from_slice(&save_res).expect("save response JSON");
    assert_eq!(save_json["saved"], serde_json::json!(true));

    // Saved row must land (off-agent observable, R1).
    let saved_rows = ssb
        .query_by_type(SsbActionType::CheckpointToolEvent(
            CheckpointToolEventKind::Saved,
        ))
        .expect("query Saved rows");
    assert!(
        !saved_rows.is_empty(),
        "CheckpointToolEvent(Saved) row must land in SSB (R1 missing=FAIL)"
    );

    // DIFF on a clean tree -> no changes.
    let diff_args = serde_json::json!({ "label": "A" });
    let diff_bytes = serde_json::to_vec(&diff_args).expect("encode diff args");
    let diff_res = translate_tool_request_for_test(
        TOOL_CHECKPOINT_DIFF,
        &diff_bytes,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("checkpoint_diff must succeed");
    let diff_json: serde_json::Value =
        serde_json::from_slice(&diff_res).expect("diff response JSON");
    assert_eq!(
        diff_json["changed_paths"],
        serde_json::json!([]),
        "clean tree must diff empty"
    );

    // Edit a.rs in-place (speculative change), then REWIND to A. This is an
    // ORDINARY divergence of a COVERED file: the rewind must succeed (restore +
    // fork-capture the speculative bytes), NOT refuse.
    let speculative_a = b"fn a() -> i32 { 999 }\n";
    std::fs::write(proj_dir.path().join("a.rs"), speculative_a)
        .expect("speculative edit a.rs");

    let rewind_args = serde_json::json!({ "label": "A" });
    let rewind_bytes = serde_json::to_vec(&rewind_args).expect("encode rewind args");
    let rewind_res = translate_tool_request_for_test(
        TOOL_CHECKPOINT_REWIND,
        &rewind_bytes,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("checkpoint_rewind must succeed");
    let rewind_json: serde_json::Value =
        serde_json::from_slice(&rewind_res).expect("rewind response JSON");
    assert_eq!(
        rewind_json["rewound"],
        serde_json::json!(true),
        "rewind of a tree that only changed a CHECKPOINTED file must succeed \
         (the change is inside the checkpoint set, not a divergence)"
    );

    // Rewound row must land.
    let rewound_rows = ssb
        .query_by_type(SsbActionType::CheckpointToolEvent(
            CheckpointToolEventKind::Rewound,
        ))
        .expect("query Rewound rows");
    assert!(
        !rewound_rows.is_empty(),
        "CheckpointToolEvent(Rewound) row must land in SSB (R1 missing=FAIL)"
    );

    // Restored bytes must blake3-match the checkpoint pre-images.
    let restored_a = std::fs::read(proj_dir.path().join("a.rs")).expect("read a.rs post-rewind");
    let restored_b = std::fs::read(proj_dir.path().join("b.rs")).expect("read b.rs post-rewind");
    assert_eq!(
        blake3::hash(&restored_a),
        blake3::hash(original_a),
        "post-rewind a.rs must match checkpoint A pre-image"
    );
    assert_eq!(
        blake3::hash(&restored_b),
        blake3::hash(original_b),
        "post-rewind b.rs must match checkpoint A pre-image"
    );

    // The speculative bytes must SURVIVE as a fork-captured redo target in the
    // CAS (H-49 undo-then-edit=fork): materialize fork-captures the divergent
    // on-disk bytes BEFORE overwriting, so the divergence is preserved, not lost.
    let spec_hash = timetravel::blake3_hex(speculative_a);
    assert!(
        timetravel::cas_has(cas_dir.path(), session, &spec_hash),
        "the speculative pre-rewind bytes of a.rs must be fork-captured into the \
         CAS (redo-able), not destroyed by the rewind"
    );

    clear_checkpoint_env();
}

// ---------------------------------------------------------------------------
// (C3) UNRECOVERABLE REFUSAL — the load-bearing safety observable (R1, stub_ban).
//
// REFUSAL fires ONLY on a genuinely unrecoverable / unsafe pre-check condition,
// NOT on an ordinary covered-file divergence (that one is rewound + fork-captured
// by C2). Here we force the unrecoverable case: the checkpoint's CAS blob is
// removed, so the saved bytes CANNOT be restored. The tool's pre-check must
// catch this BEFORE materialize, emit a RefusedDiverged row naming the path, and
// leave the on-disk bytes untouched. A rewind that silently half-restores (or
// overwrites) when the blob is missing is the broken arm the plan FAILS.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i52_c3_rewind_refuses_on_unrecoverable() {
    let _env = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    let cas_dir = TempDir::new().expect("cas tempdir");
    let proj_dir = TempDir::new().expect("project tempdir");
    let ssb_dir = TempDir::new().expect("ssb tempdir");
    let session = "i52-c3-session";
    set_checkpoint_env(session, cas_dir.path(), proj_dir.path());

    let ssb = test_ssb(&ssb_dir, session);
    let dispatcher = PanicDispatcher;
    let clock = test_clock();

    // Save label A over a.rs.
    let original = b"fn keep() {}\n";
    std::fs::write(proj_dir.path().join("a.rs"), original).expect("write a.rs");

    let save_args = serde_json::json!({ "label": "A", "paths": ["a.rs"] });
    let save_bytes = serde_json::to_vec(&save_args).expect("encode save args");
    translate_tool_request_for_test(
        TOOL_CHECKPOINT_SAVE,
        &save_bytes,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("checkpoint_save must succeed");

    // FORCE the unrecoverable condition: remove the CAS blob that backs a.rs's
    // checkpointed bytes, so there is nothing to restore from. The CAS layout is
    // {root}/{session}/objects/{hash[..2]}/{hash}; the blob hash is the blake3 of
    // the saved bytes. After this delete, a rewind cannot restore a.rs.
    let blob_hash = timetravel::blake3_hex(original);
    let blob_path = cas_dir
        .path()
        .join(session)
        .join("objects")
        .join(&blob_hash[..2])
        .join(&blob_hash);
    assert!(blob_path.exists(), "save must have written the CAS blob");
    std::fs::remove_file(&blob_path).expect("remove CAS blob to force unrecoverable");

    // Leave a.rs on disk with sentinel bytes that MUST NOT be overwritten by the
    // refused rewind. (Bytes diverge from the checkpoint AND the blob is gone.)
    let on_disk_sentinel = b"fn keep() { /* MUST SURVIVE the refusal */ }\n";
    std::fs::write(proj_dir.path().join("a.rs"), on_disk_sentinel)
        .expect("write sentinel a.rs");

    let rewind_args = serde_json::json!({ "label": "A" });
    let rewind_bytes = serde_json::to_vec(&rewind_args).expect("encode rewind args");
    let rewind_res = translate_tool_request_for_test(
        TOOL_CHECKPOINT_REWIND,
        &rewind_bytes,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("checkpoint_rewind must return Ok (a refusal is a structured Ok, not an Err)");
    let rewind_json: serde_json::Value =
        serde_json::from_slice(&rewind_res).expect("rewind response JSON");
    assert_eq!(
        rewind_json["rewound"],
        serde_json::json!(false),
        "rewind over a diverged tree must be REFUSED, not applied"
    );
    assert_eq!(
        rewind_json["refused"],
        serde_json::json!("diverged"),
        "the refusal reason must be 'diverged'"
    );
    let diverged_list = rewind_json["diverged_paths"]
        .as_array()
        .expect("diverged_paths array");
    assert!(
        diverged_list.iter().any(|v| v == "a.rs"),
        "the diverged path 'a.rs' must be named in the refusal; got {diverged_list:?}"
    );

    // RefusedDiverged row must land (the load-bearing safety observable, R1).
    let refused_rows = ssb
        .query_by_type(SsbActionType::CheckpointToolEvent(
            CheckpointToolEventKind::RefusedDiverged,
        ))
        .expect("query RefusedDiverged rows");
    assert!(
        !refused_rows.is_empty(),
        "CheckpointToolEvent(RefusedDiverged) row MUST land (R1 missing=FAIL): the \
         divergence-refusal is the safety observable, never a silent overwrite"
    );
    let refused_payload =
        decode_ssb_payload(&refused_rows[0].payload).expect("decode RefusedDiverged payload");
    match refused_payload {
        SsbPayload::CheckpointToolEvent(p) => {
            assert_eq!(p.kind, CheckpointToolEventKind::RefusedDiverged);
            assert!(
                p.diverged_paths.iter().any(|s| s == "a.rs"),
                "payload diverged_paths must name a.rs; got {:?}",
                p.diverged_paths
            );
        }
        other => panic!("expected CheckpointToolEvent payload, got {:?}", other.action_type()),
    }

    // The on-disk bytes MUST survive (no silent overwrite / half-restore — the
    // FAILS arm). A refused rewind writes NOTHING.
    let on_disk = std::fs::read(proj_dir.path().join("a.rs")).expect("read a.rs post-refusal");
    assert_eq!(
        on_disk, on_disk_sentinel,
        "the refused rewind must NOT overwrite the file when the CAS blob is \
         missing (R2: a silent partial restore is the broken arm)"
    );

    clear_checkpoint_env();
}
