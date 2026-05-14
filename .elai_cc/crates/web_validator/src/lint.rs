//! WEB.3 — Phase B / Stream B2 — real WebLint runner.
//!
//! Spawns ESLint, Stylelint, and axe-core (`@axe-core/cli`) via `npx --yes
//! <tool> --format=json` against a target project directory and parses the
//! resulting JSON into a uniform `LintFinding` stream.
//!
//! Crate boundary contract: this module never returns `WebValidationTier`;
//! it returns plain `Vec<LintFinding>`. The orchestrator wires findings
//! into its own `IntegrationCheckpoint` records (same pattern as
//! `WebBuildResult` / `DevServerStatus`).
//!
//! Anti-slopsquatting (verified 2026-05-08 via `npm view`):
//!   eslint            (eslint/eslint, official)            v10.3.0
//!   stylelint         (stylelint/stylelint, official)      v17.11.0
//!   @axe-core/cli     (dequelabs/axe-core-npm, official)   v4.11.3
//!
//! Tools live in the *target* project's node_modules, not in any Cargo
//! manifest. We invoke them via `npx --yes <tool>` so `npm install` of the
//! target project must precede a lint run.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::WebValidatorError;

// ---------------------------------------------------------------------------
// Named constants — no magic numbers.
// ---------------------------------------------------------------------------

/// Per-tool subprocess timeout. ESLint on a fresh project typically needs
/// 5-15s; 60s is a generous ceiling.
pub const LINT_TOOL_TIMEOUT_SECS: u64 = 60;

/// Stderr capture cap (mirrors `MAX_STDERR_BYTES` in `lib.rs`).
pub const LINT_MAX_STDERR_BYTES: usize = 64 * 1024;

/// Stdout capture cap. Lint output JSON can be large but we cap defensively
/// to avoid OOM if a project has thousands of files; 4 MiB is generous.
pub const LINT_MAX_STDOUT_BYTES: usize = 4 * 1024 * 1024;

const RULE_PREFIX_ESLINT: &str = "eslint:";
const RULE_PREFIX_STYLELINT: &str = "stylelint:";
const RULE_PREFIX_AXE: &str = "axe:";

const NODE_PROBE_COMMAND: &str = "node --version";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Per-finding severity. Mapped from each tool's native severity scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    /// Equivalent to ESLint level 2 / axe `serious`+`critical`.
    Error,
    /// Equivalent to ESLint level 1 / axe `moderate`+`minor`.
    Warning,
    /// Informational only.
    Info,
}

/// A single lint finding from one of the wrapped tools.
///
/// `rule_id` is namespaced per tool: `eslint:no-undef`, `stylelint:color-hex-length`,
/// `axe:color-contrast`. Phase A's WEB-* rule numbering is left untouched —
/// findings from this runner never collide with the in-house rule pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintFinding {
    pub rule_id: String,
    pub severity: LintSeverity,
    pub file: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: String,
    pub fixable: bool,
}

/// Which tools to run.
///
/// `default()` returns an all-disabled config: enabling a tool is an explicit
/// caller decision so that toolchains lacking node never auto-fail. The
/// orchestrator decides per-project whether each tool should fire.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintRunnerConfig {
    pub eslint: bool,
    pub stylelint: bool,
    pub axe: bool,
    /// If true, treat `npx` "package not found" as a soft skip (returns no
    /// findings for that tool) rather than an error. Useful for projects
    /// that opt out of one specific linter.
    #[serde(default)]
    pub skip_missing_packages: bool,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run all enabled lint tools against `project_dir` and return their findings
/// as a single flat vector.
///
/// Behaviour:
///   - If `cfg` enables zero tools, returns `Ok(Vec::new())` immediately. No
///     subprocess is spawned; node presence is NOT probed.
///   - Otherwise probes `node --version` once. If node is absent, returns
///     `Err(NodeMissing)` without invoking any tool.
///   - For each enabled tool: spawns `npx --yes <tool> --format=json`,
///     captures stdout, parses, appends findings. A single tool failure
///     produces an `Err(ToolOutputInvalid)` unless
///     `cfg.skip_missing_packages` is set, in which case it logs and skips.
pub async fn run_web_lint(
    project_dir: &Path,
    cfg: &LintRunnerConfig,
) -> Result<Vec<LintFinding>, WebValidatorError> {
    if !cfg.eslint && !cfg.stylelint && !cfg.axe {
        return Ok(Vec::new());
    }

    probe_node().await?;

    let mut findings = Vec::new();

    if cfg.eslint {
        run_one_tool(project_dir, ToolKind::Eslint, cfg.skip_missing_packages, &mut findings).await?;
    }
    if cfg.stylelint {
        run_one_tool(project_dir, ToolKind::Stylelint, cfg.skip_missing_packages, &mut findings).await?;
    }
    if cfg.axe {
        run_one_tool(project_dir, ToolKind::Axe, cfg.skip_missing_packages, &mut findings).await?;
    }

    Ok(findings)
}

// ---------------------------------------------------------------------------
// Internal — tool invocation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum ToolKind {
    Eslint,
    Stylelint,
    Axe,
}

impl ToolKind {
    fn npx_args(self, project_dir: &Path) -> (&'static str, Vec<String>) {
        match self {
            ToolKind::Eslint => (
                "npx",
                vec![
                    "--yes".into(),
                    "eslint".into(),
                    "--format".into(),
                    "json".into(),
                    project_dir.to_string_lossy().into_owned(),
                ],
            ),
            ToolKind::Stylelint => (
                "npx",
                vec![
                    "--yes".into(),
                    "stylelint".into(),
                    "--formatter".into(),
                    "json".into(),
                    "**/*.{css,scss,svelte,astro}".into(),
                ],
            ),
            ToolKind::Axe => (
                "npx",
                vec![
                    "--yes".into(),
                    "@axe-core/cli".into(),
                    project_dir.to_string_lossy().into_owned(),
                    "--stdout".into(),
                ],
            ),
        }
    }

    fn rule_prefix(self) -> &'static str {
        match self {
            ToolKind::Eslint => RULE_PREFIX_ESLINT,
            ToolKind::Stylelint => RULE_PREFIX_STYLELINT,
            ToolKind::Axe => RULE_PREFIX_AXE,
        }
    }

    fn label(self) -> &'static str {
        match self {
            ToolKind::Eslint => "eslint",
            ToolKind::Stylelint => "stylelint",
            ToolKind::Axe => "axe",
        }
    }
}

async fn run_one_tool(
    project_dir: &Path,
    tool: ToolKind,
    skip_missing: bool,
    out: &mut Vec<LintFinding>,
) -> Result<(), WebValidatorError> {
    let (program, args) = tool.npx_args(project_dir);
    let mut cmd = crate::build_command(program, &args);
    cmd.current_dir(project_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let output_future = cmd.output();
    let output = match tokio::time::timeout(
        Duration::from_secs(LINT_TOOL_TIMEOUT_SECS),
        output_future,
    )
    .await
    {
        Ok(Ok(o)) => o,
        Ok(Err(source)) => {
            return Err(WebValidatorError::ProcessSpawn {
                command: format!("{program} {}", args.join(" ")),
                source,
            });
        }
        Err(_) => {
            return Err(WebValidatorError::Timeout {
                what: format!("{} lint", tool.label()),
                after_secs: LINT_TOOL_TIMEOUT_SECS,
            });
        }
    };

    // npx exits non-zero when (a) lint findings were produced (eslint exits
    // 1 with findings — expected) OR (b) the package wasn't installable.
    // Distinguish: parseable JSON => findings exist; unparseable => probably
    // package missing or tool crashed.
    let stdout_limit = output.stdout.len().min(LINT_MAX_STDOUT_BYTES);
    let stdout = String::from_utf8_lossy(&output.stdout[..stdout_limit]).into_owned();

    if stdout.trim().is_empty() {
        if skip_missing {
            tracing::warn!(
                tool = tool.label(),
                "lint produced empty output; treating as soft-skip"
            );
            return Ok(());
        }
        let stderr_limit = output.stderr.len().min(LINT_MAX_STDERR_BYTES);
        let stderr = String::from_utf8_lossy(&output.stderr[..stderr_limit]).into_owned();
        return Err(WebValidatorError::ToolOutputInvalid {
            tool: tool.label().into(),
            reason: format!("empty stdout; stderr: {stderr}"),
        });
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| WebValidatorError::ToolOutputInvalid {
            tool: tool.label().into(),
            reason: format!("json parse: {e}"),
        })?;

    extend_findings_from_json(tool, &parsed, project_dir, out);
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal — JSON shape adapters per tool
// ---------------------------------------------------------------------------

fn extend_findings_from_json(
    tool: ToolKind,
    root: &serde_json::Value,
    project_dir: &Path,
    out: &mut Vec<LintFinding>,
) {
    match tool {
        ToolKind::Eslint | ToolKind::Stylelint => {
            // ESLint and Stylelint both emit an array of file results.
            if let Some(arr) = root.as_array() {
                for entry in arr {
                    push_per_file_messages(tool, entry, project_dir, out);
                }
            }
        }
        ToolKind::Axe => {
            // axe-core/cli emits a structure with a `violations` array.
            if let Some(violations) = root.get("violations").and_then(|v| v.as_array()) {
                for v in violations {
                    push_axe_violation(v, project_dir, out);
                }
            }
        }
    }
}

fn push_per_file_messages(
    tool: ToolKind,
    file_entry: &serde_json::Value,
    project_dir: &Path,
    out: &mut Vec<LintFinding>,
) {
    let file_path = file_entry
        .get("filePath")
        .or_else(|| file_entry.get("source"))
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| project_dir.to_path_buf());

    let messages = match tool {
        ToolKind::Eslint => file_entry.get("messages"),
        ToolKind::Stylelint => file_entry.get("warnings"),
        ToolKind::Axe => unreachable!("axe handled separately"),
    };

    let Some(arr) = messages.and_then(|m| m.as_array()) else {
        return;
    };

    for m in arr {
        let raw_rule = m
            .get("ruleId")
            .or_else(|| m.get("rule"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let severity = match tool {
            ToolKind::Eslint => match m.get("severity").and_then(|v| v.as_i64()).unwrap_or(1) {
                2 => LintSeverity::Error,
                1 => LintSeverity::Warning,
                _ => LintSeverity::Info,
            },
            ToolKind::Stylelint => {
                match m.get("severity").and_then(|v| v.as_str()).unwrap_or("warning") {
                    "error" => LintSeverity::Error,
                    "warning" => LintSeverity::Warning,
                    _ => LintSeverity::Info,
                }
            }
            ToolKind::Axe => unreachable!(),
        };
        let line = m
            .get("line")
            .and_then(|v| v.as_u64())
            .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
        let column = m
            .get("column")
            .and_then(|v| v.as_u64())
            .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
        let message = m
            .get("message")
            .or_else(|| m.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("(no message)")
            .to_owned();
        let fixable = m
            .get("fix")
            .map(|v| !v.is_null())
            .or_else(|| m.get("fixable").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        out.push(LintFinding {
            rule_id: format!("{}{}", tool.rule_prefix(), raw_rule),
            severity,
            file: file_path.clone(),
            line,
            column,
            message,
            fixable,
        });
    }
}

fn push_axe_violation(
    violation: &serde_json::Value,
    project_dir: &Path,
    out: &mut Vec<LintFinding>,
) {
    let rule = violation
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let severity = match violation
        .get("impact")
        .and_then(|v| v.as_str())
        .unwrap_or("minor")
    {
        "critical" | "serious" => LintSeverity::Error,
        "moderate" => LintSeverity::Warning,
        _ => LintSeverity::Info,
    };
    let description = violation
        .get("help")
        .or_else(|| violation.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("(no description)")
        .to_owned();

    if let Some(nodes) = violation.get("nodes").and_then(|v| v.as_array()) {
        for n in nodes {
            let target = n
                .get("target")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .unwrap_or("(no selector)");
            out.push(LintFinding {
                rule_id: format!("{RULE_PREFIX_AXE}{rule}"),
                severity,
                file: project_dir.to_path_buf(),
                line: None,
                column: None,
                message: format!("{description} — selector: {target}"),
                fixable: false,
            });
        }
    } else {
        out.push(LintFinding {
            rule_id: format!("{RULE_PREFIX_AXE}{rule}"),
            severity,
            file: project_dir.to_path_buf(),
            line: None,
            column: None,
            message: description,
            fixable: false,
        });
    }
}

// ---------------------------------------------------------------------------
// Internal — node probe (shared with cwv::run_lighthouse via pub(crate))
// ---------------------------------------------------------------------------

/// Probe `node --version` once. Returns Ok if the binary responded
/// successfully within 5 s, `NodeMissing` otherwise. Shared with `cwv` so
/// only one implementation lives in the crate.
pub(crate) async fn probe_node() -> Result<(), WebValidatorError> {
    let probe = Command::new("node")
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .output();

    match tokio::time::timeout(Duration::from_secs(5), probe).await {
        Ok(Ok(out)) if out.status.success() => Ok(()),
        _ => Err(WebValidatorError::NodeMissing {
            probe_command: NODE_PROBE_COMMAND.into(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_runner_config_default_disables_all_tools() {
        let cfg = LintRunnerConfig::default();
        assert!(!cfg.eslint);
        assert!(!cfg.stylelint);
        assert!(!cfg.axe);
        assert!(!cfg.skip_missing_packages);
    }

    #[test]
    fn lint_severity_round_trips_lowercase() {
        for sev in [LintSeverity::Error, LintSeverity::Warning, LintSeverity::Info] {
            let s = serde_json::to_string(&sev).expect("serialize");
            let back: LintSeverity = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(sev, back);
        }
    }

    #[test]
    fn extend_findings_eslint_basic_shape() {
        let json = serde_json::json!([
            {
                "filePath": "/p/src/App.svelte",
                "messages": [
                    {
                        "ruleId": "no-undef",
                        "severity": 2,
                        "line": 10,
                        "column": 5,
                        "message": "x is not defined",
                        "fix": null
                    },
                    {
                        "ruleId": "no-console",
                        "severity": 1,
                        "line": 12,
                        "column": 1,
                        "message": "Unexpected console statement"
                    }
                ]
            }
        ]);
        let mut out = Vec::new();
        extend_findings_from_json(ToolKind::Eslint, &json, Path::new("/p"), &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].rule_id, "eslint:no-undef");
        assert_eq!(out[0].severity, LintSeverity::Error);
        assert_eq!(out[0].line, Some(10));
        assert_eq!(out[1].rule_id, "eslint:no-console");
        assert_eq!(out[1].severity, LintSeverity::Warning);
    }

    #[test]
    fn extend_findings_stylelint_basic_shape() {
        let json = serde_json::json!([
            {
                "source": "/p/src/styles/global.css",
                "warnings": [
                    {
                        "rule": "color-hex-length",
                        "severity": "error",
                        "line": 4,
                        "column": 11,
                        "text": "Expected #ffffff to be #fff"
                    }
                ]
            }
        ]);
        let mut out = Vec::new();
        extend_findings_from_json(ToolKind::Stylelint, &json, Path::new("/p"), &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].rule_id, "stylelint:color-hex-length");
        assert_eq!(out[0].severity, LintSeverity::Error);
        assert_eq!(out[0].line, Some(4));
    }

    #[test]
    fn extend_findings_axe_includes_selector_in_message() {
        let json = serde_json::json!({
            "violations": [
                {
                    "id": "color-contrast",
                    "impact": "serious",
                    "help": "Elements must have sufficient color contrast",
                    "nodes": [{"target": ["p.muted"]}, {"target": ["span.note"]}]
                }
            ]
        });
        let mut out = Vec::new();
        extend_findings_from_json(ToolKind::Axe, &json, Path::new("/p"), &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].rule_id, "axe:color-contrast");
        assert_eq!(out[0].severity, LintSeverity::Error);
        assert!(out[0].message.contains("p.muted"));
        assert!(out[1].message.contains("span.note"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_web_lint_no_tools_enabled_short_circuits() {
        let cfg = LintRunnerConfig::default();
        let dir = std::env::temp_dir();
        let res = run_web_lint(&dir, &cfg).await.expect("disabled-config call must succeed");
        assert!(res.is_empty());
    }
}
