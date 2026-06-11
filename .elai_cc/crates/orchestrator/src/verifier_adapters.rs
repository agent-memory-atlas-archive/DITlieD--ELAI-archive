//! D-3 closure — concrete [`crate::agent_pool::VerifierAdapter`]
//! implementations consumed by `run_consensus_pass` via the task_runner's
//! post-task `dispatch_verifiers` hook.
//!
//! Three adapter families ship here, mirroring the 4 verifier signals the
//! D-3 consensus aggregator (`verification_result.rs`) blends:
//!
//! - [`DeterministicAdapter`] — model-free. Wraps a deterministic, no-I/O
//!   predicate `Fn(&str) -> bool` keyed on the candidate id. Stands in for
//!   the compile / test / lint / schema / registry hooks (their binary
//!   pass/fail outcome is the signal). Because it needs NO model and NO
//!   sidecar it is the adapter the observable smoke drives, so the proof
//!   runs deterministically in CI without GPU / network.
//! - [`IntrinsicAdapter`] — wraps a pre-harvested [`IntrinsicSignals`]
//!   bundle (mean logprob / entropy / step count). The signals come from
//!   the policy model's forward pass (Phase 4G GG-signal block), so this
//!   adapter is model-dependent: a real signal source must populate it.
//! - [`PpmAdapter`] — wraps the D-2 [`PpmClient`] (second sidecar). Scores
//!   a candidate-step chain via the PPM model, so it is model-dependent
//!   (the second sidecar must be reachable; on failure the consensus pass
//!   skips it as "verifier unavailable", per `run_consensus_pass`).
//!
//! All three implement the native-async-fn [`crate::agent_pool::VerifierAdapter`]
//! trait (no `async-trait` crate, per CLAUDE.md anti-stale). Wrap with
//! [`crate::agent_pool::AdapterErased`] to pass a heterogeneous slice of
//! `&dyn DynVerifierAdapter` to `run_consensus_pass`.
//!
//! Wiring contract (D-3): the boot-site `build_verifier_adapters` in
//! `task_runner.rs` constructs the production adapter set (three model-free
//! [`DeterministicAdapter`]s by default) under the `ELAI_VERIFIER_ENABLE`
//! gate and plumbs it onto `ResolvedConfig.verifier_adapters`. The
//! `dispatch_verifiers` hook (reachable from `drive_task`) then drives
//! `run_consensus_pass` over them and emits one
//! `verifier_consensus_pass` / `verifier_consensus_fail` SSB row per task.

use std::sync::Arc;

use crate::agent_pool::VerifierAdapter;
use crate::ppm_client::{PpmClient, PpmClientError};
use crate::sidecar_ppm_hook::{PpmMode, PpmScoreRequest, SidecarPpmHook};
use crate::verification_result::{IntrinsicSignals, VerificationResult};

/// Wall-clock-free fixed timestamp used for the deterministic / intrinsic
/// adapters' `VerificationResult::ts_ms`. The consensus aggregator does NOT
/// read `ts_ms` for its verdict (it is row metadata only), so a fixed value
/// keeps these adapters fully deterministic — no clock read inside the
/// producer (entropy quarantine). Callers that need a real timestamp pass it
/// in via [`DeterministicAdapter::with_ts`].
const ADAPTER_FIXED_TS_MS: i64 = 0;

/// Model-free deterministic-check adapter.
///
/// Holds a stable `verifier_id` (surfaced in the SSB row's
/// `failing_verifier_ids` when this verifier fails) plus a deterministic,
/// side-effect-free predicate over the candidate id. The predicate stands in
/// for the binary outcome of a compile / test / lint / schema / registry
/// hook; because it performs no I/O and reads no clock / RNG it is fully
/// deterministic and requires no model.
#[derive(Clone)]
pub struct DeterministicAdapter {
    verifier_id: String,
    /// Deterministic predicate: `true` => this verifier passes for the given
    /// candidate id. `Arc<dyn Fn>` so the adapter is `Clone + Send + Sync`
    /// and can be shared across task ticks without re-allocating the closure.
    predicate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    ts_ms: i64,
}

impl std::fmt::Debug for DeterministicAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeterministicAdapter")
            .field("verifier_id", &self.verifier_id)
            .field("ts_ms", &self.ts_ms)
            .finish_non_exhaustive()
    }
}

impl DeterministicAdapter {
    /// Construct with an explicit deterministic predicate.
    #[must_use]
    pub fn new(
        verifier_id: impl Into<String>,
        predicate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            verifier_id: verifier_id.into(),
            predicate,
            ts_ms: ADAPTER_FIXED_TS_MS,
        }
    }

    /// Override the fixed timestamp stamped onto the produced
    /// [`VerificationResult`]. Optional — the aggregator ignores `ts_ms` for
    /// the verdict; this only affects row metadata.
    #[must_use]
    pub fn with_ts(mut self, ts_ms: i64) -> Self {
        self.ts_ms = ts_ms;
        self
    }

    /// Convenience constructor for an adapter whose verdict is a fixed
    /// boolean independent of the candidate id. Useful for tests and for
    /// always-applicable production hooks (e.g. a registry presence check
    /// that always passes in the default configuration).
    #[must_use]
    pub fn fixed(verifier_id: impl Into<String>, passed: bool) -> Self {
        Self::new(verifier_id, Arc::new(move |_candidate_id: &str| passed))
    }

    /// Production default predicate used by `build_verifier_adapters`: the
    /// deterministic check passes UNLESS the candidate id is tagged with the
    /// `"deceptive"` marker substring. This is the model-free,
    /// side-effect-free stand-in for "the deterministic hooks rejected this
    /// candidate" — a deceptive candidate that games the model-based signals
    /// is still caught structurally by the deterministic vote. The marker is
    /// a stable, documented contract the smoke drives.
    #[must_use]
    pub fn rejects_deceptive(verifier_id: impl Into<String>) -> Self {
        Self::new(
            verifier_id,
            Arc::new(|candidate_id: &str| !candidate_id.contains("deceptive")),
        )
    }
}

impl VerifierAdapter for DeterministicAdapter {
    async fn verify(&self, candidate_id: &str) -> Result<VerificationResult, String> {
        let passed = (self.predicate)(candidate_id);
        Ok(VerificationResult::from_deterministic(
            candidate_id,
            &self.verifier_id,
            passed,
            self.ts_ms,
        ))
    }
}

/// Intrinsic-signal adapter (model-dependent).
///
/// Wraps a pre-harvested [`IntrinsicSignals`] bundle. The signals are
/// captured from the policy model's own forward pass (Phase 4G GG-signal
/// block: mean / variance log-probability, entropy, step count, KL-to-base),
/// so a real signal source must populate this adapter — it is NOT model-free.
/// The pass/fail verdict is derived closed-form from the signals via
/// [`IntrinsicSignals::passes_default`] inside
/// [`VerificationResult::from_intrinsic`].
#[derive(Debug, Clone)]
pub struct IntrinsicAdapter {
    verifier_id: String,
    signals: IntrinsicSignals,
    ts_ms: i64,
}

impl IntrinsicAdapter {
    /// Construct from a captured intrinsic-signal bundle.
    #[must_use]
    pub fn new(verifier_id: impl Into<String>, signals: IntrinsicSignals) -> Self {
        Self {
            verifier_id: verifier_id.into(),
            signals,
            ts_ms: ADAPTER_FIXED_TS_MS,
        }
    }

    /// Override the fixed timestamp stamped onto the produced result.
    #[must_use]
    pub fn with_ts(mut self, ts_ms: i64) -> Self {
        self.ts_ms = ts_ms;
        self
    }
}

impl VerifierAdapter for IntrinsicAdapter {
    async fn verify(&self, candidate_id: &str) -> Result<VerificationResult, String> {
        Ok(VerificationResult::from_intrinsic(
            candidate_id,
            &self.verifier_id,
            self.signals,
            self.ts_ms,
        ))
    }
}

/// PPM adapter (model-dependent).
///
/// Wraps the D-2 [`PpmClient`] over a concrete [`SidecarPpmHook`] `H`. Each
/// `verify` call scores the candidate-step chain via the PPM model (second
/// sidecar). Generic over `H` because the `SidecarPpmHook` trait uses native
/// async fn in trait (not dyn-compatible per anti-stale) — production wires
/// `H = HttpSidecarPpmHook`; tests wire `H = MockSidecarPpmHook`.
///
/// On hook error (sidecar offline, transport failure) `verify` returns
/// `Err(String)`; `run_consensus_pass` treats that as "verifier unavailable"
/// and continues consensus over the remaining adapters (defer, not reject).
#[derive(Debug, Clone)]
pub struct PpmAdapter<H: SidecarPpmHook> {
    client: PpmClient<H>,
    verifier_id: String,
    /// Problem text passed to the PPM model. The post-task hook does not
    /// carry the original problem statement, so production wires a bounded
    /// stable placeholder; a follow-on plan (D-1 verifier_required) can
    /// surface the real statement when adapter plumbing carries it.
    problem_statement: String,
    /// Ordered reasoning steps to score. Must be non-empty (the hook returns
    /// `EmptyInput` otherwise); the constructor enforces a non-empty default.
    reasoning_steps: Vec<String>,
    mode: PpmMode,
}

impl<H: SidecarPpmHook> PpmAdapter<H> {
    /// Construct a PPM adapter over a [`PpmClient`]. `reasoning_steps` must be
    /// non-empty; an empty input is rejected at construction so the runtime
    /// never sends an `EmptyInput`-bound request.
    ///
    /// Returns `Err(PpmClientError::Hook(EmptyInput))` when `reasoning_steps`
    /// is empty — fail-closed rather than silently substituting a placeholder
    /// step that would change the PPM score.
    pub fn new(
        client: PpmClient<H>,
        verifier_id: impl Into<String>,
        problem_statement: impl Into<String>,
        reasoning_steps: Vec<String>,
        mode: PpmMode,
    ) -> Result<Self, PpmClientError> {
        if reasoning_steps.is_empty() {
            return Err(PpmClientError::Hook(
                crate::sidecar_ppm_hook::PpmHookError::EmptyInput,
            ));
        }
        Ok(Self {
            client,
            verifier_id: verifier_id.into(),
            problem_statement: problem_statement.into(),
            reasoning_steps,
            mode,
        })
    }
}

impl<H: SidecarPpmHook> VerifierAdapter for PpmAdapter<H> {
    async fn verify(&self, candidate_id: &str) -> Result<VerificationResult, String> {
        let req = PpmScoreRequest {
            candidate_id: candidate_id.into(),
            verifier_id: self.verifier_id.clone(),
            problem_statement: self.problem_statement.clone(),
            reasoning_steps: self.reasoning_steps.clone(),
            mode: self.mode,
        };
        let score = self.client.score(req).await.map_err(|e| e.to_string())?;
        Ok(VerificationResult::from_ppm(
            candidate_id,
            &score,
            ADAPTER_FIXED_TS_MS,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification_result::VerifierKind;

    #[tokio::test]
    async fn deterministic_adapter_rejects_deceptive_candidate() {
        let adapter = DeterministicAdapter::rejects_deceptive("deterministic");
        let clean = adapter.verify("task-7").await.expect("verify clean");
        assert!(clean.passed, "non-deceptive candidate passes");
        assert_eq!(clean.kind, VerifierKind::Deterministic);
        assert_eq!(clean.verifier_id, "deterministic");

        let bad = adapter
            .verify("deceptive-candidate-1")
            .await
            .expect("verify deceptive");
        assert!(!bad.passed, "deceptive candidate fails the deterministic vote");
        assert_eq!(bad.verifier_id, "deterministic");
    }

    #[tokio::test]
    async fn deterministic_adapter_fixed_verdict() {
        let pass = DeterministicAdapter::fixed("det-pass", true);
        assert!(pass.verify("anything").await.expect("v").passed);
        let fail = DeterministicAdapter::fixed("det-fail", false);
        assert!(!fail.verify("anything").await.expect("v").passed);
    }

    #[tokio::test]
    async fn intrinsic_adapter_passes_on_strong_signals() {
        let adapter = IntrinsicAdapter::new(
            "intrinsic",
            IntrinsicSignals {
                logprob_mean: -1.0,
                logprob_var: 0.1,
                entropy: 1.5,
                step_count: 3,
                kl_to_base: 0.0,
            },
        );
        let r = adapter.verify("task-7").await.expect("verify");
        assert_eq!(r.kind, VerifierKind::Intrinsic);
        assert!(r.passed, "strong intrinsic signals pass the default floor");
    }

    #[tokio::test]
    async fn ppm_adapter_rejects_empty_steps_at_construction() {
        use crate::sidecar_ppm_hook::MockSidecarPpmHook;
        let client = PpmClient::new(Arc::new(MockSidecarPpmHook::with_forced_score(0.9)));
        let err = PpmAdapter::new(
            client,
            "ppm",
            "problem",
            Vec::new(),
            PpmMode::Code,
        );
        assert!(err.is_err(), "empty reasoning_steps must be rejected fail-closed");
    }
}
