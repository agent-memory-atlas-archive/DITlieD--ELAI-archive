//! B-24 Phase 4I Group C5 (4I.7 SKELETON): Visual feedback loop.
//!
//! Spec phase-4I.txt L407-414. The loop iterates up to
//! `DEFAULT_MAX_ITERATIONS = 3` (spec L451 — overrides the B-24 plan's
//! 5; decision-doc entry 06). Each iteration:
//!   1. Render artifact via `AdapterRegistry::get_adapter(output_type)`.
//!   2. Run Tier 1 deterministic checks on the captured frames
//!      (in C5 SKELETON we pass through to the existing Tier1 dispatch).
//!   3. If Tier1.pass and `qualitative_checks`, run Tier 2 (VLM).
//!   4. Aggregate Tier-2 Issues into `VisualIssue`s.
//!   5. Convergence check: if the (severity, category, location_bucket)
//!      tuple-set is unchanged from the previous iteration, stop early
//!      (spec adversarial L487-489 — "compare by severity + category +
//!      location, NOT raw description string").
//!   6. Tier1 fail stops the loop immediately (same veto as base
//!      validation; spec L454).
//!
//! SSB emission: each iteration emits a `visual_feedback_iteration`
//! action via the optional `&dyn pipeline::MediaSsbSink` borrow on
//! `run(...)` (decision 60 — audit WARN 2 fix). Group C4 already
//! shipped the `VisualFeedbackIterationPayload` schema
//! (crates/orchestrator/src/ssb.rs L1514-1519); the trait method
//! `MediaSsbSink::on_visual_feedback_iteration` has a no-op default
//! so the test sinks that pre-date Group C5 keep compiling. Spec
//! L493-494 — emit one event per iteration so the orchestrator can
//! detect agent liveness during long render loops.
//!
//! Anti-stale grounding:
//!   * tokio::sync::mpsc — async channels (NOT std::sync::mpsc in
//!     async contexts; CLAUDE.md).
//!   * tracing for structured logs.
//!   * No `.unwrap()` in non-test code.

use std::collections::HashSet;
use std::future::Future;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Type alias for the boxed-future shape used by `run_with_sink`'s
/// evaluator closure. Decision 72: future is `Send + 'a` so multi-thread
/// tokio executors can poll it without nested-runtime workarounds.
#[allow(dead_code)]
pub type EvaluatorFuture<'a> = std::pin::Pin<
    Box<
        dyn Future<Output = (Tier1Result, Option<Tier2Result>, Vec<VisualIssue>)> + Send + 'a,
    >,
>;

use super::render_adapters::{
    AdapterRegistry, CapturedFrame, ConvergenceKey, OutputType, RenderAdapterError, RenderConfig,
    VisualIssue,
};
use super::types::{
    DEFAULT_MAX_ITERATIONS, QualitySpec, Severity, Tier1Result, Tier2Result, Verdict,
};

// ---------------------------------------------------------------------------
// Public types (spec L407-416).
// ---------------------------------------------------------------------------

/// Per-iteration record. Captures frames + tier results + the
/// `VisualIssue`s seen for this iteration plus the set of issues
/// `resolved_from_previous` (i.e. present in iteration N-1, absent here).
///
/// Spec L412-414.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IterationRecord {
    pub iteration: u32,
    pub captured_frames: Vec<CapturedFrame>,
    pub tier1_result: Tier1Result,
    #[serde(default)]
    pub tier2_result: Option<Tier2Result>,
    pub issues: Vec<VisualIssue>,
    pub resolved_from_previous: Vec<VisualIssue>,
}

/// Aggregate result of the feedback loop. `converged` is true when the
/// loop stopped because the issues set stabilized (spec L477-478);
/// false when stopped because `iterations == max_iterations` or
/// because Tier 1 vetoed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualFeedbackResult {
    pub iterations: u32,
    pub final_verdict: Verdict,
    pub iteration_history: Vec<IterationRecord>,
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// VisualFeedbackLoop.
// ---------------------------------------------------------------------------

/// Visual-feedback orchestrator. Holds the AdapterRegistry + the
/// configured `max_iterations` (default 3 — spec L451) + the quality
/// spec needed for Tier 1 / Tier 2 dispatch.
///
/// The loop is structured as `run(...)` so callers from `pipeline::validate`
/// can integrate it into the existing Tier1→Tier2→Verdict flow. Group C6
/// wires this through the real `TaskRunner::execute_visual_feedback`
/// path; today the loop is callable but the Tier-2 invocation is
/// supplied via a closure parameter for testability (no live VLM).
pub struct VisualFeedbackLoop {
    pub adapter_registry: AdapterRegistry,
    pub max_iterations: u32,
    pub quality_spec: QualitySpec,
}

impl VisualFeedbackLoop {
    /// Construct with the provided registry. `max_iterations` defaults
    /// to `DEFAULT_MAX_ITERATIONS` (3).
    pub fn new(adapter_registry: AdapterRegistry, quality_spec: QualitySpec) -> Self {
        Self {
            adapter_registry,
            max_iterations: DEFAULT_MAX_ITERATIONS,
            quality_spec,
        }
    }

    pub fn with_max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = n;
        self
    }

    /// Run the loop. `executor_fix` is a closure that takes the issues
    /// from the previous iteration and is expected to mutate the
    /// artifact on disk before returning. The executor is supplied by
    /// the caller (TaskRunner in production; a stub in tests) so the
    /// loop's convergence logic stays decoupled from the real fix path.
    ///
    /// `render_evaluator` is an ASYNC closure (audit BLOCK 3 — decision
    /// 72): takes (artifact, frames) and returns a boxed future yielding
    /// `(Tier1Result, Option<Tier2Result>, Vec<VisualIssue>)`. The async
    /// shape lets the evaluator await `Tier1Validator::validate` directly
    /// without nesting a `current_thread` runtime — `validate_with_rce`
    /// now runs from `#[tokio::test]` / a parent runtime, and the
    /// previous sync wrapper + `Builder::new_current_thread()` chain
    /// (which panicked with "Cannot start a runtime from within a
    /// runtime") is removed.
    ///
    /// Convenience wrapper that calls [`Self::run_with_sink`] with
    /// `sink = None`. New callers should use `run_with_sink` directly
    /// so each iteration emits a `visual_feedback_iteration` SSB action
    /// (spec phase-4I.txt L493-494, audit WARN 2 fix — decision 60).
    pub async fn run<R, RFut, F>(
        &self,
        artifact: &std::path::Path,
        output_type: &OutputType,
        config: &RenderConfig,
        render_evaluator: R,
        executor_fix: F,
    ) -> Result<VisualFeedbackResult, RenderAdapterError>
    where
        R: FnMut(PathBuf, Vec<CapturedFrame>) -> RFut,
        RFut: Future<Output = (Tier1Result, Option<Tier2Result>, Vec<VisualIssue>)> + Send,
        F: FnMut(&[VisualIssue]),
    {
        self.run_with_sink(
            artifact,
            output_type,
            config,
            render_evaluator,
            executor_fix,
            None,
            None,
        )
        .await
    }

    /// Same as [`Self::run`] but takes an optional SSB sink + an
    /// optional `step_id` string. When `sink = Some(s)`, one
    /// `visual_feedback_iteration` action is emitted per iteration via
    /// [`super::pipeline::MediaSsbSink::on_visual_feedback_iteration`].
    /// `step_id` defaults to `"unknown"` when None — production
    /// callers should pass the request id stringified.
    ///
    /// `render_evaluator` is async (audit BLOCK 3 — decision 72). The
    /// boxed-future shape `FnMut(PathBuf, Vec<CapturedFrame>) -> impl Future`
    /// owns its inputs so the future is `'static`-clean and can be
    /// awaited without lifetime gymnastics; test fixtures wrap sync
    /// computations with `async move { ... }` and incur ~zero cost.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_with_sink<R, RFut, F>(
        &self,
        artifact: &std::path::Path,
        output_type: &OutputType,
        config: &RenderConfig,
        mut render_evaluator: R,
        mut executor_fix: F,
        sink: Option<&dyn super::pipeline::MediaSsbSink>,
        step_id: Option<&str>,
    ) -> Result<VisualFeedbackResult, RenderAdapterError>
    where
        R: FnMut(PathBuf, Vec<CapturedFrame>) -> RFut,
        RFut: Future<Output = (Tier1Result, Option<Tier2Result>, Vec<VisualIssue>)> + Send,
        F: FnMut(&[VisualIssue]),
    {
        let step_id = step_id.unwrap_or("unknown");
        // -----------------------------------------------------------------
        // 1. Adapter selection (deterministic — spec L448).
        // -----------------------------------------------------------------
        let adapter = self
            .adapter_registry
            .get_adapter(output_type)
            .ok_or_else(|| {
                RenderAdapterError::ArtifactPathInvalid {
                    path: artifact.to_path_buf(),
                }
            })?;
        info!(adapter = adapter.name(), output_type = ?output_type, "visual_feedback_loop_start");

        // -----------------------------------------------------------------
        // 2. Iteration loop.
        // -----------------------------------------------------------------
        let mut history: Vec<IterationRecord> = Vec::with_capacity(self.max_iterations as usize);
        let mut prev_keys: HashSet<ConvergenceKey> = HashSet::new();
        let mut converged = false;
        let mut final_verdict = Verdict::Degraded;

        // SSB emission helper — invoked once per iteration (audit WARN 2
        // fix; decision 60). When sink is None, falls back to a
        // `tracing::info!` line so production callers without an SSB still
        // leave an audit trail. `issue_count` is bounded to u32::MAX.
        let emit_iter = |iter_idx: u32, issue_count: usize, converged: bool| {
            let count = u32::try_from(issue_count).unwrap_or(u32::MAX);
            match sink {
                Some(s) => {
                    s.on_visual_feedback_iteration(step_id, iter_idx, count, converged);
                }
                None => info!(
                    step_id,
                    iteration = iter_idx,
                    issue_count = count,
                    converged,
                    "visual_feedback_iteration"
                ),
            }
        };

        for iter_idx in 0..self.max_iterations {
            // 2.1. Render.
            let frames = adapter.render(artifact, config).await?;
            debug!(iter = iter_idx, frame_count = frames.len(), "rendered");

            // 2.2. Evaluate (Tier1 + optional Tier2). Async closure so
            // Tier1Validator's awaited futures don't need a nested
            // runtime (audit BLOCK 3 — decision 72). Owned PathBuf +
            // Vec<CapturedFrame> keep the future `'static`-clean.
            let (tier1, tier2, issues) = render_evaluator(
                artifact.to_path_buf(),
                frames.clone(),
            )
            .await;

            // 2.3. Tier 1 fail -> stop immediately (spec L454).
            if !tier1.pass {
                warn!(iter = iter_idx, "tier1_veto_in_feedback_loop");
                let issue_n = issues.len();
                history.push(IterationRecord {
                    iteration: iter_idx,
                    captured_frames: frames,
                    tier1_result: tier1,
                    tier2_result: tier2,
                    issues,
                    resolved_from_previous: Vec::new(),
                });
                final_verdict = Verdict::Fail;
                // converged=false: Tier1 vetoed before stability could be observed.
                emit_iter(iter_idx, issue_n, false);
                break;
            }

            // 2.4. Compute convergence keys + resolved-from-previous.
            let cur_keys: HashSet<ConvergenceKey> =
                issues.iter().map(VisualIssue::convergence_key).collect();
            let resolved: Vec<VisualIssue> = if iter_idx == 0 {
                Vec::new()
            } else {
                // Previous iteration's issues are stored in history[last].
                history
                    .last()
                    .map(|prev| {
                        prev.issues
                            .iter()
                            .filter(|i| !cur_keys.contains(&i.convergence_key()))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            };

            // 2.5. Convergence detection (spec adversarial L487-489).
            let stabilized = iter_idx > 0 && cur_keys == prev_keys;
            let issue_n = issues.len();

            history.push(IterationRecord {
                iteration: iter_idx,
                captured_frames: frames,
                tier1_result: tier1,
                tier2_result: tier2,
                issues: issues.clone(),
                resolved_from_previous: resolved,
            });

            if stabilized {
                info!(iter = iter_idx, "feedback_loop_converged");
                converged = true;
                // Verdict: if any critical/major issue remains -> HumanReview.
                final_verdict = verdict_from_issues(&issues);
                emit_iter(iter_idx, issue_n, true);
                break;
            }

            // 2.6. If no issues at all -> Pass + converged.
            if issues.is_empty() {
                info!(iter = iter_idx, "feedback_loop_pass_no_issues");
                converged = true;
                final_verdict = Verdict::Pass;
                emit_iter(iter_idx, issue_n, true);
                break;
            }

            // 2.7. Last iteration -> stop (regardless of issues).
            if iter_idx + 1 == self.max_iterations {
                info!(iter = iter_idx, "feedback_loop_max_iterations");
                final_verdict = verdict_from_issues(&issues);
                emit_iter(iter_idx, issue_n, false);
                break;
            }

            // SSB emit BEFORE handing off to executor so consumers see one
            // event per iteration that ran to completion.
            emit_iter(iter_idx, issue_n, false);

            // 2.8. Hand off to executor for the fix attempt.
            executor_fix(&issues);
            prev_keys = cur_keys;
        }

        // Cleanup (spec adversarial L490-491).
        adapter.cleanup();

        Ok(VisualFeedbackResult {
            iterations: u32::try_from(history.len()).unwrap_or(u32::MAX),
            final_verdict,
            iteration_history: history,
            converged,
        })
    }
}

/// Map an issue set to a Verdict. Any Critical -> Fail; any Major -> HumanReview;
/// otherwise (Minor or empty) -> Pass.
fn verdict_from_issues(issues: &[VisualIssue]) -> Verdict {
    let any_critical = issues.iter().any(|i| matches!(i.severity, Severity::Critical));
    let any_major = issues.iter().any(|i| matches!(i.severity, Severity::Major));
    if any_critical {
        Verdict::Fail
    } else if any_major {
        Verdict::HumanReview
    } else {
        Verdict::Pass
    }
}

/// Borrow-discipline marker to silence the unused-import lint when
/// PathBuf is referenced only via trait signatures.
#[allow(dead_code)]
fn _path_buf_marker(_: PathBuf) {}

// ---------------------------------------------------------------------------
// Unit tests — pin the convergence, max-iteration, and Tier-1-veto
// invariants. End-to-end integration tests live under
// tests/media_validator_rce_integration.rs.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_validator::render_adapters::{
        OutputType, UiScreenshotAdapter, Viewport,
    };
    use crate::media_validator::types::{CheckResult, ToolCapabilityFlags};
    use tempfile::TempDir;

    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    fn write_png(name: &str, dir: &std::path::Path) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, TINY_PNG).unwrap();
        p
    }

    fn pass_tier1() -> Tier1Result {
        Tier1Result {
            pass: true,
            checks: vec![CheckResult {
                name: "synthetic".into(),
                pass: true,
                detail: "ok".into(),
                latency_ms: 0,
            }],
            latency_ms: 0,
        }
    }

    fn fail_tier1() -> Tier1Result {
        Tier1Result {
            pass: false,
            checks: vec![CheckResult {
                name: "synthetic".into(),
                pass: false,
                detail: "veto".into(),
                latency_ms: 0,
            }],
            latency_ms: 0,
        }
    }

    fn registry_with_ui() -> AdapterRegistry {
        let mut r = AdapterRegistry::new();
        r.register_unique(OutputType::UiScreenshot, Box::new(UiScreenshotAdapter::new()))
            .unwrap();
        r
    }

    #[tokio::test]
    async fn max_iterations_enforced_default_three() {
        let dir = TempDir::new().unwrap();
        let path = write_png("a.png", dir.path());
        let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
        // Evaluator that always returns 1 issue with the same key, but
        // executor never resolves it -> loop must stop at max_iterations.
        let issue_template = VisualIssue {
            description: "button overflow: always present".into(),
            severity: Severity::Major,
            location: None,
            viewport: Some(Viewport::desktop()),
            actionable_fix: "increase width".into(),
        };
        // We force NO convergence by varying the bucketed location each
        // iteration. This isolates the max-iterations branch from the
        // convergence branch.
        let mut iter_n = 0u32;
        let issue_t = issue_template.clone();
        let res = loop_
            .run(
                &path,
                &OutputType::UiScreenshot,
                &RenderConfig::default(),
                |_a, _f| {
                    iter_n += 1;
                    let mut i = issue_t.clone();
                    i.description = format!("issue #{}", iter_n);
                    i.location = Some(crate::media_validator::types::BoundingBox {
                        x: 0.1 * (iter_n as f32),
                        y: 0.2,
                        width: 0.1,
                        height: 0.1,
                    });
                    let tier1 = pass_tier1();
                    let result = (tier1, None, vec![i]);
                    async move { result }
                },
                |_issues| { /* no-op: simulate executor failing to fix */ },
            )
            .await
            .unwrap();
        assert_eq!(res.iterations, 3);
        assert!(!res.converged);
        assert!(matches!(res.final_verdict, Verdict::HumanReview));
    }

    #[tokio::test]
    async fn tier1_veto_stops_loop_at_iteration_one() {
        let dir = TempDir::new().unwrap();
        let path = write_png("a.png", dir.path());
        let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
        let res = loop_
            .run(
                &path,
                &OutputType::UiScreenshot,
                &RenderConfig::default(),
                |_a, _f| async move { (fail_tier1(), None, vec![]) },
                |_| {},
            )
            .await
            .unwrap();
        assert_eq!(res.iterations, 1, "Tier1 fail -> single iteration");
        assert!(matches!(res.final_verdict, Verdict::Fail));
    }

    #[tokio::test]
    async fn convergence_by_key_not_string() {
        let dir = TempDir::new().unwrap();
        let path = write_png("a.png", dir.path());
        let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
        let mut call = 0u32;
        let res = loop_
            .run(
                &path,
                &OutputType::UiScreenshot,
                &RenderConfig::default(),
                |_a, _f| {
                    call += 1;
                    // Same severity + category + location across iters,
                    // but description string differs. Loop must converge
                    // at iteration 2 (idx=1) because the convergence key
                    // is identical.
                    let i = VisualIssue {
                        description: format!(
                            "button_overflow: rephrased wording attempt {}",
                            call
                        ),
                        severity: Severity::Major,
                        location: Some(crate::media_validator::types::BoundingBox {
                            x: 0.1,
                            y: 0.2,
                            width: 0.1,
                            height: 0.1,
                        }),
                        viewport: None,
                        actionable_fix: "increase width".into(),
                    };
                    let r = (pass_tier1(), None, vec![i]);
                    async move { r }
                },
                |_| {},
            )
            .await
            .unwrap();
        assert!(res.converged, "convergence by key, not raw description");
        assert_eq!(res.iterations, 2);
        assert!(matches!(res.final_verdict, Verdict::HumanReview));
    }

    #[tokio::test]
    async fn empty_issues_converges_immediately_with_pass() {
        let dir = TempDir::new().unwrap();
        let path = write_png("a.png", dir.path());
        let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
        let res = loop_
            .run(
                &path,
                &OutputType::UiScreenshot,
                &RenderConfig::default(),
                |_a, _f| async move { (pass_tier1(), None, vec![]) },
                |_| {},
            )
            .await
            .unwrap();
        assert!(res.converged);
        assert_eq!(res.iterations, 1);
        assert!(matches!(res.final_verdict, Verdict::Pass));
    }

    /// Documents that `from_capabilities` registers UiScreenshot even
    /// when no tools are detected (file-only adapter).
    #[test]
    fn from_capabilities_smoke() {
        let r = AdapterRegistry::from_capabilities(&ToolCapabilityFlags::empty());
        assert!(r.get_adapter(&OutputType::UiScreenshot).is_some());
    }
}
