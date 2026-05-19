// C-14 — Phase 0 debt strip: pricing/cost helpers moved out of `elai_core::usage`
// into the providers crate where pricing is a provider-layer concern (a provider
// charges per token, the core token counter does not). Behaviour is byte-equivalent
// to the pre-C-14 implementation; only the module path changes.
//
// External callers that previously imported these names from `elai_core` (or
// `elai_orchestrator`'s umbrella re-export) continue to resolve through the
// updated re-export chain: `elai_providers::{ModelPricing, ...}` ->
// `elai_orchestrator::{ModelPricing, ...}` -> cli/*.

use elai_core::TokenUsage;

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
#[must_use]
pub fn pricing_for_model(model: &str) -> Option<ModelPricing> {
    let normalized = model.to_ascii_lowercase();
    if normalized.contains("haiku") {
        return Some(ModelPricing {
            input_cost_per_million: 1.0,
            output_cost_per_million: 5.0,
            cache_creation_cost_per_million: 1.25,
            cache_read_cost_per_million: 0.1,
        });
    }
    if normalized.contains("opus") {
        return Some(ModelPricing {
            input_cost_per_million: 15.0,
            output_cost_per_million: 75.0,
            cache_creation_cost_per_million: 18.75,
            cache_read_cost_per_million: 1.5,
        });
    }
    if normalized.contains("sonnet") {
        return Some(ModelPricing::default_sonnet_tier());
    }
    None
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
        summary_lines_for_model,
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
}
