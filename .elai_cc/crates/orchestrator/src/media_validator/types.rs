//! B-24 Phase 4I (Group C1): media-validator type skeleton.
//!
//! All public types declared in `docs/specs/guide/steps/phase-4I.txt`
//! sub-step 4I.1 live here. The split into types / tier1 / tier2 / clap /
//! external_tools / mock_vlm files mirrors the interface map in
//! `workflow/sessions/s-31180/context-package.md` so each Group (C1..C6)
//! lands in the smallest possible patch. NO real validator behavior is
//! implemented in this file — only data definitions, constructors, and
//! their (de)serialization plumbing.
//!
//! Anti-stale grounding (per CLAUDE.md):
//!   * `bitflags = 2.6` macro syntax (workspace adopted in Cargo.toml).
//!   * `serde::{Serialize, Deserialize}` derives — round-trip required for
//!     the SSB media payloads added in Group C4.
//!   * `thiserror` for error variants (NOT Box<dyn Error>).
//!   * No magic numbers — all default thresholds live in named consts.

use std::path::PathBuf;
use std::time::Duration;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Named constants (CLAUDE.md anti-magic-numbers rule)
// ---------------------------------------------------------------------------

/// Default Tier-1 wall-clock budget (`phase-4I.txt` adversarial section
/// `oversized media`). Caller may override per-request.
pub const DEFAULT_TIER1_TIMEOUT_MS: u64 = 5_000;

/// Default Tier-2 (VLM) startup budget. Sgang/Ollama process must answer
/// /v1/health within this window or the constructor returns
/// `MediaValidatorError::VlmStartupTimeout`. 60s is conservative for CPU EP
/// and well under typical CUDA warmup on RTX-class hardware.
pub const DEFAULT_VLM_STARTUP_TIMEOUT_SECS: u64 = 60;

/// Mode-A (Qwen3-VL 30B Q4) hardware floor on a single GPU per
/// `workflow/papers/qwen-vl-internvl-interface.md` L30-37 (~23.8 GB total
/// runtime; 24 GB is the smallest commercial card that fits).
pub const MODE_A_MIN_VRAM_GB: f32 = 24.0;

/// Mode-B (InternVL3.5 38B Q4 single-GPU) hardware floor — paper L137-141
/// confirms TP unsupported on the InternVL3.5 vision encoder, so single-GPU
/// is the only valid topology. ~21 GB at Q4.
pub const MODE_B_MIN_VRAM_GB: f32 = 21.0;

/// Default `max_iterations` for the visual-feedback loop. Per
/// `phase-4I.txt` 4I.7 + `docs/specs/integration/media-validator.txt`. The
/// B-24 plan said 5; spec wins.
pub const DEFAULT_MAX_ITERATIONS: u32 = 3;

/// Maximum file size accepted by the validator before invoking any
/// external tool or VLM. Spec phase-4I.txt L215-216 ("oversized 500MB
/// rejected pre-VLM"). 50 MiB is the conservative default; production
/// callers may override via [`MediaValidationRequest::max_file_bytes`].
pub const MAX_VALIDATION_FILE_BYTES: u64 = 50 * 1024 * 1024;

/// Bounded-queue depth for incoming validation requests. Spec
/// phase-4I.txt L223-226 (a6_queue_full). Group C5 wires the actual
/// `tokio::sync::mpsc::channel(VALIDATION_QUEUE_DEPTH)`.
pub const VALIDATION_QUEUE_DEPTH: usize = 64;

/// Maximum concurrent Tier-1 dispatches. Spec phase-4I.txt L223-226.
pub const MAX_TIER1_CONCURRENT: usize = 4;

/// Maximum concurrent Tier-2 dispatches (single VLM process serializes).
pub const MAX_TIER2_CONCURRENT: usize = 1;

/// SSIM similarity threshold default used by C3 when QualitySpec.
/// similarity_threshold is None but a reference is supplied.
pub const DEFAULT_SSIM_THRESHOLD: f32 = 0.95;

/// Audio silence detection threshold (RMS energy, normalized [0.0, 1.0]).
/// 0.001 ≈ -60 dBFS — well below the noise floor of consumer audio
/// equipment. Spec phase-4I.txt L261 ("Silence detection via RMS energy").
/// Audit WARN-2 fix (decision 40): replaces the prior zero-check which
/// flagged ONLY samples that were exactly 0 — a quiet but non-zero signal
/// (e.g. -90 dBFS) would have passed the silence check despite being
/// inaudible.
pub const SILENCE_RMS_THRESHOLD: f32 = 0.001;

/// Audio clipping detection threshold (fraction of samples at i16::MAX or
/// i16::MIN). 0.001 = 0.1% of samples clipped flags the buffer. Spec
/// phase-4I.txt L262 ("Clipping detection via peak analysis"). Audit
/// WARN-3 fix (decision 41): replaces the prior zero-tolerance check which
/// flagged ANY single clipped sample — a one-sample transient on a 48 kHz
/// 60-second buffer (2.88M samples) would have failed despite being
/// inaudible. 0.1% balances false-positive rate against actual distortion.
pub const CLIPPING_THRESHOLD_RATIO: f32 = 0.001;

// ---------------------------------------------------------------------------
// Media classification
// ---------------------------------------------------------------------------

/// Top-level media type. Drives validator selection in `tier1::dispatch`
/// and `tier2::select_validator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Model3D,
    Audio,
    UiScreenshot,
}

/// Validation depth. `Quick` runs Tier 1 only; `Full` runs Tier 1 + Tier 2
/// (when a `QualitySpec.qualitative_checks` flag is set) per the veto-logic
/// spec table in `phase-4I.txt` 4I.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMode {
    Quick,
    Full,
}

// ---------------------------------------------------------------------------
// Quality specification (subset reused from `research_tree::QualitySpec`).
// The full spec lives in `research_tree.rs` — Group C1 only needs a stable
// surface for the request struct. Group C4 wires the actual research_tree
// type back in via re-export.
// ---------------------------------------------------------------------------

/// Acceptance criteria for a media artifact. Group C1 ships the minimal
/// shape required for the type-coherence test in 4I.2; Group C5 expands it
/// to include the full SSIM / LPIPS / CLAP thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualitySpec {
    /// Optional similarity threshold (SSIM/LPIPS) compared against
    /// `MediaValidationRequest.reference_path`.
    #[serde(default)]
    pub similarity_threshold: Option<f32>,
    /// When true, Tier 2 (VLM) advisory is requested even on Tier-1 pass.
    /// Drives the spec's "only Tier 2 when QualitySpec.has_qualitative"
    /// branch (4I.5 contract).
    #[serde(default)]
    pub qualitative_checks: bool,
    /// When true, the RCE (rendering + critique engine) visual-feedback
    /// loop is invoked AFTER Tier1/Tier2 validation. Drives the spec
    /// gate at phase-4I.txt L449 — "when visual_check is false, no
    /// AdapterRegistry lookup happens, no `RenderAdapter::render` runs,
    /// no `VisualFeedbackLoop` is constructed". Defaults to `false` so
    /// existing callers retain the C3/C4 pipeline shape (Tier1->Tier2
    /// only). Group C5 audit WARN 3 fix (decision 61).
    #[serde(default)]
    pub visual_check: bool,
}

impl Default for QualitySpec {
    fn default() -> Self {
        Self {
            similarity_threshold: None,
            qualitative_checks: false,
            visual_check: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Request / result types
// ---------------------------------------------------------------------------

/// Single-shot validation request. Constructed by `task_runner` once per
/// media artifact emitted by an executor agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaValidationRequest {
    pub media_type: MediaType,
    pub file_path: PathBuf,
    pub quality_spec: QualitySpec,
    /// Optional reference image / model used by similarity-based checks.
    #[serde(default)]
    pub reference_path: Option<PathBuf>,
    pub mode: ValidationMode,
    /// Optional allowed-output-directory root. When `Some(dir)`, the
    /// pipeline canonicalizes `file_path` and `dir` and requires the
    /// canonicalized file path to live UNDER the canonicalized dir.
    /// This rejects symlink-bypass attacks that resolve to a target
    /// outside the agent's writable scope (spec phase-4I.txt L237-238,
    /// audit BLOCK-3 fix; decision-doc entry 42).
    ///
    /// `None` preserves the C2 contract for tests that don't pin an
    /// output_dir (only the `..`-component guard applies). Production
    /// callers (task_runner / Group C5) MUST set this to the agent's
    /// scoped output directory.
    #[serde(default)]
    pub output_dir: Option<PathBuf>,
    /// Opaque request identifier used to correlate SSB events for a
    /// single validation. Group C3 emits one `MediaValidationStarted` +
    /// per-tier completion events keyed on this id; the dashboard pivots
    /// on `request_id` to render the per-artifact timeline (audit BLOCK-1
    /// fix; decision-doc entry 39). `uuid::Uuid` is already in the
    /// workspace dep graph (.elai_cc/Cargo.toml L37, v4 feature) so no
    /// new dependency is added.
    #[serde(default = "default_request_id")]
    pub request_id: uuid::Uuid,
}

fn default_request_id() -> uuid::Uuid {
    uuid::Uuid::new_v4() // entropy-allow: serde-default for absent field, no EntropySource available
}

/// One Tier-1 deterministic check (e.g. resolution, manifold, sample-rate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub pass: bool,
    pub detail: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tier1Result {
    pub pass: bool,
    pub checks: Vec<CheckResult>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tier2Result {
    pub advisory_pass: bool,
    pub structured_output: serde_json::Value,
    pub model_used: String,
    pub latency_ms: u64,
}

/// VLM structured output. Group C4 enforces a JSON schema validator over
/// this struct so out-of-range values fall back to `Verdict::Degraded`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VlmValidationOutput {
    /// Range [0.0, 1.0]. Higher = better.
    pub overall_quality: f32,
    pub issues: Vec<Issue>,
    /// Range [0.0, 1.0]. VLM self-reported confidence.
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub severity: Severity,
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub location: Option<BoundingBox>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    Major,
    Minor,
}

/// Normalized bounding box in `[0.0, 1.0]` over the image extent.
/// Float coords (NOT pixel coords) so the same struct serves images,
/// 3D viewport projections, and screenshots at different resolutions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    /// Constructor that clamps to the unit square. Useful when a VLM emits
    /// slightly out-of-range coords (Qwen3-VL is not numerically perfect).
    pub fn clamped(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            width: width.clamp(0.0, 1.0),
            height: height.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Verdict {
    Pass,
    Fail,
    HumanReview,
    Degraded,
}

/// Top-level result returned to the task runner. Holds both tiers so the
/// dashboard's MediaValidationPanel can render the audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaValidationResult {
    pub tier1: Tier1Result,
    #[serde(default)]
    pub tier2: Option<Tier2Result>,
    pub final_verdict: Verdict,
    pub verdict_reason: String,
}

// ---------------------------------------------------------------------------
// Tool capability flags (bitflags 2.x)
// ---------------------------------------------------------------------------

bitflags! {
    /// Per-process snapshot of which external CLIs were detected at
    /// MediaValidator startup. Cached in `Arc<ToolCapabilityFlags>` so each
    /// Tier-1 call is a fast bitwise AND against the required tools.
    ///
    /// All 9 spec-mandated capabilities are represented even though only
    /// IMAGEMAGICK + FFPROBE + TESSERACT + BLENDER are probed in 4I.2.
    /// PLAYWRIGHT / COMFYUI / OLLAMA / SGLANG are stubbed for Group C5/C6.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ToolCapabilityFlags: u32 {
        const IMAGEMAGICK = 0b0000_0000_0001;
        const FFPROBE     = 0b0000_0000_0010;
        const FFMPEG      = 0b0000_0000_0100;
        const TESSERACT   = 0b0000_0000_1000;
        const BLENDER     = 0b0000_0001_0000;
        const PLAYWRIGHT  = 0b0000_0010_0000;
        const COMFYUI     = 0b0000_0100_0000;
        const OLLAMA      = 0b0000_1000_0000;
        const SGLANG      = 0b0001_0000_0000;
    }
}

// ---------------------------------------------------------------------------
// VLM (Tier 2) configuration — type-level Mode B + Ollama elimination.
// ---------------------------------------------------------------------------

/// VLM execution mode. `A` = single-image fast path (Ollama/Qwen3-VL 30B).
/// `B` = multi-image / sequence path (SGLang/InternVL3.5 38B). The
/// `VlmConfig` enum below makes Mode B + Ollama unconstructible at the
/// type level (no need for runtime guards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum VlmMode {
    A,
    B,
}

/// VLM serving engine. Pure metadata for telemetry; the actual ABI choice
/// is encoded in `VlmConfig` to keep the (mode, engine) invariants type-level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum VlmEngine {
    Ollama,
    Sglang,
}

/// Mode-A configuration. Always paired with Ollama at the type level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama model tag — e.g. `qwen3-vl:30b-a3b-instruct` (NOT
    /// `Qwen3.5-VL-30B-A3B`; tag corrected via paper qwen-vl L24-28).
    pub model_tag: String,
    /// Ollama server endpoint, default <http://127.0.0.1:11434>.
    pub endpoint: String,
}

/// Mode-B configuration. Always paired with SGLang at the type level.
/// Single-GPU Q4 only — paper qwen-vl L137-141 documents that InternVL3.5
/// vision encoder does NOT support tensor parallelism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SglangConfig {
    /// HuggingFace model id, e.g. `OpenGVLab/InternVL3_5-38B-Instruct`.
    pub model_path: String,
    /// SGLang server endpoint, default <http://127.0.0.1:30000>.
    pub endpoint: String,
}

/// VLM config — two variants make the (mode, engine) pair unrepresentable
/// when invalid. Specifically: there is NO `ModeB(OllamaConfig)` arm, so
/// constructing Mode B over Ollama is a compile-time error. This is the
/// "type-level enforcement" required by the spec adversarial section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "config")]
pub enum VlmConfig {
    ModeA(OllamaConfig),
    ModeB(SglangConfig),
}

impl VlmConfig {
    pub fn mode(&self) -> VlmMode {
        match self {
            VlmConfig::ModeA(_) => VlmMode::A,
            VlmConfig::ModeB(_) => VlmMode::B,
        }
    }

    pub fn engine(&self) -> VlmEngine {
        match self {
            VlmConfig::ModeA(_) => VlmEngine::Ollama,
            VlmConfig::ModeB(_) => VlmEngine::Sglang,
        }
    }
}

/// Process-scoped VLM configuration. The `allowed_paths` list is mounted
/// into the sandbox / namespace so Tier 2 can only read the artifact under
/// validation (cannot exfiltrate `/proc/self/environ`, etc., per the
/// adversarial sandbox-escape contract in 4I.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VlmProcessConfig {
    pub config: VlmConfig,
    pub allowed_paths: Vec<PathBuf>,
    #[serde(default)]
    pub socket_path: Option<PathBuf>,
    #[serde(with = "humantime_serde_duration")]
    pub startup_timeout: Duration,
}

impl VlmProcessConfig {
    /// Constructor for Mode A (single-image / Ollama). The hardware guard
    /// uses NVML when the `vlm-integration` feature is on; without the
    /// feature, the constructor unconditionally returns `HardwareInsufficient`
    /// — Phase 0 deliberately routes through the MockVlmBackend.
    #[cfg(feature = "vlm-integration")]
    pub fn new_mode_a(
        ollama: OllamaConfig,
        allowed_paths: Vec<PathBuf>,
    ) -> Result<Self, MediaValidatorError> {
        let available = probe_vram_gb()?;
        if available < MODE_A_MIN_VRAM_GB {
            return Err(MediaValidatorError::HardwareInsufficient {
                required_gb: MODE_A_MIN_VRAM_GB,
                available_gb: available,
            });
        }
        Ok(Self {
            config: VlmConfig::ModeA(ollama),
            allowed_paths,
            socket_path: None,
            startup_timeout: Duration::from_secs(DEFAULT_VLM_STARTUP_TIMEOUT_SECS),
        })
    }

    #[cfg(not(feature = "vlm-integration"))]
    pub fn new_mode_a(
        _ollama: OllamaConfig,
        _allowed_paths: Vec<PathBuf>,
    ) -> Result<Self, MediaValidatorError> {
        Err(MediaValidatorError::HardwareInsufficient {
            required_gb: MODE_A_MIN_VRAM_GB,
            available_gb: 0.0,
        })
    }

    /// Constructor for Mode B (multi-image / SGLang). Same hardware-guard
    /// pattern as `new_mode_a`. Single-GPU Q4 ONLY — TP is not configurable
    /// here because the underlying engine does not support it.
    #[cfg(feature = "vlm-integration")]
    pub fn new_mode_b(
        sglang: SglangConfig,
        allowed_paths: Vec<PathBuf>,
    ) -> Result<Self, MediaValidatorError> {
        let available = probe_vram_gb()?;
        if available < MODE_B_MIN_VRAM_GB {
            return Err(MediaValidatorError::HardwareInsufficient {
                required_gb: MODE_B_MIN_VRAM_GB,
                available_gb: available,
            });
        }
        Ok(Self {
            config: VlmConfig::ModeB(sglang),
            allowed_paths,
            socket_path: None,
            startup_timeout: Duration::from_secs(DEFAULT_VLM_STARTUP_TIMEOUT_SECS),
        })
    }

    #[cfg(not(feature = "vlm-integration"))]
    pub fn new_mode_b(
        _sglang: SglangConfig,
        _allowed_paths: Vec<PathBuf>,
    ) -> Result<Self, MediaValidatorError> {
        Err(MediaValidatorError::HardwareInsufficient {
            required_gb: MODE_B_MIN_VRAM_GB,
            available_gb: 0.0,
        })
    }
}

/// NVML-backed free-VRAM probe. Off-feature this function is absent so the
/// caller is forced through the `cfg(not(feature = "vlm-integration"))`
/// arm above (which returns `HardwareInsufficient`). This keeps nvml-wrapper
/// out of the default build graph.
///
/// Multi-GPU policy (decision: workflow/reasoning/B-24-decisions.md, follow-up:
/// Group C6 dispatcher): if the host has more than one CUDA device we refuse
/// the probe entirely. Returning device-0 free VRAM on a multi-GPU box would
/// falsely greenlight Mode B on a machine whose dispatch layer has not yet
/// been written. Single-GPU hosts keep the device-0 probe.
#[cfg(feature = "vlm-integration")]
fn probe_vram_gb() -> Result<f32, MediaValidatorError> {
    use nvml_wrapper::Nvml;
    let nvml = Nvml::init().map_err(|e| MediaValidatorError::NvmlInit(e.to_string()))?;
    let device_count = nvml
        .device_count()
        .map_err(|e| MediaValidatorError::NvmlInit(e.to_string()))?;
    if device_count > 1 {
        // Multi-GPU dispatch is Group C6 — refuse instead of greenlighting on
        // device 0 alone. `required_gb = MODE_B_MIN_VRAM_GB` matches the
        // Mode-B refusal path above; `available_gb = 0.0` signals "not probed".
        return Err(MediaValidatorError::HardwareInsufficient {
            required_gb: MODE_B_MIN_VRAM_GB,
            available_gb: 0.0,
        });
    }
    // Single-GPU host: probe device 0.
    let device = nvml
        .device_by_index(0)
        .map_err(|e| MediaValidatorError::NvmlInit(e.to_string()))?;
    let mem = device
        .memory_info()
        .map_err(|e| MediaValidatorError::NvmlInit(e.to_string()))?;
    // mem.free is bytes; convert to GiB (1024^3).
    let free_gb = (mem.free as f64) / (1024.0 * 1024.0 * 1024.0);
    Ok(free_gb as f32)
}

// ---------------------------------------------------------------------------
// CLAP audio config
// ---------------------------------------------------------------------------

/// Execution device for the CLAP ONNX session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClapDevice {
    Cpu,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClapConfig {
    pub model_path: PathBuf,
    pub device: ClapDevice,
}

// ---------------------------------------------------------------------------
// Errors (thiserror — NOT Box<dyn Error>)
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum MediaValidatorError {
    #[error("hardware insufficient: required {required_gb} GiB, available {available_gb} GiB")]
    HardwareInsufficient { required_gb: f32, available_gb: f32 },

    #[error("VLM startup timed out")]
    VlmStartupTimeout,

    #[error("Mode B requires SGLang; Ollama is single-image only")]
    ModeBRequiresSglang,

    #[error("NVML init failed: {0}")]
    NvmlInit(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),

    /// Path-guard error surfaced when a request's file_path canonicalizes
    /// outside the allowed output directory. Group C3 wires the
    /// canonicalize() check; the variant exists in C2 so the adversarial
    /// contract test (a4_path_traversal_rejected) can pin the SPECIFIC
    /// rejection signal instead of accepting the NotImplemented placeholder.
    /// Decision: workflow/reasoning/B-24-decisions.md #28.
    #[error("path {path:?} is outside the output directory")]
    PathOutsideOutputDir { path: PathBuf },

    /// Bounded-queue rejection surfaced when the validator's concurrency
    /// budget is exhausted. Group C5 wires the bounded mpsc(64) +
    /// Semaphore; the variant exists in C2 so the adversarial contract
    /// test (a6_queue_full_returns_immediately) can pin the SPECIFIC
    /// rejection signal instead of accepting NotImplemented.
    /// Decision: workflow/reasoning/B-24-decisions.md #29.
    #[error("validation queue full at depth {depth}")]
    QueueFull { depth: usize },

    #[error("not implemented yet (skeleton): {0}")]
    NotImplemented(&'static str),

    /// Serialization failure surfaced when serde_json::to_value rejects
    /// a Tier2 advisory (e.g. NaN in overall_quality, non-finite floats).
    /// Audit WARN 2 fix (decision 75): the previous `unwrap_or(Null)`
    /// silently turned a serialization failure into a Null structured
    /// output downstream, masking VLM bugs. We now route the failure
    /// to Verdict::Degraded with reason "vlm_advisory_serialization_failed".
    #[error("VLM advisory serialization failed: {0}")]
    SerializationFailure(String),
}

#[derive(Debug, Error)]
pub enum Tier2Error {
    #[error("validator backend offline")]
    BackendOffline,

    #[error("schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("not implemented yet (skeleton): {0}")]
    NotImplemented(&'static str),
}

// ---------------------------------------------------------------------------
// Duration serde helper — keeps `startup_timeout` JSON-friendly.
// ---------------------------------------------------------------------------

mod humantime_serde_duration {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // u128 -> u64 narrowing. Saturating to u64::MAX is the documented
        // graceful fallback (no panic risk); `.unwrap_or` is permitted here
        // per CLAUDE.md (the rule targets `.unwrap()` proper).
        s.serialize_u64(u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}

// ---------------------------------------------------------------------------
// Type-level invariant test: Mode B over Ollama MUST be unrepresentable.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod type_level_invariants {
    use super::*;

    /// Verifies Mode B + SGLang IS constructible (positive control).
    #[test]
    fn mode_b_with_sglang_constructs() {
        let cfg = VlmConfig::ModeB(SglangConfig {
            model_path: "OpenGVLab/InternVL3_5-38B-Instruct".into(),
            endpoint: "http://127.0.0.1:30000".into(),
        });
        assert_eq!(cfg.mode(), VlmMode::B);
        assert_eq!(cfg.engine(), VlmEngine::Sglang);
    }

    /// Verifies Mode A + Ollama IS constructible (positive control).
    #[test]
    fn mode_a_with_ollama_constructs() {
        let cfg = VlmConfig::ModeA(OllamaConfig {
            model_tag: "qwen3-vl:30b-a3b-instruct".into(),
            endpoint: "http://127.0.0.1:11434".into(),
        });
        assert_eq!(cfg.mode(), VlmMode::A);
        assert_eq!(cfg.engine(), VlmEngine::Ollama);
    }

    // The negative case ("Mode B + Ollama is unrepresentable") is enforced
    // STRUCTURALLY: `VlmConfig` has no `ModeB(OllamaConfig)` constructor.
    // Attempting to write one would be a compile error, so we cannot put
    // it in a `#[test]` — but the absence of such a variant is the proof.
    // A doc-test could be added in Group C2 with `compile_fail`.

    #[test]
    fn capability_flags_round_trip() {
        let f = ToolCapabilityFlags::IMAGEMAGICK | ToolCapabilityFlags::FFPROBE;
        assert!(f.contains(ToolCapabilityFlags::IMAGEMAGICK));
        assert!(!f.contains(ToolCapabilityFlags::BLENDER));
        // serde round-trip.
        let s = serde_json::to_string(&f).expect("serialize");
        let d: ToolCapabilityFlags = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(f, d);
    }

    #[test]
    fn bbox_clamps_out_of_range() {
        let bb = BoundingBox::clamped(-0.1, 1.5, 2.0, 0.5);
        assert_eq!(bb.x, 0.0);
        assert_eq!(bb.y, 1.0);
        assert_eq!(bb.width, 1.0);
        assert_eq!(bb.height, 0.5);
    }
}
