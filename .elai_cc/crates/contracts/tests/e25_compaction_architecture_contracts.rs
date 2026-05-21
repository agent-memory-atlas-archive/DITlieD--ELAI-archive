//! E-25 Tier-A — context compaction architecture regression contracts.
//!
//! Four negative-space guards + walker sanity, using the shared
//! `tests/common/mod.rs` workspace walker (factored in E-12.A). The walker
//! auto-skips any sibling negative-space contract (file name matches
//! `e<N>_*_contracts.rs`).

mod common;

use common::{collect_hits, format_hits, workspace_rs_files};

#[test]
fn no_parallel_telescope_collapse_payload() {
    // TelescopeCollapsePayload is the canonical payload for the per-layer
    // telescope collapse mechanism (ssb.rs). A parallel payload fragments
    // the dashboard's compaction-event surface.
    let banned = [
        "struct TelescopeCollapseV2Payload",
        "struct LayerCollapsePayload",
        "struct CompactionCollapsePayload",
        "enum TelescopeCollapseKind",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel telescope-collapse payloads are banned — extend TelescopeCollapsePayload.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_parallel_session_fork_handler() {
    // Canonical fork: cli/handoff.rs + fork_session_branch Tauri command +
    // SessionsDrawer.svelte. Parallel ForkSessionV2 / SessionForkPlus would
    // double the UX surface.
    let banned = [
        "fn fork_session_branch_v2",
        "fn create_session_fork_v2",
        "struct SessionForkPlus",
        "trait ForkSessionV2",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel session-fork handlers are banned — extend handoff.rs / fork_session_branch.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_bespoke_message_summarizer_parallel_to_compact() {
    // Tier-1 message summarization runs through compact::compact_session
    // (compact.rs:96) called from ConversationRuntime::maybe_auto_compact.
    // Bespoke summarizers bypass compact.rs's cache-key + token-budget
    // guarantees.
    let banned = [
        "fn compact_session_v2",
        "fn summarize_messages_v2",
        "trait BespokeSummarizer",
        "struct CustomMessageSummarizer",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Bespoke message summarizers parallel to compact::compact_session banned.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_silent_context_truncation_helpers() {
    // Every reduction must emit ContextReductionPayload so audit trail
    // captures it. Silent drop helpers break replay + audit.
    let banned = [
        "fn silent_drop_oldest",
        "fn truncate_silently",
        "fn silent_shorten",
        "fn drop_oldest_no_emit",
        "fn quiet_compact",
        "fn stealth_summarize",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Silent context drop helpers banned — use ContextReductionPayload-emitting helpers.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let files = workspace_rs_files();
    assert!(
        !files.is_empty(),
        "e25 contract walker found zero .rs files — the four anti-pattern asserts cannot be trusted"
    );
}
