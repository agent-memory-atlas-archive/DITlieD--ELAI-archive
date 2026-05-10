//! B-27 — Cloud mode routing + spend accumulator + cloud_200k profile contract tests.
//!
//! Covers the new public surface introduced by B-27 cloud_mode.rs:
//!   1. Router degrades gracefully with no OAuth credentials.
//!   2. Air-gapped flag short-circuits ExecMode::Cloud before any network call.
//!   3. cloud_200k() ModelCapabilityProfile factory returns a valid profile
//!      (attention_ceiling 200k, output_reserve 8k, slot weights sum to 1.0).
//!   4. SpendAccumulator computes dual-rate per-step cost (D-4:303-305 spec).
//!
//! These tests fail-to-compile until B-27 EXECUTING step 7 lands cloud_mode.rs
//! (ExecMode, CloudConfig, CloudCredentials, CloudModeRouter, CloudModeError,
//! SpendAccumulator, named pricing constants) and step 7 also lands
//! context_budget::ModelCapabilityProfile::cloud_200k().

use elai_orchestrator::cloud_mode::{
    CloudConfig, CloudCredentials, CloudModeError, CloudModeRouter, ExecMode, SpendAccumulator,
    CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN, CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN,
    DEFAULT_CLOUD_PROVIDER,
};
use elai_orchestrator::context_budget::ModelCapabilityProfile;

#[test]
fn contract_no_credentials_degrades_to_no_credentials_error() {
    let config = CloudConfig::default();
    let router = CloudModeRouter::new(config);
    let credentials: Option<CloudCredentials> = None;

    let outcome = router.route(ExecMode::Cloud, credentials);

    assert!(
        matches!(outcome, Err(CloudModeError::NoCredentials)),
        "expected NoCredentials, got {outcome:?}"
    );
}

#[test]
fn contract_air_gapped_short_circuits_cloud_mode_before_network() {
    let mut config = CloudConfig::default();
    config.air_gapped = true;
    let router = CloudModeRouter::new(config);
    let credentials = Some(CloudCredentials::for_test_with_token("dummy-bearer-token"));

    let outcome = router.route(ExecMode::Cloud, credentials);

    assert!(
        matches!(outcome, Err(CloudModeError::AirGappedConflict)),
        "expected AirGappedConflict, got {outcome:?}"
    );
}

#[test]
fn contract_air_gapped_allows_local_mode() {
    let mut config = CloudConfig::default();
    config.air_gapped = true;
    let router = CloudModeRouter::new(config);

    let outcome = router.route(ExecMode::Local, None);

    assert!(
        outcome.is_ok(),
        "Local mode must be allowed under air_gapped=true, got {outcome:?}"
    );
}

#[test]
fn contract_cloud_200k_profile_valid() {
    let profile = ModelCapabilityProfile::cloud_200k();

    assert_eq!(profile.attention_ceiling, 200_000);
    assert_eq!(profile.output_reserve, 8_000);
    assert!(
        profile.slot_weights.validate_dynamic_sum(),
        "cloud_200k slot weights must sum to ~1.0 (got: repo_map={}, code_context={}, ssb_entries={}, grc_history={})",
        profile.slot_weights.repo_map,
        profile.slot_weights.code_context,
        profile.slot_weights.ssb_entries,
        profile.slot_weights.grc_history,
    );
}

#[test]
fn contract_spend_accumulator_dual_rate_per_step() {
    let mut acc = SpendAccumulator::for_claude_sonnet_4_6();

    acc.accumulate(1_000, 200);
    acc.accumulate(500, 100);
    let total = acc.finalize();

    let expected = (1_000.0 * CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN)
        + (200.0 * CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN)
        + (500.0 * CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN)
        + (100.0 * CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN);

    assert!(
        (total - expected).abs() < 1e-9,
        "dual-rate sum mismatch: got {total}, expected {expected} \
         (input_price={CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN}, \
         output_price={CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN})"
    );
}

#[test]
fn contract_spend_accumulator_zero_state_returns_zero() {
    let acc = SpendAccumulator::for_claude_sonnet_4_6();
    let total = acc.finalize();
    assert!(total.abs() < 1e-12, "fresh accumulator must finalize to 0.0, got {total}");
}

#[test]
fn contract_default_cloud_provider_is_anthropic() {
    assert_eq!(DEFAULT_CLOUD_PROVIDER, "anthropic");
}
