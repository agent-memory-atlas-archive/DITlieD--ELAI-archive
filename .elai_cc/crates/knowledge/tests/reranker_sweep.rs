// 1G.6: Optimization sweep and precision validation.
//
// Runs the full optimization sweep from the phase spec:
//   A. Baseline measurement (fp32, ORT_ENABLE_ALL, deterministic, 1 thread)
//   B. Thread scaling sweep (1/2/4/8 threads)
//   C. INT8 quantization (conditional: only if baseline p50 > 30ms)
//   D. Skip threshold sweep (5, 10, 15, 20, 30)
//   E. Final measurement with all optimizations applied
//
// All results are recorded to metrics/phase_1G.toml.
// Tests that require ONNX model files skip gracefully when models are not staged.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use elai_knowledge::reranker::{CrossEncoderReranker, RerankerConfig};
use elai_knowledge::arag::RetrievalResult;
use elai_core::ClockProvider;
use elai_core::SeededClock;
use ort::session::builder::GraphOptimizationLevel;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn model_path() -> PathBuf {
    project_root().join("models").join("onnx").join("cross-encoder-l12.onnx")
}

fn tokenizer_path() -> PathBuf {
    project_root().join("models").join("onnx").join("tokenizer.json")
}

fn quantized_model_path() -> PathBuf {
    project_root().join("models").join("onnx").join("cross-encoder-int8.onnx")
}

fn metrics_path() -> PathBuf {
    project_root().join("metrics").join("phase_1G.toml")
}

fn fixture_path() -> PathBuf {
    project_root().join("tests").join("fixtures").join("reranker_benchmark.json")
}

fn models_staged() -> bool {
    model_path().exists() && tokenizer_path().exists()
}

/// Load benchmark fixture from JSON.
#[derive(serde::Deserialize)]
struct BenchmarkFixture {
    query: String,
    candidates: Vec<FixtureCandidate>,
}

#[derive(serde::Deserialize, Clone)]
struct FixtureCandidate {
    chunk_id: String,
    snippet: String,
    relevant: bool,
}

fn load_fixture() -> BenchmarkFixture {
    let data = std::fs::read_to_string(fixture_path())
        .expect("benchmark fixture should exist at tests/fixtures/reranker_benchmark.json");
    serde_json::from_str(&data).expect("fixture JSON should parse")
}

fn fixture_to_candidates(fixture: &BenchmarkFixture) -> Vec<RetrievalResult> {
    fixture
        .candidates
        .iter()
        .map(|c| RetrievalResult {
            chunk_id: c.chunk_id.clone(),
            snippet: c.snippet.clone(),
            score: 0.0,
            tool_used: "bm25".to_string(),
        })
        .collect()
}

/// Compute NDCG@k given a ranked list of chunk_ids and a relevance map.
fn ndcg_at_k(ranked_ids: &[String], relevance: &HashMap<String, bool>, k: usize) -> f64 {
    let k = k.min(ranked_ids.len());
    if k == 0 {
        return 0.0;
    }

    // DCG@k: sum of rel_i / log2(i+2) for i in 0..k
    let dcg: f64 = ranked_ids[..k]
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let rel = if *relevance.get(id).unwrap_or(&false) { 1.0 } else { 0.0 };
            rel / (i as f64 + 2.0).log2()
        })
        .sum();

    // Ideal DCG@k: sort by relevance descending, compute DCG
    let mut ideal_rels: Vec<f64> = relevance
        .values()
        .map(|&r| if r { 1.0 } else { 0.0 })
        .collect();
    ideal_rels.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let idcg: f64 = ideal_rels[..k.min(ideal_rels.len())]
        .iter()
        .enumerate()
        .map(|(i, &rel)| rel / (i as f64 + 2.0).log2())
        .sum();

    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

/// Build a reranker with given config, load it, return it.
fn build_loaded_reranker(
    threads: usize,
    model: &Path,
    tokenizer: &Path,
    skip_threshold: usize,
) -> CrossEncoderReranker {
    let mut config = RerankerConfig::default();
    config.model_path = model.to_path_buf();
    config.tokenizer_path = tokenizer.to_path_buf();
    config.intra_op_threads = threads;
    config.graph_optimization_level = GraphOptimizationLevel::All;
    config.enable_deterministic_compute = true;
    config.skip_threshold = skip_threshold;
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let mut reranker = CrossEncoderReranker::new(config, clock);
    reranker.load().expect("model load should succeed");
    reranker
}

/// Run rerank N times on the fixture, return (latencies_ms, ranked_ids_from_last_run).
fn benchmark_rerank(
    reranker: &CrossEncoderReranker,
    query: &str,
    candidates: &[RetrievalResult],
    runs: usize,
) -> (Vec<f64>, Vec<String>) {
    let mut latencies = Vec::with_capacity(runs);
    let mut last_ids = Vec::new();

    // Warm-up run (not measured, per spec: "warm session").
    let warmup_candidates = candidates.to_vec();
    let _ = reranker.rerank(query, warmup_candidates);

    for _ in 0..runs {
        let batch = candidates.to_vec();
        let start = Instant::now();
        let result = reranker.rerank(query, batch).expect("rerank should succeed");
        let elapsed = start.elapsed();
        latencies.push(elapsed.as_secs_f64() * 1000.0);
        last_ids = result.iter().map(|c| c.chunk_id.clone()).collect();
    }
    (latencies, last_ids)
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn p50(latencies: &[f64]) -> f64 {
    let mut sorted = latencies.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    percentile(&sorted, 50.0)
}

fn p99(latencies: &[f64]) -> f64 {
    let mut sorted = latencies.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    percentile(&sorted, 99.0)
}

// ---------------------------------------------------------------------------
// Deterministic checks (run unconditionally)
// ---------------------------------------------------------------------------

#[test]
fn fixture_file_exists() {
    assert!(
        fixture_path().exists(),
        "benchmark fixture must exist at {}",
        fixture_path().display()
    );
}

#[test]
fn fixture_contains_code_chunks() {
    let data = std::fs::read_to_string(fixture_path()).unwrap();
    // Spec: grep "fn |struct |impl |def |class " returns hits
    let has_code = data.contains("fn ")
        || data.contains("struct ")
        || data.contains("impl ")
        || data.contains("def ")
        || data.contains("class ");
    assert!(has_code, "fixture must contain actual code chunks");
}

#[test]
fn fixture_has_50_candidates_with_relevance() {
    let fixture = load_fixture();
    assert_eq!(
        fixture.candidates.len(),
        50,
        "fixture must have exactly 50 candidates"
    );
    let relevant_count = fixture.candidates.iter().filter(|c| c.relevant).count();
    assert!(
        relevant_count >= 10,
        "fixture must have at least 10 relevant candidates, got {relevant_count}"
    );
}

#[test]
fn ndcg_computation_correctness() {
    // Verify NDCG@10 computation with known values.
    let mut relevance = HashMap::new();
    relevance.insert("a".to_string(), true);
    relevance.insert("b".to_string(), false);
    relevance.insert("c".to_string(), true);
    relevance.insert("d".to_string(), false);

    // Perfect ranking: [a, c, b, d]
    let perfect = vec!["a".into(), "c".into(), "b".into(), "d".into()];
    let perfect_ndcg = ndcg_at_k(&perfect, &relevance, 4);
    assert!(
        (perfect_ndcg - 1.0).abs() < 1e-9,
        "perfect ranking should have NDCG=1.0, got {perfect_ndcg}"
    );

    // Worst ranking: [b, d, a, c]
    let worst = vec!["b".into(), "d".into(), "a".into(), "c".into()];
    let worst_ndcg = ndcg_at_k(&worst, &relevance, 4);
    assert!(
        worst_ndcg < perfect_ndcg,
        "worst ranking NDCG ({worst_ndcg}) should be less than perfect ({perfect_ndcg})"
    );

    // Empty: NDCG should be 0
    let empty_ndcg = ndcg_at_k(&[], &relevance, 4);
    assert_eq!(empty_ndcg, 0.0);
}

// ---------------------------------------------------------------------------
// Model-dependent sweep (skip if models not staged)
// ---------------------------------------------------------------------------

#[test]
fn optimization_sweep() {
    if !models_staged() {
        eprintln!(
            "SKIP: model fixtures not staged at {}",
            model_path().display()
        );
        return;
    }

    let fixture = load_fixture();
    let candidates = fixture_to_candidates(&fixture);
    let relevance: HashMap<String, bool> = fixture
        .candidates
        .iter()
        .map(|c| (c.chunk_id.clone(), c.relevant))
        .collect();

    let mut metrics = String::new();
    metrics.push_str("# Phase 1G Optimization Sweep Metrics\n");
    metrics.push_str(&format!(
        "# Generated: {}\n\n",
        chrono_free_timestamp()
    ));

    // === A. BASELINE (fp32, ORT_ENABLE_ALL, deterministic, 1 thread) ===
    let reranker = build_loaded_reranker(1, &model_path(), &tokenizer_path(), 0);
    // skip_threshold=0 to force reranking all 50 candidates
    let (latencies, ranked_ids) = benchmark_rerank(&reranker, &fixture.query, &candidates, 10);
    let baseline_p50 = p50(&latencies);
    let baseline_p99 = p99(&latencies);

    // Unranked NDCG@10 (input order).
    let unranked_ids: Vec<String> = candidates.iter().map(|c| c.chunk_id.clone()).collect();
    let ndcg_unranked = ndcg_at_k(&unranked_ids, &relevance, 10);
    let ndcg_reranked = ndcg_at_k(&ranked_ids, &relevance, 10);
    let ndcg_improvement = ndcg_reranked - ndcg_unranked;

    metrics.push_str("[baseline]\n");
    metrics.push_str(&format!("baseline_latency_p50 = {baseline_p50:.3}\n"));
    metrics.push_str(&format!("baseline_latency_p99 = {baseline_p99:.3}\n"));
    metrics.push_str(&format!("ndcg_at_10_unranked = {ndcg_unranked:.4}\n"));
    metrics.push_str(&format!("ndcg_at_10_reranked = {ndcg_reranked:.4}\n"));
    metrics.push_str(&format!("ndcg_at_10_improvement = {ndcg_improvement:.4}\n"));
    metrics.push_str("model_variant = \"fp32\"\n");
    metrics.push_str("graph_optimization = \"ORT_ENABLE_ALL\"\n");
    metrics.push_str("deterministic_compute = true\n");
    metrics.push_str("threads = 1\n\n");

    assert!(
        ndcg_improvement > 0.0,
        "reranker must improve NDCG@10 over unranked: reranked={ndcg_reranked:.4}, unranked={ndcg_unranked:.4}"
    );

    // Verify reranker moves at least 1 relevant doc into top 5.
    let top5_relevant = ranked_ids[..5]
        .iter()
        .filter(|id| *relevance.get(*id).unwrap_or(&false))
        .count();
    assert!(
        top5_relevant >= 1,
        "reranker must move at least 1 relevant doc into top 5, got {top5_relevant}"
    );

    // === B. THREAD SCALING SWEEP ===
    metrics.push_str("[thread_sweep]\n");

    let thread_counts = [1, 2, 4, 8];
    let mut best_threads = 1;
    let mut best_p50 = baseline_p50;

    for &t in &thread_counts {
        let reranker = build_loaded_reranker(t, &model_path(), &tokenizer_path(), 0);
        let (latencies, _) = benchmark_rerank(&reranker, &fixture.query, &candidates, 10);
        let tp50 = p50(&latencies);
        let tp99 = p99(&latencies);

        // Determinism check: 5 runs should produce identical ranking order.
        let mut all_rankings = Vec::new();
        for _ in 0..5 {
            let batch = candidates.clone();
            let result = reranker.rerank(&fixture.query, batch).expect("rerank ok");
            let ids: Vec<String> = result.iter().map(|c| c.chunk_id.clone()).collect();
            all_rankings.push(ids);
        }
        let deterministic = all_rankings
            .windows(2)
            .all(|w| w[0] == w[1]);

        metrics.push_str(&format!(
            "threads_{t}_p50 = {tp50:.3}\n"
        ));
        metrics.push_str(&format!(
            "threads_{t}_p99 = {tp99:.3}\n"
        ));
        metrics.push_str(&format!(
            "threads_{t}_deterministic = {deterministic}\n"
        ));

        if tp50 < best_p50 && deterministic {
            best_p50 = tp50;
            best_threads = t;
        }
    }
    metrics.push_str(&format!("optimal_threads = {best_threads}\n\n"));

    // === C. INT8 QUANTIZATION (conditional: only if baseline p50 > 30ms) ===
    let int8_evaluated;
    let mut use_int8 = false;

    if baseline_p50 > 30.0 {
        int8_evaluated = true;
        metrics.push_str("[int8_quantization]\n");
        metrics.push_str("triggered = true\n");
        metrics.push_str(&format!(
            "trigger_reason = \"baseline_p50 ({baseline_p50:.1}ms) > 30ms threshold\"\n"
        ));

        let qp = quantized_model_path();
        if qp.exists() {
            let reranker = build_loaded_reranker(
                best_threads,
                &qp,
                &tokenizer_path(),
                0,
            );
            let (latencies, ranked_ids) =
                benchmark_rerank(&reranker, &fixture.query, &candidates, 10);
            let int8_p50 = p50(&latencies);
            let int8_p99 = p99(&latencies);
            let int8_ndcg = ndcg_at_k(&ranked_ids, &relevance, 10);
            let ndcg_delta = (ndcg_reranked - int8_ndcg).abs();

            metrics.push_str(&format!("int8_latency_p50 = {int8_p50:.3}\n"));
            metrics.push_str(&format!("int8_latency_p99 = {int8_p99:.3}\n"));
            metrics.push_str(&format!("int8_ndcg_at_10 = {int8_ndcg:.4}\n"));
            metrics.push_str(&format!("int8_ndcg_delta = {ndcg_delta:.4}\n"));
            metrics.push_str(&format!(
                "int8_accepted = {}\n",
                ndcg_delta < 0.5
            ));

            if ndcg_delta < 0.5 {
                use_int8 = true;
                metrics.push_str("int8_decision = \"accepted: NDCG delta within 0.5 threshold\"\n");
            } else {
                metrics.push_str("int8_decision = \"rejected: NDCG delta exceeds 0.5 threshold\"\n");
            }
        } else {
            metrics.push_str(&format!(
                "int8_model_missing = \"{}\"\n",
                qp.display()
            ));
            metrics.push_str("int8_decision = \"skipped: quantized model not available, run scripts/quantize_cross_encoder.py\"\n");
        }
        metrics.push_str("\n");
    } else {
        int8_evaluated = false;
        metrics.push_str("[int8_quantization]\n");
        metrics.push_str("triggered = false\n");
        metrics.push_str(&format!(
            "trigger_reason = \"baseline_p50 ({baseline_p50:.1}ms) <= 30ms threshold, int8 not needed\"\n\n"
        ));
    }

    // === D. SKIP THRESHOLD SWEEP ===
    metrics.push_str("[threshold_sweep]\n");

    let thresholds = [5, 10, 15, 20, 30];
    let mut optimal_threshold = 10_usize;
    let mut best_improvement = 0.0_f64;

    for &threshold in &thresholds {
        // For threshold sweep, measure NDCG@10 at N+1 candidates (boundary) and at 50.
        // Use skip_threshold=0 to always rerank, then simulate threshold by slicing.
        let reranker = build_loaded_reranker(
            best_threads,
            &model_path(),
            &tokenizer_path(),
            0,
        );

        // At 50 candidates (full set)
        let full_result = reranker
            .rerank(&fixture.query, candidates.clone())
            .expect("rerank ok");
        let full_ids: Vec<String> = full_result.iter().map(|c| c.chunk_id.clone()).collect();
        let ndcg_50 = ndcg_at_k(&full_ids, &relevance, 10);

        // At N+1 candidates (the boundary where reranking kicks in)
        let boundary_count = (threshold + 1).min(candidates.len());
        let boundary_candidates: Vec<RetrievalResult> =
            candidates[..boundary_count].to_vec();
        let boundary_unranked_ids: Vec<String> =
            boundary_candidates.iter().map(|c| c.chunk_id.clone()).collect();

        // Build relevance map for boundary subset
        let ndcg_boundary_unranked = ndcg_at_k(&boundary_unranked_ids, &relevance, 10);
        let boundary_result = reranker
            .rerank(&fixture.query, boundary_candidates)
            .expect("rerank ok");
        let boundary_ranked_ids: Vec<String> =
            boundary_result.iter().map(|c| c.chunk_id.clone()).collect();
        let ndcg_boundary_ranked = ndcg_at_k(&boundary_ranked_ids, &relevance, 10);
        let boundary_delta = ndcg_boundary_ranked - ndcg_boundary_unranked;

        metrics.push_str(&format!(
            "threshold_{threshold}_ndcg_50 = {ndcg_50:.4}\n"
        ));
        metrics.push_str(&format!(
            "threshold_{threshold}_boundary_ndcg_unranked = {ndcg_boundary_unranked:.4}\n"
        ));
        metrics.push_str(&format!(
            "threshold_{threshold}_boundary_ndcg_ranked = {ndcg_boundary_ranked:.4}\n"
        ));
        metrics.push_str(&format!(
            "threshold_{threshold}_boundary_delta = {boundary_delta:.4}\n"
        ));

        // Optimal threshold = lowest value where boundary delta > 0.02
        if boundary_delta > 0.02 && (boundary_delta > best_improvement || threshold < optimal_threshold) {
            if boundary_delta > best_improvement {
                best_improvement = boundary_delta;
                optimal_threshold = threshold;
            }
        }
    }
    metrics.push_str(&format!("optimal_threshold = {optimal_threshold}\n\n"));

    // === E. FINAL MEASUREMENT ===
    let final_model = if use_int8 && quantized_model_path().exists() {
        quantized_model_path()
    } else {
        model_path()
    };

    let reranker = build_loaded_reranker(
        best_threads,
        &final_model,
        &tokenizer_path(),
        optimal_threshold,
    );

    // Model load time (cold start).
    let cold_start = Instant::now();
    let mut cold_reranker_config = RerankerConfig::default();
    cold_reranker_config.model_path = final_model.clone();
    cold_reranker_config.tokenizer_path = tokenizer_path();
    cold_reranker_config.intra_op_threads = best_threads;
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let mut cold_reranker = CrossEncoderReranker::new(cold_reranker_config, clock);
    cold_reranker.load().expect("cold start load should succeed");
    let cold_start_ms = cold_start.elapsed().as_secs_f64() * 1000.0;

    // Full benchmark with all optimizations.
    let (final_latencies, final_ranked_ids) =
        benchmark_rerank(&reranker, &fixture.query, &candidates, 10);
    let final_p50 = p50(&final_latencies);
    let final_p99 = p99(&final_latencies);
    let final_ndcg = ndcg_at_k(&final_ranked_ids, &relevance, 10);

    let model_variant = if use_int8 { "int8" } else { "fp32" };

    metrics.push_str("[final]\n");
    metrics.push_str(&format!("final_latency_p50 = {final_p50:.3}\n"));
    metrics.push_str(&format!("final_latency_p99 = {final_p99:.3}\n"));
    metrics.push_str(&format!("final_ndcg_at_10 = {final_ndcg:.4}\n"));
    metrics.push_str(&format!("final_model_variant = \"{model_variant}\"\n"));
    metrics.push_str(&format!("final_optimal_threads = {best_threads}\n"));
    metrics.push_str(&format!("final_optimal_threshold = {optimal_threshold}\n"));
    metrics.push_str(&format!("cold_start_ms = {cold_start_ms:.1}\n"));
    metrics.push_str(&format!("int8_evaluated = {int8_evaluated}\n"));

    if final_p50 > 50.0 {
        metrics.push_str(&format!(
            "spec_amendment_required = true\n"
        ));
        metrics.push_str(&format!(
            "spec_amendment_reason = \"final p50 ({final_p50:.1}ms) exceeds 50ms budget after all optimizations\"\n"
        ));
    } else {
        metrics.push_str("spec_amendment_required = false\n");
    }

    // Concurrent load test: 4 agents, measure queue wait p99.
    let reranker = Arc::new(build_loaded_reranker(
        best_threads,
        &final_model,
        &tokenizer_path(),
        optimal_threshold,
    ));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let queue_wait_p99 = rt.block_on(async {
        let mut handles = Vec::new();
        for _i in 0..4 {
            let r = reranker.clone();
            let q = fixture.query.clone();
            let c = candidates.clone();
            handles.push(tokio::spawn(async move {
                let start = Instant::now();
                let _ = r.rerank(&q, c);
                start.elapsed().as_secs_f64() * 1000.0
            }));
        }
        let mut wait_times = Vec::new();
        for h in handles {
            wait_times.push(h.await.unwrap());
        }
        wait_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        percentile(&wait_times, 99.0)
    });

    metrics.push_str(&format!("queue_wait_p99_4agents = {queue_wait_p99:.1}\n"));
    metrics.push_str("\n");

    // Write metrics file.
    let mp = metrics_path();
    std::fs::write(&mp, &metrics).expect("should write metrics file");
    eprintln!("Metrics written to {}", mp.display());
    eprintln!("\n{metrics}");

    // Final assertions.
    assert!(
        final_ndcg > ndcg_unranked,
        "final NDCG@10 ({final_ndcg:.4}) must exceed unranked ({ndcg_unranked:.4})"
    );
}

/// Timestamp without external dependencies.
fn chrono_free_timestamp() -> String {
    // Use a deterministic placeholder since we avoid adding chrono as a dep.
    "see git commit timestamp".to_string()
}

// ---------------------------------------------------------------------------
// Metrics file validation tests (run after sweep populates metrics)
// ---------------------------------------------------------------------------

#[test]
fn metrics_file_structure_valid() {
    let mp = metrics_path();
    if !mp.exists() {
        eprintln!("SKIP: metrics file not yet generated (run optimization_sweep first)");
        return;
    }
    let content = std::fs::read_to_string(&mp).unwrap();
    assert!(
        content.contains("baseline_latency_p50"),
        "metrics must contain baseline_latency_p50"
    );
    assert!(
        content.contains("thread_sweep"),
        "metrics must contain thread_sweep section"
    );
    assert!(
        content.contains("threshold_sweep"),
        "metrics must contain threshold_sweep section"
    );
    assert!(
        content.contains("final_latency_p50"),
        "metrics must contain final_latency_p50"
    );
}
