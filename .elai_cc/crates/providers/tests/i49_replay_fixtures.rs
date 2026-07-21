//! I-49 deterministic replay-fixture contract tests.
//!
//! These tests form the DETERMINISTIC LEG of the I-49 wiring contract.
//! They verify:
//!  1. HMAC-SHA256 against RFC 4231 test vectors (grounding: the raw crypto).
//!  2. SigV4 signing-key derivation (deterministic, known inputs).
//!  3. SigV4 signing produces a known-good Authorization header against
//!     a fixed test clock (grounding: the full signing chain).
//!  4. EventStream frame decode round-trip (valid frame -> headers + payload).
//!  5. Corrupted EventStream frame returns a TYPED error, never panics (R2).
//!  6. Azure response normalization extracts text + stop reason correctly.
//!  7. Backend enum count is 22 (I-49 added Bedrock + AzureOpenAi).
//!  8. resolve_backend accepts I-49 aliases.
//!  9. create_runtime dispatches Bedrock + AzureOpenAi arms (returns
//!     MissingField when creds absent — NOT NotImplemented).
//! 10. RuntimeBackend::all() includes both new variants.
//!
//! NO live network calls are made. NO billed API calls are made.
//! These are pure-offline correctness + wiring-contract tests.

use elai_providers::runtime::{
    // Bedrock protocol helpers
    crc32, decode_eventstream_frame, derive_signing_key,
    hex32, hmac_sha256, sha256_hex, sigv4_signing_headers,
    BedrockError,
    // Azure helpers
    build_azure_chat_request, normalize_azure_chat_response,
    // Registry
    create_runtime, resolve_backend,
};
use elai_providers::runtime::types::{
    CompletionConfig, Message, MessageRole, RuntimeAdapterError, RuntimeBackend,
    RuntimeFactoryConfig,
};
use serde_json::json;

// =========================================================================
// 1. HMAC-SHA256 RFC 4231 test vectors
// =========================================================================

/// RFC 4231 §4.2 — Test Case 1.
/// Key  = 0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b (20 bytes of 0x0b)
/// Data = "Hi There"
/// Expected HMAC-SHA-256 = b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
#[test]
fn hmac_sha256_rfc4231_test_vector_1() {
    let key = [0x0bu8; 20];
    let data = b"Hi There";
    let expected = "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7";
    let result = hmac_sha256(&key, data);
    assert_eq!(
        hex32(&result),
        expected,
        "RFC 4231 §4.2 test vector 1 HMAC-SHA256 mismatch"
    );
}

/// RFC 4231 §4.3 — Test Case 2.
/// Key  = "Jefe" (4 bytes)
/// Data = "what do ya want for nothing?"
/// Expected HMAC-SHA-256 = 5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843
/// Note: the RFC 4231 §4.3 document text has a known line-wrap truncation error;
/// the correct 64-hex-char value is confirmed by OpenSSL:
///   echo -n "what do ya want for nothing?" | openssl dgst -sha256 -hmac "Jefe"
#[test]
fn hmac_sha256_rfc4231_test_vector_2() {
    let key = b"Jefe";
    let data = b"what do ya want for nothing?";
    let expected = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843";
    let result = hmac_sha256(key, data);
    assert_eq!(
        hex32(&result),
        expected,
        "RFC 4231 §4.3 test vector 2 HMAC-SHA256 mismatch"
    );
}

/// RFC 4231 §4.4 — Test Case 3.
/// Key  = 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa (20 bytes of 0xaa)
/// Data = 0xdd repeated 50 times
/// Expected HMAC-SHA-256 = 773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe
#[test]
fn hmac_sha256_rfc4231_test_vector_3() {
    let key = [0xaau8; 20];
    let data = [0xddu8; 50];
    let expected = "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe";
    let result = hmac_sha256(&key, &data);
    assert_eq!(
        hex32(&result),
        expected,
        "RFC 4231 §4.4 test vector 3 HMAC-SHA256 mismatch"
    );
}

// =========================================================================
// 2. SigV4 signing-key derivation (deterministic)
// =========================================================================

/// Derive_signing_key is deterministic for the same inputs.
#[test]
fn sigv4_derive_signing_key_is_deterministic() {
    let k1 = derive_signing_key(
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        "20150830",
        "us-east-1",
        "iam",
    );
    let k2 = derive_signing_key(
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        "20150830",
        "us-east-1",
        "iam",
    );
    assert_eq!(k1, k2, "derive_signing_key must be deterministic");
    assert_ne!(k1, [0u8; 32], "signing key must be non-zero");
}

/// Different service names produce different signing keys.
#[test]
fn sigv4_different_service_different_key() {
    let k_iam = derive_signing_key(
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        "20150830",
        "us-east-1",
        "iam",
    );
    let k_bedrock = derive_signing_key(
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        "20150830",
        "us-east-1",
        "bedrock",
    );
    assert_ne!(
        k_iam, k_bedrock,
        "different services must produce different signing keys"
    );
}

// =========================================================================
// 3. SigV4 full signing pipeline — known-good Authorization header
// =========================================================================

/// The Authorization header produced by sigv4_signing_headers must be
/// structurally well-formed AND reproduce the exact known-good signature
/// for the fixed test inputs (I-49 stub_ban: "the fixture harness signs
/// real canonical requests against recorded keys and asserts the
/// known-good signature").
///
/// We verify with fixed clock (unix_secs = 1440938160 = 2015-08-30T12:36:00Z)
/// and the AWS test-suite canonical credentials. The expected signature
/// `31f0d4c006b5056791b678b8d5bf2269b887dac99783977b7c8b42dbcf684695`
/// is the ground truth computed via Python hmac + hashlib (RFC 2104
/// HMAC-SHA256) over the exact same canonical request the Rust code
/// builds — including RFC 3986 path encoding (`:` -> `%3A` in the
/// Bedrock model ID). The kSigning chain is independently cross-verified
/// against the aws-crt-nodejs test suite get-vanilla vector in the
/// `sigv4_derive_signing_key_known_good` unit test (bedrock.rs).
///
/// Note: 1440938160 verified by: datetime(2015,8,30,12,36,0,utc).timestamp()
#[test]
fn sigv4_authorization_header_structure() {
    let body = br#"{"prompt":"hello"}"#;
    let headers = sigv4_signing_headers(
        "POST",
        "bedrock-runtime.us-east-1.amazonaws.com",
        "/model/anthropic.claude-3-5-sonnet-20241022-v2:0/converse",
        body,
        "us-east-1",
        "AKIDEXAMPLE",
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        1_440_938_160, // 2015-08-30T12:36:00Z (verified: datetime.utcfromtimestamp)
    );

    // The x-amz-date must match the injected clock.
    assert_eq!(
        headers.x_amz_date, "20150830T123600Z",
        "x-amz-date must match injected unix_secs"
    );

    // Authorization must start with the correct algorithm.
    assert!(
        headers.authorization.starts_with("AWS4-HMAC-SHA256 "),
        "authorization must start with AWS4-HMAC-SHA256"
    );

    // Credential scope must contain the short date + region + service.
    assert!(
        headers.authorization.contains("Credential=AKIDEXAMPLE/20150830/us-east-1/bedrock/aws4_request"),
        "Credential scope missing or wrong in: {}",
        headers.authorization
    );

    // SignedHeaders must include all four signed header names.
    assert!(
        headers.authorization.contains("SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date"),
        "SignedHeaders field wrong in: {}",
        headers.authorization
    );

    // Signature must be exactly 64 hex chars.
    let sig_start = headers
        .authorization
        .rfind("Signature=")
        .expect("Signature= must be present");
    let sig = &headers.authorization[sig_start + "Signature=".len()..];
    assert_eq!(sig.len(), 64, "Signature must be 64 hex chars, got {}", sig.len());
    assert!(
        sig.chars().all(|c| c.is_ascii_hexdigit()),
        "Signature must be all hex digits: {sig}"
    );

    // I-49 stub_ban: assert the EXACT known-good signature, not just
    // structure. Ground truth computed via Python RFC 2104 HMAC-SHA256
    // over the identical canonical request the Rust code builds
    // (including RFC 3986 path encoding of the Bedrock model ID).
    assert_eq!(
        sig,
        "31f0d4c006b5056791b678b8d5bf2269b887dac99783977b7c8b42dbcf684695",
        "Signature must reproduce the known-good value for the fixed test inputs"
    );

    // x-amz-content-sha256 must be the SHA-256 of the body.
    let expected_body_hash = sha256_hex(body);
    assert_eq!(
        headers.x_amz_content_sha256, expected_body_hash,
        "x-amz-content-sha256 must be SHA-256 of request body"
    );
}

/// Signing is deterministic for the same inputs.
#[test]
fn sigv4_signing_is_deterministic() {
    let body = b"{}";
    let h1 = sigv4_signing_headers(
        "POST",
        "bedrock-runtime.us-east-1.amazonaws.com",
        "/model/test/converse",
        body,
        "us-east-1",
        "AKID",
        "SECRET",
        1_000_000,
    );
    let h2 = sigv4_signing_headers(
        "POST",
        "bedrock-runtime.us-east-1.amazonaws.com",
        "/model/test/converse",
        body,
        "us-east-1",
        "AKID",
        "SECRET",
        1_000_000,
    );
    assert_eq!(h1, h2, "sigv4_signing_headers must be deterministic");
}

// =========================================================================
// 4. EventStream frame round-trip (valid frame)
// =========================================================================

fn build_test_eventstream_frame(header_name: &[u8], header_value: &[u8], payload: &[u8]) -> Vec<u8> {
    const PRELUDE_LEN: usize = 8;
    const PRELUDE_CRC_LEN: usize = 4;
    const MSG_CRC_LEN: usize = 4;

    let mut header_bytes = Vec::new();
    header_bytes.push(header_name.len() as u8);
    header_bytes.extend_from_slice(header_name);
    header_bytes.push(7u8); // type = string
    let vlen = header_value.len() as u16;
    header_bytes.extend_from_slice(&vlen.to_be_bytes());
    header_bytes.extend_from_slice(header_value);

    let headers_len = header_bytes.len() as u32;
    let total_len =
        (PRELUDE_LEN + PRELUDE_CRC_LEN + header_bytes.len() + payload.len() + MSG_CRC_LEN) as u32;

    let mut frame = Vec::new();
    frame.extend_from_slice(&total_len.to_be_bytes());
    frame.extend_from_slice(&headers_len.to_be_bytes());
    let prelude_crc = crc32(&frame[..8]);
    frame.extend_from_slice(&prelude_crc.to_be_bytes());
    frame.extend_from_slice(&header_bytes);
    frame.extend_from_slice(payload);
    let msg_crc = crc32(&frame);
    frame.extend_from_slice(&msg_crc.to_be_bytes());
    frame
}

/// Valid EventStream frame decodes byte-exactly (header + payload round-trip).
#[test]
fn eventstream_valid_frame_decodes_byte_exactly() {
    let frame = build_test_eventstream_frame(
        b":event-type",
        b"contentBlockDelta",
        b"{\"type\":\"contentBlockDelta\",\"delta\":{\"text\":\"hello\"}}",
    );
    let msg = decode_eventstream_frame(&frame).expect("valid frame must decode");
    assert_eq!(
        msg.headers.get(":event-type").map(|s| s.as_str()),
        Some("contentBlockDelta"),
        "header :event-type must round-trip"
    );
    assert_eq!(
        msg.payload,
        b"{\"type\":\"contentBlockDelta\",\"delta\":{\"text\":\"hello\"}}",
        "payload must round-trip byte-exactly"
    );
}

/// Empty payload frame also decodes correctly.
#[test]
fn eventstream_empty_payload_decodes() {
    let frame = build_test_eventstream_frame(b":message-type", b"event", &[]);
    let msg = decode_eventstream_frame(&frame).expect("empty payload frame must decode");
    assert!(msg.payload.is_empty());
    assert_eq!(
        msg.headers.get(":message-type").map(|s| s.as_str()),
        Some("event")
    );
}

// =========================================================================
// 5. Corrupted EventStream frame -> TYPED error, no panic (R2 fail-closed)
// =========================================================================

/// Frame too short returns FrameTooShort (R2 fail-closed, not panic).
#[test]
fn eventstream_corrupted_frame_too_short_returns_typed_error() {
    let frame = [0u8; 4]; // 4 bytes < MIN_MESSAGE_LEN (16)
    let err = decode_eventstream_frame(&frame).expect_err("short frame must return error");
    assert!(
        matches!(err, BedrockError::FrameTooShort(_)),
        "expected FrameTooShort, got {err:?}"
    );
}

/// Frame with wrong prelude CRC returns PreludeCrcMismatch (not panic).
#[test]
fn eventstream_prelude_crc_mismatch_returns_typed_error() {
    const MSG_CRC_LEN: usize = 4;
    const PRELUDE_LEN: usize = 8;
    const PRELUDE_CRC_LEN: usize = 4;

    let payload = b"data";
    let total_len: u32 = (PRELUDE_LEN + PRELUDE_CRC_LEN + payload.len() + MSG_CRC_LEN) as u32;
    let headers_len: u32 = 0;

    let mut frame = Vec::new();
    frame.extend_from_slice(&total_len.to_be_bytes());
    frame.extend_from_slice(&headers_len.to_be_bytes());
    // Deliberately wrong prelude CRC.
    frame.extend_from_slice(&0xDEAD_BEEFu32.to_be_bytes());
    frame.extend_from_slice(payload);
    let msg_crc = crc32(&frame);
    frame.extend_from_slice(&msg_crc.to_be_bytes());

    let err = decode_eventstream_frame(&frame).expect_err("bad CRC must return error");
    assert!(
        matches!(err, BedrockError::PreludeCrcMismatch { .. }),
        "expected PreludeCrcMismatch, got {err:?}"
    );
}

/// Frame with wrong message CRC returns MessageCrcMismatch (not panic).
#[test]
fn eventstream_message_crc_mismatch_returns_typed_error() {
    // Build a valid frame then corrupt the message CRC at the end.
    let mut frame = build_test_eventstream_frame(b":event-type", b"chunk", b"payload");
    let n = frame.len();
    // Flip last byte (part of message CRC).
    frame[n - 1] ^= 0xff;

    let err = decode_eventstream_frame(&frame).expect_err("bad msg CRC must return error");
    assert!(
        matches!(err, BedrockError::MessageCrcMismatch { .. }),
        "expected MessageCrcMismatch, got {err:?}"
    );
}

// =========================================================================
// 6. Azure response normalization
// =========================================================================

/// Happy-path: text + stop reason + token counts are extracted correctly.
#[test]
fn azure_normalize_response_happy_path() {
    let body = json!({
        "id": "chatcmpl-abc",
        "model": "gpt-4o-2024-08-06",
        "choices": [{
            "message": { "role": "assistant", "content": "The answer is 42." },
            "finish_reason": "stop",
            "index": 0
        }],
        "usage": { "prompt_tokens": 20, "completion_tokens": 6 }
    });
    let resp = normalize_azure_chat_response("gpt-4o", &body)
        .expect("valid response must normalize");
    assert_eq!(resp.text, "The answer is 42.");
    assert_eq!(resp.stop_reason.as_deref(), Some("end_turn")); // "stop" -> "end_turn"
    assert_eq!(resp.usage, (20, 6));
    assert_eq!(resp.model, "gpt-4o-2024-08-06"); // echoed from response
}

/// Missing "choices" field returns a typed error (not panic).
#[test]
fn azure_normalize_missing_choices_is_typed_error() {
    let body = json!({ "id": "chatcmpl-abc", "model": "gpt-4o" });
    let err = normalize_azure_chat_response("gpt-4o", &body).expect_err("must error");
    assert!(matches!(err, RuntimeAdapterError::Other(_)));
}

/// Empty choices array returns a typed error.
#[test]
fn azure_normalize_empty_choices_is_typed_error() {
    let body = json!({
        "id": "chatcmpl-abc",
        "model": "gpt-4o",
        "choices": []
    });
    let err = normalize_azure_chat_response("gpt-4o", &body).expect_err("must error");
    assert!(matches!(err, RuntimeAdapterError::Other(_)));
}

/// Request body includes system message when config.system is set.
#[test]
fn azure_build_request_includes_system_message() {
    let messages = vec![Message { role: MessageRole::User, content: "Hi".to_string() }];
    let config = CompletionConfig {
        system: Some("You are helpful.".to_string()),
        max_tokens: 50,
        ..CompletionConfig::default()
    };
    let body = build_azure_chat_request("gpt-4o", &messages, &config, false);
    let msgs = body["messages"].as_array().expect("messages array");
    assert_eq!(msgs[0]["role"], json!("system"));
    assert_eq!(msgs[0]["content"], json!("You are helpful."));
    assert_eq!(msgs[1]["role"], json!("user"));
}

// =========================================================================
// 7. Backend enum count (I-49 added Bedrock + AzureOpenAi -> 22;
//    J-30 T2 added Custom -> 23; J-19 added VllmV1 -> 24)
// =========================================================================

#[test]
fn runtime_backend_all_count_is_24() {
    let all = RuntimeBackend::all();
    assert_eq!(
        all.len(),
        24,
        "RuntimeBackend::all() must have 24 entries after J-19 added VllmV1; got {}",
        all.len()
    );
}

// =========================================================================
// 8. resolve_backend accepts I-49 aliases
// =========================================================================

#[test]
fn resolve_backend_bedrock_aliases() {
    assert_eq!(
        resolve_backend("bedrock").expect("canonical"),
        RuntimeBackend::Bedrock
    );
    assert_eq!(
        resolve_backend("aws_bedrock").expect("alias"),
        RuntimeBackend::Bedrock
    );
    assert_eq!(
        resolve_backend("aws-bedrock").expect("alias"),
        RuntimeBackend::Bedrock
    );
    assert_eq!(
        resolve_backend("BEDROCK").expect("case-insensitive"),
        RuntimeBackend::Bedrock
    );
}

#[test]
fn resolve_backend_azure_openai_aliases() {
    assert_eq!(
        resolve_backend("azure_openai").expect("canonical"),
        RuntimeBackend::AzureOpenAi
    );
    assert_eq!(
        resolve_backend("azure").expect("alias"),
        RuntimeBackend::AzureOpenAi
    );
    assert_eq!(
        resolve_backend("azure-openai").expect("alias"),
        RuntimeBackend::AzureOpenAi
    );
    assert_eq!(
        resolve_backend("AZURE_OPENAI").expect("case-insensitive"),
        RuntimeBackend::AzureOpenAi
    );
}

// =========================================================================
// 9. create_runtime dispatches both new arms (MissingField, not NotImplemented)
// =========================================================================

/// Bedrock arm returns MissingField when creds absent (NOT NotImplemented).
/// This confirms the adapter EXISTS and the arm is wired into create_runtime.
#[test]
fn create_runtime_bedrock_missing_creds_returns_missing_field_not_not_implemented() {
    // Ensure env vars are not set to avoid accidental credential pickup.
    // We cannot fully guarantee clean env in all CI environments, but the
    // runtime_profile + api_key paths take priority over env vars.
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Bedrock);
    // cfg has no api_key and no runtime_profile, so from_factory_config
    // will look for AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY env vars.
    // In a clean env this is MissingField; in an env with AWS creds it
    // succeeds. We accept both outcomes — the point is it is NOT NotImplemented.
    match create_runtime(cfg) {
        Ok(adapter) => {
            // Creds were present in env — adapter was constructed successfully.
            assert_eq!(adapter.backend(), RuntimeBackend::Bedrock);
        }
        Err(RuntimeAdapterError::MissingField { backend, .. }) => {
            assert_eq!(backend, RuntimeBackend::Bedrock);
        }
        Err(RuntimeAdapterError::NotImplemented { backend, .. }) => {
            panic!(
                "Bedrock create_runtime must NOT return NotImplemented; got NotImplemented for {backend:?}"
            );
        }
        Err(other) => {
            panic!("unexpected error from Bedrock create_runtime: {other:?}");
        }
    }
}

/// AzureOpenAi arm returns MissingField when creds absent (NOT NotImplemented).
#[test]
fn create_runtime_azure_missing_creds_returns_missing_field_not_not_implemented() {
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::AzureOpenAi);
    match create_runtime(cfg) {
        Ok(adapter) => {
            assert_eq!(adapter.backend(), RuntimeBackend::AzureOpenAi);
        }
        Err(RuntimeAdapterError::MissingField { backend, .. }) => {
            assert_eq!(backend, RuntimeBackend::AzureOpenAi);
        }
        Err(RuntimeAdapterError::NotImplemented { backend, .. }) => {
            panic!(
                "AzureOpenAi create_runtime must NOT return NotImplemented; got NotImplemented for {backend:?}"
            );
        }
        Err(other) => {
            panic!("unexpected error from AzureOpenAi create_runtime: {other:?}");
        }
    }
}

// =========================================================================
// 10. RuntimeBackend::all() includes both new variants
// =========================================================================

#[test]
fn runtime_backend_all_includes_bedrock_and_azure() {
    let all = RuntimeBackend::all();
    assert!(
        all.iter().any(|b| *b == RuntimeBackend::Bedrock),
        "RuntimeBackend::all() must include Bedrock"
    );
    assert!(
        all.iter().any(|b| *b == RuntimeBackend::AzureOpenAi),
        "RuntimeBackend::all() must include AzureOpenAi"
    );
}

/// canonical_name round-trips through resolve_backend for both new variants.
#[test]
fn canonical_name_round_trips_for_new_backends() {
    for backend in [RuntimeBackend::Bedrock, RuntimeBackend::AzureOpenAi] {
        let name = backend.canonical_name();
        let resolved = resolve_backend(name)
            .unwrap_or_else(|_| panic!("canonical name '{name}' must resolve for {backend:?}"));
        assert_eq!(resolved, backend, "canonical name round-trip failed for {backend:?}");
    }
}

/// Both new backends are NOT local (cloud APIs, not local hardware).
#[test]
fn new_backends_are_not_local() {
    assert!(!RuntimeBackend::Bedrock.is_local(), "Bedrock is a cloud backend");
    assert!(!RuntimeBackend::AzureOpenAi.is_local(), "AzureOpenAi is a cloud backend");
}
