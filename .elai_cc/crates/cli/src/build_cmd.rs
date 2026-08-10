use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::redaction::redact_payload;
use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbPayload};
use elai_orchestrator::framework_profile::get_framework_profile;
use elai_orchestrator::{apply_template, classify_project_class, default_build_vars, ProjectClass, RunArtifactEmittedPayload};
use serde::Serialize;

use crate::args::CliOutputFormat;

const BUILD_PROJECT_ID: &str = "elai-build";
const BUILD_SESSION_ID: &str = "elai-build-oneshot";

#[derive(Debug, Serialize)]
pub(crate) struct BuildPlan {
    idea: String,
    #[serde(rename = "class")]
    project_class: String,
    profile: String,
    version_pin: String,
    language: String,
    package_manager: String,
    test_runner: String,
    plan_only: bool,
}

#[derive(Debug, Serialize)]
struct BuildResult {
    idea: String,
    #[serde(rename = "class")]
    project_class: String,
    profile: String,
    template: String,
    app_dir: String,
    receipt_path: String,
    plan_only: bool,
}

pub(crate) fn resolve_build_plan(idea: &str) -> anyhow::Result<BuildPlan> {
    let class = classify_project_class(idea);
    let project_root = std::env::current_dir()?;
    let profile = get_framework_profile(Path::new(&project_root), class);
    Ok(BuildPlan {
        idea: idea.to_string(),
        project_class: class.as_snake_case().to_string(),
        profile: profile.name,
        version_pin: profile.version_pin,
        language: profile.language,
        package_manager: profile.package_manager,
        test_runner: profile.test_runner,
        plan_only: true,
    })
}

pub(crate) fn run(idea: &str, plan_only: bool, output_dir: Option<&Path>, template: Option<&str>, output_format: CliOutputFormat) -> anyhow::Result<()> {
    let class = classify_project_class(idea);
    let project_root = std::env::current_dir()?;
    let profile = get_framework_profile(Path::new(&project_root), class);
    if plan_only {
        let plan = resolve_build_plan(idea)?;
        match output_format {
            CliOutputFormat::Json => println!("{}", serde_json::to_string(&plan)?),
            CliOutputFormat::Text | CliOutputFormat::Toml => {
                println!("idea: {}", plan.idea); println!("class: {}", plan.project_class); println!("profile: {}", plan.profile); println!("version_pin: {}", plan.version_pin); println!("language: {}", plan.language); println!("package_manager: {}", plan.package_manager); println!("test_runner: {}", plan.test_runner); println!("not built: --plan-only resolves the build, it does not generate an app");
            }
        }
        return Ok(());
    }
    let app_dir = output_dir.ok_or_else(|| anyhow!("generating `build` requires --out <dir>"))?;
    if std::env::var("ELAI_WRAP_RRA_DISABLE").unwrap_or_default() == "1" {
        eprintln!("build refused: ELAI_WRAP_RRA_DISABLE disables the receipt; nothing was generated (FS-3)");
        return Err(anyhow!("build receipt disabled"));
    }
    let template_name = template.unwrap_or("vanilla");
    let vars = default_build_vars(idea, None, None, None);
    if let Err(error) = apply_template(template_name, app_dir, &vars) { return Err(anyhow!("{error}")); }
    let receipt_path = app_dir.with_extension("rra");
    if let Err(error) = emit_build_rra(&project_root, &receipt_path) {
        let _ = std::fs::remove_dir_all(app_dir);
        return Err(anyhow!("receipt failed; generated app removed to preserve FS-3: {error}"));
    }
    let result = BuildResult { idea: idea.to_string(), project_class: class.as_snake_case().to_string(), profile: profile.name, template: template_name.to_string(), app_dir: app_dir.display().to_string(), receipt_path: receipt_path.display().to_string(), plan_only: false };
    match output_format {
        CliOutputFormat::Json => println!("{}", serde_json::to_string(&result)?),
        CliOutputFormat::Text | CliOutputFormat::Toml => { println!("app dir: {}", result.app_dir); println!("receipt: {}", result.receipt_path); println!("verify: elai rra verify {}", result.receipt_path); }
    }
    Ok(())
}

fn emit_build_rra(project_root: &Path, rra_path: &Path) -> anyhow::Result<()> {
    let sessions_dir = sessions_dir_path(project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let cfg = SsbConfig { db_path: sessions_dir.join("ssb.db"), pool_size: None, project_id: BUILD_PROJECT_ID.to_string(), session_id: BUILD_SESSION_ID.to_string() };
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().worker_threads(2).build()?;
    runtime.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock)?);
        use elai_orchestrator::ssb::SsbReader as _;
        let stream = ssb.query_session_stream_asc(BUILD_SESSION_ID)?;
        let pins = DeterminismConfig::from_env()?;
        let artifact = build_rra(BUILD_SESSION_ID, &stream, &pins);
        write_rra(&artifact, rra_path)?;
        let mut beacon = SsbPayload::RunArtifactEmitted(RunArtifactEmittedPayload { root_blake3: artifact.root_blake3.clone(), guarantee_level: artifact.guarantee_level.as_sql_str().to_owned(), gate_count: u32::try_from(artifact.manifest.len()).unwrap_or(u32::MAX), seed: artifact.seed, session_id: BUILD_SESSION_ID.to_owned(), objective_hash: artifact.objective_hash.clone() });
        let _ = redact_payload(&mut beacon);
        ssb.write_payload("cli:build-rra", 0, beacon, Vec::new()).await?;
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
