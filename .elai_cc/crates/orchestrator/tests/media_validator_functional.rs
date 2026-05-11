//! B-24 Phase 4I (Group C2): Functional tests — RED phase.
//!
//! One test per spec-mandated media-type behavior. Fixtures are
//! synthesized in-test (TempDir-scoped) so the repo carries no binary
//! artifacts. The pipeline entry point is `pipeline::validate`; until
//! Group C3 lands, most tests are `#[ignore]`'d behind their respective
//! impl gate.
//!
//! Spec L180-184 (4I.3): image (3), audio (3), 3D (2), UI (2).
//! Plus VLM isolation (3) and lazy-loading (1) from L185-190.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use elai_orchestrator::media_validator::{
    pipeline, MediaType, MediaValidationRequest, MediaValidatorError, MockVlmBackend,
    QualitySpec, ValidationMode, Verdict, VlmBackend,
};

// ---------------------------------------------------------------------------
// Synthetic fixture helpers
// ---------------------------------------------------------------------------

/// Write a tiny valid 1x1 RGB PNG to `dir/<name>`. Returns the path.
/// Hand-crafted minimal PNG bytes; no `png` crate needed.
fn write_tiny_png(dir: &Path, name: &str) -> PathBuf {
    // Pre-computed valid 1x1 RGB PNG (red pixel). 67 bytes.
    // Generated via `magick -size 1x1 xc:red png:- | xxd -p` once and pinned.
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length+type
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // depth, color, CRC
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01,
        0x5B, 0x42, 0x5D, 0x6A, // IDAT CRC
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82,
    ];
    let path = dir.join(name);
    fs::write(&path, TINY_PNG).expect("write png");
    path
}

/// Write a truncated PNG (signature only, no IHDR). Should trigger Tier 1 fail.
fn write_truncated_png(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, &[0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).expect("write");
    path
}

/// Write a minimal valid mono 8kHz WAV. PCM 16-bit, 1 sample of silence.
fn write_tiny_wav(dir: &Path, name: &str, sample: i16, n_samples: u32) -> PathBuf {
    let path = dir.join(name);
    let mut f = fs::File::create(&path).expect("create wav");
    let data_size = n_samples * 2; // 16-bit mono
    let chunk_size = 36 + data_size;
    let sample_rate: u32 = 8000;
    let byte_rate: u32 = sample_rate * 2;

    f.write_all(b"RIFF").unwrap();
    f.write_all(&chunk_size.to_le_bytes()).unwrap();
    f.write_all(b"WAVE").unwrap();
    f.write_all(b"fmt ").unwrap();
    f.write_all(&16u32.to_le_bytes()).unwrap(); // PCM subchunk size
    f.write_all(&1u16.to_le_bytes()).unwrap(); // PCM format
    f.write_all(&1u16.to_le_bytes()).unwrap(); // mono
    f.write_all(&sample_rate.to_le_bytes()).unwrap();
    f.write_all(&byte_rate.to_le_bytes()).unwrap();
    f.write_all(&2u16.to_le_bytes()).unwrap(); // block align
    f.write_all(&16u16.to_le_bytes()).unwrap(); // bits per sample
    f.write_all(b"data").unwrap();
    f.write_all(&data_size.to_le_bytes()).unwrap();
    for _ in 0..n_samples {
        f.write_all(&sample.to_le_bytes()).unwrap();
    }
    path
}

/// Minimal valid OBJ: one triangle. Triggers "manifold mesh" pass when
/// Blender is available.
fn write_manifold_obj(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(
        &path,
        "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
    )
    .expect("write obj");
    path
}

/// Minimal non-manifold OBJ: a triangle with a dangling vertex.
fn write_non_manifold_obj(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    // Three triangles sharing one edge but flipped normals -> non-manifold.
    fs::write(
        &path,
        "v 0 0 0\nv 1 0 0\nv 0 1 0\nv 1 1 0\nv 0 0 1\nf 1 2 3\nf 2 3 4\nf 1 3 5\nf 2 1 5\n",
    )
    .expect("write obj");
    path
}

fn request_for(path: &Path, media_type: MediaType, qualitative: bool) -> MediaValidationRequest {
    MediaValidationRequest {
        media_type,
        file_path: path.to_path_buf(),
        quality_spec: QualitySpec {
            similarity_threshold: None,
            qualitative_checks: qualitative,
            visual_check: false,
        },
        reference_path: None,
        mode: ValidationMode::Full,
        output_dir: None,
        request_id: uuid::Uuid::nil(),
    }
}

// ===========================================================================
// IMAGE (3 tests)
// ===========================================================================

#[test]
fn f_image_known_good_png_passes() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_tiny_png(dir.path(), "good.png");
    let req = request_for(&path, MediaType::Image, false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(out.tier1.pass, "good PNG must pass Tier 1: {:?}", out.tier1);
    assert_eq!(out.final_verdict, Verdict::Pass);
}

#[test]
fn f_image_truncated_png_tier1_fails() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_truncated_png(dir.path(), "trunc.png");
    let req = request_for(&path, MediaType::Image, false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(!out.tier1.pass, "truncated PNG must fail Tier 1");
    assert_eq!(out.final_verdict, Verdict::Fail);
}

#[test]
fn f_image_wrong_resolution_tier1_fails() {
    let dir = TempDir::new().expect("tempdir");
    // Tiny 1x1 PNG fails any reasonable resolution requirement.
    let path = write_tiny_png(dir.path(), "small.png");
    let mut req = request_for(&path, MediaType::Image, false);
    // Wire the resolution constraint via similarity_threshold proxy until
    // QualitySpec gets a `min_width`/`min_height` field in C3.
    req.quality_spec.similarity_threshold = Some(0.95);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(!out.tier1.pass, "1x1 PNG below threshold must fail resolution check");
}

// ===========================================================================
// AUDIO (3 tests)
// ===========================================================================

#[test]
fn f_audio_known_good_wav_passes() {
    let dir = TempDir::new().expect("tempdir");
    // Tone-like sample (non-zero) so silence detector does not flag it.
    let path = write_tiny_wav(dir.path(), "good.wav", 5000, 8000);
    let req = request_for(&path, MediaType::Audio, false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(out.tier1.pass, "good WAV must pass Tier 1");
}

#[test]
fn f_audio_silence_only_flagged() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_tiny_wav(dir.path(), "silence.wav", 0, 8000);
    let req = request_for(&path, MediaType::Audio, false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    // silence_detect must mark Tier 1 as failing.
    assert!(!out.tier1.pass, "silence-only WAV must be flagged");
    let has_silence_check = out.tier1.checks.iter().any(|c| c.name.contains("silence"));
    assert!(has_silence_check, "missing silence check; got {:?}", out.tier1.checks);
}

#[test]
fn f_audio_clipped_flagged() {
    let dir = TempDir::new().expect("tempdir");
    // i16::MAX = clipped samples.
    let path = write_tiny_wav(dir.path(), "clipped.wav", i16::MAX, 8000);
    let req = request_for(&path, MediaType::Audio, false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(!out.tier1.pass, "fully-clipped WAV must be flagged");
}

// ===========================================================================
// MODEL3D (2 tests — both gated on Blender CLI)
// ===========================================================================

#[test]
#[ignore = "requires Blender CLI + Group C3 Model3DValidator"]
fn f_model3d_known_good_obj_passes() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_manifold_obj(dir.path(), "good.obj");
    let req = request_for(&path, MediaType::Model3D, false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(out.tier1.pass, "manifold OBJ must pass Tier 1");
}

#[test]
#[ignore = "requires Blender CLI + Group C3 Model3DValidator"]
fn f_model3d_non_manifold_fails() {
    let dir = TempDir::new().expect("tempdir");
    let path = write_non_manifold_obj(dir.path(), "bad.obj");
    let req = request_for(&path, MediaType::Model3D, false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(!out.tier1.pass, "non-manifold OBJ must fail Tier 1");
}

// ===========================================================================
// UI SCREENSHOT (2 tests)
// ===========================================================================

#[test]
fn f_ui_screenshot_ssim_check_passes() {
    let dir = TempDir::new().expect("tempdir");
    let a = write_tiny_png(dir.path(), "screen.png");
    let b = write_tiny_png(dir.path(), "reference.png");
    let mut req = request_for(&a, MediaType::UiScreenshot, false);
    req.reference_path = Some(b);
    req.quality_spec.similarity_threshold = Some(0.95);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(out.tier1.pass, "identical screenshots must pass SSIM>=0.95");
}

#[test]
fn f_ui_screenshot_wrong_res_fails() {
    let dir = TempDir::new().expect("tempdir");
    // 1x1 vs reference 1x1 — but force resolution threshold above 1px.
    let a = write_tiny_png(dir.path(), "small.png");
    let mut req = request_for(&a, MediaType::UiScreenshot, false);
    req.quality_spec.similarity_threshold = Some(0.99);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(!out.tier1.pass, "1x1 UI screenshot below resolution must fail");
}

// ===========================================================================
// VLM ISOLATION (3 tests — gated on Tier-C process isolation from B-22)
// ===========================================================================

#[test]
#[ignore = "requires Tier-C process isolation (B-22 / phase 4A)"]
fn f_vlm_cannot_read_outside_allowed_paths() {
    // The test would: 1) construct VlmProcessConfig with allowed_paths=[/tmp/img]
    // 2) attempt to read /etc/passwd via the VLM tool surface 3) assert sandbox blocks.
    // Tier-C infra lives in `sandbox.rs`; the wiring happens in C5/C6.
    panic!("test body deferred to Group C6 (requires sandbox::AgentEnvironment)");
}

#[test]
#[ignore = "requires Tier-C process isolation (B-22 / phase 4A)"]
fn f_vlm_cannot_write_any_files() {
    panic!("test body deferred to Group C6 (RLIMIT_FSIZE=0 contract)");
}

#[test]
#[ignore = "requires Tier-C process isolation (B-22 / phase 4A)"]
fn f_vlm_has_no_network_access() {
    panic!("test body deferred to Group C6 (network namespace contract)");
}

// ===========================================================================
// LAZY LOADING (1 test) — pins the factory contract today
// ===========================================================================

#[test]
fn f_lazy_loading_no_vlm_process_without_media_steps() {
    use elai_orchestrator::media_validator::new_if_media_steps_stub;
    // When the pipeline declares zero media steps, the factory MUST return
    // None — no VLM process spawn, no external-tool probes, no SSB rows.
    let factory = new_if_media_steps_stub(false);
    assert!(
        factory.is_none(),
        "factory must skip MediaValidator construction when has_media=false"
    );
    // Positive side: when has_media=true the stub still returns None (C1
    // skeleton), but the contract is symmetric — the real factory in C5
    // returns Some(_) only in this branch.
    // We do not exercise the Some(_) branch here because it is not yet wired.
}

// ===========================================================================
// Bonus: deterministic mock VLM contract (passes today)
// ===========================================================================

#[test]
fn mock_vlm_is_deterministic_across_calls() {
    let backend = MockVlmBackend::new();
    let a = backend.validate_advisory(Path::new("a.png"));
    let b = backend.validate_advisory(Path::new("b.png"));
    let c = backend.validate_advisory(Path::new("c.png"));
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// Suppress unused-import warnings when many tests are ignored.
#[allow(dead_code)]
fn _silence_unused(_: &MediaValidatorError) {}
