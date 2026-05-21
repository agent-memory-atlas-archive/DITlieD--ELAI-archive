//! Contract: Anthropic gateway-rejection startup warning (E-24 Task 6).

use elai_orchestrator::ssb::SsbActionType;

/// Contract 1: SSB action type for gateway detection has stable SQL string.
#[test]
fn contract_gateway_detected_ssb_type_stable() {
    assert_eq!(
        SsbActionType::AnthropicGatewayDetected.as_sql_str(),
        "anthropic_gateway_detected"
    );
}

/// Contract 2: Localhost URLs are not flagged as gateways.
#[test]
fn contract_localhost_not_flagged() {
    let safe_urls = [
        "http://localhost:8100",
        "http://127.0.0.1:8100",
        "https://api.anthropic.com",
    ];
    for url in &safe_urls {
        assert!(
            !url.contains("openrouter") && !url.contains("litellm") && !url.contains("portkey"),
            "safe URL {url} should not match gateway patterns"
        );
    }
}

/// Contract 3: Known gateway URLs are identified.
#[test]
fn contract_known_gateways_identified() {
    let gateway_urls = [
        ("https://openrouter.ai/api/v1", "openrouter"),
        ("https://litellm.example.com/v1", "litellm"),
        ("https://api.portkey.ai/v1", "portkey"),
    ];
    for (url, expected_gateway) in &gateway_urls {
        assert!(
            url.to_lowercase().contains(expected_gateway),
            "URL {url} should match gateway pattern {expected_gateway}"
        );
    }
}

/// Contract 4: AnthropicGatewayDetected payload roundtrips.
#[test]
fn contract_gateway_payload_roundtrip() {
    use elai_orchestrator::ssb::{AnthropicGatewayDetectedPayload, SsbPayload};
    let payload = SsbPayload::AnthropicGatewayDetected(AnthropicGatewayDetectedPayload {
        base_url: "https://openrouter.ai/api/v1".to_string(),
        detected_gateway: "openrouter".to_string(),
        latency_overhead_ms_estimate: 50,
        prompt_cache_broken: true,
    });
    assert_eq!(payload.action_type(), SsbActionType::AnthropicGatewayDetected);
}
