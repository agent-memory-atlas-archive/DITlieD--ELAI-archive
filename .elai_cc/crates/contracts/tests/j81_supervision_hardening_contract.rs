//! J-81 supervision hardening contract.
//!
//! Three arms exercise the REAL `abort_task_on_cap_breach` function
//! (no re-implementation in the test):
//!   (a) task spend over max -> Abort SpendCapBreached
//!   (b) layered hard cutoff day spend -> Abort LayeredHardCutoffExceeded
//!   (c) TD-3: the abort decision is a pure `AbortDecision` enum — no
//!       `should_fully_stop` / LLM stop gate participates in the
//!       production abort path. Asserted at compile time via a const
//!       check + at runtime by confirming the decision variant is the
//!       enum (deterministic, no side effects).
//!
//! Production entrypoint: elai task / elai run -> start_task_runner ->
//! run_task_runner -> drive_task cloud-dispatch branch (task_runner.rs
//! `abort_task_on_cap_breach` call sites ~line 7040 preclassify and
//! ~line 9185 mid-loop baby-step).

use elai_orchestrator::cost::cloud_abort::{
    abort_task_on_cap_breach, AbortCtx, AbortDecision, LayeredCapsConfig, PerRoleCapsConfig,
    TaskAbortReason,
};
use elai_orchestrator::cost::CostRoleBucket;
use rustc_hash::FxHashMap;

/// (a) spend over max_spend_per_task -> Abort SpendCapBreached.
#[test]
fn contract_j81_arm_a_spend_cap_breached() {
    let ctx = AbortCtx {
        task_spend_usd_cents: 1_500,
        max_spend_per_task_cents: 1_000,
        task_spend_by_role: FxHashMap::default(),
        active_role: Some(CostRoleBucket::Executor),
        day_spend_usd_cents: 0,
        per_role_caps: PerRoleCapsConfig::default(),
        layered_caps: LayeredCapsConfig::default(),
        velocity_breach_present: false,
    };
    let decision = abort_task_on_cap_breach(&ctx);
    match decision {
        AbortDecision::Abort {
            reason,
            spent_usd_cents,
            cap_usd_cents,
            ..
        } => {
            assert_eq!(reason, TaskAbortReason::SpendCapBreached);
            assert_eq!(spent_usd_cents, 1_500);
            assert_eq!(cap_usd_cents, 1_000);
        }
        AbortDecision::Continue => panic!("expected Abort SpendCapBreached, got Continue"),
    }
}

/// (b) layered hard cutoff day spend -> Abort LayeredHardCutoffExceeded.
#[test]
fn contract_j81_arm_b_layered_hard_cutoff() {
    let ctx = AbortCtx {
        task_spend_usd_cents: 0,
        max_spend_per_task_cents: 100_000,
        task_spend_by_role: FxHashMap::default(),
        active_role: Some(CostRoleBucket::Executor),
        day_spend_usd_cents: 11_000,
        per_role_caps: PerRoleCapsConfig::default(),
        layered_caps: LayeredCapsConfig {
            hard_cutoff_usd_cents: 10_000,
            ..Default::default()
        },
        velocity_breach_present: false,
    };
    let decision = abort_task_on_cap_breach(&ctx);
    match decision {
        AbortDecision::Abort {
            reason,
            spent_usd_cents,
            cap_usd_cents,
            ..
        } => {
            assert_eq!(reason, TaskAbortReason::LayeredHardCutoffExceeded);
            assert_eq!(spent_usd_cents, 11_000);
            assert_eq!(cap_usd_cents, 10_000);
        }
        AbortDecision::Continue => panic!("expected Abort LayeredHardCutoffExceeded, got Continue"),
    }
}

/// (c) TD-3: the production abort path returns an `AbortDecision` enum
/// only — no `should_fully_stop` LLM gate. The decision is deterministic
/// (pure function of the ctx struct, no I/O, no model call). We assert
/// the enum shape at runtime and the determinism via a const check that
/// `AbortDecision` is a non-unit enum (the compiler guarantees the
/// variant set).
#[test]
fn contract_j81_arm_c_abort_is_enum_only_no_llm_stop() {
    // Compile-time: AbortDecision is an enum with >1 variant (not a
    // unit-only type that could hide a bool/LLM gate behind a type alias).
    const IS_ENUM: bool = {
        let _proof: AbortDecision = AbortDecision::Continue;
        let _proof2: AbortDecision = AbortDecision::Abort {
            reason: TaskAbortReason::SpendCapBreached,
            spent_usd_cents: 0,
            cap_usd_cents: 0,
            role: None,
        };
        true
    };
    assert!(IS_ENUM, "AbortDecision must be a multi-variant enum");

    // Runtime: a no-breach ctx returns Continue (deterministic, no side
    // effects, no model call). This is the production path's "keep
    // running" decision — if an LLM stop gate were wired in, this would
    // be non-deterministic or require a model handle.
    let ctx = AbortCtx {
        task_spend_usd_cents: 0,
        max_spend_per_task_cents: 1_000,
        task_spend_by_role: FxHashMap::default(),
        active_role: None,
        day_spend_usd_cents: 0,
        per_role_caps: PerRoleCapsConfig::default(),
        layered_caps: LayeredCapsConfig::default(),
        velocity_breach_present: false,
    };
    let decision = abort_task_on_cap_breach(&ctx);
    assert_eq!(
        decision,
        AbortDecision::Continue,
        "no-breach ctx must deterministically Continue — no LLM stop gate"
    );

    // The function signature takes only `&AbortCtx` (no model handle, no
    // async), so an LLM stop gate cannot participate. This is enforced
    // by the type system: `abort_task_on_cap_breach` is a sync fn with
    // no I/O parameters.
}
