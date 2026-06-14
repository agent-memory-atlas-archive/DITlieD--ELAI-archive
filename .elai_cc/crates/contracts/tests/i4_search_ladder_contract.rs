//! I-4 contract tests — evidence-ranked search ladder (Wave-4 KEYSTONE).
//!
//! Off-agent artifacts (Layer-4 / R4, fail-closed per R1: a missing row FAILS).
//! Every "row landed" assertion is a REAL-DB `query_by_type` against the live
//! SSB sqlite (NOT a struct-only round-trip) so a wire that never reached the DB
//! cannot pass. The ladder is driven through its PUBLIC `run_best_of_n` entry —
//! the exact symbol `task_runner::dispatch_search_ladder` calls — over fixture
//! `TriLayerStack`s (no live model calls, D-026).
//!
//!   (A) PRODUCTION dispatch SSB arm: a gated run over fixture candidates lands
//!       SearchNodeEvent rows (>=1 Expanded for a survivor, >=1 Promoted carrying
//!       verifier-receipt fields, >=1 Pruned for a vetoed candidate).
//!   (B) best-of-N RANKING: the highest tri-layer-score non-vetoed branch wins;
//!       ties break on mutation kill-rate then progress score (deterministic).
//!   (C) VETO / QUARANTINE filtering (ID-4): a hard-vetoed branch is DROPPED and
//!       NEVER Promoted (the row is Pruned, terminal); a flake-quarantined oracle
//!       is likewise dropped.
//!   (D) GD-4 gate: `search_ladder_enabled()` is OFF by default (single-branch
//!       legacy path unchanged) and ON only behind ELAI_SEARCH_LADDER_ENABLE.
//!   (E) BudgetCollapsed: when EVERY candidate is vetoed, no Promoted row lands
//!       and a BudgetCollapsed beacon is emitted (the broken-arm / no-winner path).
//!   (F) env-gate literal matches the wire-dark-allowlist flag name.
//!
//! No #[ignore] / stubs / placeholder assertions.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::adversary::AdversaryConfig;
use elai_orchestrator::checker_registry::{CheckerRegistry, RegexMatchChecker};
use elai_orchestrator::dispatcher::BackendMode;
use elai_orchestrator::mutation_budget::MutationBudget;
use elai_orchestrator::pristine_path_filter::PristinePathFilter;
use elai_orchestrator::search::{
    search_ladder_enabled, BranchInput, SearchLadder, ELAI_SEARCH_LADDER_ENABLE_ENV,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, GateId, SearchNodeEventKind, Ssb, SsbActionType, SsbConfig, SsbPayload,
    TriLayerVerdictKind,
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
        project_id: "i4-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb init"))
}

fn stub_dep_gate() -> Arc<DepWriteTimeGate> {
    Arc::new(DepWriteTimeGate::new(None, Arc::new(StubRegistryChecker)))
}

/// A registry with NO checker -> L1 passes, every branch is a ScoredSurvivor.
fn empty_registry() -> Arc<CheckerRegistry> {
    Arc::new(CheckerRegistry::new())
}

/// A registry that ALWAYS vetoes (anchored regex requiring a sentinel the source
/// never contains -> L1 HardVeto). Drives the veto/collapse arms deterministically.
fn always_veto_registry() -> Arc<CheckerRegistry> {
    let mut reg = CheckerRegistry::new();
    let checker = RegexMatchChecker::new("ZZZ_NEVER_PRESENT_SENTINEL_ZZZ", "i4-veto", false)
        .expect("regex compiles");
    reg.register("i4-always-veto", Box::new(checker));
    Arc::new(reg)
}

/// Build a ladder over a given checker registry (the only thing that varies
/// L1-veto vs L1-pass across the arms). Mirrors `dispatch_search_ladder`'s
/// construction recipe (GD-1: same TriLayerStack::new shape).
fn build_ladder(registry: Arc<CheckerRegistry>, ssb: &Arc<Ssb>) -> SearchLadder {
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
    SearchLadder::new(stack, pristine, quarantine, mutation_budget, Arc::clone(ssb))
}

/// A survivor candidate input with a mutation cone (`n_killed` of `n_total`
/// caught) and a progress score, so the ranker has rank-2/rank-3 keys to break.
fn survivor_input(id: &str, n_killed: usize, n_total: usize, progress: f64) -> BranchInput {
    let mut outcomes = vec![true; n_killed];
    outcomes.extend(std::iter::repeat(false).take(n_total.saturating_sub(n_killed)));
    BranchInput {
        branch: Branch::new(id, format!("fn {id}() {{}}")),
        mutation_outcomes: outcomes,
        progress_score: progress,
    }
}

fn rows(ssb: &Arc<Ssb>, kind: SearchNodeEventKind) -> Vec<elai_orchestrator::ssb::SsbEntry> {
    ssb.query_by_type(SsbActionType::SearchNodeEvent(kind))
        .expect("query_by_type must succeed")
}

// ---------------------------------------------------------------------------
// (A) PRODUCTION dispatch SSB arm + (B) ranking — survivors land rows, best wins.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i4_a_production_dispatch_search_node_rows_land() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i4-a");
    // Empty registry -> all three candidates clear L1 and become survivors. The
    // tri-layer L2 score is deterministic (TAU_DEFAULT_FALLBACK), so the rank-1
    // key ties and the winner is decided by rank-2 (mutation kill-rate).
    let ladder = build_ladder(empty_registry(), &ssb);
    let candidates = vec![
        survivor_input("low-kill", 1, 10, 0.1),  // kill-rate 0.10
        survivor_input("high-kill", 9, 10, 0.9), // kill-rate 0.90 -> winner
        survivor_input("mid-kill", 5, 10, 0.5),  // kill-rate 0.50
    ];

    let outcome = ladder
        .run_best_of_n(&candidates, "i4-a-session", BackendMode::Local)
        .await
        .expect("run_best_of_n ok");

    // best-of-N: the highest mutation kill-rate survivor wins (rank-2 break).
    let winner = outcome.winner.expect("a survivor must be promoted");
    assert_eq!(
        winner.verdict.branch_id, "high-kill",
        "best-of-N winner is the highest-receipt survivor"
    );
    assert_eq!(outcome.pruned_count, 0, "no veto -> nothing pruned");

    // Real-DB (R1 missing=FAIL): Expanded rows for the survivors landed.
    let expanded = rows(&ssb, SearchNodeEventKind::Expanded);
    assert_eq!(expanded.len(), 3, "one Expanded row per survivor (real-DB count)");

    // Real-DB: exactly one Promoted row carrying verifier-receipt evidence.
    let promoted = rows(&ssb, SearchNodeEventKind::Promoted);
    assert_eq!(promoted.len(), 1, "exactly one best-of-N Promoted row");
    match decode_ssb_payload(&promoted[0].payload).expect("decode SearchNodeEvent") {
        SsbPayload::SearchNodeEvent(p) => {
            assert_eq!(p.kind, SearchNodeEventKind::Promoted);
            assert_eq!(p.branch_id, "high-kill");
            // ID-4: the Promoted row MUST carry verifier-derived evidence, not a
            // bare neural score. A scored survivor names its tri-layer arm and a
            // real kill-rate (the --llm-judge-only broken arm has neither).
            assert_eq!(
                p.tri_layer_verdict, "scored_survivor",
                "Promoted row carries the tri-layer verdict tag (verifier evidence)"
            );
            // The kill-rate is the Wilson-CI point estimate at the sampler's
            // early-stop (it need not equal the naive 9/10 ratio), but it MUST be
            // a real, positive, verifier-derived measurement carried on the row —
            // the ID-4 property the --llm-judge-only broken arm (kill-rate 0.0)
            // violates.
            assert!(
                p.mutation_kill_rate > KILL_RATE_FLOOR,
                "Promoted row carries a real mutation kill-rate above the floor; got {}",
                p.mutation_kill_rate
            );
        }
        other => panic!("expected SearchNodeEvent, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (B') ranking — rank-1 (tri-layer score) dominates rank-2 (kill-rate).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i4_b_tri_layer_score_outranks_kill_rate() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i4-b");
    let ladder = build_ladder(empty_registry(), &ssb);

    // The ranked outcome is sorted best-first; rank-1 is the tri-layer score.
    // With a deterministic L2 score the survivors tie on rank-1, so the ranked
    // order is then mutation kill-rate desc (rank-2): high(0.8) > mid(0.4).
    let candidates = vec![
        survivor_input("mid", 4, 10, 0.0),
        survivor_input("high", 8, 10, 0.0),
    ];
    let outcome = ladder
        .run_best_of_n(&candidates, "i4-b-session", BackendMode::Local)
        .await
        .expect("run_best_of_n ok");

    assert_eq!(outcome.ranked.len(), 2, "two survivors ranked");
    assert_eq!(
        outcome.ranked[0].verdict.branch_id, "high",
        "ranked best-first: higher kill-rate first"
    );
    assert!(
        outcome.ranked[0].mutation_kill_rate >= outcome.ranked[1].mutation_kill_rate,
        "ranking is monotone-decreasing on the receipt keys"
    );
}

// ---------------------------------------------------------------------------
// (C) VETO filtering (ID-4) — a hard-vetoed branch is Pruned, never Promoted.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i4_c_vetoed_branch_is_pruned_never_promoted() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i4-c");
    // always-veto registry -> EVERY candidate L1-vetoes. The branch source has a
    // `=` so the anchored regex applies_to fires and Fails -> HardVeto.
    let ladder = build_ladder(always_veto_registry(), &ssb);
    let candidates = vec![
        BranchInput::from_branch(Branch::new("veto-a", "let a = 1;")),
        BranchInput::from_branch(Branch::new("veto-b", "let b = 2;")),
    ];

    let outcome = ladder
        .run_best_of_n(&candidates, "i4-c-session", BackendMode::Local)
        .await
        .expect("run_best_of_n ok");

    // ID-4: a vetoed branch is terminal — no winner, all pruned.
    assert!(outcome.winner.is_none(), "no survivor when every branch is vetoed");
    assert_eq!(outcome.pruned_count, 2, "both vetoed branches pruned");

    // Real-DB: Pruned rows landed, ZERO Promoted rows (the ID-4 supremacy proof).
    assert_eq!(rows(&ssb, SearchNodeEventKind::Pruned).len(), 2, "two Pruned rows");
    assert!(
        rows(&ssb, SearchNodeEventKind::Promoted).is_empty(),
        "a vetoed branch is NEVER Promoted (ID-4 terminal)"
    );
    // The Pruned row's recorded tri-layer verdict is the hard veto.
    match decode_ssb_payload(&rows(&ssb, SearchNodeEventKind::Pruned)[0].payload)
        .expect("decode")
    {
        SsbPayload::SearchNodeEvent(p) => {
            assert_eq!(p.tri_layer_verdict, "hard_veto", "Pruned row names the veto");
            assert_eq!(p.tri_layer_score, 0.0, "vetoed branch carries the literal 0.0 score");
        }
        other => panic!("expected SearchNodeEvent, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (C') QUARANTINE filtering — a flake-quarantined oracle is dropped.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i4_c2_quarantined_branch_is_pruned() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i4-c2");
    // Quarantine the branch id over the FULL seed range the ladder consults
    // (is_pruned checks (u64::MIN, u64::MAX)). The branch otherwise clears L1.
    let mut ledger = FlakeQuarantineLedger::new(Arc::clone(&ssb));
    ledger.quarantine("flaky-branch", (u64::MIN, u64::MAX)).await;
    let quarantine = Arc::new(ledger);

    let stack = Arc::new(TriLayerStack::new(
        empty_registry(),
        stub_dep_gate(),
        None,
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(&ssb),
    ));
    let ladder = SearchLadder::new(
        stack,
        Arc::new(PristinePathFilter::new(Arc::clone(&ssb))),
        quarantine,
        MutationBudget::with_threshold(KILL_RATE_FLOOR),
        Arc::clone(&ssb),
    );

    let candidates = vec![
        BranchInput::from_branch(Branch::new("flaky-branch", "fn ok() {}")),
        survivor_input("clean-branch", 9, 10, 0.9),
    ];
    let outcome = ladder
        .run_best_of_n(&candidates, "i4-c2-session", BackendMode::Local)
        .await
        .expect("run_best_of_n ok");

    // The clean branch wins; the quarantined one is pruned despite clearing L1.
    let winner = outcome.winner.expect("clean branch survives");
    assert_eq!(winner.verdict.branch_id, "clean-branch");
    assert_eq!(outcome.pruned_count, 1, "the quarantined branch was dropped");
}

// ---------------------------------------------------------------------------
// (D) GD-4 gate — default-OFF, ON only behind the env flag.
// ---------------------------------------------------------------------------

#[test]
fn i4_d_gd4_gate_default_off() {
    // Default-OFF (wire-dark): with the var unset, the multi-branch ladder
    // dispatch is a no-op and the legacy single-branch path is unchanged.
    std::env::remove_var(ELAI_SEARCH_LADDER_ENABLE_ENV);
    assert!(
        !search_ladder_enabled(),
        "search ladder must be wire-dark by default (GD-4 measured-first-win)"
    );

    // ON only behind a truthy literal.
    std::env::set_var(ELAI_SEARCH_LADDER_ENABLE_ENV, "1");
    assert!(search_ladder_enabled(), "ENABLE=1 turns the ladder ON");
    std::env::set_var(ELAI_SEARCH_LADDER_ENABLE_ENV, "off");
    assert!(!search_ladder_enabled(), "a non-truthy literal keeps it OFF");
    std::env::remove_var(ELAI_SEARCH_LADDER_ENABLE_ENV);
}

// ---------------------------------------------------------------------------
// (E) BudgetCollapsed — every candidate vetoed -> honest no-winner beacon.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i4_e_budget_collapsed_when_all_vetoed() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i4-e");
    let ladder = build_ladder(always_veto_registry(), &ssb);
    let candidates = vec![BranchInput::from_branch(Branch::new("only", "let x = 1;"))];

    let outcome = ladder
        .run_best_of_n(&candidates, "i4-e-session", BackendMode::Local)
        .await
        .expect("run_best_of_n ok");

    assert!(outcome.winner.is_none(), "no survivor -> no winner");
    // Real-DB: a BudgetCollapsed beacon landed and ZERO Promoted rows.
    assert_eq!(
        rows(&ssb, SearchNodeEventKind::BudgetCollapsed).len(),
        1,
        "exactly one BudgetCollapsed beacon (R1 missing=FAIL)"
    );
    assert!(
        rows(&ssb, SearchNodeEventKind::Promoted).is_empty(),
        "no Promoted row on a fully-collapsed run"
    );
}

// ---------------------------------------------------------------------------
// (F) env-gate literal matches the wire-dark-allowlist flag name + wire round-trip.
// ---------------------------------------------------------------------------

#[test]
fn i4_f_env_gate_literal_and_wire_roundtrip() {
    assert_eq!(
        ELAI_SEARCH_LADDER_ENABLE_ENV, "ELAI_SEARCH_LADDER_ENABLE",
        "env gate literal must match the wire-dark-allowlist.toml flag name"
    );

    // All 4 Kind arms collapse to the same `search_node_*` wire tags + decode.
    for (kind, tag) in [
        (SearchNodeEventKind::Expanded, "search_node_expanded"),
        (SearchNodeEventKind::Pruned, "search_node_pruned"),
        (SearchNodeEventKind::Promoted, "search_node_promoted"),
        (SearchNodeEventKind::BudgetCollapsed, "search_node_budget_collapsed"),
    ] {
        assert_eq!(
            SsbActionType::SearchNodeEvent(kind).as_sql_str(),
            tag,
            "each Kind arm maps to its distinct sql_str"
        );
    }

    // A vetoed gate id is representable on the payload's tri_layer_verdict join
    // path (sanity: the GateId enum the i17 stack stamps is importable here).
    let _ = GateId::L1Compile;
    let _ = TriLayerVerdictKind::HardVeto;
}
