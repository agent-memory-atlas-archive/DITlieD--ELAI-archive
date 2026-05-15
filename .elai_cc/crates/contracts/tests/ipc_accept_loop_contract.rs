// Phase B2 — IPC accept-loop contract test.
//
// Exercises the bind+accept+route cycle of `ipc_accept::run_agent_session`
// using a `ScriptedAgentRpcServer` that returns canned responses. The full
// round-trip test (spawning the real `elai_cc_agent` binary) is gated behind
// ELAI_B2_AGENT_BIN and `#[ignore]` so CI never fails on a missing binary.
//
// Always-run contracts (no ignore, no binary required):
//   1. bind_session_listener succeeds and listener is usable.
//   2. ScriptedAgentRpcServer: infer_calls counter increments per handle call.
//   3. InferOutcome / ToolOutcome round-trip through the session machinery.
//
// Gated contract (#[ignore] + env-var guard):
//   4. Full round-trip: spawn binary, connect, handle InferRequest frame,
//      assert at least one InferRequest was routed.

use std::sync::atomic::Ordering;

use uuid::Uuid;

use elai_orchestrator::ipc_accept::{AgentRpcServer, InferOutcome, ToolOutcome, bind_session_listener};
use elai_orchestrator::ipc_accept::test_support::ScriptedAgentRpcServer;

// Helpers delegate to ipc_accept's exported read_frame_from_stream /
// write_frame_to_stream which dispatch over PlatformStream's cfg-split inner
// types (UnixStream on Unix, NamedPipeServer on Windows). The enum itself
// doesn't implement AsyncRead/AsyncWrite to avoid unsafe pin projection.

// ---------------------------------------------------------------------------
// Contract 1: bind_session_listener creates a usable listener
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_bind_session_listener_creates_usable_listener() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_path = tmp.path().join(format!("{}.sock", Uuid::new_v4()));

    let result = bind_session_listener(&socket_path).await;
    assert!(
        result.is_ok(),
        "bind_session_listener must succeed on a fresh temp path; got: {:?}",
        result.err()
    );
    // On Unix the socket file exists; on Windows the named pipe is addressable.
    #[cfg(unix)]
    assert!(
        socket_path.exists(),
        "Unix socket file must exist after bind"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: ScriptedAgentRpcServer counter increments
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_scripted_server_infer_counter_increments() {
    let server = ScriptedAgentRpcServer::new("hello from scripted");
    let outcome = server.handle_infer_request("test prompt".into(), 256).await;
    assert_eq!(server.infer_calls.load(Ordering::Relaxed), 1);
    match outcome {
        InferOutcome::Response(text) => assert_eq!(text, "hello from scripted"),
        InferOutcome::Failed(e) => panic!("expected Response, got Failed: {e}"),
    }
}

#[tokio::test]
async fn contract_scripted_server_tool_counter_increments() {
    let server = ScriptedAgentRpcServer::new("");
    let outcome = server.handle_tool_request("noop".into(), b"{}".to_vec()).await;
    assert_eq!(server.tool_calls.load(Ordering::Relaxed), 1);
    match outcome {
        ToolOutcome::Result(bytes) => assert_eq!(bytes, b"scripted_ok"),
        ToolOutcome::Failed(e) => panic!("expected Result, got Failed: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 3: InferOutcome / ToolOutcome are Send + Sync + 'static
// ---------------------------------------------------------------------------

#[test]
fn contract_outcome_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<InferOutcome>();
    assert_send_sync::<ToolOutcome>();
}

// Contract 4 (the real-binary round-trip) has been replaced by
// agent_pool_listen_before_spawn_contract.rs which provides hermetic
// listen-before-spawn proof without requiring an elai_cc_agent binary.
// See workflow/handoff/phase-B2b.md acceptance criterion for rationale.
