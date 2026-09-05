Phase 3H.1 decisions

What: SQLite schema + Rust types for session DAG in orchestrator/session_dag.rs.
Key decisions:
  - Per-message row schema (SPEC AMENDMENT #1) over BLOB-per-snapshot. Rationale: enables selective stub recovery without full BLOB deserialization. Required by 3H.3/3H.5.
  - DagMessageRole enum local to session_dag (session::MessageRole lacks Serialize/Deserialize). Will map to/from session::MessageRole when needed in later phases.
  - journal_mode = DELETE (not WAL). Single writer per session, no multi-reader pool needed. Avoids Windows WAL file linger.
  - blake3 for summary_hash (already in Cargo.toml, faster than sha256).
  - No new dependencies. All crates already in orchestrator Cargo.toml.
  - Schema version table with single-row check on open.
  - Integrity check (PRAGMA integrity_check) on open per R-3H-7.
Expected test outcome: 9 tests green, 0 warnings.
Unknowns: exact field types for findings/code_changes/tests_written BLOBs (deferred to 3H.2 save/load).
