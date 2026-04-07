use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::pipeline::QueryPipeline;

// ---------------------------------------------------------------------------
// (k) QueryPipeline::query panics (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn k_query_pipeline_query_hits_todo() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = Bm25Index::new(tmp.path(), "test_k").unwrap();
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    let pipeline = QueryPipeline::new(bm25, catrag, arag);
    let _ = pipeline.query("find all imports");
}

// ---------------------------------------------------------------------------
// (l) QueryPipeline::query with different query panics (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn l_query_pipeline_different_query_hits_todo() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = Bm25Index::new(tmp.path(), "test_l").unwrap();
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    let pipeline = QueryPipeline::new(bm25, catrag, arag);
    let _ = pipeline.query("error handling");
}

// ---------------------------------------------------------------------------
// (m) QueryPipeline returns Vec<PipelineStage> (todo!())
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "not yet implemented")]
fn m_query_pipeline_returns_stages() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = Bm25Index::new(tmp.path(), "test_m").unwrap();
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    let pipeline = QueryPipeline::new(bm25, catrag, arag);
    let _ = pipeline.query("graph traversal");
}
