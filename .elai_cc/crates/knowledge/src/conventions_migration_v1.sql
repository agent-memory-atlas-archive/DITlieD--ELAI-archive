-- phase-3L Project Intelligence: convention metadata table (v1 baseline).
--
-- Stores the per-language convention snapshot produced by
-- `extract_conventions(&CorpusView)` so the PI Tier hooks (PiT1Hook /
-- PiT2Hook / PiT3Hook) can render CONVENTIONS blocks without re-walking
-- the corpus on every prompt build.
--
-- Companion to `rules.db` and `dkg.db`; lives in a sibling
-- `conventions.db` opened by callers via `Connection::open`. Schema lineage
-- is gated by `conventions_meta.schema_version` and the Rust-side constant
-- `CONVENTIONS_SCHEMA_VERSION` (knowledge/src/conventions.rs). The
-- `VERSION_GATE` predicate in `PiT2Hook::run` refuses to seed when on-disk
-- version is older and instead queues this migration via include_str!.
--
-- Idempotent: every CREATE uses IF NOT EXISTS so opening an existing
-- conventions.db is a no-op.

CREATE TABLE IF NOT EXISTS conventions_meta (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    schema_version  INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS conventions (
    project_id        TEXT NOT NULL,
    language          TEXT NOT NULL,
    category          TEXT NOT NULL CHECK(category IN ('functions','types','constants','modules','error_handling','module_organization','test_location')),
    convention_kind   TEXT NOT NULL CHECK(convention_kind IN ('STRONG','MODERATE','MIXED','UNKNOWN')),
    convention_value  TEXT NOT NULL DEFAULT '',
    sample_size       INTEGER NOT NULL DEFAULT 0,
    schema_version    INTEGER NOT NULL DEFAULT 1,
    generated_at      INTEGER NOT NULL,
    PRIMARY KEY (project_id, language, category)
);

CREATE INDEX IF NOT EXISTS idx_conventions_project
    ON conventions(project_id);

CREATE INDEX IF NOT EXISTS idx_conventions_lang
    ON conventions(language);

-- Seed the meta row so future migrations can compare numerically.
INSERT OR IGNORE INTO conventions_meta(key, value, schema_version)
    VALUES ('schema_version', '1', 1);
