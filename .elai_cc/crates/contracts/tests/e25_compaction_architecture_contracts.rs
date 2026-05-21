//! E-25 Tier-A — context compaction architecture regression contracts.
//!
//! Four negative-space guards (continuing the E-38.A + E-48.A pattern) +
//! one walker sanity test. Locks invariants the 6-tier compaction work
//! depends on so Tier-B builds on the existing primitives rather than
//! forking them.
//!
//! The integration-explorer-elai run for s-orch-e25 verified that ELAI
//! already has:
//!   - 3-level adaptive_reduce ladder (AdaptiveLevel) in
//!     orchestrator/src/context_budget.rs:1480 emitting ContextReductionPayload.
//!   - Tier-1 message summarisation via compact::compact_session (compact.rs:96)
//!     + ConversationRuntime::maybe_auto_compact (conversation.rs:613).
//!   - Per-layer TelescopeCollapse mechanism at telescope_caller.rs:189
//!     emitting TelescopeCollapsePayload (ssb.rs:5102).
//!   - HookPoint::PreCompact dispatch at hook_dispatcher.rs:915.
//!   - Session-fork at cli/handoff.rs:489-560 + fork_session_branch Tauri
//!     command + SessionsDrawer.svelte:44.
//!
//! Tier-B for E-25 will land: crates/orchestrator/src/compaction_ladder.rs
//! orchestrating the above primitives + a CompactionLadderPanel.svelte +
//! net-new tiers (T2 per-tool-result snip, T4 >5-turn bucket collapse, T5/T6
//! sidecar re-narration via PerRoleRuntime). See workflow/reasoning/E-25-decisions.md.

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf())
}

fn collect_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    walk_dir(root, &mut out);
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
            // Self-skip + cross-skip the sibling negative-space contracts
            // so each contract's banned-string literals do not trip the
            // others.
            if matches!(
                name,
                "e25_compaction_architecture_contracts.rs"
                    | "e38_anti_pattern_contracts.rs"
                    | "e48_research_contracts.rs"
                    | "e11_idea_to_saas_contracts.rs"
            ) {
                continue;
            }
            out.push(path);
        }
    }
}

fn file_contains_any(path: &Path, needles: &[&str]) -> Option<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return None;
    };
    for needle in needles {
        if contents.contains(needle) {
            return Some((*needle).to_string());
        }
    }
    None
}

fn collect_hits(root: &Path, needles: &[&str]) -> Vec<(PathBuf, String)> {
    let mut hits: Vec<(PathBuf, String)> = Vec::new();
    for file in collect_rust_sources(root) {
        if let Some(needle) = file_contains_any(&file, needles) {
            hits.push((file, needle));
        }
    }
    hits
}

fn format_hits(hits: &[(PathBuf, String)]) -> String {
    hits.iter()
        .map(|(p, n)| format!("{} :: {}", p.display(), n))
        .collect::<Vec<_>>()
        .join("\n  ")
}

#[test]
fn no_parallel_telescope_collapse_payload() {
    // TelescopeCollapsePayload is the canonical payload for the per-layer
    // telescope collapse mechanism (ssb.rs:5102). A parallel payload
    // fragments the dashboard's compaction-event surface.
    let root = workspace_root();
    let banned = [
        "struct TelescopeCollapseV2Payload",
        "struct LayerCollapsePayload",
        "struct CompactionCollapsePayload",
        "enum TelescopeCollapseKind",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Parallel telescope-collapse payloads are banned — extend TelescopeCollapsePayload.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_parallel_session_fork_handler() {
    // Session fork lives at cli/handoff.rs + fork_session_branch Tauri
    // command + SessionsDrawer. A parallel ForkSessionV2 / SessionForkPlus
    // means the dashboard ends up with two surfaces for the same operation.
    let root = workspace_root();
    let banned = [
        "fn fork_session_branch_v2",
        "fn create_session_fork_v2",
        "struct SessionForkPlus",
        "trait ForkSessionV2",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Parallel session-fork handlers are banned — extend handoff.rs / fork_session_branch.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_bespoke_message_summarizer_parallel_to_compact() {
    // Tier-1 message summarisation runs through compact::compact_session
    // (compact.rs:96) called from ConversationRuntime::maybe_auto_compact.
    // A bespoke summarizer that bypasses compact.rs loses the cache-key /
    // token-budget guarantees the conversation runtime relies on.
    let root = workspace_root();
    let banned = [
        "fn compact_session_v2",
        "fn summarize_messages_v2",
        "trait BespokeSummarizer",
        "struct CustomMessageSummarizer",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Bespoke message summarizers parallel to compact::compact_session banned.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_silent_context_truncation_helpers() {
    // ContextReductionPayload (orchestrator/src/context_budget.rs:1480) is
    // the canonical surface for adaptive reductions. A helper like
    // `silent_drop_oldest()` / `truncate_silently()` would drop content
    // without a SSB row, breaking the audit trail.
    let root = workspace_root();
    let banned = [
        "fn silent_drop_oldest",
        "fn truncate_silently",
        "fn drop_oldest_no_emit",
        "fn quiet_compact",
        "fn stealth_summarize",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Silent context truncation banned — use ContextReductionPayload-emitting helpers.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let root = workspace_root();
    let files = collect_rust_sources(&root);
    assert!(
        !files.is_empty(),
        "e25 contract walker found zero .rs files under {} — the four anti-pattern asserts cannot be trusted",
        root.display()
    );
}
