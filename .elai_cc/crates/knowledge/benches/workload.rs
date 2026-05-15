// B-29.5 — workload bench.
// harness=false; simulates a 1000-query realistic mix:
//   600 impact_radius  (60%)
//   250 symbol_context (25%)
//   100 call_flow      (10%)
//    50 list_symbols    (5%)
//
// Measures peak RSS, LRU hit-rate proxy, and p50/p95/p99 latency per tool.
// Output: TOML metrics to stdout + written to metrics/phase_b29_5_workload_<tier>.toml.
//
// Auto-skip: ELAI_SCALE_FIXTURE_ROOT + elai_self fallback (see symbol_graph_scale.rs).

use std::path::{Path, PathBuf};
use std::time::Instant;
use std::hint::black_box;

use elai_knowledge::symbol_graph::cache::SymbolGraphCache;
use elai_knowledge::symbol_graph::sharding::SYMBOL_FILE_LRU_SIZE;
use elai_knowledge::symbol_graph::traversal::{
    compute_call_flow, compute_impact_radius, compute_symbol_context, list_symbols, SymbolFilter,
};

mod mem_helper {
    pub fn rss_bytes() -> u64 {
        #[cfg(target_os = "linux")]
        {
            let contents = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
            for line in contents.lines() {
                if line.starts_with("VmRSS:") {
                    let kb: u64 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return kb * 1024;
                }
            }
            0
        }
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let pid = std::process::id();
            let output = Command::new("wmic")
                .args([
                    "process",
                    "where",
                    &format!("ProcessId={pid}"),
                    "get",
                    "WorkingSetSize",
                ])
                .output();
            match output {
                Err(_) => 0,
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
                            return trimmed.parse().unwrap_or(0);
                        }
                    }
                    0
                }
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TOTAL_QUERIES: usize = 1000;
const IMPACT_QUERIES: usize = 600;
const CONTEXT_QUERIES: usize = 250;
const FLOW_QUERIES: usize = 100;
const LIST_QUERIES: usize = 50;

const IMPACT_DEPTH: u32 = 2;
const FLOW_DEPTH: u32 = 3;

// ---------------------------------------------------------------------------
// Percentile helpers
// ---------------------------------------------------------------------------

fn percentile_ms(sorted_ns: &[u128], pct: f64) -> f64 {
    if sorted_ns.is_empty() {
        return 0.0;
    }
    let idx = ((sorted_ns.len() as f64) * pct / 100.0) as usize;
    sorted_ns[idx.min(sorted_ns.len() - 1)] as f64 / 1_000_000.0
}

// ---------------------------------------------------------------------------
// Workload runner
// ---------------------------------------------------------------------------

struct WorkloadResult {
    tier: String,
    queries_run: usize,
    peak_rss_mb: u64,
    lru_hit_rate: f64,
    lru_size_at_end: usize,
    impact_p50_ms: f64,
    impact_p95_ms: f64,
    impact_p99_ms: f64,
    context_p50_ms: f64,
    context_p95_ms: f64,
    context_p99_ms: f64,
    flow_p50_ms: f64,
    flow_p95_ms: f64,
    flow_p99_ms: f64,
    list_p50_ms: f64,
    list_p95_ms: f64,
    list_p99_ms: f64,
}

fn run_workload(tier_name: &str, cache: &mut SymbolGraphCache, targets: &[String]) -> WorkloadResult {
    // Collect some file names for list_symbols file-scoped queries.
    let file_targets: Vec<String> = {
        let syms = list_symbols(cache, SymbolFilter { file: None, query: None, limit: 20 })
            .unwrap_or_default();
        syms.into_iter()
            .map(|s| s.file.clone())
            .collect::<std::collections::BTreeSet<String>>()
            .into_iter()
            .take(5)
            .collect()
    };

    let mut peak_rss = mem_helper::rss_bytes();

    let mut impact_ns: Vec<u128> = Vec::with_capacity(IMPACT_QUERIES);
    let mut context_ns: Vec<u128> = Vec::with_capacity(CONTEXT_QUERIES);
    let mut flow_ns: Vec<u128> = Vec::with_capacity(FLOW_QUERIES);
    let mut list_ns: Vec<u128> = Vec::with_capacity(LIST_QUERIES);

    // Build a cycled target iterator (BTreeMap iteration order is deterministic).
    let n_targets = targets.len().max(1);

    // impact_radius: 600 queries
    for i in 0..IMPACT_QUERIES {
        let target = &targets[i % n_targets];
        let t = Instant::now();
        black_box(compute_impact_radius(cache, target, IMPACT_DEPTH).ok());
        impact_ns.push(t.elapsed().as_nanos());
        let rss = mem_helper::rss_bytes();
        if rss > peak_rss {
            peak_rss = rss;
        }
    }

    // symbol_context: 250 queries
    for i in 0..CONTEXT_QUERIES {
        let target = &targets[i % n_targets];
        let t = Instant::now();
        black_box(compute_symbol_context(cache, target, None).ok());
        context_ns.push(t.elapsed().as_nanos());
        let rss = mem_helper::rss_bytes();
        if rss > peak_rss {
            peak_rss = rss;
        }
    }

    // call_flow: 100 queries
    for i in 0..FLOW_QUERIES {
        let target = &targets[i % n_targets];
        let t = Instant::now();
        black_box(compute_call_flow(cache, target, FLOW_DEPTH).ok());
        flow_ns.push(t.elapsed().as_nanos());
        let rss = mem_helper::rss_bytes();
        if rss > peak_rss {
            peak_rss = rss;
        }
    }

    // list_symbols: 50 queries (mix file-scoped and global)
    for i in 0..LIST_QUERIES {
        let file = if !file_targets.is_empty() {
            Some(file_targets[i % file_targets.len()].clone())
        } else {
            None
        };
        let t = Instant::now();
        black_box(list_symbols(cache, SymbolFilter { file, query: None, limit: 50 }).ok());
        list_ns.push(t.elapsed().as_nanos());
        let rss = mem_helper::rss_bytes();
        if rss > peak_rss {
            peak_rss = rss;
        }
    }

    impact_ns.sort_unstable();
    context_ns.sort_unstable();
    flow_ns.sort_unstable();
    list_ns.sort_unstable();

    let stats = cache.stats();
    let total = stats.lru_hits + stats.lru_misses;
    let hit_rate = if total > 0 {
        stats.lru_hits as f64 / total as f64
    } else {
        0.0
    };

    WorkloadResult {
        tier: tier_name.to_string(),
        queries_run: TOTAL_QUERIES,
        peak_rss_mb: peak_rss / 1_048_576,
        lru_hit_rate: hit_rate,
        lru_size_at_end: stats.lru_size,
        impact_p50_ms: percentile_ms(&impact_ns, 50.0),
        impact_p95_ms: percentile_ms(&impact_ns, 95.0),
        impact_p99_ms: percentile_ms(&impact_ns, 99.0),
        context_p50_ms: percentile_ms(&context_ns, 50.0),
        context_p95_ms: percentile_ms(&context_ns, 95.0),
        context_p99_ms: percentile_ms(&context_ns, 99.0),
        flow_p50_ms: percentile_ms(&flow_ns, 50.0),
        flow_p95_ms: percentile_ms(&flow_ns, 95.0),
        flow_p99_ms: percentile_ms(&flow_ns, 99.0),
        list_p50_ms: percentile_ms(&list_ns, 50.0),
        list_p95_ms: percentile_ms(&list_ns, 95.0),
        list_p99_ms: percentile_ms(&list_ns, 99.0),
    }
}

fn result_to_toml(r: &WorkloadResult) -> String {
    format!(
        r#"[{tier}]
queries = {queries}
peak_rss_mb_during_workload = {peak_rss}
cache_lru_hit_rate = {hit_rate:.4}
cache_lru_size_at_end = {lru_size}

[{tier}.workload.impact_radius]
p50_ms = {ip50:.3}
p95_ms = {ip95:.3}
p99_ms = {ip99:.3}

[{tier}.workload.symbol_context]
p50_ms = {cp50:.3}
p95_ms = {cp95:.3}
p99_ms = {cp99:.3}

[{tier}.workload.call_flow]
p50_ms = {fp50:.3}
p95_ms = {fp95:.3}
p99_ms = {fp99:.3}

[{tier}.workload.list_symbols]
p50_ms = {lp50:.3}
p95_ms = {lp95:.3}
p99_ms = {lp99:.3}
"#,
        tier = r.tier,
        queries = r.queries_run,
        peak_rss = r.peak_rss_mb,
        hit_rate = r.lru_hit_rate,
        lru_size = r.lru_size_at_end,
        ip50 = r.impact_p50_ms,
        ip95 = r.impact_p95_ms,
        ip99 = r.impact_p99_ms,
        cp50 = r.context_p50_ms,
        cp95 = r.context_p95_ms,
        cp99 = r.context_p99_ms,
        fp50 = r.flow_p50_ms,
        fp95 = r.flow_p95_ms,
        fp99 = r.flow_p99_ms,
        lp50 = r.list_p50_ms,
        lp95 = r.list_p95_ms,
        lp99 = r.list_p99_ms,
    )
}

// ---------------------------------------------------------------------------
// Path helpers (duplicated from symbol_graph_scale.rs — bench bins are isolated)
// ---------------------------------------------------------------------------

fn resolve_db(fixture_root: &Path, subdir: &str) -> PathBuf {
    fixture_root
        .join(subdir)
        .join(".elai_cc")
        .join("indexes")
        .join("dkg.db")
}

fn elai_self_db() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&manifest)
        .ancestors()
        .nth(2)
        .unwrap_or_else(|| Path::new("."))
        .join("indexes")
        .join("dkg.db")
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let fixture_root_env = std::env::var("ELAI_SCALE_FIXTURE_ROOT").unwrap_or_default();
    let fixture_root = PathBuf::from(&fixture_root_env);

    let known_targets: &[(&str, &str, Vec<&str>)] = &[
        ("linux_kernel", "linux", vec!["kmalloc", "schedule", "copy_from_user"]),
        ("chromium", "chromium", vec!["BrowserMain", "NewBrowserContext"]),
        ("vscode", "vscode", vec!["EditorWidget", "CommandService"]),
        ("postgres", "postgres", vec!["PortalRunMulti", "heap_insert"]),
        ("elai_self", "", vec!["ClockProvider", "GrcEngine", "DkgEngine"]),
    ];

    let mut any_run = false;

    for (tier_name, subdir, targets) in known_targets {
        let db_path = if *subdir == "" {
            elai_self_db()
        } else {
            resolve_db(&fixture_root, subdir)
        };

        if !db_path.exists() {
            eprintln!("[workload] {tier_name}: DB absent at {} — skip", db_path.display());
            continue;
        }

        any_run = true;
        eprintln!("[workload] {tier_name}: running {TOTAL_QUERIES}-query workload ...");

        let mut cache = match SymbolGraphCache::new(&db_path, tier_name, SYMBOL_FILE_LRU_SIZE) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[workload] {tier_name}: open error: {e}");
                continue;
            }
        };

        let target_strings: Vec<String> = targets.iter().map(|s| s.to_string()).collect();
        let result = run_workload(tier_name, &mut cache, &target_strings);

        let toml_out = result_to_toml(&result);
        let out_path = format!("metrics/phase_b29_5_workload_{tier_name}.toml");
        match std::fs::write(&out_path, &toml_out) {
            Ok(_) => eprintln!("[workload] {tier_name}: results written to {out_path}"),
            Err(e) => eprintln!("[workload] {tier_name}: could not write {out_path}: {e}"),
        }

        println!("{toml_out}");
    }

    if !any_run {
        eprintln!("[workload] No tier DBs found. Build the ELAI index first.");
    }
}
