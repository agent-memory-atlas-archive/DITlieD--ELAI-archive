//! E-51: unit tests for platform_detect module.

use elai_providers::runtime::platform_detect::{
    detect_llamacpp_ep, detect_mlx_capable, LlamaCppEp, MlxCapability,
};

#[test]
fn detect_mlx_capable_returns_a_valid_variant() {
    // On any platform, the function must return a valid variant without panic.
    let cap = detect_mlx_capable();
    // Just verify it doesn't panic and returns a known variant.
    match cap {
        MlxCapability::NotApple => {}
        MlxCapability::AppleIntel => {}
        MlxCapability::AppleSilicon { .. } => {}
    }
}

#[test]
fn detect_llamacpp_ep_returns_a_valid_variant() {
    let ep = detect_llamacpp_ep();
    match ep {
        LlamaCppEp::Vulkan => {}
        LlamaCppEp::Cuda => {}
        LlamaCppEp::Metal => {}
        LlamaCppEp::Hip => {}
        LlamaCppEp::Cpu => {}
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn on_non_macos_mlx_returns_not_apple() {
    let cap = detect_mlx_capable();
    assert_eq!(cap, MlxCapability::NotApple);
}
