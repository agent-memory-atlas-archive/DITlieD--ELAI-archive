//! E-12 Tier-B — Sentry presence check runner.
//!
//! Companion to `otel_check_runner` — verifies the server-entry
//! initialises Sentry. Same wire-dark posture: native byte-scan
//! today, `ast-grep` integration is RUNPOD-PENDING.
//!
//! TOOLCHAIN-PENDING-AST_GREP: shared with otel_check_runner;
//!   installing ast-grep enables both runners' structural matching.

use std::path::Path;

use super::{ChecklistFinding, RunResult};

const SERVER_ENTRY_NAMES: &[&str] = &[
    "main.ts",
    "main.js",
    "server.ts",
    "server.js",
    "index.ts",
    "index.js",
    "app.py",
    "main.py",
    "main.rs",
];

const SENTRY_MARKERS: &[&str] = &[
    "Sentry.init",
    "sentry-sdk",
    "@sentry/",
    "sentry_sdk.init",
    "sentry::init",
];

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    let server_files = find_server_entries(target);
    if server_files.is_empty() {
        return RunResult::Ok {
            findings: vec![ChecklistFinding {
                severity: "info".into(),
                category: "sentry_check".into(),
                message: "no server-entry file found under target".into(),
                file: None,
                line: None,
            }],
        };
    }
    for path in server_files {
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut has_sentry = false;
        for marker in SENTRY_MARKERS {
            if contents.contains(marker) {
                has_sentry = true;
                break;
            }
        }
        if !has_sentry {
            findings.push(ChecklistFinding {
                severity: "warn".into(),
                category: "sentry_check".into(),
                message: format!(
                    "server entry `{}` has no Sentry init markers",
                    path.display()
                ),
                file: Some(path.display().to_string()),
                line: None,
            });
        }
    }
    RunResult::Ok { findings }
}

fn find_server_entries(target: &Path) -> Vec<std::path::PathBuf> {
    let mut out: Vec<std::path::PathBuf> = Vec::new();
    let entries = match std::fs::read_dir(target) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned();
        if SERVER_ENTRY_NAMES.contains(&name.as_str()) {
            out.push(path);
        }
    }
    for sub in &["src", "server", "app"] {
        let sub_path = target.join(sub);
        if !sub_path.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&sub_path) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_owned();
            if SERVER_ENTRY_NAMES.contains(&name.as_str()) {
                out.push(p);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_sentry_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
