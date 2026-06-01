//! B-15 Tranche C burndown regression contracts.
//!
//! Negative-space guards covering:
//!   (1) the 14 production files migrated from std::HashMap -> FxHashMap
//!       (mechanical sweep — file-name list is fenced; if any of these
//!       files re-grows a `use std::collections::HashMap;` import the
//!       contract fails),
//!   (2) the single Box<dyn Error> migration site at
//!       eval-corpus/src/bin/migrate_held_out.rs main(),
//!   (3) rustc-hash workspace dep presence,
//!   (4) eval-corpus anyhow.workspace dep presence.
//!
//! Two of the production files flagged in the exploration as candidates
//! (orchestrator/src/dual_ledger.rs + orchestrator/src/reasoning_template.rs)
//! are INTENTIONALLY excluded — both keep their std::HashMap import after
//! per-occurrence verification (dual_ledger.rs:402 IPC-boundary by_task()
//! returns HashMap<String, ...>; reasoning_template.rs:685-687 doc-anchor
//! stub _ensure_hashmap_alias_for_doc_consumers actually uses HashMap).
//! See workflow/reasoning/Tranche-C-B15-decisions.md DECISION-2.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::workspace_root;

/// The 14 production files migrated by the C-1 mechanical sweep. If any
/// file in this list re-acquires `use std::collections::HashMap;` the
/// contract fails — the FxHashMap migration must stay clean.
const MIGRATED_FILES: &[&str] = &[
    "crates/orchestrator/src/cross_domain_canary.rs",
    "crates/orchestrator/src/credit_attribution/types.rs",
    "crates/orchestrator/src/cost/free_tier_guard.rs",
    "crates/orchestrator/src/cost/cost_router.rs",
    "crates/orchestrator/src/cost/cloud_abort.rs",
    "crates/orchestrator/src/template_archive.rs",
    "crates/orchestrator/src/trust_telemetry.rs",
    "crates/dashboard-v2/src-tauri/src/commands_d6.rs",
    "crates/dashboard-v2/src-tauri/src/commands_d7.rs",
    "crates/dashboard-v2/src-tauri/src/commands_d9.rs",
    "crates/dashboard-v2/src-tauri/src/commands_e46.rs",
    "crates/plugins/src/tool_calibration/registry.rs",
    "crates/eval-corpus/src/generator_rotation.rs",
    "crates/eval-corpus/src/task_schema.rs",
];

/// Resolve a workspace-relative path to an absolute filesystem path.
fn resolve(rel: &str) -> PathBuf {
    workspace_root().join(rel)
}

/// True when the file contains the banned-line literal as a free-standing
/// `use` import. Uses a line-anchored substring match so a `// use std::...`
/// commented-out line does not trigger.
fn file_has_import_line(path: &Path, banned_line: &str) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    text.lines().any(|line| line.trim() == banned_line)
}

#[test]
fn test_fxhashmap_imports_replaced_in_target_files() {
    let banned = "use std::collections::HashMap;";
    let mut violators: Vec<String> = Vec::new();
    for rel in MIGRATED_FILES {
        let path = resolve(rel);
        if !path.exists() {
            violators.push(format!("MISSING: {rel}"));
            continue;
        }
        if file_has_import_line(&path, banned) {
            violators.push(format!("REGROWN: {rel} still has `{banned}`"));
        }
    }
    assert!(
        violators.is_empty(),
        "B-15 C-1 FxHashMap migration regressed:\n  {}",
        violators.join("\n  ")
    );
}

#[test]
fn test_eval_corpus_migrate_held_out_no_box_dyn_error() {
    let path = resolve("crates/eval-corpus/src/bin/migrate_held_out.rs");
    assert!(path.exists(), "migrate_held_out.rs missing at {}", path.display());
    let text = fs::read_to_string(&path).expect("read migrate_held_out.rs");
    // The C-2 site: `fn main() -> Result<(), Box<dyn std::error::Error>>`
    // must NOT reappear. anyhow::Result<()> is the canonical replacement.
    assert!(
        !text.contains("Box<dyn std::error::Error>"),
        "B-15 C-2 regressed: migrate_held_out.rs reintroduced Box<dyn std::error::Error>"
    );
    assert!(
        !text.contains("Box<dyn Error>"),
        "B-15 C-2 regressed: migrate_held_out.rs reintroduced Box<dyn Error>"
    );
    assert!(
        text.contains("use anyhow::Result;") && text.contains("fn main() -> Result<()>"),
        "B-15 C-2 invariant lost: migrate_held_out.rs main() must return anyhow::Result<()>"
    );
}

#[test]
fn test_rustc_hash_workspace_dep_present() {
    // Root Cargo.toml must keep rustc-hash at the workspace level so the
    // 14 migrated files inherit via `.workspace = true`.
    let path = workspace_root().join("Cargo.toml");
    let text = fs::read_to_string(&path).expect("read workspace Cargo.toml");
    assert!(
        text.contains("rustc-hash"),
        "workspace Cargo.toml missing rustc-hash dep; B-15 C-1 FxHashMap migration breaks"
    );
}

#[test]
fn test_anyhow_workspace_dep_present_in_eval_corpus() {
    let path = resolve("crates/eval-corpus/Cargo.toml");
    let text = fs::read_to_string(&path).expect("read eval-corpus Cargo.toml");
    assert!(
        text.contains("anyhow.workspace = true"),
        "eval-corpus Cargo.toml missing anyhow.workspace = true; migrate_held_out.rs main() depends on it"
    );
    // C-1 also requires rustc-hash in eval-corpus for the FxHashMap swap
    // across generator_rotation + task_schema.
    assert!(
        text.contains("rustc-hash.workspace = true"),
        "eval-corpus Cargo.toml missing rustc-hash.workspace = true; FxHashMap migration depends on it"
    );
}
