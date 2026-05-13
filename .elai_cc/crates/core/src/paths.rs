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

// ---------------------------------------------------------------------------
// Path helpers — pure functions of `project_root`.
// ---------------------------------------------------------------------------

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
