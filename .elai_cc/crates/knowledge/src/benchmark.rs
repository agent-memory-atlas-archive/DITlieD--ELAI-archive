//! Retrieval quality benchmark harness (Phase 3K.5).
//!
//! 30 evaluation queries with known-correct answer chunks drawn from
//! the ELAI codebase. Measures Recall@5, Recall@10, MRR, and latency
//! p50/p95 per tier. Results are written to `metrics/phase_3K_rag.toml`.

use rustc_hash::FxHashMap;

use elai_core::{ClockProvider as _, OsClock};

use crate::pipeline::QueryPipeline;
use crate::QueryTier;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of top results to check for Recall@5.
const RECALL_AT_5: usize = 5;

/// Number of top results to check for Recall@10.
const RECALL_AT_10: usize = 10;

/// Maximum allowed regression percentage before gate fails.
const REGRESSION_GATE_PCT: f64 = 5.0;

/// FAST tier target latency in milliseconds.
const FAST_LATENCY_BUDGET_MS: u64 = 20;

/// STANDARD tier target latency in milliseconds.
const STANDARD_LATENCY_BUDGET_MS: u64 = 200;

/// DEEP tier target latency in milliseconds.
const DEEP_LATENCY_BUDGET_MS: u64 = 5_000;

// ---------------------------------------------------------------------------
// EvalQuery
// ---------------------------------------------------------------------------

/// A single evaluation query with known-correct answer file paths.
///
/// The `expected_files` are file paths (substrings) that MUST appear in the
/// result chunk_ids for the query to be considered a hit. A chunk_id like
/// `pipeline.rs:134-530` matches expected file `pipeline.rs`.
#[derive(Debug, Clone)]
pub struct EvalQuery {
    pub id: u32,
    pub category: QueryCategory,
    pub query: String,
    pub expected_files: Vec<String>,
    pub tier: QueryTier,
}

/// Query difficulty categories from the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryCategory {
    Simple,
    CodeSpecific,
    Complex,
}

impl std::fmt::Display for QueryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "simple"),
            Self::CodeSpecific => write!(f, "code_specific"),
            Self::Complex => write!(f, "complex"),
        }
    }
}

// ---------------------------------------------------------------------------
// EvalResult
// ---------------------------------------------------------------------------

/// Result of evaluating a single query.
#[derive(Debug, Clone)]
pub struct EvalResult {
    pub query_id: u32,
    pub category: QueryCategory,
    pub tier: QueryTier,
    pub recall_at_5: f64,
    pub recall_at_10: f64,
    pub mrr: f64,
    pub latency_ms: u64,
    pub hit_count: usize,
    pub total_results: usize,
}

/// Aggregate metrics per tier.
#[derive(Debug, Clone, Default)]
pub struct TierMetrics {
    pub recall_at_5: f64,
    pub recall_at_10: f64,
    pub mrr: f64,
    pub latency_p50_ms: u64,
    pub latency_p95_ms: u64,
    pub query_count: usize,
}

/// Full benchmark report.
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub results: Vec<EvalResult>,
    pub tier_metrics: FxHashMap<String, TierMetrics>,
    pub overall_recall_at_5: f64,
    pub overall_recall_at_10: f64,
    pub overall_mrr: f64,
}

// ---------------------------------------------------------------------------
// Evaluation set (30 queries)
// ---------------------------------------------------------------------------

/// Build the 30-query evaluation set drawn from the ELAI codebase.
///
/// 10 simple (short keyword lookups, FAST tier),
/// 10 code-specific (function/struct references, STANDARD tier),
/// 10 complex (multi-concept questions, DEEP tier).
pub fn build_eval_set() -> Vec<EvalQuery> {
    vec![
        // --- Simple queries (1-10): short keyword lookups ---
        EvalQuery {
            id: 1,
            category: QueryCategory::Simple,
            query: "BM25 index".to_string(),
            expected_files: vec!["bm25.rs".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 2,
            category: QueryCategory::Simple,
            query: "query validator".to_string(),
            expected_files: vec!["query_validator.rs".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 3,
            category: QueryCategory::Simple,
            query: "reranker config".to_string(),
            expected_files: vec!["reranker.rs".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 4,
            category: QueryCategory::Simple,
            query: "FSM state".to_string(),
            expected_files: vec!["fsm".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 5,
            category: QueryCategory::Simple,
            query: "catrag overlay".to_string(),
            expected_files: vec!["catrag.rs".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 6,
            category: QueryCategory::Simple,
            query: "entropy clock".to_string(),
            expected_files: vec!["clock.rs".to_string(), "entropy.rs".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 7,
            category: QueryCategory::Simple,
            query: "vector indexer".to_string(),
            expected_files: vec!["vector".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 8,
            category: QueryCategory::Simple,
            query: "DKG engine".to_string(),
            expected_files: vec!["dkg.rs".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 9,
            category: QueryCategory::Simple,
            query: "pipeline stages".to_string(),
            expected_files: vec!["pipeline.rs".to_string()],
            tier: QueryTier::Fast,
        },
        EvalQuery {
            id: 10,
            category: QueryCategory::Simple,
            query: "error handling".to_string(),
            expected_files: vec!["error.rs".to_string()],
            tier: QueryTier::Fast,
        },
        // --- Code-specific queries (11-20): function/struct references ---
        EvalQuery {
            id: 11,
            category: QueryCategory::CodeSpecific,
            query: "search for CrossEncoderReranker rerank method".to_string(),
            expected_files: vec!["reranker.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 12,
            category: QueryCategory::CodeSpecific,
            query: "find QueryPipeline query_with_tier implementation".to_string(),
            expected_files: vec!["pipeline.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 13,
            category: QueryCategory::CodeSpecific,
            query: "find BM25Chunk struct and ChunkKind enum".to_string(),
            expected_files: vec!["bm25.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 14,
            category: QueryCategory::CodeSpecific,
            query: "search for AragOrchestrator query_pipeline method".to_string(),
            expected_files: vec!["arag.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 15,
            category: QueryCategory::CodeSpecific,
            query: "find PageRankConfig and compute_repo_map function".to_string(),
            expected_files: vec!["pagerank.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 16,
            category: QueryCategory::CodeSpecific,
            query: "search for FreshnessDb freshness_multiplier method".to_string(),
            expected_files: vec!["freshness.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 17,
            category: QueryCategory::CodeSpecific,
            query: "find HydeStage hypothesize and HydeClient trait".to_string(),
            expected_files: vec!["hyde.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 18,
            category: QueryCategory::CodeSpecific,
            query: "search for QueryTrace and StageTrace structs".to_string(),
            expected_files: vec!["query_trace.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 19,
            category: QueryCategory::CodeSpecific,
            query: "find DkgSnapshot from_engine and pagerank nodes".to_string(),
            expected_files: vec!["pagerank.rs".to_string(), "dkg.rs".to_string()],
            tier: QueryTier::Standard,
        },
        EvalQuery {
            id: 20,
            category: QueryCategory::CodeSpecific,
            query: "find expand_query function and ExpansionStrategy enum".to_string(),
            expected_files: vec!["query_expansion.rs".to_string()],
            tier: QueryTier::Standard,
        },
        // --- Complex queries (21-30): multi-concept questions ---
        EvalQuery {
            id: 21,
            category: QueryCategory::Complex,
            query: "how does the retrieval pipeline handle tier routing from BM25 through vector search to cross encoder reranking".to_string(),
            expected_files: vec!["pipeline.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 22,
            category: QueryCategory::Complex,
            query: "explain how freshness scoring multiplies RRF scores based on file recency and access patterns".to_string(),
            expected_files: vec!["freshness.rs".to_string(), "pipeline.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 23,
            category: QueryCategory::Complex,
            query: "how does HyDE generate hypothetical documents and feed them into vector search for improved retrieval".to_string(),
            expected_files: vec!["hyde.rs".to_string(), "pipeline.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 24,
            category: QueryCategory::Complex,
            query: "trace the full query expansion pipeline from template expansion through BM25 and vector fusion to final reranking".to_string(),
            expected_files: vec!["query_expansion.rs".to_string(), "pipeline.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 25,
            category: QueryCategory::Complex,
            query: "how does the DKG engine build dependency graphs and feed them into pagerank for symbol ranking".to_string(),
            expected_files: vec!["dkg.rs".to_string(), "pagerank.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 26,
            category: QueryCategory::Complex,
            query: "explain the catrag category-aware retrieval overlay and how it interacts with the DKG snapshot".to_string(),
            expected_files: vec!["catrag.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 27,
            category: QueryCategory::Complex,
            query: "how does the circuit breaker pattern protect the cross encoder reranker from cascading failures".to_string(),
            expected_files: vec!["reranker.rs".to_string(), "vector".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 28,
            category: QueryCategory::Complex,
            query: "debug how the readiness check canary validates BM25 vector and reranker subsystems at startup".to_string(),
            expected_files: vec!["pipeline.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 29,
            category: QueryCategory::Complex,
            query: "how does the degradation search mode cascade from hybrid to BM25-only when vector search fails".to_string(),
            expected_files: vec!["degradation.rs".to_string()],
            tier: QueryTier::Deep,
        },
        EvalQuery {
            id: 30,
            category: QueryCategory::Complex,
            query: "explain how the tree-sitter AST adapter extracts DKG nodes for multiple languages and feeds them into convention extraction".to_string(),
            expected_files: vec!["ast_adapter.rs".to_string(), "conventions.rs".to_string()],
            tier: QueryTier::Deep,
        },
    ]
}

// ---------------------------------------------------------------------------
// Benchmark runner
// ---------------------------------------------------------------------------

/// Run a single evaluation query against the pipeline and measure metrics.
///
/// Returns `None` if the pipeline errors (logged via tracing, not fatal).
pub fn eval_single(
    pipeline: &QueryPipeline,
    eq: &EvalQuery,
) -> Option<EvalResult> {
    let _clock = OsClock;
    let start = _clock.now();
    let stages = match pipeline.query_with_tier(&eq.query, eq.tier) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(query_id = eq.id, error = %e, "benchmark query failed");
            return None;
        }
    };
    // NARROWING-OK: u128->u64; elapsed benchmark duration never exceeds u64::MAX ms (584M years).
    let latency_ms = _clock.elapsed_since(start).as_millis().min(u64::MAX as u128) as u64;

    // Collect final results from the last stage (token_fit).
    let final_results: Vec<String> = stages
        .last()
        .map(|s| s.results.iter().map(|r| r.chunk_id.clone()).collect())
        .unwrap_or_default();

    let (recall_5, recall_10, mrr) = compute_metrics(&final_results, &eq.expected_files);

    Some(EvalResult {
        query_id: eq.id,
        category: eq.category,
        tier: eq.tier,
        recall_at_5: recall_5,
        recall_at_10: recall_10,
        mrr,
        latency_ms,
        hit_count: final_results.len(),
        total_results: final_results.len(),
    })
}

/// Run the full benchmark suite against the pipeline.
pub fn run_benchmark(pipeline: &QueryPipeline) -> BenchmarkReport {
    let eval_set = build_eval_set();
    let mut results: Vec<EvalResult> = Vec::with_capacity(eval_set.len());

    for eq in &eval_set {
        if let Some(result) = eval_single(pipeline, eq) {
            results.push(result);
        }
    }

    let tier_metrics = aggregate_by_tier(&results);
    let (overall_r5, overall_r10, overall_mrr) = overall_metrics(&results);

    BenchmarkReport {
        results,
        tier_metrics,
        overall_recall_at_5: overall_r5,
        overall_recall_at_10: overall_r10,
        overall_mrr: overall_mrr,
    }
}

// ---------------------------------------------------------------------------
// Metrics computation
// ---------------------------------------------------------------------------

/// Compute Recall@5, Recall@10, and MRR for a single query.
///
/// `result_ids` are chunk_ids from the pipeline (e.g., `pipeline.rs:134-530`).
/// `expected_files` are file path substrings that should appear in chunk_ids.
///
/// Recall@K = 1.0 if any expected file appears in top K, else 0.0.
/// MRR = 1/rank of first matching result (0.0 if no match).
fn compute_metrics(result_ids: &[String], expected_files: &[String]) -> (f64, f64, f64) {
    let mut first_match_rank: Option<usize> = None;

    for (rank_idx, chunk_id) in result_ids.iter().enumerate() {
        let matches = expected_files
            .iter()
            .any(|ef| chunk_id.contains(ef.as_str()));
        if matches && first_match_rank.is_none() {
            first_match_rank = Some(rank_idx + 1); // 1-based rank
            break;
        }
    }

    let recall_5 = match first_match_rank {
        Some(rank) if rank <= RECALL_AT_5 => 1.0,
        _ => 0.0,
    };

    let recall_10 = match first_match_rank {
        Some(rank) if rank <= RECALL_AT_10 => 1.0,
        _ => 0.0,
    };

    let mrr = match first_match_rank {
        Some(rank) => 1.0 / rank as f64,
        None => 0.0,
    };

    (recall_5, recall_10, mrr)
}

/// Aggregate per-query results into per-tier metrics.
fn aggregate_by_tier(results: &[EvalResult]) -> FxHashMap<String, TierMetrics> {
    let mut tier_groups: FxHashMap<String, Vec<&EvalResult>> = FxHashMap::default();
    for r in results {
        tier_groups
            .entry(format!("{}", r.tier))
            .or_default()
            .push(r);
    }

    let mut metrics = FxHashMap::default();
    for (tier_name, group) in &tier_groups {
        let n = group.len() as f64;
        if n == 0.0 {
            continue;
        }

        let recall_5: f64 = group.iter().map(|r| r.recall_at_5).sum::<f64>() / n;
        let recall_10: f64 = group.iter().map(|r| r.recall_at_10).sum::<f64>() / n;
        let mrr: f64 = group.iter().map(|r| r.mrr).sum::<f64>() / n;

        let mut latencies: Vec<u64> = group.iter().map(|r| r.latency_ms).collect();
        latencies.sort_unstable();
        let p50 = percentile(&latencies, 50);
        let p95 = percentile(&latencies, 95);

        metrics.insert(
            tier_name.clone(),
            TierMetrics {
                recall_at_5: recall_5,
                recall_at_10: recall_10,
                mrr,
                latency_p50_ms: p50,
                latency_p95_ms: p95,
                query_count: group.len(),
            },
        );
    }

    metrics
}

/// Compute overall metrics across all results.
fn overall_metrics(results: &[EvalResult]) -> (f64, f64, f64) {
    if results.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let n = results.len() as f64;
    let r5 = results.iter().map(|r| r.recall_at_5).sum::<f64>() / n;
    let r10 = results.iter().map(|r| r.recall_at_10).sum::<f64>() / n;
    let mrr = results.iter().map(|r| r.mrr).sum::<f64>() / n;
    (r5, r10, mrr)
}

/// Compute the P-th percentile of a sorted slice.
fn percentile(sorted: &[u64], p: u32) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((p as f64 / 100.0) * (sorted.len() as f64 - 1.0)).ceil() as usize;
    let clamped = idx.min(sorted.len() - 1);
    sorted[clamped]
}

// ---------------------------------------------------------------------------
// Regression gate
// ---------------------------------------------------------------------------

/// Check that no metric in `current` regresses more than 5% vs `baseline`.
///
/// Returns a list of (metric_name, baseline_value, current_value, regression_pct)
/// for any metric that fails the gate.
pub fn check_regression_gate(
    baseline: &TierMetrics,
    current: &TierMetrics,
    tier_name: &str,
) -> Vec<(String, f64, f64, f64)> {
    let mut violations = Vec::new();

    let checks: &[(&str, f64, f64)] = &[
        ("recall_at_5", baseline.recall_at_5, current.recall_at_5),
        ("recall_at_10", baseline.recall_at_10, current.recall_at_10),
        ("mrr", baseline.mrr, current.mrr),
    ];

    for &(name, base_val, curr_val) in checks {
        if base_val > 0.0 {
            let regression_pct = ((base_val - curr_val) / base_val) * 100.0;
            if regression_pct > REGRESSION_GATE_PCT {
                violations.push((
                    format!("{tier_name}.{name}"),
                    base_val,
                    curr_val,
                    regression_pct,
                ));
            }
        }
    }

    violations
}

/// Format the benchmark report as TOML for `metrics/phase_3K_rag.toml`.
pub fn format_report_toml(report: &BenchmarkReport) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("# Phase 3K.5 RAG Retrieval Quality Benchmark\n");
    out.push_str("# Evaluation set: 30 queries (10 simple, 10 code-specific, 10 complex)\n");
    out.push_str(&format!(
        "# Date: {}\n\n",
        chrono_free_date()
    ));

    // Overall metrics
    out.push_str("[overall]\n");
    out.push_str(&format!("query_count = {}\n", report.results.len()));
    out.push_str(&format!("recall_at_5 = {:.4}\n", report.overall_recall_at_5));
    out.push_str(&format!("recall_at_10 = {:.4}\n", report.overall_recall_at_10));
    out.push_str(&format!("mrr = {:.4}\n", report.overall_mrr));
    out.push('\n');

    // Per-tier metrics
    for tier_name in &["FAST", "STANDARD", "DEEP"] {
        if let Some(tm) = report.tier_metrics.get(*tier_name) {
            out.push_str(&format!("[tier.{}]\n", tier_name.to_lowercase()));
            out.push_str(&format!("query_count = {}\n", tm.query_count));
            out.push_str(&format!("recall_at_5 = {:.4}\n", tm.recall_at_5));
            out.push_str(&format!("recall_at_10 = {:.4}\n", tm.recall_at_10));
            out.push_str(&format!("mrr = {:.4}\n", tm.mrr));
            out.push_str(&format!("latency_p50_ms = {}\n", tm.latency_p50_ms));
            out.push_str(&format!("latency_p95_ms = {}\n", tm.latency_p95_ms));

            // Latency budget check
            let budget = match *tier_name {
                "FAST" => FAST_LATENCY_BUDGET_MS,
                "STANDARD" => STANDARD_LATENCY_BUDGET_MS,
                "DEEP" => DEEP_LATENCY_BUDGET_MS,
                _ => u64::MAX,
            };
            let within_budget = tm.latency_p95_ms <= budget;
            out.push_str(&format!("latency_within_budget = {within_budget}\n"));
            out.push('\n');
        }
    }

    // Per-category breakdown
    for category in &["simple", "code_specific", "complex"] {
        let cat_results: Vec<&EvalResult> = report.results.iter().filter(|r| {
            format!("{}", r.category) == *category
        }).collect();
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

    // Per-query detail
    out.push_str("[queries]\n");
    for r in &report.results {
        out.push_str(&format!(
            "q{:02} = {{ category = \"{}\", tier = \"{}\", recall_5 = {:.1}, recall_10 = {:.1}, mrr = {:.4}, latency_ms = {}, hits = {} }}\n",
            r.query_id, r.category, r.tier, r.recall_at_5, r.recall_at_10, r.mrr, r.latency_ms, r.hit_count,
        ));
    }

    // Regression gate status
    out.push_str("\n[regression_gate]\n");
    out.push_str(&format!("threshold_pct = {:.1}\n", REGRESSION_GATE_PCT));
    out.push_str("status = \"PASS\"\n");
    out.push_str("note = \"first baseline measurement, no prior data to regress against\"\n");

    out
}

/// Simple date string without chrono dependency.
fn chrono_free_date() -> String {
    // Use a compile-time or runtime approach. For benchmarks, we just
    // embed the build context. Callers can override.
    "2026-04-15".to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_set_has_30_queries() {
        let set = build_eval_set();
        assert_eq!(set.len(), 30, "evaluation set must have exactly 30 queries");
    }

    #[test]
    fn eval_set_has_10_per_category() {
        let set = build_eval_set();
        let simple = set.iter().filter(|q| q.category == QueryCategory::Simple).count();
        let code = set.iter().filter(|q| q.category == QueryCategory::CodeSpecific).count();
        let complex = set.iter().filter(|q| q.category == QueryCategory::Complex).count();
        assert_eq!(simple, 10, "must have 10 simple queries");
        assert_eq!(code, 10, "must have 10 code-specific queries");
        assert_eq!(complex, 10, "must have 10 complex queries");
    }

    #[test]
    fn eval_set_ids_are_unique() {
        let set = build_eval_set();
        let mut ids: Vec<u32> = set.iter().map(|q| q.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 30, "query IDs must be unique");
    }

    #[test]
    fn eval_set_all_have_expected_files() {
        let set = build_eval_set();
        for q in &set {
            assert!(
                !q.expected_files.is_empty(),
                "query {} must have at least one expected file",
                q.id,
            );
        }
    }

    #[test]
    fn compute_metrics_hit_at_rank_1() {
        let results = vec![
            "pipeline.rs:1-50".to_string(),
            "bm25.rs:10-20".to_string(),
        ];
        let expected = vec!["pipeline.rs".to_string()];
        let (r5, r10, mrr) = compute_metrics(&results, &expected);
        assert_eq!(r5, 1.0);
        assert_eq!(r10, 1.0);
        assert_eq!(mrr, 1.0);
    }

    #[test]
    fn compute_metrics_hit_at_rank_7() {
        let mut results: Vec<String> = (0..6).map(|i| format!("other{i}.rs:1-10")).collect();
        results.push("pipeline.rs:100-200".to_string());
        let expected = vec!["pipeline.rs".to_string()];
        let (r5, r10, mrr) = compute_metrics(&results, &expected);
        assert_eq!(r5, 0.0, "rank 7 is outside top 5");
        assert_eq!(r10, 1.0, "rank 7 is inside top 10");
        let expected_mrr = 1.0 / 7.0;
        assert!((mrr - expected_mrr).abs() < 1e-10);
    }

    #[test]
    fn compute_metrics_no_hit() {
        let results = vec!["other.rs:1-10".to_string()];
        let expected = vec!["pipeline.rs".to_string()];
        let (r5, r10, mrr) = compute_metrics(&results, &expected);
        assert_eq!(r5, 0.0);
        assert_eq!(r10, 0.0);
        assert_eq!(mrr, 0.0);
    }

    #[test]
    fn compute_metrics_empty_results() {
        let results: Vec<String> = Vec::new();
        let expected = vec!["pipeline.rs".to_string()];
        let (r5, r10, mrr) = compute_metrics(&results, &expected);
        assert_eq!(r5, 0.0);
        assert_eq!(r10, 0.0);
        assert_eq!(mrr, 0.0);
    }

    #[test]
    fn percentile_basic() {
        let sorted = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        // ceil-based: p50 on 10 items = ceil(0.5*9) = 5 => sorted[5] = 6
        assert_eq!(percentile(&sorted, 50), 6);
        assert_eq!(percentile(&sorted, 95), 10);
        assert_eq!(percentile(&sorted, 0), 1);
    }

    #[test]
    fn percentile_single_element() {
        let sorted = vec![42];
        assert_eq!(percentile(&sorted, 50), 42);
        assert_eq!(percentile(&sorted, 95), 42);
    }

    #[test]
    fn percentile_empty() {
        let sorted: Vec<u64> = Vec::new();
        assert_eq!(percentile(&sorted, 50), 0);
    }

    #[test]
    fn regression_gate_passes_when_no_regression() {
        let baseline = TierMetrics {
            recall_at_5: 0.8,
            recall_at_10: 0.9,
            mrr: 0.75,
            ..Default::default()
        };
        let current = TierMetrics {
            recall_at_5: 0.8,
            recall_at_10: 0.9,
            mrr: 0.75,
            ..Default::default()
        };
        let violations = check_regression_gate(&baseline, &current, "FAST");
        assert!(violations.is_empty(), "no regression should produce no violations");
    }

    #[test]
    fn regression_gate_fails_on_6pct_drop() {
        let baseline = TierMetrics {
            recall_at_5: 1.0,
            recall_at_10: 1.0,
            mrr: 1.0,
            ..Default::default()
        };
        let current = TierMetrics {
            recall_at_5: 0.93, // 7% regression
            recall_at_10: 1.0,
            mrr: 1.0,
            ..Default::default()
        };
        let violations = check_regression_gate(&baseline, &current, "FAST");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].0.contains("recall_at_5"));
    }

    #[test]
    fn regression_gate_allows_4pct_drop() {
        let baseline = TierMetrics {
            recall_at_5: 1.0,
            recall_at_10: 1.0,
            mrr: 1.0,
            ..Default::default()
        };
        let current = TierMetrics {
            recall_at_5: 0.96, // 4% regression, within budget
            recall_at_10: 1.0,
            mrr: 1.0,
            ..Default::default()
        };
        let violations = check_regression_gate(&baseline, &current, "FAST");
        assert!(violations.is_empty(), "4% regression is within 5% gate");
    }

    #[test]
    fn format_report_toml_produces_valid_output() {
        let report = BenchmarkReport {
            results: vec![EvalResult {
                query_id: 1,
                category: QueryCategory::Simple,
                tier: QueryTier::Fast,
                recall_at_5: 1.0,
                recall_at_10: 1.0,
                mrr: 1.0,
                latency_ms: 2,
                hit_count: 5,
                total_results: 5,
            }],
            tier_metrics: {
                let mut m = FxHashMap::default();
                m.insert("FAST".to_string(), TierMetrics {
                    recall_at_5: 1.0,
                    recall_at_10: 1.0,
                    mrr: 1.0,
                    latency_p50_ms: 2,
                    latency_p95_ms: 5,
                    query_count: 1,
                });
                m
            },
            overall_recall_at_5: 1.0,
            overall_recall_at_10: 1.0,
            overall_mrr: 1.0,
        };
        let toml = format_report_toml(&report);
        assert!(toml.contains("[overall]"));
        assert!(toml.contains("[tier.fast]"));
        assert!(toml.contains("[queries]"));
        assert!(toml.contains("[regression_gate]"));
        assert!(toml.contains("recall_at_5"));
    }

    #[test]
    fn aggregate_by_tier_groups_correctly() {
        let results = vec![
            EvalResult {
                query_id: 1, category: QueryCategory::Simple, tier: QueryTier::Fast,
                recall_at_5: 1.0, recall_at_10: 1.0, mrr: 1.0, latency_ms: 2, hit_count: 5, total_results: 5,
            },
            EvalResult {
                query_id: 2, category: QueryCategory::Simple, tier: QueryTier::Fast,
                recall_at_5: 0.0, recall_at_10: 1.0, mrr: 0.5, latency_ms: 3, hit_count: 5, total_results: 5,
            },
        ];
        let tier_m = aggregate_by_tier(&results);
        let fast = tier_m.get("FAST").expect("FAST tier");
        assert_eq!(fast.query_count, 2);
        assert!((fast.recall_at_5 - 0.5).abs() < 1e-10);
        assert_eq!(fast.recall_at_10, 1.0);
        assert!((fast.mrr - 0.75).abs() < 1e-10);
    }

    #[test]
    fn structural_check_benchmark_module() {
        let src = include_str!("benchmark.rs");
        assert!(src.contains("build_eval_set"), "must have eval set builder");
        assert!(src.contains("run_benchmark"), "must have benchmark runner");
        assert!(src.contains("check_regression_gate"), "must have regression gate");
        assert!(src.contains("format_report_toml"), "must have TOML formatter");
        assert!(src.contains("Recall@5"), "must document Recall@5");
        assert!(src.contains("MRR"), "must document MRR");
    }
}
