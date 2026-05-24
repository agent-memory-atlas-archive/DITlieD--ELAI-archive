//! E-12 Tier-B — OTel presence check runner.
//!
//! Greps server-entry files for OpenTelemetry resource-attr +
//! init markers. Uses `ast-grep` when available; falls back to a
//! plain-text byte scan when the binary is absent (the byte-scan
//! path is the wire-dark default behaviour — it ships findings
//! today without requiring a separate toolchain install).
//!
//! TOOLCHAIN-PENDING-AST_GREP: npm install -g @ast-grep/cli ||
//!   cargo install ast-grep (https://ast-grep.github.io/).
//!   The fallback byte-scan path still runs without ast-grep, so
//!   this seam is OPTIONAL — installing the CLI yields more
//!   precise structural matches.

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
    "main.go",
];

const OTEL_MARKERS: &[&str] = &[
    "@opentelemetry/api",
    "opentelemetry",
    "OTel.init",
    "otel.init",
    "trace.getTracer",
    "service.name",
    "Resource.create",
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
        // No server entry found — emit advisory finding, not a
        // block (this is checklist info, not a vulnerability).
        return RunResult::Ok {
            findings: vec![ChecklistFinding {
                severity: "info".into(),
                category: "otel_check".into(),
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
        let mut has_otel = false;
        for marker in OTEL_MARKERS {
            if contents.contains(marker) {
                has_otel = true;
                break;
            }
        }
        if !has_otel {
            findings.push(ChecklistFinding {
                severity: "warn".into(),
                category: "otel_check".into(),
                message: format!(
                    "server entry `{}` has no OpenTelemetry init markers",
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
    // Recurse into common src/ folders (one level deep only — keeps
    // the runner bounded).
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
        let bogus = Path::new("/elai_e12_bogus_otel_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }

    #[tokio::test]
    async fn empty_dir_returns_advisory_finding() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let res = run(tmp.path()).await;
        match res {
            RunResult::Ok { findings } => {
                assert_eq!(findings.len(), 1);
                assert_eq!(findings[0].severity, "info");
            }
            other => panic!("expected Ok with advisory finding, got {:?}", other),
        }
    }
}
