//! J-25 T5 slice C — writer coalescing + fork snapshot contract (source-wiring gate).
//!
//! Verifies the structural + behavioral claims from the J-25 WIRING_CONTRACT
//! for the T5 writer-coalescing/snapshot layer shipped in handoff_digest.rs:
//!
//! D7a: `handoff_digest.rs` exports the new coalescing types
//!      (`DigestWriterQueue`, `PendingDigestRequest`, `DigestWriterSnapshot`)
//!      and the `author_from_pending` fn; `lib.rs` re-exports them so they are
//!      reachable as `elai_orchestrator::...` (source-grep + type-presence).
//! D7b: production entry — `live_cli.rs` Drop path source-greps
//!      `author_from_pending` OR `DigestWriterQueue` (R7 production caller).
//! D7c: behavioral — 1-slot newest-wins: `request(A); request(B); take()`
//!      yields `B` (B.session_id), not A.
//! D7d: behavioral — snapshot frozen: after `request(req)`, mutating a clone
//!      of the original snapshot inputs does not change what `take()` returns
//!      (the snapshot inside the kept `PendingDigestRequest` is independent).
//! D7e: ABORT-1 — `author_from_pending` source contains a call to
//!      `author_topic_digest` (source-grep); no second body-builder fn name.
//! D7f: HD-2 — `request` replaces the slot; no API mutates
//!      `PendingDigestRequest.snapshot` in place after insert (source-grep
//!      absence of snapshot field writes outside struct init).
//!
//! The NEGATIVE leg (author_from_pending never called from production => DARK)
//! is locked by D7b together with the auditor's `production_caller_check.py`
//! run over the full T5 changeset. This slice locks the structural exports +
//! the deterministic behavior of the 1-slot queue + snapshot freeze.

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// D7a: handoff_digest.rs exports the coalescing types + author_from_pending;
//      lib.rs re-exports them.
// ---------------------------------------------------------------------------

/// D7a-source: handoff_digest.rs contains `pub struct DigestWriterQueue`,
/// `pub struct PendingDigestRequest`, `pub struct DigestWriterSnapshot`, and
/// `pub fn author_from_pending`.
#[test]
fn d7a_j25_writer_coalesce_types_present_in_handoff_digest() {
    let src = read_src("orchestrator/src/handoff_digest.rs");
    assert!(
        src.contains("pub struct DigestWriterSnapshot"),
        "handoff_digest.rs must export pub struct DigestWriterSnapshot"
    );
    assert!(
        src.contains("pub struct PendingDigestRequest"),
        "handoff_digest.rs must export pub struct PendingDigestRequest"
    );
    assert!(
        src.contains("pub struct DigestWriterQueue"),
        "handoff_digest.rs must export pub struct DigestWriterQueue"
    );
    assert!(
        src.contains("pub fn author_from_pending"),
        "handoff_digest.rs must export pub fn author_from_pending"
    );
}

/// D7a-reexport: lib.rs `pub use handoff_digest::{...}` block lists all four
/// new pub symbols so they are reachable as `elai_orchestrator::...`.
#[test]
fn d7a_j25_writer_coalesce_reexported_from_lib() {
    let src = read_src("orchestrator/src/lib.rs");
    let block_start = src.find("pub use handoff_digest::{");
    assert!(
        block_start.is_some(),
        "lib.rs must have a handoff_digest pub use block"
    );
    let block = &src[block_start.unwrap()..];
    let block_end = block.find("};").expect("pub use block terminates");
    let block = &block[..block_end];
    assert!(
        block.contains("DigestWriterSnapshot"),
        "lib.rs must re-export DigestWriterSnapshot"
    );
    assert!(
        block.contains("PendingDigestRequest"),
        "lib.rs must re-export PendingDigestRequest"
    );
    assert!(
        block.contains("DigestWriterQueue"),
        "lib.rs must re-export DigestWriterQueue"
    );
    assert!(
        block.contains("author_from_pending"),
        "lib.rs must re-export author_from_pending"
    );
}

/// D7a-presence: the new pub types are reachable from the
/// `elai_orchestrator` crate root (type-presence check via the re-export).
#[test]
fn d7a_j25_writer_coalesce_types_reachable_from_orchestrator() {
    // Type-presence: the symbols resolve through the lib.rs re-export.  We
    // do not construct an Ssb here (D7c/D7d exercise the queue logic
    // directly); this test only binds the public surface.
    let _ = std::any::TypeId::of::<elai_orchestrator::DigestWriterSnapshot>();
    let _ = std::any::TypeId::of::<elai_orchestrator::PendingDigestRequest>();
    let _ = std::any::TypeId::of::<elai_orchestrator::DigestWriterQueue>();
}

// ---------------------------------------------------------------------------
// D7b: production entry — live_cli.rs Drop path source-greps the new symbols
// ---------------------------------------------------------------------------

/// D7b: live_cli.rs contains a call to `author_from_pending` OR
/// `DigestWriterQueue` inside its Drop path (R7 production caller). The
/// source-grep is include_str!-shaped over the live_cli.rs source so the
/// contract does not need to link the CLI crate.
#[test]
fn d7b_j25_production_drop_path_calls_writer_coalesce() {
    let cli_src = read_src("cli/src/live_cli.rs");
    assert!(
        cli_src.contains("impl Drop for LiveCli"),
        "live_cli.rs must define impl Drop for LiveCli (production entrypoint)"
    );
    let drop_start = cli_src
        .find("impl Drop for LiveCli")
        .expect("impl Drop for LiveCli present");
    let drop_end = cli_src[drop_start..]
        .find("\n}\n")
        .map(|i| drop_start + i + 3)
        .unwrap_or(cli_src.len());
    let drop_block = &cli_src[drop_start..drop_end];
    assert!(
        drop_block.contains("author_from_pending")
            || drop_block.contains("DigestWriterQueue"),
        "Drop impl must call author_from_pending or DigestWriterQueue (production wiring); block was: {drop_block}"
    );
}

// ---------------------------------------------------------------------------
// D7c: behavioral — 1-slot newest-wins
// ---------------------------------------------------------------------------

/// D7c: `request(A); request(B); take()` yields `B` (B.session_id), not A.
/// The 1-slot queue evicts the older pending request in-place.
#[test]
fn d7c_j25_queue_newest_wins_evicts_older() {
    use elai_orchestrator::{DigestWriterQueue, DigestWriterSnapshot, PendingDigestRequest};
    use std::path::PathBuf;

    let snap = DigestWriterSnapshot {
        system_parts: vec!["sys".to_string()],
        tool_defs: vec!["tool".to_string()],
        watermark_msg_id: None,
    };
    let req_a = PendingDigestRequest {
        snapshot: snap.clone(),
        session_id: "sess-A".to_string(),
        cwd: PathBuf::from("/tmp/a"),
    };
    let req_b = PendingDigestRequest {
        snapshot: snap,
        session_id: "sess-B".to_string(),
        cwd: PathBuf::from("/tmp/b"),
    };

    let queue = DigestWriterQueue::new();
    queue.request(req_a);
    queue.request(req_b);

    let taken = queue
        .take()
        .expect("take() must yield the newest pending request after two enqueues");
    assert_eq!(
        taken.session_id, "sess-B",
        "newest-wins: take() must yield sess-B, not sess-A"
    );

    // The slot is drained; a second take yields None.
    assert!(
        queue.take().is_none(),
        "take() after drain must yield None (1-slot queue)"
    );
}

// ---------------------------------------------------------------------------
// D7d: behavioral — snapshot frozen at request() time
// ---------------------------------------------------------------------------

/// D7d: after `request(req)`, mutating a clone of the original snapshot
/// inputs does not change what `take()` returns. The snapshot inside the
/// kept `PendingDigestRequest` is independent of caller-side mutations.
#[test]
fn d7d_j25_snapshot_frozen_at_request_time() {
    use elai_orchestrator::{DigestWriterQueue, DigestWriterSnapshot, PendingDigestRequest};
    use std::path::PathBuf;

    let mut snap = DigestWriterSnapshot {
        system_parts: vec!["orig-system".to_string()],
        tool_defs: vec!["orig-tool".to_string()],
        watermark_msg_id: Some("wm-0".to_string()),
    };
    let req = PendingDigestRequest {
        snapshot: snap.clone(),
        session_id: "sess-frozen".to_string(),
        cwd: PathBuf::from("/tmp/frozen"),
    };

    let queue = DigestWriterQueue::new();
    queue.request(req);

    // Mutate the caller-side clone AFTER the request was enqueued.
    snap.system_parts.push("post-enqueue-mutation".to_string());
    snap.tool_defs.clear();
    snap.watermark_msg_id = Some("wm-mutated".to_string());

    let taken = queue
        .take()
        .expect("take() must yield the enqueued request with the frozen snapshot");
    assert_eq!(
        taken.snapshot.system_parts,
        vec!["orig-system".to_string()],
        "frozen snapshot must not reflect post-enqueue mutation of system_parts"
    );
    assert_eq!(
        taken.snapshot.tool_defs,
        vec!["orig-tool".to_string()],
        "frozen snapshot must not reflect post-enqueue mutation of tool_defs"
    );
    assert_eq!(
        taken.snapshot.watermark_msg_id,
        Some("wm-0".to_string()),
        "frozen snapshot must not reflect post-enqueue mutation of watermark_msg_id"
    );
    assert_eq!(
        taken.session_id, "sess-frozen",
        "frozen request identity must be preserved"
    );
}

// ---------------------------------------------------------------------------
// D7e: ABORT-1 — author_from_pending delegates to author_topic_digest
// ---------------------------------------------------------------------------

/// D7e: `author_from_pending` source contains a call to
/// `author_topic_digest` (ABORT-1: no second digest body builder). The
/// source-grep is over the `author_from_pending` fn body only.
#[test]
fn d7e_j25_author_from_pending_delegates_to_author_topic_digest() {
    let src = read_src("orchestrator/src/handoff_digest.rs");
    let fn_start = src
        .find("pub fn author_from_pending")
        .expect("author_from_pending fn present");
    let fn_end = src[fn_start..]
        .find("\n}\n")
        .map(|i| fn_start + i + 3)
        .unwrap_or(src.len());
    let body = &src[fn_start..fn_end];
    assert!(
        body.contains("author_topic_digest("),
        "author_from_pending must call author_topic_digest (ABORT-1: no second body builder); body was: {body}"
    );
    // ABORT-1 negative leg: no second body-builder fn name inside
    // author_from_pending. The existing private body builder is
    // `build_digest_body`; author_from_pending must NOT call it directly (it
    // delegates via author_topic_digest, which owns that call).
    assert!(
        !body.contains("build_digest_body("),
        "author_from_pending must not call build_digest_body directly (ABORT-1: single body builder path); body was: {body}"
    );
}

// ---------------------------------------------------------------------------
// D7f: HD-2 — request replaces slot; no in-place snapshot mutation after insert
// ---------------------------------------------------------------------------

/// D7f: the `request` fn replaces the pending slot entirely (`*slot = Some(req)`)
/// and there is no API that mutates `PendingDigestRequest.snapshot` fields in
/// place after insert. Source-grep absence of snapshot field writes outside
/// struct-init sites.
#[test]
fn d7f_j25_request_replaces_slot_no_inplace_snapshot_mutation() {
    let src = read_src("orchestrator/src/handoff_digest.rs");

    // The request fn body must replace the slot (not mutate fields of the
    // existing occupant).
    let req_fn_start = src
        .find("pub fn request(")
        .expect("DigestWriterQueue::request fn present");
    let req_fn_end = src[req_fn_start..]
        .find("\n    }\n")
        .map(|i| req_fn_start + i + 6)
        .unwrap_or(src.len());
    let req_body = &src[req_fn_start..req_fn_end];
    assert!(
        req_body.contains("*slot = Some(req)"),
        "request must replace the slot entirely (*slot = Some(req)); body was: {req_body}"
    );

    // HD-2 negative leg: no method on DigestWriterQueue that writes a field
    // of PendingDigestRequest.snapshot after insert. We grep for any
    // `.snapshot.` field-write outside a struct-init site. The only
    // occurrences of `.snapshot` in the coalescing layer should be reads
    // (e.g. `let _ = &req.snapshot;`) or struct-init (`snapshot: ...` in a
    // PendingDigestRequest literal). A write like `.snapshot.system_parts =
    // ...` or `.snapshot = ...` outside struct init is a HD-2 violation.
    let coalesce_start = src
        .find("Writer coalescing + fork snapshot (J-25 MOVE T5)")
        .expect("T5 coalescing section header present");
    let coalesce_end = src[coalesce_start..]
        .find("\n// ---------------------------------------------------------------------------\n// Tests")
        .map(|i| coalesce_start + i)
        .unwrap_or(src.len());
    let coalesce = &src[coalesce_start..coalesce_end];
    // No in-place write to `.snapshot` or its fields. The pattern we forbid:
    //   `.snapshot =` (assignment to the field outside struct init)
    //   `.snapshot.system_parts =`
    //   `.snapshot.tool_defs =`
    //   `.snapshot.watermark_msg_id =`
    // Struct-init uses `snapshot: <expr>,` (with a colon), which the
    // forbidden patterns do not match (they require `.snapshot`).
    let forbidden = [
        ".snapshot =",
        ".snapshot.system_parts =",
        ".snapshot.tool_defs =",
        ".snapshot.watermark_msg_id =",
    ];
    for pat in &forbidden {
        assert!(
            !coalesce.contains(pat),
            "HD-2 violation: coalescing layer must not mutate PendingDigestRequest.snapshot in place (found `{pat}`)"
        );
    }
}
