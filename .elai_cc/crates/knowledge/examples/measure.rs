use std::path::PathBuf;
use std::time::Instant;

use elai_knowledge::DkgEngine;

fn main() {
    let project_root = match std::env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().expect("cannot determine current directory"),
    };

    if !project_root.is_dir() {
        eprintln!("error: project root is not a directory: {}", project_root.display());
        std::process::exit(1);
    }

    let db_dir = std::env::temp_dir().join("elai_measure");
    std::fs::create_dir_all(&db_dir).expect("cannot create temp db directory");
    let db_path = db_dir.join("measure.db");

    // Clean previous run
    let _ = std::fs::remove_file(&db_path);

    let created_at = 0i64; // Deterministic — measurement binary, not library code

    let mut engine = DkgEngine::new(&db_path, None, created_at)
        .expect("DkgEngine::new failed");

    // Measure build_graph
    let start = Instant::now();
    let graph = engine.build_graph(&project_root)
        .expect("build_graph failed");
    let build_time_ms = start.elapsed().as_millis() as u64;

    // Collect node/edge data
    let all_nodes = engine.nodes().expect("nodes() failed");
    let all_edges = engine.edges().expect("edges() failed");
    let failed = engine.failed_files().expect("failed_files() failed");

    let total_nodes = all_nodes.len();
    let total_edges = all_edges.len();
    let files_parsed = graph.files_parsed;
    let files_skipped = failed.len();

    let edges_imports = all_edges.iter().filter(|(_, _, t)| t == "imports").count();
    let edges_contains = all_edges.iter().filter(|(_, _, t)| t == "contains").count();
    let edges_implements = all_edges.iter().filter(|(_, _, t)| t == "implements").count();

    let signatures_populated = all_nodes
        .iter()
        .filter(|n| !n.signature.is_empty())
        .count();

    // Measure query latency: 10 node_by_id lookups
    let mut latencies_us: Vec<u64> = Vec::new();
    if !all_nodes.is_empty() {
        let step = if all_nodes.len() >= 10 { all_nodes.len() / 10 } else { 1 };
        let mut idx = 0;
        for _ in 0..10.min(all_nodes.len()) {
            let node_id = all_nodes[idx].node_id;
            let t = Instant::now();
            let _ = engine.node_by_id(node_id);
            latencies_us.push(t.elapsed().as_micros() as u64);
            idx += step;
            if idx >= all_nodes.len() {
                idx = 0;
            }
        }
    }
    latencies_us.sort();

    let query_p50_ms = if latencies_us.len() >= 2 {
        latencies_us[latencies_us.len() / 2] as f64 / 1000.0
    } else if latencies_us.len() == 1 {
        latencies_us[0] as f64 / 1000.0
    } else {
        0.0
    };
    let query_p99_ms = if !latencies_us.is_empty() {
        latencies_us[latencies_us.len() - 1] as f64 / 1000.0
    } else {
        0.0
    };

    // DB size
    let db_size_bytes = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // B-30: per-language node counts. The DB returns only languages that have
    // at least one row; we merge against the fixed known key set so the table
    // is deterministic across runs (no HashMap iteration order churn) and
    // languages absent from the fixture still appear with `0`.
    let known_languages: &[&str] = &[
        "rust", "python", "go", "typescript", "tsx", "svelte", "vue",
        "java", "csharp", "ruby", "javascript", "c", "cpp", "scala",
        "php", "swift", "bash",
    ];
    let per_lang = engine.node_counts_by_language().unwrap_or_default();
    let mut lang_counts: std::collections::BTreeMap<String, i64> =
        std::collections::BTreeMap::new();
    for &lang in known_languages {
        lang_counts.insert(lang.to_string(), 0i64);
    }
    for (lang, count) in &per_lang {
        lang_counts.insert(lang.clone(), *count);
    }

    // Build the [per_language_node_counts] table in the fixed alphabetical
    // order matching the existing on-disk file (rust, python, go, ...).
    let mut per_language_table = String::from(
        "\n# Per-language node counts (B-30 Pass D schema addition).\n\
         # Populated by `cargo run --example measure` from \
         `SELECT language, COUNT(*) FROM dkg_nodes GROUP BY language`.\n\
         [per_language_node_counts]\n",
    );
    let ordered_keys: &[&str] = &[
        "rust", "python", "go", "typescript", "tsx", "svelte", "vue",
        "java", "csharp", "ruby", "javascript", "c", "cpp", "scala",
        "php", "swift", "bash",
    ];
    for &key in ordered_keys {
        let count = lang_counts.get(key).copied().unwrap_or(0);
        per_language_table.push_str(&format!("{key} = {count}\n"));
    }

    // Write TOML
    let toml_content = format!(
        "# ELAI phase 1A DKG metrics — 2026-04-07\n\
         build_time_ms = {build_time_ms}\n\
         query_p50_ms = {query_p50_ms:.3}\n\
         query_p99_ms = {query_p99_ms:.3}\n\
         db_size_bytes = {db_size_bytes}\n\
         total_nodes = {total_nodes}\n\
         total_edges = {total_edges}\n\
         files_parsed = {files_parsed}\n\
         files_skipped = {files_skipped}\n\
         edges_imports = {edges_imports}\n\
         edges_contains = {edges_contains}\n\
         edges_implements = {edges_implements}\n\
         signatures_populated = {signatures_populated}\n\
         {per_language_table}"
    );

    // Determine output path
    let metrics_dir = project_root.join("metrics");
    std::fs::create_dir_all(&metrics_dir).expect("cannot create metrics directory");
    let output_path = metrics_dir.join("phase_1A.toml");

    std::fs::write(&output_path, &toml_content).expect("cannot write phase_1A.toml");

    // Validate: re-read and check all 12 keys present
    let written = std::fs::read_to_string(&output_path).expect("cannot re-read phase_1A.toml");
    let required_keys = [
        "build_time_ms", "query_p50_ms", "query_p99_ms", "db_size_bytes",
        "total_nodes", "total_edges", "files_parsed", "files_skipped",
        "edges_imports", "edges_contains", "edges_implements", "signatures_populated",
    ];
    let mut missing = Vec::new();
    for key in &required_keys {
        if !written.lines().any(|line| line.starts_with(&format!("{key} ="))) {
            missing.push(*key);
        }
    }
    if !missing.is_empty() {
        eprintln!("VALIDATION FAILED: missing keys: {:?}", missing);
        std::process::exit(1);
    }
    if !written.contains("[per_language_node_counts]") {
        eprintln!("VALIDATION FAILED: missing [per_language_node_counts] table");
        std::process::exit(1);
    }
    let non_zero_langs = ordered_keys
        .iter()
        .filter(|&&k| {
            written
                .lines()
                .any(|line| line.starts_with(&format!("{k} = ")) && !line.starts_with(&format!("{k} = 0")))
        })
        .count();
    println!("per_language_node_counts: {non_zero_langs} languages with nodes>0");

    println!("wrote {}", output_path.display());
    println!("---");
    print!("{toml_content}");

    // Compare against <15s target from knowledge.txt
    let target_ms: u64 = 15_000;
    if build_time_ms <= target_ms {
        println!("build_time {build_time_ms}ms <= {target_ms}ms target: PASS");
    } else {
        println!("build_time {build_time_ms}ms > {target_ms}ms target: OVER (baseline recorded)");
    }

    // Cleanup temp db
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_dir(&db_dir);
}
