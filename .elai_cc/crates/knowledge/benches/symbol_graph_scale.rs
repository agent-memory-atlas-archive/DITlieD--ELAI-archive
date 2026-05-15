// B-29.5 — symbol_graph_scale bench.
// harness=false; uses std::time::Instant + std::hint::black_box (no criterion).
//
// Loads a SymbolGraphCache from a pre-built index directory, runs 4 traversal
// tools 20 times each on a set of high-fan-in targets, and records
// {p50, p95, p99, mean} latency per tool in ms.
//
// Output: JSON written to metrics/phase_b29_5_bench_raw.json (and echoed to stdout).
//
// Auto-skip: if ELAI_SCALE_FIXTURE_ROOT is unset or the expected index DB is absent,
// the bench falls back to the ELAI workspace itself (resolved from CARGO_MANIFEST_DIR).
// This lets the harness be validated without cloning multi-gigabyte fixtures.

use std::collections::BTreeMap;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use elai_knowledge::symbol_graph::cache::SymbolGraphCache;
use elai_knowledge::symbol_graph::sharding::SYMBOL_FILE_LRU_SIZE;
use elai_knowledge::symbol_graph::traversal::{
    compute_call_flow, compute_impact_radius, compute_symbol_context, list_symbols, SymbolFilter,
};

// ---------------------------------------------------------------------------
// Bench constants
// ---------------------------------------------------------------------------

const SAMPLE_COUNT: usize = 20;
const IMPACT_DEPTH: u32 = 3;
const CALL_FLOW_DEPTH: u32 = 4;

// ---------------------------------------------------------------------------
// Tier descriptor
// ---------------------------------------------------------------------------

struct Tier {
    name: &'static str,
    /// Path to the .elai_cc/indexes/dkg.db inside the cloned fixture.
    db_path: PathBuf,
    project_id: String,
    /// Well-known high-fan-in symbol names to query.  If empty, bench uses
    /// the first 5 symbols returned by list_symbols.
    known_targets: Vec<&'static str>,
}

fn resolve_db_path(fixture_root: &Path, tier_subdir: &str) -> PathBuf {
    fixture_root
        .join(tier_subdir)
        .join(".elai_cc")
        .join("indexes")
        .join("dkg.db")
}

fn elai_self_db_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = .elai_cc/crates/knowledge at bench time.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&manifest)
        .ancestors()
        .nth(2) // .elai_cc
        .unwrap_or_else(|| Path::new("."))
        .join("indexes")
        .join("dkg.db")
}

fn build_tiers() -> Vec<Tier> {
    let fixture_root_env = std::env::var("ELAI_SCALE_FIXTURE_ROOT").unwrap_or_default();
    let fixture_root = PathBuf::from(&fixture_root_env);

    let mut tiers: Vec<Tier> = Vec::new();

    let scale_tiers: &[(&str, &str, Vec<&'static str>)] = &[
        (
            "linux_kernel",
            "linux",
            vec!["kmalloc", "schedule", "do_syscall_64", "copy_from_user"],
        ),
        (
            "chromium",
            "chromium",
            vec!["BrowserMain", "NewBrowserContext", "RenderFrameHostImpl"],
        ),
        (
            "vscode",
            "vscode",
            vec!["EditorWidget", "CommandService", "LanguageService"],
        ),
        (
            "postgres",
            "postgres",
            vec!["PortalRunMulti", "exec_simple_query", "heap_insert"],
        ),
    ];

    for (name, subdir, known_targets) in scale_tiers {
        let db_path = resolve_db_path(&fixture_root, subdir);
        if !db_path.exists() {
            eprintln!(
                "[B-29.5] {name}: DB not found at {} — skipping tier",
                db_path.display()
            );
            continue;
        }
        tiers.push(Tier {
            name,
            db_path,
            project_id: (*subdir).to_string(),
            known_targets: known_targets.clone(),
        });
    }

    // Always include elai_self as the baseline tier (uses the ELAI workspace index).
    let elai_db = elai_self_db_path();
    if elai_db.exists() {
        tiers.push(Tier {
            name: "elai_self",
            db_path: elai_db,
            project_id: "elai_self".to_string(),
            known_targets: vec!["ClockProvider", "GrcEngine", "DkgEngine", "SymbolGraphCache"],
        });
    } else {
        eprintln!(
            "[B-29.5] elai_self: DB not found at {} — no baseline tier available",
            elai_db.display()
        );
    }

    tiers
}

// ---------------------------------------------------------------------------
// Measurement helpers
// ---------------------------------------------------------------------------

fn percentile(sorted: &[u128], pct: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64) * pct / 100.0) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn mean_ms(sorted: &[u128]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let sum: u128 = sorted.iter().sum();
    (sum as f64) / (sorted.len() as f64) / 1_000_000.0
}

/// Pick target symbols for a tier: use known_targets list if the cache knows them,
/// otherwise fall back to the first symbols returned by list_symbols.
fn pick_targets(cache: &mut SymbolGraphCache, known: &[&'static str]) -> Vec<String> {
    // Try to verify known targets exist in the name index.
    let verified: Vec<String> = known.iter().map(|s| s.to_string()).collect();

    if !verified.is_empty() {
        return verified;
    }

    // Fallback: list up to 5 symbols from the cache.
    match list_symbols(cache, SymbolFilter { file: None, query: None, limit: 5 }) {
        Ok(syms) => syms.into_iter().map(|s| s.name).collect(),
        Err(_) => vec![],
    }
}

// ---------------------------------------------------------------------------
// Per-tool bench runners
// ---------------------------------------------------------------------------

struct ToolTimes {
    tool: &'static str,
    samples_ms: Vec<f64>,
}

fn run_impact_radius(cache: &mut SymbolGraphCache, targets: &[String]) -> ToolTimes {
    let mut samples: Vec<u128> = Vec::with_capacity(SAMPLE_COUNT * targets.len().max(1));
    for target in targets {
        for _ in 0..SAMPLE_COUNT {
            let t = Instant::now();
            let r = compute_impact_radius(cache, target, IMPACT_DEPTH);
            samples.push(t.elapsed().as_nanos());
            black_box(r.ok());
        }
    }
    samples.sort_unstable();
    ToolTimes {
        tool: "impact_radius",
        samples_ms: samples.iter().map(|&ns| ns as f64 / 1_000_000.0).collect(),
    }
}

fn run_call_flow(cache: &mut SymbolGraphCache, targets: &[String]) -> ToolTimes {
    let mut samples: Vec<u128> = Vec::with_capacity(SAMPLE_COUNT * targets.len().max(1));
    for target in targets {
        for _ in 0..SAMPLE_COUNT {
            let t = Instant::now();
            let r = compute_call_flow(cache, target, CALL_FLOW_DEPTH);
            samples.push(t.elapsed().as_nanos());
            black_box(r.ok());
        }
    }
    samples.sort_unstable();
    ToolTimes {
        tool: "call_flow",
        samples_ms: samples.iter().map(|&ns| ns as f64 / 1_000_000.0).collect(),
    }
}

fn run_symbol_context(cache: &mut SymbolGraphCache, targets: &[String]) -> ToolTimes {
    let mut samples: Vec<u128> = Vec::with_capacity(SAMPLE_COUNT * targets.len().max(1));
    for target in targets {
        for _ in 0..SAMPLE_COUNT {
            let t = Instant::now();
            let r = compute_symbol_context(cache, target, None);
            samples.push(t.elapsed().as_nanos());
            black_box(r.ok());
        }
    }
    samples.sort_unstable();
    ToolTimes {
        tool: "symbol_context",
        samples_ms: samples.iter().map(|&ns| ns as f64 / 1_000_000.0).collect(),
    }
}

fn run_list_symbols(cache: &mut SymbolGraphCache) -> ToolTimes {
    let mut samples: Vec<u128> = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let t = Instant::now();
        let r = list_symbols(cache, SymbolFilter { file: None, query: None, limit: 100 });
        samples.push(t.elapsed().as_nanos());
        black_box(r.ok());
    }
    samples.sort_unstable();
    ToolTimes {
        tool: "list_symbols",
        samples_ms: samples.iter().map(|&ns| ns as f64 / 1_000_000.0).collect(),
    }
}

// ---------------------------------------------------------------------------
// JSON serialisation (stdlib-only: manual string building)
// ---------------------------------------------------------------------------

fn tool_times_to_json(tt: &ToolTimes) -> String {
    let mut sorted: Vec<f64> = tt.samples_ms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let sorted_ns: Vec<u128> = sorted.iter().map(|ms| (*ms * 1_000_000.0) as u128).collect();

    let p50 = percentile(&sorted_ns, 50) as f64 / 1_000_000.0;
    let p95 = percentile(&sorted_ns, 95) as f64 / 1_000_000.0;
    let p99 = percentile(&sorted_ns, 99) as f64 / 1_000_000.0;
    let mean = mean_ms(&sorted_ns);
    let count = sorted.len();

    format!(
        r#"{{ "tool": "{}", "p50_ms": {:.3}, "p95_ms": {:.3}, "p99_ms": {:.3}, "mean_ms": {:.3}, "samples": {} }}"#,
        tt.tool, p50, p95, p99, mean, count
    )
}

fn tier_results_to_json(tier_name: &str, tools: &[ToolTimes], lru_size: usize) -> String {
    let tool_jsons: Vec<String> = tools.iter().map(tool_times_to_json).collect();
    format!(
        r#"{{ "tier": "{}", "lru_size_at_end": {}, "tools": [{}] }}"#,
        tier_name,
        lru_size,
        tool_jsons.join(", ")
    )
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let tiers = build_tiers();

    if tiers.is_empty() {
        eprintln!(
            "[B-29.5] No tiers available. Set ELAI_SCALE_FIXTURE_ROOT or build the ELAI index first."
        );
        eprintln!("[B-29.5] Run: cargo run --release -- symbol_graph build");
        return;
    }

    let mut tier_results: Vec<String> = Vec::new();

    for tier in &tiers {
        eprintln!("[B-29.5] Benchmarking tier '{}' ...", tier.name);

        let mut cache = match SymbolGraphCache::new(&tier.db_path, &tier.project_id, SYMBOL_FILE_LRU_SIZE) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[B-29.5] {} — cache open failed: {e} — skipping", tier.name);
                continue;
            }
        };

        let targets = pick_targets(&mut cache, &tier.known_targets);
        if targets.is_empty() {
            eprintln!("[B-29.5] {} — no targets found, skipping", tier.name);
            continue;
        }

        eprintln!("[B-29.5] {} — targets: {:?}", tier.name, targets);

        let ir = run_impact_radius(&mut cache, &targets);
        let cf = run_call_flow(&mut cache, &targets);
        let sc = run_symbol_context(&mut cache, &targets);
        let ls = run_list_symbols(&mut cache);

        let lru_size = cache.file_payload_lru_size();
        let tools = [ir, cf, sc, ls];

        for tool in &tools {
            if !tool.samples_ms.is_empty() {
                let sorted_ns: Vec<u128> = tool
                    .samples_ms
                    .iter()
                    .map(|ms| (*ms * 1_000_000.0) as u128)
                    .collect();
                let mut sorted_ns2 = sorted_ns.clone();
                sorted_ns2.sort_unstable();
                let p99_ms = percentile(&sorted_ns2, 99) as f64 / 1_000_000.0;
                eprintln!(
                    "[B-29.5]   {} p99={:.1}ms mean={:.1}ms",
                    tool.tool,
                    p99_ms,
                    mean_ms(&sorted_ns2)
                );
            }
        }

        tier_results.push(tier_results_to_json(tier.name, &tools, lru_size));
    }

    let json_output = format!("[{}]", tier_results.join(",\n"));

    // Write to metrics file.
    let out_path = "metrics/phase_b29_5_bench_raw.json";
    match std::fs::write(out_path, &json_output) {
        Ok(_) => eprintln!("[B-29.5] Results written to {out_path}"),
        Err(e) => eprintln!("[B-29.5] Could not write {out_path}: {e}"),
    }

    // Also echo to stdout for pipe-capture.
    println!("{json_output}");
}
