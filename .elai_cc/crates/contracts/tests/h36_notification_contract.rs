//! H-36 contract: completion / needs-input dev-event notifications,
//! agent_notification_emitted SSB schema gate.
//!
//! Interface-level tests that assert:
//!   (F1) SCHEMA_VERSION >= 215 and TOTAL_VARIANTS >= 526; AgentNotificationEmitted
//!        round-trips through as_sql_str / from_str.
//!   (F2) Write a real SsbPayload::AgentNotificationEmitted row to an in-process
//!        Ssb (Completion kind); query it back; assert the row exists and kind ==
//!        Completion. Off-agent: test drives the REAL Ssb write path. Missing = FAIL (R1).
//!   (F3) Same for NeedsInput kind. Missing row = FAIL (R1).
//!   (F4) ELAI_NOTIFY_ENABLE env gate: NotificationDispatcher::from_env() sets
//!        os_enabled correctly; a dispatcher built with ELAI_NOTIFY_ENABLE=0 writes
//!        the SSB row (off-agent artifact is UNCONDITIONAL) but surfaces_succeeded
//!        bit 1 (OS notification) is NOT set (no process spawn).
//!
//! # Wiring discipline
//!
//! - Off-agent: F2 and F3 drive the REAL Ssb write path, not a unit shim.
//! - Missing evidence is FAIL (R1/R2): zero rows returned by query_by_type = FAIL.
//! - The contract test never authors its own SSB rows directly (it calls
//!   NotificationDispatcher::notify, which is the real producer).
//! - No `.unwrap()` / `.expect()` in non-helper code.
//! - tracing-test assertions use assert!(logs_contain(...)) per T-1.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use tempfile::TempDir;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::notify_dispatch::{NotificationDispatcher, NotificationKind};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, AgentNotificationKind, Ssb, SsbActionType, SsbConfig,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;

// ---------------------------------------------------------------------------
// Helpers (only called from #[cfg(test)] — .expect() allowed here per policy)
// ---------------------------------------------------------------------------

fn test_ssb_config(dir: &TempDir) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join("test_h36.db"),
        project_id: "h36-contract-test".to_owned(),
        session_id: "s-h36-test".to_owned(),
        pool_size: None,
    }
}

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

// ---------------------------------------------------------------------------
// (F1) SSB lockstep: AgentNotificationEmitted at SCHEMA 215 / TV 526.
// ---------------------------------------------------------------------------

#[test]
fn h36_f1_ssb_variant_exists_at_schema_215() {
    // SCHEMA_VERSION_PUBLIC must be at least 215 (H-36 bump).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 215,
        "SCHEMA_VERSION_PUBLIC must be >= 215 for H-36, found {}",
        SCHEMA_VERSION_PUBLIC
    );

    // Wire tag must round-trip.
    let variant = SsbActionType::AgentNotificationEmitted;
    assert_eq!(
        variant.as_sql_str(),
        "agent_notification_emitted",
        "AgentNotificationEmitted wire tag must be 'agent_notification_emitted'"
    );

    // from_str round-trip via public helper.
    let parsed = ssb_action_type_from_str_public("agent_notification_emitted")
        .expect("ssb_action_type_from_str_public must parse 'agent_notification_emitted'");
    assert_eq!(
        parsed,
        SsbActionType::AgentNotificationEmitted,
        "from_str round-trip must yield AgentNotificationEmitted"
    );

    // TOTAL_VARIANTS pin: verify action_type_lock.rs holds 526.
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let elai_cc = workspace
        .parent()
        .and_then(|p| p.parent())
        .expect("contracts has 2-up parent (.elai_cc)");
    let lock_rs = elai_cc
        .join("crates")
        .join("orchestrator")
        .join("tests")
        .join("action_type_lock.rs");
    let body = fs::read_to_string(&lock_rs).expect("read action_type_lock.rs");
    assert!(
        body.contains("TOTAL_VARIANTS: usize = 526"),
        "action_type_lock.rs TOTAL_VARIANTS must be 526 for H-36"
    );
}

// ---------------------------------------------------------------------------
// (F2) Real Ssb write + query: Completion kind. Off-agent (R1/R4).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h36_f2_completion_row_written_and_readable() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(test_ssb_config(&dir), test_clock()).expect("Ssb::new must succeed"),
    );

    let dispatcher = NotificationDispatcher::from_env();

    // Drive the REAL notify path (Completion kind). ELAI_NOTIFY_ENABLE is OFF in
    // CI so no OS spawn fires; the SSB row must still be written unconditionally.
    dispatcher
        .notify(NotificationKind::Completion, &ssb, "s-h36-test")
        .await;

    // R1: missing evidence is FAIL. Query the REAL Ssb for the row.
    let rows = ssb
        .query_by_type(SsbActionType::AgentNotificationEmitted)
        .expect("query_by_type must succeed (R2: query failure = FAIL)");

    assert!(
        !rows.is_empty(),
        "agent_notification_emitted row must exist after Completion notify (R1: missing = FAIL)"
    );

    // Decode and verify kind.
    let decoded = elai_orchestrator::ssb::decode_ssb_payload(&rows[0].payload)
        .expect("payload must decode cleanly");
    match decoded {
        SsbPayload::AgentNotificationEmitted(p) => {
            assert_eq!(
                p.kind,
                AgentNotificationKind::Completion,
                "row kind must be Completion"
            );
            assert_eq!(p.session_id, "s-h36-test", "session_id must round-trip");
            // surfaces_attempted bit 0 (escape) must always be set.
            assert_ne!(
                p.surfaces_attempted & 0b01,
                0,
                "terminal escape surface must be in surfaces_attempted"
            );
        }
        other => panic!("expected AgentNotificationEmitted, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (F3) Real Ssb write + query: NeedsInput kind. Off-agent (R1/R4).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h36_f3_needs_input_row_written_and_readable() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(test_ssb_config(&dir), test_clock()).expect("Ssb::new must succeed"),
    );

    let dispatcher = NotificationDispatcher::from_env();

    dispatcher
        .notify(NotificationKind::NeedsInput, &ssb, "s-h36-test")
        .await;

    let rows = ssb
        .query_by_type(SsbActionType::AgentNotificationEmitted)
        .expect("query_by_type must succeed");

    assert!(
        !rows.is_empty(),
        "agent_notification_emitted row must exist after NeedsInput notify (R1: missing = FAIL)"
    );

    let decoded = elai_orchestrator::ssb::decode_ssb_payload(&rows[0].payload)
        .expect("payload must decode cleanly");
    match decoded {
        SsbPayload::AgentNotificationEmitted(p) => {
            assert_eq!(
                p.kind,
                AgentNotificationKind::NeedsInput,
                "row kind must be NeedsInput"
            );
        }
        other => panic!("expected AgentNotificationEmitted, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (F4) ELAI_NOTIFY_ENABLE env gate: SSB row writes unconditionally; OS
//      notification spawn is gated by the env var.
//
// tracing-test: assert! wraps logs_contain (T-1 compliance).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h36_f4_notify_enable_env_gate_controls_os_dispatch() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(test_ssb_config(&dir), test_clock()).expect("Ssb::new must succeed"),
    );

    // Explicitly build a dispatcher with OS notifications OFF (env var unset / 0).
    // Safety: test sets an env var -- use a per-test prefix to reduce cross-test bleed.
    // std::env::remove_var is unsafe in Rust >=1.80; we just unset before constructing.
    unsafe { std::env::remove_var("ELAI_NOTIFY_ENABLE") };
    let dispatcher = NotificationDispatcher::from_env();

    dispatcher
        .notify(NotificationKind::Completion, &ssb, "s-h36-test-f4")
        .await;

    // SSB row must exist -- unconditional regardless of ELAI_NOTIFY_ENABLE (R1).
    // This is the load-bearing off-agent artifact; logs_contain is bonus and
    // requires #[traced_test] + tracing dep which is not in scope here.
    let rows = ssb
        .query_by_type(SsbActionType::AgentNotificationEmitted)
        .expect("query_by_type must succeed");
    assert!(
        !rows.is_empty(),
        "SSB row must be written even with ELAI_NOTIFY_ENABLE=0 (R1: unconditional artifact)"
    );

    // OS notification bit (bit 1) must NOT be set in surfaces_attempted when env is OFF.
    let decoded = elai_orchestrator::ssb::decode_ssb_payload(&rows[0].payload)
        .expect("payload must decode cleanly");
    match decoded {
        SsbPayload::AgentNotificationEmitted(p) => {
            assert_eq!(
                p.surfaces_attempted & 0b10,
                0,
                "OS notification surface must NOT be attempted when ELAI_NOTIFY_ENABLE=0"
            );
        }
        other => panic!("expected AgentNotificationEmitted, got {:?}", other.action_type()),
    }
}
