use elai_knowledge::arag::{
    AragConfig, AragOrchestrator, ContextTracker, RetrievalTool,
};
use elai_knowledge::pagerank::{DkgSnapshot, PageRankNodeInfo};
use elai_knowledge::SymbolKind;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_node(id: i64, name: &str) -> PageRankNodeInfo {
    PageRankNodeInfo {
        node_id: id,
        symbol_name: name.to_string(),
        file_path: PathBuf::from(format!("src/{name}.rs")),
        signature: format!("fn {name}()"),
        kind: SymbolKind::Function,
        ref_count: 1,
    }
}

fn small_snapshot() -> DkgSnapshot {
    DkgSnapshot {
        nodes: vec![make_node(1, "foo"), make_node(2, "bar")],
        edges: vec![(0, 1, 1.0)],
    }
}

// ---------------------------------------------------------------------------
// (f) execute_tool with KeywordSearch returns Ok (graceful degrade, no bm25)
// ---------------------------------------------------------------------------

#[test]
fn f_execute_tool_keyword_search_returns_ok() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::KeywordSearch {
        keywords: vec!["async".to_string(), "tokio".to_string()],
        top_k: 5,
    };
    let results = orch.execute_tool(tool, &mut tracker).unwrap();
    // No bm25 index on AragOrchestrator — graceful degrade to empty
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// (g) execute_tool with SemanticSearch returns Ok (stub, no vector index)
// ---------------------------------------------------------------------------

#[test]
fn g_execute_tool_semantic_search_returns_ok() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::SemanticSearch {
        query: "error handling patterns".to_string(),
        top_k: 10,
    };
    let results = orch.execute_tool(tool, &mut tracker).unwrap();
    // No vector index available — graceful degrade to empty
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// (h) execute_tool with ChunkRead returns results and marks tracker
// ---------------------------------------------------------------------------

#[test]
fn h_execute_tool_chunk_read_returns_results() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::ChunkRead {
        chunk_ids: vec!["chunk_001".to_string(), "chunk_002".to_string()],
    };
    let results = orch.execute_tool(tool, &mut tracker).unwrap();
    assert_eq!(results.len(), 2);
    assert!(tracker.is_read("chunk_001"));
    assert!(tracker.is_read("chunk_002"));
    for r in &results {
        assert_eq!(r.tool_used, "chunk_read");
        assert!(!r.chunk_id.is_empty());
        assert!(!r.snippet.is_empty());
    }
}

// ---------------------------------------------------------------------------
// (i) ContextTracker::token_count returns correct estimate
// ---------------------------------------------------------------------------

#[test]
fn i_context_tracker_token_count_returns_estimate() {
    let mut tracker = ContextTracker::new();
    tracker.mark_read("chunk_a".to_string());
    tracker.mark_read("chunk_b".to_string());
    let count = tracker.token_count();
    // 2 chunks * 256 tokens per chunk = 512
    assert_eq!(count, 512);
}

// ---------------------------------------------------------------------------
// (j) query_pipeline returns results
// ---------------------------------------------------------------------------

#[test]
fn j_query_pipeline_returns_results() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let snapshot = small_snapshot();
    let results = orch.query_pipeline("find all handlers", &snapshot).unwrap();
    // With no bm25/vector index, keyword+semantic return empty,
    // so query_pipeline returns empty (no chunks to read)
    assert!(results.is_empty());
}
