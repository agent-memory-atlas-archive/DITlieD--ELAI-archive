Phase 3H.2 decisions: SessionDag snapshot save/load

What: 4 methods (save_snapshot, load_snapshot, latest_snapshot, parent_chain) + SnapshotNotFound error variant.
Key decisions:
- Transaction pattern: manual BEGIN IMMEDIATE/COMMIT/ROLLBACK with closure-based error handling. Consistent with 3H.1 new(). Avoids &mut self borrow conflict from Connection::transaction().
- Pre-validation: all u64->i64 conversions and serde_json serialization happen before BEGIN IMMEDIATE. Prevents opening a transaction only to fail on data validation.
- Closure guard: entire INSERT block wrapped in closure; any error from prepare_cached or individual INSERT triggers ROLLBACK. COMMIT failure also guarded.
- commit_hashes: serde_json TEXT serialization (Vec<String> -> JSON). Already available as workspace dep.
- parent_chain cycle detection: dual guard (HashSet<String> visited set + MAX_PARENT_DEPTH=1000 depth counter).
- nonneg_u64/safe_i64 helpers: symmetric read/write validation prevents silent wraparound on corrupt or overflow values.
Expected test outcome: 18/18 green (10 existing + 8 new).
Unknowns: parent_chain N+1 query pattern may need CTE optimization if depth > 100 is common (auditor WARN, deferred to 3H.4/3H.5).
