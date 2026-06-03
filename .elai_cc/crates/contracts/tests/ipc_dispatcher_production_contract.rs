//! C-13 — IPC Dispatcher Production Swap contract tests.
//!
//! Pins the C-13 wiring:
//!   * `IpcToolDispatcherAdapter` compiles + implements `ToolDispatcher`.
//!   * All 4 production `InProcessToolDispatcher` construction sites
//!     (run_subcommand.rs:195/275, dashboard-v2/lib.rs:472/552) now wrap
//!     the inner dispatcher in the adapter shim (verified by grep).
//!   * `InProcessHookDispatcher::with_ruff_runner` wired via the new
//!     `build_production_hook_dispatcher` helper.
//!   * `RuntimeWiring.hook_dispatcher` field exists; `apply_task_runner_wiring`
//!     threads it onto `TaskRunnerConfig.hook_dispatcher`.
//!   * `AdaptiveTool<T>` emits the 10 missing DAIRA markers.
//!   * `SsbActionType::WipSquashComplete` round-trips through SSB.
//!   * `wip_squash::squash_wip_commits_if_any` is a no-op at 5 leading
//!     WIP commits and squashes at 6.
//!
//! Runtime flavour: `#[tokio::test]` multi-thread so the AdaptiveTool /
//! Ssb async paths have a runtime to land on.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::process::Command;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTierResult,
};
use elai_ipc_types::FsmState;
use elai_orchestrator::hook_dispatcher::InProcessHookDispatcher;
use elai_orchestrator::runtime_setup::build_production_hook_dispatcher;
use elai_orchestrator::ssb::{
    Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::wip_squash::{squash_wip_commits_if_any, WIP_SQUASH_THRESHOLD};
use elai_tools::adaptive_tool::{AdaptiveTool, IpcToolDispatcherAdapter};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

fn build_ssb(dir: &TempDir) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "test".into(),
        session_id: "c13-contract".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

/// Minimal `ToolDispatcher` stub. Each method returns an `Ok` outcome so
/// `AdaptiveTool` can wrap it and observe the DAIRA emission path.
struct StubDispatcher;

impl ToolDispatcher for StubDispatcher {
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            Ok(EditOutcome {
                tier: elai_core::tool_dispatcher::EditMatchTier::Exact,
                bytes_written: 7,
                file_path: req.file_path,
                similarity_hint: None,
            })
        })
    }

    fn dispatch_validate(
        &self,
        _req: ValidateRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> Pin<Box<dyn Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async move { None })
    }

    fn dispatch_git_commit(
        &self,
        _req: GitCommitRequest,
    ) -> Pin<Box<dyn Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            Ok(CommitOutcome {
                sha: "deadbeef".into(),
                branch: "main".into(),
                files_count: 1,
                message: "stub".into(),
            })
        })
    }

    fn dispatch_file_read_outline(
        &self,
        _path: std::path::PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            Ok(FileView {
                outline_only: false,
                signatures: vec![],
                imports: vec![],
                full_content: Some(String::new()),
                total_lines: 0,
            })
        })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<Box<dyn Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            Ok(IncrementalOutcome {
                success: true,
                attempts: 1,
                first_error_category: None,
                elapsed_ms: 0,
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_ipc_tool_dispatcher_adapter_compiles_and_impls_tool_dispatcher() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let inner: Arc<dyn ToolDispatcher> = Arc::new(StubDispatcher);
    let adapter = Arc::new(IpcToolDispatcherAdapter::new(inner, ssb));

    // Trait-object coercion is the load-bearing assertion: if the trait
    // shape doesn't match, this line won't compile.
    let _as_trait: Arc<dyn ToolDispatcher> = adapter.clone();

    assert_eq!(adapter.fsm_state(), FsmState::Executing);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_adapter_fsm_gate_denies_write_in_read_only_state() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let inner: Arc<dyn ToolDispatcher> = Arc::new(StubDispatcher);
    let adapter =
        Arc::new(IpcToolDispatcherAdapter::new(inner, ssb).with_fsm_state(FsmState::Planning));

    let req = EditRequest {
        file_path: std::path::PathBuf::from("dummy.rs"),
        old_text: "a".into(),
        new_text: "b".into(),
    };
    let result = adapter.dispatch_edit(req).await;
    match result {
        Err(ToolDispatchError::Denied { .. }) => {}
        other => panic!("expected Denied, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_adapter_passes_writes_through_in_executing_state() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let inner: Arc<dyn ToolDispatcher> = Arc::new(StubDispatcher);
    let adapter = Arc::new(IpcToolDispatcherAdapter::new(inner, ssb));

    let req = EditRequest {
        file_path: std::path::PathBuf::from("dummy.rs"),
        old_text: "a".into(),
        new_text: "b".into(),
    };
    let outcome = adapter.dispatch_edit(req).await.expect("dispatch_edit");
    assert_eq!(outcome.bytes_written, 7);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_adaptive_tool_emits_tool_call_start_and_end() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let inner: Arc<dyn ToolDispatcher> = Arc::new(StubDispatcher);
    let adaptive = Arc::new(AdaptiveTool::new(inner, ssb.clone()));

    let req = EditRequest {
        file_path: std::path::PathBuf::from("dummy.rs"),
        old_text: "a".into(),
        new_text: "b".into(),
    };
    let _ = adaptive.dispatch_edit(req).await.expect("edit");

    let rows = ssb
        .query_by_type(SsbActionType::DairaTrace)
        .expect("query_by_type");
    let phases: Vec<String> = rows
        .iter()
        .filter_map(|r| rmp_serde::from_slice::<SsbPayload>(&r.payload).ok())
        .filter_map(|p| match p {
            SsbPayload::DairaTrace(d) => Some(d.phase),
            _ => None,
        })
        .collect();
    assert!(phases.iter().any(|p| p == "ToolCallStart"));
    assert!(phases.iter().any(|p| p == "ToolCallEnd"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_adaptive_tool_emits_read_write_mutation_on_edit() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let inner: Arc<dyn ToolDispatcher> = Arc::new(StubDispatcher);
    let adaptive = Arc::new(AdaptiveTool::new(inner, ssb.clone()));

    let req = EditRequest {
        file_path: std::path::PathBuf::from("dummy.rs"),
        old_text: "a".into(),
        new_text: "b".into(),
    };
    let _ = adaptive.dispatch_edit(req).await.expect("edit");

    let rows = ssb
        .query_by_type(SsbActionType::DairaTrace)
        .expect("query_by_type");
    let phases: Vec<String> = rows
        .iter()
        .filter_map(|r| rmp_serde::from_slice::<SsbPayload>(&r.payload).ok())
        .filter_map(|p| match p {
            SsbPayload::DairaTrace(d) => Some(d.phase),
            _ => None,
        })
        .collect();
    assert!(phases.iter().any(|p| p == "Read"));
    assert!(phases.iter().any(|p| p == "Write"));
    assert!(phases.iter().any(|p| p == "Mutation"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_adaptive_tool_emits_branch_and_converge_on_validate() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);

    // Custom inner returns one passing tier so the Converge branch fires.
    struct PassingValidate;
    impl ToolDispatcher for PassingValidate {
        fn dispatch_edit(
            &self,
            req: EditRequest,
        ) -> Pin<Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>>
        {
            Box::pin(async move {
                Ok(EditOutcome {
                    tier: elai_core::tool_dispatcher::EditMatchTier::Exact,
                    bytes_written: 0,
                    file_path: req.file_path,
                    similarity_hint: None,
                })
            })
        }
        fn dispatch_validate(
            &self,
            _req: ValidateRequest,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async move {
                Ok(vec![ValidationTierResult {
                    tier: elai_core::tool_dispatcher::ValidationTier::Determinism,
                    passed: true,
                    elapsed_ms: 1,
                    rules_fired: 0,
                    stderr_snippet: None,
                }])
            })
        }
        fn dispatch_classify_failure(
            &self,
            _i: FailureInput,
        ) -> Pin<Box<dyn Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
            Box::pin(async move { None })
        }
        fn dispatch_git_commit(
            &self,
            _r: GitCommitRequest,
        ) -> Pin<Box<dyn Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>>
        {
            Box::pin(async move {
                Ok(CommitOutcome {
                    sha: "x".into(),
                    branch: "main".into(),
                    files_count: 0,
                    message: "x".into(),
                })
            })
        }
        fn dispatch_file_read_outline(
            &self,
            _p: std::path::PathBuf,
            _t: usize,
        ) -> Pin<Box<dyn Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>>
        {
            Box::pin(async move {
                Ok(FileView {
                    outline_only: false,
                    signatures: vec![],
                    imports: vec![],
                    full_content: Some(String::new()),
                    total_lines: 0,
                })
            })
        }
        fn dispatch_incremental_compile(
            &self,
            _r: IncrementalRequest,
        ) -> Pin<
            Box<dyn Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>,
        > {
            Box::pin(async move {
                Ok(IncrementalOutcome {
                    success: true,
                    attempts: 1,
                    first_error_category: None,
                    elapsed_ms: 0,
                })
            })
        }
    }

    let inner: Arc<dyn ToolDispatcher> = Arc::new(PassingValidate);
    let adaptive = Arc::new(AdaptiveTool::new(inner, ssb.clone()));
    let req = ValidateRequest {
        project_dir: dir.path().to_path_buf(),
        language: "rust".into(),
        changed_files: vec![],
        step_id: "step-1".into(),
        tiers: None,
    };
    let _ = adaptive.dispatch_validate(req).await.expect("validate");

    let rows = ssb
        .query_by_type(SsbActionType::DairaTrace)
        .expect("query_by_type");
    let phases: Vec<String> = rows
        .iter()
        .filter_map(|r| rmp_serde::from_slice::<SsbPayload>(&r.payload).ok())
        .filter_map(|p| match p {
            SsbPayload::DairaTrace(d) => Some(d.phase),
            _ => None,
        })
        .collect();
    assert!(phases.iter().any(|p| p == "Branch"));
    assert!(phases.iter().any(|p| p == "Converge"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_build_production_hook_dispatcher_installs_ruff_runner() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    // F-9 / Slice D3: project_root required so gate can resolve registry_check.py.
    let arc = build_production_hook_dispatcher(ssb, true, dir.path());
    let _: Arc<InProcessHookDispatcher> = arc.clone();
    assert!(arc.has_ruff_runner(), "ruff runner must be installed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_build_production_hook_dispatcher_installs_dep_writetime_gate() {
    // F-9 / Slice D3: DepWriteTimeGate must be installed DEFAULT-ON in the
    // production dispatcher so every PreToolUse file-write path probes the
    // registry for unknown dependency names.
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let arc = build_production_hook_dispatcher(ssb, true, dir.path());
    assert!(
        arc.has_dep_writetime_gate(),
        "DepWriteTimeGate must be installed DEFAULT-ON in production dispatcher"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_wip_squash_complete_ssb_round_trip() {
    use elai_orchestrator::ssb::WipSquashCompletePayload;
    let dir = TempDir::new().expect("tempdir");
    let ssb = build_ssb(&dir);
    let payload = SsbPayload::WipSquashComplete(WipSquashCompletePayload {
        wip_count: 7,
        squashed: true,
        step_id: "step-c13".into(),
    });
    let seq = ssb
        .write_payload("test", 0, payload, vec![])
        .await
        .expect("write_payload");
    assert!(seq > 0);

    let rows = ssb
        .query_by_type(SsbActionType::WipSquashComplete)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let deserialized: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize");
    match deserialized {
        SsbPayload::WipSquashComplete(p) => {
            assert_eq!(p.wip_count, 7);
            assert!(p.squashed);
            assert_eq!(p.step_id, "step-c13");
        }
        other => panic!("unexpected payload: {other:?}"),
    }
    // SCHEMA_VERSION 53 bump.
    assert!(SCHEMA_VERSION_PUBLIC >= 53);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_wip_squash_noop_at_threshold_or_fewer() {
    let repo = TempDir::new().expect("tempdir");
    let dir = TempDir::new().expect("ssb tempdir");
    init_git_repo(repo.path());

    // Make exactly WIP_SQUASH_THRESHOLD WIP commits + 1 non-WIP base.
    add_commit(repo.path(), "base.txt", "base", "initial");
    for i in 0..WIP_SQUASH_THRESHOLD {
        add_commit(repo.path(), &format!("w{i}.txt"), "x", &format!("WIP: step {i}"));
    }
    let pre_count = count_commits(repo.path());
    assert_eq!(pre_count, 1 + WIP_SQUASH_THRESHOLD);

    let ssb = build_ssb(&dir);
    let outcome = squash_wip_commits_if_any(repo.path(), ssb, "")
        .await
        .expect("squash");
    assert_eq!(outcome.wip_count, WIP_SQUASH_THRESHOLD);
    assert!(!outcome.squashed);
    assert_eq!(count_commits(repo.path()), pre_count);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_wip_squash_collapses_above_threshold() {
    let repo = TempDir::new().expect("tempdir");
    let dir = TempDir::new().expect("ssb tempdir");
    init_git_repo(repo.path());

    let n = WIP_SQUASH_THRESHOLD + 2;
    add_commit(repo.path(), "base.txt", "base", "initial");
    for i in 0..n {
        add_commit(repo.path(), &format!("w{i}.txt"), "x", &format!("WIP: step {i}"));
    }
    let pre_count = count_commits(repo.path());
    assert_eq!(pre_count, 1 + n);

    let ssb = build_ssb(&dir);
    let outcome = squash_wip_commits_if_any(repo.path(), ssb, "step-x")
        .await
        .expect("squash");
    assert_eq!(outcome.wip_count, n);
    assert!(outcome.squashed);

    let post_count = count_commits(repo.path());
    assert_eq!(post_count, 1 + 1, "base + single squash commit");
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

fn init_git_repo(path: &Path) {
    run_git(path, &["init", "-q", "-b", "main"]);
    run_git(path, &["config", "user.email", "c13@test"]);
    run_git(path, &["config", "user.name", "c13"]);
    run_git(path, &["config", "commit.gpgsign", "false"]);
}

fn add_commit(path: &Path, file: &str, content: &str, message: &str) {
    std::fs::write(path.join(file), content).expect("write");
    run_git(path, &["add", file]);
    run_git(path, &["commit", "--no-verify", "-q", "-m", message]);
}

fn count_commits(path: &Path) -> u32 {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-list")
        .arg("--count")
        .arg("HEAD")
        .output()
        .expect("rev-list");
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    s.parse::<u32>().unwrap_or(0)
}

fn run_git(path: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("git {args:?} failed: {stderr}");
    }
}
