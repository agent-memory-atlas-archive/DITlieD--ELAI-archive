//! H-25 contract tests — result manifest re-derivation gate (T-9).
//!
//! Covers (TEST-FIRST, authored against the live gate surface), over a REAL Ssb
//! SQLite db (off-agent artifact, Layer-4 / R4):
//!   (1) a complete manifest re-derives => PASS (ReDerived) with a real
//!       ResultManifestVerified SSB row Kind=ReDerived.
//!   (2) a tampered (hand-edited) value => BLOCK (FabricationDetected) reporting the
//!       FIRST result that did not re-derive (Kind=FabricationDetected).
//!   (3) an empty/trivial manifest => BLOCK (anti-sandbagging, the F-4 None-hash
//!       analog).
//!   (4) a reported number with a missing captured output (None) => BLOCK (R2
//!       missing-evidence=FAIL).
//!   (5) LOAD-BEARING: the monkeypatch AST rule (FIN-RESMANIFEST-002) FIRES on a
//!       result-fn-monkeypatch fixture (evaluator-running, Block) + does NOT fire on
//!       a benign fixture. This is the anti-wire-dark proof (R6): if the scope is
//!       inert, (5a) goes red. Plus the SchemaRequired shape rule
//!       (FIN-RESMANIFEST-001) is seeded.
//!   (6) SCHEMA_VERSION >= 230, ResultManifestVerified in the action_type bijection.
//!
//! Tier-1 proves every DECLARED result re-derives + the manifest is non-trivial; it
//! does NOT prove the declaration is COMPLETE (no hidden output). That
//! produced-vs-declared diff is the DEFERRED Tier-2 runner (TD-5 honest residue);
//! NO completeness-of-enumeration assertion is compiled here as a done-gate.
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is
//! asserted by action_type_lock.rs (compile-time exhaustiveness + count-drift) and
//! the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/result_manifest.rs (H-25 pure gate).
//!   - crates/rules/seed/repro/result_manifest.toml (H-25 seeds, scoped).
//!   - crates/rules/src/project_rule_seeder.rs::seed_result_manifest_rules.
//!   - crates/orchestrator/src/determinism.rs:364 (first-divergence-index, REUSED).
//!   - crates/orchestrator/src/evidence.rs:845 + gate.rs:441 (the F-4 anti-fab leg).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    result_manifest::{
        verify_result, ReDerivedInputs, ResultEntry, ResultManifest, ResultVerdictKind,
    },
    ssb::{
        ssb_action_type_from_str_public, ResultManifestVerifiedKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use elai_knowledge::TreeSitterAstEvaluator;
use elai_rules::predicate::{Predicate, VerdictResult};
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

fn blake3_16hex(bytes: &[u8]) -> String {
    let full = blake3::hash(bytes).to_hex();
    full.chars().take(16).collect()
}

/// A captured-output blob carrying two labeled floats (the off-agent artifact).
fn demo_captured() -> Vec<u8> {
    b"backtest complete\nsharpe=1.85\nmaxdd=0.07\n".to_vec()
}

const DATA_HASH: &str = "data0000";
const ENV_HASH: &str = "env00000";
const COMMIT: &str = "commitabc";

fn entry(name: &str, value: &str, captured: &[u8]) -> ResultEntry {
    ResultEntry {
        metric_name: name.to_string(),
        value: value.to_string(),
        captured_output_blake3: blake3_16hex(captured),
        parser_expr: format!("labeled-float:{name}="),
        data_hash: DATA_HASH.to_string(),
        code_commit: COMMIT.to_string(),
        env_lock_hash: ENV_HASH.to_string(),
        n_trials: 100,
        dsr_pvalue: Some(0.01),
        pbo: Some(0.12),
    }
}

fn rederived(captured: &[u8]) -> ReDerivedInputs {
    ReDerivedInputs {
        captured_outputs: vec![
            ("sharpe".to_string(), captured.to_vec()),
            ("maxdd".to_string(), captured.to_vec()),
        ],
        data_bytes_hash: DATA_HASH.to_string(),
        env_bytes_hash: ENV_HASH.to_string(),
        code_commit_observed: COMMIT.to_string(),
    }
}

fn only_row(ssb: &Arc<Ssb>) -> SsbPayload {
    let rows = ssb
        .query_by_type(SsbActionType::ResultManifestVerified)
        .expect("query ResultManifestVerified");
    assert_eq!(
        rows.len(),
        1,
        "exactly 1 ResultManifestVerified SSB row (off-agent artifact, R4)"
    );
    rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload")
}

/// Seed the H-25 Part-A rules into a fresh rules.db and return the connection.
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
    let inserted = ProjectRuleSeeder::seed_result_manifest_rules(&conn, &OsClock)
        .expect("seed result manifest rules");
    assert_eq!(inserted, 2, "two LOCKED Part-A rows seeded");
    conn
}

// ---------------------------------------------------------------------------
// (1) complete manifest re-derives => PASS (ReDerived) + off-agent SSB row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complete_manifest_rederives_pass_and_emits_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h25-contract-1");

    let cap = demo_captured();
    let manifest = ResultManifest {
        strategy_path: "exp.py".into(),
        run_id: "run-1".into(),
        elai_seed: Some(42),
        entries: vec![entry("sharpe", "1.85", &cap), entry("maxdd", "0.07", &cap)],
    };

    let verdict = verify_result(&ssb, manifest, Some(rederived(&cap)))
        .await
        .expect("verify must not error");
    assert_eq!(
        verdict.kind,
        ResultVerdictKind::ReDerived,
        "a manifest whose declared numbers re-parse from the captured output must PASS ReDerived"
    );
    assert_eq!(verdict.reason, "re_derived");
    assert_eq!(verdict.declared_result_count, 2);

    if let SsbPayload::ResultManifestVerified(p) = only_row(&ssb) {
        assert_eq!(p.kind, ResultManifestVerifiedKind::ReDerived);
        assert_eq!(p.strategy_path, "exp.py");
        assert_eq!(p.run_id, "run-1");
        assert_eq!(p.verdict, "re_derived");
        assert_eq!(p.declared_result_count, 2);
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
        assert!(!p.manifest_digest.is_empty(), "ReDerived carries a manifest_digest");
    } else {
        panic!("payload must be ResultManifestVerified");
    }
}

// ---------------------------------------------------------------------------
// (2) tampered (hand-edited) value => BLOCK (FabricationDetected) + first named
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tampered_value_blocks_naming_first_result() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h25-contract-2");

    let cap = demo_captured();
    // The declared sharpe was HAND-EDITED to 2.99 (the captured output says 1.85).
    let manifest = ResultManifest {
        strategy_path: "exp.py".into(),
        run_id: "run-1".into(),
        elai_seed: Some(42),
        entries: vec![entry("sharpe", "2.99", &cap), entry("maxdd", "0.07", &cap)],
    };

    let verdict = verify_result(&ssb, manifest, Some(rederived(&cap)))
        .await
        .expect("verify");
    assert_eq!(verdict.kind, ResultVerdictKind::FabricationDetected);
    assert_eq!(verdict.reason, "value_mismatch");
    assert_eq!(
        verdict.offending_metric, "sharpe",
        "the FIRST non-re-deriving result must be reported"
    );
    assert_eq!(verdict.first_divergence_index, 0);

    if let SsbPayload::ResultManifestVerified(p) = only_row(&ssb) {
        assert_eq!(p.kind, ResultManifestVerifiedKind::FabricationDetected);
        assert_eq!(p.offending_metric, "sharpe");
        assert_eq!(p.verdict, "value_mismatch");
    } else {
        panic!("payload must be ResultManifestVerified");
    }
}

// ---------------------------------------------------------------------------
// (3) empty/trivial manifest => BLOCK (anti-sandbagging, the F-4 None-hash analog)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_manifest_is_fail_closed_block_anti_sandbagging() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h25-contract-3");

    let manifest = ResultManifest {
        strategy_path: "exp.py".into(),
        run_id: "run-1".into(),
        elai_seed: Some(42),
        entries: vec![],
    };
    let verdict = verify_result(&ssb, manifest, Some(ReDerivedInputs::default()))
        .await
        .expect("verify");
    assert_eq!(
        verdict.kind,
        ResultVerdictKind::FabricationDetected,
        "a 0/empty manifest must Block (anti-sandbagging), NOT pass ReDerived"
    );
    assert_ne!(verdict.kind, ResultVerdictKind::ReDerived);
    assert_eq!(verdict.reason, "trivial_manifest");
}

// ---------------------------------------------------------------------------
// (4) reported number with a missing captured output (None) => BLOCK (R2)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_captured_output_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h25-contract-4");

    let cap = demo_captured();
    let manifest = ResultManifest {
        strategy_path: "exp.py".into(),
        run_id: "run-1".into(),
        elai_seed: Some(42),
        entries: vec![entry("sharpe", "1.85", &cap)],
    };
    // rederived = None: the orchestrator-captured output never landed.
    let verdict = verify_result(&ssb, manifest, None).await.expect("verify");
    assert_eq!(
        verdict.kind,
        ResultVerdictKind::FabricationDetected,
        "a missing captured output must Block (missing evidence = FAIL, R2)"
    );
    assert_eq!(verdict.reason, "missing_captured_output");

    if let SsbPayload::ResultManifestVerified(p) = only_row(&ssb) {
        assert_eq!(p.kind, ResultManifestVerifiedKind::FabricationDetected);
        assert_eq!(p.verdict, "missing_captured_output");
    } else {
        panic!("payload must be ResultManifestVerified");
    }
}

// ---------------------------------------------------------------------------
// (5) the SchemaRequired shape rule (FIN-RESMANIFEST-001) is seeded.
// ---------------------------------------------------------------------------

#[test]
fn part_a_schema_required_manifest_shape_is_seeded() {
    let conn = seeded_rules_db();
    let predicate_json: String = conn
        .query_row(
            "SELECT predicate FROM rules WHERE rule_id = 'FIN-RESMANIFEST-001'",
            [],
            |row| row.get(0),
        )
        .expect("FIN-RESMANIFEST-001 must be seeded");
    let predicate: Predicate =
        serde_json::from_str(&predicate_json).expect("predicate must deserialize");
    match predicate {
        Predicate::SchemaRequired {
            schema,
            output_type,
        } => {
            assert_eq!(output_type, "result_manifest");
            let required = schema
                .get("required")
                .and_then(|r| r.as_array())
                .expect("schema.required must be an array");
            let keys: Vec<String> = required
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
            // Every reported number must bind its provenance: a missing field => the
            // SchemaRequired predicate Blocks (the F-4 None-hash analog).
            for key in [
                "metric_name",
                "value",
                "captured_output_blake3",
                "parser_expr",
                "data_hash",
                "code_commit",
                "env_lock_hash",
                "run_id",
                "elai_seed",
                "n_trials",
            ] {
                assert!(
                    keys.iter().any(|k| k == key),
                    "result manifest must require '{key}'"
                );
            }
        }
        other => panic!("FIN-RESMANIFEST-001 must be a SchemaRequired, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (5b)/(5c) LOAD-BEARING: RUN the rule engine over the seeded FIN-RESMANIFEST-002
//     monkeypatch AST ban. These drive the DefaultRuleEngine (with the real
//     TreeSitterAstEvaluator) over real fixtures and assert the rule FIRES on an
//     in-scope result-emitting module that monkeypatches a result fn, and does NOT
//     fire on an out-of-scope benign module. The anti-wire-dark proof (R6): if the
//     scope is inert, (5b) goes red.
// ---------------------------------------------------------------------------

fn fin_resmanifest_002_engine_and_rule(
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
             FROM rules WHERE rule_id = 'FIN-RESMANIFEST-002'",
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
        .expect("FIN-RESMANIFEST-002 must be seeded");

    let engine = DefaultRuleEngine::new(
        Box::new(TreeSitterAstEvaluator::new()),
        Box::new(EmptyDkgQuerier),
    );
    let compiled = engine
        .compile_predicate(&predicate_json)
        .expect("FIN-RESMANIFEST-002 predicate must compile");
    let rule = Rule {
        rule_id: "FIN-RESMANIFEST-002".to_string(),
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
fn part_a_monkeypatch_ban_fires_on_in_scope_result_fn_rebind() {
    let (engine, rule, compiled) = fin_resmanifest_002_engine_and_rule();

    // Strategy code in a RESULT-EMITTING module that monkeypatches a result fn to
    // FAKE a number: the dangerous antipattern. The file is in scope (its path
    // contains a result token) AND the body matches the AST query (setattr of a
    // result-emitting fn name).
    let src = "def patch(obj):\n    setattr(obj, \"compute_sharpe\", lambda: 9.99)\n";
    let ctx = eval_context(src, "strategy/result_report.py");

    let verdicts = engine
        .evaluate(&[(rule, compiled)], &ctx)
        .expect("evaluate must not error");

    // R6 anti-wire-dark: the LOCKED ban MUST produce a Block here. If the scope is
    // inert (legacy starts_with semantics), this is Allow and the test fails.
    assert!(
        verdicts.iter().any(|v| v.result == VerdictResult::Block),
        "FIN-RESMANIFEST-002 MUST FIRE (Block) on an in-scope result-fn monkeypatch; \
         got {verdicts:?}. If this is Allow, the AST ban is runtime-inert (wire-dark)."
    );
}

#[test]
fn part_a_monkeypatch_ban_does_not_fire_on_out_of_scope_benign() {
    let (engine, rule, compiled) = fin_resmanifest_002_engine_and_rule();

    // A benign setattr in an OUT-OF-SCOPE config/DI module (legitimate dynamic
    // wiring), AND it does not target a result-emitting fn name. The file is out of
    // scope (no result token in its path), so the ban must NOT fire — no
    // false-block.
    let src = "def wire(obj):\n    setattr(obj, \"db_pool_size\", 8)\n";
    let ctx = eval_context(src, "config/wiring.py");

    let verdicts = engine
        .evaluate(&[(rule, compiled)], &ctx)
        .expect("evaluate must not error");

    assert!(
        verdicts.iter().all(|v| v.result == VerdictResult::Allow),
        "FIN-RESMANIFEST-002 must NOT fire on an out-of-scope benign config setattr; \
         got {verdicts:?}"
    );
}

// ---------------------------------------------------------------------------
// (6) SSB lockstep: SCHEMA_VERSION >= 230, action_type bijection landed
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 230,
        "SCHEMA_VERSION must be >= 230 after H-25; got {SCHEMA_VERSION_PUBLIC}"
    );

    // ResultManifestVerified must round-trip through the action_type bijection.
    let at = ssb_action_type_from_str_public("result_manifest_verified")
        .expect("result_manifest_verified must be a known action_type");
    assert_eq!(at, SsbActionType::ResultManifestVerified);
    assert_eq!(
        SsbActionType::ResultManifestVerified.as_sql_str(),
        "result_manifest_verified"
    );
}
