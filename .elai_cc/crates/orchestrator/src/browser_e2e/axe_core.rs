//! E-14 — axe-core gate via `@axe-core/playwright`.
//!
//! Runs axe-core against an already-loaded page by spawning a small Node
//! helper that requires `@axe-core/playwright` + `playwright`. Wire-dark
//! when either package is absent: returns
//! [`E2eError::ToolchainMissing`] so the caller can surface a degraded
//! verdict without panicking.
//!
//! Impact mapping follows axe-core 4.x semantics:
//!   * `critical` / `serious` -> hard Tier 1 fail (caller maps to
//!     `Verdict::Fail`).
//!   * `moderate` -> Tier 2 advisory (caller routes to VLM tier when
//!     available; else surfaces as advisory).
//!   * `minor` -> logged only.
//!
//! Output protocol from the helper (one line per artifact):
//!   `RESULTS_PATH:<absolute-path>` — JSON file with axe-core's raw report
//!   `OK` on success; `ERR:<reason>` on failure (exit non-zero).
//!
//! Grounding for the impact -> verdict mapping: Inclly 2026 + TestParty
//! 2026 (axe-core 96 rules vs Lighthouse 50; serious + critical = WCAG
//! blockers). See `docs/thefinalglance/E-plan/E-14_browser_e2e_agents.md`
//! task 3.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tracing::{debug, warn};

use super::E2eError;

/// axe-core impact levels. Stable across the 4.x line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AxeImpact {
    Critical,
    Serious,
    Moderate,
    Minor,
}

impl AxeImpact {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Serious => "serious",
            Self::Moderate => "moderate",
            Self::Minor => "minor",
        }
    }

    /// True when this impact level should force a Tier-1 fail.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Critical | Self::Serious)
    }
}

/// One axe-core finding. Matches the subset of the axe-results.json
/// shape we use — `id` + `impact` + `nodes` count is enough to render
/// the dashboard badge without exploding tokens on selector strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxeFinding {
    /// axe-core rule id, e.g. `color-contrast`.
    pub rule_id: String,
    /// Severity. `None` when axe omitted the field (rare; the helper
    /// surfaces this so the caller doesn't have to guess).
    pub impact: Option<AxeImpact>,
    /// Number of DOM nodes affected by this finding.
    pub node_count: usize,
}

/// Aggregate report from a single axe-core run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxeReport {
    pub findings: Vec<AxeFinding>,
    /// Cached counts so dashboards don't iterate.
    pub critical_count: usize,
    pub serious_count: usize,
    pub moderate_count: usize,
    pub minor_count: usize,
}

impl AxeReport {
    /// True when any finding is blocking (critical or serious).
    pub fn has_blocking(&self) -> bool {
        self.critical_count > 0 || self.serious_count > 0
    }
}

/// Default wall-clock budget for the axe-core subprocess. 60 s covers
/// cold-start `@axe-core/playwright` warmup on slow runners.
pub const DEFAULT_AXE_TIMEOUT_MS: u64 = 60_000;

/// Embedded Node helper invoking `@axe-core/playwright`. Mirrors the
/// `PlaywrightRunner` helper shape so the toolchain probe is identical.
const AXE_HELPER_JS: &str = r#"
// E-14 browser_e2e axe-core helper — runs axe-core against the URL.
const fs = require('fs');
const path = require('path');

function parseArgs(argv) {
  const out = {};
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith('--')) {
      const k = a.slice(2);
      out[k] = argv[i + 1];
      i++;
    }
  }
  return out;
}

(async () => {
  const args = parseArgs(process.argv);
  const url = args.url;
  const outDir = args.out;
  if (!url || !outDir) {
    console.log('ERR:missing_url_or_out');
    process.exit(2);
  }
  let playwright, AxeBuilder;
  try {
    playwright = require('playwright');
  } catch (e) {
    console.log('ERR:playwright_not_installed:' + (e && e.message ? e.message : e));
    process.exit(3);
  }
  try {
    AxeBuilder = require('@axe-core/playwright').default;
  } catch (e) {
    console.log('ERR:axe_core_not_installed:' + (e && e.message ? e.message : e));
    process.exit(4);
  }
  let browser;
  try {
    browser = await playwright.chromium.launch({ headless: true });
  } catch (e) {
    console.log('ERR:browser_launch_failed:' + (e && e.message ? e.message : e));
    process.exit(5);
  }
  try {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    await page.goto(url, { waitUntil: 'load', timeout: 30000 });
    const results = await new AxeBuilder({ page }).analyze();
    const resultsPath = path.join(outDir, 'axe-results.json');
    fs.writeFileSync(resultsPath, JSON.stringify(results));
    console.log('RESULTS_PATH:' + resultsPath);
    await ctx.close();
    console.log('OK');
  } catch (e) {
    console.log('ERR:axe_failed:' + (e && e.message ? e.message : e));
    process.exit(6);
  } finally {
    try { await browser.close(); } catch (_) {}
  }
})();
"#;

const AXE_HELPER_FILENAME: &str = "elai_axe_core_helper.js";

/// Run axe-core against `url`. `npx_path` empty -> resolved from PATH.
pub async fn run_axe_core(
    npx_path: &Path,
    url: &str,
    work_dir: &Path,
    timeout: Option<Duration>,
) -> Result<AxeReport, E2eError> {
    std::fs::create_dir_all(work_dir)?;
    let helper_path = work_dir.join(AXE_HELPER_FILENAME);
    std::fs::write(&helper_path, AXE_HELPER_JS)?;

    let binary: PathBuf = if npx_path.as_os_str().is_empty() {
        PathBuf::from("npx")
    } else {
        npx_path.to_path_buf()
    };
    let mut cmd = Command::new(&binary);
    cmd.arg("--no-install")
        .arg("node")
        .arg(&helper_path)
        .arg("--url")
        .arg(url)
        .arg("--out")
        .arg(work_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let timeout_ms = match timeout {
        Some(d) => u64::try_from(d.as_millis()).unwrap_or(DEFAULT_AXE_TIMEOUT_MS),
        None => DEFAULT_AXE_TIMEOUT_MS,
    };

    debug!(url, timeout_ms, "spawning axe-core helper");

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(E2eError::ToolchainMissing { tool: "npx" });
            }
            warn!(error = %e, "axe-core spawn failed");
            return Err(E2eError::Io(e));
        }
    };

    let output =
        match tokio::time::timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
        {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(E2eError::Io(e)),
            Err(_) => {
                return Err(E2eError::Timeout {
                    tool: "axe-core",
                    millis: timeout_ms,
                })
            }
        };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        if stdout.contains("ERR:playwright_not_installed") {
            return Err(E2eError::ToolchainMissing { tool: "playwright" });
        }
        if stdout.contains("ERR:axe_core_not_installed") {
            return Err(E2eError::ToolchainMissing {
                tool: "@axe-core/playwright",
            });
        }
        return Err(E2eError::SubprocessFailed {
            tool: "axe-core",
            code: output.status.code(),
            stderr: format!("stdout=`{stdout}` stderr=`{stderr}`"),
        });
    }

    let mut results_path: Option<PathBuf> = None;
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("RESULTS_PATH:") {
            results_path = Some(PathBuf::from(rest.trim()));
        }
    }
    let results_path = results_path.ok_or_else(|| E2eError::MalformedOutput {
        tool: "axe-core",
        detail: format!("no RESULTS_PATH line; stdout=`{stdout}`"),
    })?;
    let raw = std::fs::read_to_string(&results_path)?;
    parse_axe_results(&raw)
}

/// Parse the raw `axe-results.json` body into an `AxeReport`. Public so
/// tests can exercise the parser without needing the subprocess. Tolerates
/// missing impact fields (axe occasionally omits `impact` for incomplete
/// rules).
pub fn parse_axe_results(raw: &str) -> Result<AxeReport, E2eError> {
    let v: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| E2eError::MalformedOutput {
            tool: "axe-core",
            detail: format!("json parse failed: {e}"),
        })?;
    let violations = v.get("violations").and_then(|x| x.as_array()).ok_or_else(
        || E2eError::MalformedOutput {
            tool: "axe-core",
            detail: "missing `violations` array".into(),
        },
    )?;
    let mut findings: Vec<AxeFinding> = Vec::with_capacity(violations.len());
    let mut critical = 0usize;
    let mut serious = 0usize;
    let mut moderate = 0usize;
    let mut minor = 0usize;
    for vio in violations {
        let rule_id = vio
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_owned();
        let impact = match vio.get("impact").and_then(|x| x.as_str()) {
            Some("critical") => Some(AxeImpact::Critical),
            Some("serious") => Some(AxeImpact::Serious),
            Some("moderate") => Some(AxeImpact::Moderate),
            Some("minor") => Some(AxeImpact::Minor),
            _ => None,
        };
        let node_count = vio
            .get("nodes")
            .and_then(|x| x.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        match impact {
            Some(AxeImpact::Critical) => critical += 1,
            Some(AxeImpact::Serious) => serious += 1,
            Some(AxeImpact::Moderate) => moderate += 1,
            Some(AxeImpact::Minor) => minor += 1,
            None => {}
        }
        findings.push(AxeFinding {
            rule_id,
            impact,
            node_count,
        });
    }
    Ok(AxeReport {
        findings,
        critical_count: critical,
        serious_count: serious,
        moderate_count: moderate,
        minor_count: minor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_requires_axe_core_playwright() {
        assert!(AXE_HELPER_JS.contains("require('@axe-core/playwright')"));
        assert!(AXE_HELPER_JS.contains("RESULTS_PATH:"));
    }

    #[test]
    fn impact_blocking_only_critical_serious() {
        assert!(AxeImpact::Critical.is_blocking());
        assert!(AxeImpact::Serious.is_blocking());
        assert!(!AxeImpact::Moderate.is_blocking());
        assert!(!AxeImpact::Minor.is_blocking());
    }

    #[test]
    fn parse_axe_results_counts_each_impact() {
        let raw = r#"{
            "violations": [
                { "id": "color-contrast", "impact": "serious", "nodes": [{}, {}] },
                { "id": "label",          "impact": "critical","nodes": [{}] },
                { "id": "image-alt",      "impact": "moderate","nodes": [{}] },
                { "id": "region",         "impact": "minor",   "nodes": [{}] }
            ]
        }"#;
        let r = parse_axe_results(raw).expect("parse");
        assert_eq!(r.findings.len(), 4);
        assert_eq!(r.critical_count, 1);
        assert_eq!(r.serious_count, 1);
        assert_eq!(r.moderate_count, 1);
        assert_eq!(r.minor_count, 1);
        assert!(r.has_blocking());
    }

    #[test]
    fn parse_axe_results_no_violations_clean_report() {
        let raw = r#"{ "violations": [] }"#;
        let r = parse_axe_results(raw).expect("parse");
        assert!(r.findings.is_empty());
        assert!(!r.has_blocking());
    }

    #[test]
    fn parse_axe_results_missing_violations_array_is_malformed() {
        let raw = r#"{ "other": [] }"#;
        match parse_axe_results(raw) {
            Err(E2eError::MalformedOutput { tool, .. }) => assert_eq!(tool, "axe-core"),
            other => panic!("expected MalformedOutput, got {other:?}"),
        }
    }

    #[test]
    fn parse_axe_results_unknown_impact_keeps_finding_skips_count() {
        let raw = r#"{
            "violations": [
                { "id": "rule-x", "impact": "novel", "nodes": [{}] }
            ]
        }"#;
        let r = parse_axe_results(raw).expect("parse");
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].impact.is_none());
        assert_eq!(r.critical_count, 0);
    }
}
