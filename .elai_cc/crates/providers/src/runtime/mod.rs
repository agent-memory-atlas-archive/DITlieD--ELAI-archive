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
/// C-44 — Cursor Composer 2 cloud-API adapter. Paid HTTP backend
/// gated behind `[runtime.composer2].enabled = true` in elai_cc.toml.
/// See workflow/papers/composer2-interface.md for the source-grounding
/// posture (paper arXiv:2603.24477 covers model identity only;
/// integration design source-grounded against @cursor/sdk).
pub mod composer2;
pub mod copilot_cli;
/// C-44 — per-backend cost tracking + hashing helpers. Used by the
/// orchestrator's SSB writer to emit `BackendCostAccrued` rows.
pub mod cost;
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
pub mod llama_cpp;
pub mod mlx;
pub mod platform_detect;

pub use anthropic_api::AnthropicApiRuntime;
pub use claude_code::ClaudeCodeRuntime;
pub use codex_cli::CodexCliRuntime;
pub use composer2::{Composer2Error, Composer2RuntimeAdapter};
pub use copilot_cli::CopilotCliRuntime;
pub use cost::{
    backend_name_hash, cost_model_for, role_name_hash, BackendCostModel, COMPOSER2_COST,
};
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
pub use llama_cpp::LlamaCppAdapter;
pub use mlx::MlxAdapter;
pub use platform_detect::{detect_llamacpp_ep, detect_mlx_capable, AppleChipFamily, LlamaCppEp, MlxCapability};
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
        "mlx" | "mlx_adapter" => Some(RuntimeBackend::Mlx),
        "llama_cpp" | "llamacpp" | "llama-cpp" => Some(RuntimeBackend::LlamaCpp),
        // D-2 — PPM is a discovery-only slot. resolve_backend yields the
        // canonical variant; `create_runtime` returns NotImplemented (PPM
        // has no completion surface; scoring is dispatched via
        // `elai_orchestrator::http_sidecar_ppm_hook::HttpSidecarPpmHook`
        // directly).
        "ppm" | "ppm_sidecar" | "qwen_math_prm" => Some(RuntimeBackend::Ppm),
        // C-44 — Cursor Composer 2 cloud API. Accepts the canonical
        // snake_case id plus two common operator aliases.
        "composer2" | "cursor" | "cursor_composer" => Some(RuntimeBackend::Composer2),
        // E-40 — LSP subprocess oracle slot. `create_runtime` returns
        // NotImplemented (no /v1/messages surface); orchestrator dispatches
        // via symbol_graph::lsp_oracle::oracle_lookup directly.
        "lsp" | "lsp_oracle" | "language_server" => Some(RuntimeBackend::Lsp),
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
        RuntimeBackend::Mlx => {
            let adapter = MlxAdapter::new(cfg).map_err(|e| e)?;
            Ok(Box::new(adapter))
        }
        RuntimeBackend::LlamaCpp => Ok(Box::new(LlamaCppAdapter::new(cfg))),
        // D-2 — verifier-only sidecar; no RuntimeAdapter completion path
        // by design (PpmClient handles scoring directly). Surface
        // NotImplemented so `/runtime list` shows the slot but accidental
        // routing returns a typed sentinel rather than panicking. See
        // workflow/reasoning/D-2-decisions.md.
        RuntimeBackend::Ppm => Err(RuntimeAdapterError::NotImplemented {
            backend: RuntimeBackend::Ppm,
            hint: "PPM is verifier-only; route scoring via http_sidecar_ppm_hook",
        }),
        // C-44 — Composer 2 paid HTTP backend. `from_factory_config`
        // surfaces `MissingField` if CURSOR_API_KEY is absent — never
        // panics, never silently falls back (plan stub_ban).
        RuntimeBackend::Composer2 => {
            let adapter = Composer2RuntimeAdapter::from_factory_config(&cfg)?;
            Ok(Box::new(adapter))
        }
        // E-40 — LSP subprocess oracle. No /v1/messages surface; the
        // actual dispatch is in elai_orchestrator::symbol_graph::lsp_oracle
        // and uses tokio::process::Command directly. Surface
        // NotImplemented so `/runtime list` shows the slot and accidental
        // generation-path routing returns a typed sentinel rather than
        // panicking. See workflow/reasoning/E-40-decisions.md.
        RuntimeBackend::Lsp => Err(RuntimeAdapterError::NotImplemented {
            backend: RuntimeBackend::Lsp,
            hint: "Lsp is on-demand oracle only; dispatch via elai_orchestrator::symbol_graph::lsp_oracle::oracle_lookup",
        }),
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
        assert_eq!(seen.len(), 16);
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
        assert_eq!(err.supported.len(), 16);
        assert_eq!(err.candidate, "not-a-backend");
    }

    /// C-44 — Composer 2 resolves via canonical id + two operator
    /// aliases. `create_runtime` returns the live adapter when an
    /// api_key is supplied through the factory.
    #[test]
    fn resolve_composer2_aliases_match_canonical() {
        assert_eq!(
            resolve_backend("composer2").expect("canonical"),
            RuntimeBackend::Composer2
        );
        assert_eq!(
            resolve_backend("cursor").expect("alias cursor"),
            RuntimeBackend::Composer2
        );
        assert_eq!(
            resolve_backend("cursor_composer").expect("alias cursor_composer"),
            RuntimeBackend::Composer2
        );
    }

    /// C-44 — `create_runtime(Composer2)` without api_key surfaces
    /// `MissingField` rather than panicking. Mirrors the AnthropicApi
    /// missing-key behaviour.
    #[test]
    fn create_runtime_composer2_without_api_key_returns_missing_field() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Composer2);
        match create_runtime(cfg) {
            Err(RuntimeAdapterError::MissingField {
                backend, field, ..
            }) => {
                assert_eq!(backend, RuntimeBackend::Composer2);
                assert_eq!(field, "api_key");
            }
            Err(e) => panic!("expected MissingField for Composer2, got {e:?}"),
            Ok(_) => panic!("expected Err(MissingField) for Composer2, got Ok(adapter)"),
        }
    }

    /// D-2 — PPM resolves but `create_runtime` is NotImplemented by design.
    #[test]
    fn resolve_ppm_alias_returns_canonical_variant() {
        assert_eq!(resolve_backend("ppm").expect("ppm"), RuntimeBackend::Ppm);
        assert_eq!(resolve_backend("PPM").expect("upper"), RuntimeBackend::Ppm);
        assert_eq!(
            resolve_backend("ppm_sidecar").expect("alias"),
            RuntimeBackend::Ppm
        );
        assert_eq!(
            resolve_backend("qwen_math_prm").expect("alias"),
            RuntimeBackend::Ppm
        );
    }

    /// D-2 — `create_runtime(Ppm)` is a typed sentinel, not a panic.
    #[test]
    fn create_runtime_ppm_returns_not_implemented_sentinel() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Ppm);
        // `Result<Box<dyn RuntimeAdapter>, _>` does not impl Debug (the Ok arm
        // holds a trait object). Match each arm explicitly so the test still
        // panics with a useful message on regression but typechecks cleanly.
        match create_runtime(cfg) {
            Err(RuntimeAdapterError::NotImplemented { backend, .. }) => {
                assert_eq!(backend, RuntimeBackend::Ppm);
            }
            Err(e) => panic!("expected NotImplemented for Ppm, got {e:?}"),
            Ok(_) => panic!("expected Err(NotImplemented) for Ppm, got Ok(adapter)"),
        }
    }

    /// E-40 — Lsp resolves via canonical id + two operator aliases.
    /// `create_runtime` returns `NotImplemented` because there is no
    /// `/v1/messages` surface; the orchestrator dispatches the oracle
    /// via `symbol_graph::lsp_oracle::oracle_lookup` directly.
    #[test]
    fn resolve_lsp_aliases_match_canonical() {
        assert_eq!(
            resolve_backend("lsp").expect("canonical"),
            RuntimeBackend::Lsp
        );
        assert_eq!(
            resolve_backend("lsp_oracle").expect("alias lsp_oracle"),
            RuntimeBackend::Lsp
        );
        assert_eq!(
            resolve_backend("language_server").expect("alias language_server"),
            RuntimeBackend::Lsp
        );
    }

    /// E-40 — `create_runtime(Lsp)` is a typed sentinel, not a panic.
    #[test]
    fn create_runtime_lsp_returns_not_implemented_sentinel() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Lsp);
        match create_runtime(cfg) {
            Err(RuntimeAdapterError::NotImplemented { backend, .. }) => {
                assert_eq!(backend, RuntimeBackend::Lsp);
            }
            Err(e) => panic!("expected NotImplemented for Lsp, got {e:?}"),
            Ok(_) => panic!("expected Err(NotImplemented) for Lsp, got Ok(adapter)"),
        }
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
