//! E-1 — HippoRAG-style Personalized PageRank retrieval over the
//! `memory_notes` graph (Gutiérrez et al. NeurIPS 2024, arXiv:2405.14831
//! §3.2 / §3.3).
//!
//! The walker is a single in-memory PPR pass (no IRCoT inner loop —
//! HippoRAG's central efficiency win). Adjacency is built from each
//! note's `links` JSON column plus a tag-intersection edge (notes that
//! share `>= 1` tag get an undirected edge with weight = `|tags_a ∩
//! tags_b| / |tags_a ∪ tags_b|`, Jaccard).
//!
//! Constants below are bounded by the paper's stated defaults:
//!   * `PPR_DAMPING = 0.5` — HippoRAG §3.3 reports 0.5 as the search
//!     default; this is the same value as the original Pagerank paper's
//!     `1 - alpha` reset probability when alpha=0.5.
//!   * `PPR_MAX_ITERATIONS = 50` — power-iteration cap; HippoRAG §3.3
//!     converges within ~30 steps on typical fact-graph sizes, 50 is a
//!     safe ceiling.
//!   * `PPR_CONVERGENCE_TOL = 1e-6` — L1-norm delta tolerance; once the
//!     score vector moves by < tol in one iteration, the walk halts.

use rustc_hash::FxHashMap;

use crate::error::MemoryError;
use crate::notes::{MemoryNote, NoteStore};

/// PPR damping factor (= `1 - reset_probability`). HippoRAG §3.3 default.
pub const PPR_DAMPING: f64 = 0.5;

/// Hard ceiling on power-iteration count.
pub const PPR_MAX_ITERATIONS: usize = 50;

/// L1-norm convergence tolerance. Once the absolute change in score
/// across all nodes drops below this between two iterations, the walk
/// halts early.
pub const PPR_CONVERGENCE_TOL: f64 = 1e-6;

/// One retrieved row + the PPR score the walker assigned it.
#[derive(Debug, Clone)]
pub struct RetrievedNote {
    /// The underlying row.
    pub note: MemoryNote,
    /// PPR score in `[0, 1]` — higher = more relevant to the seed set.
    pub score: f64,
}

/// Personalized PageRank top-K retrieval over the `memory_notes` graph.
///
/// `seed_keywords` is the entity / keyword set from the current chat
/// turn (HippoRAG's "query nodes"). Notes whose `tags` or `keywords`
/// intersect the seed set become the initial probability-mass anchors
/// (uniform mass across matching nodes); when no notes match the seed,
/// the walker falls back to uniform-over-all-current notes so the
/// caller always receives a result.
///
/// On read the walker also bumps `last_accessed_ms` on returned rows
/// (feeds the [`crate::decay`] curve — frequent reads slow decay).
pub fn retrieve_notes(
    store: &NoteStore,
    seed_keywords: &[String],
    k: usize,
    now_ms: u128,
) -> Result<Vec<RetrievedNote>, MemoryError> {
    let candidates = store.list_recent(10_000)?;
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let n = candidates.len();

    // Build node-id -> index map.
    let mut id_to_idx: FxHashMap<String, usize> = FxHashMap::default();
    for (idx, note) in candidates.iter().enumerate() {
        id_to_idx.insert(note.note_id.clone(), idx);
    }

    // Adjacency (weighted, undirected). Stored as Vec<Vec<(usize, f64)>>
    // for cache-friendly iteration; no fancy CSR needed at 10K-note scale.
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    let mut out_weight: Vec<f64> = vec![0.0; n];

    // Edges from `links` JSON column (directed in storage but treated
    // as undirected for PPR — symmetrize).
    for (i, note) in candidates.iter().enumerate() {
        for tgt_id in &note.links {
            if let Some(&j) = id_to_idx.get(tgt_id) {
                if i == j {
                    continue;
                }
                adj[i].push((j, 1.0));
                adj[j].push((i, 1.0));
                out_weight[i] += 1.0;
                out_weight[j] += 1.0;
            }
        }
    }

    // Tag-intersection edges (Jaccard). O(n^2) which is fine at the
    // 10K-note ceiling we cap candidates at; for larger graphs an
    // inverted index by tag is the canonical optimization (deferred —
    // not needed at this volume).
    for i in 0..n {
        let tags_i: rustc_hash::FxHashSet<&str> =
            candidates[i].tags.iter().map(String::as_str).collect();
        if tags_i.is_empty() {
            continue;
        }
        for j in (i + 1)..n {
            let tags_j: rustc_hash::FxHashSet<&str> =
                candidates[j].tags.iter().map(String::as_str).collect();
            if tags_j.is_empty() {
                continue;
            }
            let inter = tags_i.intersection(&tags_j).count();
            if inter == 0 {
                continue;
            }
            let union = tags_i.union(&tags_j).count().max(1);
            let weight = inter as f64 / union as f64;
            if weight <= 0.0 {
                continue;
            }
            adj[i].push((j, weight));
            adj[j].push((i, weight));
            out_weight[i] += weight;
            out_weight[j] += weight;
        }
    }

    // Seed (personalization) vector — uniform over notes whose tags or
    // keywords match any seed keyword.
    let seed_set: rustc_hash::FxHashSet<String> =
        seed_keywords.iter().map(|s| s.to_lowercase()).collect();
    let mut seed_idxs: Vec<usize> = Vec::new();
    if !seed_set.is_empty() {
        for (idx, note) in candidates.iter().enumerate() {
            let hit_tag = note
                .tags
                .iter()
                .any(|t| seed_set.contains(&t.to_lowercase()));
            let hit_kw = note
                .keywords
                .iter()
                .any(|t| seed_set.contains(&t.to_lowercase()));
            if hit_tag || hit_kw {
                seed_idxs.push(idx);
            }
        }
    }
    let mut personalization = vec![0.0f64; n];
    if seed_idxs.is_empty() {
        // Fallback: uniform over every current note.
        let p = 1.0 / n as f64;
        for v in personalization.iter_mut() {
            *v = p;
        }
    } else {
        let p = 1.0 / seed_idxs.len() as f64;
        for &i in &seed_idxs {
            personalization[i] = p;
        }
    }

    // Power-iterate.
    let mut score = personalization.clone();
    let damping = PPR_DAMPING;
    let reset = 1.0 - damping;
    for _ in 0..PPR_MAX_ITERATIONS {
        let mut next = vec![0.0f64; n];
        // Random-jump component.
        for i in 0..n {
            next[i] = reset * personalization[i];
        }
        // Walk component — distribute each node's score across its
        // weighted out-edges; dangling nodes (no edges) leak mass back
        // to the personalization vector so probability is conserved.
        let mut dangling_mass = 0.0;
        for i in 0..n {
            if out_weight[i] <= 0.0 {
                dangling_mass += score[i];
                continue;
            }
            let share = damping * score[i] / out_weight[i];
            for &(j, w) in &adj[i] {
                next[j] += share * w;
            }
        }
        if dangling_mass > 0.0 {
            let leak = damping * dangling_mass;
            for i in 0..n {
                next[i] += leak * personalization[i];
            }
        }

        // Convergence check.
        let delta: f64 = next
            .iter()
            .zip(score.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        score = next;
        if delta < PPR_CONVERGENCE_TOL {
            break;
        }
    }

    // Rank + apply bi-temporal decay weight to the score (notes whose
    // last_accessed is stale lose ranking weight monotonically).
    let mut scored: Vec<(usize, f64)> = (0..n).map(|i| (i, score[i])).collect();
    for (idx, s) in scored.iter_mut() {
        let note = &candidates[*idx];
        let decay_factor = trust_decay_factor(note, now_ms);
        *s *= decay_factor;
    }
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(k);

    // Materialize + touch.
    let mut out = Vec::with_capacity(scored.len());
    for (idx, score) in scored.into_iter() {
        let mut note = candidates[idx].clone();
        // Best-effort touch — failure here must not deny the caller a
        // result (mirrors retrieval.rs's get_relevant_facts contract).
        match store.touch(&note.note_id, now_ms) {
            Ok(()) => note.last_accessed_ms = now_ms,
            Err(e) => tracing::warn!(
                target = "memory.hippo_rag",
                note_id = %note.note_id, err = %e,
                "touch failed on retrieval (non-fatal)"
            ),
        }
        out.push(RetrievedNote { note, score });
    }
    Ok(out)
}

/// Compute the bi-temporal decay multiplier applied to a note's PPR
/// score on retrieval. Per Zep §2.2.3 + E-5's `decay_rate_per_day`
/// model — older + un-touched notes lose ranking weight monotonically.
fn trust_decay_factor(note: &MemoryNote, now_ms: u128) -> f64 {
    if now_ms <= note.last_accessed_ms {
        return note.trust_score as f64;
    }
    let elapsed_ms = now_ms - note.last_accessed_ms;
    let elapsed_days = (elapsed_ms as f64) / 86_400_000.0;
    // Conservative half-life ~30 days for unparameterized notes; matches
    // the report's "30d/7d/1h" smoke fixture in the E-1 plan ACCEPTANCE
    // section without requiring a per-row `decay_rate_per_day` column.
    let rate_per_day = 0.023; // ln(2)/30 ~ 0.0231, ~30d half-life
    let factor = (-(rate_per_day) * elapsed_days).exp();
    (note.trust_score as f64) * factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::MemoryNote;

    fn insert_note(
        store: &NoteStore,
        id: &str,
        body: &str,
        tags: &[&str],
        links: &[&str],
        created_ms: u128,
    ) {
        let n = MemoryNote {
            note_id: id.into(),
            scope: "session".into(),
            tags: tags.iter().map(|s| (*s).into()).collect(),
            keywords: vec![],
            context: "".into(),
            body: body.into(),
            links: links.iter().map(|s| (*s).into()).collect(),
            fact_type: "note".into(),
            created_at_ms: created_ms,
            last_accessed_ms: created_ms,
            valid_to_ms: None,
            trust_score: 0.9,
        };
        store.insert(&n).expect("insert");
    }

    #[test]
    fn empty_store_returns_empty() {
        let store = NoteStore::open_in_memory().expect("store");
        let got = retrieve_notes(&store, &["blake3".into()], 5, 0).expect("retrieve");
        assert!(got.is_empty());
    }

    #[test]
    fn seed_match_ranks_first() {
        let store = NoteStore::open_in_memory().expect("store");
        insert_note(&store, "a", "blake3 preference", &["hash", "blake3"], &[], 1);
        insert_note(&store, "b", "unrelated", &["frontend"], &[], 1);
        insert_note(&store, "c", "also unrelated", &["network"], &[], 1);
        let got = retrieve_notes(&store, &["blake3".into()], 3, 1_000_000).expect("ppr");
        assert!(!got.is_empty());
        assert_eq!(got[0].note.note_id, "a");
    }

    #[test]
    fn linked_notes_propagate_mass() {
        let store = NoteStore::open_in_memory().expect("store");
        // a is seeded; b links to a; c is isolated. b should outrank c.
        insert_note(&store, "a", "hash", &["hash"], &[], 1);
        insert_note(&store, "b", "second", &["link-target"], &["a"], 1);
        insert_note(&store, "c", "unrelated", &["x"], &[], 1);
        let got = retrieve_notes(&store, &["hash".into()], 3, 1_000_000).expect("ppr");
        let order: Vec<&str> = got.iter().map(|r| r.note.note_id.as_str()).collect();
        let pos_b = order.iter().position(|s| *s == "b");
        let pos_c = order.iter().position(|s| *s == "c");
        if let (Some(pb), Some(pc)) = (pos_b, pos_c) {
            assert!(pb < pc, "b (linked to seed) must outrank isolated c");
        }
    }

    #[test]
    fn no_seed_match_returns_uniform_fallback() {
        let store = NoteStore::open_in_memory().expect("store");
        insert_note(&store, "a", "one", &["x"], &[], 1);
        insert_note(&store, "b", "two", &["y"], &[], 1);
        let got = retrieve_notes(&store, &["nonexistent".into()], 5, 1_000_000).expect("ppr");
        // Both notes appear (uniform fallback) even though no seed matched.
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn retrieval_touches_last_accessed() {
        let store = NoteStore::open_in_memory().expect("store");
        insert_note(&store, "a", "one", &["hash"], &[], 1);
        let now = 5_000_000u128;
        let _ = retrieve_notes(&store, &["hash".into()], 1, now).expect("ppr");
        let got = store.get("a").expect("get").expect("some");
        assert_eq!(got.last_accessed_ms, now);
    }

    #[test]
    fn trust_decay_factor_monotone() {
        let note = MemoryNote {
            note_id: "n".into(),
            scope: "session".into(),
            tags: vec![],
            keywords: vec![],
            context: "".into(),
            body: "b".into(),
            links: vec![],
            fact_type: "note".into(),
            created_at_ms: 0,
            last_accessed_ms: 0,
            valid_to_ms: None,
            trust_score: 1.0,
        };
        let day_ms = 86_400_000u128;
        let f1 = trust_decay_factor(&note, day_ms);
        let f30 = trust_decay_factor(&note, 30 * day_ms);
        let f365 = trust_decay_factor(&note, 365 * day_ms);
        assert!(f1 > f30, "1d > 30d");
        assert!(f30 > f365, "30d > 365d");
        // After ~30 days the factor should be ~0.5 (half-life ~30d).
        assert!((f30 - 0.5).abs() < 0.05, "approx half at ~30d, got {f30}");
    }
}
