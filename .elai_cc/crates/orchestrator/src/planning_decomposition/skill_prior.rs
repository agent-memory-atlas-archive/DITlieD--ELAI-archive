//! Voyager-style per-node skill-prior surfacing (arXiv:2305.16291).
//!
//! Plan task 8 of E-46. Successful BabyStep traces become
//! [`SkillPrior`] rows stored in the in-memory [`SkillPriorStore`].
//! On future Phase decomposition the scope-planner-elai agent retrieves
//! top-K=3 skill priors by cosine similarity over the
//! [`SkillPrior::goal_blake3_buckets`] vector against the new Phase's
//! acceptance-criteria token-bucket vector, and conditions the
//! decomposer prompt with the top-K accepted-diff summaries.
//!
//! Per the cross-check exploration GAP §78 the sidecar does NOT expose an
//! `/embed` HTTP endpoint, so this module substitutes the canonical
//! ELAI-internal token-frequency cosine over blake3-derived buckets in
//! place of a dense LLM embedding. The substitute is documented as a
//! pragmatic stand-in: when (and only when) a sidecar `/embed` route ships
//! in a follow-up patch, this module's similarity function swaps to that
//! endpoint without changing the public API.
//!
//! Cold-start: an empty store returns no candidates from
//! [`SkillPriorStore::top_k`]. The decomposer must tolerate the empty case
//! (plan task 8 §51 "Acceptable cold-start; explicitly documented in
//! module doc-comment").
//!
//! Configurable K via [`SkillPriorStore::with_k`]; setting `K = 0`
//! disables retrieval entirely (plan task 8 §51 "if K=3 retrieval surfaces
//! noise, set K=0 in [planning.skill_priors] to disable").
//!
//! Anti-stale-training: blake3 ONLY for any bucket-derivation hash (no
//! blake2 / sha2 per CLAUDE.md HARD RULE).

use serde::{Deserialize, Serialize};

/// Default top-K retrieval size per plan task 8 §51.
pub const DEFAULT_TOP_K: usize = 3;

/// Default number of blake3-derived buckets for the substitute-cosine
/// vector. 64 buckets is enough to discriminate ~10K stored priors while
/// staying small enough to compare cheaply.
pub const DEFAULT_BUCKET_COUNT: usize = 64;

/// One stored skill-prior. The `goal_blake3_buckets` vector is the
/// substitute "embedding": each token of the original goal text is
/// blake3-hashed and the byte modulo `bucket_count` increments the
/// corresponding bucket count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillPrior {
    /// Free-form id (callers use baby-step ids / step ids / phase ids).
    pub prior_id: String,
    /// Token-bucket vector — counts of blake3-derived buckets, normalized
    /// at retrieval time by [`SkillPriorStore::top_k`].
    pub goal_blake3_buckets: Vec<u32>,
    /// Short prose summary of the accepted diff (max ~1 KiB).
    pub accepted_diff_summary: String,
    /// Originating SSB row seqs (`crate::ssb::SsbRow::seq`) — caller-
    /// supplied so the decomposer can dereference back to the SSB store
    /// for full evidence on a retrieved prior.
    pub ssb_evidence_seqs: Vec<i64>,
    /// Wall-clock UNIX ms the prior was inserted.
    pub inserted_at_ms: i64,
}

impl SkillPrior {
    /// Build a [`SkillPrior`] from the originating goal text + accepted
    /// diff summary. Tokenizes the goal via
    /// [`super::decomposition_entropy::tokenize_simple`] and hashes each
    /// token into a blake3 bucket.
    #[must_use]
    pub fn from_goal_text(
        prior_id: String,
        goal_text: &str,
        accepted_diff_summary: String,
        ssb_evidence_seqs: Vec<i64>,
        inserted_at_ms: i64,
        bucket_count: usize,
    ) -> Self {
        let buckets = blake3_bucket_vector(goal_text, bucket_count);
        Self {
            prior_id,
            goal_blake3_buckets: buckets,
            accepted_diff_summary,
            ssb_evidence_seqs,
            inserted_at_ms,
        }
    }
}

/// One retrieved skill prior + its similarity score (cosine in `[0, 1]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillPriorMatch {
    pub prior_id: String,
    pub accepted_diff_summary: String,
    pub similarity: f32,
    pub ssb_evidence_seqs: Vec<i64>,
}

/// In-memory store of skill priors keyed by `prior_id`. Reads + writes
/// require `&mut self`; concurrent callers wrap in a lock.
#[derive(Debug, Clone, Default)]
pub struct SkillPriorStore {
    priors: Vec<SkillPrior>,
    k: usize,
    bucket_count: usize,
}

impl SkillPriorStore {
    /// New empty store with [`DEFAULT_TOP_K`] / [`DEFAULT_BUCKET_COUNT`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            priors: Vec::new(),
            k: DEFAULT_TOP_K,
            bucket_count: DEFAULT_BUCKET_COUNT,
        }
    }

    /// Store configured to return top-`k` matches on
    /// [`Self::top_k`]. Setting `k = 0` disables retrieval entirely (per
    /// plan task 8 §51).
    #[must_use]
    pub fn with_k(k: usize) -> Self {
        Self {
            priors: Vec::new(),
            k,
            bucket_count: DEFAULT_BUCKET_COUNT,
        }
    }

    /// Override the bucket count used by [`SkillPrior::from_goal_text`]
    /// callers that build priors through [`Self::insert_from_goal_text`].
    pub fn set_bucket_count(&mut self, bucket_count: usize) {
        // Clamp away zero / huge values.
        self.bucket_count = bucket_count.clamp(8, 4096);
    }

    /// Number of stored priors.
    #[must_use]
    pub fn len(&self) -> usize {
        self.priors.len()
    }

    /// True iff empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.priors.is_empty()
    }

    /// True iff retrieval is disabled (`k == 0`).
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.k == 0
    }

    /// Insert a pre-built prior.
    pub fn insert(&mut self, prior: SkillPrior) {
        self.priors.push(prior);
    }

    /// Build a prior from goal text and insert it. Returns the inserted
    /// `prior_id` so the caller can wire it into the SSB evidence chain.
    pub fn insert_from_goal_text(
        &mut self,
        prior_id: String,
        goal_text: &str,
        accepted_diff_summary: String,
        ssb_evidence_seqs: Vec<i64>,
        inserted_at_ms: i64,
    ) -> String {
        let bc = self.bucket_count;
        let prior = SkillPrior::from_goal_text(
            prior_id.clone(),
            goal_text,
            accepted_diff_summary,
            ssb_evidence_seqs,
            inserted_at_ms,
            bc,
        );
        self.priors.push(prior);
        prior_id
    }

    /// Retrieve the top-K priors by cosine similarity against the query
    /// goal text. Returns at most `self.k` matches sorted by descending
    /// similarity. Returns an empty vec when:
    ///   * the store is empty,
    ///   * `self.k == 0` (retrieval disabled),
    ///   * every candidate has similarity exactly 0.0 (no token overlap).
    ///
    /// Similarity is normalized cosine in `[0, 1]`; zero-vector inputs
    /// (e.g. empty goal text) yield similarity 0.0 (no panic).
    #[must_use]
    pub fn top_k(&self, query_goal_text: &str) -> Vec<SkillPriorMatch> {
        if self.is_disabled() || self.is_empty() {
            return Vec::new();
        }
        let query = blake3_bucket_vector(query_goal_text, self.bucket_count);
        let mut scored: Vec<(f32, &SkillPrior)> = self
            .priors
            .iter()
            .map(|p| (cosine_u32(&query, &p.goal_blake3_buckets), p))
            .filter(|(sim, _)| *sim > 0.0)
            .collect();
        // Sort descending by similarity. NaN never appears because cosine_u32
        // never returns NaN on integer-derived vectors (zero -> 0.0).
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(self.k)
            .map(|(sim, p)| SkillPriorMatch {
                prior_id: p.prior_id.clone(),
                accepted_diff_summary: p.accepted_diff_summary.clone(),
                similarity: sim,
                ssb_evidence_seqs: p.ssb_evidence_seqs.clone(),
            })
            .collect()
    }
}

/// Blake3-derived token-bucket vector. Each token in `text` is hashed
/// with blake3; the first byte of the hash modulo `bucket_count`
/// determines the bucket. Buckets are u32 counts (saturating add at
/// `u32::MAX` — overflow would require >4G tokens, not a real concern).
#[must_use]
pub fn blake3_bucket_vector(text: &str, bucket_count: usize) -> Vec<u32> {
    let bc = bucket_count.clamp(8, 4096);
    let mut buckets = vec![0_u32; bc];
    let toks = super::decomposition_entropy::tokenize_simple(text);
    for t in toks {
        let h = blake3::hash(t.as_bytes());
        // first byte of the 32-byte hash modulo bucket_count
        let first = h.as_bytes()[0];
        let idx = (first as usize) % bc;
        buckets[idx] = buckets[idx].saturating_add(1);
    }
    buckets
}

/// Cosine similarity between two u32 bucket vectors in `[0, 1]`. Zero-norm
/// vectors yield 0.0.
#[must_use]
pub fn cosine_u32(a: &[u32], b: &[u32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot: u64 = 0;
    let mut na: u64 = 0;
    let mut nb: u64 = 0;
    for i in 0..a.len() {
        let ai = a[i] as u64;
        let bi = b[i] as u64;
        dot += ai * bi;
        na += ai * ai;
        nb += bi * bi;
    }
    if na == 0 || nb == 0 {
        return 0.0;
    }
    let denom = ((na as f64).sqrt() * (nb as f64).sqrt()) as f32;
    if denom <= 0.0 {
        return 0.0;
    }
    let sim = (dot as f32) / denom;
    sim.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_vector_deterministic() {
        let v1 = blake3_bucket_vector("user submits form validates schema", 32);
        let v2 = blake3_bucket_vector("user submits form validates schema", 32);
        assert_eq!(v1, v2);
        // total count == token count == 5
        let total: u32 = v1.iter().sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn bucket_vector_bucket_count_clamped() {
        let v_too_small = blake3_bucket_vector("hello world", 1);
        assert_eq!(v_too_small.len(), 8); // clamped to min
        let v_too_big = blake3_bucket_vector("hello world", 100_000);
        assert_eq!(v_too_big.len(), 4096); // clamped to max
    }

    #[test]
    fn cosine_identity_one() {
        let v = blake3_bucket_vector("user submits form validates schema endpoint", 64);
        let sim = cosine_u32(&v, &v);
        assert!((sim - 1.0).abs() < 1e-4, "got sim={}", sim);
    }

    #[test]
    fn cosine_zero_on_disjoint() {
        // Force disjoint buckets by using strings whose blake3 hits different
        // mod buckets. Since blake3 is deterministic, we just need very
        // different tokens AND ensure no incidental collision — use 64 buckets
        // so collision risk on 4 distinct tokens is low.
        let v1 = vec![1_u32, 0, 0, 0, 0, 0, 0, 0];
        let v2 = vec![0_u32, 1, 0, 0, 0, 0, 0, 0];
        let sim = cosine_u32(&v1, &v2);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn cosine_zero_on_empty() {
        let empty: Vec<u32> = Vec::new();
        let v = vec![1_u32, 2, 3];
        assert_eq!(cosine_u32(&empty, &v), 0.0);
        assert_eq!(cosine_u32(&v, &empty), 0.0);
    }

    #[test]
    fn cosine_zero_on_length_mismatch() {
        let a = vec![1_u32, 2];
        let b = vec![1_u32, 2, 3];
        assert_eq!(cosine_u32(&a, &b), 0.0);
    }

    #[test]
    fn store_empty_top_k_returns_empty() {
        let store = SkillPriorStore::new();
        let matches = store.top_k("any query goal");
        assert!(matches.is_empty());
    }

    #[test]
    fn store_disabled_top_k_returns_empty() {
        let mut store = SkillPriorStore::with_k(0);
        store.insert(SkillPrior::from_goal_text(
            "p1".into(),
            "user submits form validates schema",
            "diff summary one".into(),
            vec![1, 2],
            1_700_000_000_000,
            DEFAULT_BUCKET_COUNT,
        ));
        assert!(store.is_disabled());
        let matches = store.top_k("user submits form validates schema");
        assert!(matches.is_empty());
    }

    #[test]
    fn store_top_k_returns_best_match_first() {
        let mut store = SkillPriorStore::new();
        store.insert_from_goal_text(
            "p1".into(),
            "user submits form validates schema endpoint database",
            "diff summary one".into(),
            vec![10],
            1_700_000_000_000,
        );
        store.insert_from_goal_text(
            "p2".into(),
            "completely unrelated goal about graphics rendering pipelines",
            "diff summary two".into(),
            vec![20],
            1_700_000_001_000,
        );
        let matches = store.top_k("user submits form validates schema");
        assert!(!matches.is_empty(), "expected at least 1 match");
        // First match must be the closer one (p1).
        assert_eq!(matches[0].prior_id, "p1");
        // Cap is DEFAULT_TOP_K=3 but only 2 priors, only 1 has non-zero
        // similarity in the typical case; matches.len() in {1, 2}.
        assert!(matches.len() <= DEFAULT_TOP_K);
    }

    #[test]
    fn store_top_k_respects_k_cap() {
        let mut store = SkillPriorStore::with_k(1);
        for i in 0..5 {
            store.insert_from_goal_text(
                format!("p{}", i),
                "user submits form validates schema",
                format!("diff summary {}", i),
                vec![i as i64],
                1_700_000_000_000 + (i as i64),
            );
        }
        let matches = store.top_k("user submits form validates schema");
        assert_eq!(matches.len(), 1);
    }
}
