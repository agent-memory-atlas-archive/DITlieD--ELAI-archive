// Phase A3 — fix-and-rerun outer loop contract tests.
//
// Exercises [`fix_and_rerun_until_green`] in `task_runner`. The function
// wraps a web baby step's validator outcome: on validator failure, it
// dispatches one fixer turn through a [`BabyStepExecutor`] (Arc<dyn ...>)
// and re-runs the validator. Caps at `max_iterations` (default 5). On cap,
// emits an escalation row via [`emit_autonomous_decision`] using the
// existing `AutonomyDecision::RequestHuman` variant.
//
// Hermeticity: we do NOT call `npm` / `run_web_build` directly here. The
// new function accepts a validator closure so the production wiring can
// supply `run_web_build` while this contract test injects a deterministic
// stub that reads a sentinel file on disk. The MockFixerExecutor mutates
// that sentinel file inside `execute()`; this proves the wiring:
//   validator-failure -> fixer dispatch -> re-validate -> green.
//
// Covers:
//   1. Heals deliberately-broken sentinel within <=5 cycles. Outcome is
//      `FixAndRerunOutcome::Healed { iterations <= 5 }`.
//   2. Cap escalation: a NeverFixingExecutor leaves the sentinel broken;
//      after 5 cycles, outcome is `FixAndRerunOutcome::EscalatedToHuman
//      { iterations: 5 }` AND the SSB has a `GateDecision` row with
//      decision="request_human".

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload};
use elai_orchestrator::task_runner::{
    fix_and_rerun_until_green, BabyStepExecutionCtx, BabyStepExecutionResult, BabyStepExecutor,
    FixAndRerunOutcome, FixerValidator, FixerValidatorReport,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// Sentinel filename used by both the validator stub and the fixer mocks
// so the tests share a single contract for "what does broken look like".
const SENTINEL_FILENAME: &str = "App.tsx";
const BROKEN_TOKEN: &str = "BROKEN_SYNTAX_HERE";

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn write_broken_scaffold(project_root: &std::path::Path) {
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    // Minimal package.json so the directory looks like a node project; the
    // contract test never invokes npm itself.
    std::fs::write(
        project_root.join("package.json"),
        r#"{"name":"a3-fixture","version":"0.0.0","private":true}"#,
    )
    .expect("write package.json");
    std::fs::write(
        src_dir.join(SENTINEL_FILENAME),
        format!(
            "export default function App() {{ return <div>{}</div>; }}",
            BROKEN_TOKEN
        ),
    )
    .expect("write broken App.tsx");
}

fn make_ssb(tag: &str) -> Arc<Ssb> {
    let tmp = TempDir::new().expect("ssb tempdir");
    // Leak the tempdir so the SSB file outlives this helper (closed on
    // process exit; deterministic test environment).
    let path = tmp.path().join("ssb.db");
    std::mem::forget(tmp);
    let cfg = SsbConfig {
        db_path: path,
        pool_size: Some(2),
        project_id: format!("a3-{tag}"),
        session_id: format!("a3-{tag}-session"),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

/// Validator stub: reads the sentinel file; returns Green iff the broken
/// token has been removed. Mirrors the production shape that wraps
/// `run_web_build`.
fn sentinel_validator(project_root: PathBuf) -> impl FixerValidator {
    move |dir: &std::path::Path| {
        let project_root = project_root.clone();
        let target_dir = dir.to_path_buf();
        async move {
            // The validator inspects the App.tsx file under project_root/src
            // regardless of the dir argument; both paths point at the same
            // tempdir in these tests but we keep the signature realistic.
            let _ = &target_dir;
            let path = project_root.join("src").join(SENTINEL_FILENAME);
            let contents = tokio::fs::read_to_string(&path).await.unwrap_or_default();
            if contents.contains(BROKEN_TOKEN) {
                FixerValidatorReport::Failed {
                    summary: format!(
                        "syntax error: {} still present in {}",
                        BROKEN_TOKEN,
                        path.display()
                    ),
                }
            } else {
                FixerValidatorReport::Green
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MockFixerExecutor — rewrites App.tsx to remove BROKEN_TOKEN on first call.
// Subsequent calls are no-ops; the validator then reports Green.
// ---------------------------------------------------------------------------

struct MockFixerExecutor {
    project_root: PathBuf,
    calls: Arc<AtomicU32>,
}

impl MockFixerExecutor {
    fn new(project_root: PathBuf) -> (Arc<Self>, Arc<AtomicU32>) {
        let calls = Arc::new(AtomicU32::new(0));
        let me = Arc::new(Self {
            project_root,
            calls: calls.clone(),
        });
        (me, calls)
    }
}

impl BabyStepExecutor for MockFixerExecutor {
    fn execute<'a>(
        &'a self,
        _ctx: BabyStepExecutionCtx,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = BabyStepExecutionResult> + Send + 'a>,
    > {
        let project_root = self.project_root.clone();
        let calls = self.calls.clone();
        Box::pin(async move {
            calls.fetch_add(1, Ordering::SeqCst);
            let path = project_root.join("src").join(SENTINEL_FILENAME);
            let fixed = "export default function App() { return <div>fixed</div>; }";
            tokio::fs::write(&path, fixed)
                .await
                .expect("mock fixer write");
            BabyStepExecutionResult {
                outcome: BabyStepOutcome::Passed,
                duration_ms: 1,
                exit_code: Some(0),
                stderr_tail: String::new(),
                changed_files: vec![path],
            }
        })
    }
}

// ---------------------------------------------------------------------------
// NeverFixingExecutor — increments call count but leaves the file untouched.
// Drives the cap-escalation path.
// ---------------------------------------------------------------------------

struct NeverFixingExecutor {
    calls: Arc<AtomicU32>,
}

impl NeverFixingExecutor {
    fn new() -> (Arc<Self>, Arc<AtomicU32>) {
        let calls = Arc::new(AtomicU32::new(0));
        (Arc::new(Self { calls: calls.clone() }), calls)
    }
}

impl BabyStepExecutor for NeverFixingExecutor {
    fn execute<'a>(
        &'a self,
        _ctx: BabyStepExecutionCtx,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = BabyStepExecutionResult> + Send + 'a>,
    > {
        let calls = self.calls.clone();
        Box::pin(async move {
            calls.fetch_add(1, Ordering::SeqCst);
            BabyStepExecutionResult {
                outcome: BabyStepOutcome::Failed {
                    reason: "noop fixer".into(),
                },
                duration_ms: 1,
                exit_code: Some(1),
                stderr_tail: "noop fixer made no changes".into(),
                changed_files: Vec::new(),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Contract 1 — Healed path: deliberately-broken scaffold heals within <=5.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_fix_and_rerun_heals_broken_scaffold_within_cap() {
    let tmp = TempDir::new().expect("scaffold tempdir");
    let project_root = tmp.path().to_path_buf();
    write_broken_scaffold(&project_root);

    let (fixer, calls): (Arc<dyn BabyStepExecutor>, Arc<AtomicU32>) = {
        let (m, c) = MockFixerExecutor::new(project_root.clone());
        (m as Arc<dyn BabyStepExecutor>, c)
    };
    let ssb = make_ssb("heal");
    let validator = sentinel_validator(project_root.clone());

    let outcome = fix_and_rerun_until_green(
        project_root.clone(),
        fixer,
        None,
        ssb.clone(),
        5,
        validator,
        "a3-heal-step",
    )
    .await
    .expect("fix_and_rerun_until_green succeeded");

    match outcome {
        FixAndRerunOutcome::Healed { iterations } => {
            assert!(
                iterations >= 1 && iterations <= 5,
                "heal must complete within 5 iterations, got {iterations}"
            );
        }
        other => panic!("expected Healed, got {other:?}"),
    }
    // At least one fixer turn dispatched.
    assert!(
        calls.load(Ordering::SeqCst) >= 1,
        "mock fixer must have been dispatched at least once"
    );
}

// ---------------------------------------------------------------------------
// Contract 2 — Cap escalation path: NeverFixingExecutor never patches the
// file; after 5 iterations the function returns EscalatedToHuman and a
// GateDecision row appears in the SSB.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_fix_and_rerun_escalates_after_cap() {
    let tmp = TempDir::new().expect("scaffold tempdir");
    let project_root = tmp.path().to_path_buf();
    write_broken_scaffold(&project_root);

    let (fixer, calls): (Arc<dyn BabyStepExecutor>, Arc<AtomicU32>) = {
        let (n, c) = NeverFixingExecutor::new();
        (n as Arc<dyn BabyStepExecutor>, c)
    };
    let ssb = make_ssb("cap");
    let validator = sentinel_validator(project_root.clone());

    let outcome = fix_and_rerun_until_green(
        project_root.clone(),
        fixer,
        None,
        ssb.clone(),
        5,
        validator,
        "a3-cap-step",
    )
    .await
    .expect("fix_and_rerun_until_green returned");

    match outcome {
        FixAndRerunOutcome::EscalatedToHuman { iterations } => {
            assert_eq!(iterations, 5, "cap iterations must be exactly max=5");
        }
        other => panic!("expected EscalatedToHuman, got {other:?}"),
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        5,
        "fixer must have been dispatched exactly 5 times on cap"
    );

    // SSB observable effect: a GateDecision row was written with the
    // request_human label. This proves emit_autonomous_decision was invoked.
    let rows = ssb
        .query_by_type(SsbActionType::GateDecision)
        .expect("query GateDecision rows");
    assert!(
        !rows.is_empty(),
        "cap escalation must emit at least one GateDecision SSB row"
    );
    let mut saw_request_human = false;
    for entry in rows.iter() {
        let Ok(payload) = rmp_serde::from_slice::<SsbPayload>(entry.payload.as_ref()) else {
            continue;
        };
        if let SsbPayload::GateDecision(p) = payload {
            if p.decision == "request_human" {
                saw_request_human = true;
                break;
            }
        }
    }
    assert!(
        saw_request_human,
        "GateDecision rows must include decision=request_human on cap"
    );
}
