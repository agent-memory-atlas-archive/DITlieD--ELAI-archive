//! C-27 — `elai wiring audit` CLI driver.
//!
//! Thin shell over [`elai_orchestrator::WiringAuditCmd::run`]. Resolves the
//! CLI-flag inputs into the cmd struct, calls `run()`, and renders the
//! returned [`WiringAuditReport`] in text / JSON / TOML. Exit code 0 on
//! clean, 1 on any violation. The orchestrator core owns the audit
//! semantics; this file owns only the I/O shape (flag parsing + report
//! formatting + process exit).
//!
//! C-27 BLOCK-1 follow-up: the contracts dir default now points at
//! `../docs/thefinalglance` (resolved relative to the CLI's current_dir,
//! which is `.elai_cc/` under the workspace root). The previous default
//! (`crates/orchestrator/tests`) scanned a Rust test directory with zero
//! `*.md` files, so DkgReachability + ObservableEffect auditors no-opped.

use std::path::PathBuf;

use elai_orchestrator::{
    WiringAuditCmd, WiringAuditError, WiringAuditReport, WiringAuditScope,
    WiringEvidencePayload, WiringViolationPayload,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WiringAuditOutput {
    Text,
    Json,
    Toml,
}

impl From<super::CliOutputFormat> for WiringAuditOutput {
    fn from(value: super::CliOutputFormat) -> Self {
        match value {
            super::CliOutputFormat::Text => Self::Text,
            super::CliOutputFormat::Json => Self::Json,
            super::CliOutputFormat::Toml => Self::Toml,
        }
    }
}

/// Resolve the default contracts directory. The CLI is invoked from
/// `.elai_cc/` (CI working-directory) so `../docs/thefinalglance` walks up
/// one level into the workflow root. If that path does not exist (e.g.,
/// the binary is run from elsewhere), fall back to the old
/// `crates/orchestrator/tests` default so existing scripts don't break.
fn default_contracts_dir(project_root: &std::path::Path) -> PathBuf {
    let preferred = project_root.join("..").join("docs").join("thefinalglance");
    if preferred.exists() {
        // Normalise the path so it prints cleanly in audit reports.
        preferred
            .canonicalize()
            .unwrap_or(preferred)
    } else {
        project_root.join("crates").join("orchestrator").join("tests")
    }
}

/// CLI entry point — sync wrapper that builds the [`WiringAuditCmd`] and
/// delegates to the orchestrator-side `run`.
pub(crate) fn run(
    scope: Option<String>,
    config_path: Option<String>,
    contracts_dir: Option<String>,
    output_format: super::CliOutputFormat,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let scope = match scope.as_deref() {
        Some(s) => WiringAuditScope::parse(s).map_err(anyhow::Error::msg)?,
        None => WiringAuditScope::All,
    };
    let config = config_path
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join("elai_cc.toml"));
    let contracts = contracts_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| default_contracts_dir(&project_root));

    let cmd = WiringAuditCmd {
        config_path: config,
        contracts_dir: contracts,
        source_root: project_root.clone(),
        ssb_db_path: None,
        // SP2 (WAVE 2 + WAVE 6 fix): None walks up from source_root through its
        // ancestors for the first .claude/tools/elai_graph.json that exists,
        // inside WiringAuditCmd::graph_path(). The canonical graph is generated
        // at the OUTER workflow root (one level above .elai_cc), so the inner
        // path alone would miss it. A CLI override flag can be wired later; the
        // reachability leg fails closed if no graph is found anywhere up-tree.
        graph_path: None,
        seed: 0,
        scope,
    };

    let report = match cmd.run() {
        Ok(r) => r,
        Err(WiringAuditError::ConfigMissing(p)) => {
            return Err(anyhow::Error::from(WiringAuditError::ConfigMissing(p)));
        }
        Err(WiringAuditError::ContractsDirMissing(p)) => {
            return Err(anyhow::Error::from(WiringAuditError::ContractsDirMissing(
                p,
            )));
        }
        Err(other) => return Err(anyhow::Error::from(other)),
    };

    let output_kind = WiringAuditOutput::from(output_format);
    render_report(output_kind, &report)?;

    if !report.is_clean() {
        std::process::exit(1);
    }
    Ok(())
}

fn render_report(
    output: WiringAuditOutput,
    report: &WiringAuditReport,
) -> anyhow::Result<()> {
    let evidence: &[WiringEvidencePayload] = &report.evidence;
    let violations: &[WiringViolationPayload] = &report.violations;
    let audit_id = report.audit_id;
    let total_violations = report.violation_count;

    match output {
        WiringAuditOutput::Json => {
            let json = serde_json::json!({
                "audit_id": audit_id.to_string(),
                "evidence_count": evidence.len(),
                "violation_count": violations.len(),
                "evidence": evidence,
                "violations": violations,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        WiringAuditOutput::Toml => {
            // toml::to_string requires serializable; emit a minimal table
            // structure built by hand to avoid coupling to toml-serde feature
            // flags (toml workspace pin uses the value API by default).
            println!("audit_id = \"{audit_id}\"");
            println!("evidence_count = {}", evidence.len());
            println!("violation_count = {}", violations.len());
            for v in violations {
                println!("[[violations]]");
                println!("kind = \"{:?}\"", v.violation_kind);
                println!("file = \"{}\"", v.file.replace('\\', "\\\\"));
                println!("line = {}", v.line);
                println!("pattern = \"{}\"", v.pattern.replace('"', "\\\""));
            }
        }
        WiringAuditOutput::Text => {
            for v in violations {
                println!(
                    "VIOLATION [{:?}] {}:{} pattern={}",
                    v.violation_kind, v.file, v.line, v.pattern
                );
            }
            for e in evidence {
                println!(
                    "EVIDENCE entrypoint={} reachable_count={} verified={}",
                    e.entrypoint,
                    e.reachable_count,
                    e.produces_symbols_verified.join(",")
                );
            }
            if total_violations == 0 {
                println!("WIRING AUDIT PASS (audit_id={audit_id})");
            } else {
                println!(
                    "WIRING AUDIT FAIL ({total_violations} violations; audit_id={audit_id})"
                );
            }
        }
    }
    Ok(())
}
