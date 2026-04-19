use std::sync::{Arc, RwLock};

use crate::predicate::{
    AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams,
};

/// AST evaluator that always returns empty matches (pass-through).
pub struct PassAstEvaluator;

impl AstEvaluator for PassAstEvaluator {
    fn evaluate_query(
        &self,
        _language: &str,
        _source: &str,
        _query: &str,
    ) -> Result<Vec<AstMatch>, AstError> {
        Ok(vec![])
    }

    fn validate_query(&self, _language: &str, _query: &str) -> Result<(), AstError> {
        Ok(())
    }
}

/// DKG querier that always returns empty paths (no reachability).
pub struct EmptyDkgQuerier;

impl DkgQuerier for EmptyDkgQuerier {
    fn query_reachability(
        &self,
        _params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(vec![])
    }

    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Batch-ful-a (FUL-A.1) — hot-swappable DkgQuerier wrapper.
// ---------------------------------------------------------------------------

/// A [`DkgQuerier`] whose inner implementation can be replaced at runtime.
///
/// The wrapper holds a shared `Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>>`
/// so the dashboard can construct the rule engine with `EmptyDkgQuerier`
/// at boot time and later upgrade it to `DkgQuerierAdapter` without
/// reconstructing the engine. Reads go through `RwLock::read` (multiple
/// concurrent readers); swaps go through `RwLock::write` (exclusive).
///
/// The `DkgQuerier` trait is already object-safe (`query_reachability` +
/// `validate_params` both take `&self`), so the wrapper satisfies the same
/// dyn bounds as any concrete impl.
///
/// Shared handle: the same `Arc<RwLock<...>>` used by the wrapper is threaded
/// into `TaskRunnerConfig::dkg_querier_swap` so the task_runner can perform
/// the swap after `IndexingService::run_full_index` succeeds. See
/// `crates/orchestrator/src/task_runner.rs` for the emission site.
pub struct HotSwappableDkgQuerier {
    inner: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>>,
}

impl HotSwappableDkgQuerier {
    /// Construct a wrapper around a shared `Arc<RwLock<Box<dyn DkgQuerier>>>`.
    /// The caller keeps the `Arc` so the swap trigger (task_runner post-index
    /// hook) can write to the same lock.
    pub fn new(inner: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>>) -> Self {
        Self { inner }
    }

    /// Replace the inner querier. Acquires the write lock; returns an error
    /// only if the lock is poisoned (another thread panicked while holding
    /// it). This is an explicit [`HotSwapError::Poisoned`] rather than a
    /// panic so the caller (task_runner) can surface the condition as a
    /// Finding instead of taking the runner down.
    pub fn swap(
        &self,
        new_inner: Box<dyn DkgQuerier + Send + Sync>,
    ) -> Result<(), HotSwapError> {
        let mut guard = self.inner.write().map_err(|_| HotSwapError::Poisoned)?;
        *guard = new_inner;
        Ok(())
    }
}

/// Error raised by [`HotSwappableDkgQuerier::swap`] when the underlying
/// `RwLock` is poisoned. Distinct from [`DkgError`] because a poisoned lock
/// is a runtime invariant failure, not a query-time condition.
#[derive(Debug, thiserror::Error)]
pub enum HotSwapError {
    #[error("DkgQuerier swap RwLock poisoned — a reader panicked while holding it")]
    Poisoned,
}

impl DkgQuerier for HotSwappableDkgQuerier {
    fn query_reachability(
        &self,
        params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        // On a poisoned lock, downgrade to an empty result rather than
        // bubbling a rule-engine-visible error — the rule engine surfaces
        // DkgError as a Warn verdict, which would pollute the activity feed
        // with every evaluation. The swap site logs Poisoned explicitly.
        let guard = self
            .inner
            .read()
            .map_err(|_| DkgError::QueryFailed("dkg querier lock poisoned".to_string()))?;
        guard.query_reachability(params)
    }

    fn validate_params(&self, params: &ReachabilityParams) -> Result<(), DkgError> {
        let guard = self
            .inner
            .read()
            .map_err(|_| DkgError::QueryFailed("dkg querier lock poisoned".to_string()))?;
        guard.validate_params(params)
    }
}
