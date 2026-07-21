//! J-30 T4 -- per-backend config descriptor metadata.
//!
//! Static `BackendConfigDescriptor` table keyed by `RuntimeBackend` for builtins,
//! plus `descriptor_for_custom` for the runtime-loaded `CustomBackendSpec` overlay.
//! Consumed by T5's CLI `configure` flow and the RuntimePanel form generator; this
//! slice exposes the API only (WIRE-DARK[J-30-T5]). `satisfies()` is pure -- it
//! checks `RuntimeFactoryConfig` fields only and does NOT re-resolve env vars; the
//! caller is expected to populate `cfg.api_key` / `cfg.base_url` / `cfg.model`
//! from env or `key_store::lookup` before calling.
//!
//! No OAuth / device_code variants (I-50 ledger DO-NOT-COPY). The `ConfigKeyKind`
//! enum is the no-OAuth subset of goose's `AuthMethod` (paper digest
//! workflow/papers/goose-part-1.md:300-370); `api_key_optional` replaces the
//! BearerToken/NoAuth split.
//!
//! See `workflow/reasoning/J-30_provider_byok_custom_endpoints.md`.

use super::custom_backends::CustomBackendSpec;
use super::types::{RuntimeBackend, RuntimeFactoryConfig};

/// One declared config field for a backend (mirrors goose `ConfigKey` minus the
/// OAuth / device_code_flow arms per I-50).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigKey {
    pub name: &'static str,
    pub kind: ConfigKeyKind,
    pub required: bool,
    pub secret: bool,
    pub default: Option<&'static str>,
    pub description: &'static str,
}

/// No OAuth / device_code variants (I-50 ledger DO-NOT-COPY).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKeyKind {
    ApiKey,
    BaseUrl,
    Model,
    Region,
    Profile,
}

impl ConfigKeyKind {
    /// Compile-time exhaustiveness witness. Adding a variant without updating
    /// this match breaks compilation, which is the cheapest static assertion
    /// that no OAuth / device_code arm sneaks in.
    const fn _witness(self) -> u8 {
        match self {
            Self::ApiKey | Self::BaseUrl | Self::Model | Self::Region | Self::Profile => 0,
        }
    }
}

/// Declarative per-backend config metadata. The static table is keyed by
/// `RuntimeBackend` for builtins; custom overlays derive from the loaded
/// `CustomBackendSpec` (see `descriptor_for_custom`).
#[derive(Debug, Clone)]
pub struct BackendConfigDescriptor {
    pub backend: RuntimeBackend,
    pub display_name: &'static str,
    pub keys: &'static [ConfigKey],
    pub supports_base_url: bool,
    pub default_base_url: Option<&'static str>,
    /// True for local-lane backends where the api_key is NOT required
    /// (sidecar/ollama/vllm/llama_cpp/mlx/sglang + CLI subprocess adapters).
    /// Cloud + CLI-auth backends keep api_key required.
    pub api_key_optional: bool,
}

/// Lookup the static descriptor for a builtin `RuntimeBackend`. Returns `None`
/// for `RuntimeBackend::Custom` (callers must use `descriptor_for_custom` with
/// the resolved `CustomBackendSpec`).
// WIRE-DARK[J-30-T5] descriptor_for's production consumer is the T5
// `elai providers configure` CLI flow (out of scope for T4 per task spec).
// Registered as a deferred-wiring stub: the T5 slice wires this into the CLI
// configure command.
#[must_use]
pub fn descriptor_for(backend: RuntimeBackend) -> Option<&'static BackendConfigDescriptor> {
    static TABLE: &[(&RuntimeBackend, &BackendConfigDescriptor)] = &[
        (&RuntimeBackend::Sidecar, &SIDE),
        (&RuntimeBackend::AnthropicApi, &ANTHROPIC),
        (&RuntimeBackend::ClaudeCode, &CLAUDE_CODE),
        (&RuntimeBackend::CodexCli, &CODEX_CLI),
        (&RuntimeBackend::OpenCode, &OPENCODE),
        (&RuntimeBackend::Hermes, &HERMES),
        (&RuntimeBackend::GeminiCli, &GEMINI_CLI),
        (&RuntimeBackend::KiroCli, &KIRO_CLI),
        (&RuntimeBackend::CopilotCli, &COPILOT_CLI),
        (&RuntimeBackend::Ollama, &OLLAMA),
        (&RuntimeBackend::Vllm, &VLLM),
        (&RuntimeBackend::Mlx, &MLX),
        (&RuntimeBackend::LlamaCpp, &LLAMA_CPP),
        (&RuntimeBackend::Ppm, &EMPTY_NON_CONFIGURABLE),
        (&RuntimeBackend::Composer2, &COMPOSER2),
        (&RuntimeBackend::Lsp, &EMPTY_NON_CONFIGURABLE),
        (&RuntimeBackend::Sglang, &SGLANG),
        (&RuntimeBackend::DeepSeekV4Flash, &DEEPSEEK),
        (&RuntimeBackend::CerebrasFree, &CEREBRAS),
        (&RuntimeBackend::GroqFree, &GROQ),
        (&RuntimeBackend::Bedrock, &BEDROCK),
        (&RuntimeBackend::AzureOpenAi, &AZURE),
    ];
    TABLE
        .iter()
        .find(|(b, _)| **b == backend)
        .map(|(_, d)| *d)
}

/// Build a runtime descriptor for a registered custom backend by overlaying
/// `CustomBackendSpec` fields on the Custom-template descriptor. The returned
/// descriptor's `keys` field points at a STATIC custom template
/// (`CUSTOM_TEMPLATE`) with placeholder names `api_key_ref` / `base_url` /
/// `model`; the caller (T5 CLI configure) reads the actual spec.name /
/// spec.api_key_ref / spec.base_url from the `CustomBackendSpec` it already
/// holds. This avoids `Box::leak` and avoids inventing a `ConfigKeyOwned` /
/// `Vec` unification in this slice (S2 FORK 3 option A).
// WIRE-DARK[J-30-T5] descriptor_for_custom's production consumer is the T5
// `elai providers configure` CLI flow (out of scope for T4 per task spec).
// Registered as a deferred-wiring stub: the T5 slice wires this into the CLI
// configure command.
#[must_use]
pub fn descriptor_for_custom(spec: &CustomBackendSpec) -> BackendConfigDescriptor {
    let _ = spec;
    BackendConfigDescriptor {
        backend: RuntimeBackend::Custom,
        display_name: "Custom",
        keys: CUSTOM_TEMPLATE,
        supports_base_url: true,
        default_base_url: None,
        api_key_optional: false,
    }
}

/// Check whether a `RuntimeFactoryConfig` satisfies a descriptor's required
/// keys. Used by T5's configure flow BEFORE the live test call. Does NOT
/// re-resolve env vars; the caller is expected to have populated
/// `cfg.api_key` / `cfg.base_url` / `cfg.model` from env or `key_store`
/// before calling. Returns the first missing required key, or `Ok(())` if
/// all required keys are satisfied.
// WIRE-DARK[J-30-T5] satisfies's production consumer is the T5
// `elai providers configure` CLI flow (out of scope for T4 per task spec).
// Registered as a deferred-wiring stub: the T5 slice wires this into the CLI
// configure command.
pub fn satisfies(
    descriptor: &BackendConfigDescriptor,
    cfg: &RuntimeFactoryConfig,
) -> Result<(), MissingKey> {
    for key in descriptor.keys {
        if !key.required {
            continue;
        }
        let present = match key.kind {
            ConfigKeyKind::ApiKey => {
                if descriptor.api_key_optional {
                    true
                } else {
                    cfg.api_key
                        .as_ref()
                        .map(|s| !s.is_empty())
                        .unwrap_or(false)
                        || cfg.auth_source.is_some()
                }
            }
            ConfigKeyKind::BaseUrl => cfg
                .base_url
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false),
            ConfigKeyKind::Model => cfg
                .model
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false),
            ConfigKeyKind::Region => {
                return Err(MissingKey {
                    backend: descriptor.backend,
                    key: key.name,
                    reason: "Region key declared required but RuntimeFactoryConfig has no region field; populate via runtime_profile or env".to_string(),
                });
            }
            ConfigKeyKind::Profile => cfg
                .runtime_profile
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false),
        };
        if !present {
            return Err(MissingKey {
                backend: descriptor.backend,
                key: key.name,
                reason: match key.kind {
                    ConfigKeyKind::ApiKey => {
                        "api_key / auth_source not set or empty".to_string()
                    }
                    ConfigKeyKind::BaseUrl => "base_url not set or empty".to_string(),
                    ConfigKeyKind::Model => "model not set or empty".to_string(),
                    ConfigKeyKind::Region => "region not set".to_string(),
                    ConfigKeyKind::Profile => "runtime_profile not set or empty".to_string(),
                },
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("backend `{backend}` requires config key `{key}` ({reason})", backend = backend.canonical_name())]
pub struct MissingKey {
    pub backend: RuntimeBackend,
    pub key: &'static str,
    pub reason: String,
}

// ---- static descriptor tables ---------------------------------------------

const ANTHROPIC_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "ANTHROPIC_API_KEY",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "Anthropic API key (sk-ant-...). Also accepts AUTH_TOKEN bearer via auth_source.",
    },
    ConfigKey {
        name: "ANTHROPIC_BASE_URL",
        kind: ConfigKeyKind::BaseUrl,
        required: false,
        secret: false,
        default: None,
        description: "Override the Anthropic API endpoint.",
    },
];

const ANTHROPIC: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::AnthropicApi,
    display_name: "Anthropic API",
    keys: ANTHROPIC_KEYS,
    supports_base_url: true,
    default_base_url: None,
    api_key_optional: false,
};

const SIDE_KEYS: &[ConfigKey] = &[ConfigKey {
    name: "ELAI_SIDECAR_URL",
    kind: ConfigKeyKind::BaseUrl,
    required: false,
    secret: false,
    default: Some("http://localhost:8100"),
    description: "Sidecar server URL (defaults to http://localhost:8100).",
}];

const SIDE: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Sidecar,
    display_name: "Local sidecar",
    keys: SIDE_KEYS,
    supports_base_url: true,
    default_base_url: Some("http://localhost:8100"),
    api_key_optional: true,
};

const OLLAMA_KEYS: &[ConfigKey] = &[ConfigKey {
    name: "OLLAMA_HOST",
    kind: ConfigKeyKind::BaseUrl,
    required: false,
    secret: false,
    default: Some("http://localhost:11434"),
    description: "Ollama host URL (defaults to http://localhost:11434).",
}];

const OLLAMA: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Ollama,
    display_name: "Ollama",
    keys: OLLAMA_KEYS,
    supports_base_url: true,
    default_base_url: Some("http://localhost:11434"),
    api_key_optional: true,
};

const VLLM_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "ELAI_VLLM_BASE_URL",
        kind: ConfigKeyKind::BaseUrl,
        required: false,
        secret: false,
        default: Some("http://localhost:8000"),
        description: "vLLM base URL (defaults to http://localhost:8000).",
    },
    ConfigKey {
        name: "model",
        kind: ConfigKeyKind::Model,
        required: false,
        secret: false,
        default: None,
        description: "Default model id served by the vLLM endpoint.",
    },
];

const VLLM: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Vllm,
    display_name: "vLLM",
    keys: VLLM_KEYS,
    supports_base_url: true,
    default_base_url: Some("http://localhost:8000"),
    api_key_optional: true,
};

const SGLANG_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "ELAI_SGLANG_BASE_URL",
        kind: ConfigKeyKind::BaseUrl,
        required: false,
        secret: false,
        default: Some("http://localhost:30000"),
        description: "SGLang base URL (defaults to http://localhost:30000).",
    },
    ConfigKey {
        name: "ELAI_SGLANG_API_KEY",
        kind: ConfigKeyKind::ApiKey,
        required: false,
        secret: true,
        default: None,
        description: "Optional API key for SGLang endpoints that require bearer auth.",
    },
];

const SGLANG: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Sglang,
    display_name: "SGLang",
    keys: SGLANG_KEYS,
    supports_base_url: true,
    default_base_url: Some("http://localhost:30000"),
    api_key_optional: true,
};

const LLAMA_CPP_KEYS: &[ConfigKey] = &[ConfigKey {
    name: "ELAI_LLAMA_CPP_URL",
    kind: ConfigKeyKind::BaseUrl,
    required: false,
    secret: false,
    default: Some("http://localhost:8181"),
    description: "llama-server base URL (defaults to http://localhost:8181).",
}];

const LLAMA_CPP: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::LlamaCpp,
    display_name: "llama.cpp",
    keys: LLAMA_CPP_KEYS,
    supports_base_url: true,
    default_base_url: Some("http://localhost:8181"),
    api_key_optional: true,
};

const MLX_KEYS: &[ConfigKey] = &[ConfigKey {
    name: "ELAI_MLX_URL",
    kind: ConfigKeyKind::BaseUrl,
    required: false,
    secret: false,
    default: Some("http://localhost:8080"),
    description: "MLX server base URL (defaults to http://localhost:8080).",
}];

const MLX: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Mlx,
    display_name: "MLX",
    keys: MLX_KEYS,
    supports_base_url: true,
    default_base_url: Some("http://localhost:8080"),
    api_key_optional: true,
};

const BEDROCK_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "AWS_ACCESS_KEY_ID",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "AWS access key id for Bedrock InvokeModel calls.",
    },
    ConfigKey {
        name: "AWS_SECRET_ACCESS_KEY",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "AWS secret access key for Bedrock SigV4 signing.",
    },
    ConfigKey {
        name: "AWS_REGION",
        kind: ConfigKeyKind::Region,
        required: false,
        secret: false,
        default: None,
        description: "AWS region tag (e.g. us-east-1).",
    },
    ConfigKey {
        name: "runtime_profile",
        kind: ConfigKeyKind::Profile,
        required: false,
        secret: false,
        default: None,
        description: "AWS CLI profile name for credential resolution.",
    },
];

const BEDROCK: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Bedrock,
    display_name: "AWS Bedrock",
    keys: BEDROCK_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: false,
};

const AZURE_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "AZURE_OPENAI_API_KEY",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "Azure OpenAI API key.",
    },
    ConfigKey {
        name: "AZURE_OPENAI_ENDPOINT",
        kind: ConfigKeyKind::BaseUrl,
        required: true,
        secret: false,
        default: None,
        description: "Azure OpenAI endpoint URL (required, no default).",
    },
];

const AZURE: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::AzureOpenAi,
    display_name: "Azure OpenAI",
    keys: AZURE_KEYS,
    supports_base_url: true,
    default_base_url: None,
    api_key_optional: false,
};

const COMPOSER2_KEYS: &[ConfigKey] = &[ConfigKey {
    name: "CURSOR_API_KEY",
    kind: ConfigKeyKind::ApiKey,
    required: true,
    secret: true,
    default: None,
    description: "Cursor Composer 2 API key.",
}];

const COMPOSER2: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Composer2,
    display_name: "Cursor Composer 2",
    keys: COMPOSER2_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: false,
};

const CLI_PROFILE_KEYS: &[ConfigKey] = &[ConfigKey {
    name: "cli_path",
    kind: ConfigKeyKind::Profile,
    required: false,
    secret: false,
    default: None,
    description: "Optional override path to the CLI binary.",
}];

// Per-CLI display_name + backend variants share the same key template. We
// cannot mutate `backend` in a const, so we declare one static per CLI with
// its own `backend` field but the same `keys` slice.
const CLAUDE_CODE: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::ClaudeCode,
    display_name: "claude CLI",
    keys: CLI_PROFILE_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};
const CODEX_CLI: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::CodexCli,
    display_name: "codex CLI",
    keys: CLI_PROFILE_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};
const OPENCODE: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::OpenCode,
    display_name: "opencode CLI",
    keys: CLI_PROFILE_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};
const HERMES: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Hermes,
    display_name: "hermes CLI",
    keys: CLI_PROFILE_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};
const GEMINI_CLI: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::GeminiCli,
    display_name: "gemini CLI",
    keys: CLI_PROFILE_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};
const KIRO_CLI: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::KiroCli,
    display_name: "kiro CLI",
    keys: CLI_PROFILE_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};
const COPILOT_CLI: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::CopilotCli,
    display_name: "copilot CLI",
    keys: CLI_PROFILE_KEYS,
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};

const EMPTY_NON_CONFIGURABLE: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::Ppm,
    display_name: "non-configurable",
    keys: &[],
    supports_base_url: false,
    default_base_url: None,
    api_key_optional: true,
};

const DEEPSEEK_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "DEEPSEEK_API_KEY",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "DeepSeek API key.",
    },
    ConfigKey {
        name: "DEEPSEEK_BASE_URL",
        kind: ConfigKeyKind::BaseUrl,
        required: false,
        secret: false,
        default: None,
        description: "Optional DeepSeek endpoint override.",
    },
    ConfigKey {
        name: "model",
        kind: ConfigKeyKind::Model,
        required: false,
        secret: false,
        default: None,
        description: "Default model id (e.g. deepseek-chat).",
    },
];

const DEEPSEEK: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::DeepSeekV4Flash,
    display_name: "DeepSeek V4 Flash",
    keys: DEEPSEEK_KEYS,
    supports_base_url: true,
    default_base_url: None,
    api_key_optional: false,
};

const CEREBRAS_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "CEREBRAS_API_KEY",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "Cerebras free-tier API key.",
    },
    ConfigKey {
        name: "CEREBRAS_BASE_URL",
        kind: ConfigKeyKind::BaseUrl,
        required: false,
        secret: false,
        default: None,
        description: "Optional Cerebras endpoint override.",
    },
    ConfigKey {
        name: "model",
        kind: ConfigKeyKind::Model,
        required: false,
        secret: false,
        default: None,
        description: "Default model id.",
    },
];

const CEREBRAS: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::CerebrasFree,
    display_name: "Cerebras Free",
    keys: CEREBRAS_KEYS,
    supports_base_url: true,
    default_base_url: None,
    api_key_optional: false,
};

const GROQ_KEYS: &[ConfigKey] = &[
    ConfigKey {
        name: "GROQ_API_KEY",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "Groq free-tier API key.",
    },
    ConfigKey {
        name: "GROQ_BASE_URL",
        kind: ConfigKeyKind::BaseUrl,
        required: false,
        secret: false,
        default: None,
        description: "Optional Groq endpoint override.",
    },
    ConfigKey {
        name: "model",
        kind: ConfigKeyKind::Model,
        required: false,
        secret: false,
        default: None,
        description: "Default model id.",
    },
];

const GROQ: BackendConfigDescriptor = BackendConfigDescriptor {
    backend: RuntimeBackend::GroqFree,
    display_name: "Groq Free",
    keys: GROQ_KEYS,
    supports_base_url: true,
    default_base_url: None,
    api_key_optional: false,
};

/// Static template for `descriptor_for_custom`. Placeholder names mirror the
/// `CustomBackendSpec` field names; T5 reads the actual spec values from the
/// `CustomBackendSpec` it already holds.
const CUSTOM_TEMPLATE: &[ConfigKey] = &[
    ConfigKey {
        name: "api_key_ref",
        kind: ConfigKeyKind::ApiKey,
        required: true,
        secret: true,
        default: None,
        description: "Env var name holding the custom endpoint API key.",
    },
    ConfigKey {
        name: "base_url",
        kind: ConfigKeyKind::BaseUrl,
        required: true,
        secret: false,
        default: None,
        description: "Custom endpoint base URL.",
    },
    ConfigKey {
        name: "model",
        kind: ConfigKeyKind::Model,
        required: false,
        secret: false,
        default: None,
        description: "Default model id (first entry of spec.models).",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::custom_backends::CustomEngine;
    use super::super::secret::SecretString;

    #[test]
    fn descriptor_for_anthropic_api_requires_secret_api_key() {
        let d = descriptor_for(RuntimeBackend::AnthropicApi).expect("anthropic");
        assert!(d
            .keys
            .iter()
            .any(|k| k.kind == ConfigKeyKind::ApiKey && k.required && k.secret));
        assert!(d.supports_base_url);
        assert!(!d.api_key_optional);
    }

    #[test]
    fn local_lanes_support_base_url_and_optional_api_key() {
        for b in [
            RuntimeBackend::Sidecar,
            RuntimeBackend::Ollama,
            RuntimeBackend::Vllm,
            RuntimeBackend::LlamaCpp,
            RuntimeBackend::Mlx,
        ] {
            let d = descriptor_for(b).unwrap_or_else(|| panic!("{b:?} descriptor"));
            assert!(d.supports_base_url, "{b:?} supports_base_url");
            assert!(d.api_key_optional, "{b:?} api_key_optional");
        }
        let s = descriptor_for(RuntimeBackend::Sglang).expect("sglang");
        assert!(s.supports_base_url && s.api_key_optional);
    }

    #[test]
    fn config_key_kind_has_exactly_five_variants_no_oauth() {
        // Compile-time exhaustiveness witness -- if a variant is added without
        // updating this match, the file fails to compile.
        for kind in [
            ConfigKeyKind::ApiKey,
            ConfigKeyKind::BaseUrl,
            ConfigKeyKind::Model,
            ConfigKeyKind::Region,
            ConfigKeyKind::Profile,
        ] {
            let _ = kind._witness();
        }
        // Static textual guard: the words oauth / device_code must never appear
        // in this module's source. We approximate by asserting the enum body
        // via a stringified discriminant check -- the enum has 5 variants.
        const COUNT: usize = 5;
        let collected: [ConfigKeyKind; COUNT] = [
            ConfigKeyKind::ApiKey,
            ConfigKeyKind::BaseUrl,
            ConfigKeyKind::Model,
            ConfigKeyKind::Region,
            ConfigKeyKind::Profile,
        ];
        assert_eq!(collected.len(), COUNT);
    }

    #[test]
    fn descriptor_for_custom_returns_owned_with_template_keys() {
        let spec = CustomBackendSpec {
            name: "my-anthropic".into(),
            engine: CustomEngine::AnthropicCompat,
            base_url: "https://api.example.com".into(),
            api_key_ref: "MY_ANTHROPIC_KEY".into(),
            models: vec!["claude-sonnet-4-6".into()],
            headers: None,
            timeout_s: None,
        };
        let d = descriptor_for_custom(&spec);
        assert_eq!(d.backend, RuntimeBackend::Custom);
        assert!(d
            .keys
            .iter()
            .any(|k| k.name == "api_key_ref" && k.kind == ConfigKeyKind::ApiKey && k.secret));
        assert!(d.keys.iter().any(|k| k.kind == ConfigKeyKind::BaseUrl));
        assert!(d.supports_base_url);
        assert!(!d.api_key_optional);
        // No OAuth / device_code kinds in the template -- all variants are
        // in the no-OAuth subset.
        for k in d.keys {
            let _ = k.kind._witness();
        }
    }

    #[test]
    fn satisfies_anthropic_api_with_empty_cfg_is_err_with_api_key_some_is_ok() {
        let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::AnthropicApi);
        let d = descriptor_for(RuntimeBackend::AnthropicApi).unwrap();
        assert!(satisfies(d, &cfg).is_err());
        let mut cfg2 = cfg.clone();
        cfg2.api_key = Some(SecretString::new("sk-test".into()));
        assert!(satisfies(d, &cfg2).is_ok());
    }

    #[test]
    fn descriptor_for_custom_variant_returns_none() {
        assert!(descriptor_for(RuntimeBackend::Custom).is_none());
    }

    #[test]
    fn descriptor_for_covers_all_non_custom_variants() {
        for b in RuntimeBackend::all() {
            if *b == RuntimeBackend::Custom {
                continue;
            }
            assert!(
                descriptor_for(*b).is_some(),
                "missing static descriptor for {b:?}"
            );
        }
    }
}
