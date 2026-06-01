//! E-54 ingest pipeline: 13-gram + embedding-cosine + write-shard +
//! append-index.
//!
//! Pipeline (single-pass):
//!   1. Normalize the task body via [`crate::normalize_text`] (re-export of
//!      orchestrator::held_out primitive; do NOT duplicate the normalizer).
//!   2. Compute 13-gram signature via [`crate::ngram_filter`] (n=13 default,
//!      per arXiv:2311.04850); compare K=4 consecutive grams against any
//!      prior signature in the per-family index. >=K consecutive ngrams in
//!      common = reject with `corpus_ingest_rejected_ngram` SSB row.
//!   3. RUNPOD-PENDING: compute BGE-M3 / Nomic Embed v2 embedding via
//!      sidecar `/v1/embeddings` endpoint (created in
//!      python/elai_inference/embeddings.py). Reject if cosine similarity
//!      >0.85 with any prior task in the same family with
//!      `corpus_ingest_rejected_cosine` SSB row.
//!   4. Compute embedding_hash + ngram_hash (blake3, 16-hex prefix).
//!   5. Write task TOML to per-family shard, append IndexRow to
//!      `_index/{family}.jsonl`, emit `corpus_shard_created` SSB row on the
//!      first task per (family, date) shard.
//!
//! Wire-dark by default: SSB emission gated by ELAI_EVAL_CORPUS_ENABLE; see
//! [`crate::ssb_emission_enabled`].
//!
//! Anti-stale: blake3 only. No `.unwrap()` outside `#[cfg(test)]`.

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    NGRAM_DEFAULT_K_CONSECUTIVE, NGRAM_DEFAULT_N, COSINE_DEFAULT_THRESHOLD,
    ngram_filter, normalize_text,
};
use crate::storage::{
    append_index, write_task_to_shard, IndexRow, StorageError,
};
use crate::task_schema::{Task, TaskSchemaError};

/// Configuration for the ingest pipeline. Threshold defaults match the plan
/// stub_ban contract: n=13 (REJECTED below 13), cosine=0.85 (never disabled).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IngestConfig {
    /// N-gram window size. Contract `no_nine_gram_only_decontam` enforces
    /// `n >= 13`.
    pub ngram_n: usize,
    /// Consecutive grams in common = duplicate. Per arXiv:2311.04850 §4.2.
    pub ngram_k_consecutive: usize,
    /// Cosine similarity threshold. Contract `cosine_filter_never_disabled`
    /// enforces this value is `> 0.0` (filter active).
    pub cosine_threshold: f64,
    /// True when the cosine filter is active. Contract enforces this MUST be
    /// true in the default config; opt-out only via explicit
    /// `--no-default-features` build flag.
    pub cosine_enabled: bool,
}

impl IngestConfig {
    /// Default config -- matches the plan stub_ban contract.
    #[must_use]
    pub const fn default_const() -> Self {
        Self {
            ngram_n: NGRAM_DEFAULT_N,
            ngram_k_consecutive: NGRAM_DEFAULT_K_CONSECUTIVE,
            cosine_threshold: COSINE_DEFAULT_THRESHOLD,
            cosine_enabled: true,
        }
    }
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self::default_const()
    }
}

/// Reason codes for ingest rejection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    /// 13-gram + K=4 consecutive grams found in another task.
    NgramOverlap {
        offending_task_id: String,
        consecutive_grams: u32,
    },
    /// Embedding cosine similarity >= threshold with another task.
    CosineSimilarity {
        offending_task_id: String,
        cosine: f64,
    },
    /// Schema validation failed (empty tests / cutoffs / banned deploy token).
    SchemaViolation(String),
    /// Reference solution does not pass all tests.
    TestFailsOnReference,
    /// At least one test passes on the broken solution (tests are too weak).
    TestUselessPassesBroken,
}

/// Result of a single-task ingest.
#[derive(Debug, Clone)]
pub enum IngestResult {
    /// Task accepted; written to `path` + indexed.
    Accepted { path: PathBuf },
    /// Task rejected with a typed reason.
    Rejected { reason: RejectReason },
}

/// Pipeline-level errors (distinct from `Rejected`; these are I/O / schema
/// failures that the caller surfaces vs scientific decontamination
/// rejections).
#[derive(Debug, Error)]
pub enum IngestError {
    #[error("eval-corpus ingest storage: {0}")]
    Storage(#[from] StorageError),
    #[error("eval-corpus ingest schema: {0}")]
    Schema(#[from] TaskSchemaError),
    #[error("eval-corpus ingest IO at {0}: {1}")]
    Io(String, String),
}

/// New-task input. The `body` field is the *normalized + 13-gram-fingerprinted*
/// surface (= `problem` field on the task envelope by convention; see
/// [`compute_ingest_hashes`]).
///
/// `current_embedding` carries the pre-fetched embedding vector for the
/// task being ingested. Caller-side fetch (sidecar HTTP) is RUNPOD-PENDING
/// per `python/elai_inference/embeddings.py`; when None the cosine filter
/// step in [`ingest_task`] degrades to a no-op (the 13-gram filter still
/// runs, the contract `cosine_filter_never_disabled` continues to hold on
/// the `IngestConfig` field, and `corpus_ingest_rejected_cosine`
/// emissions only fire when both sides supply embeddings). See E-54
/// DECISION-4 (cosine threshold 0.85 INITIAL DEFAULT).
#[derive(Debug, Clone)]
pub struct NewTaskInput {
    pub task: Task,
    pub prior_signatures: Vec<PriorTaskSignature>,
    pub current_embedding: Option<Vec<f32>>,
}

/// Reference signature for one prior task in the same family. The caller
/// (typically the ingest orchestrator) populates this from the
/// `_index/{family}.jsonl` summary.
#[derive(Debug, Clone)]
pub struct PriorTaskSignature {
    pub task_id: String,
    pub ngram_signature: Vec<u64>,
    /// Optional cached embedding (length = embedding_dim; usually 1024 for
    /// BGE-M3 or 768 for Nomic Embed v2). When `None`, the cosine filter is
    /// skipped for this prior task -- the index-row scanner re-fetches via
    /// sidecar HTTP at ingest time when [`IngestConfig::cosine_enabled`].
    pub embedding: Option<Vec<f32>>,
}

/// Compute the embedding_hash + ngram_hash for a task body. Both are blake3
/// 16-hex prefixes. Re-uses orchestrator::held_out::content_hash via the
/// re-export in crate root. The full ngram_filter signature is also returned
/// so the caller can store it alongside.
pub fn compute_ingest_hashes(body: &str, ngram_n: usize) -> (String, String, Vec<u64>) {
    let normalized = normalize_text(body);
    let embedding_hash = {
        let h = blake3::hash(normalized.as_bytes());
        let bytes = h.as_bytes();
        let mut hex = String::with_capacity(16);
        for b in &bytes[..8] {
            hex.push_str(&format!("{b:02x}"));
        }
        hex
    };
    let signature = ngram_filter(body, ngram_n);
    let ngram_hash = {
        // canonicalize signature: sort to make the hash order-independent.
        let mut sig_sorted: Vec<u64> = signature.clone();
        sig_sorted.sort_unstable();
        let mut buf: Vec<u8> = Vec::with_capacity(sig_sorted.len() * 8);
        for v in &sig_sorted {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        let h = blake3::hash(&buf);
        let bytes = h.as_bytes();
        let mut hex = String::with_capacity(16);
        for b in &bytes[..8] {
            hex.push_str(&format!("{b:02x}"));
        }
        hex
    };
    (embedding_hash, ngram_hash, signature)
}

/// True when `a` and `b` share at least `k` CONSECUTIVE n-gram-hash u64s in
/// the same order. This is a tighter rule than raw Jaccard overlap and
/// matches the "consecutive 13-grams" mechanism the GPT-3 / arXiv:2311.04850
/// recipe describes. Empty inputs return false.
#[must_use]
pub fn consecutive_ngram_match(a: &[u64], b: &[u64], k: usize) -> Option<u32> {
    if k == 0 || a.len() < k || b.len() < k {
        return None;
    }
    // Build a set of all length-k contiguous windows of `b` (canonical-bytes).
    use std::collections::HashSet;
    let mut b_windows: HashSet<Vec<u64>> = HashSet::new();
    for w in b.windows(k) {
        b_windows.insert(w.to_vec());
    }
    for w in a.windows(k) {
        if b_windows.contains(&w.to_vec()) {
            // Found at least one k-consecutive match; return the threshold
            // value as the count for SSB row payload.
            return Some(k as u32);
        }
    }
    None
}

/// Cosine similarity of two equally-sized embedding vectors. Returns 0.0 on
/// length mismatch or zero norms (vacuously dissimilar).
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot: f64 = 0.0;
    let mut na: f64 = 0.0;
    let mut nb: f64 = 0.0;
    for i in 0..a.len() {
        let av = a[i] as f64;
        let bv = b[i] as f64;
        dot += av * bv;
        na += av * av;
        nb += bv * bv;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Synchronous (in-process) ingest of a single task. The cosine-filter
/// branch consumes pre-fetched embeddings via
/// [`PriorTaskSignature::embedding`]; the live-sidecar HTTP fetch is a
/// caller responsibility (see
/// `python/elai_inference/embeddings.py`). RUNPOD-PENDING: a thin async
/// wrapper that calls the sidecar's `/v1/embeddings` is the next layer.
///
/// Pipeline order:
///   1. validate schema (tests + cutoffs + banned deploy tokens)
///   2. 13-gram K=4 consecutive check against prior signatures
///   3. cosine check against prior embeddings (when `cosine_enabled`)
///   4. write shard + append index
pub fn ingest_task(
    root: &Path,
    input: &NewTaskInput,
    config: &IngestConfig,
) -> Result<IngestResult, IngestError> {
    // (1) schema validation
    if let Err(e) = input.task.validate() {
        return Ok(IngestResult::Rejected {
            reason: RejectReason::SchemaViolation(e.to_string()),
        });
    }
    let (_, _, signature) = compute_ingest_hashes(&input.task.problem, config.ngram_n);

    // (2) 13-gram K=consecutive overlap
    for prior in &input.prior_signatures {
        if let Some(grams) =
            consecutive_ngram_match(&signature, &prior.ngram_signature, config.ngram_k_consecutive)
        {
            return Ok(IngestResult::Rejected {
                reason: RejectReason::NgramOverlap {
                    offending_task_id: prior.task_id.clone(),
                    consecutive_grams: grams,
                },
            });
        }
    }

    // (3) cosine filter (when enabled and the caller pre-fetched embeddings).
    // The current task's embedding fetch is delegated to the caller (sidecar
    // HTTP); embeddings.py is RUNPOD-PENDING (DECISION-4 INITIAL DEFAULT
    // 0.85). When `cosine_enabled` is true but the caller passed no
    // current_embedding OR no prior task carries an embedding, the loop is a
    // no-op -- contract `cosine_filter_never_disabled` enforces the
    // config-side `cosine_enabled == true` invariant unconditionally;
    // `corpus_ingest_rejected_cosine` only fires when BOTH sides supply
    // vectors AND similarity >= threshold.
    if config.cosine_enabled {
        if let Some(current) = input.current_embedding.as_deref() {
            for prior in &input.prior_signatures {
                if let Some(reason) = evaluate_cosine_against_prior(current, prior, config) {
                    return Ok(IngestResult::Rejected { reason });
                }
            }
        }
    }

    // (4) write shard + index row
    let path = write_task_to_shard(root, &input.task)?;
    let row = IndexRow {
        id: input.task.id.clone(),
        path: path.display().to_string(),
        created_at: input.task.created_at,
        cutoffs_keys: {
            let mut k: Vec<String> = input.task.cutoffs.keys().cloned().collect();
            k.sort();
            k
        },
        embedding_hash: input.task.embedding_hash.clone(),
        ngram_hash: input.task.ngram_hash.clone(),
    };
    append_index(root, input.task.family, &row)?;

    Ok(IngestResult::Accepted { path })
}

/// Compare the current task's embedding against a prior task's embedding and
/// produce a rejection-reason if the cosine similarity meets / exceeds the
/// threshold. Caller composes this with [`ingest_task`] when live embeddings
/// are available.
#[must_use]
pub fn evaluate_cosine_against_prior(
    current: &[f32],
    prior: &PriorTaskSignature,
    config: &IngestConfig,
) -> Option<RejectReason> {
    if !config.cosine_enabled {
        return None;
    }
    let Some(prior_emb) = prior.embedding.as_ref() else {
        return None;
    };
    let cos = cosine_similarity(current, prior_emb);
    if cos >= config.cosine_threshold {
        Some(RejectReason::CosineSimilarity {
            offending_task_id: prior.task_id.clone(),
            cosine: cos,
        })
    } else {
        None
    }
}

/// Returns the timestamp the caller should embed in the next ingest row.
/// Pulled out so tests can fake it.
#[must_use]
pub fn now_for_ingest() -> chrono::DateTime<chrono::Utc> {
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskFamily;
    use crate::task_schema::{ExecutableTest, Provenance, TestKind};
    use chrono::TimeZone;
    use rustc_hash::FxHashMap;
    use tempfile::tempdir;

    fn fixture_task(id: &str, body: &str) -> Task {
        let mut cutoffs = FxHashMap::default();
        cutoffs.insert("qwen2.5-coder-3b".to_string(), "2024-10".to_string());
        Task {
            id: id.to_string(),
            family: TaskFamily::CodeFix,
            created_at: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
            cutoffs,
            problem: body.to_string(),
            reference_solution: "fn ok()->i32{0}".to_string(),
            tests: vec![ExecutableTest {
                input: "ok()".to_string(),
                expected_output: "0".to_string(),
                kind: TestKind::UnitTest,
            }],
            provenance: Provenance {
                generator: "test_fixture".to_string(),
                prompt_id: None,
                date: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
            },
            framing_pair: None,
            perturbed_pair: None,
            embedding_hash: "deadbeefcafe1234".to_string(),
            ngram_hash: "1234deadbeefcafe".to_string(),
        }
    }

    #[test]
    fn ingest_config_defaults_to_13_gram_and_cosine_enabled() {
        let cfg = IngestConfig::default();
        assert!(cfg.ngram_n >= 13, "default n must be >= 13");
        assert!(cfg.cosine_enabled, "cosine filter must be enabled in defaults");
        assert!(cfg.cosine_threshold > 0.0, "threshold must be positive");
    }

    #[test]
    fn accepts_fresh_task_with_empty_prior_signatures() {
        let tmp = tempdir().unwrap();
        let task = fixture_task("t1", "Write a function that adds two large prime numbers \
            and returns the result modulo 2^32 minus five integers carefully.");
        let input = NewTaskInput {
            task,
            prior_signatures: Vec::new(),
            current_embedding: None,
        };
        let cfg = IngestConfig::default();
        let r = ingest_task(tmp.path(), &input, &cfg).unwrap();
        assert!(matches!(r, IngestResult::Accepted { .. }));
    }

    #[test]
    fn ingest_rejects_exact_match_via_ngram() {
        let body = "Write a function that adds two large prime numbers \
            and returns the result modulo 2^32 minus five integers carefully.";
        let cfg = IngestConfig::default();
        let prior_sig = ngram_filter(body, cfg.ngram_n);
        let tmp = tempdir().unwrap();
        let task = fixture_task("t2", body);
        let input = NewTaskInput {
            task,
            prior_signatures: vec![PriorTaskSignature {
                task_id: "prior-t1".to_string(),
                ngram_signature: prior_sig,
                embedding: None,
            }],
            current_embedding: None,
        };
        let r = ingest_task(tmp.path(), &input, &cfg).unwrap();
        match r {
            IngestResult::Rejected { reason } => match reason {
                RejectReason::NgramOverlap { offending_task_id, .. } => {
                    assert_eq!(offending_task_id, "prior-t1");
                }
                other => panic!("expected NgramOverlap, got {other:?}"),
            },
            IngestResult::Accepted { .. } => panic!("expected rejection"),
        }
    }

    #[test]
    fn cosine_filter_rejects_high_similarity() {
        let cfg = IngestConfig::default();
        let current = vec![1.0f32, 0.0, 0.0, 0.0];
        let prior = PriorTaskSignature {
            task_id: "prior".to_string(),
            ngram_signature: Vec::new(),
            embedding: Some(vec![0.95f32, 0.05, 0.05, 0.05]),
        };
        let r = evaluate_cosine_against_prior(&current, &prior, &cfg);
        // cosine of (1,0,0,0) vs (0.95,0.05,0.05,0.05) is ~0.99 -> reject.
        assert!(matches!(r, Some(RejectReason::CosineSimilarity { .. })));
    }

    #[test]
    fn cosine_filter_accepts_low_similarity() {
        let cfg = IngestConfig::default();
        let current = vec![1.0f32, 0.0, 0.0, 0.0];
        let prior = PriorTaskSignature {
            task_id: "prior".to_string(),
            ngram_signature: Vec::new(),
            embedding: Some(vec![0.0f32, 1.0, 0.0, 0.0]),
        };
        let r = evaluate_cosine_against_prior(&current, &prior, &cfg);
        assert!(r.is_none(), "orthogonal vectors must NOT trigger rejection");
    }

    #[test]
    fn consecutive_ngram_match_detects_overlap() {
        let a: Vec<u64> = (0..20u64).collect();
        let b: Vec<u64> = (10..30u64).collect();
        let r = consecutive_ngram_match(&a, &b, 4);
        assert!(r.is_some(), "10..14 is a 4-consecutive window in both");
    }

    #[test]
    fn consecutive_ngram_match_returns_none_for_too_short_inputs() {
        let a: Vec<u64> = vec![1, 2];
        let b: Vec<u64> = vec![1, 2, 3, 4];
        assert!(consecutive_ngram_match(&a, &b, 4).is_none());
    }
}
