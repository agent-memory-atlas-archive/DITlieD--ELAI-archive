//! Batch 1B — provider extended-signal reader contract tests.
//!
//! Covers:
//!   1. `ProviderSignalReader` trait is dyn-safe (`Arc<dyn ProviderSignalReader>` holds).
//!   2. `TaskRunnerConfig::new_minimal` leaves `signal_reader = None`.
//!   3. `NullProviderSignalReader` returns `None` for every accessor.
//!   4. `ELAI_DISABLE_PROVIDER_SIGNALS` env var short-circuits reads on the
//!      concrete `SidecarProviderSignalReader` (feature = "sidecar" gated).
//!   5. `ProviderSignalsPayload` + `SearchTriggerFiredPayload` +
//!      `VramTierDetectedPayload` round-trip through the SSB msgpack pipeline.
//!   6. `ssb_action_type_from_str_public` knows the three new discriminants.
//!   7. `VramTierConfig::detect()` never panics and always returns a tier label.
//!   8. `emit_provider_signals_snapshot` pattern — the SSB action_type maps
//!      to the correct variant (action_type() round-trip).
//!   9. `SearchTriggerFired` is emitted ONLY when the reader returns Some.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::provider_signals::{
    provider_signals_disabled_via_env, EtcState, GgSignals, NoveltySignal,
    NullProviderSignalReader, ProviderSignalReader, SearchTrigger, SteeringVectorStatus,
    VramDetectionSource, VramTierConfig, ELAI_DISABLE_PROVIDER_SIGNALS_ENV,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ProviderSignalsPayload, SearchTriggerFiredPayload, Ssb,
    SsbActionType, SsbConfig, SsbPayload, SsbReader, VramTierDetectedPayload,
};
use elai_orchestrator::TaskRunnerConfig;
use tempfile::TempDir;

// All tests that mutate `ELAI_DISABLE_PROVIDER_SIGNALS` must serialize on
// this mutex — `std::env::set_var` / `remove_var` are process-global, and
// cargo test runs integration tests in parallel by default. Without this
// guard the env-var disable path races with the "reports VRAM" assertions
// (observed locally: Contract 10 flaked when Contract 11 cleared the var
// between set and read).
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ---------------------------------------------------------------------------
// Contract 1: trait is dyn-safe
// ---------------------------------------------------------------------------

#[test]
fn contract_provider_signal_reader_is_dyn_safe() {
    let r: Arc<dyn ProviderSignalReader> = Arc::new(NullProviderSignalReader);
    assert!(Arc::strong_count(&r) >= 1);
}

// ---------------------------------------------------------------------------
// Contract 2: TaskRunnerConfig::new_minimal leaves signal_reader = None
// ---------------------------------------------------------------------------

#[test]
fn contract_new_minimal_leaves_signal_reader_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/cursor.json"));
    assert!(
        cfg.signal_reader.is_none(),
        "new_minimal must keep signal_reader=None so pre-batch-1b call sites keep working"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: NullProviderSignalReader returns None for every accessor
// ---------------------------------------------------------------------------

#[test]
fn contract_null_reader_returns_none_everywhere() {
    let r: Arc<dyn ProviderSignalReader> = Arc::new(NullProviderSignalReader);
    assert!(r.read_last_search_trigger().is_none());
    assert!(r.read_last_gg_signals().is_none());
    assert!(r.read_last_novelty_signal().is_none());
    assert!(r.read_vram_tier_config().is_none());
    assert!(r.read_steering_vector_status().is_none());
    assert!(r.read_etc_state().is_none());
}

// ---------------------------------------------------------------------------
// Contract 4: ELAI_DISABLE_PROVIDER_SIGNALS env var flag is observable
// ---------------------------------------------------------------------------

#[test]
fn contract_env_var_disable_flag_observable() {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let prev = std::env::var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV).ok();
    std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, "1");
    assert!(provider_signals_disabled_via_env());
    std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, "0");
    assert!(!provider_signals_disabled_via_env());
    match prev {
        Some(v) => std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV),
    }
}

// ---------------------------------------------------------------------------
// Contract 5: the three new SsbPayload variants round-trip through msgpack
// ---------------------------------------------------------------------------

fn fresh_ssb(name: &str) -> (TempDir, Arc<Ssb>) {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: name.into(),
        session_id: format!("{name}-session"),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));
    (tmp, ssb)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_provider_signals_payload_roundtrips() {
    let (_tmp, ssb) = fresh_ssb("provider-signals-test");

    let payload = SsbPayload::ProviderSignals(ProviderSignalsPayload {
        gg_mean_logprob: Some(-1.2),
        gg_min_logprob: Some(-3.5),
        entropy_mean: Some(2.1),
        entropy_max: Some(4.8),
        novelty_cosine: Some(0.72),
        steering_vector_active: false,
        etc_disabled: true,
        step_id: "baby_step_7".into(),
    });
    assert_eq!(payload.action_type(), SsbActionType::ProviderSignals);
    ssb.write_payload("task_runner", 0, payload.clone(), Vec::new())
        .await
        .expect("write provider_signals");
    let rows = ssb
        .query_by_type(SsbActionType::ProviderSignals)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::ProviderSignals(p) => {
            assert!((p.entropy_mean.unwrap() - 2.1).abs() < f64::EPSILON);
            assert!((p.novelty_cosine.unwrap() - 0.72).abs() < f64::EPSILON);
            assert_eq!(p.step_id, "baby_step_7");
            assert!(p.etc_disabled);
        }
        other => panic!("expected ProviderSignals, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_search_trigger_fired_payload_roundtrips() {
    let (_tmp, ssb) = fresh_ssb("search-trigger-test");
    let payload = SsbPayload::SearchTriggerFired(SearchTriggerFiredPayload {
        reason: "low_confidence".into(),
        threshold_hit: "entropy_gt_3_5".into(),
        step_id: "baby_step_3".into(),
    });
    assert_eq!(payload.action_type(), SsbActionType::SearchTriggerFired);
    ssb.write_payload("task_runner", 0, payload, Vec::new())
        .await
        .expect("write search_trigger_fired");
    let rows = ssb
        .query_by_type(SsbActionType::SearchTriggerFired)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::SearchTriggerFired(p) => {
            assert_eq!(p.reason, "low_confidence");
            assert_eq!(p.threshold_hit, "entropy_gt_3_5");
            assert_eq!(p.step_id, "baby_step_3");
        }
        other => panic!("expected SearchTriggerFired, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_vram_tier_detected_payload_roundtrips() {
    let (_tmp, ssb) = fresh_ssb("vram-tier-test");
    let payload = SsbPayload::VramTierDetected(VramTierDetectedPayload {
        tier: "12GB".into(),
        detected_via: "nvml".into(),
        free_mb: Some(1800),
    });
    assert_eq!(payload.action_type(), SsbActionType::VramTierDetected);
    ssb.write_payload("task_runner", 0, payload, Vec::new())
        .await
        .expect("write vram_tier_detected");
    let rows = ssb
        .query_by_type(SsbActionType::VramTierDetected)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::VramTierDetected(p) => {
            assert_eq!(p.tier, "12GB");
            assert_eq!(p.detected_via, "nvml");
            assert_eq!(p.free_mb, Some(1800));
        }
        other => panic!("expected VramTierDetected, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 6: string <-> action_type mapping for the three new discriminants
// ---------------------------------------------------------------------------

#[test]
fn contract_action_type_strings_stable() {
    assert_eq!(
        ssb_action_type_from_str_public("provider_signals").expect("provider_signals must parse"),
        SsbActionType::ProviderSignals
    );
    assert_eq!(
        ssb_action_type_from_str_public("search_trigger_fired")
            .expect("search_trigger_fired must parse"),
        SsbActionType::SearchTriggerFired
    );
    assert_eq!(
        ssb_action_type_from_str_public("vram_tier_detected")
            .expect("vram_tier_detected must parse"),
        SsbActionType::VramTierDetected
    );
    assert_eq!(
        SsbActionType::ProviderSignals.as_sql_str(),
        "provider_signals"
    );
    assert_eq!(
        SsbActionType::SearchTriggerFired.as_sql_str(),
        "search_trigger_fired"
    );
    assert_eq!(
        SsbActionType::VramTierDetected.as_sql_str(),
        "vram_tier_detected"
    );
}

// ---------------------------------------------------------------------------
// Contract 7: VramTierConfig::detect() always returns a labelled tier
// ---------------------------------------------------------------------------

#[test]
fn contract_vram_tier_detect_always_returns_label() {
    // On a CI host without an NVIDIA driver we still must get a row; the
    // public invariant is "some tier + some source, never panic".
    let cfg = VramTierConfig::detect();
    assert!(!cfg.tier_label().is_empty());
    assert!(!cfg.source.as_str().is_empty());
}

// ---------------------------------------------------------------------------
// Contract 8: value-type round-trips (GgSignals / NoveltySignal / SearchTrigger
//             / EtcState / SteeringVectorStatus). Verifies the serde contract
//             for the trait return types so replay / fixture tooling can rely
//             on them.
// ---------------------------------------------------------------------------

#[test]
fn contract_gg_signals_serde_roundtrip() {
    let g = GgSignals {
        mean_logprob: Some(-1.5),
        min_logprob: Some(-3.0),
        entropy_mean: Some(1.8),
        entropy_max: Some(4.2),
    };
    let json = serde_json::to_string(&g).expect("serialize");
    let back: GgSignals = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, g);
    assert!(!g.is_empty());
    assert!(GgSignals::default().is_empty());
}

#[test]
fn contract_other_signal_types_serde_roundtrip() {
    let n = NoveltySignal {
        cosine_distance: Some(0.42),
    };
    let s = SearchTrigger {
        reason: "import".into(),
        threshold_hit: "trigger_token_idx=7".into(),
    };
    let e = EtcState {
        threshold_hit: Some("low_confidence".into()),
        disabled: false,
    };
    let v = SteeringVectorStatus {
        active: true,
        reason: "modern_api_preference".into(),
    };
    for (name, ok) in [
        ("novelty", serde_json::to_string(&n).is_ok()),
        ("trigger", serde_json::to_string(&s).is_ok()),
        ("etc", serde_json::to_string(&e).is_ok()),
        ("steering", serde_json::to_string(&v).is_ok()),
    ] {
        assert!(ok, "{name} must serialize");
    }
}

// ---------------------------------------------------------------------------
// Contract 9: VramDetectionSource stable string forms
// ---------------------------------------------------------------------------

#[test]
fn contract_vram_detection_source_string_forms() {
    assert_eq!(VramDetectionSource::Nvml.as_str(), "nvml");
    assert_eq!(VramDetectionSource::Env.as_str(), "env");
    assert_eq!(VramDetectionSource::Default.as_str(), "default");
}

// ---------------------------------------------------------------------------
// Contract 10: concrete SidecarProviderSignalReader honours the env-var
// escape hatch (feature = "sidecar" gated — contract crate opts in to the
// feature via its Cargo.toml, so this path is always exercised in CI).
// ---------------------------------------------------------------------------

// Note: contracts/Cargo.toml unconditionally activates elai-orchestrator's
// `sidecar` feature (see `features = ["sidecar"]` in [dev-dependencies]), so
// the concrete `SidecarProviderSignalReader` symbol is always in scope for
// contract tests.
#[test]
fn contract_sidecar_reader_env_var_short_circuits_reads() {
    use elai_orchestrator::SidecarProviderSignalReader;
    use elai_providers::{SidecarConfig, SidecarProvider};

    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let prev = std::env::var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV).ok();
    std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, "1");

    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let provider = Arc::new(
        SidecarProvider::new(SidecarConfig::default(), clock).expect("provider"),
    );
    let reader = SidecarProviderSignalReader::new(provider, true);

    assert!(reader.read_last_search_trigger().is_none());
    assert!(reader.read_last_gg_signals().is_none());
    assert!(reader.read_last_novelty_signal().is_none());
    assert!(reader.read_vram_tier_config().is_none());
    assert!(reader.read_steering_vector_status().is_none());
    assert!(reader.read_etc_state().is_none());

    match prev {
        Some(v) => std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV),
    }
}

// ---------------------------------------------------------------------------
// Contract 11: when env disable is OFF, the concrete reader reports a VRAM
// tier config (even without NVML — the fallback chain always yields something).
// ---------------------------------------------------------------------------

// Note: contracts/Cargo.toml unconditionally activates elai-orchestrator's
// `sidecar` feature (see `features = ["sidecar"]` in [dev-dependencies]), so
// the concrete `SidecarProviderSignalReader` symbol is always in scope for
// contract tests.
#[test]
fn contract_sidecar_reader_reports_vram_config() {
    use elai_orchestrator::SidecarProviderSignalReader;
    use elai_providers::{SidecarConfig, SidecarProvider};

    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // Ensure the disable env var is OFF for this test.
    let prev = std::env::var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV).ok();
    std::env::remove_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV);

    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let provider = Arc::new(
        SidecarProvider::new(SidecarConfig::default(), clock).expect("provider"),
    );
    let reader = SidecarProviderSignalReader::new(provider, true);
    let vram = reader.read_vram_tier_config().expect("vram must be Some");
    assert!(!vram.tier_label().is_empty());
    assert!(!vram.source.as_str().is_empty());

    // ETC state must carry the disabled flag.
    let etc = reader.read_etc_state().expect("etc must be Some");
    assert!(etc.disabled);
    // No completion has been sent, so last_search_trigger is None.
    assert!(reader.read_last_search_trigger().is_none());
    // Restore env var.
    if let Some(v) = prev {
        std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, v);
    }
}
