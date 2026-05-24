//! C-52 — `dafny verify` CLI shellout.
//!
//! Mirrors the `tokio::process::Command` shellout pattern established
//! by `crate::github_pr::gh_cli` (C-40) and `crate::mutation_testing`
//! (C-28). Resolution rules:
//!   - The `dafny` binary is PATH-resolved at invocation. No
//!     long-lived cache. Users install via Chocolatey / scoop /
//!     manual download per dafny.org.
//!   - Local dev hosts MAY NOT have Dafny installed. The
//!     `ToolchainNotFound` error is RETURNED (not panic) so callers
//!     can record the absence and proceed via the infeasibility-note
//!     path. See `workflow/reasoning/C-52-decisions.md` and
//!     `RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution`.
//!   - The shellout captures stdout + stderr separately. Pass =
//!     exit_code == 0 AND stdout contains the canonical Dafny
//!     verifier banner "Dafny program verifier". Fail = any other
//!     terminating state.
//!   - The function does NOT enforce a hard timeout itself —
//!     timeouts are the caller's responsibility (the C-52 heal-loop
//!     wires a 5-minute per-iteration cap; sibling registry-based
//!     checkers use the 30s `SHELLOUT_TIMEOUT` constant).
//!
//! Vacuous-verification GUARD: `DafnyVerifyResult::vacuous_guard_passed`
//! is set by the caller (see `udebug_guard::check_vacuous`) AFTER the
//! Dafny exit and the functional-test suite results land. The Dafny
//! exit alone is INSUFFICIENT — paper §3 documents vacuous specs
//! that pass Dafny but fail functional tests.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;

/// Default binary name. Resolved against PATH at invocation.
const DAFNY_BINARY: &str = "dafny";

/// Banner string emitted by the Dafny verifier on a successful run.
/// Pinned for stability against future Dafny versions; if Dafny ever
/// drops this banner the contract test G2 will surface the drift.
const DAFNY_BANNER: &str = "Dafny program verifier";

/// Result of a single `dafny verify` shellout.
///
/// `vacuous_guard_passed` is initialized to `false` by `invoke_dafny_cli`
/// and is FLIPPED by the caller once the functional-test suite
/// (uDebug-style, see `udebug_guard`) has also passed. A spec is
/// considered VERIFIED iff `dafny_pass && udebug_pass &&
/// vacuous_guard_passed`. Raw Dafny exit alone is insufficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DafnyVerifyResult {
    /// Whether the `dafny verify` invocation succeeded (exit 0 AND
    /// stdout contained the canonical verifier banner).
    pub dafny_pass: bool,
    /// Whether the uDebug-style functional test suite passed. Set by
    /// `udebug_guard::check_vacuous` AFTER the Dafny exit. Defaults
    /// to `false` (assume vacuous until proven otherwise).
    pub udebug_pass: bool,
    /// Whether both `dafny_pass` AND `udebug_pass` are `true`.
    /// Derived field — true verification claim. Defaults to `false`.
    pub vacuous_guard_passed: bool,
    /// Captured stdout (truncated to 64 KiB to bound payload size).
    pub stdout: String,
    /// Process exit code. `-1` is used when the process failed to
    /// spawn (binary missing); otherwise the OS-reported code.
    pub exit_code: i32,
}

impl DafnyVerifyResult {
    /// Maximum stdout length captured. Larger payloads are truncated
    /// to keep SSB row size bounded. 64 KiB matches the mutation
    /// testing pattern in `crate::mutation_testing`.
    pub const MAX_STDOUT_BYTES: usize = 64 * 1024;

    /// Build a fresh failure result (used by the spawn-failure path).
    pub fn spawn_failure(stdout: String, exit_code: i32) -> Self {
        Self {
            dafny_pass: false,
            udebug_pass: false,
            vacuous_guard_passed: false,
            stdout,
            exit_code,
        }
    }
}

/// Error type for the shellout layer. `ToolchainNotFound` is the
/// expected path on the local dev host (Dafny binary absent) — see
/// the module header for the RUNPOD-PENDING note.
#[derive(Debug, Error)]
pub enum DafnyCliError {
    #[error("dafny toolchain not found on PATH (install per dafny.org); io: {0}")]
    ToolchainNotFound(#[source] std::io::Error),

    #[error("spec file not found at {path}")]
    SpecNotFound { path: String },

    #[error("failed to read spec stdout: {0}")]
    Io(#[from] std::io::Error),
}

/// Invoke `dafny verify <spec_path>` and parse the result.
///
/// Returns `Ok(DafnyVerifyResult)` on a clean spawn — even when the
/// verifier itself reported errors (those land as `dafny_pass:
/// false`). Returns `Err(ToolchainNotFound)` only when the binary
/// cannot be spawned at all (e.g. `dafny` is not on PATH).
///
/// The `udebug_pass` and `vacuous_guard_passed` fields are left at
/// their default `false` values; callers MUST run the functional-test
/// suite separately and call `udebug_guard::check_vacuous` to flip
/// them. See module header for the vacuous-verification GUARD note.
pub async fn invoke_dafny_cli(spec_path: &Path) -> Result<DafnyVerifyResult, DafnyCliError> {
    if !spec_path.exists() {
        return Err(DafnyCliError::SpecNotFound {
            path: spec_path.display().to_string(),
        });
    }
    invoke_with_binary(DAFNY_BINARY, spec_path).await
}

/// Inner helper that lets the contract test override the binary name
/// for offline testing (e.g. point at a `dafny_mock.sh` fixture
/// without modifying PATH).
pub async fn invoke_with_binary(
    binary: &str,
    spec_path: &Path,
) -> Result<DafnyVerifyResult, DafnyCliError> {
    let mut cmd = Command::new(binary);
    cmd.arg("verify").arg(spec_path);
    cmd.kill_on_drop(true);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = match cmd.output().await {
        Ok(out) => out,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(DafnyCliError::ToolchainNotFound(e));
        }
        Err(e) => return Err(DafnyCliError::Io(e)),
    };

    let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if stdout.len() > DafnyVerifyResult::MAX_STDOUT_BYTES {
        stdout.truncate(DafnyVerifyResult::MAX_STDOUT_BYTES);
    }
    let exit_code = output.status.code().unwrap_or(-1);
    let banner_present = stdout.contains(DAFNY_BANNER);
    let dafny_pass = output.status.success() && banner_present;

    Ok(DafnyVerifyResult {
        dafny_pass,
        // udebug_pass / vacuous_guard_passed populated by caller
        // via udebug_guard::check_vacuous. Default = false until the
        // functional-test suite has been run.
        udebug_pass: false,
        vacuous_guard_passed: false,
        stdout,
        exit_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn missing_spec_returns_spec_not_found() {
        let nonexistent = PathBuf::from("does_not_exist_xyzzy.dfy");
        let res = invoke_dafny_cli(&nonexistent).await;
        match res {
            Err(DafnyCliError::SpecNotFound { path }) => {
                assert!(path.contains("xyzzy"));
            }
            other => panic!("expected SpecNotFound; got {:?}", other),
        }
    }

    #[tokio::test]
    async fn missing_binary_returns_toolchain_not_found() {
        // Create a tempfile so SpecNotFound is NOT the failure mode;
        // we want the binary-missing path.
        let tmp = std::env::temp_dir().join("c52_empty_spec.dfy");
        std::fs::write(&tmp, "// empty Dafny spec").expect("write temp spec");

        let bogus_binary = "elai_c52_bogus_dafny_binary_xyzzy";
        let res = invoke_with_binary(bogus_binary, &tmp).await;
        // Best-effort cleanup; ignore failure.
        let _ = std::fs::remove_file(&tmp);
        match res {
            Err(DafnyCliError::ToolchainNotFound(_)) => {}
            other => panic!("expected ToolchainNotFound; got {:?}", other),
        }
    }

    #[test]
    fn spawn_failure_constructor_defaults_all_false() {
        let r = DafnyVerifyResult::spawn_failure("oops".into(), -1);
        assert!(!r.dafny_pass);
        assert!(!r.udebug_pass);
        assert!(!r.vacuous_guard_passed);
        assert_eq!(r.exit_code, -1);
    }
}
