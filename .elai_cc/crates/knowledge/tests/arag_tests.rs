use elai_knowledge::arag::{
    AragChunk, AragConfig, AragOrchestrator, ContextTracker, RetrievalTool,
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

/// Build an orchestrator pre-loaded with test chunks for keyword/semantic tests.
fn orch_with_test_chunks() -> AragOrchestrator {
    let mut orch = AragOrchestrator::new(AragConfig::default());
    orch.add_chunk(AragChunk {
        chunk_id: "chunk_alpha".to_string(),
        content: "The async runtime uses tokio for scheduling tasks. Tokio handles async IO efficiently.".to_string(),
        embedding: vec![0.9, 0.1, 0.0, 0.0],
    });
    orch.add_chunk(AragChunk {
        chunk_id: "chunk_beta".to_string(),
        content: "Error handling patterns in Rust use Result and Option types.".to_string(),
        embedding: vec![0.0, 0.1, 0.9, 0.0],
    });
    orch.add_chunk(AragChunk {
        chunk_id: "chunk_gamma".to_string(),
        content: "The tokio runtime provides async task spawning and channel communication.".to_string(),
        embedding: vec![0.8, 0.2, 0.0, 0.0],
    });
    orch
}

// ---------------------------------------------------------------------------
// (f) execute_tool with KeywordSearch returns non-empty results
// ---------------------------------------------------------------------------

#[test]
fn f_execute_tool_keyword_search_returns_ok() {
    let orch = orch_with_test_chunks();
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::KeywordSearch {
        keywords: vec!["async".to_string(), "tokio".to_string()],
        top_k: 5,
    };
    let results = orch.execute_tool(tool, &mut tracker).unwrap();
    assert!(!results.is_empty(), "keyword search must return results when chunks contain keywords");
    for r in &results {
        assert_eq!(r.tool_used, "keyword_search");
        assert!(!r.snippet.is_empty(), "snippet must not be empty");
        assert!(r.score > 0.0, "score must be positive for matching chunks");
    }
}

// ---------------------------------------------------------------------------
// (g) execute_tool with SemanticSearch returns non-empty results
// ---------------------------------------------------------------------------

#[test]
fn g_execute_tool_semantic_search_returns_ok() {
    let orch = orch_with_test_chunks();
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::SemanticSearch {
        query: "error handling patterns".to_string(),
        top_k: 10,
    };
    let results = orch.execute_tool(tool, &mut tracker).unwrap();
    assert!(!results.is_empty(), "semantic search must return results when chunks have embeddings");
    for r in &results {
        assert_eq!(r.tool_used, "semantic_search");
        assert!(!r.snippet.is_empty(), "snippet must not be empty");
    }
}

// ---------------------------------------------------------------------------
// (f2) keyword search returns snippets containing matched sentences, not full chunk
// ---------------------------------------------------------------------------

#[test]
fn f2_keyword_search_returns_sentence_snippets() {
    let mut orch = AragOrchestrator::new(AragConfig::default());
    orch.add_chunk(AragChunk {
        chunk_id: "multi_sentence".to_string(),
        content: "First unrelated sentence. The async runtime is fast. Another unrelated line. Tokio provides concurrency.".to_string(),
        embedding: vec![0.5, 0.5, 0.0, 0.0],
    });
    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::KeywordSearch {
        keywords: vec!["async".to_string()],
        top_k: 5,
    };
    let results = orch.execute_tool(tool, &mut tracker).unwrap();
    assert!(!results.is_empty());
    let snippet = &results[0].snippet;
    // Snippet should contain the matched sentence, not the full chunk
    assert!(
        snippet.contains("async"),
        "snippet must contain the keyword match"
    );
    assert!(
        !snippet.contains("First unrelated sentence"),
        "snippet must not contain unrelated sentences (progressive disclosure)"
    );
    assert!(
        snippet.starts_with("..."),
        "snippet must start with '...' prefix per spec"
    );
}

// ---------------------------------------------------------------------------
// (f3) adversarial: short chunk with keyword hits scores higher than long chunk
//      with same number of hits (length normalization)
// ---------------------------------------------------------------------------

#[test]
fn f3_keyword_length_normalization_short_beats_long() {
    let mut orch = AragOrchestrator::new(AragConfig::default());

    // Short chunk: 10 lines, exactly 3 occurrences of "async"
    let short_content = "fn run_async() {\n    let handle = async { work() };\n    spawn_async(handle);\n}\nfn more() {}\nfn other() {}\nfn last() {}\nfn end() {}\nfn done() {}\nfn fin() {}";
    orch.add_chunk(AragChunk {
        chunk_id: "short_chunk".to_string(),
        content: short_content.to_string(),
        embedding: vec![0.5, 0.5, 0.0, 0.0],
    });

    // Long chunk: 500 lines of filler with exactly 3 occurrences of "async"
    let mut long_lines: Vec<String> = (0..497)
        .map(|i| format!("// filler line {i}"))
        .collect();
    long_lines.insert(100, "fn run_async() { }".to_string());
    long_lines.insert(250, "let handle = async { work() };".to_string());
    long_lines.insert(400, "spawn_async(handle);".to_string());
    let long_content = long_lines.join("\n");

    orch.add_chunk(AragChunk {
        chunk_id: "long_chunk".to_string(),
        content: long_content,
        embedding: vec![0.5, 0.5, 0.0, 0.0],
    });

    let mut tracker = ContextTracker::new();
    let tool = RetrievalTool::KeywordSearch {
        keywords: vec!["async".to_string()],
        top_k: 5,
    };
    let results = orch.execute_tool(tool, &mut tracker).unwrap();
    assert!(results.len() >= 2, "both chunks should match");

    let short_score = results.iter().find(|r| r.chunk_id == "short_chunk").map(|r| r.score).unwrap_or(0.0);
    let long_score = results.iter().find(|r| r.chunk_id == "long_chunk").map(|r| r.score).unwrap_or(0.0);

    assert!(
        short_score > long_score,
        "short chunk (score={short_score}) must score higher than long chunk (score={long_score}) with same keyword hits — length normalization required"
    );
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
    // 2 chunks * 1000 tokens per chunk = 2000 (spec: ~1000 tokens/chunk)
    assert_eq!(count, 2000);
}

// ---------------------------------------------------------------------------
// (j) query_pipeline returns non-empty results when chunks are loaded
// ---------------------------------------------------------------------------

#[test]
fn j_query_pipeline_returns_results() {
    let orch = orch_with_test_chunks();
    let snapshot = small_snapshot();
    let results = orch.query_pipeline("find all handlers", &snapshot).unwrap();
    // With chunks loaded, keyword+semantic should return results
    assert!(!results.is_empty(), "query_pipeline must return results when chunks are loaded");
}

// ---------------------------------------------------------------------------
// (j2) query_pipeline with no chunks returns empty (graceful degrade)
// ---------------------------------------------------------------------------

#[test]
fn j2_query_pipeline_no_chunks_returns_empty() {
    let orch = AragOrchestrator::new(AragConfig::default());
    let snapshot = small_snapshot();
    let results = orch.query_pipeline("find all handlers", &snapshot).unwrap();
    assert!(results.is_empty(), "empty orchestrator must degrade gracefully");
}
