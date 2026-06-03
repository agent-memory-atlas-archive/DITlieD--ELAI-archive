//! F-11 contract: the four-guard chain rejects the right candidate per guard.
//!
//! Feature-gated (`darwinian-evolve`). One fixture per guard rejection:
//!   (a) a trap-pattern mutant (MarkerRemoval) scored on the LIVE candidate is
//!       RejectedByGuard guard="trap" (killer-risk #3 regression guard).
//!   (b) a candidate whose config touches "api_key"/"token" is RejectedByGuard
//!       guard="security" via is_security_sensitive (killer-risk #1 regression).
//!   (c) a low-margin candidate (alpha < tau) is RejectedByGuard guard="two_gate".
//! Plus the chain ORDER (tripwire -> trap -> security -> two_gate): a candidate
//! that is BOTH a trap AND security-sensitive rejects on TRAP (the earlier guard).
#![cfg(feature = "darwinian-evolve")]

use elai_orchestrator::darwin_admit_loop::{
    score_candidate, GuardOutcome, MutantCandidate, GUARD_SECURITY, GUARD_TRAP, GUARD_TWO_GATE,
};
use elai_orchestrator::promotion_gate::PromotionConfig;

fn cfg() -> PromotionConfig {
    PromotionConfig::default()
}

fn candidate(label: &str, config: serde_json::Value, eval_log: &str) -> MutantCandidate {
    MutantCandidate {
        variant_label: label.to_string(),
        pipeline_mode: "surgical".to_string(),
        config,
        eval_log: eval_log.to_string(),
    }
}

#[test]
fn trap_marker_removal_mutant_rejected_by_trap_guard() {
    let c = candidate(
        "trap-a",
        serde_json::json!({ "post_step_hook": "remove marker lines from own source", "force_alpha": 0.9 }),
        "marker_removal applied to self code",
    );
    let out = score_candidate(&c, c.derived_alpha(), 0, &cfg()).expect("guard chain");
    assert_eq!(out.guard_name(), Some(GUARD_TRAP), "got {out:?}");
}

#[test]
fn security_token_mutant_rejected_by_security_guard() {
    let c = candidate(
        "sec-b",
        serde_json::json!({ "api_key": "sk-leak", "auth_header": "Bearer t", "force_alpha": 0.9 }),
        "clean run, no traps",
    );
    let out = score_candidate(&c, c.derived_alpha(), 0, &cfg()).expect("guard chain");
    assert_eq!(out.guard_name(), Some(GUARD_SECURITY), "got {out:?}");
}

#[test]
fn low_margin_mutant_rejected_by_two_gate() {
    let c = candidate(
        "low-c",
        serde_json::json!({ "rag_top_k": 8, "force_alpha": 0.01 }),
        "marginal improvement; no traps; no security fields",
    );
    let out = score_candidate(&c, c.derived_alpha(), 0, &cfg()).expect("guard chain");
    assert_eq!(out.guard_name(), Some(GUARD_TWO_GATE), "got {out:?}");
}

#[test]
fn clean_candidate_passes_all_guards() {
    let c = candidate(
        "clean-d",
        serde_json::json!({ "rag_top_k": 8, "force_alpha": 0.7 }),
        "clean run; score 0.7",
    );
    let out = score_candidate(&c, c.derived_alpha(), 0, &cfg()).expect("guard chain");
    assert_eq!(out, GuardOutcome::Admitted, "got {out:?}");
}

#[test]
fn guard_chain_order_trap_before_security() {
    // A candidate that is BOTH a trap (marker_removal) AND security-sensitive
    // (api_key) must reject on TRAP — the earlier guard in the series. This
    // locks the chain order (tripwire -> trap -> security -> two_gate).
    let c = candidate(
        "trap-and-sec",
        serde_json::json!({
            "api_key": "sk-leak",
            "post_step_hook": "remove marker lines",
            "force_alpha": 0.9
        }),
        "marker_removal applied; carries credential",
    );
    let out = score_candidate(&c, c.derived_alpha(), 0, &cfg()).expect("guard chain");
    assert_eq!(
        out.guard_name(),
        Some(GUARD_TRAP),
        "trap guard runs BEFORE security; got {out:?}"
    );
}
