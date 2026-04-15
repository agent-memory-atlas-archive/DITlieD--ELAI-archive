// A-37: Layer routing — configures reasoning layers for a given VRAM tier.

use elai_core::{VramTier, VramTierConfig};

/// Configure the reasoning layer availability and limits for a given VRAM tier.
///
/// This is the reasoning-side entry point: given a detected tier, return the
/// canonical VramTierConfig that the reasoning engine uses to decide which
/// layers are available, AB-MCTS branch limits, and cloud importance.
///
/// Delegates to `VramTierConfig::from_tier` (the single source of truth in elai-core).
pub fn configure_layers_for_tier(tier: VramTier) -> VramTierConfig {
    VramTierConfig::from_tier(tier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::{CloudImportance, SwapStrategy, MAX_MCTS_BRANCHES_8GB, MAX_MCTS_BRANCHES_12GB};

    #[test]
    fn routing_8gb_has_swap() {
        let c = configure_layers_for_tier(VramTier::Vram8Gb);
        assert_eq!(c.offline_swap_strategy, Some(SwapStrategy::SwapModel));
        assert_eq!(c.cloud_importance, CloudImportance::MoreImportant);
        assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_8GB);
    }

    #[test]
    fn routing_12gb_full_pipeline() {
        let c = configure_layers_for_tier(VramTier::Vram12Gb);
        assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_12GB);
        assert_eq!(c.available_layers.len(), 6);
    }

    #[test]
    fn routing_cloud_primary() {
        let c = configure_layers_for_tier(VramTier::Cloud);
        assert_eq!(c.cloud_importance, CloudImportance::Primary);
    }
}
