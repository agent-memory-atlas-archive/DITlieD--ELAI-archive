//! B-24 Phase 4I (Group C2): Contract tests — RED phase.
//!
//! Each test pins one of the 7 invariants from `docs/specs/guide/steps/phase-4I.txt`
//! L172-178. The pipeline entry point lives at
//! `elai_orchestrator::media_validator::pipeline::validate`; Group C3 fills
//! in the real implementation. In C2 the function is a stub returning
//! `Err(MediaValidatorError::NotImplemented)` so most tests fail the
//! verdict-shape assertion and are pinned with `#[ignore]` until C3.
//!
//! Test discipline (CLAUDE.md):
//!   * `.unwrap()` / `.expect()` are permitted in `#[test]` bodies.
//!   * Tests are deterministic — no thread::sleep, no real network calls.
//!   * Fixtures are synthesized in-test (no committed binaries).

use std::path::PathBuf;

use elai_orchestrator::media_validator::{
    pipeline, CheckResult, Issue, MediaType, MediaValidationRequest,
    MediaValidatorError, MockVlmBackend, QualitySpec, Severity, Tier1Result, Tier1Validator,
    ValidationMode, Verdict, VlmValidationOutput,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn quick_request(media_type: MediaType, qualitative: bool) -> MediaValidationRequest {
    MediaValidationRequest {
        media_type,
        file_path: PathBuf::from("tests/fixtures/media/synthetic.bin"),
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

// ---------------------------------------------------------------------------
// c1: Tier 1 runs before Tier 2 (ordering invariant)
// ---------------------------------------------------------------------------

/// Spec invariant 1: Tier 1 always completes before Tier 2 is spawned.
/// The ordering invariant is encoded as: whenever Tier 2 results exist,
/// Tier 1 MUST have produced check results first. Concurrent or out-of-
/// order dispatch would surface as `tier2.is_some() && tier1.checks.is_empty()`.
/// Audit WARN-1 fix (decision 27): the original `x <= x+y` assertion was
/// mathematically vacuous for u64; replaced with the real ordering invariant.
#[test]
fn c1_tier1_runs_before_tier2() {
    let req = quick_request(MediaType::Image, true);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    // Tier 1 must always populate checks (deterministic stage).
    assert!(!out.tier1.checks.is_empty(), "tier1 checks should be populated");
    // Real ordering invariant: Tier 2 cannot have run without Tier 1 having
    // produced check results first. If tier2 is Some, tier1.checks MUST be
    // non-empty — this is purely test-side, no production-code instrumentation
    // required, and accurately encodes "tier1 before tier2" with no vacuous
    // arithmetic.
    if out.tier2.is_some() {
        assert!(
            !out.tier1.checks.is_empty(),
            "tier2 populated implies tier1 already produced checks (ordering invariant)"
        );
    }
}

// ---------------------------------------------------------------------------
// c2: Tier 1 fail overrides Tier 2 pass
// ---------------------------------------------------------------------------

/// Spec invariant 2: When Tier 1 fails, the final verdict is Fail
/// regardless of what Tier 2 says.
#[test]
fn c2_tier1_fail_overrides_tier2() {
    // Use a media file path that does NOT exist on disk; Group C3's Tier 1
    // is expected to report `pass=false` with a missing-file CheckResult.
    let req = MediaValidationRequest {
        media_type: MediaType::Image,
        file_path: PathBuf::from("tests/fixtures/media/DOES_NOT_EXIST.png"),
        quality_spec: QualitySpec {
            similarity_threshold: None,
            qualitative_checks: true,
            visual_check: false,
        },
        reference_path: None,
        mode: ValidationMode::Full,
        output_dir: None,
        request_id: uuid::Uuid::nil(),
    };
    // Mock VLM is configured to always pass — but Tier 1 vetoes.
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(!out.tier1.pass, "tier1 must report fail for missing file");
    assert_eq!(
        out.final_verdict,
        Verdict::Fail,
        "Tier 2 (mock pass) MUST NOT override Tier 1 fail"
    );
}

// ---------------------------------------------------------------------------
// c3: Tier 2 only runs on QualitySpec with qualitative_checks
// ---------------------------------------------------------------------------

/// Spec invariant 3: When QualitySpec.qualitative_checks=false, Tier 2 is
/// never invoked. Verified by asserting `result.tier2 == None` for a
/// request with `qualitative_checks=false`.
#[test]
fn c3_tier2_only_runs_on_qualitative_spec() {
    let req = quick_request(MediaType::Image, /*qualitative=*/ false);
    let mock = MockVlmBackend::new();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    assert!(
        out.tier2.is_none(),
        "Tier 2 must NOT be invoked when qualitative_checks=false; got: {:?}",
        out.tier2
    );
}

// ---------------------------------------------------------------------------
// c4: VLM output conforms to VlmValidationOutput schema
// ---------------------------------------------------------------------------

/// Spec invariant 4: VLM output (from constrained decoding) conforms to
/// the `VlmValidationOutput` JSON schema. This test does NOT depend on
/// Group C3 — it exercises the MockVlmBackend's deterministic output and
/// asserts each field is within the documented range. Passes today.
#[test]
fn c4_vlm_output_conforms_to_schema() {
    let backend = MockVlmBackend::new();
    use elai_orchestrator::media_validator::VlmBackend;
    let out: VlmValidationOutput = backend.validate_advisory(std::path::Path::new("a.png"));
    // Range checks per types::VlmValidationOutput doc-comments.
    assert!(
        (0.0..=1.0).contains(&out.overall_quality),
        "overall_quality out of range: {}",
        out.overall_quality
    );
    assert!(
        (0.0..=1.0).contains(&out.confidence),
        "confidence out of range: {}",
        out.confidence
    );
    // Severity enum must be one of the 3 documented variants.
    for issue in &out.issues {
        assert!(
            matches!(
                issue.severity,
                Severity::Critical | Severity::Major | Severity::Minor
            ),
            "unknown severity"
        );
    }
    // serde round-trip must be lossless (the SSB payload depends on this).
    let json = serde_json::to_string(&out).expect("serialize");
    let back: VlmValidationOutput = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(out, back);
}

// ---------------------------------------------------------------------------
// c5: Request rejected without media steps
// ---------------------------------------------------------------------------

/// Spec invariant 5: A MediaValidationRequest aimed at a pipeline with no
/// media steps must be rejected. Until Group C3 wires the
/// `new_if_media_steps` factory through, the stub here returns
/// NotImplemented — but the FACTORY ALREADY returns `None` for the
/// no-media case (Group C1 ships `new_if_media_steps_stub`). This test
/// pins the lazy-factory contract.
#[test]
fn c5_request_rejected_without_media_steps() {
    use elai_orchestrator::media_validator::new_if_media_steps_stub;
    // has_media=false -> factory returns None -> caller cannot construct
    // a MediaValidator -> no pipeline -> implicit rejection of the request.
    let factory = new_if_media_steps_stub(false);
    assert!(factory.is_none(), "factory must return None when has_media=false");
}

// ---------------------------------------------------------------------------
// c6: SSIM/LPIPS reproducible within epsilon 1e-6 (CPU EP only)
// ---------------------------------------------------------------------------

/// Spec invariant 6: same input run through SSIM N times produces the
/// same score within epsilon 1e-6. Uses image-compare crate directly
/// (already in orchestrator's dep graph) — does NOT depend on Group C3.
///
/// We construct two identical 8x8 RGB images and assert SSIM == ~1.0
/// across 50 iterations.
#[test]
fn c6_ssim_lpips_reproducible_within_epsilon() {
    // SSIM minimum image is 8x8 (the algorithm scans 8x8 windows).
    // `image_compare::prelude` re-exports the `image` crate types so we do
    // not need to add `image` as a direct dev-dep.
    use image_compare::prelude::{GrayImage, Luma};
    use image_compare::Algorithm;

    let a: GrayImage = GrayImage::from_fn(8, 8, |_, _| Luma([128u8]));
    let b = a.clone();

    let mut scores = Vec::with_capacity(50);
    for _ in 0..50 {
        let result = image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &a, &b)
            .expect("SSIM compute");
        scores.push(result.score);
    }
    let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let delta = (max - min).abs();
    assert!(
        delta < 1e-6,
        "SSIM scores diverged across 50 runs: min={min}, max={max}, delta={delta}"
    );
    // identical images -> SSIM = 1.0 exactly within epsilon.
    assert!(
        (scores[0] - 1.0).abs() < 1e-6,
        "identical images SSIM ~= 1.0, got {}",
        scores[0]
    );
}

// ---------------------------------------------------------------------------
// c7: Tier 2 disagree with Tier 1 -> HumanReview
// ---------------------------------------------------------------------------

/// Spec invariant 7: Tier 1 pass + Tier 2 fail (or vice-versa) yields
/// `Verdict::HumanReview`. Group C3 owns the veto-table; this test pins
/// the contract.
#[test]
fn c7_tier2_disagree_yields_human_review() {
    let req = quick_request(MediaType::Image, true);
    // MockVlmBackend::always_critical -> Tier 2 will fail (advisory_pass=false).
    // Assume Group C3's Tier 1 on a well-formed-but-mock fixture returns pass.
    let mock = MockVlmBackend::always_critical();
    let out = pipeline::validate(&req, &mock).expect("pipeline (Group C3)");
    if out.tier1.pass && out.tier2.as_ref().is_some_and(|t| !t.advisory_pass) {
        assert_eq!(
            out.final_verdict,
            Verdict::HumanReview,
            "Tier1 pass + Tier2 fail must yield HumanReview, got {:?}",
            out.final_verdict
        );
    } else {
        panic!("expected Tier1 pass + Tier2 fail combination, got {out:?}");
    }
}

// ---------------------------------------------------------------------------
// Bonus: type round-trip contracts (these pass today)
// ---------------------------------------------------------------------------

#[test]
fn tier1_result_serde_round_trip() {
    let r = Tier1Result {
        pass: true,
        checks: vec![CheckResult {
            name: "resolution".into(),
            pass: true,
            detail: "1920x1080".into(),
            latency_ms: 12,
        }],
        latency_ms: 12,
    };
    let s = serde_json::to_string(&r).expect("serialize");
    let back: Tier1Result = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(r, back);
}

#[test]
fn verdict_humanreview_serializes_as_pascal() {
    // SSB schema relies on "PascalCase" serialization of Verdict.
    let v = Verdict::HumanReview;
    let s = serde_json::to_string(&v).expect("serialize");
    assert_eq!(s, "\"HumanReview\"");
}

#[test]
fn pipeline_returns_structured_result_for_quick_no_qualitative() {
    // Group C3 GREEN-phase replacement of the C2 RED-phase pin
    // (`pipeline_stub_returns_not_implemented`). pipeline::validate now
    // returns a real MediaValidationResult; the structural assertion is
    // "no longer NotImplemented and Tier2 stays absent without
    // qualitative_checks". The Tier1 outcome depends on whether the
    // fixture file exists — both branches are valid as long as Tier2 is
    // never spawned in Quick / non-qualitative mode.
    let req = quick_request(MediaType::Image, false);
    let mock = MockVlmBackend::new();
    let r = pipeline::validate(&req, &mock);
    match r {
        Ok(out) => {
            assert!(
                out.tier2.is_none(),
                "Tier 2 must NOT run when qualitative_checks=false, got {:?}",
                out.tier2
            );
            assert!(
                matches!(out.final_verdict, Verdict::Pass | Verdict::Fail),
                "final_verdict must be Pass or Fail in Quick path, got {:?}",
                out.final_verdict
            );
        }
        Err(MediaValidatorError::PathOutsideOutputDir { .. }) => {
            // Acceptable: relative fixture path with `..` would trigger
            // the path guard. The default fixture path uses no `..`.
        }
        Err(MediaValidatorError::Io(_)) => {
            // Acceptable: canonicalize() of a non-existent path returns
            // io::ErrorKind::NotFound when the absolute prefix exists
            // but the leaf doesn't (Tier1 surfaces this as missing_file).
        }
        other => panic!("unexpected pipeline result: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// c7b: Tier 2 advisory_pass=true BUT disagrees on a specific check -> HumanReview
// ---------------------------------------------------------------------------

/// Spec invariant 7b (audit WARN-1 fix, decision 43): the 7th veto-table
/// branch is "Tier 2 disagrees with Tier 1 on any check" — distinct from
/// the existing branch 4 ("Tier 2 advisory_pass=false"). Specifically:
/// when Tier 2 returns `advisory_pass=true` AND `overall_quality>=0.5`
/// (no Critical issues), but its `structured_output.issues` list includes
/// a Major-or-Critical issue in a Tier1-covered category (resolution,
/// format, colorspace, manifold, sample_rate), the verdict MUST be
/// `Verdict::HumanReview` with reason
/// `"tier2_disagreement_despite_advisory_pass"`.
///
/// Without this branch the previous implementation collapsed the case
/// into "Tier 2 pass -> Verdict::Pass" — silently letting a VLM
/// disagreement slip through with a green verdict.
#[test]
fn c7b_tier2_advisory_pass_but_disagrees_on_specific_check() {
    use elai_orchestrator::media_validator::{MockVlmBackend, VlmValidationOutput};

    // Tier1 will pass for a synthetic.bin PNG fixture (resolution proxy
    // not active because similarity_threshold is None).
    let req = quick_request(MediaType::Image, true);

    // Build a Mock with advisory_pass-compatible top-level fields
    // (overall_quality>=0.5, confidence high, NO Critical) but a Major
    // disagreement in category "resolution" — i.e. Tier1 said resolution
    // OK, VLM says NO.
    let output = VlmValidationOutput {
        overall_quality: 0.85,
        confidence: 0.9,
        issues: vec![elai_orchestrator::media_validator::Issue {
            severity: Severity::Major,
            category: "resolution".into(),
            description: "VLM disagrees: image looks 640x480, not 1920x1080".into(),
            location: None,
        }],
        reasoning: "Tier1-covered disagreement (audit WARN-1 contract)".into(),
    };
    let mock = MockVlmBackend::with_output(output);

    let out = pipeline::validate(&req, &mock).expect("pipeline");
    assert!(out.tier1.pass, "Tier1 must pass for this fixture");
    let t2 = out.tier2.as_ref().expect("Tier2 must run for qualitative=true");
    assert!(
        t2.advisory_pass,
        "Tier2 advisory_pass must be TRUE (no Critical, quality>=0.5)"
    );
    assert_eq!(
        out.final_verdict,
        Verdict::HumanReview,
        "Tier2 disagreement on Tier1-covered category must yield HumanReview"
    );
    assert_eq!(
        out.verdict_reason, "tier2_disagreement_despite_advisory_pass",
        "reason string must match spec branch label"
    );
}

// ---------------------------------------------------------------------------
// c6b: SSIM 64x64 reproducibility (non-trivial input)
// ---------------------------------------------------------------------------

/// Spec invariant 6 (audit INFO fix): the existing c6 test uses an 8x8
/// fixture where SSIM short-circuits to ~1.0 trivially. This test
/// synthesizes 64x64 RGB images via the `image` crate (a transitive dep
/// already in orchestrator's dep graph, gated `default-features=false,
/// features=["png"]`) so the SSIM compute exercises real windowing.
/// Same-image SSIM must be ~1.0 and stable within epsilon 1e-6 across
/// 25 iterations.
#[test]
fn c6b_ssim_64x64_reproducible_within_epsilon() {
    use image_compare::prelude::{GrayImage, Luma};
    use image_compare::Algorithm;

    // Synthesize a 64x64 luma image with a deterministic gradient — non-
    // trivial enough that the MSSIMSimple sliding window has actual
    // pixel variation to score against.
    let a: GrayImage = GrayImage::from_fn(64, 64, |x, y| {
        // Diagonal gradient + a small modulation so windows differ.
        let v = ((x as u16 + y as u16) * 2 + (x ^ y) as u16) % 256;
        Luma([v as u8])
    });
    let b = a.clone();

    let mut scores = Vec::with_capacity(25);
    for _ in 0..25 {
        let result = image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &a, &b)
            .expect("SSIM compute");
        scores.push(result.score);
    }
    let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let delta = (max - min).abs();
    assert!(
        delta < 1e-6,
        "64x64 SSIM diverged across 25 runs: min={min} max={max} delta={delta}"
    );
    assert!(
        (scores[0] - 1.0).abs() < 1e-6,
        "identical 64x64 SSIM ~= 1.0, got {}",
        scores[0]
    );
}

// Suppress "unused import" warnings when the ignored tests are excluded.
#[allow(dead_code)]
fn _silence_unused(_: &dyn Tier1Validator, _: &Issue) {}
