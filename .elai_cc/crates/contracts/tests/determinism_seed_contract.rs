//! Phase A6 — determinism plumbing: seed wire-format contract.
//!
//! Pins the cross-language wire shape:
//!   1. `MessageRequest::seed = Some(N)` serializes to JSON containing
//!      `"seed": N` (key present, integer value).
//!   2. `MessageRequest::seed = None` ELIDES the `"seed"` key entirely
//!      (`#[serde(skip_serializing_if = "Option::is_none")]`). Pre-A6 traffic
//!      must remain byte-identical when no seed is supplied.
//!   3. The matching JSON shape round-trips back into a `MessageRequest`
//!      whose `seed` field is preserved (forward compat on the inbound side).
//!
//! Phase A6 brief:
//!   workflow/handoff/phase-A6.md  (determinism plumbing — Rust types.rs +
//!                                  Python server.py engine.generate seed kwarg)
//!
//! Initial expected state: RED (TESTING_FIRST). Pre-A6 `MessageRequest`
//! (providers/src/types.rs:5-18) has no `seed` field; the test will not even
//! compile until step 5 of phase-A6 adds `pub seed: Option<u64>`.
//!
//! Recommended invocation:
//!   cargo test -p elai_cc_contracts --test determinism_seed_contract -- --nocapture

use elai_providers::{InputMessage, MessageRequest};

/// Build a minimal MessageRequest fixture. All non-seed Option fields stay
/// None so the wire payload is the smallest possible diff against
/// pre-A6 to make missing-key assertions unambiguous.
fn fixture(seed: Option<u64>) -> MessageRequest {
    MessageRequest {
        model: "qwen".to_string(),
        max_tokens: 8,
        messages: vec![InputMessage::user_text("hello")],
        system: None,
        tools: None,
        tool_choice: None,
        stream: false,
        seed,
        effort: None,
        chat_template_kwargs: None,
    }
}

#[test]
fn seed_some_appears_in_wire_payload() {
    let req = fixture(Some(42));
    let wire = serde_json::to_value(&req)
        .expect("MessageRequest must serialize to JSON");
    let obj = wire
        .as_object()
        .expect("MessageRequest must serialise as JSON object");
    let seed_val = obj.get("seed").unwrap_or_else(|| {
        panic!(
            "A6 GATE1: serialized MessageRequest missing `seed` key when \
             seed=Some(42). Full payload: {wire}"
        )
    });
    let n = seed_val
        .as_u64()
        .unwrap_or_else(|| panic!(
            "A6 GATE1: `seed` must be a u64 integer, got {seed_val:?}"
        ));
    assert_eq!(
        n, 42,
        "A6 GATE1: `seed` round-trip mismatch — sent Some(42), wire emitted {n}"
    );
}

#[test]
fn seed_none_omits_key_from_wire_payload() {
    let req = fixture(None);
    let wire = serde_json::to_value(&req)
        .expect("MessageRequest must serialize to JSON");
    let obj = wire
        .as_object()
        .expect("MessageRequest must serialise as JSON object");
    assert!(
        !obj.contains_key("seed"),
        "A6 GATE2: `seed` key MUST be elided when None \
         (#[serde(skip_serializing_if = \"Option::is_none\")] required to \
         preserve byte-identical pre-A6 wire format). Got keys: {:?}",
        obj.keys().collect::<Vec<_>>()
    );
}

#[test]
fn seed_roundtrips_through_json() {
    let req = fixture(Some(7));
    let wire = serde_json::to_string(&req)
        .expect("MessageRequest must serialize to JSON");
    let parsed: MessageRequest = serde_json::from_str(&wire)
        .expect("MessageRequest must deserialize from its own JSON");
    assert_eq!(
        parsed.seed,
        Some(7),
        "A6 GATE3: seed field MUST survive JSON round-trip; got {:?}",
        parsed.seed
    );
}

#[test]
fn seed_absent_in_json_deserializes_as_none() {
    // Pre-A6 traffic shape — no `seed` key at all. Must parse without error
    // and yield seed=None (serde default for Option<T>).
    let pre_a6_wire = serde_json::json!({
        "model": "qwen",
        "max_tokens": 8,
        "messages": [
            { "role": "user", "content": [{ "type": "text", "text": "hello" }] }
        ],
        "stream": false
    });
    let parsed: MessageRequest = serde_json::from_value(pre_a6_wire)
        .expect("pre-A6 wire (no `seed` key) MUST still deserialize after A6");
    assert!(
        parsed.seed.is_none(),
        "A6 GATE4: absent `seed` key MUST deserialize to None for backward \
         compat; got {:?}",
        parsed.seed
    );
}

/// Negative assertion against accidental serialization of `seed=0`. Some
/// frameworks special-case 0 as "no value"; serde must NOT — explicit 0 is
/// a valid seed and must reach the sidecar.
#[test]
fn seed_zero_is_preserved_distinct_from_none() {
    let with_zero = serde_json::to_value(fixture(Some(0)))
        .expect("serialize Some(0)");
    let with_none = serde_json::to_value(fixture(None))
        .expect("serialize None");
    assert_eq!(
        with_zero.get("seed").and_then(|v| v.as_u64()),
        Some(0),
        "A6 GATE5: seed=Some(0) MUST serialize as `\"seed\": 0`, not be elided"
    );
    assert!(
        with_none.get("seed").is_none(),
        "A6 GATE5 (sanity): seed=None still elides while seed=Some(0) emits"
    );
}
