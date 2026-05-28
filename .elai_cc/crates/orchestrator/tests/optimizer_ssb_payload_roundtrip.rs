//! E-8 integration test: PromptOptimizationCompletedPayload round-trips
//! through serde for each of the 7 OptimizerPhase discriminator arms.
//! Pattern-2 single-variant precedent: ssb.rs DarwinianGenerationCompleted
//! covers the 11-arm DarwinianPhase the same way (E-7 ship).

use elai_orchestrator::ssb::{
    OptimizerPhase, PromptOptimizationCompletedPayload, SsbActionType, SsbPayload,
};

fn payload(phase: OptimizerPhase) -> PromptOptimizationCompletedPayload {
    PromptOptimizationCompletedPayload {
        phase,
        iteration_index: 1,
        run_id_prefix: [0xE8; 8],
        template_blake3_prefix: [0xE8; 8],
        val_lift_pct: Some(6.0),
        held_out_lift_pct: Some(5.0),
        reason_hash: None,
        ts_ms: 1_700_000_162_000,
    }
}

#[test]
fn all_seven_optimizer_phases_serde_roundtrip() {
    let phases = [
        OptimizerPhase::DspyInit,
        OptimizerPhase::OproStep,
        OptimizerPhase::RankPromptVote,
        OptimizerPhase::ApeProposal,
        OptimizerPhase::GepaReflect,
        OptimizerPhase::ArchivePromote,
        OptimizerPhase::ArchiveRetire,
    ];
    for p in phases {
        let pl = payload(p);
        let json = serde_json::to_string(&pl).expect("serialise");
        let back: PromptOptimizationCompletedPayload =
            serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back, pl);
    }
}

#[test]
fn action_type_maps_via_ssbpayload_envelope() {
    let pl = payload(OptimizerPhase::GepaReflect);
    let envelope = SsbPayload::PromptOptimizationCompleted(pl);
    assert_eq!(
        envelope.action_type(),
        SsbActionType::PromptOptimizationCompleted
    );
}

#[test]
fn phase_as_str_stable_wire_tags() {
    assert_eq!(OptimizerPhase::DspyInit.as_str(), "dspy_init");
    assert_eq!(OptimizerPhase::OproStep.as_str(), "opro_step");
    assert_eq!(OptimizerPhase::RankPromptVote.as_str(), "rank_prompt_vote");
    assert_eq!(OptimizerPhase::ApeProposal.as_str(), "ape_proposal");
    assert_eq!(OptimizerPhase::GepaReflect.as_str(), "gepa_reflect");
    assert_eq!(OptimizerPhase::ArchivePromote.as_str(), "archive_promote");
    assert_eq!(OptimizerPhase::ArchiveRetire.as_str(), "archive_retire");
}

#[test]
fn action_type_sql_str_pinned() {
    assert_eq!(
        SsbActionType::PromptOptimizationCompleted.as_sql_str(),
        "prompt_optimization_completed"
    );
}
