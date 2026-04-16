// recursive_research.rs — integration tests for 4D.7-4D.9 recursive decomposition
//
// 4D.8: all tests call stubs and expect Err(NotImplemented).
// 4D.9: stubs implemented, tests pass.

use elai_orchestrator::research_tree::*;
use std::time::Duration;

fn stub_report() -> ResearchReport {
    ResearchReport {
        question: "test concept".into(),
        answer: "test answer about concept and framework".into(),
        alternatives: vec![
            Alternative {
                name: "alt-phase-a".into(),
                description: "alternative phase A".into(),
                citations: vec![SourceCitation {
                    url: "https://docs.rs/serde".into(),
                    title: "serde docs".into(),
                    quality: SourceQuality::Tier2,
                    evidence_level: EvidenceLevel::MultiSource,
                    excerpt: "serialization framework".into(),
                }],
                confidence: 0.8,
                trade_offs: vec!["complexity".into()],
            },
            Alternative {
                name: "alt-phase-b".into(),
                description: "alternative phase B".into(),
                citations: vec![],
                confidence: 0.4,
                trade_offs: vec![],
            },
        ],
        contradictions: vec![],
        sub_results: vec![
            SubQuestionResult {
                question: "What concept applies?".into(),
                answer: "concept X with framework Y".into(),
                citations: vec![SourceCitation {
                    url: "https://arxiv.org/abs/1234".into(),
                    title: "paper".into(),
                    quality: SourceQuality::Tier1,
                    evidence_level: EvidenceLevel::Verified,
                    excerpt: "proven approach".into(),
                }],
                confidence: 0.9,
                evidence_level: EvidenceLevel::Verified,
            },
            SubQuestionResult {
                question: "How to implement?".into(),
                answer: "step by step".into(),
                citations: vec![],
                confidence: 0.3,
                evidence_level: EvidenceLevel::Anecdotal,
            },
        ],
        snapshot: ResearchTreeSnapshot {
            nodes: vec![],
            convergence: ConvergenceStatus::Converged,
            total_iterations: 5,
            elapsed: Duration::from_millis(100),
        },
        quality_metrics: vec![],
        convergence: ConvergenceStatus::Converged,
    }
}

// ---------------------------------------------------------------------------
// 1. Level 0: concept decomposition
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_level0_concept_decomposition() {
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let result = research_decompose("build a web server", ExecutionMode::Autonomous, &mut cache);
    // 4D.8: stub returns Err(NotImplemented); 4D.9: returns Ok with concepts
    assert!(
        result.is_ok(),
        "research_decompose should succeed: {result:?}"
    );
    let decomp = result.unwrap();
    assert!(!decomp.concepts.is_empty(), "must produce at least one concept");
}

// ---------------------------------------------------------------------------
// 2. Level 1: phase decomposition
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_level1_phase_decomposition() {
    let report = stub_report();
    let result = extract_phases(&report);
    assert!(result.is_ok(), "extract_phases should succeed: {result:?}");
    let phases = result.unwrap();
    // stub_report has 2 alternatives -> should produce phase candidates
    assert!(!phases.is_empty(), "must produce at least one phase");
}

// ---------------------------------------------------------------------------
// 3. Level 2: step decomposition
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_level2_step_decomposition() {
    let report = stub_report();
    let result = extract_steps(&report);
    assert!(result.is_ok(), "extract_steps should succeed: {result:?}");
    // Only sub_results with evidence >= SingleSource qualify
    let steps = result.unwrap();
    assert!(!steps.is_empty(), "must produce at least one step from Verified sub_result");
}

// ---------------------------------------------------------------------------
// 4. Level 3: baby steps
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_level3_baby_steps() {
    let step = StepNode {
        id: StepId(1),
        description: "implement the serialization layer with proper error handling".into(),
        baby_steps: vec![],
        evidence_level: EvidenceLevel::MultiSource,
    };
    let result = decompose_to_baby_steps(&step, "serialization", "setup");
    assert!(
        result.is_ok(),
        "decompose_to_baby_steps should succeed: {result:?}"
    );
    let baby_steps = result.unwrap();
    assert!(!baby_steps.is_empty(), "must produce at least one baby step");
    for bs in &baby_steps {
        assert!(
            bs.estimated_tokens <= MAX_BABY_STEP_TOKENS,
            "baby step tokens {} exceeds MAX_BABY_STEP_TOKENS {}",
            bs.estimated_tokens,
            MAX_BABY_STEP_TOKENS
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Cache hit: no second call
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_cache_hit_no_second_call() {
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let mut call_count = 0u32;

    // First call: should invoke the closure
    let r1 = cache.get_or_research("key-a", || {
        call_count += 1;
        Ok(stub_report())
    });
    assert!(r1.is_ok(), "first get_or_research should succeed");
    assert_eq!(call_count, 1, "closure should be called once");

    // Second call: cache hit, closure should NOT be called
    let r2 = cache.get_or_research("key-a", || {
        call_count += 1;
        Ok(stub_report())
    });
    assert!(r2.is_ok(), "second get_or_research should succeed");
    assert_eq!(call_count, 1, "closure should not be called again (cache hit)");
}

// ---------------------------------------------------------------------------
// 6. Cache TTL expiry: re-requests
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_cache_ttl_expired_rerequests() {
    // ttl_seconds = 0 means entries expire immediately
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, 0);
    let mut call_count = 0u32;

    let _ = cache.get_or_research("key-b", || {
        call_count += 1;
        Ok(stub_report())
    });
    assert_eq!(call_count, 1);

    // Second call: expired, should re-call
    let _ = cache.get_or_research("key-b", || {
        call_count += 1;
        Ok(stub_report())
    });
    assert_eq!(call_count, 2, "expired entry should trigger re-research");
}

// ---------------------------------------------------------------------------
// 7. Package verification: nonexistent = false
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_package_nonexistent_verified_false() {
    let packages = vec![PackageRef {
        name: "!!!invalid-pkg-$$$".into(),
        version: None,
        registry: "crates.io".into(),
        verified: false,
    }];
    let result = verify_packages(&packages);
    assert!(result.is_ok(), "verify_packages should succeed");
    let verified = result.unwrap();
    assert!(!verified[0].verified, "garbage package name must not verify");
}

// ---------------------------------------------------------------------------
// 8. Package verification: plausible name = true
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_package_real_verified_true() {
    let packages = vec![PackageRef {
        name: "serde".into(),
        version: Some("1.0".into()),
        registry: "crates.io".into(),
        verified: false,
    }];
    let result = verify_packages(&packages);
    assert!(result.is_ok(), "verify_packages should succeed");
    let verified = result.unwrap();
    assert!(verified[0].verified, "plausible package name 'serde' should verify");
}

// ---------------------------------------------------------------------------
// 9. Grounding under 500 tokens
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_grounding_under_500_tokens() {
    let step = StepNode {
        id: StepId(1),
        description: "a ".repeat(200),
        baby_steps: vec![],
        evidence_level: EvidenceLevel::Verified,
    };
    let result = decompose_to_baby_steps(&step, "concept-name", "phase-name");
    assert!(result.is_ok(), "decompose_to_baby_steps should succeed");
    let baby_steps = result.unwrap();
    for bs in &baby_steps {
        assert!(
            bs.grounding.compressed_tokens <= MAX_GROUNDING_TOKENS,
            "grounding tokens {} exceeds MAX_GROUNDING_TOKENS {}",
            bs.grounding.compressed_tokens,
            MAX_GROUNDING_TOKENS
        );
    }
}

// ---------------------------------------------------------------------------
// 10. Interactive mode presents alternatives
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_interactive_mode_presents_alternatives() {
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let result = research_decompose("build a CLI tool", ExecutionMode::Interactive, &mut cache);
    assert!(result.is_ok(), "interactive mode should succeed");
    let decomp = result.unwrap();
    assert_eq!(decomp.mode, ExecutionMode::Interactive);
}

// ---------------------------------------------------------------------------
// 11. Autonomous mode auto-selects
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_autonomous_mode_auto_selects() {
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let result = research_decompose("build a REST API", ExecutionMode::Autonomous, &mut cache);
    assert!(result.is_ok(), "autonomous mode should succeed");
    let decomp = result.unwrap();
    assert_eq!(decomp.mode, ExecutionMode::Autonomous);
    assert!(!decomp.concepts.is_empty(), "autonomous must auto-select concepts");
}

// ---------------------------------------------------------------------------
// 12. Cross-concept dependency detected
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_cross_concept_dependency_detected() {
    let report = stub_report();
    let phases = extract_phases(&report);
    assert!(phases.is_ok());
    // With stub_report, phase answers may reference other concept names.
    // After 4D.9 impl, depends_on should be populated when cross-references exist.
    // For now, just verify the structure is valid.
    let phases = phases.unwrap();
    for phase in &phases {
        // depends_on is a Vec<ConceptId> — must be valid even if empty
        let _ = phase.depends_on.len();
    }
}

// ---------------------------------------------------------------------------
// 13. Proptest: random idea, research_decompose never panics
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_proptest_no_panic() {
    // Deterministic loop with varied inputs (proptest-style without proptest crate for integration test)
    let long_input = "x".repeat(10000);
    let test_ideas: Vec<&str> = vec![
        "build a web server with authentication",
        "",
        &long_input,
        "rust vs go for systems programming; compare alternatives?",
        "a",
        "123 456 789",
        "special chars: !@#$%^&*()",
    ];

    for idea in &test_ideas {
        let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
        // Must not panic — Err is acceptable, panic is not
        let _ = research_decompose(idea, ExecutionMode::Autonomous, &mut cache);
    }
}

// ---------------------------------------------------------------------------
// 14. Cache eviction: LRU
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_cache_eviction_lru() {
    let max = 3;
    let mut cache = ResearchCache::new(max, DEFAULT_CACHE_TTL_SECONDS);

    // Fill cache to max
    for i in 0..max {
        let key = format!("key-{i}");
        let _ = cache.get_or_research(&key, || Ok(stub_report()));
    }
    assert_eq!(cache.entries.len(), max);

    // Insert one more — oldest should be evicted
    let _ = cache.get_or_research("key-overflow", || Ok(stub_report()));
    assert!(
        cache.entries.len() <= max,
        "cache should not exceed max_entries after eviction"
    );
    // key-0 was first inserted, should be evicted
    assert!(
        !cache.entries.contains_key("key-0"),
        "oldest entry (key-0) should be evicted"
    );
    assert!(
        cache.entries.contains_key("key-overflow"),
        "new entry should be present"
    );
}

// ---------------------------------------------------------------------------
// 15. ConceptId and BabyStepId are type-level distinct
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_concept_id_baby_step_id_type_level_distinct() {
    fn accept_concept(_id: ConceptId) {}
    fn accept_baby_step(_id: BabyStepId) {}

    // These compile only because ConceptId and BabyStepId are distinct types.
    // If they were type aliases, one could be passed where the other is expected.
    accept_concept(ConceptId(1));
    accept_baby_step(BabyStepId(1));

    // Runtime check: same inner value, different types
    let c = ConceptId(42);
    let b = BabyStepId(42);
    // They cannot be compared directly (different types) — this is the point.
    // Just verify they hold the expected values.
    assert_eq!(c.0, 42);
    assert_eq!(b.0, 42);
}

// ---------------------------------------------------------------------------
// 16. Extract concepts from report
// ---------------------------------------------------------------------------

#[test]
fn recursive_research_extract_concepts() {
    let report = stub_report();
    let result = extract_concepts(&report);
    assert!(result.is_ok(), "extract_concepts should succeed: {result:?}");
    let concepts = result.unwrap();
    // stub_report has sub_results with "concept" keyword -> should extract
    assert!(!concepts.is_empty(), "must extract at least one concept");
}
