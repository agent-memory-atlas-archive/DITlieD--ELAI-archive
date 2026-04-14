#![allow(unused_imports, dead_code)]
// Compile-time interface assertions for AragOrchestrator, ContextTracker, and RetrievalTool.
// Type-level checks — if the API surface changes, this file fails to compile.

use std::error::Error;

use elai_knowledge::arag::{
    AragConfig, AragOrchestrator, ContextTracker, RetrievalResult, RetrievalTool,
};
use elai_knowledge::pagerank::DkgSnapshot;

// Verify AragOrchestrator::new signature: (AragConfig) -> Self
fn _assert_orch_new_signature() {
    let _: fn(AragConfig) -> AragOrchestrator = AragOrchestrator::new;
}

// Verify execute_tool signature: (&self, RetrievalTool, &mut ContextTracker) -> Result<Vec<RetrievalResult>, ...>
fn _assert_execute_tool_signature(orch: &AragOrchestrator, tracker: &mut ContextTracker) {
    let tool = RetrievalTool::KeywordSearch {
        keywords: vec![],
        top_k: 1,
    };
    let _: Result<Vec<RetrievalResult>, Box<dyn Error>> =
        orch.execute_tool(tool, tracker).map_err(|e| Box::new(e) as Box<dyn Error>);
}

// Verify query_pipeline signature: (&self, &str, &DkgSnapshot) -> Result<Vec<RetrievalResult>, ...>
fn _assert_query_pipeline_signature(orch: &AragOrchestrator) {
    let snapshot = DkgSnapshot {
        nodes: vec![],
        edges: vec![],
    };
    let _: Result<Vec<RetrievalResult>, Box<dyn Error>> =
        orch.query_pipeline("q", &snapshot).map_err(|e| Box::new(e) as Box<dyn Error>);
}

// Verify ContextTracker methods
fn _assert_tracker_methods() {
    let _: fn() -> ContextTracker = ContextTracker::new;
    let tracker = ContextTracker::new();
    let _: bool = tracker.is_read("x");
}

// Verify RetrievalResult fields are public
fn _assert_result_fields(r: RetrievalResult) {
    let _: String = r.chunk_id;
    let _: String = r.snippet;
    let _: f64 = r.score;
    let _: String = r.tool_used;
}

// Verify AragConfig fields and Default
fn _assert_config_fields() {
    let config = AragConfig::default();
    let _: usize = config.max_loops;
    let _: usize = config.max_token_budget;
    let _: usize = config.top_k;
}
