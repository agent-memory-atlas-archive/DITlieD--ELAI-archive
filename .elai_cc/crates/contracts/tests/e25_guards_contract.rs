//! E-25 Tranche C (C.1) -- anti-pattern guard contracts.
//!
//! Verifies that each of the three guards (Bash complexity / approval
//! fatigue / sandbox template) refuses on its trigger condition and
//! emits a SSB payload with the SHIPPED Pattern-2 single-variant
//! `ClaudeCodeArchitectureEvent` + matching already-shipped Kind arm.
//! ZERO new SsbActionType, ZERO schema bump.

use elai_orchestrator::architecture::approval_fatigue_guard::{
    ApprovalFatigueGuard, ApprovalFatigueVerdict, ApprovalKind,
};
use elai_orchestrator::architecture::bash_complexity_guard::{
    BashComplexityGuard, BashComplexityVerdict,
};
use elai_orchestrator::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
};
use elai_orchestrator::architecture::sandbox_template_guard::SandboxTemplateGuard;

#[test]
fn bash_complexity_guard_refuses_and_emits_blocked_kind() {
    let guard = BashComplexityGuard::new();
    // 5 pipes -> trips BASH_PIPE_COUNT_THRESHOLD (4).
    let cmd = "a | b | c | d | e | f";
    let verdict = guard.check(cmd);
    assert!(
        matches!(verdict, BashComplexityVerdict::Refused { .. }),
        "expected refusal on >4 pipes, got {:?}",
        verdict
    );
    let payload = guard
        .build_payload(cmd, &verdict, Some("s-trancheC".into()), 1_700_000_000_000)
        .expect("refused payload");
    assert_eq!(
        payload.kind,
        ClaudeCodeArchitectureEventKind::BashComplexityBlocked
    );
    assert_eq!(
        payload.primitive,
        ClaudeCodeArchPrimitive::BashComplexityGuard
    );
    assert_eq!(payload.session_id.as_deref(), Some("s-trancheC"));
}

#[test]
fn approval_fatigue_guard_refuses_and_emits_detected_kind() {
    let mut guard = ApprovalFatigueGuard::with_window(10, 80);
    // Fill the window to capacity with 9 Allows + 1 Deny -> 90% over 80%.
    for _ in 0..9 {
        guard.record_and_check(ApprovalKind::Allow);
    }
    let verdict = guard.record_and_check(ApprovalKind::Deny);
    assert!(
        matches!(verdict, ApprovalFatigueVerdict::Refused { .. }),
        "expected refusal at 90% allow rate over 10-decision window, got {:?}",
        verdict
    );
    let payload = guard
        .build_payload(&verdict, Some("s-trancheC".into()), 1_700_000_000_000)
        .expect("refused payload");
    assert_eq!(
        payload.kind,
        ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected
    );
    assert_eq!(
        payload.primitive,
        ClaudeCodeArchPrimitive::ApprovalFatigueGuard
    );
}

#[test]
fn sandbox_template_guard_refuses_and_emits_warned_kind() {
    // E-22 Task 2 dep: microsandbox crate is not yet on disk, so the
    // guard always-warns in the current state. Once E-22 lands this
    // test updates to assert Allow when microsandbox_available() flips.
    let guard = SandboxTemplateGuard::new();
    let verdict = guard.check();
    assert!(verdict.is_refused(), "expected always-warn until E-22 lands");
    let payload = guard
        .build_payload(
            &verdict,
            Some("test-skill"),
            Some("s-trancheC".into()),
            1_700_000_000_000,
        )
        .expect("refused payload");
    assert_eq!(
        payload.kind,
        ClaudeCodeArchitectureEventKind::SandboxTemplateWarned
    );
    assert_eq!(
        payload.primitive,
        ClaudeCodeArchPrimitive::SandboxTemplateGuard
    );
}

// E-25 Tranche L (F.1.c) backfill: assert the ApprovalFatigueGuard's
// production wire-in path (see l_approval_fatigue_prod_contract.rs for the
// full plumbing surface). These two assertions are kept in this file so
// the guards-contract suite continues to be the single landing page for
// the three-guard family.
#[test]
fn approval_fatigue_guard_canonical_constructor_is_with_defaults() {
    // The wire-in at cli/runtime_builder.rs installs the guard via
    // ApprovalFatigueGuard::with_defaults(); this assertion locks the
    // canonical constructor name + that it produces a non-tripped guard
    // on its FIRST observation (the wire-in's first prompt must not
    // refuse out of the box).
    let mut guard = ApprovalFatigueGuard::with_defaults();
    let verdict = guard.record_and_check(ApprovalKind::Allow);
    assert!(
        matches!(verdict, ApprovalFatigueVerdict::Allow { .. }),
        "CONTRACT: a freshly-defaulted guard MUST allow its first observation \
         (window len 1 < capacity {} means Allow per substrate L165-168). Got: {:?}",
        elai_orchestrator::architecture::approval_fatigue_guard::APPROVAL_WINDOW_SIZE,
        verdict
    );
}

#[test]
fn approval_fatigue_env_gate_is_off_by_default() {
    // Default-OFF posture per E-25 substrate. The wire-in's double-gate
    // (field-Some + env-truthy) MUST keep production traffic in the
    // dormant state when no operator opts in.
    use elai_orchestrator::architecture::approval_fatigue_guard::approval_fatigue_guard_enabled;
    // Best-effort assertion: we do NOT mutate env here (other tests in
    // this binary may have set it). Instead we lock the truth that the
    // gate-check function is a pure read of the env var, so a fresh
    // shell with no operator override returns false.
    let cur = std::env::var("ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE").ok();
    if cur.is_none() {
        assert!(
            !approval_fatigue_guard_enabled(),
            "CONTRACT: ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE MUST default OFF \
             (wire-dark posture per Tranche F precedent + DECISION-L-OPTION-D)"
        );
    }
}

// E-25 stubs F1c / F1a emit contracts: verify the reason-prefix contract
// that conversation.rs uses to detect a BashComplexityGuard refusal, and
// verify the BashComplexityGuard env gate defaults off. These are the
// key observable invariants the emit caller in conversation.rs relies on.
#[test]
fn bash_complexity_guard_reason_starts_with_canonical_prefix() {
    // conversation.rs detects the bash_complexity refusal via
    // `reason.starts_with("bash_complexity:")`. This test locks the
    // prefix produced by BashComplexityGuard::check() so a rename
    // in the guard can't silently break the emit path.
    let guard = BashComplexityGuard::new();
    let cmd = "a | b | c | d | e | f"; // 5 pipes -> trips BASH_PIPE_COUNT_THRESHOLD
    let verdict = guard.check(cmd);
    match &verdict {
        BashComplexityVerdict::Refused { reason } => {
            assert!(
                reason.starts_with("bash_complexity:"),
                "CONTRACT: BashComplexityGuard Refused reason MUST start with \
                 'bash_complexity:' so conversation.rs emit detection works. Got: {:?}",
                reason
            );
        }
        BashComplexityVerdict::Allow => {
            panic!("expected Refused on 5-pipe command, got Allow");
        }
    }
}

#[test]
fn bash_complexity_env_gate_is_off_by_default() {
    use elai_orchestrator::architecture::bash_complexity_guard::bash_complexity_guard_enabled;
    let cur = std::env::var("ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE").ok();
    if cur.is_none() {
        assert!(
            !bash_complexity_guard_enabled(),
            "CONTRACT: ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE MUST default OFF \
             (wire-dark posture per E-25 substrate + DECISION-T-F-4)"
        );
    }
}
