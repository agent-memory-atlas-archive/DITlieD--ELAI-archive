//! `GenAiAdapter` — multi-provider RuntimeAdapter scaffold (E-34).
//!
//! Scope of this scaffold (E-34.A, no external crate added yet):
//! - `GenAiAdapter` struct + `RuntimeAdapter` impl with the canonical
//!   `Pin<Box<dyn Future>>` shape (mirrors `AnthropicApiRuntime` at
//!   `runtime/anthropic_api.rs`). Stays `dyn`-compatible so a future
//!   variant can drop into `create_runtime` without re-architecting the
//!   factory.
//! - All RuntimeAdapter methods surface `RuntimeAdapterError::NotImplemented`
//!   with `hint = "E-34.B: full migration"` rather than `unimplemented!()`.
//!   Per `crates/providers/src/runtime/mod.rs:176-179` + `:193-196` (Ppm /
//!   Lsp prior art), `NotImplemented` is the typed sentinel for adapter
//!   slots whose completion surface is not wired yet. `unimplemented!()` /
//!   `todo!()` would PANIC on call in a `Send + Sync` trait context
//!   (banned by CLAUDE.md HARD RULE Zero Lazy Code + the .claude/rules/
//!   stub-tracking exception requires the stub be wired through the
//!   registered marker, not a panicking macro).
//! - NO new external dependency. The `genai` crate
//!   (github.com/jeremychone/rust-genai, 0.6.0-beta.X) is the intended
//!   backing library, but `cargo search genai` + license + publish-date
//!   verification (CLAUDE.md HARD RULE Anti-Slopsquatting) was NOT run
//!   in E-34.A — adoption is deferred to E-34.B. See
//!   `.elai_cc/outdated-tech-audit.md` "Anti-slopsquatting verification
//!   log (E-34)" + `workflow/reasoning/E-34-decisions.md` Open Question 1.
//! - `RuntimeBackend` enum is INTENTIONALLY NOT extended in E-34. Adding
//!   a `GenAi` variant would break the canonical-name assertion at
//!   `runtime/mod.rs:216` (`assert_eq!(seen.len(), 17);`) and require a
//!   matching `per_role_runtime` default-map update — both out of scope.
//!   E-34.B promotes the adapter into `RuntimeBackend` once the genai
//!   crate is verified + wired.
//!
//! Why land the scaffold at all (vs deferring everything to E-34.B):
//! - Locks the adapter shape so E-34.B has a concrete landing target
//!   rather than a green-field design problem.
//! - Surfaces `GenAiAdapter` in `pub use` so downstream crates can
//!   reference the type by import path; the NotImplemented errors
//!   prevent any caller from accidentally driving the un-wired path.
//! - Carries the stub-registry row (`workflow/sessions/s-25120/stubs.md`)
//!   that the Stop hook + AUDIT FSM substate audits, per
//!   `.claude/rules/stub-tracking.md`.
//!
//! Source-grounding: `crates/providers/src/runtime/anthropic_api.rs`
//! (factory-config + `from_client` mirror), `crates/providers/src/runtime/
//! mod.rs` lines 176-179 (Ppm NotImplemented sentinel pattern), 193-196
//! (Lsp NotImplemented sentinel pattern). See
//! `workflow/reasoning/E-34-decisions.md` for the rule-amendment +
//! adoption-deferral rationale.

use futures_util::stream::{self, BoxStream};

use super::types::{
    CompletionConfig, CompletionResponse, Message, RuntimeAdapter, RuntimeAdapterError,
    RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture, RuntimeHealth,
    RuntimeHealthFuture, RuntimeStream, StreamChunk,
};

/// Hint string surfaced by every `NotImplemented` error this scaffold
/// emits. Named constant per CLAUDE.md "Named constants (no magic
/// numbers)".
pub const GENAI_NOT_IMPLEMENTED_HINT: &str =
    "E-34.B: genai 0.6 dep + full RuntimeAdapter wiring deferred (anti-slopsquatting pending)";

/// Provider identifier consumed by `GenAiAdapter`. Today the only
/// supported id is `Anthropic` (the genai migration target); future
/// E-34.B variants will widen this to cover Gemini / Groq / xAI /
/// Ollama / DeepSeek / Copilot per the genai crate's provider list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenAiProvider {
    /// Anthropic via genai's `AdapterKind::Anthropic`. Routes to
    /// `AnthropicClient`-equivalent semantics on the genai side.
    Anthropic,
}

impl GenAiProvider {
    /// Canonical snake_case id used for stub-registry rows + log lines.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
        }
    }
}

/// Default provider when `from_factory_config` is called without an
/// explicit override. Today the only supported provider so the constant
/// is fixed; E-34.B will replace this with a runtime-config switch.
const DEFAULT_PROVIDER: GenAiProvider = GenAiProvider::Anthropic;

/// `GenAiAdapter` — RuntimeAdapter scaffold pending genai 0.6 adoption
/// in E-34.B. Coexists with `AnthropicApiRuntime` (the production
/// Anthropic adapter); does NOT replace it.
#[derive(Debug, Clone)]
pub struct GenAiAdapter {
    provider: GenAiProvider,
    default_model: Option<String>,
}

impl GenAiAdapter {
    /// Construct from a factory config. The factory currently inspects
    /// only `cfg.model`; once genai 0.6 is wired in E-34.B, `cfg.api_key`
    /// + `cfg.auth_source` + `cfg.base_url` will all be honoured.
    ///
    /// Returns `MissingField` if no provider can be resolved — today
    /// `DEFAULT_PROVIDER` is hard-coded so this path is unreachable; the
    /// branch exists so E-34.B does not need to invent a new error
    /// variant when it widens provider selection.
    pub fn from_factory_config(cfg: &RuntimeFactoryConfig) -> Result<Self, RuntimeAdapterError> {
        Ok(Self {
            provider: DEFAULT_PROVIDER,
            default_model: cfg.model.clone(),
        })
    }

    /// Direct constructor for tests + callers that already hold the
    /// provider id. Mirrors `AnthropicApiRuntime::from_client`.
    #[must_use]
    pub fn from_provider(provider: GenAiProvider, default_model: Option<String>) -> Self {
        Self {
            provider,
            default_model,
        }
    }

    /// Active provider id. Cheap accessor — useful for log lines + the
    /// stub-registry row in `workflow/sessions/s-25120/stubs.md`.
    #[must_use]
    pub const fn provider(&self) -> GenAiProvider {
        self.provider
    }

    /// Echoed default model (if any). Cheap accessor.
    #[must_use]
    pub fn default_model(&self) -> Option<&str> {
        self.default_model.as_deref()
    }
}

impl RuntimeAdapter for GenAiAdapter {
    fn backend(&self) -> RuntimeBackend {
        // `GenAi` is NOT in the RuntimeBackend enum yet (E-34 deferred
        // the variant addition to keep the canonical-name assertion at
        // runtime/mod.rs:216 stable). Until E-34.B promotes the variant,
        // identify as AnthropicApi — the provider this scaffold's
        // default routing targets — so `RuntimeOrchestrator` log lines +
        // SSB writers don't see a phantom backend id.
        RuntimeBackend::AnthropicApi
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        // Conservative defaults. E-34.B fills these in from genai's
        // per-provider capability table.
        RuntimeCapabilities::none()
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let provider = self.provider;
        Box::pin(async move {
            RuntimeHealth::Unavailable {
                reason: format!(
                    "GenAiAdapter scaffold (provider={}); full wiring deferred to E-34.B",
                    provider.canonical_name()
                ),
            }
        })
    }

    fn complete(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        Box::pin(async move {
            Err(RuntimeAdapterError::NotImplemented {
                backend: RuntimeBackend::AnthropicApi,
                hint: GENAI_NOT_IMPLEMENTED_HINT,
            })
        })
    }

    fn stream(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        // Stream surfaces a single typed `Err` so callers using
        // `stream.collect::<Result<Vec<_>, _>>()` get a clean
        // short-circuit. Matches the AnthropicApiRuntime "synthetic
        // first chunk on connect" shape: the synthetic event is a typed
        // error instead of a Warning so consumers must handle it.
        let err = RuntimeAdapterError::NotImplemented {
            backend: RuntimeBackend::AnthropicApi,
            hint: GENAI_NOT_IMPLEMENTED_HINT,
        };
        Box::pin(stream::once(async move { Err(err) }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream::StreamExt;

    #[test]
    fn provider_canonical_name_is_lowercase_snake_case() {
        for provider in [GenAiProvider::Anthropic] {
            let name = provider.canonical_name();
            assert_eq!(name, name.to_ascii_lowercase());
            assert!(
                !name.contains(' '),
                "canonical name `{name}` must be snake_case"
            );
        }
    }

    #[test]
    fn from_factory_config_uses_default_provider() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::AnthropicApi);
        let adapter = GenAiAdapter::from_factory_config(&cfg).expect("config must parse");
        assert_eq!(adapter.provider(), DEFAULT_PROVIDER);
        assert_eq!(adapter.default_model(), None);
    }

    #[test]
    fn from_provider_round_trips_provider_id() {
        let adapter = GenAiAdapter::from_provider(
            GenAiProvider::Anthropic,
            Some("claude-sonnet-4-6".to_string()),
        );
        assert_eq!(adapter.provider(), GenAiProvider::Anthropic);
        assert_eq!(adapter.default_model(), Some("claude-sonnet-4-6"));
    }

    #[test]
    fn capabilities_are_conservative_defaults() {
        let adapter = GenAiAdapter::from_provider(GenAiProvider::Anthropic, None);
        let caps = adapter.capabilities();
        assert!(!caps.supports_tools);
        assert!(!caps.supports_streaming);
        assert!(!caps.supports_vision);
        assert_eq!(caps.max_context, 0);
        assert_eq!(caps.max_output, 0);
    }

    #[test]
    fn backend_identifies_as_anthropic_api_pre_e34b() {
        let adapter = GenAiAdapter::from_provider(GenAiProvider::Anthropic, None);
        // E-34: until the RuntimeBackend variant lands in E-34.B, the
        // scaffold reports AnthropicApi so log lines + SSB writers don't
        // see a phantom backend id.
        assert_eq!(adapter.backend(), RuntimeBackend::AnthropicApi);
    }

    #[tokio::test]
    async fn complete_returns_not_implemented_sentinel() {
        let adapter = GenAiAdapter::from_provider(GenAiProvider::Anthropic, None);
        let err = adapter
            .complete(vec![], CompletionConfig::default())
            .await
            .expect_err("scaffold must NotImplemented until E-34.B");
        match err {
            RuntimeAdapterError::NotImplemented { backend, hint } => {
                assert_eq!(backend, RuntimeBackend::AnthropicApi);
                assert!(
                    hint.contains("E-34.B"),
                    "hint must point to E-34.B follow-up: got `{hint}`"
                );
            }
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn stream_yields_single_not_implemented_error() {
        let adapter = GenAiAdapter::from_provider(GenAiProvider::Anthropic, None);
        let mut s = adapter.stream(vec![], CompletionConfig::default());
        let first = s
            .next()
            .await
            .expect("stream must yield exactly one error item");
        assert!(matches!(first, Err(RuntimeAdapterError::NotImplemented { .. })));
        assert!(s.next().await.is_none(), "stream must terminate after the error");
    }

    #[tokio::test]
    async fn health_check_reports_unavailable_with_e34b_pointer() {
        let adapter = GenAiAdapter::from_provider(GenAiProvider::Anthropic, None);
        let health = adapter.health_check().await;
        match health {
            RuntimeHealth::Unavailable { reason } => {
                assert!(
                    reason.contains("E-34.B"),
                    "health unavailable reason must point to E-34.B follow-up: got `{reason}`"
                );
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }
}
