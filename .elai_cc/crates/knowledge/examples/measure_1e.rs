use std::path::PathBuf;
use std::time::Instant;

use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::pipeline::QueryPipeline;

const QUERIES: &[&str] = &[
    "How does the FSM handle state transitions?",
    "BM25 query latency",
    "What implements the LlmProvider trait?",
    "Error handling in the orchestrator",
    "How are tools registered and dispatched?",
];

fn main() {
    let project_root = match std::env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().expect("cannot determine current directory"),
    };

    let source_dir = project_root.join("crates");
    if !source_dir.is_dir() {
        eprintln!("error: source directory not found: {}", source_dir.display());
        std::process::exit(1);
    }

    // Build BM25 index in tempdir
    let tmp = tempfile::tempdir().expect("cannot create tempdir");
    let project_id = "measure_1e";

    let mut bm25 =
        Bm25Index::new(tmp.path(), project_id).expect("Bm25Index::new failed");

    let build_start = Instant::now();
    let stats = bm25
        .index_directory(&source_dir)
        .expect("index_directory failed");
    let index_build_time_ms = build_start.elapsed().as_millis() as u64;

    let files_indexed = stats.files_indexed;
    let chunks_indexed = stats.chunks_indexed;

    eprintln!(
        "indexed {} files, {} chunks in {}ms",
        files_indexed, chunks_indexed, index_build_time_ms
    );

    // Construct pipeline
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    let pipeline = QueryPipeline::new(bm25, catrag, arag);

    // Warm-up query (discarded)
    let _ = pipeline.query("warmup");

    // Run 5 queries, collect latencies and per-query token savings
    let mut latencies_ms: Vec<u64> = Vec::with_capacity(QUERIES.len());
    let mut per_query_savings: Vec<f64> = Vec::with_capacity(QUERIES.len());
    let mut total_overlay_size: usize = 0;
    let mut skipped_stages: Vec<&str> = Vec::new();
    let mut arag_degraded = false;

    for (i, q) in QUERIES.iter().enumerate() {
        let t = Instant::now();
        let stages = pipeline.query(q).unwrap_or_else(|e| {
            eprintln!("query {} failed: {}", i, e);
            std::process::exit(1);
        });
        let elapsed_ms = t.elapsed().as_millis() as u64;
        latencies_ms.push(elapsed_ms);

        // Detect skipped/degraded stages (first query only for stage list)
        if i == 0 {
            for stage in &stages {
                match stage.stage.as_str() {
                    "vector" if stage.results.is_empty() => {
                        eprintln!("vector stage skipped - no vector index");
                        skipped_stages.push("vector");
                    }
                    "catrag" if stage.results.is_empty() => {
                        eprintln!(
                            "catrag density guard triggered - empty overlay, passthrough"
                        );
                        skipped_stages.push("catrag");
                    }
                    "arag" if stage.results.is_empty() => {
                        eprintln!(
                            "arag.keyword_search degraded to empty - \
                             token savings derived from BM25 stage only"
                        );
                        arag_degraded = true;
                        skipped_stages.push("arag");
                    }
                    _ => {}
                }
            }
        }

        // catrag overlay size: 0 when density guard triggers (results empty)
        let catrag_stage = stages.iter().find(|s| s.stage == "catrag");
        if let Some(cs) = catrag_stage {
            total_overlay_size += cs.results.len();
        }

        // Token savings from BM25 stage
        let bm25_stage = stages.iter().find(|s| s.stage == "bm25");
        if bm25_stage.is_none_or(|s| s.results.is_empty()) {
            eprintln!("Q{}: BM25 produced no results - index may be empty", i + 1);
            std::process::exit(1);
        }
        let savings = compute_token_savings_from_bm25(bm25_stage);
        per_query_savings.push(savings);

        eprintln!(
            "  Q{}: {}ms, bm25_results={}, savings={:.1}%",
            i + 1,
            elapsed_ms,
            bm25_stage.map_or(0, |s| s.results.len()),
            savings * 100.0
        );
    }

    // Compute percentiles (nearest-rank on sorted latencies)
    latencies_ms.sort();
    let n = latencies_ms.len(); // 5
    let pipeline_p50_ms = latencies_ms[n / 2]; // index 2 of 5
    let pipeline_p99_ms = latencies_ms[n - 1]; // index 4 of 5 = max

    // Compute means
    let token_savings_mean = if per_query_savings.is_empty() {
        0.0
    } else {
        per_query_savings.iter().sum::<f64>() / per_query_savings.len() as f64
    };
    let token_savings_mean_pct = token_savings_mean * 100.0;

    let catrag_overlay_size = total_overlay_size / QUERIES.len();

    // arag_tools_per_query: in degraded mode, ChunkRead is gated on non-empty
    // chunk_ids (arag.rs:171). KeywordSearch + SemanticSearch return empty,
    // so ChunkRead is skipped. Actual invocations = 2 when degraded, 3 when live.
    let arag_tools_per_query: f64 = if arag_degraded { 2.0 } else { 3.0 };

    let stages_skipped_str = if skipped_stages.is_empty() {
        "none".to_string()
    } else {
        skipped_stages.join(",")
    };

    // Write metrics TOML
    let metrics_dir = project_root.join("metrics");
    std::fs::create_dir_all(&metrics_dir).expect("cannot create metrics directory");
    let output_path = metrics_dir.join("phase_1E.toml");

    let toml_content = format!(
        "# ELAI phase 1E pipeline metrics\n\
         pipeline_p50_ms = {pipeline_p50_ms}\n\
         pipeline_p99_ms = {pipeline_p99_ms}\n\
         token_savings_mean_pct = {token_savings_mean_pct:.1}\n\
         catrag_overlay_size = {catrag_overlay_size}\n\
         arag_tools_per_query = {arag_tools_per_query:.1}\n\
         stages_skipped = \"{stages_skipped_str}\"\n\
         degraded_mode = {arag_degraded}\n\
         index_build_time_ms = {index_build_time_ms}\n\
         files_indexed = {files_indexed}\n\
         chunks_indexed = {chunks_indexed}\n"
    );

    std::fs::write(&output_path, &toml_content).expect("cannot write phase_1E.toml");

    // Validate: re-read and check all required keys
    let written =
        std::fs::read_to_string(&output_path).expect("cannot re-read phase_1E.toml");
    let required_keys = [
        "pipeline_p50_ms",
        "pipeline_p99_ms",
        "token_savings_mean_pct",
        "catrag_overlay_size",
        "arag_tools_per_query",
        "stages_skipped",
        "degraded_mode",
        "index_build_time_ms",
        "files_indexed",
        "chunks_indexed",
    ];
    let mut missing = Vec::new();
    for key in &required_keys {
        if !written
            .lines()
            .any(|line| line.starts_with(&format!("{key} =")))
        {
            missing.push(*key);
        }
    }
    if !missing.is_empty() {
        eprintln!("VALIDATION FAILED: missing keys: {missing:?}");
        std::process::exit(1);
    }

    // Print results
    println!("wrote {}", output_path.display());
    println!("---");
    print!("{toml_content}");

    // Threshold checks
    if pipeline_p50_ms < 2000 {
        println!("pipeline_p50 {}ms < 2000ms: PASS", pipeline_p50_ms);
    } else {
        println!("pipeline_p50 {}ms >= 2000ms: OVER", pipeline_p50_ms);
    }
    if token_savings_mean_pct > 50.0 {
        println!(
            "token_savings_mean {:.1}% > 50.0%: PASS",
            token_savings_mean_pct
        );
    } else {
        println!(
            "token_savings_mean {:.1}% <= 50.0%: FAIL",
            token_savings_mean_pct
        );
        std::process::exit(1);
    }
}

/// Compute token savings from BM25 stage results.
///
/// For each result, parse chunk_id ("path:start-end") to estimate full-chunk tokens,
/// then compare with snippet word count.
///
/// full_chunk_tokens = (line_end - line_start + 1) * 8 words/line
/// snippet_tokens = snippet.split_whitespace().count()
/// savings = 1.0 - (snippet_tokens / full_chunk_tokens), clamped [0.0, 1.0]
///
/// NOTE: BM25Hit.snippet is truncated to first 200 chars of chunk content
/// (bm25.rs:259). This creates a structural savings floor of ~87-94%
/// regardless of query quality. The metric validly measures "snippet vs
/// full chunk" as spec requires, but the high value reflects truncation
/// policy, not retrieval selectivity. Future phases with semantic snippets
/// will produce more meaningful savings measurements.
///
/// Returns mean savings across all results (0.0 if no results).
fn compute_token_savings_from_bm25(
    bm25_stage: Option<&elai_knowledge::pipeline::PipelineStage>,
) -> f64 {
    let stage = match bm25_stage {
        Some(s) if !s.results.is_empty() => s,
        _ => return 0.0,
    };

    let mut total_savings = 0.0;
    let mut count = 0usize;

    for result in &stage.results {
        // chunk_id format: "path:line_start-line_end"
        let (line_start, line_end) = match parse_line_range(&result.chunk_id) {
            Some(range) => range,
            None => continue,
        };

        let full_chunk_tokens = ((line_end - line_start + 1) * 8) as f64;
        if full_chunk_tokens <= 0.0 {
            continue;
        }

        let snippet_tokens = result.snippet.split_whitespace().count() as f64;
        let savings = (1.0 - (snippet_tokens / full_chunk_tokens)).clamp(0.0, 1.0);
        total_savings += savings;
        count += 1;
    }

    if count == 0 {
        0.0
    } else {
        total_savings / count as f64
    }
}

/// Parse "path:start-end" to extract (start, end) line numbers.
fn parse_line_range(chunk_id: &str) -> Option<(u32, u32)> {
    // Find last ':' (path may contain ':' on Windows like C:\...)
    let colon_pos = chunk_id.rfind(':')?;
    let range_str = &chunk_id[colon_pos + 1..];
    let dash_pos = range_str.find('-')?;
    let start: u32 = range_str[..dash_pos].parse().ok()?;
    let end: u32 = range_str[dash_pos + 1..].parse().ok()?;
    Some((start, end))
}
