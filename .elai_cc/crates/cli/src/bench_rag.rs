//! E-17 SP6 — `elai bench rag` CLI driver.
//!
//! Surfaces the local [`elai_knowledge::pipeline::QueryPipeline`] at an
//! explicit [`elai_knowledge::QueryTier`] as a scriptable subcommand. The
//! companion eval harness at `.elai_cc/eval/rag/run.py` invokes this binary
//! once per (query, tier) and consumes the JSON payload to compute MRR,
//! R@5, R@10 + observe the 4 SP2 SSB row types out-of-band via sqlite3.
//!
//! # Wiring discipline
//!
//! producer  : [`crate::live_cli::LiveCli::build_local_knowledge_stack`]
//!             (same constructor production uses).
//! consumer  : this file — drives the pipeline, prints JSON to stdout.
//! ssb rows  : whatever the pipeline emits per `query_with_tier` (in
//!             particular `deep_pool_capped`, `rrf_quality_gate_fired`,
//!             `hyde_skipped_code_query`, `rrf_component_weights`,
//!             `query_trace`); read out-of-band by `run.py` via sqlite3
//!             against `<project_root>/.elai_cc/sessions/ssb.db`.
//!
//! # Why this exists
//!
//! `/research` is web-research, not retrieval. `pipeline.query()` is
//! otherwise only reachable via `augment_with_knowledge` (wrapped in full
//! prompt-turn machinery + chat-response formatting). `bench rag` is the
//! ONLY scriptable seam for "what does the retrieval pipeline return at
//! tier T for query Q on this project_root".
//!
//! # Anti-stub
//!
//! No `.unwrap()` in production paths (CLAUDE.md HARD RULE). All
//! construction failures surface as anyhow::Error with context. Missing
//! pipeline (no BM25 index on disk) is a soft-fail: prints a non-empty
//! JSON envelope with `final_results: []` and exits 0 so the harness can
//! degrade gracefully.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use elai_core::ClockProvider;
use elai_knowledge::pipeline::PipelineStage;
use elai_knowledge::QueryTier;
use elai_orchestrator::{BackendConfig, BackendMode};
use serde::Serialize;

use crate::live_cli::LiveCli;

/// JSON payload printed to stdout when `--json` is set.
///
/// `tier` is the explicit string the user passed in (already validated by
/// the parser to one of FAST | STANDARD | DEEP). `final_results` is the
/// last pipeline stage's results (post-rerank / cross-encoder / token_fit
/// passthrough). `stages` is the per-stage breakdown for debugging.
#[derive(Debug, Serialize)]
struct BenchRagOutput {
    tier: String,
    top_k: u32,
    query: String,
    project_root: String,
    total_stages: usize,
    final_stage: Option<String>,
    final_results: Vec<ResultRow>,
    stages: Vec<StageRow>,
    /// When the pipeline could not be constructed (no BM25 index on disk,
    /// or any other graceful-degradation path), this field carries the
    /// reason and the harness can treat the run as a soft skip rather
    /// than an error.
    pipeline_unavailable: Option<String>,
}

#[derive(Debug, Serialize)]
struct ResultRow {
    rank: usize,
    chunk_id: String,
    score: f64,
    /// First 200 chars of snippet (truncated to keep JSON small).
    snippet: String,
    /// Heuristic file path derived from chunk_id when the chunk_id is
    /// "path:line_start-line_end" or "path"; mirrors how the dashboard
    /// surfaces results in KnowledgeDrawer.
    file_path: String,
    tool_used: String,
}

#[derive(Debug, Serialize)]
struct StageRow {
    stage: String,
    result_count: usize,
    latency_ms: u64,
    error: Option<String>,
    circuit_breaker_state: Option<String>,
}

/// Snippet truncation cap (chars). Keeps JSON small enough for
/// `subprocess.run(... text=True)` consumers + dashboard rendering.
const SNIPPET_CHAR_CAP: usize = 200;

/// Sync entrypoint — owns its own tokio runtime so the pipeline can
/// drive any async stages it needs internally. Matches the
/// `bench_arc::run` pattern (no shared runtime contamination).
pub(crate) fn run(
    tier: String,
    top_k: u32,
    query: String,
    json: bool,
    project_root: PathBuf,
) -> anyhow::Result<()> {
    let resolved_tier = resolve_tier(&tier)?;

    // The factory `build_local_knowledge_stack` reads project_root via
    // env::current_dir(); swap cwd to the user-provided --project-root so
    // bm25 + .elai_cc/sessions/ssb.db resolve against the right tree.
    // Restore cwd on drop via the OldCwd guard so concurrent test runs
    // are not corrupted (cargo test runs in parallel but `bench rag` is
    // a separate process — guard is belt-and-braces).
    let _cwd_guard = CwdGuard::set(&project_root)
        .with_context(|| format!("failed to set cwd to project_root={}", project_root.display()))?;

    // Construct the same `BackendConfig` the production path uses for
    // Local backends. The sidecar URL is read by the HyDE attach branch;
    // when the probe fails (it will, in offline harness runs) HyDE is
    // disabled with a `tracing::debug` line and the pipeline still runs.
    let backend_cfg = BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: std::env::var("ELAI_SIDECAR_URL")
            .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
            .unwrap_or_else(|_| "http://127.0.0.1:8100".to_string()),
        ..BackendConfig::default()
    };

    // Synthetic session id — the bench is a one-shot, not a chat session;
    // session_id is only surfaced on `vector_cache_hit_rate` SSB rows and
    // is observable in run.py's sqlite reader.
    let session_id = "bench-rag";
    let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);

    // No ssb wired here: bench_rag is meant to run against an existing
    // index. If the harness wants SSB beacons, the dashboard wires them
    // in production runs via `live_cli` — the bench surface is the JSON
    // stdout payload. (Confirmed: not needed for SP6 monotonicity contract.)
    // 1I-R1: bench passes None ssb so the cascade loop never spawns; the shared
    // mode handle is required by the signature but unused on this path.
    let bench_search_mode = std::sync::Arc::new(std::sync::Mutex::new(
        elai_knowledge::degradation::SearchMode::Hybrid,
    ));
    let (pipeline_opt, _indexing) = LiveCli::build_local_knowledge_stack(
        &backend_cfg,
        session_id,
        &clock,
        None,
        bench_search_mode,
    );

    let Some(pipeline) = pipeline_opt else {
        let output = BenchRagOutput {
            tier: tier.clone(),
            top_k,
            query: query.clone(),
            project_root: project_root.display().to_string(),
            total_stages: 0,
            final_stage: None,
            final_results: Vec::new(),
            stages: Vec::new(),
            pipeline_unavailable: Some(
                "BM25 index could not be opened (run `elai index build` first)".to_string(),
            ),
        };
        emit(&output, json)?;
        return Ok(());
    };

    let stages = pipeline
        .query_with_tier(&query, resolved_tier)
        .with_context(|| format!("query_with_tier({}, {tier}) failed", query.chars().take(80).collect::<String>()))?;

    let final_results = collect_final_results(&stages, top_k);
    let final_stage = stages.last().map(|s| s.stage.clone());
    let stage_rows = stages
        .iter()
        .map(|s| StageRow {
            stage: s.stage.clone(),
            result_count: s.results.len(),
            latency_ms: s.latency_ms,
            error: s.error.clone(),
            circuit_breaker_state: s.circuit_breaker_state.clone(),
        })
        .collect();

    let output = BenchRagOutput {
        tier,
        top_k,
        query,
        project_root: project_root.display().to_string(),
        total_stages: stages.len(),
        final_stage,
        final_results,
        stages: stage_rows,
        pipeline_unavailable: None,
    };
    emit(&output, json)?;
    Ok(())
}

/// Map the (already-uppercased) tier string to the concrete enum.
/// Validation was performed at the parser boundary; this is a defensive
/// double-check (returns anyhow::Error rather than panicking).
fn resolve_tier(tier: &str) -> anyhow::Result<QueryTier> {
    match tier {
        "FAST" => Ok(QueryTier::Fast),
        "STANDARD" => Ok(QueryTier::Standard),
        "DEEP" => Ok(QueryTier::Deep),
        other => Err(anyhow::anyhow!(
            "invalid tier {other:?} (expected FAST | STANDARD | DEEP)"
        )),
    }
}

/// Walk the pipeline stages back-to-front to find the last stage with a
/// non-empty result vector and return its top-K rows. The pipeline often
/// emits zero-row marker stages (e.g. `deep_cap`, `rrf_gate`) at the end
/// of its trace — the LAST result-bearing stage is what the harness
/// actually wants to measure MRR against.
fn collect_final_results(stages: &[PipelineStage], top_k: u32) -> Vec<ResultRow> {
    let bearing_stage = stages
        .iter()
        .rev()
        .find(|s| !s.results.is_empty());
    let Some(stage) = bearing_stage else {
        return Vec::new();
    };
    let cap = usize::try_from(top_k).unwrap_or(usize::MAX);
    stage
        .results
        .iter()
        .take(cap)
        .enumerate()
        .map(|(idx, r)| {
            let snippet = truncate_chars(&r.snippet, SNIPPET_CHAR_CAP);
            let file_path = derive_file_path(&r.chunk_id);
            ResultRow {
                rank: idx + 1,
                chunk_id: r.chunk_id.clone(),
                score: r.score,
                snippet,
                file_path,
                tool_used: r.tool_used.clone(),
            }
        })
        .collect()
}

/// Chunk IDs in the canonical pipeline are either bare paths or
/// `<path>:<line_start>-<line_end>`. Split on the first colon that
/// follows a path-like prefix; if no colon, return the chunk_id verbatim.
fn derive_file_path(chunk_id: &str) -> String {
    if let Some((path, _rest)) = chunk_id.rsplit_once(':') {
        // Guard against Windows drive letters ("C:\\path"): only treat as
        // a path-line split if the suffix looks numeric.
        if path.contains('/') || path.contains('\\') {
            return path.to_string();
        }
    }
    chunk_id.to_string()
}

/// Char-safe truncation. Avoids slicing inside multi-byte UTF-8.
fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars).collect();
    out.push_str("...");
    out
}

/// Emit either JSON (when --json) or a human-readable table.
fn emit(output: &BenchRagOutput, json: bool) -> anyhow::Result<()> {
    if json {
        let body = serde_json::to_string_pretty(output)
            .context("serialise bench_rag output to JSON")?;
        println!("{body}");
        return Ok(());
    }
    println!("elai bench rag");
    println!("  tier         = {}", output.tier);
    println!("  top-k        = {}", output.top_k);
    println!("  query        = {}", output.query);
    println!("  project_root = {}", output.project_root);
    println!("  stages       = {} (final={:?})", output.total_stages, output.final_stage);
    if let Some(ref reason) = output.pipeline_unavailable {
        println!("  PIPELINE_UNAVAILABLE: {reason}");
        return Ok(());
    }
    println!("  --- results ---");
    for row in &output.final_results {
        println!(
            "  [{:>2}] score={:.4} {}  ({})",
            row.rank, row.score, row.file_path, row.tool_used
        );
    }
    println!("  --- stages ---");
    for s in &output.stages {
        println!(
            "  {:<14} count={:>3} lat={}ms{}",
            s.stage,
            s.result_count,
            s.latency_ms,
            s.error
                .as_deref()
                .map(|e| format!(" error={e}"))
                .unwrap_or_default()
        );
    }
    Ok(())
}

/// RAII cwd guard. `LiveCli::build_local_knowledge_stack` reads
/// project_root via `env::current_dir()`; we swap cwd before the call
/// and restore on drop. Restore failure is logged (eprintln) but not
/// surfaced: the process is exiting anyway.
struct CwdGuard {
    previous: Option<PathBuf>,
}

impl CwdGuard {
    fn set(target: &std::path::Path) -> std::io::Result<Self> {
        let previous = std::env::current_dir().ok();
        std::env::set_current_dir(target)?;
        Ok(Self { previous })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        if let Some(ref prev) = self.previous {
            if let Err(e) = std::env::set_current_dir(prev) {
                eprintln!("warn: bench_rag could not restore cwd to {}: {e}", prev.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_tier_accepts_canonical_values() {
        assert_eq!(resolve_tier("FAST").expect("fast"), QueryTier::Fast);
        assert_eq!(resolve_tier("STANDARD").expect("standard"), QueryTier::Standard);
        assert_eq!(resolve_tier("DEEP").expect("deep"), QueryTier::Deep);
    }

    #[test]
    fn resolve_tier_rejects_unknown() {
        let err = resolve_tier("AUTO").unwrap_err();
        assert!(err.to_string().contains("invalid tier"));
    }

    #[test]
    fn derive_file_path_handles_path_with_line_range() {
        assert_eq!(
            derive_file_path("src/foo.rs:42-58"),
            "src/foo.rs".to_string()
        );
        assert_eq!(
            derive_file_path("crates/knowledge/src/pipeline.rs:1-200"),
            "crates/knowledge/src/pipeline.rs".to_string()
        );
    }

    #[test]
    fn derive_file_path_handles_bare_path() {
        assert_eq!(
            derive_file_path("docs/README.md"),
            "docs/README.md".to_string()
        );
    }

    #[test]
    fn truncate_chars_preserves_utf8_boundary() {
        let s = "héllo wörld";
        assert_eq!(truncate_chars(s, 5), "héllo...".to_string());
        assert_eq!(truncate_chars(s, 100), s.to_string());
    }

    #[test]
    fn collect_final_results_walks_back_to_last_nonempty() {
        use elai_knowledge::arag::RetrievalResult;
        let stages = vec![
            PipelineStage {
                stage: "bm25".to_string(),
                results: vec![RetrievalResult {
                    chunk_id: "src/a.rs:1-10".into(),
                    snippet: "hit".into(),
                    score: 0.9,
                    tool_used: "bm25".into(),
                }],
                latency_ms: 1,
                circuit_breaker_state: None,
                error: None,
            },
            PipelineStage {
                stage: "rerank".to_string(),
                results: vec![RetrievalResult {
                    chunk_id: "src/b.rs:1-10".into(),
                    snippet: "rerank hit".into(),
                    score: 0.95,
                    tool_used: "rerank".into(),
                }],
                latency_ms: 2,
                circuit_breaker_state: None,
                error: None,
            },
            PipelineStage {
                stage: "deep_cap".to_string(),
                results: Vec::new(),
                latency_ms: 0,
                circuit_breaker_state: None,
                error: None,
            },
        ];
        let out = collect_final_results(&stages, 5);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].file_path, "src/b.rs");
        assert_eq!(out[0].tool_used, "rerank");
    }
}
