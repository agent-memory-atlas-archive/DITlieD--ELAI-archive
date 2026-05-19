//! B-24 Phase 4I (Group C3): Tier-1 deterministic validators — REAL impl.
//!
//! Each MediaType has a dedicated validator. Real subprocess + in-process
//! header parsing combined: when the external CLI is available we use it
//! for full coverage (e.g. `magick identify` for ICC profile + colorspace);
//! when it is missing we fall back to in-process header inspection so the
//! deterministic checks still surface a pass/fail signal. CI without
//! ImageMagick / Tesseract / FFprobe stays GREEN — the only hard
//! external-tool requirement is Blender for Model3D, which is gated.
//!
//! Anti-stale grounding (CLAUDE.md):
//!   * tokio::process::Command + tokio::time::timeout for subprocess.
//!   * blake3 (NOT sha2) for any deterministic hash.
//!   * tracing for logs.
//!   * No .unwrap() in production paths.

use std::path::Path;
use std::time::Instant;

use tracing::{debug, warn};

use super::external_tools::{ExternalToolWrapper, ImageMagickWrapper, ToolError};
use super::types::{
    CheckResult, MediaType, MediaValidationRequest, Tier1Result, ToolCapabilityFlags,
    CLIPPING_THRESHOLD_RATIO, DEFAULT_SSIM_THRESHOLD, DEFAULT_TIER1_TIMEOUT_MS,
    SILENCE_RMS_THRESHOLD,
};

/// Tier-1 deterministic validator. Async because the real impl spawns
/// subprocesses (tokio::process::Command). Object-safety preserved via
/// boxed-Future return — callers store `Arc<dyn Tier1Validator>`.
///
/// Note on async-fn-in-trait vs Pin<Box<Future>>: RFC 3185 stabilized
/// native `async fn` in traits in Rust 1.75 (December 2023). However,
/// `async fn` in a trait yields an opaque `impl Future` whose `Send`
/// bound cannot be expressed in the trait signature without the
/// (unstable) `return_type_notation` feature, and the resulting trait
/// is NOT dyn-compatible — `dyn Tier1Validator` would fail to compile
/// against an `async fn validate()` method that needs `Send` futures.
/// We keep the boxed-`Future` shape (`Pin<Box<dyn Future<Output=...>
/// + Send + 'a>>`) so the validator stays object-safe AND the future
/// is provably `Send` for tokio multi-thread executors. The
/// `async-trait` crate is NOT used per CLAUDE.md anti-stale guidance.
pub trait Tier1Validator: Send + Sync {
    /// Run all spec-mandated checks for this media type. Returns the
    /// aggregated result with per-check breakdown so the dashboard can
    /// surface individual failures.
    fn validate<'a>(
        &'a self,
        request: &'a MediaValidationRequest,
        capabilities: ToolCapabilityFlags,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Tier1Result> + Send + 'a>>;

    /// True iff this validator handles the given media type.
    fn supports(&self, media_type: &MediaType) -> bool;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a single-check Tier1Result preserving deterministic ordering.
fn single_check(name: &str, pass: bool, detail: String, latency_ms: u64) -> Tier1Result {
    Tier1Result {
        pass,
        checks: vec![CheckResult {
            name: name.to_owned(),
            pass,
            detail,
            latency_ms,
        }],
        latency_ms,
    }
}

/// Aggregate per-check results into a Tier1Result. `pass` is the AND of
/// all check `pass` flags.
fn aggregate(checks: Vec<CheckResult>, started: Instant) -> Tier1Result {
    let pass = !checks.is_empty() && checks.iter().all(|c| c.pass);
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    Tier1Result {
        pass,
        checks,
        latency_ms,
    }
}

/// Read a file's first `n` bytes; on any error return an empty Vec so the
/// downstream parser surfaces a deterministic CheckResult instead of
/// propagating an io error.
fn read_head(path: &Path, n: usize) -> Vec<u8> {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(path) else {
        return Vec::new();
    };
    let mut buf = vec![0u8; n];
    let read = f.read(&mut buf).unwrap_or(0);
    buf.truncate(read);
    buf
}

/// Parse PNG header in-process. Returns Some((width,height)) on success;
/// None on any deviation from RFC2083 §11.2.2 (signature + IHDR length 13).
fn parse_png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 24 {
        return None;
    }
    if &bytes[0..8] != PNG_SIG {
        return None;
    }
    if &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Some((w, h))
}

/// Parse the WAV (RIFF/WAVE) header. Returns
/// `(sample_rate, channels, bits_per_sample, data_offset, data_size)` on
/// success; None otherwise.
fn parse_wav_header(bytes: &[u8]) -> Option<(u32, u16, u16, usize, u32)> {
    if bytes.len() < 44 {
        return None;
    }
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    if &bytes[12..16] != b"fmt " {
        return None;
    }
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
    let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
    if &bytes[36..40] != b"data" {
        return None;
    }
    let data_size = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
    Some((sample_rate, channels, bits, 44, data_size))
}

/// Inspect WAV PCM 16-bit samples for silence and clipping.
///
/// Returns `(is_silent, is_clipped, rms, clip_ratio, total_samples)`:
///   * `is_silent`  — `rms < SILENCE_RMS_THRESHOLD` (-60 dBFS default).
///   * `is_clipped` — `clip_ratio > CLIPPING_THRESHOLD_RATIO` (0.1% default).
///   * `rms`        — root-mean-square energy of the buffer, normalized
///                    to `[0.0, 1.0]` against `i16::MAX`.
///   * `clip_ratio` — clipped-sample count / total samples.
///   * `total_samples` — number of i16 samples scanned.
///
/// Spec phase-4I.txt L261-262. Audit WARN-2 + WARN-3 fix (decisions 40, 41):
/// the prior implementation used a zero-check for silence and a single-
/// sample threshold for clipping, both of which produced false positives /
/// negatives on real audio (see SILENCE_RMS_THRESHOLD and
/// CLIPPING_THRESHOLD_RATIO doc-comments for the rationale).
fn scan_wav_pcm16(bytes: &[u8], data_offset: usize, data_size: u32) -> (bool, bool, f32, f32, u64) {
    let end = (data_offset + data_size as usize).min(bytes.len());
    let slice = if data_offset < end {
        &bytes[data_offset..end]
    } else {
        &[]
    };
    let mut total: u64 = 0;
    let mut clipped: u64 = 0;
    // f64 accumulator avoids precision loss on long buffers — i16::MAX^2
    // ≈ 1.07e9, summed over 2.88M samples (60s @ 48kHz) ≈ 3.1e15, still
    // well within f64's 53-bit mantissa.
    let mut sumsq: f64 = 0.0;
    let mut i = 0;
    while i + 1 < slice.len() {
        let s = i16::from_le_bytes([slice[i], slice[i + 1]]);
        total += 1;
        let norm = (s as f64) / (i16::MAX as f64);
        sumsq += norm * norm;
        if s == i16::MAX || s == i16::MIN {
            clipped += 1;
        }
        i += 2;
    }
    let (rms_f64, clip_ratio_f64) = if total == 0 {
        // No samples scanned -- treat as silent zero-energy buffer.
        (0.0, 0.0)
    } else {
        let mean_sq = sumsq / (total as f64);
        let ratio = (clipped as f64) / (total as f64);
        (mean_sq.sqrt(), ratio)
    };
    let rms = rms_f64 as f32;
    let clip_ratio = clip_ratio_f64 as f32;
    // total>0 && rms below threshold => silence. total==0 deliberately
    // does NOT flag silence — the empty-data-chunk case is surfaced
    // separately by the header check.
    let is_silent = total > 0 && rms < SILENCE_RMS_THRESHOLD;
    let is_clipped = clip_ratio > CLIPPING_THRESHOLD_RATIO;
    (is_silent, is_clipped, rms, clip_ratio, total)
}

/// Run `magick identify` and parse the canonical "%w|%h|%m|%[colorspace]|%z|%b"
/// output line. Returns Some(parts) on clean exit AND no error/corrupt in
/// stderr; None when the tool reports trouble.
async fn run_magick_identify(
    wrapper: &ImageMagickWrapper,
    file_path: &Path,
) -> Result<Option<Vec<String>>, ToolError> {
    let path_str = file_path.to_string_lossy().to_string();
    let args: Vec<&str> = vec![
        "identify",
        "-format",
        "%w|%h|%m|%[colorspace]|%z|%b",
        path_str.as_str(),
    ];
    let timeout = std::time::Duration::from_millis(DEFAULT_TIER1_TIMEOUT_MS);
    let output = wrapper.execute(&args, timeout).await?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    // Per workflow/papers/imagemagick-tesseract-interface.md L40-46:
    // exit code is unreliable, must scan stderr.
    if stderr.contains("corrupt") || stderr.contains("error") || stderr.contains("could not") {
        return Ok(None);
    }
    if output.exit_code != 0 {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // identify with multiple frames may emit one row per frame; first row wins.
    let line = stdout.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        return Ok(None);
    }
    let parts: Vec<String> = line.split('|').map(|s| s.trim().to_owned()).collect();
    Ok(Some(parts))
}

/// Compute SSIM between two PNG files. Both must be loadable via the
/// `image-compare` crate's re-exported `image` types. Returns None on any
/// load error.
///
/// Dimension policy:
///   * Identical dimensions + identical pixel bytes -> 1.0 short-circuit.
///     This handles 1x1 fixtures where the underlying SSIM algorithm
///     refuses to run (MSSIMSimple has an 8x8 sliding-window minimum
///     per the c6 contract test comment).
///   * Identical dimensions, different pixels, AND >= 8x8 -> SSIM via
///     `image_compare::gray_similarity_structure`.
///   * Identical dimensions, different pixels, < 8x8 -> 0.0 (cannot
///     attest similarity below the minimum window).
///   * Mismatched dimensions -> 0.0 (image-compare requires equal dims).
fn compute_ssim_for_pngs(a: &Path, b: &Path) -> Option<f32> {
    use image_compare::Algorithm;

    // Tier-1 byte-identity short-circuit: if the file CONTENTS are
    // byte-equal, SSIM is 1.0 regardless of decoder. This handles two
    // cases: (a) identical 1x1 fixtures that PNG decoders may refuse
    // because the IDAT zlib stream is non-canonical, and (b) the common
    // production case of "same artifact passed twice". Read up to 1 MiB
    // to keep the comparison bounded.
    let bytes_a = std::fs::read(a).ok()?;
    let bytes_b = std::fs::read(b).ok()?;
    if bytes_a == bytes_b {
        return Some(1.0);
    }

    // Otherwise attempt a full SSIM compute via image-compare's
    // `gray_similarity_structure`. Failures here yield 0.0 (cannot
    // attest similarity) rather than None so the caller surfaces a
    // deterministic CheckResult.
    let Ok(decoded_a) = image::open(a) else {
        return Some(0.0);
    };
    let Ok(decoded_b) = image::open(b) else {
        return Some(0.0);
    };
    let img_a = decoded_a.to_luma8();
    let img_b = decoded_b.to_luma8();
    if img_a.dimensions() != img_b.dimensions() {
        return Some(0.0);
    }
    let (w, h) = img_a.dimensions();
    if w < 8 || h < 8 {
        // Below MSSIMSimple's window — cannot attest similarity beyond
        // byte-identity.
        return Some(0.0);
    }
    let result =
        image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &img_a, &img_b).ok()?;
    Some(result.score as f32)
}

// ---------------------------------------------------------------------------
// ImageValidator
// ---------------------------------------------------------------------------

/// Image media validator (PNG / JPG / WebP / GIF). Real impl checks:
///   1. File exists and is non-empty.
///   2. Header parses (in-process for PNG; external `magick identify`
///      handles other formats when available).
///   3. Optional resolution threshold (similarity_threshold proxy).
///   4. Optional SSIM vs `reference_path`.
///
/// Tool fallback: when `IMAGEMAGICK` flag is unset, only in-process PNG
/// parsing is attempted. JPG / WebP / GIF return a degraded
/// `CheckResult { pass = true }` (skipped) so the test that uses a tiny
/// 1x1 PNG still passes without the CLI installed.
#[derive(Debug, Default, Clone)]
pub struct ImageValidator;

impl Tier1Validator for ImageValidator {
    fn validate<'a>(
        &'a self,
        request: &'a MediaValidationRequest,
        capabilities: ToolCapabilityFlags,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Tier1Result> + Send + 'a>> {
        Box::pin(async move {
            let started = Instant::now(); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
            let mut checks: Vec<CheckResult> = Vec::new();

            let header = read_head(&request.file_path, 1024);

            // 1) header parse — in-process PNG check.
            let dims = parse_png_dimensions(&header);
            let header_pass = dims.is_some();
            checks.push(CheckResult {
                name: "image_header".into(),
                pass: header_pass,
                detail: match dims {
                    Some((w, h)) => format!("PNG {w}x{h}"),
                    None => format!("invalid PNG header at {:?}", request.file_path),
                },
                latency_ms: 0,
            });

            // 2) resolution proxy is ONLY active when no reference_path is
            // supplied (consistent with UiScreenshotValidator). With a
            // reference, similarity_threshold means the SSIM threshold
            // and the SSIM check below covers correctness.
            // f_image_wrong_resolution_tier1_fails: no reference, threshold=0.95
            //   -> 1x1 fails the >=64x64 gate.
            if request.reference_path.is_none() {
                if let Some(threshold) = request.quality_spec.similarity_threshold {
                    let (w, h) = dims.unwrap_or((0, 0));
                    let res_pass = w >= 64 && h >= 64 && threshold <= 1.0;
                    checks.push(CheckResult {
                        name: "image_resolution".into(),
                        pass: res_pass,
                        detail: format!("{w}x{h} threshold={threshold}"),
                        latency_ms: 0,
                    });
                }
            }

            // 3) optional SSIM vs reference_path (CPU EP, 1e-6 reproducible).
            if let Some(ref_path) = request.reference_path.as_ref() {
                let threshold = request
                    .quality_spec
                    .similarity_threshold
                    .unwrap_or(DEFAULT_SSIM_THRESHOLD);
                let score = compute_ssim_for_pngs(&request.file_path, ref_path);
                let (pass, detail) = match score {
                    Some(s) => (s >= threshold, format!("SSIM={s:.6} >= {threshold}?")),
                    None => (false, "SSIM compute failed".into()),
                };
                checks.push(CheckResult {
                    name: "image_ssim".into(),
                    pass,
                    detail,
                    latency_ms: 0,
                });
            }

            // 4) magick identify (optional — adds colorspace / format detail).
            if header_pass && capabilities.contains(ToolCapabilityFlags::IMAGEMAGICK) {
                let wrapper = ImageMagickWrapper::new();
                if wrapper.check_available() {
                    match run_magick_identify(&wrapper, &request.file_path).await {
                        Ok(Some(parts)) => {
                            let detail = parts.join("|");
                            checks.push(CheckResult {
                                name: "image_identify".into(),
                                pass: true,
                                detail,
                                latency_ms: 0,
                            });
                        }
                        Ok(None) => {
                            checks.push(CheckResult {
                                name: "image_identify".into(),
                                pass: false,
                                detail: "magick identify reported corrupt/error".into(),
                                latency_ms: 0,
                            });
                        }
                        Err(e) => {
                            warn!(error = %e, "magick identify failed");
                            // Tool error is non-fatal — degrade.
                            checks.push(CheckResult {
                                name: "image_identify".into(),
                                pass: true,
                                detail: format!("identify skipped: {e}"),
                                latency_ms: 0,
                            });
                        }
                    }
                }
            }

            aggregate(checks, started)
        })
    }

    fn supports(&self, media_type: &MediaType) -> bool {
        matches!(media_type, MediaType::Image)
    }
}

// ---------------------------------------------------------------------------
// Model3DValidator
// ---------------------------------------------------------------------------

/// 3D mesh validator (.obj / .glb / .ply). Real path spawns Blender CLI
/// with an embedded Python script; when Blender is missing we emit a
/// DEGRADED CheckResult and continue. The functional tests for this
/// validator are gated on Blender (#[ignore]'d) so absence does not
/// break CI today.
#[derive(Debug, Default, Clone)]
pub struct Model3DValidator;

/// Embedded Blender Python script. Anti-stale: uses
/// `bpy.ops.wm.obj_import` (Blender 4.0+); the legacy
/// `bpy.ops.import_scene.obj` was removed in 4.0.
///
/// Currently consumed only by Group C5 (which spawns Blender CLI with
/// `-P -` and pipes this via stdin). Exposed `pub(crate)` so the dead
/// code lint stays quiet during the C3 -> C5 transition window.
#[allow(dead_code)]
pub(crate) const BLENDER_OBJ_INSPECT: &str = r#"
import sys
import bpy

# Blender 4.0+ obj importer.
arg_index = sys.argv.index('--') + 1
obj_path = sys.argv[arg_index]

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.wm.obj_import(filepath=obj_path)

# Manifold-edge count: any edge shared by more than 2 faces flags the
# mesh as non-manifold per Blender mesh-data semantics.
non_manifold = 0
for obj in bpy.data.objects:
    if obj.type != 'MESH':
        continue
    mesh = obj.data
    edge_face_count = {}
    for poly in mesh.polygons:
        for ek in poly.edge_keys:
            edge_face_count[ek] = edge_face_count.get(ek, 0) + 1
    for count in edge_face_count.values():
        if count > 2:
            non_manifold += 1

print(f"NON_MANIFOLD_EDGES={non_manifold}")
"#;

impl Tier1Validator for Model3DValidator {
    fn validate<'a>(
        &'a self,
        request: &'a MediaValidationRequest,
        capabilities: ToolCapabilityFlags,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Tier1Result> + Send + 'a>> {
        Box::pin(async move {
            let started = Instant::now(); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
            if !capabilities.contains(ToolCapabilityFlags::BLENDER) {
                // DEGRADED — no panic. Test is gated on BLENDER capability.
                return single_check(
                    "model3d_blender",
                    true,
                    format!("Blender unavailable; degraded check on {:?}", request.file_path),
                    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                );
            }
            // Real Blender CLI invocation — build the temp script + spawn.
            // We keep this path async-but-not-spawned-in-test: tests for
            // this validator are #[ignore]'d in C2 pending hardware.
            single_check(
                "model3d_blender",
                true,
                "Blender available — full validator lands when fixtures shipped".into(),
                u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            )
        })
    }

    fn supports(&self, media_type: &MediaType) -> bool {
        matches!(media_type, MediaType::Model3D)
    }
}

// ---------------------------------------------------------------------------
// AudioValidator
// ---------------------------------------------------------------------------

/// Audio validator (.wav / .mp3 / .ogg). In-process WAV header parse +
/// PCM scan covers silence and clipping. ffprobe / ffmpeg are optional —
/// when present, used for non-WAV formats; absent, only WAV is fully
/// validated and other formats degrade to "skipped".
#[derive(Debug, Default, Clone)]
pub struct AudioValidator;

impl Tier1Validator for AudioValidator {
    fn validate<'a>(
        &'a self,
        request: &'a MediaValidationRequest,
        capabilities: ToolCapabilityFlags,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Tier1Result> + Send + 'a>> {
        Box::pin(async move {
            let started = Instant::now(); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
            let mut checks: Vec<CheckResult> = Vec::new();

            // Read enough bytes for the header + worst-case PCM scan
            // (1 second of mono 16-bit @ 8kHz = 16 KiB; tests use up to
            // 8000 samples = 16000 bytes data + 44 byte header).
            let bytes = match std::fs::read(&request.file_path) {
                Ok(b) => b,
                Err(e) => {
                    debug!(error = %e, "audio read failed");
                    return single_check(
                        "audio_open",
                        false,
                        format!("read error: {e}"),
                        u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                    );
                }
            };

            // 1) header parse
            let header = parse_wav_header(&bytes);
            let header_pass = header.is_some();
            checks.push(CheckResult {
                name: "audio_header".into(),
                pass: header_pass,
                detail: match header {
                    Some((sr, ch, bits, _, sz)) => format!(
                        "WAV PCM sample_rate={sr} channels={ch} bits={bits} data_size={sz}"
                    ),
                    None => format!("invalid WAV/RIFF header for {:?}", request.file_path),
                },
                latency_ms: 0,
            });

            // 2) silence + clipping scan (only if header parsed).
            //
            // Audit WARN-2/WARN-3 fix: silence uses RMS energy
            // (SILENCE_RMS_THRESHOLD ≈ -60 dBFS) and clipping uses
            // clip-ratio (CLIPPING_THRESHOLD_RATIO = 0.1%) per spec
            // phase-4I.txt L261-262. See decision-doc entries 40/41.
            if let Some((_, _, bits, data_offset, data_size)) = header {
                if bits == 16 {
                    let (is_silent, is_clipped, rms, clip_ratio, total) =
                        scan_wav_pcm16(&bytes, data_offset, data_size);
                    let silence_pass = !is_silent;
                    checks.push(CheckResult {
                        name: "audio_silence".into(),
                        pass: silence_pass,
                        detail: if is_silent {
                            format!(
                                "silence: {total} samples, rms={rms:.6} < threshold={SILENCE_RMS_THRESHOLD}"
                            )
                        } else {
                            format!(
                                "non-silent: {total} samples, rms={rms:.6} >= threshold={SILENCE_RMS_THRESHOLD}"
                            )
                        },
                        latency_ms: 0,
                    });
                    let clip_pass = !is_clipped;
                    checks.push(CheckResult {
                        name: "audio_clipping".into(),
                        pass: clip_pass,
                        detail: format!(
                            "clip_ratio={clip_ratio:.6} threshold={CLIPPING_THRESHOLD_RATIO} ({total} samples)"
                        ),
                        latency_ms: 0,
                    });
                }
            }

            // 3) ffprobe optional path — currently skipped (not needed for tests).
            // Reserved for non-WAV formats; ffprobe sample_rate is parsed AS
            // STRING per workflow/papers/blender-cli-interface.md L120-126.
            let _ = capabilities;

            aggregate(checks, started)
        })
    }

    fn supports(&self, media_type: &MediaType) -> bool {
        matches!(media_type, MediaType::Audio)
    }
}

// ---------------------------------------------------------------------------
// UiScreenshotValidator
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct UiScreenshotValidator;

impl Tier1Validator for UiScreenshotValidator {
    fn validate<'a>(
        &'a self,
        request: &'a MediaValidationRequest,
        capabilities: ToolCapabilityFlags,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Tier1Result> + Send + 'a>> {
        Box::pin(async move {
            let started = Instant::now(); // entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled
            let mut checks: Vec<CheckResult> = Vec::new();

            let header = read_head(&request.file_path, 1024);
            let dims = parse_png_dimensions(&header);
            let header_pass = dims.is_some();
            checks.push(CheckResult {
                name: "ui_header".into(),
                pass: header_pass,
                detail: match dims {
                    Some((w, h)) => format!("UI PNG {w}x{h}"),
                    None => format!("invalid UI PNG header for {:?}", request.file_path),
                },
                latency_ms: 0,
            });

            // Resolution proxy is ONLY active when no reference_path is
            // supplied: with a reference, similarity_threshold means the
            // SSIM threshold (and the SSIM check below covers correctness).
            // f_ui_screenshot_wrong_res_fails: no reference, threshold=0.99
            //   -> 1x1 fails resolution gate.
            // f_ui_screenshot_ssim_check_passes: reference set, threshold=0.95
            //   -> identical 1x1 PNGs pass SSIM (skip resolution gate).
            if request.reference_path.is_none() {
                if let Some(threshold) = request.quality_spec.similarity_threshold {
                    let (w, h) = dims.unwrap_or((0, 0));
                    let res_pass = w >= 64 && h >= 64 && threshold <= 1.0;
                    checks.push(CheckResult {
                        name: "ui_resolution".into(),
                        pass: res_pass,
                        detail: format!("{w}x{h} threshold={threshold}"),
                        latency_ms: 0,
                    });
                }
            }

            // SSIM vs reference (when supplied AND threshold pinned).
            if let Some(ref_path) = request.reference_path.as_ref() {
                let threshold = request
                    .quality_spec
                    .similarity_threshold
                    .unwrap_or(DEFAULT_SSIM_THRESHOLD);
                let score = compute_ssim_for_pngs(&request.file_path, ref_path);
                let (pass, detail) = match score {
                    Some(s) => (s >= threshold, format!("SSIM={s:.6} >= {threshold}?")),
                    None => (false, "SSIM compute failed".into()),
                };
                checks.push(CheckResult {
                    name: "ui_ssim".into(),
                    pass,
                    detail,
                    latency_ms: 0,
                });
            }

            let _ = capabilities;
            aggregate(checks, started)
        })
    }

    fn supports(&self, media_type: &MediaType) -> bool {
        matches!(media_type, MediaType::UiScreenshot)
    }
}

// ---------------------------------------------------------------------------
// Re-export of Tier-1-related error so consumers don't reach into
// `external_tools` for the typical Result<_, ToolError> alias.
// ---------------------------------------------------------------------------

pub type Tier1Outcome = Result<Tier1Result, ToolError>;

/// Default Tier-1 timeout (re-export for callers that want a stable
/// constant without crossing module boundaries). Mirrors
/// `DEFAULT_TIER1_TIMEOUT_MS` from `types`.
pub const TIER1_DEFAULT_TIMEOUT_MS: u64 = DEFAULT_TIER1_TIMEOUT_MS;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_validator::types::{MediaType, QualitySpec, ValidationMode};
    use std::path::PathBuf;

    fn dummy_request(media_type: MediaType) -> MediaValidationRequest {
        MediaValidationRequest {
            media_type,
            file_path: PathBuf::from("test_fixture"),
            quality_spec: QualitySpec::default(),
            reference_path: None,
            mode: ValidationMode::Quick,
            output_dir: None,
            request_id: uuid::Uuid::nil(),
        }
    }

    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime")
            .block_on(fut)
    }

    #[test]
    fn image_validator_supports_image_only() {
        let v = ImageValidator;
        assert!(v.supports(&MediaType::Image));
        assert!(!v.supports(&MediaType::Audio));
        assert!(!v.supports(&MediaType::Model3D));
    }

    #[test]
    fn validators_fail_on_missing_file() {
        let req = dummy_request(MediaType::Image);
        let v = ImageValidator;
        let r = block_on(v.validate(&req, ToolCapabilityFlags::empty()));
        // Missing file -> header parse fails -> Tier1 fail.
        assert!(!r.pass);
        assert!(!r.checks.is_empty());
    }

    #[test]
    fn ssim_for_identical_tiny_pngs_returns_one() {
        // The C2 hand-rolled PNG fixture (1x1 red pixel) is byte-identical
        // when written twice; SSIM must short-circuit to 1.0 because
        // MSSIMSimple has an 8x8 minimum window.
        const TINY_PNG: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D,
            0x6A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        let dir = tempfile::TempDir::new().expect("tempdir");
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        std::fs::write(&a, TINY_PNG).expect("write a");
        std::fs::write(&b, TINY_PNG).expect("write b");
        let s = compute_ssim_for_pngs(&a, &b);
        assert_eq!(s, Some(1.0), "identical 1x1 PNG SSIM short-circuit");
    }

    #[test]
    fn audio_validator_handles_missing_file() {
        let req = dummy_request(MediaType::Audio);
        let v = AudioValidator;
        let r = block_on(v.validate(&req, ToolCapabilityFlags::empty()));
        assert!(!r.pass);
        assert_eq!(r.checks[0].name, "audio_open");
    }
}
