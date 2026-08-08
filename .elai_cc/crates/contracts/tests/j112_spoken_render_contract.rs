//! J-112 contract: deterministic spoken notification rendering + dispatcher wiring.

use std::sync::Arc;

use tempfile::TempDir;

use elai_core::clock::OsClock;
use elai_orchestrator::notify_dispatch::{NotificationDispatcher, NotificationKind};
use elai_orchestrator::ssb::{
    AgentNotificationKind, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::voice::spoken_render::render_spoken;
use elai_orchestrator::SsbReader;  // REQUIRED: query_by_type is a trait method

fn make_ssb(dir: &TempDir) -> Arc<Ssb> {
    Arc::new(Ssb::new(
        SsbConfig {
            db_path: dir.path().join("test_j112.db"),
            project_id: "j112-contract".to_owned(),
            session_id: "s-j112".to_owned(),
            pool_size: None,
        },
        Arc::new(OsClock),
    ).expect("Ssb::new"))
}

fn surfaces(ssb: &Ssb) -> (u8, u8) {
    let rows = ssb
        .query_by_type(SsbActionType::AgentNotificationEmitted)
        .expect("query_by_type must succeed (R2)");
    assert!(!rows.is_empty(), "notify must write an SSB row (R1)");
    match elai_orchestrator::ssb::decode_ssb_payload(&rows[0].payload).expect("decode payload") {
        SsbPayload::AgentNotificationEmitted(p) => (p.surfaces_attempted, p.surfaces_succeeded),
        other => panic!("expected notification payload, got {:?}", other.action_type()),
    }
}

#[test]
fn j112_f1_render_is_byte_equal() {
    for kind in [AgentNotificationKind::Completion, AgentNotificationKind::NeedsInput] {
        let a = render_spoken(kind, "session-j112", 1_723_456_789);
        let b = render_spoken(kind, "session-j112", 1_723_456_789);
        assert_eq!(a.as_bytes(), b.as_bytes());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn j112_f2_voice_gate_on_off_via_env() {
    // ON: activate via the REAL production path (env gate), no test-only builder.
    std::env::set_var("ELAI_VOICE_ENABLE", "1");
    let dir_on = TempDir::new().expect("tempdir");
    let ssb_on = make_ssb(&dir_on);
    NotificationDispatcher::from_env()
        .notify(NotificationKind::Completion, &ssb_on, "s-j112-f2-on")
        .await;
    let (attempted_on, succeeded_on) = surfaces(&ssb_on);
    assert_ne!(attempted_on & 0b100, 0, "voice ON: surfaces_attempted bit 2 must be set");
    assert_ne!(succeeded_on & 0b100, 0, "voice ON: surfaces_succeeded bit 2 must be set");

    // OFF: clear env, re-build from_env, same notification -> bit 2 clear (R9 default).
    std::env::remove_var("ELAI_VOICE_ENABLE");
    let dir_off = TempDir::new().expect("tempdir");
    let ssb_off = make_ssb(&dir_off);
    NotificationDispatcher::from_env()
        .notify(NotificationKind::Completion, &ssb_off, "s-j112-f2-off")
        .await;
    let (attempted_off, _) = surfaces(&ssb_off);
    assert_eq!(attempted_off & 0b100, 0, "voice OFF (default): bit 2 must be clear");
}

#[test]
fn j112_f4_render_distinctness() {
    let completion = render_spoken(AgentNotificationKind::Completion, "same", 42);
    let needs_input = render_spoken(AgentNotificationKind::NeedsInput, "same", 42);
    assert_ne!(completion, needs_input);
}

#[test]
fn j112_f5_render_quarantines_entropy() {
    let first = render_spoken(AgentNotificationKind::NeedsInput, "stable", 99);
    for _ in 0..100 {
        assert_eq!(first, render_spoken(AgentNotificationKind::NeedsInput, "stable", 99));
    }
}
