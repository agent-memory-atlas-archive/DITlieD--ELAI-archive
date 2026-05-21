//! E-38 Tier-A — anti-pattern regression guards.
//!
//! Four grep-style contract tests + walker sanity, using the shared
//! `tests/common/mod.rs` workspace walker (factored in E-12.A).

mod common;

use common::{collect_hits, format_hits, workspace_rs_files};

#[test]
fn runtime_adapter_trait_stays_arch_agnostic() {
    let banned = [
        "fn requires_dense",
        "fn is_moe",
        "fn requires_moe",
        "fn dense_only",
        "fn moe_only",
        "DenseRuntime",
        "MoeRuntime",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "RuntimeAdapter must stay arch-agnostic; found banned identifiers:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_hardcoded_flash_attention_2_assumption() {
    let banned = [
        "require_flash_attention_2",
        "assert_flash_attention_2",
        "must_have_flash_attn_2",
        "REQUIRES_FA2",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "No hard-coded FA2 requirement allowed; found:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_bespoke_agent_protocol_parallel_to_mcp() {
    let banned = [
        "trait ElaiAgentProtocol",
        "struct ElaiAgentProtocol",
        "enum ElaiAgentProtocol",
        "trait BespokeAgentProtocol",
        "struct BespokeAgentProtocol",
        "trait CustomAgentProto",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "No bespoke agent protocol allowed (use MCP); found:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_operator_screenshot_polling_loop() {
    let banned = [
        "screenshot_loop",
        "browser_screenshot_poll",
        "operator_capture_loop",
        "screenshot_every_ms",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "No operator-style screenshot loop allowed; found:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let files = workspace_rs_files();
    assert!(
        !files.is_empty(),
        "workspace walker found zero .rs files — contract walker is broken, the four anti-pattern asserts cannot be trusted",
    );
}
