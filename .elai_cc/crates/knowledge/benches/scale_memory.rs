// B-29.5 — scale_memory bench.
// harness=false; measures process RSS and cache state snapshot at startup vs
// after loading a SymbolGraphCache.
//
// No external deps: RSS via /proc/self/status on Linux; via wmic on Windows;
// returns 0 on unsupported platforms.
//
// Auto-skip: same ELAI_SCALE_FIXTURE_ROOT auto-fallback as symbol_graph_scale.rs.

use std::path::{Path, PathBuf};

use elai_knowledge::symbol_graph::cache::{CacheStats, SymbolGraphCache};
use elai_knowledge::symbol_graph::sharding::SYMBOL_FILE_LRU_SIZE;
use elai_knowledge::symbol_graph::traversal::{compute_impact_radius, list_symbols, SymbolFilter};

// ---------------------------------------------------------------------------
// MemorySnapshot
// ---------------------------------------------------------------------------

/// Process-level and cache-level memory snapshot.
#[derive(Debug)]
pub struct MemorySnapshot {
    /// Resident Set Size in bytes. 0 = not measurable on this platform.
    pub rss_bytes: u64,
    /// Number of file payloads in the LRU.
    pub cache_lru_size: usize,
    /// Whether the name index is loaded in memory.
    pub name_index_loaded: bool,
    /// Whether the reverse file index is loaded in memory.
    pub reverse_index_loaded: bool,
    /// LRU hit count since cache was opened.
    pub lru_hits: u64,
    /// LRU miss count since cache was opened.
    pub lru_misses: u64,
}

impl MemorySnapshot {
    pub fn from_stats(rss_bytes: u64, stats: CacheStats) -> Self {
        Self {
            rss_bytes,
            cache_lru_size: stats.lru_size,
            name_index_loaded: stats.name_index_loaded,
            reverse_index_loaded: stats.reverse_index_loaded,
            lru_hits: stats.lru_hits,
            lru_misses: stats.lru_misses,
        }
    }

    pub fn lru_hit_rate(&self) -> f64 {
        let total = self.lru_hits + self.lru_misses;
        if total == 0 {
            return 0.0;
        }
        self.lru_hits as f64 / total as f64
    }
}

// ---------------------------------------------------------------------------
// RSS measurement — platform-specific
// ---------------------------------------------------------------------------

/// Read process RSS in bytes.
/// Returns 0 if the platform is unsupported or the read fails.
pub fn current_rss_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        rss_linux()
    }
    #[cfg(target_os = "windows")]
    {
        rss_windows()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        0
    }
}

#[cfg(target_os = "linux")]
fn rss_linux() -> u64 {
    // /proc/self/status line format: "VmRSS:\t   12345 kB"
    let contents = match std::fs::read_to_string("/proc/self/status") {
        Ok(s) => s,
        Err(_) => return 0,
    };
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
fn rss_windows() -> u64 {
    use std::process::Command;
    // wmic query: returns working set in bytes.
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
            // Output: "WorkingSetSize\n<bytes>\n"
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

// ---------------------------------------------------------------------------
// Snapshot helper
// ---------------------------------------------------------------------------

pub fn snapshot(cache: &SymbolGraphCache) -> MemorySnapshot {
    let rss = current_rss_bytes();
    MemorySnapshot::from_stats(rss, cache.stats())
}

// ---------------------------------------------------------------------------
// Main
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

fn main() {
    let fixture_root_env = std::env::var("ELAI_SCALE_FIXTURE_ROOT").unwrap_or_default();
    let fixture_root = PathBuf::from(&fixture_root_env);

    let candidates: Vec<(&str, PathBuf, &str)> = vec![
        (
            "linux_kernel",
            resolve_db(&fixture_root, "linux"),
            "linux",
        ),
        (
            "chromium",
            resolve_db(&fixture_root, "chromium"),
            "chromium",
        ),
        ("vscode", resolve_db(&fixture_root, "vscode"), "vscode"),
        (
            "postgres",
            resolve_db(&fixture_root, "postgres"),
            "postgres",
        ),
        ("elai_self", elai_self_db(), "elai_self"),
    ];

    let mut any_run = false;

    for (tier_name, db_path, project_id) in &candidates {
        if !db_path.exists() {
            eprintln!("[scale_memory] {tier_name}: DB absent — skip");
            continue;
        }
        any_run = true;
        eprintln!("[scale_memory] {tier_name}: opening cache ...");

        let baseline_rss = current_rss_bytes();
        eprintln!(
            "[scale_memory] {tier_name}: baseline RSS = {} MB",
            baseline_rss / 1_048_576
        );

        let mut cache =
            match SymbolGraphCache::new(db_path, project_id, SYMBOL_FILE_LRU_SIZE) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[scale_memory] {tier_name}: cache open error: {e}");
                    continue;
                }
            };

        let after_open = snapshot(&cache);
        eprintln!(
            "[scale_memory] {tier_name}: after open — RSS={} MB name_idx={} rev_idx={}",
            after_open.rss_bytes / 1_048_576,
            after_open.name_index_loaded,
            after_open.reverse_index_loaded
        );

        // Warm the name index (lazy load).
        let _ = list_symbols(&mut cache, SymbolFilter { file: None, query: None, limit: 10 });
        let after_name_idx = snapshot(&cache);
        eprintln!(
            "[scale_memory] {tier_name}: after name-idx load — RSS={} MB lru={}",
            after_name_idx.rss_bytes / 1_048_576,
            after_name_idx.cache_lru_size
        );

        // Run a few impact_radius queries to warm LRU.
        let warm_targets = &["init", "main", "new", "run"];
        for t in warm_targets {
            let _ = compute_impact_radius(&mut cache, t, 2);
        }
        let after_warm = snapshot(&cache);
        eprintln!(
            "[scale_memory] {tier_name}: after LRU warm — RSS={} MB lru={} hit_rate={:.2}",
            after_warm.rss_bytes / 1_048_576,
            after_warm.cache_lru_size,
            after_warm.lru_hit_rate()
        );

        let delta_mb = (after_warm.rss_bytes as i64 - baseline_rss as i64).max(0) / 1_048_576;
        println!(
            "[scale_memory] {tier_name}: delta_rss_mb={delta_mb} lru_size={} hit_rate={:.3}",
            after_warm.cache_lru_size,
            after_warm.lru_hit_rate()
        );
    }

    if !any_run {
        eprintln!("[scale_memory] No tier DBs found. Build the ELAI index first.");
    }
}
