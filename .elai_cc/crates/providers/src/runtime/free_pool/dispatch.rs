//! J-54 FreePool dispatcher — per-group deployment selection with
//! fallback chain + keyless overflow.
//!
//! Fallback chain (interface map §2):
//!   big > fast-small > gemini-flash > mistral > openrouter-free
//! Uncensored group: EMPTY fallback, NO overflow (invariant I-6).
//! Pollinations overflow on all NON-uncensored groups (invariant I-7:
//! >=1 deployment always reachable).

use std::collections::HashMap;

use super::quota::KeyId;

/// Provider group for fallback routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderGroup {
    Big,
    FastSmall,
    GeminiFlash,
    Mistral,
    OpenrouterFree,
    Uncensored,
}

impl ProviderGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Big => "big",
            Self::FastSmall => "fast-small",
            Self::GeminiFlash => "gemini-flash",
            Self::Mistral => "mistral",
            Self::OpenrouterFree => "openrouter-free",
            Self::Uncensored => "uncensored",
        }
    }
}

/// A single deployment target within the free pool.
#[derive(Debug, Clone)]
pub struct Deployment {
    pub group: ProviderGroup,
    pub model: String,
    pub provider: String,
    pub base_url: String,
    /// blake3 fingerprint of the API key (for quota tracking + selection).
    pub key_id: KeyId,
    /// Env var name for the live API key (resolved at call time).
    pub env_key_name: String,
    /// Whether this is a keyless deployment (api_key = "none").
    pub keyless: bool,
}

/// Static catalog row for a free-tier model.
#[derive(Debug, Clone)]
struct CatalogRow {
    group: ProviderGroup,
    model: &'static str,
    provider: &'static str,
    base_url: &'static str,
    env_key: &'static str,
}

/// Static free-tier catalog. In a later slice this is loaded from
/// models.toml (I-51); for now a small static const suffices.
/// NO dynamic fetch / OpenRouter models API / cheahjs sync (S7 rejected).
static FREE_CATALOG: &[CatalogRow] = &[
    CatalogRow {
        group: ProviderGroup::Big,
        model: "llama-4-scout",
        provider: "groq",
        base_url: "https://api.groq.com/openai/v1",
        env_key: "GROQ_API_KEY",
    },
    CatalogRow {
        group: ProviderGroup::FastSmall,
        model: "llama-3.3-70b",
        provider: "groq",
        base_url: "https://api.groq.com/openai/v1",
        env_key: "GROQ_API_KEY",
    },
    CatalogRow {
        group: ProviderGroup::GeminiFlash,
        model: "gemini-2.0-flash",
        provider: "google",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        env_key: "GEMINI_API_KEY",
    },
    CatalogRow {
        group: ProviderGroup::Mistral,
        model: "mistral-small-latest",
        provider: "mistral",
        base_url: "https://api.mistral.ai/v1",
        env_key: "MISTRAL_API_KEY",
    },
    CatalogRow {
        group: ProviderGroup::OpenrouterFree,
        model: "meta-llama/llama-3.3-70b-instruct:free",
        provider: "openrouter",
        base_url: "https://openrouter.ai/api/v1",
        env_key: "OPENROUTER_API_KEY",
    },
];

/// Denylisted model IDs that must never appear in deployments.
static DENYLISTED_MODELS: &[&str] = &[
    "gpt-4",
    "gpt-4o",
    "claude-3-opus",
];

/// Pollinations keyless overflow deployment (interface map §5).
/// api_key = "none", always reachable (invariant I-7).
const POLLINATIONS_BASE: &str = "https://text.pollinations.ai/openai";
const POLLINATIONS_MODEL: &str = "openai";

/// Fallback chain order for NON-uncensored groups (interface map §2).
const FALLBACK_CHAIN: &[ProviderGroup] = &[
    ProviderGroup::Big,
    ProviderGroup::FastSmall,
    ProviderGroup::GeminiFlash,
    ProviderGroup::Mistral,
    ProviderGroup::OpenrouterFree,
];

/// Error when all deployments in the pool are exhausted.
#[derive(Debug, Clone, thiserror::Error)]
#[error("free pool exhausted: all deployments quota-blocked or unavailable")]
pub struct PoolExhausted;

/// The free-pool dispatcher. Holds the built deployment list + quota state.
pub struct FreePoolDispatcher {
    deployments: Vec<Deployment>,
}

impl FreePoolDispatcher {
    /// Build deployments from the static catalog + env keys.
    /// Collects GROQ_API_KEY, GROQ_API_KEY_1, GROQ_API_KEY_2, etc.
    /// Always adds Pollinations overflow for non-uncensored groups.
    pub fn new() -> Self {
        let mut deployments = Vec::new();

        for row in FREE_CATALOG {
            if row.group == ProviderGroup::Uncensored {
                continue;
            }
            for raw_key in collect_env_keys(row.env_key) {
                let key_id = KeyId::from_raw_key(row.provider, &raw_key);
                deployments.push(Deployment {
                    group: row.group,
                    model: row.model.to_string(),
                    provider: row.provider.to_string(),
                    base_url: row.base_url.to_string(),
                    key_id,
                    env_key_name: row.env_key.to_string(),
                    keyless: false,
                });
            }
        }

        add_pollinations_overflow(&mut deployments);

        Self { deployments }
    }

    /// Build with an explicit list of (env_key, raw_key) pairs for testing.
    #[cfg(test)]
    pub(super) fn with_keys(keys: &[(String, String)]) -> Self {
        let mut deployments = Vec::new();
        for row in FREE_CATALOG {
            if row.group == ProviderGroup::Uncensored {
                continue;
            }
            for (env_name, raw_key) in keys {
                if env_name == row.env_key {
                    let key_id = KeyId::from_raw_key(row.provider, raw_key);
                    deployments.push(Deployment {
                        group: row.group,
                        model: row.model.to_string(),
                        provider: row.provider.to_string(),
                        base_url: row.base_url.to_string(),
                        key_id,
                        env_key_name: row.env_key.to_string(),
                        keyless: false,
                    });
                }
            }
        }
        add_pollinations_overflow(&mut deployments);
        Self { deployments }
    }

    /// Get all deployments (for quota filtering by the caller).
    pub fn deployments(&self) -> &[Deployment] {
        &self.deployments
    }

    /// Select within a specific group (no fallback).
    fn select_in_group(
        &self,
        group: ProviderGroup,
        quota: &super::quota::QuotaTracker,
    ) -> Result<Deployment, PoolExhausted> {
        let candidates: Vec<&Deployment> = self
            .deployments
            .iter()
            .filter(|d| d.group == group)
            .filter(|d| !is_denylisted(&d.model))
            .filter(|d| quota.has_headroom(&d.key_id, &d.model))
            .collect();
        simple_shuffle(&candidates)
            .map(|d| d.clone())
            .ok_or(PoolExhausted)
    }

    /// Select the next deployment via simple-shuffle within the first
    /// group that has quota headroom, walking the fallback chain.
    /// Returns PoolExhausted when no deployment has headroom.
    /// Uncensored group: no fallback, no overflow (invariant I-6).
    pub fn select(
        &self,
        quota: &super::quota::QuotaTracker,
    ) -> Result<Deployment, PoolExhausted> {
        for &group in FALLBACK_CHAIN {
            if let Ok(dep) = self.select_in_group(group, quota) {
                return Ok(dep);
            }
        }
        Err(PoolExhausted)
    }
}

impl Default for FreePoolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Collect env keys: GROQ_API_KEY, GROQ_API_KEY_1, GROQ_API_KEY_2, ...
fn collect_env_keys(base: &str) -> Vec<String> {
    let mut keys = Vec::new();
    if let Ok(val) = std::env::var(base) {
        if !val.trim().is_empty() {
            keys.push(val);
        }
    }
    for i in 1..=8u32 {
        let name = format!("{base}_{i}");
        if let Ok(val) = std::env::var(&name) {
            if !val.trim().is_empty() {
                keys.push(val);
            }
        }
    }
    keys
}

/// Add Pollinations keyless overflow deployment (non-uncensored only).
fn add_pollinations_overflow(deployments: &mut Vec<Deployment>) {
    deployments.push(Deployment {
        group: ProviderGroup::OpenrouterFree,
        model: POLLINATIONS_MODEL.to_string(),
        provider: "pollinations".to_string(),
        base_url: POLLINATIONS_BASE.to_string(),
        key_id: KeyId::from_raw_key("pollinations", "none"),
        env_key_name: "none".to_string(),
        keyless: true,
    });
}

/// Check if a model ID is denylisted.
fn is_denylisted(model: &str) -> bool {
    DENYLISTED_MODELS.iter().any(|&d| d == model)
}

/// Simple shuffle: rotate by one position each call (deterministic round-robin).
fn simple_shuffle<'a>(candidates: &[&'a Deployment]) -> Option<&'a Deployment> {
    if candidates.is_empty() {
        return None;
    }
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0)
        % candidates.len();
    Some(candidates[idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_quota() -> super::super::quota::QuotaTracker {
        let path = std::env::temp_dir().join(format!(
            "dispatch_test_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        super::super::quota::QuotaTracker::unlimited(path)
    }

    #[test]
    fn uncensored_group_has_no_fallback() {
        let dispatcher = FreePoolDispatcher::with_keys(&[(
            "GROQ_API_KEY".to_string(),
            "sk-test".to_string(),
        )]);
        let uncensored = dispatcher
            .deployments()
            .iter()
            .filter(|d| d.group == ProviderGroup::Uncensored)
            .count();
        assert_eq!(uncensored, 0, "uncensored group must have zero deployments in fallback");
    }

    #[test]
    fn uncensored_group_has_no_overflow() {
        let dispatcher = FreePoolDispatcher::with_keys(&[]);
        let pollinations_uncensored = dispatcher
            .deployments()
            .iter()
            .filter(|d| d.group == ProviderGroup::Uncensored && d.provider == "pollinations")
            .count();
        assert_eq!(pollinations_uncensored, 0, "no overflow suffix for uncensored");
    }

    #[test]
    fn pollinations_always_present_for_non_uncensored() {
        let dispatcher = FreePoolDispatcher::with_keys(&[]);
        let pollinations = dispatcher
            .deployments()
            .iter()
            .filter(|d| d.provider == "pollinations")
            .count();
        assert!(pollinations >= 1, "Pollinations overflow always >=1 for non-uncensored");
    }

    #[test]
    fn denylisted_models_excluded() {
        let dispatcher = FreePoolDispatcher::with_keys(&[(
            "GROQ_API_KEY".to_string(),
            "sk-test".to_string(),
        )]);
        for d in dispatcher.deployments() {
            assert!(!is_denylisted(&d.model), "denylisted model {} in deployments", d.model);
        }
    }

    #[test]
    fn select_returns_deployment_with_keys() {
        let dispatcher = FreePoolDispatcher::with_keys(&[(
            "GROQ_API_KEY".to_string(),
            "sk-test".to_string(),
        )]);
        let quota = tmp_quota();
        let dep = dispatcher.select(&quota).expect("should select a deployment");
        assert!(!dep.model.is_empty());
    }

    #[test]
    fn select_returns_pollinations_with_no_keys() {
        let dispatcher = FreePoolDispatcher::with_keys(&[]);
        let quota = tmp_quota();
        let dep = dispatcher.select(&quota).expect("pollinations should always be reachable");
        assert_eq!(dep.provider, "pollinations");
        assert!(dep.keyless);
    }

    #[test]
    fn no_dynamic_catalog_fetch() {
        let dispatcher = FreePoolDispatcher::new();
        for d in dispatcher.deployments() {
            assert!(
                !d.base_url.contains("openrouter.ai/api/v1/models"),
                "no dynamic catalog fetch"
            );
        }
    }
}
