// Track A.5 contract test: PhaseNode / StepNode / BabyStepSpec serde tolerance.
//
// RED STATE: PhaseNode.name, StepNode.description, BabyStepSpec.task are all
// strict fields (no #[serde(default)]). Deserializing JSON with any of these
// missing currently returns a serde error. This test asserts they SUCCEED —
// they will be RED until execution agent #2 adds #[serde(default)] or
// deserialize_with helpers for each strict field.
//
// BabyStepSpec.grounding is also strict (required BabyStepGrounding with no
// Option wrapper and no #[serde(default)]).
//
// NOTE: EvidenceLevel already derives Default (#[default] = Anecdotal at
// research_tree.rs:271), so StepNode.evidence_level tests may be GREEN today.
//
// NOTE: Context-package warns that the actual live-path parse failure at
// level=Phase may originate in grounding_client.rs parsing ResearchReport from
// the sidecar, not from direct PhaseNode deserialization. However the A.5 spec
// is clear: these structs must tolerate missing optional fields for any caller
// that deserializes them. These tests verify the struct-level serde contract.
//
// ID serialization: PhaseId(u64), StepId(u64), BabyStepId(u64) are newtype
// tuple structs. serde_json default derive for newtype(T) serializes as plain T
// (not as [N]). verify_phaseid_json_format.py confirmed: plain u64, not sequence.

use elai_orchestrator::research_tree::{BabyStepSpec, PhaseNode, StepNode};

// ---------------------------------------------------------------------------
// A.5.a — PhaseNode deserializes successfully when `name` is absent.
//
// RED: PhaseNode.name is a strict String field with no serde(default).
// ---------------------------------------------------------------------------

#[test]
fn phase_node_tolerates_missing_name() {
    // PhaseId is a newtype tuple struct. serde_json default derive serializes
    // newtype(T) as plain T, not as [N]. Verified by verify_phaseid_json_format.py.
    let json = r#"{
        "id": 1,
        "steps": [],
        "depends_on": [],
        "research": null,
        "is_parallel": false
    }"#;

    let result: Result<PhaseNode, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "PhaseNode must deserialize successfully when 'name' is absent; \
         expected #[serde(default)] or deserialize_with helper on the 'name' field. \
         got err: {:?}",
        result.err(),
    );
    // After the fix, name must be a valid non-panicking default (empty or "unnamed").
    // We just assert the deserialization succeeded — the exact default is implementation detail.
}

// ---------------------------------------------------------------------------
// A.5.b — StepNode deserializes successfully when `description` is absent.
//
// RED: StepNode.description is a strict String with no serde(default).
// ---------------------------------------------------------------------------

#[test]
fn step_node_tolerates_missing_description() {
    // evidence_level has #[default] on EvidenceLevel, so "Anecdotal" is what
    // the field defaults to when absent — that may already work. We omit it to
    // simultaneously test both description and evidence_level tolerances.
    let json = r#"{
        "id": 2,
        "baby_steps": []
    }"#;

    let result: Result<StepNode, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "StepNode must deserialize successfully when 'description' and 'evidence_level' are absent; \
         expected #[serde(default)] on both fields. got err: {:?}",
        result.err(),
    );
}

// ---------------------------------------------------------------------------
// A.5.c — StepNode with description present but evidence_level absent.
//
// This is expected to be GREEN today if EvidenceLevel::default() works
// and serde(default) is added. If already green, it acts as a regression guard.
// ---------------------------------------------------------------------------

#[test]
fn step_node_tolerates_missing_evidence_level() {
    let json = r#"{
        "id": 3,
        "description": "A concrete step",
        "baby_steps": []
    }"#;

    let result: Result<StepNode, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "StepNode must deserialize when evidence_level is absent \
         (EvidenceLevel derives Default = Anecdotal). got err: {:?}",
        result.err(),
    );
}

// ---------------------------------------------------------------------------
// A.5.d — BabyStepSpec deserializes successfully when `task` is absent.
//
// RED: BabyStepSpec.task is a strict String with no serde(default).
// ---------------------------------------------------------------------------

#[test]
fn baby_step_spec_tolerates_missing_task() {
    // Include grounding to isolate the `task` field tolerance.
    let json = r#"{
        "id": 4,
        "estimated_tokens": 0,
        "grounding": {
            "concept_name": "",
            "phase_name": "",
            "step_name": "",
            "top_citations": [],
            "compressed_tokens": 0
        },
        "tdd": null,
        "acceptance": null
    }"#;

    let result: Result<BabyStepSpec, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "BabyStepSpec must deserialize successfully when 'task' is absent; \
         expected #[serde(default)] on the 'task' field. got err: {:?}",
        result.err(),
    );
}

// ---------------------------------------------------------------------------
// A.5.e — BabyStepSpec deserializes successfully when `grounding` is absent.
//
// RED: BabyStepSpec.grounding is a required BabyStepGrounding struct with no
// Option wrapper and no #[serde(default)]. Requires either wrapping in
// Option<BabyStepGrounding> or impl Default for BabyStepGrounding.
// ---------------------------------------------------------------------------

#[test]
fn baby_step_spec_tolerates_missing_grounding() {
    let json = r#"{
        "id": 5,
        "task": "Do something concrete",
        "estimated_tokens": 50,
        "tdd": null,
        "acceptance": null
    }"#;

    let result: Result<BabyStepSpec, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "BabyStepSpec must deserialize when 'grounding' is absent; \
         expected #[serde(default)] on 'grounding' with Default impl for BabyStepGrounding, \
         OR wrap in Option<BabyStepGrounding>. got err: {:?}",
        result.err(),
    );
}

// ---------------------------------------------------------------------------
// A.5.f — PhaseNode roundtrip with all required fields present (baseline
//          smoke test to confirm the import paths compile correctly).
// ---------------------------------------------------------------------------

#[test]
fn phase_node_full_roundtrip_baseline() {
    let json = r#"{
        "id": 0,
        "name": "scaffold",
        "steps": [],
        "depends_on": [],
        "research": null,
        "is_parallel": false
    }"#;

    let phase: PhaseNode = serde_json::from_str(json)
        .expect("PhaseNode with all fields must always deserialize (baseline)");
    assert_eq!(phase.name, "scaffold", "name must be preserved");
    assert!(phase.steps.is_empty(), "steps must be empty");
}
