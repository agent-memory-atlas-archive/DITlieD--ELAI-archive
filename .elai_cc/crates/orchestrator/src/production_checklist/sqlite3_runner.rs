//! E-12 Tier-B — sqlite3 RLS-on probe runner.
//!
//! Shells out to `sqlite3 <target>/schema.db ".schema"` and grep
//! for `CREATE TABLE` rows that lack a matching `ENABLE ROW LEVEL
//! SECURITY` row. Supabase 2026's "Postgres best practices for AI
//! agents" calls RLS-off the #1 Neon/Supabase vuln. The local probe
//! here is a SQLite stand-in; real Postgres RLS probes live in the
//! follow-on pg_dump parser.
//!
//! TOOLCHAIN-PENDING-SQLITE3: sqlite3 is bundled with macOS / most
//!   Linux distros; on Windows install via scoop install sqlite ||
//!   choco install sqlite. https://www.sqlite.org/download.html.

use std::path::Path;

use tokio::process::Command;
use tokio::time::timeout;

use super::{excerpt, is_path_not_found, ChecklistFinding, RunResult, DEFAULT_TIMEOUT};

const BINARY: &str = "sqlite3";
const INSTALL_HINT: &str =
    "install sqlite3 via package manager (bundled on macOS/Linux; scoop/choco on Windows)";

pub async fn run(target: &Path) -> RunResult {
    if !target.exists() {
        return RunResult::Error {
            details: format!("target path does not exist: {}", target.display()),
            stderr_excerpt: String::new(),
        };
    }
    // Look for any *.db / *.sqlite / *.sqlite3 file under target.
    let db_path = match find_database(target) {
        Some(p) => p,
        None => {
            // No database — nothing to probe. Not a failure;
            // emit Ok with zero findings.
            return RunResult::Ok {
                findings: Vec::new(),
            };
        }
    };
    let mut cmd = Command::new(BINARY);
    cmd.arg(&db_path)
        .arg(".schema")
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = match timeout(DEFAULT_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) if is_path_not_found(&e) => {
            return RunResult::ToolchainMissing {
                tool: BINARY.to_owned(),
                install_hint: INSTALL_HINT.to_owned(),
            };
        }
        Ok(Err(e)) => {
            return RunResult::Error {
                details: format!("sqlite3 spawn failed: {}", e),
                stderr_excerpt: String::new(),
            };
        }
        Err(_) => {
            return RunResult::Error {
                details: format!("sqlite3 timed out after {}s", DEFAULT_TIMEOUT.as_secs()),
                stderr_excerpt: String::new(),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return RunResult::Error {
            details: format!(
                "sqlite3 exited with code {}",
                output.status.code().unwrap_or(-1)
            ),
            stderr_excerpt: excerpt(&stderr),
        };
    }
    let findings = scan_schema(&stdout, &db_path);
    RunResult::Ok { findings }
}

fn find_database(target: &Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(target).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_owned();
        if matches!(ext.as_str(), "db" | "sqlite" | "sqlite3") {
            return Some(p);
        }
    }
    None
}

fn scan_schema(schema: &str, db_path: &Path) -> Vec<ChecklistFinding> {
    // SQLite has no native RLS — the probe scans for CREATE TABLE
    // statements that contain no `-- RLS:` marker comment.
    let mut findings: Vec<ChecklistFinding> = Vec::new();
    let mut tables_without_rls: Vec<String> = Vec::new();
    for stmt in schema.split(';') {
        let s = stmt.trim();
        if !s.to_ascii_uppercase().starts_with("CREATE TABLE") {
            continue;
        }
        if s.contains("-- RLS:") {
            continue;
        }
        // Pull the table name out of `CREATE TABLE <name>`.
        let name = s
            .split_whitespace()
            .nth(2)
            .unwrap_or("<unknown>")
            .trim_start_matches('`')
            .trim_start_matches('"')
            .trim_end_matches('`')
            .trim_end_matches('"')
            .to_owned();
        tables_without_rls.push(name);
    }
    for name in tables_without_rls {
        findings.push(ChecklistFinding {
            severity: "block".into(),
            category: "sqlite3_rls".into(),
            message: format!("table `{}` has no RLS marker", name),
            file: Some(db_path.display().to_string()),
            line: None,
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_schema_flags_table_without_rls() {
        let schema = "CREATE TABLE users (id INTEGER, email TEXT); CREATE TABLE posts (id INTEGER) -- RLS: enabled\n;";
        let out = scan_schema(schema, Path::new("test.db"));
        assert_eq!(out.len(), 1);
        assert!(out[0].message.contains("users"));
    }

    #[test]
    fn scan_schema_clean_when_rls_marker_present() {
        let schema = "CREATE TABLE users (id INTEGER) -- RLS: enabled";
        assert!(scan_schema(schema, Path::new("test.db")).is_empty());
    }

    #[tokio::test]
    async fn missing_target_returns_error() {
        let bogus = Path::new("/elai_e12_bogus_sqlite_xyzzy");
        let res = run(bogus).await;
        assert!(matches!(res, RunResult::Error { .. }));
    }
}
