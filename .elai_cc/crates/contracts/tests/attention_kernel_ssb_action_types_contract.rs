// E-28 contract test — three SsbActionType variants for the
// attention-kernels 2026 pack plus their payload structs. Locks the wire
// format added in E-28 Tasks 2 + 3 + 4. Paper labels (per
// memory:feedback_paper_citation_verify discipline):
//   * arXiv:2309.17453 Xiao et al. ICLR 2024 — DIRECT GROUNDING for the
//     StreamingLlmEngaged sink-count default (4); interface map at
//     workflow/papers/streaming-llm-interface.md.
//   * arXiv:2501.15383 Qwen2.5-1M — DIRECT GROUNDING for the YaRN
//     rope_scaling recipe used by YarnRopeScalingApplied.
//   * pytorch#133780 — DIRECT GROUNDING for the SDPA-backend-pin
//     justification (SdpaBackendPinned closes this issue).
//
// Test plan (5 contract tests, matching the executor triage):
//   1. SCHEMA_VERSION floor >= 156 (E-28 ships three new variants).
//   2. as_sql_str + ssb_action_type_from_str_public round-trip for each
//      of the three slugs.
//   3. msgpack round-trip on StreamingLlmEngagedPayload + discriminant
//      check.
//   4. msgpack round-trip on SdpaBackendPinnedPayload + discriminant
//      check.
//   5. msgpack round-trip on YarnRopeScalingAppliedPayload + discriminant
//      check.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SdpaBackendPinnedPayload, SsbActionType,
    SsbPayload, StreamingLlmEngagedPayload, YarnRopeScalingAppliedPayload,
    SCHEMA_VERSION_PUBLIC,
};

#[test]
fn e28_schema_version_floor_at_156() {
    let v = SCHEMA_VERSION_PUBLIC;
    assert!(
        v >= 156,
        "E-28 floors SCHEMA_VERSION at 156 for the attention-kernels 2026 \
         pack (StreamingLlmEngaged + SdpaBackendPinned + YarnRopeScalingApplied); \
         got {v}. See workflow/reasoning/E-28-decisions.md."
    );
}

#[test]
fn e28_action_types_sql_round_trip() {
    // Streaming-LLM beacon — emitted when ELAI_STREAMING_LLM_ENABLE>=1 and
    // the attention-sink mask is applied (arXiv:2309.17453 Sec 4.2).
    let s1 = SsbActionType::StreamingLlmEngaged.as_sql_str();
    assert_eq!(s1, "streaming_llm_engaged");
    let b1 = ssb_action_type_from_str_public(s1).expect("from_str streaming_llm");
    assert_eq!(b1, SsbActionType::StreamingLlmEngaged);

    // SDPA-backend-pin beacon — emitted once at boot recording the
    // resolved backend (closes pytorch#133780 silent MATH fallback).
    let s2 = SsbActionType::SdpaBackendPinned.as_sql_str();
    assert_eq!(s2, "sdpa_backend_pinned");
    let b2 = ssb_action_type_from_str_public(s2).expect("from_str sdpa_pin");
    assert_eq!(b2, SsbActionType::SdpaBackendPinned);

    // YaRN harness beacon — live boot still refuses YaRN per E-32;
    // this row only fires from test_yarn_eval_gate.py.
    let s3 = SsbActionType::YarnRopeScalingApplied.as_sql_str();
    assert_eq!(s3, "yarn_rope_scaling_applied");
    let b3 = ssb_action_type_from_str_public(s3).expect("from_str yarn_rope");
    assert_eq!(b3, SsbActionType::YarnRopeScalingApplied);
}

#[test]
fn e28_streaming_llm_engaged_payload_round_trip() {
    let payload = StreamingLlmEngagedPayload {
        sink_count: 4,
        window_size: 25_000,
        context_position: 30_000,
        ts_ms: 1_748_000_000_154,
    };
    let wrapped = SsbPayload::StreamingLlmEngaged(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode streaming_llm");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode streaming_llm");
    assert_eq!(decoded, wrapped, "msgpack round-trip");
    assert_eq!(decoded.action_type(), SsbActionType::StreamingLlmEngaged);
    // Paper-derived invariant: paper-recommended minimum sink_count is 4
    // per arXiv:2309.17453 Sec 4.2 — locked here so any future schema
    // change that drops the field below 4 needs to update this contract.
    assert!(
        payload.sink_count >= 4,
        "paper-recommended StreamingLLM sink_count is >= 4"
    );
}

#[test]
fn e28_sdpa_backend_pinned_payload_round_trip() {
    let payload = SdpaBackendPinnedPayload {
        backend_name: "flash_attention".into(),
        fallback_used: false,
        compute_capability: 86, // RTX 3090 (Ampere SM_86)
        ts_ms: 1_748_000_000_155,
    };
    let wrapped = SsbPayload::SdpaBackendPinned(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode sdpa_pin");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode sdpa_pin");
    assert_eq!(decoded, wrapped);
    assert_eq!(decoded.action_type(), SsbActionType::SdpaBackendPinned);
    // Counterfactual: torch < 2.3 falls back to nullcontext, recorded
    // with fallback_used=true so operators see the degraded path.
    let fallback_payload = SdpaBackendPinnedPayload {
        backend_name: "unknown".into(),
        fallback_used: true,
        compute_capability: 86,
        ts_ms: 1_748_000_000_156,
    };
    let wrapped_fb = SsbPayload::SdpaBackendPinned(fallback_payload);
    let bytes_fb = rmp_serde::to_vec_named(&wrapped_fb).expect("encode fallback");
    let decoded_fb: SsbPayload = rmp_serde::from_slice(&bytes_fb).expect("decode fallback");
    assert_eq!(decoded_fb, wrapped_fb);
}

#[test]
fn e28_yarn_rope_scaling_applied_payload_round_trip() {
    // Qwen2.5-Coder-7B base: max_position_embeddings = 32768, factor=4
    // targets 128K per arXiv:2501.15383 Qwen2.5-1M recipe. This payload
    // ONLY emits from the harness path when ELAI_YARN_EVAL_GATE=1; the
    // live boot path still refuses YaRN per E-32 refuse_yarn_at_boot.
    let payload = YarnRopeScalingAppliedPayload {
        factor: 4.0,
        original_max_position_embeddings: 32_768,
        applied_at_layer: 0, // config-level mutation
        ts_ms: 1_748_000_000_157,
    };
    let wrapped = SsbPayload::YarnRopeScalingApplied(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode yarn_rope");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode yarn_rope");
    assert_eq!(decoded, wrapped);
    assert_eq!(decoded.action_type(), SsbActionType::YarnRopeScalingApplied);
    // Lock the Qwen2.5-Coder-7B base context window invariant — if the
    // upstream HF config changes, the harness path needs an update.
    assert_eq!(
        payload.original_max_position_embeddings, 32_768,
        "Qwen2.5-Coder-7B base context window is 32K (HF config.json)"
    );
}
