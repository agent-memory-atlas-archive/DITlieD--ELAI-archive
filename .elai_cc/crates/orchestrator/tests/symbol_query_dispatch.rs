// C-18 B-29 backfill — translate_tool_request symbol_* dispatch contract.
//
// Builds a real 2-file Rust project in a tempdir, runs DkgEngine +
// build_symbol_graph to populate `.elai_cc/indexes/dkg.db`, then calls
// `translate_tool_request_for_test` (C-18 D1 public test alias) for each of
// the four symbol_* tools. Every call must return Ok, and an
// `SsbActionType::SymbolQuery` row must land in the SSB.
//
// Rationale: the symbol_* dispatch path was specified by B-29 Phase 5 but
// never had an end-to-end test asserting (a) Ok-return for all four tools
// against a real symbol graph, (b) the SSB row is written. Type-level
// shape coverage already lives in
// crates/contracts/tests/symbol_graph_b29_phase5_contract.rs.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTierResult,
};
use elai_ipc_types::{
    SymbolContextRequest, SymbolFlowRequest, SymbolImpactRequest, SymbolListRequest,
    TOOL_SYMBOL_CONTEXT, TOOL_SYMBOL_FLOW, TOOL_SYMBOL_IMPACT, TOOL_SYMBOL_LIST,
};
use elai_orchestrator::production_agent_rpc::translate_tool_request_for_test;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig};
use elai_orchestrator::ssb_viewer::SsbViewer;
use std::future::Future;
use std::pin::Pin;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// NoOp dispatcher — the symbol_* arms do NOT call any ToolDispatcher method
// (they open SymbolGraphCache directly), but the function signature still
// requires `&dyn ToolDispatcher`. A pass-through stub satisfies the type.
// ---------------------------------------------------------------------------

struct NoOpDispatcher;

impl ToolDispatcher for NoOpDispatcher {
    fn dispatch_edit(
        &self,
        _req: EditRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
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
        Box::pin(async { Ok(Vec::new()) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> Pin<Box<dyn Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async { None })
    }

    fn dispatch_git_commit(
        &self,
        _req: GitCommitRequest,
    ) -> Pin<Box<dyn Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>> {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

// ---------------------------------------------------------------------------
// Project fixture: matches the precedent at
// crates/knowledge/tests/symbol_graph_build_pipeline.rs:32.
// ---------------------------------------------------------------------------

/// Project name must be the directory's `file_name()` because the symbol_*
/// arms call `project_root.file_name().and_then(to_str)` to derive project_id.
fn setup_two_file_project(root: &Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create src/");
    std::fs::write(
        src.join("a.rs"),
        "pub fn foo() -> i32 { 42 }\n",
    )
    .expect("write a.rs");
    std::fs::write(
        src.join("b.rs"),
        "use crate::a::foo;\n\npub fn bar() -> i32 { foo() }\n",
    )
    .expect("write b.rs");
}

/// Populate `.elai_cc/indexes/dkg.db` so the symbol_* arms find a real graph.
fn build_full_symbol_graph(project_root: &Path) {
    let db_path = elai_core::paths::dkg_db_path(project_root);
    std::fs::create_dir_all(
        db_path
            .parent()
            .expect("dkg_db_path always has a parent directory"),
    )
    .expect("create .elai_cc/indexes");

    // 1. DKG pass (file-level dependency graph).
    {
        let mut dkg = elai_knowledge::DkgEngine::new(&db_path, None, 0)
            .expect("DkgEngine::new for fixture");
        dkg.build_graph(project_root)
            .expect("dkg.build_graph for fixture");
        // dkg drops here, releasing the WAL write lock.
    }

    // 2. Symbol-graph pass (produces the node/edge tables consumed by the
    //    traversal helpers).
    let project_id = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .expect("project_root must have UTF-8 file_name");
    elai_knowledge::symbol_graph::build_pipeline::build_symbol_graph(
        project_id,
        project_root,
        &db_path,
        1_700_000_000_000,
    )
    .expect("build_symbol_graph for fixture");
}

// ---------------------------------------------------------------------------
// SSB construction (orchestrator test pattern).
// ---------------------------------------------------------------------------

fn make_ssb_and_viewer() -> (TempDir, Arc<Ssb>, SsbViewer, PathBuf) {
    // B-29 WIRING_CONTRACT trigger (cargo test ... symbol_query_dispatch_writes_
    // ssb_row_for_each_tool) runs this. When ELAI_SSB_SMOKE_DB is set the test
    // writes the SymbolQuery rows to that canonical db path so the off-agent
    // wiring_contract_check.py expect_ssb leg can observe them; otherwise a
    // TempDir db is used (unit-test default). Mirrors the B-23 ssb_emission_smoke
    // pattern in self_mod_layer1a.rs:2004.
    let (dir, db_path) = match std::env::var("ELAI_SSB_SMOKE_DB") {
        Ok(path) if !path.is_empty() => {
            let db_path = std::path::PathBuf::from(path);
            if let Some(parent) = db_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            // Reuse the same TempDir handle for lifecycle parity with the
            // default branch; the smoke-db path itself is outside the tempdir.
            let dir = TempDir::new().expect("tempdir creation (smoke-db mode)");
            (dir, db_path)
        }
        _ => {
            let dir = TempDir::new().expect("tempdir creation");
            let db_path = dir.path().join("c18_symbol_query_dispatch.db");
            (dir, db_path)
        }
    };
    let config = SsbConfig {
        db_path: db_path.clone(),
        project_id: "c18-symbol-query".to_owned(),
        session_id: "s-c18-symbol".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(config, clock).expect("Ssb::new"));
    let viewer = SsbViewer::open(&db_path).expect("SsbViewer::open");
    (dir, ssb, viewer, db_path)
}

async fn count_symbol_query_rows(viewer: &SsbViewer, expected_at_least: usize) -> usize {
    let action_str = SsbActionType::SymbolQuery.as_sql_str();
    // The async SSB writer flushes asynchronously — poll for up to ~1s.
    for _ in 0..50u32 {
        if let Ok(rows) = viewer.recent_rows_of_type(action_str, 32) {
            if rows.len() >= expected_at_least {
                return rows.len();
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    viewer
        .recent_rows_of_type(action_str, 32)
        .map(|r| r.len())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Contract: each of the four symbol_* tools dispatches Ok and writes a
// SymbolQuery row to the SSB.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn symbol_query_dispatch_writes_ssb_row_for_each_tool() {
    let project_dir = TempDir::new().expect("project tempdir");
    let project_root = project_dir.path().to_path_buf();
    setup_two_file_project(&project_root);
    build_full_symbol_graph(&project_root);

    let (_ssb_dir, ssb, viewer, _db_path) = make_ssb_and_viewer();
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(NoOpDispatcher);

    let pr_str = project_root
        .to_str()
        .expect("project_root must be UTF-8")
        .to_owned();

    // -- 1. symbol_impact --------------------------------------------------
    let impact_args = postcard::to_allocvec(&SymbolImpactRequest {
        project_root: pr_str.clone(),
        target: "src/a.rs".to_owned(),
        depth: 2,
    })
    .expect("encode SymbolImpactRequest");
    let result = translate_tool_request_for_test(
        TOOL_SYMBOL_IMPACT,
        &impact_args,
        dispatcher.as_ref(),
        Some(&ssb),
        None,
        &clock,
    )
    .await;
    assert!(
        result.is_ok(),
        "TOOL_SYMBOL_IMPACT must return Ok, got: {:?}",
        result.err(),
    );

    // -- 2. symbol_flow ----------------------------------------------------
    let flow_args = postcard::to_allocvec(&SymbolFlowRequest {
        project_root: pr_str.clone(),
        // entrypoint_id is "<file>::<symbol>" by build_pipeline convention.
        entrypoint_id: "src/a.rs::foo".to_owned(),
        depth: 2,
    })
    .expect("encode SymbolFlowRequest");
    let result = translate_tool_request_for_test(
        TOOL_SYMBOL_FLOW,
        &flow_args,
        dispatcher.as_ref(),
        Some(&ssb),
        None,
        &clock,
    )
    .await;
    assert!(
        result.is_ok(),
        "TOOL_SYMBOL_FLOW must return Ok, got: {:?}",
        result.err(),
    );

    // -- 3. symbol_context -------------------------------------------------
    let context_args = postcard::to_allocvec(&SymbolContextRequest {
        project_root: pr_str.clone(),
        name: "foo".to_owned(),
        file_hint: None,
    })
    .expect("encode SymbolContextRequest");
    let result = translate_tool_request_for_test(
        TOOL_SYMBOL_CONTEXT,
        &context_args,
        dispatcher.as_ref(),
        Some(&ssb),
        None,
        &clock,
    )
    .await;
    assert!(
        result.is_ok(),
        "TOOL_SYMBOL_CONTEXT must return Ok, got: {:?}",
        result.err(),
    );

    // -- 4. symbol_list ----------------------------------------------------
    let list_args = postcard::to_allocvec(&SymbolListRequest {
        project_root: pr_str.clone(),
        file: None,
        query: Some("foo".to_owned()),
        limit: 50,
    })
    .expect("encode SymbolListRequest");
    let result = translate_tool_request_for_test(
        TOOL_SYMBOL_LIST,
        &list_args,
        dispatcher.as_ref(),
        Some(&ssb),
        None,
        &clock,
    )
    .await;
    assert!(
        result.is_ok(),
        "TOOL_SYMBOL_LIST must return Ok, got: {:?}",
        result.err(),
    );

    // -- SSB assertion: at least one SymbolQuery row per call (4 total). ---
    let observed = count_symbol_query_rows(&viewer, 4).await;
    assert!(
        observed >= 4,
        "expected >= 4 SymbolQuery SSB rows (one per tool), got {observed}",
    );
}
