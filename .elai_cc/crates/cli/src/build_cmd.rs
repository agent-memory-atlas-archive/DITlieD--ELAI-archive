use std::path::Path;

use anyhow::anyhow;
use elai_orchestrator::framework_profile::get_framework_profile;
use elai_orchestrator::{classify_project_class, ProjectClass};
use serde::Serialize;

use crate::args::CliOutputFormat;

#[derive(Debug, Serialize)]
struct BuildPlan {
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

pub(crate) fn run(idea: &str, plan_only: bool, output_format: CliOutputFormat) -> anyhow::Result<()> {
    if !plan_only {
        eprintln!("elai build cannot generate yet: the app arm and its RRA receipt land together (FS-3).");
        eprintln!("run: elai build \"{idea}\" --plan-only");
        return Err(anyhow!("build generation is unavailable"));
    }

    let class = classify_project_class(idea);
    let project_root = std::env::current_dir()?;
    let profile = get_framework_profile(Path::new(&project_root), class);
    let plan = BuildPlan {
        idea: idea.to_string(),
        project_class: class.as_snake_case().to_string(),
        profile: profile.name,
        version_pin: profile.version_pin,
        language: profile.language,
        package_manager: profile.package_manager,
        test_runner: profile.test_runner,
        plan_only: true,
    };

    match output_format {
        CliOutputFormat::Json => println!("{}", serde_json::to_string(&plan)?),
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("idea: {}", plan.idea);
            println!("class: {}", plan.project_class);
            println!("profile: {}", plan.profile);
            println!("version_pin: {}", plan.version_pin);
            println!("language: {}", plan.language);
            println!("package_manager: {}", plan.package_manager);
            println!("test_runner: {}", plan.test_runner);
            println!("not built: --plan-only resolves the build, it does not generate an app");
        }
    }
    Ok(())
}
