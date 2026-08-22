use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_core::clock::OsClock;
use elai_core::tool_dispatcher::{
    CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView, IncrementalOutcome,
    IncrementalRequest, ToolDispatchError, ToolDispatcher, ValidateRequest, ValidationTierResult,
};

use super::*;
use crate::ssb::SsbConfig;

fn ssb_for_test(tmp: &tempfile::TempDir, session: &str) -> Arc<Ssb> {
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "dashboard".into(),
        session_id: session.into(),
    };
    Arc::new(Ssb::new(cfg, clock).expect("ssb"))
}

struct RecordingDispatcher {
    requests: Arc<Mutex<Vec<GitCommitRequest>>>,
}

impl ToolDispatcher for RecordingDispatcher {
    fn dispatch_edit(
        &self,
        _req: EditRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("test".into())) })
    }

    fn dispatch_validate(
        &self,
        _req: ValidateRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("test".into())) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Option<elai_core::tool_dispatcher::ClassifiedFailure>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { None })
    }

    fn dispatch_git_commit(
        &self,
        req: GitCommitRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>,
    > {
        let requests = Arc::clone(&self.requests);
        Box::pin(async move {
            let files_count = u32::try_from(req.files.len()).unwrap_or(u32::MAX);
            let message = req.message.clone();
            requests
                .lock()
                .expect("recording dispatcher mutex")
                .push(req);
            Ok(CommitOutcome {
                sha: "e44-test-sha".into(),
                branch: "e44-test-branch".into(),
                files_count,
                message,
            })
        })
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("test".into())) })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("test".into())) })
    }
}

fn compensation_seq(rows: &[crate::ssb::SsbEntry]) -> Option<i64> {
    rows.iter()
        .find_map(|row| match crate::ssb::decode_ssb_payload(&row.payload) {
            Ok(crate::ssb::SsbPayload::DurabilityEvent(payload))
                if payload.kind == crate::ssb::DurabilityEventKind::CompensationLogged =>
            {
                Some(row.seq)
            }
            _ => None,
        })
}

fn git_commit_seq(rows: &[crate::ssb::SsbEntry]) -> Option<i64> {
    rows.iter().find_map(|row| {
        matches!(
            crate::ssb::decode_ssb_payload(&row.payload),
            Ok(crate::ssb::SsbPayload::GitCommit(_))
        )
        .then_some(row.seq)
    })
}

fn assert_single_dispatch(requests: &Arc<Mutex<Vec<GitCommitRequest>>>) {
    assert_eq!(
        requests.lock().expect("recording dispatcher mutex").len(),
        1,
        "git commit dispatcher must be called once"
    );
}

fn assert_compensation_precedes_commit(rows: &[crate::ssb::SsbEntry]) {
    let compensation = compensation_seq(rows).expect("CompensationLogged row must exist");
    let git_commit = git_commit_seq(rows).expect("GitCommit row must exist");
    assert!(
        compensation < git_commit,
        "CompensationLogged seq {compensation} must precede GitCommit seq {git_commit}"
    );
}

struct E44GateGuard {
    saga_previous: Option<bool>,
    durability_previous: Option<bool>,
}

impl E44GateGuard {
    fn enabled() -> Self {
        Self {
            saga_previous: super::set_e44_saga_enabled_for_test(Some(true)),
            durability_previous:
                crate::durability::observability::set_durability_observability_enabled_for_test(
                    Some(true),
                ),
        }
    }
}

impl Drop for E44GateGuard {
    fn drop(&mut self) {
        super::set_e44_saga_enabled_for_test(self.saga_previous.take());
        crate::durability::observability::set_durability_observability_enabled_for_test(
            self.durability_previous.take(),
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn e44_git_commit_saga_logs_compensation_before_git_commit() {
    let _gate_guard = E44GateGuard::enabled();
    let tmp = tempfile::tempdir().expect("tempdir");
    let writer = ssb_for_test(&tmp, "e44-git-commit-ordering");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let dispatcher = RecordingDispatcher {
        requests: Arc::clone(&requests),
    };
    let changed_files = [PathBuf::from("src/example.rs")];

    run_git_commit_dispatch(
        &writer,
        &dispatcher,
        tmp.path(),
        &changed_files,
        "e44 saga ordering",
        GitBranchPolicy::Main,
        "e44-test-plan",
        "e44-test-step",
    )
    .await;

    tokio::task::yield_now().await;
    let rows = writer
        .query_session_stream_asc(writer.session_id())
        .expect("query drained SSB stream");
    assert_single_dispatch(&requests);
    assert_compensation_precedes_commit(&rows);
}
