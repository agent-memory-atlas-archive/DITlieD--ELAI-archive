// 4D.15: VRAM-tiered research configuration.
//
// Maps VramTier (from elai_core) to research-specific parameters:
// model selection, quantization, context budget, search breadth/depth,
// quality thresholds, and embedding device placement.

use elai_core::VramTier;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default speculative depth (disabled for all tiers except 48GB).
const DEFAULT_SPECULATIVE_DEPTH: usize = 0;

/// Speculative depth for the 48GB tier.
const SPECULATIVE_DEPTH_48GB: usize = 2;

// ---------------------------------------------------------------------------
// ResearchTierConfig
// ---------------------------------------------------------------------------

/// Research pipeline configuration for a given VRAM tier.
///
/// Parsed from `[research.vram_tiers.<tier>]` in elai_cc.toml,
/// or constructed via `research_config_for_tier()` defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTierConfig {
    pub model: String,
    pub quant: String,
    pub context_ceiling: usize,
    pub research_context_budget: usize,
    pub chunk_size: usize,
    pub breadth_factor: usize,
    pub depth_limit: usize,
    pub phi_min: f64,
    pub psi_min: f64,
    pub embedding_device: String,
    pub max_sources_per_node: usize,
    pub max_concurrent_fetches: usize,
    pub speculative_depth: usize,
}

/// Return the default research config for a given VRAM tier.
///
/// Values sourced from spec 4D.15 (phase-4D.txt L973-1057).
/// `detect_vram_tier()` lives in elai_providers; this function maps
/// the detected tier to research parameters.
pub fn research_config_for_tier(tier: VramTier) -> ResearchTierConfig {
    match tier {
        VramTier::Vram8Gb => ResearchTierConfig {
            model: "Qwen2.5-Coder-3B".to_string(),
            quant: "NF4".to_string(),
            context_ceiling: 16384,
            research_context_budget: 12000,
            chunk_size: 300,
            breadth_factor: 2,
            depth_limit: 3,
            phi_min: 0.8,
            psi_min: 0.7,
            embedding_device: "cpu".to_string(),
            max_sources_per_node: 5,
            max_concurrent_fetches: 2,
            speculative_depth: DEFAULT_SPECULATIVE_DEPTH,
        },
        VramTier::Vram12Gb => ResearchTierConfig {
            model: "Qwen2.5-Coder-3B".to_string(),
            quant: "NF4+FA2".to_string(),
            context_ceiling: 40000,
            research_context_budget: 30000,
            chunk_size: 500,
            breadth_factor: 3,
            depth_limit: 4,
            phi_min: 0.7,
            psi_min: 0.6,
            embedding_device: "cpu".to_string(),
            max_sources_per_node: 10,
            max_concurrent_fetches: 4,
            speculative_depth: DEFAULT_SPECULATIVE_DEPTH,
        },
        VramTier::Vram16Gb => ResearchTierConfig {
            model: "Qwen2.5-Coder-7B".to_string(),
            quant: "NF4+FA2".to_string(),
            context_ceiling: 40000,
            research_context_budget: 30000,
            chunk_size: 500,
            breadth_factor: 3,
            depth_limit: 5,
            phi_min: 0.7,
            psi_min: 0.6,
            embedding_device: "gpu".to_string(),
            max_sources_per_node: 10,
            max_concurrent_fetches: 4,
            speculative_depth: DEFAULT_SPECULATIVE_DEPTH,
        },
        VramTier::Vram24Gb => ResearchTierConfig {
            model: "Qwen2.5-Coder-14B".to_string(),
            quant: "NF4+FA2".to_string(),
            context_ceiling: 25000,
            research_context_budget: 18000,
            chunk_size: 500,
            breadth_factor: 3,
            depth_limit: 5,
            phi_min: 0.6,
            psi_min: 0.5,
            embedding_device: "gpu".to_string(),
            max_sources_per_node: 10,
            max_concurrent_fetches: 4,
            speculative_depth: DEFAULT_SPECULATIVE_DEPTH,
        },
        VramTier::Vram48Gb => ResearchTierConfig {
            model: "Qwen2.5-Coder-32B".to_string(),
            quant: "NF4+FA2".to_string(),
            context_ceiling: 24000,
            research_context_budget: 18000,
            chunk_size: 600,
            breadth_factor: 4,
            depth_limit: 5,
            phi_min: 0.6,
            psi_min: 0.5,
            embedding_device: "gpu".to_string(),
            max_sources_per_node: 15,
            max_concurrent_fetches: 6,
            speculative_depth: SPECULATIVE_DEPTH_48GB,
        },
        // Cloud tier: use same as 48GB (large model), no speculative.
        VramTier::Cloud => ResearchTierConfig {
            model: "Qwen2.5-Coder-32B".to_string(),
            quant: "NF4+FA2".to_string(),
            context_ceiling: 24000,
            research_context_budget: 18000,
            chunk_size: 600,
            breadth_factor: 4,
            depth_limit: 5,
            phi_min: 0.6,
            psi_min: 0.5,
            embedding_device: "gpu".to_string(),
            max_sources_per_node: 15,
            max_concurrent_fetches: 6,
            speculative_depth: DEFAULT_SPECULATIVE_DEPTH,
        },
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tiers_budget_under_ceiling() {
        let tiers = [
            VramTier::Vram8Gb,
            VramTier::Vram12Gb,
            VramTier::Vram16Gb,
            VramTier::Vram24Gb,
            VramTier::Vram48Gb,
            VramTier::Cloud,
        ];
        for tier in &tiers {
            let cfg = research_config_for_tier(*tier);
            assert!(
                cfg.research_context_budget < cfg.context_ceiling,
                "{:?}: budget {} >= ceiling {}",
                tier,
                cfg.research_context_budget,
                cfg.context_ceiling
            );
        }
    }

    #[test]
    fn speculative_depth_only_48gb() {
        let tiers = [
            VramTier::Vram8Gb,
            VramTier::Vram12Gb,
            VramTier::Vram16Gb,
            VramTier::Vram24Gb,
            VramTier::Cloud,
        ];
        for tier in &tiers {
            let cfg = research_config_for_tier(*tier);
            assert_eq!(
                cfg.speculative_depth, 0,
                "{:?} should have speculative_depth=0",
                tier
            );
        }
        let cfg_48 = research_config_for_tier(VramTier::Vram48Gb);
        assert!(
            cfg_48.speculative_depth > 0,
            "48GB should have speculative_depth > 0"
        );
    }

    #[test]
    fn tier_8gb_has_smallest_budget() {
        let cfg_8 = research_config_for_tier(VramTier::Vram8Gb);
        let cfg_12 = research_config_for_tier(VramTier::Vram12Gb);
        assert!(cfg_8.research_context_budget < cfg_12.research_context_budget);
    }

    #[test]
    fn tier_8gb_embedding_on_cpu() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb);
        assert_eq!(cfg.embedding_device, "cpu");
    }

    #[test]
    fn tier_16gb_embedding_on_gpu() {
        let cfg = research_config_for_tier(VramTier::Vram16Gb);
        assert_eq!(cfg.embedding_device, "gpu");
    }

    #[test]
    fn config_round_trips_through_serde() {
        let cfg = research_config_for_tier(VramTier::Vram12Gb);
        let json = serde_json::to_string(&cfg).expect("serialize");
        let parsed: ResearchTierConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.model, cfg.model);
        assert_eq!(parsed.research_context_budget, cfg.research_context_budget);
    }
}
