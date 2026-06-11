//! WAVE-A on-path smoke for the E-21 peer-systems SSB gate
//! (ELAI_PEER_SYSTEMS_ENABLE).
//!
//! Two real artifacts:
//!   (1) the public gate predicate
//!       `elai_orchestrator::peer_systems::peer_systems_emit_enabled()` flips
//!       ON when the env var is set (default-OFF pinned), and
//!   (2) the gated ON-path compute: the real
//!       `CriticRerank::pick_argmax` kernel selects the argmax candidate with
//!       deterministic tie-break — the off-agent observable a wired rerank
//!       round consumes before committing a patch.
//!
//! Production rerank wiring (per-candidate runtime invocation) is tracked by
//! promotion_plan_id E-21.

use std::sync::Mutex;

use elai_orchestrator::peer_systems::critic_rerank::{CriticCandidate, CriticRerank};
use elai_orchestrator::peer_systems::peer_systems_emit_enabled;

static ENV_LOCK: Mutex<()> = Mutex::new(());

const GATE: &str = "ELAI_PEER_SYSTEMS_ENABLE";

#[test]
fn peer_systems_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(GATE);
    assert!(
        !peer_systems_emit_enabled(),
        "CONTRACT: {GATE} MUST default OFF (wire-dark per E-21)"
    );
}

#[test]
fn peer_systems_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "1");
    let on = peer_systems_emit_enabled();
    std::env::remove_var(GATE);
    assert!(
        on,
        "CONTRACT: {GATE}=1 MUST open the PeerSystemsEvent emission gate"
    );
}

#[test]
fn critic_rerank_picks_argmax_candidate() {
    // ON-path compute: the reranker MUST select the highest-scored
    // candidate. This is the load-bearing decision a wired rerank round
    // makes; the kernel is the off-agent observable.
    let candidates = vec![
        CriticCandidate {
            patch: "patch-a".to_string(),
            runtime_label: "local".to_string(),
            score: 0.30,
        },
        CriticCandidate {
            patch: "patch-b".to_string(),
            runtime_label: "cloud".to_string(),
            score: 0.91,
        },
        CriticCandidate {
            patch: "patch-c".to_string(),
            runtime_label: "local".to_string(),
            score: 0.55,
        },
    ];
    let outcome = CriticRerank::pick_argmax(&candidates);
    let pick = outcome.pick.expect("argmax over 3 candidates MUST be Some");
    assert_eq!(
        pick.patch, "patch-b",
        "CONTRACT: argmax MUST pick the 0.91 candidate (patch-b); got {:?}",
        pick.patch
    );
    assert_eq!(outcome.considered, 3, "all candidates considered");
    assert!(
        (outcome.max_score - 0.91).abs() < f64::EPSILON,
        "CONTRACT: max_score MUST surface 0.91; got {}",
        outcome.max_score
    );
}

#[test]
fn critic_rerank_empty_input_picks_none() {
    let outcome = CriticRerank::pick_argmax(&[]);
    assert!(
        outcome.pick.is_none(),
        "CONTRACT: empty candidate set MUST yield pick=None (rerank-skipped)"
    );
    assert_eq!(outcome.considered, 0);
}
