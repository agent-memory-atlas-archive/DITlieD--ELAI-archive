//! E-44 — Idempotent `ToolDispatcher` wrapper.
//!
//! Wraps `Arc<dyn ToolDispatcher>` and dedupes replayed tool calls by a
//! deterministic call-shape hash inside a session. On a cache hit the
//! wrapper returns the previously observed result without re-issuing
//! the underlying dispatch — defeating semantic-rollback attacks where
//! a retried tool call produces a subtly different request even at
//! temperature=0 (GPU float-point order across runs).
//!
//! Composition: the wrapper composes AFTER the existing
//! `ToolReliabilityGate` (allow / spill / retry already settled) and
//! BEFORE the actual dispatch call so a cached result can return
//! without entering the trait method body. See E-44-decisions.md
//! DECISION-2.
//!
//! Determinism rules:
//! * `BTreeMap` (not `HashMap`) for the cache — production determinism.
//! * `tokio::sync::Mutex` (not `std::sync::Mutex`) because the trait
//!   methods are `async fn` and the mutex is held across `.await`.
//! * `blake3::hash` over a stable canonical encoding of the request —
//!   never `blake2` or `sha2`.
//! * No `unwrap()` / `expect()` in any production path.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTierResult,
};
use thiserror::Error;
use tokio::sync::Mutex;

/// Result variants stored in the dedup cache. Mirrors the six
/// `ToolDispatcher` method outputs. Only successful results are cached
/// — errors re-execute (per ACRFence: a retry of a failed irreversible
/// op is the documented escape hatch for transient infra failures).
#[derive(Debug, Clone)]
pub enum CachedToolResult {
    Edit(EditOutcome),
    Validate(Vec<ValidationTierResult>),
    ClassifyFailure(Option<ClassifiedFailure>),
    GitCommit(CommitOutcome),
    FileReadOutline(FileView),
    IncrementalCompile(IncrementalOutcome),
}

/// Errors specific to the idempotency layer. Disjoint from
/// `ToolDispatchError` because cached-result-shape mismatches are a
/// caching defect, not a dispatch defect.
#[derive(Debug, Error)]
pub enum IdempotencyError {
    /// A cache hit returned a result variant that does not match the
    /// method being dispatched (defensive — should not happen in
    /// practice because `call_shape_hash` includes the method tag).
    #[error("idempotency cache variant mismatch: expected {expected}, got {actual}")]
    VariantMismatch {
        expected: &'static str,
        actual: &'static str,
    },
}

/// Compute a deterministic 64-bit hash over the canonical tag + bytes
/// of a request shape. The tag enforces method separation so an
/// `EditRequest` and a `GitCommitRequest` that happen to serialise to
/// identical bytes still produce distinct hashes.
pub fn call_shape_hash(method_tag: &str, body: &[u8]) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(method_tag.as_bytes());
    hasher.update(&[0u8]); // separator
    hasher.update(body);
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Wrapper that dedupes `ToolDispatcher` calls inside a session.
///
/// The cache is held under `tokio::sync::Mutex<BTreeMap<u64,
/// CachedToolResult>>`. Cache lookups + insertions are amortised
/// O(log N) where N is the per-session call count — well within the
/// budget for ELAI's typical session lifetimes.
pub struct IdempotentDispatcher {
    inner: Arc<dyn ToolDispatcher>,
    cache: Mutex<BTreeMap<u64, CachedToolResult>>,
    session_id: String,
}

impl IdempotentDispatcher {
    /// Construct a fresh wrapper around `inner`. `session_id` is mixed
    /// into the call-shape tag to scope dedup to a single session
    /// (cross-session replays must re-execute).
    pub fn new(inner: Arc<dyn ToolDispatcher>, session_id: String) -> Self {
        Self {
            inner,
            cache: Mutex::new(BTreeMap::new()),
            session_id,
        }
    }

    /// Number of distinct call shapes currently cached. Used by the
    /// `elai durability status` subverb to surface cache size to the
    /// operator.
    pub async fn cache_size(&self) -> usize {
        self.cache.lock().await.len()
    }

    /// Snapshot of the current cache keys. Used by contract tests +
    /// CLI status for non-destructive inspection.
    pub async fn cache_keys(&self) -> Vec<u64> {
        self.cache.lock().await.keys().copied().collect()
    }

    fn method_tag(&self, raw: &'static str) -> String {
        format!("{}:{raw}", self.session_id)
    }
}

impl ToolDispatcher for IdempotentDispatcher {
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>>
    {
        let tag = self.method_tag("dispatch_edit");
        Box::pin(async move {
            let body = serde_json::to_vec(&req).map_err(|e| {
                ToolDispatchError::NotImplemented(format!(
                    "idempotent_dispatcher: serialise edit request failed: {e}"
                ))
            })?;
            let key = call_shape_hash(&tag, &body);
            if let Some(hit) = self.cache.lock().await.get(&key).cloned() {
                if let CachedToolResult::Edit(out) = hit {
                    return Ok(out);
                }
                // Defensive: variant mismatch indicates cache corruption
                // — re-issue the underlying call rather than panic.
            }
            let out = self.inner.dispatch_edit(req).await?;
            self.cache
                .lock()
                .await
                .insert(key, CachedToolResult::Edit(out.clone()));
            Ok(out)
        })
    }

    fn dispatch_validate(
        &self,
        req: ValidateRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        let tag = self.method_tag("dispatch_validate");
        Box::pin(async move {
            let body = serde_json::to_vec(&req).map_err(|e| {
                ToolDispatchError::NotImplemented(format!(
                    "idempotent_dispatcher: serialise validate request failed: {e}"
                ))
            })?;
            let key = call_shape_hash(&tag, &body);
            if let Some(CachedToolResult::Validate(out)) =
                self.cache.lock().await.get(&key).cloned()
            {
                return Ok(out);
            }
            let out = self.inner.dispatch_validate(req).await?;
            self.cache
                .lock()
                .await
                .insert(key, CachedToolResult::Validate(out.clone()));
            Ok(out)
        })
    }

    fn dispatch_classify_failure(
        &self,
        input: FailureInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        let tag = self.method_tag("dispatch_classify_failure");
        Box::pin(async move {
            let body = match serde_json::to_vec(&input) {
                Ok(b) => b,
                // Serialisation failure on classify is non-fatal:
                // re-issue the underlying call without caching.
                Err(_) => return self.inner.dispatch_classify_failure(input).await,
            };
            let key = call_shape_hash(&tag, &body);
            if let Some(CachedToolResult::ClassifyFailure(out)) =
                self.cache.lock().await.get(&key).cloned()
            {
                return out;
            }
            let out = self.inner.dispatch_classify_failure(input).await;
            self.cache
                .lock()
                .await
                .insert(key, CachedToolResult::ClassifyFailure(out.clone()));
            out
        })
    }

    fn dispatch_git_commit(
        &self,
        req: GitCommitRequest,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>,
    > {
        let tag = self.method_tag("dispatch_git_commit");
        Box::pin(async move {
            let body = serde_json::to_vec(&req).map_err(|e| {
                ToolDispatchError::NotImplemented(format!(
                    "idempotent_dispatcher: serialise git_commit request failed: {e}"
                ))
            })?;
            let key = call_shape_hash(&tag, &body);
            if let Some(CachedToolResult::GitCommit(out)) =
                self.cache.lock().await.get(&key).cloned()
            {
                return Ok(out);
            }
            let out = self.inner.dispatch_git_commit(req).await?;
            self.cache
                .lock()
                .await
                .insert(key, CachedToolResult::GitCommit(out.clone()));
            Ok(out)
        })
    }

    fn dispatch_file_read_outline(
        &self,
        path: PathBuf,
        line_threshold: usize,
    ) -> Pin<
        Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>,
    > {
        let tag = self.method_tag("dispatch_file_read_outline");
        Box::pin(async move {
            let body = serde_json::to_vec(&(path.clone(), line_threshold)).map_err(|e| {
                ToolDispatchError::NotImplemented(format!(
                    "idempotent_dispatcher: serialise file_read request failed: {e}"
                ))
            })?;
            let key = call_shape_hash(&tag, &body);
            if let Some(CachedToolResult::FileReadOutline(out)) =
                self.cache.lock().await.get(&key).cloned()
            {
                return Ok(out);
            }
            let out = self
                .inner
                .dispatch_file_read_outline(path, line_threshold)
                .await?;
            self.cache
                .lock()
                .await
                .insert(key, CachedToolResult::FileReadOutline(out.clone()));
            Ok(out)
        })
    }

    fn dispatch_incremental_compile(
        &self,
        req: IncrementalRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        let tag = self.method_tag("dispatch_incremental_compile");
        Box::pin(async move {
            let body = serde_json::to_vec(&req).map_err(|e| {
                ToolDispatchError::NotImplemented(format!(
                    "idempotent_dispatcher: serialise incremental request failed: {e}"
                ))
            })?;
            let key = call_shape_hash(&tag, &body);
            if let Some(CachedToolResult::IncrementalCompile(out)) =
                self.cache.lock().await.get(&key).cloned()
            {
                return Ok(out);
            }
            let out = self.inner.dispatch_incremental_compile(req).await?;
            self.cache
                .lock()
                .await
                .insert(key, CachedToolResult::IncrementalCompile(out.clone()));
            Ok(out)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_shape_hash_separates_methods() {
        let body = b"same-body";
        let h1 = call_shape_hash("dispatch_edit", body);
        let h2 = call_shape_hash("dispatch_validate", body);
        assert_ne!(h1, h2, "method tag must separate identical bodies");
    }

    #[test]
    fn call_shape_hash_is_deterministic() {
        let h1 = call_shape_hash("dispatch_edit", b"x");
        let h2 = call_shape_hash("dispatch_edit", b"x");
        assert_eq!(h1, h2, "same tag+body must produce same hash");
    }

    #[test]
    fn call_shape_hash_differs_on_body() {
        let h1 = call_shape_hash("dispatch_edit", b"a");
        let h2 = call_shape_hash("dispatch_edit", b"b");
        assert_ne!(h1, h2, "different body must produce different hash");
    }
}
