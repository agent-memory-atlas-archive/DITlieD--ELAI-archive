// I-51 — unified model capability catalog.
//
// Single source of truth for per-model metadata: context window, pricing,
// thinking support, effort mapping, VRAM tier minimum, and staleness date.
//
// The catalog is embedded at compile time via include_str! so it is always
// present even if the config directory is not on the runtime path. The path
// `../../../config/models.toml` is relative to this source file:
//   crates/providers/src/model_catalog.rs -> crates/providers/ -> crates/ -> .elai_cc/
//   then into config/models.toml.
//
// No unwrap/expect on catalog load paths (R9). Errors are surfaced as
// ModelCatalogError; callers that need a best-effort fallback use
// ModelCatalog::load_or_default().
//
// DEPENDENCY DIRECTION: this module is in providers (which depends on core).
// core must NOT depend on providers. The vram_tier thresholds in
// providers::vram_tier are kept in-situ (SLC-2 wrap-not-move). A copy of the
// threshold values is stored in [vram_thresholds] in models.toml for drift
// assertion by the contract test only — it is not the runtime source of truth.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Embedded default catalog (compile-time constant, always present).
const EMBEDDED_CATALOG_TOML: &str = include_str!("../../../config/models.toml");

/// Error type for catalog loading and lookup.
#[derive(Debug, thiserror::Error)]
pub enum ModelCatalogError {
    #[error("failed to parse models.toml: {0}")]
    ParseError(#[from] toml::de::Error),
}

/// Per-model metadata entry, mirrors models.toml [[models]] fields.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelEntry {
    /// Canonical model identifier (e.g. "claude-sonnet-family").
    pub model_id: String,
    /// Family tag used by the pricing view's family-match logic
    /// (e.g. "sonnet", "opus", "haiku", "qwen-local").
    pub family: String,
    /// Maximum context window in tokens.
    pub context_window: u32,
    /// Default max_tokens cap.
    pub max_tokens_default: u32,
    /// USD per 1M input tokens.
    pub input_cost_per_million: f64,
    /// USD per 1M output tokens.
    pub output_cost_per_million: f64,
    /// USD per 1M cache-creation tokens.
    pub cache_creation_cost_per_million: f64,
    /// USD per 1M cache-read tokens.
    pub cache_read_cost_per_million: f64,
    /// Whether the model supports extended thinking / effort parameter.
    pub thinking_support: bool,
    /// Effort mapping string: "none" | "budget_tokens", or absent.
    pub effort_mapping: Option<String>,
    /// Minimum VramTier tier name for local inference, or absent for cloud-only.
    pub vram_tier_min: Option<String>,
    /// ISO date of last pricing/spec verification (staleness stamp).
    pub last_verified: String,
}

/// VRAM threshold values reproduced from models.toml [vram_thresholds].
/// Used by the drift contract test to assert these match the in-situ constants
/// in providers::vram_tier. NOT the runtime source of truth (SLC-2).
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogVramThresholds {
    /// Minimum GiB for Vram48Gb tier.
    pub threshold_48gb_gib: u32,
    /// Minimum GiB for Vram24Gb tier.
    pub threshold_24gb_gib: u32,
    /// Minimum GiB for Vram16Gb tier.
    pub threshold_16gb_gib: u32,
    /// Minimum GiB for Vram12Gb tier.
    pub threshold_12gb_gib: u32,
}

/// The loaded model capability catalog.
///
/// Entries are indexed by `model_id` in a BTreeMap for deterministic iteration
/// (ELAI bans HashMap; BTreeMap per pricing_rollup.rs precedent). A secondary
/// index on `family` enables the pricing view's family-match lookups.
#[derive(Debug, Clone)]
pub struct ModelCatalog {
    /// Primary index: model_id -> ModelEntry.
    by_id: BTreeMap<String, ModelEntry>,
    /// Secondary index: family -> list of model_ids (sorted for determinism).
    by_family: BTreeMap<String, Vec<String>>,
    /// VRAM thresholds from [vram_thresholds] section.
    pub vram_thresholds: CatalogVramThresholds,
}

/// Raw TOML deserialization target for models.toml.
#[derive(Debug, Deserialize)]
struct RawCatalog {
    vram_thresholds: RawVramThresholds,
    models: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct RawVramThresholds {
    threshold_48gb_gib: u32,
    threshold_24gb_gib: u32,
    threshold_16gb_gib: u32,
    threshold_12gb_gib: u32,
}

impl ModelCatalog {
    /// Parse a catalog from a TOML string.
    ///
    /// Returns `Err(ModelCatalogError::ParseError)` on invalid TOML or
    /// missing required fields. Never panics (R9).
    pub fn from_toml_str(src: &str) -> Result<Self, ModelCatalogError> {
        // toml::from_str natively skips TOML comments (the CONFLICT: blocks
        // in models.toml are comments, not data). Unknown top-level keys are
        // silently dropped because RawCatalog only declares the fields it
        // needs (serde default behavior for non-#[serde(deny_unknown_fields)]
        // structs). This tolerates future extension fields without breaking.
        let raw: RawCatalog = toml::from_str(src)?;

        let thresholds = CatalogVramThresholds {
            threshold_48gb_gib: raw.vram_thresholds.threshold_48gb_gib,
            threshold_24gb_gib: raw.vram_thresholds.threshold_24gb_gib,
            threshold_16gb_gib: raw.vram_thresholds.threshold_16gb_gib,
            threshold_12gb_gib: raw.vram_thresholds.threshold_12gb_gib,
        };

        let mut by_id: BTreeMap<String, ModelEntry> = BTreeMap::new();
        let mut by_family: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for entry in raw.models {
            by_family
                .entry(entry.family.clone())
                .or_default()
                .push(entry.model_id.clone());
            by_id.insert(entry.model_id.clone(), entry);
        }

        // Sort per-family lists for deterministic ordering.
        for ids in by_family.values_mut() {
            ids.sort();
        }

        Ok(Self {
            by_id,
            by_family,
            vram_thresholds: thresholds,
        })
    }

    /// Load the embedded default catalog. This always succeeds for a
    /// well-formed compile; if parsing somehow fails (malformed embedded TOML),
    /// returns the error for the caller to handle (no panic, R9).
    pub fn load() -> Result<Self, ModelCatalogError> {
        Self::from_toml_str(EMBEDDED_CATALOG_TOML)
    }

    /// Load the embedded catalog, returning an empty catalog on error.
    ///
    /// Intended for observational consumers that need a best-effort lookup
    /// and must not fail hard on any catalog issue. Logs a warning via tracing.
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(catalog) => catalog,
            Err(e) => {
                tracing::warn!(error = %e, "failed to load models.toml catalog; using empty catalog");
                Self {
                    by_id: BTreeMap::new(),
                    by_family: BTreeMap::new(),
                    vram_thresholds: CatalogVramThresholds {
                        threshold_48gb_gib: 0,
                        threshold_24gb_gib: 0,
                        threshold_16gb_gib: 0,
                        threshold_12gb_gib: 0,
                    },
                }
            }
        }
    }

    /// Look up an entry by exact model_id.
    #[must_use]
    pub fn get(&self, model_id: &str) -> Option<&ModelEntry> {
        self.by_id.get(model_id)
    }

    /// Look up all entries for a given family tag.
    /// Returns an empty slice when the family is not found.
    #[must_use]
    pub fn get_by_family(&self, family: &str) -> Vec<&ModelEntry> {
        match self.by_family.get(family) {
            Some(ids) => ids
                .iter()
                .filter_map(|id| self.by_id.get(id.as_str()))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Returns an iterator over all entries in model_id order.
    pub fn iter(&self) -> impl Iterator<Item = &ModelEntry> {
        self.by_id.values()
    }

    /// Total number of entries in the catalog.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// True when the catalog has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_loads_without_error() {
        let catalog = ModelCatalog::load().expect("embedded catalog must parse");
        assert!(
            !catalog.is_empty(),
            "catalog must have at least one entry"
        );
    }

    #[test]
    fn family_lookup_returns_entries() {
        let catalog = ModelCatalog::load().expect("catalog");
        let sonnet = catalog.get_by_family("sonnet");
        assert!(!sonnet.is_empty(), "sonnet family must exist");
        let haiku = catalog.get_by_family("haiku");
        assert!(!haiku.is_empty(), "haiku family must exist");
        let opus = catalog.get_by_family("opus");
        assert!(!opus.is_empty(), "opus family must exist");
    }

    #[test]
    fn sonnet_pricing_matches_pricing_rs_constants() {
        let catalog = ModelCatalog::load().expect("catalog");
        let entries = catalog.get_by_family("sonnet");
        assert_eq!(entries.len(), 1, "one sonnet family entry");
        let s = &entries[0];
        assert!((s.input_cost_per_million - 15.0).abs() < f64::EPSILON);
        assert!((s.output_cost_per_million - 75.0).abs() < f64::EPSILON);
        assert!((s.cache_creation_cost_per_million - 18.75).abs() < f64::EPSILON);
        assert!((s.cache_read_cost_per_million - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn haiku_pricing_matches_pricing_rs_constants() {
        let catalog = ModelCatalog::load().expect("catalog");
        let entries = catalog.get_by_family("haiku");
        assert_eq!(entries.len(), 1);
        let h = &entries[0];
        assert!((h.input_cost_per_million - 1.0).abs() < f64::EPSILON);
        assert!((h.output_cost_per_million - 5.0).abs() < f64::EPSILON);
        assert!((h.cache_creation_cost_per_million - 1.25).abs() < f64::EPSILON);
        assert!((h.cache_read_cost_per_million - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn vram_thresholds_present() {
        let catalog = ModelCatalog::load().expect("catalog");
        // Values must be non-zero (sanity check that the section parsed).
        assert!(catalog.vram_thresholds.threshold_48gb_gib > 0);
        assert!(catalog.vram_thresholds.threshold_24gb_gib > 0);
        assert!(catalog.vram_thresholds.threshold_16gb_gib > 0);
        assert!(catalog.vram_thresholds.threshold_12gb_gib > 0);
        // Ordering must be strictly decreasing (larger tier = higher threshold).
        assert!(
            catalog.vram_thresholds.threshold_48gb_gib
                > catalog.vram_thresholds.threshold_24gb_gib
        );
    }

    #[test]
    fn parse_error_on_bad_toml() {
        let result = ModelCatalog::from_toml_str("not valid toml [[[");
        assert!(result.is_err(), "bad TOML must return Err");
    }
}
