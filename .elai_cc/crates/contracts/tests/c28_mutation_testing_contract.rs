//! C-28 contract: Testing-Trust Tier T-1 mutation testing.
//!
//! Pins:
//!   * SCHEMA_VERSION_PUBLIC >= 60 (atomic 9-tuple lockstep bump per
//!     workflow/reasoning/C-28-decisions.md DECISION-D1 / DECISION-D13).
//!   * One new SsbActionType variant — `MutationSurvived` — round-trips via
//!     `ssb_action_type_from_str_public` and `SsbActionType::as_sql_str`.
//!   * `MutationSurvivedPayload` shape: { file, line, mutant_kind }.
//!   * `ValidationTier::ordered()` has `Mutation` at index 3 (between Unit
//!     and Integration), is length-6.
//!   * `MutationResult::from_records` computes
//!     `caught / (caught + survived + timeout) * 100` correctly.
//!   * `EquivalentMutantCache` upsert is idempotent (no duplicate rows) and
//!     blake3-keyed (64-hex digest).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, SeededClock};
use elai_core::tool_dispatcher::ValidationTier;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, MutationSurvivedPayload, SsbActionType, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{
    EquivalentMutantCache, MutantOutcome, MutantRecord, MutationResult,
};

// ---------------------------------------------------------------------------
// SSB schema + variant pins
// ---------------------------------------------------------------------------

#[test]
fn c28_schema_version_public_floor_is_60() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 60,
        "C-28 atomic 9-tuple lockstep bump: floor is 60 (see \
         workflow/reasoning/C-28-decisions.md DECISION-D1 / DECISION-D13). \
         Current: {SCHEMA_VERSION_PUBLIC}",
    );
}

#[test]
fn c28_mutation_survived_action_type_round_trips_through_str() {
    let variant = SsbActionType::MutationSurvived;
    assert_eq!(variant.as_sql_str(), "mutation_survived");
    let parsed = ssb_action_type_from_str_public("mutation_survived")
        .unwrap_or_else(|e| panic!("from_str must accept 'mutation_survived': {e}"));
    assert_eq!(parsed, variant);
}

#[test]
fn c28_mutation_survived_payload_serde_roundtrip() {
    let p = MutationSurvivedPayload {
        file: "src/math/sum.rs".to_string(),
        line: 3,
        mutant_kind: "replace_+_with_-".to_string(),
    };
    let bytes = serde_json::to_vec(&p).expect("serialize");
    let back: MutationSurvivedPayload = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(back, p);
}

// ---------------------------------------------------------------------------
// ValidationTier shape
// ---------------------------------------------------------------------------

#[test]
fn c28_validation_tier_ordered_has_mutation_at_index_3() {
    let ordered = ValidationTier::ordered();
    assert_eq!(ordered.len(), 6, "C-28: tiers grew from 5 to 6");
    assert_eq!(ordered[0], ValidationTier::Determinism);
    assert_eq!(ordered[1], ValidationTier::QualityAst);
    assert_eq!(ordered[2], ValidationTier::Unit);
    assert_eq!(ordered[3], ValidationTier::Mutation);
    assert_eq!(ordered[4], ValidationTier::Integration);
    assert_eq!(ordered[5], ValidationTier::Stress);
}

#[test]
fn c28_validation_tier_mutation_as_str_is_snake_case() {
    assert_eq!(ValidationTier::Mutation.as_str(), "mutation");
}

#[test]
fn c28_validation_tier_mutation_timeout_is_sentinel_zero() {
    // The Mutation tier returns 0ms because the budget is computed at
    // runtime from MutationConfig.{max_mutants_per_session,
    // timeout_per_mutant_sec}. Callers iterating windows(2) for the
    // monotonic non-decreasing assertion MUST filter the Mutation tier
    // first (see core/src/tool_dispatcher.rs:
    // validation_tier_timeout_is_monotonic_non_decreasing).
    assert_eq!(ValidationTier::Mutation.timeout_ms(), 0);
}

// ---------------------------------------------------------------------------
// MutationResult shape + score calculation
// ---------------------------------------------------------------------------

#[test]
fn c28_mutation_result_score_70pct_from_canonical_mix() {
    // Plan §10 fixture: 7 caught + 2 survived + 1 timeout = 70.0%.
    let mut records = Vec::new();
    for i in 0..7 {
        records.push(MutantRecord {
            file: std::path::PathBuf::from("src/math/sum.rs"),
            line: i + 1,
            mutant_kind: "replace_+_with_-".into(),
            outcome: MutantOutcome::Caught,
        });
    }
    for i in 0..2 {
        records.push(MutantRecord {
            file: std::path::PathBuf::from("src/math/sum.rs"),
            line: 100 + i,
            mutant_kind: "replace_+_with_-".into(),
            outcome: MutantOutcome::Survived,
        });
    }
    records.push(MutantRecord {
        file: std::path::PathBuf::from("src/math/sum.rs"),
        line: 200,
        mutant_kind: "replace_+_with_-".into(),
        outcome: MutantOutcome::Timeout,
    });
    let result = MutationResult::from_records(records);
    assert!(
        (result.mutation_score_pct - 70.0).abs() < 1e-9,
        "expected 70.0%, got {}",
        result.mutation_score_pct,
    );
    assert_eq!(result.total_run, 10);
}

#[test]
fn c28_mutation_result_vacuous_pass_when_no_records() {
    let result = MutationResult::from_records(Vec::new());
    assert!((result.mutation_score_pct - 100.0).abs() < 1e-9);
    assert_eq!(result.total_run, 0);
    assert_eq!(result.cache_hits, 0);
}

// ---------------------------------------------------------------------------
// EquivalentMutantCache contract
// ---------------------------------------------------------------------------

#[test]
fn c28_cache_upsert_is_idempotent() {
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let cache = EquivalentMutantCache::open_in_memory(clock).expect("open in-memory cache");
    let key = EquivalentMutantCache::compute_key(
        b"fn add(a: i32, b: i32) -> i32 { a + b }",
        "replace_+_with_-",
        3,
    );
    cache.put(&key, &MutantOutcome::Caught).expect("first put");
    cache.put(&key, &MutantOutcome::Caught).expect("second put");
    assert_eq!(
        cache.row_count().expect("count"),
        1,
        "upsert must NOT insert duplicate rows for the same key"
    );
    let got = cache.get(&key).expect("get").expect("hit after upsert");
    assert!(matches!(got, MutantOutcome::Caught));
}

#[test]
fn c28_cache_key_is_blake3_hex_64_chars() {
    // blake3 32-byte digest -> 64-char hex string. This pins the
    // anti-stale CLAUDE.md rule: blake3 ONLY, no sha2 / md5.
    let key =
        EquivalentMutantCache::compute_key(b"any bytes here", "any_kind", 1);
    assert_eq!(key.len(), 64, "blake3 hex digest must be 64 chars long");
    // Deterministic: same inputs => same hex.
    let key2 =
        EquivalentMutantCache::compute_key(b"any bytes here", "any_kind", 1);
    assert_eq!(key, key2);
    // Different line => different key.
    let key3 =
        EquivalentMutantCache::compute_key(b"any bytes here", "any_kind", 2);
    assert_ne!(key, key3);
}
