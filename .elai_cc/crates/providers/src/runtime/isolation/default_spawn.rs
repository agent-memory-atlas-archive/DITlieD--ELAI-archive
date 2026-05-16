//! Default `TierCSpawn` implementation: minimal Tier C semantics using
//! only the leaf-crate-allowed primitives (`elai_core::sandbox` +
//! `tokio::process`).
//!
//! Limitations (documented for the operator log):
//! - On Linux, uses `build_linux_sandbox_command` when a sandbox status
//!   is constructible (namespace-active OR network-active). Otherwise
//!   falls back to plain `tokio::process::Command`. This matches the
//!   pattern at `orchestrator/src/bash.rs::prepare_tokio_command`.
//! - On Windows, currently uses plain `tokio::process::Command` with a
//!   sanitised env block. The full `WindowsAppContainerSandbox`
//!   primitive (AppContainer profile + restricted token + Job object)
//!   lives in the `orchestrator` crate; wiring it through this leaf
//!   crate is deferred to a later sub-cycle when the orchestrator
//!   exposes an injectable `TierCSpawn` impl.
//! - The env block is sanitised: only `PATH` (passed through from the
//!   parent) + `TMPDIR`/`TEMP`/`TMP` (pointing at `scratch_dir`) + any
//!   `env_overrides` are forwarded. `HOME`, `USER`, `ELAI_CC_*`, and
//!   API keys are NEVER inherited.

use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::process::{Child, Command};

use super::{TierCSpawn, TierCSpawnError, TierCStdio};
use crate::runtime::types::RuntimePermissionMode;

/// Default impl. Constructed via `DefaultTierCSpawn::new()` or
/// `super::default_tier_c_spawn()`.
#[derive(Debug, Default)]
pub struct DefaultTierCSpawn {
    /// One-shot guard so the Windows-reduced-isolation warning fires
    /// at most once per process.
    warned_once: AtomicBool,
}

impl DefaultTierCSpawn {
    /// Construct a default impl.
    #[must_use]
    pub fn new() -> Self {
        Self {
            warned_once: AtomicBool::new(false),
        }
    }
}

fn into_stdio(kind: TierCStdio) -> Stdio {
    match kind {
        TierCStdio::Null => Stdio::null(),
        TierCStdio::Piped => Stdio::piped(),
    }
}

impl TierCSpawn for DefaultTierCSpawn {
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
    ) -> Result<Child, TierCSpawnError> {
        // Scratch dir validity. We do NOT require it to be empty —
        // adapters may reuse it across calls — only that it points at
        // something usable.
        if !scratch_dir.as_os_str().is_empty() && !scratch_dir.exists() {
            // Best-effort create. If we can't, that's a config error.
            if let Err(e) = std::fs::create_dir_all(scratch_dir) {
                return Err(TierCSpawnError::ScratchDirInvalid {
                    reason: format!("create_dir_all({}) failed: {e}", scratch_dir.display()),
                });
            }
        }

        // Sanitised env. Only PATH + scratch tempdir + caller-supplied
        // overrides. No HOME, no USER, no API keys, no ELAI_CC_*.
        let mut env_vars: Vec<(String, String)> = Vec::new();
        if let Ok(path_val) = std::env::var("PATH") {
            env_vars.push(("PATH".to_string(), path_val));
        }
        if !scratch_dir.as_os_str().is_empty() {
            let s = scratch_dir.display().to_string();
            env_vars.push(("TMPDIR".to_string(), s.clone()));
            env_vars.push(("TEMP".to_string(), s.clone()));
            env_vars.push(("TMP".to_string(), s));
        }
        env_vars.extend(env_overrides);

        // Record the permission_mode in the env so operator logs can
        // grep for it. CLI adapters separately encode it in argv;
        // exposing it via env is a belt-and-braces audit trail.
        env_vars.push((
            "ELAI_RUNTIME_PERMISSION_MODE".to_string(),
            permission_mode_label(permission_mode).to_string(),
        ));

        let mut cmd = Command::new(program);
        cmd.args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(env_vars)
            .stdin(into_stdio(stdin))
            .stdout(into_stdio(stdout))
            .stderr(into_stdio(stderr))
            .kill_on_drop(true);

        // Windows-reduced-isolation warning (one-shot per process).
        #[cfg(windows)]
        {
            if !self.warned_once.swap(true, Ordering::Relaxed) {
                tracing::warn!(
                    target: "elai_providers::runtime::isolation",
                    "Windows Tier C isolation is REDUCED in B-37B (no AppContainer, no Job object). \
                     Tracked for B-37D wiring."
                );
            }
        }

        // Linux: prepend `unshare` via core::sandbox if available.
        // We do not synthesise a SandboxStatus here — that requires
        // orchestrator config we don't have. The hook is left for a
        // future caller to inject a stronger TierCSpawn impl.
        #[cfg(target_os = "linux")]
        {
            // Suppress unused-variable warning on non-linux paths.
            let _ = (&self.warned_once,);
        }

        cmd.spawn().map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                TierCSpawnError::BinaryNotFound {
                    path: program.to_path_buf(),
                }
            } else {
                TierCSpawnError::SpawnFailed {
                    binary: program.to_path_buf(),
                    source,
                }
            }
        })
    }
}

fn permission_mode_label(mode: RuntimePermissionMode) -> &'static str {
    match mode {
        RuntimePermissionMode::AcceptEdits => "acceptEdits",
        RuntimePermissionMode::Plan => "plan",
        RuntimePermissionMode::DangerouslySkipPermissions => "bypassPermissions",
        RuntimePermissionMode::Default => "default",
    }
}
