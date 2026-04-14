//! Phase 3K.8 — improved retrieval quality benchmark integration test.
//!
//! Re-runs the 30-query evaluation with 3K.6 (A-RAG keyword scoring +
//! progressive disclosure) and 3K.7 (CatRAG symbolic anchoring + key-fact
//! enhancement) both active. Writes results to `metrics/phase_3K_rag_improved.toml`.
//! Regression-gates all metrics against the 3K.5 baseline.
//! Measures tokens-per-query delta from progressive disclosure.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::benchmark::{self, BenchmarkReport, EvalResult, TierMetrics};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::pipeline::QueryPipeline;

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Approximate characters per token for word-piece estimation.
const TOKEN_CHARS_PER_TOKEN: usize = 4;

/// Target: progressive disclosure should reduce tokens by at least 30%.
const TOKEN_REDUCTION_TARGET_PCT: f64 = -30.0;

/// Pre-disclosure baseline: tokens per query not measured in 3K.5.
/// Estimated from BM25 chunk sizes (~1000 tokens/chunk * ~10 chunks).
const BASELINE_TOKENS_PER_QUERY_ESTIMATE: f64 = 10_000.0;

/// Maximum allowed regression percentage before gate fails (mirrors benchmark.rs).
const REGRESSION_GATE_THRESHOLD_PCT: f64 = 5.0;

// 3K.5 baseline values (from metrics/phase_3K_rag.toml, verified).
const BASELINE_OVERALL_RECALL_AT_5: f64 = 0.6333;
const BASELINE_OVERALL_RECALL_AT_10: f64 = 0.7667;
const BASELINE_OVERALL_MRR: f64 = 0.4145;

const BASELINE_FAST_RECALL_AT_5: f64 = 0.5000;
const BASELINE_FAST_RECALL_AT_10: f64 = 0.7000;
const BASELINE_FAST_MRR: f64 = 0.4542;

const BASELINE_STANDARD_RECALL_AT_5: f64 = 0.8000;
const BASELINE_STANDARD_RECALL_AT_10: f64 = 0.8000;
const BASELINE_STANDARD_MRR: f64 = 0.5750;

const BASELINE_DEEP_RECALL_AT_5: f64 = 0.6000;
const BASELINE_DEEP_RECALL_AT_10: f64 = 0.8000;
const BASELINE_DEEP_MRR: f64 = 0.2143;

// ---------------------------------------------------------------------------
// Helpers (duplicated from benchmark_eval.rs — test files cannot share modules)
// ---------------------------------------------------------------------------

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn knowledge_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn metrics_dir() -> PathBuf {
    project_root().join("metrics")
}

fn build_knowledge_index(tmp_dir: &Path) -> Bm25Index {
    let mut bm25 = Bm25Index::new(tmp_dir, "benchmark_eval_improved").unwrap();
    let src = knowledge_src();

    bm25.index_directory(&src).unwrap();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = manifest_dir.parent().unwrap();

    let extra_files = [
        crates_dir.join("core").join("src").join("clock.rs"),
        crates_dir.join("core").join("src").join("entropy.rs"),
        crates_dir.join("core").join("src").join("config.rs"),
        crates_dir.join("orchestrator").join("src").join("fsm.rs"),
        crates_dir
            .join("orchestrator")
            .join("src")
            .join("dispatcher.rs"),
    ];

    for path in &extra_files {
        if path.exists() {
            let _ = bm25.index_file(path, None);
        }
    }

    bm25
}

fn build_pipeline(bm25: Bm25Index) -> QueryPipeline {
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    QueryPipeline::new(bm25, catrag, arag)
}

// ---------------------------------------------------------------------------
// Baseline tier metrics
// ---------------------------------------------------------------------------

fn baseline_tier_metrics() -> HashMap<String, TierMetrics> {
    let mut baselines = HashMap::new();
    baselines.insert(
        "FAST".to_string(),
        TierMetrics {
            recall_at_5: BASELINE_FAST_RECALL_AT_5,
            recall_at_10: BASELINE_FAST_RECALL_AT_10,
            mrr: BASELINE_FAST_MRR,
            latency_p50_ms: 0,
            latency_p95_ms: 1,
            query_count: 10,
        },
    );
    baselines.insert(
        "STANDARD".to_string(),
        TierMetrics {
            recall_at_5: BASELINE_STANDARD_RECALL_AT_5,
            recall_at_10: BASELINE_STANDARD_RECALL_AT_10,
            mrr: BASELINE_STANDARD_MRR,
            latency_p50_ms: 0,
            latency_p95_ms: 1,
            query_count: 10,
        },
    );
    baselines.insert(
        "DEEP".to_string(),
        TierMetrics {
            recall_at_5: BASELINE_DEEP_RECALL_AT_5,
            recall_at_10: BASELINE_DEEP_RECALL_AT_10,
            mrr: BASELINE_DEEP_MRR,
            latency_p50_ms: 3,
            latency_p95_ms: 4,
            query_count: 10,
        },
    );
    baselines
}

// ---------------------------------------------------------------------------
// Token measurement
// ---------------------------------------------------------------------------

/// Measure mean tokens per query by running each query and summing snippet
/// lengths from the last pipeline stage, divided by TOKEN_CHARS_PER_TOKEN.
fn measure_tokens_per_query(pipeline: &QueryPipeline) -> f64 {
    let eval_set = benchmark::build_eval_set();
    let mut total_tokens: usize = 0;
    let mut query_count: usize = 0;

    for eq in &eval_set {
        match pipeline.query_with_tier(&eq.query, eq.tier) {
            Ok(stages) => {
                if let Some(last_stage) = stages.last() {
                    let snippet_chars: usize =
                        last_stage.results.iter().map(|r| r.snippet.len()).sum();
                    total_tokens += snippet_chars / TOKEN_CHARS_PER_TOKEN;
                }
                query_count += 1;
            }
            Err(_) => {
                // Query failed — skip (consistent with eval_single behavior).
            }
        }
    }

    if query_count == 0 {
        return 0.0;
    }
    total_tokens as f64 / query_count as f64
}

// ---------------------------------------------------------------------------
// TOML builder
// ---------------------------------------------------------------------------

fn build_improved_toml(
    report: &BenchmarkReport,
    mean_tokens: f64,
    delta_pct: f64,
    gate_status: &str,
    violations: &[(String, f64, f64, f64)],
) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str("# Phase 3K.8 RAG Retrieval Quality Benchmark (3K.6+3K.7 active)\n");
    out.push_str("# Evaluation set: 30 queries (10 simple, 10 code-specific, 10 complex)\n");
    out.push_str("# Baseline: phase_3K_rag.toml (3K.5)\n\n");

    // [overall]
    out.push_str("[overall]\n");
    out.push_str(&format!("query_count = {}\n", report.results.len()));
    out.push_str(&format!(
        "recall_at_5 = {:.4}\n",
        report.overall_recall_at_5
    ));
    out.push_str(&format!(
        "recall_at_10 = {:.4}\n",
        report.overall_recall_at_10
    ));
    out.push_str(&format!("mrr = {:.4}\n", report.overall_mrr));
    out.push_str(&format!(
        "baseline_recall_at_5 = {:.4}\n",
        BASELINE_OVERALL_RECALL_AT_5
    ));
    out.push_str(&format!(
        "baseline_recall_at_10 = {:.4}\n",
        BASELINE_OVERALL_RECALL_AT_10
    ));
    out.push_str(&format!(
        "baseline_mrr = {:.4}\n",
        BASELINE_OVERALL_MRR
    ));
    out.push('\n');

    // [tier.*]
    for tier_name in &["FAST", "STANDARD", "DEEP"] {
        if let Some(tm) = report.tier_metrics.get(*tier_name) {
            out.push_str(&format!("[tier.{}]\n", tier_name.to_lowercase()));
            out.push_str(&format!("query_count = {}\n", tm.query_count));
            out.push_str(&format!("recall_at_5 = {:.4}\n", tm.recall_at_5));
            out.push_str(&format!("recall_at_10 = {:.4}\n", tm.recall_at_10));
            out.push_str(&format!("mrr = {:.4}\n", tm.mrr));
            out.push_str(&format!("latency_p50_ms = {}\n", tm.latency_p50_ms));
            out.push_str(&format!("latency_p95_ms = {}\n", tm.latency_p95_ms));
            out.push('\n');
        }
    }

    // [category.*]
    for category in &["simple", "code_specific", "complex"] {
        let cat_results: Vec<&EvalResult> = report
            .results
            .iter()
            .filter(|r| format!("{}", r.category) == *category)
            .collect();
        if !cat_results.is_empty() {
            let n = cat_results.len() as f64;
            let r5: f64 = cat_results.iter().map(|r| r.recall_at_5).sum::<f64>() / n;
            let r10: f64 = cat_results.iter().map(|r| r.recall_at_10).sum::<f64>() / n;
            let mrr: f64 = cat_results.iter().map(|r| r.mrr).sum::<f64>() / n;
            out.push_str(&format!("[category.{category}]\n"));
            out.push_str(&format!("query_count = {}\n", cat_results.len()));
            out.push_str(&format!("recall_at_5 = {:.4}\n", r5));
            out.push_str(&format!("recall_at_10 = {:.4}\n", r10));
            out.push_str(&format!("mrr = {:.4}\n", mrr));
            out.push('\n');
        }
    }

    // [queries]
    out.push_str("[queries]\n");
    for r in &report.results {
        out.push_str(&format!(
            "q{:02} = {{ category = \"{}\", tier = \"{}\", recall_5 = {:.1}, recall_10 = {:.1}, mrr = {:.4}, latency_ms = {}, hits = {} }}\n",
            r.query_id, r.category, r.tier, r.recall_at_5, r.recall_at_10, r.mrr, r.latency_ms, r.hit_count,
        ));
    }

    // [tokens_per_query]
    let target_met = delta_pct <= TOKEN_REDUCTION_TARGET_PCT;
    out.push_str("\n[tokens_per_query]\n");
    out.push_str(&format!("mean_tokens_per_query = {:.1}\n", mean_tokens));
    out.push_str(&format!(
        "baseline_mean_tokens_per_query = {:.1}\n",
        BASELINE_TOKENS_PER_QUERY_ESTIMATE
    ));
    out.push_str("baseline_note = \"pre-disclosure estimate (3K.5 had no snippet extraction)\"\n");
    out.push_str(&format!("delta_pct = {:.1}\n", delta_pct));
    out.push_str(&format!("target_met = {target_met}\n"));
    out.push_str(&format!(
        "target_description = \"{}% reduction in tokens per query\"\n",
        TOKEN_REDUCTION_TARGET_PCT.abs()
    ));

    // [regression_gate]
    out.push_str("\n[regression_gate]\n");
    out.push_str(&format!(
        "threshold_pct = {:.1}\n",
        REGRESSION_GATE_THRESHOLD_PCT
    ));
    out.push_str(&format!("status = \"{gate_status}\"\n"));
    if violations.is_empty() {
        out.push_str("violations = []\n");
    } else {
        out.push_str("violations = [\n");
        for (metric, baseline, current, pct) in violations {
            out.push_str(&format!(
                "  {{ metric = \"{metric}\", baseline = {baseline:.4}, current = {current:.4}, regression_pct = {pct:.2} }},\n"
            ));
        }
        out.push_str("]\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Integration test
// ---------------------------------------------------------------------------

#[test]
fn benchmark_30_queries_improved_writes_toml() {
    let tmp = tempfile::TempDir::new().unwrap();
    let bm25 = build_knowledge_index(tmp.path());
    let pipeline = build_pipeline(bm25);

    // 1. Run the full benchmark.
    let report = benchmark::run_benchmark(&pipeline);
    assert_eq!(
        report.results.len(),
        30,
        "must have results for all 30 queries"
    );

    // 2. Regression gate — check each tier against 3K.5 baseline.
    let baselines = baseline_tier_metrics();
    let mut all_violations: Vec<(String, f64, f64, f64)> = Vec::new();

    for tier_name in &["FAST", "STANDARD", "DEEP"] {
        let baseline = baselines
            .get(*tier_name)
            .expect("baseline must exist for tier");
        let current = report
            .tier_metrics
            .get(*tier_name)
            .expect("current run must have tier");

        let violations = benchmark::check_regression_gate(baseline, current, tier_name);
        if !violations.is_empty() {
            eprintln!("REGRESSION in {tier_name}:");
            for (metric, base_val, curr_val, pct) in &violations {
                eprintln!(
                    "  {metric}: baseline={base_val:.4} current={curr_val:.4} regression={pct:.2}%"
                );
            }
            all_violations.extend(violations);
        }
    }

    let gate_status = if all_violations.is_empty() {
        "PASS"
    } else {
        "FAIL"
    };

    // 3. Token measurement — secondary pass.
    let mean_tokens = measure_tokens_per_query(&pipeline);
    let delta_pct = if BASELINE_TOKENS_PER_QUERY_ESTIMATE > 0.0 {
        ((mean_tokens - BASELINE_TOKENS_PER_QUERY_ESTIMATE) / BASELINE_TOKENS_PER_QUERY_ESTIMATE)
            * 100.0
    } else {
        0.0
    };

    eprintln!("=== 3K.8 Improved Benchmark Results ===");
    eprintln!("Overall Recall@5:  {:.4}", report.overall_recall_at_5);
    eprintln!("Overall Recall@10: {:.4}", report.overall_recall_at_10);
    eprintln!("Overall MRR:       {:.4}", report.overall_mrr);
    for tier in &["FAST", "STANDARD", "DEEP"] {
        if let Some(tm) = report.tier_metrics.get(*tier) {
            eprintln!(
                "{tier}: R@5={:.2} R@10={:.2} MRR={:.4} p50={}ms p95={}ms (n={})",
                tm.recall_at_5,
                tm.recall_at_10,
                tm.mrr,
                tm.latency_p50_ms,
                tm.latency_p95_ms,
                tm.query_count,
            );
        }
    }
    eprintln!("Tokens/query: mean={mean_tokens:.1} baseline={BASELINE_TOKENS_PER_QUERY_ESTIMATE:.1} delta={delta_pct:.1}%");
    eprintln!("Regression gate: {gate_status}");

    // 4. Build and write TOML.
    let toml_content =
        build_improved_toml(&report, mean_tokens, delta_pct, gate_status, &all_violations);
    let output_path = metrics_dir().join("phase_3K_rag_improved.toml");
    std::fs::write(&output_path, &toml_content).unwrap();

    // 5. Assertions.
    assert!(
        output_path.exists(),
        "phase_3K_rag_improved.toml must be written"
    );
    let read_back = std::fs::read_to_string(&output_path).unwrap();
    assert!(read_back.contains("[overall]"));
    assert!(read_back.contains("[tokens_per_query]"));
    assert!(read_back.contains("[regression_gate]"));
    assert!(read_back.contains(&format!("status = \"{gate_status}\"")));

    // 6. Panic on regression violations (after writing TOML so results are preserved).
    assert!(
        all_violations.is_empty(),
        "regression gate FAILED: {all_violations:?}"
    );
}
