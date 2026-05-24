//! E-12 Tier-B — Production checklist runners (15 sub-modules).
//!
//! Each runner shells out to one external CLI (gitleaks, trufflehog,
//! semgrep, npm-audit, pip-audit, cargo-audit, lighthouse, biome,
//! ruff, clippy, sqlite3, cargo-machete, otel-check, sentry-check,
//! lsof) and returns a structured [`RunResult`]. The 15 CLIs are
//! NOT installed by default on the local dev host — every runner
//! returns [`RunResult::ToolchainMissing`] instead of panicking when
//! the binary is absent. Live invocation against representative apps
//! is RUNPOD-PENDING per
//! `workflow/reasoning/E-12-tier-b-decisions.md`.
//!
//! Wire-dark posture (canonical 5th application per memory rule
//! `wire-dark-posture` + gotchas.md:22-23):
//!   - Real `tokio::process::Command` subprocess wires.
//!   - Real structured `thiserror` error type — no `Box<dyn Error>`,
//!     no `.unwrap()`, no `.expect()` outside tests, no string-only
//!     `Result<_, String>`.
//!   - Real per-runner timeout (30s default, configurable to 60s per
//!     runner).
//!   - `// TOOLCHAIN-PENDING-<TOOL>: <install command>` seam marker
//!     in every runner file so the live-toolchain dispatcher can grep
//!     for pending wiring.
//!   - `ToolchainMissing` branch is NOT a stub — it is a sentinel
//!     return value that downstream consumers (auditor / dashboard /
//!     SSB row) match on alongside `Ok` and `Error`.
//!
//! Runner registry: [`ALL_RUNNERS`] lists the 15 runners in
//! [`SecurityCheckId`] declaration order. The
//! `crate::framework_profile::FrameworkAdapter::production_checklist`
//! method returns the subset the framework opts into; the
//! orchestrator dispatches each in sequence and collects the results.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::framework_profile::profile::SecurityCheckId;

pub mod biome_runner;
pub mod cargo_audit_runner;
pub mod cargo_machete_runner;
pub mod clippy_runner;
pub mod gitleaks_runner;
pub mod lighthouse_runner;
pub mod lsof_port_runner;
pub mod npm_audit_runner;
pub mod otel_check_runner;
pub mod pip_audit_runner;
pub mod ruff_runner;
pub mod semgrep_runner;
pub mod sentry_check_runner;
pub mod sqlite3_runner;
pub mod trufflehog_runner;

/// Default per-runner timeout (30 seconds). Heavy runners override
/// to 60s via per-runner `RUNNER_TIMEOUT`.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Heavy-runner timeout (60 seconds). Used by `lighthouse_runner` +
/// `semgrep_runner`.
pub const HEAVY_TIMEOUT: Duration = Duration::from_secs(60);

/// A single finding emitted by a checklist runner. Plain-text
/// severity + category + message + optional file/line — no enum
/// proliferation, so the runner output stays serde-compatible with
/// the auditor's existing `Finding` shape at `evidence::Finding`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistFinding {
    /// Severity wire label: `block` / `warn` / `info`.
    pub severity: String,
    /// Category wire label (snake_case). Matches the runner's
    /// `SecurityCheckId::as_str()` by default but may be narrowed
    /// (e.g. `secrets_scan/sk_prefix`).
    pub category: String,
    /// Human-readable finding message. Bounded to 512 chars by the
    /// runner; longer messages are truncated.
    pub message: String,
    /// Optional file path the finding pertains to.
    pub file: Option<String>,
    /// Optional 1-based line number.
    pub line: Option<u32>,
}

/// Outcome of a single runner invocation. Three explicit variants
/// — `ToolchainMissing` is distinct from `Error` so consumers can
/// gate the production-grade verdict on toolchain availability
/// without losing diagnostic detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunResult {
    /// External CLI absent from PATH. Carries the install hint so
    /// the dashboard can render a one-line action item.
    ToolchainMissing {
        tool: String,
        install_hint: String,
    },
    /// Runner completed; findings (possibly empty) attached.
    Ok { findings: Vec<ChecklistFinding> },
    /// Runner spawned but the CLI failed (non-zero exit + non-empty
    /// stderr, timeout, or unparseable output). `details` carries
    /// the canonical failure message; `stderr_excerpt` carries the
    /// first 512 bytes of captured stderr (or an empty string when
    /// the failure was a timeout / spawn error).
    Error {
        details: String,
        stderr_excerpt: String,
    },
}

impl RunResult {
    /// Convenience: true iff the runner returned `Ok` with NO
    /// findings.
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Ok { findings } if findings.is_empty())
    }

    /// Convenience: true iff this runner could not execute because
    /// the CLI binary was absent.
    pub fn is_toolchain_missing(&self) -> bool {
        matches!(self, Self::ToolchainMissing { .. })
    }
}

/// Structured error type for runner-internal failures. Public so
/// the orchestrator can `match` on the kind. The variants are NOT a
/// 1:1 wrapper around `RunResult::Error` — that variant is the
/// stable wire shape; this enum carries source-pinned errors that
/// the runner converts into `RunResult::Error` before returning.
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("io error invoking `{tool}`: {source}")]
    Io {
        tool: String,
        #[source]
        source: std::io::Error,
    },
    #[error("`{tool}` timed out after {elapsed_secs}s")]
    Timeout { tool: String, elapsed_secs: u64 },
    #[error("`{tool}` exited with code {code}: {stderr_excerpt}")]
    NonZeroExit {
        tool: String,
        code: i32,
        stderr_excerpt: String,
    },
    #[error("unable to parse `{tool}` output: {reason}")]
    ParseError { tool: String, reason: String },
    #[error("target path does not exist: {path}")]
    TargetMissing { path: String },
}

impl RunnerError {
    /// Render this error as the canonical `RunResult::Error` wire
    /// shape, preserving the stderr excerpt when present.
    pub fn to_run_result(&self) -> RunResult {
        match self {
            Self::NonZeroExit {
                stderr_excerpt, ..
            } => RunResult::Error {
                details: self.to_string(),
                stderr_excerpt: stderr_excerpt.clone(),
            },
            _ => RunResult::Error {
                details: self.to_string(),
                stderr_excerpt: String::new(),
            },
        }
    }
}

/// Trim a captured stderr / stdout buffer to the bounded 512-byte
/// excerpt the wire types carry. Avoids unbounded SSB row growth on
/// runners whose stderr can balloon (semgrep / lighthouse).
pub fn excerpt(buf: &str) -> String {
    const MAX: usize = 512;
    if buf.len() <= MAX {
        buf.to_owned()
    } else {
        // Find a valid char boundary at or below MAX.
        let mut end = MAX;
        while end > 0 && !buf.is_char_boundary(end) {
            end -= 1;
        }
        buf[..end].to_owned()
    }
}

/// `is_path_not_found_error` returns true when the io error
/// corresponds to a missing PATH-resolved binary (the wire-dark
/// `ToolchainMissing` branch). Tier-A guard 2 + memory:wire-dark-posture
/// canonical.
pub fn is_path_not_found(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::NotFound
}

/// Dispatch one runner by [`SecurityCheckId`]. Returns the
/// [`RunResult`] without touching the SSB — the orchestrator caller
/// is responsible for emitting the matching SSB rows on top.
///
/// The 15-runner dispatch is sequential (NOT parallel) to keep the
/// total wall-clock bounded and to make the auditor's verdict
/// deterministic. Parallel dispatch is a follow-on patch-N if the
/// 30s * 15 = 7.5min budget becomes a bottleneck on real apps.
pub async fn run_check(check: SecurityCheckId, target: &Path) -> RunResult {
    match check {
        SecurityCheckId::SecretsScan => gitleaks_runner::run(target).await,
        SecurityCheckId::TrufflehogScan => trufflehog_runner::run(target).await,
        SecurityCheckId::OwaspStatic => semgrep_runner::run(target).await,
        SecurityCheckId::NpmAudit => npm_audit_runner::run(target).await,
        SecurityCheckId::PipAudit => pip_audit_runner::run(target).await,
        SecurityCheckId::CargoAudit => cargo_audit_runner::run(target).await,
        SecurityCheckId::Lighthouse => lighthouse_runner::run(target).await,
        SecurityCheckId::Biome => biome_runner::run(target).await,
        SecurityCheckId::Ruff => ruff_runner::run(target).await,
        SecurityCheckId::Clippy => clippy_runner::run(target).await,
        SecurityCheckId::Sqlite3Rls => sqlite3_runner::run(target).await,
        SecurityCheckId::CargoMachete => cargo_machete_runner::run(target).await,
        SecurityCheckId::OtelCheck => otel_check_runner::run(target).await,
        SecurityCheckId::SentryCheck => sentry_check_runner::run(target).await,
        SecurityCheckId::LsofPort => lsof_port_runner::run(target).await,
    }
}

/// Run every check in the given list against `target`. Returns the
/// per-check results in declaration order. Toolchain-missing entries
/// are recorded but do NOT short-circuit — the auditor consumer
/// folds them into the verdict via the `degraded_no_toolchain` flag
/// (see auditor_mode.rs).
pub async fn run_all(target: &Path, checks: &[SecurityCheckId]) -> Vec<(SecurityCheckId, RunResult)> {
    let mut out: Vec<(SecurityCheckId, RunResult)> = Vec::with_capacity(checks.len());
    for &check in checks {
        let res = run_check(check, target).await;
        out.push((check, res));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpt_bounds_to_512() {
        let long = "a".repeat(1000);
        let e = excerpt(&long);
        assert!(e.len() <= 512);
    }

    #[test]
    fn excerpt_preserves_short_input() {
        let s = "short";
        assert_eq!(excerpt(s), "short");
    }

    #[test]
    fn run_result_is_clean_only_on_empty_findings() {
        let r = RunResult::Ok {
            findings: Vec::new(),
        };
        assert!(r.is_clean());
        let r2 = RunResult::Ok {
            findings: vec![ChecklistFinding {
                severity: "info".into(),
                category: "x".into(),
                message: "y".into(),
                file: None,
                line: None,
            }],
        };
        assert!(!r2.is_clean());
    }

    #[test]
    fn run_result_toolchain_missing_distinct_from_error() {
        let tm = RunResult::ToolchainMissing {
            tool: "gitleaks".into(),
            install_hint: "brew install gitleaks".into(),
        };
        assert!(tm.is_toolchain_missing());
        let err = RunResult::Error {
            details: "boom".into(),
            stderr_excerpt: String::new(),
        };
        assert!(!err.is_toolchain_missing());
    }

    #[tokio::test]
    async fn run_check_with_missing_target_returns_toolchain_missing_or_error() {
        // The 15 CLIs are absent on dev host -> every runner returns
        // ToolchainMissing. This wire-dark posture is the canonical
        // baseline; live invocation lives behind a RUNPOD-PENDING
        // marker per E-12-tier-b-decisions.md.
        let bogus = std::path::Path::new("/elai_e12_bogus_target_for_runners_xyzzy");
        let res = run_check(SecurityCheckId::SecretsScan, bogus).await;
        // Either the binary is missing (ToolchainMissing) or the
        // target is missing (Error). Both are stable wire shapes.
        assert!(
            matches!(res, RunResult::ToolchainMissing { .. } | RunResult::Error { .. }),
            "unexpected RunResult: {:?}",
            res
        );
    }
}
