use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTierResult,
};
use elai_ipc_types::{CanvasBindNodeRequest, CanvasDeclareEdgeRequest};
use elai_orchestrator::{
    production_agent_rpc::translate_tool_request_for_test,
    ssb::{Ssb, SsbConfig},
    SsbViewer,
};
use tempfile::TempDir;

fn make_test_ssb(dir: &TempDir) -> Ssb {
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let config = SsbConfig {
        db_path: dir.path().join("contract_j31.db"),
        pool_size: Some(2),
        project_id: "j31-contract".into(),
        session_id: "s-j31-contract".into(),
    };
    Ssb::new(config, clock).expect("ssb init")
}

struct NoOpDispatcher;

impl ToolDispatcher for NoOpDispatcher {
    fn dispatch_edit(
        &self,
        _req: EditRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>>
    {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
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
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>>
    {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

async fn wait_for_canvas<F>(db_path: &std::path::Path, predicate: F)
where
    F: Fn(&elai_orchestrator::system_canvas::SystemCanvas) -> bool,
{
    for _ in 0..40 {
        let viewer = SsbViewer::open(db_path).expect("viewer");
        let canvas = elai_orchestrator::system_canvas::get_system_canvas(&viewer, 64)
            .expect("canvas");
        if predicate(&canvas) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let viewer = SsbViewer::open(db_path).expect("viewer");
    let canvas = elai_orchestrator::system_canvas::get_system_canvas(&viewer, 64)
        .expect("canvas");
    panic!("timed out waiting for system canvas event; canvas={canvas:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bind_node_row_lands_and_rederives() {
    let dir = TempDir::new().expect("tempdir");
    let sink = Arc::new(make_test_ssb(&dir));
    let dispatcher = NoOpDispatcher;
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let bind = CanvasBindNodeRequest {
        node_id: "producer".into(),
        binding_kind: "contract".into(),
        binding_ref: "p".into(),
        display: "Producer".into(),
        col: 2,
        row: 3,
        mode: "system".into(),
    };
    translate_tool_request_for_test(
        elai_ipc_types::TOOL_CANVAS_BIND_NODE,
        &postcard::to_allocvec(&bind).expect("encode bind"),
        &dispatcher,
        Some(&sink),
        None,
        &clock,
    )
    .await
    .expect("bind");

    wait_for_canvas(&dir.path().join("contract_j31.db"), |canvas| {
        canvas.nodes.iter().any(|node| {
            node.node_id == "producer" && node.display == "Producer"
        })
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn declare_edge_row_lands_and_rederives() {
    let dir = TempDir::new().expect("tempdir");
    let sink = Arc::new(make_test_ssb(&dir));
    let dispatcher = NoOpDispatcher;
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let edge = CanvasDeclareEdgeRequest {
        producer_node: "producer".into(),
        consumer_node: "consumer".into(),
        produces: "x".into(),
        consumes: "x".into(),
        reachable_from: "p".into(),
        observable_effect: "e".into(),
        mode: "system".into(),
    };
    translate_tool_request_for_test(
        elai_ipc_types::TOOL_CANVAS_DECLARE_EDGE,
        &postcard::to_allocvec(&edge).expect("encode edge"),
        &dispatcher,
        Some(&sink),
        None,
        &clock,
    )
    .await
    .expect("edge");

    wait_for_canvas(&dir.path().join("contract_j31.db"), |canvas| {
        canvas.edges.iter().any(|item| {
            item.producer_node == "producer" && item.consumer_node == "consumer"
        })
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bind_node_design_mode_is_rejected() {
    let dir = TempDir::new().expect("tempdir");
    let sink = Arc::new(make_test_ssb(&dir));
    let dispatcher = NoOpDispatcher;
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let bind = CanvasBindNodeRequest {
        node_id: "design-node".into(),
        binding_kind: "contract".into(),
        binding_ref: "p".into(),
        display: "Design".into(),
        col: 0,
        row: 0,
        mode: "design".into(),
    };
    let result = translate_tool_request_for_test(
        elai_ipc_types::TOOL_CANVAS_BIND_NODE,
        &postcard::to_allocvec(&bind).expect("encode bind"),
        &dispatcher,
        Some(&sink),
        None,
        &clock,
    )
    .await;
    assert!(result.is_err(), "design-mode bind must be rejected");
}
