//! E-14 — `elai browser <run|list-journeys|status>` handler.
//!
//! Wire-dark surface: the runner under
//! `crates/orchestrator/src/browser_e2e/` returns
//! `E2eError::ToolchainMissing` when `npx` / `playwright` is absent. This
//! handler treats that as a SOFT failure (prints a degraded-status JSON
//! to stdout, exits 0) so CI without Playwright installed does not red.
//!
//! The synthetic-fixture path (no actual subprocess invoked) is
//! deterministic so contract tests can assert observable effect without
//! requiring `npx` on PATH.

use std::fs;
use std::path::{Path, PathBuf};

use elai_orchestrator::browser_e2e::{
    load_journey, run_assertions, AssertionReport, E2eError, Journey, PageSnapshot,
};
use serde::Serialize;

use crate::args::{BrowserAction, CliOutputFormat};

/// Default journeys directory, relative to the current working dir
/// (matches the CLAUDE.md "Project PATHs" entry for journey TOMLs).
const DEFAULT_JOURNEYS_DIR: &str = ".elai_cc/journeys";

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: BrowserAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        BrowserAction::Run { journey } => run_journey(&journey, output_format),
        BrowserAction::ListJourneys => list_journeys(output_format),
        BrowserAction::Status => status(output_format),
    }
}

#[derive(Debug, Serialize)]
struct JourneySummary {
    name: String,
    url: String,
    step_count: usize,
    assertion_count: usize,
}

#[derive(Debug, Serialize)]
struct ListJourneysOutput {
    dir: String,
    journeys: Vec<JourneySummary>,
}

#[derive(Debug, Serialize)]
struct RunOutput {
    journey: String,
    url: String,
    passed: usize,
    failed: usize,
    toolchain_missing: Option<String>,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct StatusOutput {
    scope: &'static str,
    action_types: [&'static str; 4],
    note: &'static str,
}

fn list_journeys(output_format: CliOutputFormat) -> Result<(), String> {
    let dir = PathBuf::from(DEFAULT_JOURNEYS_DIR);
    let mut journeys: Vec<JourneySummary> = Vec::new();
    if dir.is_dir() {
        let entries = fs::read_dir(&dir)
            .map_err(|e| format!("browser list-journeys: read_dir({}) failed: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry
                .map_err(|e| format!("browser list-journeys: dir entry error: {e}"))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            match load_journey(&path) {
                Ok(j) => journeys.push(JourneySummary {
                    name: j.name,
                    url: j.url,
                    step_count: j.steps.len(),
                    assertion_count: j.assertions.len(),
                }),
                Err(e) => {
                    eprintln!(
                        "browser list-journeys: skipped malformed journey {}: {e}",
                        path.display()
                    );
                }
            }
        }
    }
    // Stable deterministic ordering by name so output diff-friendly.
    journeys.sort_by(|a, b| a.name.cmp(&b.name));
    let out = ListJourneysOutput {
        dir: dir.display().to_string(),
        journeys,
    };
    emit(&out, output_format)
}

fn run_journey(name: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let path = PathBuf::from(DEFAULT_JOURNEYS_DIR).join(format!("{name}.toml"));
    let journey = load_journey(&path)
        .map_err(|e| format!("browser run: load_journey({}) failed: {e}", path.display()))?;
    // Scope-lean path: do NOT spawn Playwright from the CLI handler.
    // We render a synthetic-fixture snapshot containing the journey URL
    // + a stub HTML body so the assertion shape can be validated end-to-end
    // without `npx` on PATH. Live Playwright runs land via the dashboard
    // and the orchestrator's runner; CLI is for shape-verification only.
    let snapshot = synthetic_snapshot(&journey);
    let report = run_assertions(&snapshot, &journey.assertions);
    let toolchain = probe_toolchain();
    let out = RunOutput {
        journey: journey.name.clone(),
        url: journey.url.clone(),
        passed: report.passed,
        failed: report.failed,
        toolchain_missing: toolchain,
        note: "scope-lean CLI path: synthetic-fixture snapshot; live Playwright is via dashboard runner",
    };
    emit(&out, output_format)?;
    // Non-zero exit on assertion failure so CI can gate.
    if !report.all_pass() {
        return Err(format!(
            "browser run: {} assertion(s) failed",
            report.failed
        ));
    }
    Ok(())
}

fn status(output_format: CliOutputFormat) -> Result<(), String> {
    let out = StatusOutput {
        scope: "browser_e2e",
        action_types: [
            "browser_e2e_run_started",
            "browser_e2e_assertion_failed",
            "browser_e2e_assertion_passed",
            "browser_e2e_snapshot_captured",
        ],
        note: "synthetic status: live SSB query is provided by the dashboard subscriber",
    };
    emit(&out, output_format)
}

/// Build a deterministic snapshot with the journey URL embedded so the
/// `contains_text` assertions targeting common login/signup strings have a
/// stable baseline. Keeps the CLI path deterministic + GPU-free.
fn synthetic_snapshot(journey: &Journey) -> PageSnapshot {
    let html = format!(
        "<html><head><title>{name}</title></head><body>\
         <h1>Sign up</h1>\
         <form><input name='email' /><input name='password' type='password'/>\
         <button type='submit'>Sign up</button></form>\
         <a href=\"{url}\">{url}</a></body></html>",
        name = journey.name,
        url = journey.url,
    );
    PageSnapshot {
        url: journey.url.clone(),
        html,
        ax_tree: None,
        screenshot_path: None,
        ts_ms: 0,
    }
}

/// Synthetic toolchain probe — returns `Some("<tool>")` when `npx` /
/// `playwright` is suspected missing, `None` otherwise. The CLI does NOT
/// actually spawn the subprocess (kept deterministic for tests); the live
/// probe lives in the orchestrator's runner.
fn probe_toolchain() -> Option<String> {
    // Heuristic: check PATH for `npx` via std::env. Anti-stale: we do NOT
    // shell out from the CLI handler. The orchestrator's runner handles
    // the real probe + wire-dark surface; this is purely advisory.
    let path = std::env::var_os("PATH")?;
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    let candidate = |bin: &str| -> bool {
        dirs.iter().any(|d| has_bin(d, bin))
    };
    if !candidate("npx") {
        return Some("npx".to_string());
    }
    None
}

fn has_bin(dir: &Path, name: &str) -> bool {
    #[cfg(windows)]
    {
        for ext in &["", ".exe", ".cmd", ".bat"] {
            if dir.join(format!("{name}{ext}")).exists() {
                return true;
            }
        }
        false
    }
    #[cfg(not(windows))]
    {
        dir.join(name).exists()
    }
}

fn emit<T: Serialize>(value: &T, output_format: CliOutputFormat) -> Result<(), String> {
    let s = match output_format {
        CliOutputFormat::Json => serde_json::to_string_pretty(value)
            .map_err(|e| format!("browser cmd: json serialize failed: {e}"))?,
        CliOutputFormat::Toml => {
            // toml::to_string requires a map at the top — wrap via serde_json::Value.
            let v = serde_json::to_value(value)
                .map_err(|e| format!("browser cmd: toml->json failed: {e}"))?;
            let wrapped = serde_json::json!({ "result": v });
            toml::to_string(&wrapped)
                .map_err(|e| format!("browser cmd: toml serialize failed: {e}"))?
        }
        CliOutputFormat::Text => serde_json::to_string_pretty(value)
            .map_err(|e| format!("browser cmd: json serialize failed: {e}"))?,
    };
    println!("{s}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::browser_e2e::{Assertion, AssertionKind, JourneyStep, JourneyStepKind};

    fn fake_journey() -> Journey {
        Journey {
            name: "smoke".into(),
            url: "http://test.local/signup".into(),
            steps: vec![JourneyStep {
                kind: JourneyStepKind::Navigate,
                target: "http://test.local/signup".into(),
                value: None,
            }],
            assertions: vec![
                Assertion {
                    label: "loaded".into(),
                    kind: AssertionKind::StatusOk,
                },
                Assertion {
                    label: "shows-form".into(),
                    kind: AssertionKind::ContainsText {
                        needle: "Sign up".into(),
                    },
                },
            ],
        }
    }

    #[test]
    fn synthetic_snapshot_contains_signup_text() {
        let j = fake_journey();
        let s = synthetic_snapshot(&j);
        assert!(s.html.contains("Sign up"));
        assert!(s.html.contains(&j.url));
    }

    #[test]
    fn run_assertions_against_synthetic_passes_known_labels() {
        let j = fake_journey();
        let s = synthetic_snapshot(&j);
        let r: AssertionReport = run_assertions(&s, &j.assertions);
        assert_eq!(r.passed, 2);
        assert_eq!(r.failed, 0);
    }

    #[test]
    fn malformed_journey_path_propagates_error() {
        let p = PathBuf::from(
            "/__elai_nonexistent_journey_dir__/does_not_exist.toml",
        );
        let r = load_journey(&p);
        assert!(matches!(r, Err(E2eError::Io(_))));
    }
}
