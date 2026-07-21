//! Multi-runtime adapter layer (B-37 Phase 4M).
//!
//! Provides a single `RuntimeAdapter` trait covering the backends
//! enumerated in `RuntimeBackend` (22 as of I-49). B-37A wired Sidecar +
//! AnthropicApi end-to-end; B-37B lands the 7 CLI subprocess adapters
//! (ClaudeCodeRuntime, CodexCliRuntime, OpenCodeRuntime, HermesRuntime,
//! GeminiCliRuntime, KiroCliRuntime, CopilotCliRuntime). Ollama + Vllm
//! remain stubs until B-37C.
//!
//! Subsequent sub-cycles fill in the remaining stubs:
//! - B-37C: Ollama + vLLM native HTTP adapters; PerRoleRuntime.
//! - B-37D: `/runtime` CLI command + dashboard RuntimePanel.
//! - I-49: AWS Bedrock + Azure OpenAI direct lanes (protocol logic +
//!   replay fixtures; live calls D-026 PENDING).
//!
//! See `workflow/reasoning/B-37-decisions.md` for the design contract.

/// I-49 — AWS Bedrock direct lane. SigV4 signing (hand-rolled HMAC-SHA256
/// via sha2 0.10, no new dep) + binary EventStream decoder + BedrockRuntime
/// adapter. Live billed calls deferred to D-026. See
/// `workflow/reasoning/I-49-decisions.md`.
pub mod bedrock;
/// I-49 — Azure OpenAI direct lane. Chat-completions-compatible adapter
/// (reuses openai_compat shim); live billed calls D-026 PENDING. See
/// `workflow/reasoning/I-49-decisions.md`.
pub mod azure_openai;
pub mod anthropic_api;
pub mod cli_common;
pub mod claude_code;
/// E-20 -- `BackendDifficultyAdapter`: pre-inference difficulty
/// classifier exposed as a [`RuntimeAdapter`]. Returns
/// `RuntimeAdapterError::NotImplemented` from `complete()` / `stream()`
/// until the live RouteLLM weights ship via E-20.B. Use
/// [`cloud_route_difficulty::classify_difficulty`] or
/// [`cloud_route_difficulty::BackendDifficultyAdapter::classify`] for
/// the actual difficulty verdict.
pub mod cloud_route_difficulty;
pub mod codex_cli;
/// E-31 — `RuntimeRoutingPolicy`: pre-route data + decision table.
/// Distinct from the orchestrator's E-32 `escalation_policy` (SessionDag
/// deep-mode handoff) and from C-9 `autonomous_gate::EscalationPolicy`
/// (gate-mode). Named to disambiguate the 3-way collision. See
/// `workflow/reasoning/E-31-decisions.md` DRIFT-2 reconcile.
pub mod escalation_policy;
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
/// J-30 T2 -- user-registered custom endpoints (custom_providers.toml).
/// Process-local OnceLock<BTreeMap<String, CustomBackendSpec>> registry
/// layered on `resolve_backend` / `create_runtime`. See
/// `workflow/reasoning/J-30_provider_byok_custom_endpoints.md`.
pub mod custom_backends;
/// J-30 T4 -- per-backend config descriptor metadata. Static
/// `BackendConfigDescriptor` table keyed by `RuntimeBackend` for builtins
/// plus `descriptor_for_custom` for the runtime-loaded `CustomBackendSpec`
/// overlay. Consumed by T5's CLI `configure` flow
/// (`providers_cmd::configure_with_factory`) and T6's status handler.
/// See `workflow/reasoning/J-30_provider_byok_custom_endpoints.md`.
pub mod config_descriptor;
/// J-30 T6 -- per-backend inventory status (configured / verified_at /
/// models / stale). Read-side companion to T5's `configure` flow.
/// `provider_status.json` sidecar in `default_config_home()`; no secrets.
/// CLI `elai providers status` is the production consumer, wired via
/// `providers_cmd::handle_providers_status` + `LiveCli::print_providers_status`.
/// See `workflow/reasoning/J-30_provider_byok_custom_endpoints.md`.
pub mod inventory;
pub mod event_sink;
pub mod gemini_cli;
/// E-34 — `GenAiAdapter` scaffold. RuntimeAdapter slot reserved for the
/// genai 0.6 multi-provider client (github.com/jeremychone/rust-genai).
/// All methods surface `RuntimeAdapterError::NotImplemented` until E-34.B
/// completes the anti-slopsquatting verification + Cargo.toml dep add +
/// per-provider wiring. See `workflow/reasoning/E-34-decisions.md` +
/// `.elai_cc/outdated-tech-audit.md` "Anti-slopsquatting verification log
/// (E-34)" for the deferral rationale. `RuntimeBackend` enum intentionally
/// NOT extended in E-34 (would break the canonical-name assertion at
/// mod.rs:216).
pub mod genai_adapter;
pub mod hermes;
pub mod isolation;
pub mod kiro_cli;
/// J-30 T1 -- per-provider BYOK key store (env -> 0600 file fallback).
/// Reuses `SecretString`; no OAuth / device_code arms. The production
/// consumer is `providers::anthropic::resolve_startup_auth_source`.
pub mod key_store;
pub mod ollama;
pub mod opencode;
pub mod secret;
/// E-26 (inference-engines 2026 pack) — SGLang HTTP runtime adapter.
/// Speaks OpenAI `/v1/chat/completions` with `extra_body.backend` selecting
/// XGrammar / Outlines / LMFE for structured output. Default base URL
/// `http://localhost:30000`. Long-lived health-probed cloud throughput
/// adapter; disambiguated from the media-validator `SglangConfig`.
pub mod sglang;
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

pub use bedrock::{
    BedrockRuntime, BedrockError, SigV4Headers,
    crc32, decode_eventstream_frame, derive_signing_key, format_amz_date,
    hex32, hmac_sha256, sha256_hex, sigv4_signing_headers, EventStreamMessage,
};
pub use azure_openai::{
    AzureOpenAiRuntime, build_azure_chat_request, normalize_azure_chat_response,
    normalize_finish_reason as azure_normalize_finish_reason,
};
pub use anthropic_api::AnthropicApiRuntime;
pub use claude_code::ClaudeCodeRuntime;
pub use cloud_route_difficulty::{
    classifier_unavailable_reason, classify_difficulty, BackendDifficultyAdapter,
    DifficultyBin as BackendDifficultyBin, DifficultyVerdict as BackendDifficultyVerdict,
};
pub use codex_cli::CodexCliRuntime;
pub use composer2::{Composer2Error, Composer2RuntimeAdapter};
pub use copilot_cli::CopilotCliRuntime;
pub use escalation_policy::{
    RoutingChoice, RoutingContext, RoutingCriticality, RoutingDecision, RoutingDifficulty,
    RoutingReason, RuntimeRoutingPolicy, RuntimeRoutingPolicyError,
};
pub use cost::{
    backend_name_hash, cost_model_for, role_name_hash, BackendCostModel, COMPOSER2_COST,
};
pub use event_sink::{
    install_runtime_event_sink, redact_argv, runtime_event_sink, NoopRuntimeEventSink,
    RuntimeEvent, RuntimeEventSink,
};
pub use gemini_cli::GeminiCliRuntime;
pub use genai_adapter::{GenAiAdapter, GenAiProvider, GENAI_NOT_IMPLEMENTED_HINT};
pub use hermes::HermesRuntime;
pub use kiro_cli::KiroCliRuntime;
pub use ollama::OllamaRuntime;
pub use opencode::OpenCodeRuntime;
pub use key_store::{lookup as lookup_key, store as store_key, KeyStoreError};
pub use secret::SecretString;
pub use sglang::{SglangRuntime, SglangRuntimeConfig, StructuredOutputBackend};
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
pub use config_descriptor::{
    BackendConfigDescriptor, ConfigKey, ConfigKeyKind, MissingKey,
    descriptor_for, descriptor_for_custom, satisfies,
};
pub use inventory::{
    all_inventory, inventory_status, inventory_status_custom,
    read_status, stamp_verified_at,
    InventoryError, InventoryStatus, ProviderStatusRow,
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
        // E-26 — SGLang HTTP runtime adapter. Canonical id + two operator
        // aliases. See `crates/providers/src/runtime/sglang.rs`.
        "sglang" | "sgl" | "sglang_runtime" => Some(RuntimeBackend::Sglang),
        // E-39 -- 3 cloud-floor / free-tier slots. RuntimeAdapter
        // wiring RUNPOD-PENDING per anti-slopsquatting verification of
        // api-docs.deepseek.com / cloud.cerebras.ai / console.groq.com
        // base URLs + model identifiers. See
        // workflow/reasoning/E-39-decisions.md.
        "deepseek_v4_flash" | "deepseek_flash" | "deepseek" => {
            Some(RuntimeBackend::DeepSeekV4Flash)
        }
        "cerebras_free" | "cerebras" => Some(RuntimeBackend::CerebrasFree),
        "groq_free" | "groq" => Some(RuntimeBackend::GroqFree),
        // I-49 — Bedrock + Azure direct lanes.
        "bedrock" | "aws_bedrock" | "aws-bedrock" => Some(RuntimeBackend::Bedrock),
        "azure_openai" | "azure" | "azure-openai" => Some(RuntimeBackend::AzureOpenAi),
        _ => None,
    };
    // J-30 T2 -- fallback to the custom-backend registry when the static
    // match misses. The registry is populated at startup from
    // custom_providers.toml; a hit yields RuntimeBackend::Custom (the name
    // is threaded via RuntimeFactoryConfig::runtime_profile). On first
    // unknown lookup we ensure the registry is populated from
    // custom_providers.toml so a present file is consulted even if the
    // startup boot hook has not run yet.
    let resolved = resolved.or_else(|| {
        if custom_backends::resolve_custom(&normalised).is_some() {
            Some(RuntimeBackend::Custom)
        } else {
            None
        }
    }).or_else(|| {
        match custom_backends::ensure_custom_backends_loaded() {
            Ok(()) => {
                if custom_backends::resolve_custom(&normalised).is_some() {
                    Some(RuntimeBackend::Custom)
                } else {
                    None
                }
            }
            Err(err) => {
                tracing::warn!(
                    target = "custom_backends",
                    "ensure_custom_backends_loaded failed during resolve_backend({normalised}): {err}"
                );
                None
            }
        }
    });
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
        // E-26 — SGLang HTTP runtime adapter (inference-engines 2026
        // pack). Constructs via `SglangRuntime::new`; base URL resolved
        // from cfg.base_url > ELAI_SGLANG_BASE_URL > DEFAULT_BASE_URL.
        RuntimeBackend::Sglang => Ok(Box::new(SglangRuntime::new(cfg))),
        // E-39 -- DeepSeek V4-Flash cloud-floor backend. Anti-
        // slopsquatting verification of api-docs.deepseek.com base URL
        // + model identifier + last-published date is RUNPOD-PENDING.
        // Until verified, surface NotImplemented so accidental routing
        // returns a typed sentinel rather than slopsquatting risk. The
        // cost-router still ships the variant in the
        // `(hard + tight)` cell; live wiring lands once the verification
        // row is recorded in E-39-decisions.md.
        RuntimeBackend::DeepSeekV4Flash => Err(RuntimeAdapterError::NotImplemented {
            backend: RuntimeBackend::DeepSeekV4Flash,
            hint: "DeepSeek V4-Flash RuntimeAdapter wiring RUNPOD-PENDING; verify api-docs.deepseek.com URL + model id + last-published date before live dispatch",
        }),
        // E-39 -- Cerebras free-tier backend. Same RUNPOD-PENDING
        // discipline as DeepSeek. The free-tier guard at
        // `crate::cost::free_tier_guard` already tracks daily quota.
        RuntimeBackend::CerebrasFree => Err(RuntimeAdapterError::NotImplemented {
            backend: RuntimeBackend::CerebrasFree,
            hint: "Cerebras free-tier RuntimeAdapter wiring RUNPOD-PENDING; verify cloud.cerebras.ai free-tier model list + base URL + last-published date before live dispatch",
        }),
        // E-39 -- Groq free-tier backend. Same RUNPOD-PENDING
        // discipline.
        RuntimeBackend::GroqFree => Err(RuntimeAdapterError::NotImplemented {
            backend: RuntimeBackend::GroqFree,
            hint: "Groq free-tier RuntimeAdapter wiring RUNPOD-PENDING; verify console.groq.com Llama 4 Scout / Qwen3 32B / Kimi K2 IDs + base URL + last-published date before live dispatch",
        }),
        // I-49 — AWS Bedrock direct lane. Adapter is BUILDABLE (SigV4 +
        // EventStream protocol logic ships Tier-1 with replay fixtures).
        // `from_factory_config` surfaces `MissingField` when AWS creds are
        // absent — never NotImplemented (the adapter EXISTS; cf Composer2).
        // Live billed Converse calls are Tier-2 / D-026 PENDING.
        RuntimeBackend::Bedrock => {
            let adapter = BedrockRuntime::from_factory_config(&cfg)?;
            Ok(Box::new(adapter))
        }
        // I-49 — Azure OpenAI direct lane. Adapter is BUILDABLE
        // (chat-completions protocol + openai_compat normalization shim
        // ships Tier-1 with replay fixtures). `from_factory_config` surfaces
        // `MissingField` when Azure creds / endpoint are absent. Live billed
        // Azure calls are Tier-2 / D-026 PENDING.
        RuntimeBackend::AzureOpenAi => {
            let adapter = AzureOpenAiRuntime::from_factory_config(&cfg)?;
            Ok(Box::new(adapter))
        }
        // J-30 T2 -- custom endpoint. The name is threaded via
        // `cfg.runtime_profile`; the registry maps it to a
        // `CustomBackendSpec` which `build_adapter` turns into a
        // anthropic-compat or openai-compat `RuntimeAdapter`. Belt-and-
        // suspenders: ensure the registry is populated before resolving,
        // in case the startup boot hook has not run yet.
        RuntimeBackend::Custom => {
            let name = cfg.runtime_profile.as_deref().ok_or_else(|| {
                RuntimeAdapterError::MissingField {
                    backend: RuntimeBackend::Custom,
                    field: "runtime_profile",
                    reason: "Custom backend requires runtime_profile = custom provider name"
                        .to_string(),
                }
            })?;
            if let Err(err) = custom_backends::ensure_custom_backends_loaded() {
                tracing::warn!(
                    target = "custom_backends",
                    "ensure_custom_backends_loaded failed during create_runtime(Custom): {err}"
                );
            }
            let spec = custom_backends::resolve_custom(name).ok_or_else(|| {
                RuntimeAdapterError::MissingField {
                    backend: RuntimeBackend::Custom,
                    field: "runtime_profile",
                    reason: format!(
                        "Custom backend `{name}` not registered (load custom_providers.toml at startup)"
                    ),
                }
            })?;
            custom_backends::build_adapter(&spec)
        }
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
        // I-49 added Bedrock + AzureOpenAi -> 22 total; J-30 T2 added Custom -> 23.
        assert_eq!(seen.len(), 23);
    }

    #[test]
    fn resolve_aliases_round_trip_to_canonical() {
        for backend in RuntimeBackend::all() {
            // Custom requires a registered spec in the custom_backends
            // registry; the round-trip is exercised in custom_backends tests.
            if *backend == RuntimeBackend::Custom {
                continue;
            }
            let resolved =
                resolve_backend(backend.canonical_name()).expect("canonical names must resolve");
            assert_eq!(resolved, *backend);
        }
    }

    #[test]
    fn resolve_backend_miss_returns_full_supported_list() {
        let err = resolve_backend("not-a-backend").expect_err("must miss");
        // I-49 added Bedrock + AzureOpenAi -> 22 total; J-30 T2 added Custom -> 23.
        assert_eq!(err.supported.len(), 23);
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
