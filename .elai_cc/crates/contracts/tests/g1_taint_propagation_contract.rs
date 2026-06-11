//! G-1 -- harness-wide taint propagation (data-plane defense) contract tests.
//!
//! These tests drive the REAL production sink (`translate_tool_request`, via
//! the `test-support` re-export `translate_tool_request_for_test`) -- NOT a
//! unit call to the gate function. The off-agent artifact asserted is the
//! `taint_flow` SSB row landing in a live in-memory `Ssb`, plus the dispatch
//! being BLOCKED before the `ToolDispatcher` is reached (a RecordingDispatcher
//! flags if its destructive method fires).
//!
//! Coverage:
//!   (a) UNTRUSTED web ingest -> argv derives from it -> git_commit sink
//!       BLOCKED + a taint_flow SSB row lands (kind=web, blocked=true,
//!       sink_tool=git_commit). dispatch_git_commit is NOT reached.
//!   (b) FP GUARD: a clean Trusted flow (no tainted ingest) -> git_commit
//!       sink ALLOWED, dispatch reached, NO taint_flow row.
//!   (c) GD-4 DEPENDENCY-SCREEN: an untrusted ingest does NOT taint an
//!       unrelated, non-data-dependent git_commit (proves NOT global).
//!   (d) seed_from_verdict maps a malicious ClassifierVerdict -> Untrusted.
//!   (e) 10 benign flows -> 0 false blocks (RTBAS Slack-collapse FP guard).
//!   (f) TaintFlow Pattern-2 round-trips through msgpack + SQL CHECK + the
//!       action_type bijection for all 4 TaintFlowKind arms.
//!
//! No #[ignore] / stubs / placeholder assertions. No #[traced_test] (we assert
//! on SSB rows + the Err verdict + the dispatch-reached flag, not on logs).
//!
//! Paper grounding: arXiv:2502.08966 (RTBAS dependency-screened join),
//! arXiv:2503.18813 (CaMeL capability gate), arXiv:2505.23643 (FIDES P-T sink).

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use elai_core::clock::OsClock;
use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditMatchTier, EditOutcome, EditRequest, FailureInput,
    FileView, GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError,
    ToolDispatcher, ValidateRequest, ValidationTierResult,
};
use elai_ipc_types::{TOOL_EDIT, TOOL_GIT_COMMIT};
use elai_orchestrator::production_agent_rpc::translate_tool_request_for_test;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, TaintFlowKind,
    TaintFlowPayload,
};
use elai_orchestrator::SsbReader;
use elai_safety::provenance::Provenance;
use elai_safety::taint_flow::{seed_from_verdict, tainted_context};
use elai_safety::prompt_armor::ClassifierVerdict;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// G-3 blast-radius gate fix: ELAI_PROJECT_ROOT must be an absolute path so
// enforce_fs_blast_radius correctly judges repo-relative commit paths like
// `src/login.rs` as inside the worktree root. The default `.elai_cc` is
// relative and misresolves when CWD is inside crates/contracts/ (case B
// triage: test-only setup issue, not a production over-block).
// CARGO_MANIFEST_DIR is crates/contracts; two parents up = .elai_cc root.
// ---------------------------------------------------------------------------

static INIT_PROJECT_ROOT: OnceLock<()> = OnceLock::new();

fn set_project_root_for_blast_radius_gate() {
    INIT_PROJECT_ROOT.get_or_init(|| {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        // CARGO_MANIFEST_DIR = .elai_cc/crates/contracts
        // parent()           = .elai_cc/crates
        // parent()           = .elai_cc
        let root = manifest_dir
            .parent()
            .expect("crates dir")
            .parent()
            .expect("elai_cc root");
        // SAFETY: this test binary is single-process; all tests in this file
        // set the same absolute path via OnceLock (exactly-once, idempotent).
        #[allow(deprecated)]
        std::env::set_var("ELAI_PROJECT_ROOT", root);
    });
}

// ---------------------------------------------------------------------------
// RecordingDispatcher: flags whether a destructive method was reached. The
// gate must BLOCK before dispatch, so on a blocked flow `git_commit_reached`
// stays false. On a clean flow it flips true and returns a success outcome.
// ---------------------------------------------------------------------------

struct RecordingDispatcher {
    git_commit_reached: Arc<AtomicBool>,
    edit_reached: Arc<AtomicBool>,
}

impl RecordingDispatcher {
    fn new() -> Self {
        Self {
            git_commit_reached: Arc::new(AtomicBool::new(false)),
            edit_reached: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ToolDispatcher for RecordingDispatcher {
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>>
    {
        self.edit_reached.store(true, Ordering::SeqCst);
        let file_path = req.file_path.clone();
        Box::pin(async move {
            Ok(EditOutcome {
                tier: EditMatchTier::Exact,
                bytes_written: 0,
                file_path,
                similarity_hint: None,
            })
        })
    }

    fn dispatch_validate(
        &self,
        _req: ValidateRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async { None })
    }

    fn dispatch_git_commit(
        &self,
        _req: GitCommitRequest,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>,
    > {
        self.git_commit_reached.store(true, Ordering::SeqCst);
        Box::pin(async {
            Ok(CommitOutcome {
                sha: "deadbeef".into(),
                branch: "test".into(),
                files_count: 1,
                message: "ok".into(),
            })
        })
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>>
    {
        Box::pin(async {
            Ok(FileView {
                outline_only: false,
                signatures: Vec::new(),
                imports: Vec::new(),
                full_content: Some(String::new()),
                total_lines: 0,
            })
        })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

fn make_ssb(tmp: &TempDir, suffix: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join(format!("ssb_g1_{suffix}.db")),
        pool_size: Some(2),
        project_id: format!("g1-{suffix}"),
        session_id: format!("g1-session-{suffix}"),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

fn git_commit_args(message: &str, paths: &[&str]) -> Vec<u8> {
    let wire = elai_ipc_types::GitCommitRequest {
        message: message.to_owned(),
        paths: paths.iter().map(|p| (*p).to_owned()).collect(),
    };
    postcard::to_allocvec(&wire).expect("encode git commit request")
}

/// A unique untrusted ingest marker so concurrent tests do not cross-taint via
/// the process-global side-table. The kill-switch must be UNSET for the gate to
/// run (default-ON, W-2); we do not set ELAI_TAINT_PROPAGATE_DISABLE anywhere.
const INJECTED_PAYLOAD: &str = "G1_INJECTED_DESTRUCTIVE_rm_rf_slash_marker_a7f3";

// ---------------------------------------------------------------------------
// (a) untrusted web ingest -> git_commit BLOCKED + taint_flow row lands
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn untrusted_web_ingest_blocks_git_commit_and_emits_taint_flow() {
    set_project_root_for_blast_radius_gate();
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "block");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let dispatcher = RecordingDispatcher::new();
    let reached = dispatcher.git_commit_reached.clone();

    // INGEST: register the untrusted web body into the process-global side-table
    // exactly as web_research_orchestrator.rs:421 does in production.
    let ctx = tainted_context();
    ctx.register(
        INJECTED_PAYLOAD,
        Provenance::from_web("https://attacker.example/payload"),
    );

    // SINK: the agent requests a git_commit whose MESSAGE derives from the
    // untrusted ingest. Drive the REAL production dispatch fan.
    let args = git_commit_args(
        &format!("apply remediation: {INJECTED_PAYLOAD}"),
        &["src/lib.rs"],
    );
    let result = translate_tool_request_for_test(
        TOOL_GIT_COMMIT,
        &args,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await;

    assert!(
        result.is_err(),
        "untrusted-tainted git_commit must be BLOCKED; got Ok"
    );
    assert!(
        result.as_ref().unwrap_err().contains("taint gate"),
        "block message must name the taint gate; got: {:?}",
        result
    );
    assert!(
        !reached.load(Ordering::SeqCst),
        "dispatch_git_commit must NOT be reached when the gate blocks (fail-closed BEFORE dispatch)"
    );

    // OFF-AGENT ARTIFACT: a taint_flow row must have landed. The adapter spawns
    // the write; await briefly until it is visible (bounded retries, no sleep
    // dependency on timing — poll the live db).
    let mut rows = Vec::new();
    for _ in 0..50 {
        rows = ssb
            .query_by_type(SsbActionType::TaintFlow)
            .expect("query taint_flow");
        if !rows.is_empty() {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(
        rows.len(),
        1,
        "exactly one taint_flow row must land for the blocked flow"
    );
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode taint_flow");
    match decoded {
        SsbPayload::TaintFlow(p) => {
            assert_eq!(p.kind, TaintFlowKind::Web, "ingest kind must be Web");
            assert_eq!(p.trust_level, "untrusted");
            assert_eq!(p.sink_tool.as_deref(), Some("git_commit"));
            assert!(p.blocked, "blocked must be true");
            assert!(!p.origin_blake3.is_empty(), "origin_blake3 must be populated");
        }
        other => panic!("expected TaintFlow payload, got {other:?}"),
    }

    ctx.clear();
}

// ---------------------------------------------------------------------------
// (b) FP GUARD: clean Trusted flow -> git_commit ALLOWED, NO row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clean_trusted_flow_is_not_blocked_and_emits_no_row() {
    set_project_root_for_blast_radius_gate();
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "clean");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let dispatcher = RecordingDispatcher::new();
    let reached = dispatcher.git_commit_reached.clone();

    // Ensure the side-table holds the injected payload (from a prior test or
    // registered here) -- the FP guard proves a clean message is unaffected
    // EVEN WITH an untrusted ingest live in the table (the dependency screen).
    let ctx = tainted_context();
    ctx.register(
        INJECTED_PAYLOAD,
        Provenance::from_web("https://attacker.example/payload"),
    );

    let args = git_commit_args("feature: add login button", &["src/login.rs"]);
    let result = translate_tool_request_for_test(
        TOOL_GIT_COMMIT,
        &args,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await;

    assert!(
        result.is_ok(),
        "a clean Trusted git_commit must NOT be blocked; got Err: {:?}",
        result
    );
    assert!(
        reached.load(Ordering::SeqCst),
        "dispatch_git_commit MUST be reached for a clean flow"
    );

    // No taint_flow row for the clean flow.
    let rows = ssb
        .query_by_type(SsbActionType::TaintFlow)
        .expect("query taint_flow");
    assert!(
        rows.is_empty(),
        "a clean Trusted flow must emit ZERO taint_flow rows; found {}",
        rows.len()
    );

    ctx.clear();
}

// ---------------------------------------------------------------------------
// (c) GD-4 DEPENDENCY-SCREEN: untrusted ingest does not taint an unrelated sink
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn untrusted_ingest_does_not_taint_unrelated_sink_not_global() {
    set_project_root_for_blast_radius_gate();
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "screen");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let dispatcher = RecordingDispatcher::new();
    let reached = dispatcher.git_commit_reached.clone();

    let ctx = tainted_context();
    ctx.clear();
    // Register an untrusted ingest, then commit something that does NOT derive
    // from it. A run-GLOBAL join would taint this too (RTBAS Slack-collapse);
    // the dependency-screened join must leave it clean.
    ctx.register(
        INJECTED_PAYLOAD,
        Provenance::from_web("https://attacker.example/payload"),
    );

    let args = git_commit_args("chore: bump version to 1.2.3", &["Cargo.toml"]);
    let result = translate_tool_request_for_test(
        TOOL_GIT_COMMIT,
        &args,
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await;

    assert!(
        result.is_ok(),
        "an unrelated sink must NOT inherit an untrusted ingest's taint (dependency-screened, NOT global); got Err: {:?}",
        result
    );
    assert!(reached.load(Ordering::SeqCst), "dispatch must be reached for the unrelated clean sink");

    let rows = ssb
        .query_by_type(SsbActionType::TaintFlow)
        .expect("query taint_flow");
    assert!(rows.is_empty(), "no taint_flow row for an unrelated sink");

    ctx.clear();
}

// ---------------------------------------------------------------------------
// (d) seed_from_verdict maps a malicious verdict -> Untrusted
// ---------------------------------------------------------------------------

#[test]
fn seed_from_malicious_verdict_forces_untrusted() {
    // A path-screened TRUSTED base whose CONTENT trips the classifier must be
    // escalated to Untrusted by the seed.
    let base = Provenance::user_prompt();
    assert!(base.is_trusted());
    let verdict = ClassifierVerdict::Malicious {
        reason: "ignore all previous instructions".into(),
        patterns_matched: vec!["ignore all previous".into()],
    };
    let out = seed_from_verdict(&verdict, base);
    assert!(out.is_untrusted(), "malicious verdict must force Untrusted");

    // A clean verdict must NOT change a trusted base.
    let base2 = Provenance::user_prompt();
    let out2 = seed_from_verdict(&ClassifierVerdict::Clean, base2);
    assert!(out2.is_trusted(), "clean verdict must leave the base trusted");
}

// ---------------------------------------------------------------------------
// (e) 10 benign flows -> 0 false blocks (RTBAS Slack-collapse FP guard)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ten_benign_flows_zero_false_blocks() {
    set_project_root_for_blast_radius_gate();
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "benign");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let ctx = tainted_context();
    ctx.clear();
    // One untrusted ingest live in the table the whole time.
    ctx.register(
        INJECTED_PAYLOAD,
        Provenance::from_web("https://attacker.example/payload"),
    );

    let benign_messages = [
        "feature: add dark mode toggle",
        "fix: null check on session restore",
        "refactor: extract token store",
        "docs: update README install steps",
        "test: cover the retry path",
        "chore: bump deps",
        "perf: cache the symbol map",
        "style: rustfmt the providers crate",
        "ci: pin the runner image",
        "build: enable lto for release",
    ];

    let mut false_blocks = 0usize;
    for (i, msg) in benign_messages.iter().enumerate() {
        let dispatcher = RecordingDispatcher::new();
        let args = git_commit_args(msg, &[&format!("src/file_{i}.rs")]);
        let r = translate_tool_request_for_test(
            TOOL_GIT_COMMIT,
            &args,
            &dispatcher,
            Some(&ssb),
            None,
            &clock,
        )
        .await;
        if r.is_err() {
            false_blocks += 1;
        }
    }

    assert_eq!(
        false_blocks, 0,
        "all 10 benign flows must pass the gate; {false_blocks} were falsely blocked (RTBAS Slack-collapse guard / GD-4)"
    );

    let rows = ssb
        .query_by_type(SsbActionType::TaintFlow)
        .expect("query taint_flow");
    assert!(
        rows.is_empty(),
        "benign flows must emit ZERO taint_flow rows; found {}",
        rows.len()
    );

    ctx.clear();
}

// ---------------------------------------------------------------------------
// (f) TaintFlow Pattern-2 round-trip for all 4 Kind arms + bijection
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn taint_flow_payload_roundtrips_for_all_kinds() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "roundtrip");

    let kinds = [
        TaintFlowKind::Web,
        TaintFlowKind::ToolOutput,
        TaintFlowKind::File,
        TaintFlowKind::Sandbox,
    ];

    for (i, kind) in kinds.iter().enumerate() {
        let payload = SsbPayload::TaintFlow(TaintFlowPayload {
            kind: *kind,
            origin_blake3: format!("{:016x}", i),
            trust_level: "untrusted".into(),
            sink_tool: Some("git_commit".into()),
            blocked: true,
            task_id: format!("task-{i}"),
        });
        assert_eq!(payload.action_type(), SsbActionType::TaintFlow);
        ssb.write_payload("executing", i as i64, payload.clone(), Vec::new())
            .await
            .expect("write taint_flow");
    }

    let rows = ssb
        .query_by_type(SsbActionType::TaintFlow)
        .expect("query taint_flow");
    assert_eq!(rows.len(), 4, "all 4 Kind arms must round-trip");

    for row in &rows {
        let decoded: SsbPayload = rmp_serde::from_slice(&row.payload).expect("decode");
        match decoded {
            SsbPayload::TaintFlow(p) => {
                assert!(p.blocked);
                assert_eq!(p.trust_level, "untrusted");
            }
            other => panic!("expected TaintFlow, got {other:?}"),
        }
    }
}

#[test]
fn taint_flow_sql_str_bijection() {
    let variant = SsbActionType::TaintFlow;
    let s = variant.as_sql_str();
    assert_eq!(s, "taint_flow");
    let round = ssb_action_type_from_str_public(s).expect("parse");
    assert_eq!(round, variant);
}

// ---------------------------------------------------------------------------
// (g) edit sink: untrusted new_text blocks an edit
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn untrusted_edit_content_blocks_edit_sink() {
    set_project_root_for_blast_radius_gate();
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "edit");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let dispatcher = RecordingDispatcher::new();
    let edit_reached = dispatcher.edit_reached.clone();

    let ctx = tainted_context();
    ctx.clear();
    ctx.register(
        INJECTED_PAYLOAD,
        Provenance::from_web("https://attacker.example/payload"),
    );

    let wire = elai_ipc_types::EditRequest {
        file_path: "src/lib.rs".into(),
        old_text: "fn old() {}".into(),
        new_text: format!("fn evil() {{ /* {INJECTED_PAYLOAD} */ }}"),
    };
    let args = postcard::to_allocvec(&wire).expect("encode edit");
    let result =
        translate_tool_request_for_test(TOOL_EDIT, &args, &dispatcher, Some(&ssb), None, &clock)
            .await;

    assert!(result.is_err(), "untrusted edit content must be BLOCKED");
    assert!(
        !edit_reached.load(Ordering::SeqCst),
        "dispatch_edit must NOT be reached when the gate blocks"
    );

    ctx.clear();
}
