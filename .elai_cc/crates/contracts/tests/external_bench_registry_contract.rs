// E-9 contract: external agent-eval benchmark suite registry surface.
//
// Verifies (atomic SCHEMA bump 164 -> 165, +1 SsbActionType variant per
// Pattern-2 17th application):
// 1. SCHEMA_VERSION_PUBLIC >= 165 (E-9 floor; BenchmarkRunObserved variant
//    landed at this bump).
// 2. SsbActionType::BenchmarkRunObserved + payload round-trip via
//    as_sql_str / ssb_action_type_from_str_public + JSON + msgpack (binds
//    the wire-format string + payload shape including the 5-arm BenchPhase
//    discriminator).
// 3. BenchPhase 5-arm closed enum {Loaded, Started, ItemEvaluated,
//    Completed, Aborted} -- the discriminator that collapses 12 plan-
//    proposed variants into 1 per Pattern-2.
// 4. external_benches::list_suites().len() == EXPECTED_SUITE_COUNT (11)
//    -- the 11 external benchmark suites are all registered.
// 5. All 11 suites have non-empty `name`, non-empty `version`, and a
//    `local_floor` <= `cloud_floor` (cloud route is always at least as
//    capable as local).
// 6. `get_suite` returns Some for every registered suite name + None for
//    unknowns.

use elai_orchestrator::external_benches::{
    get_suite, list_suites, suite_metadata, suite_names_sorted, BenchmarkSuite,
    EXPECTED_SUITE_COUNT,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, BenchPhase, BenchmarkRunObservedPayload, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};

/// Floor for E-9: head SCHEMA_VERSION at completion-drive close. Atomic +1
/// bump from E-43's 164 head (single new variant BenchmarkRunObserved; see
/// workflow/reasoning/E-9-decisions.md DECISION-1).
const E9_SCHEMA_FLOOR: i64 = 165;

#[test]
fn schema_version_includes_e9_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E9_SCHEMA_FLOOR,
        "schema must include E-9 BenchmarkRunObserved variant (got {SCHEMA_VERSION_PUBLIC}, floor {E9_SCHEMA_FLOOR})"
    );
}

#[test]
fn benchmark_run_observed_wire_tag_round_trips() {
    let tag = SsbActionType::BenchmarkRunObserved.as_sql_str();
    assert_eq!(tag, "benchmark_run_observed");
    let back = ssb_action_type_from_str_public(tag).expect("parse round-trip");
    assert_eq!(back, SsbActionType::BenchmarkRunObserved);
}

#[test]
fn benchmark_run_observed_payload_roundtrips_via_msgpack_and_json() {
    let payload = BenchmarkRunObservedPayload {
        phase: BenchPhase::Completed,
        suite_name: "swe-bench-pro".into(),
        suite_version: "2509.16941".into(),
        score: Some(0.23),
        n: Some(200),
        sota_reference: Some(0.23),
        p_value: Some(0.04),
        low_power: false,
        required_n: Some(196),
        model_effect_pp: None,
        harness_effect_pp: None,
        coverage_gap_pp: None,
        runtime_lane: "cloud_api".into(),
        harness_blake3_prefix: "e90000000000a9b1".into(),
        ts_ms: 1_700_000_165_001,
    };

    // msgpack round-trip
    let mp = rmp_serde::to_vec_named(&payload).expect("msgpack encode");
    let back: BenchmarkRunObservedPayload = rmp_serde::from_slice(&mp).expect("msgpack decode");
    assert_eq!(back, payload);

    // JSON round-trip
    let j = serde_json::to_string(&payload).expect("json encode");
    let back2: BenchmarkRunObservedPayload =
        serde_json::from_str(&j).expect("json decode");
    assert_eq!(back2, payload);

    // SsbPayload outer variant binds to the right action_type
    let outer = SsbPayload::BenchmarkRunObserved(payload);
    assert_eq!(outer.action_type(), SsbActionType::BenchmarkRunObserved);
}

#[test]
fn bench_phase_five_arms_have_distinct_wire_tags() {
    let tags: Vec<&str> = [
        BenchPhase::Loaded,
        BenchPhase::Started,
        BenchPhase::ItemEvaluated,
        BenchPhase::Completed,
        BenchPhase::Aborted,
    ]
    .iter()
    .map(|p| p.as_str())
    .collect();
    let mut sorted = tags.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(tags.len(), sorted.len(), "BenchPhase arms must be distinct");
    // Expected snake_case labels
    assert_eq!(BenchPhase::Loaded.as_str(), "loaded");
    assert_eq!(BenchPhase::Started.as_str(), "started");
    assert_eq!(BenchPhase::ItemEvaluated.as_str(), "item_evaluated");
    assert_eq!(BenchPhase::Completed.as_str(), "completed");
    assert_eq!(BenchPhase::Aborted.as_str(), "aborted");
}

#[test]
fn registry_size_matches_expected_suite_count() {
    assert_eq!(list_suites().len(), EXPECTED_SUITE_COUNT);
    assert_eq!(EXPECTED_SUITE_COUNT, 11);
}

#[test]
fn registry_carries_all_11_canonical_names() {
    let names = suite_names_sorted();
    let expected: Vec<&str> = vec![
        "bfcl-v3",
        "clawsbench-split",
        "deep-planning",
        "gaia-l1-l2",
        "livecodebench-v6",
        "math-floors",
        "refinebench",
        "swe-bench-live",
        "swe-bench-pro",
        "sycophancy-probes",
        "tau2-bench",
    ];
    let got: Vec<&str> = names.iter().map(String::as_str).collect();
    assert_eq!(got, expected);
}

#[test]
fn registry_versions_non_empty_and_floors_consistent() {
    for suite in list_suites() {
        let suite: Box<dyn BenchmarkSuite> = suite;
        assert!(
            !suite.name().is_empty(),
            "suite name must not be empty"
        );
        assert!(
            !suite.version().is_empty(),
            "suite '{}' has empty version",
            suite.name()
        );
        assert!(
            suite.local_floor() <= suite.cloud_floor(),
            "suite '{}' local floor {} > cloud floor {} (cloud must be >= local)",
            suite.name(),
            suite.local_floor(),
            suite.cloud_floor()
        );
        assert!(
            (0.0..=1.0).contains(&suite.local_floor()),
            "suite '{}' local floor {} out of [0,1]",
            suite.name(),
            suite.local_floor()
        );
        assert!(
            (0.0..=1.0).contains(&suite.cloud_floor()),
            "suite '{}' cloud floor {} out of [0,1]",
            suite.name(),
            suite.cloud_floor()
        );
    }
}

#[test]
fn get_suite_returns_some_for_every_registered_name() {
    for name in suite_names_sorted() {
        assert!(
            get_suite(&name).is_some(),
            "get_suite({name}) returned None for registered suite"
        );
    }
}

#[test]
fn get_suite_returns_none_for_unknown_name() {
    assert!(get_suite("definitely-not-a-suite").is_none());
}

#[test]
fn suite_metadata_keys_match_registry() {
    let m = suite_metadata();
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    let expected = suite_names_sorted();
    let got: Vec<String> = keys.iter().map(|s| (*s).clone()).collect();
    assert_eq!(got, expected);
}
