//! B-1 regression: scan non-test, non-example Rust source under
//! `.elai_cc/crates/` for the literal `".elai/"` (legacy data dir) and
//! fail if any in-scope hit appears.
//!
//! In-scope = production source in the listed crates. Out-of-scope =
//! anything in `tests/`, `examples/`, `benches/`, or sites the B-1
//! context package explicitly enumerated as deferred:
//!
//!   * cli/src/init.rs — gitignore template (legacy `.elai/` entries);
//!   * cli/src/commands.rs — help text;
//!   * cli/src/main.rs:3640-3641 — legacy schema upgrade reader;
//!   * cli/src/main.rs:3892 — steering cache subdir;
//!   * cli/src/main.rs:5365 — sessions listing (uses `.elai/sessions`);
//!   * orchestrator/src/prompt.rs — incidental hits;
//!   * tools/src/lib.rs — Linux home-dir defaults.
//!
//! ssb.rs and session_control.rs removed from whitelist after GAP 1+2 fixed (B-1 delta).
//! The whitelist below is treated as ground truth; if a NEW hit appears
//! outside the whitelist, the test fails.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Files allowed to retain a `".elai/"` literal (out-of-scope per B-1).
/// Paths are repo-root-relative (forward slashes for portability).
const WHITELIST: &[&str] = &[
    "crates/cli/src/init.rs",
    "crates/cli/src/commands.rs",
    "crates/cli/src/main.rs", // legacy upgrade dir + steering + sessions listing
    "crates/orchestrator/src/prompt.rs",
    "crates/tools/src/lib.rs",
    // NOTE(B-1 v2): line-range-level scoping would allow individual function exemptions
    // without file-level whitelisting (WARN-4). Current file-level whitelist is acceptable
    // for B-1 POLISH phase; v2 enhancement would require test infrastructure changes.
    // dashboard-v2/state.rs: legacy fallback at array[1] is intentional per B-1 L94-98
    "crates/dashboard-v2/src-tauri/src/state.rs",
];

/// Crates scanned for the regression. Confined to .elai_cc workspace.
const SCAN_CRATES: &[&str] = &[
    "crates/core",
    "crates/orchestrator",
    "crates/cli",
    "crates/dashboard",
    "crates/dashboard-v2",
    "crates/knowledge",
    "crates/rules",
    "crates/providers",
    "crates/tools",
    "crates/plugins",
    "crates/reasoning",
    "crates/agent",
    "crates/web_search",
    "crates/web_validator",
    "crates/scaffold",
];

/// Locate the workspace root (`.elai_cc/`) starting from the test's
/// `CARGO_MANIFEST_DIR` (which is `.elai_cc/crates/contracts/`).
fn workspace_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent() // crates/
        .and_then(Path::parent) // .elai_cc/
        .map(Path::to_path_buf)
        .expect("CARGO_MANIFEST_DIR has at least 2 ancestors")
}

/// Walk `dir` collecting `.rs` files outside `tests/`, `examples/`,
/// `benches/`. Returns paths relative to `workspace_root`.
fn collect_src_files(workspace_root: &Path, sub: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(sub) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if path.is_dir() {
            // Skip test/example/bench subtrees: production source only.
            if matches!(name_s.as_ref(), "tests" | "examples" | "benches" | "target") {
                continue;
            }
            collect_src_files(workspace_root, &path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(rel) = path.strip_prefix(workspace_root) {
                // Skip test scaffolding files even in src/ directories
                let file_name = name_s.to_string();
                if file_name.ends_with("_test.rs") || file_name.ends_with("_contract.rs") {
                    continue;
                }
                out.push(rel.to_path_buf());
            }
        }
    }
}

/// Convert a relative path to a forward-slash string for whitelist
/// comparison (works on Windows + Unix uniformly).
fn rel_to_forward(path: &Path) -> String {
    path.components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[test]
fn no_dot_elai_literal_in_in_scope_source() {
    let workspace_root = workspace_root();
    let whitelist: BTreeSet<&str> = WHITELIST.iter().copied().collect();

    let mut files = Vec::new();
    for crate_sub in SCAN_CRATES {
        let dir = workspace_root.join(crate_sub).join("src");
        collect_src_files(&workspace_root, &dir, &mut files);
    }

    // Paranoia: the scanner found at least one production .rs file. If
    // this assertion ever trips, the directory layout drifted and the
    // regression check would silently pass on zero files.
    assert!(
        !files.is_empty(),
        "B-1 regression scanner found no .rs files under {SCAN_CRATES:?} — layout changed?",
    );

    let mut offenders: Vec<(String, usize, String)> = Vec::new();
    for rel in &files {
        let rel_s = rel_to_forward(rel);
        if whitelist.contains(rel_s.as_str()) {
            continue;
        }
        let abs = workspace_root.join(rel);
        let body = match fs::read_to_string(&abs) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for (i, line) in body.lines().enumerate() {
            // Skip line comments (`//`, `//!`, `///`) — narrative
            // mentions of `".elai/"` in doc-blocks are not regressions.
            // Block comments (`/* … */`) are not stripped; deliberately
            // forbid that style for path literals to keep the scan
            // simple and predictable.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            // Match the literal `".elai/"` (with the leading double quote
            // so we only catch string literals, not `.elai_cc/` prefixes).
            if line.contains("\".elai/") {
                offenders.push((rel_s.clone(), i + 1, line.trim().to_string()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "B-1 regression: literal `\".elai/\"` reintroduced in {} in-scope file(s):\n{}",
        offenders.len(),
        offenders
            .iter()
            .map(|(f, l, s)| format!("  {f}:{l}  {s}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

#[test]
fn ssb_sessions_subdir_is_canonical() {
    // B-1 GAP 1 deterministic check: SSB_SESSIONS_SUBDIR must point to .elai_cc/sessions.
    assert!(
        elai_orchestrator::SSB_SESSIONS_SUBDIR.contains(".elai_cc"),
        "SSB_SESSIONS_SUBDIR must reference .elai_cc (B-1 GAP 1 unresolved); got: {}",
        elai_orchestrator::SSB_SESSIONS_SUBDIR,
    );
}

#[test]
fn core_paths_helper_is_referenced_by_consumers() {
    // The B-1 deterministic check requires at least 4 references to
    // `core::paths::dkg_db_path` (or its imported short form) across
    // .elai_cc/crates non-test source. After refactor the expected
    // call sites are: cli/main.rs (REPL pipeline), cli/main.rs
    // (telescope), cli/main.rs (rule engine), dashboard/lib.rs,
    // orchestrator/indexing_service.rs (re-export consumer or direct).
    let workspace_root = workspace_root();

    let mut hits: usize = 0;
    let mut files = Vec::new();
    for crate_sub in SCAN_CRATES {
        let dir = workspace_root.join(crate_sub).join("src");
        collect_src_files(&workspace_root, &dir, &mut files);
    }
    for rel in &files {
        let abs = workspace_root.join(rel);
        let body = match fs::read_to_string(&abs) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for line in body.lines() {
            // Accept either the fully-qualified path or the imported
            // short form `dkg_db_path(` (after a `use` statement).
            if line.contains("paths::dkg_db_path") || line.contains("dkg_db_path(") {
                hits += 1;
            }
        }
    }

    assert!(
        hits >= 4,
        "B-1 deterministic check: expected >=4 references to dkg_db_path across in-scope source, found {hits}",
    );
}
