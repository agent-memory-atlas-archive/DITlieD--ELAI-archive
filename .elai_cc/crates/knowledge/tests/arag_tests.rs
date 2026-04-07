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
// (f) execute_tool with KeywordSearch panics (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn f_execute_tool_keyword_search_hits_todo() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::KeywordSearch {
        keywords: vec!["async".to_string(), "tokio".to_string()],
        top_k: 5,
    };
    let _ = orch.execute_tool(tool, &mut tracker);
}

// ---------------------------------------------------------------------------
// (g) execute_tool with SemanticSearch panics (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn g_execute_tool_semantic_search_hits_todo() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::SemanticSearch {
        query: "error handling patterns".to_string(),
        top_k: 10,
    };
    let _ = orch.execute_tool(tool, &mut tracker);
}

// ---------------------------------------------------------------------------
// (h) execute_tool with ChunkRead panics (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn h_execute_tool_chunk_read_hits_todo() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::ChunkRead {
        chunk_ids: vec!["chunk_001".to_string(), "chunk_002".to_string()],
    };
    let _ = orch.execute_tool(tool, &mut tracker);
}

// ---------------------------------------------------------------------------
// (i) ContextTracker::token_count panics (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn i_context_tracker_token_count_hits_todo() {
    let mut tracker = ContextTracker::new();
    tracker.mark_read("chunk_a".to_string());
    tracker.mark_read("chunk_b".to_string());
    let _ = tracker.token_count();
}

// ---------------------------------------------------------------------------
// (j) query_pipeline panics (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn j_query_pipeline_hits_todo() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let snapshot = small_snapshot();
    let _ = orch.query_pipeline("find all handlers", &snapshot);
}
