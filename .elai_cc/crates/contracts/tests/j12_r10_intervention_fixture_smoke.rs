//! J-12 R10 residual: SHADOW->ACTIVE intervention protocol_gate fixture smoke.
//!
//! Integration fixture at the dispatcher grain (R10 integration-seam gate):
//! drives the REAL `build_production_hook_dispatcher` entrypoint with a
//! TempDir project_root whose `.elai_cc/rules.db` carries an intervention
//! protocol_gate rule. Verifies the three observable contracts:
//!   1. SHADOW arm: `dispatch_pre_tool_use` produces NO `harness_intervention`
//!      Block finding (observe-only; `GateVerdict::Exec`).
//!   2. ACTIVE arm (SHADOW-promoted-to-ACTIVE via status UPDATE): same
//!      dispatch yields a Block finding with `category = harness_intervention`
//!      + the rule_id, AND a `HarnessInterventionEvent` SSB row
//!      (`ActionGateBlocked`) lands via `query_by_type`.
//!   3. Control: empty project_root (no rules.db) -> no harness_intervention
//!      Block.
//!
//! Falsification: delete/replace `.with_action_gate(Arc::new(load_action_gate(..)))`
//! with `ActionGate::empty()` in `build_production_hook_dispatcher` -> ACTIVE
//! arm goes RED (no Block, no SSB row).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::rules_db_path;
use elai_orchestrator::hook_dispatcher::{
    HookDispatcher, HookInvocation, InProcessHookDispatcher,
};
use elai_orchestrator::runtime_setup::build_production_hook_dispatcher;
use elai_orchestrator::ssb::{
    HarnessInterventionKind, Ssb, SsbActionType, SsbConfig, SsbReader,
};
use elai_rules::schema::open_rules_db;
use rusqlite::params;
use tempfile::TempDir;
use tracing_test::traced_test;

const RULE_ID: &str = "j12-r10-fixture";
const TOOL_NAME: &str = "Write";
const PREDICATE_JSON: &str = r#"{"type":"protocol_gate","tool_name":"Write","required_args":["path"]}"#;
const PREDICATE_NO_TOOL_JSON: &str = r#"{"type":"protocol_gate","required_args":["path"]}"#;

fn build_ssb(dir: &TempDir) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "j12-r10".into(),
        session_id: "j12-r10-smoke".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

fn insert_intervention_rule(project_root: &std::path::Path, status: &str) {
    let db_path = rules_db_path(project_root);
    std::fs::create_dir_all(db_path.parent().expect("rules.db parent exists")).expect("mkdir parent");
    let conn = open_rules_db(&db_path, &OsClock).expect("open rules.db");
    conn.execute(
        "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
         predicate, severity, mutable, created_at, source) \
         VALUES (?1, 'intervention', 1, ?2, 'J-12 R10 fixture', 'smoke', \
         ?3, 'WARN', 1, 1000, 'SEED')",
        params![RULE_ID, status, PREDICATE_JSON],
    )
    .expect("insert rule");
}

fn update_rule_status(project_root: &std::path::Path, status: &str) {
    let db_path = rules_db_path(project_root);
    let conn = open_rules_db(&db_path, &OsClock).expect("reopen rules.db");
    conn.execute(
        "UPDATE rules SET status = ?1 WHERE rule_id = ?2",
        params![status, RULE_ID],
    )
    .expect("update rule status");
}

fn assert_no_harness_intervention_block(findings: &[elai_orchestrator::hook_dispatcher::HookFinding]) {
    for f in findings {
        if f.category == "harness_intervention" && f.is_block() {
            panic!(
                "expected NO harness_intervention Block finding, got rule_id={:?} msg={}",
                f.rule_id, f.message
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn r10_shadow_rule_does_not_block() {
    let dir = TempDir::new().expect("tempdir");
    insert_intervention_rule(dir.path(), "SHADOW");
    let ssb = build_ssb(&dir);
    let dispatcher: Arc<InProcessHookDispatcher> =
        build_production_hook_dispatcher(ssb, true, dir.path(), None);

    let inv = HookInvocation::for_pre_tool_use(TOOL_NAME, "{}", "r10-shadow");
    let findings = dispatcher
        .dispatch_pre_tool_use(inv)
        .await
        .expect("dispatch_pre_tool_use shadow");
    assert_no_harness_intervention_block(&findings);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn r10_active_rule_blocks_and_emits_ssb() {
    let dir = TempDir::new().expect("tempdir");
    insert_intervention_rule(dir.path(), "SHADOW");
    update_rule_status(dir.path(), "ACTIVE");

    let ssb = build_ssb(&dir);
    let dispatcher: Arc<InProcessHookDispatcher> =
        build_production_hook_dispatcher(ssb.clone(), true, dir.path(), None);

    let inv = HookInvocation::for_pre_tool_use(TOOL_NAME, "{}", "r10-active");
    let findings = dispatcher
        .dispatch_pre_tool_use(inv)
        .await
        .expect("dispatch_pre_tool_use active");

    let block = findings
        .iter()
        .find(|f| f.category == "harness_intervention" && f.is_block())
        .expect("expected harness_intervention Block finding on ACTIVE rule");
    assert_eq!(
        block.rule_id.as_deref(),
        Some(RULE_ID),
        "block rule_id must match the fixture rule"
    );

    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    let rows = ssb
        .query_by_type(SsbActionType::HarnessInterventionEvent(
            HarnessInterventionKind::ActionGateBlocked,
        ))
        .expect("query_by_type HarnessInterventionEvent");
    assert!(
        !rows.is_empty(),
        "expected at least one HarnessInterventionEvent ActionGateBlocked SSB row"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn r10_blank_project_no_block() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let dispatcher: Arc<InProcessHookDispatcher> =
        build_production_hook_dispatcher(ssb, true, dir.path(), None);

    let inv = HookInvocation::for_pre_tool_use(TOOL_NAME, "{}", "r10-empty");
    let findings = dispatcher
        .dispatch_pre_tool_use(inv)
        .await
        .expect("dispatch_pre_tool_use empty");
    assert_no_harness_intervention_block(&findings);
}

// ---------------------------------------------------------------------------
// (F4) ACTIVE rule with predicate missing tool_name -> load_action_gate warns
//      "rules_load: rule {rule_id} missing tool_name" and skips the rule, so
//      the gate is empty and dispatch produces NO harness_intervention Block.
//      T-1/T-2: #[traced_test] + assert!(logs_contain(...)) mandatory.
// ---------------------------------------------------------------------------

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn r10_no_tool_name_warns() {
    let dir = TempDir::new().expect("tempdir");
    {
        let db_path = rules_db_path(dir.path());
        std::fs::create_dir_all(db_path.parent().expect("rules.db parent exists"))
            .expect("mkdir parent");
        let conn = open_rules_db(&db_path, &OsClock).expect("open rules.db");
        conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, mutable, created_at, source) \
             VALUES (?1, 'intervention', 1, 'ACTIVE', 'J-12 R10 no-tool', 'smoke', \
             ?2, 'WARN', 1, 1000, 'SEED')",
            params![RULE_ID, PREDICATE_NO_TOOL_JSON],
        )
        .expect("insert rule");
    }

    let ssb = build_ssb(&dir);
    let dispatcher: Arc<InProcessHookDispatcher> =
        build_production_hook_dispatcher(ssb, true, dir.path(), None);

    let inv = HookInvocation::for_pre_tool_use(TOOL_NAME, "{}", "r10-no-tool");
    let findings = dispatcher
        .dispatch_pre_tool_use(inv)
        .await
        .expect("dispatch_pre_tool_use no-tool");

    assert!(
        logs_contain("missing tool_name"),
        "tracing log must contain 'missing tool_name' (T-1/T-2: load_action_gate warn path)"
    );
    assert_no_harness_intervention_block(&findings);
}
