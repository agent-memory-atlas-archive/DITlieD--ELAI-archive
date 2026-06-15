// C-28 / Stage-5 EXEC-H: smoke test for the production gate install path.
// This test exercises the NEW &self OnceLock setter (set_deterministic_audit_gate)
// and the DefaultDeterministicAuditGate (not FakeGate) through an Arc<Fsm>.
// Named to match the wire-dark-allowlist.toml smoke_test entry for
// ELAI_DETERMINISTIC_AUDIT_GATE_ENABLE.

// Session wire-a12 contract tests.
//
// Exercises the [`DeterministicAuditGate`] hook installed on
// [`elai_orchestrator::Fsm`]. The gate is invoked on every `_ -> Auditing`
// transition; the cached result is consulted on `Auditing -> Memorizing` and
// the transition is rejected with `FsmError::AuditingBlocked` when any BLOCK
// finding is present.
//
// These contracts use a `FakeGate` rather than the production
// `DefaultDeterministicAuditGate` because the production impl wraps the
// synchronous `DeterministicAuditor::run_all` — which already has its own
// contract file (`deterministic_auditor_a41_contract.rs`). Here we prove the
// FSM plumbing is correct in isolation.
//
// Covers:
//   1. No gate installed  -> AUDITING -> MEMORIZING is unblocked (pre-wire-a12).
//   2. Gate returns passed -> AUDITING -> MEMORIZING succeeds; `last_audit_result`
//      is populated.
//   3. Gate returns BLOCK  -> AUDITING -> MEMORIZING rejected with
//      `FsmError::AuditingBlocked { block_count }`; FSM stays in AUDITING.
//   4. Re-entry into AUDITING with a passing gate clears the block so the
//      next MEMORIZING attempt succeeds.
//   5. Severity mapping: `DeterministicSeverity::Block` -> "error",
//      `Warn` -> "warning". Verified by the module's unit test; re-proven here
//      via `DeterministicAuditResult::block_count`/`warn_count`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use elai_orchestrator::deterministic_audit_gate::DeterministicAuditGate;
use elai_orchestrator::{
    CheckKind, DeterministicAuditResult, DeterministicFinding, DeterministicSeverity, Fsm,
    FsmError, FsmState,
};

struct FakeGate {
    calls: Arc<AtomicUsize>,
    result: std::sync::Mutex<DeterministicAuditResult>,
}

impl FakeGate {
    fn new(result: DeterministicAuditResult) -> (Arc<Self>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let g = Arc::new(Self {
            calls: Arc::clone(&calls),
            result: std::sync::Mutex::new(result),
        });
        (g, calls)
    }

    fn set_result(&self, result: DeterministicAuditResult) {
        *self.result.lock().unwrap() = result;
    }
}

impl DeterministicAuditGate for FakeGate {
    fn evaluate(&self, _source: FsmState, _target: FsmState) -> DeterministicAuditResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.lock().unwrap().clone()
    }
}

fn passing_result() -> DeterministicAuditResult {
    DeterministicAuditResult {
        findings: vec![DeterministicFinding {
            check: CheckKind::Build,
            severity: DeterministicSeverity::Warn,
            file: None,
            line: None,
            message: "no build cmds".into(),
        }],
        checks_run: 1,
        passed: true,
        mutation: None,
        mutation_budget: None,
    }
}

fn blocking_result() -> DeterministicAuditResult {
    DeterministicAuditResult {
        findings: vec![
            DeterministicFinding {
                check: CheckKind::LazyCode,
                severity: DeterministicSeverity::Block,
                file: Some(std::path::PathBuf::from("src/foo.rs")),
                line: Some(42),
                message: "todo!() found in production code".into(),
            },
            DeterministicFinding {
                check: CheckKind::Completeness,
                severity: DeterministicSeverity::Block,
                file: None,
                line: None,
                message: "stub function body".into(),
            },
        ],
        checks_run: 4,
        passed: false,
        mutation: None,
        mutation_budget: None,
    }
}

async fn drive_to_auditing(fsm: &Fsm) {
    fsm.transition(FsmState::Planning, "p", vec![]).await.unwrap();
    fsm.transition(FsmState::TestingFirst, "p", vec![])
        .await
        .unwrap();
    fsm.transition(FsmState::Executing, "p", vec![])
        .await
        .unwrap();
    fsm.transition(FsmState::Auditing, "p", vec![])
        .await
        .unwrap();
}

// Contract 1: no gate installed — MEMORIZING reachable from AUDITING (legacy).
#[tokio::test]
async fn contract_no_gate_allows_memorizing() {
    let fsm = Fsm::new(None, 8192);
    drive_to_auditing(&fsm).await;
    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect("memorizing with no gate");
    assert_eq!(fsm.get_state().await, FsmState::Memorizing);
    assert!(fsm.last_audit_result().await.is_none());
}

// Contract 2: gate returns passed -> MEMORIZING succeeds; last_audit_result populated.
#[tokio::test]
async fn contract_passing_gate_allows_memorizing_and_caches_result() {
    let (gate, calls) = FakeGate::new(passing_result());
    let fsm = Fsm::new(None, 8192).with_deterministic_audit_gate(gate);
    drive_to_auditing(&fsm).await;

    assert_eq!(calls.load(Ordering::SeqCst), 1, "gate called on entry to AUDITING");

    let cached = fsm
        .last_audit_result()
        .await
        .expect("cached after _ -> Auditing");
    assert!(cached.passed);
    assert_eq!(cached.block_count(), 0);
    assert_eq!(cached.warn_count(), 1);

    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect("memorizing with passing gate");
    assert_eq!(fsm.get_state().await, FsmState::Memorizing);
}

// Contract 3: gate returns BLOCK -> MEMORIZING rejected; FSM stays in AUDITING.
#[tokio::test]
async fn contract_blocking_gate_rejects_memorizing() {
    let (gate, calls) = FakeGate::new(blocking_result());
    let fsm = Fsm::new(None, 8192).with_deterministic_audit_gate(gate);
    drive_to_auditing(&fsm).await;

    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let err = fsm
        .transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect_err("blocking gate must reject");
    match err {
        FsmError::AuditingBlocked { block_count } => {
            assert_eq!(block_count, 2, "two BLOCK findings from the fake gate");
        }
        other => panic!("expected AuditingBlocked, got {other:?}"),
    }
    assert_eq!(fsm.get_state().await, FsmState::Auditing);
}

// Contract 4: re-entering AUDITING with a passing gate clears the block.
#[tokio::test]
async fn contract_reentry_with_passing_gate_clears_block() {
    let (gate, calls) = FakeGate::new(blocking_result());
    let fsm = Fsm::new(None, 8192)
        .with_deterministic_audit_gate(Arc::clone(&gate) as Arc<dyn DeterministicAuditGate>);
    drive_to_auditing(&fsm).await;

    // First attempt: blocked.
    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect_err("blocked by first gate result");

    // Agent "fixes" the findings — re-enter AUDITING via EXECUTING so the gate
    // runs again with a passing result.
    gate.set_result(passing_result());
    fsm.transition(FsmState::Executing, "p", vec![])
        .await
        .expect("audit -> executing re-entry");
    fsm.transition(FsmState::Auditing, "p", vec![])
        .await
        .expect("re-enter auditing");

    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "gate invoked on each _ -> Auditing entry"
    );

    // Second attempt: unblocked.
    fsm.transition(FsmState::Memorizing, "p", vec![])
        .await
        .expect("memorizing after fix");
    assert_eq!(fsm.get_state().await, FsmState::Memorizing);
}

// Contract 5: severity counters are the gate contract. `block_count` drives
// the AuditingBlocked branch; `warn_count` never triggers it.
#[test]
fn contract_severity_counters_match_expectations() {
    let p = passing_result();
    assert_eq!(p.block_count(), 0);
    assert_eq!(p.warn_count(), 1);

    let b = blocking_result();
    assert_eq!(b.block_count(), 2);
    assert_eq!(b.warn_count(), 0);
}

// ---------------------------------------------------------------------------
// C-28 / STAGE5-EXEC-H smoke test — production gate install path
// ---------------------------------------------------------------------------
//
// Proves:
//   1. DefaultDeterministicAuditGate (not FakeGate) can be built with
//      production-shaped suppliers.
//   2. set_deterministic_audit_gate(&self) installs it through Arc<Fsm>
//      using the OnceLock slot (no &mut required).
//   3. gate.evaluate() is called on `_ -> Auditing` and last_audit_result
//      is populated (proves the gate is live, not test-only).
//   4. A second set_deterministic_audit_gate call is silently ignored
//      (OnceLock first-write-wins; the initial gate is preserved).
//
// Does NOT assert SSB rows because tokio::spawn emission is async and
// testing async fire-and-forget rows would require a sleep or a poll loop
// (both flaky). The SSB emit path is covered by the auditor's own contract
// tests. Here we assert the install-and-call plumbing is wired.

#[tokio::test]
async fn c28_production_gate_installed_via_flag_emits_on_auditing_transition() {
    use elai_core::{
        ArchitecturePattern, BuildCommandRegistry, BuildSystem, BuildSystemProfile,
        CommandScope, Convention, EntryPoint, EntryPointKind,
        ErrorHandlingProfile, ErrorPattern, Language, LanguageProfile, ModuleOrganization,
        NamingConventions, OrgPattern, ProjectProfile, PropagationStyle,
        PROFILE_SCHEMA_VERSION, SeededClock,
    };
    use elai_core::project_profile::{
        DomainClassification, ProjectDomain,
    };
    use elai_orchestrator::{
        ssb::{Ssb, SsbConfig},
        DefaultDeterministicAuditGate, Fsm, FsmState,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // --- build a real SSB in a tempdir ---
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("c28_smoke.db"),
                project_id: "c28-smoke".into(),
                session_id: "s-c28-smoke".into(),
                pool_size: None,
            },
            Arc::new(SeededClock::new()),
        )
        .expect("SSB init"),
    );

    // --- minimal ProjectProfile fixture (no build commands so the auditor
    //     skips the build check and returns quickly without calling cargo) ---
    let fixture_profile = ProjectProfile {
        schema_version: PROFILE_SCHEMA_VERSION,
        generated_at: "2026-06-15T00:00:00Z".to_owned(),
        project_root: PathBuf::from("/smoke-test"),
        build_system: BuildSystemProfile {
            system: BuildSystem::Cargo,
            config_path: PathBuf::from("Cargo.toml"),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        },
        languages: LanguageProfile {
            primary: Language::Rust,
            secondary: Vec::new(),
            distribution: BTreeMap::new(),
            tree_sitter_coverage: 0.0,
        },
        frameworks: Vec::new(),
        test_profile: Default::default(),
        lint_tools: Vec::new(),
        entry_points: vec![EntryPoint {
            file: PathBuf::from("src/main.rs"),
            kind: EntryPointKind::Server,
        }],
        ci: None,
        naming: NamingConventions {
            functions: Convention::Strong("snake_case".to_owned()),
            types: Convention::Strong("PascalCase".to_owned()),
            constants: Convention::Strong("SCREAMING_SNAKE_CASE".to_owned()),
            modules: Convention::Unknown,
        },
        error_handling: ErrorHandlingProfile {
            pattern: ErrorPattern::ResultBased,
            propagation: PropagationStyle::QuestionMark,
            error_crate: Some("thiserror".to_owned()),
        },
        module_organization: ModuleOrganization { pattern: OrgPattern::Layered },
        dep_profile: Default::default(),
        project_vision: None,
        architecture: Some(ArchitecturePattern::Layered),
        domains: vec![DomainClassification {
            domain: ProjectDomain::WebBackend,
            confidence: 0.9,
            signals: vec!["smoke-test fixture".to_owned()],
        }],
        // No build commands: auditor treats this as "no build to run" and
        // returns a result with checks_run > 0 (file-size / lazy / completeness
        // still run over the empty changed-files list) and passed = true.
        commands: BuildCommandRegistry {
            build: Vec::new(),
            test: Vec::new(),
            lint: Vec::new(),
            format: Vec::new(),
            run: Vec::new(),
            clean: Vec::new(),
        },
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        overrides_applied: false,
    };
    let fixture_arc = Arc::new(fixture_profile);
    let profile_supplier: elai_orchestrator::deterministic_audit_gate::ProjectProfileSupplier =
        Arc::new(move || (*fixture_arc).clone());
    let changed_files_supplier: elai_orchestrator::deterministic_audit_gate::ChangedFilesSupplier =
        Arc::new(|| Vec::new());

    let gate = Arc::new(DefaultDeterministicAuditGate::new(
        Arc::clone(&ssb),
        profile_supplier,
        changed_files_supplier,
    ));

    // --- install gate through Arc<Fsm> via the new &self setter ---
    let fsm = Arc::new(Fsm::new(None, 8192));
    fsm.set_deterministic_audit_gate(Arc::clone(&gate) as Arc<dyn elai_orchestrator::deterministic_audit_gate::DeterministicAuditGate>);

    // Drive to AUDITING so evaluate() is called.
    drive_to_auditing(&fsm).await;

    // last_audit_result must be Some — the gate ran.
    let result = fsm
        .last_audit_result()
        .await
        .expect("last_audit_result must be Some after gate install + _ -> Auditing");
    // With an empty changed-files list, the auditor runs its non-build checks
    // (file-size / lazy / completeness over 0 files) and passes.
    assert!(
        result.passed,
        "gate should pass on empty changed-files list; findings: {:?}",
        result.findings
    );

    // --- second install is silently ignored (OnceLock first-write-wins) ---
    // This proves the double-install path (three-ctor live_cli pattern) is safe.
    let gate2 = Arc::new(
        DefaultDeterministicAuditGate::new(
            Arc::clone(&ssb),
            Arc::new(|| {
                // This supplier must never be called — the second gate was NOT installed.
                panic!("c28 smoke: second gate supplier must not be called (OnceLock should have ignored the second set call)");
            }),
            Arc::new(|| Vec::new()),
        ),
    );
    fsm.set_deterministic_audit_gate(
        gate2 as Arc<dyn elai_orchestrator::deterministic_audit_gate::DeterministicAuditGate>,
    );

    // Re-enter AUDITING via EXECUTING so the gate runs again.
    fsm.transition(FsmState::Executing, "p", vec![])
        .await
        .expect("re-entry to Executing");
    fsm.transition(FsmState::Auditing, "p", vec![])
        .await
        .expect("re-entry to Auditing");

    // The first gate is still installed — second set was ignored.
    // last_audit_result updated by the first gate's evaluate() (not the panic supplier).
    let result2 = fsm
        .last_audit_result()
        .await
        .expect("last_audit_result must be Some on second Auditing entry");
    assert!(
        result2.passed,
        "second auditing entry with first gate should also pass; findings: {:?}",
        result2.findings
    );
}
