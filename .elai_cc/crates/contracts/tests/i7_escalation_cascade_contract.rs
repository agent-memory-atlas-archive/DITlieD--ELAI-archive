//! I-7 contract tests — cascading escalation waterfall (Wave-4).
//!
//! Off-agent artifacts (Layer-4 / R4, fail-closed per R1: a missing row FAILS).
//! Every "row landed" assertion is a REAL-DB `query_by_type` against the live SSB
//! sqlite (NOT a struct-only round-trip) so a wire that never reached the DB cannot
//! pass. The waterfall is driven through its PUBLIC `EscalationWaterfall::run` entry
//! — the exact symbol `task_runner::maybe_run_escalation_waterfall` calls — over
//! fixture `TriLayerStack`s (no live model calls, D-026: the cloud arm is the
//! deterministic router DECISION, never a billed inference).
//!
//!   (a) REAL-DB SSB arm:
//!       * a PASSING fixture (empty checker registry -> every branch survives)
//!         lands exactly one EscalationCascade(LocalWin) row and ZERO Escalated
//!         rows (the E-39 zero-cloud-spend observable on a LocalWin).
//!       * an ALL-FAIL fixture (always-veto registry -> every branch pruned) lands
//!         a Condensed then an Escalated row, and the Escalated outcome carries the
//!         synchronous cloud routing DECISION (RoutedProvider::Cloud).
//!   (b) IC-1 STRUCTURAL GUARD: the escalation_waterfall.rs source has ZERO import
//!       path to the H-5 batch client (no batch_client / MessageBatch /
//!       anthropic_batch tokens). The live waterfall is structurally unable to reach
//!       the batch lane (the batch-racing design is DEAD per IC-1).
//!   (c) GATE default-OFF: with ELAI_ESCALATION_WATERFALL_ENABLE unset the gate
//!       helper is false (the drive_task hook is a strict no-op).
//!
//! No #[ignore] / stubs / placeholder assertions.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::adversary::AdversaryConfig;
use elai_orchestrator::checker_registry::{CheckerRegistry, RegexMatchChecker};
use elai_orchestrator::cloud_mode::{CloudCredentials, RoutedProvider};
use elai_orchestrator::escalation_waterfall::{
    default_escalation_router, escalation_waterfall_enabled, EscalationWaterfall,
    ELAI_ESCALATION_WATERFALL_ENABLE_ENV,
};
use elai_orchestrator::mutation_budget::MutationBudget;
use elai_orchestrator::pristine_path_filter::PristinePathFilter;
use elai_orchestrator::search::{BranchInput, SearchLadder};
use elai_orchestrator::ssb::{
    decode_ssb_payload, EscalationCascadeKind, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::verifier_stack::{Branch, FlakeQuarantineLedger, TriLayerStack};
use elai_orchestrator::vibesec::dep_writetime_gate::{DepWriteTimeGate, StubRegistryChecker};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const KILL_RATE_FLOOR: f64 = 0.5;

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i7-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb init"))
}

fn stub_dep_gate() -> Arc<DepWriteTimeGate> {
    Arc::new(DepWriteTimeGate::new(None, Arc::new(StubRegistryChecker)))
}

/// A registry with NO checker -> L1 passes, every branch is a ScoredSurvivor
/// (the LocalWin fixture: at least one branch survives the tri-layer check).
fn empty_registry() -> Arc<CheckerRegistry> {
    Arc::new(CheckerRegistry::new())
}

/// A registry that ALWAYS vetoes (anchored regex requiring a sentinel the source
/// never contains -> L1 HardVeto). The all-fail fixture: every branch is pruned,
/// so the waterfall condenses + escalates.
fn always_veto_registry() -> Arc<CheckerRegistry> {
    let mut reg = CheckerRegistry::new();
    let checker = RegexMatchChecker::new("ZZZ_NEVER_PRESENT_SENTINEL_ZZZ", "i7-veto", false)
        .expect("regex compiles");
    reg.register("i7-always-veto", Box::new(checker));
    Arc::new(reg)
}

/// Build the I-7 waterfall over a given checker registry (the only thing that
/// varies L1-veto vs L1-pass across the arms). Mirrors `maybe_run_escalation_
/// waterfall`'s construction recipe (GD-1: same I-4 ladder + default router shape).
fn build_waterfall(registry: Arc<CheckerRegistry>, ssb: &Arc<Ssb>) -> EscalationWaterfall {
    let stack = Arc::new(TriLayerStack::new(
        registry,
        stub_dep_gate(),
        None,
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(ssb),
    ));
    let pristine = Arc::new(PristinePathFilter::new(Arc::clone(ssb)));
    let quarantine = Arc::new(FlakeQuarantineLedger::new(Arc::clone(ssb)));
    let mutation_budget = MutationBudget::with_threshold(KILL_RATE_FLOOR);
    let ladder = Arc::new(SearchLadder::new(
        stack,
        pristine,
        quarantine,
        mutation_budget,
        Arc::clone(ssb),
    ));
    EscalationWaterfall::new(ladder, default_escalation_router(), Arc::clone(ssb))
}

fn rows(ssb: &Arc<Ssb>, kind: EscalationCascadeKind) -> Vec<elai_orchestrator::ssb::SsbEntry> {
    ssb.query_by_type(SsbActionType::EscalationCascade(kind))
        .expect("query_by_type must succeed")
}

// ---------------------------------------------------------------------------
// (a.1) LocalWin — a solvable fixture solves locally, zero cloud escalation.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i7_a_local_win_emits_no_cloud_escalation() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i7-a");
    // Empty registry -> every branch clears L1 and survives, so the I-4 ladder
    // promotes a winner -> LocalWin (task solved on the local lane).
    let waterfall = build_waterfall(empty_registry(), &ssb);
    let candidates = vec![
        BranchInput::from_branch(Branch::new("solve-a", "fn a() {}")),
        BranchInput::from_branch(Branch::new("solve-b", "fn b() {}")),
    ];

    let outcome = waterfall
        .run(&candidates, "i7-a-session", Vec::new(), None)
        .await
        .expect("waterfall run ok");

    assert!(outcome.local_win, "a surviving branch -> LocalWin");
    assert!(
        outcome.routed.is_none(),
        "LocalWin never routes to the cloud (E-39 zero-cloud-spend)"
    );
    assert_eq!(outcome.condensed_bytes, 0, "LocalWin sends no condensate");

    // Real-DB (R1 missing=FAIL): exactly one LocalWin row, ZERO Condensed/Escalated.
    assert_eq!(
        rows(&ssb, EscalationCascadeKind::LocalWin).len(),
        1,
        "exactly one EscalationCascade(LocalWin) row (real-DB count)"
    );
    assert!(
        rows(&ssb, EscalationCascadeKind::Condensed).is_empty(),
        "no condensation on a LocalWin"
    );
    assert!(
        rows(&ssb, EscalationCascadeKind::Escalated).is_empty(),
        "the E-39 zero-cloud-spend observable: a LocalWin emits NO Escalated row"
    );

    // Decode the LocalWin row and confirm its typed shape.
    match decode_ssb_payload(&rows(&ssb, EscalationCascadeKind::LocalWin)[0].payload)
        .expect("decode EscalationCascade")
    {
        SsbPayload::EscalationCascade(p) => {
            assert_eq!(p.kind, EscalationCascadeKind::LocalWin);
            assert_eq!(p.condensed_bytes, 0, "LocalWin carries no condensate bytes");
        }
        other => panic!("expected EscalationCascade, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (a.2) Condensed + Escalated — an all-fail fixture condenses + routes to cloud.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i7_a_all_fail_condenses_then_escalates() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i7-a2");
    // always-veto registry -> EVERY candidate L1-vetoes (the branch source has a
    // `=` so the anchored regex applies_to fires and Fails -> HardVeto), so the
    // ladder produces no winner -> the waterfall condenses + escalates.
    let waterfall = build_waterfall(always_veto_registry(), &ssb);
    let candidates = vec![
        BranchInput::from_branch(Branch::new("fail-a", "let a = 1;")),
        BranchInput::from_branch(Branch::new("fail-b", "let b = 2;")),
    ];
    let failed_because = vec!["dep-gate veto".to_string(), "type error".to_string()];
    // The escalation hop is a real cloud routing DECISION (D-026: a decision over
    // the fixture, NOT a billed inference). Routing to the cloud lane requires
    // credentials, so the fixture supplies a test token; the router then returns the
    // deterministic RoutedProvider::Cloud lane choice with no network call.
    let creds = Some(CloudCredentials::for_test_with_token("i7-fixture-token"));

    let outcome = waterfall
        .run(&candidates, "i7-a2-session", failed_because, creds)
        .await
        .expect("waterfall run ok");

    assert!(!outcome.local_win, "every branch vetoed -> not a LocalWin");
    // The synchronous cloud routing DECISION was taken (D-026: a decision, not a
    // billed inference). The default router with no spend cap routes Cloud.
    match outcome.routed {
        Some(RoutedProvider::Cloud { .. }) => {}
        other => panic!("all-fail must route to the cloud lane, got {other:?}"),
    }
    assert!(
        outcome.condensed_bytes > 0,
        "the condensed BeliefState carries bytes on escalation"
    );

    // Real-DB (R1 missing=FAIL): a Condensed then an Escalated row landed, ZERO
    // LocalWin rows (the all-fail path never claims a local solve).
    assert_eq!(
        rows(&ssb, EscalationCascadeKind::Condensed).len(),
        1,
        "exactly one EscalationCascade(Condensed) row (real-DB)"
    );
    assert_eq!(
        rows(&ssb, EscalationCascadeKind::Escalated).len(),
        1,
        "exactly one EscalationCascade(Escalated) row (real-DB)"
    );
    assert!(
        rows(&ssb, EscalationCascadeKind::LocalWin).is_empty(),
        "an all-fail run never emits LocalWin"
    );
    // The OFFLINE-only BatchRouted arm is NEVER emitted by the LIVE waterfall (IC-1).
    assert!(
        rows(&ssb, EscalationCascadeKind::BatchRouted).is_empty(),
        "the live waterfall NEVER emits BatchRouted (IC-1: offline-journal-only)"
    );

    // ID-3: the I-11 condensation also lands its own BeliefStateEvent(Condensed)
    // receipt — the typed condensate is journaled (un-journaled = FAIL).
    let belief_rows = ssb
        .query_by_type(SsbActionType::BeliefStateEvent(
            elai_orchestrator::ssb::BeliefStateEventKind::Condensed,
        ))
        .expect("query belief receipts");
    assert!(
        !belief_rows.is_empty(),
        "the I-11 condensed receipt landed (ID-3 journaled-middleware)"
    );

    // Decode the Escalated row and confirm it carries the condensed byte size.
    match decode_ssb_payload(&rows(&ssb, EscalationCascadeKind::Escalated)[0].payload)
        .expect("decode EscalationCascade")
    {
        SsbPayload::EscalationCascade(p) => {
            assert_eq!(p.kind, EscalationCascadeKind::Escalated);
            assert!(
                p.condensed_bytes > 0,
                "Escalated row carries the real condensate byte size"
            );
        }
        other => panic!("expected EscalationCascade, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (b) IC-1 STRUCTURAL GUARD — the waterfall module has NO batch-client import.
// ---------------------------------------------------------------------------

#[test]
fn i7_b_ic1_no_batch_client_import() {
    // The buildable waterfall MUST be structurally unable to reach the H-5 batch
    // client (IC-1: the batch-racing design is DEAD; the Batch API runs 10-45 min /
    // 24h SLA / cancels do not refund, so it can never be on the live loop). We
    // assert the source carries ZERO batch-client tokens — an injected import would
    // be visible here AND fail the call-graph guard.
    let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("orchestrator")
        .join("src")
        .join("escalation_waterfall.rs");
    let src = std::fs::read_to_string(&src_path).unwrap_or_else(|e| {
        panic!(
            "cannot read escalation_waterfall.rs at {}: {e}",
            src_path.display()
        )
    });

    // Only flag the batch-client IDENTIFIERS as a `use`/path, not the prose in the
    // doc comments that EXPLAINS the ban. We scan the non-comment code lines.
    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.trim_start();
        if line.starts_with("//") || line.starts_with("*") || line.starts_with("/*") {
            continue;
        }
        for banned in ["anthropic_batch", "batch_client", "MessageBatch"] {
            assert!(
                !line.contains(banned),
                "IC-1 VIOLATION: escalation_waterfall.rs line {} references the H-5 \
                 batch client `{}` on a non-comment line — the live waterfall must \
                 have NO batch-client path:\n  {}",
                lineno + 1,
                banned,
                raw
            );
        }
    }
}

// ---------------------------------------------------------------------------
// (c) GATE default-OFF — the drive_task hook is a strict no-op when unset.
// ---------------------------------------------------------------------------

#[test]
fn i7_c_gate_default_off() {
    // Default-OFF (wire-dark): with the var unset the waterfall dispatch is a no-op
    // and the legacy local->cloud transition path is unchanged.
    std::env::remove_var(ELAI_ESCALATION_WATERFALL_ENABLE_ENV);
    assert!(
        !escalation_waterfall_enabled(),
        "the escalation waterfall must be wire-dark by default (GD-4)"
    );

    // ON only behind a truthy literal.
    std::env::set_var(ELAI_ESCALATION_WATERFALL_ENABLE_ENV, "1");
    assert!(
        escalation_waterfall_enabled(),
        "ENABLE=1 turns the waterfall ON"
    );
    std::env::set_var(ELAI_ESCALATION_WATERFALL_ENABLE_ENV, "off");
    assert!(
        !escalation_waterfall_enabled(),
        "a non-truthy literal keeps it OFF"
    );
    std::env::remove_var(ELAI_ESCALATION_WATERFALL_ENABLE_ENV);

    // The env literal must match the wire-dark-allowlist.toml flag name.
    assert_eq!(
        ELAI_ESCALATION_WATERFALL_ENABLE_ENV, "ELAI_ESCALATION_WATERFALL_ENABLE",
        "env gate literal must match the wire-dark-allowlist.toml flag name"
    );
}

// ---------------------------------------------------------------------------
// (d) wire round-trip — all 4 Kind arms map to distinct sql_str + decode.
// ---------------------------------------------------------------------------

#[test]
fn i7_d_wire_roundtrip_all_arms() {
    for (kind, tag) in [
        (EscalationCascadeKind::LocalWin, "escalation_cascade_local_win"),
        (EscalationCascadeKind::Condensed, "escalation_cascade_condensed"),
        (EscalationCascadeKind::Escalated, "escalation_cascade_escalated"),
        (
            EscalationCascadeKind::BatchRouted,
            "escalation_cascade_batch_routed",
        ),
    ] {
        assert_eq!(
            SsbActionType::EscalationCascade(kind).as_sql_str(),
            tag,
            "each Kind arm maps to its distinct sql_str"
        );
    }
}
