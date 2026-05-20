//! Multi-runtime adapter layer (B-37 Phase 4M).
//!
//! Provides a single `RuntimeAdapter` trait covering the 11 backends
//! enumerated in `RuntimeBackend`. B-37A wired Sidecar + AnthropicApi
//! end-to-end; B-37B lands the 7 CLI subprocess adapters
//! (ClaudeCodeRuntime, CodexCliRuntime, OpenCodeRuntime, HermesRuntime,
//! GeminiCliRuntime, KiroCliRuntime, CopilotCliRuntime). Ollama + Vllm
//! remain stubs until B-37C.
//!
//! Subsequent sub-cycles fill in the remaining stubs:
//! - B-37C: Ollama + vLLM native HTTP adapters; PerRoleRuntime.
//! - B-37D: `/runtime` CLI command + dashboard RuntimePanel.
//!
//! See `workflow/reasoning/B-37-decisions.md` for the design contract.

pub mod anthropic_api;
pub mod cli_common;
pub mod claude_code;
pub mod codex_cli;
pub mod copilot_cli;
pub mod event_sink;
pub mod gemini_cli;
pub mod hermes;
pub mod isolation;
pub mod kiro_cli;
pub mod ollama;
pub mod opencode;
pub mod secret;
pub mod sidecar;
/// E-35: typed Rust-side hook for the sidecar's XGrammarLogitsProcessor.
/// See `sidecar_xgrammar_hook.rs` for the wire-format contract + the
/// `followups_for_other_sessions` block (Python sidecar wiring is owned by
/// a separate orchestration session).
pub mod sidecar_xgrammar_hook;
pub mod types;
pub mod vllm;

pub use anthropic_api::AnthropicApiRuntime;
pub use claude_code::ClaudeCodeRuntime;
pub use codex_cli::CodexCliRuntime;
pub use copilot_cli::CopilotCliRuntime;
pub use event_sink::{
    install_runtime_event_sink, redact_argv, runtime_event_sink, NoopRuntimeEventSink,
    RuntimeEvent, RuntimeEventSink,
};
pub use gemini_cli::GeminiCliRuntime;
pub use hermes::HermesRuntime;
pub use kiro_cli::KiroCliRuntime;
pub use ollama::OllamaRuntime;
pub use opencode::OpenCodeRuntime;
pub use secret::SecretString;
pub use sidecar::SidecarRuntime;
pub use sidecar_xgrammar_hook::{
    build_xgrammar_request, verify_response_contract, xgrammar_cache_key, xgrammar_schema_id,
    SidecarXGrammarHook, XGrammarHookError,
};
pub use vllm::VllmRuntime;
pub use types::{
    unavailable, AuthSource, CompletionConfig, CompletionResponse, Message, MessageRole,
    ResolveBackendError, RuntimeAdapter, RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities,
    RuntimeFactoryConfig, RuntimeFuture, RuntimeHealth, RuntimeHealthFuture,
    RuntimePermissionMode, RuntimeStream, SidecarRuntimeConfig, StreamChunk,
};

/// Resolve a config-file or CLI candidate string to a canonical
/// `RuntimeBackend`. Case- and whitespace-insensitive. Mirrors
/// ouroboros's `resolve_runtime_backend_name`, extended with the
/// supported-list error per spec phase-4M.txt:119-123.
pub fn resolve_backend(candidate: &str) -> Result<RuntimeBackend, ResolveBackendError> {
    let normalised = candidate.trim().to_ascii_lowercase();
    let resolved = match normalised.as_str() {
        "sidecar" | "local" => Some(RuntimeBackend::Sidecar),
        "anthropic_api" | "anthropic" | "api" | "claude_api" => Some(RuntimeBackend::AnthropicApi),
        "claude_code" | "claude" => Some(RuntimeBackend::ClaudeCode),
        "codex_cli" | "codex" => Some(RuntimeBackend::CodexCli),
        "opencode" | "opencode_cli" => Some(RuntimeBackend::OpenCode),
        "hermes" | "hermes_cli" => Some(RuntimeBackend::Hermes),
        "gemini_cli" | "gemini" => Some(RuntimeBackend::GeminiCli),
        "kiro" | "kiro_cli" => Some(RuntimeBackend::KiroCli),
        "copilot" | "copilot_cli" => Some(RuntimeBackend::CopilotCli),
        "ollama" => Some(RuntimeBackend::Ollama),
        "vllm" => Some(RuntimeBackend::Vllm),
        _ => None,
    };
    match resolved {
        Some(backend) => Ok(backend),
        None => Err(ResolveBackendError {
            candidate: normalised,
            supported: RuntimeBackend::all()
                .iter()
                .map(|b| b.canonical_name().to_string())
                .collect(),
        }),
    }
}

/// Factory entry point. Construct the adapter for the configured
/// backend. Sidecar + AnthropicApi + the 7 CLI adapters return live
/// adapters; Ollama + Vllm return stubs whose `health_check` reports
/// `Unavailable` until B-37C wires them.
pub fn create_runtime(
    cfg: RuntimeFactoryConfig,
) -> Result<Box<dyn RuntimeAdapter>, RuntimeAdapterError> {
    match cfg.backend {
        RuntimeBackend::Sidecar => {
            let adapter = SidecarRuntime::from_factory_config(&cfg)?;
            Ok(Box::new(adapter))
        }
        RuntimeBackend::AnthropicApi => {
            let adapter = AnthropicApiRuntime::from_factory_config(&cfg)?;
            Ok(Box::new(adapter))
        }
        RuntimeBackend::ClaudeCode => Ok(Box::new(ClaudeCodeRuntime::new(cfg))),
        RuntimeBackend::CodexCli => Ok(Box::new(CodexCliRuntime::new(cfg))),
        RuntimeBackend::OpenCode => Ok(Box::new(OpenCodeRuntime::new(cfg))),
        RuntimeBackend::Hermes => Ok(Box::new(HermesRuntime::new(cfg))),
        RuntimeBackend::GeminiCli => Ok(Box::new(GeminiCliRuntime::new(cfg))),
        RuntimeBackend::KiroCli => Ok(Box::new(KiroCliRuntime::new(cfg))),
        RuntimeBackend::CopilotCli => Ok(Box::new(CopilotCliRuntime::new(cfg))),
        RuntimeBackend::Ollama => Ok(Box::new(OllamaRuntime::new(cfg))),
        RuntimeBackend::Vllm => Ok(Box::new(VllmRuntime::new(cfg))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_names_unique_and_lowercase() {
        let mut seen = std::collections::BTreeSet::new();
        for backend in RuntimeBackend::all() {
            let name = backend.canonical_name();
            assert!(seen.insert(name), "duplicate canonical name {name}");
            assert_eq!(name, name.to_ascii_lowercase(), "{name} must be lowercase");
        }
        assert_eq!(seen.len(), 11);
    }

    #[test]
    fn resolve_aliases_round_trip_to_canonical() {
        for backend in RuntimeBackend::all() {
            let resolved =
                resolve_backend(backend.canonical_name()).expect("canonical names must resolve");
            assert_eq!(resolved, *backend);
        }
    }

    #[test]
    fn resolve_backend_miss_returns_full_supported_list() {
        let err = resolve_backend("not-a-backend").expect_err("must miss");
        assert_eq!(err.supported.len(), 11);
        assert_eq!(err.candidate, "not-a-backend");
    }

    #[test]
    fn resolve_backend_is_case_insensitive() {
        assert_eq!(
            resolve_backend("SIDECAR").expect("uppercase"),
            RuntimeBackend::Sidecar
        );
        assert_eq!(
            resolve_backend("  Ollama  ").expect("whitespace"),
            RuntimeBackend::Ollama
        );
    }
}
