//! H-26 contract tests — shadow-exec + financial-invariant metamorphic-relation gate
//! (T-10).
//!
//! Covers (TEST-FIRST, authored against the live gate surface), over a REAL Ssb
//! SQLite db (off-agent artifact, Layer-4 / R4):
//!   (1) a consistent run (primary==shadow + all invariants hold, non-trivial) =>
//!       PASS (InvariantHeld) with a real DifferentialDivergence SSB row.
//!   (2) a share-conservation / PnL-additivity violation (EXACT integer cents) =>
//!       BLOCK (InvariantViolated) naming the invariant.
//!   (3) a primary-vs-shadow divergence beyond the LOCKED float band => BLOCK
//!       (ShadowDiverged) naming the first divergent field.
//!   (4) an empty / zero-trade run => BLOCK (degenerate guard, NOT a vacuous pass).
//!   (5) LOAD-BEARING: the naked-float-`==` AST rule (FIN-INVARIANT-001) FIRES on a
//!       money-float-`==` fixture (evaluator-running, Block) + does NOT fire on a
//!       benign tolerance compare. This is the anti-wire-dark proof (R6): if the
//!       query/scope is inert, (5a) goes red.
//!   (6) SCHEMA_VERSION >= 232, DifferentialDivergence in the action_type bijection.
//!
//! Tier-1 checks a FIXED set of textbook-correct invariants in-process; it does NOT
//! mine invariants (Daikon) nor runtime-monitor (RV-Monitor/MOP) -- those are
//! design-inspired for the DEFERRED Tier-2 (no completeness-of-enumeration assertion
//! is compiled here as a done-gate).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is
//! asserted by action_type_lock.rs (compile-time exhaustiveness + count-drift) and
//! the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/shadow_exec.rs (H-26 pure gate).
//!   - crates/rules/seed/finance/financial_invariants.toml (H-26 seed, scoped).
//!   - crates/rules/src/project_rule_seeder.rs::seed_financial_invariant_rules.
//!   - crates/orchestrator/src/lookahead_differential.rs (FpTolerance, REUSED).
//!   - crates/orchestrator/src/determinism.rs:364 (first-divergence-index, REUSED).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    shadow_exec::{
        diff_shadow_impls, DerivedFloat, FinancialInvariantInputs, ShadowOutcome,
        ShadowResultSet, ShadowVerdictKind, TradeRecord,
    },
    ssb::{
        ssb_action_type_from_str_public, DifferentialDivergenceKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use elai_knowledge::TreeSitterAstEvaluator;
use elai_rules::predicate::VerdictResult;
use elai_rules::project_rule_seeder::ProjectRuleSeeder;
use elai_rules::rule_engine::{DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine};
use elai_rules::stubs::EmptyDkgQuerier;
use rusqlite::Connection;
use rustc_hash::FxHashMap;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
}

/// A consistent two-trade primary (integer cents): shares conserved,
/// cash+position*price==equity, PnL additive.
fn clean_primary() -> ShadowResultSet {
    ShadowResultSet {
        strategy_path: "pnl_engine.rs".into(),
        run_id: "run-1".into(),
        elai_seed: Some(42),
        trades: vec![
            TradeRecord {
                shares_in: 10,
                shares_out: 10,
                cash_cents: 100_00,
                position_qty: 10,
                price_cents: 50_00,
                equity_cents: 100_00 + 10 * 50_00,
                realized_pnl_cents: 1_50,
            },
            TradeRecord {
                shares_in: 5,
                shares_out: 5,
                cash_cents: 200_00,
                position_qty: 4,
                price_cents: 25_00,
                equity_cents: 200_00 + 4 * 25_00,
                realized_pnl_cents: 2_25,
            },
        ],
        declared_total_pnl_cents: 1_50 + 2_25,
        derived_floats: vec![DerivedFloat {
            name: "option_price".into(),
            value: 12.345_678_9,
        }],
    }
}

fn clean_shadow() -> ShadowOutcome {
    ShadowOutcome::Produced(Box::new(ShadowResultSet {
        derived_floats: vec![DerivedFloat {
            name: "option_price".into(),
            value: 12.345_678_9,
        }],
        ..clean_primary()
    }))
}

fn only_row(ssb: &Arc<Ssb>) -> SsbPayload {
    let rows = ssb
        .query_by_type(SsbActionType::DifferentialDivergence)
        .expect("query DifferentialDivergence");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 DifferentialDivergence SSB row (off-agent artifact, R4)"
    );
    rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload")
}

// ---------------------------------------------------------------------------
// (1) consistent run => PASS (InvariantHeld) + off-agent SSB row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consistent_run_holds_invariants_pass_and_emits_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h26-contract-1");

    let verdict = diff_shadow_impls(
        &ssb,
        clean_primary(),
        clean_shadow(),
        FinancialInvariantInputs::default(),
        42,
    )
    .await
    .expect("diff must not error");
    assert_eq!(
        verdict.kind,
        ShadowVerdictKind::InvariantHeld,
        "a consistent non-degenerate run whose invariants hold must PASS InvariantHeld"
    );
    assert_eq!(verdict.reason, "invariant_held");
    assert_eq!(verdict.declared_trade_count, 2);

    if let SsbPayload::DifferentialDivergence(p) = only_row(&ssb) {
        assert_eq!(p.kind, DifferentialDivergenceKind::InvariantHeld);
        assert_eq!(p.strategy_path, "pnl_engine.rs");
        assert_eq!(p.verdict, "invariant_held");
        assert_eq!(p.declared_trade_count, 2);
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
        assert!(!p.primary_digest.is_empty(), "InvariantHeld carries a primary_digest");
    } else {
        panic!("payload must be DifferentialDivergence");
    }
}

// ---------------------------------------------------------------------------
// (2) accounting-identity violation (EXACT integer cents) => BLOCK naming it
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pnl_additivity_violation_blocks_exact_integer() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h26-contract-2a");

    let mut p = clean_primary();
    // Break PnL additivity by exactly 1 cent -- the EXACT integer-cents regime catches
    // a cent-level error a float tolerance would hide.
    p.declared_total_pnl_cents += 1;

    let verdict = diff_shadow_impls(&ssb, p, clean_shadow(), FinancialInvariantInputs::default(), 42)
        .await
        .expect("diff");
    assert_eq!(verdict.kind, ShadowVerdictKind::InvariantViolated);
    assert_eq!(verdict.reason, "pnl_additivity");
    assert_eq!(verdict.offending_invariant, "pnl_additivity");

    if let SsbPayload::DifferentialDivergence(p) = only_row(&ssb) {
        assert_eq!(p.kind, DifferentialDivergenceKind::InvariantViolated);
        assert_eq!(p.offending_invariant, "pnl_additivity");
    } else {
        panic!("payload must be DifferentialDivergence");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn share_conservation_violation_blocks_exact_integer() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h26-contract-2b");

    let mut p = clean_primary();
    p.trades[0].shares_out = 9; // shares not conserved

    let verdict = diff_shadow_impls(&ssb, p, clean_shadow(), FinancialInvariantInputs::default(), 42)
        .await
        .expect("diff");
    assert_eq!(verdict.kind, ShadowVerdictKind::InvariantViolated);
    assert_eq!(verdict.reason, "share_conservation");
    assert_eq!(verdict.offending_invariant, "share_conservation");
}

// ---------------------------------------------------------------------------
// (3) primary-vs-shadow divergence beyond the band => BLOCK naming the field
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shadow_divergence_beyond_band_blocks() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h26-contract-3");

    // A sign-flipped derived value: diverges far beyond the LOCKED band.
    let shadow = ShadowOutcome::Produced(Box::new(ShadowResultSet {
        derived_floats: vec![DerivedFloat {
            name: "option_price".into(),
            value: -12.345_678_9,
        }],
        ..clean_primary()
    }));

    let verdict = diff_shadow_impls(
        &ssb,
        clean_primary(),
        shadow,
        FinancialInvariantInputs::default(),
        42,
    )
    .await
    .expect("diff");
    assert_eq!(verdict.kind, ShadowVerdictKind::ShadowDiverged);
    assert_eq!(verdict.reason, "shadow_diverged");
    assert_eq!(verdict.first_divergent_field, "option_price");

    if let SsbPayload::DifferentialDivergence(p) = only_row(&ssb) {
        assert_eq!(p.kind, DifferentialDivergenceKind::ShadowDiverged);
        assert_eq!(p.first_divergent_field, "option_price");
    } else {
        panic!("payload must be DifferentialDivergence");
    }
}

// ---------------------------------------------------------------------------
// (4) empty / zero-trade run => BLOCK (degenerate guard, not a vacuous pass)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn degenerate_empty_run_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h26-contract-4");

    let empty = ShadowResultSet {
        strategy_path: "pnl_engine.rs".into(),
        run_id: "run-1".into(),
        elai_seed: Some(42),
        trades: vec![],
        declared_total_pnl_cents: 0,
        derived_floats: vec![],
    };
    let shadow = ShadowOutcome::Produced(Box::new(empty.clone()));

    let verdict = diff_shadow_impls(&ssb, empty, shadow, FinancialInvariantInputs::default(), 42)
        .await
        .expect("diff");
    assert_eq!(
        verdict.kind,
        ShadowVerdictKind::InvariantViolated,
        "a zero-trade run must Block (degenerate guard), NOT pass vacuously"
    );
    assert_ne!(verdict.kind, ShadowVerdictKind::InvariantHeld);
    assert_eq!(verdict.reason, "degenerate_run");
}

// ---------------------------------------------------------------------------
// (5) LOAD-BEARING: RUN the rule engine over the seeded FIN-INVARIANT-001 naked-
//     float-`==` ban. Drives the DefaultRuleEngine (with the real
//     TreeSitterAstEvaluator) over real fixtures and asserts the rule FIRES on an
//     in-scope execution module with a naked float-`==` on money, and does NOT fire
//     on a benign tolerance compare. The anti-wire-dark proof (R6): if the
//     query/scope is inert, (5a) goes red.
// ---------------------------------------------------------------------------

fn seeded_rules_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory rules db");
    conn.execute_batch(
        "CREATE TABLE rules (
            rule_id TEXT PRIMARY KEY,
            domain TEXT NOT NULL,
            tier INTEGER NOT NULL,
            status TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            predicate TEXT NOT NULL,
            phase_mask INTEGER NOT NULL,
            severity TEXT NOT NULL,
            mutable INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            source TEXT NOT NULL
        );",
    )
    .expect("create rules table");
    let inserted = ProjectRuleSeeder::seed_financial_invariant_rules(&conn, &OsClock)
        .expect("seed financial invariant rules");
    assert_eq!(inserted, 1, "one LOCKED FIN-INVARIANT-001 row seeded");
    conn
}

fn fin_invariant_001_engine_and_rule(
) -> (DefaultRuleEngine, Rule, elai_rules::rule_engine::CompiledPredicate) {
    let conn = seeded_rules_db();
    let (predicate_json, domain, tier, severity, title, phase_mask): (
        String,
        String,
        u8,
        String,
        String,
        u32,
    ) = conn
        .query_row(
            "SELECT predicate, domain, tier, severity, title, phase_mask \
             FROM rules WHERE rule_id = 'FIN-INVARIANT-001'",
            [],
            |row| {
                let tier_i: i64 = row.get(2)?;
                let mask_i: i64 = row.get(5)?;
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    u8::try_from(tier_i).unwrap_or(1),
                    row.get(3)?,
                    row.get(4)?,
                    u32::try_from(mask_i).unwrap_or(u32::MAX),
                ))
            },
        )
        .expect("FIN-INVARIANT-001 must be seeded");

    let engine = DefaultRuleEngine::new(
        Box::new(TreeSitterAstEvaluator::new()),
        Box::new(EmptyDkgQuerier),
    );
    let compiled = engine
        .compile_predicate(&predicate_json)
        .expect("FIN-INVARIANT-001 predicate must compile");
    let rule = Rule {
        rule_id: "FIN-INVARIANT-001".to_string(),
        domain,
        tier,
        severity,
        title,
        predicate_json,
        phase_mask,
        escalate_target: None,
    };
    (engine, rule, compiled)
}

fn eval_context<'a>(source: &'a str, file_path: &'a str) -> EvaluationContext<'a> {
    EvaluationContext {
        action: "write",
        source,
        file_path,
        fsm_phase: FsmPhaseBit::TESTING_FIRST,
        metric_values: FxHashMap::default(),
        language_agnostic: false,
    }
}

#[test]
fn naked_float_eq_ban_fires_on_in_scope_money_float_eq() {
    let (engine, rule, compiled) = fin_invariant_001_engine_and_rule();

    // Execution code that compares a float money/price quantity with a naked `==`
    // against a float literal: the precision-bug antipattern. The file is in scope
    // (its path contains an execution token, e.g. "pnl") AND the body matches the AST
    // query (a `== <float_literal>` binary_expression).
    let src = "fn settle(pnl: f64) -> bool {\n    pnl == 0.0\n}\n";
    let ctx = eval_context(src, "strategy/pnl_engine.rs");

    let verdicts = engine
        .evaluate(&[(rule, compiled)], &ctx)
        .expect("evaluate must not error");

    // R6 anti-wire-dark: the LOCKED ban MUST produce a Block here. If the query is
    // wrong or the scope is inert, this is Allow and the test fails.
    assert!(
        verdicts.iter().any(|v| v.result == VerdictResult::Block),
        "FIN-INVARIANT-001 MUST FIRE (Block) on an in-scope naked float-== on money; \
         got {verdicts:?}. If this is Allow, the AST ban is runtime-inert (wire-dark)."
    );
}

#[test]
fn naked_float_eq_ban_does_not_fire_on_tolerance_compare() {
    let (engine, rule, compiled) = fin_invariant_001_engine_and_rule();

    // A correct tolerance compare in the SAME in-scope execution module: it must NOT
    // fire (no false-block on the right way to compare floats).
    let src = "fn close(a: f64, b: f64) -> bool {\n    (a - b).abs() < 1e-9\n}\n";
    let ctx = eval_context(src, "strategy/pnl_engine.rs");

    let verdicts = engine
        .evaluate(&[(rule, compiled)], &ctx)
        .expect("evaluate must not error");

    assert!(
        verdicts.iter().all(|v| v.result == VerdictResult::Allow),
        "FIN-INVARIANT-001 must NOT fire on a tolerance compare; got {verdicts:?}"
    );
}

// ---------------------------------------------------------------------------
// (6) SSB lockstep: SCHEMA_VERSION >= 232, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 232,
        "SCHEMA_VERSION must be >= 232 after H-26; got {SCHEMA_VERSION_PUBLIC}"
    );

    // DifferentialDivergence must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("differential_divergence")
        .expect("differential_divergence must be a known action_type");
    assert_eq!(at, SsbActionType::DifferentialDivergence);
    assert_eq!(
        SsbActionType::DifferentialDivergence.as_sql_str(),
        "differential_divergence"
    );
}
