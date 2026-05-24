//! E-1 — A-MEM Zettelkasten attribute-evolution loop on note insert
//! (Xu et al. arXiv:2502.12110 §3.2 + §3.3) consolidated with Mem0's
//! ADD/UPDATE/DELETE/NOOP dispatcher pattern (Chhikara et al. arXiv:
//! 2504.19413 §3.2).
//!
//! When a new candidate note is inserted the A-MEM paper specifies an
//! evolution loop that updates 0-N already-stored neighbour notes:
//!   * `tags` may absorb new tags from the inserted note (set union)
//!   * `links` may gain a back-reference to the inserted note
//!   * `context` may be appended with a one-line provenance note
//!
//! This module is the producer the orchestrator wraps with an
//! [`crate::SsbActionType::MemoryNoteEvolved`] emit per diff returned
//! (one SSB row per attribute change). The function itself is pure —
//! it returns the diff list and the caller batches the write +
//! emission.
//!
//! Bound: 1-hop neighbours only (per E-1 plan Task 2 Open Question 1
//! "<10% body-edit churn after turn 50"). Multi-hop expansion is left
//! to a future paper-ingestor pass.

use crate::error::MemoryError;
use crate::notes::{MemoryNote, NoteStore};
use blake3;

/// Hard cap on how many linked-neighbour notes one insert can mutate.
/// Mirrors A-MEM's stability finding — past ~16 the loop thrashes on
/// 7B-class models (E-1 plan Task 2 OQ1).
pub const MAX_NEIGHBOURS_PER_INSERT: usize = 16;

/// One attribute-level mutation applied to an existing `memory_notes`
/// row by [`evolve_on_insert`]. The orchestrator wraps each diff in a
/// [`crate::SsbActionType::MemoryNoteEvolved`] emit; the underlying
/// row is updated atomically via [`apply_diff`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeDiff {
    /// `note_id` of the row being mutated.
    pub note_id: String,
    /// Per-field changes — at least one of the four fields is set.
    pub kind: AttributeDiffKind,
    /// Blake3 of the body BEFORE this diff (so the orchestrator can
    /// fill `MemoryNoteEvolvedPayload::old_content_blake3` without a
    /// re-read).
    pub old_blake3: String,
    /// Blake3 of the body AFTER this diff (`new_content_blake3`).
    /// Identical to `old_blake3` when the diff touches tags/links
    /// only — the body itself is unchanged.
    pub new_blake3: String,
}

/// Discriminated payload describing one attribute-level mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeDiffKind {
    /// New tags appended (set union with existing).
    TagsAdded {
        /// Tags that were not previously on the neighbour note.
        added: Vec<String>,
    },
    /// New `note_id` link appended to `links` (Zettelkasten back-edge).
    LinkAdded {
        /// The `note_id` reference appended.
        target: String,
    },
    /// `context` text appended (one-line provenance prepend).
    ContextAppended {
        /// The new line appended.
        line: String,
    },
}

impl AttributeDiff {
    /// Number of new tags this diff added (0 for non-Tag diffs).
    pub fn tags_added_count(&self) -> u32 {
        match &self.kind {
            AttributeDiffKind::TagsAdded { added } => added.len() as u32,
            _ => 0,
        }
    }

    /// Number of new links this diff added (0 for non-Link diffs).
    pub fn links_added_count(&self) -> u32 {
        match &self.kind {
            AttributeDiffKind::LinkAdded { .. } => 1,
            _ => 0,
        }
    }
}

/// Apply a single [`AttributeDiff`] to its target row.
///
/// Atomic per-row UPDATE — the caller wraps a multi-diff batch in a
/// SQLite transaction at the orchestrator boundary if it needs
/// cross-row atomicity.
pub fn apply_diff(store: &NoteStore, diff: &AttributeDiff) -> Result<(), MemoryError> {
    let mut existing = store
        .get(&diff.note_id)?
        .ok_or_else(|| MemoryError::NotFound(diff.note_id.clone()))?;
    match &diff.kind {
        AttributeDiffKind::TagsAdded { added } => {
            for t in added {
                if !existing.tags.iter().any(|x| x == t) {
                    existing.tags.push(t.clone());
                }
            }
        }
        AttributeDiffKind::LinkAdded { target } => {
            if !existing.links.iter().any(|x| x == target) {
                existing.links.push(target.clone());
            }
        }
        AttributeDiffKind::ContextAppended { line } => {
            if !existing.context.is_empty() {
                existing.context.push('\n');
            }
            existing.context.push_str(line);
        }
    }
    store.replace_attributes(&existing)?;
    Ok(())
}

/// A-MEM attribute-evolution pass on insert.
///
/// Compares the candidate `new` against each `linked` neighbour and
/// returns the list of [`AttributeDiff`] mutations the caller should
/// apply (+ emit on SSB as `MemoryNoteEvolved` rows).
///
/// Rules (mirror A-MEM §3.2):
///   * any neighbour sharing >= 1 tag with `new` AND missing tags that
///     `new` carries gets a [`AttributeDiffKind::TagsAdded`] diff.
///   * any neighbour whose `links` does not already point back at `new`
///     gets a [`AttributeDiffKind::LinkAdded`] diff (back-edge).
///   * neighbours with empty `context` get a single one-line
///     `[ContextAppended]` diff naming the inserted note's id.
///
/// Bounded by [`MAX_NEIGHBOURS_PER_INSERT`].
pub fn evolve_on_insert(
    new: &MemoryNote,
    linked: &[MemoryNote],
) -> Vec<AttributeDiff> {
    let mut out = Vec::new();
    let bound = linked.len().min(MAX_NEIGHBOURS_PER_INSERT);
    for neighbour in linked.iter().take(bound) {
        if neighbour.note_id == new.note_id {
            continue;
        }

        // TagsAdded — only if there is overlap (anchor) AND novelty.
        let overlap: bool = neighbour
            .tags
            .iter()
            .any(|t| new.tags.iter().any(|nt| nt == t));
        if overlap {
            let novel: Vec<String> = new
                .tags
                .iter()
                .filter(|nt| !neighbour.tags.iter().any(|t| t == *nt))
                .cloned()
                .collect();
            if !novel.is_empty() {
                let old_blake = blake3_hex(&neighbour.body);
                out.push(AttributeDiff {
                    note_id: neighbour.note_id.clone(),
                    kind: AttributeDiffKind::TagsAdded { added: novel },
                    old_blake3: old_blake.clone(),
                    new_blake3: old_blake,
                });
            }
        }

        // LinkAdded — back-edge if missing.
        let has_link = neighbour
            .links
            .iter()
            .any(|l| l == &new.note_id);
        if !has_link && overlap {
            let old_blake = blake3_hex(&neighbour.body);
            out.push(AttributeDiff {
                note_id: neighbour.note_id.clone(),
                kind: AttributeDiffKind::LinkAdded {
                    target: new.note_id.clone(),
                },
                old_blake3: old_blake.clone(),
                new_blake3: old_blake,
            });
        }

        // ContextAppended — only on empty-context neighbours, single-shot.
        if overlap && neighbour.context.trim().is_empty() {
            let line = format!("linked from note {}", new.note_id);
            let old_blake = blake3_hex(&neighbour.body);
            out.push(AttributeDiff {
                note_id: neighbour.note_id.clone(),
                kind: AttributeDiffKind::ContextAppended { line },
                old_blake3: old_blake.clone(),
                new_blake3: old_blake,
            });
        }
    }
    out
}

/// HippoRAG-named alias for [`crate::hippo_rag::retrieve_notes`].
///
/// Per the E-1 WIRING_CONTRACT this surface is called
/// `ppr_seeded_top_k(entities, k)` — the underlying implementation
/// (Personalised PageRank seeded by tag/keyword intersection) lives in
/// [`crate::hippo_rag`]. This thin wrapper preserves the contract
/// name; new callers should prefer it for clarity.
pub fn ppr_seeded_top_k(
    store: &NoteStore,
    entities: &[String],
    k: usize,
    now_ms: u128,
) -> Result<Vec<crate::hippo_rag::RetrievedNote>, MemoryError> {
    crate::hippo_rag::retrieve_notes(store, entities, k, now_ms)
}

fn blake3_hex(body: &str) -> String {
    blake3::hash(body.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(id: &str, body: &str, tags: &[&str], links: &[&str]) -> MemoryNote {
        MemoryNote {
            note_id: id.into(),
            scope: "session".into(),
            tags: tags.iter().map(|s| (*s).into()).collect(),
            keywords: vec![],
            context: String::new(),
            body: body.into(),
            links: links.iter().map(|s| (*s).into()).collect(),
            fact_type: "note".into(),
            created_at_ms: 1,
            last_accessed_ms: 1,
            valid_to_ms: None,
            trust_score: 0.8,
        }
    }

    #[test]
    fn evolve_emits_tag_add_on_overlap_with_novelty() {
        let new = note("new", "x", &["hash", "blake3", "rust"], &[]);
        let n1 = note("n1", "old", &["hash"], &[]);
        let diffs = evolve_on_insert(&new, &[n1]);
        let tags: Vec<_> = diffs
            .iter()
            .filter_map(|d| match &d.kind {
                AttributeDiffKind::TagsAdded { added } => Some(added.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], vec!["blake3".to_string(), "rust".to_string()]);
    }

    #[test]
    fn evolve_emits_link_back_edge_on_overlap() {
        let new = note("new", "x", &["hash"], &[]);
        let n1 = note("n1", "old", &["hash"], &[]);
        let diffs = evolve_on_insert(&new, &[n1]);
        let links: Vec<_> = diffs
            .iter()
            .filter_map(|d| match &d.kind {
                AttributeDiffKind::LinkAdded { target } => Some(target.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(links, vec!["new".to_string()]);
    }

    #[test]
    fn evolve_no_diff_on_zero_overlap() {
        let new = note("new", "x", &["frontend"], &[]);
        let n1 = note("n1", "old", &["network"], &[]);
        let diffs = evolve_on_insert(&new, &[n1]);
        assert!(diffs.is_empty());
    }

    #[test]
    fn evolve_skips_self() {
        let new = note("a", "x", &["t"], &[]);
        let same = note("a", "x", &["t"], &[]);
        let diffs = evolve_on_insert(&new, &[same]);
        assert!(diffs.is_empty());
    }

    #[test]
    fn evolve_bounded_by_cap() {
        let new = note("new", "x", &["t"], &[]);
        let many: Vec<MemoryNote> = (0..(MAX_NEIGHBOURS_PER_INSERT + 5))
            .map(|i| note(&format!("n{i}"), "y", &["t", "novel-tag"], &[]))
            .collect();
        let diffs = evolve_on_insert(&new, &many);
        // Cap is on neighbour iteration, not on emitted diffs — but for
        // empty-context neighbours the upper bound on diffs is 2 per
        // neighbour (LinkAdded + ContextAppended; no TagsAdded since
        // each neighbour already has "novel-tag" anchor + "t"). Both
        // possible diffs per neighbour, capped at MAX_NEIGHBOURS_PER_INSERT
        // neighbours.
        assert!(diffs.len() <= MAX_NEIGHBOURS_PER_INSERT * 2);
    }

    #[test]
    fn apply_link_diff_is_idempotent() {
        let store = NoteStore::open_in_memory().expect("store");
        let n1 = note("n1", "body", &["t"], &[]);
        store.insert(&n1).expect("ins");
        let diff = AttributeDiff {
            note_id: "n1".into(),
            kind: AttributeDiffKind::LinkAdded { target: "x".into() },
            old_blake3: blake3_hex(&n1.body),
            new_blake3: blake3_hex(&n1.body),
        };
        apply_diff(&store, &diff).expect("apply1");
        apply_diff(&store, &diff).expect("apply2");
        let got = store.get("n1").expect("get").expect("some");
        assert_eq!(got.links, vec!["x".to_string()]);
    }

    #[test]
    fn ppr_alias_returns_same_shape_as_underlying() {
        let store = NoteStore::open_in_memory().expect("store");
        let mut n = note("a", "blake3 preference", &["hash", "blake3"], &[]);
        n.keywords = vec!["blake3".into()];
        store.insert(&n).expect("ins");
        let via_alias = ppr_seeded_top_k(&store, &["blake3".into()], 3, 1_000_000)
            .expect("alias");
        let via_direct =
            crate::hippo_rag::retrieve_notes(&store, &["blake3".into()], 3, 2_000_000)
                .expect("direct");
        assert_eq!(via_alias.len(), via_direct.len());
        assert!(!via_alias.is_empty());
        assert_eq!(via_alias[0].note.note_id, "a");
    }
}
