// Compile-time interface assertions for A-37 VRAM-tier types.
// Tests VramTier, VramTierConfig, CloudImportance, SwapStrategy, ReasoningLayer
// from elai-core, and detect_vram_tier from elai-providers,
// and configure_layers_for_tier from elai-reasoning.

use elai_core::{
    CloudImportance, ReasoningLayer, SwapStrategy, VramTier, VramTierConfig,
    MAX_MCTS_BRANCHES_8GB, MAX_MCTS_BRANCHES_12GB, MAX_MCTS_BRANCHES_16GB_PLUS,
};

// VramTier enum: all 6 variants must exist
fn _assert_vram_tier_variants() {
    let _ = VramTier::Vram8Gb;
    let _ = VramTier::Vram12Gb;
    let _ = VramTier::Vram16Gb;
    let _ = VramTier::Vram24Gb;
    let _ = VramTier::Vram48Gb;
    let _ = VramTier::Cloud;
}

// VramTierConfig::from_tier signature
fn _assert_from_tier_signature() {
    let _: fn(VramTier) -> VramTierConfig = VramTierConfig::from_tier;
}

// VramTierConfig fields
fn _assert_vram_tier_config_fields(c: VramTierConfig) {
    let _: f32 = c.vram_detected_gb;
    let _: VramTier = c.tier;
    let _: &'static [ReasoningLayer] = c.available_layers;
    let _: usize = c.ab_mcts_max_branches;
    let _: CloudImportance = c.cloud_importance;
    let _: Option<SwapStrategy> = c.offline_swap_strategy;
}

// CloudImportance variants
fn _assert_cloud_importance_variants() {
    let _ = CloudImportance::MoreImportant;
    let _ = CloudImportance::Secondary;
    let _ = CloudImportance::Primary;
}

// SwapStrategy variants
fn _assert_swap_strategy_variants() {
    let _ = SwapStrategy::SwapModel;
}

// ReasoningLayer: Layer1..Layer6
fn _assert_reasoning_layer_variants() {
    let _ = ReasoningLayer::Layer1;
    let _ = ReasoningLayer::Layer2;
    let _ = ReasoningLayer::Layer3;
    let _ = ReasoningLayer::Layer4;
    let _ = ReasoningLayer::Layer5;
    let _ = ReasoningLayer::Layer6;
}

// VramTierConfig for 8GB: max_branches=5, cloud=MoreImportant, swap=Some(SwapModel)
#[test]
fn tier_8gb_config_values() {
    let c = VramTierConfig::from_tier(VramTier::Vram8Gb);
    assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_8GB);
    assert_eq!(c.cloud_importance, CloudImportance::MoreImportant);
    assert_eq!(c.offline_swap_strategy, Some(SwapStrategy::SwapModel));
    assert_eq!(c.available_layers.len(), 6);
    assert_eq!(c.vram_detected_gb, 0.0);
}

// VramTierConfig for 12GB: max_branches=15, cloud=Secondary, swap=None
#[test]
fn tier_12gb_config_values() {
    let c = VramTierConfig::from_tier(VramTier::Vram12Gb);
    assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_12GB);
    assert_eq!(c.cloud_importance, CloudImportance::Secondary);
    assert_eq!(c.offline_swap_strategy, None);
    assert_eq!(c.available_layers.len(), 6);
}

// VramTierConfig for 16GB: max_branches=20
#[test]
fn tier_16gb_config_values() {
    let c = VramTierConfig::from_tier(VramTier::Vram16Gb);
    assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_16GB_PLUS);
    assert_eq!(c.cloud_importance, CloudImportance::Secondary);
}

// VramTierConfig for Cloud: max_branches=20, cloud=Primary
#[test]
fn tier_cloud_config_values() {
    let c = VramTierConfig::from_tier(VramTier::Cloud);
    assert_eq!(c.ab_mcts_max_branches, MAX_MCTS_BRANCHES_16GB_PLUS);
    assert_eq!(c.cloud_importance, CloudImportance::Primary);
    assert_eq!(c.offline_swap_strategy, None);
}

// with_detected_gb builder
#[test]
fn with_detected_gb_populates_field() {
    let c = VramTierConfig::from_tier(VramTier::Vram12Gb).with_detected_gb(11.7);
    assert_eq!(c.vram_detected_gb, 11.7);
    assert_eq!(c.tier, VramTier::Vram12Gb);
}

// detect_vram_tier exists in providers and returns Result
fn _assert_detect_vram_tier_signature() {
    let _: fn() -> Result<elai_core::VramTier, elai_providers::VramDetectionError> =
        elai_providers::detect_vram_tier;
}

// configure_layers_for_tier exists in reasoning and returns VramTierConfig
fn _assert_configure_layers_signature() {
    let _: fn(VramTier) -> VramTierConfig =
        elai_reasoning::configure_layers_for_tier;
}
