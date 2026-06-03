//! Layer-4 runtime trace-assertion harness (WIRING-FIX-ELAI Phase 3 / SP4).
//!
//! AUDITOR-AUTHORED, role-boundary deliberate: this file lives in the
//! test-only `elai_cc_contracts` crate, NOT in the orchestrator crate that
//! produces the events it asserts on. The feature agent that wires an
//! observable_effect producer does NOT write this harness (wiring-enforcement
//! R-role-boundary: an agent that writes its own smoke test routes it around
//! the real path). The auditor owns it.
//!
//! WHAT THIS PROVES (wiring-enforcement R4 + the four self-check questions):
//!   1. A REAL production entrypoint ran -- here `Fsm::transition`, the same
//!      call site the orchestrator drives in production (fsm.rs::transition at
//!      fsm.rs:878, the proven SSB-emitting path covered by the orchestrator's
//!      own test_transition_emits_phase_transition_ssb at fsm.rs:2141).
//!      NOT a unit-test call to the produced function directly.
//!   2. It left an artifact the harness did NOT author: an off-agent SSB row
//!      (action_type `phase_transition`) read back through the read-only
//!      `SsbViewer`, never through the writer's own return value.
//!   3. Missing evidence is a HARD FAIL: db-not-found => FAIL (R2), zero rows
//!      in the window => FAIL (R1). Neither is SKIP/UNVERIFIED/WARN.
//!
//! WHY Fsm::transition and not `elai wiring audit`:
//!   `WiringAuditCmd::run` captures auditor payloads into an in-memory vec and
//!   does NOT persist a durable SSB row today, so it is not a valid Layer-4
//!   producer yet. `Fsm::transition` IS a proven real SSB-writing entrypoint
//!   (fsm.rs:2141-2174 reads back exactly one `phase_transition` row through
//!   `SsbViewer`). Once D-WC wires the wiring-audit ObservableEffect to emit a
//!   durable `wiring_evidence` row, the sibling `#[ignore]`d dogfood test below
//!   asserts that path with the identical pattern.
//!
//! OTel NOTE (carried for the day spans are added): if/when this harness gains
//! OpenTelemetry spans, use `SimpleSpanProcessor` + `InMemorySpanExporter`
//! ONLY. `BatchSpanProcessor` is BANNED in test setup -- its flush timing
//! causes silent fail-open (a span that never flushes before assertion looks
//! like a missing span, but the gate must distinguish "not produced" from
//! "not flushed"; Simple+InMemory makes the write synchronous so absence is
//! real). No otel dep is added by this file; the note gates a future change.

use std::sync::Arc;
use std::time::Duration;

use elai_core::ClockProvider;
use elai_core::OsClock;
use elai_orchestrator::fsm::{Fsm, FsmState};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_orchestrator::ssb_viewer::SsbViewer;
use tempfile::TempDir;

/// Build a real, in-process `Ssb` bound to a temp-dir SQLite db BEFORE any
/// production code runs. Mirrors the exact construction used by the
/// orchestrator's own proven test (fsm.rs:2146-2154):
/// `SsbConfig { db_path, pool_size, project_id, session_id }` + `OsClock`.
///
/// Returns the `Ssb` handle (wrapped in `Arc` for the FSM writer) and the
/// db path so the harness can open a sibling read-only `SsbViewer`.
fn bind_ssb(dir: &TempDir) -> (Arc<Ssb>, std::path::PathBuf) {
    let db_path = dir.path().join("layer4_ssb.db");
    let config = SsbConfig {
        db_path: db_path.clone(),
        pool_size: Some(2),
        project_id: "wiring-l4".into(),
        session_id: "s-wiring-l4".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(config, clock).expect("layer4 harness: ssb init"));
    (ssb, db_path)
}

/// FAIL-CLOSED window poll for an off-agent SSB row.
///
/// Polls the read-only `SsbViewer` at `phase_transition`/`fsm_transition`
/// (via `recent_phase_transitions`, ssb_viewer.rs:959) until at least one row
/// appears OR the deadline elapses. Returns `Ok(count)` only when `count > 0`.
///
/// FAIL-CLOSED semantics (wiring-enforcement R1/R2):
///   * db file never appeared by the deadline           => Err (FAIL)
///   * `SsbViewer::open` errors (corrupt / mid-create)   => keep polling,
///     but a persistent open error past the deadline      => Err (FAIL)
///   * zero rows at the deadline                          => Err (FAIL)
/// The harness MUST fail-close: "not found by deadline" == "the feature never
/// ran". The production `ObservableEffectRunner` fail-OPENs on a transiently
/// missing db (keeps polling); the auditor harness must invert that and SCREAM
/// when evidence is absent at the deadline rather than shrug.
async fn expect_phase_transition_within(
    db_path: &std::path::Path,
    deadline: Duration,
) -> Result<usize, String> {
    let start = std::time::Instant::now();
    let poll = Duration::from_millis(25);
    loop {
        if db_path.exists() {
            match SsbViewer::open(db_path) {
                Ok(viewer) => {
                    let rows = viewer
                        .recent_phase_transitions(16)
                        .map_err(|e| format!("viewer query failed: {e}"))?;
                    if !rows.is_empty() {
                        return Ok(rows.len());
                    }
                }
                Err(e) => {
                    // DB present but not yet openable (writer mid-create).
                    // Tolerated UNTIL the deadline; a persistent error past
                    // the deadline falls through to the FAIL below.
                    if start.elapsed() >= deadline {
                        return Err(format!(
                            "FAIL: SSB db at {} present but unopenable by deadline: {e}",
                            db_path.display()
                        ));
                    }
                }
            }
        }
        if start.elapsed() >= deadline {
            // R2: db-not-found OR zero-rows by deadline is a HARD FAIL.
            let existed = db_path.exists();
            return Err(format!(
                "FAIL: no off-agent phase_transition SSB row within {deadline:?} \
                 (db_exists={existed}, path={}). Missing evidence == feature \
                 never ran (wiring-enforcement R1/R2).",
                db_path.display()
            ));
        }
        tokio::time::sleep(poll).await;
    }
}

/// Layer-4 PASS path: drive the REAL `Fsm::transition` entrypoint with a live
/// SSB writer, then assert the off-agent `phase_transition` row landed.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn layer4_real_entrypoint_leaves_off_agent_ssb_row() {
    let dir = TempDir::new().expect("tempdir");
    let (ssb, db_path) = bind_ssb(&dir);

    // Drive the REAL production entrypoint (NOT the produced fn directly).
    let fsm = Fsm::new(None, 8000).with_ssb_writer(Arc::clone(&ssb));
    fsm.transition(FsmState::Planning, "wiring-l4-phase", vec![])
        .await
        .expect("production Fsm::transition must succeed");

    // Assert the OFF-AGENT artifact (a row this test did not write) appeared.
    let count = expect_phase_transition_within(&db_path, Duration::from_secs(5))
        .await
        .expect("layer4: off-agent SSB row assertion");
    assert!(count >= 1, "expected >= 1 phase_transition row, got {count}");
}

/// FAIL-CLOSED self-test: the harness MUST fail when the db never appears.
/// Proves the gate screams on missing evidence rather than shrugging (R2).
/// Drives NO entrypoint, so no row is ever written; the poll must return Err.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn layer4_missing_db_is_hard_fail() {
    let dir = TempDir::new().expect("tempdir");
    let absent = dir.path().join("never_created_ssb.db");
    let verdict = expect_phase_transition_within(&absent, Duration::from_millis(200)).await;
    assert!(
        verdict.is_err(),
        "missing-db / zero-row MUST be a hard FAIL, got Ok({verdict:?})"
    );
    let msg = verdict.unwrap_err();
    assert!(
        msg.starts_with("FAIL:"),
        "fail message must be explicit: {msg}"
    );
}

/// FAIL-CLOSED self-test: an EMPTY but openable db (no transition driven) must
/// also be a hard FAIL. Distinguishes "db exists, zero rows" from "db absent"
/// -- both must FAIL (R1). This is the stronger of the two missing-evidence
/// cases: the db is real and queryable, the query simply returns zero rows.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn layer4_empty_db_zero_rows_is_hard_fail() {
    let dir = TempDir::new().expect("tempdir");
    // Construct the Ssb (which runs migrations and creates the db file) but
    // drive NO entrypoint, so the `ssb` table has zero phase_transition rows.
    let (_ssb, db_path) = bind_ssb(&dir);
    assert!(db_path.exists(), "Ssb::new must create the db file");

    let verdict = expect_phase_transition_within(&db_path, Duration::from_millis(300)).await;
    assert!(
        verdict.is_err(),
        "zero-rows in an existing db MUST be a hard FAIL, got Ok({verdict:?})"
    );
    let msg = verdict.unwrap_err();
    assert!(
        msg.starts_with("FAIL:"),
        "fail message must be explicit: {msg}"
    );
    assert!(
        msg.contains("db_exists=true"),
        "zero-row FAIL must report the db existed: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Cross-file contract placeholder (D-WC): once the wiring-audit
// ObservableEffect is wired to PERSIST a durable `wiring_evidence` SSB row
// (today WiringAuditCmd::run captures into an in-memory vec only), this test
// drives `elai wiring audit` end-to-end and asserts the `wiring_evidence`
// action_type lands -- the dogfood: the wiring gate passes its OWN
// WIRING_CONTRACT by leaving a real SSB row. Ignored until D-WC lands so the
// suite stays green; the auditor flips `#[ignore]` off when the producer
// ships (and a default-OFF state of THIS test is itself wire-dark, so the
// auditor tracks it). The expected action_type string is "wiring_evidence"
// (ssb.rs SsbActionType::WiringEvidence::as_sql_str) -- the shared surface
// with D-WC's ObservableEffect.expect_ssb.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "D-WC: wiring-audit must persist wiring_evidence to a real SSB before this can assert"]
async fn layer4_wiring_audit_dogfood_emits_wiring_evidence() {
    // Intentionally a deliberate no-op until D-WC wires the durable producer.
    // The harness shape is identical to
    // layer4_real_entrypoint_leaves_off_agent_ssb_row: bind Ssb to a temp db
    // -> drive `elai wiring audit` against a fixture contract dir -> assert a
    // real `wiring_evidence` row lands within the deadline via SsbViewer.
    // The #[ignore] keeps the green suite green; the auditor un-ignores it
    // when D-WC ships the durable wiring_evidence producer.
    assert!(
        true,
        "placeholder: un-ignore once D-WC persists wiring_evidence rows"
    );
}
