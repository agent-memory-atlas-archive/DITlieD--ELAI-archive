//! I-17 contract tests — tri-layer evaluation stack (veto-supreme branch scorer).
//!
//! Off-agent artifacts (Layer-4 / R4, fail-closed per R1: a missing row FAILS).
//! Every "row landed" assertion is a REAL-DB `query_by_type` against the live
//! SSB sqlite (NOT a struct-only round-trip) so a wire that never reached the DB
//! cannot pass (closes the I-5/I-13 struct-only fail-open class).
//!
//!   (F1) HARD VETO arm (ID-4 supremacy): an L1 checker veto produces a
//!        `TriLayerVerdict(HardVeto)` VerdictRow with score == 0.0 (LITERAL, no
//!        soft-veto weight), `l2_call_count == 0` (the observable proof L2 never
//!        saw a vetoed branch), and a `TriLayerVerdict` row in the SSB DB.
//!   (F2) SCORED SURVIVOR arm: a branch that clears L1 gets an L2 score and a
//!        `ScoredSurvivor` row lands in the SSB DB with `l2_call_count >= 1`.
//!   (F3) DISAGREEMENT -> ESCALATION arm: a survivor set whose L2 scores diverge
//!        past L3_VARIANCE_THRESHOLD routes the D-13 adversary and lands a
//!        `DisagreementEscalated` row in the SSB DB.
//!   (F4) HALTING-BOUND watchdog (I-8 stub resolved): a DeadEndCache built with a
//!        ~0 ms bound + an SSB writer spawns a real watchdog whose breach lands a
//!        `TrajectoryAborted(HaltingBound)` row in the SSB DB.
//!   (F5) Wire-tag round-trip: `tri_layer_verdict` is in the SSB bijection; all 4
//!        Kind arms map through payload.action_type().
//!   (F6) ID-4 broken-arm guard: the veto score is asserted == 0.0; a patched
//!        "strong negative weight" (e.g. -1.0 or 0.001) would fail this — the
//!        guard the spec's --l2-overrides-l1 broken arm exists to catch.
//!   (F7) Env-gate literal matches the wire-dark-allowlist flag name.
//!
//! No #[ignore] / stubs / placeholder assertions.
//!
//! Grounding:
//!   - crates/orchestrator/src/verifier_stack.rs (TriLayerStack, Branch, VerdictRow,
//!     FlakeQuarantineLedger, VETO_SCORE, L3_VARIANCE_THRESHOLD, TRI_LAYER_ENABLE_ENV)
//!   - crates/orchestrator/src/dead_end_cache.rs (check_halting_bound watchdog)
//!   - crates/orchestrator/src/ssb.rs (TriLayerVerdict* + SsbReader::query_by_type)
//!   - ID-4 veto supremacy (no soft veto, no L2 override); GD-1 compose-not-restate.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::adversary::{
    AdversaryConfig, AdversaryErased, CounterEvidenceSchema, DynAdversary, ModelFamily, MockAdversary,
};
use elai_orchestrator::checker_registry::{CheckerRegistry, RegexMatchChecker};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, GateId, Ssb, SsbActionType, SsbConfig,
    SsbPayload, TrajectoryAbortedKind, TriLayerVerdictKind, TriLayerVerdictPayload,
};
use elai_orchestrator::verifier_stack::{
    tri_layer_enabled, Branch, FlakeQuarantineLedger, TriLayerStack, L3_VARIANCE_THRESHOLD,
    TRI_LAYER_ENABLE_ENV, VETO_SCORE,
};
use elai_orchestrator::vibesec::dep_writetime_gate::{DepWriteTimeGate, StubRegistryChecker};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i17-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb init"))
}

/// An offline-safe dep-gate (the StubRegistryChecker treats unknown packages as
/// Found, so legitimate source never spuriously vetoes).
fn stub_dep_gate() -> Arc<DepWriteTimeGate> {
    Arc::new(DepWriteTimeGate::new(None, Arc::new(StubRegistryChecker)))
}

/// A checker registry that ALWAYS vetoes: a RegexMatchChecker (anchored, `contains
/// = false`) requiring a sentinel the empty/short branch source never contains, so
/// `check` returns Fail -> L1 HardVeto. Drives the F1 + F6 arms deterministically
/// with no subprocess / network.
fn always_veto_registry() -> Arc<CheckerRegistry> {
    let mut reg = CheckerRegistry::new();
    let checker = RegexMatchChecker::new("ZZZ_NEVER_PRESENT_SENTINEL_ZZZ", "i17-veto", false)
        .expect("regex compiles");
    reg.register("i17-always-veto", Box::new(checker));
    Arc::new(reg)
}

/// An empty checker registry: no checker applies, so L1 passes -> the branch
/// reaches L2 (ScoredSurvivor). Drives F2.
fn empty_registry() -> Arc<CheckerRegistry> {
    Arc::new(CheckerRegistry::new())
}

fn mock_adversary() -> Arc<dyn DynAdversary> {
    let counter = CounterEvidenceSchema {
        counter_id: "i17-counter-1".into(),
        original_claim_id: "branch-disputed".into(),
        counter_text: "the high-scoring branch has an unhandled edge case".into(),
        counter_evidence: Vec::new(),
        severity: 0.9,
    };
    // Adversary uses native async-fn-in-trait (not dyn-compatible); erase it into
    // the object-safe DynAdversary the stack stores.
    Arc::new(AdversaryErased::new(MockAdversary::with_counter(
        ModelFamily::Anthropic,
        counter,
    )))
}

/// Count SSB rows of a given action_type via the REAL DB reader (not a struct
/// round-trip). Pattern-2: any Kind arm matches the same `tri_layer_verdict` tag.
fn count_rows(ssb: &Arc<Ssb>, action: SsbActionType) -> usize {
    ssb.query_by_type(action)
        .expect("query_by_type must succeed")
        .len()
}

// ---------------------------------------------------------------------------
// (F1) HARD VETO — score 0.0, l2_call_count == 0, real-DB row.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i17_f1_hard_veto_l2_call_count_zero() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i17-f1");
    let stack = TriLayerStack::new(
        always_veto_registry(),
        stub_dep_gate(),
        None,
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(&ssb),
    );

    // The branch source contains a `=` so the (universally-applicable) regex
    // checker's applies_to fires; the anchored pattern never matches -> Fail.
    let branch = Branch::new("branch-veto", "let x = 1;");
    let row = stack.run_verdict(&branch).await.expect("run_verdict ok");

    // ID-4: hard veto is supreme.
    assert_eq!(row.verdict, TriLayerVerdictKind::HardVeto, "L1 fail must be HardVeto");
    assert!(row.is_vetoed(), "is_vetoed() must be true for a HardVeto row");
    assert_eq!(row.score, VETO_SCORE, "vetoed branch score must be the literal VETO_SCORE");
    assert_eq!(row.score, 0.0, "VETO_SCORE must be a hard 0.0 (no soft veto, ID-4)");
    // THE observable proof: a vetoed branch never reached L2.
    assert_eq!(row.l2_call_count, 0, "L2 MUST NOT run on a vetoed branch (l2_call_count == 0)");
    assert!(row.gate.is_some(), "HardVeto must carry the vetoing GateId");

    // Real-DB (R1 missing=FAIL): a tri_layer_verdict row landed.
    let rows = ssb
        .query_by_type(SsbActionType::TriLayerVerdict(TriLayerVerdictKind::HardVeto))
        .expect("query TriLayerVerdict rows");
    assert!(!rows.is_empty(), "TriLayerVerdict row must land in SSB DB (R1 missing=FAIL)");
    let payload = decode_ssb_payload(&rows[0].payload).expect("decode TriLayerVerdict");
    match payload {
        SsbPayload::TriLayerVerdict(p) => {
            assert_eq!(p.kind, TriLayerVerdictKind::HardVeto);
            assert_eq!(p.l2_call_count, 0, "DB row also proves l2_call_count == 0");
            assert_eq!(p.score, 0.0, "DB row score must be literal 0.0");
            assert!(p.gate.is_some(), "DB row must carry the gate");
        }
        other => panic!("expected TriLayerVerdict, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (F2) SCORED SURVIVOR — clears L1, gets an L2 score, real-DB row.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i17_f2_scored_survivor_gets_l2_score() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i17-f2");
    let stack = TriLayerStack::new(
        empty_registry(), // no checker applies -> L1 passes
        stub_dep_gate(),
        None, // no calibration corpus -> L2 degrades to TAU_DEFAULT_FALLBACK (does NOT panic)
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(&ssb),
    );

    let branch = Branch::new("branch-survivor", "fn ok() {}");
    let row = stack.run_verdict(&branch).await.expect("run_verdict ok");

    assert_eq!(
        row.verdict,
        TriLayerVerdictKind::ScoredSurvivor,
        "an L1-passing branch must be a ScoredSurvivor"
    );
    assert!(!row.is_vetoed(), "survivor is not vetoed");
    assert_eq!(row.l2_call_count, 1, "L2 ran exactly once for the survivor");
    assert!(row.gate.is_none(), "a survivor carries no veto gate");
    assert!(
        (0.0..=1.0).contains(&row.score),
        "L2 score must be in [0,1]; got {}",
        row.score
    );

    // Real-DB COUNT (not struct-only): exactly one tri_layer_verdict row landed.
    let n = count_rows(
        &ssb,
        SsbActionType::TriLayerVerdict(TriLayerVerdictKind::ScoredSurvivor),
    );
    assert_eq!(n, 1, "exactly one tri_layer_verdict row for the survivor (real-DB count)");
}

// ---------------------------------------------------------------------------
// (F3) DISAGREEMENT -> ESCALATION — diverging survivors route the adversary.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i17_f3_disagreement_escalates_to_adversary() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i17-f3");
    let stack = TriLayerStack::new(
        empty_registry(),
        stub_dep_gate(),
        None,
        Some(mock_adversary()), // L3 target plumbed
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(&ssb),
    );

    // Score two survivors with high variance (0.05 vs 0.95; var = 0.2025 >>
    // L3_VARIANCE_THRESHOLD). Build them via run_verdict so they are real rows,
    // then hand the SET to escalate_on_disagreement.
    let lo = stack.run_verdict(&Branch::new("branch-lo", "fn a() {}")).await.expect("lo ok");
    let hi = stack.run_verdict(&Branch::new("branch-hi", "fn b() {}")).await.expect("hi ok");
    // The L2 score is deterministic here (TAU_DEFAULT_FALLBACK), so synthesise
    // the divergence directly on the VerdictRow set the L3 gate inspects.
    let mut a = lo;
    let mut b = hi;
    a.score = 0.05;
    b.score = 0.95;
    assert!(
        b.score - a.score > L3_VARIANCE_THRESHOLD,
        "fixture must diverge past the threshold"
    );

    let escalated = stack
        .escalate_on_disagreement(&[a, b])
        .await
        .expect("escalate_on_disagreement ok");
    assert!(escalated, "diverging survivors must escalate to L3");

    // Real-DB: a DisagreementEscalated row landed.
    let rows = ssb
        .query_by_type(SsbActionType::TriLayerVerdict(TriLayerVerdictKind::DisagreementEscalated))
        .expect("query TriLayerVerdict rows");
    assert!(
        !rows.is_empty(),
        "DisagreementEscalated row must land in SSB DB (R1 missing=FAIL)"
    );

    // Agreeing survivors must NOT escalate.
    assert!(
        !TriLayerStack::needs_l3_escalation(&[
            super_survivor("x", 0.70),
            super_survivor("y", 0.71),
        ]),
        "tight cluster must not escalate"
    );
}

/// Local survivor builder for the agreement-negative assertion (VerdictRow has no
/// public ctor by design — it is produced by run_verdict; the contract builds a
/// minimal one through the public payload shape via a scored run is overkill, so
/// we reconstruct through the public escalation predicate's input shape).
fn super_survivor(id: &str, score: f64) -> elai_orchestrator::verifier_stack::VerdictRow {
    elai_orchestrator::verifier_stack::VerdictRow {
        branch_id: id.into(),
        verdict: TriLayerVerdictKind::ScoredSurvivor,
        score,
        gate: None,
        l2_call_count: 1,
    }
}

// ---------------------------------------------------------------------------
// (F4) HALTING-BOUND watchdog — I-8 stub resolved; breach lands a DB row.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i17_f4_halting_bound_watchdog_emits_row() {
    use elai_orchestrator::dead_end_cache::{DeadEndCache, DEFAULT_DEAD_END_CACHE_CAPACITY};

    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i17-f4");

    // A ~0 ms bound + an installed SSB writer -> check_halting_bound spawns a
    // real watchdog that fires almost immediately.
    let cache = DeadEndCache::new_with_halting_bound(DEFAULT_DEAD_END_CACHE_CAPACITY, 1)
        .with_ssb_writer(Arc::clone(&ssb));
    let handle = cache.check_halting_bound();
    assert!(
        handle.is_some(),
        "watchdog MUST spawn when both a bound and an SSB writer are present (I-17 stub resolved)"
    );
    // Await the watchdog so the SSB write completes deterministically.
    handle.expect("handle present").await.expect("watchdog task joins");

    // Real-DB: the watchdog emitted a TrajectoryAborted(HaltingBound) row.
    let rows = ssb
        .query_by_type(SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::HaltingBound))
        .expect("query TrajectoryAborted rows");
    assert!(
        !rows.is_empty(),
        "halting-bound watchdog must land a TrajectoryAborted row in SSB DB (R1 missing=FAIL)"
    );
    let payload = decode_ssb_payload(&rows[0].payload).expect("decode TrajectoryAborted");
    match payload {
        SsbPayload::TrajectoryAborted(p) => {
            assert_eq!(p.kind, TrajectoryAbortedKind::HaltingBound, "kind must be HaltingBound");
            assert!(
                p.abort_reason_text.is_some(),
                "halting-bound abort must carry a reason"
            );
        }
        other => panic!("expected TrajectoryAborted, got {:?}", other.action_type()),
    }

    // Honest no-op: a cache WITHOUT a writer spawns NO watchdog (keeps the i8_f4
    // synchronous contract-test path safe).
    let no_writer = DeadEndCache::new_with_halting_bound(DEFAULT_DEAD_END_CACHE_CAPACITY, 1);
    assert!(
        no_writer.check_halting_bound().is_none(),
        "no SSB writer -> no watchdog (honest no-op)"
    );
}

// ---------------------------------------------------------------------------
// (F5) Wire-tag round-trip — bijection + all 4 Kind arms map.
// ---------------------------------------------------------------------------

#[test]
fn i17_f5_wire_tag_roundtrips() {
    // Tag is in the from_str bijection.
    let parsed = ssb_action_type_from_str_public("tri_layer_verdict")
        .expect("tri_layer_verdict must parse");
    assert_eq!(parsed.as_sql_str(), "tri_layer_verdict");

    // All 4 Kind arms collapse to the same wire tag (Pattern-2).
    for kind in [
        TriLayerVerdictKind::HardVeto,
        TriLayerVerdictKind::ScoredSurvivor,
        TriLayerVerdictKind::DisagreementEscalated,
        TriLayerVerdictKind::FlakeQuarantined,
    ] {
        let payload = SsbPayload::TriLayerVerdict(TriLayerVerdictPayload {
            branch_id: "b".into(),
            kind,
            gate: if matches!(kind, TriLayerVerdictKind::HardVeto) {
                Some(GateId::L1Compile)
            } else {
                None
            },
            l2_call_count: 0,
            score: 0.0,
            ts_ms: 0,
        });
        assert_eq!(
            payload.action_type().as_sql_str(),
            "tri_layer_verdict",
            "every Kind arm maps to the tri_layer_verdict tag"
        );
    }
}

// ---------------------------------------------------------------------------
// (F6) ID-4 broken-arm guard + FlakeQuarantine ledger key (blake3).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i17_f6_veto_is_hard_zero_and_flake_quarantine_emits() {
    // ID-4 broken-arm guard: VETO_SCORE is a hard 0.0. A patched "strong negative
    // weight" (-1.0) or "soft veto" (0.001) would make this fail — exactly the
    // --l2-overrides-l1 broken arm the spec requires to FAIL.
    assert_eq!(VETO_SCORE, 0.0, "ID-4: veto is a hard 0.0, never a penalty weight");
    assert!(VETO_SCORE >= 0.0, "veto score is non-negative (no strong-negative-weight arm)");

    // FlakeQuarantine: blake3 key + a real DB FlakeQuarantined row on quarantine.
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i17-f6");
    let mut ledger = FlakeQuarantineLedger::new(Arc::clone(&ssb));
    assert!(
        !ledger.is_quarantined("oracle-flaky", (0, 1000)),
        "oracle is not quarantined before ledgering"
    );
    ledger.quarantine("oracle-flaky", (0, 1000)).await;
    assert!(
        ledger.is_quarantined("oracle-flaky", (0, 1000)),
        "oracle is quarantined after ledgering (hard gate, not a soft weight)"
    );

    let rows = ssb
        .query_by_type(SsbActionType::TriLayerVerdict(TriLayerVerdictKind::FlakeQuarantined))
        .expect("query TriLayerVerdict rows");
    assert!(
        !rows.is_empty(),
        "FlakeQuarantined row must land in SSB DB (R1 missing=FAIL)"
    );

    // blake3 key is deterministic + 64-hex.
    let k = FlakeQuarantineLedger::quarantine_key("oracle-flaky", (0, 1000));
    assert_eq!(k.len(), 64, "blake3 hex key is 64 chars");
}

// ---------------------------------------------------------------------------
// (F7) Env-gate literal matches the wire-dark-allowlist flag name.
// ---------------------------------------------------------------------------

#[test]
fn i17_f7_env_gate_literal_matches_allowlist() {
    assert_eq!(
        TRI_LAYER_ENABLE_ENV, "ELAI_TRI_LAYER_ENABLE",
        "env gate literal must match the wire-dark-allowlist.toml flag name"
    );
    // Default-OFF (wire-dark): with the var unset, the gate is closed.
    std::env::remove_var(TRI_LAYER_ENABLE_ENV);
    assert!(!tri_layer_enabled(), "tri-layer must be wire-dark by default");
}
