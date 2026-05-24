//! C-44 — Cursor Composer 2 runtime backend contract.
//!
//! Locks the wire surface for the Composer 2 paid HTTP backend tier
//! (RuntimeAdapter impl, RuntimeBackend enum variant, cost model,
//! BackendCostAccrued SSB payload, toml `[runtime.composer2]` block).
//!
//! Grounding posture: PARTIAL paper grounding. arXiv:2603.24477
//! ("Composer 2 Technical Report", Cursor Research, 2026-03-25) is a
//! VALID cite for the sentence "Composer 2 is Cursor Research's
//! coding model" — paper-ingestor verified abs page on 2026-05-24 via
//! workflow/papers/composer2-interface.md. The paper does NOT ground
//! the integration design (endpoint, auth, pricing, request shape);
//! that surface is source-grounded against @cursor/sdk +
//! cursor.com/docs/api. This contract pins ELAI-internal wire
//! invariants, NOT paper-derived numbers — see G6 below.

use elai_orchestrator::ssb::{ssb_action_type_from_str_public, BackendCostAccruedPayload};
use elai_orchestrator::{SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC};
use elai_providers::runtime::{
    backend_name_hash, cost_model_for, create_runtime, resolve_backend, role_name_hash,
    Composer2RuntimeAdapter, RuntimeAdapter, RuntimeAdapterError, RuntimeBackend,
    RuntimeFactoryConfig, SecretString, COMPOSER2_COST,
};

// ---------------------------------------------------------------------------
// (G1) RuntimeBackend::Composer2 is reachable via canonical id + aliases,
//      and `create_runtime` returns a typed `MissingField` error when no
//      api_key is supplied (NEVER a panic, NEVER a silent fallback).
// ---------------------------------------------------------------------------

#[test]
fn c44_g1_composer2_resolves_via_canonical_and_aliases() {
    assert_eq!(
        resolve_backend("composer2").expect("canonical"),
        RuntimeBackend::Composer2
    );
    assert_eq!(
        resolve_backend("cursor").expect("alias cursor"),
        RuntimeBackend::Composer2
    );
    assert_eq!(
        resolve_backend("cursor_composer").expect("alias cursor_composer"),
        RuntimeBackend::Composer2
    );
    assert_eq!(RuntimeBackend::Composer2.canonical_name(), "composer2");
}

#[test]
fn c44_g1_create_runtime_without_api_key_surfaces_missing_field() {
    let cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Composer2);
    let result = create_runtime(cfg);
    match result {
        Err(RuntimeAdapterError::MissingField {
            backend,
            field,
            reason,
        }) => {
            assert_eq!(backend, RuntimeBackend::Composer2);
            assert_eq!(field, "api_key");
            assert!(
                reason.contains("CURSOR_API_KEY"),
                "missing-field reason must mention env var: {reason}"
            );
        }
        Err(other) => panic!("expected MissingField, got {other:?}"),
        Ok(_) => panic!("expected Err(MissingField), got Ok(adapter)"),
    }
}

// ---------------------------------------------------------------------------
// (G2) Composer2RuntimeAdapter implements the RuntimeAdapter trait with
//      the wire shape B-37 froze (object-safe, native async fn via
//      Pin<Box<dyn Future + Send>>). Capabilities match the
//      source-grounded cursor.com/docs values.
// ---------------------------------------------------------------------------

#[test]
fn c44_g2_composer2_implements_runtime_adapter_trait() {
    let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::Composer2);
    cfg.api_key = Some(SecretString::new("test-key".to_string()));
    let adapter =
        Composer2RuntimeAdapter::from_factory_config(&cfg).expect("composer2 builds with api_key");
    // Trait dispatch through a Box<dyn> proves object-safety.
    let boxed: Box<dyn RuntimeAdapter> = Box::new(adapter);
    assert_eq!(boxed.backend(), RuntimeBackend::Composer2);
    let caps = boxed.capabilities();
    assert_eq!(caps.max_context, 200_000);
    assert_eq!(caps.max_output, 8_192);
    assert!(caps.supports_tools);
    assert!(caps.supports_streaming);
    assert!(!caps.supports_vision);
}

// ---------------------------------------------------------------------------
// (G3) BackendCostAccrued SSB variant round-trips through the typed
//      payload + the wire-string lookup (ssb_action_type_from_str).
// ---------------------------------------------------------------------------

#[test]
fn c44_g3_backend_cost_accrued_action_type_round_trips() {
    // String -> enum.
    let parsed = ssb_action_type_from_str_public("backend_cost_accrued")
        .expect("backend_cost_accrued resolves");
    assert_eq!(parsed, SsbActionType::BackendCostAccrued);
    // Enum -> string.
    assert_eq!(parsed.as_sql_str(), "backend_cost_accrued");
}

#[test]
fn c44_g3_backend_cost_accrued_payload_round_trips_via_msgpack() {
    let payload = BackendCostAccruedPayload {
        backend_name_hash: 0xCAFE_BABE_DEAD_BEEF,
        role_name_hash: 0x0123_4567_89AB_CDEF,
        tokens_in: 1_024,
        tokens_out: 256,
        est_cost_micros: 1_152,
        ts_ms: 1_748_044_900_000,
    };
    let wrapped = SsbPayload::BackendCostAccrued(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    match decoded {
        SsbPayload::BackendCostAccrued(p) => assert_eq!(p, payload),
        other => panic!("expected BackendCostAccrued, got {other:?}"),
    }
}

#[test]
fn c44_g3_schema_version_floor_includes_c44_bump() {
    // C-44 bumped SCHEMA_VERSION 116 -> 117 atomically for the
    // BackendCostAccrued variant. Test asserts the floor; future plans
    // may bump further but never below this line.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 117,
        "SCHEMA_VERSION must include the C-44 BackendCostAccrued bump (>= 117), got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (G4) BackendCostModel pricing produces the expected micros for known
//      token counts. Pricing comes from cursor.com/docs/api 2026-05-24:
//      $0.50/M input, $2.50/M output. ELAI converts to micros (1e-6 USD).
// ---------------------------------------------------------------------------

#[test]
fn c44_g4_composer2_cost_model_pricing_matches_documented_rates() {
    // 1000 input tokens at $0.50/M = 500 micros.
    assert_eq!(COMPOSER2_COST.estimate_micros(1_000, 0), 500);
    // 1000 output tokens at $2.50/M = 2500 micros.
    assert_eq!(COMPOSER2_COST.estimate_micros(0, 1_000), 2_500);
    // Combined: 1000 in + 1000 out = 3000 micros total.
    assert_eq!(COMPOSER2_COST.estimate_micros(1_000, 1_000), 3_000);
}

#[test]
fn c44_g4_cost_model_for_distinguishes_paid_vs_free_backends() {
    // Composer2 is a paid backend with a registered cost model.
    let model = cost_model_for(RuntimeBackend::Composer2)
        .expect("composer2 has a registered cost model");
    assert_eq!(model.input_per_1k_micros, 500);
    assert_eq!(model.output_per_1k_micros, 2_500);
    // Local backends are free; no cost model.
    assert!(cost_model_for(RuntimeBackend::Sidecar).is_none());
    assert!(cost_model_for(RuntimeBackend::LlamaCpp).is_none());
    assert!(cost_model_for(RuntimeBackend::Mlx).is_none());
}

#[test]
fn c44_g4_cost_model_hashes_are_deterministic_and_distinct() {
    let composer2 = backend_name_hash(RuntimeBackend::Composer2);
    // Same backend -> same hash across calls.
    assert_eq!(composer2, backend_name_hash(RuntimeBackend::Composer2));
    // Different backends -> different hashes (collision check; not a
    // cryptographic guarantee but the canonical_name strings differ
    // enough for FxHasher to spread cleanly across 15 variants).
    assert_ne!(composer2, backend_name_hash(RuntimeBackend::Sidecar));
    assert_ne!(composer2, backend_name_hash(RuntimeBackend::AnthropicApi));

    let executor = role_name_hash(elai_ipc_types::AgentRole::Executor);
    assert_eq!(executor, role_name_hash(elai_ipc_types::AgentRole::Executor));
    assert_ne!(executor, role_name_hash(elai_ipc_types::AgentRole::Auditor));
}

// ---------------------------------------------------------------------------
// (G5) RuntimeBackend::all() includes Composer2 in deterministic order.
//      Adapter-iteration sites (RuntimeConfig::default factory_cfg seed,
//      RuntimePanel.svelte RUNTIME_BACKEND_IDS) iterate through all()
//      and must surface the new tier without manual updates.
// ---------------------------------------------------------------------------

#[test]
fn c44_g5_runtime_backend_all_includes_composer2() {
    let all = RuntimeBackend::all();
    assert_eq!(
        all.len(),
        16,
        "RuntimeBackend::all() must include the 14 pre-C-44 variants + Composer2 + Lsp (E-40)"
    );
    assert!(
        all.iter().any(|b| *b == RuntimeBackend::Composer2),
        "Composer2 must appear in RuntimeBackend::all()"
    );
}

// ---------------------------------------------------------------------------
// (G6) No paper-derived invariant pinned here.
//
// The C-44 plan cited arXiv:2603.24477 as "paper grounding" for the
// integration design. Per workflow/papers/composer2-interface.md,
// the cite is VALID (paper exists, title/authors/date match) but
// SHALLOW-FIT — the paper grounds model identity only, NOT
// integration design. This test documents the absence so a future
// plan that tries to encode a paper-claimed pricing / context /
// throughput number as a contract invariant trips review on this
// file. See workflow/sessions/s-25120/mistakes.md for the
// "shallow-cite-fit" failure mode (4th cite-quality incident in the
// s-25120 cluster).
// ---------------------------------------------------------------------------

#[test]
fn c44_g6_no_paper_derived_invariants_pinned_intentionally() {
    // This test is deliberately a no-op. The presence of the test +
    // the comment above documents that pricing / context numbers in
    // this contract come from cursor.com/docs/api source-grounding,
    // NOT from arXiv:2603.24477 paper grounding.
    //
    // If a future commit adds an assertion like
    //   `assert_eq!(caps.max_context, paper::COMPOSER2_CTX_FROM_PAPER);`
    // wiring-contract-validator should BLOCK on review and demand a
    // separate `c44_<name>_paper_invariant_contract.rs` file with an
    // explicit "paper-grounded" header — preserving the C-43 / C-44
    // pattern of one contract file per grounding posture.
}
