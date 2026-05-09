//! B-1 contract tests for `elai_core::paths`.
//!
//! Every helper is a pure function of `project_root`. Tests:
//! 1. Each helper produces the spec-canonical layout (`.elai_cc/...`).
//! 2. Helpers are idempotent (called twice, same `PathBuf`).
//! 3. Helpers compose correctly with both Windows-style (`C:\proj`) and
//!    Unix-style (`/proj`) project roots — verified via `Path::ends_with`
//!    on platform-native `MAIN_SEPARATOR`-joined components rather than
//!    hard-coded `/` or `\`, so the suite passes on either OS.
//! 4. The leaf component of each helper matches its declared `*_NAME` /
//!    `*_FILENAME` constant (regression guard against silent renames).

use std::path::{Component, MAIN_SEPARATOR, Path, PathBuf};

use elai_core::paths::{
    bm25_dir_path, dkg_db_path, metrics_dir_path, models_dir_path, rules_db_path,
    sessions_dir_path, vectors_db_path, DKG_DB_FILENAME, ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME,
    METRICS_DIR_NAME, MODELS_DIR_NAME, RULES_DB_FILENAME, SESSIONS_DIR_NAME, VECTORS_FILE_NAME,
};

/// Build a project-root `PathBuf` from `Component::Normal` segments using
/// the host platform's `MAIN_SEPARATOR`. Avoids hard-coded slashes.
fn join_root(parts: &[&str]) -> PathBuf {
    let mut s = String::new();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            s.push(MAIN_SEPARATOR);
        }
        s.push_str(p);
    }
    PathBuf::from(s)
}

/// Collect normal-component names (filters out RootDir / Prefix / CurDir
/// so the assertion is portable across `C:\proj` and `/proj`).
fn normal_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Constants are stable string values (these are part of the public API).
// ---------------------------------------------------------------------------

#[test]
fn constants_have_expected_values() {
    assert_eq!(ELAI_DATA_DIR_NAME, ".elai_cc");
    assert_eq!(INDEXES_DIR_NAME, "indexes");
    assert_eq!(DKG_DB_FILENAME, "dkg.db");
    assert_eq!(VECTORS_FILE_NAME, "vector_index.json");
    assert_eq!(SESSIONS_DIR_NAME, "sessions");
    assert_eq!(METRICS_DIR_NAME, "metrics");
    assert_eq!(MODELS_DIR_NAME, "models");
    assert_eq!(RULES_DB_FILENAME, "rules.db");
}

// ---------------------------------------------------------------------------
// dkg_db_path
// ---------------------------------------------------------------------------

#[test]
fn dkg_db_path_layout_native_separator() {
    let root = join_root(&["proj"]);
    let p = dkg_db_path(&root);
    let parts = normal_components(&p);
    assert_eq!(
        parts,
        vec![
            "proj".to_string(),
            ELAI_DATA_DIR_NAME.to_string(),
            INDEXES_DIR_NAME.to_string(),
            DKG_DB_FILENAME.to_string(),
        ],
    );
    assert!(p.file_name().and_then(|s| s.to_str()) == Some(DKG_DB_FILENAME));
}

#[test]
fn dkg_db_path_idempotent() {
    let root = PathBuf::from("proj");
    assert_eq!(dkg_db_path(&root), dkg_db_path(&root));
}

#[test]
fn dkg_db_path_with_unix_style_root() {
    // Even on Windows, "/proj" is a valid Path; helpers must not panic.
    let root = PathBuf::from("/proj");
    let p = dkg_db_path(&root);
    assert!(p.ends_with(format!(
        "{}{MAIN_SEPARATOR}{}{MAIN_SEPARATOR}{}",
        ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME, DKG_DB_FILENAME
    )));
}

#[test]
fn dkg_db_path_with_windows_style_root() {
    // Even on Unix, the helper must not crash — Path is OS-string opaque.
    let root = PathBuf::from(r"C:\proj");
    let p = dkg_db_path(&root);
    let leaf = p.file_name().and_then(|s| s.to_str());
    assert_eq!(leaf, Some(DKG_DB_FILENAME));
}

// ---------------------------------------------------------------------------
// bm25_dir_path
// ---------------------------------------------------------------------------

#[test]
fn bm25_dir_path_resolves_to_indexes_dir() {
    let root = PathBuf::from("proj");
    let p = bm25_dir_path(&root);
    assert_eq!(
        normal_components(&p),
        vec![
            "proj".to_string(),
            ELAI_DATA_DIR_NAME.to_string(),
            INDEXES_DIR_NAME.to_string(),
        ],
    );
}

#[test]
fn bm25_dir_path_shares_parent_with_dkg_db() {
    let root = PathBuf::from("proj");
    assert_eq!(dkg_db_path(&root).parent(), Some(bm25_dir_path(&root).as_path()));
}

// ---------------------------------------------------------------------------
// vectors_db_path
// ---------------------------------------------------------------------------

#[test]
fn vectors_db_path_layout() {
    let root = PathBuf::from("proj");
    let p = vectors_db_path(&root);
    assert_eq!(
        normal_components(&p),
        vec![
            "proj".to_string(),
            ELAI_DATA_DIR_NAME.to_string(),
            INDEXES_DIR_NAME.to_string(),
            VECTORS_FILE_NAME.to_string(),
        ],
    );
}

#[test]
fn vectors_db_path_lives_in_indexes_dir() {
    let root = PathBuf::from("proj");
    assert_eq!(
        vectors_db_path(&root).parent(),
        Some(bm25_dir_path(&root).as_path())
    );
}

// ---------------------------------------------------------------------------
// sessions_dir_path
// ---------------------------------------------------------------------------

#[test]
fn sessions_dir_path_layout() {
    let root = PathBuf::from("proj");
    let p = sessions_dir_path(&root);
    assert_eq!(
        normal_components(&p),
        vec![
            "proj".to_string(),
            ELAI_DATA_DIR_NAME.to_string(),
            SESSIONS_DIR_NAME.to_string(),
        ],
    );
}

// ---------------------------------------------------------------------------
// metrics_dir_path
// ---------------------------------------------------------------------------

#[test]
fn metrics_dir_path_layout() {
    let root = PathBuf::from("proj");
    let p = metrics_dir_path(&root);
    assert_eq!(
        normal_components(&p),
        vec![
            "proj".to_string(),
            ELAI_DATA_DIR_NAME.to_string(),
            METRICS_DIR_NAME.to_string(),
        ],
    );
}

// ---------------------------------------------------------------------------
// models_dir_path
// ---------------------------------------------------------------------------

#[test]
fn models_dir_path_layout_does_not_bake_onnx_subdir() {
    // Per B-1-decisions.md: callers append `onnx/` themselves.
    let root = PathBuf::from("proj");
    let p = models_dir_path(&root);
    assert_eq!(
        normal_components(&p),
        vec![
            "proj".to_string(),
            ELAI_DATA_DIR_NAME.to_string(),
            MODELS_DIR_NAME.to_string(),
        ],
    );
    assert!(
        !p.ends_with("onnx"),
        "models_dir_path must NOT bake /onnx/ — caller responsibility",
    );
}

#[test]
fn models_dir_path_caller_can_compose_onnx_subdir() {
    // Guarantees B-1 callers (cross-encoder, SPLADE, GraphCodeBERT,
    // Granite) can each layer their own subdir without the helper.
    let root = PathBuf::from("proj");
    let onnx = models_dir_path(&root).join("onnx");
    let parts = normal_components(&onnx);
    assert_eq!(parts.last().map(String::as_str), Some("onnx"));
}

// ---------------------------------------------------------------------------
// rules_db_path
// ---------------------------------------------------------------------------

#[test]
fn rules_db_path_layout() {
    let root = PathBuf::from("proj");
    let p = rules_db_path(&root);
    assert_eq!(
        normal_components(&p),
        vec![
            "proj".to_string(),
            ELAI_DATA_DIR_NAME.to_string(),
            RULES_DB_FILENAME.to_string(),
        ],
    );
}

// ---------------------------------------------------------------------------
// Purity / determinism — no IO, repeated calls equal.
// ---------------------------------------------------------------------------

#[test]
fn helpers_are_pure_repeated_calls_equal() {
    let root = PathBuf::from("proj");
    assert_eq!(dkg_db_path(&root), dkg_db_path(&root));
    assert_eq!(bm25_dir_path(&root), bm25_dir_path(&root));
    assert_eq!(vectors_db_path(&root), vectors_db_path(&root));
    assert_eq!(sessions_dir_path(&root), sessions_dir_path(&root));
    assert_eq!(metrics_dir_path(&root), metrics_dir_path(&root));
    assert_eq!(models_dir_path(&root), models_dir_path(&root));
    assert_eq!(rules_db_path(&root), rules_db_path(&root));
}

#[test]
fn helpers_do_not_create_filesystem_state() {
    // Pick a guaranteed-nonexistent root; helpers must NOT touch the FS.
    let root = PathBuf::from("__b1_nonexistent_root__");
    let _ = dkg_db_path(&root);
    let _ = bm25_dir_path(&root);
    let _ = vectors_db_path(&root);
    let _ = sessions_dir_path(&root);
    let _ = metrics_dir_path(&root);
    let _ = models_dir_path(&root);
    let _ = rules_db_path(&root);
    assert!(!root.exists(), "paths helpers must be IO-free");
}
