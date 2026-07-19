/// A/B retrieval-quality measurement: CodeRankEmbed vs GraphCodeBERT
/// on CPU ONNX Runtime (CPUExecutionProvider only, no GPU).
///
/// Loads BOTH `models/onnx/coderankembed.onnx` and
/// `models/onnx/graphcodebert.onnx` via `OrtSession::new(..., 512, 768)`,
/// indexes the SAME bounded corpus twice (document path = `embed_code`,
/// NO query prefix), runs the ELAI goldset with the query path
/// (`embed_code_query`, which prepends `CODE_RANK_EMBED_QUERY_PREFIX` for
/// the CodeRank lane and the same call shape for the GraphCodeBERT lane
/// so the A/B is honest), and prints side-by-side recall@5 / recall@10 /
/// MRR (+ optional p50/p95 latency) labelled EXACTLY:
///   onnx-cpu:coderankembed
///   onnx-cpu:graphcodebert
///
/// Skip behaviour: if EITHER ONNX asset (model or tokenizer) is absent,
/// prints a clear SKIP with the reason and exits 0 (the example still
/// compiles). This keeps the harness green on hosts that have not staged
/// the assets via `stage_onnx_models.py --only coderankembed`.
///
/// Zero local GPU: CPUExecutionProvider / existing OrtSession path only.
/// Never llama.cpp / Vulkan / local Qwen / api-free.
///
/// Run: cargo run -p elai-knowledge --example measure_code_embed_ab
///      cargo run -p elai-knowledge --example measure_code_embed_ab -- /path/to/repo
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::vector::indexer::VectorIndexer;
use elai_knowledge::vector::ort_session::OrtSession;
use elai_knowledge::vector::{VectorError, VectorModule, CODE_VEC_DIM};
use elai_knowledge::ReadinessStatus;

const PROJECT_ID: &str = "code-ab";
const TOP_K: usize = 10;
const SOURCE_EXTS: &[&str] = &["rs", "py", "ts", "svelte"];

const FILLER_FILES: usize = 250;
const FILLER_CRATE_DIRS: &[&str] =
    &["/knowledge/", "/orchestrator/", "/providers/", "/core/", "/cli/"];

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

/// Single-session VectorModule: both `embed_code` and `embed_text` route
/// to the same ONNX session. The A/B is code-only (document path uses
/// `embed_code`, query path uses `embed_code_query` which prepends the
/// CodeRankEmbed prefix). `embed_text` is never called on the document
/// path by `VectorIndexer::index_file` (it uses `embed_text` for the
/// primary text index and `embed_code` for the secondary code index);
/// this module routes both to the same session so the A/B measures the
/// code-embed model in isolation, independent of the text lane.
struct CodeEmbedModule {
    session: OrtSession,
}
impl VectorModule for CodeEmbedModule {
    fn embed_code(&self, text: &str) -> Result<Vec<f32>, VectorError> {
        self.session
            .embed(text)
            .map_err(|e| VectorError::OrtError(e.to_string()))
    }
    fn embed_text(&self, text: &str) -> Result<Vec<f32>, VectorError> {
        self.session
            .embed(text)
            .map_err(|e| VectorError::OrtError(e.to_string()))
    }
    fn readiness(&self) -> ReadinessStatus {
        if self.session.embed("fn main()").is_ok() {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::Unavailable
        }
    }
}

fn norm(p: &str) -> String {
    p.replace('\\', "/")
}

struct LaneResult {
    label: &'static str,
    chunks_embedded: usize,
    index_ms: u128,
    query_ms: Vec<u128>,
    rows: Vec<Row>,
}

struct Row {
    first_hit_rank: Option<usize>,
    hit_at_5: bool,
    hit_at_10: bool,
    mrr: f64,
}

fn main() {
    let project_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let crates_dir = project_root.join("crates");
    let onnx_dir = project_root.join("models").join("onnx");
    if !crates_dir.is_dir() {
        eprintln!(
            "error: crates/ missing under {} (wrong project root?)",
            project_root.display()
        );
        std::process::exit(1);
    }
    if !onnx_dir.is_dir() {
        println!(
            "SKIP measure_code_embed_ab: models/onnx/ directory absent under {} \
             (models/ is gitignored; stage via \
             `python .elai_cc/scripts/stage_onnx_models.py --only coderankembed` \
             and `--only graphcodebert`). Exit 0.",
            project_root.display()
        );
        std::process::exit(0);
    }

    let cre_model = onnx_dir.join("coderankembed.onnx");
    let cre_tok = onnx_dir.join("coderankembed-tokenizer.json");
    let gcb_model = onnx_dir.join("graphcodebert.onnx");
    let gcb_tok = onnx_dir.join("graphcodebert-tokenizer.json");

    let cre_present = cre_model.is_file() && cre_tok.is_file();
    let gcb_present = gcb_model.is_file() && gcb_tok.is_file();

    if !cre_present || !gcb_present {
        let mut missing: Vec<&str> = Vec::new();
        if !cre_present { missing.push("coderankembed.onnx / coderankembed-tokenizer.json"); }
        if !gcb_present { missing.push("graphcodebert.onnx / graphcodebert-tokenizer.json"); }
        println!(
            "SKIP measure_code_embed_ab: missing ONNX assets: {}; stage them via \
             `python .elai_cc/scripts/stage_onnx_models.py --only coderankembed` \
             (and --only graphcodebert). Exit 0.",
            missing.join(", ")
        );
        std::process::exit(0);
    }

    eprintln!("=== Step 1: building bounded corpus (goldset + filler) ===");
    let all_files = collect_source_files(&crates_dir);
    let expected_substrs: Vec<String> = GOLDSET
        .iter()
        .flat_map(|g| g.expected.iter().map(|e| e.to_string()))
        .collect();
    let mut corpus: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for f in &all_files {
        let n = norm(&f.to_string_lossy());
        if expected_substrs.iter().any(|e| n.contains(e.as_str())) && seen.insert(n.clone()) {
            corpus.push(f.clone());
        }
    }
    let forced = corpus.len();
    let mut filler: Vec<PathBuf> = all_files
        .iter()
        .filter(|f| {
            let n = norm(&f.to_string_lossy());
            !seen.contains(&n) && FILLER_CRATE_DIRS.iter().any(|d| n.contains(d))
        })
        .cloned()
        .collect();
    filler.sort();
    filler.truncate(FILLER_FILES);
    for f in filler {
        let n = norm(&f.to_string_lossy());
        if seen.insert(n) {
            corpus.push(f);
        }
    }
    eprintln!(
        "  corpus: {} files ({} forced goldset + {} filler)",
        corpus.len(),
        forced,
        corpus.len() - forced
    );

    // BM25 is not used for the A/B (vector-only recall), but the existing
    // measure harness builds it for context. We skip it here to keep the
    // slice minimal and avoid tantivy index churn; the A/B is vector-only.
    let tmp = tempfile::tempdir().expect("tempdir");
    let _bm25 = Bm25Index::new(tmp.path(), PROJECT_ID).expect("Bm25Index::new");

    let mut lanes: Vec<LaneResult> = Vec::with_capacity(2);

    // Lane 1: CodeRankEmbed
    let cre_lane = run_lane(
        "onnx-cpu:coderankembed",
        &cre_model,
        &cre_tok,
        &corpus,
    );
    lanes.push(cre_lane);

    // Lane 2: GraphCodeBERT
    let gcb_lane = run_lane(
        "onnx-cpu:graphcodebert",
        &gcb_model,
        &gcb_tok,
        &corpus,
    );
    lanes.push(gcb_lane);

    // ----- print side-by-side -----
    println!("\n=== CODE EMBED A/B (CPU ONNX Runtime, 512-cap, vector-only recall) ===\n");
    println!(
        "{:<28} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "label", "chunks", "recall@5", "recall@10", "MRR", "p50_ms"
    );
    println!("{}", "-".repeat(82));
    for lane in &lanes {
        let n = lane.rows.len() as f64;
        let r5 = lane.rows.iter().filter(|r| r.hit_at_5).count() as f64 / n;
        let r10 = lane.rows.iter().filter(|r| r.hit_at_10).count() as f64 / n;
        let mrr = lane.rows.iter().map(|r| r.mrr).sum::<f64>() / n;
        let mut q: Vec<u128> = lane.query_ms.clone();
        q.sort();
        let p50 = if q.is_empty() { 0 } else { q[q.len() / 2] };
        println!(
            "{:<28} {:>10} {:>10.3} {:>10.3} {:>10.3} {:>10}",
            lane.label, lane.chunks_embedded, r5, r10, mrr, p50
        );
    }
    println!("{}", "-".repeat(82));

    // Per-query detail
    println!("\n--- per-query ---");
    println!(
        "{:<8} {:<28} {:>9} {:>5} {:>6} {:>6}",
        "id", "label", "hit_rank", "mrr", "hit@5", "hit@10"
    );
    println!("{}", "-".repeat(70));
    for (qi, g) in GOLDSET.iter().enumerate() {
        for lane in &lanes {
            let r = &lane.rows[qi];
            println!(
                "{:<8} {:<28} {:>9} {:>5.2} {:>6} {:>7}",
                g.id,
                lane.label,
                r.first_hit_rank
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "MISS".into()),
                r.mrr,
                r.hit_at_5,
                r.hit_at_10
            );
        }
    }

    // TOML summary to stdout (auditor-capturable)
    println!("\n--- toml ---");
    println!("# measure_code_embed_ab output (CPU ONNX, 512-cap, vector-only)");
    println!("corpus_files = {}", corpus.len());
    println!("forced_goldset_files = {}", forced);
    println!("queries = {}", GOLDSET.len());
    for lane in &lanes {
        let n = lane.rows.len() as f64;
        let r5 = lane.rows.iter().filter(|r| r.hit_at_5).count() as f64 / n;
        let r10 = lane.rows.iter().filter(|r| r.hit_at_10).count() as f64 / n;
        let mrr = lane.rows.iter().map(|r| r.mrr).sum::<f64>() / n;
        let mut q: Vec<u128> = lane.query_ms.clone();
        q.sort();
        let p50 = if q.is_empty() { 0 } else { q[q.len() / 2] };
        let p95 = if q.is_empty() {
            0
        } else {
            q[((q.len() as f64) * 0.95) as usize]
        };
        println!();
        println!("[[lane]]");
        println!("label = \"{}\"", lane.label);
        println!("chunks_embedded = {}", lane.chunks_embedded);
        println!("index_ms = {}", lane.index_ms);
        println!("recall_at_5 = {:.3}", r5);
        println!("recall_at_10 = {:.3}", r10);
        println!("mrr = {:.3}", mrr);
        println!("query_p50_ms = {}", p50);
        println!("query_p95_ms = {}", p95);
    }
}

fn run_lane(
    label: &'static str,
    model: &Path,
    tok: &Path,
    corpus: &[PathBuf],
) -> LaneResult {
    eprintln!("=== Lane {label}: loading ONNX ===");
    let session = match OrtSession::new(model, tok, 512, CODE_VEC_DIM) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  FATAL: {label} load failed: {e}");
            std::process::exit(2);
        }
    };
    let vm = CodeEmbedModule { session };

    eprintln!("=== Lane {label}: indexing (embed_code, no prefix) ===");
    let mut indexer = VectorIndexer::new(CODE_VEC_DIM);
    let t = Instant::now();
    let mut chunks_embedded = 0usize;
    for (i, p) in corpus.iter().enumerate() {
        match indexer.index_file(p, &vm, None) {
            Ok(s) => chunks_embedded += s.chunks_embedded,
            Err(e) => eprintln!("  WARN index {}: {e}", p.display()),
        }
        if i % 25 == 0 {
            eprintln!(
                "  {label} {}/{} ({} chunks, {:.1}s)",
                i + 1,
                corpus.len(),
                chunks_embedded,
                t.elapsed().as_secs_f64()
            );
        }
    }
    let index_ms = t.elapsed().as_millis();
    eprintln!(
        "  {label}: {} chunks, {}ms",
        chunks_embedded, index_ms
    );

    eprintln!("=== Lane {label}: scoring {} goldset queries (embed_code_query) ===", GOLDSET.len());
    let mut rows: Vec<Row> = Vec::with_capacity(GOLDSET.len());
    let mut query_ms: Vec<u128> = Vec::with_capacity(GOLDSET.len());
    for g in GOLDSET {
        let t = Instant::now();
        let qvec = match vm.embed_code_query(g.text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("  {} embed_code_query error: {e}", g.id);
                query_ms.push(t.elapsed().as_millis());
                rows.push(Row {
                    first_hit_rank: None,
                    hit_at_5: false,
                    hit_at_10: false,
                    mrr: 0.0,
                });
                continue;
            }
        };
        let hits = match indexer.search(&qvec, TOP_K) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("  {} search error: {e}", g.id);
                query_ms.push(t.elapsed().as_millis());
                rows.push(Row {
                    first_hit_rank: None,
                    hit_at_5: false,
                    hit_at_10: false,
                    mrr: 0.0,
                });
                continue;
            }
        };
        let elapsed = t.elapsed().as_millis();
        query_ms.push(elapsed);

        let mut first_hit_rank: Option<usize> = None;
        for (rank, (meta, _score)) in hits.iter().enumerate() {
            let cid = norm(&meta.path.to_string_lossy());
            if g.expected.iter().any(|e| cid.contains(e)) {
                first_hit_rank = Some(rank + 1);
                break;
            }
        }
        let hit_at_5 = first_hit_rank.map(|r| r <= 5).unwrap_or(false);
        let hit_at_10 = first_hit_rank.is_some();
        let mrr = first_hit_rank.map(|r| 1.0 / r as f64).unwrap_or(0.0);
        eprintln!(
            "  {label} {} hit@rank={:?} mrr={:.2} ({}ms)",
            g.id, first_hit_rank, mrr, elapsed
        );
        rows.push(Row {
            first_hit_rank,
            hit_at_5,
            hit_at_10,
            mrr,
        });
    }

    LaneResult {
        label,
        chunks_embedded,
        index_ms,
        query_ms,
        rows,
    }
}

fn collect_source_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_recursive(root, &mut out);
    out
}
fn collect_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let ns = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                ns.as_ref(),
                "target" | "node_modules" | ".git" | ".elai" | "__pycache__"
            ) {
                continue;
            }
            collect_recursive(&path, out);
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if SOURCE_EXTS.contains(&ext) {
                out.push(path);
            }
        }
    }
}
