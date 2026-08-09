//! `elai review --catalog` handler.
//!
//! This module loads the J-23 arch-gate catalog, probes each tool binary, and emits
//! `review_gate_catalog_event` rows through the run's live SSB handle. The per-tool
//! shell-out is J-23 slice 2 and SARIF ingestion is slice 3; neither exists yet.
//! Catalog verdicts are observe-only and never block, so a missing tool is a loud
//! finding with exit code 0.
//!
//! The catalog handler lives separately because it crossed `review_cmd.rs` past the
//! 1000-line maintainability boundary and belongs in its own focused module.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};

use crate::args::CliOutputFormat;

/// Stable session bucket for observe-only catalog rows.
const CATALOG_SESSION_ID: &str = "cli-review-catalog";

/// Resolve the project root and print the observe-only catalog report.
pub(crate) fn run_catalog_cmd(
    language: Option<String>,
    catalog_path: Option<PathBuf>,
    output_format: CliOutputFormat,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("review --catalog: cannot determine cwd: {e}"))?;
    let report = run_catalog_at(&project_root, language.as_deref(), catalog_path.as_deref())?;
    match output_format {
        CliOutputFormat::Json => {
            let rows: Vec<_> = report.rows.iter().map(|r| {
                serde_json::json!({
                    "language": &r.row.language,
                    "tool": &r.row.tool,
                    "pinned_version": &r.row.version,
                    "binary_found": matches!(r.probe, elai_arch_gate::ToolProbe::Present(_)),
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "catalog_path": report.catalog_path,
                "rows": rows,
                "eligible_tool_count": report.eligible_tool_count,
                "missing_tool_count": report.missing_tool_count,
                "invoked": report.invoked.iter().map(|o| serde_json::json!({
                    "tool": &o.tool,
                    "finding_count": o.finding_count,
                    "exit_code": o.exit_code,
                })).collect::<Vec<_>>(),
                "total_finding_count": report.total_finding_count,
            }))?);
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            for row in &report.rows {
                let found = matches!(row.probe, elai_arch_gate::ToolProbe::Present(_));
                println!("{} {} pinned_version={} binary_found={}", row.row.language, row.row.tool, row.row.version, found);
                if !found {
                    println!("TOOL_MISSING tool={} binary={}", row.row.tool, row.row.invoke_cmd[0]);
                }
            }
            for outcome in &report.invoked {
                println!("GATE_INVOKED tool={} findings={} exit_code={}", outcome.tool, outcome.finding_count, outcome.exit_code.map_or_else(|| "none".to_string(), |code| code.to_string()));
            }
            println!("catalog summary: eligible={} missing={}", report.eligible_tool_count, report.missing_tool_count);
            println!("catalog verdict: total_findings={} invoked={}", report.total_finding_count, report.invoked.len());
        }
    }
    // Catalog verdicts are observe-only; missing tools are findings, not blocks.
    Ok(())
}

/// Testable catalog runner: opens the run's SSB bucket and delegates probing.
fn run_catalog_at(
    project_root: &Path,
    language: Option<&str>,
    catalog_path: Option<&Path>,
) -> anyhow::Result<elai_arch_gate::CatalogReport> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    let parent = db_path.parent().ok_or_else(|| anyhow::anyhow!("review --catalog: ssb.db has no parent dir"))?;
    std::fs::create_dir_all(parent).map_err(|e| anyhow::anyhow!("review --catalog: create sessions dir failed: {e}"))?;
    let project_id = elai_core::project_id_from_path(project_root);
    let cfg = SsbConfig { db_path, pool_size: None, project_id, session_id: CATALOG_SESSION_ID.to_string() };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()
        .map_err(|e| anyhow::anyhow!("review --catalog: build async runtime failed: {e}"))?;
    let now_ms = clock.now_ms() as i64;
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| anyhow::anyhow!("review --catalog: open SSB handle failed: {e}"))?;
        elai_arch_gate::run_catalog(project_root, catalog_path, language, now_ms, &ssb)
            .await
            .map_err(|e| anyhow::anyhow!("review --catalog: catalog run failed: {e}"))
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{CliAction, ReviewAction};
    use crate::review_cmd::run_review;
    use std::sync::Mutex;

    /// Process-wide lock serializing the CWD-mutating catalog test (set_current_dir is
    /// process-global, like env vars). The other in-module tests do not touch the CWD.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    /// R3/R7 reachability proof: a real entrypoint reaches `run_catalog` and leaves
    /// off-agent SSB rows; the covered mutation is deletion of the
    /// `ReviewAction::Catalog` arm in `run_review`.
    #[cfg(unix)]
    #[test]
    fn run_review_catalog_arm_emits_catalog_built_rows() {
        use elai_orchestrator::ssb::{ReviewGateCatalogEventKind, SsbReader};

        let _cwd = CWD_LOCK.lock().expect("cwd lock");
        let dir = tempfile::TempDir::new().expect("temporary project root");
        let root = dir.path();
        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(root).expect("set cwd");

        let missing_version = "9.9.9";
        let shell_version = "0.0.1";
        let fixture_path = root.join("j23-catalog.toml");
        std::fs::write(
            &fixture_path,
            format!(
                r#"[[tool]]
language = "python"
tool = "elai-nonexistent-tool-j23"
version = "{}"
invoke_cmd = ["elai-nonexistent-tool-j23", "--check"]
parser_fn = "json"
license = "MIT"

[[tool]]
language = "python"
tool = "posix-shell"
version = "{}"
invoke_cmd = ["sh", "-c", "true"]
parser_fn = "exit_code"
license = "MIT"
"#,
                missing_version, shell_version
            ),
        )
        .expect("write catalog fixture");

        let action = ReviewAction::Catalog {
            language: Some("python".to_string()),
            catalog_path: Some(fixture_path.clone()),
        };
        run_review(action, CliOutputFormat::Text).expect("the catalog arm must run");

        let cfg = SsbConfig {
            db_path: sessions_dir_path(root).join("ssb.db"),
            pool_size: None,
            project_id: elai_core::project_id_from_path(root),
            session_id: CATALOG_SESSION_ID.to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build Tokio runtime for SSB read-back");
        let rows = rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock).expect("open emitted SSB");
            ssb.query_by_session(CATALOG_SESSION_ID).expect("read emitted rows")
        });

        let payloads: Vec<_> = rows
            .iter()
            .map(|row| match rmp_serde::from_slice::<SsbPayload>(&row.payload).expect("decode row") {
                SsbPayload::ReviewGateCatalogEvent(payload) => payload,
                other => panic!("expected ReviewGateCatalogEvent payload, got {other:?}"),
            })
            .collect();
        std::env::set_current_dir(&prev_cwd).expect("restore cwd");

        use ReviewGateCatalogEventKind::CatalogBuilt;
        let built: Vec<_> = payloads.iter().filter(|p| p.kind == CatalogBuilt).collect();
        assert_eq!(built.len(), 3, "catalog dispatcher emits two tool rows and one aggregate row");
        let aggregates: Vec<_> = built.iter().filter(|p| p.tool.is_none()).collect();
        assert_eq!(aggregates.len(), 1, "catalog emits exactly one aggregate row");
        let aggregate = aggregates[0];
        assert_eq!(aggregate.eligible_tool_count, Some(1), "aggregate counts the eligible shell tool");
        assert_eq!(aggregate.missing_tool_count, Some(1), "aggregate counts the missing tool");
        assert_eq!(aggregate.verdict.as_deref(), Some("catalog_built"), "aggregate records catalog_built");
        assert_eq!(aggregate.refusal_reason, None, "aggregate has no refusal reason");
        assert_eq!(aggregate.language.as_deref(), Some("python"), "aggregate preserves requested language");

        let missing = built.iter().find(|p| p.tool.as_deref() == Some("elai-nonexistent-tool-j23"))
            .expect("missing-tool row is emitted");
        assert_eq!(missing.verdict.as_deref(), Some("tool_missing"), "missing tool records tool_missing verdict");
        assert_eq!(missing.refusal_reason.as_deref(), Some("tool_missing"), "missing tool records refusal reason");
        assert_eq!(missing.tool_version.as_deref(), Some(missing_version), "missing tool preserves fixture version");
        assert_eq!(missing.parser_fn.as_deref(), Some("json"), "missing tool preserves parser function");
        assert!(!missing.sarif, "missing tool row is not SARIF");

        let shell = built.iter().find(|p| p.tool.as_deref() == Some("posix-shell"))
            .expect("shell row is emitted");
        assert_eq!(shell.verdict.as_deref(), Some("eligible"), "installed shell records eligible verdict");
        assert_eq!(shell.refusal_reason, None, "installed shell has no refusal reason");
        assert_eq!(shell.tool_version.as_deref(), Some(shell_version), "shell preserves fixture version");
        assert_eq!(shell.parser_fn.as_deref(), Some("exit_code"), "shell preserves parser function");
        assert!(built.iter().all(|p| p.kind == ReviewGateCatalogEventKind::CatalogBuilt));
    }


    #[cfg(unix)]
    fn catalog_payloads(root: &Path) -> Vec<elai_orchestrator::ssb::ReviewGateCatalogEventPayload> {
        use elai_orchestrator::ssb::SsbReader;

        let cfg = SsbConfig {
            db_path: sessions_dir_path(root).join("ssb.db"),
            pool_size: None,
            project_id: elai_core::project_id_from_path(root),
            session_id: CATALOG_SESSION_ID.to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build Tokio runtime for SSB read-back");
        let rows = rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock).expect("open emitted SSB");
            ssb.query_by_session(CATALOG_SESSION_ID).expect("read emitted rows")
        });
        rows.iter()
            .map(|row| match rmp_serde::from_slice::<SsbPayload>(&row.payload).expect("decode row") {
                SsbPayload::ReviewGateCatalogEvent(payload) => payload,
                other => panic!("expected ReviewGateCatalogEvent payload, got {other:?}"),
            })
            .collect()
    }

    #[cfg(unix)]
    #[test]
    fn run_review_catalog_arm_emits_gate_invoked_and_clean_verdict() {
        let _cwd = CWD_LOCK.lock().expect("cwd lock");
        let dir = tempfile::TempDir::new().expect("temporary project root");
        let root = dir.path();
        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(root).expect("set cwd");
        let fixture_path = root.join("j23-catalog.toml");
        std::fs::write(&fixture_path, r#"[[tool]]
language = "python"
tool = "posix-shell-clean"
version = "0.0.1"
invoke_cmd = ["sh", "-c", "true"]
parser_fn = "exit_code"
license = "MIT"
"#).expect("write catalog fixture");

        run_review(ReviewAction::Catalog {
            language: Some("python".to_string()),
            catalog_path: Some(fixture_path),
        }, CliOutputFormat::Text).expect("the catalog arm must run");
        let payloads = catalog_payloads(root);
        std::env::set_current_dir(&prev_cwd).expect("restore cwd");

        let invoked: Vec<_> = payloads.iter().filter(|p| p.kind == elai_orchestrator::ssb::ReviewGateCatalogEventKind::GateInvoked).collect();
        assert_eq!(invoked.len(), 1, "one eligible tool emits one GateInvoked row");
        let invocation = invoked[0];
        assert_eq!(invocation.tool.as_deref(), Some("posix-shell-clean"));
        assert_eq!(invocation.finding_count, Some(0));
        assert_eq!(invocation.verdict.as_deref(), Some("invoked"));
        assert_eq!(invocation.refusal_reason, None);
        assert_eq!(invocation.parser_fn.as_deref(), Some("exit_code"));
        assert_eq!(invocation.tool_version.as_deref(), Some("0.0.1"));

        let verdicts: Vec<_> = payloads.iter().filter(|p| p.kind == elai_orchestrator::ssb::ReviewGateCatalogEventKind::VerdictEmitted).collect();
        assert_eq!(verdicts.len(), 1, "a clean catalog run emits one verdict row");
        let verdict = verdicts[0];
        assert_eq!(verdict.verdict.as_deref(), Some("clean"), "a clean run must emit a row and must not turn a passing tool into a phantom finding; silence would make no findings identical to the gate never running");
        assert_eq!(verdict.finding_count, Some(0));
        assert_eq!(verdict.tool, None);
        assert_eq!(verdict.eligible_tool_count, Some(1));
        assert_eq!(verdict.missing_tool_count, Some(0));
    }

    #[cfg(unix)]
    #[test]
    fn run_review_catalog_arm_emits_warn_verdict_for_json_findings() {
        let _cwd = CWD_LOCK.lock().expect("cwd lock");
        let dir = tempfile::TempDir::new().expect("temporary project root");
        let root = dir.path();
        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(root).expect("set cwd");
        let fixture_path = root.join("j23-catalog.toml");
        std::fs::write(&fixture_path, r#"[[tool]]
language = "python"
tool = "posix-shell-json"
version = "0.0.2"
invoke_cmd = ["sh", "-c", "printf '[{\"id\":1},{\"id\":2}]'"]
parser_fn = "json"
license = "MIT"
"#).expect("write catalog fixture");

        run_review(ReviewAction::Catalog {
            language: Some("python".to_string()),
            catalog_path: Some(fixture_path),
        }, CliOutputFormat::Text).expect("the catalog arm must run");
        let payloads = catalog_payloads(root);
        std::env::set_current_dir(&prev_cwd).expect("restore cwd");

        let invoked: Vec<_> = payloads.iter().filter(|p| p.kind == elai_orchestrator::ssb::ReviewGateCatalogEventKind::GateInvoked).collect();
        assert_eq!(invoked.len(), 1, "one eligible tool emits one GateInvoked row");
        assert_eq!(invoked[0].finding_count, Some(2));
        assert_eq!(invoked[0].parser_fn.as_deref(), Some("json"));
        let verdicts: Vec<_> = payloads.iter().filter(|p| p.kind == elai_orchestrator::ssb::ReviewGateCatalogEventKind::VerdictEmitted).collect();
        assert_eq!(verdicts.len(), 1, "a findings catalog run emits one verdict row");
        assert_eq!(verdicts[0].verdict.as_deref(), Some("warn"));
        assert_eq!(verdicts[0].finding_count, Some(2));
    }

    #[cfg(unix)]
    #[test]
    fn run_review_catalog_arm_ingests_sarif_and_warns() {
        use elai_orchestrator::ssb::ReviewGateCatalogEventKind;

        let _cwd = CWD_LOCK.lock().expect("cwd lock");
        let dir = tempfile::TempDir::new().expect("temporary project root");
        let root = dir.path();
        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(root).expect("set cwd");

        let sarif_path = root.join("fixture.sarif");
        std::fs::write(
            &sarif_path,
            r#"{
  "version": "2.1.0",
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "runs": [{
    "tool": {"driver": {"name": "semgrep"}},
    "results": [
      {
        "ruleId": "python.lang.security.audit",
        "level": "warning",
        "message": {"text": "unsafe operation"},
        "locations": [{"physicalLocation": {"artifactLocation": {"uri": "app.py"}, "region": {"startLine": 7}}}]
      },
      {"message": {"text": "another finding"}}
    ]
  }]
}"#,
        )
        .expect("write SARIF fixture");

        let catalog_path = root.join("j23-sarif-catalog.toml");
        std::fs::write(
            &catalog_path,
            format!(
                r#"[[tool]]
language = "python"
tool = "sarif-fixture"
version = "0.0.3"
invoke_cmd = ["cat", "{}"]
parser_fn = "exit_code"
sarif = true
license = "MIT"
"#,
                sarif_path.display()
            ),
        )
        .expect("write SARIF catalog fixture");

        let action = ReviewAction::Catalog {
            language: Some("python".to_string()),
            catalog_path: Some(catalog_path.clone()),
        };
        run_review(action, CliOutputFormat::Text).expect("the catalog arm must run");
        let payloads = catalog_payloads(root);
        std::env::set_current_dir(&prev_cwd).expect("restore cwd");

        let sarif_rows: Vec<_> = payloads
            .iter()
            .filter(|p| p.kind == ReviewGateCatalogEventKind::SarifIngested)
            .collect();
        assert_eq!(sarif_rows.len(), 1, "a SARIF row proves the production parser is not dark");
        let sarif = sarif_rows[0];
        assert_eq!(sarif.finding_count, Some(2), "two findings prove production parsed SARIF rather than the zero-result exit-code parser");
        assert!(sarif.sarif, "the ingested row must identify the SARIF path");
        assert_eq!(sarif.tool.as_deref(), Some("sarif-fixture"));
        assert_eq!(sarif.tool_version.as_deref(), Some("0.0.3"));
        assert_eq!(sarif.verdict.as_deref(), Some("sarif_ingested"));

        let invoked: Vec<_> = payloads
            .iter()
            .filter(|p| p.kind == ReviewGateCatalogEventKind::GateInvoked)
            .collect();
        assert_eq!(invoked.len(), 1, "one eligible SARIF tool emits one GateInvoked row");
        assert_eq!(invoked[0].finding_count, Some(2), "the gate receives SARIF's two findings instead of the exit-code parser's zero");
        assert_eq!(invoked[0].verdict.as_deref(), Some("invoked"));

        let verdicts: Vec<_> = payloads
            .iter()
            .filter(|p| p.kind == ReviewGateCatalogEventKind::VerdictEmitted)
            .collect();
        assert_eq!(verdicts.len(), 1, "the SARIF run emits exactly one final verdict row");
        assert_eq!(verdicts[0].verdict.as_deref(), Some("warn"), "two SARIF findings make the final verdict warn");
        assert_eq!(verdicts[0].finding_count, Some(2), "the final verdict preserves the two SARIF findings");
    }

    #[test]
    fn parse_review_rejects_both_and_neither_sub_actions() {
        let both = crate::args::parse_review_subcommand_args(
            &["--cockpit".to_string(), "--catalog".to_string()],
            CliOutputFormat::Text,
        );
        let err = both.expect_err("parser rejects both review sub-actions");
        assert!(err.contains("--cockpit") && err.contains("--catalog"), "both-sub-actions error names both flags");

        let neither = crate::args::parse_review_subcommand_args(&[], CliOutputFormat::Text);
        assert!(neither.is_err(), "parser rejects neither review sub-action");

        let parsed = crate::args::parse_review_subcommand_args(
            &["--catalog".to_string(), "--lang".to_string(), "python".to_string()],
            CliOutputFormat::Text,
        ).expect("parser accepts catalog language");
        let CliAction::Review { action: ReviewAction::Catalog { language: Some(l), catalog_path: None }, .. } = parsed else {
            panic!("parser must produce catalog action with python language and no path");
        };
        assert_eq!(l, "python", "catalog parser preserves language");
    }

    #[test]
    fn parse_review_catalog_rejects_cockpit_only_flags() {
        let result = crate::args::parse_review_subcommand_args(
            &["--catalog".to_string(), "--session".to_string(), "s1".to_string()],
            CliOutputFormat::Text,
        );
        let err = result.expect_err("catalog rejects cockpit-only session flag");
        assert!(err.contains("--session"), "catalog rejection names --session");
    }

}
