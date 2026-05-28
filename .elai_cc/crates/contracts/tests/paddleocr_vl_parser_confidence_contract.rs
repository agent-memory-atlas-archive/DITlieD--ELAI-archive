//! E-36 Tier-B — paddleocr_vl_parser_confidence_contract.
//!
//! Locks plan task 3 + task 8 fallback semantics:
//!   * a synthetic "noisy scan" page with confidence < threshold AND
//!     `mode_allows_cloud == true` triggers `fallback_recommended = true`.
//!   * a clean-page fixture (all confidences >= threshold) does NOT.
//!   * `--mode local` forces `fallback_recommended = false` regardless
//!     of page confidence (HARD anti-cloud rule).
//!   * the default `cloud_fallback_threshold` is the documented 0.7
//!     (per `[knowledge.doc_parser] cloud_fallback_threshold = 0.7`).
//!
//! Paper grounding: arXiv:2510.14528 (PaddleOCR-VL 0.9B, 92.56 on
//! OmniDocBench v1.5). The threshold is a policy knob, not a paper
//! claim — the test pins the SHIPPING DEFAULT, not the model's
//! accuracy headline.

use elai_knowledge::chunking::pdf_structured::{
    compute_fallback_recommended, ParserBackend, ParseError, PdfStructuredChunker,
    DEFAULT_CLOUD_FALLBACK_THRESHOLD,
};

#[test]
fn default_threshold_is_zero_point_seven() {
    // Pin the shipping default. Operators flip via
    // `[knowledge.doc_parser] cloud_fallback_threshold` in elai_cc.toml.
    assert!((DEFAULT_CLOUD_FALLBACK_THRESHOLD - 0.7_f32).abs() < f32::EPSILON);
}

#[test]
fn noisy_scan_triggers_fallback_in_hybrid_mode() {
    // Synthetic noisy scan: one page below threshold.
    let confs = [0.95_f32, 0.45_f32, 0.92_f32];
    assert!(compute_fallback_recommended(
        &confs,
        DEFAULT_CLOUD_FALLBACK_THRESHOLD,
        true, // mode_allows_cloud (Hybrid / Cloud)
    ));
}

#[test]
fn clean_pages_do_not_trigger_fallback() {
    let confs = [0.95_f32, 0.91_f32, 0.99_f32];
    assert!(!compute_fallback_recommended(
        &confs,
        DEFAULT_CLOUD_FALLBACK_THRESHOLD,
        true,
    ));
}

#[test]
fn local_mode_blocks_fallback_regardless_of_confidence() {
    // Even with a 0.1 page, `--mode local` forces the result to false.
    let confs = [0.10_f32, 0.05_f32];
    assert!(!compute_fallback_recommended(
        &confs,
        DEFAULT_CLOUD_FALLBACK_THRESHOLD,
        false, // mode_allows_cloud
    ));
}

#[test]
fn local_backends_pass_eligibility_in_any_mode() {
    for backend in [
        ParserBackend::PaddleOcrVl,
        ParserBackend::MinerU,
        ParserBackend::Dolphin,
    ] {
        assert!(
            PdfStructuredChunker::check_backend_eligibility(backend, false).is_ok(),
            "{backend:?} local backend must pass eligibility in --mode local"
        );
        assert!(PdfStructuredChunker::check_backend_eligibility(backend, true).is_ok());
    }
}

#[test]
fn cloud_backends_blocked_in_local_mode() {
    for backend in [
        ParserBackend::MistralCloud,
        ParserBackend::ReductoCloud,
        ParserBackend::LlamaParseCloud,
    ] {
        let r = PdfStructuredChunker::check_backend_eligibility(backend, false);
        assert!(
            matches!(r, Err(ParseError::CloudRequired { .. })),
            "{backend:?} must be blocked in --mode local; got {r:?}"
        );
    }
}

#[test]
fn finalize_attaches_fallback_flag_for_low_confidence_page() {
    let out = PdfStructuredChunker::finalize(
        "body".into(),
        vec!["".into()],
        vec![],
        vec![0.55],
        ParserBackend::PaddleOcrVl,
        DEFAULT_CLOUD_FALLBACK_THRESHOLD,
        true,
    );
    assert!(out.fallback_recommended);
    assert_eq!(out.parser, ParserBackend::PaddleOcrVl);
    assert_eq!(out.page_count(), 1);
}

#[test]
fn finalize_does_not_attach_fallback_when_local_mode() {
    let out = PdfStructuredChunker::finalize(
        "body".into(),
        vec!["".into()],
        vec![],
        vec![0.55, 0.40],
        ParserBackend::PaddleOcrVl,
        DEFAULT_CLOUD_FALLBACK_THRESHOLD,
        false,
    );
    assert!(!out.fallback_recommended);
}
