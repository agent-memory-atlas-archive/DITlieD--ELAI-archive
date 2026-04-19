// Session 4E.15 contract tests: ScaffoldApplier wiring into the
// dashboard-facing task runner.
//
// The task_runner's own tests cover detector heuristics + happy-path
// applier invocation; these contracts cover:
//   1. `is_web_scaffold_task` heuristic matches the documented triggers
//      without misclassifying dev-server or generic build tasks.
//   2. `detect_scaffold_template` picks the correct built-in template.
//   3. Scaffold task with no applier configured emits a Finding with
//      category = "web_scaffold_failed" — no PhaseComplete.
//   4. Scaffold task whose applier returns `Err` emits a Finding — the
//      runner stays alive to consume the next task.
//   5. Happy path: applier is invoked once per submitted task, template
//      name + target_dir + task_text all reach the applier verbatim,
//      PhaseComplete is emitted.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use elai_core::clock::OsClock;
use elai_core::GitBranchPolicy;
use elai_orchestrator::ssb::{
    Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader, TaskSubmittedPayload,
};
use elai_orchestrator::{
    detect_scaffold_template, is_web_scaffold_task, start_task_runner, ProjectDirResolver,
    ScaffoldApplier, TaskRunnerConfig, DEFAULT_SCAFFOLD_TEMPLATE, WEB_SCAFFOLD_FAILED_CATEGORY,
    WEB_SCAFFOLD_TEMPLATE_ASTRO, WEB_SCAFFOLD_TEMPLATE_REACT, WEB_SCAFFOLD_TEMPLATE_SVELTE,
    WEB_SCAFFOLD_TEMPLATE_VANILLA,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn wait_for_action(
    writer: &Arc<Ssb>,
    action: SsbActionType,
    min_count: usize,
    timeout: Duration,
) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        let entries = writer.query_by_type(action).unwrap_or_default();
        if entries.len() >= min_count {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

fn new_ssb(db_path: PathBuf) -> Arc<Ssb> {
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "dashboard".into(),
        session_id: "test-4e15".into(),
    };
    Arc::new(Ssb::new(cfg, clock).expect("ssb"))
}

struct FixedResolver {
    dir: PathBuf,
}

impl ProjectDirResolver for FixedResolver {
    fn resolve(&self, _project_id: &str) -> Option<PathBuf> {
        Some(self.dir.clone())
    }
}

/// Records each `apply` call so tests can assert template name + target
/// dir + task text round-trip through the runner.
struct RecordingScaffoldApplier {
    calls: Arc<Mutex<Vec<(String, PathBuf, String)>>>,
    result: Result<(), String>,
}

impl RecordingScaffoldApplier {
    fn ok(calls: Arc<Mutex<Vec<(String, PathBuf, String)>>>) -> Self {
        Self {
            calls,
            result: Ok(()),
        }
    }

    fn failing(
        calls: Arc<Mutex<Vec<(String, PathBuf, String)>>>,
        msg: impl Into<String>,
    ) -> Self {
        Self {
            calls,
            result: Err(msg.into()),
        }
    }
}

impl ScaffoldApplier for RecordingScaffoldApplier {
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        task_text: &str,
    ) -> Result<(), String> {
        self.calls
            .lock()
            .expect("calls mutex")
            .push((template_name.to_owned(), target_dir.to_path_buf(), task_text.to_owned()));
        self.result.clone()
    }
}

fn finding_with_category(writer: &Arc<Ssb>, category: &str) -> bool {
    let rows = writer
        .query_by_type(SsbActionType::Finding)
        .unwrap_or_default();
    for row in rows {
        if let Ok(SsbPayload::Finding(f)) =
            rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
        {
            if f.category == category {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Contract 1: detector heuristic
// ---------------------------------------------------------------------------

#[test]
fn is_web_scaffold_task_matches_documented_triggers() {
    assert!(is_web_scaffold_task("Make me a React app"));
    assert!(is_web_scaffold_task("make me an Svelte landing page"));
    assert!(is_web_scaffold_task("Create a Vanilla webapp"));
    assert!(is_web_scaffold_task("create an Astro frontend"));
    assert!(is_web_scaffold_task("build me a React SPA"));
    assert!(is_web_scaffold_task("bootstrap a Svelte website"));
    assert!(is_web_scaffold_task("new React landing page"));
    assert!(is_web_scaffold_task("  Make me a vanilla website  "));

    // NOT match — prefix present but no web keyword.
    assert!(!is_web_scaffold_task("make me a function"));
    assert!(!is_web_scaffold_task("create a rust crate"));
    assert!(!is_web_scaffold_task("build me a CLI tool"));

    // NOT match — keyword only, no scaffold prefix.
    assert!(!is_web_scaffold_task("research React apps"));
    assert!(!is_web_scaffold_task("refactor the frontend module"));

    // NOT match — "Scaffold a Vite SPA and serve it" is the existing
    // dev-server integration flow and MUST stay on the default pipeline.
    assert!(!is_web_scaffold_task(
        "Scaffold a Vite hello-world SPA and serve it"
    ));

    // Edge cases.
    assert!(!is_web_scaffold_task(""));
    assert!(!is_web_scaffold_task("   "));
}

// ---------------------------------------------------------------------------
// Contract 2: template picker
// ---------------------------------------------------------------------------

#[test]
fn detect_scaffold_template_picks_expected_template() {
    assert_eq!(
        detect_scaffold_template("Make me a React app"),
        WEB_SCAFFOLD_TEMPLATE_REACT
    );
    assert_eq!(
        detect_scaffold_template("Create a Svelte landing page"),
        WEB_SCAFFOLD_TEMPLATE_SVELTE
    );
    assert_eq!(
        detect_scaffold_template("bootstrap an Astro site"),
        WEB_SCAFFOLD_TEMPLATE_ASTRO
    );
    assert_eq!(
        detect_scaffold_template("Make me a vanilla webapp"),
        WEB_SCAFFOLD_TEMPLATE_VANILLA
    );
    // No framework keyword -> default template.
    assert_eq!(
        detect_scaffold_template("Make me a website"),
        DEFAULT_SCAFFOLD_TEMPLATE
    );
    // First-match-wins ordering: react precedes svelte in detection order.
    assert_eq!(
        detect_scaffold_template("A React app with Svelte sidebar"),
        WEB_SCAFFOLD_TEMPLATE_REACT
    );
}

// ---------------------------------------------------------------------------
// Contract 3: missing applier emits Finding, no PhaseComplete
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn web_scaffold_task_without_applier_emits_finding() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ssb = new_ssb(tmp.path().join("ssb.db"));
    let cursor_path = tmp.path().join("cursor.json");

    let (handle, cancel) = start_task_runner(
        ssb.clone(),
        TaskRunnerConfig {
            cursor_path,
            dev_server_launcher: None,
            project_dir_resolver: None,
            dev_server_registry: None,
            web_research_runner: None,
            scaffold_applier: None,
            rule_evaluator: None,
            interrogation_store: None,
            checkpoint_scratch_dir: None,
            baby_step_executor: None,
            anti_spiral_enabled: false,
            llm_provider: None,
            backend_config: None,
            indexing_service: None,
            hook_dispatcher: None,
            signal_reader: None,
            tool_dispatcher: None,
            git_branch_policy: GitBranchPolicy::default(),
            system_prober: None,
            feasibility_prober: None,
            autonomous_gate: None,
            layer1a_processor: None,
            dispatch_context_profile: None,
            reflection_client: None,
            irreversible_classifier: None,
            dkg_querier_swap: None,
        },
    );

    tokio::time::sleep(Duration::from_millis(100)).await;

    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "Make me a React app".into(),
        project_id: "scaffold-proj-1".into(),
        submitted_at_ms: 0,
    });
    ssb.write_payload("dashboard", 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted");

    let got = wait_for_action(
        &ssb,
        SsbActionType::Finding,
        1,
        Duration::from_secs(10),
    )
    .await;
    assert!(got, "expected Finding when scaffold_applier is missing");

    assert!(
        finding_with_category(&ssb, WEB_SCAFFOLD_FAILED_CATEGORY),
        "expected Finding with category={}",
        WEB_SCAFFOLD_FAILED_CATEGORY
    );

    // PhaseComplete must NOT be emitted — the task could not complete.
    let phase_rows = ssb
        .query_by_type(SsbActionType::PhaseComplete)
        .unwrap_or_default();
    assert!(
        phase_rows.is_empty(),
        "missing applier must not emit PhaseComplete"
    );

    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
}

// ---------------------------------------------------------------------------
// Contract 4: applier error surfaces as Finding, runner survives
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn web_scaffold_task_applier_error_emits_finding() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ssb = new_ssb(tmp.path().join("ssb.db"));
    let cursor_path = tmp.path().join("cursor.json");
    let project_dir = tmp.path().join("dest");

    let resolver: Arc<dyn ProjectDirResolver> = Arc::new(FixedResolver {
        dir: project_dir.clone(),
    });
    let calls = Arc::new(Mutex::new(Vec::<(String, PathBuf, String)>::new()));
    let applier: Arc<dyn ScaffoldApplier> = Arc::new(RecordingScaffoldApplier::failing(
        calls.clone(),
        "disk full",
    ));

    let (handle, cancel) = start_task_runner(
        ssb.clone(),
        TaskRunnerConfig {
            cursor_path,
            dev_server_launcher: None,
            project_dir_resolver: Some(resolver),
            dev_server_registry: None,
            web_research_runner: None,
            scaffold_applier: Some(applier),
            rule_evaluator: None,
            interrogation_store: None,
            checkpoint_scratch_dir: None,
            baby_step_executor: None,
            anti_spiral_enabled: false,
            llm_provider: None,
            backend_config: None,
            indexing_service: None,
            hook_dispatcher: None,
            signal_reader: None,
            tool_dispatcher: None,
            git_branch_policy: GitBranchPolicy::default(),
            system_prober: None,
            feasibility_prober: None,
            autonomous_gate: None,
            layer1a_processor: None,
            dispatch_context_profile: None,
            reflection_client: None,
            irreversible_classifier: None,
            dkg_querier_swap: None,
        },
    );

    tokio::time::sleep(Duration::from_millis(100)).await;

    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "Make me a React app".into(),
        project_id: "scaffold-proj-2".into(),
        submitted_at_ms: 0,
    });
    ssb.write_payload("dashboard", 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted");

    let got = wait_for_action(&ssb, SsbActionType::Finding, 1, Duration::from_secs(10)).await;
    assert!(got, "expected Finding when applier returns Err");

    assert!(
        finding_with_category(&ssb, WEB_SCAFFOLD_FAILED_CATEGORY),
        "expected Finding with category={}",
        WEB_SCAFFOLD_FAILED_CATEGORY
    );

    // Applier was still invoked exactly once.
    assert_eq!(calls.lock().expect("calls").len(), 1);

    // PhaseComplete must NOT be emitted on the error path.
    let phase_rows = ssb
        .query_by_type(SsbActionType::PhaseComplete)
        .unwrap_or_default();
    assert!(
        phase_rows.is_empty(),
        "applier error must not emit PhaseComplete"
    );

    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
}

// ---------------------------------------------------------------------------
// Contract 5: happy path — applier invoked once, PhaseComplete emitted,
// template + target_dir + task text round-trip verbatim.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn web_scaffold_task_happy_path_round_trips_through_applier() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ssb = new_ssb(tmp.path().join("ssb.db"));
    let cursor_path = tmp.path().join("cursor.json");
    let project_dir = tmp.path().join("react-proj");

    let resolver: Arc<dyn ProjectDirResolver> = Arc::new(FixedResolver {
        dir: project_dir.clone(),
    });
    let calls = Arc::new(Mutex::new(Vec::<(String, PathBuf, String)>::new()));
    let applier: Arc<dyn ScaffoldApplier> =
        Arc::new(RecordingScaffoldApplier::ok(calls.clone()));

    let (handle, cancel) = start_task_runner(
        ssb.clone(),
        TaskRunnerConfig {
            cursor_path,
            dev_server_launcher: None,
            project_dir_resolver: Some(resolver),
            dev_server_registry: None,
            web_research_runner: None,
            scaffold_applier: Some(applier),
            rule_evaluator: None,
            interrogation_store: None,
            checkpoint_scratch_dir: None,
            baby_step_executor: None,
            anti_spiral_enabled: false,
            llm_provider: None,
            backend_config: None,
            indexing_service: None,
            hook_dispatcher: None,
            signal_reader: None,
            tool_dispatcher: None,
            git_branch_policy: GitBranchPolicy::default(),
            system_prober: None,
            feasibility_prober: None,
            autonomous_gate: None,
            layer1a_processor: None,
            dispatch_context_profile: None,
            reflection_client: None,
            irreversible_classifier: None,
            dkg_querier_swap: None,
        },
    );

    tokio::time::sleep(Duration::from_millis(100)).await;

    let task_text = "Make me a React app";
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: task_text.into(),
        project_id: "scaffold-proj-3".into(),
        submitted_at_ms: 0,
    });
    ssb.write_payload("dashboard", 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted");

    let got = wait_for_action(
        &ssb,
        SsbActionType::PhaseComplete,
        1,
        Duration::from_secs(10),
    )
    .await;
    assert!(got, "expected PhaseComplete for happy-path scaffold");

    // Applier called exactly once, with the expected arguments.
    let recorded = calls.lock().expect("calls").clone();
    assert_eq!(recorded.len(), 1, "applier must be invoked exactly once");
    assert_eq!(recorded[0].0, WEB_SCAFFOLD_TEMPLATE_REACT);
    assert_eq!(recorded[0].1, project_dir);
    assert_eq!(recorded[0].2, task_text);

    // No scaffold-failed Finding on the happy path.
    assert!(
        !finding_with_category(&ssb, WEB_SCAFFOLD_FAILED_CATEGORY),
        "happy path must not emit scaffold-failed Finding"
    );

    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
}
