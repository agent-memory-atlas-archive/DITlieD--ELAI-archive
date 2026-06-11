//! WAVE-A on-path smoke for the E-25 sandbox-template guard gate
//! (ELAI_E25_SANDBOX_TEMPLATE_GUARD_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `sandbox_template_guard_enabled()` (default-OFF pinned, ON when env
//! truthy) AND exercises the gated guard ON-path: `SandboxTemplateGuard::
//! check()` returns the Refused-with-jail-fallback verdict the wired
//! dispatch site consumes when microsandbox is unavailable. The guard
//! verdict is the off-agent observable. Production wire of the guard into the
//! skill-template dispatch is tracked by promotion_plan_id E-25.

use std::sync::Mutex;

use elai_orchestrator::architecture::sandbox_template_guard::{
    sandbox_template_guard_enabled, SandboxTemplateGuard, SandboxTemplateVerdict,
    SKILL_TEMPLATE_JAIL_PATH,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const GATE: &str = "ELAI_E25_SANDBOX_TEMPLATE_GUARD_ENABLE";

#[test]
fn sandbox_template_guard_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(GATE);
    assert!(
        !sandbox_template_guard_enabled(),
        "CONTRACT: {GATE} MUST default OFF (wire-dark per E-25)"
    );
}

#[test]
fn sandbox_template_guard_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "1");
    let on = sandbox_template_guard_enabled();
    std::env::remove_var(GATE);
    assert!(
        on,
        "CONTRACT: {GATE}=1 MUST open the E-25 sandbox-template guard gate"
    );
}

#[test]
fn guard_refuses_with_jail_fallback_when_microsandbox_absent() {
    // ON-path observable: microsandbox is not on disk today (E-22 dep), so
    // the guard MUST refuse and surface the canonical jail path the caller
    // applies as the fallback sandbox.
    let guard = SandboxTemplateGuard::new();
    let verdict = guard.check();
    assert!(
        verdict.is_refused(),
        "CONTRACT: guard MUST Refuse when microsandbox is unavailable; got {:?}",
        verdict
    );
    match verdict {
        SandboxTemplateVerdict::Refused { jail_path, .. } => {
            assert_eq!(
                jail_path, SKILL_TEMPLATE_JAIL_PATH,
                "CONTRACT: Refused verdict MUST carry the canonical jail path"
            );
        }
        SandboxTemplateVerdict::Allow => {
            panic!("expected Refused (microsandbox absent), got Allow");
        }
    }
}
