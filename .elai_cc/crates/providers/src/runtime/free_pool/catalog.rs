//! TOML-driven free-pool catalog loader (J-54 residual unit 1).
//!
//! Replaces the hardcoded `FREE_CATALOG` const in `dispatch.rs` with a
//! typed loader that reads `[free_pool]` data from the embedded
//! `config/models.toml`.

use serde::Deserialize;

use super::dispatch::ProviderGroup;

const EMBEDDED_CATALOG_TOML: &str = include_str!("../../../../../config/models.toml");

/// Owned catalog entry mirroring the old `CatalogRow` but with `String`
/// fields (TOML values are owned, not `&'static`).
#[derive(Debug, Clone)]
pub struct FreePoolCatalogEntry {
    pub group: ProviderGroup,
    pub model: String,
    pub provider: String,
    pub base_url: String,
    pub env_key: String,
}

/// Denylisted model IDs that must never appear in deployments.
pub static DENYLISTED_MODEL_IDS: &[&str] = &[
    "gpt-4",
    "gpt-4o",
    "claude-3-opus",
];

#[derive(Debug, Deserialize)]
struct RawFreePool {
    #[serde(default)]
    denylisted: Vec<String>,
    #[serde(default)]
    deployments: Vec<RawDeployment>,
}

#[derive(Debug, Deserialize)]
struct RawDeployment {
    group: String,
    model: String,
    provider: String,
    base_url: String,
    env_key: String,
}

#[derive(Debug, Deserialize)]
struct RawCatalog {
    #[serde(default)]
    free_pool: Option<RawFreePool>,
}

fn parse_group(s: &str) -> Option<ProviderGroup> {
    match s {
        "big" => Some(ProviderGroup::Big),
        "fast-small" => Some(ProviderGroup::FastSmall),
        "gemini-flash" => Some(ProviderGroup::GeminiFlash),
        "mistral" => Some(ProviderGroup::Mistral),
        "openrouter-free" => Some(ProviderGroup::OpenrouterFree),
        "uncensored" => Some(ProviderGroup::Uncensored),
        _ => None,
    }
}

/// Load the free-pool catalog from the embedded `config/models.toml`.
///
/// Parses the `[free_pool]` table, maps group strings to `ProviderGroup`,
/// skips unknown groups (with a warn), and filters out denylisted model IDs.
/// Returns an empty `Vec` on parse failure (logged) so the dispatcher can
/// still fall back to Pollinations keyless overflow.
pub fn load_free_pool_catalog() -> Vec<FreePoolCatalogEntry> {
    let raw: RawCatalog = match toml::from_str(EMBEDDED_CATALOG_TOML) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                target = "free_pool.catalog",
                "failed to parse embedded models.toml for free_pool: {e}"
            );
            return Vec::new();
        }
    };

    let Some(free_pool) = raw.free_pool else {
        tracing::warn!(
            target = "free_pool.catalog",
            "no [free_pool] table in models.toml, returning empty catalog"
        );
        return Vec::new();
    };

    let denylisted: Vec<&str> = free_pool
        .denylisted
        .iter()
        .map(String::as_str)
        .collect();

    let mut entries = Vec::new();
    for dep in free_pool.deployments {
        let Some(group) = parse_group(&dep.group) else {
            tracing::warn!(
                target = "free_pool.catalog",
                "unknown group '{}' in free_pool.deployments, skipping",
                dep.group
            );
            continue;
        };

        if denylisted.iter().any(|&d| d == dep.model) {
            tracing::debug!(
                target = "free_pool.catalog",
                "skipping denylisted model '{}' in free_pool.deployments",
                dep.model
            );
            continue;
        }

        entries.push(FreePoolCatalogEntry {
            group,
            model: dep.model,
            provider: dep.provider,
            base_url: dep.base_url,
            env_key: dep.env_key,
        });
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_five_deployments_from_embedded_toml() {
        let catalog = load_free_pool_catalog();
        assert!(
            catalog.len() >= 5,
            "expected at least 5 deployments, got {}",
            catalog.len()
        );
    }

    #[test]
    fn contains_groq_llama4_scout() {
        let catalog = load_free_pool_catalog();
        let has = catalog.iter().any(|e| {
            e.provider == "groq" && e.model == "llama-4-scout"
        });
        assert!(has, "groq llama-4-scout must be in catalog");
    }

    #[test]
    fn denylisted_models_filtered() {
        let catalog = load_free_pool_catalog();
        for entry in &catalog {
            assert!(
                !DENYLISTED_MODEL_IDS.iter().any(|&d| d == entry.model),
                "denylisted model {} in catalog",
                entry.model
            );
        }
    }

    #[test]
    fn unknown_group_skipped() {
        let toml_src = r#"
[free_pool]
denylisted = []

[[free_pool.deployments]]
group = "unknown-group"
model = "test-model"
provider = "test"
base_url = "https://example.com"
env_key = "TEST_KEY"

[[free_pool.deployments]]
group = "big"
model = "llama-4-scout"
provider = "groq"
base_url = "https://api.groq.com/openai/v1"
env_key = "GROQ_API_KEY"
"#;
        let raw: RawCatalog = toml::from_str(toml_src).expect("parse");
        let free_pool = raw.free_pool.expect("free_pool");
        let entries: Vec<_> = free_pool
            .deployments
            .into_iter()
            .filter_map(|dep| {
                let group = parse_group(&dep.group)?;
                Some(FreePoolCatalogEntry {
                    group,
                    model: dep.model,
                    provider: dep.provider,
                    base_url: dep.base_url,
                    env_key: dep.env_key,
                })
            })
            .collect();
        assert_eq!(entries.len(), 1, "unknown group should be skipped");
        assert_eq!(entries[0].model, "llama-4-scout");
    }
}
