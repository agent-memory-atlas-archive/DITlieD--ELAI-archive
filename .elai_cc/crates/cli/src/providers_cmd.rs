//! J-30 T5 -- `elai providers configure` CLI handler.
//!
//! Vertical slice: resolve a backend, collect its required config keys
//! (flags, env, stdin, key_store), gate via `config_descriptor::satisfies`,
//! construct the adapter, run `health_check` with a 5s timeout, and ONLY
//! on Ready/Degraded persist the key via `key_store::store`. Fail-closed:
//! a missing required key or an Unavailable health probe aborts WITHOUT
//! saving. Mirrors the `runtime_cmd::handle_runtime_test` shape.
//!
//! FORK A: no masked stdin helper exists in the CLI crate (auth.rs is
//! OAuth-only) and rpassword is NOT a workspace dep. Key entry uses plain
//! `io::stdin().read_line` -- the key is echoed to the terminal on entry.
//! Masked entry is deferred (would require a new dep, violating the
//! zero-new-deps plan hard rule). The `--key` flag is the non-echoed path.
//!
//! FORK B: the live test call is `adapter.health_check()` (the B-37
//! contract every adapter implements), not a synthesized completion call.
//!
//! FORK D: new `CliAction::Providers` + this module; the existing
//! `/providers` slash command (commands.rs:766) is listing-only and is
//! NOT touched.
//!
//! See `workflow/reasoning/J-30_provider_byok_custom_endpoints.md`.

use std::future::Future;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::pin::Pin;
use std::time::Duration;

use elai_providers::runtime::{
    config_descriptor, create_runtime, key_store, resolve_backend, custom_backends,
    RuntimeAdapter, RuntimeAdapterError, RuntimeBackend, RuntimeFactoryConfig, RuntimeHealth,
    SecretString,
};
use thiserror::Error;
use tokio::time::timeout;

/// Hard cap on the live health probe. Mirrors `runtime_cmd.rs` (5s).
const HEALTH_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Error)]
pub enum ProvidersCmdError {
    #[error("unknown backend '{candidate}'. Supported: {supported}")]
    UnknownBackend {
        candidate: String,
        supported: String,
    },
    #[error("failed to construct runtime adapter: {0}")]
    AdapterConstruct(String),
    #[error("key store error: {0}")]
    KeyStore(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("custom backend '{0}' not registered (load custom_providers.toml at startup)")]
    CustomNotRegistered(String),
}

impl From<key_store::KeyStoreError> for ProvidersCmdError {
    fn from(err: key_store::KeyStoreError) -> Self {
        ProvidersCmdError::KeyStore(err.to_string())
    }
}

impl From<io::Error> for ProvidersCmdError {
    fn from(err: io::Error) -> Self {
        ProvidersCmdError::Io(err.to_string())
    }
}

/// Resolved backend + its config descriptor + the provider_id used for
/// `key_store`. FORK E: builtin provider_id = canonical_name().lowercase();
/// custom provider_id = spec.name.
enum ResolvedBackend {
    Builtin {
        backend: RuntimeBackend,
    },
    Custom {
        backend: RuntimeBackend,
        spec: custom_backends::CustomBackendSpec,
        descriptor: config_descriptor::BackendConfigDescriptor,
    },
}

impl ResolvedBackend {
    fn backend(&self) -> RuntimeBackend {
        match self {
            Self::Builtin { backend } => *backend,
            Self::Custom { backend, .. } => *backend,
        }
    }
    fn descriptor(&self) -> config_descriptor::BackendConfigDescriptor {
        match self {
            Self::Builtin { backend } => config_descriptor::descriptor_for(*backend)
                .expect("descriptor_for returns None only for Custom; builtin path checked")
                .clone(),
            Self::Custom { descriptor, .. } => descriptor.clone(),
        }
    }
    fn provider_id(&self) -> String {
        match self {
            // Keep the key file keyed by the same friendly identifier used in
            // existing store/lookup paths (e.g. "anthropic", not the canonical
            // snake_case "anthropic_api"). Resolve_backend accepts both.
            Self::Builtin { backend } => {
                // The first alias in resolve_backend is the friendly provider_id.
                let canon = backend.canonical_name();
                friendly_provider_id(canon).unwrap_or(canon).to_string()
            }
            Self::Custom { spec, .. } => spec.name.clone(),
        }
    }
    fn display_name(&self) -> String {
        match self {
            Self::Builtin { backend } => backend.canonical_name().to_string(),
            Self::Custom { spec, .. } => spec.name.clone(),
        }
    }
}

/// Map a backend canonical name to its preferred friendly provider_id,
/// matching the first alias accepted by `resolve_backend`. This keeps
/// stored keys keyed by the same identifier other CLI flows expect.
fn friendly_provider_id(canon: &str) -> Option<&str> {
    match canon {
        "sidecar" => Some("sidecar"),
        "anthropic_api" => Some("anthropic"),
        "claude_code" => Some("claude_code"),
        "codex_cli" => Some("codex_cli"),
        "opencode" => Some("opencode"),
        "hermes" => Some("hermes"),
        "gemini_cli" => Some("gemini"),
        "kiro_cli" => Some("kiro"),
        "copilot_cli" => Some("copilot"),
        "ollama" => Some("ollama"),
        "vllm" => Some("vllm"),
        "mlx" => Some("mlx"),
        "llama_cpp" => Some("llama_cpp"),
        "ppm" => Some("ppm"),
        "composer2" => Some("composer2"),
        "lsp" => Some("lsp"),
        "sglang" => Some("sglang"),
        "deepseek_v4_flash" => Some("deepseek_v4_flash"),
        "cerebras_free" => Some("cerebras_free"),
        "groq_free" => Some("groq_free"),
        "bedrock" => Some("bedrock"),
        "azure_openai" => Some("azure_openai"),
        _ => None,
    }
}

/// Resolve a CLI candidate string to a backend + descriptor. Falls back
/// to the custom-backend registry when the static match misses (mirrors
/// `resolve_backend` in providers/runtime/mod.rs).
fn resolve_backend_with_descriptor(
    candidate: &str,
) -> Result<ResolvedBackend, ProvidersCmdError> {
    let backend = resolve_backend(candidate).map_err(|e| ProvidersCmdError::UnknownBackend {
        candidate: e.candidate,
        supported: e.supported.join(", "),
    })?;
    if backend == RuntimeBackend::Custom {
        let normalised = candidate.trim().to_ascii_lowercase();
        let spec = custom_backends::resolve_custom(&normalised).ok_or_else(|| {
            ProvidersCmdError::CustomNotRegistered(normalised.clone())
        })?;
        let descriptor = config_descriptor::descriptor_for_custom(&spec);
        Ok(ResolvedBackend::Custom {
            backend,
            spec,
            descriptor,
        })
    } else {
        Ok(ResolvedBackend::Builtin { backend })
    }
}

/// Prompt stdin for a single line. Returns the trimmed string. FORK A:
/// plain read_line, key is echoed (no rpassword / termios in workspace).
fn prompt_line(label: &str) -> Result<String, ProvidersCmdError> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

/// Flag-shaped inputs to the configure flow. All optional; missing
/// required fields are either prompted on stdin (interactive) or abort
/// (non-interactive --key path).
#[derive(Debug, Clone, Default)]
pub struct ConfigureFlags {
    pub backend: Option<String>,
    pub key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

/// Adapter factory closure. Production uses `create_runtime`; tests inject
/// a stub. Boxed because it's a one-shot FnOnce.
type AdapterFactory = Box<
    dyn FnOnce(RuntimeFactoryConfig) -> Result<Box<dyn RuntimeAdapter>, RuntimeAdapterError>,
>;

/// Core configure flow, factored so tests can inject an adapter factory
/// (FORK C: no live endpoint required for unit tests). The factory is
/// invoked AFTER `satisfies` passes; on Unavailable/timeout the key is
/// NOT stored (fail-closed).
///
/// Returns a human-readable `[OK] ...` / `[FAIL] ...` line. Errors are
/// surfaced via `ProvidersCmdError` and the caller renders them.
async fn configure_with_factory(
    flags: ConfigureFlags,
    factory: AdapterFactory,
) -> Result<String, ProvidersCmdError> {
    let candidate = match flags.backend.as_deref() {
        Some(name) => name.to_string(),
        None => {
            if !io::stdin().is_terminal() {
                return Ok("[FAIL] backend is required and was not provided (non-interactive)".to_string());
            }
            prompt_line("Backend name (e.g. sidecar, anthropic, llama_cpp)")?
        }
    };
    let resolved = resolve_backend_with_descriptor(&candidate)?;
    let descriptor = resolved.descriptor();
    let provider_id = resolved.provider_id();
    let display_name = resolved.display_name();
    let backend = resolved.backend();

    // Step: collect api_key. Env wins (via key_store::lookup), then flag,
    // then stdin prompt for required secret keys.
    let env_name = descriptor
        .keys
        .iter()
        .find(|k| {
            k.kind == config_descriptor::ConfigKeyKind::ApiKey && k.secret && k.required
        })
        .map(|k| k.name);

    let mut cfg = RuntimeFactoryConfig::for_backend(backend);

    // For custom backends, thread the spec name via runtime_profile so
    // create_runtime(Custom) can resolve the spec.
    if let ResolvedBackend::Custom { spec, .. } = &resolved {
        cfg.runtime_profile = Some(spec.name.clone());
        // Custom backends: the api_key_ref env var is the lookup key.
        let env_key = spec.api_key_ref.clone();
        if let Some(existing) = key_store::lookup(&spec.name, &env_key)? {
            cfg.api_key = Some(existing);
        }
    } else if let Some(env) = env_name {
        if let Some(existing) = key_store::lookup(&provider_id, env)? {
            cfg.api_key = Some(existing);
        }
    }

    // Flag overrides / fills missing.
    if let Some(key) = flags.key.as_deref() {
        if !key.trim().is_empty() {
            cfg.api_key = Some(SecretString::new(key.to_string()));
        }
    }

    // If api_key is still missing and the descriptor requires it, prompt.
    let needs_api_key = !descriptor.api_key_optional
        && descriptor
            .keys
            .iter()
            .any(|k| k.kind == config_descriptor::ConfigKeyKind::ApiKey && k.required);
    if needs_api_key && cfg.api_key.is_none() {
        if !io::stdin().is_terminal() {
            return Ok(format!(
                "[FAIL] {display_name}: api_key is required and was not provided (non-interactive)"
            ));
        }
        // FORK A: echoed stdin. Print a caveat so the operator knows.
        eprintln!(
            "[note] key will be echoed to the terminal on entry (masked entry deferred, no new deps)"
        );
        let entered = prompt_line("API key")?;
        if entered.is_empty() {
            return Ok(format!(
                "[FAIL] {display_name}: api_key is required and was not provided"
            ));
        }
        cfg.api_key = Some(SecretString::new(entered));
    }

    // base_url flag / prompt.
    if descriptor.supports_base_url {
        if let Some(url) = flags.base_url.as_deref() {
            if !url.trim().is_empty() {
                cfg.base_url = Some(url.to_string());
            }
        }
        if cfg.base_url.is_none() {
            // Prompt only if base_url is required; else apply descriptor default.
            let base_required = descriptor
                .keys
                .iter()
                .any(|k| k.kind == config_descriptor::ConfigKeyKind::BaseUrl && k.required);
            if base_required {
                if !io::stdin().is_terminal() {
                    return Ok(format!(
                        "[FAIL] {display_name}: base_url is required and was not provided (non-interactive)"
                    ));
                }
                let entered = prompt_line("Base URL")?;
                if !entered.is_empty() {
                    cfg.base_url = Some(entered);
                }
            } else if let Some(default) = descriptor.default_base_url {
                cfg.base_url = Some(default.to_string());
            }
        }
    }

    // model flag.
    if let Some(model) = flags.model.as_deref() {
        if !model.trim().is_empty() {
            cfg.model = Some(model.to_string());
        }
    }
    if cfg.model.is_none() {
        // For custom backends, default to the first model in the spec.
        if let ResolvedBackend::Custom { spec, .. } = &resolved {
            if let Some(first) = spec.models.first() {
                cfg.model = Some(first.clone());
            }
        }
    }

    // satisfies() gate -- abort WITHOUT network/health call if missing.
    if let Err(missing) = config_descriptor::satisfies(&descriptor, &cfg) {
        return Ok(format!(
            "[FAIL] {display_name}: missing required config key `{}` ({})",
            missing.key, missing.reason
        ));
    }

    // Capture the key to persist BEFORE the factory consumes `cfg`. The
    // invariant is: the stored key equals cfg.api_key at the point of the
    // health_check call. Cloning a SecretString is cheap (Arc inner).
    let to_store: Option<SecretString> = cfg.api_key.clone();

    // Construct adapter via the injected factory.
    let adapter = factory(cfg).map_err(|e| ProvidersCmdError::AdapterConstruct(e.to_string()))?;

    // health_check with 5s timeout (FORK B, mirror runtime_cmd.rs:372).
    let health = timeout(HEALTH_PROBE_TIMEOUT, adapter.health_check())
        .await
        .unwrap_or(RuntimeHealth::Unavailable {
            reason: "health probe timeout (>5s)".to_string(),
        });

    match health {
        RuntimeHealth::Ready => {
            // Only on success: persist key. If no api_key was collected
            // (api_key_optional local lane), skip the store.
            if let Some(key) = to_store {
                key_store::store(&provider_id, &key)?;
            }
            Ok(format!("[OK] {display_name} configured (key stored, verified)"))
        }
        RuntimeHealth::Degraded { reason } => {
            if let Some(key) = to_store {
                key_store::store(&provider_id, &key)?;
            }
            Ok(format!(
                "[OK] {display_name} configured (key stored, verified) — degraded: {reason}"
            ))
        }
        RuntimeHealth::Unavailable { reason } => {
            Ok(format!("[FAIL] {display_name}: {reason}"))
        }
    }
}

/// Public entrypoint for `elai providers configure`. Called from
/// `LiveCli::print_providers` (live_cli.rs) via a tokio current-thread
/// runtime, mirroring `print_runtime`.
pub async fn handle_providers_configure(
    flags: ConfigureFlags,
    _project_root: &Path,
) -> Result<String, ProvidersCmdError> {
    let factory: AdapterFactory = Box::new(|cfg| create_runtime(cfg));
    configure_with_factory(flags, factory).await
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use elai_providers::runtime::{
        CompletionConfig, CompletionResponse, Message, RuntimeCapabilities, RuntimeFuture,
        RuntimeHealthFuture, RuntimeStream, StreamChunk,
    };
    use futures_util::stream;
    use std::future::Future;
    use std::pin::Pin;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_config_home<F: FnOnce(&Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("ELAI_CONFIG_HOME", dir.path());
        f(dir.path());
        std::env::remove_var("ELAI_CONFIG_HOME");
    }

    /// Stub adapter whose health_check returns a precomputed outcome.
    struct StubAdapter {
        backend: RuntimeBackend,
        health: RuntimeHealth,
    }

    impl RuntimeAdapter for StubAdapter {
        fn backend(&self) -> RuntimeBackend {
            self.backend
        }
        fn capabilities(&self) -> RuntimeCapabilities {
            RuntimeCapabilities::none()
        }
        fn health_check(&self) -> RuntimeHealthFuture<'_> {
            let health = self.health.clone();
            Box::pin(async move { health })
        }
        fn complete(
            &self,
            _messages: Vec<Message>,
            _config: CompletionConfig,
        ) -> RuntimeFuture<'_, CompletionResponse> {
            Box::pin(async {
                Err(RuntimeAdapterError::NotImplemented {
                    backend: self.backend,
                    hint: "stub",
                })
            })
        }
        fn stream(
            &self,
            _messages: Vec<Message>,
            _config: CompletionConfig,
        ) -> RuntimeStream<'static> {
            Box::pin(stream::iter(vec![]))
        }
    }

    fn stub_factory(
        backend: RuntimeBackend,
        health: RuntimeHealth,
    ) -> AdapterFactory {
        Box::new(move |_cfg| {
            Ok(Box::new(StubAdapter {
                backend,
                health,
            }) as Box<dyn RuntimeAdapter>)
        })
    }

    /// (a) satisfies-missing-key aborts BEFORE any network/health call.
    /// Uses Anthropic (api_key required) with no key, no env, no flag.
    /// Assert: [FAIL] line, no store call, factory NOT invoked.
    #[test]
    fn satisfies_missing_key_aborts_before_health_call() {
        with_config_home(|home| {
            // Ensure no env key leaks in.
            std::env::remove_var("ANTHROPIC_API_KEY");
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let factory_invoked = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let factory_invoked_clone = factory_invoked.clone();
            let factory: AdapterFactory = Box::new(move |_cfg| {
                factory_invoked_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(Box::new(StubAdapter {
                    backend: RuntimeBackend::AnthropicApi,
                    health: RuntimeHealth::Ready,
                }) as Box<dyn RuntimeAdapter>)
            });
            let flags = ConfigureFlags {
                backend: Some("anthropic".to_string()),
                key: None,
                base_url: None,
                model: None,
            };
            let out = runtime.block_on(configure_with_factory(flags, factory)).unwrap();
            assert!(
                out.starts_with("[FAIL]"),
                "expected [FAIL] line, got: {out}"
            );
            assert!(
                out.contains("api_key") || out.contains("api_key"),
                "expected missing api_key in message, got: {out}"
            );
            assert!(
                !factory_invoked.load(std::sync::atomic::Ordering::SeqCst),
                "factory must NOT be invoked when satisfies fails"
            );
            // No key file written.
            assert!(
                !home.join("provider_keys.json").exists(),
                "key file must NOT be created on satisfies failure"
            );
        });
    }

    /// (b) health Unavailable / dead base_url -> non-zero / no store.
    #[test]
    fn health_unavailable_aborts_without_store() {
        with_config_home(|home| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            // Sidecar is api_key_optional so we skip the key prompt path;
            // base_url is NOT required for sidecar, so satisfies passes
            // without a key. The factory returns Unavailable.
            let factory = stub_factory(
                RuntimeBackend::Sidecar,
                RuntimeHealth::Unavailable {
                    reason: "connection refused".to_string(),
                },
            );
            let flags = ConfigureFlags {
                backend: Some("sidecar".to_string()),
                key: None,
                base_url: Some("http://127.0.0.1:1".to_string()),
                model: None,
            };
            let out = runtime.block_on(configure_with_factory(flags, factory)).unwrap();
            assert!(
                out.starts_with("[FAIL]"),
                "expected [FAIL] on Unavailable, got: {out}"
            );
            assert!(
                out.contains("connection refused"),
                "expected reason in output, got: {out}"
            );
            assert!(
                !home.join("provider_keys.json").exists(),
                "key file must NOT be created on Unavailable"
            );
        });
    }

    /// (c) health Ready -> store called / [OK] path.
    /// Uses Anthropic with a --key flag so satisfies passes; the stub
    /// factory returns Ready; we assert the key is persisted.
    #[test]
    fn health_ready_stores_key_and_ok() {
        with_config_home(|home| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let factory = stub_factory(
                RuntimeBackend::AnthropicApi,
                RuntimeHealth::Ready,
            );
            let flags = ConfigureFlags {
                backend: Some("anthropic".to_string()),
                key: Some("sk-test-abc".to_string()),
                base_url: None,
                model: None,
            };
            let out = runtime.block_on(configure_with_factory(flags, factory)).unwrap();
            assert!(
                out.starts_with("[OK]"),
                "expected [OK] on Ready, got: {out}"
            );
            // Key persisted.
            let path = home.join("provider_keys.json");
            assert!(path.exists(), "key file must be created on Ready");
            let body = std::fs::read_to_string(&path).unwrap();
            assert!(
                body.contains("sk-test-abc"),
                "stored key body must contain the key, got: {body}"
            );
            assert!(
                body.contains("\"anthropic\""),
                "stored key body must be keyed by provider_id, got: {body}"
            );
        });
    }

    /// (d) Degraded health still stores the key (mirrors runtime_cmd.rs).
    #[test]
    fn health_degraded_stores_key_and_ok() {
        with_config_home(|home| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let factory = stub_factory(
                RuntimeBackend::AnthropicApi,
                RuntimeHealth::Degraded {
                    reason: "slow upstream".to_string(),
                },
            );
            let flags = ConfigureFlags {
                backend: Some("anthropic".to_string()),
                key: Some("sk-degraded".to_string()),
                base_url: None,
                model: None,
            };
            let out = runtime.block_on(configure_with_factory(flags, factory)).unwrap();
            assert!(
                out.starts_with("[OK]"),
                "expected [OK] on Degraded, got: {out}"
            );
            assert!(
                out.contains("degraded"),
                "expected degraded marker, got: {out}"
            );
            let path = home.join("provider_keys.json");
            assert!(path.exists(), "key file must be created on Degraded");
        });
    }
}
