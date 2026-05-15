-- B-31 Phase 1: ProjectMetadataStore schema (project_metadata + file_hashes).
-- Lives in a per-project `metadata.db` (sibling of `dkg.db`) — its
-- `schema_version` lineage is independent of `dkg_meta.schema_version`
-- (decision DECISION-2, workflow/reasoning/B-31-decisions.md).
--
-- Idempotent: every CREATE uses IF NOT EXISTS so opening an existing DB is a
-- no-op. The CHECK constraint on `indexing_status` locks the 3-value enum at
-- the DB layer; extending it is destructive (gotchas.md:391 — drop+recreate).
-- file_hashes is keyed by (project_id, relative_path) to support multiple
-- project roots sharing a single metadata.db should that ever happen.

CREATE TABLE IF NOT EXISTS project_metadata (
    project_id          TEXT PRIMARY KEY,
    project_path        TEXT NOT NULL,
    indexed_files       INTEGER NOT NULL DEFAULT 0,
    total_files         INTEGER NOT NULL DEFAULT 0,
    indexing_status     TEXT NOT NULL CHECK(indexing_status IN ('in-progress','completed','cancelled')),
    last_checkpoint_at  INTEGER NOT NULL,
    batch_size          INTEGER NOT NULL DEFAULT 50,
    batches_processed   INTEGER NOT NULL DEFAULT 0,
    batches_total       INTEGER,
    schema_version      INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS file_hashes (
    project_id        TEXT NOT NULL,
    relative_path     TEXT NOT NULL,
    content_hash      TEXT NOT NULL,
    last_indexed_at   INTEGER NOT NULL,
    PRIMARY KEY (project_id, relative_path)
);

CREATE INDEX IF NOT EXISTS idx_file_hashes_project ON file_hashes(project_id);
