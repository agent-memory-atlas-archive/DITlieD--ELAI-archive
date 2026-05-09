// Session patch-1.32 contract tests: SidecarProvider wiring surface on the
// dashboard path. Closes MVP audit Section 4 item #3.
//
// Covers (without a live sidecar):
//   1. `TaskRunnerConfig` exposes an `llm_provider: Option<Arc<dyn LlmProvider>>`
//      field and `backend_config: Option<BackendConfig>` field.
//   2. `TaskRunnerConfig::new_minimal` defaults both fields to None so pre-
//      patch-1.32 call sites keep working.
//   3. A dashboard-like construction (mock `LlmProvider` + `BackendConfig`)
//      populates the fields and `start_task_runner` accepts the config.
//   4. `build_dispatch_context` accepts the same `BackendConfig` and yields a
//      non-empty `DispatchContext` for FsmState::Planning with a seeded tool
//      list — this proves the per-dispatch pathway compiles and runs.
//
// Note: `research_decompose` itself is covered by
// `research_decompose_real_prompts_contract.rs`; this test focuses on the
// provider-plumbing surface, not on the decomposition call sites.

use std::collections::HashMap;
use elai_core::GitBranchPolicy;
use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::{
    build_dispatch_context, start_task_runner, BackendConfig, BackendMode, FsmState, Ssb, SsbConfig,
    TaskRunnerConfig,
};
use elai_providers::{ApiError, LlmFuture, LlmProvider, OllamaConfig};

// ---------------------------------------------------------------------------
// Mock provider: records completion calls. Never touches the network.
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct RecordingProvider {
    calls: std::sync::atomic::AtomicUsize,
}

impl LlmProvider for RecordingProvider {
    fn complete<'a>(
        &'a self,
        _prompt: &'a str,
        _config: &'a OllamaConfig,
    ) -> LlmFuture<'a, Vec<String>> {
        self.calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move { Ok(vec!["mock".to_string()]) })
    }

    fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async move { Err(ApiError::Auth("mock-provider: no embed".into())) })
    }

    fn token_count(&self, text: &str) -> Result<usize, ApiError> {
        Ok(text.len() / 4 + 1)
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / 4 + 1
    }
}

// ---------------------------------------------------------------------------
// Contract 1: TaskRunnerConfig::new_minimal leaves llm_provider = None and
// backend_config = None — prevents silent behaviour flips on pre-existing
// callers (e.g. CLI-only tests that use `new_minimal`).
// ---------------------------------------------------------------------------

#[test]
fn contract_new_minimal_leaves_llm_provider_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("cursor.json"));
    assert!(
        cfg.llm_provider.is_none(),
        "new_minimal must leave llm_provider=None; dashboard path is the only opt-in site",
    );
    assert!(
        cfg.backend_config.is_none(),
        "new_minimal must leave backend_config=None so dispatcher defaults apply",
    );
}

// ---------------------------------------------------------------------------
// Contract 2: TaskRunnerConfig accepts a populated llm_provider + backend_config
// via struct literal (the pattern the dashboard uses at lib.rs:~390).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_task_runner_accepts_llm_provider() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let clock: Arc<dyn elai_core::clock::ClockProvider> =
        Arc::new(elai_core::clock::OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: db_path.clone(),
                pool_size: Some(1),
                project_id: "contract-test".into(),
                session_id: "patch-1.32-wiring".into(),
            },
            clock,
        )
        .expect("open ssb"),
    );

    let provider: Arc<dyn LlmProvider> = Arc::new(RecordingProvider::default());
    let backend_config = Some(BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:9999".to_string(),
        ..BackendConfig::default()
    });

    let (_join, cancel) = start_task_runner(
        ssb.clone(),
        TaskRunnerConfig {
            cursor_path: tmp.path().join("cursor.json"),
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
            llm_provider: Some(provider),
            backend_config,
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
            task_splitter: None,
            pipeline_grc: None,
            model_capability_profile: None,
        },
    );

    // Immediately cancel: the assertion is that start_task_runner accepted
    // the config shape — no runtime behaviour is exercised here (that's the
    // job of research_decompose_real_prompts_contract + the dispatch-context
    // direct-call contract below).
    cancel.cancel();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

// ---------------------------------------------------------------------------
// Contract 3: build_dispatch_context accepts the dashboard's BackendConfig
// and produces a DispatchContext with a non-empty filtered_tool_names for
// an FSM state that allows reads. Proves patch-1.32's per-dispatch plumbing
// is reachable from the public API.
// ---------------------------------------------------------------------------

#[test]
fn contract_build_dispatch_context_with_dashboard_config() {
    let cfg = BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:8100".to_string(),
        ..BackendConfig::default()
    };
    let tools: Vec<String> = vec![
        "read_file".into(),
        "write_file".into(),
        "bash".into(),
        "web_search".into(),
    ];
    let priority: HashMap<String, u8> = HashMap::new();

    // Planning allows read_file + web_search; blocks write/bash. The exact
    // filter is orchestrator-internal — the contract only requires that
    // (a) the call compiles against the dashboard config, (b) returns Ok,
    // and (c) filters non-trivially (at least one tool allowed, at least
    // one blocked) so we know the FSM gate fired.
    let ctx = build_dispatch_context(FsmState::Planning, &cfg, &tools, &priority, None, None)
        .expect("build_dispatch_context accepts dashboard config");
    assert_eq!(ctx.backend_mode, BackendMode::Local);
    assert_eq!(ctx.sidecar_url, "http://localhost:8100");
    assert!(
        !ctx.filtered_tool_names.is_empty(),
        "Planning state must allow at least one tool (read_file)",
    );
    assert!(
        ctx.filtered_tool_names.len() < tools.len(),
        "Planning state must block at least one tool (write/bash), got {:?}",
        ctx.filtered_tool_names,
    );
    assert!(ctx.budget.is_some(), "budget must be populated");
}

// ---------------------------------------------------------------------------
// Contract 4: mock provider calls are observable — verifies the mock path
// we'd use in integration tests actually exercises `complete`. This is the
// hermetic analogue of "sidecar HTTP endpoint fires per dispatch".
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_mock_provider_records_completion_calls() {
    let provider = RecordingProvider::default();
    let config = OllamaConfig::default();
    let _ = provider.complete("test prompt", &config).await;
    let _ = provider.complete("second prompt", &config).await;
    assert_eq!(
        provider.calls.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "mock provider must observe complete() invocations",
    );
}
