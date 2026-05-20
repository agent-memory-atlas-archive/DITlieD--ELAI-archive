//! D-18 — observable_effect smoke integration test.
//!
//! The orchestrator brief asks for an "observable_effect FOR
//! observable_effect" — three mock effects, each demonstrating one
//! canonical (TriggerSpec, TypedObservation) pairing, must all return
//! `VerificationOutcome::Pass`, MUST each emit an
//! `observable_effect_verified` SSB row, and the read-side
//! `recent_observable_effects` projection MUST surface all three
//! rows.
//!
//! No live executor / network / filesystem dependency — every effect
//! runs against [`MockSandboxExecutor`] with pre-loaded captures.
//!
//! Producer: `elai_orchestrator::observable_effect::verify_observable_effect`.
//! Consumer (read): `elai_orchestrator::observable_effect::recent_observable_effects`
//! (and downstream `crates/dashboard-v2/src-tauri/src/commands_d18.rs`).
//!
//! Grounding: see crates/orchestrator/src/observable_effect.rs
//! TODO_DOC[D-18] — the brief's arXiv:2412.16367 citation was rejected
//! (WebSearch May 2026 confirmed it is unrelated battery-thermal-runaway
//! work). Paper-ingestor follow-up should anchor against Voyager
//! arxiv:2305.16291 §3 + Coscientist arxiv:2304.05332 §4.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::SeededClock;
use elai_core::ClockProvider;

use elai_orchestrator::observable_effect::{
    recent_observable_effects, verify_observable_effect, CapturedObservation, ConfoundFlags,
    MockSandboxExecutor, ObservableEffect, SandboxTier, TriggerSpec, TypedObservation,
    VerificationOutcome,
};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbReader};
use elai_orchestrator::ssb_viewer::SsbViewer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn three_orthogonal_effects_all_pass_and_emit_ssb_rows() {
    // ----- Ssb wired against a fresh tempdir-backed db ----------------
    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_observable_effect_smoke.db"),
        pool_size: Some(2),
        project_id: "test-project-d18".into(),
        session_id: "test-session-d18".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let ssb = Arc::new(Ssb::new(cfg.clone(), clock).expect("ssb init"));

    // ----- Mock executor with three fixtures --------------------------
    let mut mock = MockSandboxExecutor::new();
    mock.insert(
        "cargo test --quiet",
        "test_command",
        CapturedObservation::Stdout("running 7 tests ... 7 passed".into()),
    );
    let present_path = PathBuf::from("/sentinel/exists.txt");
    mock.insert(
        present_path.display().to_string(),
        "file_presence",
        CapturedObservation::FilePresent(true),
    );
    mock.insert(
        "sentinel-substring",
        "stdout_contains",
        CapturedObservation::Stdout("preamble sentinel-substring postamble".into()),
    );

    // ----- Three effects, three orthogonal trigger kinds --------------
    let effect_a = ObservableEffect {
        effect_id: "smoke-a".into(),
        trigger: TriggerSpec::TestCommand("cargo test --quiet".into()),
        expected: TypedObservation::StringMatch("7 passed".into()),
        sandbox: SandboxTier::ProcessIsolated,
        confound_flags: ConfoundFlags::default(),
    };
    let effect_b = ObservableEffect {
        effect_id: "smoke-b".into(),
        trigger: TriggerSpec::FilePresence(present_path),
        expected: TypedObservation::StringMatch("present".into()),
        sandbox: SandboxTier::ContainerIsolated,
        confound_flags: ConfoundFlags {
            environment_dependent: true,
            ..ConfoundFlags::default()
        },
    };
    let effect_c = ObservableEffect {
        effect_id: "smoke-c".into(),
        trigger: TriggerSpec::StdoutContains("sentinel-substring".into()),
        expected: TypedObservation::StringMatch("sentinel-substring".into()),
        sandbox: SandboxTier::NoSandbox,
        confound_flags: ConfoundFlags {
            time_dependent: true,
            nondeterministic: true,
            ..ConfoundFlags::default()
        },
    };

    // ----- Verify all three -------------------------------------------
    let now_ms = 1_700_000_200_000;
    let oa = verify_observable_effect(&effect_a, &mock, &ssb, "agent-d18", 0, now_ms)
        .await
        .expect("verify A");
    let ob = verify_observable_effect(&effect_b, &mock, &ssb, "agent-d18", 0, now_ms + 1)
        .await
        .expect("verify B");
    let oc = verify_observable_effect(&effect_c, &mock, &ssb, "agent-d18", 0, now_ms + 2)
        .await
        .expect("verify C");
    assert_eq!(oa, VerificationOutcome::Pass);
    assert_eq!(ob, VerificationOutcome::Pass);
    assert_eq!(oc, VerificationOutcome::Pass);

    // ----- SSB direct-query: exactly 3 ObservableEffectVerified rows --
    let rows = ssb
        .query_by_type(SsbActionType::ObservableEffectVerified)
        .expect("query observable_effect_verified rows");
    assert_eq!(
        rows.len(),
        3,
        "exactly one observable_effect_verified row per verify"
    );

    // ----- Force pool flush + reopen via SsbViewer (mirrors dashboard) -
    drop(ssb);
    let viewer = SsbViewer::open(&cfg.db_path).expect("ssb viewer open");
    let surfaced = recent_observable_effects(&viewer, 50);
    assert_eq!(
        surfaced.len(),
        3,
        "recent_observable_effects must surface all three rows"
    );
    let ids: Vec<&str> = surfaced.iter().map(|r| r.effect_id.as_str()).collect();
    assert!(ids.contains(&"smoke-a"), "smoke-a missing in surfaced rows");
    assert!(ids.contains(&"smoke-b"), "smoke-b missing in surfaced rows");
    assert!(ids.contains(&"smoke-c"), "smoke-c missing in surfaced rows");

    for row in &surfaced {
        assert_eq!(row.outcome, "pass", "{}: outcome must be pass", row.effect_id);
    }

    // ----- Confound-flag round-trip preserved through SSB → viewer ----
    let by_id = |id: &str| surfaced.iter().find(|r| r.effect_id == id).unwrap();
    assert_eq!(by_id("smoke-a").confound_flags, "");
    assert_eq!(by_id("smoke-b").confound_flags, "environment_dependent");
    assert_eq!(
        by_id("smoke-c").confound_flags,
        "time_dependent,nondeterministic"
    );

    // ----- Sandbox-tier round-trip ------------------------------------
    assert_eq!(by_id("smoke-a").sandbox_tier, "process_isolated");
    assert_eq!(by_id("smoke-b").sandbox_tier, "container_isolated");
    assert_eq!(by_id("smoke-c").sandbox_tier, "no_sandbox");

    // ----- Trigger-kind round-trip ------------------------------------
    assert_eq!(by_id("smoke-a").trigger_kind, "test_command");
    assert_eq!(by_id("smoke-b").trigger_kind, "file_presence");
    assert_eq!(by_id("smoke-c").trigger_kind, "stdout_contains");
}
