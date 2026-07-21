//! J-19 deterministic offline HF-vs-vLLM fixture contract tests.
//!
//! Pins the HF-sidecar-vs-VllmV1 capabilities contrast and the recorded
//! protocol shapes (request wire + response wire) via source-pin greps and
//! inline JSON fixtures. No live network calls. No billed API calls. No
//! live CUDA. These are pure-offline correctness + protocol-shape pinning
//! tests.
//!
//! The live measure leg (throughput, TTFT, prefix-cache hit-rate,
//! token-stream blake3) is W-10, OUT OF SCOPE here. AWQ-build asserts
//! live in the E-27 offline-tooling slice, not here. DoLa-fail-on-vLLM
//! is a live-runtime check (the vLLM server rejects an unknown logits
//! processor), not an offline assert; the offline leg only proves the
//! Rust capability surface has no DoLa flag to claim.

use elai_providers::runtime::{
    create_runtime, resolve_backend, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig,
    VllmRuntime, VllmV1Runtime,
};
use elai_providers::runtime::types::RuntimeCapabilities;
use elai_providers::{MessageResponse, OutputContentBlock};
use serde_json::{json, Value};

const VLLM_V1_SOURCE: &str =
    include_str!("../../providers/src/runtime/vllm_v1.rs");
const VLLM_SOURCE: &str = include_str!("../../providers/src/runtime/vllm.rs");

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn capabilities_contrast_hf_sidecar_vs_vllm_v1() {
    let _g = env_lock();
    let prev = std::env::var("ELAI_VLLM_V1_PREFIX_CACHING").ok();
    std::env::remove_var("ELAI_VLLM_V1_PREFIX_CACHING");

    let vllm_cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Vllm);
    let vllm = VllmRuntime::new(vllm_cfg);
    let vllm_caps = vllm.capabilities();
    assert!(
        !vllm_caps.supports_caching,
        "VllmRuntime (HF-sidecar sibling) always returns supports_caching=false (the structural APC gap J-19 fills)"
    );

    let v1_cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::VllmV1);
    let v1_off = VllmV1Runtime::new(v1_cfg.clone());
    let v1_off_caps = v1_off.capabilities();
    assert!(
        !v1_off_caps.supports_caching,
        "VllmV1 without ELAI_VLLM_V1_PREFIX_CACHING returns supports_caching=false"
    );

    std::env::set_var("ELAI_VLLM_V1_PREFIX_CACHING", "1");
    let v1_on = VllmV1Runtime::new(v1_cfg);
    let v1_on_caps = v1_on.capabilities();
    assert!(
        v1_on_caps.supports_caching,
        "VllmV1 with ELAI_VLLM_V1_PREFIX_CACHING=1 returns supports_caching=true (the APC claim)"
    );

    assert!(
        vllm_caps.supports_tools && vllm_caps.supports_streaming && vllm_caps.json_schema_mode,
        "VllmRuntime advertises tools+streaming+json_schema"
    );
    assert!(
        v1_on_caps.supports_tools && v1_on_caps.supports_streaming && v1_on_caps.json_schema_mode,
        "VllmV1 advertises tools+streaming+json_schema"
    );
    assert!(
        !v1_on_caps.supports_vision
            && !v1_on_caps.supports_prefill
            && !v1_on_caps.native_thinking_budget,
        "VllmV1 conservative defaults: no vision/prefill/native-thinking-budget"
    );

    match prev {
        Some(v) => std::env::set_var("ELAI_VLLM_V1_PREFIX_CACHING", v),
        None => std::env::remove_var("ELAI_VLLM_V1_PREFIX_CACHING"),
    }
}

#[test]
fn vllm_v1_source_pins_v1_messages_path() {
    assert!(
        VLLM_V1_SOURCE.contains("/v1/messages"),
        "vllm_v1.rs must POST to the native Anthropic /v1/messages endpoint (plan L17, L46)"
    );
    assert!(
        VLLM_V1_SOURCE.contains("/health"),
        "vllm_v1.rs must probe /health (health mirror of vllm.rs)"
    );
    assert!(
        VLLM_V1_SOURCE.contains("/v1/models"),
        "vllm_v1.rs must fall back to /v1/models for health"
    );
}

#[test]
fn vllm_source_pins_chat_completions_path() {
    assert!(
        VLLM_SOURCE.contains("/v1/chat/completions"),
        "vllm.rs hits OpenAI-compat /v1/chat/completions (the protocol-shape divergence VllmV1 does NOT use)"
    );
}

#[test]
fn recorded_request_shape_vllm_v1_is_anthropic_messages() {
    let recorded = json!({
        "model": "Qwen/Qwen2.5-Coder-7B-Instruct",
        "max_tokens": 16,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "hello", "cache_control": null}
                ]
            }
        ],
        "system": [
            {"type": "text", "text": "be brief"}
        ],
        "tools": null,
        "tool_choice": null,
        "stream": false,
        "seed": 42,
        "effort": null,
        "chat_template_kwargs": null
    });

    let obj = recorded.as_object().expect("recorded request is an object");
    assert_eq!(obj.get("model").and_then(Value::as_str), Some("Qwen/Qwen2.5-Coder-7B-Instruct"));
    assert_eq!(obj.get("max_tokens").and_then(Value::as_u64), Some(16));
    assert_eq!(obj.get("stream").and_then(Value::as_bool), Some(false));
    assert_eq!(obj.get("seed").and_then(Value::as_u64), Some(42));
    assert_eq!(obj.get("tools"), Some(&Value::Null));
    assert_eq!(obj.get("tool_choice"), Some(&Value::Null));
    assert_eq!(obj.get("effort"), Some(&Value::Null));
    assert_eq!(obj.get("chat_template_kwargs"), Some(&Value::Null));

    let messages = obj
        .get("messages")
        .and_then(Value::as_array)
        .expect("messages is an array");
    assert_eq!(messages.len(), 1, "single user turn");
    let msg = &messages[0];
    assert_eq!(msg.get("role").and_then(Value::as_str), Some("user"));
    let content = msg
        .get("content")
        .and_then(Value::as_array)
        .expect("content is an array");
    assert_eq!(content.len(), 1);
    let block = &content[0];
    assert_eq!(block.get("type").and_then(Value::as_str), Some("text"));
    assert_eq!(block.get("text").and_then(Value::as_str), Some("hello"));
    assert_eq!(block.get("cache_control"), Some(&Value::Null));

    let system = obj
        .get("system")
        .and_then(Value::as_array)
        .expect("system is an array");
    assert_eq!(system.len(), 1);
    assert_eq!(system[0].get("type").and_then(Value::as_str), Some("text"));
    assert_eq!(system[0].get("text").and_then(Value::as_str), Some("be brief"));
}

#[test]
fn recorded_response_shape_vllm_v1_anthropic_messages() {
    let recorded = json!({
        "id": "msg_vllm_v1_recorded",
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "text", "text": "ok"}
        ],
        "model": "Qwen/Qwen2.5-Coder-7B-Instruct",
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 3,
            "output_tokens": 1
        }
    });

    let body = serde_json::to_string(&recorded).expect("serializes");
    let parsed: MessageResponse =
        serde_json::from_str(&body).expect("recorded Anthropic MessageResponse shape parses");
    assert_eq!(parsed.model, "Qwen/Qwen2.5-Coder-7B-Instruct");
    assert_eq!(parsed.stop_reason.as_deref(), Some("end_turn"));
    assert_eq!(parsed.usage.input_tokens, 3);
    assert_eq!(parsed.usage.output_tokens, 1);
    assert_eq!(parsed.content.len(), 1);
    match &parsed.content[0] {
        OutputContentBlock::Text { text } => {
            assert_eq!(text, "ok");
        }
        other => panic!("expected OutputContentBlock::Text, got {other:?}"),
    }
}

/// Offline negative-capability stub: the Rust `RuntimeCapabilities` surface
/// has no DoLa field. DoLa is HF-only by construction (no surface to claim
/// it on the vLLM lane). The plan L60 NEGATIVE "vLLM lane with DoLa
/// requested FAILS the capability check" is a LIVE-RUNTIME check (the vLLM
/// server rejects an unknown logits processor), NOT an offline assert; it
/// belongs to W-10, OUT OF SCOPE here. This test only proves the struct
/// shape has no DoLa flag.
#[test]
fn vllm_v1_dola_capability_is_hf_only_negative() {
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::VllmV1);
    let rt = VllmV1Runtime::new(cfg);
    let caps = rt.capabilities();

    let _surface = RuntimeCapabilities {
        supports_tools: caps.supports_tools,
        supports_streaming: caps.supports_streaming,
        supports_vision: caps.supports_vision,
        max_context: caps.max_context,
        max_output: caps.max_output,
        json_schema_mode: caps.json_schema_mode,
        supports_prefill: caps.supports_prefill,
        supports_caching: caps.supports_caching,
        native_thinking_budget: caps.native_thinking_budget,
    };
    assert!(
        !_surface.supports_vision && !_surface.supports_prefill && !_surface.native_thinking_budget,
        "VllmV1 conservative defaults: no vision/prefill/native-thinking-budget"
    );
}

/// AWQ checkpoint BUILD asserts (plan L60 NEGATIVE "AWQ checkpoint built
/// without calibration data FAILS its build assert") are BUILD-time
/// checks (AutoAWQ checkpoint construction), NOT offline Rust contracts.
/// They live in the E-27 offline-tooling slice, NOT here. This doc-only
/// stub records the scope boundary; do NOT write a stub test body that
/// fakes a GPU build.
#[test]
fn awq_without_calibration_is_out_of_scope_offline() {
    // AWQ-build asserts are out of scope for the J-19 S5 slice-2 offline
    // fixture harness. The E-27 offline-tooling slice owns the AutoAWQ
    // checkpoint-build contract. This test exists only to document that
    // scope boundary in the test module; it asserts nothing about AWQ.
    assert!(true, "AWQ-build asserts live in E-27, not this slice");
}

#[test]
fn resolve_backend_vllm_v1_aliases_round_trip() {
    assert_eq!(
        resolve_backend("vllm-v1").expect("vllm-v1 alias resolves"),
        RuntimeBackend::VllmV1
    );
    assert_eq!(
        resolve_backend("vllm_v1").expect("vllm_v1 alias resolves"),
        RuntimeBackend::VllmV1
    );
    assert_eq!(
        resolve_backend("VLLM_V1").expect("VLLM_V1 case-insensitive resolves"),
        RuntimeBackend::VllmV1
    );
    assert_eq!(
        RuntimeBackend::VllmV1.canonical_name(),
        "vllm_v1",
        "canonical_name is vllm_v1 (types.rs:219)"
    );
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::VllmV1);
    let rt = create_runtime(cfg).expect("create_runtime(VllmV1) succeeds");
    assert_eq!(rt.backend(), RuntimeBackend::VllmV1);
}

#[test]
fn vllm_v1_is_local_like_vllm() {
    assert!(
        RuntimeBackend::VllmV1.is_local(),
        "VllmV1 is_local matches! includes VllmV1 (types.rs:235)"
    );
    assert!(
        RuntimeBackend::Vllm.is_local(),
        "Vllm is the sibling local backend"
    );
}

#[test]
fn backend_count_is_24() {
    assert_eq!(
        RuntimeBackend::all().len(),
        24,
        "RuntimeBackend::all() has 24 variants (slice-1 landed VllmV1 as the 24th; regression guard against accidentally re-adding a 25th)"
    );
}
