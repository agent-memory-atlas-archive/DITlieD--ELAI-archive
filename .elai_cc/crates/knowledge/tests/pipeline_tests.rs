use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::pipeline::QueryPipeline;

// ---------------------------------------------------------------------------
// (k) QueryPipeline::query returns stages in correct order
// ---------------------------------------------------------------------------

#[test]
fn k_query_pipeline_query_returns_nine_stages() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = Bm25Index::new(tmp.path(), "test_k").unwrap();
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    let pipeline = QueryPipeline::new(bm25, catrag, arag);
    let stages = pipeline.query("find all imports").unwrap();
    let names: Vec<&str> = stages.iter().map(|s| s.stage.as_str()).collect();
    // Phase 3K.3 added hyde stage between query_expansion and bm25 (10 stages without splade).
    assert_eq!(
        names,
        vec!["query_expansion", "hyde", "bm25", "vector", "pagerank", "catrag", "arag", "rerank", "cross_encoder", "token_fit"]
    );
}

// ---------------------------------------------------------------------------
// (l) QueryPipeline::query stages have latency_ms field (caller-measured)
// ---------------------------------------------------------------------------

#[test]
fn l_query_pipeline_stages_have_latency() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = Bm25Index::new(tmp.path(), "test_l").unwrap();
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    let pipeline = QueryPipeline::new(bm25, catrag, arag);
    let stages = pipeline.query("error handling").unwrap();
    assert_eq!(stages.len(), 10, "expected 10 stages (query_expansion + hyde + bm25..token_fit)");
    // With an empty index, individual stages may complete in sub-millisecond time.
    // Verify all stages have the latency_ms field populated (>= 0).
    // With real data, bm25 stage will exceed 0ms.
    for stage in &stages {
        assert!(stage.latency_ms < 60_000, "stage {} latency sanity check", stage.stage);
    }
}

// ---------------------------------------------------------------------------
// (m) QueryPipeline returns Vec<PipelineStage> with consistent results
// ---------------------------------------------------------------------------

#[test]
fn m_query_pipeline_returns_stages() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = Bm25Index::new(tmp.path(), "test_m").unwrap();
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    let pipeline = QueryPipeline::new(bm25, catrag, arag);
    let stages = pipeline.query("graph traversal").unwrap();
    assert_eq!(stages.len(), 10, "expected 10 stages (query_expansion + hyde + bm25..token_fit)");
    // With empty index, bm25 returns empty results, cascading empty through all stages
    for stage in &stages {
        assert!(!stage.stage.is_empty());
    }
}
