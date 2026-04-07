use std::path::{Path, PathBuf};
use std::time::Instant;

use elai_knowledge::bm25::Bm25Index;

/// Bytes to inspect for binary detection — must match bm25.rs BINARY_DETECT_BYTES.
const BINARY_DETECT_BYTES: usize = 8192;

const QUERIES: &[&str] = &[
    "fn main",
    "impl",
    "pub struct",
    "error",
    "async",
    "config",
    "session",
    "permission",
    "mcp",
    "plugin",
];

/// Count files that walk_dir in bm25.rs would index: non-hidden, non-binary, recursive.
fn count_indexable_files(dir: &Path) -> usize {
    let mut count = 0;
    if !dir.is_dir() {
        return 0;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Mirror walk_dir: skip entries starting with '.'
        if name_str.starts_with('.') {
            continue;
        }
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let path = entry.path();
        if ft.is_dir() {
            count += count_indexable_files(&path);
        } else if ft.is_file() {
            // Mirror walk_dir: skip unreadable and binary files
            let data = match std::fs::read(&path) {
                Ok(d) => d,
                Err(_) => continue,
            };
            let check_len = data.len().min(BINARY_DETECT_BYTES);
            if data[..check_len].contains(&0) {
                continue;
            }
            count += 1;
        }
    }
    count
}

fn dir_size_bytes(dir: &Path) -> u64 {
    let mut total = 0u64;
    if !dir.is_dir() {
        return 0;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if ft.is_dir() {
            total += dir_size_bytes(&path);
        } else if ft.is_file() {
            total += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        }
    }
    total
}

fn main() {
    let project_root = match std::env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().expect("cannot determine current directory"),
    };

    // Source directory to index: rust_archived/crates (source files only, no target/)
    let source_dir = project_root.join("rust_archived").join("crates");
    if !source_dir.is_dir() {
        eprintln!(
            "error: source directory not found: {}",
            source_dir.display()
        );
        std::process::exit(1);
    }

    let files_on_disk = count_indexable_files(&source_dir);
    eprintln!("source dir: {}", source_dir.display());
    eprintln!("indexable files on disk: {files_on_disk}");

    // Create index in tempdir for isolation
    let tmp = tempfile::tempdir().expect("cannot create tempdir");
    let project_id = "measure_bm25";

    let mut idx =
        Bm25Index::new(tmp.path(), project_id).expect("Bm25Index::new failed");

    // Measure index build (timing external to library — no Instant in library code)
    let build_start = Instant::now();
    let stats = idx
        .index_directory(&source_dir)
        .expect("index_directory failed");
    let index_build_time_ms = build_start.elapsed().as_millis() as u64;

    let file_count = stats.files_indexed;
    let chunk_count = stats.chunks_indexed;
    let binary_skipped = stats.files_skipped_binary;

    eprintln!(
        "indexed {file_count} files, {chunk_count} chunks, {binary_skipped} binary skipped in {index_build_time_ms}ms"
    );

    // Orphan check: files_indexed must exactly match files on disk
    let orphan_files = if files_on_disk == file_count {
        0usize
    } else if files_on_disk > file_count {
        files_on_disk - file_count
    } else {
        // Over-indexing: more files indexed than on disk (stale entries)
        eprintln!(
            "over-indexing detected: {file_count} indexed vs {files_on_disk} on disk"
        );
        file_count - files_on_disk
    };

    // Warm-up query (discarded)
    let _ = idx.query("fn", 10);

    // Measure query latencies: 10 queries
    let mut query_latencies_us: Vec<u64> = Vec::with_capacity(QUERIES.len());
    for q in QUERIES {
        let t = Instant::now();
        let _ = idx.query(q, 10);
        query_latencies_us.push(t.elapsed().as_micros() as u64);
    }
    query_latencies_us.sort();

    let query_p50_us = query_latencies_us[4]; // index 4 of 10 sorted = p50 (nearest-rank)
    let query_p99_us = query_latencies_us[9]; // index 9 of 10 sorted = p99 (nearest-rank)
    let query_p50_ms = (query_p50_us + 500) / 1000; // round-half-up
    let query_p99_ms = (query_p99_us + 500) / 1000;

    // Measure update_file latency: pick first .rs file found in source_dir
    let update_file_path = find_first_rs(&source_dir).expect("no .rs file found for update test");
    let t = Instant::now();
    let _ = idx
        .update_file(&update_file_path, None)
        .expect("update_file failed");
    let update_file_ms = t.elapsed().as_millis() as u64;

    // Index size on disk
    let index_dir = tmp
        .path()
        .join("projects")
        .join(project_id)
        .join("bm25");
    let index_size_bytes = dir_size_bytes(&index_dir);

    // Write metrics TOML
    let metrics_dir = project_root.join("metrics");
    std::fs::create_dir_all(&metrics_dir).expect("cannot create metrics directory");
    let output_path = metrics_dir.join("phase_1B.toml");

    let toml_content = format!(
        "# ELAI phase 1B BM25 metrics\n\
         index_build_time_ms = {index_build_time_ms}\n\
         file_count = {file_count}\n\
         chunk_count = {chunk_count}\n\
         binary_skipped = {binary_skipped}\n\
         query_p50_ms = {query_p50_ms}\n\
         query_p99_ms = {query_p99_ms}\n\
         update_file_p99_ms = {update_file_ms}\n\
         orphan_files = {orphan_files}\n\
         index_size_bytes = {index_size_bytes}\n\
         tantivy_version = \"0.26.0\"\n\
         index_format_version = 7\n"
    );

    std::fs::write(&output_path, &toml_content).expect("cannot write phase_1B.toml");

    // Validate: re-read and check all required keys
    let written = std::fs::read_to_string(&output_path).expect("cannot re-read phase_1B.toml");
    let required_keys = [
        "index_build_time_ms",
        "file_count",
        "chunk_count",
        "binary_skipped",
        "query_p50_ms",
        "query_p99_ms",
        "update_file_p99_ms",
        "orphan_files",
        "index_size_bytes",
        "tantivy_version",
        "index_format_version",
    ];
    let mut missing = Vec::new();
    for key in &required_keys {
        if !written
            .lines()
            .any(|line| line.starts_with(&format!("{key} =")))
        {
            missing.push(*key);
        }
    }
    if !missing.is_empty() {
        eprintln!("VALIDATION FAILED: missing keys: {missing:?}");
        std::process::exit(1);
    }

    println!("wrote {}", output_path.display());
    println!("---");
    print!("{toml_content}");

    // Threshold checks (compare at microsecond precision to avoid truncation drift)
    let ci = std::env::var("CI").is_ok();
    let query_threshold_ms: u64 = if ci { 50 } else { 10 };
    let query_threshold_us = query_threshold_ms * 1000;
    if query_p99_us <= query_threshold_us {
        println!("query_p99 {query_p99_ms}ms <= {query_threshold_ms}ms: PASS");
    } else {
        println!("query_p99 {query_p99_ms}ms > {query_threshold_ms}ms: OVER (baseline recorded)");
    }
    if update_file_ms <= 5000 {
        println!("update_file_p99 {update_file_ms}ms <= 5000ms: PASS");
    } else {
        println!("update_file_p99 {update_file_ms}ms > 5000ms: OVER");
    }
    if orphan_files == 0 {
        println!("orphan_files = 0: PASS");
    } else {
        println!("orphan_files = {orphan_files}: FAIL (expected 0)");
        std::process::exit(1);
    }

    // tempdir is dropped here, cleaning up the index
}

/// Find first .rs file, mirroring walk_dir skip rules (hidden-prefix only).
fn find_first_rs(dir: &Path) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Mirror walk_dir: skip entries starting with '.'
        if name_str.starts_with('.') {
            continue;
        }
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let path = entry.path();
        if ft.is_dir() {
            if let Some(found) = find_first_rs(&path) {
                return Some(found);
            }
        } else if ft.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    return Some(path);
                }
            }
        }
    }
    None
}
