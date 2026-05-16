// Phase B3 — end-to-end full-pipeline local test.
//
// Submits a TODO prompt against the orchestrator wired with
// ProductionAgentRpcServer + real sidecar, waits up to 15 min for
// decomposition to fire, then asserts the served HTML body >1KB AND
// contains "todo"/"task" case-insensitively.
//
// HOW TO RUN (requires sidecar + tunnel per CLAUDE.md "Remote sidecar (canonical)"):
//   ELAI_E2E_FULL_PIPELINE=1 \
//   ANTHROPIC_BASE_URL=http://localhost:8100 \
//   ANTHROPIC_API_KEY=local-dummy \
//   ELAI_LOCAL_MIN_PROMPT=1 \
//   ELAI_LOCAL_MAX_CONTEXT=20000 \
//   cargo test -p elai-orchestrator --test e2e_full_pipeline_local \
//     -- --ignored --nocapture --test-threads=1
//
// GATE BEHAVIOUR:
//   #[ignore] — skipped by default `cargo test`.
//   Env-gate: also returns early when ELAI_E2E_FULL_PIPELINE != "1" so the
//   test can be --ignored-pinned in CI without any pod.
//
// STUB BAN:
//   No .unwrap() in helper fns. Test code uses .expect("msg") with context.
//   No "// TODO" / "// FIXME". No silent assertion swallowing.
//   No stub/placeholder comments.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use elai_core::SeededRng;
use elai_orchestrator::ssb::{SsbActionType, SsbPayload, SsbReader, TaskSubmittedPayload};
use elai_orchestrator::{
    apply_task_runner_wiring, build_default_baby_step_executor, build_task_runner_wiring,
    locate_default_agent_binary, start_task_runner, ProjectDirResolver, RuntimeWiring,
    ScaffoldApplier,
};
use elai_orchestrator::ipc_gateway::IpcGateway;
use elai_orchestrator::sandbox::SelfModificationLocks;

// ---------------------------------------------------------------------------
// Inline NoOpIntegrationRunner — mirrors cli/src/run_subcommand.rs pattern.
// Re-defined here because the CLI struct is not exported; the canonical recipe
// is to reproduce it inline per context-package-B3.md "Locked" note.
// ---------------------------------------------------------------------------

struct NoOpIntegrationRunner;

impl elai_tools::tiered_validation::IntegrationCommandRunner for NoOpIntegrationRunner {
    fn run_integration<'a>(
        &'a self,
        _level: &'a str,
        _dir: &'a std::path::Path,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = elai_tools::tiered_validation::IntegrationRunOutcome,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async {
            elai_tools::tiered_validation::IntegrationRunOutcome {
                passed: false,
                rules_fired: 0,
                stderr_snippet: Some(
                    "integration tier not wired (NoOpIntegrationRunner in B3 e2e test)"
                        .to_string(),
                ),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Test-local scaffold applier — mirrors e2e_autonomous_build.rs:185-217.
// ---------------------------------------------------------------------------

struct TestScaffoldApplier {
    templates_root: PathBuf,
}

impl ScaffoldApplier for TestScaffoldApplier {
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        task_text: &str,
    ) -> Result<(), String> {
        let variables: BTreeMap<String, String> =
            scaffold::default_build_vars(task_text, None, None, None);
        scaffold::apply::apply_template_from_root(
            &self.templates_root,
            template_name,
            target_dir,
            &variables,
        )
        .map_err(|e| {
            format!(
                "apply_template_from_root(root={}, template={}, target={}) failed: {e}",
                self.templates_root.display(),
                template_name,
                target_dir.display()
            )
        })
    }
}

// ---------------------------------------------------------------------------
// Test-local project-dir resolver — mirrors e2e_autonomous_build.rs:223-231.
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
// Shared helpers
// ---------------------------------------------------------------------------

/// Walk up from `CARGO_MANIFEST_DIR` until `templates/web/vite-react` is found,
/// or honour ELAI_TEMPLATES_DIR. Mirrors e2e_autonomous_build.rs:237-260.
fn find_templates_dir() -> PathBuf {
    if let Ok(override_path) = std::env::var("ELAI_TEMPLATES_DIR") {
        let p = PathBuf::from(override_path);
        assert!(
            p.join("vite-react").is_dir(),
            "ELAI_TEMPLATES_DIR={} does not contain vite-react/",
            p.display()
        );
        return p;
    }
    let mut cur: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = cur.join("templates").join("web");
        if candidate.join("vite-react").is_dir() {
            return candidate;
        }
        if !cur.pop() {
            panic!(
                "could not locate `templates/web/vite-react` walking up from \
                 CARGO_MANIFEST_DIR; set ELAI_TEMPLATES_DIR to override"
            );
        }
    }
}

/// Hash a file on disk using blake3. Returns hex string of the hash.
/// Returns an error string instead of panicking so callers decide how to handle
/// missing files at assertion time.
fn hash_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| {
        format!("hash_file: could not read {}: {e}", path.display())
    })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Write a step marker to stderr and flush immediately. Mirrors e2e_autonomous_build.rs:315.
fn step(stage: &str) {
    use std::io::Write;
    let elapsed_secs = STEP_CLOCK
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs();
    let mut err = std::io::stderr().lock();
    let _ = writeln!(err, "[b3 step +{elapsed_secs}s] {stage}");
    let _ = err.flush();
}

static STEP_CLOCK: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Drain an async reader for at most `timeout`. Mirrors e2e_autonomous_build.rs:331-339.
async fn drain_with_timeout<R>(reader: &mut R, timeout: Duration) -> Vec<u8>
where
    R: tokio::io::AsyncRead + Unpin,
{
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::<u8>::new();
    let _ = tokio::time::timeout(timeout, reader.read_to_end(&mut buf)).await;
    buf
}

/// Poll `127.0.0.1:port` with a short TCP connect until success or `timeout`.
/// Mirrors e2e_autonomous_build.rs:342-354.
async fn wait_for_tcp(port: u16, timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    false
}

/// Run a subprocess with `cmd /C` on Windows (so npm.cmd resolves).
/// Returns `(exit_code, stdout, stderr)`.
async fn run_subprocess(
    program: &str,
    args: &[&str],
    current_dir: &Path,
    timeout: Duration,
) -> (i32, String, String) {
    let output_future = if cfg!(windows) {
        let mut joined = vec![program.to_string()];
        joined.extend(args.iter().map(|s| s.to_string()));
        let cmd_args: Vec<String> = std::iter::once("/C".to_string()).chain(joined).collect();
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.args(&cmd_args).current_dir(current_dir);
        cmd.output()
    } else {
        let mut cmd = tokio::process::Command::new(program);
        cmd.args(args).current_dir(current_dir);
        cmd.output()
    };

    let output = tokio::time::timeout(timeout, output_future)
        .await
        .unwrap_or_else(|_| {
            panic!(
                "subprocess `{} {}` in {} exceeded timeout {:?}",
                program,
                args.join(" "),
                current_dir.display(),
                timeout
            )
        })
        .unwrap_or_else(|e| {
            panic!(
                "subprocess `{} {}` in {} failed to spawn: {e}",
                program,
                args.join(" "),
                current_dir.display()
            )
        });

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

/// Build a per-variant SSB tally string for failure diagnostics.
fn ssb_tally_string(writer: &Arc<elai_orchestrator::Ssb>) -> String {
    let variants = [
        SsbActionType::TaskSubmitted,
        SsbActionType::ReasoningTrace,
        SsbActionType::Checkpoint,
        SsbActionType::Finding,
        SsbActionType::PhaseTransition,
        SsbActionType::PhaseComplete,
        SsbActionType::ConceptDecompositionNode,
        SsbActionType::FileWrite,
        SsbActionType::WebResearchStep,
        SsbActionType::ResearchPathChosen,
        SsbActionType::GroundingCacheEvent,
        SsbActionType::ContextBudgetComputed,
        SsbActionType::ToolDispatch,
    ];
    variants
        .iter()
        .filter_map(|v| {
            let n = writer.query_by_type(*v).unwrap_or_default().len();
            if n > 0 {
                Some(format!("{v:?}={n}"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// List files under `dir` with sizes, returning a formatted string.
fn list_dir_files(dir: &Path) -> String {
    let mut lines = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let meta = entry.metadata().ok();
            let size = meta.map(|m| m.len()).unwrap_or(0);
            lines.push(format!("  {} ({} bytes)", entry.path().display(), size));
        }
    }
    if lines.is_empty() {
        format!("  <empty or missing: {}>", dir.display())
    } else {
        lines.join("\n")
    }
}

/// Read the first 200 chars of the first App.{jsx,tsx} found under `src_dir`.
fn read_app_snippet(src_dir: &Path) -> String {
    for ext in ["jsx", "tsx"] {
        let candidate = src_dir.join(format!("App.{ext}"));
        if candidate.exists() {
            if let Ok(bytes) = std::fs::read(&candidate) {
                let text = String::from_utf8_lossy(&bytes);
                return text.chars().take(200).collect();
            }
        }
    }
    "<App.jsx / App.tsx not found>".to_string()
}

// ---------------------------------------------------------------------------
// The test
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires --ignored + ELAI_E2E_FULL_PIPELINE=1 + sidecar booted per CLAUDE.md \
            (EU-RO-1 pod capacity-constrained; not run in CI)"]
async fn e2e_full_pipeline_local() {
    // GATE: skip unless ELAI_E2E_FULL_PIPELINE=1.
    if std::env::var("ELAI_E2E_FULL_PIPELINE").ok().as_deref() != Some("1") {
        eprintln!(
            "B3 full-pipeline test requires ELAI_E2E_FULL_PIPELINE=1 \
             and a manually-booted sidecar. See CLAUDE.md \"Remote sidecar (canonical)\" \
             and workflow/remote-setup/REMOTE-SIDECAR.md for the 3-step health check."
        );
        return;
    }

    // B. Detect cloud-vs-sidecar mode. Cloud mode is triggered by
    //    ELAI_E2E_FULL_PIPELINE_CLOUD=1 (test-local) OR by ANTHROPIC_BASE_URL
    //    matching api.anthropic.com (user override). Cloud mode uses
    //    resolve_auth_with_auto_refresh -> BearerToken; sidecar uses
    //    "local-dummy" placeholder against http://localhost:8100.
    let cloud_mode = std::env::var("ELAI_E2E_FULL_PIPELINE_CLOUD").as_deref() == Ok("1")
        || std::env::var("ANTHROPIC_BASE_URL")
            .map(|u| u.contains("api.anthropic.com"))
            .unwrap_or(false);
    let base_url = if cloud_mode {
        std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string())
    } else {
        std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8100".to_string())
    };
    eprintln!(
        "[setup] mode={} base_url={}",
        if cloud_mode { "cloud" } else { "sidecar" },
        base_url
    );
    if cloud_mode {
        // Route the orchestrator's decomposition path through
        // CloudGroundingClient instead of SidecarGroundingClient.
        // (task_runner.rs:4134 reads this env var.)
        std::env::set_var("ELAI_GROUNDING_BACKEND", "cloud");
        // Cloud auth comes from saved OAuth (auto-refreshed via patch-R1).
        // Do NOT set ANTHROPIC_API_KEY in cloud mode — that would route to
        // AuthSource::ApiKey and bypass the BearerToken (OAuth) headers
        // d063f97 requires for the Claude Code rate-limit bucket.
        std::env::remove_var("ANTHROPIC_API_KEY");
    } else {
        std::env::set_var("ANTHROPIC_API_KEY", "local-dummy");
    }
    if std::env::var("ELAI_LOCAL_MIN_PROMPT").is_err() {
        std::env::set_var("ELAI_LOCAL_MIN_PROMPT", "1");
    }
    if std::env::var("ELAI_LOCAL_MAX_CONTEXT").is_err() {
        std::env::set_var("ELAI_LOCAL_MAX_CONTEXT", "20000");
    }

    // C. Health-check BEFORE starting the test timer. Bad auth or
    //    unreachable endpoint panics fast rather than burning 15 min.
    //    cloud_mode: resolve OAuth -> tiny send_message -> assert 200.
    //    sidecar:    send dummy 4-token request to the local tunnel.
    let resolved_auth = if cloud_mode {
        use elai_providers::{AuthSource, InputMessage, MessageRequest};
        // resolve_saved_oauth_token_set internally uses Runtime::new()::block_on,
        // which collides with our #[tokio::test] runtime; bounce through
        // spawn_blocking (mirrors providers/tests/anthropic_oauth_live_smoke.rs).
        let auth = tokio::task::spawn_blocking(elai_providers::resolve_auth_with_auto_refresh)
            .await
            .expect("spawn_blocking join")
            .expect(
                "cloud mode: resolve_auth_with_auto_refresh must produce a credential. \
                 Run `elai login` first if ~/.elai/credentials.json is missing.",
            );
        let health_client = elai_providers::AnthropicClient::from_auth(auth.clone())
            .with_base_url(base_url.clone());
        let health_req = MessageRequest {
            model: std::env::var("ELAI_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
            max_tokens: 4,
            messages: vec![InputMessage::user_text("ok")],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: None,
        };
        let response = health_client
            .send_message(&health_req)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "cloud health-check failed against {base_url}: {e}. \
                     Run `elai login` to refresh OAuth, or check `/api/oauth/usage`."
                )
            });
        eprintln!(
            "[health] cloud OK model={} usage_in={} usage_out={}",
            response.model, response.usage.input_tokens, response.usage.output_tokens
        );
        Some(auth)
    } else {
        use elai_providers::{InputMessage, MessageRequest};
        let health_client = elai_providers::AnthropicClient::new("local-dummy")
            .with_base_url(base_url.clone());
        let health_req = MessageRequest {
            model: "qwen".to_string(),
            max_tokens: 4,
            messages: vec![InputMessage::user_text("ok")],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: None,
        };
        health_client
            .send_message(&health_req)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "sidecar not reachable at {base_url} — start the pod + tunnel first. \
                     Error: {e}"
                )
            });
        None
    };

    // 20-minute outer wallclock (15 min LLM/decomposition + 5 min build/preview).
    tokio::time::timeout(Duration::from_secs(1200), async {
        step("01_test_started");

        // D. Fresh project_root + session_id distinct from the real-sidecar test.
        let tmp = tempfile::tempdir().expect("tempdir for e2e_full_pipeline_local");
        let project_root = tmp.path().to_path_buf();
        let session_id = "test-b3-full-pipeline";
        step("02_tempdir_ready");

        // Build production wiring bundle (same chokepoint the CLI + dashboard use).
        let mut wiring: RuntimeWiring = build_task_runner_wiring(&project_root, session_id)
            .await
            .expect("build_task_runner_wiring for B3 test");
        step("03_wiring_built");

        // F. Wire ProductionAgentRpcServer.
        //    Mirrors cli/src/run_subcommand.rs:170-237 sidecar branch exactly,
        //    with NoOpIntegrationRunner defined at the top of this file.
        {
            let ipc_dir = project_root
                .join(".elai_cc")
                .join("sessions")
                .join(session_id)
                .join("ipc");
            std::fs::create_dir_all(&ipc_dir)
                .expect("create ipc_dir for B3 test");

            match locate_default_agent_binary() {
                Ok(agent_bin) => {
                    let gateway = Arc::new(tokio::sync::Mutex::new(
                        IpcGateway::with_os_clock(
                            ipc_dir.clone(),
                            SelfModificationLocks::new(),
                        ),
                    ));

                    let model = std::env::var("ELAI_MODEL")
                        .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
                    let max_tokens = std::env::var("ELAI_MAX_TOKENS")
                        .ok()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(4096);

                    let dispatcher: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> = {
                        let validator = elai_tools::dispatcher::default_validator(
                            Arc::new(NoOpIntegrationRunner),
                            "rust",
                        );
                        Arc::new(elai_tools::dispatcher::InProcessToolDispatcher::new(
                            Arc::new(validator),
                        ))
                    };

                    // cloud_mode uses the resolved OAuth BearerToken from step
                    // C; sidecar mode uses the local-dummy placeholder.
                    let anthropic_client = match &resolved_auth {
                        Some(auth) => elai_providers::AnthropicClient::from_auth(auth.clone())
                            .with_base_url(base_url.clone()),
                        None => elai_providers::AnthropicClient::new("local-dummy")
                            .with_base_url(base_url.clone()),
                    };
                    let rpc_server: Arc<
                        dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static,
                    > = Arc::new(elai_orchestrator::ProductionAgentRpcServer::new(
                        anthropic_client,
                        dispatcher,
                        model,
                        max_tokens,
                    ));

                    wiring.baby_step_executor = Some(build_default_baby_step_executor(
                        SeededRng::new(0xB3_F011),
                        agent_bin,
                        ipc_dir,
                        gateway,
                        rpc_server,
                    ));
                    step("03b_production_rpc_server_installed");
                }
                Err(e) => {
                    panic!(
                        "B3 test requires the agent binary to be built. \
                         locate_default_agent_binary() failed: {e}. \
                         Build with `cargo build -p elai-cc-agent` first, \
                         or set ELAI_AGENT_BIN to the binary path."
                    );
                }
            }
        }

        let project_id = elai_core::project_id_from_path(&project_root);

        // Apply wiring to TaskRunnerConfig.
        let cursor_path = project_root.join(".elai_cc").join("cursor.json");
        let mut cfg = apply_task_runner_wiring(cursor_path, &wiring);

        // Some+Some chokepoint guard.
        assert!(
            cfg.feasibility_prober.is_some(),
            "apply_task_runner_wiring must populate feasibility_prober (B3 chokepoint)"
        );
        assert!(
            cfg.layer1a_processor.is_some(),
            "apply_task_runner_wiring must populate layer1a_processor (B3 chokepoint)"
        );

        // H. Inject scaffold applier + project-dir resolver.
        let templates_root = find_templates_dir();
        let app_dir = project_root.join("app");

        cfg.scaffold_applier = Some(Arc::new(TestScaffoldApplier {
            templates_root: templates_root.clone(),
        }));
        cfg.project_dir_resolver = Some(Arc::new(StaticTestResolver {
            dir: app_dir.clone(),
        }));

        // Record baseline App.{jsx,tsx} hash AFTER scaffold is applied.
        // The scaffold hasn't been applied yet (runner applies it on the first
        // TaskSubmitted row via drive_web_scaffold_task or the agent path).
        // We record the hash post-submission once the app_dir exists.

        // H-spawn: start the task runner.
        let writer = wiring.ssb.clone();
        let (handle, cancel) = start_task_runner(writer.clone(), cfg);
        step("04_runner_spawned");

        tokio::time::sleep(Duration::from_millis(100)).await;

        // I. Submit TaskSubmitted row with the verbatim B3 prompt.
        // "Build me a single-page React TODO app..." does NOT match the
        // "make me a " scaffold short-circuit (task_runner.rs:~3482), so it
        // routes through the full agent-decomposition path. Confirmed by
        // context-package-B3.md MEDIUM validation note.
        let prompt = "Build me a single-page React TODO app with localStorage persistence. \
                      Allow add, mark-complete, and delete actions.";
        let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
            task: prompt.to_string(),
            project_id: project_id.clone(),
            submitted_at_ms: 0,
        });
        writer
            .write_payload(&project_id, 0, payload, Vec::new())
            .await
            .expect("write TaskSubmitted to SSB for B3 test");
        step("05_task_submitted");

        // J. Poll SSB for up to 15 min. Success signal: ≥1 ConceptDecompositionNode
        //    AND (≥1 FileWrite OR app_dir contains non-scaffold App.jsx/tsx).
        //    On heartbeat print tally; on timeout panic with full diagnostics.
        let llm_deadline = std::time::Instant::now() + Duration::from_secs(900);
        let mut seen_findings: usize = 0;
        let mut got_decomposition = false;
        let mut got_file_write = false;
        let mut last_heartbeat = std::time::Instant::now();

        step("05a_llm_phase_wait_start");
        while std::time::Instant::now() < llm_deadline {
            // Check for ConceptDecompositionNode (assertion 1).
            let decomp_entries = writer
                .query_by_type(SsbActionType::ConceptDecompositionNode)
                .unwrap_or_default();
            if !decomp_entries.is_empty() {
                got_decomposition = true;
            }

            // Check for FileWrite rows (assertion 2 — SSB path).
            let fw_entries = writer
                .query_by_type(SsbActionType::FileWrite)
                .unwrap_or_default();
            if !fw_entries.is_empty() {
                got_file_write = true;
            }

            // Check for PhaseComplete as an end-of-phase signal.
            let phase_done = !writer
                .query_by_type(SsbActionType::PhaseComplete)
                .unwrap_or_default()
                .is_empty();

            if got_decomposition && (got_file_write || phase_done) {
                step(&format!(
                    "05b_signals_received decomp={} fw={} phase_done={}",
                    got_decomposition, got_file_write, phase_done
                ));
                break;
            }

            // Stream new Finding rows to stderr.
            let findings = writer
                .query_by_type(SsbActionType::Finding)
                .unwrap_or_default();
            for entry in findings.iter().skip(seen_findings) {
                if let Ok(SsbPayload::Finding(p)) = rmp_serde::from_slice(&entry.payload) {
                    eprintln!(
                        "[b3 diag] finding seq={} severity={} category={} message={:?}",
                        entry.seq, p.severity, p.category, p.message
                    );
                }
            }
            seen_findings = findings.len();

            if last_heartbeat.elapsed() >= Duration::from_secs(30) {
                let tally = ssb_tally_string(&writer);
                step(&format!(
                    "05c_heartbeat decomp={} fw={} ssb=[{}]",
                    got_decomposition, got_file_write, tally
                ));
                last_heartbeat = std::time::Instant::now();
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // K. Timeout check — panic with full diagnostics.
        if !got_decomposition {
            let tally = ssb_tally_string(&writer);
            let src_listing = list_dir_files(&app_dir.join("src"));
            let app_snippet = read_app_snippet(&app_dir.join("src"));
            cancel.cancel();
            panic!(
                "B3: no ConceptDecompositionNode within 15 min. \
                 project_root={} project_id={} prompt={:?}\n\
                 ssb_tally=[{}]\n\
                 tempdir/src listing:\n{}\n\
                 App snippet: {:?}",
                project_root.display(),
                project_id,
                prompt,
                tally,
                src_listing,
                app_snippet
            );
        }
        step("06_decomposition_confirmed");

        // L. Assertion 1: ≥1 ConceptDecompositionNode (already verified above).
        let decomp_rows = writer
            .query_by_type(SsbActionType::ConceptDecompositionNode)
            .expect("query ConceptDecompositionNode rows");
        assert!(
            !decomp_rows.is_empty(),
            "B3 assertion 1 FAILED: expected ≥1 ConceptDecompositionNode SSB row \
             (proves SidecarGroundingClient decomposition fired). ssb_tally=[{}]",
            ssb_tally_string(&writer)
        );

        // M. Assertion 2: ≥1 FileWrite row whose path is under app_dir/src/,
        //    OR App.{jsx,tsx} on disk has content that differs from the raw
        //    vite-react scaffold template baseline (direct-fs path via B2c IPC).
        let fw_rows = writer
            .query_by_type(SsbActionType::FileWrite)
            .expect("query FileWrite rows");

        let src_dir = app_dir.join("src");
        let src_dir_prefix = src_dir.to_string_lossy().to_lowercase();

        let ssb_write_under_src = fw_rows.iter().any(|entry| {
            if let Ok(SsbPayload::FileWrite(p)) = rmp_serde::from_slice(&entry.payload) {
                p.path.to_lowercase().contains(&src_dir_prefix)
                    || p.path.to_lowercase().contains("app.jsx")
                    || p.path.to_lowercase().contains("app.tsx")
            } else {
                false
            }
        });

        // Fallback: compare App.jsx/tsx on disk against the vite-react template
        // baseline. A real agent write must differ from the scaffold default.
        let disk_write_detected = {
            let mut found = false;
            for ext in ["jsx", "tsx"] {
                let app_file = src_dir.join(format!("App.{ext}"));
                let template_file = templates_root
                    .join("vite-react")
                    .join("src")
                    .join(format!("App.{ext}"));
                if app_file.exists() && template_file.exists() {
                    let app_hash = hash_file(&app_file).unwrap_or_default();
                    let tmpl_hash = hash_file(&template_file).unwrap_or_default();
                    if !app_hash.is_empty() && app_hash != tmpl_hash {
                        found = true;
                    }
                }
            }
            found
        };

        assert!(
            ssb_write_under_src || disk_write_detected,
            "B3 assertion 2 FAILED: expected ≥1 FileWrite SSB row under src/ \
             OR App.{{jsx,tsx}} differing from template baseline. \
             fw_rows={} ssb_write_under_src={} disk_write_detected={}\n\
             tempdir/src listing:\n{}\n\
             App snippet: {:?}\n\
             ssb_tally=[{}]",
            fw_rows.len(),
            ssb_write_under_src,
            disk_write_detected,
            list_dir_files(&src_dir),
            read_app_snippet(&src_dir),
            ssb_tally_string(&writer)
        );
        step("07_file_write_confirmed");

        // N. npm install + npm run build (via web_validator helpers, cmd /C on Windows).
        let package_json = app_dir.join("package.json");
        assert!(
            package_json.exists(),
            "B3: package.json missing at {} — scaffold or agent did not write it",
            package_json.display()
        );

        step("08_npm_install_start");
        let (npm_install_code, npm_install_stdout, npm_install_stderr) =
            run_subprocess("npm", &["install"], &app_dir, Duration::from_secs(240)).await;
        step("09_npm_install_done");
        assert_eq!(
            npm_install_code, 0,
            "B3 assertion 3 (npm install): `npm install` in {} exited {}\n\
             stdout:\n{}\nstderr:\n{}",
            app_dir.display(),
            npm_install_code,
            npm_install_stdout,
            npm_install_stderr
        );

        step("10_npm_build_start");
        let build_result = elai_web_validator::run_web_build(
            &app_dir,
            "npm run build",
            &app_dir.join("dist"),
        )
        .await
        .expect("run_web_build invocation for B3 test");
        step("11_npm_build_done");
        assert!(
            build_result.success,
            "B3 assertion 3 FAILED: `npm run build` in {} reported success=false \
             exit_code={} stderr_errors={:?}\n\
             ssb_tally=[{}]",
            app_dir.display(),
            build_result.exit_code,
            build_result.stderr_errors,
            ssb_tally_string(&writer)
        );

        // O. Dynamic preview port — bind 0, note port, build command, drop listener,
        //    spawn vite preview. `--host 127.0.0.1` mandatory for Windows dual-stack.
        //    Mirrors e2e_autonomous_build.rs:658-713.
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind 127.0.0.1:0 for B3 free-port discovery");
        let preview_port: u16 = listener
            .local_addr()
            .expect("listener.local_addr for B3 preview port")
            .port();
        let preview_port_str = preview_port.to_string();

        let mut preview_cmd = if cfg!(windows) {
            let mut c = tokio::process::Command::new("cmd");
            c.args([
                "/C",
                "npm",
                "run",
                "preview",
                "--",
                "--host",
                "127.0.0.1",
                "--port",
                &preview_port_str,
                "--strictPort",
            ]);
            c
        } else {
            let mut c = tokio::process::Command::new("npm");
            c.args([
                "run",
                "preview",
                "--",
                "--host",
                "127.0.0.1",
                "--port",
                &preview_port_str,
                "--strictPort",
            ]);
            c
        };
        preview_cmd
            .current_dir(&app_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        step(&format!("12_preview_port_picked port={preview_port}"));
        drop(listener);
        let mut preview_child = preview_cmd
            .spawn()
            .expect("failed to spawn `npm run preview` subprocess for B3 test");
        step("13_preview_spawned");

        // P. Wait up to 90s for the preview port to accept TCP.
        let port_up = wait_for_tcp(preview_port, Duration::from_secs(90)).await;
        step(&format!("14_wait_for_tcp_returned port_up={port_up}"));
        if !port_up {
            let _ = preview_child.start_kill();
            let _ = tokio::time::timeout(Duration::from_secs(5), preview_child.wait()).await;
            let mut stdout_buf = Vec::<u8>::new();
            let mut stderr_buf = Vec::<u8>::new();
            if let Some(mut s) = preview_child.stdout.take() {
                stdout_buf = drain_with_timeout(&mut s, Duration::from_secs(3)).await;
            }
            if let Some(mut s) = preview_child.stderr.take() {
                stderr_buf = drain_with_timeout(&mut s, Duration::from_secs(3)).await;
            }
            let stdout_txt = String::from_utf8_lossy(&stdout_buf);
            let stderr_txt = String::from_utf8_lossy(&stderr_buf);
            panic!(
                "B3: vite preview never became reachable on 127.0.0.1:{} within 90s\n\
                 ssb_tally=[{}]\n\
                 --- preview stdout ({} bytes) ---\n{}\n\
                 --- preview stderr ({} bytes) ---\n{}",
                preview_port,
                ssb_tally_string(&writer),
                stdout_buf.len(),
                stdout_txt,
                stderr_buf.len(),
                stderr_txt,
            );
        }

        let preview_url = format!("http://127.0.0.1:{preview_port}");
        step("15_http_probe_start");

        // Q. HTTP probe: 5x retry with 200ms gap. Mirror e2e_autonomous_build.rs:770-845.
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("reqwest Client for B3 test");

        let mut last_err: Option<reqwest::Error> = None;
        let mut resp_opt: Option<reqwest::Response> = None;
        for attempt in 0..5u8 {
            if let Ok(Some(status)) = preview_child.try_wait() {
                eprintln!(
                    "[b3 diag] preview child exited before HTTP GET succeeded: {:?}",
                    status
                );
                break;
            }
            match http_client.get(&preview_url).send().await {
                Ok(r) => {
                    resp_opt = Some(r);
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "[b3 diag] HTTP GET attempt {} on {} failed: {}",
                        attempt + 1,
                        preview_url,
                        e
                    );
                    last_err = Some(e);
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            }
        }

        let resp = match resp_opt {
            Some(r) => r,
            None => {
                let _ = preview_child.start_kill();
                let _ = tokio::time::timeout(Duration::from_secs(5), preview_child.wait()).await;
                let mut stdout_buf = Vec::<u8>::new();
                let mut stderr_buf = Vec::<u8>::new();
                if let Some(mut s) = preview_child.stdout.take() {
                    stdout_buf = drain_with_timeout(&mut s, Duration::from_secs(3)).await;
                }
                if let Some(mut s) = preview_child.stderr.take() {
                    stderr_buf = drain_with_timeout(&mut s, Duration::from_secs(3)).await;
                }
                let stdout_txt = String::from_utf8_lossy(&stdout_buf);
                let stderr_txt = String::from_utf8_lossy(&stderr_buf);
                panic!(
                    "B3: HTTP GET {preview_url} failed after 5 retries (last err: {:?})\n\
                     ssb_tally=[{}]\n\
                     tempdir/src listing:\n{}\n\
                     App snippet: {:?}\n\
                     --- preview stdout ({} bytes) ---\n{}\n\
                     --- preview stderr ({} bytes) ---\n{}",
                    last_err,
                    ssb_tally_string(&writer),
                    list_dir_files(&app_dir.join("src")),
                    read_app_snippet(&app_dir.join("src")),
                    stdout_buf.len(),
                    stdout_txt,
                    stderr_buf.len(),
                    stderr_txt,
                );
            }
        };

        // R. Assertion 4: 200 OK, body >1KB, body contains "todo" or "task".
        let status = resp.status();
        let body = resp
            .text()
            .await
            .expect("read preview HTTP response body for B3 test");

        // Kill preview before any assertion can panic and leave a dangling process.
        let _ = preview_child.start_kill();

        assert!(
            status.is_success(),
            "B3 assertion 4a FAILED: preview HTTP status {} (expected 2xx) for {preview_url}\n\
             ssb_tally=[{}]",
            status,
            ssb_tally_string(&writer)
        );

        assert!(
            body.len() > 1024,
            "B3 assertion 4b FAILED: preview body too short ({} bytes, expected >1KB) \
             for {preview_url}\n\
             ssb_tally=[{}]\n\
             tempdir/src listing:\n{}\n\
             App snippet: {:?}\n\
             body first 200 bytes: {:?}",
            body.len(),
            ssb_tally_string(&writer),
            list_dir_files(&app_dir.join("src")),
            read_app_snippet(&app_dir.join("src")),
            &body.chars().take(200).collect::<String>()
        );

        let body_lower = body.to_ascii_lowercase();
        assert!(
            body_lower.contains("todo") || body_lower.contains("task"),
            "B3 assertion 4c FAILED: preview body does not contain \"todo\" or \"task\" \
             (body_bytes={}) for {preview_url}\n\
             ssb_tally=[{}]\n\
             tempdir/src listing:\n{}\n\
             App snippet: {:?}\n\
             body first 200 chars: {:?}",
            body.len(),
            ssb_tally_string(&writer),
            list_dir_files(&app_dir.join("src")),
            read_app_snippet(&app_dir.join("src")),
            &body.chars().take(200).collect::<String>()
        );

        eprintln!(
            "[b3 diag] assertion 4 PASS: HTTP status={} body_bytes={} url={}",
            status,
            body.len(),
            preview_url
        );
        step("16_http_probe_ok");

        // Graceful shutdown.
        cancel.cancel();
        let join_result = tokio::time::timeout(Duration::from_secs(10), handle)
            .await
            .expect("task runner JoinHandle did not complete within 10s of cancel()");
        let runner_result = join_result.expect("task runner panicked during B3 test");
        runner_result.expect("task runner returned an error after B3 cancel()");

        let _ = tokio::time::timeout(Duration::from_secs(5), preview_child.wait()).await;
    })
    .await
    .expect(
        "B3 e2e_full_pipeline_local exceeded 20-minute wall-clock budget \
         (15 min LLM + 5 min build/preview). \
         Check sidecar latency and pod connectivity."
    );
}
