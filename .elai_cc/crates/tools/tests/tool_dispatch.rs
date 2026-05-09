// Phase 4B: Tool dispatch integration tests
//
// Tests cover: IpcToolRegistry dispatch, FSM state filtering, postcard roundtrip,
// all 8 tool systems + DKG query tools + failure taxonomy.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use elai_ipc_types::*;
use elai_tools::ipc_registry::{IpcToolRegistry, ToolError, ToolHandler};
use elai_tools::failure_taxonomy::{
    ClassifyHandler, DiagnosticResult, FailureCategory, RootCauseKind,
};
use elai_tools::dkg_tools::{DkgBackend, DkgCallersHandler, DkgDependentsHandler};

// ---------------------------------------------------------------------------
// Test DKG backend (canned responses)
// ---------------------------------------------------------------------------

struct TestDkgBackend;

impl DkgBackend for TestDkgBackend {
    fn query_callers(&self, symbol: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let sym = symbol.to_string();
        Box::pin(async move {
            Ok(vec![format!("main.rs:10: calls {sym}"), format!("lib.rs:20: calls {sym}")])
        })
    }

    fn query_dependents(&self, symbol: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let sym = symbol.to_string();
        Box::pin(async move {
            Ok(vec![format!("{sym} depends on: utils::helper")])
        })
    }

    fn query_data_flow(&self, variable: &str, direction: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let var = variable.to_string();
        let dir = direction.to_string();
        Box::pin(async move {
            Ok(vec![format!("{var} flows {dir}: step1 -> step2")])
        })
    }

    fn query_subgraph(&self, entry_point: &str, depth: u32) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let ep = entry_point.to_string();
        Box::pin(async move {
            Ok(vec![format!("{ep} (depth={depth}): node_a -> node_b")])
        })
    }
}

// ---------------------------------------------------------------------------
// Postcard roundtrip tests (ipc-types typed arg structs)
// ---------------------------------------------------------------------------

#[test]
fn postcard_roundtrip_git_commit_request() {
    let req = GitCommitRequest {
        message: "test commit".to_string(),
        paths: vec!["src/main.rs".to_string()],
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let decoded: GitCommitRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(req, decoded);
}

#[test]
fn postcard_roundtrip_git_undo_request() {
    let req = GitUndoRequest {
        commit_hash: "abc123".to_string(),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let decoded: GitUndoRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(req, decoded);
}

#[test]
fn postcard_roundtrip_validate_request() {
    let req = ValidateRequest {
        file_path: "src/lib.rs".to_string(),
        language: "rust".to_string(),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let decoded: ValidateRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(req, decoded);
}

#[test]
fn postcard_roundtrip_file_read_request() {
    let req = FileReadRequest {
        file_path: "README.md".to_string(),
        offset: Some(10),
        limit: Some(50),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let decoded: FileReadRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(req, decoded);
}

#[test]
fn postcard_roundtrip_compile_check_request() {
    let req = CompileCheckRequest {
        file_path: "src/main.rs".to_string(),
        language: "rust".to_string(),
        retry_round: 1,
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let decoded: CompileCheckRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(req, decoded);
}

#[test]
fn postcard_roundtrip_dkg_callers_request() {
    let req = DkgCallersRequest {
        symbol: "my_function".to_string(),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let decoded: DkgCallersRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(req, decoded);
}

#[test]
fn postcard_roundtrip_classify_request() {
    let req = ClassifyRequest {
        error_output: "error[E0433]: cannot find".to_string(),
        file_path: Some("src/lib.rs".to_string()),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let decoded: ClassifyRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(req, decoded);
}

#[test]
fn postcard_roundtrip_tool_result_payload() {
    let payload = ToolResultPayload {
        output: "success".to_string(),
        diagnostics: vec!["info: ok".to_string()],
    };
    let bytes = postcard::to_allocvec(&payload).unwrap();
    let decoded: ToolResultPayload = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(payload, decoded);
}

// ---------------------------------------------------------------------------
// IpcToolRegistry dispatch tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_dispatch_unknown_tool_returns_not_found() {
    let registry = IpcToolRegistry::new();
    let result = registry.dispatch("nonexistent_tool", &[], FsmState::Executing).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ToolError::NotFound { tool_id } => assert_eq!(tool_id, "nonexistent_tool"),
        other => panic!("expected NotFound, got: {other:?}"),
    }
}

#[tokio::test]
async fn registry_blocks_write_tools_in_planning() {
    let mut registry = IpcToolRegistry::new();

    // Create a dummy write tool handler for testing
    struct DummyWriteTool;
    impl ToolHandler for DummyWriteTool {
        fn execute<'a>(&'a self, _arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
            Box::pin(async { Ok(ToolResultPayload { output: "ok".to_string(), diagnostics: Vec::new() }) })
        }
        fn tool_id(&self) -> &str { "test_write" }
        fn is_write_tool(&self) -> bool { true }
    }

    registry.register(Arc::new(DummyWriteTool));

    let result = registry.dispatch("test_write", &[], FsmState::Planning).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ToolError::NotAllowedInState { tool_id, state } => {
            assert_eq!(tool_id, "test_write");
            assert_eq!(state, FsmState::Planning);
        }
        other => panic!("expected NotAllowedInState, got: {other:?}"),
    }
}

#[tokio::test]
async fn registry_allows_write_tools_in_executing() {
    let mut registry = IpcToolRegistry::new();

    struct DummyWriteTool;
    impl ToolHandler for DummyWriteTool {
        fn execute<'a>(&'a self, _arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
            Box::pin(async { Ok(ToolResultPayload { output: "ok".to_string(), diagnostics: Vec::new() }) })
        }
        fn tool_id(&self) -> &str { "test_write" }
        fn is_write_tool(&self) -> bool { true }
    }

    registry.register(Arc::new(DummyWriteTool));

    let result = registry.dispatch("test_write", &[], FsmState::Executing).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn registry_blocks_compile_check_in_testing_first() {
    let mut registry = IpcToolRegistry::new();

    struct FakeCompileCheck;
    impl ToolHandler for FakeCompileCheck {
        fn execute<'a>(&'a self, _arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
            Box::pin(async { Ok(ToolResultPayload { output: "ok".to_string(), diagnostics: Vec::new() }) })
        }
        fn tool_id(&self) -> &str { TOOL_COMPILE_CHECK }
        fn is_write_tool(&self) -> bool { false }
    }

    registry.register(Arc::new(FakeCompileCheck));

    let result = registry.dispatch(TOOL_COMPILE_CHECK, &[], FsmState::TestingFirst).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ToolError::NotAllowedInState { tool_id, state } => {
            assert_eq!(tool_id, TOOL_COMPILE_CHECK);
            assert_eq!(state, FsmState::TestingFirst);
        }
        other => panic!("expected NotAllowedInState, got: {other:?}"),
    }
}

#[test]
fn registry_available_tools_filters_write_in_planning() {
    let mut registry = IpcToolRegistry::new();

    struct DummyRead;
    impl ToolHandler for DummyRead {
        fn execute<'a>(&'a self, _arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
            Box::pin(async { Ok(ToolResultPayload { output: "ok".to_string(), diagnostics: Vec::new() }) })
        }
        fn tool_id(&self) -> &str { "read_tool" }
        fn is_write_tool(&self) -> bool { false }
    }

    struct DummyWrite;
    impl ToolHandler for DummyWrite {
        fn execute<'a>(&'a self, _arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
            Box::pin(async { Ok(ToolResultPayload { output: "ok".to_string(), diagnostics: Vec::new() }) })
        }
        fn tool_id(&self) -> &str { "write_tool" }
        fn is_write_tool(&self) -> bool { true }
    }

    registry.register(Arc::new(DummyRead));
    registry.register(Arc::new(DummyWrite));

    let planning_tools = registry.available_tools(FsmState::Planning);
    let tool_ids: Vec<&str> = planning_tools.iter().map(|t| t.as_str()).collect();
    assert!(tool_ids.contains(&"read_tool"));
    assert!(!tool_ids.contains(&"write_tool"));

    let exec_tools = registry.available_tools(FsmState::Executing);
    let tool_ids: Vec<&str> = exec_tools.iter().map(|t| t.as_str()).collect();
    assert!(tool_ids.contains(&"read_tool"));
    assert!(tool_ids.contains(&"write_tool"));
}

// ---------------------------------------------------------------------------
// Failure taxonomy tests (RA-1)
// ---------------------------------------------------------------------------

#[test]
fn classify_runtime_panic_from_panicked_at() {
    // B-20A 9-variant taxonomy: `LazyCode` was dropped; a `panicked at`
    // signal now classifies as `RuntimePanic`.
    let handler = ClassifyHandler::new();
    let req = ClassifyRequest {
        error_output: "thread 'main' panicked at 'oops' in src/main.rs:10".to_string(),
        file_path: None,
    };
    let bytes = postcard::to_allocvec(&req).unwrap();

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let result = rt.block_on(handler.execute(&bytes)).unwrap();
    let diag: DiagnosticResult = serde_json::from_str(&result.output).unwrap();
    assert_eq!(diag.category, FailureCategory::RuntimePanic);
    assert!(
        diag.fix_instruction.contains("panic")
            || diag.fix_instruction.contains("runtime")
    );
}

#[test]
fn classify_missing_import_with_root_cause() {
    // B-20A 9-variant taxonomy: `CompilationError` was split. An
    // `unresolved import` signal now classifies as `MissingImport`.
    let handler = ClassifyHandler::new();
    let req = ClassifyRequest {
        error_output:
            "error[E0432]: unresolved import `foo::bar`\n  --> src/lib.rs:5"
                .to_string(),
        file_path: Some("src/lib.rs".to_string()),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let result = rt.block_on(handler.execute(&bytes)).unwrap();
    let diag: DiagnosticResult = serde_json::from_str(&result.output).unwrap();
    assert_eq!(diag.category, FailureCategory::MissingImport);
    assert!(diag.root_cause.is_some());
    assert_eq!(diag.root_cause.as_ref().unwrap().kind, RootCauseKind::MissingImport);
    assert!(!diag.escalate); // has root cause -> high confidence -> no escalate
}

// ---------------------------------------------------------------------------
// DKG tools tests (RA-3)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dkg_callers_returns_results() {
    let handler = DkgCallersHandler::new(Box::new(TestDkgBackend));
    let req = DkgCallersRequest {
        symbol: "process_data".to_string(),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let result = handler.execute(&bytes).await.unwrap();
    assert!(result.output.contains("main.rs:10"));
    assert!(result.output.contains("lib.rs:20"));
    assert_eq!(result.diagnostics[0], "2 callers found");
}

#[tokio::test]
async fn dkg_dependents_returns_results() {
    let handler = DkgDependentsHandler::new(Box::new(TestDkgBackend));
    let req = DkgDependentsRequest {
        symbol: "my_func".to_string(),
    };
    let bytes = postcard::to_allocvec(&req).unwrap();
    let result = handler.execute(&bytes).await.unwrap();
    assert!(result.output.contains("depends on"));
    assert_eq!(result.diagnostics[0], "1 dependents found");
}

// ---------------------------------------------------------------------------
// Deserialization error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bad_postcard_bytes_returns_deserialization_error() {
    let handler = ClassifyHandler::new();
    let bad_bytes = &[0xFF, 0xFE, 0xFD];
    let result = handler.execute(bad_bytes).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ToolError::DeserializationFailed { tool_id, .. } => {
            assert_eq!(tool_id, TOOL_CLASSIFY);
        }
        other => panic!("expected DeserializationFailed, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Well-known tool ID constants test
// ---------------------------------------------------------------------------

#[test]
fn tool_id_constants_are_unique() {
    let ids = [
        TOOL_GIT_COMMIT, TOOL_GIT_UNDO, TOOL_GIT_BRANCH, TOOL_VALIDATE,
        TOOL_TRACE, TOOL_EDIT, TOOL_SIMPLIFY, TOOL_CLASSIFY, TOOL_FILE_READ,
        TOOL_COMPILE_CHECK, TOOL_DKG_CALLERS, TOOL_DKG_DEPENDENTS,
        TOOL_DKG_DATA_FLOW, TOOL_DKG_SUBGRAPH, TOOL_INTEGRATION_CHECK,
    ];
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(seen.insert(id), "duplicate tool ID: {id}");
    }
    assert_eq!(ids.len(), 15);
}
