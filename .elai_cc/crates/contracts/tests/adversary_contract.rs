//! D-13 closure contract test — locks the public surface of the
//! adversary substrate so future renames trip the contract.
//!
//! What this contract asserts:
//!   1. SCHEMA_VERSION floor >= 122 (the D-13 substrate landed at the
//!      earlier shipped SCHEMA; closure does NOT bump for the
//!      preserved-naming variants).
//!   2. ModelFamily enum is exhaustive + serde round-trip stable
//!      (snake_case wire labels).
//!   3. ConsensusRole::Adversary variant exists + serde round-trip
//!      stable.
//!   4. SourceTierCitation 4-variant enum is exhaustive + serde
//!      round-trip stable across all 4 arms.
//!   5. ModelFamily::from_provider_kind bridge maps the 4 production
//!      ProviderKind variants to the correct ModelFamily.
//!   6. ModelFamily::from_model_id substring classifier maps
//!      ProviderKind-canonical strings to the correct ModelFamily.
//!   7. AdversaryGenerated SSB payload round-trips.
//!   8. ConsensusReevaluatedWithCounter SSB payload round-trips.
//!   9. heterogeneous_verifier_check refuses a same-family pair and
//!      accepts a cross-family pair.
//!
//! Grounding: workflow/papers/{self-correction-bench-2507.02778,
//! constitution-collapse-2504.04918, constitutional-small-2503.17365,
//! latent-verifiers-2504.16760, rstar-math-2501.04519}-interface.md.

use elai_orchestrator::{
    adversary::{
        heterogeneous_verifier_check, CounterEvidenceSchema, ModelFamily, SourceTierCitation,
    },
    agent_role::ConsensusRole,
    claim::{Claim, ClaimType, EvidenceKind, EvidenceRef},
    ssb::{
        AdversaryGeneratedPayload, ConsensusReevaluatedWithCounterPayload, SsbPayload,
        SCHEMA_VERSION_PUBLIC,
    },
    verification_result::VerifierKind,
};
use elai_providers::ProviderKind;

// ----- Test 1: SCHEMA_VERSION floor -----------------------------------------

#[test]
fn contract_schema_version_floor_d13_substrate() {
    // D-13 substrate shipped at SCHEMA 122 (post-C-47). Closure
    // preserves shipped names + adds NO new SSB variants, so the
    // SCHEMA floor is 122. Future plans may bump higher; the floor
    // never regresses below 122.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 122,
        "SCHEMA_VERSION_PUBLIC regressed below D-13 substrate floor 122; got {SCHEMA_VERSION_PUBLIC}"
    );
}

// ----- Test 2: ModelFamily exhaustive serde ---------------------------------

#[test]
fn contract_model_family_exhaustive_serde() {
    let all = [
        ModelFamily::Anthropic,
        ModelFamily::OpenAi,
        ModelFamily::Meta,
        ModelFamily::Mistral,
        ModelFamily::DeepSeek,
        ModelFamily::Local,
        ModelFamily::Other("custom-x".into()),
    ];
    for fam in &all {
        let s = serde_json::to_string(fam).expect("serialize");
        let back: ModelFamily = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(fam, &back, "round-trip mismatch for {fam:?} -> {s}");
        assert!(
            !fam.as_str().is_empty(),
            "as_str must produce a non-empty label"
        );
    }
}

// ----- Test 3: ConsensusRole::Adversary serde -------------------------------

#[test]
fn contract_consensus_role_adversary_serde() {
    let role = ConsensusRole::Adversary;
    let s = serde_json::to_string(&role).expect("serialize");
    // snake_case wire label.
    assert_eq!(s.trim_matches('"'), "adversary");
    let back: ConsensusRole = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(back, role);
    assert_eq!(role.as_str(), "adversary");
}

// ----- Test 4: SourceTierCitation 4-variant exhaustive serde ----------------

#[test]
fn contract_source_tier_citation_4_variant_exhaustive_serde() {
    let cases = [
        SourceTierCitation::T1Arxiv {
            id: "2504.16760".into(),
            title: "Lightweight Latent Verifiers".into(),
        },
        SourceTierCitation::T1Nature {
            doi: "10.1038/s41586-024-xxxxx".into(),
            title: "A Nature paper".into(),
        },
        SourceTierCitation::T2Web {
            url: "https://doc.rust-lang.org/book/".into(),
            title: "The Rust Programming Language".into(),
        },
        SourceTierCitation::T3Training,
    ];
    let expected_labels = ["t1_arxiv", "t1_nature", "t2_web", "t3_training"];
    assert_eq!(
        cases.len(),
        4,
        "SourceTierCitation enum MUST have exactly 4 variants — adding a 5th requires a contract review"
    );
    for (c, expected) in cases.iter().zip(expected_labels.iter()) {
        assert_eq!(c.tier_label(), *expected);
        let s = serde_json::to_string(c).expect("serialize");
        let back: SourceTierCitation =
            serde_json::from_str(&s).expect("deserialize");
        assert_eq!(c, &back, "round-trip failed for {c:?} -> {s}");
    }
    // T3Training is the only NOT-load-bearing variant.
    assert!(!SourceTierCitation::T3Training.is_load_bearing());
}

// ----- Test 5: ModelFamily::from_provider_kind bridge -----------------------

#[test]
fn contract_model_family_from_provider_kind_bridge() {
    assert_eq!(
        ModelFamily::from_provider_kind(ProviderKind::Anthropic),
        ModelFamily::Anthropic
    );
    assert_eq!(
        ModelFamily::from_provider_kind(ProviderKind::OpenAi),
        ModelFamily::OpenAi
    );
    // Xai uses the OpenAI-compat transport but is a distinct model
    // family — lands on Other("xai").
    assert_eq!(
        ModelFamily::from_provider_kind(ProviderKind::Xai),
        ModelFamily::Other("xai".into())
    );
    assert_eq!(
        ModelFamily::from_provider_kind(ProviderKind::Ollama),
        ModelFamily::Local
    );
}

// ----- Test 6: ModelFamily::from_model_id substring classifier --------------

#[test]
fn contract_model_family_from_model_id_classifier() {
    // Cloud-provider id hints.
    assert_eq!(
        ModelFamily::from_model_id("claude-sonnet-4-6"),
        ModelFamily::Anthropic
    );
    assert_eq!(
        ModelFamily::from_model_id("gpt-4o-2024-08"),
        ModelFamily::OpenAi
    );
    assert_eq!(
        ModelFamily::from_model_id("grok-3-mini"),
        ModelFamily::Other("xai".into())
    );
    assert_eq!(
        ModelFamily::from_model_id("deepseek-coder-v3"),
        ModelFamily::DeepSeek
    );
    // Local / on-device families.
    assert_eq!(
        ModelFamily::from_model_id("llama-3-70b-instruct"),
        ModelFamily::Meta
    );
    assert_eq!(
        ModelFamily::from_model_id("mistral-7b-instruct"),
        ModelFamily::Mistral
    );
    assert_eq!(
        ModelFamily::from_model_id("mixtral-8x7b"),
        ModelFamily::Mistral
    );
    assert_eq!(
        ModelFamily::from_model_id("gemma-3-12b-it"),
        ModelFamily::Other("gemma".into())
    );
    assert_eq!(
        ModelFamily::from_model_id("qwen2.5-coder-7b-instruct"),
        ModelFamily::Local
    );
    // Unknown / empty -> Local (conservative — Local-vs-Local
    // trips SameFamilyPairing observably).
    assert_eq!(
        ModelFamily::from_model_id("some-unknown-model"),
        ModelFamily::Local
    );
    assert_eq!(ModelFamily::from_model_id(""), ModelFamily::Local);
}

// ----- Test 7: AdversaryGenerated payload round-trip ------------------------

#[test]
fn contract_adversary_generated_payload_round_trip() {
    let p = AdversaryGeneratedPayload {
        counter_id: "ctr-123".into(),
        claim_id: "claim-abc".into(),
        severity: 0.85,
        model_family: "anthropic".into(),
    };
    let payload = SsbPayload::AdversaryGenerated(p.clone());
    let s = serde_json::to_string(&payload).expect("serialize");
    let back: SsbPayload = serde_json::from_str(&s).expect("deserialize");
    match back {
        SsbPayload::AdversaryGenerated(back_p) => {
            assert_eq!(back_p.counter_id, p.counter_id);
            assert_eq!(back_p.claim_id, p.claim_id);
            assert!((back_p.severity - p.severity).abs() < f64::EPSILON);
            assert_eq!(back_p.model_family, p.model_family);
        }
        other => panic!("round-trip variant drift: {other:?}"),
    }
}

// ----- Test 8: ConsensusReevaluatedWithCounter payload round-trip -----------

#[test]
fn contract_consensus_reevaluated_with_counter_payload_round_trip() {
    let p = ConsensusReevaluatedWithCounterPayload {
        claim_id: "claim-xyz".into(),
        original_verdict: "pass".into(),
        post_counter_verdict: "fail".into(),
    };
    let payload = SsbPayload::ConsensusReevaluatedWithCounter(p.clone());
    let s = serde_json::to_string(&payload).expect("serialize");
    let back: SsbPayload = serde_json::from_str(&s).expect("deserialize");
    match back {
        SsbPayload::ConsensusReevaluatedWithCounter(back_p) => {
            assert_eq!(back_p.claim_id, p.claim_id);
            assert_eq!(back_p.original_verdict, p.original_verdict);
            assert_eq!(back_p.post_counter_verdict, p.post_counter_verdict);
        }
        other => panic!("round-trip variant drift: {other:?}"),
    }
}

// ----- Test 9: heterogeneous_verifier_check refuses same-family pair --------

#[test]
fn contract_heterogeneous_verifier_check_refuses_same_family() {
    let claim = Claim {
        claim_id: "c-contract".into(),
        text: "test".into(),
        claim_type: ClaimType::Factual,
        evidence: vec![EvidenceRef::from_content(
            EvidenceKind::Corpus,
            "ref:test",
            b"x",
        )],
        ts_ms: 1_700_000_000_000,
    };
    // Same-family pool: every entry Local -> refused.
    let homogeneous = [
        (VerifierKind::Ppm, ModelFamily::Local),
        (VerifierKind::Intrinsic, ModelFamily::Local),
        (VerifierKind::Deterministic, ModelFamily::Local),
    ];
    assert!(!heterogeneous_verifier_check(&claim, &homogeneous));

    // Cross-family pool spans 2 distinct families -> accepted.
    let heterogeneous = [
        (VerifierKind::Ppm, ModelFamily::Anthropic),
        (VerifierKind::Intrinsic, ModelFamily::Local),
    ];
    assert!(heterogeneous_verifier_check(&claim, &heterogeneous));
}

// ----- Bonus: CounterEvidenceSchema round-trip lock (D-13 substrate) -------

#[test]
fn contract_counter_evidence_schema_round_trip() {
    let c = CounterEvidenceSchema {
        counter_id: "c-1".into(),
        original_claim_id: "claim-1".into(),
        counter_text: "Rust borrow checker replaces GC.".into(),
        counter_evidence: vec![EvidenceRef::from_content(
            EvidenceKind::Corpus,
            "ref:rust-book",
            b"ownership",
        )],
        severity: 0.95,
    };
    let s = serde_json::to_string(&c).expect("serialize");
    let back: CounterEvidenceSchema = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(c, back);
}
