// HostInstaller — Rust re-entry surface for `elai install`.
//
// The 8-step install state machine itself lives in scripts/install/install.{sh,ps1}
// (B-43 Phases 1-6). HostInstaller spawns the appropriate shell installer as a
// subprocess with forwarded flags. After the subprocess exits, it reads the
// per-step JSON state files at ~/.elai/install-state/<step>.json (Phase 3) and
// renders an InstallReport summarising which steps were done / skipped / failed.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{anyhow, Context};
use serde_json::Value;

use crate::CliOutputFormat;

pub(crate) const INSTALL_STEPS: &[&str] = &[
    "env_probe",
    "arch_gate",
    "fetch_binary",
    "stage_models",
    "bootstrap_searxng",
    "render_config",
    "doctor",
];

#[derive(Debug, Clone)]
pub(crate) struct InstallArgs {
    pub(crate) skip_models: bool,
    pub(crate) reset_config: bool,
    pub(crate) project_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallStepState {
    Done,
    Skipped,
    Failed,
    Pending,
    InProgress,
    Unknown,
}

impl InstallStepState {
    fn from_status_str(s: &str) -> Self {
        match s {
            "done" => Self::Done,
            "skipped" => Self::Skipped,
            "failed" => Self::Failed,
            "pending" => Self::Pending,
            "in_progress" => Self::InProgress,
            _ => Self::Unknown,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InstallStepStatus {
    pub(crate) name: String,
    pub(crate) state: InstallStepState,
}

#[derive(Debug, Clone)]
pub(crate) struct InstallReport {
    pub(crate) steps: Vec<InstallStepStatus>,
    pub(crate) script_path: PathBuf,
    pub(crate) subprocess_exit: i32,
}

impl InstallReport {
    pub(crate) fn render_text(&self) -> String {
        let mut lines = vec![
            "elai install".to_string(),
            format!("  script: {}", self.script_path.display()),
            format!("  subprocess exit: {}", self.subprocess_exit),
            "  step status:".to_string(),
        ];
        for step in &self.steps {
            lines.push(format!("    {:<20} {}", step.name, step.state.label()));
        }
        lines.join("\n")
    }
}

pub(crate) struct HostInstaller {
    install_dir: PathBuf,
    script_dir: PathBuf,
    args: InstallArgs,
}

impl HostInstaller {
    pub(crate) fn new(args: InstallArgs) -> Result<Self, anyhow::Error> {
        let install_dir = env::var_os("ELAI_INSTALL_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let base = env::var_os("HOME")
                    .or_else(|| env::var_os("USERPROFILE"))
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                base.join(".elai")
            });
        let script_dir = Self::resolve_script_dir(&install_dir)?;
        Ok(Self {
            install_dir,
            script_dir,
            args,
        })
    }

    fn resolve_script_dir(install_dir: &Path) -> Result<PathBuf, anyhow::Error> {
        let installed = install_dir.join("scripts").join("install");
        if installed.is_dir() {
            return Ok(installed);
        }
        if let Ok(cwd) = env::current_dir() {
            let dev = cwd.join("scripts").join("install");
            if dev.is_dir() {
                return Ok(dev);
            }
            let workflow_root_dev = cwd
                .ancestors()
                .find_map(|a| {
                    let candidate = a.join("scripts").join("install");
                    if candidate.is_dir() { Some(candidate) } else { None }
                });
            if let Some(found) = workflow_root_dev {
                return Ok(found);
            }
        }
        Err(anyhow!(
            "cannot locate scripts/install/ directory (looked in {} and CWD)",
            install_dir.display()
        ))
    }

    fn script_path(&self) -> PathBuf {
        if cfg!(windows) {
            self.script_dir.join("install.ps1")
        } else {
            self.script_dir.join("install.sh")
        }
    }

    fn forward_flags(&self) -> Vec<OsString> {
        let mut flags = Vec::new();
        if cfg!(windows) {
            if self.args.skip_models {
                flags.push(OsString::from("-SkipModels"));
            }
            if self.args.reset_config {
                flags.push(OsString::from("-ResetConfig"));
            }
        } else {
            if self.args.skip_models {
                flags.push(OsString::from("--skip-models"));
            }
            if self.args.reset_config {
                flags.push(OsString::from("--reset-config"));
            }
        }
        flags
    }

    fn spawn_installer(&self, script: &Path) -> Result<ExitStatus, anyhow::Error> {
        let flags = self.forward_flags();
        let status = if cfg!(windows) {
            Command::new("powershell")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-File")
                .arg(script)
                .args(&flags)
                .status()
                .with_context(|| {
                    format!("failed to spawn powershell for {}", script.display())
                })?
        } else {
            Command::new("bash")
                .arg(script)
                .args(&flags)
                .status()
                .with_context(|| format!("failed to spawn bash for {}", script.display()))?
        };
        Ok(status)
    }

    pub(crate) fn run(&self) -> Result<InstallReport, anyhow::Error> {
        let script = self.script_path();
        if !script.is_file() {
            return Err(anyhow!(
                "install script not found at {}",
                script.display()
            ));
        }
        let status = self.spawn_installer(&script)?;
        let exit = status.code().unwrap_or(-1);
        let steps = read_step_status(&self.install_dir);
        Ok(InstallReport {
            steps,
            script_path: script,
            subprocess_exit: exit,
        })
    }
}

fn read_step_status(install_dir: &Path) -> Vec<InstallStepStatus> {
    let state_dir = install_dir.join("install-state");
    INSTALL_STEPS
        .iter()
        .map(|name| {
            let state = read_step_state(&state_dir, name);
            InstallStepStatus {
                name: (*name).to_string(),
                state,
            }
        })
        .collect()
}

fn read_step_state(state_dir: &Path, step: &str) -> InstallStepState {
    let path = state_dir.join(format!("{step}.json"));
    let Ok(contents) = fs::read_to_string(&path) else {
        return InstallStepState::Pending;
    };
    let Ok(parsed): Result<Value, _> = serde_json::from_str(&contents) else {
        return InstallStepState::Unknown;
    };
    parsed
        .get("status")
        .and_then(Value::as_str)
        .map(InstallStepState::from_status_str)
        .unwrap_or(InstallStepState::Unknown)
}

pub(crate) fn run_install(
    args: InstallArgs,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let installer = HostInstaller::new(args)?;
    let report = installer.run()?;
    let text = report.render_text();
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => println!("{text}"),
        CliOutputFormat::Json => {
            let json = serde_json::json!({
                "kind": "install",
                "script": report.script_path.display().to_string(),
                "subprocess_exit": report.subprocess_exit,
                "steps": report
                    .steps
                    .iter()
                    .map(|s| serde_json::json!({"name": s.name, "state": s.state.label()}))
                    .collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }
    if report.subprocess_exit != 0 {
        return Err(anyhow!(
            "install script exited with code {}",
            report.subprocess_exit
        ));
    }
    let any_failed = report
        .steps
        .iter()
        .any(|s| s.state == InstallStepState::Failed);
    if any_failed {
        return Err(anyhow!(
            "one or more install steps failed; re-run after remediation"
        ));
    }
    Ok(())
}
