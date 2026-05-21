//! E-17 SP2 task 5 — heuristic query-class classifier contract test.
//!
//! 30-query fixture set with labels matching the heuristic itself.
//! Asserts overall accuracy >= 90% per class (heuristic-based test of the
//! heuristic implementation; real F1 measurement vs a gold set is deferred
//! to SP6 task 10 per E-17 plan acceptance criteria).

use elai_knowledge::query_class::{classify, QueryClass};

#[test]
fn classifier_accuracy_meets_90pct_per_class() {
    // 10 CodeSymbol fixtures.
    let code_symbol: &[&str] = &[
        "BTreeMap::remove",
        "rrf_rerank",
        "RuntimeAdapter",
        "fn build_session",
        "models/onnx/cross-encoder-l12.onnx",
        "crates/knowledge/src/pipeline.rs",
        "QueryPipeline::query_with_tier",
        "impl KnowledgeSsbSink",
        "struct DkgEngine",
        "knowledge_ssb_sink::emit_hyde_query",
    ];
    // 10 NaturalLanguage fixtures (each contains >= 4 stopwords).
    let natural_language: &[&str] = &[
        "how does the degradation cascade work in the knowledge layer",
        "explain why the auditor fails on missing tests",
        "what does the freshness multiplier do in retrieval",
        "where is the rrf quality gate applied in the pipeline",
        "when does the cross encoder reranker degrade to L 6",
        "why is the search mode hybrid in production",
        "how does the anti spiral detection choose a persona",
        "explain how the rag pipeline merges bm25 and vector",
        "what is the role of the freshness db in scoring",
        "describe how the hyde stage caches the hypotheses",
    ];
    // 10 Mixed fixtures (short phrases, neither bucket dominates).
    let mixed: &[&str] = &[
        "knowledge layer",
        "anti spiral detector",
        "rrf k constant",
        "deep cap pool",
        "tiered analysis",
        "code review",
        "cli launcher",
        "memory bank",
        "task runner",
        "session handoff",
    ];

    let mut code_hits = 0;
    for q in code_symbol {
        if classify(q) == QueryClass::CodeSymbol {
            code_hits += 1;
        }
    }
    let mut nl_hits = 0;
    for q in natural_language {
        if classify(q) == QueryClass::NaturalLanguage {
            nl_hits += 1;
        }
    }
    let mut mixed_hits = 0;
    for q in mixed {
        if classify(q) == QueryClass::Mixed {
            mixed_hits += 1;
        }
    }

    assert!(
        code_hits >= 9,
        "CodeSymbol accuracy too low: {code_hits}/10"
    );
    assert!(nl_hits >= 9, "NL accuracy too low: {nl_hits}/10");
    assert!(mixed_hits >= 9, "Mixed accuracy too low: {mixed_hits}/10");
}

#[test]
fn empty_query_is_mixed() {
    assert_eq!(classify(""), QueryClass::Mixed);
    assert_eq!(classify("   "), QueryClass::Mixed);
}

#[test]
fn single_token_is_code_symbol() {
    assert_eq!(classify("foo"), QueryClass::CodeSymbol);
    assert_eq!(classify("RuntimeAdapter"), QueryClass::CodeSymbol);
    assert_eq!(classify("snake_case_var"), QueryClass::CodeSymbol);
}
