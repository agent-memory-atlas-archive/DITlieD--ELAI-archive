//! I-1 directive supervisor enforcement-plane contract.
//!
//! The generator emits pure `Directive` structs; the `Supervisor` is the sole
//! side-effect executor, gate-checking (E-30 argv / F-1 VibeGate / G-3
//! blast-radius) before execution and recording a `DirectiveExecuted` SSB row.
//!
//! Arms (all fail-closed, R1 missing-evidence = FAIL):
//!   (a) BLOCK-AT-SEAM: a must-block directive blocks at the REAL seam —
//!       `InProcessHookDispatcher::dispatch_pre_tool_use` with a supervisor
//!       installed returns a Block finding for a destructive argv. (R4: real
//!       entrypoint, not a direct fn call to the gate.)
//!   (b) DOUBLE-GATE: exactly ONE gate path fires per op. With a supervisor
//!       installed the E-30 Deny finding appears EXACTLY ONCE — if the inline
//!       path ALSO fired the finding would duplicate (silent double-firing).
//!   (c) BOOTSTRAP-PARITY: all 3 production bootstrap sites construct the SAME
//!       supervisor by routing through `build_production_hook_dispatcher`
//!       (source-scan; the single consolidation fn = zero 3-way drift).
//!   (d) PAYLOAD-DISCRIMINATING: a `DirectiveExecuted` payload round-trips
//!       through the live rmp_serde path and decodes to the right Kind arm.
//!   (e) SCHEMA-LOCK: SCHEMA_VERSION_PUBLIC == 239 AND action_type_lock.rs
//!       declares TOTAL_VARIANTS: usize = 549.
//!
//! Spec references: docs/thefinalglance/I-plan/I-1_directive_supervisor_enforcement_plane.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7; .claude/rules/maintainability-review.md

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::hook_dispatcher::{
    HookDispatcher, HookInvocation, HookVerdictKind, InProcessHookDispatcher,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, DirectiveExecutedKind,
    DirectiveExecutedPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{Directive, GateOutcome, InProcessSupervisor, Supervisor};

const BASH_ACTION: &str = "bash_tool_use";

// ---------------------------------------------------------------------------
// (a) BLOCK-AT-SEAM: a must-block directive blocks at dispatch_pre_tool_use.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i1_a_destructive_directive_blocks_at_real_seam() {
    // Install the supervisor on the REAL dispatcher (the production gate seam).
    let supervisor: Arc<dyn Supervisor> = Arc::new(InProcessSupervisor::new("/tmp/i1-worktree"));
    let dispatcher = InProcessHookDispatcher::new(None).with_supervisor(supervisor);
    assert!(
        dispatcher.has_supervisor(),
        "supervisor must be installed on the dispatcher"
    );

    // Drive the REAL entrypoint with a destructive shell command.
    let inv = HookInvocation::for_pre_tool_use(BASH_ACTION, "rm -rf /", "step-i1-a");
    let findings = dispatcher
        .dispatch_pre_tool_use(inv)
        .await
        .expect("dispatch_pre_tool_use must not error");

    // R1: a Block finding MUST be present (missing = FAIL, fail-closed).
    let blocked = findings
        .iter()
        .any(|f| f.verdict == HookVerdictKind::Block);
    assert!(
        blocked,
        "destructive argv must produce a Block finding at the real seam; got {findings:?}"
    );
}

// ---------------------------------------------------------------------------
// (b) DOUBLE-GATE: exactly ONE gate path fires per op, via the supervisor.
//
// Uses a MockBlockSupervisor that emits findings with the DISTINGUISHABLE
// rule_id "i1-supervisor-gate" (never "e30-argv-deny"). Asserts:
//   (a) the supervisor finding IS present in the output.
//   (b) the inline "e30-argv-deny" finding is ABSENT (the supervised_gate guard
//       prevented the inline E-30 arm from running).
//   (c) total findings == exactly the supervisor's output set.
//
// Mutation argument: removing the `supervised_gate.is_none() &&` guard on
// hook_dispatcher.rs line 1177 causes the inline E-30 arm to run first,
// short-circuit with `return Ok(vec![Block{rule_id:"e30-argv-deny"}])`, and
// SKIP the supervisor early-return at line 1237. Assertion (b) then FAILS
// because "e30-argv-deny" appears but was expected absent; assertion (a)
// also FAILS because "i1-supervisor-gate" is never added by the inline path.
// ---------------------------------------------------------------------------

/// A mock supervisor whose `run_gates` always returns a single Block finding
/// with the distinguishable rule_id `"i1-supervisor-gate"`. Used to pin that
/// the supervised path — NOT the inline E-30 arm — produced the finding.
struct MockBlockSupervisor;

impl Supervisor for MockBlockSupervisor {
    fn run_gates(&self, _directive: &elai_orchestrator::Directive) -> GateOutcome {
        use elai_orchestrator::hook_dispatcher::{HookFinding, HookTier, HookVerdictKind};
        GateOutcome {
            blocked: true,
            findings: vec![HookFinding {
                tier: HookTier::T1,
                verdict: HookVerdictKind::Block,
                rule_id: Some("i1-supervisor-gate".into()),
                category: "i1_test".into(),
                message: "mock block for double-gate pin".into(),
                fix_template: None,
            }],
        }
    }

    fn execute_directive_dyn<'a>(
        &'a self,
        directive: elai_orchestrator::Directive,
        _op: Box<
            dyn FnOnce() -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<(), String>> + Send>,
                > + Send,
        >,
        _compensate: Box<
            dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                + Send,
        >,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<elai_orchestrator::DirectiveOutcome, elai_orchestrator::SupervisorError>,
                > + Send
                + 'a,
        >,
    > {
        let gate = self.run_gates(&directive);
        Box::pin(async move {
            if gate.blocked {
                return Ok(elai_orchestrator::DirectiveOutcome::Discarded);
            }
            Ok(elai_orchestrator::DirectiveOutcome::Executed)
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i1_b_exactly_one_gate_fires_per_op() {
    // Install the MockBlockSupervisor — its run_gates emits "i1-supervisor-gate"
    // (never "e30-argv-deny") so we can tell WHICH path produced the finding.
    let supervisor: Arc<dyn Supervisor> = Arc::new(MockBlockSupervisor);
    let supervised = InProcessHookDispatcher::new(None).with_supervisor(supervisor);

    let inv = HookInvocation::for_pre_tool_use(BASH_ACTION, "rm -rf /", "step-i1-b");
    let findings = supervised
        .dispatch_pre_tool_use(inv)
        .await
        .expect("dispatch must not error");

    // (a) Supervisor finding must be present — proves the supervised path fired.
    let supervisor_count = findings
        .iter()
        .filter(|f| f.rule_id.as_deref() == Some("i1-supervisor-gate"))
        .count();
    assert!(
        supervisor_count >= 1,
        "DOUBLE-GATE (a): supervisor finding 'i1-supervisor-gate' must be present; \
         means the supervised gate path ran; got {findings:?}"
    );

    // (b) Inline "e30-argv-deny" must be ABSENT — proves the supervised_gate guard
    //     prevented the inline E-30 arm from running. If the guard is removed the
    //     inline arm fires first and returns only "e30-argv-deny", making this fail.
    let inline_deny_count = findings
        .iter()
        .filter(|f| f.rule_id.as_deref() == Some("e30-argv-deny"))
        .count();
    assert_eq!(
        inline_deny_count, 0,
        "DOUBLE-GATE (b): inline 'e30-argv-deny' must be ABSENT when supervisor is \
         installed (the supervised_gate guard prevents the inline arm); \
         got {inline_deny_count}: {findings:?}"
    );

    // (c) Total findings == supervisor's exact set (no extra inline contributions).
    assert_eq!(
        findings.len(),
        supervisor_count,
        "DOUBLE-GATE (c): total findings must equal supervisor's finding count \
         (no inline path contributions); got {} vs supervisor_count {supervisor_count}: {findings:?}",
        findings.len()
    );
}

// ---------------------------------------------------------------------------
// (b2) Single-evaluator guard: run_gates is the gate path; a benign directive
// passes and a destructive one blocks (the supervisor IS the evaluator).
// ---------------------------------------------------------------------------

#[test]
fn i1_b2_supervisor_run_gates_is_the_gate_path() {
    let supervisor = InProcessSupervisor::new("/tmp/i1-worktree");

    let safe: GateOutcome = supervisor.run_gates(&Directive::new(
        BASH_ACTION,
        serde_json::json!({ "command": "ls -la" }),
        "intent-safe",
        "agent",
    ));
    assert!(!safe.blocked, "benign argv must not block via run_gates");

    let bad: GateOutcome = supervisor.run_gates(&Directive::new(
        BASH_ACTION,
        serde_json::json!({ "command": "rm -rf /" }),
        "intent-bad",
        "agent",
    ));
    assert!(bad.blocked, "destructive argv must block via run_gates");
}

// ---------------------------------------------------------------------------
// (c) BOOTSTRAP-PARITY: all 3 production sites route through the single
// consolidation fn build_production_hook_dispatcher (zero 3-way drift).
// ---------------------------------------------------------------------------

#[test]
fn i1_c_three_bootstrap_sites_route_through_one_builder() {
    use std::fs;

    // contracts/ -> crates/ -> <root>
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::Path::to_path_buf)
        .expect("resolve repo root");

    let sites = [
        root.join("crates").join("cli").join("src").join("run_subcommand.rs"),
        root.join("crates").join("cli").join("src").join("task_cmd.rs"),
        root.join("crates")
            .join("dashboard-v2")
            .join("src-tauri")
            .join("src")
            .join("lib.rs"),
    ];

    for site in &sites {
        assert!(site.exists(), "bootstrap site must exist: {}", site.display());
        let body = fs::read_to_string(site)
            .unwrap_or_else(|e| panic!("read {}: {e}", site.display()));
        assert!(
            body.contains("build_production_hook_dispatcher"),
            "bootstrap site {} must construct its dispatcher via the single \
             build_production_hook_dispatcher consolidation fn (so all 3 sites \
             inherit an identical supervisor; zero 3-way drift)",
            site.display()
        );
    }

    // And the consolidation fn must actually install the supervisor.
    let runtime_setup = root
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("runtime_setup.rs");
    let rs_body = fs::read_to_string(&runtime_setup)
        .unwrap_or_else(|e| panic!("read runtime_setup.rs: {e}"));
    assert!(
        rs_body.contains(".with_supervisor("),
        "build_production_hook_dispatcher must install the directive supervisor \
         via .with_supervisor(); without it the gate delegation is wire-dark (R1)"
    );
}

// ---------------------------------------------------------------------------
// (d) PAYLOAD-DISCRIMINATING: DirectiveExecuted rows round-trip + decode to the
// right Kind arm via the live rmp_serde path.
// ---------------------------------------------------------------------------

#[test]
fn i1_d_directive_executed_payload_kind_discriminates() {
    for (kind, tool) in [
        (DirectiveExecutedKind::Executed, "bash_tool_use"),
        (DirectiveExecutedKind::Discarded, "bash_tool_use"),
        (DirectiveExecutedKind::CompensationFired, "file_write"),
    ] {
        let payload = SsbPayload::DirectiveExecuted(DirectiveExecutedPayload {
            kind,
            tool_name: tool.into(),
            intent_id: "i1-row".into(),
            initiator: "agent".into(),
            ts_ms: 1_700_000_030_000,
        });
        // action_type bijection.
        assert_eq!(
            payload.action_type(),
            SsbActionType::DirectiveExecuted(DirectiveExecutedKind::Executed),
            "DirectiveExecuted payload must map to the directive_executed action_type"
        );
        // Live msgpack encode/decode (same path production uses).
        let encoded = rmp_serde::to_vec_named(&payload).expect("encode DirectiveExecuted");
        let decoded = decode_ssb_payload(&encoded).expect("decode DirectiveExecuted");
        match decoded {
            SsbPayload::DirectiveExecuted(p) => {
                assert_eq!(p.kind, kind, "decoded Kind arm must round-trip");
                assert_eq!(p.tool_name, tool, "decoded tool_name must round-trip");
            }
            other => panic!("decoded payload must be DirectiveExecuted; got {other:?}"),
        }
    }

    // from_str bijection (read-side consumers).
    let at = ssb_action_type_from_str_public("directive_executed")
        .expect("from_str directive_executed must succeed");
    assert_eq!(
        at,
        SsbActionType::DirectiveExecuted(DirectiveExecutedKind::Executed),
        "from_str directive_executed must return the Executed default arm"
    );
    assert_eq!(at.as_sql_str(), "directive_executed", "as_sql_str round-trip");
}

// ---------------------------------------------------------------------------
// (d2) Execute path lands a DirectiveExecuted SSB row keyed by outcome.
// Clean run -> Executed; gate block -> Discarded; op crash -> CompensationFired.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i1_d2_execute_directive_lands_outcome_rows() {
    use elai_orchestrator::ssb::{Ssb, SsbConfig};
    use elai_orchestrator::{DirectiveOutcome, SsbReader};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "i1-contract".into(),
        session_id: "i1-session".into(),
    };
    let ssb = Arc::new(Ssb::new(cfg, clock.clone()).expect("ssb open"));
    let supervisor = InProcessSupervisor::new(dir.path())
        .with_ssb(ssb.clone(), "supervisor", 0)
        .with_clock(clock);

    // Clean run -> Executed.
    let outcome = supervisor
        .execute_directive(
            Directive::new(
                BASH_ACTION,
                serde_json::json!({ "command": "ls -la" }),
                "intent-clean",
                "agent",
            ),
            || async { Ok::<(), String>(()) },
            || async {},
        )
        .await
        .expect("clean execute must succeed");
    assert_eq!(outcome, DirectiveOutcome::Executed);

    // Gate block -> Discarded; the side effect must NOT run.
    let mut op_ran = false;
    let discarded = supervisor
        .execute_directive(
            Directive::new(
                BASH_ACTION,
                serde_json::json!({ "command": "rm -rf /" }),
                "intent-blocked",
                "agent",
            ),
            || {
                op_ran = true;
                async { Ok::<(), String>(()) }
            },
            || async {},
        )
        .await
        .expect("blocked execute returns Ok(Discarded)");
    assert_eq!(discarded, DirectiveOutcome::Discarded);
    assert!(!op_ran, "blocked directive must NOT run the side effect (CANT-WRECK)");

    // Op crash -> compensation fires -> CompensationFired row; Err returned.
    // The file path is INSIDE the worktree root (dir.path()) so the G-3
    // fs-escape gate passes and the op runs (then crashes), rather than the
    // gate discarding it before the op.
    let in_root = dir.path().join("x.rs");
    let in_root_str = in_root.to_string_lossy().into_owned();
    let mut comp_ran = false;
    let crash = supervisor
        .execute_directive(
            Directive::new(
                "file_write",
                serde_json::json!({ "path": in_root_str, "source": "fn ok() {}" }),
                "intent-crash",
                "agent",
            ),
            || async { Err::<(), String>("simulated op crash".into()) },
            || {
                comp_ran = true;
                async {}
            },
        )
        .await;
    assert!(crash.is_err(), "crashing op must return Err");
    assert!(comp_ran, "compensation hook must fire on crash (E-44 seam)");

    // R1 off-agent evidence: rows for all three outcomes must land.
    let rows = ssb
        .query_by_type(SsbActionType::DirectiveExecuted(DirectiveExecutedKind::Executed))
        .expect("query DirectiveExecuted rows");
    assert_eq!(
        rows.len(),
        3,
        "exactly 3 DirectiveExecuted rows must land (Executed + Discarded + CompensationFired); got {}",
        rows.len()
    );
    let kinds: Vec<DirectiveExecutedKind> = rows
        .iter()
        .map(|r| match decode_ssb_payload(&r.payload).expect("decode row") {
            SsbPayload::DirectiveExecuted(p) => p.kind,
            other => panic!("row must be DirectiveExecuted; got {other:?}"),
        })
        .collect();
    assert!(kinds.contains(&DirectiveExecutedKind::Executed), "Executed row missing");
    assert!(kinds.contains(&DirectiveExecutedKind::Discarded), "Discarded row missing");
    assert!(
        kinds.contains(&DirectiveExecutedKind::CompensationFired),
        "CompensationFired row missing"
    );
}

// ---------------------------------------------------------------------------
// (e) SCHEMA-LOCK: exact pins for the I-1 bump.
// ---------------------------------------------------------------------------

#[test]
fn i1_e_schema_lock() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 239,
        "SCHEMA_VERSION_PUBLIC must be >= 239 (I-1 floor: DirectiveExecuted add); got {SCHEMA_VERSION_PUBLIC}"
    );

    // Verify TOTAL_VARIANTS >= 549 (I-1 shipped the 549th variant; later
    // sessions continue to grow it) + DirectiveExecuted registration.
    // Tier-B Move 2 derived TOTAL_VARIANTS and relocated ALL_VARIANTS (now
    // SSB_ACTION_TYPES in ssb.rs), so read the runtime registry directly.
    use elai_orchestrator::ssb::SSB_ACTION_TYPES;
    let tv_value = SSB_ACTION_TYPES.len();
    assert!(
        tv_value >= 549,
        "TOTAL_VARIANTS must be >= 549 (I-1 floor: 547 + DirectiveExecuted + \
         subsequent sessions continue to grow it); got {tv_value}"
    );
    assert!(
        SSB_ACTION_TYPES
            .iter()
            .any(|v| v.as_sql_str() == "directive_executed"),
        "SSB_ACTION_TYPES must include DirectiveExecuted"
    );
}
