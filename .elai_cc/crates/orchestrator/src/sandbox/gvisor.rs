//! gVisor (runsc) KVM-less sandbox backend.
//! E-15: When kvm_available() returns false, spawn agent via runsc with
//! platform=systrap (gVisor default since 2023).
//! Performance note (gvisor.dev/docs/architecture_guide/performance + usenix.org HotCloud19):
//!   gVisor syscall overhead ~10-20% vs Firecracker 1-2%.
//!   Acceptable as KVM-less fallback; not acceptable as primary on KVM hosts.
//! Emits sandbox_backend_selected SSB with backend=Gvisor per spawn.

use std::path::{Path, PathBuf};

/// gVisor backend that spawns agent processes via runsc.
pub struct GvisorBackend {
    platform: String,
    runsc_binary: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum GvisorError {
    #[error("runsc binary not found on PATH")]
    BinaryNotFound,
    #[error("runsc spawn failed: {0}")]
    SpawnFailed(String),
    #[error("gVisor unavailable: KVM present (use Firecracker instead)")]
    KvmPresent,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct GvisorSpawnResult {
    pub platform: String,
    pub runsc_path: Option<PathBuf>,
    pub was_fallback: bool,
}

impl GvisorBackend {
    /// Create a gVisor backend with Systrap platform (default since gVisor 2023).
    /// Systrap avoids ptrace() overhead via inline trampolines (~10-20% overhead).
    pub fn new() -> Self { Self { platform: "systrap".to_string(), runsc_binary: find_runsc_binary() } }

    pub fn with_platform(platform: impl Into<String>) -> Self {
        Self { platform: platform.into(), runsc_binary: find_runsc_binary() }
    }

    pub fn is_available(&self) -> bool { self.runsc_binary.is_some() }

    /// Spawn a command via runsc.
    /// Returns GvisorError::KvmPresent if KVM detected. Set force=true to bypass (for tests).
    pub fn spawn(&self, command: &str, args: &[String], scratch_dir: &Path, force: bool) -> Result<GvisorSpawnResult, GvisorError> {
        #[cfg(target_os = "linux")]
        if !force && super::kvm_available() { return Err(GvisorError::KvmPresent); }
        let runsc = self.runsc_binary.as_ref().ok_or(GvisorError::BinaryNotFound)?;
        let status = std::process::Command::new(runsc).arg("run").arg(format!("--platform={}", self.platform)).arg("--").arg(command).args(args).current_dir(scratch_dir).status().map_err(|e| GvisorError::SpawnFailed(format!("{e}")))?;
        if !status.success() { return Err(GvisorError::SpawnFailed(format!("runsc exited with: {status}"))); }
        tracing::debug!(platform = %self.platform, "sandbox_backend_selected: backend=Gvisor");
        Ok(GvisorSpawnResult { platform: self.platform.clone(), runsc_path: self.runsc_binary.clone(), was_fallback: true })
    }
}

impl Default for GvisorBackend { fn default() -> Self { Self::new() } }

fn find_runsc_binary() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| { std::env::split_paths(&path_var).map(|dir| dir.join("runsc")).find(|p| p.is_file()) })
}
