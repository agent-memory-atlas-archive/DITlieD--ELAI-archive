//! Shared workspace walker for negative-space regression contracts.
//!
//! Established by E-12.A after the 4-way cross-skip list (e11 + e25 + e38 +
//! e48) hit the threshold where adding a new sibling cost N edits. The
//! workspace_rs_files() helper auto-skips ANY file matching the
//! `e<N>_*_contracts.rs` pattern, so adding a new negative-space contract
//! requires zero edits to the existing siblings.
//!
//! Standard Rust test discovery convention: tests/common/mod.rs is loaded
//! via `mod common;` at the top of each test file. Files under tests/common/
//! are NOT discovered as integration tests (Rust treats only top-level
//! tests/*.rs as tests, not subdirectories), so this helper module ships
//! safely alongside the contracts without bloating the test-runner output.

use std::fs;
use std::path::{Path, PathBuf};

/// Workspace root resolved from CARGO_MANIFEST_DIR. The contracts crate sits
/// at `.elai_cc/crates/contracts/`; the workspace root is two parents up.
pub fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf())
}

/// True when the file name matches the negative-space contract pattern
/// `e<NUMBER>_*_contracts.rs`. Used to auto-skip sibling contracts during
/// the workspace walk so each contract's banned-string literals do not trip
/// the others.
fn is_sibling_negative_space_contract(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('e') else {
        return false;
    };
    // After "e" expect at least one digit, then "_", then arbitrary tail,
    // then "_contracts.rs".
    let mut chars = rest.chars();
    // At least one digit.
    let first = chars.next();
    if !first.map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return false;
    }
    // Consume remaining digits.
    let after_digits: String = chars
        .by_ref()
        .skip_while(|c| c.is_ascii_digit())
        .collect();
    // Expect the rest to start with "_" and end with "_contracts.rs".
    after_digits.starts_with('_') && after_digits.ends_with("_contracts.rs")
}

/// Recursively walk the workspace root, returning all `.rs` files except
/// build artifacts (target/, .git/, .elai/, indexes/, scip_indices/, models/,
/// node_modules/) AND the negative-space contract siblings.
pub fn workspace_rs_files() -> Vec<PathBuf> {
    let root = workspace_root();
    let mut out: Vec<PathBuf> = Vec::new();
    walk_dir(&root, &mut out);
    out
}

fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if matches!(
                name,
                "target"
                    | ".git"
                    | "node_modules"
                    | ".elai"
                    | "indexes"
                    | "scip_indices"
                    | "models"
            ) {
                continue;
            }
            walk_dir(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            // Auto-skip ALL sibling negative-space contracts. Adding a new
            // contract requires no edit here.
            if is_sibling_negative_space_contract(name) {
                continue;
            }
            out.push(path);
        }
    }
}

/// Find first occurrence of any needle in the file. Returns the needle
/// string when matched, None when no match.
pub fn file_contains_any(path: &Path, needles: &[&str]) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    for needle in needles {
        if contents.contains(needle) {
            return Some((*needle).to_string());
        }
    }
    None
}

/// Collect (path, matched_needle) pairs across the workspace. Wraps
/// workspace_rs_files() + file_contains_any() for the canonical
/// "ban these strings everywhere" assertion shape.
pub fn collect_hits(needles: &[&str]) -> Vec<(PathBuf, String)> {
    let mut hits: Vec<(PathBuf, String)> = Vec::new();
    for file in workspace_rs_files() {
        if let Some(needle) = file_contains_any(&file, needles) {
            hits.push((file, needle));
        }
    }
    hits
}

/// Format a hit list for the failure message of an assert. Each entry as
/// "path :: needle" on its own line, joined with newline+two-space indent.
pub fn format_hits(hits: &[(PathBuf, String)]) -> String {
    hits.iter()
        .map(|(p, n)| format!("{} :: {}", p.display(), n))
        .collect::<Vec<_>>()
        .join("\n  ")
}
