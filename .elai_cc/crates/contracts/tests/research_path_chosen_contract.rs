// Track A.2 contract test: research_path_chosen SSB variant + ResearchAnswer.path field.
//
// RED STATE: the following symbols do NOT yet exist and will cause compile
// errors until execution agent #2 implements A.2:
//   - SsbActionType::ResearchPathChosen (enum variant absent)
//   - SsbPayload::ResearchPathChosen(ResearchPathChosenPayload) (variant absent)
//   - ResearchPathChosenPayload (struct absent from ssb.rs)
//   - ResearchAnswer::path (field absent from elai-core/src/research_types.rs)
//
// When A.2 production code lands all compile errors resolve and all tests
// will turn green.
//
// Test strategy: two pure-struct msgpack round-trip tests assert the new SSB
// variant shape; two tokio tests drive phase_f_synthesize directly with
// CannedSynthesisGroundingClient / SyntheticGroundingClient to verify the
// path field on ResearchAnswer. No live sidecar required.

use elai_orchestrator::grounding_client::{
    CannedSynthesisGroundingClient, SyntheticGroundingClient,
};
use elai_orchestrator::research_pipeline::{phase_f_synthesize, ClassifiedResults};
use elai_orchestrator::ssb::{ResearchPathChosenPayload, SsbActionType, SsbPayload};
use elai_core::research_types::{ResearchAnswer, ResearchQuery, VerificationLevel, VerifiedClaim};

// ---------------------------------------------------------------------------
// A.2.a — ResearchPathChosenPayload struct and SsbActionType variant
//          round-trip through msgpack.
//
// RED: struct and variant do not exist yet.
// ---------------------------------------------------------------------------

#[test]
fn research_path_chosen_payload_roundtrips_msgpack() {
    let payload = ResearchPathChosenPayload {
        path: "llm".into(),
        reason: "cited_prose".into(),
        ts_ms: 1_000_000,
    };
    let wrapped = SsbPayload::ResearchPathChosen(payload);
    assert_eq!(
        wrapped.action_type(),
        SsbActionType::ResearchPathChosen,
        "action_type() must be ResearchPathChosen",
    );

    let bytes = rmp_serde::to_vec_named(&wrapped)
        .expect("ResearchPathChosen msgpack serialize");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("ResearchPathChosen msgpack deserialize");

    match decoded {
        SsbPayload::ResearchPathChosen(p) => {
            assert_eq!(p.path, "llm", "path must survive round-trip");
            assert_eq!(p.reason, "cited_prose", "reason must survive round-trip");
            assert_eq!(p.ts_ms, 1_000_000, "ts_ms must survive round-trip");
        }
        other => panic!("expected ResearchPathChosen variant; got {other:?}"),
    }
}

#[test]
fn research_path_chosen_deterministic_fallback_roundtrips() {
    let payload = ResearchPathChosenPayload {
        path: "deterministic_fallback".into(),
        reason: "no_citation_markers".into(),
        ts_ms: 42,
    };
    let wrapped = SsbPayload::ResearchPathChosen(payload);
    let bytes = rmp_serde::to_vec_named(&wrapped)
        .expect("ResearchPathChosen fallback msgpack serialize");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("ResearchPathChosen fallback msgpack deserialize");

    match decoded {
        SsbPayload::ResearchPathChosen(p) => {
            assert_eq!(p.path, "deterministic_fallback");
            assert_eq!(p.reason, "no_citation_markers");
        }
        other => panic!("expected ResearchPathChosen; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// A.2.b — ResearchAnswer.path field is "llm" when CannedSynthesisGroundingClient
//          returns cited prose through phase_f_synthesize.
//
// RED: ResearchAnswer.path does not exist yet.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn research_answer_path_is_llm_when_synthesis_succeeds() {
    // Cited prose with at least one [N] marker — triggers the "llm" branch in
    // phase_f_synthesize (confirmed at research_pipeline.rs:458).
    let cited_prose = "Hash tables use an array with a hash function. [1] They offer O(1) average lookup. [2]".to_owned();
    let grounding = CannedSynthesisGroundingClient::new(cited_prose);

    let query = ResearchQuery {
        question: "What is a hash table?".into(),
        context: None,
        max_sources: 5,
        required_tier: None,
    };

    // Build a non-empty VerifiedClaim slice so phase_f does not short-circuit
    // at the no_claims branch (research_pipeline.rs:415-427).
    let claims = vec![VerifiedClaim {
        claim: "Hash tables offer O(1) lookup".into(),
        supporting_facts: vec![],
        verification_level: VerificationLevel::Supported,
        independent_source_count: 1,
    }];
    let classified = ClassifiedResults {
        usable: vec![],
        rejected: vec![],
        classification_errors: vec![],
    };

    let answer: ResearchAnswer = phase_f_synthesize(&query, &claims, &classified, &grounding).await;

    // RED: ResearchAnswer.path does not exist yet.
    assert_eq!(
        answer.path, "llm",
        "ResearchAnswer.path must be 'llm' when CannedSynthesisGroundingClient returns cited prose",
    );
}

// ---------------------------------------------------------------------------
// A.2.c — ResearchAnswer.path is "deterministic_fallback" when synthesize_text
//          returns EmptyResponse (SyntheticGroundingClient default).
//
// RED: ResearchAnswer.path does not exist yet.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn research_answer_path_is_deterministic_fallback_when_synthesis_fails() {
    // SyntheticGroundingClient.synthesize_text always returns EmptyResponse
    // which forces phase_f into the deterministic_fallback branch.
    let grounding = SyntheticGroundingClient::new();

    let query = ResearchQuery {
        question: "What is a Bloom filter?".into(),
        context: None,
        max_sources: 5,
        required_tier: None,
    };

    let claims = vec![VerifiedClaim {
        claim: "Bloom filter is probabilistic".into(),
        supporting_facts: vec![],
        verification_level: VerificationLevel::Supported,
        independent_source_count: 1,
    }];
    let classified = ClassifiedResults {
        usable: vec![],
        rejected: vec![],
        classification_errors: vec![],
    };

    let answer: ResearchAnswer = phase_f_synthesize(&query, &claims, &classified, &grounding).await;

    // RED: ResearchAnswer.path does not exist yet.
    assert_eq!(
        answer.path, "deterministic_fallback",
        "ResearchAnswer.path must be 'deterministic_fallback' when synthesize_text returns error",
    );
}
