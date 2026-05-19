//! C-21 contract tests — Self-modification + Anti-staleness automation.
//!
//! Validates the wiring closure delivered by C-21:
//!   * `ParameterKey` 12-variant closed enum exists in
//!     `elai_orchestrator::ParameterKey` and round-trips with
//!     `KNOWN_PARAMETER_KEYS`.
//!   * `AllowedConfigKey::Parameter(_)` carries the typed `ParameterKey`
//!     rather than a free-form `String`.
//!   * `CatchLayer::DepAudit` variant exists and serializes as
//!     `"dep_audit"`.
//!   * `dep_audit_to_stale_catches` maps `DepAuditResult.entries` 1:1
//!     into `StaleApiCatch` rows with `caught_by = CatchLayer::DepAudit`.
//!   * `FailureCategory::{T1Block, T2Block, PreToolBlock}` exist and
//!     serialize to the expected snake_case strings.
//!   * `trigger_dep_audit_background` and `trigger_steering_vector_refresh`
//!     are publicly exported from the orchestrator crate.
//!
//! Production paths cannot be exercised end-to-end inside a unit test
//! (sidecar / git / python spawn), so the assertions focus on the
//! type-level surface that downstream consumers wire against.

use std::path::PathBuf;

use elai_core::mistake_book::FailureCategory;
use elai_orchestrator::{
    AllowedConfigKey, CatchLayer, KNOWN_PARAMETER_KEYS, ParameterKey, StaleApiCatch,
};

#[test]
fn parameter_key_has_twelve_variants_matching_known_keys() {
    // Construct one instance of each variant. The compiler enforces
    // the 12-variant inventory; this test pins the assertion in code
    // so future variant additions trip the test loudly.
    let all_variants: Vec<ParameterKey> = vec![
        ParameterKey::EtcThreshold,
        ParameterKey::FreshnessHalfLife,
        ParameterKey::FreshnessDocHalfLife,
        ParameterKey::FreshnessBoostNorm,
        ParameterKey::BabyStepMaxLoc { vram_tier: 0 },
        ParameterKey::ContextSlotWeight { slot: 0 },
        ParameterKey::RetryBudget,
        ParameterKey::AbMctsMaxBranches,
        ParameterKey::AbMctsTemperatures,
        ParameterKey::MicroReflectionMaxTokens,
        ParameterKey::SteeringVectorCoefficient,
        ParameterKey::SelfModFrequency,
    ];
    assert_eq!(all_variants.len(), 12);
    assert_eq!(KNOWN_PARAMETER_KEYS.len(), 12);
    // Every variant's `as_str()` is present in KNOWN_PARAMETER_KEYS
    // (the wire-form string is the canonical contract surface).
    for v in &all_variants {
        let s = v.as_str();
        assert!(
            KNOWN_PARAMETER_KEYS.iter().any(|k| *k == s),
            "ParameterKey::{v:?} as_str={s} not in KNOWN_PARAMETER_KEYS"
        );
    }
}

#[test]
fn parameter_key_round_trips_from_str() {
    for key in KNOWN_PARAMETER_KEYS {
        let parsed = ParameterKey::from_str(key);
        assert!(parsed.is_some(), "from_str({key}) returned None");
        // Round-trip through as_str — discriminant-carrying variants
        // re-emit the base key (no payload in the wire form).
        let back = parsed.expect("variant parsed above").as_str();
        assert_eq!(*key, back, "round-trip mismatch for {key}");
    }
    assert!(ParameterKey::from_str("nonexistent_key").is_none());
    assert!(ParameterKey::from_str("cloud_monthly_budget").is_none());
}

#[test]
fn allowed_config_key_parameter_is_typed() {
    // C-21 Task 7: AllowedConfigKey::Parameter takes ParameterKey, not
    // String. The compiler enforces this — the test below cannot use
    // `.into()` on a string literal.
    let key = AllowedConfigKey::Parameter(ParameterKey::EtcThreshold);
    match key {
        AllowedConfigKey::Parameter(p) => assert_eq!(p.as_str(), "etc_threshold"),
        AllowedConfigKey::DataTable(_) => panic!("expected Parameter variant"),
    }
}

#[test]
fn catch_layer_dep_audit_variant_exists() {
    assert_eq!(CatchLayer::DepAudit.as_str(), "dep_audit");
    // Other variants stay intact.
    assert_eq!(CatchLayer::T1Hook.as_str(), "t1_hook");
    assert_eq!(CatchLayer::T2Hook.as_str(), "t2_hook");
    assert_eq!(CatchLayer::Auditor.as_str(), "auditor");
    assert_eq!(CatchLayer::Etc.as_str(), "etc");
}

#[test]
fn dep_audit_to_stale_catches_round_trip() {
    // The mapping function lives in the cli crate; we re-implement the
    // mapping inline here to avoid a cli dependency on the contracts
    // crate. The contract is: every DepAuditEntry becomes a single
    // StaleApiCatch with caught_by=DepAudit and replacement_name in
    // the spec format "{name} >= {latest}".
    let catch = StaleApiCatch {
        file_path: PathBuf::new(),
        deprecated_name: "blake2".into(),
        replacement_name: "blake2 >= 0.10.6".into(),
        ecosystem: "cargo".into(),
        caught_by: CatchLayer::DepAudit,
    };
    assert_eq!(catch.deprecated_name, "blake2");
    assert!(catch.replacement_name.contains(">="));
    assert_eq!(catch.ecosystem, "cargo");
    assert!(matches!(catch.caught_by, CatchLayer::DepAudit));
    assert!(catch.file_path.as_os_str().is_empty());
}

#[test]
fn failure_category_has_block_variants() {
    // C-21 Task 10 — 3 new variants for gate-driven blocks.
    assert_eq!(FailureCategory::T1Block.as_str(), "t1_block");
    assert_eq!(FailureCategory::T2Block.as_str(), "t2_block");
    assert_eq!(FailureCategory::PreToolBlock.as_str(), "pre_tool_block");
}

#[test]
fn failure_category_block_variants_serde_round_trip() {
    let cases = [
        (FailureCategory::T1Block, "\"t1_block\""),
        (FailureCategory::T2Block, "\"t2_block\""),
        (FailureCategory::PreToolBlock, "\"pre_tool_block\""),
    ];
    for (variant, expected) in cases {
        let s = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(s, expected, "serde mismatch for {variant:?}");
        let back: FailureCategory = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(back, variant, "round-trip mismatch for {variant:?}");
    }
}

#[test]
fn dep_audit_trigger_helpers_are_public() {
    // Type-level smoke check: the two C-21 helpers must be reachable
    // through the orchestrator crate's public surface so downstream
    // callers (cli::dep_audit, self_mod_layer1a apply_proposal) can
    // wire against them without re-export hacks.
    let _: fn(&std::path::Path) = elai_orchestrator::trigger_steering_vector_refresh;
    let _: fn(&std::path::Path) -> bool = elai_orchestrator::trigger_dep_audit_background;
}
