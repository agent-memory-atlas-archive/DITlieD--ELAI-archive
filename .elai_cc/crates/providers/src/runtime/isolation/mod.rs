//! Tier C isolation abstraction for the B-37B CLI adapters.
//!
//! The providers crate is a LEAF in the dependency graph (per
//! `systemPatterns.md` L30 + L138 — "providers crate is leaf — no
//! orchestrator dep edge"). The real Tier C primitive
//! (`WindowsAppContainerSandbox`, `LinuxSandbox`) lives in the
//! `orchestrator` crate, which depends on `providers` — taking a
//! reverse edge here would violate the layering rule.
//!
//! Resolution (HIGH-2 from context-package-B37B.md): define a small
//! `TierCSpawn` trait inside `providers` whose default implementation
//! (`DefaultTierCSpawn`) uses `elai_core::sandbox::build_linux_sandbox_command`
//! on Linux and plain `tokio::process::Command` on Windows. The
//! orchestrator may later provide a `WindowsAppContainerSandbox`-backed
//! implementation by implementing the trait separately and injecting
//! it via the per-adapter `with_spawn` constructor.
//!
//! Documented limitation: Windows isolation in B-37B is REDUCED
//! relative to `elai_cc_agent` (no AppContainer profile, no restricted
//! token, no Job object). The default impl logs a one-shot `tracing::warn!`
//! at construction so operators can grep for the gap.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::process::Child;

use super::types::RuntimePermissionMode;

pub mod default_spawn;

pub use default_spawn::DefaultTierCSpawn;

/// stdio-channel selector for `TierCSpawn::spawn_isolated`. Mirrors
/// `std::process::Stdio::{null, piped}` semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierCStdio {
    /// `/dev/null`-equivalent: no inheritance, no pipe.
    Null,
    /// Pipe creation; caller takes ownership of the
    /// `ChildStdin`/`ChildStdout`/`ChildStderr` handle.
    Piped,
}

/// Errors surfaced by the Tier C spawn primitive. Adapter call paths
/// translate these into `RuntimeAdapterError` for the public surface.
#[derive(Debug, thiserror::Error)]
pub enum TierCSpawnError {
    /// CLI binary not found on disk.
    #[error("CLI binary not found: {path}")]
    BinaryNotFound { path: PathBuf },
    /// `Command::spawn` returned an OS error.
    #[error("spawn failed for `{binary}`: {source}")]
    SpawnFailed {
        binary: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// Scratch dir failed validity check (missing, not a directory).
    #[error("scratch dir invalid: {reason}")]
    ScratchDirInvalid { reason: String },
}

/// Tier C subprocess spawn abstraction.
///
/// All B-37B CLI adapters spawn through this trait; the default
/// implementation provides minimal Linux isolation
/// (`build_linux_sandbox_command`) and plain tokio on Windows. The
/// orchestrator may swap in a stronger impl later by implementing
/// this trait against `WindowsAppContainerSandbox` / `LinuxSandbox`.
///
/// `env_overrides`: explicit env vars to set on the child. The default
/// implementation NEVER inherits the parent process's full env block;
/// only `PATH` and entries in `env_overrides` are passed through. This
/// matches the existing `core::sandbox::build_linux_sandbox_command`
/// behaviour (sanitised env at sandbox.rs:241-260) and prevents
/// credential leakage into untrusted CLI subprocesses.
pub trait TierCSpawn: Send + Sync {
    /// Spawn `program` with `args`, attached to `cwd`, with isolation
    /// applied. Returns the live `tokio::process::Child`; the caller
    /// is responsible for `.kill().await` / `.wait().await`.
    #[allow(clippy::too_many_arguments)]
    fn spawn_isolated(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        scratch_dir: &Path,
        stdin: TierCStdio,
        stdout: TierCStdio,
        stderr: TierCStdio,
        permission_mode: RuntimePermissionMode,
        env_overrides: Vec<(String, String)>,
    ) -> Result<Child, TierCSpawnError>;
}

/// Construct the default `TierCSpawn` implementation. Returns
/// `Arc<dyn TierCSpawn>` so adapters can hold it cheaply via clone.
#[must_use]
pub fn default_tier_c_spawn() -> Arc<dyn TierCSpawn> {
    Arc::new(DefaultTierCSpawn::new())
}
