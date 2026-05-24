//! Heuristic contradiction adjudicator per E-5 plan Task 11
//! (BeliefShift stability-adaptability tradeoff).
//!
//! Scans [`crate::FactStore`] for currently-valid rows that share
//! `(user_id, fact_type, key)` and project_id-compatible scope but
//! carry different `value` payloads. Classifies each conflict pair as:
//!
//! - **CHECK**: same key + scope, different values — surface to the
//!   user via SSB `MemoryConflictDetected` (consolidated under
//!   existing variant per E-5-decisions.md). Per the BeliefShift defense
//!   the adjudicator does NOT auto-overwrite — both rows survive with
//!   overlapping `valid_from`/`valid_to` until the user resolves.
//! - **SUPERSEDE_BY_TIER**: same key + scope, different values, AND the
//!   newer row has a strictly higher trust_tier than the older one
//!   (T0 > T1 > T2 > T3). The older row is soft-closed; the user is
//!   still notified via the conflict event so they can audit.
//!
//! This is a HEURISTIC v1 — LLM-backed semantic conflict detection
//! (cosine similarity over value embeddings < -0.4) is deferred to
//! E-5-followup. The heuristic compares serialized JSON for exact
//! equality of subject/alternative fields (the extractor's standard
//! shape) and treats them as conflicting if at least one shared
//! semantic key (`subject` / `alternative` / `value`) differs.

use crate::error::MemoryError;
use crate::fact::{FactType, MemoryFact};
use crate::store::FactStore;
use crate::trust::TrustTier;

/// Outcome of evaluating one (older, newer) candidate pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictAction {
    /// Both rows survive; surface to the user. Carries the conflict
    /// classification tag (`"duplicate"` | `"contradiction"`).
    Surface {
        /// Older row's id.
        older_fact_id: String,
        /// Newer row's id.
        newer_fact_id: String,
        /// `"duplicate"` (same value) | `"contradiction"` (differing
        /// values).
        conflict_kind: String,
    },
    /// Newer row's tier strictly higher than older; older was soft-closed.
    /// Caller still emits `MemoryConflictDetected` so the user can audit.
    SupersededByTier {
        /// Older row's id (now soft-closed).
        older_fact_id: String,
        /// Newer row's id.
        newer_fact_id: String,
    },
    /// No action needed (e.g. row already superseded by id, scopes
    /// incompatible, or only one row matches the key).
    NoOp,
}

/// Scan all currently-valid rows for a user + fact_type and return one
/// action per conflict pair detected.
///
/// `auto_supersede_by_tier == true` enables the tier-driven soft-close
/// of older rows. When `false`, every detected conflict returns
/// `ConflictAction::Surface` regardless of tier — the user adjudicates
/// manually.
pub fn audit_conflicts(
    store: &mut FactStore,
    user_id: &str,
    fact_type: FactType,
    auto_supersede_by_tier: bool,
    now_ms: u128,
) -> Result<Vec<ConflictAction>, MemoryError> {
    let rows = store.list_current(user_id, None, Some(fact_type), 512)?;
    let mut actions: Vec<ConflictAction> = Vec::new();

    // Group by (project_id, scope.as_str(), key).
    let mut groups: std::collections::BTreeMap<(Option<String>, &'static str, String), Vec<MemoryFact>> =
        std::collections::BTreeMap::new();
    for r in rows {
        let key = (r.project_id.clone(), r.scope.as_str(), r.key.clone());
        groups.entry(key).or_default().push(r);
    }

    let mut to_close: Vec<(String, u128)> = Vec::new();

    for (_group_key, mut bucket) in groups {
        if bucket.len() < 2 {
            continue;
        }
        // Sort by valid_from_ms ascending so older rows come first.
        bucket.sort_by_key(|r| r.valid_from_ms);

        // Pair-wise (older, newer) conflicts. We pair the latest row
        // against each older row — typical case is 2 rows total, but
        // the loop handles N-way.
        let last_idx = bucket.len() - 1;
        let newer = bucket[last_idx].clone();
        for older in bucket.into_iter().take(last_idx) {
            let same_value = older.value == newer.value;
            let kind = if same_value {
                "duplicate".to_string()
            } else {
                "contradiction".to_string()
            };

            if auto_supersede_by_tier
                && !same_value
                && newer.trust_tier_priority() > older.trust_tier_priority()
            {
                to_close.push((older.fact_id.clone(), now_ms));
                actions.push(ConflictAction::SupersededByTier {
                    older_fact_id: older.fact_id,
                    newer_fact_id: newer.fact_id.clone(),
                });
            } else {
                actions.push(ConflictAction::Surface {
                    older_fact_id: older.fact_id,
                    newer_fact_id: newer.fact_id.clone(),
                    conflict_kind: kind,
                });
            }
        }
    }

    // Apply tier-driven closures after the scan to avoid mutating during
    // iteration. Errors here are propagated — soft_close fails only if
    // the row was already closed by another path, which would indicate a
    // race the caller needs to know about.
    for (fact_id, ts) in to_close {
        match store.soft_close(&fact_id, ts) {
            Ok(()) => {}
            Err(MemoryError::NotFound(_)) => {
                // Row was already closed concurrently — treat as no-op.
                tracing::info!(
                    target = "memory.contradiction",
                    fact_id = %fact_id,
                    "row already closed when tier-supersession ran"
                );
            }
            Err(e) => return Err(e),
        }
    }

    Ok(actions)
}

// Helper trait extension on MemoryFact: numeric priority for tier
// comparisons. T0 highest -> 3, T3 lowest -> 0.
trait TrustTierPriority {
    fn trust_tier_priority(&self) -> u8;
}

impl TrustTierPriority for MemoryFact {
    fn trust_tier_priority(&self) -> u8 {
        match self.trust_tier {
            TrustTier::T0UserDirect => 3,
            TrustTier::T1CodebaseDerived => 2,
            TrustTier::T2WebSourced => 1,
            TrustTier::T3ModelEmitted => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::{MemoryFact, Scope};
    use crate::trust::Provenance;

    fn fact(
        id: &str,
        key: &str,
        value: serde_json::Value,
        tier: TrustTier,
        t: u128,
    ) -> MemoryFact {
        MemoryFact {
            fact_id: id.into(),
            user_id: "default".into(),
            project_id: Some("p1".into()),
            scope: Scope::Project,
            fact_type: FactType::Preference,
            key: key.into(),
            value,
            confidence: 0.9,
            trust_score: 0.9,
            ingested_at_ms: t,
            valid_from_ms: t,
            valid_to_ms: None,
            decay_rate_per_day: 0.001,
            last_observed_at_ms: t,
            evidence_pointers: vec!["sess".into(), "msg".into()],
            superseded_by: None,
            trust_tier: tier,
            provenance: Provenance::user_direct("test"),
        }
    }

    #[test]
    fn surfaces_contradiction_without_auto_supersede() {
        let mut store = FactStore::open_in_memory().expect("open");
        let t0 = 1_700_000_000_000u128;
        store
            .insert(&fact(
                "old",
                "prefers_lang",
                serde_json::json!({"subject": "rust"}),
                TrustTier::T1CodebaseDerived,
                t0,
            ))
            .expect("ins old");
        store
            .insert(&fact(
                "new",
                "prefers_lang",
                serde_json::json!({"subject": "python"}),
                TrustTier::T1CodebaseDerived,
                t0 + 1000,
            ))
            .expect("ins new");

        let acts =
            audit_conflicts(&mut store, "default", FactType::Preference, false, t0 + 2000)
                .expect("audit");
        assert_eq!(acts.len(), 1);
        match &acts[0] {
            ConflictAction::Surface { conflict_kind, .. } => {
                assert_eq!(conflict_kind, "contradiction")
            }
            other => panic!("expected Surface, got {other:?}"),
        }
    }

    #[test]
    fn supersedes_older_when_newer_has_higher_tier() {
        let mut store = FactStore::open_in_memory().expect("open");
        let t0 = 1_700_000_000_000u128;
        store
            .insert(&fact(
                "old",
                "prefers_lang",
                serde_json::json!({"subject": "rust"}),
                TrustTier::T3ModelEmitted,
                t0,
            ))
            .expect("ins old");
        store
            .insert(&fact(
                "new",
                "prefers_lang",
                serde_json::json!({"subject": "python"}),
                TrustTier::T0UserDirect,
                t0 + 1000,
            ))
            .expect("ins new");

        let acts =
            audit_conflicts(&mut store, "default", FactType::Preference, true, t0 + 2000)
                .expect("audit");
        assert_eq!(acts.len(), 1);
        assert!(matches!(acts[0], ConflictAction::SupersededByTier { .. }));
        // Old row should be soft-closed.
        let remaining = store
            .list_current("default", Some(Scope::Project), Some(FactType::Preference), 10)
            .expect("list");
        assert!(remaining.iter().all(|r| r.fact_id != "old"));
    }

    #[test]
    fn duplicate_values_classified_as_duplicate() {
        let mut store = FactStore::open_in_memory().expect("open");
        let t0 = 1_700_000_000_000u128;
        let v = serde_json::json!({"subject": "rust"});
        store
            .insert(&fact("a", "k", v.clone(), TrustTier::T1CodebaseDerived, t0))
            .expect("a");
        store
            .insert(&fact(
                "b",
                "k",
                v.clone(),
                TrustTier::T1CodebaseDerived,
                t0 + 10,
            ))
            .expect("b");

        let acts = audit_conflicts(&mut store, "default", FactType::Preference, false, t0 + 20)
            .expect("audit");
        assert_eq!(acts.len(), 1);
        if let ConflictAction::Surface { conflict_kind, .. } = &acts[0] {
            assert_eq!(conflict_kind, "duplicate");
        } else {
            panic!("expected Surface duplicate, got {:?}", acts[0]);
        }
    }
}
