Phase 1B.5 decisions

What: Add elapsed_ms timing to index_directory, create measure_bm25.rs example binary that indexes rust_archived/crates, measures query/update latency, validates zero-orphan invariant, writes metrics/phase_1B.toml.

Key decisions:
- Timing: std::time::Instant wrapping walk_dir+commit+reload in index_directory. No new deps.
- Measurement binary: examples/measure_bm25.rs (matches existing measure.rs pattern for DKG).
- Target directory: .elai_cc/rust_archived/crates (73 indexable text files). walk_dir only skips hidden-prefixed entries, not target/. This dir has no target/ subdir so no exclusion needed. walk_dir indexes all non-binary text files (Cargo.toml, .sh, etc.), not just .rs.
- p50/p99 method: 10 query samples sorted, index 4 = p50, index 9 = p99 (nearest-rank for n=10). Round-half-up ms conversion. 1 warm-up query discarded before measurement. Threshold comparison at microsecond precision.
- Orphan check: walk source dir counting .rs files on disk, compare against files_indexed from IndexStats. orphan_files = disk_count - files_indexed. Must be 0.
- update_file latency: pick one indexed file, measure delete+re-index via update_file. Single sample reported as update_file_p99_ms per spec (not a true percentile — single-sample worst-case proxy). File selection is non-deterministic (fs ordering).
- TOML output: format! string, no toml crate. All required fields per spec.
- Index in tempdir: tempfile::tempdir() for isolation, auto-cleanup on drop.

Expected test outcome: 27 existing tests GREEN, 0 warnings. Measurement binary produces phase_1B.toml with all required fields. query_p99_ms < 10ms (dev), index build < 5s for 63 files.

Unknowns: actual latency on dev machine (Windows 11). tantivy cold-start overhead on first query.
