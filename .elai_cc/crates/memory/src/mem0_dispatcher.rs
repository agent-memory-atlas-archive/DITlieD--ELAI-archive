//! Mem0-style ADD/UPDATE/DELETE/NOOP dispatcher (arXiv:2504.19413).
//!
//! Folds a list of [`crate::ExtractedFact`] candidates into [`crate::FactStore`]
//! mutations. For each candidate the dispatcher decides one of four
//! operations against the currently-valid row set keyed by
//! `(user_id, scope, fact_type, key)`:
//!
//! - ADD: no row matches the key -> insert.
//! - UPDATE: existing row matches but `value` differs and new confidence
//!   is non-decreasing -> [`FactStore::supersede`] (closes old row, links
//!   via `superseded_by`, inserts new row).
//! - DELETE: candidate carries an explicit `tombstone` marker in its JSON
//!   value -> soft-close (set `valid_to_ms = now_ms`, no replacement row).
//! - NOOP: existing row matches AND value is byte-identical OR new
//!   confidence is strictly lower than existing.
//!
//! This is the thin wrapper around the already-shipped FactStore
//! primitives. It does NOT itself perform any defense — the
//! [`crate::defense`] gate runs upstream and rejects unsafe candidates
//! before reaching the dispatcher.
//!
//! The dispatcher is intentionally pure-Rust + sync — the orchestrator
//! call site owns the tokio runtime and the SSB emission. This module
//! returns one [`DispatchOp`] per candidate so the caller can emit the
//! appropriate SSB beacon (consolidated under
//! `MemoryFactExtracted` / `MemoryFactSuperseded` / `MemoryConflictDetected`
//! per E-5-decisions.md SCHEMA-LOCK).

use crate::error::MemoryError;
use crate::extractor::ExtractedFact;
use crate::fact::{FactType, MemoryFact, Scope};
use crate::store::FactStore;
use crate::trust::{Provenance, TrustTier};

/// Tombstone marker shape on the candidate `value` JSON that signals a
/// DELETE op. The presence of any top-level `"tombstone": true` field
/// triggers soft-close of the matching row.
pub const TOMBSTONE_MARKER: &str = "tombstone";

/// One of the four Mem0 outcomes per candidate. The orchestrator caller
/// emits the appropriate SSB row from this discriminator (consolidated
/// per E-5-decisions.md: ADD/UPDATE map to MemoryFactExtracted /
/// MemoryFactSuperseded; DELETE / NOOP map to MemoryEvolutionRun phases).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOp {
    /// New row inserted. Carries the new `fact_id`.
    Add {
        /// New row's primary key.
        fact_id: String,
    },
    /// Existing row superseded. Carries (old_fact_id, new_fact_id).
    Update {
        /// Row that was closed by `supersede`.
        old_fact_id: String,
        /// Replacement row's primary key.
        new_fact_id: String,
    },
    /// Existing row soft-closed (tombstone). Carries the closed `fact_id`.
    Delete {
        /// Soft-closed row's primary key.
        fact_id: String,
    },
    /// Candidate matched an existing row but produced no mutation.
    /// Carries the matched `fact_id` and a short reason tag for the
    /// SSB row (`"duplicate_value"` | `"lower_confidence"`).
    Noop {
        /// Matched row's primary key (or empty when no match found).
        fact_id: String,
        /// Snake_case reason tag (`"duplicate_value"` | `"lower_confidence"`
        /// | `"tombstone_no_match"`).
        reason: String,
    },
}

impl DispatchOp {
    /// Short snake_case tag for SSB consolidation. Maps:
    ///   `Add` -> `"add"`, `Update` -> `"update"`,
    ///   `Delete` -> `"delete"`, `Noop` -> `"noop"`.
    pub fn op_tag(&self) -> &'static str {
        match self {
            DispatchOp::Add { .. } => "add",
            DispatchOp::Update { .. } => "update",
            DispatchOp::Delete { .. } => "delete",
            DispatchOp::Noop { .. } => "noop",
        }
    }
}

/// Configuration for the dispatcher. The orchestrator caller fills these
/// in per-turn (user_id, project_id, scope, provenance, now_ms).
#[derive(Debug, Clone)]
pub struct DispatchContext {
    /// User identifier for the new rows.
    pub user_id: String,
    /// Per-seat namespace key (J-59). Stamped on every fact built from this
    /// context so rows are namespace-isolated at retrieval. Defaults to
    /// `"local"` for single-seat installs.
    pub seat_id: String,
    /// Project identifier when `scope == Project`. `None` for `Global`.
    pub project_id: Option<String>,
    /// Per-row scope (`Project` for newly extracted rows by default).
    pub scope: Scope,
    /// Provenance of the extractor source (C-48 SafeHarbor classification).
    pub provenance: Provenance,
    /// Trust tier of the provenance source.
    pub trust_tier: TrustTier,
    /// Initial trust score for newly inserted rows (`[0,1]`).
    pub initial_trust_score: f32,
    /// Wall-clock time in ms (from ClockProvider::now_ms()). Used for
    /// `ingested_at_ms`, `valid_from_ms`, `last_observed_at_ms`,
    /// and as the soft-close timestamp on DELETE.
    pub now_ms: u128,
}

impl DispatchContext {
    /// Validate context invariants before any dispatch op. Returns
    /// `MemoryError::Invalid` on out-of-range values so the caller can
    /// surface a typed error.
    pub fn validate(&self) -> Result<(), MemoryError> {
        if self.user_id.is_empty() {
            return Err(MemoryError::Invalid("user_id is empty".into()));
        }
        if self.seat_id.is_empty() {
            return Err(MemoryError::Invalid("seat_id is empty".into()));
        }
        if !(0.0..=1.0).contains(&self.initial_trust_score) {
            return Err(MemoryError::Invalid(format!(
                "initial_trust_score out of [0,1]: {}",
                self.initial_trust_score
            )));
        }
        if matches!(self.scope, Scope::Project) && self.project_id.is_none() {
            return Err(MemoryError::Invalid(
                "scope=project requires project_id".into(),
            ));
        }
        Ok(())
    }
}

/// Dispatch a single [`ExtractedFact`] candidate against `store`.
///
/// Decision tree:
///   1. If the candidate's value carries `{"tombstone": true}` -> DELETE
///      (when matching row exists) or NOOP (no row to close).
///   2. If no existing row matches `(user_id, scope, fact_type, key)`
///      -> ADD.
///   3. If matching row exists and serialized values differ AND new
///      confidence is non-decreasing -> UPDATE via supersede.
///   4. Otherwise (byte-identical value or lower confidence) -> NOOP.
pub fn dispatch_one(
    store: &mut FactStore,
    candidate: &ExtractedFact,
    ctx: &DispatchContext,
) -> Result<DispatchOp, MemoryError> {
    ctx.validate()?;
    if candidate.evidence.is_empty() {
        return Err(MemoryError::Invalid(
            "candidate evidence_pointers must be non-empty".into(),
        ));
    }

    let existing = find_matching_current(
        store,
        &ctx.user_id,
        ctx.scope,
        candidate.fact_type,
        &candidate.key,
        ctx.project_id.as_deref(),
    )?;

    let is_tombstone = is_tombstone_value(&candidate.value);
    match (existing, is_tombstone) {
        (Some(row), true) => {
            // Soft-close: set valid_to_ms to now_ms. We re-use update_trust
            // for the read column, then issue a direct UPDATE for valid_to.
            close_row(store, &row.fact_id, ctx.now_ms)?;
            Ok(DispatchOp::Delete {
                fact_id: row.fact_id,
            })
        }
        (None, true) => Ok(DispatchOp::Noop {
            fact_id: String::new(),
            reason: "tombstone_no_match".into(),
        }),
        (None, false) => {
            // ADD path.
            let fact = build_fact_from_candidate(candidate, ctx);
            store.insert(&fact)?;
            Ok(DispatchOp::Add {
                fact_id: fact.fact_id,
            })
        }
        (Some(row), false) => {
            // UPDATE vs NOOP.
            let same_value = row.value == candidate.value;
            if same_value {
                return Ok(DispatchOp::Noop {
                    fact_id: row.fact_id,
                    reason: "duplicate_value".into(),
                });
            }
            if candidate.confidence < row.confidence {
                return Ok(DispatchOp::Noop {
                    fact_id: row.fact_id,
                    reason: "lower_confidence".into(),
                });
            }
            // Supersede path.
            let new_fact = build_fact_from_candidate(candidate, ctx);
            let new_fact_id = new_fact.fact_id.clone();
            store.supersede(&row.fact_id, &new_fact, ctx.now_ms)?;
            Ok(DispatchOp::Update {
                old_fact_id: row.fact_id,
                new_fact_id,
            })
        }
    }
}

/// Dispatch a batch of candidates in source order. Each candidate is
/// processed independently — a per-candidate failure does NOT abort the
/// batch. The return shape pairs `(candidate_index, Result<DispatchOp>)`
/// so the caller can surface per-row outcomes to the SSB.
pub fn dispatch_batch(
    store: &mut FactStore,
    candidates: &[ExtractedFact],
    ctx: &DispatchContext,
) -> Vec<(usize, Result<DispatchOp, MemoryError>)> {
    candidates
        .iter()
        .enumerate()
        .map(|(idx, c)| (idx, dispatch_one(store, c, ctx)))
        .collect()
}

fn find_matching_current(
    store: &FactStore,
    user_id: &str,
    scope: Scope,
    fact_type: FactType,
    key: &str,
    project_id: Option<&str>,
) -> Result<Option<MemoryFact>, MemoryError> {
    // Use list_current with the scope+fact_type filter, then linearly
    // scan for matching key + project_id. Candidate set is bounded —
    // list_current's LIMIT defaults are sufficient for the per-key
    // duplicate scan.
    let rows = store.list_current(user_id, Some(scope), Some(fact_type), 256)?;
    for r in rows {
        if r.key != key {
            continue;
        }
        // For project-scoped rows, project_id must match (or both None
        // which is a misuse — Scope::Project requires project_id per
        // MemoryFact::validate).
        if matches!(scope, Scope::Project) && r.project_id.as_deref() != project_id {
            continue;
        }
        return Ok(Some(r));
    }
    Ok(None)
}

fn build_fact_from_candidate(candidate: &ExtractedFact, ctx: &DispatchContext) -> MemoryFact {
    MemoryFact {
        fact_id: MemoryFact::new_id(),
        user_id: ctx.user_id.clone(),
        seat_id: ctx.seat_id.clone(),
        project_id: ctx.project_id.clone(),
        scope: ctx.scope,
        fact_type: candidate.fact_type,
        key: candidate.key.clone(),
        value: strip_tombstone_marker(&candidate.value),
        confidence: candidate.confidence,
        trust_score: ctx.initial_trust_score,
        ingested_at_ms: ctx.now_ms,
        valid_from_ms: ctx.now_ms,
        valid_to_ms: None,
        decay_rate_per_day: crate::decay::default_decay_rate(candidate.fact_type),
        last_observed_at_ms: ctx.now_ms,
        evidence_pointers: candidate.evidence.clone(),
        superseded_by: None,
        trust_tier: ctx.trust_tier,
        provenance: ctx.provenance.clone(),
        last_validated_at_ms: None,
    }
}

fn is_tombstone_value(value: &serde_json::Value) -> bool {
    value
        .as_object()
        .and_then(|m| m.get(TOMBSTONE_MARKER))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn strip_tombstone_marker(value: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = value.as_object() {
        let filtered: serde_json::Map<String, serde_json::Value> = obj
            .iter()
            .filter(|(k, _)| k.as_str() != TOMBSTONE_MARKER)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::Value::Object(filtered)
    } else {
        value.clone()
    }
}

fn close_row(store: &mut FactStore, fact_id: &str, now_ms: u128) -> Result<(), MemoryError> {
    // FactStore does not expose a public soft-close primitive (the only
    // existing close path is supersede, which requires a replacement row).
    // We open a transaction via the underlying connection accessor we expose
    // here for the DELETE op. Adds a thin method on FactStore.
    store.soft_close(fact_id, now_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust::{Provenance, TrustTier};

    fn ctx(now: u128) -> DispatchContext {
        DispatchContext {
            user_id: "default".into(),
            seat_id: "local".into(),
            project_id: Some("proj-a".into()),
            scope: Scope::Project,
            provenance: Provenance::user_direct("test"),
            trust_tier: TrustTier::T0UserDirect,
            initial_trust_score: 0.8,
            now_ms: now,
        }
    }

    fn cand(key: &str, val: serde_json::Value, conf: f32) -> ExtractedFact {
        ExtractedFact {
            fact_type: FactType::Preference,
            key: key.into(),
            value: val,
            confidence: conf,
            evidence: vec!["sess-1".into(), "msg-1".into()],
        }
    }

    #[test]
    fn add_when_no_existing_row() {
        let mut store = FactStore::open_in_memory().expect("open");
        let c = cand("k1", serde_json::json!({"v": 1}), 0.9);
        let op = dispatch_one(&mut store, &c, &ctx(1_700_000_000_000))
            .expect("dispatch");
        assert!(matches!(op, DispatchOp::Add { .. }));
    }

    #[test]
    fn noop_on_duplicate_value() {
        let mut store = FactStore::open_in_memory().expect("open");
        let c = cand("k1", serde_json::json!({"v": 1}), 0.9);
        dispatch_one(&mut store, &c, &ctx(1_700_000_000_000)).expect("first");
        let op = dispatch_one(&mut store, &c, &ctx(1_700_000_000_001))
            .expect("second");
        assert!(matches!(op, DispatchOp::Noop { .. }));
    }

    #[test]
    fn update_on_value_change_with_higher_conf() {
        let mut store = FactStore::open_in_memory().expect("open");
        let c1 = cand("k1", serde_json::json!({"v": 1}), 0.7);
        let c2 = cand("k1", serde_json::json!({"v": 2}), 0.9);
        dispatch_one(&mut store, &c1, &ctx(1_700_000_000_000)).expect("first");
        let op = dispatch_one(&mut store, &c2, &ctx(1_700_000_000_001))
            .expect("second");
        assert!(matches!(op, DispatchOp::Update { .. }));
    }

    #[test]
    fn delete_on_tombstone_marker() {
        let mut store = FactStore::open_in_memory().expect("open");
        let c1 = cand("k1", serde_json::json!({"v": 1}), 0.9);
        dispatch_one(&mut store, &c1, &ctx(1_700_000_000_000)).expect("first");
        let tomb = cand("k1", serde_json::json!({"tombstone": true}), 0.9);
        let op = dispatch_one(&mut store, &tomb, &ctx(1_700_000_000_001))
            .expect("tombstone");
        match op {
            DispatchOp::Delete { fact_id } => {
                // Row should no longer be in list_current.
                let rows = store
                    .list_current("default", Some(Scope::Project), Some(FactType::Preference), 10)
                    .expect("list");
                assert!(rows.iter().all(|r| r.fact_id != fact_id));
            }
            other => panic!("expected Delete, got {other:?}"),
        }
    }
}
