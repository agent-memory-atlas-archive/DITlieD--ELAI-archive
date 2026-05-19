//! C-16 Test File 3 — CosmosStage wiring in QueryPipeline.
//!
//! Verifies the cosmos pipeline stage:
//! - appears in STANDARD/DEEP tier query results when configured
//! - is omitted for FAST tier queries
//! - returns empty results when the worker has no refreshed artifacts
//! - returns non-empty results after tick() writes a body
//! - does not abort the pipeline when cosmos_get fails (results empty)
//!
//! Plan reference: docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md
//! file 3 — cosmos_pipeline_stage.rs.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::cosmos_worker::{
    CosmosClock, CosmosError, CosmosManifest, CosmosWorker, ResearchDispatcher,
};
use elai_knowledge::pipeline::QueryPipeline;
use elai_knowledge::{CosmosStage, QueryTier};

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

struct FixedClock(std::sync::atomic::AtomicI64);
impl CosmosClock for FixedClock {
    fn now_secs(&self) -> i64 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

struct CannedResearch(String);
impl ResearchDispatcher for CannedResearch {
    fn run<'a>(
        &'a self,
        _q: &'a str,
        _t: &'a str,
        _m: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        let body = self.0.clone();
        Box::pin(async move { Ok((body, "T1".to_string())) })
    }
}

fn build_manifest_for_query(query_tokens: &[&str]) -> CosmosManifest {
    // Build an artifact whose `research_query` contains all tokens — the
    // CosmosStage name-router is permissive (any token overlap), so this
    // guarantees a match.
    let research_query = query_tokens.join(" ");
    let toml = format!(
        r#"
[[artifacts]]
name = "test_artifact"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "daily"
research_query = "{research_query}"
"#,
    );
    CosmosManifest::from_toml_str(&toml).expect("manifest")
}

fn build_pipeline_with_cosmos(
    body: &str,
    query_tokens: &[&str],
) -> (tempfile::TempDir, QueryPipeline, Arc<CosmosWorker>) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bm25 = Bm25Index::new(tmp.path(), "cosmos_stage_test").expect("bm25");
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());

    let cosmos_root = tmp.path().join("cosmos");
    std::fs::create_dir_all(&cosmos_root).expect("mkdir cosmos_root");
    let manifest = build_manifest_for_query(query_tokens);
    let clock: Arc<dyn CosmosClock> =
        Arc::new(FixedClock(std::sync::atomic::AtomicI64::new(1_700_000_000)));
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch(body.to_string()));
    let worker = Arc::new(
        CosmosWorker::new(cosmos_root, manifest, clock, research).expect("worker"),
    );
    let stage = CosmosStage::new(worker.clone());
    let pipeline = QueryPipeline::new(bm25, catrag, arag).with_cosmos_stage(stage);
    (tmp, pipeline, worker)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn cosmos_stage_appears_in_standard_tier() {
    let (_tmp, pipeline, _worker) =
        build_pipeline_with_cosmos("# alpha body", &["alpha", "test"]);

    let stages = pipeline
        .query_with_tier("alpha", QueryTier::Standard)
        .expect("query");

    let stage_names: Vec<&str> = stages.iter().map(|s| s.stage.as_str()).collect();
    assert!(
        stage_names.iter().any(|s| *s == "cosmos"),
        "expected cosmos stage in STANDARD tier, got: {:?}",
        stage_names
    );
}

#[test]
fn cosmos_stage_appears_in_deep_tier() {
    let (_tmp, pipeline, _worker) =
        build_pipeline_with_cosmos("# alpha body", &["alpha", "test"]);

    let stages = pipeline
        .query_with_tier("alpha", QueryTier::Deep)
        .expect("query");

    let stage_names: Vec<&str> = stages.iter().map(|s| s.stage.as_str()).collect();
    assert!(
        stage_names.iter().any(|s| *s == "cosmos"),
        "expected cosmos stage in DEEP tier, got: {:?}",
        stage_names
    );
}

#[test]
fn cosmos_stage_omitted_in_fast_tier() {
    let (_tmp, pipeline, _worker) =
        build_pipeline_with_cosmos("# alpha body", &["alpha", "test"]);

    let stages = pipeline
        .query_with_tier("alpha", QueryTier::Fast)
        .expect("query");

    let stage_names: Vec<&str> = stages.iter().map(|s| s.stage.as_str()).collect();
    assert!(
        !stage_names.iter().any(|s| *s == "cosmos"),
        "cosmos stage MUST NOT appear in FAST tier, got: {:?}",
        stage_names
    );
}

#[test]
fn cosmos_stage_results_empty_when_worker_unrefreshed() {
    // Pipeline built but tick() not called: no body on disk, cosmos_get
    // returns MissingBody for each artifact, the stage logs+skips, the
    // results list is empty (but the stage still appears).
    let (_tmp, pipeline, _worker) =
        build_pipeline_with_cosmos("# body", &["unique", "marker"]);
    let stages = pipeline
        .query_with_tier("unique marker", QueryTier::Standard)
        .expect("query");
    let cosmos_stage = stages
        .iter()
        .find(|s| s.stage == "cosmos")
        .expect("cosmos stage present");
    assert!(
        cosmos_stage.results.is_empty(),
        "expected empty results pre-tick, got {} rows",
        cosmos_stage.results.len()
    );
    // No error propagation — the pipeline must continue (error: None).
    assert!(cosmos_stage.error.is_none());
}

#[tokio::test]
async fn cosmos_stage_results_populated_after_tick() {
    let (_tmp, pipeline, worker) =
        build_pipeline_with_cosmos("# alpha body content", &["alpha", "test"]);
    let report = worker.tick().await;
    assert_eq!(report.refreshed.len(), 1, "tick refreshed: {:?}", report);

    let stages = pipeline
        .query_with_tier("alpha", QueryTier::Standard)
        .expect("query");
    let cosmos_stage = stages
        .iter()
        .find(|s| s.stage == "cosmos")
        .expect("cosmos stage present");
    assert!(
        !cosmos_stage.results.is_empty(),
        "expected cosmos results after tick"
    );
    // Each result should be tagged tool_used="cosmos".
    for r in &cosmos_stage.results {
        assert_eq!(r.tool_used, "cosmos");
        assert!(r.snippet.contains("alpha body content"));
    }
}

#[tokio::test]
async fn cosmos_stage_does_not_abort_pipeline_on_missing_body() {
    // Tick writes the body; we then DELETE the body file under the worker's
    // feet to simulate a torn-disk / MissingBody scenario at query time.
    // The stage must log+skip the failing artifact without surfacing an
    // error to the pipeline (the pipeline continues).
    let (tmp, pipeline, worker) =
        build_pipeline_with_cosmos("# transient body", &["transient", "marker"]);
    let report = worker.tick().await;
    assert_eq!(report.refreshed.len(), 1);

    // Locate the body file and delete it.
    let bodies_dir = tmp.path().join("cosmos").join("artifacts");
    for entry in std::fs::read_dir(&bodies_dir).expect("read_dir") {
        let entry = entry.expect("entry");
        let p = entry.path();
        // Delete .md file but keep .meta.json so cosmos_get returns
        // MissingBody (meta present, body absent).
        if p.extension().and_then(|s| s.to_str()) == Some("md") {
            std::fs::remove_file(&p).expect("remove body");
        }
    }

    let stages = pipeline
        .query_with_tier("transient marker", QueryTier::Standard)
        .expect("query continues despite MissingBody");
    let cosmos_stage = stages
        .iter()
        .find(|s| s.stage == "cosmos")
        .expect("cosmos stage present");
    assert!(
        cosmos_stage.results.is_empty(),
        "missing body yields empty results"
    );
    // Pipeline did NOT abort — error: None on the stage record.
    assert!(cosmos_stage.error.is_none(), "stage error stays None");
}
