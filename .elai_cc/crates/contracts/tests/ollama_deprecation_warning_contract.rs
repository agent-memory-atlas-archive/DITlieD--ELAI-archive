//! E-26 contract: Ollama backend deprecation surface.
//!
//! Verifies:
//! 1. `dp-rt-ollama` row is present in DEPRECATED_PATTERNS_SEED.
//! 2. The row carries severity WARN (not BLOCK — adapter remains
//!    per CLAUDE.md no-removal rule).
//! 3. The row's `replacement_name` mentions both Sidecar (local) and
//!    Sglang (cloud) — the two E-26-recommended alternatives.
//! 4. The row's `reason` cites the morphllm.com 19x peak-throughput
//!    figure (source report anchor).
//! 5. `OllamaRuntime::new` constructs without panicking (the
//!    `tracing::warn!` is observable via `caplog` machinery in
//!    downstream integration tests; this contract only verifies the
//!    construction path remains live).
//! 6. `RuntimeBackend::Ollama` still resolves via the factory (the
//!    adapter is NOT deleted — CLAUDE.md no-removal rule).

use elai_providers::runtime::{
    create_runtime, resolve_backend, OllamaRuntime, RuntimeAdapter, RuntimeBackend,
    RuntimeFactoryConfig,
};
use elai_rules::schema::query_deprecated_patterns;
use elai_rules::schema::open_rules_db;
use elai_core::clock::SeededClock;
use std::path::Path;

#[test]
fn ollama_deprecation_seed_row_present() {
    let conn =
        open_rules_db(Path::new(":memory:"), &SeededClock::new()).expect("in-memory rules DB");
    let warns =
        query_deprecated_patterns(&conn, Some("WARN")).expect("query WARN deprecated patterns");
    let row = warns
        .iter()
        .find(|r| r.deprecated_name.contains("OllamaRuntime"))
        .expect("dp-rt-ollama row must be present in DEPRECATED_PATTERNS_SEED");

    assert_eq!(row.severity, "WARN", "Ollama deprecation is WARN, not BLOCK");
    assert!(
        row.replacement_name.contains("Sidecar"),
        "replacement must mention Sidecar (local tier)"
    );
    assert!(
        row.replacement_name.contains("Sglang"),
        "replacement must mention Sglang (cloud tier)"
    );
}

#[test]
fn ollama_deprecation_seed_row_cites_19x_gap() {
    let conn =
        open_rules_db(Path::new(":memory:"), &SeededClock::new()).expect("in-memory rules DB");
    // Use raw SQL query to grab the reason for the canonical row id.
    let reason: String = conn
        .query_row(
            "SELECT reason FROM deprecated_patterns WHERE pattern_id = 'dp-rt-ollama'",
            [],
            |row| row.get(0),
        )
        .expect("dp-rt-ollama row must be queryable");

    assert!(
        reason.contains("19x") || reason.contains("19 x") || reason.contains("19-x"),
        "reason must cite the morphllm.com 19x peak-throughput gap (got: {reason})"
    );
    assert!(
        reason.to_lowercase().contains("morphllm")
            || reason.to_lowercase().contains("codersera")
            || reason.to_lowercase().contains("particula"),
        "reason must cite a source report anchor (morphllm / codersera / particula)"
    );
}

#[test]
fn ollama_runtime_still_constructs_after_deprecation() {
    // CLAUDE.md no-removal rule + B-37 commit fd5c2e5 still references
    // RuntimeBackend::Ollama — the adapter MUST remain constructible.
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Ollama);
    let rt = OllamaRuntime::new(cfg);
    assert_eq!(rt.backend(), RuntimeBackend::Ollama);
}

#[test]
fn ollama_factory_create_runtime_still_works() {
    // create_runtime(Ollama) must NOT return NotImplemented — the
    // adapter is deprecated, not removed.
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Ollama);
    let adapter = create_runtime(cfg).expect("Ollama factory remains live post-deprecation");
    assert_eq!(adapter.backend(), RuntimeBackend::Ollama);
}

#[test]
fn ollama_resolves_via_canonical_name() {
    // Canonical name `ollama` still resolves (alias surface unchanged).
    assert_eq!(
        resolve_backend("ollama").expect("ollama"),
        RuntimeBackend::Ollama
    );
}
