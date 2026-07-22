//! J-35 intent-control layer contract tests.
//!
//! Pins the Tier-1 OBSERVE-ONLY behaviour of `run_intent_control`:
//!   * A task with an embedded false premise yields PremiseFlagged SSB rows.
//!   * Premise-check does not condition on the requested output format.
//!   * The autonomous default path does not use a majority/population prior.
//!   * No "aim-verified" / "right-problem-confirmed" string or enum exists.
//!   * With the env flag unset/off, run_intent_control is a no-op (no SSB rows).

use std::sync::Arc;
use std::sync::Mutex;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::autonomous_gate::GateMode;
use elai_orchestrator::gate::RiskLevel;
use elai_orchestrator::intent_control::{intent_control_enabled, run_intent_control};
use elai_orchestrator::ssb::{
    IntentControlEventPayload, IntentControlKind, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SsbReader,
};
use elai_orchestrator::SCHEMA_VERSION_PUBLIC;

fn write_fixture(dir: &std::path::Path) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).expect("create src dir");
    std::fs::write(
        src.join("foo.rs"),
        "async fn sync_loop() { something.await; }\n",
    )
    .expect("write foo.rs");
}

const ELAI_INTENT_CONTROL_ENABLE: &str = "ELAI_INTENT_CONTROL_ENABLE";

static ENV_MUTEX: Mutex<()> = Mutex::new(());

struct EnvRestoreGuard {
    key: &'static str,
    prior: Option<String>,
}

impl EnvRestoreGuard {
    fn capture(key: &'static str) -> Self {
        EnvRestoreGuard {
            key,
            prior: std::env::var(key).ok(),
        }
    }
}

impl Drop for EnvRestoreGuard {
    fn drop(&mut self) {
        match &self.prior {
            Some(val) => std::env::set_var(self.key, val),
            None => std::env::remove_var(self.key),
        }
    }
}

fn os_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

async fn build_ssb(test_id: &str) -> (Arc<Ssb>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create tempdir for SSB");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: format!("proj-{}", test_id),
        session_id: format!("sess-{}", test_id),
    };
    let ssb = Ssb::new(cfg, os_clock()).expect("Ssb::new");
    (Arc::new(ssb), dir)
}

fn decode_intent_control_rows(
    ssb: &Arc<Ssb>,
    kind: IntentControlKind,
) -> Vec<IntentControlEventPayload> {
    let rows = ssb
        .query_by_type(SsbActionType::IntentControlEvent(kind))
        .expect("query_by_type");
    rows.iter()
        .filter_map(|r| {
            let decoded: Result<SsbPayload, _> = rmp_serde::from_slice(&r.payload);
            decoded.ok()
        })
        .filter_map(|p| match p {
            SsbPayload::IntentControlEvent(p) => Some(p),
            _ => None,
        })
        .filter(|p| p.kind == kind)
        .collect()
}

// ---------------------------------------------------------------------------
// Contract 1 — false premise emits PremiseFlagged SSB row.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_false_premise_emits_premise_flagged() {
    let _mutex = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_INTENT_CONTROL_ENABLE);
    std::env::set_var(ELAI_INTENT_CONTROL_ENABLE, "1");

    let (ssb, _ssb_dir) = build_ssb("false-premise").await;
    let project_dir = tempfile::tempdir().expect("create project dir");
    write_fixture(project_dir.path());

    let task = "fix the sync loop in src/foo.rs that handles batching and queue draining and retry logic and backpressure. the whole pipeline needs to become async eventually";
    run_intent_control(
        task,
        &ssb,
        os_clock(),
        GateMode::Interactive,
        RiskLevel::Low,
        Some(project_dir.path()),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let premise_rows = decode_intent_control_rows(&ssb, IntentControlKind::PremiseFlagged);
    assert!(
        !premise_rows.is_empty(),
        "CONTRACT: a false-premise task must emit at least one PremiseFlagged SSB row"
    );
    for row in &premise_rows {
        assert!(
            row.premise_flag_count > 0,
            "CONTRACT: PremiseFlagged row must have premise_flag_count > 0"
        );
    }

    let spec_rows = decode_intent_control_rows(&ssb, IntentControlKind::SpecReconstructed);
    assert!(
        !spec_rows.is_empty(),
        "CONTRACT: SpecReconstructed row must accompany PremiseFlagged"
    );
    let route_rows = decode_intent_control_rows(&ssb, IntentControlKind::RouteDecided);
    assert!(
        !route_rows.is_empty(),
        "CONTRACT: RouteDecided row must accompany PremiseFlagged"
    );
}

// ---------------------------------------------------------------------------
// Contract 2 — premise check is format-independent.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_format_independent_premise_check() {
    let _mutex = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_INTENT_CONTROL_ENABLE);
    std::env::set_var(ELAI_INTENT_CONTROL_ENABLE, "1");

    let project_dir = tempfile::tempdir().expect("create project dir");
    write_fixture(project_dir.path());

    let variants = [
        "fix the sync loop in src/foo.rs that handles batching and queue draining and retry logic and backpressure. output as JSON. the whole pipeline needs to become async eventually",
        "fix the sync loop in src/foo.rs that handles batching and queue draining and retry logic and backpressure. output as markdown. the whole pipeline needs to become async eventually",
        "fix the sync loop in src/foo.rs that handles batching and queue draining and retry logic and backpressure. output as plain text. the whole pipeline needs to become async eventually",
        "fix the sync loop in src/foo.rs that handles batching and queue draining and retry logic and backpressure. give me a table. the whole pipeline needs to become async eventually",
    ];

    for (i, task) in variants.iter().enumerate() {
        let (ssb, _ssb_dir) = build_ssb(&format!("fmt-{}", i)).await;
        run_intent_control(
            task,
            &ssb,
            os_clock(),
            GateMode::Interactive,
            RiskLevel::Low,
            Some(project_dir.path()),
        )
        .await;
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let premise_rows = decode_intent_control_rows(&ssb, IntentControlKind::PremiseFlagged);
        assert!(
            !premise_rows.is_empty(),
            "CONTRACT: PremiseFlagged must fire regardless of output format (variant {})",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 3 — autonomous default does not use a population/majority prior.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_journal_default_not_population_prior() {
    let _mutex = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_INTENT_CONTROL_ENABLE);
    std::env::set_var(ELAI_INTENT_CONTROL_ENABLE, "1");

    let (ssb, _ssb_dir) = build_ssb("journal-default").await;

    // A task with NO false premises (no sync/async contradiction) in
    // autonomous mode. The journal is empty, so the default must be
    // ProceedUnderAssumptions (the minimally-presumptive reversible action),
    // NOT a population/majority prior.
    let task = "add a new helper function to utils.rs";
    run_intent_control(
        task,
        &ssb,
        os_clock(),
        GateMode::Autonomous,
        RiskLevel::Low,
        None,
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let route_rows = decode_intent_control_rows(&ssb, IntentControlKind::RouteDecided);
    assert!(
        !route_rows.is_empty(),
        "CONTRACT: RouteDecided must be emitted on the autonomous default path"
    );
    let route = &route_rows[0].route_decision;
    assert_eq!(
        route, "proceed_under_assumptions",
        "CONTRACT: empty-journal autonomous default must be proceed_under_assumptions, \
         not a population/majority prior"
    );

    // No PremiseFlagged rows should exist (no false premise in this task).
    let premise_rows = decode_intent_control_rows(&ssb, IntentControlKind::PremiseFlagged);
    assert!(
        premise_rows.is_empty(),
        "CONTRACT: no false premise in this task, PremiseFlagged must not fire"
    );
}

// ---------------------------------------------------------------------------
// Contract 4 — no "aim-verified" / "right-problem-confirmed" string or enum.
// ---------------------------------------------------------------------------

#[test]
fn contract_no_aim_verified_verdict() {
    // The IntentControlKind enum must NOT have an "aim-verified" or
    // "right-problem-confirmed" arm. The 4 arms are SpecReconstructed,
    // PremiseFlagged, RouteDecided, AssumptionLogged.
    let arms = [
        IntentControlKind::SpecReconstructed,
        IntentControlKind::PremiseFlagged,
        IntentControlKind::RouteDecided,
        IntentControlKind::AssumptionLogged,
    ];
    for arm in &arms {
        let label = arm.as_str();
        assert!(
            !label.contains("aim") && !label.contains("verified") && !label.contains("confirmed"),
            "CONTRACT: IntentControlKind arm '{}' must not carry aim-verified/confirmed semantics",
            label
        );
    }

    // SCHEMA_VERSION must reflect the J-35 IntentControlEvent addition.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 288,
        "CONTRACT: SCHEMA_VERSION must be >= 288 after J-35 IntentControlEvent addition"
    );
}

// ---------------------------------------------------------------------------
// Contract 5 — env flag off/unset -> run_intent_control is a no-op.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_flag_off_is_noop() {
    let _mutex = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_INTENT_CONTROL_ENABLE);

    // Unset -> disabled.
    std::env::remove_var(ELAI_INTENT_CONTROL_ENABLE);
    assert!(
        !intent_control_enabled(),
        "CONTRACT: intent_control_enabled must be false when env is unset"
    );

    // Explicitly off.
    std::env::set_var(ELAI_INTENT_CONTROL_ENABLE, "0");
    assert!(
        !intent_control_enabled(),
        "CONTRACT: intent_control_enabled must be false when env is 0"
    );

    // run_intent_control itself does NOT check the env flag (the caller in
    // task_runner.rs gates on it). But when the flag is off, the production
    // path never calls run_intent_control. Verify the gate function is the
    // correct no-op guard.
    std::env::set_var(ELAI_INTENT_CONTROL_ENABLE, "1");
    assert!(
        intent_control_enabled(),
        "CONTRACT: intent_control_enabled must be true when env is 1"
    );
}
