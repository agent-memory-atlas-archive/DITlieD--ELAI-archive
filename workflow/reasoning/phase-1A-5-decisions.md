Phase 1A.5 MEASURE decisions

Approach: Option B — Rust example binary at crates/knowledge/examples/measure.rs.
Cargo auto-detects examples/ directory, no Cargo.toml edit needed.
Shell script rejected: Windows platform fragility (bash path separators, stat syntax).

TOML serialization: manual format! macro, not toml crate.
Reason: toml crate not in Cargo.toml deps, and Cargo.toml is locked by session s-523956.

Metrics source mapping:
  build_time_ms        <- std::time::Instant around build_graph() call
  query_p50_ms         <- sort 10 node_by_id() timings, take median
  query_p99_ms         <- sort 10 node_by_id() timings, take index 9
  db_size_bytes        <- std::fs::metadata(db_path).len()
  total_nodes          <- nodes().len()
  total_edges          <- edges().len()
  files_parsed         <- DependencyGraph.files_parsed
  files_skipped        <- failed_files().len()
  edges_imports        <- edges().iter().filter(kind == "imports").count()
  edges_contains       <- edges().iter().filter(kind == "contains").count()
  edges_implements     <- edges().iter().filter(kind == "implements").count()
  signatures_populated <- nodes().iter().filter(signature non-empty).count()

Entropy quarantine: does not apply to examples/ (grep confirmed zero ClockProvider hits in knowledge crate).
std::time::Instant is safe for wall-clock measurement in example binary.

Output path: .elai_cc/metrics/phase_1A.toml (flat key=value, matches phase_0 convention).

Project root: passed as CLI arg (argv[1]), defaults to current directory.
DB path: tempfile in system temp dir (measurement is ephemeral, not persisted).

Unknowns:
  Query latency with only 10 samples has high variance. Acceptable for baseline.
  Build time depends on .elai_cc/ codebase size (~69 .rs files per phase_0_baseline.toml).

Expected test outcome: measure binary runs, writes valid TOML with 12 keys, all values > 0 except possibly files_skipped and edges_implements.
