//! Contract tests for B-23 Phase 4H Group B — self_mod_proposal.rs.
//!
//! Verifies the log-driven SelfModProposal builder reads SSB rows for
//! evidence, attaches reasoning_trace evidence, and the routing function
//! dispatches Class A (DataTable) vs Class B (Parameter) correctly.

use elai_core::clock::OsClock;
use elai_orchestrator::self_mod_proposal::{
    route_proposal, ProposalClass, ProposalEngine,
};
use elai_orchestrator::{
    AllowedConfigKey, DataTable, PipelineMode, ProposedChange, ReasoningTraceRef,
    SelfModProposal, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader, SsbWriteRequest,
};
use elai_orchestrator::ssb::ReasoningTracePayload;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn proposal_with(key: AllowedConfigKey, change: ProposedChange) -> SelfModProposal {
    SelfModProposal {
        variant_id: "v1".into(),
        parent_id: "v0".into(),
        target_config_key: key,
        proposed_change: change,
        ssb_evidence: vec!["seq:1:reasoning_trace".into()],
        reasoning_trace_evidence: vec![],
        rationale: "test".into(),
        scope: None,
    }
}

#[test]
fn route_data_table_to_layer_one_a() {
    let p = proposal_with(
        AllowedConfigKey::DataTable(DataTable::DeprecatedPatterns),
        ProposedChange::InsertRow {
            table: "deprecated_patterns".into(),
            row: vec![1, 2, 3],
        },
    );
    assert_eq!(route_proposal(&p), ProposalClass::LayerOneA);
}

#[test]
fn route_parameter_to_staged_eval() {
    let p = proposal_with(
        AllowedConfigKey::Parameter("etc_threshold".into()),
        ProposedChange::UpdateKey {
            key: "etc_threshold".into(),
            old_value: "0.6".into(),
            new_value: "0.65".into(),
        },
    );
    assert_eq!(route_proposal(&p), ProposalClass::StagedEval);
}

#[test]
fn proposal_class_distinguishes_handlers() {
    // Sanity: the two classes must be distinct (catch any collapse to a single
    // variant).
    assert_ne!(ProposalClass::LayerOneA, ProposalClass::StagedEval);
}

// ---------------------------------------------------------------------------
// Log-driven proposal builder (reads SSB)
// ---------------------------------------------------------------------------

async fn ssb_with_traces(traces: Vec<(String, String, String)>) -> (TempDir, Arc<Ssb>) {
    // traces: Vec<(agent_id, step_type, grounding_level)>
    // GROUNDED traces use budget_remaining=5 + non-failure summary; UNGROUNDED
    // traces use budget_remaining=0 + "stale grounding" summary so the
    // proposal engine treats them as evidence per its failure-detection rules.
    let tmp = tempfile::tempdir().unwrap();
    let db_path: PathBuf = tmp.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "proj".into(),
        session_id: "sess".into(),
    };
    let ssb = Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init");
    for (i, (agent, step_type, grounding)) in traces.iter().enumerate() {
        let is_grounded = grounding.eq_ignore_ascii_case("GROUNDED");
        let payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
            agent_id: agent.clone(),
            step_index: i as u32,
            step_type: step_type.clone(),
            content_summary: if is_grounded { "ok".into() } else { "stale grounding".into() },
            grounding_source: "training".into(),
            grounding_level: grounding.clone(),
            confidence: 0.9,
            elapsed_ms: 100,
            topic: "phase".into(),
            budget_remaining: if is_grounded { 5 } else { 0 },
        });
        let bytes = rmp_serde::to_vec_named(&payload).expect("serialize");
        ssb.write(SsbWriteRequest {
            action_type: SsbActionType::ReasoningTrace,
            agent_id: agent.clone(),
            superstep: 0,
            payload: bytes,
            files: vec![],
        })
        .await
        .expect("write");
    }
    (tmp, Arc::new(ssb))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_proposals_emits_at_least_one_with_evidence() {
    // 3 ungrounded reasoning rows on the same agent => at least one proposal
    // with non-empty ssb_evidence + reasoning_trace_evidence.
    let traces: Vec<_> = (0..3)
        .map(|i| ("executor".to_string(), format!("observe-{i}"), "UNGROUNDED".to_string()))
        .collect();
    let (_tmp, ssb) = ssb_with_traces(traces).await;
    let reader: Arc<dyn SsbReader + Send + Sync> = ssb.clone();

    let engine = ProposalEngine::new(reader, vec!["executor".into()]);
    let proposals = engine.build_proposals(PipelineMode::Surgical).await.unwrap();
    assert!(!proposals.is_empty(), "expected at least one proposal");
    for p in &proposals {
        assert!(!p.ssb_evidence.is_empty(), "ssb_evidence must be non-empty (spec 4H.4)");
    }
    // At least one proposal carries reasoning_trace_evidence.
    let have_trace = proposals.iter().any(|p| !p.reasoning_trace_evidence.is_empty());
    assert!(have_trace, "expected reasoning_trace_evidence on at least one proposal");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_proposals_skips_grounded_traces() {
    // Pure grounded traces produce no proposals — no failure to learn from.
    let traces: Vec<_> = (0..3)
        .map(|i| ("executor".to_string(), format!("observe-{i}"), "GROUNDED".to_string()))
        .collect();
    let (_tmp, ssb) = ssb_with_traces(traces).await;
    let reader: Arc<dyn SsbReader + Send + Sync> = ssb.clone();
    let engine = ProposalEngine::new(reader, vec!["executor".into()]);
    let proposals = engine.build_proposals(PipelineMode::Surgical).await.unwrap();
    assert!(proposals.is_empty(), "grounded traces should not produce proposals");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn proposals_attach_pipeline_mode_scope() {
    let traces: Vec<_> = (0..3)
        .map(|i| ("executor".to_string(), format!("observe-{i}"), "UNGROUNDED".to_string()))
        .collect();
    let (_tmp, ssb) = ssb_with_traces(traces).await;
    let reader: Arc<dyn SsbReader + Send + Sync> = ssb.clone();
    let engine = ProposalEngine::new(reader, vec!["executor".into()]);
    let proposals = engine.build_proposals(PipelineMode::Greenfield).await.unwrap();
    for p in proposals {
        if let Some(scope) = p.scope {
            assert_eq!(scope.pipeline_mode, Some(PipelineMode::Greenfield));
        }
    }
}

#[test]
fn reasoning_trace_ref_minimum_fields() {
    let r = ReasoningTraceRef {
        agent_id: "executor".into(),
        phase: "phase-1".into(),
        grc_step_index: 4,
        step_type: "observe".into(),
        failure_mode: "stale_grounding".into(),
    };
    assert_eq!(r.grc_step_index, 4);
    assert_eq!(r.step_type, "observe");
}
