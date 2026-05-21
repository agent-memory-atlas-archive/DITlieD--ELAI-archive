//! E-17 SP6 — RAG monotonicity contract.
//!
//! Pins the invariant that DEEP-tier retrieval never regresses below
//! STANDARD-tier retrieval on a deterministic synthetic corpus. This is
//! the in-CI sibling of the production eval harness at
//! `.elai_cc/eval/rag/run.py` (which measures the same invariant on the
//! real index but is non-deterministic between sessions / requires a
//! built index / depends on optional ONNX models being present).
//!
//! # Why >= and not >
//!
//! `pipeline.rs:813` uses `std::HashMap` for the RRF accumulator; ties
//! on score iterate in unstable order before `sort_by`. On a 12-doc
//! synthetic corpus ties are common — `>` would flake. The plan
//! invariant is "no regression"; `>=` encodes that without flake.
//! See `workflow/reasoning/E-17-SP6-decisions.md` DECISION-D4.
//!
//! # Synthetic corpus design (deterministic)
//!
//! 4 STRONGLY-RELEVANT docs    — content describes "stream step" via
//!                                RuntimeAdapter; matches the query
//!                                "stream step" + "RuntimeAdapter".
//! 4 WEAKLY-RELEVANT  docs     — content about runtime config + tier
//!                                routing (shares "runtime" but not
//!                                "stream").
//! 4 IRRELEVANT       docs     — pricing / billing prose; no overlap.
//!
//! Goldset = the 4 strongly-relevant docs (their on-disk file paths).
//! MRR is computed against this goldset.
//!
//! # What this test does NOT prove
//!
//! - cAST recall@5 — SP3 was skipped; cAST chunking is not exercised.
//! - HyDE-NL baseline — sidecar unavailable in CI; HyDE stage is None
//!   for both tiers in this test. The "use_hyde" branch in
//!   pipeline.rs is exercised at runtime by the eval harness only.
//! - Real cross-encoder rerank — ONNX models are not loaded in CI; the
//!   cross_encoder stage is None for both tiers, so the rerank diff
//!   between STANDARD and DEEP comes from query_expansion only.
//!
//! These deltas are deferred to ops follow-up per the E-17 SP6
//! decision doc. The monotonicity invariant on the limited surface
//! exercised here is still load-bearing: a regression would mean
//! pipeline.rs::query_with_tier returns FEWER hits on DEEP than on
//! STANDARD for a corpus where DEEP can at worst tie.

use std::path::Path;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::{Bm25Index, BM25Chunk, ChunkKind};
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::pipeline::QueryPipeline;
use elai_knowledge::QueryTier;
use tempfile::TempDir;

/// Strongly-relevant docs — should win at any healthy retrieval tier.
const STRONGLY_RELEVANT: &[(&str, &str)] = &[
    (
        "stream_step_adapter.rs",
        "pub fn stream_step(&self, request: StreamRequest) -> StreamResponse { \
         self.runtime_adapter.dispatch(request) } \
         // RuntimeAdapter implements StreamingTransport via stream_step.",
    ),
    (
        "runtime_adapter_stream.rs",
        "impl RuntimeAdapter { pub async fn stream_step(&mut self, frame: Frame) -> Result<Frame> { \
         self.streaming.step(frame).await } } \
         // canonical RuntimeAdapter stream_step dispatch site.",
    ),
    (
        "streaming_dispatch_stream_step.rs",
        "pub trait StreamingTransport { fn stream_step(&self, f: Frame) -> Frame; } \
         // StreamingTransport.stream_step is the per-frame entry point.",
    ),
    (
        "stream_step_trace.rs",
        "// Tracing wrapper around RuntimeAdapter::stream_step. \
         tracing::info!(target=\"runtime.stream_step\", \"adapter dispatching stream step\");",
    ),
];

/// Weakly-relevant docs — share \"runtime\" but lack \"stream\". A
/// healthy retriever should rank these BELOW the strongly-relevant set.
const WEAKLY_RELEVANT: &[(&str, &str)] = &[
    (
        "runtime_config.rs",
        "pub struct RuntimeConfig { pub backend_mode: BackendMode } \
         // RuntimeConfig is read at boot to select the backend.",
    ),
    (
        "runtime_tier_routing.rs",
        "match runtime { Runtime::Local => dispatch_local(), Runtime::Cloud => dispatch_cloud() } \
         // tier routing decides which runtime owns a request.",
    ),
    (
        "runtime_adapter_init.rs",
        "impl RuntimeAdapter { pub fn new(cfg: RuntimeConfig) -> Self { Self { cfg } } } \
         // RuntimeAdapter constructor; no streaming wiring here.",
    ),
    (
        "runtime_backend.rs",
        "pub enum BackendMode { Local, Cloud } \
         // BackendMode is part of RuntimeConfig.",
    ),
];

/// Irrelevant docs — pricing prose. Should not show up in the top-K.
const IRRELEVANT: &[(&str, &str)] = &[
    (
        "pricing_tiers.rs",
        "pub const SONNET_PRICING: f64 = 3.00; pub const HAIKU_PRICING: f64 = 0.25; \
         // model pricing constants in USD per million tokens.",
    ),
    (
        "billing_invoice.rs",
        "impl Invoice { pub fn total(&self) -> f64 { self.lines.iter().map(|l| l.amount).sum() } }",
    ),
    (
        "currency_format.rs",
        "pub fn format_usd(cents: i64) -> String { format!(\"${}.{:02}\", cents / 100, cents % 100) }",
    ),
    (
        "billing_period.rs",
        "pub struct BillingPeriod { pub start: NaiveDate, pub end: NaiveDate }",
    ),
];

/// Query exercising the monotonicity invariant. Uses 3+ words so it
/// won't be auto-routed to FAST (which skips vector/rerank entirely).
/// At STANDARD tier: BM25 + vector + reranker (vector + reranker are
/// no-ops here — no ONNX module). At DEEP tier: + query_expansion
/// + HyDE (HyDE is None here so also no-op).
const QUERY: &str = "runtime adapter stream step dispatch";

/// Goldset = the 4 strongly-relevant docs' on-disk paths. Match by
/// path substring (case-insensitive) against chunk_id and snippet.
fn goldset_matches(chunk_id: &str, snippet: &str) -> bool {
    let needle_paths = [
        "stream_step_adapter",
        "runtime_adapter_stream",
        "streaming_dispatch_stream_step",
        "stream_step_trace",
    ];
    let hay = chunk_id.to_lowercase();
    let hay_snip = snippet.to_lowercase();
    needle_paths.iter().any(|n| hay.contains(n) || hay_snip.contains(n))
}

/// Build a pipeline whose BM25 index has been populated with the
/// synthetic corpus. Tier is selected on each query, NOT on the
/// pipeline (the pipeline is tier-agnostic). Returns the pipeline
/// alongside the TempDir so the caller keeps both alive for the
/// duration of the test.
fn build_pipeline(tmp: &TempDir) -> QueryPipeline {
    let project_root = tmp.path();
    let session_id = "rag-monotonicity-contract";

    // 1) Build BM25 index over synthetic docs on disk. The synthetic
    //    files are written into a sibling `docs/` dir so the chunk
    //    paths surfacing in chunk_id are distinguishable from the
    //    tantivy index files.
    let docs_dir = project_root.join("docs");
    std::fs::create_dir_all(&docs_dir).expect("create docs dir");
    let mut bm25 = Bm25Index::new(project_root, session_id).expect("build bm25 index");
    write_and_index(&mut bm25, &docs_dir, STRONGLY_RELEVANT);
    write_and_index(&mut bm25, &docs_dir, WEAKLY_RELEVANT);
    write_and_index(&mut bm25, &docs_dir, IRRELEVANT);

    // 2) CatRag is a thin overlay that needs a DkgSnapshot to actually
    //    contribute. Without a DKG it's a no-op pass-through (same as
    //    bench_rag in offline mode). Default config is fine.
    let catrag = CatRag::new(CatRagConfig::default());

    // 3) AragOrchestrator with no chunks loaded -> arag stage returns
    //    empty results when use_vector is true (no embedder either).
    //    The bm25 stage is the only "real" stage in this test.
    let arag = AragOrchestrator::new(AragConfig::default());

    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    QueryPipeline::new(bm25, catrag, arag).with_clock(clock)
}

fn write_and_index(bm25: &mut Bm25Index, docs_dir: &Path, docs: &[(&str, &str)]) {
    for (name, body) in docs {
        let path = docs_dir.join(name);
        std::fs::write(&path, body).expect("write synthetic doc");
        // Custom chunks (one per file) keep the index deterministic
        // and avoid AST chunkers tripping on the synthetic .rs-suffixed
        // prose (their Rust syntax is intentionally informal).
        let chunk = BM25Chunk {
            path: path.clone(),
            content: (*body).to_string(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::File,
        };
        bm25.index_file(&path, Some(vec![chunk]))
            .expect("index synthetic chunk");
    }
}

/// Compute MRR for the goldset on a final pipeline stage's results.
/// Returns 0.0 if no goldset doc is in the top_k.
fn mrr_at(results: &[elai_knowledge::arag::RetrievalResult], top_k: usize) -> f64 {
    for (idx, r) in results.iter().take(top_k).enumerate() {
        if goldset_matches(&r.chunk_id, &r.snippet) {
            return 1.0 / ((idx + 1) as f64);
        }
    }
    0.0
}

/// Recall@K for the goldset on a final pipeline stage's results.
fn recall_at(results: &[elai_knowledge::arag::RetrievalResult], top_k: usize) -> f64 {
    let goldset_size = STRONGLY_RELEVANT.len() as f64;
    let hits = results
        .iter()
        .take(top_k)
        .filter(|r| goldset_matches(&r.chunk_id, &r.snippet))
        .count() as f64;
    hits / goldset_size
}

/// Drive a query at the requested tier and return the LAST
/// result-bearing stage (mirrors bench_rag::collect_final_results so
/// the contract and the harness use the same MRR surface).
fn final_results(
    pipeline: &QueryPipeline,
    tier: QueryTier,
) -> Vec<elai_knowledge::arag::RetrievalResult> {
    let stages = pipeline
        .query_with_tier(QUERY, tier)
        .expect("query_with_tier must succeed on synthetic corpus");
    let last = stages.iter().rev().find(|s| !s.results.is_empty());
    last.map(|s| s.results.clone()).unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn monotonicity_deep_geq_standard_on_synthetic_corpus() {
    let tmp = TempDir::new().expect("tempdir");
    let pipeline = build_pipeline(&tmp);
    let top_k = 10;

    let standard = final_results(&pipeline, QueryTier::Standard);
    let deep = final_results(&pipeline, QueryTier::Deep);

    let standard_mrr = mrr_at(&standard, top_k);
    let deep_mrr = mrr_at(&deep, top_k);

    assert!(
        deep_mrr >= standard_mrr,
        "DEEP MRR regression on synthetic corpus: standard={:.4} deep={:.4} delta={:+.4}; \
         SP2 invariant — investigate deep_pool_cap + rrf_quality_gate + 3-ranker RRF weights",
        standard_mrr,
        deep_mrr,
        deep_mrr - standard_mrr
    );

    // A passing run with both tiers returning ZERO results would
    // trivially satisfy >=; guard against that to prove the harness
    // is actually finding the goldset.
    assert!(
        standard_mrr > 0.0 || deep_mrr > 0.0,
        "both STANDARD and DEEP returned zero goldset hits on the synthetic corpus; \
         the BM25 index is mis-wired (query = {:?})",
        QUERY,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn standard_mrr_meets_soft_floor_on_synthetic_corpus() {
    // SOFT floor: 0.30 on the synthetic corpus (DECISION-D4). The
    // production target is 0.55 but only against the real ELAI repo
    // corpus where 700+ source files give the BM25 retriever real
    // signal-to-noise. 0.30 catches a wholesale-broken pipeline
    // (e.g. BM25 returning empty), not a tuning regression.
    const SOFT_FLOOR: f64 = 0.30;
    let tmp = TempDir::new().expect("tempdir");
    let pipeline = build_pipeline(&tmp);
    let standard = final_results(&pipeline, QueryTier::Standard);
    let standard_mrr = mrr_at(&standard, 10);
    assert!(
        standard_mrr >= SOFT_FLOOR,
        "STANDARD MRR {standard_mrr:.4} < soft floor {SOFT_FLOOR}; \
         synthetic corpus retrieval is broken at the BM25 layer."
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deep_returns_at_least_one_goldset_hit_at_recall_5() {
    // Recall@5 on the goldset must be > 0.0 at DEEP tier. Stronger
    // than monotonicity — proves DEEP actually finds AT LEAST one of
    // the 4 strongly-relevant docs in its top 5. Mirrors the E-17
    // spec's "HyDE-NL recall@5 >= baseline" assertion in shape, even
    // though HyDE itself is None in this test.
    let tmp = TempDir::new().expect("tempdir");
    let pipeline = build_pipeline(&tmp);
    let deep = final_results(&pipeline, QueryTier::Deep);
    let r_at_5 = recall_at(&deep, 5);
    assert!(
        r_at_5 > 0.0,
        "DEEP recall@5 = 0.0 on synthetic corpus; pipeline returned NO goldset doc \
         in top 5 — this is a strict regression below the no-retrieval baseline."
    );
}
