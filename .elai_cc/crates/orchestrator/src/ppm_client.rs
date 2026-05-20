//! D-2: PPM (Process Preference Model) client — orchestrator-side wrapper
//! around the `SidecarPpmHook` trait.
//!
//! Responsibilities:
//! 1. Cache repeated scoring requests by `blake3` content hash so the
//!    consensus verifier path does not pay sidecar latency for identical
//!    candidate-step chains.
//! 2. Apply a gating threshold (`min_score_mean`) on top of the raw
//!    `PpmScoreResponse`, producing a `PpmScore` value that downstream
//!    gates (D-3 pre-promotion chain gate consumer in this file's
//!    `apply_ppm_gate`) can branch on without re-implementing the floor
//!    logic.
//! 3. Provide an SSB producer (`emit_ppm_score`) that emits the new
//!    `PpmScored` row (and `PpmGatingFailed` when the gate trips). The
//!    function takes the hook + a candidate + an SSB writer, runs scoring,
//!    persists the row, and returns the `PpmScore` for caller use.
//!
//! # Wire contract
//!
//! See `sidecar_ppm_hook.rs` doc-comment for the Python sidecar JSON
//! request / response format. This file does NOT talk HTTP directly; the
//! `SidecarPpmHook` trait is the boundary. Production wiring will
//! instantiate `HttpSidecarPpmHook` (follow-up patch — kept out of this
//! session per orchestration ownership constraints) and inject it into
//! `PpmClient::new(hook)`.

use std::sync::Arc;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::sidecar_ppm_hook::{
    PpmHookError, PpmMode, PpmScoreRequest, PpmScoreResponse, SidecarPpmHook,
};
use crate::ssb::{PpmGatingFailedPayload, PpmScoredPayload, SsbPayload};

// ---------------------------------------------------------------------------
// PpmScore: intrinsic-signal value type
// ---------------------------------------------------------------------------

/// The decision-grade signal produced by one PPM scoring call.
///
/// Intrinsic signals (mean / variance / num_steps) are forwarded as-is
/// from the sidecar response; the gating verdict (`gate_passed`) is
/// computed locally from `score_mean >= min_score_threshold`.
///
/// `score_mean` and `score_var` are `f32` to match the sidecar wire
/// format. Callers that need higher precision should aggregate
/// multiple `PpmScore`s rather than promote to `f64` here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PpmScore {
    pub candidate_id: String,
    pub verifier_id: String,
    pub mode: PpmMode,
    pub score_mean: f32,
    pub score_var: f32,
    pub num_steps: u32,
    pub step_scores: Vec<f32>,
    /// `score_mean >= min_score_threshold` (closed form, no LLM callout).
    pub gate_passed: bool,
    /// The threshold the gate was evaluated against. Echoed so SSB
    /// consumers can reconstruct the decision.
    pub min_score_threshold: f32,
    /// `latency_ms` from the underlying `PpmScoreResponse`.
    pub latency_ms: u64,
    /// True if this score was served from the `PpmClient` cache and the
    /// sidecar was not contacted.
    pub cache_hit: bool,
}

impl PpmScore {
    /// Default minimum mean score required to pass the gate.
    /// Per the D-2 plan §config (`min_score = 0.7`); also the floor
    /// recommended by rStar-Math §3.2 (T1) "promote step-chains with PPM
    /// >= 0.7 across all steps". Callers can override via `apply_ppm_gate`.
    pub const DEFAULT_MIN_SCORE_THRESHOLD: f32 = 0.7;
}

// ---------------------------------------------------------------------------
// PpmClient errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum PpmClientError {
    /// The underlying `SidecarPpmHook` returned an error.
    #[error("ppm hook error: {0}")]
    Hook(#[from] PpmHookError),

    /// `min_score_threshold` was outside `[0.0, 1.0]`.
    #[error("invalid threshold: {0} (must be in [0.0, 1.0])")]
    InvalidThreshold(f32),

    /// SSB write failure.
    #[error("ssb write error: {0}")]
    Ssb(String),
}

// ---------------------------------------------------------------------------
// PpmClient
// ---------------------------------------------------------------------------

/// Cached, threshold-aware wrapper around a `SidecarPpmHook` implementation.
///
/// `H` is the concrete hook type (parametric — Rust native async fn in trait
/// is NOT dyn-compatible per the anti-stale rule, so we stay generic and let
/// monomorphisation produce one specialised client per hook impl). The
/// `Arc<H>` wrapper keeps cloning cheap; the cache is `Arc<Mutex<_>>`-shared
/// across clones.
///
/// Cache entries live for the lifetime of the client instance (no TTL)
/// because the cache key is a content hash; the same candidate-step
/// chain always produces the same score.
pub struct PpmClient<H: SidecarPpmHook> {
    hook: Arc<H>,
    cache: Arc<Mutex<FxHashMap<String, PpmScoreResponse>>>,
    min_score_threshold: f32,
}

impl<H: SidecarPpmHook> std::fmt::Debug for PpmClient<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PpmClient")
            .field("min_score_threshold", &self.min_score_threshold)
            .finish_non_exhaustive()
    }
}

impl<H: SidecarPpmHook> Clone for PpmClient<H> {
    fn clone(&self) -> Self {
        Self {
            hook: Arc::clone(&self.hook),
            cache: Arc::clone(&self.cache),
            min_score_threshold: self.min_score_threshold,
        }
    }
}

impl<H: SidecarPpmHook> PpmClient<H> {
    /// Construct a client with the default threshold
    /// (`PpmScore::DEFAULT_MIN_SCORE_THRESHOLD`).
    pub fn new(hook: Arc<H>) -> Self {
        Self {
            hook,
            cache: Arc::new(Mutex::new(FxHashMap::default())),
            min_score_threshold: PpmScore::DEFAULT_MIN_SCORE_THRESHOLD,
        }
    }

    /// Construct a client with an explicit threshold. Returns
    /// `PpmClientError::InvalidThreshold` if `t` is outside `[0.0, 1.0]`.
    pub fn with_threshold(hook: Arc<H>, t: f32) -> Result<Self, PpmClientError> {
        if !(0.0..=1.0).contains(&t) {
            return Err(PpmClientError::InvalidThreshold(t));
        }
        Ok(Self {
            hook,
            cache: Arc::new(Mutex::new(FxHashMap::default())),
            min_score_threshold: t,
        })
    }

    /// Current gating threshold.
    pub fn threshold(&self) -> f32 {
        self.min_score_threshold
    }

    /// Number of cached score entries.
    pub async fn cache_len(&self) -> usize {
        self.cache.lock().await.len()
    }

    /// Score one candidate-step chain. Caches the response by blake3
    /// cache key (see `PpmScoreRequest::cache_key_hex`). Repeated calls
    /// with identical `(verifier_id, mode, problem_statement,
    /// reasoning_steps)` return the cached response with `cache_hit=true`.
    pub async fn score(
        &self,
        req: PpmScoreRequest,
    ) -> Result<PpmScore, PpmClientError> {
        let cache_key = req.cache_key_hex();
        let (response, cache_hit) = {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                debug!(
                    candidate_id = %req.candidate_id,
                    cache_key = %&cache_key[..16],
                    "ppm cache hit",
                );
                (cached.clone(), true)
            } else {
                drop(cache);
                let resp = self.hook.score(req.clone()).await?;
                let mut cache = self.cache.lock().await;
                cache.insert(cache_key, resp.clone());
                (resp, false)
            }
        };

        // Defensive shape check — even cached responses must match the live
        // request shape (the cache key includes step count so this is a
        // belt-and-braces assertion).
        if response.step_scores.len() != req.reasoning_steps.len() {
            return Err(PpmClientError::Hook(PpmHookError::ShapeMismatch {
                request_steps: req.reasoning_steps.len(),
                response_scores: response.step_scores.len(),
            }));
        }

        let gate_passed = response.score_mean >= self.min_score_threshold;
        Ok(PpmScore {
            candidate_id: response.candidate_id,
            verifier_id: response.verifier_id,
            mode: response.mode,
            score_mean: response.score_mean,
            score_var: response.score_var,
            num_steps: response.num_steps,
            step_scores: response.step_scores,
            gate_passed,
            min_score_threshold: self.min_score_threshold,
            latency_ms: response.latency_ms,
            cache_hit,
        })
    }
}

// ---------------------------------------------------------------------------
// Consumer: apply_ppm_gate
// ---------------------------------------------------------------------------

/// Consumer-side gate decision built on top of `PpmScore`. Returns
/// `true` if the candidate clears the configured threshold; otherwise
/// returns `false`. Layered next to the existing pre-promotion chain gate
/// (D-3 will fold this into the consensus rule); for D-2, exposing the
/// function publicly is sufficient observable wiring.
///
/// `PpmScore.gate_passed` is the single source of truth; this function
/// exists so call sites can read intent (`apply_ppm_gate(&score)`)
/// instead of `.gate_passed`.
#[must_use]
pub fn apply_ppm_gate(score: &PpmScore) -> bool {
    score.gate_passed
}

// ---------------------------------------------------------------------------
// SSB producer: emit_ppm_score
// ---------------------------------------------------------------------------

/// Production producer wiring. Run a PPM scoring call against `hook`,
/// emit one SSB row (`PpmScored` on success, `PpmGatingFailed` if the
/// gate trips), and return the `PpmScore` for the caller's use.
///
/// The SSB emitter is abstracted through the `PpmSsbEmitter` trait so
/// tests can capture rows without spinning up a real SSB writer. The
/// production wiring point is `agent_pool.rs::run_ppm_verifier_check`
/// (this same file's call site requirement).
pub async fn emit_ppm_score<H, E>(
    client: &PpmClient<H>,
    req: PpmScoreRequest,
    emitter: &E,
) -> Result<PpmScore, PpmClientError>
where
    H: SidecarPpmHook,
    E: PpmSsbEmitter + ?Sized,
{
    let score = client.score(req).await?;
    let candidate_id = score.candidate_id.clone();
    let verifier_id = score.verifier_id.clone();
    let mode_label = score.mode.as_str().to_string();
    if score.gate_passed {
        let payload = SsbPayload::PpmScored(PpmScoredPayload {
            candidate_id,
            verifier_id,
            mode: mode_label,
            score_mean: score.score_mean as f64,
            score_var: score.score_var as f64,
            num_steps: score.num_steps,
            latency_ms: score.latency_ms,
            cache_hit: score.cache_hit,
        });
        emitter
            .emit(payload)
            .await
            .map_err(PpmClientError::Ssb)?;
    } else {
        warn!(
            candidate_id = %candidate_id,
            score_mean = score.score_mean,
            threshold = score.min_score_threshold,
            "ppm gate failed",
        );
        let payload = SsbPayload::PpmGatingFailed(PpmGatingFailedPayload {
            candidate_id,
            verifier_id,
            mode: mode_label,
            score_mean: score.score_mean as f64,
            threshold: score.min_score_threshold as f64,
            num_steps: score.num_steps,
        });
        emitter
            .emit(payload)
            .await
            .map_err(PpmClientError::Ssb)?;
    }
    Ok(score)
}

/// Minimal trait the SSB producer uses to write a payload. Production
/// callers wrap `Ssb::write_payload` (the orchestrator's typed SSB
/// writer); test callers use `VecPpmSsbEmitter` to capture rows in-process.
pub trait PpmSsbEmitter: Send + Sync {
    fn emit(
        &self,
        payload: SsbPayload,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

/// Test-only emitter that pushes payloads into an in-memory vector.
/// Lives next to `PpmClient` so unit tests + the integration test can
/// share it without a separate test helper module.
#[derive(Debug, Default)]
pub struct VecPpmSsbEmitter {
    rows: Mutex<Vec<SsbPayload>>,
}

impl VecPpmSsbEmitter {
    /// Construct an empty emitter.
    pub fn new() -> Self {
        Self {
            rows: Mutex::new(Vec::new()),
        }
    }

    /// Snapshot of the rows captured so far.
    pub async fn rows(&self) -> Vec<SsbPayload> {
        self.rows.lock().await.clone()
    }

    /// Number of rows captured.
    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl PpmSsbEmitter for VecPpmSsbEmitter {
    async fn emit(&self, payload: SsbPayload) -> Result<(), String> {
        let mut guard = self.rows.lock().await;
        guard.push(payload);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidecar_ppm_hook::MockSidecarPpmHook;

    fn make_req(steps: Vec<&str>) -> PpmScoreRequest {
        PpmScoreRequest {
            candidate_id: "cand-1".into(),
            verifier_id: "ppm-test".into(),
            problem_statement: "test problem".into(),
            reasoning_steps: steps.into_iter().map(String::from).collect(),
            mode: PpmMode::Math,
        }
    }

    #[tokio::test]
    async fn client_score_round_trip_with_default_threshold() {
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.85));
        let client = PpmClient::new(hook);
        let score = client.score(make_req(vec!["a", "b"])).await.expect("ok");
        assert_eq!(score.num_steps, 2);
        assert!(score.gate_passed);
        assert!((score.score_mean - 0.85).abs() < 1e-6);
        assert_eq!(score.min_score_threshold, PpmScore::DEFAULT_MIN_SCORE_THRESHOLD);
        assert!(!score.cache_hit);
    }

    #[tokio::test]
    async fn client_with_threshold_validates_range() {
        let hook = Arc::new(MockSidecarPpmHook::default());
        match PpmClient::with_threshold(hook.clone(), -0.1) {
            Err(PpmClientError::InvalidThreshold(t)) => assert!((t + 0.1).abs() < 1e-6),
            other => panic!("expected InvalidThreshold, got {other:?}"),
        }
        match PpmClient::with_threshold(hook, 1.1) {
            Err(PpmClientError::InvalidThreshold(t)) => assert!((t - 1.1).abs() < 1e-6),
            other => panic!("expected InvalidThreshold, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn client_gate_fails_when_below_threshold() {
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.3));
        let client = PpmClient::new(hook);
        let score = client.score(make_req(vec!["a"])).await.expect("ok");
        assert!(!score.gate_passed);
        assert!(!apply_ppm_gate(&score));
    }

    #[tokio::test]
    async fn client_caches_identical_request() {
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        let client = PpmClient::new(hook);
        let s1 = client.score(make_req(vec!["x"])).await.expect("ok");
        assert!(!s1.cache_hit);
        let s2 = client.score(make_req(vec!["x"])).await.expect("ok");
        assert!(s2.cache_hit);
        assert_eq!(client.cache_len().await, 1);
    }

    #[tokio::test]
    async fn client_does_not_cache_different_steps() {
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        let client = PpmClient::new(hook);
        let _ = client.score(make_req(vec!["x"])).await.expect("ok");
        let _ = client.score(make_req(vec!["y"])).await.expect("ok");
        assert_eq!(client.cache_len().await, 2);
    }

    #[tokio::test]
    async fn client_propagates_unavailable() {
        let hook = Arc::new(MockSidecarPpmHook::always_unavailable());
        let client = PpmClient::new(hook);
        match client.score(make_req(vec!["a"])).await {
            Err(PpmClientError::Hook(PpmHookError::Unavailable)) => {}
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn emit_ppm_score_writes_ppm_scored_when_gate_passes() {
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.95));
        let client = PpmClient::new(hook);
        let emitter = VecPpmSsbEmitter::new();
        let score = emit_ppm_score(&client, make_req(vec!["a", "b"]), &emitter)
            .await
            .expect("ok");
        assert!(score.gate_passed);
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 1);
        match &rows[0] {
            SsbPayload::PpmScored(p) => {
                assert_eq!(p.candidate_id, "cand-1");
                assert_eq!(p.verifier_id, "ppm-test");
                assert_eq!(p.num_steps, 2);
                assert!((p.score_mean - 0.95).abs() < 1e-6);
                assert!(!p.cache_hit);
            }
            other => panic!("expected PpmScored, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn emit_ppm_score_writes_gating_failed_when_below_threshold() {
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.4));
        let client = PpmClient::new(hook);
        let emitter = VecPpmSsbEmitter::new();
        let score = emit_ppm_score(&client, make_req(vec!["a"]), &emitter)
            .await
            .expect("ok");
        assert!(!score.gate_passed);
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 1);
        match &rows[0] {
            SsbPayload::PpmGatingFailed(p) => {
                assert_eq!(p.candidate_id, "cand-1");
                assert!((p.threshold - 0.7).abs() < 1e-6);
                assert!((p.score_mean - 0.4).abs() < 1e-6);
            }
            other => panic!("expected PpmGatingFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn emit_ppm_score_caches_on_repeat_call() {
        let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
        let client = PpmClient::new(hook);
        let emitter = VecPpmSsbEmitter::new();
        let s1 = emit_ppm_score(&client, make_req(vec!["a"]), &emitter)
            .await
            .expect("ok");
        assert!(!s1.cache_hit);
        let s2 = emit_ppm_score(&client, make_req(vec!["a"]), &emitter)
            .await
            .expect("ok");
        assert!(s2.cache_hit);
        assert_eq!(emitter.len().await, 2, "both calls emit a row");
    }
}
