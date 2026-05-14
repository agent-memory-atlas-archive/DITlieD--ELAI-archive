// Cloud-Sonnet E2E pipeline validation test.
//
// Drives the production task_runner with a *real* Anthropic Claude Sonnet
// provider routed through the saved OAuth token at ~/.elai/credentials.json,
// submits a single small non-scaffold task, and polls the SSB for up to
// 25 minutes while logging every emitted row to stderr. Asserts the
// autonomous pipeline phases fired (TaskSubmitted -> Decomposition ->
// ReasoningTrace); does *not* assert task success (that's a quality bar,
// not a wiring bar).
//
// Gating:
//   ELAI_E2E_CLOUD_SONNET=1 cargo test -p elai-orchestrator \
//     --test e2e_cloud_sonnet_pipeline -- --ignored --nocapture
//
// HARDWARE-HEAVY caveats: this hits api.anthropic.com with real cloud
// spend. The companion `e2e_real_sidecar_full_build` test documents the
// sidecar variant; this test documents the cloud variant. Both share the
// same "drive the production wiring end-to-end" shape so a regression in
// one surfaces in the other.
//
// Adapter rationale:
//   `AnthropicClient` (crates/providers/src/providers/anthropic.rs) does
//   NOT implement the orchestrator's `LlmProvider` trait — that trait
//   targets the `/v1/completions` shape used by the sidecar and Ollama
//   paths, while `AnthropicClient::send_message` targets `/v1/messages`.
//   To plug the cloud client into `TaskRunnerConfig::llm_provider` we
//   ship a *test-only* adapter (`AnthropicLlmProviderAdapter`) inline
//   here that maps `complete(prompt, _)` to a single-turn
//   `MessageRequest` and extracts the first `Text` output block. The
//   adapter is a test fixture, not production code — production cloud
//   wiring goes through the dashboard's HTTP-streaming path
//   (commands.rs:start_chat_stream), not the LlmProvider trait. This
//   test exists to confirm the pipeline *phases* fire end-to-end with
//   a real provider attached; the integration shape mirrors what a
//   future production AnthropicLlmProviderAdapter would look like.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::ssb::{
    SsbActionType, SsbEntry, SsbPayload, SsbReader, TaskSubmittedPayload,
};
use elai_orchestrator::{
    apply_task_runner_wiring, build_task_runner_wiring, start_task_runner, ProjectDirResolver,
    RuntimeWiring,
};

use elai_providers::{
    AnthropicClient, ApiError, AuthSource, InputContentBlock, InputMessage, LlmFuture,
    LlmProvider, MessageRequest, OllamaConfig, OutputContentBlock,
};

// ---------------------------------------------------------------------------
// Tunables. Kept named so the assertion failure messages cite the budget the
// test was running on, not a magic number.
// ---------------------------------------------------------------------------

/// Outer wallclock budget for the full E2E test. 25 minutes per the task
/// brief — enough headroom for a real Sonnet round-trip plus orchestrator
/// pipeline phases on a cold tempdir.
const E2E_OUTER_TIMEOUT_SECS: u64 = 1500;

/// SSB poll cadence inside the wallclock budget. 1 s matches the wait-for-
/// any-action helpers used by the sibling sidecar smoke test.
const SSB_POLL_INTERVAL: Duration = Duration::from_millis(1000);

/// Truncation budget for the per-row payload preview emitted to stderr.
/// 80 chars keeps the per-row line readable when watching a live tail.
const PAYLOAD_PREVIEW_MAX_CHARS: usize = 80;

/// Shutdown timeout for the task_runner JoinHandle after cancel() flips
/// the CancelHandle. 15 s mirrors the autonomous-build test's grace
/// period (the runner shuts down in <50ms in practice; the margin
/// absorbs the worst-case in-flight cloud round-trip the cancel might
/// race with).
const SHUTDOWN_TIMEOUT_SECS: u64 = 15;

/// The exhaustive list of SsbActionType variants the polling loop walks.
/// Hardcoded because the enum has no `strum::IntoEnumIterator` derive at
/// this revision (ssb.rs:202 only derives Debug/Clone/Copy/PartialEq/Eq/
/// Hash/Serialize/Deserialize). New variants added upstream will simply
/// not be tallied here — they're additive, never load-bearing for the
/// assertion. Keep the list in sync with `SsbActionType::as_sql_str`'s
/// match in ssb.rs:549 if a regression test demands coverage of a
/// specific new variant.
const ALL_ACTION_TYPES: &[SsbActionType] = &[
    SsbActionType::FileWrite,
    SsbActionType::FileRead,
    SsbActionType::ToolResult,
    SsbActionType::TestOutcome,
    SsbActionType::Finding,
    SsbActionType::Claim,
    SsbActionType::Checkpoint,
    SsbActionType::ModelSwap,
    SsbActionType::PhaseTransition,
    SsbActionType::KnowledgeDegradation,
    SsbActionType::IsolationMetric,
    SsbActionType::PlanUpdate,
    SsbActionType::ReplanRequest,
    SsbActionType::StubReference,
    SsbActionType::ReasoningTrace,
    SsbActionType::PhaseComplete,
    SsbActionType::MemoryMonitoring,
    SsbActionType::WorkloadProfile,
    SsbActionType::DeathEvent,
    SsbActionType::RecoveryEvent,
    SsbActionType::SupervisionExhausted,
    SsbActionType::SpiralDetected,
    SsbActionType::ContextReduction,
    SsbActionType::BranchMerge,
    SsbActionType::MistakeEntry,
    SsbActionType::ResearchStep,
    SsbActionType::ResearchDecomposeLevel,
    SsbActionType::WebResearchStep,
    SsbActionType::PreviewAvailable,
    SsbActionType::InterventionRequest,
    SsbActionType::InterventionResponse,
    SsbActionType::TaskSubmitted,
    SsbActionType::HookEvent,
    SsbActionType::ProviderSignals,
    SsbActionType::SearchTriggerFired,
    SsbActionType::VramTierDetected,
    SsbActionType::IndexingProgress,
    SsbActionType::DkgReady,
    SsbActionType::TieredAnalysis,
    SsbActionType::ShortCoderCompression,
    SsbActionType::ToolDispatch,
    SsbActionType::ValidationTier,
    SsbActionType::FailureClassified,
    SsbActionType::GitCommit,
    SsbActionType::DairaTrace,
    SsbActionType::SystemProbe,
    SsbActionType::FeasibilityReport,
    SsbActionType::NovelStepDetected,
    SsbActionType::StubRun,
    SsbActionType::GateDecision,
    SsbActionType::TaskSplit,
    SsbActionType::EscalationFired,
    SsbActionType::ModeChanged,
    SsbActionType::ProposalGenerated,
    SsbActionType::ProposalApplied,
    SsbActionType::ProposalRejected,
    SsbActionType::MsiUpdate,
    SsbActionType::ConfigArchived,
    SsbActionType::ProfileSelected,
    SsbActionType::MicroReflection,
    SsbActionType::TelescopeCollapse,
    SsbActionType::ConfigWarning,
    SsbActionType::ConstraintsInjected,
    SsbActionType::InterventionResolved,
    SsbActionType::DkgQuerierSwapped,
    SsbActionType::SteeringComputed,
    SsbActionType::SteeringComputeFailed,
    SsbActionType::McpServerConnected,
    SsbActionType::McpToolCalled,
    SsbActionType::McpServerDisconnected,
    SsbActionType::ChatMessage,
    SsbActionType::ChatResponse,
    SsbActionType::ExecutionModeChanged,
    SsbActionType::ContextBudgetComputed,
    SsbActionType::SsbFilteredByOverlap,
    SsbActionType::HookLatencyViolation,
    SsbActionType::ResearchPathChosen,
    SsbActionType::GroundingCacheEvent,
    SsbActionType::CircuitBreakerStateChange,
    SsbActionType::BubbleUpTyped,
    SsbActionType::IntegrationCheckpointSuccess,
    SsbActionType::DagRepairRound,
    SsbActionType::ConceptDecompositionNode,
    SsbActionType::EvidenceGateEvaluated,
    SsbActionType::SupervisionEvent,
    SsbActionType::BspSuperstep,
    SsbActionType::AgentLifecycle,
    SsbActionType::AntiSpiralHit,
    SsbActionType::CloudSpend,
    SsbActionType::EscalateToCloud,
    SsbActionType::DataTableGrew,
    SsbActionType::ConvergenceStagnant,
    SsbActionType::ConfigOscillation,
    SsbActionType::RegressionDetected,
    SsbActionType::EvolutionWithheld,
    SsbActionType::ReflectTriaged,
    SsbActionType::ReflectCandidate,
    SsbActionType::ReflectAuditVerdict,
    SsbActionType::ReflectRefined,
    SsbActionType::MediaValidationStarted,
    SsbActionType::RenderComplete,
    SsbActionType::VisualFeedbackIteration,
    SsbActionType::MediaHumanReviewRequired,
    SsbActionType::GenerationModeSelected,
    SsbActionType::StreamingEarlyTerminated,
    SsbActionType::PipelinePreAssembled,
    SsbActionType::DependencyResolved,
    SsbActionType::MicroStepCreated,
];

// ---------------------------------------------------------------------------
// Test-only LlmProvider adapter wrapping a real AnthropicClient.
//
// Lives inline in the test file (not the providers crate) because (a) it
// is a fixture, not a production component, and (b) shipping it from
// `providers` would require deciding the prod cloud routing path, which
// is the dashboard's HTTP-streaming surface today — not the LlmProvider
// trait. The adapter purposefully matches the contract the orchestrator
// already exercises against SidecarProvider / OllamaProvider:
//   - `complete(prompt, _config)` -> single completion string
//   - `embed` -> NotImplemented (cloud embedding routing is out of scope)
//   - `token_count` / `token_count_estimate` -> 4-chars-per-token heuristic
//     (matches SidecarProvider's heuristic at sidecar_client.rs:525-531)
//   - `model_name` -> the cloud model id the adapter is pinned to
// ---------------------------------------------------------------------------

struct AnthropicLlmProviderAdapter {
    client: AnthropicClient,
    model: String,
    /// Max output tokens per Sonnet round-trip. Kept small (1024) because
    /// this test is a wiring smoke check — large completions inflate the
    /// per-test cloud spend without adding signal.
    max_tokens: u32,
}

impl AnthropicLlmProviderAdapter {
    fn new(client: AnthropicClient, model: impl Into<String>, max_tokens: u32) -> Self {
        Self {
            client,
            model: model.into(),
            max_tokens,
        }
    }

    fn extract_text(response_content: &[OutputContentBlock]) -> Option<String> {
        response_content.iter().find_map(|block| match block {
            OutputContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        })
    }
}

impl LlmProvider for AnthropicLlmProviderAdapter {
    fn complete<'a>(
        &'a self,
        prompt: &'a str,
        _config: &'a OllamaConfig,
    ) -> LlmFuture<'a, Vec<String>> {
        Box::pin(async move {
            let request = MessageRequest {
                model: self.model.clone(),
                max_tokens: self.max_tokens,
                messages: vec![InputMessage {
                    role: "user".to_string(),
                    content: vec![InputContentBlock::Text {
                        text: prompt.to_string(),
                        cache_control: None,
                    }],
                }],
                system: None,
                tools: None,
                tool_choice: None,
                stream: false,
                seed: None,
            };
            let response = self.client.send_message(&request).await?;
            match Self::extract_text(&response.content) {
                Some(text) => Ok(vec![text]),
                // No Text block in the response. Treat as an empty
                // completion rather than an error — the orchestrator's
                // default branch handles empty completions deterministically
                // (no panic), and an Err here would short-circuit before
                // the pipeline phases get a chance to fire.
                None => Ok(vec![]),
            }
        })
    }

    fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async move {
            Err(ApiError::Api {
                status: reqwest::StatusCode::NOT_IMPLEMENTED,
                error_type: Some("not_implemented".to_string()),
                message: Some(
                    "AnthropicLlmProviderAdapter (test fixture) does not implement embeddings"
                        .to_string(),
                ),
                body: String::new(),
                retryable: false,
            })
        })
    }

    fn token_count(&self, text: &str) -> Result<usize, ApiError> {
        // 4-chars-per-token heuristic, same shape SidecarProvider uses
        // (sidecar_client.rs:525-527). Cloud token counts come back on
        // the response.usage payload — this estimate is only used by
        // upstream budgeting code that calls `token_count` before the
        // round-trip fires.
        Ok(text.len() / 4 + 1)
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / 4 + 1
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// ---------------------------------------------------------------------------
// Test-only ProjectDirResolver. Same shape as the sibling sidecar test:
// every project_id resolves to the same on-disk directory inside the
// tempdir, so the orchestrator's filesystem-touching paths land somewhere
// deterministic and discardable. Mirrors e2e_autonomous_build.rs:214.
// ---------------------------------------------------------------------------

struct StaticTestResolver {
    dir: PathBuf,
}

impl ProjectDirResolver for StaticTestResolver {
    fn resolve(&self, _project_id: &str) -> Option<PathBuf> {
        Some(self.dir.clone())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// UTF-8-safe truncation. Slicing a `String` by byte index splits codepoints;
/// `chars().take(n)` keeps us boundary-clean for arbitrary cloud responses.
fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Decode an `SsbEntry`'s payload bytes into a one-line preview string.
/// Falls back to the action-type name + payload byte length when the
/// payload is not valid UTF-8 (binary payloads are legal in the SSB
/// schema; the polling loop must not panic on them).
fn payload_preview(entry: &SsbEntry) -> String {
    match std::str::from_utf8(&entry.payload) {
        Ok(utf8) => {
            let cleaned: String = utf8
                .chars()
                .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
                .collect();
            truncate_chars(&cleaned, PAYLOAD_PREVIEW_MAX_CHARS)
        }
        Err(_) => format!(
            "<binary {} bytes, action={}>",
            entry.payload.len(),
            entry.action_type.as_sql_str()
        ),
    }
}

/// Snapshot every action-type bucket. Returns a Vec ordered by the
/// declaration order in `ALL_ACTION_TYPES` so iteration is deterministic
/// without requiring `SsbActionType: Ord` (the enum derives Hash + Eq but
/// not Ord at this revision — ssb.rs:202). Empty buckets are elided to
/// keep the snapshot small.
fn snapshot_all(writer: &Arc<elai_orchestrator::Ssb>) -> Vec<(SsbActionType, Vec<SsbEntry>)> {
    let mut out: Vec<(SsbActionType, Vec<SsbEntry>)> = Vec::new();
    for action_type in ALL_ACTION_TYPES {
        match writer.query_by_type(*action_type) {
            Ok(entries) => {
                if !entries.is_empty() {
                    out.push((*action_type, entries));
                }
            }
            Err(e) => {
                eprintln!(
                    "[poll] query_by_type({:?}) failed: {e} -- treating as empty",
                    action_type
                );
            }
        }
    }
    out
}

/// Comparator key: `SsbActionType` derives `Hash + Eq` but not `Ord`, so we
/// can't drop it into a `BTreeMap`. Use `as_sql_str` as a stable string
/// key for tally output instead. Returns owned String so the caller doesn't
/// have to juggle 'static lifetimes across the borrow checker.
fn action_label(action: SsbActionType) -> String {
    action.as_sql_str().to_string()
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires ELAI_E2E_CLOUD_SONNET=1 + saved OAuth at ~/.elai/credentials.json + live api.anthropic.com access (real cloud spend)"]
async fn e2e_cloud_sonnet_pipeline() {
    // 1. Env gate. Defer all setup until we've confirmed the user actually
    //    wants to spend real cloud tokens.
    if std::env::var("ELAI_E2E_CLOUD_SONNET").ok().as_deref() != Some("1") {
        eprintln!(
            "e2e_cloud_sonnet_pipeline requires ELAI_E2E_CLOUD_SONNET=1 + saved OAuth at \
             ~/.elai/credentials.json. Skipping. \
             See workflow/remote-setup/REMOTE-SIDECAR.md for the canonical \
             sidecar variant; this test exercises the cloud variant."
        );
        return;
    }

    // 2. Process env knobs the production wiring reads. These are SAFE to
    //    set process-wide here because the test is `#[ignore]`d by default
    //    -- it only runs when the user explicitly opts in, and concurrent
    //    test execution is disabled by the `--ignored --test-threads=1`
    //    invocation typical for E2E tests.
    //
    //    SAFETY (Rust 2024 `set_var` unsafe gate): no other thread is
    //    spawned yet at this point in the test body, so the env write is
    //    not racing reads from anywhere else. The 5 keys below are all
    //    documented in the task brief.
    unsafe {
        std::env::set_var("ELAI_DEFAULT_EXECUTION_MODE", "CLOUD");
        std::env::set_var("ELAI_NO_SIDECAR", "1");
        std::env::set_var("ELAI_STUB_SEARCH", "1");
        std::env::set_var("ELAI_GATE_MODE", "autonomous");
        std::env::set_var("ELAI_GROUNDING_BACKEND", "cloud");
        std::env::set_var("ELAI_CLOUD_MODEL", "claude-sonnet-4-6");
        // Deliberately DO NOT touch ANTHROPIC_BASE_URL — let
        // read_base_url() default to https://api.anthropic.com so the
        // adapter hits the real cloud endpoint.
        // Deliberately DO NOT set ANTHROPIC_API_KEY — force the
        // AuthSource::from_env_or_saved() resolution to fall through to
        // the saved OAuth at ~/.elai/credentials.json.
    }

    // Wrap the rest of the test in the outer wallclock so a hung pipeline
    // doesn't burn CI all night.
    let outer = tokio::time::timeout(
        Duration::from_secs(E2E_OUTER_TIMEOUT_SECS + 60), // +60s grace for shutdown
        run_inner(),
    )
    .await;
    match outer {
        Ok(()) => {}
        Err(_) => panic!(
            "e2e_cloud_sonnet_pipeline exceeded {}s outer wallclock (incl. shutdown grace)",
            E2E_OUTER_TIMEOUT_SECS + 60
        ),
    }
}

async fn run_inner() {
    // 3. Fresh project_root + app subdir.
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let app_dir = project_root.join("app");
    std::fs::create_dir_all(&app_dir).expect("create app/ subdir under project_root");
    let session_id = "test-cloud-sonnet-e2e";

    // 4. Build the production wiring + apply it to a fresh
    //    TaskRunnerConfig. Same chokepoint helpers the dashboard +
    //    `elai run` use (build_task_runner_wiring -> apply_task_runner_wiring
    //    -- contracts/tests/task_runner_chokepoint_contract.rs).
    let wiring: RuntimeWiring = build_task_runner_wiring(&project_root, session_id)
        .await
        .expect("build_task_runner_wiring(project_root) succeeds");
    let cursor_path = project_root.join(".elai_cc").join("cursor.json");
    let mut cfg = apply_task_runner_wiring(cursor_path, &wiring);

    // 5. Some+Some chokepoint guard — mirrors the sibling tests. If these
    //    fire we have a regression in apply_task_runner_wiring that needs
    //    fixing before any cloud assertion can be trusted.
    assert!(
        cfg.feasibility_prober.is_some(),
        "apply_task_runner_wiring must populate feasibility_prober (Some+Some chokepoint)"
    );
    assert!(
        cfg.layer1a_processor.is_some(),
        "apply_task_runner_wiring must populate layer1a_processor (Some+Some chokepoint)"
    );

    // 6. Wire the AnthropicClient -> LlmProvider adapter. Auth resolution
    //    flow per anthropic.rs:524-555 (ANTHROPIC_API_KEY > ANTHROPIC_AUTH_TOKEN
    //    > saved OAuth at ~/.elai/credentials.json). We expect the saved-OAuth
    //    branch; the test will panic early with a descriptive message if no
    //    credential source is present.
    let auth = AuthSource::from_env_or_saved().expect(
        "AuthSource::from_env_or_saved() must resolve a credential; \
         expected saved OAuth at ~/.elai/credentials.json. Run \
         `elai login` (or set ANTHROPIC_API_KEY) before running this test.",
    );
    let base_url = elai_providers::read_base_url();
    eprintln!(
        "[setup] AnthropicClient base_url={} auth_kind={}",
        base_url,
        match &auth {
            AuthSource::ApiKey(_) => "api_key",
            AuthSource::BearerToken(_) => "bearer_token",
            AuthSource::ApiKeyAndBearer { .. } => "api_key+bearer",
            AuthSource::None => "none",
        }
    );
    let anthropic_client = AnthropicClient::from_auth(auth).with_base_url(base_url);
    let cloud_model = elai_providers::DEFAULT_CLOUD_MODEL_ID;
    let adapter: Arc<dyn LlmProvider> = Arc::new(AnthropicLlmProviderAdapter::new(
        anthropic_client,
        cloud_model,
        1024,
    ));
    cfg.llm_provider = Some(adapter);

    // 7. Pin every project_id to the app/ subdir. Without this, the
    //    production CwdProjectDirResolver returns the cargo test process
    //    CWD (the workspace root), which would leak orchestrator-driven
    //    writes into the repo.
    cfg.project_dir_resolver = Some(Arc::new(StaticTestResolver {
        dir: app_dir.clone(),
    }));
    // 8. Scaffold applier skipped intentionally — the test task is
    //    explicitly non-scaffold (no react/vite/webapp/astro/svelte
    //    keyword, no "make me a" prefix). The orchestrator's
    //    `is_web_scaffold_task` classifier (task_runner.rs:~3482) routes
    //    it through the decomposition + baby-step branch instead.

    // 9. Spawn the runner against the same Arc<Ssb> we'll write to.
    let writer = wiring.ssb.clone();
    let (handle, cancel) = start_task_runner(writer.clone(), cfg);

    // Give the runner a moment to subscribe before the first write so we
    // don't race the broadcast subscription. The replay_backlog path on
    // the runner side would catch a missed row anyway, but the short
    // delay shaves variance off the first-row latency.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 10. Synthetic TaskSubmitted row. Plain non-scaffold, non-web-research
    //     task — should route through full decomposition -> baby-step
    //     pipeline.
    let project_id = elai_core::project_id_from_path(&project_root);
    let task_text = "Create a single file named `greeting.py` in the project \
                     directory that defines a function greet(name: str) -> str \
                     returning the string 'Hello from ELAI Sonnet, {name}!' and \
                     prints greet('world') when invoked as `python greeting.py`. \
                     Add a docstring to the function. No external dependencies."
        .to_string();
    eprintln!(
        "[submit] project_root={} project_id={} task_len={}",
        project_root.display(),
        project_id,
        task_text.len()
    );
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: task_text.clone(),
        project_id: project_id.clone(),
        submitted_at_ms: 0,
    });
    writer
        .write_payload(&project_id, 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted row to SSB");

    // 11. SSB polling loop. Maintain per-action-type seen counts; print a
    //     one-line summary every time a bucket grows. Bail out early on
    //     terminal signals (PhaseComplete, or PhaseTransition targeting
    //     COMPLETE/IDLE).
    let started_at = std::time::Instant::now();
    let mut last_counts: BTreeMap<String, usize> = BTreeMap::new();

    let poll_outcome = tokio::time::timeout(
        Duration::from_secs(E2E_OUTER_TIMEOUT_SECS),
        async {
            loop {
                let snapshot = snapshot_all(&writer);
                let mut terminal_signal: Option<&'static str> = None;

                for (action, entries) in snapshot.iter() {
                    let label = action_label(*action);
                    let prev = last_counts.get(&label).copied().unwrap_or(0);
                    if entries.len() > prev {
                        // Print only newly-emitted rows, oldest first.
                        for entry in entries.iter().skip(prev) {
                            eprintln!(
                                "[ssb] +{:>4} seq={} action={} agent={} super={} payload={}",
                                entries.len() - prev,
                                entry.seq,
                                label,
                                entry.agent_id,
                                entry.superstep,
                                payload_preview(entry)
                            );

                            // Early-exit check: a PhaseComplete row, or a
                            // PhaseTransition payload that contains the
                            // string "COMPLETE" or "IDLE" in its body,
                            // signals the pipeline reached a terminal
                            // state. The check is intentionally string-
                            // based to avoid pulling the full
                            // PhaseTransitionPayload deserialisation
                            // surface into this fixture — a regression in
                            // that payload shape is the dashboard's
                            // concern, not this test's.
                            if matches!(*action, SsbActionType::PhaseComplete) {
                                terminal_signal = Some("PhaseComplete row emitted");
                            } else if matches!(
                                *action,
                                SsbActionType::PhaseTransition
                            ) {
                                let body = std::str::from_utf8(&entry.payload)
                                    .unwrap_or("");
                                if body.contains("COMPLETE") || body.contains("IDLE") {
                                    terminal_signal = Some(
                                        "PhaseTransition row targeting COMPLETE/IDLE emitted",
                                    );
                                }
                            }
                        }
                        last_counts.insert(label, entries.len());
                    }
                }

                if let Some(reason) = terminal_signal {
                    eprintln!("[ssb] early-exit: {}", reason);
                    return;
                }
                tokio::time::sleep(SSB_POLL_INTERVAL).await;
            }
        },
    )
    .await;
    let elapsed = started_at.elapsed();
    match poll_outcome {
        Ok(()) => eprintln!(
            "[ssb] poll loop exited via terminal signal after {:.1}s",
            elapsed.as_secs_f64()
        ),
        Err(_) => eprintln!(
            "[ssb] poll loop hit {}s wallclock cap (no terminal signal)",
            E2E_OUTER_TIMEOUT_SECS
        ),
    }

    // 12. Final tally — one stderr block summarising what was observed.
    //     The assertion messages below cite this same tally so a failure
    //     report is self-contained.
    let final_snapshot = snapshot_all(&writer);
    let tally: BTreeMap<String, usize> = final_snapshot
        .iter()
        .map(|(action, entries)| (action_label(*action), entries.len()))
        .collect();
    eprintln!("=== E2E Cloud Sonnet Pipeline -- Final Tally ===");
    eprintln!("elapsed_secs: {}", elapsed.as_secs());
    for (label, count) in tally.iter() {
        eprintln!("{}: {}", label, count);
    }
    eprintln!("=== End Final Tally ===");

    let tally_text = tally
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(", ");

    // 13. Phase assertions. Each failure message includes the full tally
    //     so a developer triaging a red CI run sees the SSB state directly.
    //
    //     We INTENTIONALLY do not assert that a TaskCompleted row fired
    //     or that the file `greeting.py` exists on disk — that's a
    //     quality bar (cloud Sonnet successfully completed the task) not
    //     a wiring bar (the orchestrator phases dispatched). A green run
    //     here means "pipeline composed end-to-end with a real provider"
    //     not "cloud Sonnet produced correct output". The Lighthouse-
    //     style quality assertions live in `e2e_real_sidecar_full_build`.

    let task_submitted_count = tally
        .get(&action_label(SsbActionType::TaskSubmitted))
        .copied()
        .unwrap_or(0);
    assert!(
        task_submitted_count >= 1,
        "expected at least 1 TaskSubmitted row (sanity -- the write at \
         line ~submit-task above should have landed); got 0. Tally: {tally_text}"
    );

    let decomposition_count = tally
        .get(&action_label(SsbActionType::ConceptDecompositionNode))
        .copied()
        .unwrap_or(0);
    assert!(
        decomposition_count >= 1,
        "expected at least 1 ConceptDecompositionNode row (L1 decomposition \
         must have fired); got 0. The orchestrator's drive_task path emits \
         decomposition nodes once the LlmProvider returns its first \
         completion, so a zero count here means either (a) the AnthropicClient \
         adapter never reached api.anthropic.com (check auth + base_url logs \
         above), or (b) the decomposer never invoked the provider. Tally: {tally_text}"
    );

    let reasoning_count = tally
        .get(&action_label(SsbActionType::ReasoningTrace))
        .copied()
        .unwrap_or(0);
    assert!(
        reasoning_count >= 1,
        "expected at least 1 ReasoningTrace row (agent dispatch fired); got \
         0. The unconditional `observe` row at task_runner.rs:~3507 lands \
         regardless of provider success, so a zero count here means the \
         drive_task function itself never ran -- check the TaskSubmitted \
         row count and the runner's startup log. Tally: {tally_text}"
    );

    // Optional sanity probe: a GateDecision row would prove the
    // AutonomousGate fired. We log this as informational rather than
    // assert -- the gate may legitimately skip on tasks that route
    // through a deterministic fast-path. Treating it as a hard
    // assertion would couple the test to internal gate policy.
    let gate_count = tally
        .get(&action_label(SsbActionType::GateDecision))
        .copied()
        .unwrap_or(0);
    if gate_count == 0 {
        eprintln!(
            "[note] no GateDecision rows observed (informational, not an assertion). \
             This is fine for tasks that bypass AutonomousGate. Tally: {tally_text}"
        );
    } else {
        eprintln!("[note] GateDecision rows observed: {gate_count}");
    }

    // 14. Graceful shutdown. cancel() flips the CancelHandle; the runner's
    //     tokio::select! wakes on the notify and returns. The 15s budget
    //     is conservative -- the runner shuts down in <50ms in practice;
    //     the margin absorbs an in-flight cloud round-trip the cancel
    //     might race with.
    cancel.cancel();
    let join_result = tokio::time::timeout(Duration::from_secs(SHUTDOWN_TIMEOUT_SECS), handle)
        .await
        .expect("task_runner JoinHandle did not complete within shutdown grace");
    let runner_result = join_result.expect("task_runner panicked");
    runner_result.expect("task_runner returned an error after cancel()");
}
