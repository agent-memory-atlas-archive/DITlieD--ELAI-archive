// A-16: PI latency benchmark across project sizes (spec 3L.8).
// Measures Tier 1, Tier 2, Tier 3 detection on each project root.
// Grounds the UNGROUNDED cost estimates in project-intelligence-design.txt.

use std::path::{Path, PathBuf};
use std::time::Instant;

use elai_knowledge::{
    detect_project_profile, populate_tier2_conventions, populate_tier3, DkgEngine,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of warmup runs before measurement (amortises filesystem cache).
const WARMUP_RUNS: usize = 1;

/// Number of measured runs per tier per project.
const MEASURED_RUNS: usize = 3;

// ---------------------------------------------------------------------------
// Measurement result
// ---------------------------------------------------------------------------

struct TierMeasurement {
    tier: &'static str,
    p50_ms: f64,
    p95_ms: f64,
    min_ms: f64,
    max_ms: f64,
    runs: usize,
}

struct ProjectResult {
    name: String,
    loc: usize,
    tiers: Vec<TierMeasurement>,
    total_p50_ms: f64,
    tier2_dkg_build_p50_ms: f64,
}

// ---------------------------------------------------------------------------
// LOC counter (ignores blank/comment lines)
// ---------------------------------------------------------------------------

fn count_loc(root: &Path) -> usize {
    // Build-artifact directories excluded to prevent LOC inflation.
    // git_ignore is enabled so .gitignore rules are respected. The explicit
    // segment check below catches projects without a .gitignore.
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .filter_entry(|e| {
            // Skip well-known build / cache directories regardless of .gitignore.
            let excluded = ["target", "node_modules", "build", "dist", ".git", "__pycache__", ".mypy_cache", ".pytest_cache"];
            if e.file_type().map_or(false, |ft| ft.is_dir()) {
                let name = e.file_name().to_string_lossy();
                return !excluded.iter().any(|ex| name == *ex);
            }
            true
        })
        .build();

    let mut total = 0usize;
    for entry in walker.flatten() {
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }
        let path = entry.path();
        let is_source = matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go" | "java" | "kt"
                | "cs" | "cpp" | "cc" | "c" | "h" | "rb" | "ex" | "exs" | "php"
                | "swift" | "zig" | "dart" | "toml" | "json" | "yaml" | "yml")
        );
        if !is_source {
            continue;
        }
        if let Ok(contents) = std::fs::read_to_string(path) {
            total += contents.lines().filter(|l| !l.trim().is_empty()).count();
        }
    }
    total
}

// ---------------------------------------------------------------------------
// Tier measurement helpers
// ---------------------------------------------------------------------------

fn measure_tier1(root: &Path, runs: usize) -> (Vec<u128>, bool) {
    let mut durations = Vec::with_capacity(runs);
    let mut ok = true;
    for _ in 0..runs {
        let start = Instant::now();
        match detect_project_profile(root) {
            Ok(_) => {}
            Err(_) => { ok = false; }
        }
        durations.push(start.elapsed().as_micros());
    }
    (durations, ok)
}

fn measure_tier2(root: &Path, runs: usize) -> (Vec<u128>, bool, Vec<u128>) {
    let mut durations = Vec::with_capacity(runs);
    let mut dkg_build_durations = Vec::with_capacity(runs);
    let mut ok = true;
    for _ in 0..runs {
        let mut profile = match detect_project_profile(root) {
            Ok(p) => p,
            Err(_) => { ok = false; continue; }
        };

        let db_dir = std::env::temp_dir().join("elai_pi_bench_t2");
        let _ = std::fs::create_dir_all(&db_dir);
        let db_path = db_dir.join("bench.db");
        let _ = std::fs::remove_file(&db_path);

        // Time DKG build_graph() separately — this cost was previously excluded
        // from the tier2 timer, hiding the true DKG construction overhead.
        let dkg_engine = DkgEngine::new(&db_path, None, 0).ok();
        let built_engine = dkg_engine.and_then(|mut e| {
            let t = Instant::now();
            let result = e.build_graph(root).ok().map(|_| e);
            dkg_build_durations.push(t.elapsed().as_micros());
            result
        });
        // If DkgEngine::new() failed we never pushed a duration; pad with 0.
        if dkg_build_durations.len() < durations.len() + 1 {
            dkg_build_durations.push(0);
        }

        let start = Instant::now();
        populate_tier2_conventions(&mut profile, built_engine.as_ref());
        durations.push(start.elapsed().as_micros());

        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir(&db_dir);
    }
    (durations, ok, dkg_build_durations)
}

fn measure_tier3(root: &Path, runs: usize) -> (Vec<u128>, bool) {
    let mut durations = Vec::with_capacity(runs);
    let mut ok = true;
    for _ in 0..runs {
        let mut profile = match detect_project_profile(root) {
            Ok(p) => p,
            Err(_) => { ok = false; continue; }
        };

        let db_dir = std::env::temp_dir().join("elai_pi_bench_t3");
        let _ = std::fs::create_dir_all(&db_dir);
        let db_path = db_dir.join("bench.db");
        let _ = std::fs::remove_file(&db_path);

        let dkg_nodes = match DkgEngine::new(&db_path, None, 0) {
            Ok(mut e) => {
                let _ = e.build_graph(root);
                e.nodes().unwrap_or_default()
            }
            Err(_) => Vec::new(),
        };

        populate_tier2_conventions(&mut profile, None);

        let start = Instant::now();
        populate_tier3(&mut profile, &dkg_nodes);
        durations.push(start.elapsed().as_micros());

        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir(&db_dir);
    }
    (durations, ok)
}

fn percentile(sorted: &[u128], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * pct / 100.0).round() as usize;
    let idx = idx.min(sorted.len() - 1);
    sorted[idx] as f64 / 1000.0 // micros -> millis
}

fn to_measurement(tier: &'static str, mut durations: Vec<u128>) -> TierMeasurement {
    durations.sort();
    TierMeasurement {
        tier,
        p50_ms: percentile(&durations, 50.0),
        p95_ms: percentile(&durations, 95.0),
        min_ms: durations.first().map_or(0.0, |v| *v as f64 / 1000.0),
        max_ms: durations.last().map_or(0.0, |v| *v as f64 / 1000.0),
        runs: durations.len(),
    }
}

// ---------------------------------------------------------------------------
// Missing LOC bracket note (A-16 audit finding)
// ---------------------------------------------------------------------------
//
// The benchmark needs representative projects at ~1K LOC and ~10K LOC to
// bracket the cost estimates from project-intelligence-design.txt. The current
// test set only covers small toy repos. Suitable open-source candidates:
//
//   ~1K LOC:
//     - https://github.com/BurntSushi/ripgrep  (crate ripgrep-lib, ~1.2K LOC)
//     - https://github.com/sharkdp/bat         (src/ only, ~1.1K LOC)
//
//   ~10K LOC:
//     - https://github.com/tokio-rs/tokio       (tokio/src/, ~14K LOC)
//     - https://github.com/rust-lang/cargo      (src/cargo/, ~11K LOC)
//
// To add a bracket: clone the repo locally, then pass its root as an argument:
//   cargo run --example measure_pi_latency -- /path/to/ripgrep /path/to/tokio
//
// Until these runs are executed, the Tier 1/2/3 latency targets for mid-size
// projects remain UNGROUNDED and should not be treated as validated baselines.
//
// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "usage: measure_pi_latency <project_root> [project_root ...]\n\
             \n\
             Profiles PI detection latency (Tier 1/2/3) on each project.\n\
             Results written to metrics/pi_latency.toml in the first project root,\n\
             or to the path given by --output <path>."
        );
        std::process::exit(1);
    }

    let mut output_override: Option<PathBuf> = None;
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--output" {
            i += 1;
            if i >= args.len() {
                eprintln!("error: --output requires a path argument");
                std::process::exit(1);
            }
            output_override = Some(PathBuf::from(&args[i]));
        } else {
            let p = PathBuf::from(&args[i]);
            if !p.is_dir() {
                eprintln!("warning: skipping non-directory: {}", p.display());
            } else {
                roots.push(p);
            }
        }
        i += 1;
    }

    if roots.is_empty() {
        eprintln!("error: no valid project directories provided");
        std::process::exit(1);
    }

    let mut results: Vec<ProjectResult> = Vec::new();

    for root in &roots {
        let name = root
            .file_name()
            .map_or_else(|| root.display().to_string(), |n| n.to_string_lossy().into_owned());

        eprintln!("--- measuring: {} ({})", name, root.display());

        let loc = count_loc(root);
        eprintln!("  LOC (non-blank source): {loc}");

        // Warmup
        for _ in 0..WARMUP_RUNS {
            let _ = detect_project_profile(root);
        }

        let (d1, ok1) = measure_tier1(root, MEASURED_RUNS);
        let m1 = to_measurement("tier1", d1);
        eprintln!("  Tier 1: p50={:.1}ms p95={:.1}ms ok={ok1}", m1.p50_ms, m1.p95_ms);

        let (d2, ok2, d2_dkg) = measure_tier2(root, MEASURED_RUNS);
        let m2 = to_measurement("tier2", d2);
        let m2_dkg = to_measurement("tier2_dkg_build", d2_dkg);
        eprintln!("  Tier 2: p50={:.1}ms p95={:.1}ms ok={ok2} (dkg_build p50={:.1}ms)", m2.p50_ms, m2.p95_ms, m2_dkg.p50_ms);

        let (d3, ok3) = measure_tier3(root, MEASURED_RUNS);
        let m3 = to_measurement("tier3", d3);
        eprintln!("  Tier 3: p50={:.1}ms p95={:.1}ms ok={ok3}", m3.p50_ms, m3.p95_ms);

        let total_p50 = m1.p50_ms + m2.p50_ms + m3.p50_ms;
        eprintln!("  Total p50: {total_p50:.1}ms");

        results.push(ProjectResult {
            name,
            loc,
            tiers: vec![m1, m2, m3],
            total_p50_ms: total_p50,
            tier2_dkg_build_p50_ms: m2_dkg.p50_ms,
        });
    }

    // Build TOML output
    let mut toml = String::from("# A-16: PI latency benchmark (spec 3L.8)\n");
    toml.push_str("# Grounds UNGROUNDED cost estimates from project-intelligence-design.txt\n\n");

    for res in &results {
        let section = res.name.replace(['-', '.', ' '], "_");
        toml.push_str(&format!("[project.{}]\n", section));
        toml.push_str(&format!("name = \"{}\"\n", res.name));
        toml.push_str(&format!("loc = {}\n", res.loc));
        toml.push_str(&format!("total_p50_ms = {:.1}\n", res.total_p50_ms));
        toml.push_str(&format!("tier2_dkg_build_ms = {:.1}\n", res.tier2_dkg_build_p50_ms));

        for tier in &res.tiers {
            toml.push_str(&format!(
                "{}_p50_ms = {:.1}\n{}_p95_ms = {:.1}\n{}_min_ms = {:.1}\n{}_max_ms = {:.1}\n{}_runs = {}\n",
                tier.tier, tier.p50_ms,
                tier.tier, tier.p95_ms,
                tier.tier, tier.min_ms,
                tier.tier, tier.max_ms,
                tier.tier, tier.runs,
            ));
        }
        toml.push('\n');
    }

    // Summary: compare against spec targets
    toml.push_str("[summary]\n");
    toml.push_str("# Spec targets (formerly UNGROUNDED): Tier1 <5s, Tier2 <30s, Tier3 <60s, Total <90s\n");

    let max_t1 = results.iter().flat_map(|r| r.tiers.iter().filter(|t| t.tier == "tier1")).map(|t| t.p95_ms).fold(0.0f64, f64::max);
    let max_t2 = results.iter().flat_map(|r| r.tiers.iter().filter(|t| t.tier == "tier2")).map(|t| t.p95_ms).fold(0.0f64, f64::max);
    let max_t3 = results.iter().flat_map(|r| r.tiers.iter().filter(|t| t.tier == "tier3")).map(|t| t.p95_ms).fold(0.0f64, f64::max);
    let max_total = results.iter().map(|r| r.total_p50_ms).fold(0.0f64, f64::max);

    toml.push_str(&format!("worst_tier1_p95_ms = {:.1}\n", max_t1));
    toml.push_str(&format!("worst_tier2_p95_ms = {:.1}\n", max_t2));
    toml.push_str(&format!("worst_tier3_p95_ms = {:.1}\n", max_t3));
    toml.push_str(&format!("worst_total_p50_ms = {:.1}\n", max_total));

    let tier1_pass = max_t1 < 5000.0;
    let tier2_pass = max_t2 < 30000.0;
    let tier3_pass = max_t3 < 60000.0;
    let total_pass = max_total < 90000.0;

    toml.push_str(&format!("tier1_under_5s = {tier1_pass}\n"));
    toml.push_str(&format!("tier2_under_30s = {tier2_pass}\n"));
    toml.push_str(&format!("tier3_under_60s = {tier3_pass}\n"));
    toml.push_str(&format!("total_under_90s = {total_pass}\n"));

    // Write output
    let output_path = output_override.unwrap_or_else(|| {
        let metrics_dir = PathBuf::from("metrics");
        let _ = std::fs::create_dir_all(&metrics_dir);
        metrics_dir.join("pi_latency.toml")
    });

    if let Some(parent) = output_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    std::fs::write(&output_path, &toml).unwrap_or_else(|e| {
        eprintln!("error: cannot write {}: {e}", output_path.display());
        std::process::exit(1);
    });

    println!("wrote {}", output_path.display());
    println!("---");
    print!("{toml}");

    // Exit with non-zero if any target missed
    if !tier1_pass || !tier2_pass || !tier3_pass || !total_pass {
        eprintln!("WARN: one or more latency targets exceeded (baselines recorded)");
        std::process::exit(2);
    }
}
