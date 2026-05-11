//! B-24 Phase 4I Group C5 (4I.8 TESTS): RCE contract invariants.
//!
//! Spec phase-4I.txt L447-455: 7 contract invariants for the
//! visual-feedback loop. Each test pins one invariant. All 7 pass
//! TODAY (Group C5 SKELETON) because the contract is exercised against
//! the in-process `VisualFeedbackLoop` + a stub evaluator — no real
//! Playwright / Blender / VLM dependency.
//!
//! Test discipline (CLAUDE.md):
//!   * `.unwrap()` / `.expect()` permitted in `#[test]` bodies.
//!   * Deterministic — no thread::sleep, no network.
//!   * Synthetic fixtures only.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tempfile::TempDir;

use elai_orchestrator::media_validator::feedback_loop::{
    IterationRecord, VisualFeedbackLoop, VisualFeedbackResult,
};
use elai_orchestrator::media_validator::render_adapters::{
    AdapterRegistry, BlenderRenderAdapter, CapturedFrame, OutputType, RenderAdapterError,
    RenderConfig, UiScreenshotAdapter, Viewport, VisualIssue, WebRenderAdapter,
};
use elai_orchestrator::media_validator::types::{
    BoundingBox, CheckResult, QualitySpec, Severity, Tier1Result, ToolCapabilityFlags, Verdict,
    DEFAULT_MAX_ITERATIONS,
};

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn write_png(dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, TINY_PNG).unwrap();
    p
}

fn registry_with_ui() -> AdapterRegistry {
    let mut r = AdapterRegistry::new();
    r.register_unique(OutputType::UiScreenshot, Box::new(UiScreenshotAdapter::new()))
        .unwrap();
    r
}

fn tier1_pass() -> Tier1Result {
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

fn tier1_fail() -> Tier1Result {
    Tier1Result {
        pass: false,
        checks: vec![CheckResult {
            name: "synthetic_veto".into(),
            pass: false,
            detail: "synthetic Tier1 veto".into(),
            latency_ms: 0,
        }],
        latency_ms: 0,
    }
}

// ---------------------------------------------------------------------------
// rce_c1: Adapter selection is deterministic.
// ---------------------------------------------------------------------------

/// Spec L448: same output_type ⇒ same adapter (no LLM in the selection
/// path). 100 lookups must return the SAME adapter instance (by
/// `name()` since the trait object pointer can't be compared directly).
#[test]
fn rce_c1_adapter_selection_deterministic() {
    let mut r = AdapterRegistry::new();
    r.register_unique(OutputType::Web, Box::new(WebRenderAdapter::new()))
        .unwrap();
    r.register_unique(
        OutputType::UiScreenshot,
        Box::new(UiScreenshotAdapter::new()),
    )
    .unwrap();
    for _ in 0..100 {
        let web = r.get_adapter(&OutputType::Web).unwrap();
        let ui = r.get_adapter(&OutputType::UiScreenshot).unwrap();
        assert_eq!(web.name(), "web");
        assert_eq!(ui.name(), "ui_screenshot");
    }
    // Unregistered output type returns None — deterministic too.
    assert!(r.get_adapter(&OutputType::Model3D).is_none());
}

// ---------------------------------------------------------------------------
// rce_c2: visual_check=false ⇒ no adapter instantiated.
// ---------------------------------------------------------------------------

/// Spec L449: when `request.quality_spec.visual_check == false`, the
/// `pipeline::validate_with_rce` entry point MUST NOT consult the
/// AdapterRegistry — no `get_adapter`, no `RenderAdapter::render`. We
/// pin this with a SPY adapter that increments a shared AtomicU32 on
/// every `can_handle` / `render` call, then run the full validation
/// pipeline through `validate_with_rce` and assert the counter is
/// still zero.
///
/// Audit WARN 3 fix (decision 61): previously this test was vacuous
/// (asserted calls==0 without ever invoking the pipeline). Now it
/// exercises the real gate via `pipeline::validate_with_rce`.
/// Decision 72 (audit BLOCK 3): the function is now `async` (no
/// internal current-thread runtime — that path panicked with
/// "Cannot start a runtime from within a runtime"). Test runs under
/// `#[tokio::test]`.
#[tokio::test]
async fn rce_c2_visual_check_false_skips_render() {
    // Sentinel: counting adapter wraps UiScreenshotAdapter and bumps a
    // shared AtomicU32 on every can_handle / render call.
    struct CountingAdapter {
        inner: UiScreenshotAdapter,
        calls: Arc<AtomicU32>,
    }
    impl elai_orchestrator::media_validator::render_adapters::MediaRenderAdapter for CountingAdapter {
        fn can_handle(&self, ot: &OutputType) -> bool {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.can_handle(ot)
        }
        fn render<'a>(
            &'a self,
            artifact: &'a Path,
            config: &'a RenderConfig,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<Vec<CapturedFrame>, RenderAdapterError>,
                    > + Send
                    + 'a,
            >,
        > {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.render(artifact, config)
        }
        fn name(&self) -> &'static str {
            "counting"
        }
    }
    let calls = Arc::new(AtomicU32::new(0));
    let registry = {
        let mut r = AdapterRegistry::new();
        r.register_unique(
            OutputType::UiScreenshot,
            Box::new(CountingAdapter {
                inner: UiScreenshotAdapter::new(),
                calls: calls.clone(),
            }),
        )
        .unwrap();
        r
    };

    // Build a real validation request with visual_check=false. We use
    // an .ico file path so Tier1 reports pass=false (file missing) and
    // the existing veto-table terminates the validation at Tier1 — the
    // visual-feedback loop is gated OUT regardless of Tier1 verdict.
    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "ui.png");
    let req = elai_orchestrator::media_validator::types::MediaValidationRequest {
        media_type: elai_orchestrator::media_validator::types::MediaType::UiScreenshot,
        file_path: path,
        quality_spec: QualitySpec {
            similarity_threshold: None,
            qualitative_checks: false,
            visual_check: false,
        },
        reference_path: None,
        mode: elai_orchestrator::media_validator::types::ValidationMode::Quick,
        output_dir: None,
        request_id: uuid::Uuid::nil(),
    };
    let mock = elai_orchestrator::media_validator::MockVlmBackend::new();
    let res = elai_orchestrator::media_validator::pipeline::validate_with_rce(
        &req,
        &mock,
        None,
        registry,
        Some(&OutputType::UiScreenshot),
        &RenderConfig::default(),
    )
    .await;
    assert!(res.is_ok(), "pipeline must succeed with visual_check=false");

    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "spec L449: no adapter invocation when visual_check is false"
    );
}

// ---------------------------------------------------------------------------
// rce_c3: visual_check=true + output_type=None ⇒ rejected at validation.
// ---------------------------------------------------------------------------

/// Spec L450: missing output_type is a validation error, not a runtime
/// error. We pin this at the registry level: an unregistered output
/// type returns `None` from `get_adapter`, and `VisualFeedbackLoop::run`
/// surfaces the missing-adapter case as an immediate error (no Render
/// attempt).
#[tokio::test]
async fn rce_c3_no_output_type_rejected_at_validation() {
    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "a.png");
    let loop_ = VisualFeedbackLoop::new(AdapterRegistry::new(), QualitySpec::default());
    let res = loop_
        .run(
            &path,
            &OutputType::Web, // no adapter for Web in empty registry
            &RenderConfig::default(),
            |_a, _f| async move { (tier1_pass(), None, vec![]) },
            |_| {},
        )
        .await;
    // Empty registry -> missing adapter -> error BEFORE any iteration.
    assert!(
        res.is_err(),
        "missing adapter must error before runtime loop start"
    );
}

// ---------------------------------------------------------------------------
// rce_c4: Max iterations enforced.
// ---------------------------------------------------------------------------

/// Spec L451: default max_iterations = 3. With an executor that NEVER
/// resolves the issue AND issues whose convergence key drifts every
/// iteration, the loop MUST stop at iteration 3.
#[tokio::test]
async fn rce_c4_max_iterations_enforced() {
    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "a.png");
    let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
    let mut iter_n = 0u32;
    let res: VisualFeedbackResult = loop_
        .run(
            &path,
            &OutputType::UiScreenshot,
            &RenderConfig::default(),
            |_a, _f| {
                iter_n += 1;
                let issue = VisualIssue {
                    description: format!("drifting issue #{}", iter_n),
                    severity: Severity::Major,
                    location: Some(BoundingBox {
                        x: 0.05 * iter_n as f32,
                        y: 0.5,
                        width: 0.1,
                        height: 0.1,
                    }),
                    viewport: None,
                    actionable_fix: "increase max-width".into(),
                };
                let r = (tier1_pass(), None, vec![issue]);
                async move { r }
            },
            |_| {},
        )
        .await
        .unwrap();
    assert_eq!(res.iterations, DEFAULT_MAX_ITERATIONS);
    assert_eq!(res.iterations, 3, "spec L451 mandates default = 3");
    assert!(!res.converged, "drift prevents convergence");
}

// ---------------------------------------------------------------------------
// rce_c5: Convergence detection by category, not raw string.
// ---------------------------------------------------------------------------

/// Spec L488-490: convergence compares by (severity, category, location),
/// NOT raw description string. Issues with the same key but
/// rephrased descriptions MUST converge.
#[tokio::test]
async fn rce_c5_convergence_detection_by_category_not_string() {
    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "a.png");
    let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
    let mut call = 0u32;
    let res = loop_
        .run(
            &path,
            &OutputType::UiScreenshot,
            &RenderConfig::default(),
            |_a, _f| {
                call += 1;
                // Convergence key uses the first token before `:` or `,` as
                // the category. By keeping the prefix identical we exercise
                // the "wording drift below the category boundary" case
                // mandated by spec L488-490.
                let i = VisualIssue {
                    description: format!("button_overflow: rephrasing attempt #{}", call),
                    severity: Severity::Major,
                    location: Some(BoundingBox {
                        x: 0.1,
                        y: 0.2,
                        width: 0.1,
                        height: 0.1,
                    }),
                    viewport: None,
                    actionable_fix: "increase max-width".into(),
                };
                let r = (tier1_pass(), None, vec![i]);
                async move { r }
            },
            |_| {},
        )
        .await
        .unwrap();
    assert!(
        res.converged,
        "convergence must use (severity, category, location) — NOT description string"
    );
    assert_eq!(res.iterations, 2, "stable on iteration 2");
}

// ---------------------------------------------------------------------------
// rce_c6: Tier 1 fail stops loop immediately.
// ---------------------------------------------------------------------------

/// Spec L454: Tier 1 fail in the feedback loop = loop stops at
/// iteration 1 (same veto as base validation). Verdict is Fail.
#[tokio::test]
async fn rce_c6_tier1_fail_stops_loop_immediately() {
    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "a.png");
    let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
    let mut call_count = 0u32;
    let res = loop_
        .run(
            &path,
            &OutputType::UiScreenshot,
            &RenderConfig::default(),
            |_a, _f| {
                call_count += 1;
                async move { (tier1_fail(), None, vec![]) }
            },
            |_| {},
        )
        .await
        .unwrap();
    assert_eq!(res.iterations, 1);
    assert_eq!(call_count, 1, "evaluator called exactly once");
    assert!(matches!(res.final_verdict, Verdict::Fail));
}

// ---------------------------------------------------------------------------
// rce_c7: Feedback structure is actionable.
// ---------------------------------------------------------------------------

/// Spec L455: every emitted VisualIssue MUST have a non-empty
/// `actionable_fix`. We synthesize a typical Tier-2 issue set and
/// assert that none of them are emitted with an empty fix string.
#[tokio::test]
async fn rce_c7_feedback_structure_actionable() {
    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "a.png");
    let loop_ = VisualFeedbackLoop::new(registry_with_ui(), QualitySpec::default());
    let res = loop_
        .run(
            &path,
            &OutputType::UiScreenshot,
            &RenderConfig::default(),
            |_a, _f| {
                let issues = vec![
                    VisualIssue {
                        description: "button: text overflow at mobile".into(),
                        severity: Severity::Major,
                        location: None,
                        viewport: Some(Viewport::mobile()),
                        actionable_fix: "set max-width: 90% on .btn-primary".into(),
                    },
                    VisualIssue {
                        description: "color: insufficient contrast".into(),
                        severity: Severity::Minor,
                        location: None,
                        viewport: None,
                        actionable_fix: "raise --fg from #888 to #555".into(),
                    },
                ];
                let r = (tier1_pass(), None, issues);
                async move { r }
            },
            |_| {},
        )
        .await
        .unwrap();
    // Aggregate issues across all iterations.
    let all_issues: Vec<&VisualIssue> = res
        .iteration_history
        .iter()
        .flat_map(|r: &IterationRecord| r.issues.iter())
        .collect();
    assert!(!all_issues.is_empty(), "should have synthesized issues");
    for issue in all_issues {
        assert!(
            !issue.actionable_fix.trim().is_empty(),
            "every VisualIssue must have non-empty actionable_fix; got {issue:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Bonus: adapter registry duplicate rejection (spec adversarial L428-429).
// ---------------------------------------------------------------------------

#[test]
fn rce_registry_rejects_duplicate_registration() {
    let mut r = AdapterRegistry::new();
    r.register_unique(OutputType::Web, Box::new(WebRenderAdapter::new()))
        .unwrap();
    let err = r
        .register_unique(OutputType::Web, Box::new(WebRenderAdapter::new()))
        .expect_err("duplicate must error");
    match err {
        RenderAdapterError::DuplicateAdapter(OutputType::Web) => {}
        other => panic!("expected DuplicateAdapter(Web), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Bonus: from_capabilities respects bitflags (spec L406).
// ---------------------------------------------------------------------------

#[test]
fn rce_registry_from_capabilities_filters_by_flags() {
    // Only BLENDER available -> Model3D registered, Web/Animation absent.
    let caps = ToolCapabilityFlags::BLENDER;
    let r = AdapterRegistry::from_capabilities(&caps);
    assert!(r.get_adapter(&OutputType::Model3D).is_some());
    assert!(r.get_adapter(&OutputType::Web).is_none());
    assert!(r.get_adapter(&OutputType::Animation).is_none());
    // UiScreenshot is always present (no external tool).
    assert!(r.get_adapter(&OutputType::UiScreenshot).is_some());
}

// ---------------------------------------------------------------------------
// rce_c8: validate_with_rce propagates the loop verdict.
// ---------------------------------------------------------------------------

/// Audit BLOCK 2 (decision 71): the previous body of `validate_with_rce`
/// discarded the loop result (`let _result = ...; Ok(base)`) so a loop
/// that surfaced Critical-severity Tier2 issues still returned
/// `Verdict::Pass`. This test pins the new behavior: a request whose
/// loop will encounter Critical issues must yield `HumanReview` (or
/// `Fail`), NOT `Pass`.
///
/// We synthesize the Critical-issue path by configuring a custom
/// MockVlmBackend whose `validate_advisory` returns one Critical-severity
/// Issue. The base validate_with_sink path returns Pass (Tier1 passes
/// on the synthetic PNG, qualitative_checks gate is on, MockVlm
/// returns advisory_pass=true for the base call), so the loop fires
/// and surfaces the Critical issue.
#[tokio::test]
async fn rce_c8_validate_with_rce_propagates_loop_verdict() {
    use elai_orchestrator::media_validator::types::{
        Issue, MediaType, MediaValidationRequest, ValidationMode, Verdict, VlmValidationOutput,
    };
    use elai_orchestrator::media_validator::VlmBackend;

    // VLM that always returns one Critical-severity issue. Used in the
    // loop's evaluator to force the rce_c8 branch (`critical_iter`).
    struct CriticalVlm;
    impl VlmBackend for CriticalVlm {
        fn validate_advisory(&self, _path: &Path) -> VlmValidationOutput {
            VlmValidationOutput {
                overall_quality: 0.9,
                confidence: 0.9,
                issues: vec![Issue {
                    category: "rendering".into(),
                    severity: Severity::Critical,
                    description: "critical: corrupted output".into(),
                    location: None,
                }],
                reasoning: "synthetic critical issue for rce_c8".into(),
            }
        }
        fn model_id(&self) -> &str {
            "mock-critical"
        }
    }

    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "ui.png");
    let req = MediaValidationRequest {
        media_type: MediaType::UiScreenshot,
        file_path: path,
        quality_spec: QualitySpec {
            similarity_threshold: None,
            qualitative_checks: true,
            visual_check: true,
        },
        reference_path: None,
        mode: ValidationMode::Full,
        output_dir: None,
        request_id: uuid::Uuid::nil(),
    };
    let registry = registry_with_ui();
    let backend = CriticalVlm;
    let res = elai_orchestrator::media_validator::pipeline::validate_with_rce(
        &req,
        &backend,
        None,
        registry,
        Some(&OutputType::UiScreenshot),
        &RenderConfig::default(),
    )
    .await;
    let result = res.expect("validate_with_rce must succeed");
    // Spec audit BLOCK 2 fix: loop verdict surfaces; NOT silently Pass.
    assert!(
        matches!(
            result.final_verdict,
            Verdict::Fail | Verdict::HumanReview | Verdict::Degraded
        ),
        "expected Fail/HumanReview/Degraded, got {:?} (reason: {})",
        result.final_verdict,
        result.verdict_reason
    );
}

// ---------------------------------------------------------------------------
// Bonus: validate_with_rce smoke test inside #[tokio::test] — verifies
// no nested-runtime panic (audit BLOCK 3 — decision 72).
// ---------------------------------------------------------------------------

/// Spec audit BLOCK 3: previously `validate_with_rce` built an internal
/// `current_thread` runtime AND the inner `run_tier1_for_frame` helper
/// built ANOTHER, panicking with "Cannot start a runtime from within a
/// runtime" the moment any caller drove the function from an existing
/// runtime. This test runs the function with `visual_check=true` from a
/// `#[tokio::test]` context; success here proves no nested runtime.
#[tokio::test]
async fn rce_c8b_no_nested_runtime_panic_under_tokio_test() {
    use elai_orchestrator::media_validator::types::{
        MediaType, MediaValidationRequest, ValidationMode,
    };
    let dir = TempDir::new().unwrap();
    let path = write_png(dir.path(), "smoke.png");
    let req = MediaValidationRequest {
        media_type: MediaType::UiScreenshot,
        file_path: path,
        quality_spec: QualitySpec {
            similarity_threshold: None,
            qualitative_checks: false,
            visual_check: true,
        },
        reference_path: None,
        mode: ValidationMode::Quick,
        output_dir: None,
        request_id: uuid::Uuid::nil(),
    };
    let mock = elai_orchestrator::media_validator::MockVlmBackend::new();
    // Must NOT panic. Even on Pass/Fail/HumanReview branch, the call
    // must return cleanly.
    let _ = elai_orchestrator::media_validator::pipeline::validate_with_rce(
        &req,
        &mock,
        None,
        registry_with_ui(),
        Some(&OutputType::UiScreenshot),
        &RenderConfig::default(),
    )
    .await;
}

// Suppress unused-import warnings when test feature flags toggle.
#[allow(dead_code)]
fn _silence_unused(_: &BlenderRenderAdapter) {}
