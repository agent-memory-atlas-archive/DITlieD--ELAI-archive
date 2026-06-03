//! C-48: MemoryView — query-type-aware filter applied before fact retrieval.
//!
//! Safety-critical query types (destructive_command_intent, merge_decision,
//! auth_decision, credential_fetch) use MemoryView::T0T1Only which transparently
//! excludes T2/T3 rows from all downstream retrieval calls.

use crate::trust::TrustTier;

/// Declared safety-critical query types per C-48 plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyCriticalQueryType {
    /// User intent to execute a destructive command (rm, force-push, db-drop, etc.)
    DestructiveCommandIntent,
    /// Decision to merge a branch into trunk or main.
    MergeDecision,
    /// Authentication or authorization decision (login, permission grant).
    AuthDecision,
    /// Fetching or using a credential or secret.
    CredentialFetch,
}

impl SafetyCriticalQueryType {
    /// Returns true if the given query type is safety-critical.
    pub fn is_safety_critical(query_type: &str) -> bool {
        matches!(
            query_type,
            "destructive_command_intent"
                | "merge_decision"
                | "auth_decision"
                | "credential_fetch"
        )
    }
}

/// Tier ceiling for a memory query. Controls which trust tiers are visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierCeiling {
    /// All tiers visible (T0+T1+T2+T3). Default for non-safety queries.
    AllTiers,
    /// Only T0+T1 visible. Mandatory for safety-critical queries.
    T0T1Only,
}

impl TierCeiling {
    /// Returns true if the given tier is visible under this ceiling.
    pub fn permits(&self, tier: TrustTier) -> bool {
        match self {
            TierCeiling::AllTiers => true,
            TierCeiling::T0T1Only => tier.is_safety_critical_allowed(),
        }
    }
}

impl Default for TierCeiling {
    fn default() -> Self {
        TierCeiling::AllTiers
    }
}

#[cfg(test)]
mod cold_wiring_tests {
    //! Cold-wiring (s-81344): deterministic coverage for the safety-critical
    //! query-type matcher + tier ceiling predicate (previously no inline tests).
    use super::*;

    #[test]
    fn is_safety_critical_matches_declared_types() {
        for q in [
            "destructive_command_intent",
            "merge_decision",
            "auth_decision",
            "credential_fetch",
        ] {
            assert!(SafetyCriticalQueryType::is_safety_critical(q));
        }
        assert!(!SafetyCriticalQueryType::is_safety_critical("fetch_data"));
        assert!(!SafetyCriticalQueryType::is_safety_critical(""));
        // Match is exact/case-sensitive on the snake_case wire tag.
        assert!(!SafetyCriticalQueryType::is_safety_critical("MERGE_DECISION"));
    }

    #[test]
    fn tier_ceiling_all_tiers_permits_everything() {
        let c = TierCeiling::AllTiers;
        for t in [
            TrustTier::T0UserDirect,
            TrustTier::T1CodebaseDerived,
            TrustTier::T2WebSourced,
            TrustTier::T3ModelEmitted,
        ] {
            assert!(c.permits(t));
        }
    }

    #[test]
    fn tier_ceiling_t0t1_only_excludes_t2_t3() {
        let c = TierCeiling::T0T1Only;
        assert!(c.permits(TrustTier::T0UserDirect));
        assert!(c.permits(TrustTier::T1CodebaseDerived));
        assert!(!c.permits(TrustTier::T2WebSourced));
        assert!(!c.permits(TrustTier::T3ModelEmitted));
    }

    #[test]
    fn tier_ceiling_default_is_all_tiers() {
        assert_eq!(TierCeiling::default(), TierCeiling::AllTiers);
    }
}
