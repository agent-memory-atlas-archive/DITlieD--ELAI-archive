//! E-29 Tier-B -- per-row cost-rollup helper.
//!
//! REUSES [`elai_providers::pricing::pricing_for_model`] (C-14 SHIPPED)
//! -- this module does NOT duplicate the per-model pricing table. The
//! SpendDrawer already shipped (s-orch-e29 Tier-A) over the
//! `get_cloud_spend` Tauri command; this module is the in-orchestrator
//! per-session rollup primitive consumed by future detail surfaces.
//!
//! cache_creation / cache_read attribution mirrors the OTel GenAI
//! semconv 1.40.0 `gen_ai.usage.cache_creation.input_tokens` /
//! `cache_read.input_tokens` attribute names, so a future
//! ssb-otel-export emit can read these rollup values verbatim.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use elai_providers::pricing::{pricing_for_model, ModelPricing};

/// One row of the per-phase / per-agent / per-tool cost rollup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricingRollupRow {
    /// Aggregation key label (phase name, agent role, or tool id).
    pub key: String,
    /// Aggregation key category: `"phase"` / `"agent"` / `"tool"`.
    pub category: String,
    /// USD cost summed over the bucket.
    pub cost_usd: f64,
    /// Total input tokens (excluding cache_read).
    pub input_tokens: u64,
    /// Total output tokens.
    pub output_tokens: u64,
    /// Cache-creation input tokens (Anthropic prompt-caching attribution).
    pub cache_creation_input_tokens: u64,
    /// Cache-read input tokens.
    pub cache_read_input_tokens: u64,
}

/// One usage record fed to the rollup. Mirrors the on-disk schema
/// emitted by per-role runtime SSB rows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageInput {
    pub model: String,
    pub phase: String,
    pub agent: String,
    pub tool: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
}

/// Compute the USD cost of a single usage record using the C-14
/// pricing table. Returns 0.0 when the model is unknown -- the caller
/// surfaces the unknown-model count as an audit warning rather than
/// extrapolating.
#[must_use]
pub fn compute_row_cost_usd(input: &UsageInput) -> f64 {
    let pricing = match pricing_for_model(&input.model) {
        Some(p) => p,
        None => return 0.0,
    };
    apply_pricing(&pricing, input)
}

/// Compute the per-phase cost rollup for a homogeneous batch of
/// [`UsageInput`] records.
///
/// Returns one [`PricingRollupRow`] per (category, key) bucket; the
/// caller chooses the category via the helper functions
/// [`rollup_cost_by_phase`] / `rollup_cost_by_agent` /
/// `rollup_cost_by_tool`. Buckets are sorted ascending by key for
/// stable PR diffs (BTreeMap iteration order).
#[must_use]
pub fn rollup_cost_by_phase(inputs: &[UsageInput]) -> Vec<PricingRollupRow> {
    rollup_by(inputs, |i| ("phase", i.phase.clone()))
}

/// Per-agent rollup. Same shape as [`rollup_cost_by_phase`].
#[must_use]
pub fn rollup_cost_by_agent(inputs: &[UsageInput]) -> Vec<PricingRollupRow> {
    rollup_by(inputs, |i| ("agent", i.agent.clone()))
}

/// Per-tool rollup. Same shape as [`rollup_cost_by_phase`].
#[must_use]
pub fn rollup_cost_by_tool(inputs: &[UsageInput]) -> Vec<PricingRollupRow> {
    rollup_by(inputs, |i| ("tool", i.tool.clone()))
}

fn rollup_by(
    inputs: &[UsageInput],
    key_fn: impl Fn(&UsageInput) -> (&'static str, String),
) -> Vec<PricingRollupRow> {
    let mut acc: BTreeMap<(&'static str, String), PricingRollupRow> = BTreeMap::new();
    for input in inputs {
        let (category, key) = key_fn(input);
        let entry = acc
            .entry((category, key.clone()))
            .or_insert_with(|| PricingRollupRow {
                key: key.clone(),
                category: category.to_string(),
                cost_usd: 0.0,
                input_tokens: 0,
                output_tokens: 0,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
            });
        entry.cost_usd += compute_row_cost_usd(input);
        entry.input_tokens += input.input_tokens;
        entry.output_tokens += input.output_tokens;
        entry.cache_creation_input_tokens += input.cache_creation_input_tokens;
        entry.cache_read_input_tokens += input.cache_read_input_tokens;
    }
    acc.into_values().collect()
}

fn apply_pricing(pricing: &ModelPricing, input: &UsageInput) -> f64 {
    let per_million = 1_000_000.0_f64;
    let mut total = 0.0;
    total += (input.input_tokens as f64) / per_million * pricing.input_cost_per_million;
    total += (input.output_tokens as f64) / per_million * pricing.output_cost_per_million;
    total += (input.cache_creation_input_tokens as f64) / per_million
        * pricing.cache_creation_cost_per_million;
    total += (input.cache_read_input_tokens as f64) / per_million
        * pricing.cache_read_cost_per_million;
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_model_yields_zero_cost() {
        let input = UsageInput {
            model: "no-such-model".into(),
            phase: "p1".into(),
            agent: "a1".into(),
            tool: "t1".into(),
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };
        assert_eq!(compute_row_cost_usd(&input), 0.0);
    }

    #[test]
    fn rollup_buckets_by_phase() {
        let inputs = vec![
            UsageInput {
                phase: "phase1".into(),
                model: "no-such-model".into(),
                input_tokens: 100,
                ..UsageInput::default()
            },
            UsageInput {
                phase: "phase1".into(),
                model: "no-such-model".into(),
                output_tokens: 200,
                ..UsageInput::default()
            },
            UsageInput {
                phase: "phase2".into(),
                model: "no-such-model".into(),
                input_tokens: 50,
                ..UsageInput::default()
            },
        ];
        let rolled = rollup_cost_by_phase(&inputs);
        assert_eq!(rolled.len(), 2);
        assert!(rolled.iter().any(|r| r.key == "phase1"
            && r.input_tokens == 100
            && r.output_tokens == 200));
        assert!(rolled.iter().any(|r| r.key == "phase2" && r.input_tokens == 50));
    }
}
