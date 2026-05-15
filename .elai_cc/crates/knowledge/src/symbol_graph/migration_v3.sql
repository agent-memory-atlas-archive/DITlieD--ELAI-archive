-- B-29 Phase 1: symbol graph schema migration (v1 -> v3).
-- v2 was reserved in the plan; this file implements v3 directly.
-- NOTE: column names use *_msgpack naming per spec contract, but blobs are
-- encoded/decoded via postcard (not rmp-serde) — postcard is already a
-- workspace dep, keeping zero new dependencies.
-- All statements are idempotent (CREATE TABLE IF NOT EXISTS / CREATE INDEX IF NOT EXISTS).

CREATE TABLE IF NOT EXISTS symbol_name_index (
    shard_key   TEXT NOT NULL,       -- 'a'..'z' or '_'
    symbol_name TEXT NOT NULL,
    refs_msgpack BLOB NOT NULL,      -- postcard-encoded Vec<SymbolRef { file, id }>
    PRIMARY KEY (shard_key, symbol_name)
);

CREATE TABLE IF NOT EXISTS symbol_reverse_index (
    shard_bucket INTEGER NOT NULL,   -- 0..255 (first byte of blake3(callee_file))
    callee_file  TEXT NOT NULL,
    caller_files_msgpack BLOB NOT NULL, -- postcard-encoded Vec<String>
    PRIMARY KEY (shard_bucket, callee_file)
);

CREATE TABLE IF NOT EXISTS symbol_file_payload (
    file         TEXT PRIMARY KEY,
    language     TEXT NOT NULL,
    content_hash TEXT NOT NULL,      -- blake3 hex, for staleness detection
    symbols_msgpack BLOB NOT NULL,   -- postcard-encoded Vec<SymbolNode>
    edges_msgpack   BLOB NOT NULL    -- postcard-encoded Vec<SymbolEdge>
);

CREATE TABLE IF NOT EXISTS symbol_graph_meta (
    project_id         TEXT PRIMARY KEY,
    symbol_count       INTEGER NOT NULL,
    edge_count         INTEGER NOT NULL,
    file_count         INTEGER NOT NULL,
    unresolved_edge_pct REAL NOT NULL,
    built_at           INTEGER NOT NULL,  -- ClockProvider::now_ms
    schema_version     INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_symbol_name_shard ON symbol_name_index(shard_key);
CREATE INDEX IF NOT EXISTS idx_symbol_reverse_shard ON symbol_reverse_index(shard_bucket);
