//! E-41: Semgrep gate — shells `semgrep` over the working copy and
//! summarises findings by severity.
//!
//! Wire-dark fallback: when the `semgrep` binary is absent from PATH the
//! call returns [`GithubPrError::InvalidRequest`] with a `ToolchainMissing`
//! marker reason so the dashboard / CLI can render an actionable warn
//! (install semgrep) instead of treating absence as PASS.
//!
//! SSB emission: one `pr_semgrep_gate_result` row per call, carrying the
//! aggregated finding count + max-severity hash.

use std::sync::Arc;

use tokio::process::Command;
use tracing::warn;

use crate::ssb::{PrSemgrepGateResultPayload, Ssb, SsbPayload};

use super::error::GithubPrError;
use super::gh_cli::GhCliAdapter;

const SEMGREP_AGENT_ID: &str = "github-pr-semgrep";
const SEMGREP_BINARY: &str = "semgrep";

/// Severity classes we lift from semgrep's JSON output. The mapping is
/// lossy on purpose — semgrep adds severities over time, and dashboards
/// branch on these five canonical labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemgrepSeverity {
    Info,
    Warning,
    Error,
    /// Semgrep's `ERROR` with an `OWASP` / `LLM` tag — escalated to
    /// the highest tier so the dashboard surfaces it red regardless of
    /// the underlying severity field.
    OwaspLlm,
    /// Catch-all for severities we don't recognise; treated as warning
    /// at the consumer level.
    Other,
}

impl SemgrepSeverity {
    /// Canonical wire label fed into the SSB `severity_max_hash`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SemgrepSeverity::Info => "info",
            SemgrepSeverity::Warning => "warning",
            SemgrepSeverity::Error => "error",
            SemgrepSeverity::OwaspLlm => "owasp_llm",
            SemgrepSeverity::Other => "other",
        }
    }
}

/// Result of one Semgrep gate run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemgrepResult {
    /// Total number of findings across all severities.
    pub findings_count: u32,
    /// Highest severity observed in this run (or `Info` when zero
    /// findings).
    pub severity_max: SemgrepSeverity,
}

/// Run semgrep against the current working directory with the supplied
/// ruleset (e.g. `"p/ci"`, `"auto"`, or a local `.semgrep/` path).
/// Emits one `pr_semgrep_gate_result` SSB row.
///
/// # Errors
/// - [`GithubPrError::InvalidRequest`] with reason `"ToolchainMissing: semgrep"`
///   when the binary is not on PATH.
/// - [`GithubPrError::GhCommandFailed`] for semgrep exit codes >= 2
///   (1 == findings present, treated as success here so the caller can
///   decide blocking semantics separately).
pub async fn run_semgrep_gate(
    ssb: Arc<Ssb>,
    pr_number: u32,
    ruleset: &str,
) -> Result<SemgrepResult, GithubPrError> {
    if ruleset.trim().is_empty() {
        return Err(GithubPrError::InvalidRequest {
            reason: "semgrep ruleset must not be empty".to_string(),
        });
    }
    // Probe presence first so we can surface ToolchainMissing distinctly.
    match Command::new(SEMGREP_BINARY).arg("--version").output().await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // ToolchainMissing wire-dark — emit the SSB row with zero
            // findings + an explicit "missing" severity hash so the
            // dashboard can render the warning.
            let payload = SsbPayload::PrSemgrepGateResult(PrSemgrepGateResultPayload {
                pr_number_hash: GhCliAdapter::hash_prefix(&pr_number.to_string()),
                findings_count: 0,
                severity_max_hash: GhCliAdapter::hash_prefix("toolchain_missing"),
                ts_ms: super::clock::now_ms(),
            });
            if let Err(emit_err) = ssb
                .write_payload(SEMGREP_AGENT_ID, 0, payload, vec![])
                .await
            {
                warn!(
                    target: "github_pr",
                    error = %emit_err,
                    "ssb write failed for semgrep toolchain-missing row; row dropped",
                );
            }
            return Err(GithubPrError::InvalidRequest {
                reason: "ToolchainMissing: semgrep binary not on PATH".to_string(),
            });
        }
        Err(e) => return Err(GithubPrError::from(e)),
    }

    // Run semgrep with `--json` for parseable output.
    let argv = ["--config", ruleset, "--json", "--quiet"];
    let output = Command::new(SEMGREP_BINARY)
        .args(argv)
        .output()
        .await
        .map_err(GithubPrError::from)?;
    // semgrep exit codes: 0 = no findings, 1 = findings present, >=2 = error.
    let exit_code = output.status.code().unwrap_or(2);
    if exit_code >= 2 {
        let label = format!("semgrep {}", argv.join(" "));
        return Err(GithubPrError::from_exit(label, output.status, output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let (findings_count, severity_max) = parse_semgrep_findings(&stdout);

    let payload = SsbPayload::PrSemgrepGateResult(PrSemgrepGateResultPayload {
        pr_number_hash: GhCliAdapter::hash_prefix(&pr_number.to_string()),
        findings_count,
        severity_max_hash: GhCliAdapter::hash_prefix(severity_max.label()),
        ts_ms: super::clock::now_ms(),
    });
    if let Err(e) = ssb.write_payload(SEMGREP_AGENT_ID, 0, payload, vec![]).await {
        warn!(
            target: "github_pr",
            error = %e,
            "ssb write failed for pr_semgrep_gate_result row; row dropped",
        );
    }

    Ok(SemgrepResult {
        findings_count,
        severity_max,
    })
}

/// Parse semgrep's `--json` output (a JSON object with a `results`
/// array). Returns the finding count + highest severity observed.
/// Pure function — exposed for unit tests.
#[must_use]
pub fn parse_semgrep_findings(json: &str) -> (u32, SemgrepSeverity) {
    let trimmed = json.trim();
    if trimmed.is_empty() {
        return (0, SemgrepSeverity::Info);
    }
    let value: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return (0, SemgrepSeverity::Info),
    };
    let results = value.get("results").and_then(|r| r.as_array());
    let Some(arr) = results else {
        return (0, SemgrepSeverity::Info);
    };
    let count = u32::try_from(arr.len()).unwrap_or(u32::MAX);
    let mut max = SemgrepSeverity::Info;
    for r in arr {
        let extra = r.get("extra");
        let severity = extra
            .and_then(|e| e.get("severity"))
            .and_then(|s| s.as_str())
            .unwrap_or("");
        let metadata = extra.and_then(|e| e.get("metadata"));
        let owasp_tag = metadata
            .and_then(|m| m.get("owasp"))
            .map(|v| !v.is_null())
            .unwrap_or(false);
        let category = metadata
            .and_then(|m| m.get("category"))
            .and_then(|c| c.as_str())
            .unwrap_or("");
        let s = classify_severity(severity, owasp_tag, category);
        if s > max {
            max = s;
        }
    }
    (count, max)
}

fn classify_severity(severity_field: &str, owasp_tag: bool, category: &str) -> SemgrepSeverity {
    if owasp_tag || category.eq_ignore_ascii_case("llm") {
        return SemgrepSeverity::OwaspLlm;
    }
    match severity_field.to_ascii_uppercase().as_str() {
        "INFO" => SemgrepSeverity::Info,
        "WARNING" => SemgrepSeverity::Warning,
        "ERROR" => SemgrepSeverity::Error,
        _ => SemgrepSeverity::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_json_returns_zero() {
        let (count, sev) = parse_semgrep_findings("");
        assert_eq!(count, 0);
        assert_eq!(sev, SemgrepSeverity::Info);
    }

    #[test]
    fn single_warning_classified() {
        let json = r#"{ "results": [ { "extra": { "severity": "WARNING" } } ] }"#;
        let (count, sev) = parse_semgrep_findings(json);
        assert_eq!(count, 1);
        assert_eq!(sev, SemgrepSeverity::Warning);
    }

    #[test]
    fn owasp_escalates() {
        let json = r#"{ "results": [
            { "extra": { "severity": "WARNING", "metadata": { "owasp": ["A03:2021"] } } },
            { "extra": { "severity": "INFO" } }
        ] }"#;
        let (count, sev) = parse_semgrep_findings(json);
        assert_eq!(count, 2);
        assert_eq!(sev, SemgrepSeverity::OwaspLlm);
    }

    #[test]
    fn max_severity_picked() {
        let json = r#"{ "results": [
            { "extra": { "severity": "INFO" } },
            { "extra": { "severity": "ERROR" } },
            { "extra": { "severity": "WARNING" } }
        ] }"#;
        let (_, sev) = parse_semgrep_findings(json);
        assert_eq!(sev, SemgrepSeverity::Error);
    }
}
