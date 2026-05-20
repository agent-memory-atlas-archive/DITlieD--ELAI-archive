//! D-2: PPM (Process Preference Model) sidecar hook — Rust-side trait +
//! wire-format interface contract for the Python sidecar implementation.
//!
//! # Why a separate file
//!
//! Per the D-2 plan + orchestration constraints (s-62376), the Python
//! sidecar workspace (`.elai_cc/python/elai_inference/`) is owned by a
//! parallel session. This file documents the WIRE CONTRACT the Python
//! `server.py` PPM worker MUST implement; the actual `server.py` patch
//! lands in a follow-up session.
//!
//! # Wire format (HTTP POST `/v1/ppm/score`)
//!
//! Request body (JSON, mirrors the Anthropic-style envelope used by the
//! policy SLM sidecar — same FastAPI scaffolding, separate endpoint):
//! ```json
//! {
//!   "problem_statement": "Compute the area of a rectangle with width 4 and height 5",
//!   "reasoning_steps": [
//!     "Step 1: area = width * height",
//!     "Step 2: area = 4 * 5",
//!     "Step 3: area = 20"
//!   ],
//!   "mode": "math",
//!   "candidate_id": "cand_5f3e1d2c",
//!   "verifier_id": "ppm_qwen25_math_prm_7b"
//! }
//! ```
//!
//! Response body (JSON):
//! ```json
//! {
//!   "candidate_id": "cand_5f3e1d2c",
//!   "verifier_id": "ppm_qwen25_math_prm_7b",
//!   "step_scores": [0.91, 0.94, 0.97],
//!   "score_mean": 0.94,
//!   "score_var": 0.0006,
//!   "num_steps": 3,
//!   "mode": "math",
//!   "latency_ms": 412
//! }
//! ```
//!
//! ## Contract clauses (HARD)
//!
//! 1. `step_scores[i]` MUST be in `[0.0, 1.0]`. Out-of-range -> `PpmHookError::InvalidScore`.
//! 2. `num_steps == step_scores.len() == reasoning_steps.len()` (server-side echo
//!    of the request shape). Mismatch -> `PpmHookError::ShapeMismatch`.
//! 3. `score_mean` is the arithmetic mean of `step_scores`; `score_var` is the
//!    population variance (NOT sample). Trusted client-side recomputation MAY
//!    be performed by the caller for sanity-check.
//! 4. `mode` MUST be one of `"math"` / `"code"` (snake_case). Mirror of the
//!    request `mode` field. Anything else -> `PpmHookError::InvalidMode`.
//! 5. Empty `reasoning_steps` MUST be rejected at the request side
//!    (`PpmHookError::EmptyInput`) — the hook MUST NOT issue a sidecar call.
//! 6. The hook is `async fn` (native async-trait, no `async-trait` crate per
//!    CLAUDE.md anti-stale rule). Implementations run on tokio.
//! 7. No same-model fallback path. The hook is for the SEPARATE Qwen2.5-
//!    Math-PRM-7B sidecar — never the policy SLM. Per the literature
//!    requirement (arxiv:2504.16760, T1), heterogeneous-verifier is
//!    load-bearing; collapsing to the policy SLM defeats the whole point.
//!
//! # Reliability tier
//!
//! This interface contract is HIGH reliability (cross-referenced with
//! `workflow/papers/rstar-math-2501.04519-interface.md` §3.2 PPM format).
//! The Python implementation that ultimately ships against it will be
//! reviewed source (MEDIUM) until exercised end-to-end with a real model.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Mode enum (closed)
// ---------------------------------------------------------------------------

/// PPM scoring mode. Closed enum — the wire contract pins exactly these
/// two values. Future modes (e.g. `Prose`) MUST be added here AND in the
/// Python worker simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PpmMode {
    Math,
    Code,
}

impl PpmMode {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Math => "math",
            Self::Code => "code",
        }
    }
}

// ---------------------------------------------------------------------------
// Request / Response value types
// ---------------------------------------------------------------------------

/// Input to one PPM scoring call. The caller (e.g. the verifier consensus
/// path in `agent_pool.rs`) constructs this once per candidate-step chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PpmScoreRequest {
    /// Stable opaque id for the candidate being scored. Used for SSB row
    /// correlation; not consumed by the PPM model itself.
    pub candidate_id: String,
    /// Stable opaque id for the verifier sidecar (e.g.
    /// `"ppm_qwen25_math_prm_7b"`). Used for SSB row correlation +
    /// future multi-verifier consensus disambiguation.
    pub verifier_id: String,
    /// Problem text the steps reason about.
    pub problem_statement: String,
    /// Ordered reasoning steps to score. Must be non-empty
    /// (`EmptyInput` otherwise).
    pub reasoning_steps: Vec<String>,
    /// `Math` for `Qwen2.5-Math-PRM-7B`; `Code` for the code-PRM variant
    /// (deferred — wire is reserved).
    pub mode: PpmMode,
}

impl PpmScoreRequest {
    /// Deterministic blake3 cache key for repeated scoring of identical
    /// inputs. Used by `PpmClient` to short-circuit duplicate sidecar
    /// calls within a single session. blake3 chosen per the CLAUDE.md
    /// anti-stale rule (never blake2 / sha2 for non-RFC uses).
    #[must_use]
    pub fn cache_key_hex(&self) -> String {
        let mut h = Hasher::new();
        h.update(self.verifier_id.as_bytes());
        h.update(b"\x00");
        h.update(self.mode.as_str().as_bytes());
        h.update(b"\x00");
        h.update(self.problem_statement.as_bytes());
        h.update(b"\x00");
        for step in &self.reasoning_steps {
            h.update(step.as_bytes());
            h.update(b"\x01");
        }
        h.finalize().to_hex().to_string()
    }
}

/// Output of one PPM scoring call. `step_scores` length matches
/// `request.reasoning_steps.len()`; `score_mean` / `score_var` are the
/// canonical intrinsic signals the consensus rule (D-3) consumes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PpmScoreResponse {
    pub candidate_id: String,
    pub verifier_id: String,
    pub mode: PpmMode,
    pub step_scores: Vec<f32>,
    pub score_mean: f32,
    pub score_var: f32,
    pub num_steps: u32,
    pub latency_ms: u64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// All errors a `SidecarPpmHook` implementation can surface.
#[derive(Debug, Error, PartialEq)]
pub enum PpmHookError {
    /// `reasoning_steps` was empty. Clause 5 of the wire contract.
    #[error("ppm hook rejected empty reasoning_steps")]
    EmptyInput,

    /// `step_scores[i]` was not in `[0.0, 1.0]`. Clause 1.
    #[error("ppm hook returned out-of-range score at index {index}: {value}")]
    InvalidScore { index: usize, value: f32 },

    /// `step_scores.len()` did not match `reasoning_steps.len()`. Clause 2.
    #[error(
        "ppm hook shape mismatch: reasoning_steps={request_steps} step_scores={response_scores}"
    )]
    ShapeMismatch {
        request_steps: usize,
        response_scores: usize,
    },

    /// Wire-format `mode` outside the closed `PpmMode` enum. Clause 4.
    #[error("ppm hook returned invalid mode: {0}")]
    InvalidMode(String),

    /// Sidecar HTTP / transport failure.
    #[error("ppm hook transport error: {0}")]
    Transport(String),

    /// Sidecar offline (no process / port closed). Distinct from
    /// `Transport` so the caller can decide whether to emit a
    /// `PpmGatingFailed` SSB row or a `PpmSidecarUnavailable` row.
    #[error("ppm sidecar unavailable")]
    Unavailable,

    /// Sidecar response could not be parsed.
    #[error("ppm hook parse error: {0}")]
    Parse(String),
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Hook contract the Python PPM sidecar implements. The hook is the only
/// Rust-side surface that talks to the second sidecar (port 8101); the
/// `PpmClient` wraps this trait for caching + SSB emission.
///
/// Native async fn in trait (Rust 1.75+, per CLAUDE.md anti-stale rule —
/// no `async-trait` macro crate).
pub trait SidecarPpmHook: Send + Sync {
    /// Score one candidate-step chain. Implementations MUST validate the
    /// wire-contract clauses (1, 2, 4, 5) defensively even if the Python
    /// side already did — the hook is a trust boundary.
    fn score(
        &self,
        req: PpmScoreRequest,
    ) -> impl std::future::Future<Output = Result<PpmScoreResponse, PpmHookError>> + Send;
}

// ---------------------------------------------------------------------------
// Mock implementation (for unit + integration tests)
// ---------------------------------------------------------------------------

/// Deterministic mock used in tests + the `observable_effect` smoke. NEVER
/// wired in production paths; gated by the construction call site (the
/// production path constructs `HttpSidecarPpmHook` instead — see
/// `ppm_client.rs`).
#[derive(Debug, Clone, Default)]
pub struct MockSidecarPpmHook {
    /// If set, every step gets this score. Otherwise a deterministic
    /// hash-derived score in `[0.0, 1.0]` is produced.
    pub forced_score: Option<f32>,
    /// If true, every call returns `PpmHookError::Unavailable`. Lets the
    /// consensus path exercise the offline branch in tests.
    pub force_unavailable: bool,
    /// If true, every call returns a contract violation. Lets tests
    /// exercise the defensive validation.
    pub force_invalid_score: bool,
}

impl MockSidecarPpmHook {
    /// Construct a mock that returns `forced` for every step.
    #[must_use]
    pub fn with_forced_score(forced: f32) -> Self {
        Self {
            forced_score: Some(forced),
            force_unavailable: false,
            force_invalid_score: false,
        }
    }

    /// Construct a mock that always returns `Unavailable`.
    #[must_use]
    pub fn always_unavailable() -> Self {
        Self {
            forced_score: None,
            force_unavailable: true,
            force_invalid_score: false,
        }
    }

    /// Hash-derived deterministic score for one step (no `forced_score`).
    /// Returns a value in `[0.0, 1.0]` derived from the blake3 hash of the
    /// step text — same input -> same score across runs.
    fn derive_score(&self, step: &str, idx: usize) -> f32 {
        let mut h = Hasher::new();
        h.update(step.as_bytes());
        h.update(&(idx as u64).to_le_bytes());
        let bytes = h.finalize();
        let first4 = &bytes.as_bytes()[..4];
        let raw = u32::from_le_bytes([first4[0], first4[1], first4[2], first4[3]]);
        // Map u32 -> [0, 1]. Division by u32::MAX as f32 keeps it total.
        (raw as f32) / (u32::MAX as f32)
    }
}

impl SidecarPpmHook for MockSidecarPpmHook {
    async fn score(&self, req: PpmScoreRequest) -> Result<PpmScoreResponse, PpmHookError> {
        if req.reasoning_steps.is_empty() {
            return Err(PpmHookError::EmptyInput);
        }
        if self.force_unavailable {
            return Err(PpmHookError::Unavailable);
        }
        if self.force_invalid_score {
            return Err(PpmHookError::InvalidScore {
                index: 0,
                value: 1.5,
            });
        }

        let step_scores: Vec<f32> = req
            .reasoning_steps
            .iter()
            .enumerate()
            .map(|(i, s)| self.forced_score.unwrap_or_else(|| self.derive_score(s, i)))
            .collect();

        // Defensive validation — same clauses the production HTTP hook MUST run.
        for (i, v) in step_scores.iter().enumerate() {
            if !(0.0..=1.0).contains(v) {
                return Err(PpmHookError::InvalidScore {
                    index: i,
                    value: *v,
                });
            }
        }

        let num_steps = step_scores.len() as u32;
        let score_mean = step_scores.iter().sum::<f32>() / (num_steps as f32);
        let score_var = {
            let m = score_mean;
            step_scores.iter().map(|s| (s - m) * (s - m)).sum::<f32>() / (num_steps as f32)
        };

        Ok(PpmScoreResponse {
            candidate_id: req.candidate_id,
            verifier_id: req.verifier_id,
            mode: req.mode,
            step_scores,
            score_mean,
            score_var,
            num_steps,
            latency_ms: 0,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn mock_score_returns_per_step_in_range() {
        let hook = MockSidecarPpmHook::default();
        let req = make_req(vec!["step a", "step b", "step c"]);
        let resp = hook.score(req).await.expect("score ok");
        assert_eq!(resp.num_steps, 3);
        assert_eq!(resp.step_scores.len(), 3);
        for s in resp.step_scores {
            assert!((0.0..=1.0).contains(&s), "score in range");
        }
    }

    #[tokio::test]
    async fn mock_score_with_forced_score_emits_constant() {
        let hook = MockSidecarPpmHook::with_forced_score(0.42);
        let req = make_req(vec!["a", "b"]);
        let resp = hook.score(req).await.expect("score ok");
        for s in resp.step_scores {
            assert!((s - 0.42).abs() < 1e-6);
        }
        assert!((resp.score_mean - 0.42).abs() < 1e-6);
        assert!(resp.score_var.abs() < 1e-6);
    }

    #[tokio::test]
    async fn mock_score_empty_steps_rejected() {
        let hook = MockSidecarPpmHook::default();
        let req = make_req(vec![]);
        match hook.score(req).await {
            Err(PpmHookError::EmptyInput) => {}
            other => panic!("expected EmptyInput, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn mock_score_always_unavailable_returns_unavailable() {
        let hook = MockSidecarPpmHook::always_unavailable();
        let req = make_req(vec!["a"]);
        match hook.score(req).await {
            Err(PpmHookError::Unavailable) => {}
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn mock_score_invalid_score_caught_by_validation() {
        let hook = MockSidecarPpmHook {
            forced_score: None,
            force_unavailable: false,
            force_invalid_score: true,
        };
        let req = make_req(vec!["a"]);
        match hook.score(req).await {
            Err(PpmHookError::InvalidScore { index, value }) => {
                assert_eq!(index, 0);
                assert!(value > 1.0);
            }
            other => panic!("expected InvalidScore, got {other:?}"),
        }
    }

    #[test]
    fn cache_key_is_deterministic_and_input_sensitive() {
        let r1 = make_req(vec!["a", "b"]);
        let r2 = make_req(vec!["a", "b"]);
        let r3 = make_req(vec!["a", "c"]);
        assert_eq!(r1.cache_key_hex(), r2.cache_key_hex());
        assert_ne!(r1.cache_key_hex(), r3.cache_key_hex());
        // 64-char lowercase hex (blake3).
        assert_eq!(r1.cache_key_hex().len(), 64);
        assert!(r1.cache_key_hex().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn mode_as_str_round_trip() {
        assert_eq!(PpmMode::Math.as_str(), "math");
        assert_eq!(PpmMode::Code.as_str(), "code");
    }

    #[tokio::test]
    async fn deterministic_derive_same_input_same_output() {
        let hook = MockSidecarPpmHook::default();
        let req1 = make_req(vec!["alpha", "beta", "gamma"]);
        let req2 = make_req(vec!["alpha", "beta", "gamma"]);
        let r1 = hook.score(req1).await.expect("ok");
        let r2 = hook.score(req2).await.expect("ok");
        assert_eq!(r1.step_scores, r2.step_scores);
        assert!((r1.score_mean - r2.score_mean).abs() < 1e-6);
    }
}
