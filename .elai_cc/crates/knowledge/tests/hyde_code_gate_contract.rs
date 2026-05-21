//! E-17 SP2 task 5 — HyDE NL-only gate contract test.
//!
//! All cases live in one #[test] to keep env-var manipulation serial
//! (the gate reads std::env::var which is process-global and Rust's
//! default cargo test parallelism would race on it).

use elai_knowledge::hyde::should_use_hyde;

#[test]
fn hyde_gate_truth_table() {
    // Default: unset = NL-only ON.
    // SAFETY: process-global env. We hold this single fn body for the
    // duration; cargo test parallelism is per-fn, not per-line.
    unsafe {
        std::env::remove_var("ELAI_HYDE_NL_ONLY");
    }

    // (a) Long NL queries — pass gate (HyDE = true).
    let nl_queries = [
        "how does the degradation cascade work in the knowledge layer",
        "explain why the auditor fails on missing tests",
        "what does the freshness multiplier do in retrieval",
        "where is the rrf quality gate applied in the pipeline",
        "when does the cross encoder reranker degrade to L 6",
        "why does the search mode default to hybrid in production",
        "how does the anti spiral detection choose a persona",
    ];
    for q in nl_queries {
        assert!(should_use_hyde(q), "expected HyDE on for NL query: {q}");
    }

    // (b) Long code-symbol queries — gated off.
    let code_queries = [
        "models/onnx/cross-encoder-l12.onnx file exists locally",
        "crates/knowledge/src/pipeline.rs rrf_rerank function",
        "RuntimeAdapter stream_step batch context limit",
        "BTreeMap::remove HashMap::entry insert clone",
    ];
    for q in code_queries {
        assert!(
            !should_use_hyde(q),
            "expected HyDE off for code query: {q}"
        );
    }

    // (c) Short queries — always skip.
    let short = ["find main", "how", "rrf", "rrf rerank", "the knowledge"];
    for q in short {
        assert!(!should_use_hyde(q), "expected short skip for: {q}");
    }

    // (d) Env-off disables the gate and lets long code-symbol queries
    //     through (tier-only routing).
    unsafe {
        std::env::set_var("ELAI_HYDE_NL_ONLY", "0");
    }
    let q = "RuntimeAdapter stream_step batch context limit";
    assert!(should_use_hyde(q), "expected HyDE on when gate disabled");
    unsafe {
        std::env::remove_var("ELAI_HYDE_NL_ONLY");
    }
}
