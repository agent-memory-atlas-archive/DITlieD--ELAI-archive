//! E-51: contract tests for LlamaCppAdapter.
//!
//! Verifies:
//! 1. LlamaCppAdapter implements RuntimeAdapter trait (compile-time).
//! 2. new() always succeeds on all platforms.
//! 3. backend() returns RuntimeBackend::LlamaCpp.
//! 4. EP detection returns a valid variant.

use elai_providers::runtime::{
    LlamaCppAdapter, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig,
};
use elai_providers::runtime::platform_detect::{detect_llamacpp_ep, LlamaCppEp};

fn make_llamacpp_cfg() -> RuntimeFactoryConfig {
    RuntimeFactoryConfig::for_backend(RuntimeBackend::LlamaCpp)
}

// Compile-time check: LlamaCppAdapter implements RuntimeAdapter.
fn _assert_impl_runtime_adapter(_: &dyn RuntimeAdapter) {}

#[test]
fn llamacpp_adapter_new_always_succeeds() {
    // Cross-platform: new() never fails.
    let adapter = LlamaCppAdapter::new(make_llamacpp_cfg());
    assert_eq!(adapter.backend(), RuntimeBackend::LlamaCpp);
}

#[test]
fn llamacpp_adapter_capabilities_non_zero_context() {
    let adapter = LlamaCppAdapter::new(make_llamacpp_cfg());
    let caps = adapter.capabilities();
    assert!(caps.max_context > 0, "max_context must be positive");
}

#[test]
fn llamacpp_ep_detected_is_valid() {
    let ep = detect_llamacpp_ep();
    // Just verifying no panic and it's a known variant.
    match ep {
        LlamaCppEp::Vulkan | LlamaCppEp::Cuda | LlamaCppEp::Metal | LlamaCppEp::Hip | LlamaCppEp::Cpu => {}
    }
}

#[test]
fn llamacpp_backend_canonical_name() {
    assert_eq!(RuntimeBackend::LlamaCpp.canonical_name(), "llama_cpp");
}

#[test]
fn llamacpp_adapter_box_dyn_runtime_adapter() {
    let cfg = make_llamacpp_cfg();
    let adapter = LlamaCppAdapter::new(cfg);
    let _boxed: Box<dyn RuntimeAdapter> = Box::new(adapter);
    // Verifies object-safety of RuntimeAdapter with LlamaCppAdapter.
}
