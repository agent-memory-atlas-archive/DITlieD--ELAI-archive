use std::env;
use std::path::Path;
use std::process::Command;

use elai_orchestrator::{
    ConfigLoader, ProjectContext, load_oauth_credentials, resolve_sandbox_status, RuntimeConfig,
};
use elai_providers::{oauth_token_is_expired, OAuthTokenSet};

use crate::{
    dep_audit, CliOutputFormat, StatusContext, DEFAULT_DATE,
    parse_git_status_metadata, parse_git_workspace_summary,
    VERSION, BUILD_TARGET, GIT_SHA,
};

use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagnosticLevel {
    Ok,
    Warn,
    Fail,
}

impl DiagnosticLevel {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }

    pub(crate) fn is_failure(self) -> bool {
        matches!(self, Self::Fail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticCheck {
    pub(crate) name: &'static str,
    pub(crate) level: DiagnosticLevel,
    pub(crate) summary: String,
    pub(crate) details: Vec<String>,
}

impl DiagnosticCheck {
    pub(crate) fn new(name: &'static str, level: DiagnosticLevel, summary: impl Into<String>) -> Self {
        Self {
            name,
            level,
            summary: summary.into(),
            details: Vec::new(),
        }
    }

    pub(crate) fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DoctorReport {
    pub(crate) checks: Vec<DiagnosticCheck>,
}

impl DoctorReport {
    pub(crate) fn has_failures(&self) -> bool {
        self.checks.iter().any(|check| check.level.is_failure())
    }

    pub(crate) fn render(&self) -> String {
        let ok_count = self
            .checks
            .iter()
            .filter(|check| check.level == DiagnosticLevel::Ok)
            .count();
        let warn_count = self
            .checks
            .iter()
            .filter(|check| check.level == DiagnosticLevel::Warn)
            .count();
        let fail_count = self
            .checks
            .iter()
            .filter(|check| check.level == DiagnosticLevel::Fail)
            .count();
        let mut lines = vec![
            "Doctor".to_string(),
            format!(
                "Summary\n  OK               {ok_count}\n  Warnings         {warn_count}\n  Failures         {fail_count}"
            ),
        ];
        lines.extend(self.checks.iter().map(render_diagnostic_check));
        lines.join("\n\n")
    }
}

fn render_diagnostic_check(check: &DiagnosticCheck) -> String {
    let mut lines = vec![format!(
        "{}\n  Status           {}\n  Summary          {}",
        check.name,
        check.level.label(),
        check.summary
    )];
    if !check.details.is_empty() {
        lines.push("  Details".to_string());
        lines.extend(check.details.iter().map(|detail| format!("    - {detail}")));
    }
    lines.join("\n")
}

pub(crate) fn render_doctor_report() -> Result<DoctorReport, anyhow::Error> {
    let cwd = env::current_dir()?;
    let config_loader = ConfigLoader::default_for(&cwd);
    let config = config_loader.load();
    let discovered_config = config_loader.discover();
    let project_context = ProjectContext::discover_with_git(&cwd, DEFAULT_DATE)?;
    let (project_root, git_branch) =
        parse_git_status_metadata(project_context.git_status.as_deref());
    let git_summary = parse_git_workspace_summary(project_context.git_status.as_deref());
    let empty_config = elai_orchestrator::RuntimeConfig::empty();
    let sandbox_config = config.as_ref().ok().unwrap_or(&empty_config);
    let context = StatusContext {
        cwd: cwd.clone(),
        session_path: None,
        loaded_config_files: config
            .as_ref()
            .ok()
            .map_or(0, |runtime_config| runtime_config.loaded_entries().len()),
        discovered_config_files: discovered_config.len(),
        memory_file_count: project_context.instruction_files.len(),
        project_root,
        git_branch,
        git_summary,
        sandbox_status: resolve_sandbox_status(sandbox_config.sandbox(), &cwd),
    };
    #[cfg_attr(not(feature = "obscura"), allow(unused_mut))]
    let mut checks = vec![
        check_auth_health(),
        check_config_health(&config_loader, config.as_ref()),
        check_workspace_health(&context),
        check_sandbox_health(&context.sandbox_status),
        check_system_health(&cwd, config.as_ref().ok()),
    ];

    // A5.2: include obscura binary check when the feature is compiled in.
    #[cfg(feature = "obscura")]
    checks.push(check_obscura_health());

    Ok(DoctorReport { checks })
}

pub(crate) fn run_dep_audit_command(
    project_dir: &Path,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let clock = elai_core::OsClock;
    let rules_db_path = project_dir.join("rules.db");
    let result = dep_audit::run_dep_audit(project_dir, &rules_db_path, &clock)?;

    // Trigger steering vector recomputation when deprecated_patterns changed.
    // total_outdated > 0 means at least one pattern was upserted this run.
    if result.total_outdated > 0 {
        tracing::info!(
            count = result.total_outdated,
            "deprecated_patterns changed, triggering steering vector recomputation"
        );
        let script_path = project_dir.join("metrics").join("compute_steering_vector.py");
        let status = Command::new("python")
            .arg(&script_path)
            .arg("--batch")
            .current_dir(project_dir)
            .status();
        match status {
            Ok(s) if s.success() => {
                tracing::info!("steering vector recomputation succeeded");
            }
            Ok(s) => {
                tracing::warn!(
                    exit_code = ?s.code(),
                    "steering vector recomputation exited with non-zero status; continuing"
                );
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "steering vector recomputation failed to launch; continuing"
                );
            }
        }
    }

    match output_format {
        CliOutputFormat::Text => {
            println!("dep-audit: {} ecosystem(s) scanned, {} outdated",
                result.ecosystems_scanned.len(), result.total_outdated);
            for entry in &result.entries {
                println!("  [{severity}] {eco}/{name}: {cur} -> {lat}",
                    severity = entry.severity,
                    eco = entry.ecosystem,
                    name = entry.name,
                    cur = entry.current_version,
                    lat = entry.latest_version,
                );
            }
            if !result.errors.is_empty() {
                println!("  {} error(s) during audit", result.errors.len());
                for err in &result.errors {
                    println!("    - {err}");
                }
            }
        }
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "dep-audit",
                "total_outdated": result.total_outdated,
                "ecosystems_scanned": result.ecosystems_scanned,
                "entries": result.entries,
                "errors": result.errors,
                "timestamp": result.timestamp,
            }))?
        ),
        // C-21 Task 6 — `--output-format toml`. The TOML report is always
        // emitted to `metrics/dep_audit_<YYYY-MM-DD>.toml` inside
        // `run_dep_audit`. In Toml mode we additionally emit
        // `metrics/dep_audit_latest.toml` (copy of the dated file) so
        // self_mod_layer1a::trigger_dep_audit_background can locate the
        // most recent run by a stable path.
        CliOutputFormat::Toml => {
            let metrics_dir = project_dir.join("metrics");
            let date_part = &result.timestamp[..10];
            let dated_path = metrics_dir.join(format!("dep_audit_{date_part}.toml"));
            let latest_path = metrics_dir.join("dep_audit_latest.toml");
            // Windows-friendly copy (symlink would require elevated perms).
            if dated_path.exists() {
                if let Err(e) = std::fs::copy(&dated_path, &latest_path) {
                    tracing::warn!(
                        error = %e,
                        dated = %dated_path.display(),
                        latest = %latest_path.display(),
                        "failed to copy dated dep-audit report to latest path; continuing"
                    );
                } else {
                    tracing::info!(
                        path = %latest_path.display(),
                        "wrote dep_audit_latest.toml"
                    );
                }
            }
            println!(
                "dep-audit: {} ecosystem(s) scanned, {} outdated; report at {}",
                result.ecosystems_scanned.len(),
                result.total_outdated,
                latest_path.display()
            );
        }
    }
    Ok(())
}

pub(crate) fn run_doctor(output_format: CliOutputFormat) -> Result<(), anyhow::Error> {
    let report = render_doctor_report()?;
    let message = report.render();
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => println!("{message}"),
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "doctor",
                "message": message,
                "report": message,
                "has_failures": report.has_failures(),
            }))?
        ),
    }
    if report.has_failures() {
        return Err(anyhow::anyhow!("doctor found failing checks"));
    }
    Ok(())
}

pub(crate) fn check_auth_health() -> DiagnosticCheck {
    let api_key_present = env::var("ANTHROPIC_API_KEY")
        .ok()
        .is_some_and(|value| !value.trim().is_empty());
    let auth_token_present = env::var("ANTHROPIC_AUTH_TOKEN")
        .ok()
        .is_some_and(|value| !value.trim().is_empty());

    match load_oauth_credentials() {
        Ok(Some(token_set)) => {
            let expired = oauth_token_is_expired(&OAuthTokenSet {
                access_token: token_set.access_token.clone(),
                refresh_token: token_set.refresh_token.clone(),
                expires_at: token_set.expires_at,
                scopes: token_set.scopes.clone(),
            });
            let mut details = vec![
                format!(
                    "Environment       api_key={} auth_token={}",
                    if api_key_present { "present" } else { "absent" },
                    if auth_token_present {
                        "present"
                    } else {
                        "absent"
                    }
                ),
                format!(
                    "Saved OAuth       expires_at={} refresh_token={} scopes={}",
                    token_set
                        .expires_at
                        .map_or_else(|| "<none>".to_string(), |value| value.to_string()),
                    if token_set.refresh_token.is_some() {
                        "present"
                    } else {
                        "absent"
                    },
                    if token_set.scopes.is_empty() {
                        "<none>".to_string()
                    } else {
                        token_set.scopes.join(",")
                    }
                ),
            ];
            if expired {
                details.push(
                    "Suggested action  elai login to refresh local OAuth credentials".to_string(),
                );
            }
            DiagnosticCheck::new(
                "Auth",
                if expired {
                    DiagnosticLevel::Warn
                } else {
                    DiagnosticLevel::Ok
                },
                if expired {
                    "saved OAuth credentials are present but expired"
                } else if api_key_present || auth_token_present {
                    "environment and saved credentials are available"
                } else {
                    "saved OAuth credentials are available"
                },
            )
            .with_details(details)
        }
        Ok(None) => DiagnosticCheck::new(
            "Auth",
            if api_key_present || auth_token_present {
                DiagnosticLevel::Ok
            } else {
                DiagnosticLevel::Warn
            },
            if api_key_present || auth_token_present {
                "environment credentials are configured"
            } else {
                "no API key or saved OAuth credentials were found"
            },
        )
        .with_details(vec![format!(
            "Environment       api_key={} auth_token={}",
            if api_key_present { "present" } else { "absent" },
            if auth_token_present {
                "present"
            } else {
                "absent"
            }
        )]),
        Err(error) => DiagnosticCheck::new(
            "Auth",
            DiagnosticLevel::Fail,
            format!("failed to inspect saved credentials: {error}"),
        ),
    }
}

pub(crate) fn check_config_health(
    config_loader: &ConfigLoader,
    config: Result<&RuntimeConfig, &elai_orchestrator::ConfigError>,
) -> DiagnosticCheck {
    let discovered = config_loader.discover();
    let discovered_count = discovered.len();
    let discovered_paths = discovered
        .iter()
        .map(|entry| entry.path.display().to_string())
        .collect::<Vec<_>>();
    match config {
        Ok(runtime_config) => {
            let loaded_entries = runtime_config.loaded_entries();
            let mut details = vec![format!(
                "Config files      loaded {}/{}",
                loaded_entries.len(),
                discovered_count
            )];
            if let Some(model) = runtime_config.model() {
                details.push(format!("Resolved model    {model}"));
            }
            details.push(format!(
                "MCP servers       {}",
                runtime_config.mcp().servers().len()
            ));
            if discovered_paths.is_empty() {
                details.push("Discovered files  <none>".to_string());
            } else {
                details.extend(
                    discovered_paths
                        .into_iter()
                        .map(|path| format!("Discovered file   {path}")),
                );
            }
            DiagnosticCheck::new(
                "Config",
                if discovered_count == 0 {
                    DiagnosticLevel::Warn
                } else {
                    DiagnosticLevel::Ok
                },
                if discovered_count == 0 {
                    "no config files were found; defaults are active"
                } else {
                    "runtime config loaded successfully"
                },
            )
            .with_details(details)
        }
        Err(error) => DiagnosticCheck::new(
            "Config",
            DiagnosticLevel::Fail,
            format!("runtime config failed to load: {error}"),
        )
        .with_details(if discovered_paths.is_empty() {
            vec!["Discovered files  <none>".to_string()]
        } else {
            discovered_paths
                .into_iter()
                .map(|path| format!("Discovered file   {path}"))
                .collect()
        }),
    }
}

pub(crate) fn check_workspace_health(context: &StatusContext) -> DiagnosticCheck {
    let in_repo = context.project_root.is_some();
    DiagnosticCheck::new(
        "Workspace",
        if in_repo {
            DiagnosticLevel::Ok
        } else {
            DiagnosticLevel::Warn
        },
        if in_repo {
            format!(
                "project root detected on branch {}",
                context.git_branch.as_deref().unwrap_or("unknown")
            )
        } else {
            "current directory is not inside a git project".to_string()
        },
    )
    .with_details(vec![
        format!("Cwd              {}", context.cwd.display()),
        format!(
            "Project root     {}",
            context
                .project_root
                .as_ref()
                .map_or_else(|| "<none>".to_string(), |path| path.display().to_string())
        ),
        format!(
            "Git branch       {}",
            context.git_branch.as_deref().unwrap_or("unknown")
        ),
        format!("Git state        {}", context.git_summary.headline()),
        format!("Changed files    {}", context.git_summary.changed_files),
        format!(
            "Memory files     {} · config files loaded {}/{}",
            context.memory_file_count, context.loaded_config_files, context.discovered_config_files
        ),
    ])
}

pub(crate) fn check_sandbox_health(status: &elai_orchestrator::SandboxStatus) -> DiagnosticCheck {
    let degraded = status.enabled && !status.active;
    let mut details = vec![
        format!("Enabled          {}", status.enabled),
        format!("Active           {}", status.active),
        format!("Supported        {}", status.supported),
        format!("Filesystem mode  {}", status.filesystem_mode.as_str()),
        format!("Filesystem live  {}", status.filesystem_active),
    ];
    if let Some(reason) = &status.fallback_reason {
        details.push(format!("Fallback reason  {reason}"));
    }
    DiagnosticCheck::new(
        "Sandbox",
        if degraded {
            DiagnosticLevel::Warn
        } else {
            DiagnosticLevel::Ok
        },
        if degraded {
            "sandbox was requested but is not currently active"
        } else if status.active {
            "sandbox protections are active"
        } else {
            "sandbox is not active for this session"
        },
    )
    .with_details(details)
}

pub(crate) fn check_system_health(cwd: &Path, config: Option<&RuntimeConfig>) -> DiagnosticCheck {
    let mut details = vec![
        format!("OS               {} {}", env::consts::OS, env::consts::ARCH),
        format!("Working dir      {}", cwd.display()),
        format!("Version          {}", VERSION),
        format!("Build target     {}", BUILD_TARGET.unwrap_or("<unknown>")),
        format!("Git SHA          {}", GIT_SHA.unwrap_or("<unknown>")),
    ];
    if let Some(model) = config.and_then(RuntimeConfig::model) {
        details.push(format!("Default model    {model}"));
    }
    DiagnosticCheck::new(
        "System",
        DiagnosticLevel::Ok,
        "captured local runtime metadata",
    )
    .with_details(details)
}

// A5.2: obscura binary health check — only compiled when cfg(feature = "obscura").
#[cfg(feature = "obscura")]
pub(crate) fn check_obscura_health() -> DiagnosticCheck {
    let report = elai_web_search::obscura_fetcher::doctor();
    match (report.binary_path, report.version, report.error) {
        (Some(path), Some(version), None) => DiagnosticCheck::new(
            "Obscura",
            DiagnosticLevel::Ok,
            "obscura headless-browser binary found and executable",
        )
        .with_details(vec![
            format!("Binary           {}", path.display()),
            format!("Version          {version}"),
        ]),
        (Some(path), Some(version), Some(err)) => DiagnosticCheck::new(
            "Obscura",
            DiagnosticLevel::Warn,
            "obscura binary found but reported a non-fatal error",
        )
        .with_details(vec![
            format!("Binary           {}", path.display()),
            format!("Version          {version}"),
            format!("Error            {err}"),
        ]),
        (Some(path), None, Some(err)) => DiagnosticCheck::new(
            "Obscura",
            DiagnosticLevel::Warn,
            "obscura binary found but --version failed",
        )
        .with_details(vec![
            format!("Binary           {}", path.display()),
            format!("Error            {err}"),
        ]),
        (Some(path), None, None) => DiagnosticCheck::new(
            "Obscura",
            DiagnosticLevel::Warn,
            "obscura binary found but version could not be determined",
        )
        .with_details(vec![format!("Binary           {}", path.display())]),
        (None, _, Some(err)) => DiagnosticCheck::new(
            "Obscura",
            DiagnosticLevel::Fail,
            "obscura headless-browser binary not found",
        )
        .with_details(vec![
            format!("Error            {err}"),
            "Suggested action  install obscura to tools/obscura/obscura.exe or set OBSCURA_PATH".to_owned(),
        ]),
        (None, _, None) => DiagnosticCheck::new(
            "Obscura",
            DiagnosticLevel::Fail,
            "obscura binary not found and no error detail available",
        ),
    }
}
