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
