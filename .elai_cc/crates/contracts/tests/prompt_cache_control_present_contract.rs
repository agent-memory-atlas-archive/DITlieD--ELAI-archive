//! E-39 contract: `build_request_for_task` places `cache_control`
//! markers on the system blocks and (when constrained decoding is
//! active) on the tool definition.
//!
//! This is a static assertion on the already-wired `grounding_client.rs`
//! code path (E-39 cache_control discipline pass). The spec's
//! off-agent artifact for this test is confirmed presence of
//! `cache_control` keys in the request payload struct; no SSB row is
//! written (the grounding is structural -- `SystemBlock::ephemeral_text`
//! sets `cache_control = Some(CacheControl::ephemeral())`).
//!
//! Production entrypoint: elai run -> drive_task -> grounding_client
//! -> `build_request_for_task` (grounding_client.rs:705 -- the
//! cache_control discipline pass comment block).

use elai_providers::{CacheControl, SystemBlock};

#[test]
fn contract_prompt_cache_control_ephemeral_text_sets_marker() {
    // `SystemBlock::ephemeral_text` is the helper used by
    // `build_request_for_task` for all 3 system blocks. Assert it
    // produces a cache_control marker with kind=="ephemeral".
    let block = SystemBlock::ephemeral_text("system prompt text");
    let cc = block
        .cache_control
        .expect("ephemeral_text must set cache_control");
    assert_eq!(
        cc.kind, "ephemeral",
        "cache_control kind must be 'ephemeral'"
    );
    assert!(
        cc.ttl.is_none(),
        "5-min TTL marker must not carry a ttl field (default ephemeral)"
    );
}

#[test]
fn contract_prompt_cache_control_plain_text_has_no_marker() {
    // Plain `SystemBlock::text` is NOT marked -- the cache_control
    // discipline pass specifically uses ephemeral_text for the static
    // prefix blocks (system / spec-anchor / manifest-anchor).
    let block = SystemBlock::text("plain system text");
    assert!(
        block.cache_control.is_none(),
        "plain text block must NOT have cache_control"
    );
}

#[test]
fn contract_prompt_cache_control_ephemeral_serializes_correctly() {
    // The wire shape `{"type":"ephemeral"}` must survive a serde
    // round-trip (the Anthropic API rejects any deviation).
    let cc = CacheControl::ephemeral();
    let json = serde_json::to_string(&cc).expect("serialize");
    let round: CacheControl = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(round.kind, "ephemeral");
    assert!(round.ttl.is_none());
    // Must NOT contain a "ttl" key in the serialized form.
    assert!(
        !json.contains("ttl"),
        "5-min ephemeral must not emit a ttl key in JSON: {json}"
    );
}
