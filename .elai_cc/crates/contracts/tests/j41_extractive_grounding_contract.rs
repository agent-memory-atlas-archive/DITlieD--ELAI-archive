//! J-41 T1 — extractive grounding pass contract (falsification gate).
//!
//! T1-T3: run_extractive_grounding unit correctness (exact, fuzzy, zero-span).
//! T4: prebind_research_answer carries extractive_grounding (FALSIFICATION:
//!     delete the run_extractive_grounding call inside prebind -> T4 goes RED).
//! T5: observe-only — has_ghost unchanged, zero_span empty when all claims supported.

use elai_core::research_types::{
    AnswerConfidence, ExtractedFact, ResearchAnswer, VerifiedClaim, VerificationLevel,
};
use elai_core::source_tier::{ClassificationMethod, SourceClassification, SourceTier};
use elai_orchestrator::{prebind_research_answer, run_extractive_grounding};

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
fn t1_run_extractive_grounding_exact_match() {
    let claim = "The function returns an error when the input is empty and the buffer has not been initialized";
    let evidence = "The function returns an error when the input is empty and the buffer has not been initialized before use.";

    let result = run_extractive_grounding(claim, &[evidence]);

    assert_eq!(result.total_claims, 1, "T1: should detect 1 claim");
    assert_eq!(result.grounded_claims, 1, "T1: claim should be grounded");
    assert!(
        result.zero_span_claims.is_empty(),
        "T1: no zero-span claims expected"
    );
    assert_eq!(
        result.tier1_exact_matches, 1,
        "T1: should be a tier-1 exact match"
    );
}

#[test]
fn t2_run_extractive_grounding_fuzzy_match() {
    let claim = "The system uses a token-based authentication mechanism for API requests";
    let evidence = "A token based auth mechanism is used by the system for all API request authentication.";

    let result = run_extractive_grounding(claim, &[evidence]);

    assert_eq!(
        result.grounded_claims, 1,
        "T2: claim should be grounded"
    );
    assert!(
        result.zero_span_claims.is_empty(),
        "T2: no zero-span claims expected"
    );
    assert_eq!(
        result.tier1_exact_matches + result.tier2_fuzzy_matches + result.tier3_keyword_matches,
        1,
        "T2: exactly one tier should match"
    );
}

#[test]
fn t3_run_extractive_grounding_zero_span() {
    let claim = "The quantum entanglement protocol ensures faster-than-light communication";
    let evidence = "The system uses HTTP for network communication.";

    let result = run_extractive_grounding(claim, &[evidence]);

    assert_eq!(
        result.grounded_claims, 0,
        "T3: claim should NOT be grounded"
    );
    assert_eq!(
        result.zero_span_claims.len(),
        1,
        "T3: exactly one zero-span claim expected"
    );
    assert!(
        result.zero_span_claims[0]
            .claim_text
            .contains("quantum"),
        "T3: zero-span claim text should contain 'quantum'"
    );
}

#[test]
fn t4_prebind_research_answer_carries_extractive_grounding() {
    let exact_quote = "The function returns an error when the input is empty and the buffer has not been initialized before use";
    let answer_text = format!(
        "{}. The quantum entanglement protocol ensures faster than light communication.",
        "The function returns an error when the input is empty and the buffer has not been initialized"
    );

    let answer = make_answer(&answer_text, exact_quote);
    let prebind = prebind_research_answer(&answer);

    let eg = prebind
        .extractive_grounding
        .as_ref()
        .expect("T4: extractive_grounding must be Some on the normal path");

    assert!(
        eg.total_claims >= 2,
        "T4: at least 2 claims expected, got {}",
        eg.total_claims
    );
    assert!(
        !eg.zero_span_claims.is_empty(),
        "T4: at least 1 zero-span claim expected (the quantum sentence has no evidence support)"
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
        "T5: has_ghost must be false (observe-only, no ghost citations)"
    );

    let eg = prebind
        .extractive_grounding
        .as_ref()
        .expect("T5: extractive_grounding must be Some");
    assert!(
        eg.zero_span_claims.is_empty(),
        "T5: zero_span_claims must be empty when all claims are supported"
    );
}
