// A-37: VRAM-tier layer configuration types.
// Determines which reasoning layers are available based on detected GPU VRAM.

/// VRAM tier classification for hardware-specific model configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VramTier {
    Vram8Gb,
    Vram12Gb,
    Vram16Gb,
    Vram24Gb,
    Vram48Gb,
    /// Cloud tier — main compute is remote API. Not reachable via NVML detection;
    /// set via ELAI_VRAM_OVERRIDE=cloud or injected programmatically.
    Cloud,
}

/// Reasoning architecture layers (1-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasoningLayer {
    Layer1,
    Layer2,
    Layer3,
    Layer4,
    Layer5,
    Layer6,
}

/// Cloud importance relative to local compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloudImportance {
    /// Cloud is critical path (8GB tier — local capacity limited).
    MoreImportant,
    /// Cloud is enhancement only (12-48GB tiers — full local pipeline).
    Secondary,
    /// Cloud is the main compute surface (Cloud tier).
    Primary,
}

/// Offline model swap strategy for constrained VRAM tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapStrategy {
    /// Swap 3B out -> load 7B -> diagnose -> swap back (~10-15s latency).
    SwapModel,
}

/// AB-MCTS branch limits per VRAM tier (from RA design spec).
pub const MAX_MCTS_BRANCHES_8GB: usize = 5;
pub const MAX_MCTS_BRANCHES_12GB: usize = 15;
pub const MAX_MCTS_BRANCHES_16GB_PLUS: usize = 20;

/// All six reasoning layers (static — every tier enables all layers;
/// tier-specific behavior is controlled by branch limits and cloud importance).
pub static ALL_LAYERS: [ReasoningLayer; 6] = [
    ReasoningLayer::Layer1,
    ReasoningLayer::Layer2,
    ReasoningLayer::Layer3,
    ReasoningLayer::Layer4,
    ReasoningLayer::Layer5,
    ReasoningLayer::Layer6,
];

/// Per-tier runtime configuration for the reasoning engine.
/// Constructed once at startup from detected VRAM via `from_tier`.
#[derive(Debug, Clone, PartialEq)]
pub struct VramTierConfig {
    /// Raw detected VRAM in GiB (0.0 when constructed from tier alone).
    pub vram_detected_gb: f32,
    pub tier: VramTier,
    /// All tiers currently enable layers 1-6. The slice reference avoids
    /// heap allocation; future phases may introduce tier-specific subsets.
    pub available_layers: &'static [ReasoningLayer],
    pub ab_mcts_max_branches: usize,
    pub cloud_importance: CloudImportance,
    pub offline_swap_strategy: Option<SwapStrategy>,
}

impl VramTierConfig {
    /// Build the canonical layer configuration for a given VRAM tier.
    ///
    /// `vram_detected_gb` is set to 0.0 — call `with_detected_gb` after
    /// hardware detection to populate the actual value.
    ///
    /// All tiers enable layers 1-6. Branch limits and cloud importance vary:
    ///   8GB:  5 branches, MoreImportant cloud, SwapModel offline strategy
    ///   12GB: 15 branches, Secondary cloud
    ///   16GB+: 20 branches, Secondary cloud
    ///   Cloud: 20 branches, Primary cloud
    pub fn from_tier(tier: VramTier) -> Self {
        let layers: &'static [ReasoningLayer] = &ALL_LAYERS;
        match tier {
            VramTier::Vram8Gb => Self {
                vram_detected_gb: 0.0,
                tier,
                available_layers: layers,
                ab_mcts_max_branches: MAX_MCTS_BRANCHES_8GB,
                cloud_importance: CloudImportance::MoreImportant,
                offline_swap_strategy: Some(SwapStrategy::SwapModel),
            },
            VramTier::Vram12Gb => Self {
                vram_detected_gb: 0.0,
                tier,
                available_layers: layers,
                ab_mcts_max_branches: MAX_MCTS_BRANCHES_12GB,
                cloud_importance: CloudImportance::Secondary,
                offline_swap_strategy: None,
            },
            VramTier::Vram16Gb | VramTier::Vram24Gb | VramTier::Vram48Gb => Self {
                vram_detected_gb: 0.0,
                tier,
                available_layers: layers,
                ab_mcts_max_branches: MAX_MCTS_BRANCHES_16GB_PLUS,
                cloud_importance: CloudImportance::Secondary,
                offline_swap_strategy: None,
            },
            VramTier::Cloud => Self {
                vram_detected_gb: 0.0,
                tier,
                available_layers: layers,
                ab_mcts_max_branches: MAX_MCTS_BRANCHES_16GB_PLUS,
                cloud_importance: CloudImportance::Primary,
                offline_swap_strategy: None,
            },
        }
    }

    /// Set the raw detected VRAM value (GiB) after hardware detection.
    pub fn with_detected_gb(mut self, gb: f32) -> Self {
        self.vram_detected_gb = gb;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_8gb_has_swap_strategy() {
        let c = VramTierConfig::from_tier(VramTier::Vram8Gb);
        assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_8GB);
        assert_eq!(c.cloud_importance, CloudImportance::MoreImportant);
        assert_eq!(c.offline_swap_strategy, Some(SwapStrategy::SwapModel));
        assert_eq!(c.available_layers.len(), 6);
        assert_eq!(c.vram_detected_gb, 0.0);
    }

    #[test]
    fn tier_12gb_full_pipeline() {
        let c = VramTierConfig::from_tier(VramTier::Vram12Gb);
        assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_12GB);
        assert_eq!(c.cloud_importance, CloudImportance::Secondary);
        assert_eq!(c.offline_swap_strategy, None);
    }

    #[test]
    fn tier_48gb_matches_16gb_branches() {
        let c = VramTierConfig::from_tier(VramTier::Vram48Gb);
        assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_16GB_PLUS);
    }

    #[test]
    fn cloud_tier_primary_importance() {
        let c = VramTierConfig::from_tier(VramTier::Cloud);
        assert_eq!(c.cloud_importance, CloudImportance::Primary);
        assert_eq!(c.offline_swap_strategy, None);
    }

    #[test]
    fn with_detected_gb_sets_value() {
        let c = VramTierConfig::from_tier(VramTier::Vram12Gb).with_detected_gb(11.7);
        assert_eq!(c.vram_detected_gb, 11.7);
        assert_eq!(c.tier, VramTier::Vram12Gb);
    }
}
