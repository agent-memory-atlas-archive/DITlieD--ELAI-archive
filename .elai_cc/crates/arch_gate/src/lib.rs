//! J-23 Tier-2 arch-gate catalog loader and prober.
//! Production consumer is the `elai review --catalog` arm added in slice 1c.
//! Shell-out is implemented in slice 2; SARIF ingestion is slice 3 and does not exist yet.
//! Catalog verdicts are observe-only until the GD-4 false-positive rate is bounded.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use elai_orchestrator::ssb::{ReviewGateCatalogEventKind, ReviewGateCatalogEventPayload};
use elai_orchestrator::{Ssb, SsbPayload};

pub const DEFAULT_CATALOG_RELPATH: &str = "crates/arch_gate/arch_gate.toml";

#[derive(Debug, thiserror::Error)]
pub enum ArchGateError {
    #[error("catalog missing: {0}")]
    CatalogMissing(PathBuf),
    #[error("catalog read {path}: {source}")]
    CatalogRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("catalog parse {path}: {source}")]
    CatalogParse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("invalid row {index} {tool}: {reason}")]
    CatalogRowInvalid {
        index: usize,
        tool: String,
        reason: String,
    },
    #[error("catalog empty: {0}")]
    CatalogEmpty(PathBuf),
    #[error("tool spawn {tool}: {source}")]
    ToolSpawn { tool: String, source: std::io::Error },
    #[error("tool output parse {tool} ({parser_fn}): {reason}")]
    ToolOutputParse { tool: String, parser_fn: &'static str, reason: String },
    #[error("SSB write {kind}: {reason}")]
    SsbWrite {
        kind: &'static str,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserFn {
    Json,
    ExitCode,
    TextRegex,
}

impl ParserFn {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::ExitCode => "exit_code",
            Self::TextRegex => "text_regex",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogRow {
    pub language: String,
    pub tool: String,
    pub version: String,
    pub invoke_cmd: Vec<String>,
    pub parser_fn: ParserFn,
    #[serde(default)]
    pub sarif: bool,
    pub license: String,
    #[serde(default)]
    pub staleness_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolProbe {
    Present(PathBuf),
    Missing,
}

#[derive(Debug, Clone)]
pub struct ProbedRow {
    pub row: CatalogRow,
    pub probe: ToolProbe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvokeOutcome {
    pub tool: String,
    pub finding_count: u64,
    /// Raw process exit code; None when the process was killed by a signal.
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CatalogReport {
    pub catalog_path: PathBuf,
    pub language_filter: Option<String>,
    pub rows: Vec<ProbedRow>,
    pub eligible_tool_count: u64,
    pub missing_tool_count: u64,
    pub invoked: Vec<InvokeOutcome>,
    pub total_finding_count: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogFile {
    #[serde(default)]
    tool: Vec<CatalogRow>,
}

pub fn load_catalog(path: &Path) -> Result<Vec<CatalogRow>, ArchGateError> {
    if !path.is_file() {
        return Err(ArchGateError::CatalogMissing(path.into()));
    }
    let text = std::fs::read_to_string(path).map_err(|source| ArchGateError::CatalogRead {
        path: path.into(),
        source,
    })?;
    let rows = toml::from_str::<CatalogFile>(&text)
        .map_err(|source| ArchGateError::CatalogParse {
            path: path.into(),
            source,
        })?
        .tool;
    if rows.is_empty() {
        return Err(ArchGateError::CatalogEmpty(path.into()));
    }
    for (index, row) in rows.iter().enumerate() {
        let reason = if row.invoke_cmd.is_empty() {
            Some("invoke_cmd is empty")
        } else if row.invoke_cmd[0].is_empty() {
            Some("invoke_cmd[0] is empty")
        } else if row.version.trim().is_empty() {
            Some("version pin is required (anti-slopsquat)")
        } else if row.language.trim().is_empty() {
            Some("language is required")
        } else if row.tool.trim().is_empty() {
            Some("tool is required")
        } else if row.license.trim().is_empty() {
            Some("license is required")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(ArchGateError::CatalogRowInvalid {
                index,
                tool: row.tool.clone(),
                reason: reason.into(),
            });
        }
    }
    Ok(rows)
}

pub fn probe_tool(binary: &str) -> ToolProbe {
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let p = dir.join(binary);
        if p.is_file() {
            return ToolProbe::Present(p);
        }
    }
    ToolProbe::Missing
}

pub fn invoke_row(row: &CatalogRow) -> Result<InvokeOutcome, ArchGateError> {
    let output = Command::new(&row.invoke_cmd[0])
        .args(&row.invoke_cmd[1..])
        .output()
        .map_err(|source| ArchGateError::ToolSpawn {
            tool: row.tool.clone(),
            source,
        })?;
    let exit_code = output.status.code();
    let finding_count = match row.parser_fn {
        ParserFn::ExitCode => u64::from(!output.status.success()),
        ParserFn::Json => {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
                ArchGateError::ToolOutputParse {
                    tool: row.tool.clone(),
                    parser_fn: "json",
                    reason: format!("invalid JSON: {e}"),
                }
            })?;
            match value {
                serde_json::Value::Array(items) => items.len() as u64,
                serde_json::Value::Object(mut object) => match object.remove("findings") {
                    Some(serde_json::Value::Array(items)) => items.len() as u64,
                    Some(_) => {
                        return Err(ArchGateError::ToolOutputParse {
                            tool: row.tool.clone(),
                            parser_fn: "json",
                            reason: "object field `findings` is not an array".into(),
                        });
                    }
                    None => {
                        return Err(ArchGateError::ToolOutputParse {
                            tool: row.tool.clone(),
                            parser_fn: "json",
                            reason: "object does not contain a `findings` array".into(),
                        });
                    }
                },
                _ => {
                    return Err(ArchGateError::ToolOutputParse {
                        tool: row.tool.clone(),
                        parser_fn: "json",
                        reason: "top-level JSON value is neither an array nor an object with a `findings` array".into(),
                    });
                }
            }
        }
        ParserFn::TextRegex => {
            return Err(ArchGateError::ToolOutputParse {
                tool: row.tool.clone(),
                parser_fn: "text_regex",
                reason: "text_regex parsing is not implemented (J-23 slice 2 covers json and exit_code only)".into(),
            });
        }
    };
    Ok(InvokeOutcome { tool: row.tool.clone(), finding_count, exit_code })
}

pub async fn run_catalog(
    project_root: &Path,
    catalog_path: Option<&Path>,
    language: Option<&str>,
    now_ms: i64,
    ssb: &Ssb,
) -> Result<CatalogReport, ArchGateError> {
    let path = catalog_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project_root.join(DEFAULT_CATALOG_RELPATH));
    let rows = load_catalog(&path)?;
    let mut out = Vec::new();
    let (mut eligible, mut missing) = (0, 0);
    let (mut invoked_count, mut total_findings, mut invoke_failed_count) = (0_u64, 0_u64, 0_u64);
    let mut invoked = Vec::new();
    for row in rows.into_iter().filter(|r| match language {
        Some(l) => r.language == l,
        None => true,
    }) {
        let probe = probe_tool(&row.invoke_cmd[0]);
        let (v, refusal) = match &probe {
            ToolProbe::Present(_) => {
                eligible += 1;
                ("eligible", None)
            }
            ToolProbe::Missing => {
                missing += 1;
                tracing::warn!(
                    tool = %row.tool,
                    language = %row.language,
                    binary = %row.invoke_cmd[0],
                    "catalog tool missing"
                );
                ("tool_missing", Some("tool_missing".into()))
            }
        };
        let payload = SsbPayload::ReviewGateCatalogEvent(ReviewGateCatalogEventPayload {
            kind: ReviewGateCatalogEventKind::CatalogBuilt,
            catalog_path: path.to_string_lossy().into(),
            language: Some(row.language.clone()),
            tool: Some(row.tool.clone()),
            tool_version: Some(row.version.clone()),
            parser_fn: Some(row.parser_fn.as_str().into()),
            sarif: row.sarif,
            eligible_tool_count: None,
            missing_tool_count: None,
            finding_count: None,
            verdict: Some(v.into()),
            refusal_reason: refusal,
            ts_ms: now_ms,
        });
        ssb.write_payload("cli:review-catalog", 0, payload, Vec::new())
            .await
            .map_err(|e| ArchGateError::SsbWrite {
                kind: "catalog_built_row",
                reason: e.to_string(),
            })?;
        if matches!(probe, ToolProbe::Present(_)) {
            let invocation = invoke_row(&row);
            let (finding_count, verdict, refusal_reason) = match invocation {
                Ok(outcome) => {
                    invoked_count += 1;
                    total_findings += outcome.finding_count;
                    let finding_count = outcome.finding_count;
                    invoked.push(outcome);
                    (Some(finding_count), "invoked", None)
                }
                Err(error) => {
                    invoke_failed_count += 1;
                    tracing::warn!(tool = %row.tool, error = %error, "catalog tool invocation failed");
                    (None, "invoke_failed", Some(error.to_string()))
                }
            };
            let payload = SsbPayload::ReviewGateCatalogEvent(ReviewGateCatalogEventPayload {
                kind: ReviewGateCatalogEventKind::GateInvoked,
                catalog_path: path.to_string_lossy().into(),
                language: Some(row.language.clone()),
                tool: Some(row.tool.clone()),
                tool_version: Some(row.version.clone()),
                parser_fn: Some(row.parser_fn.as_str().into()),
                sarif: row.sarif,
                eligible_tool_count: None,
                missing_tool_count: None,
                finding_count,
                verdict: Some(verdict.into()),
                refusal_reason,
                ts_ms: now_ms,
            });
            ssb.write_payload("cli:review-catalog", 0, payload, Vec::new())
                .await
                .map_err(|e| ArchGateError::SsbWrite {
                    kind: "gate_invoked_row",
                    reason: e.to_string(),
                })?;
        }
        out.push(ProbedRow { row, probe });
    }
    let payload = SsbPayload::ReviewGateCatalogEvent(ReviewGateCatalogEventPayload {
        kind: ReviewGateCatalogEventKind::CatalogBuilt,
        catalog_path: path.to_string_lossy().into(),
        language: language.map(str::to_string),
        tool: None,
        tool_version: None,
        parser_fn: None,
        sarif: false,
        eligible_tool_count: Some(eligible),
        missing_tool_count: Some(missing),
        finding_count: None,
        verdict: Some("catalog_built".into()),
        refusal_reason: None,
        ts_ms: now_ms,
    });
    ssb.write_payload("cli:review-catalog", 0, payload, Vec::new())
        .await
        .map_err(|e| ArchGateError::SsbWrite {
            kind: "catalog_aggregate_row",
            reason: e.to_string(),
        })?;
    let verdict = if total_findings > 0 {
        "warn"
    } else if invoke_failed_count > 0 {
        "warn_invoke_failed"
    } else {
        "clean"
    };
    let payload = SsbPayload::ReviewGateCatalogEvent(ReviewGateCatalogEventPayload {
        kind: ReviewGateCatalogEventKind::VerdictEmitted,
        catalog_path: path.to_string_lossy().into(),
        language: language.map(str::to_string),
        tool: None,
        tool_version: None,
        parser_fn: None,
        sarif: false,
        eligible_tool_count: Some(eligible),
        missing_tool_count: Some(missing),
        finding_count: Some(total_findings),
        verdict: Some(verdict.into()),
        refusal_reason: None,
        ts_ms: now_ms,
    });
    ssb.write_payload("cli:review-catalog", 0, payload, Vec::new())
        .await
        .map_err(|e| ArchGateError::SsbWrite {
            kind: "verdict_emitted_row",
            reason: e.to_string(),
        })?;
    let _ = invoked_count;
    Ok(CatalogReport {
        catalog_path: path,
        language_filter: language.map(str::to_string),
        rows: out,
        eligible_tool_count: eligible,
        missing_tool_count: missing,
        invoked,
        total_finding_count: total_findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use tempfile::tempdir;

    fn row_toml(
        language: &str,
        tool: &str,
        version: &str,
        invoke_cmd: &str,
        parser_fn: &str,
        license: &str,
    ) -> String {
        format!(
            "[[tool]]\nlanguage=\"{}\"\ntool=\"{}\"\nversion=\"{}\"\ninvoke_cmd={}\nparser_fn=\"{}\"\nlicense=\"{}\"\n",
            language, tool, version, invoke_cmd, parser_fn, license
        )
    }

    #[test]
    fn load_catalog_well_formed_two_rows() {
        let d = tempdir().unwrap();
        let p = d.path().join("c");
        fs::write(
            &p,
            format!(
                "{}\n{}",
                row_toml("python", "ruff", "1", "[\"ruff\"]", "json", "MIT"),
                row_toml(
                    "rust",
                    "clippy",
                    "1",
                    "[\"cargo\", \"clippy\"]",
                    "exit_code",
                    "MIT"
                )
            ),
        )
        .unwrap();
        let r = load_catalog(&p).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[1].parser_fn, ParserFn::ExitCode);
        assert!(!r[0].sarif);
    }

    #[test]
    fn load_catalog_absent_path() {
        assert!(matches!(
            load_catalog(Path::new("/no/j23")),
            Err(ArchGateError::CatalogMissing(_))
        ));
    }

    #[test]
    fn load_catalog_empty_invoke_cmd() {
        let d = tempdir().unwrap();
        let p = d.path().join("c");
        fs::write(
            &p,
            row_toml("python", "ruff", "1", "[]", "json", "MIT"),
        )
        .unwrap();
        assert!(matches!(
            load_catalog(&p),
            Err(ArchGateError::CatalogRowInvalid { reason, .. })
                if reason.contains("invoke_cmd")
        ));
    }

    #[test]
    fn load_catalog_empty_version() {
        let d = tempdir().unwrap();
        let p = d.path().join("c");
        fs::write(
            &p,
            row_toml("python", "ruff", "", "[\"ruff\"]", "json", "MIT"),
        )
        .unwrap();
        assert!(matches!(
            load_catalog(&p),
            Err(ArchGateError::CatalogRowInvalid { reason, .. })
                if reason.contains("version pin")
        ));
    }

    #[test]
    fn load_catalog_zero_rows() {
        let d = tempdir().unwrap();
        let p = d.path().join("c");
        fs::write(&p, "").unwrap();
        assert!(matches!(
            load_catalog(&p),
            Err(ArchGateError::CatalogEmpty(_))
        ));
    }

    #[test]
    fn load_catalog_unknown_parser() {
        let d = tempdir().unwrap();
        let p = d.path().join("c");
        fs::write(
            &p,
            row_toml("python", "ruff", "1", "[\"ruff\"]", "sniff", "MIT"),
        )
        .unwrap();
        assert!(matches!(
            load_catalog(&p),
            Err(ArchGateError::CatalogParse { .. })
        ));
    }

    #[test]
    fn probe_tool_missing() {
        assert!(matches!(
            probe_tool("__j23_missing__"),
            ToolProbe::Missing
        ));
    }

    #[cfg(unix)]
    fn invoke_test_row(command: Vec<String>, parser_fn: ParserFn) -> CatalogRow {
        CatalogRow {
            language: "test".into(),
            tool: "fixture".into(),
            version: "1".into(),
            invoke_cmd: command,
            parser_fn,
            sarif: false,
            license: "MIT".into(),
            staleness_flag: false,
        }
    }

    #[cfg(unix)]
    #[test]
    fn invoke_row_exit_code_zero_is_clean() {
        let row = invoke_test_row(vec!["sh".into(), "-c".into(), "true".into()], ParserFn::ExitCode);
        let outcome = invoke_row(&row).unwrap();
        assert_eq!(outcome.finding_count, 0);
        assert_eq!(outcome.exit_code, Some(0));
    }

    #[cfg(unix)]
    #[test]
    fn invoke_row_exit_code_nonzero_is_one_finding() {
        let row = invoke_test_row(vec!["sh".into(), "-c".into(), "exit 3".into()], ParserFn::ExitCode);
        let outcome = invoke_row(&row).unwrap();
        assert_eq!(outcome.finding_count, 1);
        assert_eq!(outcome.exit_code, Some(3));
    }

    #[cfg(unix)]
    #[test]
    fn invoke_row_json_array_counts_elements() {
        let row = invoke_test_row(
            vec!["sh".into(), "-c".into(), "printf '[{\"a\":1},{\"a\":2}]'".into()],
            ParserFn::Json,
        );
        assert_eq!(invoke_row(&row).unwrap().finding_count, 2);
    }

    #[cfg(unix)]
    #[test]
    fn invoke_row_json_findings_object_counts_array() {
        let row = invoke_test_row(
            vec!["sh".into(), "-c".into(), "printf '{\"findings\":[{},{},{}]}'".into()],
            ParserFn::Json,
        );
        assert_eq!(invoke_row(&row).unwrap().finding_count, 3);
    }

    #[cfg(unix)]
    #[test]
    fn invoke_row_json_unparseable_is_typed_error() {
        let row = invoke_test_row(vec!["sh".into(), "-c".into(), "printf 'not json'".into()], ParserFn::Json);
        let result = invoke_row(&row);
        assert!(matches!(result, Err(ArchGateError::ToolOutputParse { .. })),
            "unparseable JSON must be a typed error, never an Ok zero-finding result");
    }

    #[test]
    fn invoke_row_missing_binary_is_tool_spawn_error() {
        let row = CatalogRow {
            language: "test".into(),
            tool: "missing".into(),
            version: "1".into(),
            invoke_cmd: vec!["elai-nonexistent-tool-j23".into()],
            parser_fn: ParserFn::ExitCode,
            sarif: false,
            license: "MIT".into(),
            staleness_flag: false,
        };
        assert!(matches!(invoke_row(&row), Err(ArchGateError::ToolSpawn { .. })));
    }

    #[test]
    fn invoke_row_text_regex_refuses_loudly() {
        let row = CatalogRow {
            language: "test".into(),
            tool: "fixture".into(),
            version: "1".into(),
            invoke_cmd: vec!["true".into()],
            parser_fn: ParserFn::TextRegex,
            sarif: false,
            license: "MIT".into(),
            staleness_flag: false,
        };
        assert!(matches!(invoke_row(&row), Err(ArchGateError::ToolOutputParse { parser_fn: "text_regex", .. })));
    }

}
