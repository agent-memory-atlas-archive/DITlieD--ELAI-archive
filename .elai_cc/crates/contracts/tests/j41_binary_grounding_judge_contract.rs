use elai_core::research_types::{
    AnswerConfidence, ExtractedFact, ResearchAnswer, VerifiedClaim, VerificationLevel,
};
use elai_core::source_tier::{ClassificationMethod, SourceClassification, SourceTier};
use elai_memory::{BinaryGroundingJudge, NliJudge, NliLabel};
use elai_orchestrator::{judge_zero_span_claims, prebind_research_answer};

fn make_answer(answer_text: &str, exact_quote: &str) -> ResearchAnswer {
    ResearchAnswer {
        question: "test question".to_owned(),
        answer_text: answer_text.to_owned(),
        evidence_chain: vec![VerifiedClaim {
            claim: "test claim".to_owned(),
            supporting_facts: vec![ExtractedFact {
                id: "test-fact-id".to_owned(),
                claim: "test fact claim".to_owned(),
                source_url: "https://example.com/source".to_owned(),
                source_tier: SourceTier::T1Authoritative,
                exact_quote: Some(exact_quote.to_owned()),
                page_section: None,
                extraction_confidence: 1.0,
            }],
            verification_level: VerificationLevel::Verified,
            independent_source_count: 1,
        }],
        confidence: AnswerConfidence::High,
        caveats: Vec::new(),
        unanswered_parts: Vec::new(),
        sources_consulted: vec![SourceClassification {
            url: "https://example.com/source".to_owned(),
            domain: "example.com".to_owned(),
            tier: SourceTier::T1Authoritative,
            classification_method: ClassificationMethod::Default,
        }],
        sources_rejected: Vec::new(),
        path: "test".to_owned(),
    }
}

#[test]
fn t1_binary_grounding_judge_supported_claim() {
    let claim = "The function returns an error when the input is empty";
    let evidence = "The function returns an error when the input is empty.";

    let judge = BinaryGroundingJudge::new();
    let verdict = judge.classify(claim, evidence).expect("T1: classify must succeed");

    assert_eq!(
        verdict.label,
        NliLabel::Entailment,
        "T1: supported claim must map to Entailment"
    );
    assert_eq!(
        verdict.confidence, 1.0,
        "T1: binary mapping confidence must be 1.0"
    );
}

#[test]
fn t2_binary_grounding_judge_unsupported_claim() {
    let claim = "The quantum entanglement protocol ensures faster than light communication";
    let evidence = "The system uses HTTP for network communication.";

    let judge = BinaryGroundingJudge::new();
    let verdict = judge.classify(claim, evidence).expect("T2: classify must succeed");

    assert_eq!(
        verdict.label,
        NliLabel::Neutral,
        "T2: unsupported claim must map to Neutral"
    );
    assert_eq!(
        verdict.confidence, 1.0,
        "T2: binary mapping confidence must be 1.0"
    );
}

#[test]
fn t3_judge_zero_span_claims_over_claims() {
    let claims: Vec<(&str, usize)> = vec![
        ("The quantum protocol ensures faster than light communication", 0),
        ("The system uses HTTP for network communication", 1),
    ];
    let evidence = vec!["The system uses HTTP for network communication."];

    let result = judge_zero_span_claims(&claims, &evidence);

    assert_eq!(
        result.zero_span_verdicts.len(),
        2,
        "T3: should produce 2 verdicts"
    );
    assert!(
        result.unsupported_count >= 1,
        "T3: at least 1 unsupported (the quantum claim)"
    );
    assert!(
        result.supported_count >= 1,
        "T3: at least 1 supported (the HTTP claim)"
    );
}

#[test]
fn t4_prebind_research_answer_carries_binary_grounding() {
    let exact_quote = "The function returns an error when the input is empty and the buffer has not been initialized before use";
    let answer_text = format!(
        "{}. The quantum entanglement protocol ensures faster than light communication.",
        "The function returns an error when the input is empty and the buffer has not been initialized"
    );

    let answer = make_answer(&answer_text, exact_quote);
    let prebind = prebind_research_answer(&answer);

    let bg = prebind
        .binary_grounding
        .as_ref()
        .expect("T4: binary_grounding must be Some when zero-span claims exist");

    assert!(
        bg.zero_span_verdicts.len() >= 1,
        "T4: at least 1 zero-span verdict expected"
    );
    assert!(
        bg.unsupported_count >= 1,
        "T4: at least 1 unsupported verdict expected"
    );
}

#[test]
fn t5_prebind_research_answer_observe_only_no_ghost_change() {
    let exact_quote = "The function returns an error when the input is empty and the buffer has not been initialized before use";
    let answer_text =
        "The function returns an error when the input is empty and the buffer has not been initialized.";

    let answer = make_answer(answer_text, exact_quote);
    let prebind = prebind_research_answer(&answer);

    assert!(
        !prebind.has_ghost,
        "T5: has_ghost must be false (observe-only)"
    );
    assert!(
        prebind.binary_grounding.is_none(),
        "T5: binary_grounding must be None when no zero-span claims exist"
    );
}
