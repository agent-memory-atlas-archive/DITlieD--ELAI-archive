use std::path::Path;

use elai_orchestrator::framework_profile;
use elai_orchestrator::idea_framing::ProjectClass;
use elai_orchestrator::production_checklist;

use crate::args::CliOutputFormat;

pub(crate) fn run_production_checklist_cli(
    project_root: &Path,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let profile = framework_profile::get_framework_profile(project_root, ProjectClass::Other);
    let checks = profile.security_checks.clone();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("build async runtime: {e}"))?;

    let results = rt.block_on(async {
        production_checklist::run_all(project_root, &checks).await
    });

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&results)
                .map_err(|e| anyhow::anyhow!("serialize results to json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text => {
            for (check_id, result) in results {
                match result {
                    production_checklist::RunResult::ToolchainMissing { tool, install_hint } => {
                        println!("{check_id:?}: ToolchainMissing (tool={tool}, hint={install_hint})");
                    }
                    production_checklist::RunResult::Ok { findings } => {
                        println!("{check_id:?}: Ok ({} findings)", findings.len());
                    }
                    production_checklist::RunResult::Error { details, stderr_excerpt } => {
                        println!("{check_id:?}: Error ({details})");
                        if !stderr_excerpt.is_empty() {
                            println!("  stderr: {stderr_excerpt}");
                        }
                    }
                }
            }
        }
        CliOutputFormat::Toml => {
            let toml = toml::to_string(&results)
                .map_err(|e| anyhow::anyhow!("serialize results to toml: {e}"))?;
            println!("{toml}");
        }
    }

    Ok(())
}
