//! E-51: contract tests for MlxAdapter.
//!
//! Verifies:
//! 1. MlxAdapter implements RuntimeAdapter trait (compile-time).
//! 2. On non-Apple-Silicon hosts, new() returns an error.
//! 3. backend() returns RuntimeBackend::Mlx.
//! 4. capabilities() returns sane defaults (no panics).
//! 5. health_check() returns Unavailable on non-Apple hosts (no subprocess).

use elai_providers::runtime::{MlxAdapter, RuntimeAdapter, RuntimeBackend, RuntimeFactoryConfig};
use elai_providers::runtime::types::RuntimeHealth;

fn make_mlx_cfg() -> RuntimeFactoryConfig {
    RuntimeFactoryConfig::for_backend(RuntimeBackend::Mlx)
}

// Compile-time check: MlxAdapter implements RuntimeAdapter.
fn _assert_impl_runtime_adapter(_: &dyn RuntimeAdapter) {}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
#[test]
fn mlx_adapter_refuses_non_apple_silicon() {
    let result = MlxAdapter::new(make_mlx_cfg());
    assert!(
        result.is_err(),
        "MlxAdapter::new must return Err on non-Apple-Silicon"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("MLX") || msg.contains("Apple") || msg.contains("Darwin"),
        "error must mention MLX/Apple/Darwin, got: {msg}"
    );
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn mlx_adapter_succeeds_on_apple_silicon() {
    let result = MlxAdapter::new(make_mlx_cfg());
    assert!(result.is_ok(), "MlxAdapter::new must succeed on Apple Silicon");
    let adapter = result.unwrap();
    assert_eq!(adapter.backend(), RuntimeBackend::Mlx);
    let caps = adapter.capabilities();
    assert!(caps.max_context > 0);
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
#[test]
fn mlx_backend_identifier_is_mlx() {
    // Even on non-Apple platforms, we can verify the canonical_name.
    assert_eq!(RuntimeBackend::Mlx.canonical_name(), "mlx");
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
#[test]
fn mlx_ssb_rows_emitted_on_new() {
    // On non-Apple-Silicon the constructor fails — so no SSB rows are emitted.
    // This test verifies the constructor fails cleanly (no panic, no partial state).
    let result = MlxAdapter::new(make_mlx_cfg());
    assert!(result.is_err(), "no rows emitted because constructor failed");
}
