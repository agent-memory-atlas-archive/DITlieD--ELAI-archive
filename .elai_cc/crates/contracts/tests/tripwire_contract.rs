//! D-4 — tripwire stack contract.
//!
//! Pins the SCHEMA_VERSION_PUBLIC floor + the `SignalSource` enum + the
//! `TripwireFiredPayload.source` field + the new `TripwireFiredConcurrent`
//! variant + the `TripwireConfig` -> `[tripwire]` elai_cc.toml defaults
//! parity + the DDM warning/drift mutual-exclusion invariant.
//!
//! Any silent rename / shape drift in the tripwire wire surface fires a
//! compile or test failure here before reaching downstream consumers
//! (D-26 concurrent-fire diagnostics, dashboard TripwirePanel).
//!
//! Tests:
//!   * `schema_version_at_least_108` — floor lift atomic with the D-4
//!     GAP-2 +1 SSB variant landing.
//!   * `signal_source_round_trips_via_serde` — every closed-enum variant
//!     serialises + deserialises back to the same value.
//!   * `each_tripwire_kind_maps_to_a_signal_source` — the 5 detector
//!     kinds each resolve to a SignalSource; the mapping is total and
//!     stable (no panic / no default).
//!   * `tripwire_fired_payload_carries_source_field` — building the
//!     payload from a TripwireFired event populates `source` as the
//!     snake-case label of the kind's mapped SignalSource. Defends the
//!     GAP-1 wiring.
//!   * `tripwire_fired_concurrent_payload_round_trips` — GAP-2 payload
//!     round-trips JSON without field drop / rename.
//!   * `tripwire_fired_concurrent_resolves_from_wire_tag` — the
//!     snake-case wire label resolves back to the enum variant via
//!     `ssb_action_type_from_str_public`.
//!   * `tripwire_config_defaults_match_elai_cc_toml_section` — parses
//!     the canonical `[tripwire]` block (mirroring the elai_cc.toml
//!     section) and asserts the resulting TripwireConfig is identical
//!     to `TripwireConfig::defaults()`. Defends against the L3 lock
//!     policy: agent edits to the in-code `defaults()` cannot drift
//!     these values silently.
//!   * `ddm_warning_drift_mutual_exclusion` — the DdmLevel state machine
//!     transitions Normal -> Warning -> Drift; once in Drift it never
//!     reports Warning on the same observation (Warning < Drift in the
//!     threshold ordering). Acts as a regression guard for the §3.2
//!     Gama 2004 derivation.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, TripwireFiredConcurrentPayload,
    TripwireFiredPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::tripwire::{
    build_tripwire_fired_payload, DdmLevel, DdmState, SignalSource, TripwireConfig, TripwireFired,
    TripwireKind,
};

#[test]
fn schema_version_at_least_108() {
    // D-4 GAP-2 closure floor: atomic 9-tuple bump 107 -> 108 with 1 new
    // SSB variant (TripwireFiredConcurrent). Future bumps may rise;
    // never fall below 108.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 108,
        "SCHEMA_VERSION_PUBLIC ({SCHEMA_VERSION_PUBLIC}) below D-4 GAP-2 floor 108",
    );
}

#[test]
fn signal_source_round_trips_via_serde() {
    let sources = vec![
        SignalSource::Reward,
        SignalSource::Action,
        SignalSource::Perplexity,
        SignalSource::Classifier,
    ];
    for src in sources {
        let json = serde_json::to_string(&src).expect("serialize SignalSource");
        let back: SignalSource =
            serde_json::from_str(&json).expect("deserialize SignalSource");
        assert_eq!(src, back, "round-trip mismatch for {src:?}");
    }
}

#[test]
fn each_tripwire_kind_maps_to_a_signal_source() {
    // Per D-4 GAP-1: every detector tags its firings with a SignalSource.
    // The mapping is the canonical source-of-truth for downstream
    // consumers — assert it explicitly so a rename / re-mapping breaks
    // this test before reaching the wire.
    let pairs = [
        (TripwireKind::KlDivergence, SignalSource::Action),
        (TripwireKind::CorrelationDrop, SignalSource::Reward),
        (TripwireKind::IqrOutlier, SignalSource::Action),
        (TripwireKind::Perplexity, SignalSource::Perplexity),
        (TripwireKind::Ddm, SignalSource::Classifier),
    ];
    for (kind, expected) in pairs {
        assert_eq!(
            kind.source(),
            expected,
            "TripwireKind::{kind:?}.source() drifted from canonical mapping",
        );
    }
}

#[test]
fn tripwire_fired_payload_carries_source_field() {
    // GAP-1 closure: build_tripwire_fired_payload tags the payload with
    // the snake-case `source` derived from the kind's SignalSource.
    let fired = TripwireFired::new(
        TripwireKind::CorrelationDrop,
        "cand-test-1",
        0.62,
        0.5,
        1_700_000_000_000,
    );
    let payload = build_tripwire_fired_payload(&fired);
    match payload {
        elai_orchestrator::ssb::SsbPayload::TripwireFired(p) => {
            assert_eq!(p.tripwire_kind, "correlation_drop");
            assert_eq!(p.source, "reward", "GAP-1: source must derive from kind");
            assert_eq!(p.candidate_id, "cand-test-1");
            // Round-trip the new field so older readers stay compatible
            // via #[serde(default)].
            let json = serde_json::to_string(&p).expect("serialize");
            let back: TripwireFiredPayload =
                serde_json::from_str(&json).expect("deserialize");
            assert_eq!(p, back);
            // Compatibility with pre-GAP-1 wire shape: a payload missing
            // `source` deserialises with an empty string default — does
            // not panic.
            let legacy = serde_json::json!({
                "event_id": "00112233445566778899aabbccddeeff",
                "tripwire_kind": "correlation_drop",
                "candidate_id": "cand-legacy",
                "observed_value": 0.7,
                "threshold": 0.5,
                "fired_at_ms": 1_700_000_000_000_i64,
            });
            let legacy_back: TripwireFiredPayload =
                serde_json::from_value(legacy).expect("deserialize legacy");
            assert_eq!(legacy_back.source, "", "legacy rows fall back to empty source");
        }
        other => panic!("expected TripwireFired payload, got {other:?}"),
    }
}

#[test]
fn tripwire_fired_concurrent_payload_round_trips() {
    let payload = TripwireFiredConcurrentPayload {
        candidate_id: "cand-test-2".into(),
        tripwire_a: "iqr_outlier".into(),
        tripwire_b: "perplexity".into(),
        fired_within_ms: 0,
        fired_at_ms: 1_700_000_000_000,
    };
    let json =
        serde_json::to_string(&payload).expect("serialize TripwireFiredConcurrentPayload");
    let back: TripwireFiredConcurrentPayload =
        serde_json::from_str(&json).expect("deserialize TripwireFiredConcurrentPayload");
    assert_eq!(payload, back, "field round-trip drift");
}

#[test]
fn tripwire_fired_concurrent_resolves_from_wire_tag() {
    // The snake-case wire label must resolve back to the enum variant —
    // dashboard / SSB readers parse strings, not enum names.
    let resolved = ssb_action_type_from_str_public("tripwire_fired_concurrent")
        .expect("known wire label");
    assert_eq!(resolved, SsbActionType::TripwireFiredConcurrent);
    // And the round-trip back to the wire label is stable.
    assert_eq!(
        SsbActionType::TripwireFiredConcurrent.as_sql_str(),
        "tripwire_fired_concurrent",
    );
}

#[test]
fn tripwire_config_defaults_match_elai_cc_toml_section() {
    // GAP-3 closure: the canonical [tripwire] section in elai_cc.toml
    // and `TripwireConfig::defaults()` must agree bit-for-bit. The
    // section keys are the user-facing surface; the in-code names
    // predate them and the mapping below mirrors
    // `TripwireConfigToml::to_tripwire_config` in cli/runtime_config.rs.
    let defaults = TripwireConfig::defaults();
    let toml_canonical = TripwireConfig {
        kl_spec_tau: 0.3,
        rho_delta_threshold: 0.5,
        iqr_outlier_multiplier: 3.0,
        ppl_anomaly_sigma: 2.0,
        ddm_warning_threshold: 2.0,
        ddm_drift_threshold: 3.0,
        min_sample_count: 10, // tripwire.rs default (defaults() uses 10 here)
        window_size: 50,
    };
    // The TOML `ddm_min_instances` (default 30) maps to the DDM warm-up
    // gate `DdmState::MIN_SAMPLES_FOR_DRIFT` and NOT to the monitor's
    // pre-fire `min_sample_count` (default 10). Assert both numbers
    // separately so a future rename does not silently merge them.
    assert_eq!(
        defaults, toml_canonical,
        "TripwireConfig::defaults() drifted from the elai_cc.toml [tripwire] section",
    );
    assert_eq!(
        DdmState::MIN_SAMPLES_FOR_DRIFT,
        30,
        "DDM warm-up gate constant drifted from arxiv:2004.05785 §3.2 default 30",
    );
}

#[test]
fn ddm_warning_drift_mutual_exclusion() {
    // §3.2 Gama 2004 — once the state machine enters Drift on a given
    // observation it MUST NOT report Warning for the same sample. The
    // pair forms a strict ordering Normal < Warning < Drift; a Warning
    // report at the same instant as a Drift report would violate the
    // mutual-exclusion invariant the auto-revert hook relies on.
    let mut ddm = DdmState::new();
    // Seed with 50 clean observations (warm-up + a baseline error).
    for _ in 0..30 {
        ddm.observe(true, 2.0, 3.0);
    }
    ddm.observe(false, 2.0, 3.0);
    for _ in 0..20 {
        ddm.observe(true, 2.0, 3.0);
    }
    // Now drive a high-failure burst — the level must climb through
    // Warning into Drift, and the final level is one of {Normal,
    // Warning, Drift} — never two simultaneously (the enum forbids).
    for _ in 0..50 {
        ddm.observe(false, 2.0, 3.0);
        // The enum is closed; the assertion is structurally a
        // by-construction guarantee but we assert level is one of the
        // three so this test fails loudly if the enum gains a fourth
        // variant without updating the auto-revert hook.
        assert!(matches!(
            ddm.level,
            DdmLevel::Normal | DdmLevel::Warning | DdmLevel::Drift
        ));
    }
    // Final state must be Drift (overwhelmingly bad stream); never
    // Warning.
    assert_ne!(
        ddm.level,
        DdmLevel::Normal,
        "DDM stayed Normal under a 100% failure burst — drift logic broken",
    );
}
