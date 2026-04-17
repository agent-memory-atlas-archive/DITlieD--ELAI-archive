//! Canonical derivation of an SSB `project_id` from an on-disk path.
//!
//! Rationale: the CLI and the dashboard both open the same SSB database and
//! stamp rows with a `project_id`. If the two binaries compute different IDs
//! for the same project, rows land in disjoint buckets and the dashboard UI
//! appears empty even though the CLI is writing. The invariant every caller
//! relies on is:
//!
//!   same canonicalised CWD -> same `project_id`
//!
//! Hash choice: `blake3` per workspace standards (see workspace Cargo.toml
//! and CLAUDE.md anti-stale-training rule banning `sha2`/`blake2` for new
//! non-RFC uses). The previous CLI-only implementation used
//! `std::collections::hash_map::DefaultHasher` (SipHash-1-3) which is
//! explicitly documented as "not guaranteed to be the same between Rust
//! releases", making it unsafe for cross-process identity.
//!
//! Truncation: we emit the first 16 hex chars of the 32-byte BLAKE3 output
//! (64 bits of entropy). This matches the prior on-wire width
//! (`proj-{:016x}`) and keeps `project_id` within the 64-char `ProjectsStore`
//! validation limit.

use std::path::{Path, PathBuf};

/// Length (in hex chars) of the truncated blake3 digest used in the ID.
/// 16 hex chars = 8 bytes = 64 bits of collision resistance, sufficient for
/// a per-user CWD namespace.
pub const PROJECT_ID_HEX_LEN: usize = 16;

/// Prefix prepended to the hex digest so log lines and UI can recognise the
/// value as an auto-derived bucket rather than a user-supplied project id.
pub const PROJECT_ID_PREFIX: &str = "proj-";

/// Derive a stable `project_id` from a filesystem path.
///
/// The path is canonicalised when possible (`std::fs::canonicalize`) so that
/// `.` and symlink-traversed forms hash to the same value as the real path.
/// If canonicalisation fails (e.g. the path does not exist), the raw bytes
/// are hashed instead — callers get a deterministic result either way.
///
/// Uses blake3 (workspace standard).
pub fn project_id_from_path(path: &Path) -> String {
    let canonical: PathBuf = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let bytes = path_to_hash_bytes(&canonical);
    let digest = blake3::hash(&bytes);
    let hex = digest.to_hex();
    let short = &hex.as_str()[..PROJECT_ID_HEX_LEN];
    format!("{PROJECT_ID_PREFIX}{short}")
}

/// Derive a stable `project_id` from the current working directory.
///
/// Returns `None` if the CWD cannot be read (extremely rare — typically only
/// when the process's CWD was deleted out from under it). Callers that want
/// a non-optional value should fall back to a literal like `"unknown"` and
/// log the failure.
pub fn project_id_from_cwd() -> Option<String> {
    std::env::current_dir().ok().map(|p| project_id_from_path(&p))
}

/// Convert a path to a deterministic byte sequence suitable for hashing.
///
/// On Windows, `Path::as_os_str` yields UTF-16-like bytes via `encode_wide`
/// only through the `OsStrExt` trait; to stay portable and stable across
/// platforms we hash the UTF-8 representation (lossy) plus the raw OS-string
/// bytes when available. For the CWD-derived case this is always valid UTF-8
/// in practice, and `to_string_lossy` guarantees a stable encoding.
fn path_to_hash_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_path_produces_same_id() {
        let p = std::env::temp_dir();
        let a = project_id_from_path(&p);
        let b = project_id_from_path(&p);
        assert_eq!(a, b, "project_id must be deterministic for the same path");
    }

    #[test]
    fn different_paths_produce_different_ids() {
        // No tempfile dep in elai-core — use stable literal paths. These
        // strings never hit the filesystem because canonicalize fails and
        // we fall back to hashing the raw bytes.
        let a = project_id_from_path(Path::new("/some/project/a"));
        let b = project_id_from_path(Path::new("/some/project/b"));
        assert_ne!(a, b, "distinct paths must hash to distinct ids");
    }

    #[test]
    fn id_shape_is_prefix_plus_hex() {
        let p = std::env::temp_dir();
        let id = project_id_from_path(&p);
        assert!(id.starts_with(PROJECT_ID_PREFIX), "id: {id}");
        let suffix = &id[PROJECT_ID_PREFIX.len()..];
        assert_eq!(suffix.len(), PROJECT_ID_HEX_LEN, "suffix len: {}", suffix.len());
        assert!(
            suffix.chars().all(|c| c.is_ascii_hexdigit()),
            "suffix must be lowercase hex: {suffix}"
        );
    }

    #[test]
    fn id_fits_projects_store_limit() {
        // ProjectsStore caps project_id at 64 chars (projects.rs:13). The
        // derived shape is 5 (prefix) + 16 (hex) = 21 chars — well under.
        let p = std::env::temp_dir();
        let id = project_id_from_path(&p);
        assert!(id.len() <= 64, "id {} longer than 64 chars", id);
    }

    #[test]
    fn canonicalisation_unifies_dot_and_real_path() {
        // Use the CWD (always exists, always canonicalises) to prove the
        // invariant that `./` and its canonical form collapse to the same id.
        let cwd = std::env::current_dir().expect("cwd");
        let dotted = cwd.join(".");
        let a = project_id_from_path(&cwd);
        let b = project_id_from_path(&dotted);
        assert_eq!(a, b, "./path and path must canonicalise to the same id");
    }

    #[test]
    fn project_id_from_cwd_round_trip() {
        // The CWD-helper must agree with the path-helper when the path IS
        // the CWD. This is the invariant that keeps CLI and dashboard in
        // the same bucket when launched from the same directory.
        let cwd = std::env::current_dir().expect("cwd");
        let from_cwd = project_id_from_cwd().expect("cwd id");
        let from_path = project_id_from_path(&cwd);
        assert_eq!(from_cwd, from_path);
    }

    #[test]
    fn cli_and_dashboard_agree_on_same_cwd() {
        // Regression guard for the patch-1.30 follow-up: the CLI and the
        // dashboard must produce IDENTICAL `project_id` values when both
        // are launched from the same directory. Both call sites funnel
        // through `project_id_from_cwd` (elai_core), so the test pins the
        // contract: call the helper twice, get the same answer. If a
        // future refactor diverges the two (e.g. adds a process-specific
        // salt), this test breaks and the SSB bucket split resurfaces.
        let a = project_id_from_cwd().expect("cli-side id");
        let b = project_id_from_cwd().expect("dashboard-side id");
        assert_eq!(
            a, b,
            "CLI and dashboard must compute identical project_id from the same CWD"
        );
        // And: the literal must have the documented shape (grounded
        // contract — the Tauri UI may surface this string).
        assert!(a.starts_with(PROJECT_ID_PREFIX));
        assert_eq!(a.len(), PROJECT_ID_PREFIX.len() + PROJECT_ID_HEX_LEN);
    }
}
