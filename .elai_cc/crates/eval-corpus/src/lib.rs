//! E-54: eval-corpus -- repair D-6 9-gram filter pre-deploy.
//!
//! Adds 13-gram + embedding-cosine + per-family directory shards +
//! test-triplet schema + matched-pair sycophancy probes + KOR-orthogonality
//! check on D-7. The CanaryFamily 5-arm at crates/orchestrator/src/canary.rs
//! is the canonical task-family surface; KnowledgeQa is routed via
//! ToolUse-flavored tasks (web-grounded MCQs from SearXNG -- task 9
//! dynamic_knowledge_qa) per workflow/reasoning/E-54-decisions.md DECISION-1.
//!
//! Modules:
//!   - [`task_schema`] -- canonical Task envelope + ExecutableTest +
//!     Provenance + FramingPair + PerturbedPair.
//!   - [`storage`] -- per-family directory shard layout +
//!     `_index/{family}.jsonl` summary append.
//!   - [`ingest`] -- 13-gram + cosine ingest pipeline (reuses
//!     orchestrator::held_out primitives: normalize_text / ngram_filter /
//!     jaccard_overlap).
//!   - [`test_executor`] -- run tests vs reference solution + broken solution
//!     per arXiv:2411.04905 OpenCoder recipe.
//!   - [`kor_orthogonality`] -- cross-family transfer < random + 10pp gate
//!     per arXiv:2410.06526 KOR-Bench (ELAI-specific operationalization).
//!   - [`sycophancy_matched_pair`] -- eval-vs-deploy framing + sandbagging
//!     anomaly per arXiv:2510.20487 + arXiv:2508.00943.
//!   - [`generator_rotation`] -- min-30% per generator (3+ generators) per
//!     arXiv:2412.08905 Phi-4 lesson.
//!   - [`perf_delta_detector`] -- ConStat-style + 2-sigma fallback.
//!   - [`dynamic_knowledge_qa`] -- OKBench-style daily MCQ generation via
//!     sidecar SearXNG endpoint (arXiv:2511.08598).
//!   - [`prod_traffic_grade`] -- opt-in silent probes per arXiv:2403.04132.
//!
//! Wire-dark default: SSB emission gated by `ELAI_EVAL_CORPUS_ENABLE=1` env
//! var (consumed by `ingest::should_emit`).
//!
//! Anti-stale: blake3 only (workspace baseline; see CLAUDE.md HARD RULE).
//! No `.unwrap()` outside `#[cfg(test)]` in library code.

pub mod task_schema;
pub mod storage;
pub mod ingest;
pub mod test_executor;
pub mod kor_orthogonality;
pub mod sycophancy_matched_pair;
pub mod generator_rotation;
pub mod perf_delta_detector;
pub mod dynamic_knowledge_qa;
pub mod prod_traffic_grade;

// E-54 DECISION-2: re-export the held_out primitives (normalize_text /
// content_hash / ngram_filter / jaccard_overlap) so eval-corpus consumers
// import a single canonical surface. orchestrator::held_out remains the
// owner; eval-corpus is the public surface for new corpus pipelines.
pub use elai_orchestrator::held_out::{
    content_hash, jaccard_overlap, ngram_filter, normalize_text,
};

// E-54 DECISION-1: TaskFamily is an alias for the FROZEN CanaryFamily 5-arm.
// KnowledgeQa is NOT a separate family; web-grounded MCQs are ToolUse-flavored
// tasks routed through dynamic_knowledge_qa. See E-54-decisions.md.
pub use elai_orchestrator::canary::CanaryFamily as TaskFamily;
pub use elai_orchestrator::canary::CANARY_FAMILY_VARIANT_COUNT as TASK_FAMILY_VARIANT_COUNT;

/// E-54 default 13-gram window size. Matches GPT-3's threshold per
/// arXiv:2311.04850 (Yang & Chiang -- 13B model evades baselines with
/// paraphrases the 13-gram never sees). Override via the `[corpus.ngram]`
/// section of elai_cc.toml (`n`).
pub const NGRAM_DEFAULT_N: usize = 13;

/// E-54 default consecutive-gram match count. ≥K=4 consecutive 13-grams =
/// "near-duplicate" per arXiv:2311.04850 §4.2.
pub const NGRAM_DEFAULT_K_CONSECUTIVE: usize = 4;

/// E-54 default embedding-cosine threshold. INITIAL DEFAULT pending
/// ELAI-corpus precision/recall calibration (Open Q1). arXiv:2511.17602
/// reports F1=0.76 at the semantic level; the 0.85 number is an ELAI
/// engineering default, NOT a paper-derived threshold. See
/// workflow/reasoning/E-54-decisions.md DECISION-4 (FIX-2).
pub const COSINE_DEFAULT_THRESHOLD: f64 = 0.85;

/// E-54 SSB emission env-var gate. Wire-dark until `ELAI_EVAL_CORPUS_ENABLE=1`.
pub const SSB_GATE_ENV_VAR: &str = "ELAI_EVAL_CORPUS_ENABLE";

/// E-54: returns `true` when SSB emission should be active (env var set to
/// `1` / `true` / `yes` / `on`). Treats missing env var as OFF.
#[must_use]
pub fn ssb_emission_enabled() -> bool {
    match std::env::var(SSB_GATE_ENV_VAR) {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}
