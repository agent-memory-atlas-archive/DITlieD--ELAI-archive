//! D-15 contract test — locks the public surface of the novelty-class
//! labeling substrate so future renames trip the contract.
//!
//! What this contract asserts:
//!   1. SCHEMA_VERSION floor >= 83 (the shipped novelty-substrate
//!      baseline). Concurrent piggybacks may bump higher; the
//!      D-15 closure does NOT bump (substrate already shipped).
//!   2. NoveltyClass enum is exhaustive (3 variants per arXiv:2406.05761
//!      §4) + serde round-trip stable (snake_case wire labels).
//!   3. NoveltyClassified SSB payload round-trips through both
//!      JSON and MessagePack.
//!   4. `novelty_evidence_from_checker_result` is deterministic —
//!      same input yields identical evidence + class on repeat calls.
//!   5. 3-bucket threshold invariants hold: the default
//!      NoveltyConfig (`search_dkg_min=5`, `near_duplicate_high=0.8`,
//!      `near_duplicate_low=0.4`) routes each canonical evidence
//!      bundle to the expected class.
//!
//! Grounding:
//!   workflow/papers/funsearch-nature-2023-interface.md,
//!   workflow/papers/alphaevolve-2506.13131-interface.md,
//!   workflow/papers/chollet-arc-1911.01547-interface.md,
//!   workflow/reasoning/D-15-decisions.md.

use elai_orchestrator::archive_chain::ArchiveChain;
use elai_orchestrator::checker_registry::CheckerResult;
use elai_orchestrator::novelty::{
    classify, novelty_evidence_from_checker_result, NoveltyClass,
    NoveltyConfig, NoveltyEvidence,
};
use elai_orchestrator::reasoning_template::{
    compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate,
    ReasoningTemplate, TemplateBody, TemplateId, VerifierKind,
};
use elai_orchestrator::ssb::{
    NoveltyClassifiedPayload, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

fn make_template(body: &str) -> ReasoningTemplate {
    let bounded = TemplateBody::try_new(body).expect("body fits");
    let hash = compute_content_hash(bounded.as_str(), None);
    ReasoningTemplate {
        id: TemplateId::new_v4(),
        domain: Domain::Code,
        pattern_kind: PatternKind::Decomposition,
        template_body: bounded,
        eligibility_predicate: Predicate::Always,
        verifier_required: VerifierKind::Refuse,
        content_hash: hash,
        created_at_ms: 1_700_000_000_000,
        parent_id: None,
        archive_status: ArchiveStatus::Promoted,
    }
}

fn empty_archive() -> ArchiveChain {
    ArchiveChain::new(std::path::PathBuf::from("test-novelty-contract.jsonl"))
}

// ----- Test 1: SCHEMA_VERSION floor -----------------------------------------

#[test]
fn contract_schema_version_floor_d15_baseline() {
    // D-15 substrate shipped at SCHEMA 83. Floor never regresses below
    // 83. The D-15 closure does NOT bump — the single
    // `NoveltyClassified` SSB variant was already in place. Concurrent
    // D-cluster work piggybacks the schema (head ~125 at closure time).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 83,
        "SCHEMA_VERSION_PUBLIC regressed below D-15 floor 83; got {SCHEMA_VERSION_PUBLIC}"
    );
}

// ----- Test 2: NoveltyClass 3-arm exhaustive serde --------------------------

#[test]
fn contract_novelty_class_3_arm_exhaustive_serde() {
    let all = NoveltyClass::all();
    assert_eq!(
        all.len(),
        3,
        "NoveltyClass MUST have exactly 3 variants \
         (arXiv:2406.05761 §4 — familiar / recombination / OOD); \
         adding a 4th requires a contract review"
    );
    let expected_labels = ["search", "distant_search", "discovery_candidate"];
    for (k, expected) in all.iter().zip(expected_labels.iter()) {
        assert_eq!(k.as_str(), *expected, "as_str mismatch for {k:?}");
        let s = serde_json::to_string(k).expect("serialize");
        let back: NoveltyClass =
            serde_json::from_str(&s).expect("deserialize");
        assert_eq!(k, &back, "round-trip mismatch for {k:?} -> {s}");
    }
}

// ----- Test 3: NoveltyClassified payload round-trip -------------------------

#[test]
fn contract_novelty_classified_payload_round_trip() {
    let payload = NoveltyClassifiedPayload {
        template_id: "template-d15-contract".into(),
        class: "discovery_candidate".into(),
        dkg_match_count: 0,
        near_duplicate_threshold: 0.12,
        ts_ms: 1_700_000_000_000,
    };
    let wrapped = SsbPayload::NoveltyClassified(payload.clone());

    // MessagePack round-trip (SSB on-disk format).
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert_eq!(back, wrapped);
    match back {
        SsbPayload::NoveltyClassified(p) => {
            assert_eq!(p.template_id, payload.template_id);
            assert_eq!(p.class, payload.class);
            assert_eq!(p.dkg_match_count, payload.dkg_match_count);
            assert!(
                (p.near_duplicate_threshold - payload.near_duplicate_threshold)
                    .abs()
                    < f64::EPSILON
            );
            assert_eq!(p.ts_ms, payload.ts_ms);
        }
        other => panic!("unexpected variant {other:?}"),
    }

    // JSON round-trip (dashboard wire format).
    let json = serde_json::to_string(&wrapped).expect("json serialize");
    let back_json: SsbPayload =
        serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(back_json, wrapped);
}

// ----- Test 4: novelty_evidence_from_checker_result determinism -------------

#[test]
fn contract_novelty_evidence_projection_is_deterministic() {
    // Pure function — same input must yield identical evidence on
    // repeat calls (no clock, no PRNG, no I/O). The class assigned by
    // the predicate must also match across calls.
    let template = make_template("body-projection-contract");
    let archive = empty_archive();
    let cfg = NoveltyConfig::default();

    let cases = [
        CheckerResult::Pass,
        CheckerResult::Fail {
            reason: "checker rejected".into(),
        },
        CheckerResult::Inapplicable {
            reason: "wrong claim type".into(),
        },
        CheckerResult::Error {
            source: "subprocess timed out".into(),
        },
    ];

    for r in &cases {
        let a = novelty_evidence_from_checker_result(r);
        let b = novelty_evidence_from_checker_result(r);
        assert_eq!(a, b, "projection not deterministic for {r:?}");
        let class_a = classify(&template, &archive, a, cfg);
        let class_b = classify(&template, &archive, b, cfg);
        assert_eq!(class_a, class_b, "class drift for {r:?}");
    }

    // Each variant must route to its expected class under the default
    // NoveltyConfig. This locks the projection->predicate contract.
    assert_eq!(
        classify(
            &template,
            &archive,
            novelty_evidence_from_checker_result(&CheckerResult::Pass),
            cfg,
        ),
        NoveltyClass::Search
    );
    assert_eq!(
        classify(
            &template,
            &archive,
            novelty_evidence_from_checker_result(&CheckerResult::Fail {
                reason: "x".into(),
            }),
            cfg,
        ),
        NoveltyClass::DistantSearch
    );
    assert_eq!(
        classify(
            &template,
            &archive,
            novelty_evidence_from_checker_result(
                &CheckerResult::Inapplicable {
                    reason: "x".into(),
                },
            ),
            cfg,
        ),
        NoveltyClass::DiscoveryCandidate
    );
    assert_eq!(
        classify(
            &template,
            &archive,
            novelty_evidence_from_checker_result(&CheckerResult::Error {
                source: "x".into(),
            }),
            cfg,
        ),
        NoveltyClass::DiscoveryCandidate
    );
}

// ----- Test 5: 3-bucket threshold invariants under default config -----------

#[test]
fn contract_three_bucket_threshold_invariants_default_config() {
    let template = make_template("body-threshold-contract");
    let archive = empty_archive();
    let cfg = NoveltyConfig::default();
    // Document the locked default thresholds at the contract layer.
    assert_eq!(cfg.search_dkg_min, 5);
    assert!((cfg.near_duplicate_high - 0.8).abs() < f64::EPSILON);
    assert!((cfg.near_duplicate_low - 0.4).abs() < f64::EPSILON);

    // Search arm — meets BOTH dkg_min AND near_duplicate_high.
    let e_search = NoveltyEvidence::new(10, 0.95, 0.0, 0.9);
    assert_eq!(
        classify(&template, &archive, e_search, cfg),
        NoveltyClass::Search
    );

    // DistantSearch arm — partial DKG matches OR mid-range duplicate.
    let e_distant_mid = NoveltyEvidence::new(3, 0.55, 0.5, 0.5);
    assert_eq!(
        classify(&template, &archive, e_distant_mid, cfg),
        NoveltyClass::DistantSearch
    );

    // DistantSearch boundary — strict `<` boundary at near_duplicate_low.
    let e_distant_boundary = NoveltyEvidence::new(0, 0.4, 0.0, 0.0);
    assert_eq!(
        classify(&template, &archive, e_distant_boundary, cfg),
        NoveltyClass::DistantSearch
    );

    // DiscoveryCandidate arm — zero DKG matches AND sub-low duplicate.
    let e_discovery = NoveltyEvidence::new(0, 0.1, 1.4, 0.05);
    assert_eq!(
        classify(&template, &archive, e_discovery, cfg),
        NoveltyClass::DiscoveryCandidate
    );
}
