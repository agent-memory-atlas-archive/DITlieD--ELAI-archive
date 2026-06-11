//! H-24 contract tests — sim-to-real parity gate (T-8).
//!
//! Covers (TEST-FIRST, authored against the live gate surface), over a REAL Ssb
//! SQLite db (off-agent artifact, Layer-4 / R4):
//!   (1) replay == golden => PASS (Parity) with a real SimRealParity SSB row
//!       Kind=Parity.
//!   (2) replay != golden => BLOCK (ReplayGoldenMismatch) reporting the FIRST
//!       divergent event (Kind=ReplayGoldenMismatch).
//!   (3) a mode-divergent branch in EXECUTION scope => the Part-A AST rule flags
//!       it (the seeded FIN-SIMREAL-001 row carries an execution-path scope).
//!   (4) a mode-branch in a LEGITIMATE (logging/notification) scope does NOT
//!       false-block (the scope field is present and bounded, not blanket).
//!   (5) missing single-entrypoint manifest => the Part-A SchemaRequired rule
//!       (FIN-SIMREAL-002) requires the manifest keys (Block on missing).
//!   (6) missing golden / missing replay => Block (fail-closed, W-1/R1).
//!   (7) SCHEMA_VERSION >= 229, SimRealParity in the action_type bijection.
//!
//! Part B (numeric live-vs-backtest reconciliation) is DEFERRED Tier-2 (GD-5):
//! NO live-parity assertion is compiled here as a done-gate (structural A+C are
//! necessary-not-sufficient).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is
//! asserted by action_type_lock.rs (compile-time exhaustiveness + count-drift)
//! and the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/sim_real_parity.rs (H-24 Part-C gate).
//!   - crates/rules/seed/repro/sim_real_parity.toml (H-24 Part-A seeds, scoped).
//!   - crates/rules/src/project_rule_seeder.rs::seed_sim_real_parity_rules.
//!   - crates/orchestrator/src/determinism.rs:364 (first-divergence-index, REUSED).
//!   - workflow/papers/lean-sim-real-parity-interface.md (LEAN scoping ground).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    sim_real_parity::{verify_replay_golden, EventLog, SimRealInput, SimRealVerdictKind},
    ssb::{
        ssb_action_type_from_str_public, SimRealParityKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use elai_knowledge::TreeSitterAstEvaluator;
use elai_rules::predicate::{Predicate, VerdictResult};
use elai_rules::project_rule_seeder::ProjectRuleSeeder;
use elai_rules::rule_engine::{
    DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
};
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

fn el(pairs: Vec<(&str, &str)>) -> EventLog {
    EventLog::from_pairs(
        pairs
            .into_iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect(),
    )
}

fn input(golden: EventLog, replay: Option<EventLog>, seed: u64) -> SimRealInput {
    SimRealInput {
        strategy_path: "strat.py".to_string(),
        golden,
        replay,
        seed,
    }
}

fn only_row(ssb: &Arc<Ssb>) -> SsbPayload {
    let rows = ssb
        .query_by_type(SsbActionType::SimRealParity)
        .expect("query SimRealParity");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 SimRealParity SSB row (off-agent artifact, R4)"
    );
    rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload")
}

/// Seed the H-24 Part-A rules into a fresh rules.db and return the connection.
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
    let inserted = ProjectRuleSeeder::seed_sim_real_parity_rules(&conn, &OsClock)
        .expect("seed sim-real parity rules");
    assert_eq!(inserted, 2, "two LOCKED Part-A rows seeded");
    conn
}

// ---------------------------------------------------------------------------
// (1) replay == golden => PASS (Parity) + off-agent SSB row Kind=Parity (R4)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replay_equals_golden_passes_parity_and_emits_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h24-contract-1");

    let golden = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "bbbb")]);
    let replay = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "bbbb")]);

    let verdict = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
        .await
        .expect("verify must not error");
    assert_eq!(
        verdict.kind,
        SimRealVerdictKind::Parity,
        "a seeded replay that reproduces the golden must PASS Parity"
    );
    assert_eq!(verdict.reason, "parity");
    assert_eq!(verdict.declared_event_count, 2);

    if let SsbPayload::SimRealParity(p) = only_row(&ssb) {
        assert_eq!(p.kind, SimRealParityKind::Parity);
        assert_eq!(p.strategy_path, "strat.py");
        assert_eq!(p.verdict, "parity");
        assert_eq!(p.declared_event_count, 2);
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
        assert!(!p.golden_digest.is_empty(), "Parity carries a golden_digest");
        assert_eq!(
            p.golden_digest, p.replay_digest,
            "Parity => golden_digest == replay_digest"
        );
    } else {
        panic!("payload must be SimRealParity");
    }
}

// ---------------------------------------------------------------------------
// (2) replay != golden => BLOCK (ReplayGoldenMismatch) + first event named
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn divergent_replay_blocks_naming_first_event() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h24-contract-2");

    let golden = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "bbbb")]);
    // replay's fill digest differs (a different backtest fill model).
    let replay = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "cccc")]);

    let verdict = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
        .await
        .expect("verify");
    assert_eq!(verdict.kind, SimRealVerdictKind::ReplayGoldenMismatch);
    assert_eq!(verdict.reason, "replay_golden_mismatch");
    assert_eq!(
        verdict.divergent_event, "fill:AAPL",
        "the FIRST divergent event must be reported"
    );
    assert_eq!(verdict.first_divergence_index, 1);

    if let SsbPayload::SimRealParity(p) = only_row(&ssb) {
        assert_eq!(p.kind, SimRealParityKind::ReplayGoldenMismatch);
        assert_eq!(p.divergent_event, "fill:AAPL");
        assert_eq!(p.first_divergence_index, 1);
        assert_eq!(p.verdict, "replay_golden_mismatch");
    } else {
        panic!("payload must be SimRealParity");
    }
}

// ---------------------------------------------------------------------------
// (3) a mode-divergent branch in EXECUTION scope => the Part-A AST rule flags it
//     (the seeded FIN-SIMREAL-001 carries an execution-path scope + a query that
//     matches the if-LiveMode / if-backtest branch).
// ---------------------------------------------------------------------------

#[test]
fn part_a_ast_ban_is_seeded_with_execution_path_scope() {
    let conn = seeded_rules_db();
    let (predicate_json, scope_ok): (String, bool) = conn
        .query_row(
            "SELECT predicate, severity FROM rules WHERE rule_id = 'FIN-SIMREAL-001'",
            [],
            |row| {
                let p: String = row.get(0)?;
                let sev: String = row.get(1)?;
                Ok((p, sev == "BLOCK"))
            },
        )
        .expect("FIN-SIMREAL-001 must be seeded");
    assert!(scope_ok, "the AST ban must be a BLOCK-severity rule");

    let predicate: Predicate =
        serde_json::from_str(&predicate_json).expect("predicate must deserialize");
    match predicate {
        Predicate::AstPattern {
            language,
            query,
            scope,
            ..
        } => {
            assert_eq!(language, "python");
            // The query must match a mode-divergent branch (LiveMode / live /
            // backtest / mode).
            assert!(
                query.contains("LiveMode") && query.contains("backtest"),
                "the AST query must flag a LiveMode / backtest mode-divergent branch"
            );
            // CRITICAL (LEAN scoping): the ban must be SCOPED to the execution
            // path, NOT blanket. A None/empty scope would false-block every
            // legitimate notification/logging LiveMode conditional.
            let scope = scope.expect("the AST ban MUST carry an execution-path scope (not blanket)");
            assert!(
                scope.contains("order") && scope.contains("signal"),
                "the scope must narrow the ban to the order/signal execution path \
                 (LEAN: LiveMode is legitimate for notifications/logging)"
            );
        }
        other => panic!("FIN-SIMREAL-001 must be an AstPattern, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (4) a mode-branch in a LEGITIMATE (logging/notification) scope does NOT
//     false-block: the ban's scope is BOUNDED (not blanket) — it does not list
//     notification/logging/display, so those modules are out of scope.
// ---------------------------------------------------------------------------

#[test]
fn part_a_ast_ban_scope_excludes_legitimate_notification_logging() {
    let conn = seeded_rules_db();
    let predicate_json: String = conn
        .query_row(
            "SELECT predicate FROM rules WHERE rule_id = 'FIN-SIMREAL-001'",
            [],
            |row| row.get(0),
        )
        .expect("FIN-SIMREAL-001 must be seeded");
    let predicate: Predicate =
        serde_json::from_str(&predicate_json).expect("predicate must deserialize");
    if let Predicate::AstPattern { scope, .. } = predicate {
        let scope = scope.expect("scope present");
        // The scope is an execution-path allowlist. It must NOT name
        // notification/logging/display surfaces — those legitimate LiveMode uses
        // are out of scope and therefore not flagged (no blanket false-block).
        assert!(
            !scope.contains("notif")
                && !scope.contains("logging")
                && !scope.contains("display"),
            "the execution-path scope must not include notification/logging/display \
             (those are LEGITIMATE LiveMode uses per LEAN; including them would \
             false-block)"
        );
        // Sanity: the scope is bounded (not the empty/universal string).
        assert!(!scope.is_empty(), "the scope must be a bounded allowlist, not blanket");
    } else {
        panic!("FIN-SIMREAL-001 must be an AstPattern");
    }
}

// ---------------------------------------------------------------------------
// (3b)/(4b) LOAD-BEARING: RUN the rule engine over the seeded FIN-SIMREAL-001
//     AST ban. Tests (3)/(4) above only assert the scope STRING; they never run
//     the evaluator, so they pass even when the ban is RUNTIME-INERT. These two
//     tests drive the DefaultRuleEngine (with the real TreeSitterAstEvaluator)
//     over real fixtures and assert the rule FIRES on an in-scope execution-path
//     module and does NOT fire on an out-of-scope logging module. This is the
//     anti-wire-dark proof (R6): if the scope is inert, (3b) goes red.
// ---------------------------------------------------------------------------

/// Build a `DefaultRuleEngine` with the production tree-sitter AST evaluator and
/// load the seeded FIN-SIMREAL-001 (Rule + CompiledPredicate) from a fresh
/// rules.db, so the test exercises the EXACT shipped scope + query.
fn fin_simreal_001_engine_and_rule() -> (DefaultRuleEngine, Rule, elai_rules::rule_engine::CompiledPredicate)
{
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
             FROM rules WHERE rule_id = 'FIN-SIMREAL-001'",
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
        .expect("FIN-SIMREAL-001 must be seeded");

    let engine = DefaultRuleEngine::new(
        Box::new(TreeSitterAstEvaluator::new()),
        Box::new(EmptyDkgQuerier),
    );
    let compiled = engine
        .compile_predicate(&predicate_json)
        .expect("FIN-SIMREAL-001 predicate must compile");
    let rule = Rule {
        rule_id: "FIN-SIMREAL-001".to_string(),
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
fn part_a_ast_ban_fires_on_in_scope_execution_mode_branch() {
    let (engine, rule, compiled) = fin_simreal_001_engine_and_rule();

    // A mode-divergent branch guarding ORDER EMISSION on the execution path:
    // the dangerous antipattern (LEAN). The file is in scope (path contains an
    // execution token) AND the body matches the AST query.
    let src = "def OnData(self, slice):\n    if self.LiveMode:\n        self.MarketOrder(\"AAPL\", 100)\n";
    let ctx = eval_context(src, "strategy/order_emission.py");

    let verdicts = engine
        .evaluate(&[(rule, compiled)], &ctx)
        .expect("evaluate must not error");

    // R6 anti-wire-dark: the LOCKED ban MUST produce a Block here. If the scope
    // is inert (legacy starts_with semantics), this is Allow and the test fails.
    assert!(
        verdicts
            .iter()
            .any(|v| v.result == VerdictResult::Block),
        "FIN-SIMREAL-001 MUST FIRE (Block) on an in-scope execution-path \
         mode-divergent branch; got {verdicts:?}. If this is Allow, the AST ban \
         is runtime-inert (wire-dark)."
    );
}

#[test]
fn part_a_ast_ban_does_not_fire_on_out_of_scope_logging() {
    let (engine, rule, compiled) = fin_simreal_001_engine_and_rule();

    // The SAME mode-branch shape, but in a LOGGING module (legitimate LiveMode
    // use per LEAN: notifications/logging/runtime-stats). The file is OUT of
    // scope (its path contains no execution token), so the ban must NOT fire —
    // no false-block.
    let src = "def emit(self):\n    if self.LiveMode:\n        log.info(\"live run started\")\n";
    let ctx = eval_context(src, "utils/logging.py");

    let verdicts = engine
        .evaluate(&[(rule, compiled)], &ctx)
        .expect("evaluate must not error");

    assert!(
        verdicts
            .iter()
            .all(|v| v.result == VerdictResult::Allow),
        "FIN-SIMREAL-001 must NOT fire on an out-of-scope logging module \
         (legitimate LiveMode use per LEAN); got {verdicts:?}"
    );
}

// ---------------------------------------------------------------------------
// (5) missing single-entrypoint manifest => the Part-A SchemaRequired rule
//     (FIN-SIMREAL-002) requires the manifest keys.
// ---------------------------------------------------------------------------

#[test]
fn part_a_single_entrypoint_schema_required_is_seeded() {
    let conn = seeded_rules_db();
    let predicate_json: String = conn
        .query_row(
            "SELECT predicate FROM rules WHERE rule_id = 'FIN-SIMREAL-002'",
            [],
            |row| row.get(0),
        )
        .expect("FIN-SIMREAL-002 must be seeded");
    let predicate: Predicate =
        serde_json::from_str(&predicate_json).expect("predicate must deserialize");
    match predicate {
        Predicate::SchemaRequired {
            schema,
            output_type,
        } => {
            assert_eq!(output_type, "sim_real_entrypoint_manifest");
            let required = schema
                .get("required")
                .and_then(|r| r.as_array())
                .expect("schema.required must be an array");
            let keys: Vec<String> = required
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
            // The single-entrypoint manifest must force a single shared entrypoint
            // + the swappable adapters (LEAN adapter-swap pattern). A missing
            // manifest => the SchemaRequired predicate Blocks.
            for key in [
                "entrypoint",
                "init_entrypoint",
                "data_source_adapter",
                "fill_model_adapter",
                "clock_adapter",
            ] {
                assert!(
                    keys.iter().any(|k| k == key),
                    "single-entrypoint manifest must require '{key}'"
                );
            }
        }
        other => panic!("FIN-SIMREAL-002 must be a SchemaRequired, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (6) missing golden / missing replay => Block (fail-closed, W-1/R1)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_replay_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h24-contract-6a");

    let golden = el(vec![("order:AAPL", "aaaa")]);
    let verdict = verify_replay_golden(&ssb, input(golden, None, 42))
        .await
        .expect("verify");
    assert_eq!(
        verdict.kind,
        SimRealVerdictKind::ReplayGoldenMismatch,
        "a missing replay must Block (a single artifact cannot self-certify, W-1/R1)"
    );
    assert_eq!(verdict.reason, "missing_replay");

    if let SsbPayload::SimRealParity(p) = only_row(&ssb) {
        assert_eq!(p.kind, SimRealParityKind::ReplayGoldenMismatch);
        assert_eq!(p.verdict, "missing_replay");
    } else {
        panic!("payload must be SimRealParity");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_or_empty_golden_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h24-contract-6b");

    // An empty golden would hash equal to an empty replay (blake3 of empty is a
    // fixed constant); Block (hazard B), never a silent pass.
    let golden = EventLog::default();
    let replay = EventLog::default();
    let verdict = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
        .await
        .expect("verify");
    assert_eq!(
        verdict.kind,
        SimRealVerdictKind::ReplayGoldenMismatch,
        "an empty golden must Block (hazard B), NOT pass Parity"
    );
    assert_ne!(verdict.kind, SimRealVerdictKind::Parity);
    assert_eq!(verdict.reason, "empty_golden");
}

// ---------------------------------------------------------------------------
// (7) SSB lockstep: SCHEMA_VERSION >= 229, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 229,
        "SCHEMA_VERSION must be >= 229 after H-24; got {SCHEMA_VERSION_PUBLIC}"
    );

    // SimRealParity must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("sim_real_parity")
        .expect("sim_real_parity must be a known action_type");
    assert_eq!(at, SsbActionType::SimRealParity);
    assert_eq!(
        SsbActionType::SimRealParity.as_sql_str(),
        "sim_real_parity"
    );
}
