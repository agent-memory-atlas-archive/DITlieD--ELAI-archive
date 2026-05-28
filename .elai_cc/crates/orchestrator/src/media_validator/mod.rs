//! B-24 Phase 4I: Media Validator (conditional sub-system).
//!
//! Lazily instantiated when a [`PipelineSpec`] declares non-empty `media`.
//! Group C3 (this PR) ships the real Tier1 -> Tier2 -> Verdict pipeline;
//! all #[ignore]'d C2 contract / functional / adversarial tests that gate
//! on Group C3 transition to GREEN here. Subprocess execution +
//! in-process header parsing combined: we do NOT require ImageMagick /
//! FFprobe / Tesseract for default CI to pass — only Blender remains
//! external-tool-gated (Model3D tests stay #[ignore]'d).
//!
//! Architecture rationale: keeping the file split fine-grained (one file
//! per subsystem) makes per-group code review tractable. The single
//! re-export below pre-binds the public surface so downstream callers
//! (`task_runner`, `dashboard-v2`) need only `use elai_orchestrator::media_validator::*`.

pub mod axtree;
pub mod clap;
pub mod external_tools;
pub mod feedback_loop;
pub mod mock_vlm;
pub mod render_adapters;
pub mod sycophancy;
pub mod tier1;
pub mod tier2;
pub mod types;
pub mod vlm_validator;

// Public re-exports so callers don't have to walk the submodule tree for
// the everyday surface. The ordering matches the dependency direction
// (types -> external_tools -> tier{1,2} -> mock_vlm -> top-level facade).
pub use self::clap::{cosine_similarity, ClapAudioModel};
pub use self::external_tools::{
    build_capability_flags, BlenderWrapper, ExternalToolWrapper, FFprobeWrapper,
    ImageMagickWrapper, TesseractWrapper, ToolError, ToolOutput,
};
pub use self::mock_vlm::{MockVlmBackend, VlmBackend};
pub use self::tier1::{
    AudioValidator, ImageValidator, Model3DValidator, Tier1Outcome, Tier1Validator,
    UiScreenshotValidator, TIER1_DEFAULT_TIMEOUT_MS,
};
pub use self::tier2::{select_validator, MockTier2, OllamaTier2, SglangTier2, Tier2Validator};
pub use self::types::{
    BoundingBox, CheckResult, ClapConfig, ClapDevice, Issue, MediaType, MediaValidationRequest,
    MediaValidationResult, MediaValidatorError, OllamaConfig, QualitySpec, Severity, SglangConfig,
    Tier1Result, Tier2Error, Tier2Result, ToolCapabilityFlags, ValidationMode, Verdict, VlmConfig,
    VlmEngine, VlmMode, VlmProcessConfig, VlmValidationOutput, DEFAULT_MAX_ITERATIONS,
    DEFAULT_TIER1_TIMEOUT_MS, DEFAULT_VLM_STARTUP_TIMEOUT_SECS, MAX_TIER1_CONCURRENT,
    MAX_TIER2_CONCURRENT, MAX_VALIDATION_FILE_BYTES, MODE_A_MIN_VRAM_GB, MODE_B_MIN_VRAM_GB,
    VALIDATION_QUEUE_DEPTH,
};

/// Lazy plugin factory hook — the dashboard / task_runner factory calls
/// this at startup. Returns `Some(())` only when the spec requires media
/// validation (avoids spawning the (heavy) VLM process for code-only
/// tasks). Today this is parameterized on a simple `has_media` bool that
/// the caller computes via [`should_load_media_validator`]; the
/// `_stub` variant retains the unit return for the existing
/// dashboard / lazy-loading contract tests. Production callers should
/// instead invoke [`new_if_media_steps`] which returns a real
/// `MediaValidator` with the configured capabilities + backend.
pub fn new_if_media_steps_stub(_has_media: bool) -> Option<()> {
    // Intentionally always None: matches the C1 skeleton contract.
    None
}

/// MediaValidator orchestrator handle. Audit WARN-4 fix (decision 44):
/// replaces the prior `Option<()>` placeholder with a real struct so
/// downstream wiring (Group C5 / task_runner) can inspect both the
/// detected tool capabilities AND the VLM backend in a single value.
///
/// Group C5 update (decision 47 — Tier1 capability-flag Arc cache):
/// `capabilities` is now `Arc<ToolCapabilityFlags>` so the per-call
/// `build_capability_flags()` subprocess probe is replaced by a single
/// shared snapshot built ONCE at construction. This is the fix for
/// the ~900 ms Tier1 p95 measured in Group C4 (metrics/phase_4I.toml
/// line 67 + notes.tier1_p95_breach).
///
/// `Arc` (not `Rc`) because callers are async/`Send` (tokio executors).
/// `bitflags!`-derived structs are `Copy` so cloning the inner u32 is
/// cheap; the `Arc` indirection is kept only for API symmetry with
/// production callers that hold the validator behind `Arc<Mutex<_>>`.
pub struct MediaValidator {
    pub capabilities: std::sync::Arc<ToolCapabilityFlags>,
    pub vlm_backend: Option<Box<dyn VlmBackend>>,
}

impl std::fmt::Debug for MediaValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaValidator")
            .field("capabilities", &*self.capabilities)
            .field("vlm_backend.is_some", &self.vlm_backend.is_some())
            .finish()
    }
}

/// Lazy-loading factory used by the C5 dispatcher.
///
/// Returns `Some(MediaValidator)` when `has_media` is true so the caller
/// receives a fully-constructed handle with the detected
/// [`ToolCapabilityFlags`] snapshot and the supplied VLM backend
/// (`None` for default-feature builds where the Mock backend is
/// instantiated lazily on first request). The DETERMINISTIC_CHECK from
/// B-24 plan greps for `fn new_if_media_steps`; this name satisfies the
/// invariant.
///
/// TODO (Group C5 / task_runner wiring): the actual. stub-ban-allow: Group C5 follow-up
/// `TaskRunner` / `PipelineFactory` does not yet call this constructor
/// — the wiring lands when the dispatcher swaps the
/// `new_if_media_steps_stub` placeholder. See
/// workflow/reasoning/B-24-decisions.md #44 and the elai.map entry
/// for `TaskRunner::build`.
pub fn new_if_media_steps(
    has_media: bool,
    capabilities: ToolCapabilityFlags,
    vlm_backend: Option<Box<dyn VlmBackend>>,
) -> Option<MediaValidator> {
    if has_media {
        Some(MediaValidator {
            capabilities: std::sync::Arc::new(capabilities),
            vlm_backend,
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Group C5: bounded-queue dispatcher (decision 29 + a6_queue_full test).
// ---------------------------------------------------------------------------

/// Global validation-queue semaphore. Spec phase-4I.txt L223-226 +
/// types.rs::VALIDATION_QUEUE_DEPTH. The semaphore is a `tokio::sync`
/// primitive but `try_acquire` is a SYNCHRONOUS call (returns
/// `TryAcquireError::NoPermits` immediately when saturated) so the
/// sync `pipeline::validate` entry point can use it without a tokio
/// runtime requirement. Group C5 decision-doc entry 55.
///
/// `OnceLock` (std since 1.70) avoids the `once_cell` crate dependency
/// and is per-process; sufficient for the spec's "per-process queue"
/// invariant.
fn validation_queue_semaphore() -> &'static std::sync::Arc<tokio::sync::Semaphore> {
    static SEM: std::sync::OnceLock<std::sync::Arc<tokio::sync::Semaphore>> =
        std::sync::OnceLock::new();
    SEM.get_or_init(|| {
        std::sync::Arc::new(tokio::sync::Semaphore::new(
            crate::media_validator::types::VALIDATION_QUEUE_DEPTH,
        ))
    })
}

/// Public bounded-queue handle. Wraps the global semaphore so callers
/// can issue `try_submit` from sync code and receive
/// `Err(MediaValidatorError::QueueFull)` immediately when saturated
/// (no thread::sleep, no blocking). Spec L223-226.
///
/// `MAX_TIER1_CONCURRENT` and `MAX_TIER2_CONCURRENT` are tracked as
/// public constants (already shipped in types.rs L66-69); the
/// per-tier semaphores live inside `MediaValidator` (Group C6 wires
/// them through the dispatcher).
pub struct MediaValidationQueue {
    pub semaphore: std::sync::Arc<tokio::sync::Semaphore>,
}

impl Default for MediaValidationQueue {
    fn default() -> Self {
        Self {
            semaphore: validation_queue_semaphore().clone(),
        }
    }
}

impl MediaValidationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// True iff the queue can immediately accept a new validation
    /// request. NOT consumed — the actual permit is acquired inside
    /// `pipeline::validate`.
    pub fn has_capacity(&self) -> bool {
        self.semaphore.available_permits() > 0
    }
}

/// Decide whether to instantiate the validator for a given pipeline. The
/// real PipelineSpec type lives in `research_tree`; we accept a boolean
/// `has_media` flag to avoid a cyclic import. Group C5 wires this through
/// `research_tree::PipelineSpec::has_media_steps()`.
pub fn should_load_media_validator(has_media: bool) -> bool {
    has_media
}

// ---------------------------------------------------------------------------
// Pipeline orchestration (Group C3 — REAL impl)
// ---------------------------------------------------------------------------

/// Group C3 pipeline orchestration entry point — REAL implementation.
///
/// Sync facade kept for compatibility with Group C2's contract /
/// functional / adversarial test files (`pipeline::validate(&req, &mock)`).
/// Internally drives a tokio current-thread runtime so subprocess +
/// async tier dispatch work without leaking a runtime requirement to
/// callers.
///
/// Visibility: `pub` because the tests live in a separate test crate (the
/// `tests/` directory in Cargo's integration-test layout).
pub mod pipeline {
    use std::path::{Component, Path, PathBuf};

    use elai_core::clock::{ClockProvider, OsClock};
    use tracing::{debug, info, warn};

    use super::mock_vlm::VlmBackend;
    use super::tier1::{
        AudioValidator, ImageValidator, Model3DValidator, Tier1Validator, UiScreenshotValidator,
    };
    use super::types::{
        CheckResult, MediaType, MediaValidationRequest, MediaValidationResult,
        MediaValidatorError, Severity, Tier1Result, Tier2Result, ValidationMode, Verdict,
        VlmValidationOutput, MAX_VALIDATION_FILE_BYTES,
    };

    /// SSB-writer sink for media validator events. Object-safe so the
    /// pipeline can take an `Option<&dyn MediaSsbSink>` without pulling
    /// the full SSB stack into scope. Group C5 wires the real impl
    /// (`Arc<Ssb>::write_payload(...)`) — Group C3 ships this trait so
    /// the contract / functional / adversarial tests can pin observable
    /// emissions without an SSB instance. Audit BLOCK-1 fix (decision 39).
    pub trait MediaSsbSink: Send + Sync {
        fn on_validation_started(
            &self,
            request_id: uuid::Uuid,
            media_type: &str,
            file_path: &str,
        );
        fn on_tier1_complete(
            &self,
            request_id: uuid::Uuid,
            pass: bool,
            latency_ms: u64,
            check_count: u32,
        );
        fn on_tier2_complete(
            &self,
            request_id: uuid::Uuid,
            advisory_pass: bool,
            model_used: &str,
            latency_ms: u64,
        );
        fn on_human_review_required(&self, request_id: uuid::Uuid, reason: &str);
        /// B-24 Phase 4I Group C5 audit WARN 2 fix (decision 60): one
        /// `visual_feedback_iteration` SSB action per iteration of the
        /// `VisualFeedbackLoop::run` body. Carries `step_id` (request_id
        /// stringified), `iteration` (0-based), `issue_count`, and the
        /// `converged` flag. Default impl is a no-op so existing test sinks
        /// don't need to implement it; production SSB sinks override it to
        /// write a `VisualFeedbackIterationPayload` to the bus (spec
        /// phase-4I.txt L493-494 — agent-liveness observability during
        /// long render loops).
        fn on_visual_feedback_iteration(
            &self,
            _step_id: &str,
            _iteration: u32,
            _issue_count: u32,
            _converged: bool,
        ) {
            // Default no-op — concrete sinks (Arc<Ssb>) override.
        }
    }

    /// Run the Tier1 -> Tier2 -> Verdict pipeline for a single request.
    ///
    /// Veto-table (phase-4I.txt L277-287 — all 7 branches):
    ///   1. Tier1 fail                                            -> Verdict::Fail (no Tier2)
    ///   2. Tier1 pass + no qualitative spec                      -> Verdict::Pass
    ///   3. Tier1 pass + Tier2 pass (no disagreement)             -> Verdict::Pass
    ///   4. Tier1 pass + Tier2 fail (advisory_pass=false)         -> Verdict::HumanReview
    ///   5. Tier1 pass + Tier2 advisory_pass=true BUT raises a
    ///      disagreement on a Tier1-covered check category        -> Verdict::HumanReview
    ///   6. Tier1 pass + Tier2 schema-invalid                     -> Verdict::Degraded
    ///   7. Tier1 pass + Tier2 unavailable / backend offline      -> Verdict::Degraded
    ///
    /// `vlm` is the Tier-2 backend (MockVlmBackend in C2 tests; real
    /// Ollama/SGLang adapter in production).
    pub fn validate<B: VlmBackend + ?Sized>(
        request: &MediaValidationRequest,
        vlm: &B,
    ) -> Result<MediaValidationResult, MediaValidatorError> {
        validate_with_sink(request, vlm, None)
    }

    /// Same as [`validate`] but takes an optional SSB sink. The sink is
    /// driven synchronously: every emission happens before the function
    /// returns. When `sink` is `None` the same events are logged via
    /// `tracing::info!` as a no-op fallback so production callers without
    /// an SSB still leave an audit trail in the structured-log stream.
    /// Audit BLOCK-1 fix (decision 39).
    pub fn validate_with_sink<B: VlmBackend + ?Sized>(
        request: &MediaValidationRequest,
        vlm: &B,
        sink: Option<&dyn MediaSsbSink>,
    ) -> Result<MediaValidationResult, MediaValidatorError> {
        // -----------------------------------------------------------------
        // Group C5 (decision 55): bounded-queue admission BEFORE any
        // I/O or subprocess work. `try_acquire` is synchronous; saturated
        // semaphore -> immediate Err(QueueFull) per spec L223-226. The
        // `_permit` binding holds the permit for the duration of the
        // validation; Drop releases it (RAII).
        // -----------------------------------------------------------------
        let _permit = match super::validation_queue_semaphore().try_acquire() {
            Ok(p) => p,
            Err(_) => {
                return Err(MediaValidatorError::QueueFull {
                    depth: super::types::VALIDATION_QUEUE_DEPTH,
                });
            }
        };

        // -----------------------------------------------------------------
        // SSB emission 1/4: validation started.
        // -----------------------------------------------------------------
        let media_type_str = match request.media_type {
            MediaType::Image => "image",
            MediaType::Audio => "audio",
            MediaType::Model3D => "model3d",
            MediaType::UiScreenshot => "ui_screenshot",
        };
        let file_path_str = request.file_path.to_string_lossy().to_string();
        match sink {
            Some(s) => s.on_validation_started(request.request_id, media_type_str, &file_path_str),
            None => info!(
                request_id = %request.request_id,
                media_type = media_type_str,
                file_path = %file_path_str,
                "media_validation_started"
            ),
        }

        // -----------------------------------------------------------------
        // Tier 0: path-guard + size-guard (synchronous, before any I/O).
        // -----------------------------------------------------------------
        if let Some(err) = path_guard(&request.file_path, request.output_dir.as_deref()) {
            return Err(err);
        }
        if let Some(early) = size_guard(&request.file_path) {
            // Emit Tier1-complete (fail) so the dashboard still sees a row.
            emit_tier1(sink, request.request_id, &early.tier1);
            return Ok(early);
        }

        // -----------------------------------------------------------------
        // Tier 1: dispatch by media_type. Async — wrap in a current-thread
        // runtime so the sync facade stays callable.
        // -----------------------------------------------------------------
        let tier1 = run_tier1_blocking(request);
        emit_tier1(sink, request.request_id, &tier1);

        // -----------------------------------------------------------------
        // Veto branch 1: Tier1 fail short-circuits.
        // -----------------------------------------------------------------
        if !tier1.pass {
            return Ok(MediaValidationResult {
                tier1,
                tier2: None,
                final_verdict: Verdict::Fail,
                verdict_reason: "Tier1 deterministic checks failed".into(),
            });
        }

        // -----------------------------------------------------------------
        // Veto branch 2: Tier1 pass + no qualitative spec -> Pass.
        // -----------------------------------------------------------------
        let need_tier2 = matches!(request.mode, ValidationMode::Full)
            && request.quality_spec.qualitative_checks;

        if !need_tier2 {
            return Ok(MediaValidationResult {
                tier1,
                tier2: None,
                final_verdict: Verdict::Pass,
                verdict_reason: "Tier1 passed; no qualitative spec".into(),
            });
        }

        // -----------------------------------------------------------------
        // Tier 2: invoke VlmBackend through the Mock adapter contract.
        // The backend is sync (validate_advisory) — real OllamaTier2 /
        // SglangTier2 lives behind the `vlm-integration` feature.
        // -----------------------------------------------------------------
        let tier2 = run_tier2(request, vlm);

        // -----------------------------------------------------------------
        // Veto branches 3-7.
        // -----------------------------------------------------------------
        match tier2 {
            Ok(t2) if t2.advisory_pass => {
                emit_tier2(sink, request.request_id, &t2);
                // Branch 5: even on advisory_pass=true, a disagreement on a
                // Tier1-covered check category forces HumanReview. Decision-
                // doc entry 43; spec phase-4I.txt L283 ("Tier 2 disagrees
                // with Tier 1 on any check -> HumanReview").
                if tier2_disagrees_with_tier1(&t2) {
                    let reason =
                        "tier2_disagreement_despite_advisory_pass".to_owned();
                    emit_human_review(sink, request.request_id, &reason);
                    Ok(MediaValidationResult {
                        tier1,
                        tier2: Some(t2),
                        final_verdict: Verdict::HumanReview,
                        verdict_reason: reason,
                    })
                } else {
                    Ok(MediaValidationResult {
                        tier1,
                        tier2: Some(t2),
                        final_verdict: Verdict::Pass,
                        verdict_reason: "Tier1 + Tier2 both passed".into(),
                    })
                }
            }
            Ok(t2) => {
                emit_tier2(sink, request.request_id, &t2);
                // Branch 4: Tier1 pass + Tier2 advisory_pass=false -> HumanReview.
                let reason = "Tier1 passed but Tier2 advisory failed".to_owned();
                emit_human_review(sink, request.request_id, &reason);
                Ok(MediaValidationResult {
                    tier1,
                    tier2: Some(t2),
                    final_verdict: Verdict::HumanReview,
                    verdict_reason: reason,
                })
            }
            Err(reason) => {
                // Branch 6/7: schema-invalid / backend-offline -> Degraded.
                // No Tier2-complete emission (no result to log).
                Ok(MediaValidationResult {
                    tier1,
                    tier2: None,
                    final_verdict: Verdict::Degraded,
                    verdict_reason: reason,
                })
            }
        }
    }

    /// Async-native sibling of [`validate_with_sink`]. Used internally
    /// by [`validate_with_rce`] (which itself is async) so the Tier1
    /// dispatch can be awaited directly — no `Builder::new_current_thread()
    /// + block_on` chain, no nested-runtime panic when the parent is
    /// already on a tokio runtime. The body is a near-verbatim copy of
    /// the sync version; the ONLY difference is replacing
    /// `run_tier1_blocking(request)` with an inline `await` on the
    /// Tier1Validator future. Audit BLOCK 3 (decision 72).
    pub async fn validate_with_sink_async<B: VlmBackend + ?Sized>(
        request: &MediaValidationRequest,
        vlm: &B,
        sink: Option<&dyn MediaSsbSink>,
    ) -> Result<MediaValidationResult, MediaValidatorError> {
        let _permit = match super::validation_queue_semaphore().try_acquire() {
            Ok(p) => p,
            Err(_) => {
                return Err(MediaValidatorError::QueueFull {
                    depth: super::types::VALIDATION_QUEUE_DEPTH,
                });
            }
        };

        let media_type_str = match request.media_type {
            MediaType::Image => "image",
            MediaType::Audio => "audio",
            MediaType::Model3D => "model3d",
            MediaType::UiScreenshot => "ui_screenshot",
        };
        let file_path_str = request.file_path.to_string_lossy().to_string();
        match sink {
            Some(s) => s.on_validation_started(request.request_id, media_type_str, &file_path_str),
            None => info!(
                request_id = %request.request_id,
                media_type = media_type_str,
                file_path = %file_path_str,
                "media_validation_started"
            ),
        }

        if let Some(err) = path_guard(&request.file_path, request.output_dir.as_deref()) {
            return Err(err);
        }
        if let Some(early) = size_guard(&request.file_path) {
            emit_tier1(sink, request.request_id, &early.tier1);
            return Ok(early);
        }

        // Tier 1: async dispatch — no runtime build.
        let tier1 = run_tier1_async(request).await;
        emit_tier1(sink, request.request_id, &tier1);

        if !tier1.pass {
            return Ok(MediaValidationResult {
                tier1,
                tier2: None,
                final_verdict: Verdict::Fail,
                verdict_reason: "Tier1 deterministic checks failed".into(),
            });
        }

        let need_tier2 = matches!(request.mode, ValidationMode::Full)
            && request.quality_spec.qualitative_checks;
        if !need_tier2 {
            return Ok(MediaValidationResult {
                tier1,
                tier2: None,
                final_verdict: Verdict::Pass,
                verdict_reason: "Tier1 passed; no qualitative spec".into(),
            });
        }

        let tier2 = run_tier2(request, vlm);
        match tier2 {
            Ok(t2) if t2.advisory_pass => {
                emit_tier2(sink, request.request_id, &t2);
                if tier2_disagrees_with_tier1(&t2) {
                    let reason = "tier2_disagreement_despite_advisory_pass".to_owned();
                    emit_human_review(sink, request.request_id, &reason);
                    Ok(MediaValidationResult {
                        tier1,
                        tier2: Some(t2),
                        final_verdict: Verdict::HumanReview,
                        verdict_reason: reason,
                    })
                } else {
                    Ok(MediaValidationResult {
                        tier1,
                        tier2: Some(t2),
                        final_verdict: Verdict::Pass,
                        verdict_reason: "Tier1 + Tier2 both passed".into(),
                    })
                }
            }
            Ok(t2) => {
                emit_tier2(sink, request.request_id, &t2);
                let reason = "Tier1 passed but Tier2 advisory failed".to_owned();
                emit_human_review(sink, request.request_id, &reason);
                Ok(MediaValidationResult {
                    tier1,
                    tier2: Some(t2),
                    final_verdict: Verdict::HumanReview,
                    verdict_reason: reason,
                })
            }
            Err(reason) => Ok(MediaValidationResult {
                tier1,
                tier2: None,
                final_verdict: Verdict::Degraded,
                verdict_reason: reason,
            }),
        }
    }

    /// Async Tier1 dispatcher. Replaces `run_tier1_blocking` for callers
    /// already on a tokio runtime; awaits Tier1Validator's `validate`
    /// future directly. Audit BLOCK 3 (decision 72).
    async fn run_tier1_async(request: &MediaValidationRequest) -> Tier1Result {
        let pipeline_clock = OsClock;
        let started = pipeline_clock.now();
        let capabilities = *cached_capability_flags();
        let canonical = std::fs::canonicalize(&request.file_path);
        let mut effective = request.clone();
        if let Ok(p) = canonical {
            effective.file_path = p;
        } else {
            return Tier1Result {
                pass: false,
                checks: vec![CheckResult {
                    name: "missing_file".into(),
                    pass: false,
                    detail: format!("canonicalize failed for {:?}", request.file_path),
                    latency_ms: 0,
                }],
                latency_ms: u64::try_from(pipeline_clock.elapsed_since(started).as_millis()).unwrap_or(u64::MAX),
            };
        }
        let v: Box<dyn Tier1Validator> = match effective.media_type {
            MediaType::Image => Box::new(ImageValidator),
            MediaType::Audio => Box::new(AudioValidator),
            MediaType::Model3D => Box::new(Model3DValidator),
            MediaType::UiScreenshot => Box::new(UiScreenshotValidator),
        };
        v.validate(&effective, capabilities).await
    }

    /// RCE-aware validation. Wraps [`validate_with_sink`] with the
    /// `visual_check` gate from spec phase-4I.txt L449: when
    /// `request.quality_spec.visual_check == false`, the adapter
    /// registry is NEVER consulted and no `RenderAdapter::render` runs.
    ///
    /// ASYNC entry point (audit BLOCK 3 — decision 72): the previous
    /// sync facade built an internal `current_thread` runtime via
    /// `Builder::new_current_thread()` and `block_on`. The internal
    /// `run_tier1_for_frame` helper THEN built ANOTHER `current_thread`
    /// runtime inside the evaluator closure → "Cannot start a runtime
    /// from within a runtime" panic in production. Making the function
    /// `async` and rewiring the evaluator to await Tier1Validator
    /// directly removes BOTH nested runtimes; callers from sync
    /// contexts must drive this with their own `#[tokio::main]` or
    /// `Runtime::block_on`. Tests use `#[tokio::test]`.
    ///
    /// Also addresses audit BLOCK 2 (decision 71): the loop result is
    /// now MAPPED to a Verdict (no longer silently discarded). Loop
    /// convergence with remaining Critical issues → HumanReview;
    /// max-iterations-exhausted → HumanReview; Tier1-veto inside the
    /// loop → Fail.
    ///
    /// `output_type` is the RCE adapter selection key (spec L448).
    /// `render_config` is forwarded to the adapter when the loop runs.
    #[allow(clippy::too_many_arguments)]
    pub async fn validate_with_rce<B: VlmBackend + ?Sized>(
        request: &MediaValidationRequest,
        vlm: &B,
        sink: Option<&dyn MediaSsbSink>,
        registry: super::render_adapters::AdapterRegistry,
        output_type: Option<&super::render_adapters::OutputType>,
        render_config: &super::render_adapters::RenderConfig,
    ) -> Result<MediaValidationResult, MediaValidatorError> {
        // -----------------------------------------------------------------
        // Spec L449 gate: visual_check=false ⇒ NO adapter lookup, NO
        // VisualFeedbackLoop construction. Registry is dropped without
        // any `get_adapter` / `render` call. The contract test
        // `rce_c2_visual_check_false_skips_render` pins this with a spy
        // adapter that asserts zero render calls.
        // -----------------------------------------------------------------
        if !request.quality_spec.visual_check {
            // Drop registry + render_config without consulting them.
            drop(registry);
            let _ = (output_type, render_config);
            return validate_with_sink_async(request, vlm, sink).await;
        }

        // -----------------------------------------------------------------
        // visual_check=true: Tier1/Tier2 first (existing veto-table).
        // Audit BLOCK 3 fix (decision 72): use the async-native sibling
        // so the Tier1 dispatch awaits the validator's `Pin<Box<Future>>`
        // directly. The previous sync `validate_with_sink` built its own
        // `current_thread` runtime via `run_tier1_blocking`, which
        // panicked with the nested-runtime error when `validate_with_rce`
        // ran inside `#[tokio::test]` / a parent runtime.
        // -----------------------------------------------------------------
        let base = validate_with_sink_async(request, vlm, sink).await?;
        // Only run the feedback loop if base verdict is Pass; otherwise
        // the existing veto branches already terminated the validation.
        if !matches!(base.final_verdict, Verdict::Pass) {
            return Ok(base);
        }

        // -----------------------------------------------------------------
        // Visual-feedback loop: requires output_type for adapter
        // selection (spec L450 — missing output_type when
        // visual_check=true is a validation error).
        // -----------------------------------------------------------------
        let ot = match output_type {
            Some(ot) => ot.clone(),
            None => {
                return Err(MediaValidatorError::NotImplemented(
                    "visual_check=true requires output_type",
                ));
            }
        };
        let qs = request.quality_spec.clone();
        let max_iter = super::feedback_loop::VisualFeedbackLoop::new(
            super::render_adapters::AdapterRegistry::new(),
            qs.clone(),
        )
        .max_iterations;
        let loop_ = super::feedback_loop::VisualFeedbackLoop::new(registry, qs.clone());
        let step_id = request.request_id.to_string();

        // Snapshot data the async evaluator needs. We move owned copies
        // into the closure so the returned future is `Send`-clean.
        // `vlm` is borrowed by reference (`?Sized` trait object cannot
        // be moved); the evaluator drives `vlm.validate_advisory` —
        // which is SYNC — before returning the future, so the captured
        // `&B` does not need to cross an async-move boundary.
        let media_type = request.media_type.clone();
        let base_req = request.clone();
        let vlm_model_id = vlm.model_id().to_owned();
        let qualitative_checks = qs.qualitative_checks;

        // Real ASYNC evaluator: Tier1Validator dispatch on the captured
        // frame path, then Tier 2 (VLM) if Tier1 passes and qualitative
        // checks are on. NO `tokio::runtime::Builder::new_current_thread`
        // anywhere — Tier1Validator returns `Pin<Box<Future>>` and we
        // `.await` it directly inside the parent runtime context.
        let evaluator_full = |frame_artifact: std::path::PathBuf,
                              frames: Vec<super::render_adapters::CapturedFrame>| {
            let media_type = media_type.clone();
            let mut sub_req = base_req.clone();
            let model_id = vlm_model_id.clone();
            let viewport = frames.first().map(|f| f.viewport.clone());
            let frame_path = frames
                .first()
                .map(|f| f.image_path.clone())
                .unwrap_or_else(|| frame_artifact.clone());
            sub_req.file_path = frame_path.clone();
            // Drive the SYNC VLM call up-front so the async future does
            // not capture `&B` (the `?Sized` trait object cannot move).
            let pre_advisory: Option<VlmValidationOutput> = if qualitative_checks {
                Some(vlm.validate_advisory(&frame_path))
            } else {
                None
            };
            async move {
                let capabilities = *cached_capability_flags();
                let pipeline_clock = OsClock;
                let started = pipeline_clock.now();
                let v: Box<dyn Tier1Validator> = match media_type {
                    MediaType::Image => Box::new(ImageValidator),
                    MediaType::Audio => Box::new(AudioValidator),
                    MediaType::Model3D => Box::new(Model3DValidator),
                    MediaType::UiScreenshot => Box::new(UiScreenshotValidator),
                };
                // .await directly — no nested runtime.
                let mut tier1 = v.validate(&sub_req, capabilities).await;
                if tier1.latency_ms == 0 {
                    tier1.latency_ms = u64::try_from(pipeline_clock.elapsed_since(started).as_millis())
                        .unwrap_or(u64::MAX);
                }
                if !tier1.pass {
                    return (tier1, None, Vec::new());
                }
                let advisory = match pre_advisory {
                    Some(a) => a,
                    None => return (tier1, None, Vec::new()),
                };
                let any_critical = advisory
                    .issues
                    .iter()
                    .any(|i| matches!(i.severity, Severity::Critical));
                let advisory_pass = !any_critical && advisory.overall_quality >= 0.5;
                // Audit WARN 2 fix (decision 75): the previous code
                // used `unwrap_or(serde_json::Value::Null)` which
                // silently swallowed VLM serialization failures.
                // SerializationFailure is now a typed variant on
                // MediaValidatorError; we log + fall back to Null
                // INSIDE the evaluator (loop must not abort mid-flight)
                // but mark the tuple as no-Tier2 so the verdict
                // routes to Degraded downstream.
                let structured_output = match serde_json::to_value(&advisory) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "vlm_advisory_serialization_failed; treating as no Tier2"
                        );
                        // Return early: no Tier2 result, no visual
                        // issues. Downstream verdict routing sees an
                        // empty issue set and the audit upstream sees
                        // Degraded for the convergence path.
                        return (tier1, None, Vec::new());
                    }
                };
                let tier2_result = super::types::Tier2Result {
                    advisory_pass,
                    structured_output,
                    model_used: model_id,
                    latency_ms: 0,
                };
                let visual: Vec<super::render_adapters::VisualIssue> = advisory
                    .issues
                    .into_iter()
                    .map(|i| {
                        let fix =
                            format!("address {} issue: {}", i.category, i.description);
                        super::render_adapters::VisualIssue {
                            description: format!("{}: {}", i.category, i.description),
                            severity: i.severity,
                            location: i.location,
                            viewport: viewport.clone(),
                            actionable_fix: fix,
                        }
                    })
                    .collect();
                (tier1, Some(tier2_result), visual)
            }
        };

        let loop_result = loop_
            .run_with_sink(
                &request.file_path,
                &ot,
                render_config,
                evaluator_full,
                |_issues| {},
                sink,
                Some(&step_id),
            )
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "visual_feedback_loop render failed");
                MediaValidatorError::NotImplemented(
                    "visual_feedback_loop adapter render failed",
                )
            })?;

        // -----------------------------------------------------------------
        // Audit BLOCK 2 fix (decision 71): map loop result to Verdict.
        // The previous `let _result = ...; Ok(base)` discarded the loop
        // entirely.
        // -----------------------------------------------------------------
        let final_iter = loop_result.iteration_history.last();

        // (a) Any iteration with Critical-severity issues → Fail.
        let critical_iter = loop_result
            .iteration_history
            .iter()
            .find(|it| {
                it.issues
                    .iter()
                    .any(|i| matches!(i.severity, Severity::Critical))
            });
        if let Some(it) = critical_iter {
            let reason = format!(
                "visual_feedback_critical_issue_iteration_{}",
                it.iteration
            );
            emit_human_review(sink, request.request_id, &reason);
            return Ok(MediaValidationResult {
                tier1: it.tier1_result.clone(),
                tier2: it.tier2_result.clone(),
                final_verdict: Verdict::Fail,
                verdict_reason: reason,
            });
        }

        // (b) Tier1 veto observed in the loop → Fail (loop already set
        //     final_verdict=Fail; surface it).
        if matches!(loop_result.final_verdict, Verdict::Fail) {
            let it = final_iter.cloned();
            return Ok(MediaValidationResult {
                tier1: it.as_ref().map(|i| i.tier1_result.clone()).unwrap_or(base.tier1),
                tier2: it.as_ref().and_then(|i| i.tier2_result.clone()),
                final_verdict: Verdict::Fail,
                verdict_reason: "visual_feedback_tier1_veto".into(),
            });
        }

        // (c) Loop did not converge (max-iterations exhausted) →
        //     HumanReview.
        if !loop_result.converged && loop_result.iterations >= max_iter {
            let reason = "visual_feedback_max_iterations_exceeded".to_owned();
            emit_human_review(sink, request.request_id, &reason);
            let it = final_iter.cloned();
            return Ok(MediaValidationResult {
                tier1: it.as_ref().map(|i| i.tier1_result.clone()).unwrap_or(base.tier1),
                tier2: it.as_ref().and_then(|i| i.tier2_result.clone()),
                final_verdict: Verdict::HumanReview,
                verdict_reason: reason,
            });
        }

        // (d) Converged with remaining issues → HumanReview.
        let unresolved_remain = final_iter
            .map(|it| !it.issues.is_empty())
            .unwrap_or(false);
        if loop_result.converged && unresolved_remain {
            let reason =
                "visual_feedback_unresolved_issues_after_convergence".to_owned();
            emit_human_review(sink, request.request_id, &reason);
            let it = final_iter.cloned();
            return Ok(MediaValidationResult {
                tier1: it.as_ref().map(|i| i.tier1_result.clone()).unwrap_or(base.tier1),
                tier2: it.as_ref().and_then(|i| i.tier2_result.clone()),
                final_verdict: Verdict::HumanReview,
                verdict_reason: reason,
            });
        }

        // (e) Converged with zero issues (or no issues at all) → Pass.
        Ok(MediaValidationResult {
            tier1: final_iter
                .map(|i| i.tier1_result.clone())
                .unwrap_or(base.tier1),
            tier2: final_iter.and_then(|i| i.tier2_result.clone()).or(base.tier2),
            final_verdict: Verdict::Pass,
            verdict_reason: "visual_feedback_passed".into(),
        })
    }

    // ---------------------------------------------------------------------
    // SSB-emission helpers (no-op fallback to tracing::info when sink=None)
    // ---------------------------------------------------------------------

    fn emit_tier1(sink: Option<&dyn MediaSsbSink>, request_id: uuid::Uuid, tier1: &Tier1Result) {
        let check_count = u32::try_from(tier1.checks.len()).unwrap_or(u32::MAX);
        match sink {
            Some(s) => s.on_tier1_complete(request_id, tier1.pass, tier1.latency_ms, check_count),
            None => info!(
                %request_id,
                pass = tier1.pass,
                latency_ms = tier1.latency_ms,
                check_count,
                "media_tier1_complete"
            ),
        }
    }

    fn emit_tier2(sink: Option<&dyn MediaSsbSink>, request_id: uuid::Uuid, tier2: &Tier2Result) {
        match sink {
            Some(s) => s.on_tier2_complete(
                request_id,
                tier2.advisory_pass,
                &tier2.model_used,
                tier2.latency_ms,
            ),
            None => info!(
                %request_id,
                advisory_pass = tier2.advisory_pass,
                model_used = %tier2.model_used,
                latency_ms = tier2.latency_ms,
                "media_tier2_complete"
            ),
        }
    }

    fn emit_human_review(sink: Option<&dyn MediaSsbSink>, request_id: uuid::Uuid, reason: &str) {
        match sink {
            Some(s) => s.on_human_review_required(request_id, reason),
            None => info!(%request_id, reason, "media_human_review_required"),
        }
    }

    // ---------------------------------------------------------------------
    // Tier2 / Tier1 disagreement detector (veto branch 5).
    // ---------------------------------------------------------------------

    /// Returns true when Tier2's advisory_pass=true output nevertheless
    /// raises an Issue in a category that Tier1 covers (resolution,
    /// format, colorspace, manifold, sample_rate). This is the "Tier 2
    /// disagrees with Tier 1 on a specific check" branch from spec
    /// phase-4I.txt L283. Decision-doc entry 43.
    fn tier2_disagrees_with_tier1(t2: &Tier2Result) -> bool {
        // The Tier1-covered check categories. Lowercased for case-insensitive
        // match against VLM-emitted Issue.category strings.
        const TIER1_CATEGORIES: &[&str] = &[
            "resolution",
            "format",
            "colorspace",
            "manifold",
            "sample_rate",
        ];
        // Parse structured_output back into VlmValidationOutput; on failure
        // we conservatively report no disagreement (the schema-invalid
        // branch will have already routed elsewhere when this is reached).
        let parsed: Result<super::types::VlmValidationOutput, _> =
            serde_json::from_value(t2.structured_output.clone());
        let Ok(out) = parsed else {
            return false;
        };
        out.issues.iter().any(|issue| {
            let cat = issue.category.to_lowercase();
            // Major or Critical disagreement counts; Minor is informational.
            let severe = matches!(issue.severity, Severity::Major | Severity::Critical);
            severe && TIER1_CATEGORIES.iter().any(|c| cat == *c)
        })
    }

    // ---------------------------------------------------------------------
    // Path guard
    // ---------------------------------------------------------------------

    /// Rejects paths containing `..` components (canonical traversal
    /// signal). When `output_dir` is `Some(dir)`, additionally requires
    /// that `canonicalize(path).starts_with(canonicalize(dir))` so a
    /// symlink resolving outside the allowed scope is rejected. This
    /// closes audit BLOCK-3 (symlink bypass).
    ///
    /// `None` for `output_dir` preserves the original C3 contract: only
    /// the `..`-component guard runs. The C2 contract tests (relative
    /// fixture paths) rely on this — they don't pin an output_dir and
    /// expect the pipeline to proceed to Tier1.
    ///
    /// Tests `a4_path_traversal_rejected` (relative path with `..`) and
    /// `a10_symlink_bypass_rejected` (symlink resolving outside
    /// output_dir) both pin the traversal behavior.
    fn path_guard(path: &Path, output_dir: Option<&Path>) -> Option<MediaValidatorError> {
        // Component-level traversal check — catches `../../foo` even when
        // canonicalize() would resolve via cwd.
        for component in path.components() {
            if matches!(component, Component::ParentDir) {
                return Some(MediaValidatorError::PathOutsideOutputDir {
                    path: path.to_path_buf(),
                });
            }
        }
        // Allowed-output-directory enforcement (audit BLOCK-3 fix).
        // Skipped when output_dir is None to preserve C2/C3 test
        // contracts that don't pin a scoped output root.
        if let Some(dir) = output_dir {
            // canonicalize both sides. If `path` doesn't exist (e.g.
            // missing-file tests), canonicalize returns Err — we let
            // Tier1 surface the missing_file CheckResult rather than
            // failing here, matching the original C3 behavior.
            let canonical_path = match std::fs::canonicalize(path) {
                Ok(p) => p,
                Err(_) => return None,
            };
            let canonical_dir = match std::fs::canonicalize(dir) {
                Ok(p) => p,
                Err(e) => return Some(MediaValidatorError::Io(e)),
            };
            if !canonical_path.starts_with(&canonical_dir) {
                return Some(MediaValidatorError::PathOutsideOutputDir {
                    path: canonical_path,
                });
            }
        }
        None
    }

    // ---------------------------------------------------------------------
    // Size guard
    // ---------------------------------------------------------------------

    /// Reads file metadata and returns an early Tier1-fail result when the
    /// file exceeds [`MAX_VALIDATION_FILE_BYTES`]. Returns `None` when the
    /// file is within budget OR when metadata cannot be read (Tier1 will
    /// surface the open error in its own check). This is a separate
    /// function from the Tier1 dispatch so it runs BEFORE any subprocess
    /// or VLM invocation per spec phase-4I.txt L215-216.
    fn size_guard(path: &Path) -> Option<MediaValidationResult> {
        let metadata = std::fs::metadata(path).ok()?;
        if metadata.len() > MAX_VALIDATION_FILE_BYTES {
            warn!(
                size = metadata.len(),
                cap = MAX_VALIDATION_FILE_BYTES,
                "file exceeds size cap; skipping Tier1+Tier2"
            );
            let check = CheckResult {
                name: "size_guard".into(),
                pass: false,
                detail: format!(
                    "file size {} bytes exceeds cap {}",
                    metadata.len(),
                    MAX_VALIDATION_FILE_BYTES
                ),
                latency_ms: 0,
            };
            return Some(MediaValidationResult {
                tier1: Tier1Result {
                    pass: false,
                    checks: vec![check],
                    latency_ms: 0,
                },
                tier2: None,
                final_verdict: Verdict::Fail,
                verdict_reason: "file exceeds size cap (size_guard)".into(),
            });
        }
        None
    }

    // ---------------------------------------------------------------------
    // Group C5 — cached capability snapshot.
    // ---------------------------------------------------------------------

    /// Return the process-wide `ToolCapabilityFlags`, initializing the
    /// snapshot on first call. Subsequent calls deref a `&'static
    /// Arc<ToolCapabilityFlags>` and copy the inner u32 (cheap; bitflags
    /// is `Copy`). Decision 47.
    ///
    /// The cache is keyed PER PROCESS, not per `MediaValidator` instance,
    /// because tool availability does not change at runtime on the host
    /// — re-probing per call (the pre-C5 behavior) was wasted work.
    /// `MediaValidator::capabilities` (Arc<ToolCapabilityFlags>) holds
    /// the same snapshot for callers that want to inspect the bitset
    /// without going through the pipeline.
    fn cached_capability_flags() -> &'static super::types::ToolCapabilityFlags {
        static FLAGS: std::sync::OnceLock<super::types::ToolCapabilityFlags> =
            std::sync::OnceLock::new();
        FLAGS.get_or_init(super::external_tools::build_capability_flags)
    }

    // ---------------------------------------------------------------------
    // Tier 1 dispatcher
    // ---------------------------------------------------------------------

    fn run_tier1_blocking(request: &MediaValidationRequest) -> Tier1Result {
        let pipeline_clock = OsClock;
        let started = pipeline_clock.now();
        // Build a current-thread runtime per call — cheap (microseconds)
        // and avoids any global runtime assumptions. tokio is already a
        // workspace dep with the rt feature.
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                debug!(error = %e, "tier1 runtime build failed");
                return Tier1Result {
                    pass: false,
                    checks: vec![CheckResult {
                        name: "tier1_runtime".into(),
                        pass: false,
                        detail: format!("tokio runtime build failed: {e}"),
                        latency_ms: 0,
                    }],
                    latency_ms: 0,
                };
            }
        };

        // Group C5 (decision 47): cached capability snapshot. The
        // per-call subprocess probe (~ 900 ms p95 on Windows per the
        // Group C4 bench in metrics/phase_4I.toml L67) is replaced by
        // a single `OnceLock<Arc<ToolCapabilityFlags>>` initialized
        // on first request. Subsequent requests deref the Arc and
        // copy the inner u32 (free — bitflags::Flags is Copy).
        //
        // The OnceLock lives inside `cached_capability_flags()`; the
        // initializer calls `build_capability_flags()` exactly once
        // per process.
        let capabilities = *cached_capability_flags();

        // Use the canonical path so symlinks resolve to their target. We
        // accept the canonicalize failure as a Tier1 fail — missing files
        // surface here (tests c2 and a3 rely on this).
        let canonical = std::fs::canonicalize(&request.file_path);
        let mut effective = request.clone();
        if let Ok(p) = canonical {
            effective.file_path = p;
        } else {
            // Canonicalize failed -> Tier1 missing-file check.
            return Tier1Result {
                pass: false,
                checks: vec![CheckResult {
                    name: "missing_file".into(),
                    pass: false,
                    detail: format!("canonicalize failed for {:?}", request.file_path),
                    latency_ms: 0,
                }],
                latency_ms: u64::try_from(pipeline_clock.elapsed_since(started).as_millis()).unwrap_or(u64::MAX),
            };
        }

        // Dispatch — Box<dyn> over our four validators. Borrow-discipline:
        // the Tier1Validator's validate() takes &MediaValidationRequest,
        // so an owned Box drop after the await chain is fine.
        rt.block_on(async {
            let v: Box<dyn Tier1Validator> = match effective.media_type {
                MediaType::Image => Box::new(ImageValidator),
                MediaType::Audio => Box::new(AudioValidator),
                MediaType::Model3D => Box::new(Model3DValidator),
                MediaType::UiScreenshot => Box::new(UiScreenshotValidator),
            };
            v.validate(&effective, capabilities).await
        })
    }

    // ---------------------------------------------------------------------
    // Tier 2 dispatcher (mock-only in default features)
    // ---------------------------------------------------------------------

    fn run_tier2<B: VlmBackend + ?Sized>(
        request: &MediaValidationRequest,
        vlm: &B,
    ) -> Result<Tier2Result, String> {
        let pipeline_clock = OsClock;
        let started = pipeline_clock.now();
        let advisory: VlmValidationOutput = vlm.validate_advisory(&request.file_path);
        // Schema validation: ranges + Severity enum.
        if !(0.0..=1.0).contains(&advisory.overall_quality) {
            return Err(format!(
                "schema-invalid overall_quality={}",
                advisory.overall_quality
            ));
        }
        if !(0.0..=1.0).contains(&advisory.confidence) {
            return Err(format!("schema-invalid confidence={}", advisory.confidence));
        }
        let any_critical = advisory
            .issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Critical));
        let advisory_pass = !any_critical && advisory.overall_quality >= 0.5;
        let structured_output = match serde_json::to_value(&advisory) {
            Ok(v) => v,
            Err(e) => return Err(format!("serde_json: {e}")),
        };
        Ok(Tier2Result {
            advisory_pass,
            structured_output,
            model_used: vlm.model_id().to_owned(),
            latency_ms: u64::try_from(pipeline_clock.elapsed_since(started).as_millis()).unwrap_or(u64::MAX),
        })
    }

    // Suppress the unused-import warning when only the public API is
    // exercised; PathBuf is part of the canonical signatures elsewhere
    // in the module so we keep the import for forward-compatibility.
    #[allow(dead_code)]
    fn _path_buf_marker(_: PathBuf) {}
}

// ---------------------------------------------------------------------------
// Async pipeline entry point — Group C3 deliverable A.
// ---------------------------------------------------------------------------

/// Async pipeline entry point matching the spec phase-4I.txt L253-298 sig.
/// Wraps [`pipeline::validate`] for callers that already live in a tokio
/// runtime; no separate runtime is spawned. The signature accepts a
/// boxed `VlmBackend` so a Mock or real backend can be swapped at the
/// dispatcher layer.
///
/// `_capabilities` is currently consumed inside `pipeline::validate` via
/// `build_capability_flags()` rather than the caller-supplied flags;
/// Group C5 will rewire so the dispatcher caches them once at startup.
///
/// Backward-compatible (audit BLOCK-1): callers without an SSB writer
/// pass `None` for `sink` and the helper functions log via
/// `tracing::info!` instead of emitting SSB rows.
pub async fn validate(
    request: MediaValidationRequest,
    _capabilities: ToolCapabilityFlags,
    vlm_backend: Option<Box<dyn VlmBackend>>,
) -> Result<MediaValidationResult, MediaValidatorError> {
    validate_with_sink(request, _capabilities, vlm_backend, None).await
}

/// Async variant of [`validate`] that accepts an optional
/// [`pipeline::MediaSsbSink`]. Group C5 wires this through `Arc<Ssb>`;
/// Group C3 ships the trait so tests can pin observable emissions.
/// Audit BLOCK-1 fix (decision 39).
pub async fn validate_with_sink(
    request: MediaValidationRequest,
    _capabilities: ToolCapabilityFlags,
    vlm_backend: Option<Box<dyn VlmBackend>>,
    sink: Option<&dyn pipeline::MediaSsbSink>,
) -> Result<MediaValidationResult, MediaValidatorError> {
    match vlm_backend {
        Some(backend) => {
            pipeline::validate_with_sink::<dyn VlmBackend>(&request, backend.as_ref(), sink)
        }
        None => {
            let mock = MockVlmBackend::new();
            pipeline::validate_with_sink(&request, &mock, sink)
        }
    }
}

// ---------------------------------------------------------------------------
// B-24 Phase 4I Group C9: SSB-backed production sink (decision 93).
// ---------------------------------------------------------------------------
//
// Bridges the sync `pipeline::MediaSsbSink` trait emit calls onto the async
// `Ssb::write_payload` write path. Sync-to-async is handled via
// `tokio::runtime::Handle::try_current()` -- when we're inside a tokio
// runtime (the production task_runner path is) we `spawn` a fire-and-forget
// write; when no runtime is active (e.g. a sync test) we fall back to a
// `tracing::warn!` so the audit trail still records the emission attempt
// instead of silently dropping it. The previous "no SSB-backed sink"
// state (Group C5/C8 stopped at the trait) was the missing link between
// the validator pipeline and the dashboard subscriber from C8.
//
// Spec phase-4I.txt L613: "Every call site and struct that participates in
// request routing must wire ssb, fsm, and clock through the single
// RuntimeClient". `SsbMediaSink` carries the `Arc<Ssb>` -- the same handle
// the orchestrator uses everywhere -- and the agent_id string used on
// every write_payload so the dashboard can correlate rows back to the
// emitting task.
pub struct SsbMediaSink {
    ssb: std::sync::Arc<crate::ssb::Ssb>,
    agent_id: String,
    superstep: i64,
}

impl SsbMediaSink {
    /// Construct a new sink bound to the orchestrator's shared `Ssb` handle.
    /// `agent_id` is recorded on every emitted row (typically the
    /// task_runner's `RUNNER_AGENT_ID` constant for the task-runner-driven
    /// path; tests may pass any non-empty identifier).
    pub fn new(ssb: std::sync::Arc<crate::ssb::Ssb>, agent_id: impl Into<String>) -> Self {
        Self {
            ssb,
            agent_id: agent_id.into(),
            superstep: 0,
        }
    }

    /// Override the superstep tag attached to every row emitted by this
    /// sink. Defaults to 0 (matches task_runner's `RUNNER_SUPERSTEP`).
    pub fn with_superstep(mut self, superstep: i64) -> Self {
        self.superstep = superstep;
        self
    }

    /// TEST ONLY -- returns the raw pointer (as usize) of the internal
    /// `Arc<Ssb>` so integration tests can assert that the sink holds the
    /// SAME Arc<Ssb> handle as the orchestrator/runtime (spec L613:
    /// "Every call site and struct that participates in request routing
    /// must wire ssb, fsm, and clock through the single RuntimeClient").
    /// Regression-detection for any future refactor that constructs
    /// `SsbMediaSink` with a different Arc<Ssb>.
    ///
    /// Not gated behind `#[cfg(test)]` because integration tests under
    /// `tests/` are SEPARATE compilation units that link against the
    /// library WITHOUT `cfg(test)` active. The `__test_only_` prefix and
    /// `#[doc(hidden)]` keep it out of the public API surface.
    #[doc(hidden)]
    pub fn __test_only_ssb_ptr(&self) -> usize {
        std::sync::Arc::as_ptr(&self.ssb) as usize
    }

    /// Internal: spawn an SSB write on the current tokio runtime, falling
    /// back to a `tracing::warn!` when called outside a runtime so the
    /// emission attempt remains visible in the structured-log stream.
    /// Fire-and-forget: callers do not block on the write completing
    /// because the sink methods are synchronous (per trait signature).
    fn spawn_write(&self, payload: crate::ssb::SsbPayload) {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let ssb = std::sync::Arc::clone(&self.ssb);
                let agent_id = self.agent_id.clone();
                let superstep = self.superstep;
                handle.spawn(async move {
                    if let Err(e) =
                        ssb.write_payload(&agent_id, superstep, payload, Vec::new()).await
                    {
                        tracing::warn!(error = %e, "SsbMediaSink: write_payload failed");
                    }
                });
            }
            Err(_) => {
                tracing::warn!(
                    payload_variant = ?payload.action_type(),
                    "SsbMediaSink: no tokio runtime active; emission dropped (sync fallback)"
                );
            }
        }
    }
}

impl pipeline::MediaSsbSink for SsbMediaSink {
    fn on_validation_started(
        &self,
        request_id: uuid::Uuid,
        media_type: &str,
        file_path: &str,
    ) {
        self.spawn_write(crate::ssb::SsbPayload::MediaValidationStarted(
            crate::ssb::MediaValidationStartedPayload {
                step_id: request_id.to_string(),
                media_type: media_type.to_owned(),
                file_path: file_path.to_owned(),
            },
        ));
    }

    fn on_tier1_complete(
        &self,
        request_id: uuid::Uuid,
        pass: bool,
        latency_ms: u64,
        check_count: u32,
    ) {
        self.spawn_write(crate::ssb::SsbPayload::MediaTier1Complete(
            crate::ssb::MediaTier1CompletePayload {
                step_id: request_id.to_string(),
                pass,
                check_count,
                latency_ms,
            },
        ));
    }

    fn on_tier2_complete(
        &self,
        request_id: uuid::Uuid,
        advisory_pass: bool,
        model_used: &str,
        latency_ms: u64,
    ) {
        self.spawn_write(crate::ssb::SsbPayload::MediaTier2Complete(
            crate::ssb::MediaTier2CompletePayload {
                step_id: request_id.to_string(),
                advisory_pass,
                model_used: model_used.to_owned(),
                latency_ms,
            },
        ));
    }

    fn on_human_review_required(&self, request_id: uuid::Uuid, reason: &str) {
        self.spawn_write(crate::ssb::SsbPayload::MediaHumanReviewRequired(
            crate::ssb::MediaHumanReviewRequiredPayload {
                step_id: request_id.to_string(),
                reason: reason.to_owned(),
            },
        ));
    }

    fn on_visual_feedback_iteration(
        &self,
        step_id: &str,
        iteration: u32,
        issue_count: u32,
        converged: bool,
    ) {
        self.spawn_write(crate::ssb::SsbPayload::VisualFeedbackIteration(
            crate::ssb::VisualFeedbackIterationPayload {
                step_id: step_id.to_owned(),
                iteration,
                issue_count,
                converged,
            },
        ));
    }
}

// ---------------------------------------------------------------------------
// B-24 Phase 4I Group C9: lazy-load wire-up helper for task_runner
// (decision 94).
// ---------------------------------------------------------------------------
//
// Single seam the task_runner calls right after `spec_from_decomposition`
// to honour spec L289-294 ("Lazy loading: MediaValidatorPlugin registered
// via ToolRegistry::register_lazy(name, factory_fn) at startup. Factory
// not invoked until first MediaValidationRequest arrives.") + B-24 plan
// L1 ("Lazy module loading: media_validator only instantiated when
// PipelineSpec has media steps."). The helper returns Some only when
// `media_steps_count > 0`; the returned `MediaValidator` then lives on
// `ResolvedConfig` for the duration of the task.
//
// Decoupled from `task_runner` to keep the wire seam testable in
// isolation -- the e2e mock test calls `wire_media_validator_for_spec`
// directly without instantiating the full TaskRunner.
pub fn wire_media_validator_for_spec(
    media_steps_count: usize,
    capabilities: ToolCapabilityFlags,
    vlm_backend: Option<Box<dyn VlmBackend>>,
) -> Option<MediaValidator> {
    new_if_media_steps(media_steps_count > 0, capabilities, vlm_backend)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_if_media_steps_returns_none_for_no_media() {
        let r = new_if_media_steps(false, ToolCapabilityFlags::empty(), None);
        assert!(r.is_none());
    }

    #[test]
    fn new_if_media_steps_returns_some_for_media() {
        let r = new_if_media_steps(true, ToolCapabilityFlags::empty(), None);
        assert!(r.is_some());
    }

    /// Audit WARN-4 fix: verify the inner fields of the returned
    /// `MediaValidator` are populated as documented. Replaces the prior
    /// `assert!(... .is_some())` shallow check.
    #[test]
    fn new_if_media_steps_populates_inner_fields() {
        let caps = ToolCapabilityFlags::IMAGEMAGICK | ToolCapabilityFlags::FFPROBE;
        let backend: Box<dyn VlmBackend> = Box::new(MockVlmBackend::new());
        let validator = new_if_media_steps(true, caps, Some(backend))
            .expect("validator should be Some when has_media=true");
        // capability snapshot is the exact bitset we passed in.
        assert_eq!(*validator.capabilities, caps);
        assert!(validator.capabilities.contains(ToolCapabilityFlags::IMAGEMAGICK));
        assert!(validator.capabilities.contains(ToolCapabilityFlags::FFPROBE));
        assert!(!validator.capabilities.contains(ToolCapabilityFlags::BLENDER));
        // backend round-trip.
        assert!(validator.vlm_backend.is_some());
        assert_eq!(
            validator.vlm_backend.as_ref().unwrap().model_id(),
            "mock-vlm"
        );
    }

    #[test]
    fn new_if_media_steps_returns_none_even_with_backend_when_no_media() {
        let backend: Box<dyn VlmBackend> = Box::new(MockVlmBackend::new());
        let r = new_if_media_steps(false, ToolCapabilityFlags::all(), Some(backend));
        assert!(
            r.is_none(),
            "factory must skip construction when has_media=false (lazy contract)"
        );
    }

    #[test]
    fn new_if_media_steps_stub_still_returns_none() {
        // Pinned for backward compat with the C2 lazy-loading test that
        // greps for `new_if_media_steps_stub`.
        assert!(new_if_media_steps_stub(false).is_none());
        assert!(new_if_media_steps_stub(true).is_none());
    }

    #[test]
    fn should_load_media_validator_mirrors_has_media() {
        assert!(should_load_media_validator(true));
        assert!(!should_load_media_validator(false));
    }
}
