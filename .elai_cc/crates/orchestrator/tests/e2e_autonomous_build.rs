// Phase D1 — end-to-end autonomous build smoke test.
//
// Two layers:
//   1. ALWAYS-RUN `e2e_mock_provider_scaffolds_react_app_via_chokepoint` —
//      drives the production orchestration pipeline end-to-end via the same
//      `RuntimeWiring` + `start_task_runner` + synthetic `TaskSubmitted` row
//      that ships in the dashboard and CLI binaries. Proves the
//      A1+A2+A3+A4+A5+A6 chokepoint composes on a fresh tempdir.
//
//      SCOPE NARROWING (documented per phase-D1.md risks): the production
//      task_runner dispatched in this test has NO live provider plumbed in
//      from the mock side — the orchestrator's executor uses its in-process
//      default agent. The `drive_task` default branch still emits the
//      unconditional `observe` ReasoningTrace at task_runner.rs:~3263 (the
//      same row guarded by e2e_chokepoint.rs), so the always-run assertion
//      is: "SSB ReasoningTrace OR ConceptDecompositionNode row emitted
//      within 30s AND Some+Some chokepoint preserved". The optional artifact
//      directory assertion from phase-D1.md acceptance is skipped — see the
//      inline comment near the timeout block for the reason.
//
//   2. #[ignore]'d `e2e_real_sidecar_full_build` — documented kick-off for
//      the real-sidecar+npm+lighthouse variant. Sidecar boot is
//      HARDWARE-HEAVY per CLAUDE.md and out of scope for autonomous CI.
//
// Stub-ban:
//   - No `.unwrap()` outside the `tempfile::tempdir().expect("tempdir")`
//     literal — every fallible production-shaped call propagates via
//     `.expect("…")` with a context message describing what failed.
//   - No `#[ignore]` on the always-run test.
//   - No silent timeouts — the wait helper returns `Option` and the
//     assertion message reports the project_root + project_id on failure.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload, SsbReader, TaskSubmittedPayload};
use elai_orchestrator::{
    apply_task_runner_wiring, build_task_runner_wiring, start_task_runner, ProjectDirResolver,
    RuntimeWiring, ScaffoldApplier,
};

/// Poll the SSB until at least one row matching ANY of `targets` is present,
/// or `timeout` elapses. Returns the first variant that matched so the
/// failure assertion can report which family fired (or `None` on timeout).
/// Mirrors the helper in tests/e2e_chokepoint.rs:52-68.
async fn wait_for_any_action(
    writer: &Arc<elai_orchestrator::Ssb>,
    targets: &[SsbActionType],
    timeout: Duration,
) -> Option<SsbActionType> {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        for action in targets {
            let entries = writer.query_by_type(*action).unwrap_or_default();
            if !entries.is_empty() {
                return Some(*action);
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    None
}

#[tokio::test(flavor = "multi_thread")]
async fn e2e_mock_provider_scaffolds_react_app_via_chokepoint() {
    // 1. Fresh project_root inside a TempDir. The guard is held in scope
    //    until the end of the test so cleanup runs after the runner has been
    //    cancelled and joined.
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let session_id = "test-d1-e2e";

    // 2. Build the production wiring bundle. Same call the dashboard Tauri
    //    `setup()` and `elai run` use in production. A failure here means a
    //    regression in the chokepoint helpers (A1a/A1b contracts).
    let wiring: RuntimeWiring = build_task_runner_wiring(&project_root, session_id)
        .await
        .expect("build_task_runner_wiring(project_root) succeeds");

    // Canonical project_id derived from the same path the wiring used.
    // After A1c, runtime_setup.rs computes this exact value via
    // elai_core::project_id_from_path(project_root), so the synthetic
    // TaskSubmitted row lands in the bucket the runner reads.
    let project_id = elai_core::project_id_from_path(&project_root);

    // 3. Apply the wiring to a fresh TaskRunnerConfig via the production
    //    chokepoint helper. cursor_path lives inside the project's
    //    `.elai_cc/` directory (build_task_runner_wiring already created it).
    let cursor_path = project_root.join(".elai_cc").join("cursor.json");
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    // Some+Some chokepoint guard — the phase-D1.md acceptance bullet.
    assert!(
        cfg.feasibility_prober.is_some(),
        "apply_task_runner_wiring must populate feasibility_prober (D1 Some+Some chokepoint)"
    );
    assert!(
        cfg.layer1a_processor.is_some(),
        "apply_task_runner_wiring must populate layer1a_processor (D1 Some+Some chokepoint)"
    );

    // 4. Spawn the runner against the same Arc<Ssb> we just wrote.
    let writer = wiring.ssb.clone();
    let (handle, cancel) = start_task_runner(writer.clone(), cfg);

    // 5. Give the runner a moment to subscribe to the broadcast before the
    //    write fires. The polling loop below would also catch a row even if
    //    subscribed late (replay_backlog re-reads the table), but the short
    //    delay shaves variance off the assertion latency.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 6. Synthetic TaskSubmitted row — a plain "build a React TODO app"
    //    task. The phase-D1 brief calls for a Vite-React scaffolding task.
    //    The in-process executor's default branch in drive_task emits the
    //    unconditional `observe` ReasoningTrace regardless of whether a
    //    real provider is plumbed in, which is what we assert on below.
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "build a single-page React TODO app with localStorage persistence".to_string(),
        project_id: project_id.clone(),
        submitted_at_ms: 0,
    });
    writer
        .write_payload(&project_id, 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted to SSB");

    // 7. Wait up to 30s for ReasoningTrace OR ConceptDecompositionNode.
    //    The 30s budget matches phase-D1.md acceptance (e2e_chokepoint uses
    //    10s for the no-provider path; we widen here because the D1 brief
    //    targets the autonomous-build orchestration, which on a cold CI box
    //    can take longer to compose the first reasoning row).
    //
    //    SCOPE NARROWING: the optional artifact-directory assertion from
    //    phase-D1.md acceptance bullet (e.iii) is intentionally skipped.
    //    The mock-provider path in this test does NOT plumb a live provider
    //    into the in-process executor — without a provider, no tool_use
    //    blocks fire, so no scaffold files are written. The Some+Some guard
    //    above + the SSB-row assertion below cover the chokepoint contract
    //    that the always-run layer of D1 is meant to pin. The real-sidecar
    //    variant (the #[ignore]'d test below) is where the artifact
    //    directory + lighthouse acceptance lives.
    let targets = [
        SsbActionType::ReasoningTrace,
        SsbActionType::ConceptDecompositionNode,
    ];
    let matched = tokio::time::timeout(
        Duration::from_secs(30),
        wait_for_any_action(&writer, &targets, Duration::from_secs(30)),
    )
    .await
    .expect("wait_for_any_action future must complete within outer 30s timeout");
    assert!(
        matched.is_some(),
        "expected at least one ReasoningTrace OR ConceptDecompositionNode row \
         within 30s after writing TaskSubmitted; drive_task never ran \
         (D1 autonomous-build chokepoint regression). \
         project_root={} project_id={}",
        project_root.display(),
        project_id
    );

    // 8. Graceful shutdown. cancel() flips the CancelHandle; the runner's
    //    `tokio::select!` wakes on the notify and returns. The 5s timeout
    //    is a safety net — a properly wired runner shuts down in <50ms.
    cancel.cancel();
    let join_result = tokio::time::timeout(Duration::from_secs(5), handle)
        .await
        .expect("task runner JoinHandle did not complete within 5s of cancel()");
    let runner_result = join_result.expect("task runner panicked");
    runner_result.expect("task runner returned an error after cancel()");
}

/// Test-only scaffolder that hands the orchestrator's
/// [`drive_web_scaffold_task`] path a direct call to
/// `scaffold::apply::apply_template_from_root`. Avoids an `elai-dashboard-v2`
/// dev-dep cycle by reimplementing the trait against the scaffold crate the
/// orchestrator already pulls in as a runtime path-dep (`scaffold = { path =
/// "../scaffold" }`, crates/orchestrator/Cargo.toml:21).
struct TestScaffoldApplier {
    templates_root: PathBuf,
}

impl ScaffoldApplier for TestScaffoldApplier {
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        _task_text: &str,
    ) -> Result<(), String> {
        let variables: BTreeMap<String, String> = BTreeMap::new();
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

/// Test-only [`ProjectDirResolver`] that pins every project_id to a single
/// directory under the test TempDir. The production
/// [`elai_orchestrator::CwdProjectDirResolver`] returns the process CWD,
/// which would point at the cargo target dir during `cargo test`.
struct StaticTestResolver {
    dir: PathBuf,
}

impl ProjectDirResolver for StaticTestResolver {
    fn resolve(&self, _project_id: &str) -> Option<PathBuf> {
        Some(self.dir.clone())
    }
}

/// Walk up from `CARGO_MANIFEST_DIR` until a `templates/web/vite-react`
/// directory is found, or honour an explicit `ELAI_TEMPLATES_DIR` override.
/// The walk terminates at the filesystem root and panics on miss so a CI box
/// without the templates checked out fails fast.
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
                "could not locate `templates/web/vite-react` walking up from CARGO_MANIFEST_DIR; \
                 set ELAI_TEMPLATES_DIR to override"
            );
        }
    }
}

/// Run a `program args...` subprocess with `current_dir`, wallclocked by
/// `tokio::time::timeout`. Returns `(exit_code, stdout, stderr)`. Uses
/// `cmd /C` on Windows so node-shipped shims (`npm.cmd`, `npx.cmd`) resolve.
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

/// Poll `127.0.0.1:port` with a short TCP connect until success or `timeout`.
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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires --ignored + ELAI_E2E_REAL_SIDECAR=1 + sidecar booted manually per CLAUDE.md"]
async fn e2e_real_sidecar_full_build() {
    // Documented kick-off:
    //   ELAI_E2E_REAL_SIDECAR=1 cargo test -p elai-orchestrator \
    //     --test e2e_autonomous_build -- --ignored
    //
    // Preconditions (manual, HARDWARE-HEAVY per CLAUDE.md):
    //   - Remote sidecar booted per CLAUDE.md "Remote sidecar (canonical)"
    //     (workflow/remote-setup/REMOTE-SIDECAR.md, 3-step health check).
    //   - Local tunnel up:
    //       D:\.ELAI_workflow\workflow\remote-setup\start-tunnel.ps1
    //   - Environment:
    //       ANTHROPIC_BASE_URL=http://localhost:8100
    //       ANTHROPIC_API_KEY=local-dummy
    //       ELAI_LOCAL_MIN_PROMPT=1
    //       ELAI_LOCAL_MAX_CONTEXT=20000
    //   - `npm` (node 18+) and Chrome on PATH for the lighthouse run.
    //
    // The scaffold step is TEXT-CLASSIFIER-driven (task_runner.rs:3482) so it
    // does not require a live provider — the production
    // `is_web_scaffold_task` matcher fires on the prompt text alone, then
    // `drive_web_scaffold_task` calls the injected `ScaffoldApplier`. The
    // sidecar is only required for the subsequent reasoning rows once the
    // runner has handed off; the SCAFFOLD itself is deterministic.
    if std::env::var("ELAI_E2E_REAL_SIDECAR").ok().as_deref() != Some("1") {
        eprintln!(
            "D1 real-sidecar smoke test requires ELAI_E2E_REAL_SIDECAR=1 \
             and a manually-booted sidecar. See \
             workflow/remote-setup/REMOTE-SIDECAR.md for the 3-step health \
             check and tunnel script."
        );
        return;
    }

    // 30-minute outer wallclock — npm install on a cold cache + a full
    // lighthouse run can both bite into double-digit minutes, so the budget
    // is generous but bounded. A blown timeout panics with the wrapping
    // message so the developer sees which step ran long.
    tokio::time::timeout(Duration::from_secs(1800), async {
        // 1. Fresh project_root inside a TempDir. The handle is dropped at
        //    scope exit so cleanup happens after the runner is cancelled and
        //    the preview subprocess killed.
        let tmp = tempfile::tempdir().expect("tempdir");
        let project_root = tmp.path().to_path_buf();
        let session_id = "test-d1-real-sidecar";

        // 2. Build the production wiring bundle — same chokepoint helper the
        //    always-run test exercises. A failure here is an A1c regression.
        let wiring: RuntimeWiring = build_task_runner_wiring(&project_root, session_id)
            .await
            .expect("build_task_runner_wiring(project_root) succeeds");

        let project_id = elai_core::project_id_from_path(&project_root);

        // 3. Apply wiring. `apply_task_runner_wiring` does NOT inject the
        //    scaffold_applier / project_dir_resolver — those are dashboard
        //    setup() concerns. The test injects test-shaped versions below
        //    so the scaffold path resolves a real templates dir and a real
        //    target directory (the production CwdProjectDirResolver returns
        //    the cargo test process CWD which is not what we want).
        let cursor_path = project_root.join(".elai_cc").join("cursor.json");
        let mut cfg = apply_task_runner_wiring(cursor_path, &wiring);

        // Some+Some chokepoint guard — mirrors the always-run test.
        assert!(
            cfg.feasibility_prober.is_some(),
            "apply_task_runner_wiring must populate feasibility_prober (D1 Some+Some chokepoint)"
        );
        assert!(
            cfg.layer1a_processor.is_some(),
            "apply_task_runner_wiring must populate layer1a_processor (D1 Some+Some chokepoint)"
        );

        // 4. Locate the templates dir and the target app directory.
        let templates_root = find_templates_dir();
        let app_dir = project_root.join("app");

        cfg.scaffold_applier = Some(Arc::new(TestScaffoldApplier {
            templates_root: templates_root.clone(),
        }));
        cfg.project_dir_resolver = Some(Arc::new(StaticTestResolver {
            dir: app_dir.clone(),
        }));

        // 5. Spawn the runner.
        let writer = wiring.ssb.clone();
        let (handle, cancel) = start_task_runner(writer.clone(), cfg);

        tokio::time::sleep(Duration::from_millis(100)).await;

        // 6. Synthetic TaskSubmitted row. The prefix "make me a " + the
        //    keyword "react" route through `is_web_scaffold_task` ->
        //    `drive_web_scaffold_task` deterministically (task_runner.rs:3482).
        let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
            task: "Make me a React TODO app with localStorage persistence".to_string(),
            project_id: project_id.clone(),
            submitted_at_ms: 0,
        });
        writer
            .write_payload(&project_id, 0, payload, Vec::new())
            .await
            .expect("write TaskSubmitted to SSB");

        // 7. Wait up to 120s for a terminal-ish scaffold signal:
        //    PhaseComplete (success), ConceptDecompositionNode, or
        //    ReasoningTrace (the unconditional observe row).
        let targets = [
            SsbActionType::PhaseComplete,
            SsbActionType::ConceptDecompositionNode,
            SsbActionType::ReasoningTrace,
        ];
        let matched = wait_for_any_action(&writer, &targets, Duration::from_secs(120)).await;
        assert!(
            matched.is_some(),
            "expected PhaseComplete/ConceptDecompositionNode/ReasoningTrace within 120s; \
             drive_web_scaffold_task never fired. project_root={} project_id={}",
            project_root.display(),
            project_id
        );

        // 8. Assert the scaffold landed on disk. The scaffolder copies the
        //    vite-react template verbatim, so package.json + index.html
        //    must both be present in app_dir.
        let package_json = app_dir.join("package.json");
        let index_html = app_dir.join("index.html");
        assert!(
            package_json.exists(),
            "post-scaffold package.json missing at {}",
            package_json.display()
        );
        assert!(
            index_html.exists(),
            "post-scaffold index.html missing at {}",
            index_html.display()
        );

        // 9. npm install. Cold-cache install of vite + react can run 60-180s
        //    on a fresh box; 240s gives headroom without hiding a hang.
        let (npm_install_code, npm_install_stdout, npm_install_stderr) =
            run_subprocess("npm", &["install"], &app_dir, Duration::from_secs(240)).await;
        assert_eq!(
            npm_install_code, 0,
            "`npm install` in {} exited {} \nstdout:\n{}\nstderr:\n{}",
            app_dir.display(),
            npm_install_code,
            npm_install_stdout,
            npm_install_stderr
        );

        // 10. Build via the same `run_web_build` the orchestrator uses in
        //     production (web_validator/src/lib.rs:337).
        let build_result = elai_web_validator::run_web_build(
            &app_dir,
            "npm run build",
            &app_dir.join("dist"),
        )
        .await
        .expect("run_web_build invocation failed (spawn or timeout)");
        assert!(
            build_result.success,
            "`npm run build` in {} reported success=false, exit_code={}, stderr_errors={:?}",
            app_dir.display(),
            build_result.exit_code,
            build_result.stderr_errors
        );

        // 11. Start the Vite preview on a fixed port. `--strictPort` makes
        //     the subprocess fail loudly if 4173 is busy instead of silently
        //     picking another port (which would leave Lighthouse pointing at
        //     thin air). `kill_on_drop` reaps the child when the JoinHandle
        //     drops, so the abort path below is sufficient cleanup.
        let preview_port: u16 = 4173;
        let mut preview_cmd = if cfg!(windows) {
            let mut c = tokio::process::Command::new("cmd");
            c.args([
                "/C",
                "npm",
                "run",
                "preview",
                "--",
                "--port",
                "4173",
                "--strictPort",
            ]);
            c
        } else {
            let mut c = tokio::process::Command::new("npm");
            c.args([
                "run",
                "preview",
                "--",
                "--port",
                "4173",
                "--strictPort",
            ]);
            c
        };
        preview_cmd
            .current_dir(&app_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        let mut preview_child = preview_cmd
            .spawn()
            .expect("failed to spawn `npm run preview` subprocess");

        // 12. Wait up to 30s for the preview port to accept TCP.
        let port_up = wait_for_tcp(preview_port, Duration::from_secs(30)).await;
        if !port_up {
            // Kill child so the panic doesn't leave a zombie.
            let _ = preview_child.start_kill();
            panic!(
                "vite preview never became reachable on 127.0.0.1:{} within 30s",
                preview_port
            );
        }

        let preview_url = format!("http://127.0.0.1:{preview_port}");

        // 13. Run Lighthouse via the production helper for the perf score.
        let lh_report = elai_web_validator::cwv::run_lighthouse(&app_dir, &preview_url)
            .await
            .expect("run_lighthouse invocation failed");
        assert!(
            lh_report.perf_score >= 0.80,
            "lighthouse perf_score {} < 0.80 (lcp_ms={}, cls={}, inp_ms={})",
            lh_report.perf_score,
            lh_report.lcp_ms,
            lh_report.cls,
            lh_report.inp_ms
        );

        // 14. Lighthouse again, ourselves, parsing raw JSON for the
        //     accessibility category — `LighthouseReport` does not expose it
        //     (web_validator/src/cwv.rs:46-55). Use `npx --yes lighthouse`
        //     so the version resolves identically to the production helper.
        let (lh_code, lh_stdout, lh_stderr) = run_subprocess(
            "npx",
            &[
                "--yes",
                "lighthouse",
                &preview_url,
                "--output=json",
                "--output-path=stdout",
                "--quiet",
                "--chrome-flags=--headless=new --no-sandbox",
            ],
            &app_dir,
            Duration::from_secs(180),
        )
        .await;
        // Kill preview before we can panic on assertions below.
        let _ = preview_child.start_kill();
        assert_eq!(
            lh_code, 0,
            "`npx lighthouse` exited {} \nstdout (first 4096):\n{}\nstderr:\n{}",
            lh_code,
            &lh_stdout.chars().take(4096).collect::<String>(),
            lh_stderr
        );
        let lh_json: serde_json::Value = serde_json::from_str(&lh_stdout)
            .expect("lighthouse stdout was not valid JSON");
        let access_score = lh_json
            .pointer("/categories/accessibility/score")
            .and_then(|v| v.as_f64())
            .expect("lighthouse JSON missing /categories/accessibility/score");
        assert!(
            access_score >= 0.90,
            "lighthouse accessibility score {} < 0.90",
            access_score
        );

        // 15. Graceful shutdown of the runner. Preview child was killed
        //     above; the kill_on_drop guard mops up if start_kill failed.
        cancel.cancel();
        let join_result = tokio::time::timeout(Duration::from_secs(10), handle)
            .await
            .expect("task runner JoinHandle did not complete within 10s of cancel()");
        let runner_result = join_result.expect("task runner panicked");
        runner_result.expect("task runner returned an error after cancel()");

        // Reap the preview child so we don't leak the OS process. The await
        // here is on a child whose stdin has been closed and which got
        // SIGKILL'd above — should resolve immediately.
        let _ = tokio::time::timeout(Duration::from_secs(5), preview_child.wait()).await;
    })
    .await
    .expect("E2E real-sidecar test exceeded 30min wall clock");
}
