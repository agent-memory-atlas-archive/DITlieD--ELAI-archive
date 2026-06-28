/// MEMORY PROBE (retrieval-quality-rigorous Task 2): build the DKG over a
/// bounded subtree and report node/edge counts plus a self-sampled peak-RSS
/// timeline so the index-time memory hog can be attributed to a phase and the
/// full-corpus build can be extrapolated before any unguarded run.
///
/// Usage: dkg_mem_probe <subtree_root>   (e.g. crates/orchestrator)
/// Set ELAI_DKG_PHASE_TRACE=1 to interleave `DKG_PHASE <name>` markers (stderr)
/// with the RSS samples this probe prints, so peak RSS is attributable to a phase.
/// Run under the external 6GB watchdog; this binary does NOT cap itself.
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use elai_knowledge::dkg::DkgEngine;

fn main() {
    let subtree = std::env::args().nth(1).map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("crates/knowledge"));
    let tmp = tempfile::tempdir().expect("tempdir");
    let db = tmp.path().join("probe_dkg.sqlite");
    let mut engine = DkgEngine::new(&db, None, 0).expect("DkgEngine::new");

    // Background RSS sampler: prints the running peak every ~250ms so the
    // operator can correlate the climb against the DKG_PHASE stderr markers.
    let stop = Arc::new(AtomicBool::new(false));
    let peak_kb = Arc::new(AtomicU64::new(0));
    let sampler = {
        let stop = stop.clone();
        let peak_kb = peak_kb.clone();
        std::thread::spawn(move || {
            let start = Instant::now();
            while !stop.load(Ordering::Relaxed) {
                if let Some(kb) = current_rss_kb() {
                    let prev = peak_kb.load(Ordering::Relaxed);
                    if kb > prev {
                        peak_kb.store(kb, Ordering::Relaxed);
                        eprintln!(
                            "RSS_SAMPLE t={:.1}s rss_mb={:.0} (new peak)",
                            start.elapsed().as_secs_f64(),
                            kb as f64 / 1024.0
                        );
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        })
    };

    let t = Instant::now();
    let graph = engine.build_graph(&subtree).expect("build_graph");
    let secs = t.elapsed().as_secs_f64();

    stop.store(true, Ordering::Relaxed);
    let _ = sampler.join();
    let peak_mb = peak_kb.load(Ordering::Relaxed) as f64 / 1024.0;

    let mut files = 0usize;
    walk_count(&subtree, &mut files);

    eprintln!(
        "DKG_PROBE subtree={} source_files={} parsed={} nodes={} edges={} build_secs={:.1} self_peak_rss_mb={:.0}",
        subtree.display(), files, graph.files_parsed, graph.node_count, graph.edge_count, secs, peak_mb
    );
    println!(
        "source_files={files} parsed={} nodes={} edges={} build_secs={:.1} self_peak_rss_mb={:.0}",
        graph.files_parsed, graph.node_count, graph.edge_count, secs, peak_mb
    );
}

/// Read this process's current resident set size in KiB. Windows-only path
/// uses GetProcessMemoryInfo via a tasklist fallback-free PEB read is overkill;
/// we shell out to `tasklist` for the current PID. Returns None on any failure.
fn current_rss_kb() -> Option<u64> {
    let pid = std::process::id();
    let out = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    // CSV row: "name","pid","session","session#","mem usage"
    let line = text.lines().find(|l| l.contains(&pid.to_string()))?;
    let mem_field = line.rsplit("\",\"").next()?;
    let digits: String = mem_field.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<u64>().ok()
}

fn walk_count(dir: &std::path::Path, n: &mut usize) {
    let rd = match std::fs::read_dir(dir) { Ok(r) => r, Err(_) => return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name();
        let ns = name.to_string_lossy();
        if p.is_dir() {
            if matches!(ns.as_ref(), "target" | "node_modules" | ".git" | ".elai" | ".elai_cc" | "__pycache__") { continue; }
            walk_count(&p, n);
        } else if p.is_file() {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "rs" | "py" | "ts" | "svelte" | "tsx" | "js" | "go" | "java") { *n += 1; }
        }
    }
}
