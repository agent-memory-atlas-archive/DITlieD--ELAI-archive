// C-14 — Phase 0 debt strip: pricing/cost helpers moved out of `elai_core::usage`
// into the providers crate where pricing is a provider-layer concern (a provider
// charges per token, the core token counter does not). Behaviour is byte-equivalent
// to the pre-C-14 implementation; only the module path changes.
//
// External callers that previously imported these names from `elai_core` (or
// `elai_orchestrator`'s umbrella re-export) continue to resolve through the
// updated re-export chain: `elai_providers::{ModelPricing, ...}` ->
// `elai_orchestrator::{ModelPricing, ...}` -> cli/*.
//
// I-51 — pricing.rs is now a VIEW over the model_catalog. The public API is
// byte-identical to the pre-I-51 implementation (behavior-neutral transcription,
// D-018). The DEFAULT_* constants below are kept private for the `const fn`
// `default_sonnet_tier()` which cannot use runtime catalog lookups. The drift
// contract test (i51_model_catalog_drift_contract.rs) asserts these constants
// match the catalog values so any divergence is caught at test time.

use elai_core::TokenUsage;

use crate::model_catalog::ModelCatalog;

// Private fallback constants — kept for the `const fn default_sonnet_tier()`.
// Source of truth: config/models.toml [models] family="sonnet" entry.
// I-51 drift contract asserts these == catalog values.
const DEFAULT_INPUT_COST_PER_MILLION: f64 = 15.0;
const DEFAULT_OUTPUT_COST_PER_MILLION: f64 = 75.0;
const DEFAULT_CACHE_CREATION_COST_PER_MILLION: f64 = 18.75;
const DEFAULT_CACHE_READ_COST_PER_MILLION: f64 = 1.5;

/// Per-million-token pricing used for cost estimation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPricing {
    pub input_cost_per_million: f64,
    pub output_cost_per_million: f64,
    pub cache_creation_cost_per_million: f64,
    pub cache_read_cost_per_million: f64,
}

impl ModelPricing {
    #[must_use]
    pub const fn default_sonnet_tier() -> Self {
        Self {
            input_cost_per_million: DEFAULT_INPUT_COST_PER_MILLION,
            output_cost_per_million: DEFAULT_OUTPUT_COST_PER_MILLION,
            cache_creation_cost_per_million: DEFAULT_CACHE_CREATION_COST_PER_MILLION,
            cache_read_cost_per_million: DEFAULT_CACHE_READ_COST_PER_MILLION,
        }
    }
}

/// Estimated dollar cost derived from a [`TokenUsage`] sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UsageCostEstimate {
    pub input_cost_usd: f64,
    pub output_cost_usd: f64,
    pub cache_creation_cost_usd: f64,
    pub cache_read_cost_usd: f64,
}

impl UsageCostEstimate {
    #[must_use]
    pub fn total_cost_usd(self) -> f64 {
        self.input_cost_usd
            + self.output_cost_usd
            + self.cache_creation_cost_usd
            + self.cache_read_cost_usd
    }
}

/// Returns pricing metadata for a known model alias or family.
///
/// I-51: this function is a VIEW over the model catalog. It reads pricing
/// from the embedded `config/models.toml` catalog via family-match on the
/// model id string (same matching logic as before: contains "haiku"/"opus"/
/// "sonnet"). Return values are byte-identical to the pre-I-51 implementation
/// (D-018 behavior-neutral transcription). The catalog is loaded once via
/// `ModelCatalog::load_or_default()` for each call; for high-frequency
/// callers the catalog is cheap to load (no I/O, embedded string parse).
/// If the catalog fails to load (malformed embed), falls back to the
/// inline constant values to preserve behavior neutrality.
#[must_use]
pub fn pricing_for_model(model: &str) -> Option<ModelPricing> {
    let normalized = model.to_ascii_lowercase();

    // Determine the family tag from the model id (same logic as pre-I-51).
    let family = if normalized.contains("haiku") {
        "haiku"
    } else if normalized.contains("opus") {
        "opus"
    } else if normalized.contains("sonnet") {
        "sonnet"
    } else {
        return None;
    };

    // Look up the family in the catalog. On catalog load failure, fall back
    // to inline constants so behavior is preserved (R9: no panic).
    let catalog = ModelCatalog::load_or_default();
    let entries = catalog.get_by_family(family);
    if let Some(entry) = entries.first() {
        return Some(ModelPricing {
            input_cost_per_million: entry.input_cost_per_million,
            output_cost_per_million: entry.output_cost_per_million,
            cache_creation_cost_per_million: entry.cache_creation_cost_per_million,
            cache_read_cost_per_million: entry.cache_read_cost_per_million,
        });
    }

    // Fallback: catalog has no entry for this family (should not happen with
    // a well-formed embedded models.toml). Use inline constants to preserve
    // the pre-I-51 return values exactly.
    match family {
        "haiku" => Some(ModelPricing {
            input_cost_per_million: 1.0,
            output_cost_per_million: 5.0,
            cache_creation_cost_per_million: 1.25,
            cache_read_cost_per_million: 0.1,
        }),
        "opus" | "sonnet" => Some(ModelPricing {
            input_cost_per_million: DEFAULT_INPUT_COST_PER_MILLION,
            output_cost_per_million: DEFAULT_OUTPUT_COST_PER_MILLION,
            cache_creation_cost_per_million: DEFAULT_CACHE_CREATION_COST_PER_MILLION,
            cache_read_cost_per_million: DEFAULT_CACHE_READ_COST_PER_MILLION,
        }),
        _ => None,
    }
}

/// Formats a dollar-denominated value for CLI display.
#[must_use]
pub fn format_usd(amount: f64) -> String {
    format!("${amount:.4}")
}

fn cost_for_tokens(tokens: u32, usd_per_million_tokens: f64) -> f64 {
    f64::from(tokens) / 1_000_000.0 * usd_per_million_tokens
}

/// Compute a per-bucket cost estimate from a [`TokenUsage`] sample using
/// the supplied pricing. Pure function; no allocation beyond the returned
/// estimate. Behaviour matches the pre-C-14 `TokenUsage::estimate_cost_usd_with_pricing`.
#[must_use]
pub fn estimate_cost_usd_with_pricing(
    usage: TokenUsage,
    pricing: ModelPricing,
) -> UsageCostEstimate {
    UsageCostEstimate {
        input_cost_usd: cost_for_tokens(usage.input_tokens, pricing.input_cost_per_million),
        output_cost_usd: cost_for_tokens(usage.output_tokens, pricing.output_cost_per_million),
        cache_creation_cost_usd: cost_for_tokens(
            usage.cache_creation_input_tokens,
            pricing.cache_creation_cost_per_million,
        ),
        cache_read_cost_usd: cost_for_tokens(
            usage.cache_read_input_tokens,
            pricing.cache_read_cost_per_million,
        ),
    }
}

/// Convenience: estimate cost using the default sonnet-tier pricing.
#[must_use]
pub fn estimate_cost_usd(usage: TokenUsage) -> UsageCostEstimate {
    estimate_cost_usd_with_pricing(usage, ModelPricing::default_sonnet_tier())
}

/// Render the two-line cost summary used by /cost and tool-summary JSON.
/// Behaviour matches the pre-C-14 `TokenUsage::summary_lines_for_model`.
#[must_use]
pub fn summary_lines_for_model(
    usage: TokenUsage,
    label: &str,
    model: Option<&str>,
) -> Vec<String> {
    let pricing = model.and_then(pricing_for_model);
    let cost = pricing.map_or_else(
        || estimate_cost_usd(usage),
        |pricing| estimate_cost_usd_with_pricing(usage, pricing),
    );
    let model_suffix =
        model.map_or_else(String::new, |model_name| format!(" model={model_name}"));
    let pricing_suffix = if pricing.is_some() {
        ""
    } else if model.is_some() {
        " pricing=estimated-default"
    } else {
        ""
    };
    vec![
        format!(
            "{label}: total_tokens={} input={} output={} cache_write={} cache_read={} estimated_cost={}{}{}",
            usage.total_tokens(),
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_creation_input_tokens,
            usage.cache_read_input_tokens,
            format_usd(cost.total_cost_usd()),
            model_suffix,
            pricing_suffix,
        ),
        format!(
            "  cost breakdown: input={} output={} cache_write={} cache_read={}",
            format_usd(cost.input_cost_usd),
            format_usd(cost.output_cost_usd),
            format_usd(cost.cache_creation_cost_usd),
            format_usd(cost.cache_read_cost_usd),
        ),
    ]
}

/// Two-line summary using the default sonnet-tier pricing (no model hint).
#[must_use]
pub fn summary_lines(usage: TokenUsage, label: &str) -> Vec<String> {
    summary_lines_for_model(usage, label, None)
}

#[cfg(test)]
mod tests {
    use super::{
        estimate_cost_usd, estimate_cost_usd_with_pricing, format_usd, pricing_for_model,
        summary_lines_for_model, ModelPricing,
    };
    use elai_core::TokenUsage;

    #[test]
    fn computes_cost_summary_lines() {
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            cache_creation_input_tokens: 100_000,
            cache_read_input_tokens: 200_000,
        };

        let cost = estimate_cost_usd(usage);
        assert_eq!(format_usd(cost.input_cost_usd), "$15.0000");
        assert_eq!(format_usd(cost.output_cost_usd), "$37.5000");
        let lines = summary_lines_for_model(usage, "usage", Some("claude-sonnet-4-20250514"));
        assert!(lines[0].contains("estimated_cost=$54.6750"));
        assert!(lines[0].contains("model=claude-sonnet-4-20250514"));
        assert!(lines[1].contains("cache_read=$0.3000"));
    }

    #[test]
    fn supports_model_specific_pricing() {
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        let haiku = pricing_for_model("claude-haiku-4-5-20251001").expect("haiku pricing");
        let opus = pricing_for_model("claude-opus-4-6").expect("opus pricing");
        let haiku_cost = estimate_cost_usd_with_pricing(usage, haiku);
        let opus_cost = estimate_cost_usd_with_pricing(usage, opus);
        assert_eq!(format_usd(haiku_cost.total_cost_usd()), "$3.5000");
        assert_eq!(format_usd(opus_cost.total_cost_usd()), "$52.5000");
    }

    #[test]
    fn marks_unknown_model_pricing_as_fallback() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 100,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };
        let lines = summary_lines_for_model(usage, "usage", Some("custom-model"));
        assert!(lines[0].contains("pricing=estimated-default"));
    }

    // Cold-wiring (s-81344): cover pricing_for_model branches NOT exercised by
    // the cost tests above — case-insensitivity, the sonnet branch, and the
    // explicit None for non-Anthropic-family models.
    #[test]
    fn pricing_for_model_case_insensitivity_and_branches() {
        // Case-insensitive match (the .to_ascii_lowercase() path).
        assert!(pricing_for_model("CLAUDE-OPUS-4-6").is_some());
        assert!(pricing_for_model("Claude-Haiku-4-5").is_some());
        // Sonnet branch returns the default sonnet tier exactly.
        assert_eq!(
            pricing_for_model("claude-sonnet-4-6"),
            Some(ModelPricing::default_sonnet_tier())
        );
        // No haiku/opus/sonnet substring -> None.
        assert!(pricing_for_model("gpt-4o").is_none());
        assert!(pricing_for_model("").is_none());
        // Haiku is the cheapest tier (distinct from the sonnet/opus default).
        let haiku = pricing_for_model("some-haiku-alias").expect("haiku pricing");
        assert!((haiku.input_cost_per_million - 1.0).abs() < f64::EPSILON);
        assert!((haiku.cache_read_cost_per_million - 0.1).abs() < f64::EPSILON);
    }
}
