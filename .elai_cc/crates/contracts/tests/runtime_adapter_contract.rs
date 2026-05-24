//! B-37 Phase 4M — RuntimeAdapter trait contract.
//!
//! Pins the load-bearing invariants of `elai_providers::runtime`:
//!   - 11 canonical RuntimeBackend variants, lower-snake_case, unique.
//!   - `resolve_backend()` round-trips canonical names and well-known
//!     aliases.
//!   - `resolve_backend()` miss carries the full supported list.
//!   - `RuntimePermissionMode::default() == AcceptEdits` (NEVER
//!     DangerouslySkipPermissions; spec phase-4M.txt + WIRING_CONTRACT
//!     stub_ban).
//!   - 4 PermissionMode variants serde-round-trip (camelCase per
//!     types.rs:129).
//!   - 3 RuntimeHealth variants serde-round-trip with `status` tag.
//!   - `create_runtime(...)` returns `Box<dyn RuntimeAdapter>` whose
//!     `backend()` is reachable through the trait object.
//!   - `cli_common::sanitize_stdout_line` strips ANSI + caps at
//!     `MAX_LINE_LEN`.
//!   - `cli_common::enforce_permission_opt_in` refuses
//!     DangerouslySkipPermissions unless `allow_danger=true`.
//!
//! This file is the TDD "red" baseline for B-37A+B. Tests that depend
//! on unmerged surface area are marked `#[ignore]` with a
//! `// TODO: ...` comment so the EXECUTING wave knows what to wire.

#[allow(unused_imports)]
use elai_providers::runtime::RuntimeAdapter; // brings backend()/capabilities() into scope for dyn calls
use elai_providers::runtime::{
    create_runtime, resolve_backend, CompletionConfig, Message, MessageRole, RuntimeAdapterError,
    RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth, RuntimePermissionMode, StreamChunk,
};

#[test]
fn canonical_names_are_unique_and_lowercase() {
    let mut seen = std::collections::BTreeSet::new();
    for backend in RuntimeBackend::all() {
        let name = backend.canonical_name();
        assert!(
            seen.insert(name),
            "duplicate canonical name `{name}` in RuntimeBackend::all()"
        );
        assert_eq!(
            name,
            name.to_ascii_lowercase(),
            "canonical name `{name}` must be lowercase"
        );
        assert!(
            !name.contains(' '),
            "canonical name `{name}` must be snake_case (no spaces)"
        );
        assert!(
            !name.contains('-'),
            "canonical name `{name}` must be snake_case (no hyphens)"
        );
    }
    assert!(
        seen.len() >= 11,
        "expected at least 11 RuntimeBackend variants, got {}",
        seen.len()
    );
}

#[test]
fn canonical_name_round_trips_via_resolve_backend() {
    for backend in RuntimeBackend::all() {
        let name = backend.canonical_name();
        let resolved = resolve_backend(name)
            .unwrap_or_else(|e| panic!("canonical name `{name}` failed to resolve: {e:?}"));
        assert_eq!(resolved, *backend, "round-trip mismatch for `{name}`");
    }
}

#[test]
fn resolve_backend_unknown_returns_full_supported_list() {
    let err = resolve_backend("definitely-not-a-backend").expect_err("unknown must miss");
    assert!(
        err.supported.len() >= 11,
        "ResolveBackendError must include all {}+ canonical names, got {}",
        11,
        err.supported.len()
    );
    // Sanity: the candidate is preserved (lowercased per mod.rs:54).
    assert_eq!(err.candidate, "definitely-not-a-backend");
    // Sanity: well-known canonical names appear in the supported list.
    for must_have in ["sidecar", "anthropic_api", "claude_code", "ollama", "vllm"] {
        assert!(
            err.supported.iter().any(|s| s == must_have),
            "supported list missing `{must_have}`; got {:?}",
            err.supported
        );
    }
}

#[test]
fn resolve_backend_aliases() {
    // Aliases pulled verbatim from mod.rs:55-67. These are the
    // load-bearing entries the CLI / dashboard / config will use.
    let cases = [
        ("claude", RuntimeBackend::ClaudeCode),
        ("claude_code", RuntimeBackend::ClaudeCode),
        ("anthropic", RuntimeBackend::AnthropicApi),
        ("anthropic_api", RuntimeBackend::AnthropicApi),
        ("api", RuntimeBackend::AnthropicApi),
        ("claude_api", RuntimeBackend::AnthropicApi),
        ("sidecar", RuntimeBackend::Sidecar),
        ("local", RuntimeBackend::Sidecar),
        ("codex", RuntimeBackend::CodexCli),
        ("codex_cli", RuntimeBackend::CodexCli),
        ("opencode", RuntimeBackend::OpenCode),
        ("opencode_cli", RuntimeBackend::OpenCode),
        ("hermes", RuntimeBackend::Hermes),
        ("hermes_cli", RuntimeBackend::Hermes),
        ("gemini", RuntimeBackend::GeminiCli),
        ("gemini_cli", RuntimeBackend::GeminiCli),
        ("kiro", RuntimeBackend::KiroCli),
        ("kiro_cli", RuntimeBackend::KiroCli),
        ("copilot", RuntimeBackend::CopilotCli),
        ("copilot_cli", RuntimeBackend::CopilotCli),
        ("ollama", RuntimeBackend::Ollama),
        ("vllm", RuntimeBackend::Vllm),
    ];
    for (alias, expected) in cases {
        let got = resolve_backend(alias)
            .unwrap_or_else(|e| panic!("alias `{alias}` failed to resolve: {e:?}"));
        assert_eq!(got, expected, "alias `{alias}` resolved to {got:?}");
    }
}

#[test]
fn resolve_backend_is_case_insensitive() {
    assert_eq!(
        resolve_backend("SIDECAR").expect("uppercase must resolve"),
        RuntimeBackend::Sidecar
    );
    assert_eq!(
        resolve_backend("  Ollama  ").expect("whitespace must resolve"),
        RuntimeBackend::Ollama
    );
}

#[test]
fn runtime_permission_mode_default_is_accept_edits() {
    assert_eq!(
        RuntimePermissionMode::default(),
        RuntimePermissionMode::AcceptEdits,
        "spec phase-4M.txt + WIRING_CONTRACT stub_ban: default MUST be AcceptEdits"
    );
}

#[test]
fn runtime_permission_mode_four_variants_serde_round_trip() {
    // types.rs:129 declares `#[serde(rename_all = "camelCase")]`.
    let cases = [
        (RuntimePermissionMode::AcceptEdits, "\"acceptEdits\""),
        (RuntimePermissionMode::Plan, "\"plan\""),
        (
            RuntimePermissionMode::DangerouslySkipPermissions,
            "\"dangerouslySkipPermissions\"",
        ),
        (RuntimePermissionMode::Default, "\"default\""),
    ];
    for (mode, expected_json) in cases {
        let encoded = serde_json::to_string(&mode)
            .unwrap_or_else(|e| panic!("serialize {mode:?} failed: {e}"));
        assert_eq!(
            encoded, expected_json,
            "serialize mismatch for {mode:?}"
        );
        let decoded: RuntimePermissionMode = serde_json::from_str(&encoded)
            .unwrap_or_else(|e| panic!("deserialize {encoded} failed: {e}"));
        assert_eq!(decoded, mode, "round-trip mismatch for {mode:?}");
    }
}

#[test]
fn runtime_health_serde_round_trip() {
    // types.rs:185 declares `#[serde(tag = "status", rename_all = "snake_case")]`.
    let cases = [
        RuntimeHealth::Ready,
        RuntimeHealth::Degraded {
            reason: "slow".to_string(),
        },
        RuntimeHealth::Unavailable {
            reason: "binary missing".to_string(),
        },
    ];
    for health in cases {
        let encoded = serde_json::to_string(&health)
            .unwrap_or_else(|e| panic!("serialize {health:?} failed: {e}"));
        // Sanity: every variant carries a `"status":` discriminant.
        assert!(
            encoded.contains("\"status\""),
            "RuntimeHealth must serialize with a `status` tag, got {encoded}"
        );
        let decoded: RuntimeHealth = serde_json::from_str(&encoded)
            .unwrap_or_else(|e| panic!("deserialize {encoded} failed: {e}"));
        assert_eq!(decoded, health, "round-trip mismatch for {health:?}");
    }
}

#[test]
fn create_runtime_returns_box_dyn_for_every_backend() {
    // For each backend, create_runtime() must hand back a usable
    // Box<dyn RuntimeAdapter>. Stubs (Ollama, Vllm) succeed at
    // construction and only report Unavailable through health_check.
    // Sidecar + AnthropicApi rely on factory fields that aren't
    // present in a "for_backend"-only config, so we expect either
    // Ok(adapter) or Err(MissingField); both are acceptable outcomes,
    // but NEVER a panic.
    for backend in RuntimeBackend::all() {
        let cfg = RuntimeFactoryConfig::for_backend(*backend);
        let result = create_runtime(cfg);
        match result {
            Ok(adapter) => {
                // The trait object MUST report the backend we asked for.
                assert_eq!(
                    adapter.backend(),
                    *backend,
                    "create_runtime returned wrong backend"
                );
                // capabilities() is cheap and must not panic.
                let _caps = adapter.capabilities();
            }
            Err(RuntimeAdapterError::MissingField { backend: b, .. }) => {
                // Sidecar / AnthropicApi may reject construction
                // without auth; that's acceptable so long as the
                // backend is identified.
                assert_eq!(b, *backend, "MissingField must echo requested backend");
            }
            Err(RuntimeAdapterError::Provider(_)) => {
                // Auth/HTTP failure during construction is acceptable
                // for cloud-shaped backends without credentials.
            }
            Err(RuntimeAdapterError::Other(msg)) => {
                // Platform-restricted backends (e.g. Mlx requires
                // Darwin+arm64) and CLI-launch backends may surface a
                // setup-time Other error on unsupported hosts. Accept
                // that as a non-panic outcome; the contract is just
                // "never panic on a known variant".
                // E-51 + C-44 added Mlx/LlamaCpp. C-25 added more cloud.
                let _ = msg;
            }
            Err(RuntimeAdapterError::NotImplemented { backend: b, .. }) => {
                // D-2 — PPM is a verifier-only discovery slot. Its
                // `create_runtime` arm intentionally returns NotImplemented
                // as a typed sentinel against accidental generation-path
                // routing. See workflow/reasoning/D-2-decisions.md.
                assert_eq!(
                    b, *backend,
                    "NotImplemented sentinel must echo requested backend"
                );
            }
            Err(other) => panic!(
                "create_runtime({backend:?}) returned unexpected error: {other:?}"
            ),
        }
    }
}

#[test]
fn cli_common_sanitize_stdout_strips_ansi_and_caps_length() {
    use elai_providers::runtime::cli_common::{sanitize_stdout_line, MAX_LINE_LEN};
    // ANSI red + text + reset.
    let ansi = "\x1b[31mhello\x1b[0m";
    let clean = sanitize_stdout_line(ansi);
    assert_eq!(clean, "hello", "ANSI escapes must be stripped");
    // Length cap.
    let huge = "a".repeat(2 * MAX_LINE_LEN);
    let capped = sanitize_stdout_line(&huge);
    assert!(
        capped.len() <= MAX_LINE_LEN + 128,
        "sanitized line exceeded MAX_LINE_LEN+128: {} > {}",
        capped.len(),
        MAX_LINE_LEN + 128
    );
    assert!(
        capped.contains("[truncated"),
        "truncation marker missing from oversized output"
    );
    // Control chars (NUL, BEL) must be dropped but TAB/NL preserved.
    let mixed = "a\tb\nc\x07d\x00e";
    let clean = sanitize_stdout_line(mixed);
    assert!(clean.contains('\t'));
    assert!(clean.contains('\n'));
    assert!(!clean.contains('\x07'));
    assert!(!clean.contains('\x00'));
}

#[test]
fn cli_common_parse_malformed_jsonl_does_not_panic() {
    // cli_common doesn't expose a `parse_jsonl_event` entrypoint as a
    // single function — each per-CLI adapter parses inline. The
    // contract we DO want to pin here: feeding malformed JSON through
    // `sanitize_stdout_line` returns the malformed text unchanged
    // (sanitization is a separate concern from JSON parsing), and
    // serde_json::from_str on that line yields a recoverable Err that
    // adapters drop with a Warning rather than aborting the stream.
    use elai_providers::runtime::cli_common::sanitize_stdout_line;
    let malformed = "{not json";
    let clean = sanitize_stdout_line(malformed);
    assert_eq!(clean, malformed, "sanitize_stdout_line must be lossless on plain ASCII");
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&clean);
    assert!(parsed.is_err(), "malformed JSONL must surface a recoverable Err");
}

#[test]
fn tier_c_spawn_default_refuses_dangerous_without_opt_in() {
    // `enforce_permission_opt_in` (cli_common.rs:296) is the central
    // gate. With allow_danger=false + permission_mode=Dangerous it MUST
    // return PermissionDenied. With allow_danger=true it MUST permit.
    use elai_providers::runtime::cli_common::enforce_permission_opt_in;

    let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode);
    cfg.permission_mode = RuntimePermissionMode::DangerouslySkipPermissions;
    cfg.allow_danger = false;
    match enforce_permission_opt_in(&cfg) {
        Err(RuntimeAdapterError::PermissionDenied { .. }) => {}
        other => panic!(
            "expected PermissionDenied with allow_danger=false, got {other:?}"
        ),
    }
    cfg.allow_danger = true;
    enforce_permission_opt_in(&cfg).expect("allow_danger=true must permit");
}

#[test]
fn completion_config_default_is_deterministic() {
    // Locked decision (B-37-decisions.md): max_tokens=512, temperature=0.0.
    let cfg = CompletionConfig::default();
    assert_eq!(cfg.max_tokens, 512);
    assert_eq!(cfg.temperature, 0.0);
    assert!(cfg.seed.is_none());
    assert!(cfg.stop_sequences.is_empty());
}

#[test]
fn message_role_serde_round_trip_lowercase() {
    for role in [
        MessageRole::User,
        MessageRole::Assistant,
        MessageRole::System,
        MessageRole::Tool,
    ] {
        let s = serde_json::to_string(&role).expect("serialize MessageRole");
        // types.rs:305 declares `#[serde(rename_all = "lowercase")]`.
        assert_eq!(s, format!("\"{}\"", role.as_str()));
        let back: MessageRole = serde_json::from_str(&s).expect("deserialize MessageRole");
        assert_eq!(back, role);
    }
}

#[test]
fn stream_chunk_serde_round_trip_snake_case_tag() {
    let cases = [
        StreamChunk::TextDelta {
            text: "hi".to_string(),
        },
        StreamChunk::Stop {
            reason: Some("end_turn".to_string()),
        },
        StreamChunk::Warning {
            message: "rate".to_string(),
        },
    ];
    for chunk in cases {
        let s = serde_json::to_string(&chunk).expect("serialize StreamChunk");
        assert!(s.contains("\"type\""), "StreamChunk must serialize with `type` tag, got {s}");
        let back: StreamChunk = serde_json::from_str(&s).expect("deserialize StreamChunk");
        assert_eq!(back, chunk);
    }
}

#[test]
fn message_envelope_round_trips() {
    let msg = Message {
        role: MessageRole::User,
        content: "ping".to_string(),
    };
    let s = serde_json::to_string(&msg).expect("serialize Message");
    let back: Message = serde_json::from_str(&s).expect("deserialize Message");
    assert_eq!(back, msg);
}
