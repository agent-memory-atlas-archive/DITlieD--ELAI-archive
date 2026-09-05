Phase 3J.WIRE.STUBS decisions

What: Replace PassAstEvaluator/EmptyDkgQuerier with real TreeSitterAstEvaluator and DkgQuerierAdapter.

Key decision: DkgQuerierAdapter connection strategy
- Chosen: open own read-only Connection via Connection::open_with_flags(path, SQLITE_OPEN_READ_ONLY)
- Rejected: accessing DkgEngine.conn (private field, would require pub(crate) change)
- Rejected: shared Arc<Mutex<Connection>> (unnecessary complexity for query-time usage)
- Rationale: read-only avoids WAL conflicts with DkgEngine's write connection; no API changes needed

Key decision: connection lifecycle
- Each query_reachability call opens fresh read-only connection, closes on drop
- Alternative: hold persistent connection in struct — rejected because DkgQuerierAdapter must handle db-not-yet-built case gracefully

Expected test outcome: cargo check --workspace PASS, existing 114 tests still GREEN

Unknowns:
- tree-sitter 0.26 Query::new API signature (need to verify exact error type)
- Whether tree-sitter-rust 0.24 LANGUAGE constant works with tree-sitter 0.26
