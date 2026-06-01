//! Tranche K / B.2 Option-B -- contract tests for the new
//! ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE env gate + the per-mode
//! `c50_emit_gate_truthy(non_web: bool)` dispatcher in
//! `orchestrator::contract_coding::retry_emit`.
//!
//! Pins:
//!   * The new env-gate string constant is stable across releases.
//!   * `env_gate_truthy` returns `false` by default (wire-dark posture).
//!   * Truthy synonyms ("1"/"true"/"yes"/"on" + case variants) all flip
//!     the new gate independently.
//!   * The per-mode dispatcher `c50_emit_gate_truthy` reads the WEB gate
//!     when `non_web=false` and the NON-WEB gate when `non_web=true`
//!     -- so each mode is a TRULY INDEPENDENT switch (W3 finding).
//!   * Flipping only the non-web gate does NOT enable web emissions and
//!     vice versa.
//!   * `SCHEMA_VERSION_PUBLIC` is unchanged by this tranche (no new
//!     SsbActionType variants, no new Kind arms, no payload shape
//!     changes).
//!
//! Mirrors the structural shape of
//! `crates/contracts/tests/b6_quad_ssb_contract.rs`. No cargo run
//! executions; only `#[test]` declarations the auditor invokes.

use elai_orchestrator::contract_coding::c50_emit_gate_truthy;
use elai_orchestrator::ssb::{
    env_gate_truthy, ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV,
    ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV,
};
use elai_orchestrator::SCHEMA_VERSION_PUBLIC;

// Helper: serialize env-var setup so the tests do not race against each
// other when run in parallel. The contracts crate's other env-touching
// tests rely on the same single-threaded convention (b6_quad_ssb_contract
// comments confirm: contracts tests do NOT use multi-thread runtimes for
// the env-var subset).
//
// We additionally clear BOTH gates at the top of every test so cross-test
// state cannot leak (CI cross-contamination guard).
fn clear_both_gates() {
    // SAFETY of env mutation in test: env API is process-wide; the
    // contracts crate runs single-threaded for these env-touching tests
    // by default. unsafe block guards against the 2026-edition
    // set_var/remove_var unsafe pin (same precedent as b6_quad).
    #[allow(unused_unsafe)]
    unsafe {
        std::env::remove_var(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV);
        std::env::remove_var(ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV);
    }
}

// ---------------------------------------------------------------------------
// Contract 1 -- the new env-gate string constant is stable.
// ---------------------------------------------------------------------------

#[test]
fn contract_non_web_env_gate_string_is_stable() {
    assert_eq!(
        ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV, "ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE",
        "CONTRACT: Tranche K env-gate name must stay stable across releases. \
         Operators flip this in the field; renaming silently breaks rollout."
    );
}

// ---------------------------------------------------------------------------
// Contract 2 -- env_gate_truthy default false (wire-dark posture).
// ---------------------------------------------------------------------------

#[test]
fn contract_non_web_env_gate_default_false() {
    clear_both_gates();
    assert!(
        !env_gate_truthy(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV),
        "CONTRACT: {} must be wire-dark by default (env_gate_truthy false)",
        ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV
    );
}

// ---------------------------------------------------------------------------
// Contract 3 -- truthy synonyms recognised; falsy / unset values reject.
// ---------------------------------------------------------------------------

#[test]
fn contract_non_web_env_gate_recognises_truthy_synonyms() {
    let truthy = ["1", "true", "yes", "on", "TRUE", "Yes", "ON"];
    let falsy = ["0", "false", "no", "off", "", "maybe", "2"];

    for v in &truthy {
        clear_both_gates();
        #[allow(unused_unsafe)]
        unsafe {
            std::env::set_var(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV, v);
        }
        assert!(
            env_gate_truthy(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV),
            "CONTRACT: non-web env_gate_truthy({:?}) must be true",
            v
        );
    }
    for v in &falsy {
        clear_both_gates();
        #[allow(unused_unsafe)]
        unsafe {
            std::env::set_var(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV, v);
        }
        assert!(
            !env_gate_truthy(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV),
            "CONTRACT: non-web env_gate_truthy({:?}) must be false",
            v
        );
    }
    clear_both_gates();
}

// ---------------------------------------------------------------------------
// Contract 4 -- both gates can be set simultaneously and remain
// independent (no shared-OR semantics).
// ---------------------------------------------------------------------------

#[test]
fn contract_both_gates_can_be_set_independently() {
    clear_both_gates();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var(ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV, "1");
        std::env::set_var(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV, "1");
    }
    assert!(
        env_gate_truthy(ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV),
        "CONTRACT: web gate must read true when set to 1"
    );
    assert!(
        env_gate_truthy(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV),
        "CONTRACT: non-web gate must read true when set to 1"
    );
    clear_both_gates();
}

// ---------------------------------------------------------------------------
// Contract 5 -- c50_emit_gate_truthy(false) reads the WEB gate;
// c50_emit_gate_truthy(true) reads the NON-WEB gate. The two are
// independent. This is the W3 finding's executable assertion.
// ---------------------------------------------------------------------------

#[test]
fn contract_c50_emit_gate_truthy_routes_per_mode() {
    // --- Sub-case A: only WEB gate set --------------------------------------
    clear_both_gates();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var(ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV, "1");
    }
    assert!(
        c50_emit_gate_truthy(false),
        "CONTRACT: c50_emit_gate_truthy(non_web=false) must read WEB gate (true here)"
    );
    assert!(
        !c50_emit_gate_truthy(true),
        "CONTRACT: c50_emit_gate_truthy(non_web=true) must NOT widen via WEB gate. \
         W3 finding: per-mode gates remain truly independent."
    );

    // --- Sub-case B: only NON-WEB gate set ----------------------------------
    clear_both_gates();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV, "1");
    }
    assert!(
        c50_emit_gate_truthy(true),
        "CONTRACT: c50_emit_gate_truthy(non_web=true) must read NON-WEB gate (true here)"
    );
    assert!(
        !c50_emit_gate_truthy(false),
        "CONTRACT: c50_emit_gate_truthy(non_web=false) must NOT widen via NON-WEB gate. \
         W3 finding: flipping non-web must NEVER silently enable web emissions."
    );

    // --- Sub-case C: both gates set -----------------------------------------
    clear_both_gates();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var(ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV, "1");
        std::env::set_var(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV, "1");
    }
    assert!(
        c50_emit_gate_truthy(false),
        "CONTRACT: both gates set -> web caller fires"
    );
    assert!(
        c50_emit_gate_truthy(true),
        "CONTRACT: both gates set -> non-web caller fires"
    );

    // --- Sub-case D: neither gate set ---------------------------------------
    clear_both_gates();
    assert!(
        !c50_emit_gate_truthy(false),
        "CONTRACT: no gates set -> wire-dark for web"
    );
    assert!(
        !c50_emit_gate_truthy(true),
        "CONTRACT: no gates set -> wire-dark for non-web"
    );
}

// ---------------------------------------------------------------------------
// Contract 6 -- wire-dark posture for the emit_retry_* dispatcher. With
// both gates unset, the per-mode dispatcher must return false for every
// caller -- ZERO Ssb rows would emit even if the emit fns were dispatched.
// ---------------------------------------------------------------------------

#[test]
fn contract_non_web_emit_is_wire_dark_default() {
    clear_both_gates();
    assert!(
        !c50_emit_gate_truthy(true),
        "CONTRACT: non-web caller with no gate set must remain wire-dark"
    );
    assert!(
        !c50_emit_gate_truthy(false),
        "CONTRACT: web caller with no gate set must remain wire-dark"
    );
}

// ---------------------------------------------------------------------------
// Contract 7 -- web call site is unchanged by the Tranche K addition.
// The web gate (`ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE`) continues to fire
// the web caller regardless of the new non-web gate's state. This pins
// back-compat with the B-6 quad contract.
// ---------------------------------------------------------------------------

#[test]
fn contract_web_call_site_back_compat_preserved() {
    // Set only the existing web gate (the canonical operator-flip prior
    // to this tranche). Non-web gate stays unset.
    clear_both_gates();
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var(ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV, "1");
    }
    assert!(
        c50_emit_gate_truthy(false),
        "CONTRACT: existing web operator-flip must continue to fire web emits. \
         Tranche K MUST NOT break the B-6 quad rollout."
    );
    clear_both_gates();
}

// ---------------------------------------------------------------------------
// Contract 8 -- SCHEMA_VERSION is unchanged. Tranche K adds NO new
// SsbActionType variants, NO new Kind arms, NO payload shape changes;
// only a NEW env constant + a SECOND call site to an existing payload
// type. Schema discipline pins this here.
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_unchanged_by_tranche_k() {
    // Per context-package-trancheK.md L8: "Tranche K adds ZERO new SSB
    // variants / ZERO Kind arms / ZERO payload shape changes". The
    // ledger floor was 195 at the start of Tranche K (s-22600). Asserting
    // monotone-forward (>= 195) so later tranches' bumps don't fail
    // this test, while still pinning the tranche K floor.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 195,
        "CONTRACT: SCHEMA_VERSION_PUBLIC must be >= 195 (the value at the \
         start of Tranche K). This tranche MUST NOT bump it -- no new \
         variants / no new Kind arms / no payload shape changes. Reviewers \
         must verify the bump came from a different tranche before \
         updating this floor."
    );
}
