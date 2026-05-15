// Phase C1 — Lighthouse validator contract tests.
//
// Exercises the threshold-check logic that the production validator closure in
// task_runner.rs uses after a Lighthouse run. Uses stub `LighthouseScores`
// values — no Lighthouse spawn, no Chrome, no network. Mirrors the
// sentinel-file pattern in fix_and_rerun_contract.rs.
//
// Covers:
//   1. perf-fail: scores {performance:0.5, a11y:0.7} -> Failed, summary
//      mentions "lighthouse_perf" or "performance".
//   2. perf+a11y-pass: scores {performance:0.85, a11y:0.95} -> Green.
//   3. chrome-missing: chrome_path=None -> Failed, summary contains "chrome".

use std::path::PathBuf;

use elai_orchestrator::task_runner::{lighthouse_validator_report, FixerValidatorReport};
use elai_web_validator::cwv::LighthouseScores;

// ---------------------------------------------------------------------------
// Contract 1 — perf score below threshold -> Failed.
// ---------------------------------------------------------------------------

#[test]
fn contract_lighthouse_perf_below_threshold_returns_failed() {
    let scores = LighthouseScores {
        performance: 0.5,
        accessibility: 0.7,
    };
    // chrome_path = Some(dummy) so chrome-missing is not triggered.
    let chrome = PathBuf::from("C:/fake/chrome.exe");
    let report = lighthouse_validator_report(Some(chrome.as_path()), &scores);
    match report {
        FixerValidatorReport::Failed { summary } => {
            assert!(
                summary.contains("lighthouse_perf") || summary.contains("performance"),
                "Failed summary must mention perf failure, got: {summary}"
            );
            assert!(
                summary.contains("0.80") || summary.contains("0.5"),
                "Failed summary must include threshold or actual score, got: {summary}"
            );
        }
        FixerValidatorReport::Green => {
            panic!("expected Failed for perf=0.5 a11y=0.7, got Green");
        }
    }
}

// ---------------------------------------------------------------------------
// Contract 2 — both scores at/above threshold -> Green.
// ---------------------------------------------------------------------------

#[test]
fn contract_lighthouse_scores_above_threshold_returns_green() {
    let scores = LighthouseScores {
        performance: 0.85,
        accessibility: 0.95,
    };
    let chrome = PathBuf::from("C:/fake/chrome.exe");
    let report = lighthouse_validator_report(Some(chrome.as_path()), &scores);
    assert!(
        matches!(report, FixerValidatorReport::Green),
        "expected Green for perf=0.85 a11y=0.95, got Failed"
    );
}

// ---------------------------------------------------------------------------
// Contract 3 — chrome_path absent -> Failed mentioning chrome.
// ---------------------------------------------------------------------------

#[test]
fn contract_chrome_missing_returns_failed() {
    let scores = LighthouseScores {
        performance: 0.95,
        accessibility: 0.95,
    };
    let report = lighthouse_validator_report(None, &scores);
    match report {
        FixerValidatorReport::Failed { summary } => {
            assert!(
                summary.to_lowercase().contains("chrome"),
                "Failed summary must mention chrome, got: {summary}"
            );
        }
        FixerValidatorReport::Green => {
            panic!("expected Failed when chrome_path=None, got Green");
        }
    }
}

// ---------------------------------------------------------------------------
// Boundary: exact threshold values -> Green.
// ---------------------------------------------------------------------------

#[test]
fn contract_lighthouse_exact_threshold_passes() {
    let scores = LighthouseScores {
        performance: 0.80,
        accessibility: 0.90,
    };
    let chrome = PathBuf::from("C:/fake/chrome.exe");
    let report = lighthouse_validator_report(Some(chrome.as_path()), &scores);
    assert!(
        matches!(report, FixerValidatorReport::Green),
        "score exactly at threshold must be Green"
    );
}

// ---------------------------------------------------------------------------
// Boundary: a11y below threshold, perf passes -> Failed mentioning a11y.
// ---------------------------------------------------------------------------

#[test]
fn contract_lighthouse_a11y_below_threshold_returns_failed() {
    let scores = LighthouseScores {
        performance: 0.90,
        accessibility: 0.70,
    };
    let chrome = PathBuf::from("C:/fake/chrome.exe");
    let report = lighthouse_validator_report(Some(chrome.as_path()), &scores);
    match report {
        FixerValidatorReport::Failed { summary } => {
            assert!(
                summary.contains("lighthouse_a11y") || summary.contains("accessibility"),
                "Failed summary must mention a11y failure, got: {summary}"
            );
        }
        FixerValidatorReport::Green => {
            panic!("expected Failed for a11y=0.70, got Green");
        }
    }
}
