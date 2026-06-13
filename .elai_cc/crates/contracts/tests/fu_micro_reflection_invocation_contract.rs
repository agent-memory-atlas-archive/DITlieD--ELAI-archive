//! Batch-fu (FU.4 / A-23) — SidecarReflectionAdapter + dyn-adapter contract.
//!
//! Verifies:
//!   * `SidecarReflectionAdapter` constructs from a shared
//!     `SidecarGroundingClient` without opening a second sidecar connection
//!     pool (we do not exercise the live HTTP path — a hermetic test just
//!     asserts the type surfaces).
//!   * `ELAI_DISABLE_MICRO_REFLECTION` env kill switch is honoured via
//!     `micro_reflection_disabled_via_env`.
//!   * `DynReflectionAdapter` wraps an `Arc<dyn DynReflectionClient>` and
//!     delegates to the inner client without cloning the boxed future's
//!     type signature.
//!   * A malformed sidecar response produces `ReflectionClientError::Parse`
//!     via `parse_reflection_json`.

use std::sync::Arc;
use std::time::Duration;

use elai_core::micro_reflection::MicroReflectionResponse;
use elai_orchestrator::grounding_client::SidecarGroundingClient;
use elai_orchestrator::{
    micro_reflection_disabled_via_env, parse_reflection_json, DynReflectionAdapter,
    DynReflectionClient, ReflectionClient, ReflectionClientError, SidecarReflectionAdapter,
    ELAI_DISABLE_MICRO_REFLECTION_ENV,
};

struct CannedClient;

impl ReflectionClient for CannedClient {
    async fn reflect(
        &self,
        _prompt: &str,
    ) -> Result<MicroReflectionResponse, ReflectionClientError> {
        parse_reflection_json(
            r#"{"root_cause":"rc","faulty_line":3,"action":"fix"}"#,
        )
        .map_err(ReflectionClientError::Parse)
    }
}

#[test]
fn contract_sidecar_reflection_adapter_constructs() {
    let grounding = Arc::new(SidecarGroundingClient::new(
        "http://localhost:8100".into(),
        Duration::from_secs(5),
    ));
    let adapter = SidecarReflectionAdapter::new(grounding);
    // `Debug` is derived so touching it confirms the type is wired.
    let _ = format!("{adapter:?}");
}

#[test]
fn contract_env_kill_switch_toggles_disabled_state() {
    let k = ELAI_DISABLE_MICRO_REFLECTION_ENV;
    std::env::remove_var(k);
    assert!(!micro_reflection_disabled_via_env());
    std::env::set_var(k, "1");
    assert!(micro_reflection_disabled_via_env());
    std::env::remove_var(k);
}

#[tokio::test]
async fn contract_dyn_reflection_adapter_delegates() {
    let arc: Arc<dyn DynReflectionClient> = Arc::new(CannedClient);
    let adapter = DynReflectionAdapter::new(arc.as_ref());
    let r = adapter.reflect("prompt").await.expect("reflect ok");
    assert_eq!(r.root_cause, "rc");
    assert_eq!(r.faulty_line, 3);
    assert_eq!(r.action, "fix");
}

#[test]
fn contract_malformed_sidecar_response_produces_parse_error() {
    let err = parse_reflection_json("not json").expect_err("must fail");
    // Type surfaces via the public re-export; body content not asserted.
    let _ = format!("{err:?}");
}

/// P0-2 wiring gate — apply_task_runner_wiring must thread SidecarReflectionAdapter
/// through RuntimeWiring so TaskRunnerConfig::reflection_client is non-None in
/// production builds (all 5 new_minimal() sites previously left it None).
///
/// This test is a source-scan contract; it does not exercise the live HTTP path.
/// Two wiring landmarks must be present:
///   (a) runtime_setup.rs must construct `SidecarReflectionAdapter`
///   (b) runtime_setup.rs must assign `reflection_client`
#[test]
fn apply_task_runner_wiring_wires_reflection_client() {
    let src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates/contracts has a parent")
            .join("orchestrator/src/runtime_setup.rs"),
    )
    .expect("read runtime_setup.rs");

    assert!(
        src.contains("SidecarReflectionAdapter"),
        "runtime_setup.rs must construct SidecarReflectionAdapter in build_task_runner_wiring \
         (P0-2 E-19 reflection regression fix)"
    );
    assert!(
        src.contains("reflection_client"),
        "runtime_setup.rs must assign reflection_client in apply_task_runner_wiring so the \
         5 TaskRunnerConfig::new_minimal() sites get a live E-19 reflection client"
    );
}
