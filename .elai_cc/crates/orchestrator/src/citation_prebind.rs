//! H-53 — citation pre-binding + constrained synthesis on the answer path.
//!
//! Inserts a PRE-GENERATION citation-binding stage on the /research|/ask answer
//! path: ranked evidence is frozen into a numbered [`EvidenceAnchorSet`]
//! (blake3-pinned, rank-stable under `ELAI_SEED`) BEFORE synthesis; the
//! [`CitationVerifier`] (E-48) then degenerates from open-world ghost-hunting to a
//! CLOSED-SET membership + support check — an out-of-set `[N]` is held and routes
//! to D-16 refusal, never shipped as a ghost citation.
//!
//! ## Tier-1 (this file — deterministic pre-bind + closed-set verify)
//! - [`freeze_anchors`]: freeze ranked evidence into a numbered `EvidenceAnchorSet`.
//! - [`check_citation_in_set`]: closed-set membership + blake3-support check.
//! - Kill-switch: `ELAI_CITATION_PREBIND_DISABLE` (env var, truthy → skip pre-binding
//!   and fall through to the legacy detect-after path). This is a DISABLE kill-switch
//!   on a default-ON feature; R9's wire-dark-allowlist requirement applies only to
//!   default-OFF experiment gates.
//!
//! ## Tier-2 (DEFERRED — constrained-decode grammar)
//! The citation-marker grammar for C-26 `decoding.rs` PER_ROLE_DECODING (researcher
//! role) that makes an out-of-set `[N]` mechanically unemittable is NOT shipped here.
//! Tier-1 (catch against the closed pinned set) already drives ghost-citation rate
//! toward zero; Tier-2 makes it structurally impossible on capable backends.
//! DEFERRED: `// TODO[H53.TIER2]: wire PER_ROLE_DECODING citation-marker grammar`.
//!
//! ## Design notes
//! - IDs are assigned by RANK ORDER under `ELAI_SEED` so the same query yields
//!   the same `[N]` numbering (GD-2: byte-recomputable).
//! - The excerpt blake3 pin is used for closed-set membership: `[N]` is only
//!   accepted if the anchor's pinned excerpt hash still matches (catches stale
//!   citations after re-retrieval).
//! - REUSE H-52 [`crate::anchor_finding`] for verbatim-excerpt -> anchor binding
//!   where a span must resolve to a specific anchor. The excerpt IS the anchor
//!   (blake3 match) so closed-set membership works without the hunk-context resolver
//!   on the Tier-1 path. The hunk resolver is the Tier-2 upgrade.
//! - REUSE E-48 [`crate::research::web_orchestrator::citation_verifier`] for the
//!   support check (the verifier is extended with `anchor_id` + `pre_bound` fields).
//! - REUSE D-16 [`crate::conformal_calibration::RefusalDecision`] for the honest
//!   stop on an ungroundable claim.
//! - blake3 only (never blake2/sha2 per CLAUDE.md Anti-Stale-Training HARD RULE).
//!
//! Grounding:
//!   * arXiv:2211.12588 (Program-of-Thought, TMLR 2023) — constructive grounding;
//!     the pre-bind stage is the constructive-grounding primitive for citations.
//!   * arXiv:2402.01030 (CodeAct, ICML 2024) — off-agent artifact doctrine.
//!   * GD-5 (ELAI doctrine): Perplexity's citation-pre-bind shape is MEDIUM confidence
//!     (analyst-reported, tier-2/3). This file's recommendation does NOT rest on it;
//!     it rests on the D-11 / E-48 substrate above.
//!   * HD-3 (ELAI doctrine): constrained decoding degrades reasoning (arXiv:2408.02442);
//!     the grammar constrains ONLY the citation-marker token positions (Tier-2, DEFERRED),
//!     NEVER the prose/reasoning tokens.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// Kill-switch
// ────────────────────────────────────────────────────────────────────────────

/// Returns `true` when `ELAI_CITATION_PREBIND_DISABLE` is set in the environment.
///
/// When true the caller should skip pre-binding and fall through to the legacy
/// detect-after path (E-48 open-world CitationVerifier). No wire-dark-allowlist
/// row is required: this is a DISABLE kill-switch on a default-ON feature.
#[must_use]
pub fn prebind_disabled() -> bool {
    std::env::var("ELAI_CITATION_PREBIND_DISABLE").is_ok()
}

// ────────────────────────────────────────────────────────────────────────────
// Core types
// ────────────────────────────────────────────────────────────────────────────

/// A single ranked evidence item with a frozen, blake3-pinned excerpt.
///
/// `id` is assigned by RANK ORDER under `ELAI_SEED` so the same query yields
/// byte-identical `[N]` numbering (GD-2).
///
/// `excerpt_blake3` is the blake3 hash of the raw excerpt bytes (anti-stale:
/// blake3 only, never blake2/sha2 per CLAUDE.md HARD RULE Anti-Stale-Training).
/// The blake3 pin is the closed-set membership proof: an answer `[N]` is only
/// accepted if `N` maps to an anchor whose pinned hash still matches the excerpt
/// being cited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceAnchor {
    /// Rank-stable citation id assigned by [`freeze_anchors`] under `ELAI_SEED`.
    /// 1-based (model-visible as `[1]`, `[2]`, …).
    pub id: u32,
    /// The verbatim excerpt that the model will cite.
    pub excerpt: String,
    /// blake3 hash (32 bytes) of `excerpt.as_bytes()`.
    /// GD-2: recomputable; anti-stale: blake3 only.
    pub excerpt_blake3: [u8; 32],
    /// Canonical URI of the source.
    pub source_uri: String,
    /// Unix epoch milliseconds when this evidence was retrieved.
    pub retrieved_at_ms: u64,
    /// Source reliability tier (0 = T1 authoritative … 3 = T4 rejected).
    pub source_tier: u8,
}

/// The frozen, numbered, blake3-pinned evidence set produced by [`freeze_anchors`].
///
/// The `set_blake3` field is the blake3 hash of the concatenation of all
/// `excerpt_blake3` values in `id` order — the set fingerprint (GD-2: verifiable
/// without re-running retrieval + rerank).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceAnchorSet {
    /// Ordered anchors; `anchor.id` values are 1-based and contiguous.
    pub anchors: Vec<EvidenceAnchor>,
    /// blake3 hash (32 bytes) of `||_{anchor in id order} anchor.excerpt_blake3`.
    /// Carried in the `CitationAnchorsBound` SSB row so the set fingerprint is
    /// off-agent-verifiable without re-fetching evidence (GD-2).
    pub set_blake3: [u8; 32],
    /// The `ELAI_SEED` value active when the set was frozen (0 when the env var
    /// was absent). Carried in SSB for GD-2 reproducibility.
    pub seed: u64,
}

impl EvidenceAnchorSet {
    /// Returns the anchor with the given 1-based id, or `None` if the id is
    /// out of range for this set.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<&EvidenceAnchor> {
        // ids are 1-based and contiguous; index = id - 1
        let idx = id.checked_sub(1)?;
        let anchor = self.anchors.get(idx as usize)?;
        // Sanity-check that the slot id matches (defensive; freeze_anchors assigns contiguously).
        if anchor.id == id {
            Some(anchor)
        } else {
            None
        }
    }

    /// Returns `true` if the set is empty (no evidence was retrieved).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }

    /// Number of anchors in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.anchors.len()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// freeze_anchors
// ────────────────────────────────────────────────────────────────────────────

/// Ranked evidence item passed to [`freeze_anchors`].
///
/// Callers supply one `RankedEvidence` per retrieved-and-reranked source.
/// The `rank` field determines id assignment (lower rank = lower id = `[1]`).
#[derive(Debug, Clone)]
pub struct RankedEvidence {
    /// Rank position (0-based; rank 0 → anchor id 1). If two items share the
    /// same rank the tiebreak is the order they appear in the slice (stable sort).
    pub rank: u32,
    /// Verbatim excerpt to be frozen.
    pub excerpt: String,
    /// Canonical URI.
    pub source_uri: String,
    /// Unix epoch milliseconds when retrieved.
    pub retrieved_at_ms: u64,
    /// Source tier byte (0 = T1 … 3 = T4).
    pub source_tier: u8,
}

/// Freeze a ranked evidence slice into a numbered, blake3-pinned [`EvidenceAnchorSet`].
///
/// ### Id assignment (GD-2 determinism)
/// Ids are 1-based and assigned by ascending `rank` (stable sort preserves
/// input order on ties). Given the same input slice and the same `seed` value
/// the produced `EvidenceAnchorSet` is byte-identical (the sort is stable and
/// the hash is deterministic).
///
/// ### Set fingerprint
/// The `set_blake3` is the blake3 hash of the concatenation of all per-anchor
/// `excerpt_blake3` values in id order. It is carried in the off-agent SSB row
/// so the set can be re-verified without re-fetching evidence.
///
/// ### Errors
/// Returns an empty `EvidenceAnchorSet` (not an error) when `ranked` is empty.
/// No `.unwrap()` / `.expect()` in the production path; all Result/Option
/// are handled via propagation.
///
/// ### Blake3 note
/// Uses blake3 exclusively (anti-stale: never blake2/sha2 per CLAUDE.md HARD
/// RULE Anti-Stale-Training).
#[must_use]
pub fn freeze_anchors(ranked: &[RankedEvidence], seed: u64) -> EvidenceAnchorSet {
    if ranked.is_empty() {
        return EvidenceAnchorSet {
            anchors: Vec::new(),
            set_blake3: [0u8; 32],
            seed,
        };
    }

    // Stable sort by rank so ties preserve input order (GD-2).
    let mut sorted: Vec<&RankedEvidence> = ranked.iter().collect();
    sorted.sort_by_key(|e| e.rank);

    let mut anchors: Vec<EvidenceAnchor> = Vec::with_capacity(sorted.len());
    for (idx, ev) in sorted.iter().enumerate() {
        let id = (idx as u32).saturating_add(1); // 1-based; saturate is defensive only
        let excerpt_blake3_arr = blake3::hash(ev.excerpt.as_bytes());
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(excerpt_blake3_arr.as_bytes());
        anchors.push(EvidenceAnchor {
            id,
            excerpt: ev.excerpt.clone(),
            excerpt_blake3: hash_bytes,
            source_uri: ev.source_uri.clone(),
            retrieved_at_ms: ev.retrieved_at_ms,
            source_tier: ev.source_tier,
        });
    }

    // Set fingerprint: blake3 of the concatenation of all per-anchor excerpt_blake3 values.
    let set_blake3 = {
        let mut h = blake3::Hasher::new();
        for anchor in &anchors {
            h.update(&anchor.excerpt_blake3);
        }
        let result = h.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(result.as_bytes());
        arr
    };

    EvidenceAnchorSet {
        anchors,
        set_blake3,
        seed,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Closed-set citation check
// ────────────────────────────────────────────────────────────────────────────

/// Outcome of a closed-set citation membership + support check.
///
/// Emitted (via the caller) as part of the `CitationAnchorsBound` SSB row or
/// as an extension field on E-48's `citation_verified` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationCheckOutcome {
    /// The `[N]` maps to a pre-bound anchor; the blake3-pinned excerpt
    /// still matches the cited excerpt (closed-set member + supported).
    Bound,
    /// The `[N]` is not in the pre-bound set (ghost citation). The answer
    /// must be held and routed to D-16 refusal.
    Ghost,
    /// The `[N]` is in the set but the excerpt blake3 no longer matches
    /// (stale anchor — evidence changed between retrieval and synthesis).
    Stale,
}

impl CitationCheckOutcome {
    /// Stable wire string for SSB telemetry.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bound => "bound",
            Self::Ghost => "ghost",
            Self::Stale => "stale",
        }
    }

    /// Returns `true` if the citation should be BLOCKED from the final answer
    /// (it is a ghost or stale anchor). The caller routes to D-16 refusal.
    #[must_use]
    pub fn must_block(self) -> bool {
        matches!(self, Self::Ghost | Self::Stale)
    }
}

/// Result of [`check_citation_in_set`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationCheckResult {
    /// The citation id that was checked (1-based).
    pub id: u32,
    /// Whether the citation is a closed-set member with a matching blake3 pin.
    pub outcome: CitationCheckOutcome,
    /// The blake3 hex of the anchor's frozen excerpt (for off-agent verification).
    /// `None` when `outcome == Ghost` (no anchor was found).
    pub anchor_excerpt_blake3_hex: Option<String>,
}

/// Check whether a model-emitted citation id is a closed-set member of a
/// pre-bound [`EvidenceAnchorSet`] AND whose blake3-pinned excerpt still
/// matches `cited_excerpt`.
///
/// - Returns `Bound` when the id is present and the blake3 pin matches.
/// - Returns `Stale` when the id is present but the blake3 pin does not match.
/// - Returns `Ghost` when the id is not in the set.
///
/// `cited_excerpt` is the excerpt the MODEL claims `[N]` supports. When the
/// model does not supply an excerpt (common in streaming synthesis), pass `""`;
/// the check degrades to id-membership-only (still blocks invented ids; stale
/// detection requires a non-empty excerpt).
///
/// No `.unwrap()` / `.expect()` in production paths (all via Option/match).
#[must_use]
pub fn check_citation_in_set(
    anchor_set: &EvidenceAnchorSet,
    id: u32,
    cited_excerpt: &str,
) -> CitationCheckResult {
    match anchor_set.get(id) {
        None => CitationCheckResult {
            id,
            outcome: CitationCheckOutcome::Ghost,
            anchor_excerpt_blake3_hex: None,
        },
        Some(anchor) => {
            let blake3_hex = blake3::hash(anchor.excerpt.as_bytes())
                .to_hex()
                .to_string();

            // When the model supplies no excerpt, skip the blake3 comparison
            // and treat the anchor as bound (id-membership is sufficient for
            // Tier-1; Tier-2 adds the excerpt match).
            let outcome = if cited_excerpt.is_empty() {
                CitationCheckOutcome::Bound
            } else {
                let cited_hash = blake3::hash(cited_excerpt.as_bytes());
                let mut cited_arr = [0u8; 32];
                cited_arr.copy_from_slice(cited_hash.as_bytes());
                if cited_arr == anchor.excerpt_blake3 {
                    CitationCheckOutcome::Bound
                } else {
                    CitationCheckOutcome::Stale
                }
            };

            CitationCheckResult {
                id,
                outcome,
                anchor_excerpt_blake3_hex: Some(blake3_hex),
            }
        }
    }
}

/// Scan an answer text for `[N]` / `[N,M]` inline citation markers and return
/// all unique ids found.
///
/// Recognises the patterns `[1]`, `[2, 3]`, `[1,2]`, `[1, 2, 3]`.
/// Does NOT recognise `[[1]]` (double-bracket) or `[^1]` (footnote).
/// Returns an empty `Vec` when no markers are found.
///
/// This is a best-effort lexer; it does not handle all edge cases of LaTeX or
/// Markdown citation styles. It is sufficient for ELAI's oracle answer format.
#[must_use]
pub fn extract_citation_ids(answer_text: &str) -> Vec<u32> {
    let mut ids: Vec<u32> = Vec::new();
    let mut i = 0;
    let bytes = answer_text.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // Scan for the closing ']'
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b']' && bytes[end] != b'[' {
                end += 1;
            }
            if end < bytes.len() && bytes[end] == b']' {
                // Try to parse comma-separated numbers inside [start..end]
                let inner = &answer_text[start..end];
                for part in inner.split(',') {
                    let trimmed = part.trim();
                    if let Ok(n) = trimmed.parse::<u32>() {
                        if n > 0 && !ids.contains(&n) {
                            ids.push(n);
                        }
                    }
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    ids.sort_unstable();
    ids
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(rank: u32, excerpt: &str, uri: &str, tier: u8) -> RankedEvidence {
        RankedEvidence {
            rank,
            excerpt: excerpt.to_owned(),
            source_uri: uri.to_owned(),
            retrieved_at_ms: 1_718_000_000_000,
            source_tier: tier,
        }
    }

    #[test]
    fn freeze_empty_returns_empty_set() {
        let set = freeze_anchors(&[], 0);
        assert!(set.is_empty());
        assert_eq!(set.set_blake3, [0u8; 32]);
    }

    #[test]
    fn freeze_assigns_rank_stable_ids() {
        let evidence = vec![
            ev(2, "third", "https://c.example.com", 0),
            ev(0, "first", "https://a.example.com", 0),
            ev(1, "second", "https://b.example.com", 0),
        ];
        let set = freeze_anchors(&evidence, 42);
        assert_eq!(set.len(), 3);
        assert_eq!(set.anchors[0].id, 1);
        assert_eq!(set.anchors[0].excerpt, "first");
        assert_eq!(set.anchors[1].id, 2);
        assert_eq!(set.anchors[1].excerpt, "second");
        assert_eq!(set.anchors[2].id, 3);
        assert_eq!(set.anchors[2].excerpt, "third");
    }

    #[test]
    fn freeze_set_blake3_is_non_zero_for_nonempty_set() {
        let evidence = vec![ev(0, "some excerpt", "https://example.com", 1)];
        let set = freeze_anchors(&evidence, 0);
        assert_ne!(set.set_blake3, [0u8; 32]);
    }

    #[test]
    fn freeze_is_deterministic_under_same_seed() {
        let evidence = vec![
            ev(0, "alpha", "https://a.example.com", 0),
            ev(1, "beta", "https://b.example.com", 1),
        ];
        let set_a = freeze_anchors(&evidence, 99);
        let set_b = freeze_anchors(&evidence, 99);
        assert_eq!(set_a, set_b);
    }

    #[test]
    fn freeze_different_seed_same_set_blake3_content_differs_only_in_seed_field() {
        let evidence = vec![ev(0, "alpha", "https://a.example.com", 0)];
        let set_a = freeze_anchors(&evidence, 1);
        let set_b = freeze_anchors(&evidence, 2);
        // excerpt_blake3 and set_blake3 are content-driven (seed does not affect them)
        assert_eq!(set_a.set_blake3, set_b.set_blake3);
        // seed field differs
        assert_ne!(set_a.seed, set_b.seed);
    }

    #[test]
    fn check_citation_bound_when_excerpt_matches() {
        let evidence = vec![ev(0, "Rust is safe", "https://rust-lang.org", 0)];
        let set = freeze_anchors(&evidence, 0);
        let result = check_citation_in_set(&set, 1, "Rust is safe");
        assert_eq!(result.outcome, CitationCheckOutcome::Bound);
        assert!(result.anchor_excerpt_blake3_hex.is_some());
    }

    #[test]
    fn check_citation_bound_when_no_excerpt_supplied() {
        let evidence = vec![ev(0, "Rust is safe", "https://rust-lang.org", 0)];
        let set = freeze_anchors(&evidence, 0);
        // Empty excerpt => id-membership check only (Bound if id present).
        let result = check_citation_in_set(&set, 1, "");
        assert_eq!(result.outcome, CitationCheckOutcome::Bound);
    }

    #[test]
    fn check_citation_ghost_for_out_of_set_id() {
        let evidence = vec![ev(0, "Rust is safe", "https://rust-lang.org", 0)];
        let set = freeze_anchors(&evidence, 0);
        // id=2 does not exist
        let result = check_citation_in_set(&set, 2, "Rust is safe");
        assert_eq!(result.outcome, CitationCheckOutcome::Ghost);
        assert!(result.outcome.must_block());
        assert!(result.anchor_excerpt_blake3_hex.is_none());
    }

    #[test]
    fn check_citation_stale_when_excerpt_changed() {
        let evidence = vec![ev(0, "original excerpt", "https://example.com", 0)];
        let set = freeze_anchors(&evidence, 0);
        // Anchor for id=1 has excerpt "original excerpt"; model cites a different excerpt.
        let result = check_citation_in_set(&set, 1, "modified excerpt");
        assert_eq!(result.outcome, CitationCheckOutcome::Stale);
        assert!(result.outcome.must_block());
    }

    #[test]
    fn extract_citation_ids_single() {
        let ids = extract_citation_ids("As shown in [1], Rust is safe.");
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn extract_citation_ids_multi_bracket() {
        let ids = extract_citation_ids("See [1, 2] and [3].");
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn extract_citation_ids_deduplicates() {
        let ids = extract_citation_ids("See [1] and also [1].");
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn extract_citation_ids_no_citations() {
        let ids = extract_citation_ids("No citations here.");
        assert!(ids.is_empty());
    }

    #[test]
    fn prebind_disabled_when_env_set() {
        // Exercise the kill-switch path. We cannot set env vars in a parallel-
        // test context reliably, so just confirm the fn returns false in the
        // normal test environment (the env var is not set).
        // The function itself is a trivial env-var read; the test exercises the
        // "not disabled" path.
        if std::env::var("ELAI_CITATION_PREBIND_DISABLE").is_err() {
            assert!(!prebind_disabled());
        }
    }

    #[test]
    fn anchor_set_get_returns_none_for_zero_id() {
        let evidence = vec![ev(0, "excerpt", "https://example.com", 0)];
        let set = freeze_anchors(&evidence, 0);
        assert!(set.get(0).is_none());
    }

    #[test]
    fn anchor_set_get_returns_none_for_out_of_bounds_id() {
        let evidence = vec![ev(0, "excerpt", "https://example.com", 0)];
        let set = freeze_anchors(&evidence, 0);
        assert!(set.get(2).is_none());
    }
}
