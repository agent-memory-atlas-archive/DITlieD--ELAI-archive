//! E-14 — Browser end-to-end agents module.
//!
//! Scope-lean implementation of the E-14 plan
//! (`docs/thefinalglance/E-plan/E-14_browser_e2e_agents.md`). Provides:
//!
//!   1. [`playwright`] — Playwright CLI snapshot-to-disk runner via
//!      `tokio::process::Command` (no in-process browser dep).
//!   2. [`snapshot`] — typed `PageSnapshot` holding HTML + optional
//!      accessibility tree + screenshot.
//!   3. [`assertion`] — `Assertion` / `AssertionReport` types and a pure
//!      `run_assertions` function operating on a snapshot.
//!   4. [`journey`] — TOML-driven journey loader for
//!      `.elai_cc/journeys/*.toml` fixtures.
//!   5. [`axe_core`] — axe-core subprocess gate (`@axe-core/playwright`
//!      via `npx`).
//!
//! Out of scope for the scope-lean cut (RUNPOD-PENDING):
//!   * VLM Mode A / Mode B integration (24 GB+ GPU required).
//!   * Image-crate-backed hybrid diff cropping.
//!   * Stagehand fallback adapter.
//!
//! Anti-stale grounding:
//!   * `tokio::process::Command` (NOT `std::process` in async).
//!   * `tokio::time::timeout` for hard kill on hang.
//!   * `thiserror` enums per submodule (NEVER `Box<dyn Error>`).
//!   * `tracing` for subprocess output (NEVER the `log` crate).
//!   * `u64::try_from` for ms narrowing (NEVER `as` casts).
//!   * No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.
//!   * Toolchain-missing surfaces wire-dark via [`E2eError::ToolchainMissing`]
//!     so the rest of the pipeline degrades to `Verdict::Degraded` rather
//!     than panicking — consistent with the C-52 dafny wire-dark precedent.

pub mod assertion;
pub mod axe_core;
pub mod journey;
pub mod playwright;
pub mod snapshot;

pub use self::assertion::{
    run_assertions, Assertion, AssertionKind, AssertionOutcome, AssertionReport,
};
pub use self::axe_core::{
    run_axe_core, AxeFinding, AxeImpact, AxeReport,
};
pub use self::journey::{load_journey, Journey, JourneyStep, JourneyStepKind};
pub use self::playwright::{PlaywrightRunner, PlaywrightSnapshotOpts};
pub use self::snapshot::PageSnapshot;

use thiserror::Error;

/// Unified error type for the browser_e2e surface. Per-submodule errors
/// flatten into this so a single `Result<_, E2eError>` covers the whole
/// pipeline at call sites. Anti-stale: `thiserror` enum, NOT `Box<dyn Error>`.
#[derive(Debug, Error)]
pub enum E2eError {
    /// Required CLI tool (npx / node / playwright / axe-core) absent from
    /// PATH. Wire-dark posture: the caller should record this as
    /// `Verdict::Degraded` and surface a SSB row, never panic.
    #[error("toolchain missing: {tool}")]
    ToolchainMissing { tool: &'static str },

    /// Subprocess exited non-zero. `code` is the OS exit status when
    /// available, `stderr` is the captured stderr (trimmed).
    #[error("subprocess `{tool}` exited code={code:?}: {stderr}")]
    SubprocessFailed {
        tool: &'static str,
        code: Option<i32>,
        stderr: String,
    },

    /// Subprocess exceeded its wall-clock budget. `millis` is the deadline
    /// that was breached (NOT the elapsed time — `tokio::time::timeout`
    /// drops the future without measuring elapsed precisely).
    #[error("subprocess `{tool}` timed out after {millis} ms")]
    Timeout { tool: &'static str, millis: u64 },

    /// Subprocess output did not parse against the expected schema (e.g.
    /// axe-results.json missing a documented field, journey TOML referring
    /// to an unknown kind). Distinct from `SubprocessFailed` so the
    /// dashboard can surface a parse-error badge separately from a tool
    /// crash.
    #[error("malformed output from `{tool}`: {detail}")]
    MalformedOutput { tool: &'static str, detail: String },

    /// I/O error reading or writing a snapshot / journey / artifact. Carries
    /// the underlying `io::Error` via `#[from]` so the `?` operator works.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolchain_missing_renders_tool_name() {
        let e = E2eError::ToolchainMissing { tool: "npx" };
        let s = format!("{e}");
        assert!(s.contains("npx"));
        assert!(s.contains("toolchain missing"));
    }

    #[test]
    fn subprocess_failed_includes_exit_code() {
        let e = E2eError::SubprocessFailed {
            tool: "playwright",
            code: Some(127),
            stderr: "command not found".into(),
        };
        let s = format!("{e}");
        assert!(s.contains("127"));
        assert!(s.contains("playwright"));
    }
}
