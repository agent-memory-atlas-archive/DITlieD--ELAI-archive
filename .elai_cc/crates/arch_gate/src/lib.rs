//! J-23 Tier-2 arch-gate catalog loader and prober.
//! Production consumer is the `elai review --catalog` arm added in slice 1c.
//! Shell-out is slice 2 and SARIF ingestion is slice 3; neither exists yet.
//! Catalog verdicts are observe-only until the GD-4 false-positive rate is bounded.

use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone)]
pub struct CatalogReport {
    pub catalog_path: PathBuf,
    pub language_filter: Option<String>,
    pub rows: Vec<ProbedRow>,
    pub eligible_tool_count: u64,
    pub missing_tool_count: u64,
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
    Ok(CatalogReport {
        catalog_path: path,
        language_filter: language.map(str::to_string),
        rows: out,
        eligible_tool_count: eligible,
        missing_tool_count: missing,
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
}
