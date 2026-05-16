-- B-35 Phase A: artifact index state table.
-- Applied via include_str! in ProjectMetadataStore::open alongside v2 migration.
-- Idempotent: IF NOT EXISTS.

CREATE TABLE IF NOT EXISTS artifact_index_state (
    artifact_name      TEXT PRIMARY KEY,
    last_indexed_at    INTEGER NOT NULL,
    file_hashes_json   TEXT NOT NULL,
    chunk_count        INTEGER NOT NULL,
    status             TEXT NOT NULL
);
