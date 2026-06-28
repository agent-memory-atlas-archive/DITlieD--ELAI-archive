/// Token-savings AND retrieval-quality measurement for the ELAI cascade.
///
/// MEMORY-BOUNDED (peak RSS well under 1GB): single text model (Granite,
/// ~121MB; GraphCodeBERT never loaded), NO DKG (the per-query
/// DkgSnapshot::from_engine materialises the whole graph and is the multi-GB
/// hog — excluded; pagerank/catrag/arag stay empty), bounded corpus, one file
/// embedded at a time.
///
/// QUALITY: scored against the repo's labelled goldset (eval/rag/queries.toml,
/// reproduced inline). Each query has expected_paths (OR-match substrings);
/// a top-K result is a HIT if its chunk_id contains any expected substring.
/// Reports recall@5, recall@10, MRR, hit@1 per query + means.
///
/// SAVINGS: delivered chunk content (REAL file lines) vs (a) the full files
/// those chunks live in, and (b) the indexed corpus.
///
/// CAVEAT: the corpus FORCE-INCLUDES every goldset expected file plus bounded
/// filler, so recall is OPTIMISTIC vs the full repo (fewer distractors). It is
/// an indicative quality signal, not the full-repo number.
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use elai_core::OsClock;
use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::degradation::NoopSsbNotifier;
use elai_knowledge::pipeline::QueryPipeline;
use elai_knowledge::reranker::{CrossEncoderReranker, RerankerConfig};
use elai_knowledge::vector::indexer::VectorIndexer;
use elai_knowledge::vector::ort_session::OrtSession;
use elai_knowledge::vector::{VectorError, VectorModule, TEXT_VEC_DIM};
use elai_knowledge::{QueryTier, ReadinessStatus};

const PROJECT_ID: &str = "tok-save";
const TOP_K: usize = 10;
const SOURCE_EXTS: &[&str] = &["rs", "py", "ts", "svelte"];

// Filler corpus size (on top of the force-included goldset files). Bounded for
// memory + time; the goldset expected files are ALWAYS included regardless.
const FILLER_FILES: usize = 250;
const FILLER_CRATE_DIRS: &[&str] =
    &["/knowledge/", "/orchestrator/", "/providers/", "/core/", "/cli/"];

/// One labelled goldset query (mirrors eval/rag/queries.toml).
struct Gold {
    id: &'static str,
    text: &'static str,
    expected: &'static [&'static str],
}

const GOLDSET: &[Gold] = &[
    Gold { id: "code-01", text: "RuntimeAdapter::stream_step", expected: &[
        "crates/providers/src/runtime/types.rs",
        "crates/providers/src/runtime/ollama.rs",
        "crates/providers/src/runtime/vllm.rs",
    ]},
    Gold { id: "code-02", text: "SCHEMA_VERSION constant", expected: &[
        "crates/orchestrator/src/ssb.rs",
    ]},
    Gold { id: "code-03", text: "fn rrf_rerank", expected: &[
        "crates/knowledge/src/pipeline.rs",
    ]},
    Gold { id: "code-04", text: "QueryTier enum variants", expected: &[
        "crates/knowledge/src/lib.rs",
    ]},
    Gold { id: "code-05", text: "ssb_db_path canonical layout", expected: &[
        "crates/dashboard-v2/src-tauri/src/state.rs",
        "crates/core/src/paths.rs",
    ]},
    Gold { id: "code-06", text: "CrossEncoderReranker with_ssb", expected: &[
        "crates/knowledge/src/reranker.rs",
    ]},
    Gold { id: "code-07", text: "fn deep_pool_cap", expected: &[
        "crates/knowledge/src/deep.rs",
        "crates/knowledge/src/pipeline.rs",
    ]},
    Gold { id: "code-08", text: "SsbActionType::DeepPoolCapped", expected: &[
        "crates/orchestrator/src/ssb.rs",
    ]},
    Gold { id: "nl-09", text: "how does the knowledge pipeline rerank candidates after RRF fusion", expected: &[
        "crates/knowledge/src/pipeline.rs",
        "crates/knowledge/src/reranker.rs",
    ]},
    Gold { id: "nl-10", text: "where is the SQLite database for SSB events stored on disk", expected: &[
        "crates/core/src/paths.rs",
        "crates/dashboard-v2/src-tauri/src/state.rs",
        "crates/orchestrator/src/ssb.rs",
    ]},
    Gold { id: "nl-11", text: "explain the difference between FAST STANDARD and DEEP query tiers", expected: &[
        "crates/knowledge/src/lib.rs",
        "crates/knowledge/src/pipeline.rs",
    ]},
    Gold { id: "nl-12", text: "how does ELAI prevent the reranker from saturating on too many candidates", expected: &[
        "crates/knowledge/src/deep.rs",
        "crates/knowledge/src/pipeline.rs",
        "crates/knowledge/src/reranker.rs",
    ]},
    Gold { id: "nl-13", text: "how does the catrag stage attach contextual prefix metadata to chunks before reranking", expected: &[
        "crates/knowledge/src/catrag.rs",
    ]},
    Gold { id: "nl-14", text: "describe the SSB row types emitted by the knowledge pipeline", expected: &[
        "crates/orchestrator/src/ssb.rs",
        "crates/knowledge/src/knowledge_ssb_sink.rs",
    ]},
    Gold { id: "nl-15", text: "why does HyDE get disabled for code-symbol queries", expected: &[
        "crates/knowledge/src/hyde.rs",
        "crates/knowledge/src/query_class.rs",
        "crates/knowledge/src/pipeline.rs",
    ]},
];

/// Text-only VectorModule (Granite). embed_code errors so GraphCodeBERT is
/// never loaded; safe because index cache=None + code_index disabled.
struct TextOnlyModule {
    text: OrtSession,
}
impl VectorModule for TextOnlyModule {
    fn embed_code(&self, _t: &str) -> Result<Vec<f32>, VectorError> {
        Err(VectorError::OrtError("code embeddings disabled".into()))
    }
    fn embed_text(&self, t: &str) -> Result<Vec<f32>, VectorError> {
        self.text.embed(t).map_err(|e| VectorError::OrtError(e.to_string()))
    }
    fn readiness(&self) -> ReadinessStatus {
        if self.text.embed("t").is_ok() { ReadinessStatus::Ready } else { ReadinessStatus::Unavailable }
    }
}

fn norm(p: &str) -> String {
    p.replace('\\', "/")
}

fn main() {
    let project_root = std::env::args().nth(1).map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let crates_dir = project_root.join("crates");
    let onnx_dir = project_root.join("models").join("onnx");
    if !crates_dir.is_dir() || !onnx_dir.is_dir() {
        eprintln!("error: crates/ or models/onnx/ missing under {}", project_root.display());
        std::process::exit(1);
    }

    // ----- Step 1: corpus = goldset files (forced) + bounded filler -----
    eprintln!("=== Step 1: building bounded corpus (goldset + filler) ===");
    let all_files = collect_source_files(&crates_dir);
    let expected_substrs: Vec<String> =
        GOLDSET.iter().flat_map(|g| g.expected.iter().map(|e| e.to_string())).collect();

    let mut corpus: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    // force-include every file matching an expected substring
    for f in &all_files {
        let n = norm(&f.to_string_lossy());
        if expected_substrs.iter().any(|e| n.contains(e.as_str())) && seen.insert(n.clone()) {
            corpus.push(f.clone());
        }
    }
    let forced = corpus.len();
    // filler from relevant crates, sorted, up to FILLER_FILES
    let mut filler: Vec<PathBuf> = all_files.iter()
        .filter(|f| {
            let n = norm(&f.to_string_lossy());
            !seen.contains(&n) && FILLER_CRATE_DIRS.iter().any(|d| n.contains(d))
        })
        .cloned().collect();
    filler.sort();
    filler.truncate(FILLER_FILES);
    for f in filler { let n = norm(&f.to_string_lossy()); if seen.insert(n) { corpus.push(f); } }
    eprintln!("  corpus: {} files ({} forced goldset + {} filler)", corpus.len(), forced, corpus.len() - forced);

    // ----- Step 2: BM25 -----
    eprintln!("=== Step 2: BM25 ===");
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut bm25 = Bm25Index::new(tmp.path(), PROJECT_ID).expect("Bm25Index::new");
    let mut bm25_chunks = 0usize;
    let t = Instant::now();
    for p in &corpus {
        if let Ok(n) = bm25.index_file(p, None) { bm25_chunks += n; }
    }
    eprintln!("  BM25: {} files, {} chunks, {}ms", corpus.len(), bm25_chunks, t.elapsed().as_millis());

    // ----- Step 3: vector index (text-only) -----
    eprintln!("=== Step 3: vector embed (text-only) ===");
    let text_session = OrtSession::new(
        &onnx_dir.join("granite-embed.onnx"),
        &onnx_dir.join("granite-embed-tokenizer.json"),
        512, TEXT_VEC_DIM,
    ).expect("granite load");
    let vm = TextOnlyModule { text: text_session };
    let mut indexer = VectorIndexer::new(TEXT_VEC_DIM);
    let t = Instant::now();
    let mut chunks_embedded = 0usize;
    for (i, p) in corpus.iter().enumerate() {
        match indexer.index_file(p, &vm, None) {
            Ok(s) => { chunks_embedded += s.chunks_embedded; }
            Err(e) => eprintln!("  WARN embed {}: {e}", p.display()),
        }
        if i % 25 == 0 {
            eprintln!("  {}/{} ({} chunks, {:.1}s)", i + 1, corpus.len(), chunks_embedded, t.elapsed().as_secs_f64());
        }
    }
    eprintln!("  vector: {} chunks, {:.1}s", chunks_embedded, t.elapsed().as_secs_f64());

    // ----- Step 4: reranker -----
    eprintln!("=== Step 4: reranker ===");
    let mut rcfg = RerankerConfig::with_project_root(&project_root);
    rcfg.model_path = onnx_dir.join("cross-encoder-l12.onnx");
    rcfg.tokenizer_path = onnx_dir.join("tokenizer.json");
    rcfg.skip_threshold = 0;
    let clock = Arc::new(OsClock);
    let notifier: Arc<dyn elai_knowledge::degradation::SsbNotifier> = Arc::new(NoopSsbNotifier);
    let mut reranker = CrossEncoderReranker::with_ssb(rcfg, clock, notifier);
    let reranker_ok = reranker.load().is_ok();
    eprintln!("  reranker_loaded={reranker_ok}");

    // ----- Step 5: pipeline (bm25 + vector + rerank; NO DKG) -----
    let pipeline = QueryPipeline::new(bm25, CatRag::new(CatRagConfig::default()), AragOrchestrator::new(AragConfig::default()))
        .with_vector_module(Box::new(vm))
        .with_vector_indexer(indexer)
        .with_cross_encoder(reranker);

    // ----- corpus token baseline -----
    let mut corpus_chars = 0usize;
    for p in &corpus { if let Ok(c) = std::fs::read_to_string(p) { corpus_chars += c.len(); } }
    let baseline_corpus_tokens = chars_to_tokens(corpus_chars);

    // ----- Step 6: run goldset, score quality + savings -----
    eprintln!("=== Step 6: scoring {} goldset queries (STANDARD) ===", GOLDSET.len());
    let mut rows: Vec<Row> = Vec::new();
    let mut q1_vec = 0usize;

    // Diagnostic mode: when ELAI_HARNESS_DIAG is set, dump per-stage ranks of
    // the expected files (even beyond TOP_K) for every query, so a MISS can be
    // traced to the stage where the expected chunk fell out of the pool.
    let diag = std::env::var("ELAI_HARNESS_DIAG").is_ok();
    // Restrict the deep dump to specific ids (comma-separated) if requested.
    let diag_ids: Vec<String> = std::env::var("ELAI_HARNESS_DIAG_IDS")
        .ok()
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    for (qi, g) in GOLDSET.iter().enumerate() {
        let stages = pipeline.query_with_tier(g.text, QueryTier::Standard)
            .unwrap_or_else(|e| { eprintln!("  {} pipeline error: {e}", g.id); std::process::exit(1); });
        if qi == 0 {
            q1_vec = stages.iter().find(|s| s.stage == "vector").map(|s| s.results.len()).unwrap_or(0);
        }
        let final_stage = stages.iter().rev().find(|s| !s.results.is_empty())
            .or_else(|| stages.iter().find(|s| s.stage == "bm25"))
            .expect("result-bearing stage");

        if diag && (diag_ids.is_empty() || diag_ids.iter().any(|d| d == g.id)) {
            eprintln!("\n[DIAG] {} text={:?}", g.id, g.text);
            // Per stage of interest, find the 1-based rank of the first expected
            // file in that stage's FULL result list.
            for sname in ["bm25", "vector", "rerank", "cross_encoder"] {
                if let Some(s) = stages.iter().find(|s| s.stage == sname) {
                    let mut found: Option<(usize, String)> = None;
                    for (rank, r) in s.results.iter().enumerate() {
                        let cid = norm(&r.chunk_id);
                        if g.expected.iter().any(|e| cid.contains(e)) {
                            found = Some((rank + 1, cid));
                            break;
                        }
                    }
                    match found {
                        Some((rank, cid)) => eprintln!("  [DIAG]   {sname:<14} expected@rank={rank} ({}) total={}", cid, s.results.len()),
                        None => eprintln!("  [DIAG]   {sname:<14} expected NOT in {} results", s.results.len()),
                    }
                }
            }
            // Show the top-5 final-stage chunk_ids so we see what won.
            for (i, r) in final_stage.results.iter().take(5).enumerate() {
                eprintln!("  [DIAG]   final#{:<2} {} (tool={})", i + 1, norm(&r.chunk_id), r.tool_used);
            }
        }

        let topk: Vec<&elai_knowledge::arag::RetrievalResult> = final_stage.results.iter().take(TOP_K).collect();

        // QUALITY: first hit rank against expected_paths (OR-match on normalised chunk_id)
        let mut first_hit_rank: Option<usize> = None;
        for (rank, r) in topk.iter().enumerate() {
            let cid = norm(&r.chunk_id);
            if g.expected.iter().any(|e| cid.contains(e)) {
                first_hit_rank = Some(rank + 1);
                break;
            }
        }
        let hit_at_1 = first_hit_rank == Some(1);
        let hit_at_5 = first_hit_rank.map(|r| r <= 5).unwrap_or(false);
        let hit_at_10 = first_hit_rank.is_some();
        let mrr = first_hit_rank.map(|r| 1.0 / r as f64).unwrap_or(0.0);

        // SAVINGS
        let mut ranges: Vec<(String, u32, u32)> = Vec::new();
        let mut unique_files: HashSet<String> = HashSet::new();
        for r in &topk {
            if let Some((f, a, b)) = parse_line_range(&r.chunk_id) {
                unique_files.insert(f.to_string());
                ranges.push((f.to_string(), a, b));
            }
        }
        let mut delivered_chars = 0usize;
        for (f, a, b) in &ranges {
            if let Ok(c) = std::fs::read_to_string(PathBuf::from(f)) {
                let lines: Vec<&str> = c.lines().collect();
                let s = (*a as usize).saturating_sub(1);
                let e = (*b as usize).min(lines.len());
                if s < e { for l in &lines[s..e] { delivered_chars += l.len() + 1; } }
            }
        }
        let delivered = chars_to_tokens(delivered_chars);
        let mut files_chars = 0usize;
        for f in &unique_files { if let Ok(c) = std::fs::read_to_string(PathBuf::from(f)) { files_chars += c.len(); } }
        let files_tok = chars_to_tokens(files_chars);
        let sav_files = if files_tok > 0 { 1.0 - delivered as f64 / files_tok as f64 } else { 0.0 };

        eprintln!("  {} hit@rank={:?} mrr={:.2} sav_files={:.0}%", g.id, first_hit_rank, mrr, sav_files * 100.0);
        rows.push(Row {
            id: g.id.to_string(), class: if g.id.starts_with("code") { "code" } else { "nl" },
            first_hit_rank, hit_at_1, hit_at_5, hit_at_10, mrr,
            delivered, files_tok, sav_files_pct: sav_files * 100.0,
        });
    }

    // ----- aggregate -----
    let n = rows.len() as f64;
    let degraded = q1_vec == 0;
    let mean = |f: &dyn Fn(&Row) -> f64| rows.iter().map(|r| f(r)).sum::<f64>() / n;
    let frac = |f: &dyn Fn(&Row) -> bool| rows.iter().filter(|r| f(r)).count() as f64 / n;
    let mrr = mean(&|r| r.mrr);
    let r_at_1 = frac(&|r| r.hit_at_1);
    let r_at_5 = frac(&|r| r.hit_at_5);
    let r_at_10 = frac(&|r| r.hit_at_10);
    let sav = mean(&|r| r.sav_files_pct);
    // class split
    let code_rows: Vec<&Row> = rows.iter().filter(|r| r.class == "code").collect();
    let nl_rows: Vec<&Row> = rows.iter().filter(|r| r.class == "nl").collect();
    let mrr_code = if code_rows.is_empty() {0.0} else { code_rows.iter().map(|r| r.mrr).sum::<f64>() / code_rows.len() as f64 };
    let mrr_nl = if nl_rows.is_empty() {0.0} else { nl_rows.iter().map(|r| r.mrr).sum::<f64>() / nl_rows.len() as f64 };

    // ----- write TOML -----
    let metrics_dir = project_root.join("metrics");
    std::fs::create_dir_all(&metrics_dir).ok();
    let out = metrics_dir.join("token_savings_real.toml");
    let mut toml = format!(
        "# ELAI retrieval quality + token-savings (bounded corpus, bm25+vector+rerank, no DKG)\n\
         degraded_mode = {degraded}\n\
         reranker_loaded = {reranker_ok}\n\
         corpus_files = {}\n\
         forced_goldset_files = {forced}\n\
         chunks_embedded = {chunks_embedded}\n\
         baseline_corpus_tokens = {baseline_corpus_tokens}\n\
         queries = {}\n\
         mrr = {mrr:.3}\n\
         recall_at_1 = {r_at_1:.3}\n\
         recall_at_5 = {r_at_5:.3}\n\
         recall_at_10 = {r_at_10:.3}\n\
         mrr_code = {mrr_code:.3}\n\
         mrr_nl = {mrr_nl:.3}\n\
         mean_savings_vs_files_pct = {sav:.2}\n\n",
        corpus.len(), rows.len()
    );
    for r in &rows {
        toml.push_str("[[query]]\n");
        toml.push_str(&format!(
            "id = \"{}\"\nclass = \"{}\"\nfirst_hit_rank = {}\nmrr = {:.3}\nhit_at_5 = {}\nhit_at_10 = {}\nsavings_vs_files_pct = {:.2}\ndelivered_tokens = {}\n\n",
            r.id, r.class, r.first_hit_rank.map(|x| x as i64).unwrap_or(-1), r.mrr, r.hit_at_5, r.hit_at_10, r.sav_files_pct, r.delivered,
        ));
    }
    std::fs::write(&out, &toml).expect("write toml");

    // ----- print -----
    println!("\n=== RETRIEVAL QUALITY + TOKEN SAVINGS (bounded, memory-safe) ===\n");
    println!("degraded_mode={degraded} (Q1 vector hits={q1_vec})  reranker_loaded={reranker_ok}");
    println!("corpus={} files ({} forced goldset + filler), {} chunks\n", corpus.len(), forced, chunks_embedded);
    println!("{:<8} {:<5} {:>9} {:>5} {:>6} {:>7} {:>11}", "id", "class", "hit_rank", "mrr", "hit@5", "hit@10", "sav_files_%");
    println!("{}", "-".repeat(62));
    for r in &rows {
        println!("{:<8} {:<5} {:>9} {:>5.2} {:>6} {:>7} {:>10.1}%",
            r.id, r.class,
            r.first_hit_rank.map(|x| x.to_string()).unwrap_or_else(|| "MISS".into()),
            r.mrr, r.hit_at_5, r.hit_at_10, r.sav_files_pct);
    }
    println!("{}", "-".repeat(62));
    println!("\nQUALITY:  MRR={mrr:.3}  recall@1={r_at_1:.2}  recall@5={r_at_5:.2}  recall@10={r_at_10:.2}");
    println!("          MRR(code)={mrr_code:.3}  MRR(nl)={mrr_nl:.3}");
    println!("SAVINGS:  mean_vs_files={sav:.1}%");
    if degraded { println!("\nWARNING: vector empty on Q1 — degraded, quality reflects bm25 only."); }
    println!("\nwrote {}", out.display());
}

struct Row {
    id: String, class: &'static str,
    first_hit_rank: Option<usize>, hit_at_1: bool, hit_at_5: bool, hit_at_10: bool, mrr: f64,
    delivered: usize, files_tok: usize, sav_files_pct: f64,
}

fn chars_to_tokens(c: usize) -> usize { (c + 3) / 4 }

fn parse_line_range(chunk_id: &str) -> Option<(&str, u32, u32)> {
    let colon = chunk_id.rfind(':')?;
    let file = &chunk_id[..colon];
    let range = &chunk_id[colon + 1..];
    let dash = range.find('-')?;
    let start: u32 = range[..dash].parse().ok()?;
    let end: u32 = range[dash + 1..].parse().ok()?;
    if start == 0 || end < start || end > 300_000 { return None; }
    Some((file, start, end))
}

fn collect_source_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_recursive(root, &mut out);
    out
}
fn collect_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = match std::fs::read_dir(dir) { Ok(r) => r, Err(_) => return };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let ns = name.to_string_lossy();
        if path.is_dir() {
            if matches!(ns.as_ref(), "target" | "node_modules" | ".git" | ".elai" | "__pycache__") { continue; }
            collect_recursive(&path, out);
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if SOURCE_EXTS.contains(&ext) { out.push(path); }
        }
    }
}
