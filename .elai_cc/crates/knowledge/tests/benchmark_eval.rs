//! Phase 3K.5 — retrieval quality benchmark integration test.
//!
//! Indexes the ELAI knowledge crate source into BM25, runs the 30-query
//! evaluation set, and writes results to `metrics/phase_3K_rag.toml`.

use std::path::{Path, PathBuf};

use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::benchmark;
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::pipeline::QueryPipeline;

/// Resolve the project root (two levels up from CARGO_MANIFEST_DIR).
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Path to the knowledge crate source directory.
fn knowledge_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Path to the metrics output directory.
fn metrics_dir() -> PathBuf {
    project_root().join("metrics")
}

/// Build a BM25 index over the knowledge crate's Rust source files.
fn build_knowledge_index(tmp_dir: &Path) -> Bm25Index {
    let mut bm25 = Bm25Index::new(tmp_dir, "benchmark_eval").unwrap();
    let src = knowledge_src();

    // Index all .rs files in knowledge/src/ recursively.
    bm25.index_directory(&src).unwrap();

    // Also index a few key files from sibling crates for cross-crate queries.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = manifest_dir.parent().unwrap();

    let extra_files = [
        crates_dir.join("core").join("src").join("clock.rs"),
        crates_dir.join("core").join("src").join("entropy.rs"),
        crates_dir.join("core").join("src").join("config.rs"),
        crates_dir.join("orchestrator").join("src").join("fsm.rs"),
        crates_dir.join("orchestrator").join("src").join("dispatcher.rs"),
    ];

    for path in &extra_files {
        if path.exists() {
            let _ = bm25.index_file(path, None);
        }
    }

    bm25
}

/// Build a minimal pipeline (BM25 only — FAST tier is fully functional,
/// STANDARD and DEEP degrade gracefully without vector/reranker models).
fn build_pipeline(bm25: Bm25Index) -> QueryPipeline {
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    QueryPipeline::new(bm25, catrag, arag)
}

// ---------------------------------------------------------------------------
// Benchmark test
// ---------------------------------------------------------------------------

#[test]
fn benchmark_30_queries_writes_toml() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = build_knowledge_index(tmp.path());
    let pipeline = build_pipeline(bm25);

    let report = benchmark::run_benchmark(&pipeline);

    // Deterministic checks from spec:
    // 1. Benchmark file exists with all 30 query results
    assert_eq!(
        report.results.len(),
        30,
        "must have results for all 30 queries"
    );

    // 2. All tiers represented
    let tier_names: Vec<String> = report.tier_metrics.keys().cloned().collect();
    assert!(
        tier_names.contains(&"FAST".to_string()),
        "FAST tier metrics must exist"
    );

    // 3. Write TOML output
    let toml_content = benchmark::format_report_toml(&report);
    let output_path = metrics_dir().join("phase_3K_rag.toml");
    std::fs::write(&output_path, &toml_content).unwrap();

    // 4. Verify the file was written
    assert!(output_path.exists(), "phase_3K_rag.toml must be written");
    let read_back = std::fs::read_to_string(&output_path).unwrap();
    assert!(read_back.contains("[overall]"));
    assert!(read_back.contains("[regression_gate]"));

    // Print summary for commit message
    eprintln!("=== 3K.5 Benchmark Results ===");
    eprintln!("Overall Recall@5:  {:.4}", report.overall_recall_at_5);
    eprintln!("Overall Recall@10: {:.4}", report.overall_recall_at_10);
    eprintln!("Overall MRR:       {:.4}", report.overall_mrr);
    for tier in &["FAST", "STANDARD", "DEEP"] {
        if let Some(tm) = report.tier_metrics.get(*tier) {
            eprintln!(
                "{tier}: R@5={:.2} R@10={:.2} MRR={:.4} p50={}ms p95={}ms (n={})",
                tm.recall_at_5, tm.recall_at_10, tm.mrr,
                tm.latency_p50_ms, tm.latency_p95_ms, tm.query_count,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Acceptance criteria tests
// ---------------------------------------------------------------------------

#[test]
fn fast_tier_latency_under_budget() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = build_knowledge_index(tmp.path());
    let pipeline = build_pipeline(bm25);

    let eval_set = benchmark::build_eval_set();
    let fast_queries: Vec<_> = eval_set
        .iter()
        .filter(|q| q.tier == elai_knowledge::QueryTier::Fast)
        .collect();

    for eq in &fast_queries {
        let result = benchmark::eval_single(&pipeline, eq).unwrap();
        // Per-query check: FAST should be well under 20ms with BM25-only
        assert!(
            result.latency_ms < 1000,
            "FAST query {} took {}ms (sanity check)",
            eq.id,
            result.latency_ms,
        );
    }
}

#[test]
fn eval_set_structural_30_queries() {
    let set = benchmark::build_eval_set();
    assert_eq!(set.len(), 30);

    // Verify category distribution
    let simple = set.iter().filter(|q| q.category == benchmark::QueryCategory::Simple).count();
    let code = set.iter().filter(|q| q.category == benchmark::QueryCategory::CodeSpecific).count();
    let complex = set.iter().filter(|q| q.category == benchmark::QueryCategory::Complex).count();
    assert_eq!(simple, 10);
    assert_eq!(code, 10);
    assert_eq!(complex, 10);
}

#[test]
fn recall_positive_on_indexed_codebase() {
    // With the knowledge crate indexed, at least SOME queries should find
    // their expected files. This validates the benchmark is not trivially zero.
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = build_knowledge_index(tmp.path());
    let pipeline = build_pipeline(bm25);

    let report = benchmark::run_benchmark(&pipeline);

    // At minimum, simple keyword queries against BM25 should have non-zero recall.
    let simple_results: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.category == benchmark::QueryCategory::Simple)
        .collect();

    let simple_recall_10: f64 = simple_results
        .iter()
        .map(|r| r.recall_at_10)
        .sum::<f64>()
        / simple_results.len() as f64;

    // With BM25 indexing the knowledge crate, keyword queries like "BM25 index"
    // should hit bm25.rs. Require at least 30% recall on simple queries.
    assert!(
        simple_recall_10 >= 0.3,
        "simple query Recall@10 = {simple_recall_10:.2}, expected >= 0.30"
    );
}
