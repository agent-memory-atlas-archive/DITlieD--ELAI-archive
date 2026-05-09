//! WEB.3 — Phase B / Streams B3+B4 — Core Web Vitals via Lighthouse.
//!
//! Spawns Google Lighthouse (`npx --yes lighthouse <url>
//! --output=json --output-path=stdout`) against an already-running dev
//! server, parses the resulting JSON, and returns a `LighthouseReport`
//! with LCP / CLS / INP and the overall performance score.
//!
//! Anti-slopsquatting (verified 2026-05-08 via `npm view`):
//!   lighthouse        (GoogleChrome/lighthouse, official)   v13.3.0
//!
//! Hardware: Lighthouse needs a controllable Chromium / Chrome binary on
//! the host. We call `chromium --version` (with a fallback to `google-chrome`)
//! once before invoking Lighthouse and surface a `ChromiumMissing` error
//! when neither is on PATH so the orchestrator can degrade the
//! `WebPerformance` tier gracefully.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::WebValidatorError;

// ---------------------------------------------------------------------------
// Named constants.
// ---------------------------------------------------------------------------

/// Default timeout for one Lighthouse invocation. Lighthouse on a small
/// dev-server payload typically takes 25-45 s; 180 s is a safe ceiling.
pub const LIGHTHOUSE_DEFAULT_TIMEOUT_SECS: u64 = 180;

/// Stdout cap for Lighthouse JSON. Reports for typical SPAs are 1-5 MiB; we
/// cap defensively at 16 MiB to avoid OOM on a misbehaving target.
pub const LIGHTHOUSE_MAX_STDOUT_BYTES: usize = 16 * 1024 * 1024;

const CHROMIUM_PROBE_FIRST: &str = "chromium --version";
const CHROMIUM_PROBE_FALLBACK: &str = "google-chrome --version";

// ---------------------------------------------------------------------------
// Public types.
// ---------------------------------------------------------------------------

/// Distilled Core Web Vitals and overall performance score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LighthouseReport {
    /// Largest Contentful Paint, ms (lower is better).
    pub lcp_ms: u32,
    /// Cumulative Layout Shift, unitless (lower is better, target < 0.1).
    pub cls: f64,
    /// Interaction-to-Next-Paint, ms (lower is better, target < 200).
    pub inp_ms: u32,
    /// Performance score in [0, 1] (higher is better).
    pub perf_score: f64,
}

// ---------------------------------------------------------------------------
// Public API.
// ---------------------------------------------------------------------------

/// Probe whether a usable Chromium / Chrome binary is on PATH.
///
/// Returns `Ok(())` if either `chromium` or `google-chrome` responds to
/// `--version` within 5 seconds. Returns `Err(ChromiumMissing)` otherwise.
pub async fn probe_chromium() -> Result<(), WebValidatorError> {
    if probe_one("chromium").await {
        return Ok(());
    }
    if probe_one("google-chrome").await {
        return Ok(());
    }
    Err(WebValidatorError::ChromiumMissing {
        probe_command: format!("{CHROMIUM_PROBE_FIRST} || {CHROMIUM_PROBE_FALLBACK}"),
    })
}

/// Run Lighthouse against `url` from `project_dir` and return the distilled
/// CWV report.
///
/// Behaviour:
///   - Probes `node` and a Chromium binary first. Either probe failing
///     short-circuits with `NodeMissing` or `ChromiumMissing` — we never
///     spawn Lighthouse without both prerequisites.
///   - Caps stdout at `LIGHTHOUSE_MAX_STDOUT_BYTES` and the wallclock at
///     `LIGHTHOUSE_DEFAULT_TIMEOUT_SECS`.
pub async fn run_lighthouse(
    project_dir: &Path,
    url: &str,
) -> Result<LighthouseReport, WebValidatorError> {
    crate::lint::probe_node().await?;
    probe_chromium().await?;

    let mut cmd = Command::new("npx");
    cmd.args([
        "--yes",
        "lighthouse",
        url,
        "--output=json",
        "--output-path=stdout",
        "--quiet",
        "--chrome-flags=--headless=new --no-sandbox",
    ])
    .current_dir(project_dir)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .kill_on_drop(true);

    let output_future = cmd.output();
    let output = match tokio::time::timeout(
        Duration::from_secs(LIGHTHOUSE_DEFAULT_TIMEOUT_SECS),
        output_future,
    )
    .await
    {
        Ok(Ok(o)) => o,
        Ok(Err(source)) => {
            return Err(WebValidatorError::ProcessSpawn {
                command: "npx --yes lighthouse".into(),
                source,
            });
        }
        Err(_) => {
            return Err(WebValidatorError::Timeout {
                what: "lighthouse".into(),
                after_secs: LIGHTHOUSE_DEFAULT_TIMEOUT_SECS,
            });
        }
    };

    let limit = output.stdout.len().min(LIGHTHOUSE_MAX_STDOUT_BYTES);
    let stdout = String::from_utf8_lossy(&output.stdout[..limit]).into_owned();

    if stdout.trim().is_empty() {
        return Err(WebValidatorError::ToolOutputInvalid {
            tool: "lighthouse".into(),
            reason: "empty stdout — likely chromium spawn failure".into(),
        });
    }

    parse_lighthouse_report(&stdout)
}

/// Public for testing: parse a Lighthouse JSON document into a distilled
/// report. Returns `ToolOutputInvalid` if any required field is missing or
/// malformed.
pub fn parse_lighthouse_report(json: &str) -> Result<LighthouseReport, WebValidatorError> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| WebValidatorError::ToolOutputInvalid {
            tool: "lighthouse".into(),
            reason: format!("json parse: {e}"),
        })?;

    let lcp_ms = pull_audit_numeric_value(&v, "largest-contentful-paint")?;
    let cls = pull_audit_numeric_value(&v, "cumulative-layout-shift")?;
    let inp_ms = pull_audit_numeric_value(&v, "interaction-to-next-paint").unwrap_or(0.0);

    let perf_score = v
        .pointer("/categories/performance/score")
        .and_then(|s| s.as_f64())
        .ok_or_else(|| WebValidatorError::ToolOutputInvalid {
            tool: "lighthouse".into(),
            reason: "missing categories.performance.score".into(),
        })?;

    Ok(LighthouseReport {
        lcp_ms: lcp_ms.round().max(0.0).min(f64::from(u32::MAX)) as u32,
        cls,
        inp_ms: inp_ms.round().max(0.0).min(f64::from(u32::MAX)) as u32,
        perf_score,
    })
}

// ---------------------------------------------------------------------------
// Internal — probe helpers.
// ---------------------------------------------------------------------------

async fn probe_one(program: &str) -> bool {
    let probe = Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .output();
    matches!(
        tokio::time::timeout(Duration::from_secs(5), probe).await,
        Ok(Ok(out)) if out.status.success()
    )
}

fn pull_audit_numeric_value(
    root: &serde_json::Value,
    audit_id: &str,
) -> Result<f64, WebValidatorError> {
    root.pointer(&format!("/audits/{audit_id}/numericValue"))
        .and_then(|n| n.as_f64())
        .ok_or_else(|| WebValidatorError::ToolOutputInvalid {
            tool: "lighthouse".into(),
            reason: format!("missing audits.{audit_id}.numericValue"),
        })
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lighthouse_default_timeout_is_named() {
        assert!(LIGHTHOUSE_DEFAULT_TIMEOUT_SECS >= 30);
        assert!(LIGHTHOUSE_DEFAULT_TIMEOUT_SECS <= 600);
    }

    #[test]
    fn lighthouse_report_round_trips_serde() {
        let r = LighthouseReport {
            lcp_ms: 1234,
            cls: 0.05,
            inp_ms: 200,
            perf_score: 0.93,
        };
        let json = serde_json::to_string(&r).expect("serialize");
        let back: LighthouseReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r.lcp_ms, back.lcp_ms);
    }

    #[test]
    fn parse_lighthouse_report_extracts_metrics() {
        let json = r#"{
            "categories": { "performance": { "score": 0.87 } },
            "audits": {
                "largest-contentful-paint": { "numericValue": 1500.0 },
                "cumulative-layout-shift": { "numericValue": 0.04 },
                "interaction-to-next-paint": { "numericValue": 175.0 }
            }
        }"#;
        let r = parse_lighthouse_report(json).expect("parse ok");
        assert_eq!(r.lcp_ms, 1500);
        assert!((r.cls - 0.04).abs() < 1e-9);
        assert_eq!(r.inp_ms, 175);
        assert!((r.perf_score - 0.87).abs() < 1e-9);
    }

    #[test]
    fn parse_lighthouse_report_missing_perf_score_errors() {
        let json = r#"{
            "categories": { "performance": {} },
            "audits": { "largest-contentful-paint": { "numericValue": 1.0 },
                        "cumulative-layout-shift": { "numericValue": 0.0 } }
        }"#;
        let err = parse_lighthouse_report(json).expect_err("missing score must error");
        let msg = format!("{err}");
        assert!(msg.contains("performance.score") || msg.contains("score"));
    }

    #[test]
    fn parse_lighthouse_report_inp_optional() {
        let json = r#"{
            "categories": { "performance": { "score": 0.9 } },
            "audits": {
                "largest-contentful-paint": { "numericValue": 800.0 },
                "cumulative-layout-shift": { "numericValue": 0.01 }
            }
        }"#;
        let r = parse_lighthouse_report(json).expect("inp absent must succeed");
        assert_eq!(r.inp_ms, 0);
    }

    #[test]
    fn parse_lighthouse_report_invalid_json_errors() {
        let err = parse_lighthouse_report("not json").expect_err("bad json must error");
        let msg = format!("{err}");
        assert!(msg.contains("json"));
    }
}
