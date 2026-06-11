//! B-1 Path Unification: single source of truth for ELAI on-disk path layout.
//!
//! Every ELAI crate that needs to locate the DKG database, BM25 index,
//! vector index, sessions dir, metrics dir, ONNX models dir, or rules
//! database MUST go through this module. String literals like `".elai/"`
//! or `".elai_cc/dkg.db"` are forbidden in non-test source.
//!
//! Helpers are pure path computations — zero IO, zero allocation beyond
//! the returned `PathBuf`, no panics, no `unwrap`/`expect`.
//!
//! Layout (relative to `project_root`):
//!
//! ```text
//! <project_root>/
//!   .elai_cc/
//!     indexes/
//!       dkg.db                  -> dkg_db_path
//!       vector_index.json       -> vectors_db_path
//!       (BM25 SQLite files)     -> bm25_dir_path
//!     sessions/                 -> sessions_dir_path
//!     metrics/                  -> metrics_dir_path
//!     models/                   -> models_dir_path  (callers append /onnx/)
//!     rules.db                  -> rules_db_path
//! ```
//!
//! Pre-existing `.elai/` directories on user machines are NOT auto-deleted
//! (per B-1 spec L94-98 and B-1-decisions.md). Bootstrap may emit a
//! deprecation warning instead.

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public constants — single source of truth for path-segment names.
// ---------------------------------------------------------------------------

/// Top-level data directory under the project root.
pub const ELAI_DATA_DIR_NAME: &str = ".elai_cc";

/// Subdirectory under `ELAI_DATA_DIR_NAME` that holds DKG + BM25 + vector
/// indexes.
pub const INDEXES_DIR_NAME: &str = "indexes";

/// Filename of the DKG SQLite database inside the indexes dir.
pub const DKG_DB_FILENAME: &str = "dkg.db";

/// Filename of the persisted vector index (single JSON file per project).
pub const VECTORS_FILE_NAME: &str = "vector_index.json";

/// 1C-R2: filename of the dual-vector embedding cache (SQLite) inside the
/// indexes dir. Sibling of [`VECTORS_FILE_NAME`]; the cache stores
/// per-content-hash dual vectors so a re-index reuses prior embeddings and
/// `VectorCache::invalidate_file` can evict exactly the rows a rewritten
/// file produced.
pub const VECTOR_CACHE_DB_FILENAME: &str = "vector_cache.db";

/// Subdirectory holding session persistence artifacts.
pub const SESSIONS_DIR_NAME: &str = "sessions";

/// Subdirectory holding emitted metrics (Phase 3J convention).
pub const METRICS_DIR_NAME: &str = "metrics";

/// Subdirectory holding ONNX/local model artifacts. Callers that need a
/// specific subtree (e.g. `onnx/`) append it themselves — the helper
/// returns the parent.
pub const MODELS_DIR_NAME: &str = "models";

/// Filename of the project rules SQLite database (was `.elai/rules.db`
/// pre-B-1; new canonical location is `.elai_cc/rules.db`).
pub const RULES_DB_FILENAME: &str = "rules.db";

/// Phase B4 — Artifacts Pipeline. Subdirectory under `ELAI_DATA_DIR_NAME`
/// that holds task-step artifacts (bytes + JSON manifests). Layout:
/// `<root>/.elai_cc/artifacts/<task_id>/<step_id>/<artifact_id>.<ext>` and
/// `<artifact_id>.json` sit side-by-side in the same step directory.
pub const ARTIFACTS_DIR_NAME: &str = "artifacts";

/// Manifest file extension for ArtifactStore-emitted JSON descriptors.
/// Kept as a constant so callers cannot accidentally drift to `.toml` or
/// `.yaml`.
pub const ARTIFACT_MANIFEST_EXT: &str = "json";

/// E-5 — user-modeling memory store. Subdirectory under
/// `ELAI_DATA_DIR_NAME` that holds the per-project `memory_facts` sqlite
/// database (bi-temporal episodic store; see
/// `crates/memory/src/store.rs`). Layout: `<root>/.elai_cc/memory/memory_facts.sqlite`.
pub const MEMORY_DIR_NAME: &str = "memory";

/// Filename of the per-project `memory_facts` sqlite database.
pub const MEMORY_FACTS_DB_FILENAME: &str = "memory_facts.sqlite";

/// D-6 — held-out task store. Subdirectory under `ELAI_DATA_DIR_NAME` that
/// holds date-anchored JSONL fixture directories + the rusqlite-backed
/// index. Layout: `<root>/.elai_cc/held-out/{YYYY-MM}/*.jsonl` for raw
/// task fixtures and `<root>/.elai_cc/held-out/held_out.sqlite` for the
/// indexed task store (mirrors the memory crate's per-project sqlite
/// pattern; see `MEMORY_FACTS_DB_FILENAME`). LiveCodeBench
/// (arxiv:2403.07974, T1) is the load-bearing time-windowed-split +
/// 9-gram contamination-filter pattern this directory backs.
pub const HELD_OUT_DIR_NAME: &str = "held-out";

/// Filename of the per-project `held_out_tasks` sqlite index. Sibling of
/// the JSONL fixture directories under `HELD_OUT_DIR_NAME`.
pub const HELD_OUT_DB_FILENAME: &str = "held_out.sqlite";

// ---------------------------------------------------------------------------
// Path helpers — pure functions of `project_root`.
// ---------------------------------------------------------------------------

/// Filename of the per-project index manifest emitted by `IndexManifest`
/// (C-pack C2). Lives inside `INDEXES_DIR_NAME`.
pub const INDEX_MANIFEST_FILENAME: &str = "manifest.json";

/// `<project_root>/.elai_cc/indexes`
///
/// Canonical home for all on-disk indexes (DKG SQLite, vector JSON, BM25
/// files, the C-pack `manifest.json`). Identical to `bm25_dir_path` by
/// construction; kept as a separate helper so non-BM25 callers (e.g. the
/// IndexManifest loader/saver) don't tunnel through a BM25-flavoured name.
#[inline]
pub fn indexes_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(INDEXES_DIR_NAME)
}

/// `<project_root>/.elai_cc/indexes/manifest.json`
///
/// C-pack C2: single source of truth for index manifest location. Callers
/// must use this helper rather than building the path piecewise — the
/// path_unification contract test forbids `".elai/"` and ad-hoc segment
/// concatenation in non-test source.
#[inline]
pub fn index_manifest_path(project_root: &Path) -> PathBuf {
    indexes_dir_path(project_root).join(INDEX_MANIFEST_FILENAME)
}

/// `<project_root>/.elai_cc/indexes/dkg.db`
#[inline]
pub fn dkg_db_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(INDEXES_DIR_NAME)
        .join(DKG_DB_FILENAME)
}

/// `<project_root>/.elai_cc/indexes`
///
/// BM25 lives in the same indexes directory as the DKG database; the BM25
/// crate manages its own subfile layout.
#[inline]
pub fn bm25_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(INDEXES_DIR_NAME)
}

/// `<project_root>/.elai_cc/indexes/vector_index.json`
#[inline]
pub fn vectors_db_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(INDEXES_DIR_NAME)
        .join(VECTORS_FILE_NAME)
}

/// `<project_root>/.elai_cc/indexes/vector_cache.db`
///
/// 1C-R2: the dual-vector embedding cache lives in the same indexes
/// directory as the vector index and DKG/BM25 dbs, so it is discovered and
/// cleaned alongside them. Derived the same way as [`vectors_db_path`] so
/// the cache never escapes the project's data dir.
#[inline]
pub fn vector_cache_db_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(INDEXES_DIR_NAME)
        .join(VECTOR_CACHE_DB_FILENAME)
}

/// `<project_root>/.elai_cc/sessions`
#[inline]
pub fn sessions_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(SESSIONS_DIR_NAME)
}

/// `<project_root>/.elai_cc/metrics`
#[inline]
pub fn metrics_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(METRICS_DIR_NAME)
}

/// `<project_root>/.elai_cc/models`
///
/// Callers append `onnx/<model>.onnx` themselves — see B-1-decisions.md
/// (helper deliberately does NOT bake `onnx/` so the cross-encoder,
/// SPLADE, GraphCodeBERT, and Granite callers can each pick their own
/// subdir layout).
#[inline]
pub fn models_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(MODELS_DIR_NAME)
}

/// `<project_root>/.elai_cc/rules.db`
#[inline]
pub fn rules_db_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(RULES_DB_FILENAME)
}

// ---------------------------------------------------------------------------
// Phase B4 — Artifacts Pipeline helpers.
//
// `ArtifactStore::publish` (orchestrator/src/artifacts.rs) routes every
// on-disk write through these three functions; any other path-construction
// site for artifacts is forbidden by the same rules that forbid `".elai/"`
// literals (see path_unification contract test).
// ---------------------------------------------------------------------------

/// `<project_root>/.elai_cc/artifacts`
#[inline]
pub fn artifacts_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(ARTIFACTS_DIR_NAME)
}

/// `<project_root>/.elai_cc/artifacts/<task_id>/<step_id>/<artifact_id>.<ext>`
///
/// All four ID segments are joined as-is — callers are responsible for
/// validating that task/step/artifact IDs are filesystem-safe (no `/`, no
/// `..`, no NUL). The orchestrator's `ContextArtifact` constructor performs
/// that validation; bypassing it is a contract violation.
#[inline]
pub fn artifact_path(
    project_root: &Path,
    task_id: &str,
    step_id: &str,
    artifact_id: &str,
    ext: &str,
) -> PathBuf {
    artifacts_dir_path(project_root)
        .join(task_id)
        .join(step_id)
        .join(format!("{artifact_id}.{ext}"))
}

/// `<project_root>/.elai_cc/memory`
///
/// Parent directory for the E-5 per-project memory store. Currently
/// holds a single sqlite file ([`memory_facts_db_path`]); reserved for
/// future per-user / per-tier extensions.
#[inline]
pub fn memory_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(MEMORY_DIR_NAME)
}

/// `<project_root>/.elai_cc/memory/memory_facts.sqlite`
///
/// Canonical location for the E-5 `memory_facts` bi-temporal episodic
/// store. Per-project scope (matches the rest of `.elai_cc/`). The
/// per-user global tier lives at [`user_profile_db_path`] (see below);
/// callers that need cross-project facts consult the per-user store
/// AFTER the per-project store.
#[inline]
pub fn memory_facts_db_path(project_root: &Path) -> PathBuf {
    memory_dir_path(project_root).join(MEMORY_FACTS_DB_FILENAME)
}

/// E-5 session-2 — per-user global memory tier directory.
///
/// On Windows the resolver returns `%USERPROFILE%\.elai_user\`; on Unix
/// it returns `$HOME/.elai_user/`. Returns `None` when neither
/// environment variable is set (caller falls back to per-project tier
/// only).
///
/// IMPORTANT: this is the FACT store (per-user `memory_facts.sqlite`
/// + projects index), NOT a rule store. The CLAUDE.md "memory directory
/// is read-only for rules/feedback" prohibition stands — that rule
/// applies to `C:\Users\<u>\.claude\projects\...` user-rule memory, not
/// to this E-5 fact tier.
#[inline]
pub fn user_profile_dir() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("USERPROFILE") {
        if !v.is_empty() {
            return Some(PathBuf::from(v).join(USER_PROFILE_DIR_NAME));
        }
    }
    if let Ok(v) = std::env::var("HOME") {
        if !v.is_empty() {
            return Some(PathBuf::from(v).join(USER_PROFILE_DIR_NAME));
        }
    }
    None
}

/// E-5 session-2 — per-user global memory store path.
/// Returns `None` when [`user_profile_dir`] returns `None`.
#[inline]
pub fn user_profile_db_path() -> Option<PathBuf> {
    user_profile_dir().map(|d| d.join(USER_PROFILE_DB_FILENAME))
}

/// E-5 session-2 — per-user projects index path.
///
/// JSON file listing every project the user has worked in. Used by the
/// per-user store's promotion gate (N>=2 distinct projects requirement)
/// to track which project_ids have contributed evidence for a candidate
/// global fact. Returns `None` when [`user_profile_dir`] returns `None`.
#[inline]
pub fn user_projects_index_path() -> Option<PathBuf> {
    user_profile_dir().map(|d| d.join(USER_PROJECTS_INDEX_FILENAME))
}

/// Per-user data directory name (under `%USERPROFILE%` / `$HOME`).
pub const USER_PROFILE_DIR_NAME: &str = ".elai_user";

/// Filename of the per-user global `memory_facts` sqlite database.
pub const USER_PROFILE_DB_FILENAME: &str = "profile.sqlite";

/// Filename of the per-user projects index (promotion-gate input).
pub const USER_PROJECTS_INDEX_FILENAME: &str = "projects.json";

/// `<project_root>/.elai_cc/held-out`
///
/// Parent directory for the D-6 held-out task store. Date-anchored
/// subdirectories (`{YYYY-MM}/`) hold the raw JSONL fixture files; the
/// `held_out.sqlite` index lives at the same level as the date dirs.
/// Time-windowed eligibility (LiveCodeBench-style) is enforced in
/// `crate::held_out::HeldOutStore`; this helper is pure path-math.
#[inline]
pub fn held_out_dir_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join(HELD_OUT_DIR_NAME)
}

/// `<project_root>/.elai_cc/held-out/held_out.sqlite`
///
/// Canonical location for the rusqlite-backed held-out task index. The
/// schema (table `held_out_tasks`) is created lazily by
/// `crate::held_out::HeldOutStore::open` on first use. The JSONL fixture
/// directories under the same parent are NOT indexed by this file — they
/// are the seed input that `HeldOutStore::ingest_fixture_dir` consumes.
#[inline]
pub fn held_out_db_path(project_root: &Path) -> PathBuf {
    held_out_dir_path(project_root).join(HELD_OUT_DB_FILENAME)
}

/// Sibling JSON manifest for the artifact returned by `artifact_path`.
/// Lives in the same step directory; extension is always
/// `ARTIFACT_MANIFEST_EXT` (`"json"`) — never inferred from the artifact's
/// own extension.
#[inline]
pub fn artifact_manifest_path(
    project_root: &Path,
    task_id: &str,
    step_id: &str,
    artifact_id: &str,
) -> PathBuf {
    artifacts_dir_path(project_root)
        .join(task_id)
        .join(step_id)
        .join(format!("{artifact_id}.{ext}", ext = ARTIFACT_MANIFEST_EXT))
}
