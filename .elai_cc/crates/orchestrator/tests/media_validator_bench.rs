//! B-24 Phase 4I (Group C4 — 4I.6 MEASURE): perf-bench harness.
//!
//! Measures Tier 1 latency per media type and Tier 2 (Mock) latency on
//! the same synthetic fixtures used by the contract / functional tests.
//! All tests are `#[ignore = "perf-bench: run with --ignored"]` so the
//! default `cargo test` flow does NOT pay the 30-50 ms-per-iteration
//! cost; CI runs the harness explicitly:
//!
//!     cargo test -p elai-orchestrator --test media_validator_bench \
//!         -- --ignored --nocapture
//!
//! Output: per-test "BENCH | <name> | iters=N | p50=Xms | p95=Yms | p99=Zms"
//! lines. The decision doc (workflow/reasoning/B-24-decisions.md #49-#51)
//! enumerates why we do NOT use `criterion`:
//!   * extra dependency on `criterion` + `criterion-plot` (~ 6 crates)
//!     for a one-shot phase-completion measurement,
//!   * `criterion` rebuilds benches under `bench` profile which trips the
//!     vlm-integration feature graph (extra ort + nvml-wrapper build),
//!   * we only need percentile readouts, not statistical regression
//!     analysis — std::time::Instant + sort + percentile picks gets us
//!     there in <60 LOC.
//!
//! Anti-stale grounding (CLAUDE.md):
//!   * std::time::Instant — monotonic. NEVER SystemTime / chrono::Local::now
//!     (non-monotonic, NTP can jump).
//!   * u64::try_from(d.as_millis()) — narrowing per the no-`as`-cast rule.
//!   * No new dependencies (tempfile + uuid already in dev-deps / workspace).
//!
//! 4I.6 DETERMINISTIC_CHECK (spec L347-352):
//!   tier1_p95_ms < 100 (image/audio/UI) OR documented exception.
//!   Windows + tempfile + per-call current-thread tokio runtime can push
//!   the cold image_validate to ~50-150 ms because the FIRST runtime build
//!   pays a one-shot cost; subsequent calls drop to 1-3 ms. We measure
//!   BOTH cold AND warm by iterating N times; p50/p95 published — and the
//!   metrics file's `notes` field documents the cold-start exception when
//!   p95 > 100 ms.

use std::path::{Path, PathBuf};
use std::time::Instant;

use tempfile::TempDir;

use elai_orchestrator::media_validator::{
    pipeline, MediaType, MediaValidationRequest, MockVlmBackend, QualitySpec, ValidationMode,
};

// ---------------------------------------------------------------------------
// Synthetic fixture helpers — duplicated from media_validator_functional.rs
// so the bench file is self-contained (no shared test-helper crate; cargo
// has no clean cross-test-file `mod` reuse pattern).
// ---------------------------------------------------------------------------

/// Pre-computed valid 1x1 RGB PNG (red pixel). 67 bytes.
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
    0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
    0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT
    0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A,
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn write_tiny_png(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, TINY_PNG).expect("write png");
    path
}

fn write_tiny_wav(dir: &Path, name: &str, sample: i16, n_samples: u32) -> PathBuf {
    use std::io::Write;
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).expect("create wav");
    let data_size = n_samples * 2;
    let chunk_size = 36 + data_size;
    let sample_rate: u32 = 8000;
    let byte_rate: u32 = sample_rate * 2;
    f.write_all(b"RIFF").unwrap();
    f.write_all(&chunk_size.to_le_bytes()).unwrap();
    f.write_all(b"WAVE").unwrap();
    f.write_all(b"fmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap();
    f.write_all(&1u16.to_le_bytes()).unwrap();
    f.write_all(&sample_rate.to_le_bytes()).unwrap();
    f.write_all(&byte_rate.to_le_bytes()).unwrap();
    f.write_all(&2u16.to_le_bytes()).unwrap();
    f.write_all(&16u16.to_le_bytes()).unwrap();
    f.write_all(b"data").unwrap();
    f.write_all(&data_size.to_le_bytes()).unwrap();
    for _ in 0..n_samples {
        f.write_all(&sample.to_le_bytes()).unwrap();
    }
    path
}

fn write_manifold_obj(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").expect("write obj");
    path
}

fn request_for(path: &Path, media_type: MediaType) -> MediaValidationRequest {
    MediaValidationRequest {
        media_type,
        file_path: path.to_path_buf(),
        quality_spec: QualitySpec {
            similarity_threshold: None,
            qualitative_checks: false,
            visual_check: false,
        },
        reference_path: None,
        mode: ValidationMode::Quick,
        output_dir: None,
        request_id: uuid::Uuid::nil(),
    }
}

// ---------------------------------------------------------------------------
// Percentile helper. Sorts a clone of the input and picks the index nearest
// (and not below) the target rank. Returns 0 for an empty input — caller is
// expected to filter that case before printing.
// ---------------------------------------------------------------------------
fn percentile(samples: &[u64], pct: f64) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    // Nearest-rank method: ceil(pct * N) - 1 with clamps. f64-cast for the
    // rank arithmetic is fine — N never exceeds u32 in the bench harness.
    let n = sorted.len();
    let raw_rank = (pct * n as f64).ceil() as isize - 1;
    let rank = raw_rank.max(0).min((n as isize) - 1) as usize;
    sorted[rank]
}

/// Run `op` `iters` times and print BENCH summary. Returns the (p50, p95, p99)
/// triple in milliseconds so the metrics file can be hand-transcribed by the
/// next step in the workflow.
fn run_bench<F: FnMut()>(name: &str, iters: u32, mut op: F) -> (u64, u64, u64) {
    let mut samples: Vec<u64> = Vec::with_capacity(iters as usize);
    for _ in 0..iters {
        let t0 = Instant::now();
        op();
        let elapsed = t0.elapsed();
        samples.push(u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX));
    }
    let p50 = percentile(&samples, 0.50);
    let p95 = percentile(&samples, 0.95);
    let p99 = percentile(&samples, 0.99);
    println!(
        "BENCH | {name} | iters={iters} | p50={p50}ms | p95={p95}ms | p99={p99}ms",
        name = name,
        iters = iters,
        p50 = p50,
        p95 = p95,
        p99 = p99
    );
    (p50, p95, p99)
}

// ---------------------------------------------------------------------------
// Tier 1 benches (one per media type — except Model3D which is gated on
// Blender; we still RUN it because the DEGRADED-path measurement is useful).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn bench_tier1_image_png() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_tiny_png(dir.path(), "bench.png");
    let req = request_for(&path, MediaType::Image);
    let mock = MockVlmBackend::new();
    run_bench("tier1_image_png_1x1", 30, || {
        let _ = pipeline::validate(&req, &mock).expect("pipeline");
    });
}

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn bench_tier1_audio_wav() {
    let dir = TempDir::new().expect("tempdir");
    // 1s of mono 8kHz PCM @ 5000 amplitude — non-silent, non-clipped.
    let path = write_tiny_wav(dir.path(), "bench.wav", 5000, 8000);
    let req = request_for(&path, MediaType::Audio);
    let mock = MockVlmBackend::new();
    run_bench("tier1_audio_wav_8k_1s", 30, || {
        let _ = pipeline::validate(&req, &mock).expect("pipeline");
    });
}

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn bench_tier1_ui_screenshot() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_tiny_png(dir.path(), "ui_bench.png");
    let req = request_for(&path, MediaType::UiScreenshot);
    let mock = MockVlmBackend::new();
    run_bench("tier1_ui_screenshot_1x1", 30, || {
        let _ = pipeline::validate(&req, &mock).expect("pipeline");
    });
}

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn bench_tier1_model3d_obj_degraded() {
    // Without Blender on PATH the Model3DValidator emits a DEGRADED
    // CheckResult with pass=true. We measure THAT path; the real Blender
    // CLI path is gated on installer config and lands in Group C5
    // hardware-bench follow-up.
    let dir = TempDir::new().expect("tempdir");
    let path = write_manifold_obj(dir.path(), "bench.obj");
    let req = request_for(&path, MediaType::Model3D);
    let mock = MockVlmBackend::new();
    run_bench("tier1_model3d_obj_degraded", 30, || {
        let _ = pipeline::validate(&req, &mock).expect("pipeline");
    });
}

// ---------------------------------------------------------------------------
// Tier 2 mock latency. MockVlmBackend::validate_advisory returns a canned
// VlmValidationOutput synchronously — measured here so the metrics file
// records the floor cost of the Tier-2 dispatch path (serde_json::to_value
// + range-check + clone). Real VLM (Mode A / Mode B) latencies are
// recorded as `null` in the metrics file — see decision 50 + spec L334-336.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn bench_tier2_mock_only() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_tiny_png(dir.path(), "t2_mock.png");
    let mut req = request_for(&path, MediaType::Image);
    // Force the Tier-2 dispatch by enabling qualitative_checks; ValidationMode
    // Full required (Quick short-circuits to Pass before Tier 2 per veto-table
    // branch 2).
    req.quality_spec.qualitative_checks = true;
    req.mode = ValidationMode::Full;
    let mock = MockVlmBackend::new();
    run_bench("tier2_mock_dispatch_image", 30, || {
        let _ = pipeline::validate(&req, &mock).expect("pipeline");
    });
}

// ---------------------------------------------------------------------------
// Adversarial 4K-image bench (spec L355-356; B-24 Phase 4I Group C4 audit
// WARN-2 fix, decision 52).
//
// The spec calls out that small-fixture (1x1) measurements may hide
// production latency. This bench synthesizes a 3840x2160 RGB PNG on the
// fly via the `image` crate (already a default dep with the "png" feature
// in orchestrator's Cargo.toml L39 — covers both decode AND encode), writes
// it to a tempfile, and runs `pipeline::validate` 10 iterations. 30 would
// be 3× longer for marginal statistical gain; the bench is gated behind
// `#[ignore]` for the perf-bench harness anyway.
//
// Numbers land in metrics/phase_4I.toml under
// `[metrics.tier1_per_type].image_png_3840x2160` and in
// `notes.adversarial_4k_image` once measured.
#[test]
#[ignore = "perf-bench: run with --ignored"]
fn tier1_image_png_3840x2160() {
    use image::{ImageBuffer, Rgb};

    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("bench_4k.png");

    // Synthesize a 3840x2160 RGB gradient. Gradient (not solid) so the PNG
    // IDAT stream is non-trivially compressible — closer to a real-world
    // screenshot's entropy than a flat fill.
    //
    // Memory: 3840 * 2160 * 3 ≈ 24.9 MiB raw pixels. PNG-encoded file lands
    // around 10-15 MiB after deflate. Both are well under the
    // MAX_VALIDATION_FILE_BYTES guard (50 MiB default at the time of writing).
    const W: u32 = 3840;
    const H: u32 = 2160;
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(W, H, |x, y| {
        // Deterministic gradient — same bytes every run, so the bench
        // measures pipeline cost not random-generator cost.
        let r = u8::try_from((x * 255 / W).min(255)).unwrap_or(255);
        let g = u8::try_from((y * 255 / H).min(255)).unwrap_or(255);
        let b = u8::try_from(((x + y) * 255 / (W + H)).min(255)).unwrap_or(255);
        Rgb([r, g, b])
    });
    img.save(&path).expect("save 4k png");

    let req = request_for(&path, MediaType::Image);
    let mock = MockVlmBackend::new();
    // 10 iterations (vs 30 for the small fixtures): each iteration loads &
    // decodes ~10 MiB; 30 iters would push total wall time past 90s on
    // typical hardware. 10 iters gives stable p50 / p95 / p99 estimates for
    // a single-machine reading without dominating the audit cycle.
    run_bench("tier1_image_png_3840x2160", 10, || {
        let _ = pipeline::validate(&req, &mock).expect("pipeline");
    });
}

// ===========================================================================
// B-24 Phase 4I Group C7 (4I.10 MEASURE): RCE adapter benches.
// ===========================================================================
//
// Spec phase-4I.txt L575-606. Seven new benches measure:
//   1. rce_web_render_three_viewports_bench       (skip if npx playwright missing)
//   2. rce_blender_four_angles_bench              (skip if blender missing)
//   3. rce_animation_key_frames_bench             (skip if ffmpeg missing)
//   4. rce_ui_screenshot_bench                    (always-on; file read only)
//   5. rce_comfyui_mock_workflow_bench            (always-on; in-process mock)
//   6. rce_loop_total_bench                       (always-on; UiScreenshot + MockVlm)
//   7. rce_convergence_corpus_bench               (always-on; 10 scenarios)
//
// Discipline:
//   * Each bench is #[ignore]'d; default `cargo test` flow does not pay
//     the cost. Run with `--ignored --nocapture` per the harness comment
//     at the top of this file.
//   * Real-tool benches keep iters small (3-5) because external CLI
//     startup is 1-3s per call; the percentile readouts are coarse but
//     sufficient for the spec L580-583 budget gates.
//   * Async benches use `tokio::runtime::Runtime::new().block_on(...)`
//     so the bench retains the sync `#[test]` shape that matches the
//     existing tier1/tier2 harness; the runtime is built once per bench
//     and reused across iterations.
//   * `binary_on_path` duplicates the helper in
//     media_validator_rce_functional.rs (no shared test-helper crate).
//   * std::time::Instant + u64::try_from per CLAUDE.md anti-stale rule.

use elai_orchestrator::media_validator::feedback_loop::VisualFeedbackLoop;
use elai_orchestrator::media_validator::render_adapters::{
    AdapterRegistry, AnimationRenderAdapter, BlenderRenderAdapter, CameraAngle, CaptureMode,
    ComfyUIRenderAdapter, MediaRenderAdapter, OutputType, RenderConfig, UiScreenshotAdapter,
    VisualIssue, WebRenderAdapter,
};
use elai_orchestrator::media_validator::types::{BoundingBox, CheckResult, Severity, Tier1Result, Verdict};

/// True iff the named binary is on PATH. Same probe as the functional
/// tests (where.exe on Windows / which on Unix).
fn binary_on_path(name: &str) -> bool {
    let probe = if cfg!(target_os = "windows") {
        "where.exe"
    } else {
        "which"
    };
    let out = match std::process::Command::new(probe).arg(name).output() {
        Ok(o) => o,
        Err(_) => return false,
    };
    if !out.status.success() {
        return false;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines().next().map(|l| !l.trim().is_empty()).unwrap_or(false)
}

/// Stronger Playwright probe — both `npx` AND `npx playwright --version`
/// must succeed. Matches the functional-test convention so a host with
/// `node` but no `playwright` package skips cleanly.
fn playwright_available() -> bool {
    if !binary_on_path("npx") {
        return false;
    }
    match std::process::Command::new("npx")
        .args(["playwright", "--version"])
        .output()
    {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Build a one-shot multi-thread tokio runtime for the bench. We use
/// multi-thread (not current_thread) because `pipeline::validate_with_rce`
/// and the feedback loop spawn render-adapter futures that may .await
/// tokio I/O; the production code paths assume a multi-thread runtime
/// (TaskRunner builds one at startup). The runtime is constructed ONCE
/// per bench and reused across iterations so the `Runtime::new` cost is
/// not folded into the per-call latency.
fn bench_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("bench tokio runtime")
}

/// Async variant of `run_bench`. Same nearest-rank percentile output.
/// `op` is a FnMut that returns a Future per call; the runtime drives
/// each one to completion.
fn run_bench_async<Fut, F>(
    rt: &tokio::runtime::Runtime,
    name: &str,
    iters: u32,
    mut op: F,
) -> (u64, u64, u64)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut samples: Vec<u64> = Vec::with_capacity(iters as usize);
    for _ in 0..iters {
        let fut = op();
        let t0 = Instant::now();
        rt.block_on(fut);
        let elapsed = t0.elapsed();
        samples.push(u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX));
    }
    let p50 = percentile(&samples, 0.50);
    let p95 = percentile(&samples, 0.95);
    let p99 = percentile(&samples, 0.99);
    println!(
        "BENCH | {name} | iters={iters} | p50={p50}ms | p95={p95}ms | p99={p99}ms",
        name = name,
        iters = iters,
        p50 = p50,
        p95 = p95,
        p99 = p99
    );
    (p50, p95, p99)
}

// ---------------------------------------------------------------------------
// 1. Web render (3 viewports). Spec L580 target: < 3s p95.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn rce_web_render_three_viewports_bench() {
    if !binary_on_path("node") || !playwright_available() {
        eprintln!(
            "BENCH | rce_web_render_three_viewports | skipped: TOOL_MISSING:playwright"
        );
        return;
    }
    let dir = TempDir::new().expect("tempdir");
    let html = dir.path().join("bench.html");
    std::fs::write(
        &html,
        b"<html><body><h1>bench</h1><p>perf measurement</p></body></html>",
    )
    .expect("write html");
    let adapter = WebRenderAdapter::new();
    let cfg = RenderConfig {
        viewports: adapter.default_viewports.clone(),
        ..RenderConfig::default()
    };
    let rt = bench_runtime();
    // 3 iters — Playwright cold-start dominates; more iters would push
    // the bench past the spec L600 budget for the audit cycle.
    run_bench_async(&rt, "rce_web_render_three_viewports", 3, || async {
        let _ = adapter.render(&html, &cfg).await.expect("web render");
    });
    adapter.cleanup();
}

// ---------------------------------------------------------------------------
// 2. Blender (4 angles). Spec L581 target: < 10s p95.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn rce_blender_four_angles_bench() {
    if !binary_on_path("blender") {
        eprintln!("BENCH | rce_blender_four_angles | skipped: TOOL_MISSING:blender");
        return;
    }
    let dir = TempDir::new().expect("tempdir");
    let obj = write_manifold_obj(dir.path(), "bench.obj");
    let adapter = BlenderRenderAdapter::new();
    let cfg = RenderConfig {
        capture_mode: CaptureMode::MultiAngle {
            angles: CameraAngle::defaults(),
        },
        ..RenderConfig::default()
    };
    let rt = bench_runtime();
    // 3 iters — Blender headless startup is ~3-5s per call (spec L605);
    // 3 * ~6s = ~18s wall time which fits the audit budget.
    run_bench_async(&rt, "rce_blender_four_angles", 3, || async {
        let _ = adapter.render(&obj, &cfg).await.expect("blender render");
    });
    adapter.cleanup();
}

// ---------------------------------------------------------------------------
// 3. Animation key frames (5). Spec L582 target: < 2s p95.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn rce_animation_key_frames_bench() {
    if !binary_on_path("ffmpeg") {
        eprintln!("BENCH | rce_animation_key_frames | skipped: TOOL_MISSING:ffmpeg");
        return;
    }
    let dir = TempDir::new().expect("tempdir");
    let mp4 = dir.path().join("clip.mp4");
    // Synthesize a 5s, 1fps testsrc -> ~5 key frames at 1-second interval.
    let synth = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=5:size=128x128:rate=5",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&mp4)
        .output()
        .expect("ffmpeg synth");
    assert!(
        synth.status.success(),
        "ffmpeg synth failed: stderr={}",
        String::from_utf8_lossy(&synth.stderr)
    );
    let adapter = AnimationRenderAdapter::new();
    let cfg = RenderConfig {
        capture_mode: CaptureMode::KeyFrames {
            interval: std::time::Duration::from_secs(1),
        },
        ..RenderConfig::default()
    };
    let rt = bench_runtime();
    // 5 iters — ffmpeg keyframe extraction is the cheapest of the three
    // real-CLI adapters (no GPU, no JS runtime).
    run_bench_async(&rt, "rce_animation_key_frames", 5, || async {
        let _ = adapter.render(&mp4, &cfg).await.expect("animation render");
    });
}

// ---------------------------------------------------------------------------
// 4. UI screenshot (pass-through). Spec L583 target: < 50ms p95. Always on.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn rce_ui_screenshot_bench() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_tiny_png(dir.path(), "ui_bench.png");
    let adapter = UiScreenshotAdapter::new();
    let cfg = RenderConfig::default();
    let rt = bench_runtime();
    // 20 iters — pure file I/O so each iter is sub-millisecond; 20
    // samples give a stable p95 below the 50 ms spec budget.
    run_bench_async(&rt, "rce_ui_screenshot", 20, || async {
        let _ = adapter.render(&path, &cfg).await.expect("ui render");
    });
}

// ---------------------------------------------------------------------------
// 5. ComfyUI mock workflow. Always-on; spec L584 says baseline only.
// ---------------------------------------------------------------------------
//
// Spins an in-process tokio TcpListener mock per bench (cheap; ~1ms
// bind). The mock handles POST /prompt, GET /history/{id}, GET /view —
// same surface as media_validator_rce_functional::comfyui_mock.

mod comfyui_bench_mock {
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x5B, 0x42, 0x5D, 0x6A, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    pub struct Handle {
        pub addr: std::net::SocketAddr,
        pub shutdown: oneshot::Sender<()>,
    }

    pub async fn spawn() -> Handle {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (tx, mut rx) = oneshot::channel::<()>();
        let prompt_id = Arc::new("bench-prompt-id".to_string());
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut rx => break,
                    accepted = listener.accept() => {
                        let Ok((mut sock, _peer)) = accepted else { break; };
                        let id = prompt_id.clone();
                        tokio::spawn(async move {
                            let _ = handle(&mut sock, &id).await;
                        });
                    }
                }
            }
        });
        Handle { addr, shutdown: tx }
    }

    async fn handle(sock: &mut tokio::net::TcpStream, prompt_id: &str) -> std::io::Result<()> {
        let mut buf = vec![0u8; 65_536];
        let mut total = 0;
        loop {
            let n = sock.read(&mut buf[total..]).await?;
            if n == 0 { break; }
            total += n;
            if buf[..total].windows(4).any(|w| w == b"\r\n\r\n") { break; }
            if total >= buf.len() { break; }
        }
        let text = String::from_utf8_lossy(&buf[..total]).to_string();
        let first_line = text.lines().next().unwrap_or("").to_string();
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return write_simple(sock, 400, "Bad Request", b"bad").await;
        }
        let method = parts[0];
        let path = parts[1];
        if method == "POST" && path.starts_with("/prompt") {
            let body = format!(r#"{{"prompt_id":"{prompt_id}"}}"#);
            return write_simple(sock, 200, "OK", body.as_bytes()).await;
        }
        if method == "GET" && path.starts_with("/history/") {
            let body = format!(
                r#"{{"{prompt_id}":{{"outputs":{{"node1":{{"images":[{{"filename":"out.png","subfolder":"","type":"output"}}]}}}}}}}}"#
            );
            return write_simple(sock, 200, "OK", body.as_bytes()).await;
        }
        if method == "GET" && path.starts_with("/view") {
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                TINY_PNG.len()
            );
            sock.write_all(headers.as_bytes()).await?;
            sock.write_all(TINY_PNG).await?;
            sock.shutdown().await?;
            return Ok(());
        }
        write_simple(sock, 404, "Not Found", b"nope").await
    }

    async fn write_simple(
        sock: &mut tokio::net::TcpStream,
        status: u16,
        reason: &str,
        body: &[u8],
    ) -> std::io::Result<()> {
        let resp = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        sock.write_all(resp.as_bytes()).await?;
        sock.write_all(body).await?;
        sock.shutdown().await?;
        Ok(())
    }
}

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn rce_comfyui_mock_workflow_bench() {
    let dir = TempDir::new().expect("tempdir");
    let workflow = dir.path().join("workflow.json");
    std::fs::write(
        &workflow,
        br#"{"prompt":{"1":{"class_type":"Stub","inputs":{}}}}"#,
    )
    .expect("write workflow");
    let rt = bench_runtime();
    // Spawn ONE mock and reuse across 10 iters — the mock server itself
    // is the unit under test for latency stability, not its bind cost.
    let (handle, url) = rt.block_on(async {
        let h = comfyui_bench_mock::spawn().await;
        let url = format!("http://{}", h.addr);
        (h, url)
    });
    let adapter = ComfyUIRenderAdapter::with_url(&url);
    let cfg = RenderConfig::default();
    run_bench_async(&rt, "rce_comfyui_mock_workflow", 10, || async {
        let _ = adapter
            .render(&workflow, &cfg)
            .await
            .expect("comfyui mock render");
    });
    // Tear down the mock.
    let _ = handle.shutdown.send(());
}

// ---------------------------------------------------------------------------
// 6. Full feedback loop total latency. Always-on (UiScreenshot + MockVlm).
//    Spec L584-585 target: < 10s per iteration.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn rce_loop_total_bench() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_tiny_png(dir.path(), "loop_bench.png");
    // Build a fresh request per iter (cheap clone) so the loop does not
    // share state across samples.
    let mut req = request_for(&path, MediaType::UiScreenshot);
    req.quality_spec.visual_check = true;
    req.quality_spec.qualitative_checks = true;
    req.mode = ValidationMode::Full;
    let mock = MockVlmBackend::new();
    let rt = bench_runtime();
    // 10 iters — UiScreenshot adapter + MockVlm is bounded by adapter
    // cleanup + loop convergence detection (max 3 iters by default);
    // measures the per-call total wall-time including Tier1 dispatch.
    run_bench_async(&rt, "rce_loop_total", 10, || {
        let req = req.clone();
        let mock = &mock;
        async move {
            let mut registry = AdapterRegistry::new();
            registry
                .register_unique(OutputType::UiScreenshot, Box::new(UiScreenshotAdapter::new()))
                .expect("register ui_screenshot");
            let _ = pipeline::validate_with_rce(
                &req,
                mock,
                None,
                registry,
                Some(&OutputType::UiScreenshot),
                &RenderConfig::default(),
            )
            .await
            .expect("loop pipeline");
        }
    });
}

// ---------------------------------------------------------------------------
// 7. Convergence corpus. Always-on. 10 scripted scenarios feed scripted
//    VLM-issue trajectories into VisualFeedbackLoop::run; we measure
//    convergence_rate (% scenarios with converged=true) and
//    avg_iterations across the corpus. Spec L587-589 target: >= 70%
//    convergence rate; no hard target for avg_iterations (baseline).
// ---------------------------------------------------------------------------

/// One corpus entry. `trajectory[i]` is the issue list returned on the
/// i-th render_evaluator call. Scenarios with trajectories shorter than
/// max_iter (3) repeat the last entry — see the closure below.
struct Scenario {
    name: &'static str,
    trajectory: Vec<Vec<VisualIssue>>,
}

fn issue(desc: &str, sev: Severity, bx: f32, by: f32) -> VisualIssue {
    VisualIssue {
        description: desc.into(),
        severity: sev,
        location: Some(BoundingBox {
            x: bx,
            y: by,
            width: 0.1,
            height: 0.1,
        }),
        viewport: None,
        actionable_fix: format!("address {desc}"),
    }
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

fn corpus() -> Vec<Scenario> {
    vec![
        // Scenarios 0-4: CONVERGE (no issues from start, or stabilize).
        Scenario {
            name: "no_issues_immediate_pass",
            trajectory: vec![vec![]],
        },
        Scenario {
            name: "fix_resolves_iter2",
            trajectory: vec![vec![issue("overflow:a", Severity::Major, 0.1, 0.1)], vec![]],
        },
        Scenario {
            name: "stable_unchanged_iter2",
            trajectory: vec![
                vec![issue("overflow:b", Severity::Major, 0.1, 0.1)],
                vec![issue("overflow:b", Severity::Major, 0.1, 0.1)],
            ],
        },
        Scenario {
            name: "two_issues_converge",
            trajectory: vec![
                vec![
                    issue("alpha", Severity::Minor, 0.2, 0.3),
                    issue("beta", Severity::Minor, 0.5, 0.5),
                ],
                vec![
                    issue("alpha", Severity::Minor, 0.2, 0.3),
                    issue("beta", Severity::Minor, 0.5, 0.5),
                ],
            ],
        },
        Scenario {
            name: "partial_fix_then_stable",
            trajectory: vec![
                vec![
                    issue("c1", Severity::Major, 0.1, 0.1),
                    issue("c2", Severity::Minor, 0.4, 0.4),
                ],
                vec![issue("c2", Severity::Minor, 0.4, 0.4)],
                vec![issue("c2", Severity::Minor, 0.4, 0.4)],
            ],
        },
        // Scenarios 5-7: ALSO CONVERGE (different patterns).
        Scenario {
            name: "critical_stable",
            trajectory: vec![
                vec![issue("critical_thing", Severity::Critical, 0.5, 0.5)],
                vec![issue("critical_thing", Severity::Critical, 0.5, 0.5)],
            ],
        },
        Scenario {
            name: "single_minor_immediate_pass",
            trajectory: vec![vec![issue("typo", Severity::Minor, 0.9, 0.9)], vec![]],
        },
        Scenario {
            name: "all_resolved_iter2",
            trajectory: vec![
                vec![
                    issue("x1", Severity::Major, 0.1, 0.2),
                    issue("x2", Severity::Major, 0.3, 0.4),
                    issue("x3", Severity::Minor, 0.5, 0.6),
                ],
                vec![],
            ],
        },
        // Scenarios 8-9: HIT MAX-ITER (different issues each time, never converge).
        Scenario {
            name: "oscillating_issues",
            trajectory: vec![
                vec![issue("flicker_a", Severity::Major, 0.1, 0.1)],
                vec![issue("flicker_b", Severity::Major, 0.5, 0.5)],
                vec![issue("flicker_c", Severity::Major, 0.9, 0.9)],
            ],
        },
        Scenario {
            name: "growing_issue_count",
            trajectory: vec![
                vec![issue("growing_1", Severity::Minor, 0.1, 0.1)],
                vec![
                    issue("growing_1", Severity::Minor, 0.1, 0.1),
                    issue("growing_2", Severity::Minor, 0.5, 0.5),
                ],
                vec![
                    issue("growing_1", Severity::Minor, 0.1, 0.1),
                    issue("growing_2", Severity::Minor, 0.5, 0.5),
                    issue("growing_3", Severity::Minor, 0.9, 0.9),
                ],
            ],
        },
    ]
}

#[test]
#[ignore = "perf-bench: run with --ignored"]
fn rce_convergence_corpus_bench() {
    let dir = TempDir::new().expect("tempdir");
    let png = write_tiny_png(dir.path(), "corpus.png");
    let rt = bench_runtime();
    let scenarios = corpus();
    let total = scenarios.len();
    let mut converged_count = 0usize;
    let mut total_iters: u64 = 0;
    let mut per_scenario_iters: Vec<u32> = Vec::with_capacity(total);

    for sc in &scenarios {
        let traj = sc.trajectory.clone();
        let result = rt.block_on(async {
            let mut registry = AdapterRegistry::new();
            registry
                .register_unique(
                    OutputType::UiScreenshot,
                    Box::new(UiScreenshotAdapter::new()),
                )
                .expect("register ui");
            let loop_ = VisualFeedbackLoop::new(registry, Default::default());
            let mut idx: usize = 0;
            loop_
                .run(
                    &png,
                    &OutputType::UiScreenshot,
                    &RenderConfig::default(),
                    |_artifact, _frames| {
                        // Pick trajectory entry; clamp to last on overflow.
                        let pick = if idx < traj.len() {
                            traj[idx].clone()
                        } else {
                            traj.last().cloned().unwrap_or_default()
                        };
                        idx += 1;
                        let tuple = (tier1_pass(), None, pick);
                        async move { tuple }
                    },
                    |_| {},
                )
                .await
                .expect("loop run")
        });
        let iters = result.iterations;
        per_scenario_iters.push(iters);
        total_iters += iters as u64;
        if result.converged {
            converged_count += 1;
        }
        println!(
            "BENCH | rce_convergence_scenario | name={name} | iters={iters} | converged={c} | verdict={v:?}",
            name = sc.name,
            iters = iters,
            c = result.converged,
            v = result.final_verdict
        );
    }

    let rate_x100 = (converged_count * 100) / total.max(1);
    let avg_iter_x10 = if total == 0 { 0 } else { (total_iters * 10) / total as u64 };
    println!(
        "BENCH | rce_convergence_corpus | scenarios={total} | converged={converged_count} | convergence_rate_pct={rate_x100} | avg_iterations_x10={avg_iter_x10}"
    );
    // Sanity: per-scenario iters are bounded by max_iterations (3).
    for it in &per_scenario_iters {
        assert!(*it <= 3, "iteration count must respect max_iterations=3");
    }
    // No hard assert on convergence_rate — baseline established here per
    // spec L601 ("baseline established here").
    let _ = Verdict::Pass;
}
