//! WAVE-A on-path smoke for the E-20 privacy BLOCK gate
//! (ELAI_PRIVACY_BLOCK_ENABLE).
//!
//! Two real artifacts are asserted:
//!   (1) the public gate predicate
//!       `elai_orchestrator::privacy::privacy_block_enabled()` flips ON when
//!       the env var is set (default-OFF pinned), and
//!   (2) the gated ON-path behavior itself: the real scanner
//!       `scan_prompt_for_secrets` returns severity=Block on a
//!       hard-secret-shaped input, which is what a privacy-block dispatch
//!       site consumes to refuse cloud send. The scanner is the off-agent
//!       observable (it computes a verdict the test did not hand it).
//!
//! Production dispatch-site wiring is tracked by promotion_plan_id E-20.

use std::sync::Mutex;

use elai_orchestrator::privacy::secret_scan::scan_prompt_for_secrets;
use elai_orchestrator::privacy::{privacy_block_enabled, ScanSeverity};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const GATE: &str = "ELAI_PRIVACY_BLOCK_ENABLE";

#[test]
fn privacy_block_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(GATE);
    assert!(
        !privacy_block_enabled(),
        "CONTRACT: {GATE} MUST default OFF (wire-dark per E-20)"
    );
}

#[test]
fn privacy_block_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "1");
    let on = privacy_block_enabled();
    std::env::remove_var(GATE);
    assert!(on, "CONTRACT: {GATE}=1 MUST open the E-20 privacy block gate");
}

#[test]
fn scanner_blocks_on_aws_access_key_shape() {
    // ON-path behaviour the gated dispatch site consumes: a hard-secret
    // shape (AWS access key prefix) MUST be classified Block so the caller
    // refuses cloud dispatch. AKIA + 16 uppercase/digit chars.
    let prompt = "here is my key AKIAIOSFODNN7EXAMPLE please use it";
    let result = scan_prompt_for_secrets(prompt);
    assert_eq!(
        result.severity,
        ScanSeverity::Block,
        "CONTRACT: an AWS access-key shape MUST scan as Block (refuse cloud send); got {:?}",
        result.severity
    );
    assert!(
        result.is_block(),
        "CONTRACT: ScanResult::is_block() MUST be true for the block-severity verdict"
    );
    assert!(
        result.match_count() >= 1,
        "CONTRACT: at least one match span MUST be recorded"
    );
}

#[test]
fn scanner_clean_on_benign_prompt() {
    // Negative arm: a benign prompt MUST NOT be blocked (the gate is a
    // precision-sensitive refusal; false positives are recoverable but the
    // smoke pins the clean-path baseline so a regex over-broadening is caught).
    let result = scan_prompt_for_secrets("refactor the parse function to use iterators");
    assert_eq!(
        result.severity,
        ScanSeverity::Clean,
        "CONTRACT: a benign prompt MUST scan Clean; got {:?}",
        result.severity
    );
}
