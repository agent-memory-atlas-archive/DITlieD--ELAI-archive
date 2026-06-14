//! I-36 — Cross-model prompt compilation contract tests (SCHEMA_VERSION 267).
//!
//! Coverage (5 arms; no #[ignore] except the regen helper, no stubs, no placeholder asserts):
//!
//! (a) `PromptCompiledPayload` rmp_serde roundtrip through a live `Ssb::write_payload`
//!     + `query_by_type` + msgpack decode, asserting every field is preserved.
//!     Catches the SQL CHECK comma gotcha: wrong sql_str -> runtime INSERT failure
//!     invisible to `cargo check`.
//!
//! (b) Schema-lock: `SCHEMA_VERSION_PUBLIC == 267` (i36 baseline).
//!
//! (c) Kill-switch observable: `CompiledPromptStore::kill_switch_active()` returns
//!     true when `ELAI_PROMPT_COMPILE_DISABLE=1` and false when unset. This is the
//!     unit contract that backs the allowlist.toml smoke_test reference.
//!
//! (d) Store round-trip: insert a `CompiledPromptVariant` to a TempDir-backed store,
//!     persist, reload from disk, look up by (role, backend), assert all fields round-trip.
//!     Catches the msgpack schema and atomic-rename path end-to-end.
//!
//! (e) CoT scaffold guard: inserting a variant whose compiled_text contains a known
//!     CoT scaffold marker (`<thinking>`) must be rejected (returns Err) and the store
//!     must contain zero entries after the failed insert.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    PromptCompiledKind, PromptCompiledPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SsbReader, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{CompiledPromptStore, CompiledPromptVariant};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ssb_config(dir: &TempDir, label: &str) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join(format!("{label}.db")),
        pool_size: Some(2),
        project_id: "i36-prompt-compile-contract".to_string(),
        session_id: format!("i36-contract-{label}"),
    }
}

fn clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn fixture_variant(role: &str, backend: &str) -> CompiledPromptVariant {
    let compiled_text = format!(
        "You are an {role} agent. Respond only with verifiable facts. \
         Use structured output. Backend: {backend}."
    );
    let hash = blake3::hash(compiled_text.as_bytes()).to_hex().to_string();
    CompiledPromptVariant {
        role: role.to_string(),
        backend: backend.to_string(),
        compiled_text: compiled_text.clone(),
        variant_hash: hash,
        ledger_pass_count: 5,
        fixture_pass_rate: Some(0.92),
        source_pass_rate: Some(0.78),
        promoted_at_ms: 1_700_000_000_000,
        last_used_at_ms: 0,
    }
}

// ---------------------------------------------------------------------------
// (a) rmp_serde roundtrip through live Ssb::write_payload
// ---------------------------------------------------------------------------

/// A real `Ssb::write_payload` + `query_by_type` + msgpack decode round-trip
/// for `PromptCompiledPayload`.  Catches the SQL CHECK comma gotcha: the
/// `as_sql_str()` value must exist in the SQLite CHECK list — a wrong wire tag
/// causes a runtime INSERT failure invisible to `cargo check`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i36_prompt_compiled_payload_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(Ssb::new(ssb_config(&tmp, "roundtrip"), clock()).expect("ssb open"));

    let variant_hash = blake3::hash(b"i36_contract_roundtrip").to_hex();
    let payload = PromptCompiledPayload {
        kind: PromptCompiledKind::Compiled,
        role: "executor".to_string(),
        backend: "anthropic_api".to_string(),
        variant_hash: variant_hash.to_string()[..16].to_string(),
        fixture_delta: 0.0,
        ts_ms: 1_700_000_000_001,
    };

    // Write via the production write_payload path.
    ssb.write_payload(
        "i36-contract-agent",
        0,
        SsbPayload::PromptCompiled(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload must succeed (SQL CHECK comma gotcha arm)");

    // Read back and decode via msgpack.
    let rows = ssb
        .query_by_type(SsbActionType::PromptCompiled(PromptCompiledKind::Compiled))
        .expect("query_by_type must succeed");
    assert_eq!(rows.len(), 1, "expected exactly one PromptCompiled row after write");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload)
        .expect("msgpack decode must succeed for PromptCompiledPayload");

    match decoded {
        SsbPayload::PromptCompiled(p) => {
            assert_eq!(p.kind, PromptCompiledKind::Compiled, "kind must round-trip");
            assert_eq!(p.role, payload.role, "role must round-trip");
            assert_eq!(p.backend, payload.backend, "backend must round-trip");
            assert_eq!(p.variant_hash, payload.variant_hash, "variant_hash must round-trip");
            assert!(
                (p.fixture_delta - payload.fixture_delta).abs() < f32::EPSILON,
                "fixture_delta must round-trip"
            );
            assert_eq!(p.ts_ms, payload.ts_ms, "ts_ms must round-trip");
        }
        other => panic!("expected PromptCompiled payload, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// (b) Schema-lock
// ---------------------------------------------------------------------------

/// I-36 floor: SCHEMA_VERSION_PUBLIC must be at least 267 (PromptCompiled present).
/// Tier-B counters are DERIVED (SCHEMA = -309 + SSB_ACTION_TYPES.len()), so this is a
/// FLOOR not an exact pin: later sub-phases legitimately add action_types and grow it.
/// Exact-count integrity is owned by action_type_lock + e45, never a per-plan assertion.
#[test]
fn i36_schema_version_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 267,
        "SCHEMA_VERSION_PUBLIC must be >= 267 once I-36 PromptCompiled is registered \
         (derived = -309 + SSB_ACTION_TYPES.len()); got {SCHEMA_VERSION_PUBLIC}. \
         A value below the floor means the PromptCompiled action_type was dropped."
    );
}

// ---------------------------------------------------------------------------
// (c) Kill-switch observable
// ---------------------------------------------------------------------------

/// `CompiledPromptStore::kill_switch_active()` must return true when
/// `ELAI_PROMPT_COMPILE_DISABLE=1` and false when unset.
///
/// This is the backing proof for the allowlist.toml smoke_test field:
/// `prompt_artifacts.rs::tests::kill_switch_suppresses_lookup`.
#[test]
fn i36_kill_switch_active_when_env_set() {
    let key = "ELAI_PROMPT_COMPILE_DISABLE";
    // Ensure unset -> false first.
    std::env::remove_var(key);
    assert!(
        !CompiledPromptStore::kill_switch_active(),
        "kill_switch_active must be false when ELAI_PROMPT_COMPILE_DISABLE is unset"
    );
    // Set to "1" -> true.
    std::env::set_var(key, "1");
    assert!(
        CompiledPromptStore::kill_switch_active(),
        "kill_switch_active must be true when ELAI_PROMPT_COMPILE_DISABLE=1"
    );
    // Set to "0" -> false.
    std::env::set_var(key, "0");
    assert!(
        !CompiledPromptStore::kill_switch_active(),
        "kill_switch_active must be false when ELAI_PROMPT_COMPILE_DISABLE=0"
    );
    // Restore.
    std::env::remove_var(key);
}

// ---------------------------------------------------------------------------
// (d) Store round-trip (TempDir-backed disk persistence)
// ---------------------------------------------------------------------------

/// `CompiledPromptStore::insert` + `persist_to_disk` + `load_from_disk` + `lookup`
/// must preserve all `CompiledPromptVariant` fields end-to-end.
///
/// This exercises the msgpack / rmp_serde codec AND the atomic tempfile+rename
/// path in a single test arm (load-on-corrupt=empty is tested in unit tests in
/// prompt_artifacts.rs; this arm validates the happy path end-to-end).
#[test]
fn i36_store_roundtrip_persists_and_loads_variant() {
    let tmp = TempDir::new().expect("tempdir");
    let cache_path = CompiledPromptStore::standard_path(tmp.path());
    let variant = fixture_variant("executor", "anthropic_api");

    // Insert into a fresh in-memory store.
    let mut store = CompiledPromptStore::new(cache_path.clone());
    store.insert(variant.clone()).expect("insert must succeed for a valid variant");

    // Persist to disk.
    store
        .persist_to_disk()
        .expect("persist_to_disk must succeed to a writable TempDir");

    // Load from disk into a new store.
    let mut loaded = CompiledPromptStore::new(cache_path);
    loaded.load_from_disk();

    // Lookup by (role, backend).
    let found = loaded.lookup(&variant.role, &variant.backend);
    let found = found.expect("lookup must return Some after persist+load round-trip");

    assert_eq!(found.role, variant.role, "role must survive round-trip");
    assert_eq!(found.backend, variant.backend, "backend must survive round-trip");
    assert_eq!(found.compiled_text, variant.compiled_text, "compiled_text must survive round-trip");
    assert_eq!(found.variant_hash, variant.variant_hash, "variant_hash must survive round-trip");
    assert_eq!(found.ledger_pass_count, variant.ledger_pass_count, "ledger_pass_count must survive round-trip");
    assert_eq!(
        found.fixture_pass_rate, variant.fixture_pass_rate,
        "fixture_pass_rate must survive round-trip"
    );
    assert_eq!(
        found.source_pass_rate, variant.source_pass_rate,
        "source_pass_rate must survive round-trip"
    );
    assert_eq!(found.promoted_at_ms, variant.promoted_at_ms, "promoted_at_ms must survive round-trip");
}

// ---------------------------------------------------------------------------
// (e) CoT scaffold guard
// ---------------------------------------------------------------------------

/// Inserting a `CompiledPromptVariant` whose `compiled_text` contains a known
/// CoT scaffold marker (`<thinking>`) must be rejected (returns `Err`) and the
/// store must contain zero entries afterward (no partial write).
///
/// Deliberative-hygiene (landmine #6: Opus 4.7+ extended thinking leaks chain-
/// of-thought verbatim into a compiled string, poisoning the system prompt).
#[test]
fn i36_cot_scaffold_insertion_is_rejected() {
    let tmp = TempDir::new().expect("tempdir");
    let cache_path = CompiledPromptStore::standard_path(tmp.path());
    let mut store = CompiledPromptStore::new(cache_path);

    let cot_text = "You are an executor. <thinking>Step 1: do X. Step 2: do Y.</thinking> Respond concisely.".to_string();
    let hash = blake3::hash(cot_text.as_bytes()).to_hex().to_string();
    let bad_variant = CompiledPromptVariant {
        role: "executor".to_string(),
        backend: "anthropic_api".to_string(),
        compiled_text: cot_text,
        variant_hash: hash,
        ledger_pass_count: 5,
        fixture_pass_rate: Some(0.9),
        source_pass_rate: Some(0.75),
        promoted_at_ms: 1_700_000_000_002,
        last_used_at_ms: 0,
    };

    let result = store.insert(bad_variant);
    assert!(
        result.is_err(),
        "insert must return Err when compiled_text contains a CoT scaffold marker"
    );
    // variant_status() returns all stored variants; must be empty after rejected insert.
    assert!(
        store.variant_status().is_empty(),
        "store must remain empty after a rejected CoT-scaffold insert"
    );
}
