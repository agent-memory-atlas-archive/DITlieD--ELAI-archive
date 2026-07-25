//! J-30 T2 -- user-registered custom endpoints (custom_providers.toml).
//!
//! Process-local `OnceLock<BTreeMap<String, CustomBackendSpec>>` registry
//! layered on `resolve_backend` / `create_runtime`. Loaded at startup from
//! `custom_providers.toml` in the ELAI config dir; missing file = Ok(empty).
//! Two engines: `anthropic-compat` (reuses `AnthropicApiRuntime` via a thin
//! wrapper that overrides `backend()` to report `Custom`) and `openai-compat`
//! (a thin `CustomOpenAiCompatRuntime` that speaks `/v1/chat/completions`
//! directly via `reqwest`). No OAuth / device_code arms (I-50 owns those).
//! See `workflow/reasoning/J-30_provider_byok_custom_endpoints.md`.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex, OnceLock};

use elai_core::config::default_config_home;
use futures_util::stream::{self, BoxStream};
use serde::Deserialize;

use super::anthropic_api::AnthropicApiRuntime;
use super::azure_openai::normalize_finish_reason;
use super::event_sink::{RuntimeEvent, runtime_event_sink};
use super::secret::SecretString;
use super::types::{
    unavailable, AuthSource, CompletionConfig, CompletionResponse, Message, MessageRole,
    RuntimeAdapter, RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig,
    RuntimeFuture, RuntimeHealthFuture, StreamChunk,
};

const CONFIG_FILE_NAME: &str = "custom_providers.toml";
const DEFAULT_MAX_TOKENS: u32 = 512;

#[derive(Debug, Clone, Deserialize)]
pub struct CustomBackendSpec {
    pub name: String,
    pub engine: CustomEngine,
    pub base_url: String,
    pub api_key_ref: String,
    pub models: Vec<String>,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub timeout_s: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CustomEngine {
    AnthropicCompat,
    #[serde(alias = "openai-compat")]
    OpenAiCompat,
}

#[derive(Debug, thiserror::Error)]
pub enum CustomBackendError {
    #[error("custom_providers.toml io: {0}")]
    Io(#[from] std::io::Error),
    #[error("custom_providers.toml parse: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("custom backend `{name}` missing api_key_ref")]
    MissingKeyRef { name: String },
    #[error("custom backend has empty name")]
    EmptyName,
    #[error("custom backend `{name}` has no models")]
    NoModels { name: String },
    #[error("custom backend `{name}` key resolution failed: {source}")]
    KeyResolve {
        name: String,
        #[source]
        source: super::key_store::KeyStoreError,
    },
}

#[derive(Debug, Deserialize)]
struct CustomProvidersFile {
    #[serde(default)]
    providers: Vec<CustomBackendSpec>,
}

static REGISTRY: OnceLock<Mutex<Option<BTreeMap<String, CustomBackendSpec>>>> = OnceLock::new();

fn registry() -> &'static Mutex<Option<BTreeMap<String, CustomBackendSpec>>> {
    REGISTRY.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn test_reset_registry() {
    *registry().lock().unwrap() = None;
}

fn config_path() -> std::path::PathBuf {
    default_config_home().join(CONFIG_FILE_NAME)
}

/// Load custom_providers.toml on first unknown-backend lookup if the
/// registry is still empty. Idempotent: a populated registry short-circuits,
/// and a missing file is Ok(empty) so the call is side-effect-free on a
/// clean install. This is the in-crate production caller for
/// `load_custom_providers` + `register_custom_backends` (R7: the startup
/// boot hook is a belt-and-suspenders second caller, not the only one).
pub fn ensure_custom_backends_loaded() -> Result<(), CustomBackendError> {
    {
        let guard = registry().lock().expect("custom_backends registry lock");
        if guard.is_some() {
            return Ok(());
        }
    }
    let specs = load_custom_providers()?;
    if specs.is_empty() {
        register_custom_backends(Vec::new())?;
    } else {
        register_custom_backends(specs)?;
    }
    Ok(())
}

pub fn load_custom_providers() -> Result<Vec<CustomBackendSpec>, CustomBackendError> {
    let path = config_path();
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    if buf.trim().is_empty() {
        return Ok(Vec::new());
    }
    let parsed: CustomProvidersFile = toml::from_str(&buf)?;
    for spec in &parsed.providers {
        if spec.name.trim().is_empty() {
            return Err(CustomBackendError::EmptyName);
        }
        if spec.api_key_ref.trim().is_empty() {
            return Err(CustomBackendError::MissingKeyRef {
                name: spec.name.clone(),
            });
        }
        if spec.models.is_empty() {
            return Err(CustomBackendError::NoModels {
                name: spec.name.clone(),
            });
        }
    }
    Ok(parsed.providers)
}

pub fn register_custom_backends(
    specs: Vec<CustomBackendSpec>,
) -> Result<(), CustomBackendError> {
    let map: BTreeMap<String, CustomBackendSpec> =
        specs.into_iter().map(|s| (s.name.clone(), s)).collect();
    let names: Vec<String> = map.keys().cloned().collect();
    let mut guard = registry().lock().expect("custom_backends registry lock");
    if guard.is_some() {
        tracing::warn!(
            target = "custom_backends",
            "register_custom_backends called twice; ignoring second registration"
        );
        return Ok(());
    }
    *guard = Some(map);
    for name in names {
        runtime_event_sink().record(RuntimeEvent::CustomBackendRegistered {
            backend: name,
            ok: true,
        });
    }
    Ok(())
}

pub fn resolve_custom(name: &str) -> Option<CustomBackendSpec> {
    registry()
        .lock()
        .ok()
        .and_then(|g| g.as_ref().and_then(|m| m.get(name).cloned()))
}

pub fn build_adapter(
    spec: &CustomBackendSpec,
) -> Result<Box<dyn RuntimeAdapter>, RuntimeAdapterError> {
    match spec.engine {
        CustomEngine::AnthropicCompat => build_anthropic_compat(spec),
        CustomEngine::OpenAiCompat => Ok(Box::new(CustomOpenAiCompatRuntime::from_spec(spec)?)),
    }
}

fn resolve_key(spec: &CustomBackendSpec) -> Result<SecretString, RuntimeAdapterError> {
    let key = super::key_store::lookup(&spec.name, &spec.api_key_ref).map_err(|source| {
        RuntimeAdapterError::Other(format!(
            "custom backend `{}` key resolution failed: {}",
            spec.name, source
        ))
    })?;
    key.ok_or_else(|| RuntimeAdapterError::MissingField {
        backend: RuntimeBackend::Custom,
        field: "api_key_ref",
        reason: format!(
            "custom backend `{}`: key not found in env `{}` or key store",
            spec.name, spec.api_key_ref
        ),
    })
}

fn build_anthropic_compat(
    spec: &CustomBackendSpec,
) -> Result<Box<dyn RuntimeAdapter>, RuntimeAdapterError> {
    let key = resolve_key(spec)?;
    let cfg = RuntimeFactoryConfig {
        backend: RuntimeBackend::AnthropicApi,
        cli_path: None,
        base_url: Some(spec.base_url.clone()),
        api_key: None,
        auth_source: Some(AuthSource::ApiKey(key)),
        model: Some(spec.models[0].clone()),
        permission_mode: super::types::RuntimePermissionMode::default(),
        runtime_profile: Some(spec.name.clone()),
        cwd: None,
        startup_output_timeout_seconds: None,
        stdout_idle_timeout_seconds: None,
        sidecar: None,
        allow_danger: false,
        transport: super::types::BridgeTransport::default(),
    };
    let inner = AnthropicApiRuntime::from_factory_config(&cfg)?;
    Ok(Box::new(CustomAnthropicCompatRuntime {
        inner,
        name: spec.name.clone(),
    }))
}

#[derive(Clone)]
struct CustomAnthropicCompatRuntime {
    inner: AnthropicApiRuntime,
    name: String,
}

impl std::fmt::Debug for CustomAnthropicCompatRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomAnthropicCompatRuntime")
            .field("name", &self.name)
            .finish()
    }
}

impl RuntimeAdapter for CustomAnthropicCompatRuntime {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::Custom
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        self.inner.capabilities()
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        Box::pin(async move {
            unavailable(format!(
                "custom anthropic-compat backend `{}` health check requires live endpoint",
                self.name
            ))
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let inner = self.inner.clone();
        Box::pin(async move { inner.complete(messages, config).await })
    }

    fn stream(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        self.inner.stream(messages, config)
    }
}

#[derive(Clone)]
struct CustomOpenAiCompatRuntime {
    http: reqwest::Client,
    base_url: String,
    api_key: SecretString,
    default_model: String,
    name: String,
    headers: Option<BTreeMap<String, String>>,
}

impl std::fmt::Debug for CustomOpenAiCompatRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomOpenAiCompatRuntime")
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("default_model", &self.default_model)
            .finish()
    }
}

impl CustomOpenAiCompatRuntime {
    fn from_spec(spec: &CustomBackendSpec) -> Result<Self, RuntimeAdapterError> {
        let key = resolve_key(spec)?;
        let mut client_builder = reqwest::Client::builder();
        if let Some(timeout) = spec.timeout_s {
            client_builder = client_builder
                .timeout(std::time::Duration::from_secs(timeout));
        }
        let http = client_builder.build().map_err(|e| {
            RuntimeAdapterError::Other(format!("custom backend http client build failed: {e}"))
        })?;
        Ok(Self {
            http,
            base_url: spec.base_url.trim_end_matches('/').to_string(),
            api_key: key,
            default_model: spec.models[0].clone(),
            name: spec.name.clone(),
            headers: spec.headers.clone(),
        })
    }

    fn endpoint_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }

    fn build_request_body(
        &self,
        messages: Vec<Message>,
        config: &CompletionConfig,
    ) -> serde_json::Value {
        use serde_json::json;

        let model = config
            .model
            .clone()
            .unwrap_or_else(|| self.default_model.clone());
        let max_tokens = if config.max_tokens == 0 {
            DEFAULT_MAX_TOKENS
        } else {
            config.max_tokens
        };

        let mut msgs: Vec<serde_json::Value> = Vec::new();
        if let Some(system) = &config.system {
            msgs.push(json!({ "role": "system", "content": system }));
        }
        for m in messages {
            let role = match m.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
                MessageRole::Tool => "tool",
            };
            msgs.push(json!({ "role": role, "content": m.content }));
        }

        let mut body = json!({
            "model": model,
            "messages": msgs,
            "max_tokens": max_tokens,
            "temperature": config.temperature,
            "stream": false,
        });
        if !config.stop_sequences.is_empty() {
            body["stop"] = json!(config.stop_sequences);
        }
        if let Some(seed) = config.seed {
            body["seed"] = json!(seed);
        }
        body
    }

    async fn send_completion(
        &self,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeAdapterError> {
        let mut req = self
            .http
            .post(self.endpoint_url())
            .bearer_auth(self.api_key.expose_secret())
            .json(&body);
        if let Some(extra) = &self.headers {
            for (k, v) in extra {
                req = req.header(k, v);
            }
        }
        let response = req.send().await.map_err(|e| {
            RuntimeAdapterError::Network(format!(
                "custom backend `{}` request failed: {e}",
                self.name
            ))
        })?;
        let status = response.status();
        if status.is_success() {
            return response.json().await.map_err(|e| {
                RuntimeAdapterError::Other(format!(
                    "custom backend `{}` response decode failed: {e}",
                    self.name
                ))
            });
        }
        let body_tail = response.text().await.unwrap_or_default();
        let body_tail = if body_tail.len() > 256 {
            format!("{}...", &body_tail[..256])
        } else {
            body_tail
        };
        Err(RuntimeAdapterError::Other(format!(
            "custom backend `{}` returned status {status}: {body_tail}",
            self.name
        )))
    }
}

impl RuntimeAdapter for CustomOpenAiCompatRuntime {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::Custom
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: false,
            supports_streaming: false,
            supports_vision: false,
            max_context: 128_000,
            max_output: 16_384,
            json_schema_mode: true,
            supports_prefill: false,
            supports_caching: false,
            native_thinking_budget: false,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let name = self.name.clone();
        Box::pin(async move {
            unavailable(format!(
                "custom openai-compat backend `{name}` health check requires live endpoint"
            ))
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let this = self.clone();
        Box::pin(async move {
            let body = this.build_request_body(messages, &config);
            let payload = this.send_completion(body).await?;
            parse_openai_chat_response(&this.default_model, &payload)
        })
    }

    fn stream(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        let err = RuntimeAdapterError::Other(format!(
            "custom backend `{}` streaming not yet implemented",
            self.name
        ));
        Box::pin(stream::once(async move { Err(err) }))
    }
}

fn parse_openai_chat_response(
    default_model: &str,
    body: &serde_json::Value,
) -> Result<CompletionResponse, RuntimeAdapterError> {
    let choices = body
        .get("choices")
        .and_then(|c| c.as_array())
        .ok_or_else(|| {
            RuntimeAdapterError::Other("openai-compat response missing 'choices' array".to_string())
        })?;
    let choice = choices.first().ok_or_else(|| {
        RuntimeAdapterError::Other("openai-compat response 'choices' array is empty".to_string())
    })?;
    let text = choice
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    let stop_reason = choice
        .get("finish_reason")
        .and_then(|r| r.as_str())
        .map(|r| normalize_finish_reason(r).to_string());
    let (input_tokens, output_tokens) = body
        .get("usage")
        .map(|u| {
            let inp = u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let out = u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            (inp, out)
        })
        .unwrap_or((0, 0));
    let model_name = body
        .get("model")
        .and_then(|m| m.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(default_model)
        .to_string();
    Ok(CompletionResponse {
        text,
        stop_reason,
        usage: (input_tokens, output_tokens),
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
        model: model_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_config_home<F: FnOnce(&std::path::Path)>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        test_reset_registry();
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("ELAI_CONFIG_HOME", dir.path());
        f(dir.path());
        std::env::remove_var("ELAI_CONFIG_HOME");
    }

    fn write_config(home: &std::path::Path, contents: &str) {
        std::fs::write(home.join(CONFIG_FILE_NAME), contents).expect("write config");
    }

    #[test]
    fn load_missing_file_returns_empty() {
        with_config_home(|_| {
            let specs = load_custom_providers().expect("load");
            assert!(specs.is_empty());
        });
    }

    #[test]
    fn load_empty_file_returns_empty() {
        with_config_home(|home| {
            write_config(home, "");
            let specs = load_custom_providers().expect("load");
            assert!(specs.is_empty());
        });
    }

    #[test]
    fn load_valid_spec_parses() {
        with_config_home(|home| {
            write_config(
                home,
                r#"[[providers]]
name = "my-anthropic"
engine = "anthropic-compat"
base_url = "https://api.example.com"
api_key_ref = "MY_ANTHROPIC_KEY"
models = ["claude-sonnet-4-6"]
"#,
            );
            let specs = load_custom_providers().expect("load");
            assert_eq!(specs.len(), 1);
            assert_eq!(specs[0].name, "my-anthropic");
            assert_eq!(specs[0].engine, CustomEngine::AnthropicCompat);
            assert_eq!(specs[0].models, vec!["claude-sonnet-4-6".to_string()]);
        });
    }

    #[test]
    fn load_missing_api_key_ref_fails_loud() {
        with_config_home(|home| {
            write_config(
                home,
                r#"[[providers]]
name = "bad"
engine = "openai-compat"
base_url = "https://api.example.com"
api_key_ref = ""
models = ["gpt-4o"]
"#,
            );
            let err = load_custom_providers().expect_err("must fail");
            assert!(matches!(err, CustomBackendError::MissingKeyRef { .. }));
        });
    }

    #[test]
    fn load_empty_models_fails_loud() {
        with_config_home(|home| {
            write_config(
                home,
                r#"[[providers]]
name = "bad"
engine = "openai-compat"
base_url = "https://api.example.com"
api_key_ref = "KEY"
models = []
"#,
            );
            let err = load_custom_providers().expect_err("must fail");
            assert!(matches!(err, CustomBackendError::NoModels { .. }));
        });
    }

    #[test]
    fn register_and_resolve_round_trip() {
        with_config_home(|_| {
            let spec = CustomBackendSpec {
                name: "round-trip-test".to_string(),
                engine: CustomEngine::OpenAiCompat,
                base_url: "https://api.example.com".to_string(),
                api_key_ref: "TEST_KEY".to_string(),
                models: vec!["gpt-4o".to_string()],
                headers: None,
                timeout_s: None,
            };
            register_custom_backends(vec![spec.clone()]).expect("register");
            let resolved = resolve_custom("round-trip-test").expect("must resolve");
            assert_eq!(resolved.base_url, "https://api.example.com");
            assert!(resolve_custom("nonexistent").is_none());
        });
    }

    #[test]
    fn resolve_backend_falls_through_to_custom_registry() {
        with_config_home(|_| {
            register_custom_backends(vec![CustomBackendSpec {
                name: "my-custom-endpoint".to_string(),
                engine: CustomEngine::OpenAiCompat,
                base_url: "https://api.example.com".to_string(),
                api_key_ref: "KEY".to_string(),
                models: vec!["gpt-4o".to_string()],
                headers: None,
                timeout_s: None,
            }])
            .expect("register");
            let resolved =
                super::super::resolve_backend("my-custom-endpoint").expect("must resolve");
            assert_eq!(resolved, RuntimeBackend::Custom);
        });
    }

    #[test]
    fn parse_openai_chat_response_extracts_fields() {
        let body = serde_json::json!({
            "model": "gpt-4o",
            "choices": [{
                "message": { "role": "assistant", "content": "Hello!" },
                "finish_reason": "stop",
                "index": 0
            }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 3 }
        });
        let resp = parse_openai_chat_response("gpt-4o", &body).expect("parse");
        assert_eq!(resp.text, "Hello!");
        assert_eq!(resp.stop_reason.as_deref(), Some("end_turn"));
        assert_eq!(resp.usage, (10, 3));
        assert_eq!(resp.model, "gpt-4o");
    }
}
