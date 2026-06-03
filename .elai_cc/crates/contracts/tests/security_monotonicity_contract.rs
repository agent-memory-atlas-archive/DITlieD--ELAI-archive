//! F-10 / Slice A4 — security-monotonicity iteration guard contract tests.
//!
//! Interface-level guarantees for the F-10 standalone gate:
//!   (a) the `SecurityClassRegressionBlocked` SSB payload round-trips through
//!       `rmp_serde` with `loop_kind` (and every other field) preserved — the
//!       same `to_vec_named` encoder `Ssb::write_payload` uses on the wire.
//!   (b) `SecurityMonotonicityGate::check_monotonicity` returns `Regressed`
//!       (with the newly introduced CWE id) when the current iteration emits a
//!       CWE class absent from the prior iteration's set.
//!   (c) it returns `Monotone` when the current classes are a subset of prior.
//!   (d) the `ELAI_SECURITY_MONOTONICITY_DISABLE` kill-switch forces `Monotone`
//!       even on a clear regression (W-2: kill-switch, never a default-off
//!       ENABLE flag).
//!   (e) a cold-start `Evidence` observes `security_monotonicity == true`
//!       (vacuous default — a context that did not run the gate is not blocked),
//!       so the existing autonomous_gate cold-start contracts pass UNCHANGED.
//!
//! Pure + deterministic: no network, no filesystem, no clock. The env mutation
//! in (d) is tightly scoped and restored before asserting.

use std::collections::BTreeSet;

use elai_orchestrator::autonomous_gate::Evidence;
use elai_orchestrator::ssb::{LoopKind, ELAI_SECURITY_MONOTONICITY_DISABLE_ENV};
use elai_orchestrator::{
    CweClass, MonotonicityVerdict, SecurityMonotonicityGate, SsbPayload,
};
// The payload struct lives under the public `ssb` module (not a top-level
// re-export); reach it directly for the round-trip construction.
use elai_orchestrator::ssb::SecurityClassRegressionBlockedPayload;

const LOOP_ID: &str = "f10-contract-loop";

// ---------------------------------------------------------------------------
// (a) SSB SecurityClassRegressionBlocked round-trip incl loop_kind
// ---------------------------------------------------------------------------

#[test]
fn contract_ssb_payload_round_trips_with_loop_kind() {
    let payload = SsbPayload::SecurityClassRegressionBlocked(SecurityClassRegressionBlockedPayload {
        delta_blake3: "0123456789abcdef".to_string(),
        introduced_classes: vec!["CWE-78".to_string()],
        prior_classes: vec!["CWE-89".to_string()],
        loop_kind: LoopKind::AutoAdvance,
        loop_id: LOOP_ID.to_string(),
        lang: "python".to_string(),
    });

    // Encode with the SAME encoder Ssb::write_payload uses on the wire
    // (rmp_serde::to_vec_named) so the contract reflects the real serialization.
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode SecurityClassRegressionBlocked");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("decode SecurityClassRegressionBlocked");

    // Full structural equality (SsbPayload + its payload struct both derive
    // PartialEq), which proves loop_kind round-trips along with the rest.
    assert_eq!(decoded, payload, "payload must round-trip identically");

    // Explicit loop_kind assertion so a future field reorder can't silently
    // drop the discriminator the 3 production legs are distinguished by.
    match decoded {
        SsbPayload::SecurityClassRegressionBlocked(p) => {
            assert_eq!(p.loop_kind, LoopKind::AutoAdvance, "loop_kind preserved");
            assert_eq!(p.introduced_classes, vec!["CWE-78".to_string()]);
            assert_eq!(p.prior_classes, vec!["CWE-89".to_string()]);
            assert_eq!(p.loop_id, LOOP_ID);
            assert_eq!(p.lang, "python");
        }
        other => panic!("expected SecurityClassRegressionBlocked, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (b) Regressed when current introduces a class absent from prior
// ---------------------------------------------------------------------------

#[test]
fn contract_regressed_when_new_class_introduced() {
    // Ensure the kill-switch is unset for this test (default-ON, W-2).
    let prev = scoped_unset_killswitch();

    // prior carries SQLi only; current introduces CWE-78 (python os.system).
    let prior: BTreeSet<CweClass> = [CweClass::Sqli].into_iter().collect();
    let verdict = SecurityMonotonicityGate::check_monotonicity(
        LoopKind::Refine,
        LOOP_ID,
        &prior,
        "import os\nos.system(user_input)\n",
        "python",
    );

    restore_killswitch(prev);

    match verdict {
        MonotonicityVerdict::Regressed { introduced_classes } => {
            assert!(
                introduced_classes.contains(&"CWE-78".to_string()),
                "CWE-78 must be in introduced set, got {introduced_classes:?}"
            );
            // CWE-89 was already in prior -> must NOT be reported as introduced.
            assert!(
                !introduced_classes.contains(&"CWE-89".to_string()),
                "a class already in prior is not a regression"
            );
        }
        MonotonicityVerdict::Monotone => panic!("expected Regressed, got Monotone"),
    }
}

// ---------------------------------------------------------------------------
// (c) Monotone when current classes are a subset of prior
// ---------------------------------------------------------------------------

#[test]
fn contract_monotone_when_current_subset_of_prior() {
    let prev = scoped_unset_killswitch();

    // prior already contains CWE-78; current re-emits it -> no NEW class.
    let prior: BTreeSet<CweClass> = [CweClass::CommandInjection].into_iter().collect();
    let verdict = SecurityMonotonicityGate::check_monotonicity(
        LoopKind::AutoAdvance,
        LOOP_ID,
        &prior,
        "import os\nos.system(user_input)\n",
        "python",
    );

    restore_killswitch(prev);

    assert_eq!(
        verdict,
        MonotonicityVerdict::Monotone,
        "a class already present in prior must not regress"
    );
    assert!(!verdict.is_regressed());
}

// ---------------------------------------------------------------------------
// (d) Kill-switch forces Monotone even on a clear regression
// ---------------------------------------------------------------------------

#[test]
fn contract_killswitch_forces_monotone() {
    let key = ELAI_SECURITY_MONOTONICITY_DISABLE_ENV;
    let prev = std::env::var(key).ok();
    std::env::set_var(key, "1");

    // Empty prior + a clear CWE-78 in current would normally Regress.
    let prior: BTreeSet<CweClass> = BTreeSet::new();
    let verdict = SecurityMonotonicityGate::check_monotonicity(
        LoopKind::SelfMod,
        LOOP_ID,
        &prior,
        "import os\nos.system(user_input)\n",
        "python",
    );

    // Restore env BEFORE asserting so a failed assert still cleans up.
    match prev {
        Some(p) => std::env::set_var(key, p),
        None => std::env::remove_var(key),
    }

    assert_eq!(
        verdict,
        MonotonicityVerdict::Monotone,
        "kill-switch must force Monotone (W-2)"
    );
}

// ---------------------------------------------------------------------------
// (e) Cold-start Evidence observes security_monotonicity == true
// ---------------------------------------------------------------------------

#[test]
fn contract_cold_start_evidence_security_monotonicity_true() {
    let ev = Evidence::default();
    assert!(
        ev.security_monotonicity,
        "vacuous default must be true so non-gate contexts are not blocked"
    );
}

// ---------------------------------------------------------------------------
// Env helpers — scope the kill-switch mutation tightly and restore it so the
// per-binary env stays clean across tests sharing the process.
// ---------------------------------------------------------------------------

/// Remove the kill-switch var, returning the prior value for restoration.
fn scoped_unset_killswitch() -> Option<String> {
    let prev = std::env::var(ELAI_SECURITY_MONOTONICITY_DISABLE_ENV).ok();
    std::env::remove_var(ELAI_SECURITY_MONOTONICITY_DISABLE_ENV);
    prev
}

/// Restore a previously captured kill-switch value (or leave it unset).
fn restore_killswitch(prev: Option<String>) {
    match prev {
        Some(p) => std::env::set_var(ELAI_SECURITY_MONOTONICITY_DISABLE_ENV, p),
        None => std::env::remove_var(ELAI_SECURITY_MONOTONICITY_DISABLE_ENV),
    }
}
