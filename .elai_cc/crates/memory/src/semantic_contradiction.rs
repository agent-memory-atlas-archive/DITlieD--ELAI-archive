//! F-12 — semantic contradiction pre-filter (Tier-1, fully OFFLINE).
//!
//! A trigger-only SECOND pass on the contradiction adjudicator. ELAI's
//! [`crate::audit_conflicts`] only flags a contradiction when two
//! currently-valid rows share a BYTE-IDENTICAL `(project_id, scope, key)`
//! and their `value` JSON differs. It cannot detect two facts that are
//! semantically opposed across NON-matching keys (e.g.
//! `prefers_db = postgres` vs `migrated_off = postgres in favour of
//! sqlite`). This module closes that recall gap WITHOUT importing
//! Graphiti's cloud-LLM judge (which would break ELAI_SEED determinism per
//! arXiv:2506.09501).
//!
//! Tier split:
//! - **Tier-1 (this file, ships NOW, no model)**: the [`NliJudge`] trait,
//!   the LOCAL deterministic [`DeterministicNliJudge`] (a lexical
//!   opposition heuristic — fully deterministic argmax, no `ort`), the
//!   cross-key candidate collector, the adjudicator, the SSB-reuse
//!   plumbing shape, and the blake3 pair fingerprint. The default build
//!   compiles WITHOUT `ort`. The deterministic judge is the DEFAULT-ON
//!   enforcement (no default-OFF graveyard).
//! - **Tier-2 (`nli_onnx.rs`, feature `nli-onnx`, dark-of-the-model)**:
//!   `OnnxNliJudge` over the already-vetted `ort`/ONNX runtime, plugged in
//!   via the SAME [`NliJudge`] trait. Staged separately because no NLI
//!   ONNX model is on disk today.
//!
//! Determinism framing (HONEST): the deterministic judge here is bit-stable
//! by construction (pure integer/string lexical math, no floating-point
//! reduction order). The Tier-2 cross-encoder is "practically stable
//! argmax, NOT bit-identical" — the same GPU FP batch-variance class
//! (arXiv:2506.09501) applies to its logits.
//!
//! Resolution semantics (non-destructive, mirrors Graphiti
//! `invalid_at = valid_at`): on a `Contradiction` at or above
//! [`SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD`], the OLDER row is
//! soft-closed with `valid_to_ms = newer.valid_from_ms` (NOT `now_ms`);
//! the newer row survives. Everything else (Neutral / Entailment /
//! below-threshold / judge error) returns `SurfaceToUser` — the SAFE
//! default, identical to today's [`crate::audit_conflicts`] surface
//! behaviour. There is NO branch that auto-closes below threshold.

use crate::error::MemoryError;
use crate::fact::{FactType, MemoryFact, Scope};
use crate::store::FactStore;

/// Conservative-by-default auto-close threshold (ELAI-own design value).
///
/// An `InvalidatedOlder` soft-close fires ONLY when the judge returns
/// `Contradiction` with `confidence >=` this value. Set high to bound the
/// false-positive auto-close hazard (killer-risk 2); everything below is
/// `SurfaceToUser`. Documented + revisited only after measured pair-level
/// precision justifies loosening it.
pub const SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD: f32 = 0.85;

/// Hard cap on currently-valid rows pulled per `(user_id, fact_type)`
/// before pairing — mirrors [`crate::audit_conflicts`]'s `512` bound so
/// the localize-first funnel never fans out unboundedly.
const CANDIDATE_ROW_HARD_CAP: usize = 512;

/// Minimum length for a token to count as a shared "content" token (drops
/// stop-word-length noise like "a" / "of" / "to" from the overlap test).
const MIN_CONTENT_TOKEN_LEN: usize = 3;

/// Jaccard overlap at or above which two near-identical value strings with
/// NO opposition marker are classified `Entailment` (restatement).
const ENTAILMENT_JACCARD_FLOOR: f32 = 0.80;

/// 3-class NLI label (the standard SNLI/MNLI output space).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NliLabel {
    /// The hypothesis follows from / restates the premise.
    Entailment,
    /// No entailment and no contradiction (unrelated / underspecified).
    Neutral,
    /// The hypothesis is opposed to the premise.
    Contradiction,
}

/// A judge verdict: the argmax label + the argmax confidence (the wide
/// 3-class logit-gap margin used for the threshold gate).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NliVerdict {
    /// The argmax 3-class label.
    pub label: NliLabel,
    /// Confidence in `[0.0, 1.0]` — the softmax-max for the ONNX judge, a
    /// bounded heuristic score for the deterministic judge.
    pub confidence: f32,
}

/// The pluggable judge boundary — the ONLY seam the live ONNX model plugs
/// into. The candidate-pairing + adjudication logic depends on this trait,
/// NOT on `ort`, so the harness + contract tests run with a local
/// deterministic judge and the module ships before any model is staged.
///
/// `classify` is a synchronous, LOCAL function returning a plain `Result`:
/// no async, no network client, no cloud lane. A cloud LLM judge is
/// REJECTED by construction (it would break ELAI_SEED determinism per
/// arXiv:2506.09501).
pub trait NliJudge {
    /// Classify the (premise, hypothesis) pair into a 3-class verdict.
    fn classify(&self, premise: &str, hypothesis: &str) -> Result<NliVerdict, MemoryError>;
}

/// The LOCAL deterministic Tier-1 judge — a lexical opposition heuristic.
///
/// This is NOT a neural NLI model (that is the Tier-2 `OnnxNliJudge`). It
/// is a conservative, explainable, fully-deterministic argmax over word
/// sets: it flags `Contradiction` only when an explicit opposition /
/// switch marker co-occurs with a shared content token (a shared subject
/// being negated/replaced), classifies near-identical strings as
/// `Entailment`, and defaults to `Neutral` otherwise. Because it is pure
/// integer/string math it is bit-stable across replays (no FP reduction
/// order), and because it defaults to sub-threshold confidence on anything
/// ambiguous it errs toward `SurfaceToUser` — the safe default.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicNliJudge;

/// Opposition / switch markers. Presence of any of these on EITHER side,
/// combined with a shared content token, is the contradiction signal.
const OPPOSITION_MARKERS: &[&str] = &[
    "not",
    "no",
    "never",
    "instead",
    "migrated",
    "switched",
    "switch",
    "dropped",
    "drop",
    "replaced",
    "replace",
    "stopped",
    "stop",
    "abandoned",
    "abandon",
    "former",
    "formerly",
    "deprecated",
    "removed",
    "remove",
    "off",
    "away",
    "ditched",
    "moved",
];

/// Multi-word opposition phrases checked against the raw lowercased text
/// (single-token markers miss these).
const OPPOSITION_PHRASES: &[&str] = &[
    "in favour of",
    "in favor of",
    "no longer",
    "rather than",
    "as opposed to",
    "switched from",
    "moved off",
    "moved away",
];

impl DeterministicNliJudge {
    /// Construct the judge. Zero-sized; infallible.
    pub fn new() -> Self {
        DeterministicNliJudge
    }
}

/// Lowercase content-token set for a string (tokens >= [`MIN_CONTENT_TOKEN_LEN`]).
fn content_tokens(s: &str) -> std::collections::BTreeSet<String> {
    s.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= MIN_CONTENT_TOKEN_LEN)
        .map(|t| t.to_owned())
        .collect()
}

/// All lowercase tokens (any length) for marker detection.
fn all_tokens(s: &str) -> std::collections::BTreeSet<String> {
    s.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_owned())
        .collect()
}

/// Count opposition markers + phrases present across the combined text.
fn opposition_signal(premise: &str, hypothesis: &str) -> u32 {
    let toks: std::collections::BTreeSet<String> = all_tokens(premise)
        .into_iter()
        .chain(all_tokens(hypothesis))
        .collect();
    let mut count: u32 = 0;
    for m in OPPOSITION_MARKERS {
        if toks.contains(*m) {
            count += 1;
        }
    }
    let combined = format!(
        "{} {}",
        premise.to_ascii_lowercase(),
        hypothesis.to_ascii_lowercase()
    );
    for p in OPPOSITION_PHRASES {
        if combined.contains(p) {
            count += 1;
        }
    }
    count
}

/// Jaccard overlap of two content-token sets.
fn jaccard(a: &std::collections::BTreeSet<String>, b: &std::collections::BTreeSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        inter as f32 / union as f32
    }
}

impl NliJudge for DeterministicNliJudge {
    fn classify(&self, premise: &str, hypothesis: &str) -> Result<NliVerdict, MemoryError> {
        let pa = content_tokens(premise);
        let pb = content_tokens(hypothesis);
        let shared = pa.intersection(&pb).count();
        let overlap = jaccard(&pa, &pb);
        let markers = opposition_signal(premise, hypothesis);

        // Contradiction: an explicit opposition marker over a shared
        // content token (a shared subject being negated / switched). The
        // confidence scales with both signals and is bounded to [0, 0.99]
        // so a CLEAR opposition over a shared subject clears the
        // conservative threshold while a weak/ambiguous one does not.
        if markers >= 1 && shared >= 1 {
            // Base 0.80 for one marker + one shared token; +0.06 per extra
            // marker, +0.05 per extra shared token; capped at 0.99.
            let extra_markers = markers.saturating_sub(1) as f32;
            let extra_shared = (shared.saturating_sub(1)) as f32;
            let confidence = (0.80 + 0.06 * extra_markers + 0.05 * extra_shared).min(0.99);
            return Ok(NliVerdict {
                label: NliLabel::Contradiction,
                confidence,
            });
        }

        // Entailment: near-identical strings with NO opposition marker
        // (one restates the other). High-overlap, no-negation.
        if markers == 0 && overlap >= ENTAILMENT_JACCARD_FLOOR {
            return Ok(NliVerdict {
                label: NliLabel::Entailment,
                confidence: overlap.min(0.99),
            });
        }

        // Everything else is Neutral — the funnel surfaces nothing to
        // auto-close. Confidence is the inverse of the (absent) signal.
        Ok(NliVerdict {
            label: NliLabel::Neutral,
            confidence: (1.0 - overlap).clamp(0.5, 0.99),
        })
    }
}

/// A candidate (older, newer) pair to adjudicate. The pair shares a
/// `(project_id, scope)` but has DIFFERENT keys — exactly the cross-key
/// case [`crate::audit_conflicts`] cannot reach.
#[derive(Debug, Clone)]
pub struct ContradictionCandidate {
    /// The older row (lower `valid_from_ms`).
    pub older: MemoryFact,
    /// The newer row (higher `valid_from_ms`).
    pub newer: MemoryFact,
}

/// The action taken (or surfaced) for one adjudicated pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticAction {
    /// The older row was soft-closed (`valid_to_ms = newer.valid_from_ms`).
    InvalidatedOlder,
    /// Surfaced to the user; both rows survive (the SAFE default).
    SurfaceToUser,
}

/// The outcome of adjudicating one [`ContradictionCandidate`].
#[derive(Debug, Clone)]
pub struct SemanticConflictOutcome {
    /// Older row id.
    pub older_fact_id: String,
    /// Newer row id.
    pub newer_fact_id: String,
    /// The judge's argmax label.
    pub label: NliLabel,
    /// The judge's argmax confidence.
    pub confidence: f32,
    /// What happened (auto-closed vs surfaced).
    pub action: SemanticAction,
}

/// Localize-FIRST candidate collector (Graphiti's cost-control discipline).
///
/// Re-pairs currently-valid rows across DIFFERENT keys within the SAME
/// `(project_id, scope)`, so the judge only ever sees a bounded set of
/// pairs — never the full cross-product. Uses [`FactStore::list_current`]
/// (the SAME query [`crate::audit_conflicts`] uses) so no new query is
/// introduced. `scope_hint` narrows the SQL filter when known.
///
/// A pair is only emitted when the two rows differ in BOTH key AND value
/// (same-key pairs are [`crate::audit_conflicts`]'s job; identical-value
/// pairs are duplicates, not contradictions).
pub fn collect_cross_key_candidates(
    store: &FactStore,
    user_id: &str,
    fact_type: FactType,
    scope_hint: Option<Scope>,
) -> Result<Vec<ContradictionCandidate>, MemoryError> {
    let rows = store.list_current(user_id, scope_hint, Some(fact_type), CANDIDATE_ROW_HARD_CAP)?;

    // Group by (project_id, scope) so cross-key pairs stay within one
    // localized bucket (never cross project/scope boundaries — that would
    // reintroduce the O(n^2) blow-up the localize-first discipline exists
    // to prevent).
    let mut groups: std::collections::BTreeMap<(Option<String>, &'static str), Vec<MemoryFact>> =
        std::collections::BTreeMap::new();
    for r in rows {
        let bucket_key = (r.project_id.clone(), r.scope.as_str());
        groups.entry(bucket_key).or_default().push(r);
    }

    let mut candidates: Vec<ContradictionCandidate> = Vec::new();
    for (_bucket_key, mut bucket) in groups {
        if bucket.len() < 2 {
            continue;
        }
        // Oldest first.
        bucket.sort_by_key(|r| r.valid_from_ms);
        // Pair the latest row against each strictly-older DIFFERENT-key row
        // with a different value (the cross-key residue audit_conflicts
        // never reaches). Typical case is 2 rows total.
        let last_idx = bucket.len() - 1;
        let newer = bucket[last_idx].clone();
        for older in bucket.into_iter().take(last_idx) {
            if older.key == newer.key {
                // Same key — that is audit_conflicts' lane, not ours.
                continue;
            }
            if older.value == newer.value {
                // Identical value across different keys is not a
                // contradiction signal.
                continue;
            }
            candidates.push(ContradictionCandidate {
                older,
                newer: newer.clone(),
            });
        }
    }
    Ok(candidates)
}

/// Render a fact's `value` JSON into a stable premise/hypothesis string
/// for the judge. Strings pass through; structured values serialize to a
/// canonical JSON string (serde_json's object serialization is key-sorted
/// for `Value::Object` via BTreeMap when the `preserve_order` feature is
/// off, which it is here, giving a deterministic render).
fn render_value(fact: &MemoryFact) -> String {
    match &fact.value {
        serde_json::Value::String(s) => s.clone(),
        other => {
            // key + serialized value gives the judge the semantic subject
            // even when the payload is structured.
            let body = serde_json::to_string(other).unwrap_or_else(|_| other.to_string());
            format!("{} {}", fact.key, body)
        }
    }
}

/// Adjudicate each candidate pair with the supplied judge.
///
/// For each pair: render premise/hypothesis, call `judge.classify`, and:
/// - on `Contradiction` with `confidence >= threshold`: soft-close the
///   OLDER row with `valid_to_ms = newer.valid_from_ms` (the Graphiti
///   event-time semantic) and record `InvalidatedOlder`;
/// - on ANYTHING else (Neutral / Entailment / below-threshold / a judge
///   `Err`): record `SurfaceToUser` and leave both rows valid — the SAFE
///   default. A judge failure NEVER auto-closes.
///
/// A `soft_close` returning `NotFound` (the row was concurrently closed)
/// is treated as a no-op surfacing — never an error that aborts the sweep
/// (mirrors [`crate::audit_conflicts`]'s race handling).
pub fn adjudicate_semantic(
    store: &mut FactStore,
    judge: &dyn NliJudge,
    candidates: &[ContradictionCandidate],
    threshold: f32,
    _now_ms: u128,
) -> Result<Vec<SemanticConflictOutcome>, MemoryError> {
    let mut outcomes: Vec<SemanticConflictOutcome> = Vec::with_capacity(candidates.len());

    for cand in candidates {
        let premise = render_value(&cand.older);
        let hypothesis = render_value(&cand.newer);

        // Fail-SAFE: a judge error degrades to SurfaceToUser, never an
        // auto-close, and never aborts the sweep.
        let verdict = match judge.classify(&premise, &hypothesis) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    target = "memory.semantic_contradiction",
                    older = %cand.older.fact_id,
                    newer = %cand.newer.fact_id,
                    error = %e,
                    "nli judge errored; degrading to SurfaceToUser (fail-safe)"
                );
                outcomes.push(SemanticConflictOutcome {
                    older_fact_id: cand.older.fact_id.clone(),
                    newer_fact_id: cand.newer.fact_id.clone(),
                    label: NliLabel::Neutral,
                    confidence: 0.0,
                    action: SemanticAction::SurfaceToUser,
                });
                continue;
            }
        };

        let auto_close =
            verdict.label == NliLabel::Contradiction && verdict.confidence >= threshold;

        let action = if auto_close {
            // Graphiti invalid_at = valid_at: the loser's event-time
            // invalidation is the INVALIDATING fact's valid_from_ms.
            match store.soft_close(&cand.older.fact_id, cand.newer.valid_from_ms) {
                Ok(()) => SemanticAction::InvalidatedOlder,
                Err(MemoryError::NotFound(_)) => {
                    // Row already closed concurrently — surface instead of
                    // erroring (no-op, mirrors audit_conflicts).
                    tracing::info!(
                        target = "memory.semantic_contradiction",
                        fact_id = %cand.older.fact_id,
                        "older row already closed when semantic invalidation ran"
                    );
                    SemanticAction::SurfaceToUser
                }
                Err(e) => return Err(e),
            }
        } else {
            SemanticAction::SurfaceToUser
        };

        outcomes.push(SemanticConflictOutcome {
            older_fact_id: cand.older.fact_id.clone(),
            newer_fact_id: cand.newer.fact_id.clone(),
            label: verdict.label,
            confidence: verdict.confidence,
            action,
        });
    }

    Ok(outcomes)
}

/// blake3 (NOT blake2/sha2) pair fingerprint tying an SSB beacon to the
/// inspectable pair. 16-char hex prefix, mirroring the F-5 broadcast_id
/// format (`format!("{:.16}", ...to_hex())`).
pub fn pair_fingerprint(older: &MemoryFact, newer: &MemoryFact) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(older.fact_id.as_bytes());
    hasher.update(b"\x00");
    hasher.update(newer.fact_id.as_bytes());
    hasher.update(b"\x00");
    hasher.update(&newer.valid_from_ms.to_le_bytes());
    format!("{:.16}", hasher.finalize().to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_judge_flags_opposition_over_shared_subject() {
        let j = DeterministicNliJudge::new();
        let v = j
            .classify("postgres", "migrated off postgres in favour of sqlite")
            .expect("classify");
        assert_eq!(v.label, NliLabel::Contradiction);
        assert!(
            v.confidence >= SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD,
            "clear opposition over a shared subject must clear the threshold, got {}",
            v.confidence
        );
    }

    #[test]
    fn deterministic_judge_neutral_on_unrelated() {
        let j = DeterministicNliJudge::new();
        let v = j.classify("postgres", "neovim editor").expect("classify");
        assert_eq!(v.label, NliLabel::Neutral);
    }

    #[test]
    fn deterministic_judge_entailment_on_restatement() {
        let j = DeterministicNliJudge::new();
        let v = j
            .classify("prefers the rust language", "prefers the rust language")
            .expect("classify");
        assert_eq!(v.label, NliLabel::Entailment);
    }

    #[test]
    fn deterministic_judge_is_bit_stable() {
        let j = DeterministicNliJudge::new();
        let a = j.classify("postgres", "dropped postgres").expect("a");
        let b = j.classify("postgres", "dropped postgres").expect("b");
        assert_eq!(a.label, b.label);
        assert_eq!(a.confidence, b.confidence);
    }
}
