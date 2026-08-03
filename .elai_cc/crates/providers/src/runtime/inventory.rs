//! J-30 T6 -- per-backend inventory status (configured / verified_at /
//! models / stale).
//!
//! Read-side companion to T5's `configure` flow. `stamp_verified_at`
//! records a successful configure/verify as a unix-ms timestamp in
//! `provider_status.json` (a non-secret sidecar file in
//! `default_config_home()`). `inventory_status` / `all_inventory` read
//! that sidecar plus the descriptor + key_store + model catalog to
//! produce an `InventoryStatus` row per backend. The `stale` flag is
//! `verified_at.is_none() || age > STALE_THRESHOLD_MS`.
//!
//! No secrets live in `provider_status.json` (only `verified_at` +
//! `models`). No new crate deps. No network, no GPU; the clock is
//! injected so tests use `SeededClock`. See
//! `workflow/reasoning/J-30_provider_byok_custom_endpoints.md`.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use elai_core::clock::ClockProvider;
use elai_core::config::default_config_home;
use serde::{Deserialize, Serialize};

use super::config_descriptor::{
    descriptor_for, descriptor_for_custom, satisfies, BackendConfigDescriptor,
};
use super::custom_backends::{load_custom_providers, CustomBackendSpec};
use super::key_store::lookup as lookup_key;
use crate::model_catalog::ModelCatalog;
use super::secret::SecretString;
use super::types::{RuntimeBackend, RuntimeFactoryConfig};

const STATUS_FILE_NAME: &str = "provider_status.json";
const STALE_THRESHOLD_MS: u64 = 7 * 24 * 60 * 60 * 1000;
const STALE_OVERRIDE_ENV: &str = "ELAI_INVENTORY_STALE_MS";

#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("inventory io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("inventory parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("inventory home directory not set (ELAI_CONFIG_HOME / HOME)")]
    HomeNotSet,
    #[error("inventory custom backend load failed: {0}")]
    Custom(#[from] super::custom_backends::CustomBackendError),
    #[error("inventory has no builtin descriptor for backend `{0:?}`; use inventory_status_custom for Custom")]
    NoBuiltinDescriptor(RuntimeBackend),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderStatusRow {
    #[serde(default)]
    pub verified_at: Option<u64>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryStatus {
    pub backend: String,
    pub display_name: String,
    pub configured: bool,
    pub verified_at: Option<u64>,
    pub models: Vec<String>,
    pub stale: bool,
}

fn status_path() -> Result<PathBuf, InventoryError> {
    let home = default_config_home();
    if home.as_os_str().is_empty() {
        return Err(InventoryError::HomeNotSet);
    }
    Ok(home.join(STATUS_FILE_NAME))
}

fn read_status_map(path: &Path) -> Result<BTreeMap<String, ProviderStatusRow>, InventoryError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BTreeMap::new());
        }
        Err(err) => return Err(err.into()),
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    if buf.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    let map: BTreeMap<String, ProviderStatusRow> = serde_json::from_str(&buf)?;
    Ok(map)
}

fn write_status_atomic(
    path: &Path,
    map: &BTreeMap<String, ProviderStatusRow>,
) -> Result<(), InventoryError> {
    let mut tmp_path = OsString::from(path);
    tmp_path.push(".tmp");
    let tmp_path = PathBuf::from(tmp_path);
    {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;
        let mut writer = BufWriter::new(file);
        let body = serde_json::to_string_pretty(map)?;
        writer.write_all(body.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        let file = writer
            .into_inner()
            .map_err(|err| std::io::Error::other(err.to_string()))?;
        file.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn stale_threshold_ms() -> u64 {
    match std::env::var(STALE_OVERRIDE_ENV) {
        Ok(value) => value.parse::<u64>().unwrap_or(STALE_THRESHOLD_MS),
        Err(_) => STALE_THRESHOLD_MS,
    }
}

fn now_ms(clock: &dyn ClockProvider) -> u64 {
    clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0) as u64
}

fn is_stale(verified_at: Option<u64>, now_ms: u64) -> bool {
    match verified_at {
        None => true,
        Some(ts) => now_ms.saturating_sub(ts) > stale_threshold_ms(),
    }
}

fn family_for_builtin(backend: RuntimeBackend) -> Option<&'static str> {
    match backend {
        RuntimeBackend::AnthropicApi => Some("sonnet"),
        RuntimeBackend::ClaudeCode => Some("sonnet"),
        RuntimeBackend::CodexCli => Some("sonnet"),
        RuntimeBackend::OpenCode => Some("sonnet"),
        RuntimeBackend::Hermes => Some("sonnet"),
        RuntimeBackend::KiroCli => Some("sonnet"),
        RuntimeBackend::CopilotCli => Some("sonnet"),
        RuntimeBackend::Composer2 => Some("sonnet"),
        RuntimeBackend::Bedrock => Some("sonnet"),
        RuntimeBackend::AzureOpenAi => Some("sonnet"),
        RuntimeBackend::GeminiCli => Some("haiku"),
        RuntimeBackend::DeepSeekV4Flash => None,
        RuntimeBackend::CerebrasFree => None,
        RuntimeBackend::GroqFree => None,
        RuntimeBackend::Sidecar
        | RuntimeBackend::Ollama
        | RuntimeBackend::Vllm
        | RuntimeBackend::VllmV1
        | RuntimeBackend::Mlx
        | RuntimeBackend::LlamaCpp
        | RuntimeBackend::Sglang => Some("qwen-local"),
        RuntimeBackend::Ppm | RuntimeBackend::Lsp | RuntimeBackend::Custom
        | RuntimeBackend::FreePool => None,
    }
}

fn models_for_builtin(backend: RuntimeBackend, catalog: &ModelCatalog) -> Vec<String> {
    match family_for_builtin(backend) {
        Some(family) => catalog
            .get_by_family(family)
            .into_iter()
            .map(|entry| entry.model_id.clone())
            .collect(),
        None => Vec::new(),
    }
}

fn env_non_empty(name: &str) -> bool {
    match std::env::var_os(name) {
        Some(value) => !value.to_string_lossy().trim().is_empty(),
        None => false,
    }
}

fn first_api_key_env(descriptor: &BackendConfigDescriptor) -> Option<&'static str> {
    descriptor.keys.iter().find_map(|k| {
        if k.kind == super::config_descriptor::ConfigKeyKind::ApiKey && k.secret {
            Some(k.name)
        } else {
            None
        }
    })
}

fn first_base_url_env(descriptor: &BackendConfigDescriptor) -> Option<&'static str> {
    descriptor.keys.iter().find_map(|k| {
        if k.kind == super::config_descriptor::ConfigKeyKind::BaseUrl {
            Some(k.name)
        } else {
            None
        }
    })
}

fn first_model_env(descriptor: &BackendConfigDescriptor) -> Option<&'static str> {
    descriptor.keys.iter().find_map(|k| {
        if k.kind == super::config_descriptor::ConfigKeyKind::Model {
            Some(k.name)
        } else {
            None
        }
    })
}

fn populate_cfg_from_env_and_store(
    backend: RuntimeBackend,
    descriptor: &BackendConfigDescriptor,
    provider_id: &str,
) -> RuntimeFactoryConfig {
    let mut cfg = RuntimeFactoryConfig::for_backend(backend);
    if let Some(env_name) = first_api_key_env(descriptor) {
        if let Ok(Some(key)) = lookup_key(provider_id, env_name) {
            cfg.api_key = Some(key);
        } else if env_non_empty(env_name) {
            cfg.api_key = Some(SecretString::new(
                std::env::var_os(env_name)
                    .map(|v| v.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ));
        }
    }
    if let Some(env_name) = first_base_url_env(descriptor) {
        if let Ok(value) = std::env::var(env_name) {
            if !value.trim().is_empty() {
                cfg.base_url = Some(value);
            }
        }
    }
    if let Some(env_name) = first_model_env(descriptor) {
        if let Ok(value) = std::env::var(env_name) {
            if !value.trim().is_empty() {
                cfg.model = Some(value);
            }
        }
    }
    cfg
}

pub fn stamp_verified_at(
    provider_id: &str,
    models: Vec<String>,
    clock: &dyn ClockProvider,
) -> Result<(), InventoryError> {
    let path = status_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut map = read_status_map(&path)?;
    map.insert(
        provider_id.to_string(),
        ProviderStatusRow {
            verified_at: Some(now_ms(clock)),
            models,
        },
    );
    write_status_atomic(&path, &map)?;
    Ok(())
}

pub fn read_status(provider_id: &str) -> Result<Option<ProviderStatusRow>, InventoryError> {
    let path = status_path()?;
    let map = read_status_map(&path)?;
    Ok(map.get(provider_id).cloned())
}

pub fn inventory_status(
    backend: RuntimeBackend,
    clock: &dyn ClockProvider,
) -> Result<InventoryStatus, InventoryError> {
    let descriptor = match descriptor_for(backend) {
        Some(d) => d,
        None => {
            return Err(InventoryError::NoBuiltinDescriptor(backend));
        }
    };
    let canon = backend.canonical_name();
    let provider_id = friendly_provider_id(canon).unwrap_or(canon).to_string();
    let cfg = populate_cfg_from_env_and_store(backend, descriptor, &provider_id);
    let configured = satisfies(descriptor, &cfg).is_ok();
    let catalog = ModelCatalog::load_or_default();
    let models = models_for_builtin(backend, &catalog);
    let row = read_status(&provider_id)?;
    let verified_at = row.as_ref().and_then(|r| r.verified_at);
    let now = now_ms(clock);
    let stale = is_stale(verified_at, now);
    Ok(InventoryStatus {
        backend: canon.to_string(),
        display_name: descriptor.display_name.to_string(),
        configured,
        verified_at,
        models,
        stale,
    })
}

pub fn inventory_status_custom(
    spec: &CustomBackendSpec,
    clock: &dyn ClockProvider,
) -> Result<InventoryStatus, InventoryError> {
    let descriptor = descriptor_for_custom(spec);
    let provider_id = spec.name.clone();
    let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Custom);
    cfg.runtime_profile = Some(spec.name.clone());
    cfg.base_url = Some(spec.base_url.clone());
    if !spec.models.is_empty() {
        cfg.model = Some(spec.models[0].clone());
    }
    if let Ok(Some(key)) = lookup_key(&provider_id, &spec.api_key_ref) {
        cfg.api_key = Some(key);
    } else if env_non_empty(&spec.api_key_ref) {
        cfg.api_key = Some(SecretString::new(
            std::env::var_os(&spec.api_key_ref)
                .map(|v| v.to_string_lossy().into_owned())
                .unwrap_or_default(),
        ));
    }
    let configured = satisfies(&descriptor, &cfg).is_ok();
    let models = spec.models.clone();
    let row = read_status(&provider_id)?;
    let verified_at = row.as_ref().and_then(|r| r.verified_at);
    let now = now_ms(clock);
    let stale = is_stale(verified_at, now);
    Ok(InventoryStatus {
        backend: spec.name.clone(),
        display_name: spec.name.clone(),
        configured,
        verified_at,
        models,
        stale,
    })
}

pub fn all_inventory(clock: &dyn ClockProvider) -> Result<Vec<InventoryStatus>, InventoryError> {
    let mut out: Vec<InventoryStatus> = Vec::new();
    for backend in RuntimeBackend::all() {
        if *backend == RuntimeBackend::Custom {
            continue;
        }
        if let Ok(status) = inventory_status(*backend, clock) {
            out.push(status);
        }
    }
    for spec in load_custom_providers()? {
        if let Ok(status) = inventory_status_custom(&spec, clock) {
            out.push(status);
        }
    }
    Ok(out)
}

fn friendly_provider_id(canon: &str) -> Option<&'static str> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::{LogicalInstant, SeededClock};
    use std::sync::Mutex;
    use std::time::{Duration, Instant, SystemTime};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct FixedClock {
        now: SystemTime,
    }

    impl FixedClock {
        fn new(now: SystemTime) -> Self {
            Self { now }
        }
    }

    impl ClockProvider for FixedClock {
        fn now(&self) -> LogicalInstant {
            LogicalInstant::from_tick(0)
        }
        fn elapsed_since(&self, _mark: LogicalInstant) -> Duration {
            Duration::ZERO
        }
        fn system_now(&self) -> SystemTime {
            self.now
        }
        fn instant_now(&self) -> Instant {
            Instant::now()
        }
    }

    fn with_config_home<F: FnOnce(&Path)>(f: F) {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let dir = tempfile::tempdir().expect("tempdir");
        std::env::set_var("ELAI_CONFIG_HOME", dir.path());
        f(dir.path());
        std::env::remove_var("ELAI_CONFIG_HOME");
    }

    fn seeded_clock_with_epoch(epoch: SystemTime) -> SeededClock {
        SeededClock::with_epoch(epoch)
    }

    #[test]
    fn stamp_then_read_returns_stamped_row() {
        with_config_home(|_| {
            let clock = seeded_clock_with_epoch(UNIX_EPOCH + Duration::from_secs(1_700_000_000));
            let now = now_ms(&clock);
            stamp_verified_at("anthropic", vec!["claude-sonnet-4-6".to_string()], &clock)
                .expect("stamp");
            let row = read_status("anthropic").expect("read").expect("present");
            assert_eq!(row.verified_at, Some(now));
            assert_eq!(row.models, vec!["claude-sonnet-4-6".to_string()]);
        });
    }

    #[test]
    fn stamp_is_atomic_and_persistent() {
        with_config_home(|home| {
            let clock = SeededClock::new();
            stamp_verified_at("openai", vec![], &clock).expect("stamp");
            assert!(home.join(STATUS_FILE_NAME).exists());
            assert!(!home.join("provider_status.json.tmp").exists());
        });
    }

    #[test]
    fn read_status_returns_none_when_absent() {
        with_config_home(|_| {
            let row = read_status("nobody").expect("read");
            assert!(row.is_none());
        });
    }

    #[test]
    fn inventory_status_fresh_after_stamp_is_not_stale() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let clock = seeded_clock_with_epoch(UNIX_EPOCH + Duration::from_secs(1_700_000_000));
            stamp_verified_at("anthropic", vec![], &clock).expect("stamp");
            let status = inventory_status(RuntimeBackend::AnthropicApi, &clock).expect("status");
            assert_eq!(status.backend, "anthropic_api");
            assert!(!status.stale, "freshly stamped must not be stale");
            assert!(status.verified_at.is_some());
        });
    }

    #[test]
    fn inventory_status_stale_after_8_days() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let t0 = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
            let stamp_clock = seeded_clock_with_epoch(t0);
            stamp_verified_at("anthropic", vec![], &stamp_clock).expect("stamp");
            let later_clock = FixedClock::new(t0 + Duration::from_secs(8 * 24 * 60 * 60));
            let status =
                inventory_status(RuntimeBackend::AnthropicApi, &later_clock).expect("status");
            assert!(status.stale, "8 days past stamp must be stale");
        });
    }

    #[test]
    fn inventory_status_never_stamped_is_stale() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let clock = SeededClock::new();
            let status = inventory_status(RuntimeBackend::AnthropicApi, &clock).expect("status");
            assert!(status.stale);
            assert!(status.verified_at.is_none());
        });
    }

    #[test]
    fn inventory_status_sidecar_configured_no_key() {
        with_config_home(|_| {
            std::env::remove_var("ELAI_SIDECAR_URL");
            let clock = SeededClock::new();
            let status = inventory_status(RuntimeBackend::Sidecar, &clock).expect("status");
            assert!(
                status.configured,
                "sidecar is api_key_optional, must be configured without a key"
            );
            assert_eq!(status.backend, "sidecar");
        });
    }

    #[test]
    fn inventory_status_anthropic_unconfigured_without_key() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            let clock = SeededClock::new();
            let status = inventory_status(RuntimeBackend::AnthropicApi, &clock).expect("status");
            assert!(
                !status.configured,
                "anthropic requires a key; without one configured must be false"
            );
        });
    }

    #[test]
    fn inventory_status_custom_reports_spec_models() {
        with_config_home(|_| {
            std::env::remove_var("MY_CUSTOM_KEY");
            let spec = CustomBackendSpec {
                name: "my-vllm".into(),
                engine: super::super::custom_backends::CustomEngine::OpenAiCompat,
                base_url: "https://api.example.com".into(),
                api_key_ref: "MY_CUSTOM_KEY".into(),
                models: vec!["llama-3-70b".into()],
                headers: None,
                timeout_s: None,
            };
            let clock = SeededClock::new();
            let status = inventory_status_custom(&spec, &clock).expect("status");
            assert_eq!(status.backend, "my-vllm");
            assert_eq!(status.display_name, "my-vllm");
            assert_eq!(status.models, vec!["llama-3-70b".to_string()]);
            assert!(!status.configured, "no key env, must be unconfigured");
            assert!(status.stale);
        });
    }

    #[test]
    fn inventory_status_custom_configured_when_env_key_present() {
        with_config_home(|_| {
            std::env::set_var("MY_CUSTOM_KEY", "sk-test");
            let spec = CustomBackendSpec {
                name: "my-vllm-2".into(),
                engine: super::super::custom_backends::CustomEngine::OpenAiCompat,
                base_url: "https://api.example.com".into(),
                api_key_ref: "MY_CUSTOM_KEY".into(),
                models: vec!["llama-3-70b".into()],
                headers: None,
                timeout_s: None,
            };
            let clock = SeededClock::new();
            let status = inventory_status_custom(&spec, &clock).expect("status");
            std::env::remove_var("MY_CUSTOM_KEY");
            assert!(status.configured, "env key set, must be configured");
        });
    }

    #[test]
    fn all_inventory_includes_builtins_and_customs() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::fs::write(
                default_config_home().join("custom_providers.toml"),
                r#"[[providers]]
name = "my-vllm-all"
engine = "openai-compat"
base_url = "https://api.example.com"
api_key_ref = "MY_CUSTOM_KEY_ALL"
models = ["llama-3-70b"]
"#,
            )
            .expect("write custom toml");
            let clock = SeededClock::new();
            let all = all_inventory(&clock).expect("all");
            assert!(
                all.iter().any(|s| s.backend == "anthropic_api"),
                "builtins present"
            );
            assert!(
                all.iter().any(|s| s.backend == "my-vllm-all"),
                "custom present"
            );
        });
    }

    #[test]
    fn stale_threshold_override_shortens_window() {
        with_config_home(|_| {
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::set_var("ELAI_INVENTORY_STALE_MS", "1000");
            let t0 = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
            let stamp_clock = seeded_clock_with_epoch(t0);
            stamp_verified_at("anthropic", vec![], &stamp_clock).expect("stamp");
            let later_clock = FixedClock::new(t0 + Duration::from_millis(2000));
            let status =
                inventory_status(RuntimeBackend::AnthropicApi, &later_clock).expect("status");
            std::env::remove_var("ELAI_INVENTORY_STALE_MS");
            assert!(status.stale, "2s > 1s override, must be stale");
        });
    }
}
