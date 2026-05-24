//! C-52 — Self-healing iteration loop per arXiv:2604.22601 RQ3.
//!
//! Paper §3 caps self-healing at 10 iterations. Each iteration:
//!   1. Sends the current `(nl_problem, prior_attempts)` tuple to the
//!      LLM (`model_invoker`) and receives the next Dafny source.
//!   2. The caller writes that source to disk + invokes
//!      `cli_shellout::invoke_dafny_cli`.
//!   3. If `DafnyVerifyResult::dafny_pass && udebug_pass`, the loop
//!      terminates with success. Otherwise the verifier feedback is
//!      appended to the prior-attempts list and the loop iterates.
//!
//! This module owns the LOOP CONTROL only — the LLM dispatch and
//! Dafny shellout happen via injected callables so the loop is
//! testable without a live LLM or live Dafny binary.

use std::future::Future;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dafny_verify::cli_shellout::DafnyVerifyResult;

/// Hard cap on self-healing iterations per paper §3. The paper does
/// not characterise diminishing returns past 10; do NOT raise without
/// a fresh ablation.
pub const MAX_HEAL_ITER: u32 = 10;

/// Per-iteration record. `dafny_source` is the LLM output that was
/// fed to the verifier; `verifier_feedback` is the captured stdout
/// (truncated per `DafnyVerifyResult::MAX_STDOUT_BYTES`). The trace
/// is the audit-time evidence of what the loop tried.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealStep {
    pub iter_index: u32,
    pub dafny_source: String,
    pub verifier_feedback: String,
    pub dafny_pass: bool,
    pub udebug_pass: bool,
}

/// Captured trace of a self-healing run. Bounded by `MAX_HEAL_ITER`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HealTrace {
    pub steps: Vec<HealStep>,
    /// Whether the final iteration succeeded.
    pub final_success: bool,
    /// Total iterations actually run. `<= MAX_HEAL_ITER`.
    pub iterations_used: u32,
}

impl HealTrace {
    /// Convenience predicate: the loop ran to exhaustion without
    /// finding a verifying spec.
    pub fn exhausted(&self) -> bool {
        !self.final_success && self.iterations_used >= MAX_HEAL_ITER
    }
}

/// Error type. `IterationCapBreached` cannot occur from the public
/// API (the loop counter is monotonic and capped) — the variant
/// exists for defensive callers wrapping this surface.
#[derive(Debug, Error)]
pub enum HealError {
    #[error("model invoker returned an error at iteration {iter}: {source}")]
    ModelInvocation {
        iter: u32,
        #[source]
        source: anyhow::Error,
    },
    #[error("iteration cap {MAX_HEAL_ITER} breached (defensive guard)")]
    IterationCapBreached,
}

/// Run the self-healing loop.
///
/// Arguments:
///   - `initial`: the verifier result from the FIRST attempt (often
///     the Signature-tier RQ2 attempt). If `dafny_pass && udebug_pass`
///     already, the loop returns immediately with one successful step.
///   - `initial_source`: the Dafny source corresponding to `initial`.
///   - `max_iter`: cap on iterations beyond the initial attempt.
///     Caller MAY pass a value smaller than `MAX_HEAL_ITER` (e.g. for
///     a budget-constrained run); MUST NOT exceed `MAX_HEAL_ITER` —
///     the function clamps silently.
///   - `model_invoker`: async closure that takes a freshly-built
///     prompt and returns the next Dafny source. Errors propagate as
///     `HealError::ModelInvocation`.
///   - `dafny_verifier`: async closure that takes the latest Dafny
///     source and returns `DafnyVerifyResult`. Allows the caller to
///     inject a test-friendly fake (e.g. fixture-driven) without
///     spawning a real `dafny` process.
///
/// The loop never panics; cap-breach is defensive.
pub async fn self_heal_loop<M, MFut, V, VFut>(
    initial: DafnyVerifyResult,
    initial_source: String,
    max_iter: u32,
    mut model_invoker: M,
    mut dafny_verifier: V,
) -> Result<HealTrace, HealError>
where
    M: FnMut(&str, &[HealStep]) -> MFut,
    MFut: Future<Output = Result<String, anyhow::Error>>,
    V: FnMut(&str) -> VFut,
    VFut: Future<Output = DafnyVerifyResult>,
{
    let cap = max_iter.min(MAX_HEAL_ITER);
    let mut trace = HealTrace::default();

    // Initial step always recorded (iter 0). The loop "iterations
    // beyond the initial" are bounded by `cap`.
    let initial_step = HealStep {
        iter_index: 0,
        dafny_source: initial_source,
        verifier_feedback: initial.stdout.clone(),
        dafny_pass: initial.dafny_pass,
        udebug_pass: initial.udebug_pass,
    };
    let initial_success = initial.dafny_pass && initial.udebug_pass;
    trace.steps.push(initial_step);
    trace.iterations_used = 0;
    trace.final_success = initial_success;
    if initial_success {
        return Ok(trace);
    }

    for iter in 1..=cap {
        // Defensive: should never trigger because `cap <= MAX_HEAL_ITER`.
        if iter > MAX_HEAL_ITER {
            return Err(HealError::IterationCapBreached);
        }

        // Build the per-iteration prompt body via the model_invoker
        // closure (the closure embeds the NL problem + builds the
        // tier=self_healing prompt internally; we pass only the prior
        // trace so the closure can choose feedback formatting).
        let next_source = model_invoker("self_healing", &trace.steps)
            .await
            .map_err(|source| HealError::ModelInvocation { iter, source })?;

        let result = dafny_verifier(&next_source).await;
        let success = result.dafny_pass && result.udebug_pass;

        trace.steps.push(HealStep {
            iter_index: iter,
            dafny_source: next_source,
            verifier_feedback: result.stdout.clone(),
            dafny_pass: result.dafny_pass,
            udebug_pass: result.udebug_pass,
        });
        trace.iterations_used = iter;

        if success {
            trace.final_success = true;
            return Ok(trace);
        }
    }

    trace.final_success = false;
    Ok(trace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dafny_verify::cli_shellout::DafnyVerifyResult;

    fn fail_result(banner: &str) -> DafnyVerifyResult {
        DafnyVerifyResult {
            dafny_pass: false,
            udebug_pass: false,
            vacuous_guard_passed: false,
            stdout: banner.to_string(),
            exit_code: 1,
        }
    }

    fn pass_result(banner: &str) -> DafnyVerifyResult {
        DafnyVerifyResult {
            dafny_pass: true,
            udebug_pass: true,
            vacuous_guard_passed: true,
            stdout: banner.to_string(),
            exit_code: 0,
        }
    }

    #[tokio::test]
    async fn initial_success_short_circuits() {
        let initial = pass_result("Dafny program verifier finished");
        let trace = self_heal_loop(
            initial,
            "method foo() { /* good */ }".into(),
            5,
            |_, _| async { Ok::<_, anyhow::Error>("unused".to_string()) },
            |_| async { fail_result("never called") },
        )
        .await
        .expect("loop");
        assert!(trace.final_success);
        assert_eq!(trace.iterations_used, 0);
        assert_eq!(trace.steps.len(), 1);
    }

    #[tokio::test]
    async fn loop_respects_max_iter_cap() {
        let initial = fail_result("err: precondition");
        // Force every subsequent iteration to fail.
        let trace = self_heal_loop(
            initial,
            "method foo() { /* bad */ }".into(),
            MAX_HEAL_ITER + 5, // caller-passed cap exceeds paper cap; clamp expected
            |_, prior| {
                let n = prior.len();
                async move { Ok::<_, anyhow::Error>(format!("attempt {n}")) }
            },
            |_| async { fail_result("err: still bad") },
        )
        .await
        .expect("loop");

        assert!(!trace.final_success);
        assert_eq!(trace.iterations_used, MAX_HEAL_ITER);
        // initial + MAX_HEAL_ITER per-iteration entries.
        assert_eq!(trace.steps.len(), (MAX_HEAL_ITER as usize) + 1);
        assert!(trace.exhausted());
    }

    #[tokio::test]
    async fn loop_terminates_on_first_success() {
        let initial = fail_result("err: postcondition");
        let trace = self_heal_loop(
            initial,
            "method foo() { /* v0 */ }".into(),
            5,
            |_, prior| {
                let n = prior.len();
                async move { Ok::<_, anyhow::Error>(format!("v{n}")) }
            },
            // First retry succeeds.
            |_| async { pass_result("Dafny program verifier finished") },
        )
        .await
        .expect("loop");
        assert!(trace.final_success);
        assert_eq!(trace.iterations_used, 1);
        assert_eq!(trace.steps.len(), 2);
    }
}
