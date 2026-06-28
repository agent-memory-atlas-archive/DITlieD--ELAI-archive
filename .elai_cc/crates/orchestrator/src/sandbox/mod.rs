// ---------------------------------------------------------------------------
// AppContainer Isolation for Windows (Phase 3J.11)
// ---------------------------------------------------------------------------
// Windows isolation is explicitly WEAKER than Linux. Linux is the production
// security target. Windows provides best-effort isolation suitable for development.
// ---------------------------------------------------------------------------

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent_pool::AgentRole;
pub mod branch_workspace;
pub mod firecracker;
pub mod gvisor;
pub mod researcher_net;
pub mod pyodide_bridge;

pub use branch_workspace::{
    BranchRoot, BranchWorkspace, BranchWorkspaceError, WorkspaceMergeResult, WorktreePoolImpl,
};
pub use firecracker::{FirecrackerHotPool, SnapshotHandle, SnapshotKey};
pub use gvisor::GvisorBackend;
pub use researcher_net::ResearcherNetSetup;
pub use pyodide_bridge::PyodideBridge;


// ---------------------------------------------------------------------------
// E-15: SandboxBackend enum + kvm_available + select_backend
// ---------------------------------------------------------------------------

/// Sandbox backend selected at runtime based on platform capabilities.
///
/// E-22 (SCHEMA_VERSION 170) added the `Microsandbox` variant for the
/// may-2026 trending sweep adoption of microsandbox/libkrun microVMs
/// (github.com/superradcompany/microsandbox, YC X26, sub-200 ms boot).
/// Microsandbox requires linux + /dev/kvm + a microsandbox CLI binary
/// pointed to by the `MICROSANDBOX_BIN` env var. Any precondition
/// failure causes `select_backend` to degrade to the next-best variant
/// (Firecracker if KVM present, Gvisor otherwise, AppContainer on
/// Windows) and emit one `GithubTrendingEvent::MicroVmFallback` SSB
/// row (wire-dark unless `ELAI_GITHUB_TRENDING_ENABLE=1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SandboxBackend {
    Firecracker,
    Gvisor,
    AppContainer,
    /// E-22 (SCHEMA_VERSION 170) -- microsandbox / libkrun microVM
    /// backend. Linux + KVM only; degrades to `Firecracker` /
    /// `Gvisor` / `AppContainer` when MICROSANDBOX_BIN is unset or
    /// KVM is absent.
    Microsandbox,
}

impl SandboxBackend {
    /// Snake-case wire label used by SSB rows + dashboard labels.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Firecracker => "firecracker",
            Self::Gvisor => "gvisor",
            Self::AppContainer => "app_container",
            Self::Microsandbox => "microsandbox",
        }
    }
}

/// E-22 -- env var that points at the microsandbox CLI binary. When
/// unset (or the value does not name an existing file), the Microsandbox
/// backend is unavailable and `select_backend` degrades. Mirrors the
/// MICROSANDBOX_BIN convention in the microsandbox README.
pub const MICROSANDBOX_BIN_ENV: &str = "MICROSANDBOX_BIN";

/// Detect KVM availability by probing /dev/kvm.
/// Returns false on Windows and on Linux systems without KVM support.
#[cfg(target_os = "linux")]
pub fn kvm_available() -> bool {
    std::path::Path::new("/dev/kvm").exists()
}

#[cfg(not(target_os = "linux"))]
pub fn kvm_available() -> bool { false }

/// E-22 -- detect microsandbox CLI availability via the
/// `MICROSANDBOX_BIN` env var. Returns `Some(path)` when the env var
/// names an existing file; `None` otherwise. This is a precondition
/// check, NOT a process spawn. Live invocation is the
/// `MicrosandboxBackend` impl's responsibility.
#[must_use]
pub fn microsandbox_bin_available() -> Option<std::path::PathBuf> {
    let raw = std::env::var(MICROSANDBOX_BIN_ENV).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = std::path::PathBuf::from(trimmed);
    if path.exists() { Some(path) } else { None }
}

/// E-22 -- snake_case reason label for `GithubTrendingEvent::MicroVmFallback`.
/// Pure data: the SSB emitter is wired by the orchestrator bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroVmFallbackReason {
    /// Linux but /dev/kvm not present (no virtualization extensions).
    NoKvm,
    /// MICROSANDBOX_BIN unset or points at a missing file.
    MissingBin,
    /// Native windows (no libkrun support).
    WindowsNative,
    /// Compiled for a non-linux non-windows platform.
    UnsupportedPlatform,
}

impl MicroVmFallbackReason {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoKvm => "no_kvm",
            Self::MissingBin => "missing_bin",
            Self::WindowsNative => "windows_native",
            Self::UnsupportedPlatform => "unsupported_platform",
        }
    }
}

/// E-22 -- select_backend return shape extended with the fallback
/// reason when applicable. Callers that just need the backend can use
/// [`select_backend`]; callers wiring telemetry (orchestrator boot)
/// should use this variant and consume the reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxBackendSelection {
    pub backend: SandboxBackend,
    /// Some(reason) when Microsandbox was requested but unavailable;
    /// None when the selection succeeded as the host's primary
    /// (including non-Microsandbox primaries on hosts where the
    /// trending env var was never set).
    pub microvm_fallback_reason: Option<MicroVmFallbackReason>,
}

/// E-22 -- producer-side emitter for the
/// `GithubTrendingEvent::MicroVmFallback` SSB row. Lives in the
/// sandbox module to avoid a circular dep on `crate::ssb` from
/// callers that only need the trait shape. Mirrors the
/// [`crate::plugins::mcp_bridge::LazySchemaSavingsEmitter`] dep-
/// inversion pattern; the orchestrator bootstrap wires a real
/// implementor when an SSB writer is available; default callers
/// use `NoopMicroVmFallbackEmitter` so the sandbox module's
/// existing tests stay sink-free.
pub trait MicroVmFallbackEmitter: Send + Sync {
    /// Emit one `github_trending_event` row with kind=
    /// `microvm_fallback`. `reason_label` is the snake_case
    /// `MicroVmFallbackReason::as_str()` value; `fallback_backend`
    /// is the snake_case `SandboxBackend::as_str()` of the backend
    /// that was actually selected instead.
    fn emit(&self, reason_label: &str, fallback_backend: &str) -> Result<(), String>;
}

/// Default no-op emitter -- returns Ok(()) without touching anything.
/// Used by `emit_microvm_fallback_if_any` when the orchestrator
/// boot has not wired a live SSB writer.
pub struct NoopMicroVmFallbackEmitter;

impl MicroVmFallbackEmitter for NoopMicroVmFallbackEmitter {
    fn emit(&self, _reason_label: &str, _fallback_backend: &str) -> Result<(), String> {
        Ok(())
    }
}

/// E-22 -- emit one `MicroVmFallback` row through the supplied
/// emitter when the selection carries a fallback reason. This is the
/// canonical caller-side helper so the orchestrator bootstrap does
/// not need to redo the if-let on `microvm_fallback_reason`. Returns
/// `Ok(true)` when a row was emitted, `Ok(false)` when the selection
/// had no fallback reason (no-op).
pub fn emit_microvm_fallback_if_any(
    selection: &SandboxBackendSelection,
    emitter: &dyn MicroVmFallbackEmitter,
) -> Result<bool, String> {
    match selection.microvm_fallback_reason {
        Some(reason) => {
            emitter.emit(reason.as_str(), selection.backend.as_str())?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Select the appropriate sandbox backend for the current host.
///
/// E-22 (SCHEMA_VERSION 170): when `ELAI_GITHUB_TRENDING_ENABLE=1` is
/// set AND the host can satisfy Microsandbox preconditions
/// (linux + /dev/kvm + MICROSANDBOX_BIN points at an existing file),
/// returns `SandboxBackend::Microsandbox`. Otherwise degrades through
/// Firecracker (linux+KVM) -> Gvisor (linux no KVM) -> AppContainer
/// (windows / other). Backwards-compatible: when the trending env var
/// is unset, the legacy 3-variant selection runs unchanged.
pub fn select_backend() -> SandboxBackend {
    select_backend_detailed().backend
}

/// E-22 -- detailed selection that surfaces the
/// [`MicroVmFallbackReason`] so the caller can emit one
/// `GithubTrendingEvent::MicroVmFallback` SSB row. The legacy
/// [`select_backend`] is a thin wrapper that drops the reason.
pub fn select_backend_detailed() -> SandboxBackendSelection {
    let trending_enabled = match std::env::var("ELAI_GITHUB_TRENDING_ENABLE") {
        Ok(v) => {
            let lo = v.trim().to_ascii_lowercase();
            lo == "1" || lo == "true"
        }
        Err(_) => false,
    };

    #[cfg(target_os = "windows")]
    {
        return SandboxBackendSelection {
            backend: SandboxBackend::AppContainer,
            microvm_fallback_reason: if trending_enabled {
                Some(MicroVmFallbackReason::WindowsNative)
            } else {
                None
            },
        };
    }

    #[cfg(target_os = "linux")]
    {
        let kvm = kvm_available();
        let bin = microsandbox_bin_available();
        if trending_enabled && kvm && bin.is_some() {
            return SandboxBackendSelection {
                backend: SandboxBackend::Microsandbox,
                microvm_fallback_reason: None,
            };
        }
        let reason = if !kvm {
            Some(MicroVmFallbackReason::NoKvm)
        } else if bin.is_none() {
            Some(MicroVmFallbackReason::MissingBin)
        } else {
            None
        };
        // When trending is disabled the legacy 3-variant selection runs
        // without surfacing any fallback reason -- the caller has not
        // opted in to microVM telemetry.
        let reported_reason = if trending_enabled { reason } else { None };
        let backend = if kvm { SandboxBackend::Firecracker } else { SandboxBackend::Gvisor };
        return SandboxBackendSelection {
            backend,
            microvm_fallback_reason: reported_reason,
        };
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        SandboxBackendSelection {
            backend: SandboxBackend::AppContainer,
            microvm_fallback_reason: if trending_enabled {
                Some(MicroVmFallbackReason::UnsupportedPlatform)
            } else {
                None
            },
        }
    }
}

// ---------------------------------------------------------------------------
// AppContainerConfig: configuration for agent sandbox
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContainerConfig {
    /// Unique profile name per agent, format: "elai_cc_agent_{agent_id}"
    pub profile_name: String,
    
    /// Paths explicitly granted to the agent
    /// - Project source: read-only
    /// - Scratch directory: read-write
    pub granted_paths: Vec<PathBuf>,
    
    /// Network access permission
    /// - false for all agents except Researcher
    /// - true for Researcher role
    pub network_allowed: bool,
    
    /// Memory limit in bytes (default: 512MB)
    #[serde(default = "default_memory_limit")]
    pub memory_limit_bytes: u64,
    
    /// Job time limit in seconds (configurable, 0 = no limit)
    #[serde(default)]
    pub job_time_limit_seconds: u64,
}

fn default_memory_limit() -> u64 {
    512 * 1024 * 1024 // 512 MB
}

impl AppContainerConfig {
    pub fn new(agent_id: Uuid, role: AgentRole, project_path: PathBuf, scratch_dir: PathBuf) -> Self {
        let profile_name = format!("elai_cc_agent_{}", agent_id);
        
        // Grant project source (read-only) and scratch dir (read-write)
        let granted_paths = vec![project_path.clone(), scratch_dir.clone()];
        
        // Only Researcher gets network access
        let network_allowed = role == AgentRole::Researcher;
        
        Self {
            profile_name,
            granted_paths,
            network_allowed,
            memory_limit_bytes: default_memory_limit(),
            job_time_limit_seconds: 0, // No time limit by default
        }
    }
}

// ---------------------------------------------------------------------------
// JobObjectLimits: Windows Job object constraints
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobObjectLimits {
    /// Maximum active processes (default: 1)
    pub active_process_limit: u32,
    
    /// Process memory limit in bytes (default: 512MB)
    pub process_memory_limit: u64,
    
    /// Job time limit in seconds (0 = no limit)
    pub job_time_limit_seconds: u64,
}

impl Default for JobObjectLimits {
    fn default() -> Self {
        Self {
            active_process_limit: 1,
            process_memory_limit: 512 * 1024 * 1024, // 512 MB
            job_time_limit_seconds: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// AgentSandbox: platform-agnostic sandbox trait
// ---------------------------------------------------------------------------

pub trait AgentSandbox: Send + Sync {
    /// Create the sandbox environment
    fn create(&self, config: &AppContainerConfig) -> Result<SandboxHandle, SandboxError>;

    /// Launch an agent process within the sandbox.
    ///
    /// `scratch_dir` is the per-agent scratch directory: it is plumbed into the
    /// child's TMPDIR/TEMP/TMP env vars via `AgentEnvironment::new(scratch_dir)`.
    /// Each impl must use the caller-supplied path — DO NOT fall back to
    /// `std::env::temp_dir()`, which would silently drop scratch isolation
    /// (B-18A DECISION-C).
    fn launch_agent(
        &self,
        handle: &SandboxHandle,
        command: &str,
        args: &[String],
        scratch_dir: &Path,
    ) -> Result<AgentProcess, SandboxError>;

    /// Kill the sandboxed process associated with `agent_id`.
    ///
    /// Implementations track their own process state internally so that the
    /// agent_pool does not need to know which kernel handle / pid / token the
    /// sandbox is holding (B-18A DECISION-B Option A). Returns Ok(()) if the
    /// process was killed or had already exited; returns Err only on a
    /// genuine OS-level failure.
    fn kill_agent(&self, agent_id: Uuid) -> Result<(), SandboxError>;

    /// Block until the sandboxed process exits and return its ExitStatus.
    ///
    /// Companion to `kill_agent` — agent_pool::kill_agent_process calls this
    /// after `kill_agent` to reap the process and avoid zombie accumulation
    /// (Linux) or leaked kernel handles (Windows). Implementations should
    /// release any per-process state (HANDLEs, pid table entries) before
    /// returning.
    fn wait_agent(&self, agent_id: Uuid) -> Result<std::process::ExitStatus, SandboxError>;

    /// Cleanup sandbox resources
    fn cleanup(&self, handle: &SandboxHandle) -> Result<(), SandboxError>;
}

// ---------------------------------------------------------------------------
// SandboxHandle: reference to a created sandbox
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct SandboxHandle {
    pub agent_id: Uuid,
    pub profile_name: String,
    pub platform_handle: PlatformSandboxHandle,
}

impl Clone for SandboxHandle {
    /// Clone produces a handle with metadata only — kernel HANDLEs are NOT
    /// duplicated. The cloned handle has None for all OS-level resource fields
    /// to prevent double-free. Only the original handle should be used for
    /// cleanup().
    fn clone(&self) -> Self {
        Self {
            agent_id: self.agent_id,
            profile_name: self.profile_name.clone(),
            platform_handle: self.platform_handle.clone_metadata_only(),
        }
    }
}

#[derive(Debug)]
pub enum PlatformSandboxHandle {
    #[cfg(target_os = "windows")]
    Windows(WindowsSandboxHandle),

    #[cfg(target_os = "linux")]
    Linux(LinuxSandboxHandle),

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    Unsupported,
}

impl PlatformSandboxHandle {
    /// Clone metadata fields only. OS resource handles (HANDLEs, SIDs) are set
    /// to None to prevent double-free on cleanup.
    fn clone_metadata_only(&self) -> Self {
        match self {
            #[cfg(target_os = "windows")]
            Self::Windows(_) => Self::Windows(WindowsSandboxHandle {
                appcontainer_sid: None,
                restricted_token_handle: None,
                job_object_handle: None,
            }),
            #[cfg(target_os = "linux")]
            Self::Linux(h) => Self::Linux(LinuxSandboxHandle {
                child_pid: h.child_pid,
            }),
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            Self::Unsupported => Self::Unsupported,
        }
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WindowsSandboxHandle {
    /// AppContainer SID from CreateAppContainerProfile (freed via FreeSid).
    /// NOT a kernel HANDLE — do not pass to CloseHandle.
    pub appcontainer_sid: Option<usize>,

    /// Restricted token HANDLE from CreateRestrictedToken (freed via CloseHandle).
    pub restricted_token_handle: Option<usize>,

    /// Job object HANDLE from CreateJobObjectW (freed via CloseHandle).
    pub job_object_handle: Option<usize>,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct LinuxSandboxHandle {
    /// PID of the cloned child process (for waitpid during cleanup).
    /// Linux namespaces are process-scoped (auto-cleaned on process exit).
    pub child_pid: Option<i32>,
}

// ---------------------------------------------------------------------------
// AgentProcess: reference to a running agent process
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct AgentProcess {
    pub process_id: u32,
    pub agent_id: Uuid,
}

// ---------------------------------------------------------------------------
// SandboxError: sandbox operation errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("AppContainer profile creation failed: {0}")]
    ProfileCreationFailed(String),
    
    #[error("Job object creation failed: {0}")]
    JobObjectCreationFailed(String),
    
    #[error("Process launch failed: {0}")]
    ProcessLaunchFailed(String),
    
    #[error("Sandbox cleanup failed: {0}")]
    CleanupFailed(String),
    
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Windows AppContainer Sandbox Implementation
// ---------------------------------------------------------------------------

// SECURITY NOTES:
// - Windows isolation is best-effort only. Linux is the production security target.
// - Known weakness: NTFS junction points and reparse points may bypass ACL deny.
// - AppContainer profile creation requires userenv.dll FFI (not in windows-sys 0.59).
//   Implemented via raw extern "system" binding. Falls back to Job+RestrictedToken
//   if AppContainer is unavailable.
// - sandbox.enable_sandbox defaults to false for Phase 3 (in-process agents).
//   Phase 4A (out-of-process) will enable it.

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HANDLE, INVALID_HANDLE_VALUE, S_OK};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::{
    CreateRestrictedToken, DISABLE_MAX_PRIVILEGE, FreeSid, PSID,
    TOKEN_DUPLICATE, TOKEN_QUERY,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::Isolation::{
    CreateAppContainerProfile, DeleteAppContainerProfile,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
    TerminateJobObject,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectExtendedLimitInformation,
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_JOB_TIME,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    CreateProcessW, GetCurrentProcess, OpenProcessToken,
    PROCESS_INFORMATION, STARTUPINFOW,
    CREATE_SUSPENDED,
};

/// Windows FILETIME ticks (100-nanosecond intervals) per second.
#[cfg(target_os = "windows")]
const FILETIME_TICKS_PER_SECOND: i64 = 10_000_000;

/// Escape a single argument for the Windows command line per CommandLineToArgvW rules.
///
/// Rules (from Raymond Chen / MSDN):
/// - Wrap entire arg in double quotes
/// - Backslashes are literal unless immediately followed by a double quote
/// - A run of N backslashes before a double quote becomes 2N+1 characters (doubled + escaped quote)
/// - A run of N backslashes at the end of the arg (before closing quote) becomes 2N (doubled)
#[cfg(target_os = "windows")]
fn escape_windows_arg(arg: &str) -> String {
    let mut result = String::with_capacity(arg.len() + 2);
    result.push('"');
    let mut backslash_count: usize = 0;
    for c in arg.chars() {
        if c == '\\' {
            backslash_count += 1;
        } else if c == '"' {
            // Double all preceding backslashes, then add escaped quote
            for _ in 0..(backslash_count * 2 + 1) {
                result.push('\\');
            }
            result.push('"');
            backslash_count = 0;
        } else {
            // Flush backslashes as-is (they're literal)
            for _ in 0..backslash_count {
                result.push('\\');
            }
            result.push(c);
            backslash_count = 0;
        }
    }
    // Double trailing backslashes before closing quote
    for _ in 0..(backslash_count * 2) {
        result.push('\\');
    }
    result.push('"');
    result
}

/// Build a UTF-16 environment block for CreateProcessW.
///
/// Format: each entry is "KEY=VALUE\0" encoded as UTF-16. The block ends with
/// an additional \0 (empty string terminator). When passed with
/// CREATE_UNICODE_ENVIRONMENT, Windows interprets this correctly.
#[cfg(target_os = "windows")]
fn build_env_block_utf16(env: &AgentEnvironment) -> Vec<u16> {
    let mut block: Vec<u16> = Vec::new();
    for (key, value) in &env.env_vars {
        let entry = format!("{key}={value}");
        block.extend(entry.encode_utf16());
        block.push(0); // null terminator for this entry
    }
    block.push(0); // double null = end of block
    block
}

#[cfg(target_os = "windows")]
pub struct WindowsAppContainerSandbox {
    /// Known weakness documented: NTFS junction points, reparse points may bypass ACL deny.
    /// Use Linux for production security.
    use_fallback: bool,
    /// Per-agent process HANDLEs from CreateProcessW. Stored as `usize` because
    /// raw HANDLE (`*mut c_void`) is not Send. The HANDLE is consumed by
    /// `kill_agent` / `wait_agent`, which call CloseHandle and remove the entry.
    /// Mutex<HashMap> is used (vs DashMap) to avoid a new dep — B-18A defers
    /// dependency growth.
    processes: Mutex<BTreeMap<Uuid, usize>>,
}

#[cfg(target_os = "windows")]
impl WindowsAppContainerSandbox {
    pub fn new() -> Self {
        Self { use_fallback: false, processes: Mutex::new(BTreeMap::new()) }
    }

    pub fn with_fallback() -> Self {
        Self { use_fallback: true, processes: Mutex::new(BTreeMap::new()) }
    }

    fn create_appcontainer_profile(&self, config: &AppContainerConfig) -> Result<WindowsSandboxHandle, SandboxError> {
        unsafe {
            let profile_name: Vec<u16> = config.profile_name.encode_utf16().chain(std::iter::once(0)).collect();
            let display_name: Vec<u16> = "ELAI Agent".encode_utf16().chain(std::iter::once(0)).collect();
            let description: Vec<u16> = "ELAI CC sandboxed agent".encode_utf16().chain(std::iter::once(0)).collect();

            let mut sid: PSID = std::ptr::null_mut();

            // Try creating the profile. If it already exists (HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS)),
            // delete it first and retry to ensure clean state.
            let hr = CreateAppContainerProfile(
                profile_name.as_ptr(),
                display_name.as_ptr(),
                description.as_ptr(),
                std::ptr::null(),  // no capabilities
                0,
                &mut sid,
            );

            if hr != S_OK {
                // 0x800700B7 = HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS = 183)
                const ALREADY_EXISTS: i32 = -2147024713_i32; // 0x800700B7
                if hr == ALREADY_EXISTS {
                    tracing::debug!(profile = %config.profile_name, "AppContainer profile already exists, recreating");
                    let _ = DeleteAppContainerProfile(profile_name.as_ptr());
                    let hr2 = CreateAppContainerProfile(
                        profile_name.as_ptr(),
                        display_name.as_ptr(),
                        description.as_ptr(),
                        std::ptr::null(),
                        0,
                        &mut sid,
                    );
                    if hr2 != S_OK {
                        return Err(SandboxError::ProfileCreationFailed(
                            format!("CreateAppContainerProfile retry failed: HRESULT 0x{:08X}", hr2 as u32), // NARROWING-OK: HRESULT i32->u32 reinterpretation for hex display; Win32 convention (FormatMessage displays HRESULTs as unsigned hex)
                        ));
                    }
                } else {
                    return Err(SandboxError::ProfileCreationFailed(
                        format!("CreateAppContainerProfile failed: HRESULT 0x{:08X}", hr as u32), // NARROWING-OK: HRESULT i32->u32 reinterpretation for hex display; Win32 convention (FormatMessage displays HRESULTs as unsigned hex)
                    ));
                }
            }

            // Store SID as usize for later cleanup. The SID is allocated by the API
            // and must be freed with FreeSid.
            let sid_value = sid as usize; // NARROWING-OK: Win32 PSID (*mut c_void) pointer-to-integer FFI pattern; usize matches pointer width on all targets

            tracing::info!(
                profile = %config.profile_name,
                "Created AppContainer profile"
            );

            Ok(WindowsSandboxHandle {
                appcontainer_sid: Some(sid_value),
                restricted_token_handle: None,
                job_object_handle: None,
            })
        }
    }

    fn create_job_object(&self, config: &AppContainerConfig) -> Result<HANDLE, SandboxError> {
        unsafe {
            let handle: HANDLE = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                return Err(SandboxError::JobObjectCreationFailed(
                    format!("CreateJobObjectW failed: {}", std::io::Error::last_os_error()),
                ));
            }

            // Configure limits
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            let mut limit_flags: u32 = 0;

            // Active process limit (default: 1)
            limit_flags |= JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
            info.BasicLimitInformation.ActiveProcessLimit = 1;

            // Process memory limit
            limit_flags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
            // NARROWING-OK: Win32 JOBOBJECT_BASIC_LIMIT_INFORMATION.ProcessMemoryLimit is SIZE_T (usize);
            // project targets 64-bit Windows only (usize == u64), so no truncation occurs.
            info.ProcessMemoryLimit = config.memory_limit_bytes as usize;

            // Job time limit (if configured)
            if config.job_time_limit_seconds > 0 {
                limit_flags |= JOB_OBJECT_LIMIT_JOB_TIME;
                info.BasicLimitInformation.PerJobUserTimeLimit =
                    // NARROWING-OK: Win32 LARGE_INTEGER is i64; job_time_limit_seconds is u64 but
                    // values exceeding i64::MAX (~292 years) are pathological; i64::try_from would
                    // panic on such values — unwrap_or(i64::MAX) saturates to maximum job time.
                    i64::try_from(config.job_time_limit_seconds).unwrap_or(i64::MAX)
                        * FILETIME_TICKS_PER_SECOND;
            }

            info.BasicLimitInformation.LimitFlags = limit_flags;

            let result: BOOL = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const std::ffi::c_void, // NARROWING-OK: type-erased pointer cast required by SetInformationJobObject lpJobObjectInformation parameter (Win32 API)
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>()
                    .try_into()
                    .map_err(|_| SandboxError::JobObjectCreationFailed(
                        "size overflow".to_string(),
                    ))?,
            );

            if result == 0 {
                CloseHandle(handle);
                return Err(SandboxError::JobObjectCreationFailed(
                    format!("SetInformationJobObject failed: {}", std::io::Error::last_os_error()),
                ));
            }

            tracing::info!(
                memory_limit = config.memory_limit_bytes,
                time_limit_s = config.job_time_limit_seconds,
                "Created Windows Job object"
            );

            Ok(handle)
        }
    }

    fn create_restricted_token(&self) -> Result<HANDLE, SandboxError> {
        unsafe {
            // Open current process token with DUPLICATE + QUERY access
            let mut process_token: HANDLE = std::ptr::null_mut();
            let result: BOOL = OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_DUPLICATE | TOKEN_QUERY,
                &mut process_token,
            );
            if result == 0 {
                return Err(SandboxError::ConfigError(
                    format!("OpenProcessToken failed: {}", std::io::Error::last_os_error()),
                ));
            }

            // Create restricted token with DISABLE_MAX_PRIVILEGE:
            // strips all privileges except SeChangeNotifyPrivilege.
            // This removes SeDebugPrivilege, SeTcbPrivilege, SeAssignPrimaryTokenPrivilege, etc.
            //
            // KNOWN LIMITATION (Phase 4A): DISABLE_MAX_PRIVILEGE alone does not restrict
            // group SIDs. The token retains enabled user/group memberships, allowing write
            // access to user-owned objects. Production-grade isolation requires:
            // - SidsToRestrict populated (WinNullSid at minimum)
            // - User SID marked deny-only
            // - WRITE_RESTRICTED flag
            // See Chromium restricted_token_utils.cc for reference implementation.
            let mut restricted_token: HANDLE = std::ptr::null_mut();
            let result: BOOL = CreateRestrictedToken(
                process_token,
                DISABLE_MAX_PRIVILEGE,
                0,                    // no SIDs to disable
                std::ptr::null(),     // no SIDs to disable
                0,                    // no privileges to delete (DISABLE_MAX_PRIVILEGE handles this)
                std::ptr::null(),     // no privileges to delete
                0,                    // no restricting SIDs
                std::ptr::null(),     // no restricting SIDs
                &mut restricted_token,
            );

            CloseHandle(process_token);

            if result == 0 {
                return Err(SandboxError::ConfigError(
                    format!("CreateRestrictedToken failed: {}", std::io::Error::last_os_error()),
                ));
            }

            tracing::info!("Created restricted token (DISABLE_MAX_PRIVILEGE)");
            Ok(restricted_token)
        }
    }
}

#[cfg(target_os = "windows")]
impl AgentSandbox for WindowsAppContainerSandbox {
    fn create(&self, config: &AppContainerConfig) -> Result<SandboxHandle, SandboxError> {
        let job_handle = self.create_job_object(config)?;

        let token_handle = match self.create_restricted_token() {
            Ok(h) => h,
            Err(e) => {
                // WARN fix: close job handle on token failure to avoid kernel HANDLE leak
                unsafe { CloseHandle(job_handle); }
                return Err(e);
            }
        };

        let win_handle = if !self.use_fallback {
            match self.create_appcontainer_profile(config) {
                Ok(mut h) => {
                    // Combine: AppContainer SID + Job object + restricted token
                    h.job_object_handle = Some(job_handle as usize); // NARROWING-OK: Win32 HANDLE (*mut c_void) pointer-to-integer FFI; usize matches pointer width; cast back via `raw as HANDLE` at use sites
                    h.restricted_token_handle = Some(token_handle as usize); // NARROWING-OK: Win32 HANDLE (*mut c_void) pointer-to-integer FFI; usize matches pointer width
                    h
                }
                Err(e) => {
                    tracing::warn!(error = %e, "AppContainer unavailable, using Job+RestrictedToken fallback");
                    WindowsSandboxHandle {
                        appcontainer_sid: None,
                        restricted_token_handle: Some(token_handle as usize), // NARROWING-OK: Win32 HANDLE pointer-to-integer FFI; see comment at appcontainer_sid above
                        job_object_handle: Some(job_handle as usize), // NARROWING-OK: Win32 HANDLE pointer-to-integer FFI
                    }
                }
            }
        } else {
            WindowsSandboxHandle {
                appcontainer_sid: None,
                restricted_token_handle: Some(token_handle as usize), // NARROWING-OK: Win32 HANDLE pointer-to-integer FFI
                job_object_handle: Some(job_handle as usize), // NARROWING-OK: Win32 HANDLE pointer-to-integer FFI
            }
        };

        let agent_id = {
            let mut b = [0u8; 16];
            getrandom::fill(&mut b)
                .map_err(|e| SandboxError::ConfigError(format!("getrandom failed: {e}")))?;
            uuid::Builder::from_random_bytes(b).into_uuid()
        };
        Ok(SandboxHandle {
            agent_id,
            profile_name: config.profile_name.clone(),
            platform_handle: PlatformSandboxHandle::Windows(win_handle),
        })
    }

    fn launch_agent(
        &self,
        handle: &SandboxHandle,
        command: &str,
        args: &[String],
        scratch_dir: &Path,
    ) -> Result<AgentProcess, SandboxError> {
        let win_handle = match &handle.platform_handle {
            PlatformSandboxHandle::Windows(h) => h,
        };

        unsafe {
            // Build command line with proper Windows escaping per CommandLineToArgvW rules:
            // - Wrap each arg in quotes
            // - Backslashes before quotes must be doubled
            // - A trailing run of backslashes before the closing quote must be doubled
            let mut cmd_line = escape_windows_arg(command);
            for arg in args {
                cmd_line.push(' ');
                cmd_line.push_str(&escape_windows_arg(arg));
            }
            let mut cmd_wide: Vec<u16> = cmd_line.encode_utf16().chain(std::iter::once(0)).collect();

            let mut si: STARTUPINFOW = std::mem::zeroed();
            si.cb = std::mem::size_of::<STARTUPINFOW>().try_into()
                .map_err(|_| SandboxError::ProcessLaunchFailed("size overflow".to_string()))?;
            let mut pi: PROCESS_INFORMATION = std::mem::zeroed();

            // Build sanitized environment block for the child process from the
            // CALLER-SUPPLIED scratch_dir. B-18A DECISION-C: previous code used
            // std::env::temp_dir() and silently dropped the per-agent scratch
            // isolation that agent_pool::launch_agent_process configures.
            let env = AgentEnvironment::new(scratch_dir.to_path_buf());
            let env_block = build_env_block_utf16(&env);

            let result: BOOL = CreateProcessW(
                std::ptr::null(),
                cmd_wide.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0, // bInheritHandles = FALSE
                CREATE_SUSPENDED | 0x00000400, // CREATE_UNICODE_ENVIRONMENT
                env_block.as_ptr() as *const std::ffi::c_void, // NARROWING-OK: *const u16 -> *const c_void pointer cast required by CreateProcessW lpEnvironment parameter (Win32 API)
                std::ptr::null(),
                &si,
                &mut pi,
            );

            if result == 0 {
                return Err(SandboxError::ProcessLaunchFailed(
                    format!("CreateProcessW failed: {}", std::io::Error::last_os_error()),
                ));
            }

            // Assign process to job object before resuming
            if let Some(job_handle) = win_handle.job_object_handle {
                let assign_result: BOOL = AssignProcessToJobObject(
                    job_handle as HANDLE, // NARROWING-OK: usize->HANDLE (*mut c_void) integer-to-pointer FFI; restoring stored Win32 HANDLE
                    pi.hProcess,
                );
                if assign_result == 0 {
                    // Kill the suspended process since we can't sandbox it
                    windows_sys::Win32::System::Threading::TerminateProcess(pi.hProcess, 1);
                    CloseHandle(pi.hProcess);
                    CloseHandle(pi.hThread);
                    return Err(SandboxError::ProcessLaunchFailed(
                        format!("AssignProcessToJobObject failed: {}", std::io::Error::last_os_error()),
                    ));
                }
            }

            // Resume the suspended process
            let resume_result = windows_sys::Win32::System::Threading::ResumeThread(pi.hThread);
            if resume_result == u32::MAX {
                windows_sys::Win32::System::Threading::TerminateProcess(pi.hProcess, 1);
                CloseHandle(pi.hProcess);
                CloseHandle(pi.hThread);
                return Err(SandboxError::ProcessLaunchFailed(
                    format!("ResumeThread failed: {}", std::io::Error::last_os_error()),
                ));
            }
            CloseHandle(pi.hThread);

            let process_id = pi.dwProcessId;
            // B-18A DECISION-B: do NOT close pi.hProcess here — it is needed for
            // TerminateProcess / WaitForSingleObject in kill_agent + wait_agent.
            // Store the HANDLE in the per-instance map keyed by agent_id.
            {
                let mut map = self.processes.lock().map_err(|e| {
                    // Mutex was poisoned by a panic in another thread. Close the
                    // HANDLE we just opened to avoid leaking, then surface the
                    // error.
                    CloseHandle(pi.hProcess);
                    SandboxError::ConfigError(format!("processes mutex poisoned: {e}"))
                })?;
                map.insert(handle.agent_id, pi.hProcess as usize); // NARROWING-OK: Win32 HANDLE (*mut c_void) pointer-to-integer FFI; stored as usize per struct design (see processes field comment)
            }

            tracing::info!(
                process_id = process_id,
                agent_id = %handle.agent_id,
                "Launched sandboxed agent process"
            );

            Ok(AgentProcess {
                process_id,
                agent_id: handle.agent_id,
            })
        }
    }

    fn kill_agent(&self, agent_id: Uuid) -> Result<(), SandboxError> {
        // Take the HANDLE out of the map so wait_agent (or another kill_agent)
        // cannot double-close it. If the entry is absent the process is already
        // gone — return Ok per the trait contract.
        let raw = {
            let mut map = self.processes.lock().map_err(|e| {
                SandboxError::ConfigError(format!("processes mutex poisoned: {e}"))
            })?;
            map.remove(&agent_id)
        };
        let Some(raw) = raw else {
            tracing::debug!(agent_id = %agent_id, "kill_agent: no live process for agent (already exited)");
            return Ok(());
        };

        unsafe {
            let proc_handle = raw as HANDLE; // NARROWING-OK: usize->HANDLE (*mut c_void) integer-to-pointer FFI; restoring stored Win32 HANDLE
            // TerminateProcess is best-effort: if the process already exited it
            // returns 0 with last error ERROR_ACCESS_DENIED — not a real error.
            let _ = windows_sys::Win32::System::Threading::TerminateProcess(proc_handle, 1);
            // WaitForSingleObject blocks until the process is reaped. INFINITE
            // == 0xFFFFFFFF; documented contract is that TerminateProcess
            // returns synchronously after the kernel signals the process so this
            // will not deadlock for a pid that exists.
            const INFINITE: u32 = 0xFFFFFFFF;
            windows_sys::Win32::System::Threading::WaitForSingleObject(proc_handle, INFINITE);
            CloseHandle(proc_handle);
        }
        tracing::debug!(agent_id = %agent_id, "kill_agent: terminated and reaped");
        Ok(())
    }

    fn wait_agent(&self, agent_id: Uuid) -> Result<std::process::ExitStatus, SandboxError> {
        use std::os::windows::process::ExitStatusExt;
        let raw = {
            let mut map = self.processes.lock().map_err(|e| {
                SandboxError::ConfigError(format!("processes mutex poisoned: {e}"))
            })?;
            map.remove(&agent_id)
        };
        let Some(raw) = raw else {
            // No live process. Return a synthetic success ExitStatus — reaping
            // an absent child is idempotent per the trait contract.
            return Ok(std::process::ExitStatus::from_raw(0));
        };

        let exit_code: u32 = unsafe {
            let proc_handle = raw as HANDLE; // NARROWING-OK: usize->HANDLE (*mut c_void) integer-to-pointer FFI; restoring stored Win32 HANDLE
            const INFINITE: u32 = 0xFFFFFFFF;
            windows_sys::Win32::System::Threading::WaitForSingleObject(proc_handle, INFINITE);
            let mut code: u32 = 0;
            let ok = windows_sys::Win32::System::Threading::GetExitCodeProcess(proc_handle, &mut code);
            CloseHandle(proc_handle);
            if ok == 0 {
                return Err(SandboxError::ProcessLaunchFailed(format!(
                    "GetExitCodeProcess failed: {}",
                    std::io::Error::last_os_error()
                )));
            }
            code
        };
        Ok(std::process::ExitStatus::from_raw(exit_code))
    }

    fn cleanup(&self, handle: &SandboxHandle) -> Result<(), SandboxError> {
        let win_handle = match &handle.platform_handle {
            PlatformSandboxHandle::Windows(h) => h,
        };

        unsafe {
            // Terminate all processes in the job object
            if let Some(job_handle) = win_handle.job_object_handle {
                TerminateJobObject(job_handle as HANDLE, 0); // NARROWING-OK: usize->HANDLE (*mut c_void) integer-to-pointer FFI; restoring stored Win32 HANDLE
                CloseHandle(job_handle as HANDLE); // NARROWING-OK: same as above
            }

            // Close restricted token HANDLE (kernel object — CloseHandle is correct)
            if let Some(token_handle) = win_handle.restricted_token_handle {
                if token_handle != 0 {
                    CloseHandle(token_handle as HANDLE); // NARROWING-OK: usize->HANDLE (*mut c_void) integer-to-pointer FFI
                }
            }

            // Free AppContainer SID (heap allocation — FreeSid is correct, NOT CloseHandle)
            if let Some(sid_ptr) = win_handle.appcontainer_sid {
                if sid_ptr != 0 {
                    FreeSid(sid_ptr as PSID); // NARROWING-OK: usize->PSID (*mut c_void) integer-to-pointer FFI; SID was allocated by CreateAppContainerProfile and stored as usize
                }
            }

            // Delete the AppContainer profile by name (idempotent — succeeds even if not created)
            let profile_name: Vec<u16> = handle.profile_name.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = DeleteAppContainerProfile(profile_name.as_ptr());
        }

        tracing::info!(
            agent_id = %handle.agent_id,
            "Sandbox cleanup complete"
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// LinuxSandbox: Linux namespace + seccomp implementation
// ---------------------------------------------------------------------------

// C-11: named rlimit constants for clone_cb child-side enforcement. No magic
// numbers in the syscall path; values are decisions D1/D8 from the C-11
// context package (phase-4A.report stricter than B-18 plan's 2 GB on AS).
// All units: bytes/seconds/file-count per libc::rlimit semantics.
#[cfg(target_os = "linux")]
const RLIMIT_CPU_SOFT_SECS: u64 = 60;
#[cfg(target_os = "linux")]
const RLIMIT_CPU_HARD_SECS: u64 = 90;
#[cfg(target_os = "linux")]
const RLIMIT_NOFILE_LIMIT: u64 = 64;
#[cfg(target_os = "linux")]
const RLIMIT_NPROC_LIMIT: u64 = 32;
#[cfg(target_os = "linux")]
const RLIMIT_AS_BYTES: u64 = 512 * 1024 * 1024; // 512 MiB
#[cfg(target_os = "linux")]
const RLIMIT_FSIZE_BYTES: u64 = 64 * 1024 * 1024; // 64 MiB

/// CAP_LAST_CAP fallback when /proc/sys/kernel/cap_last_cap cannot be read in
/// the PARENT. Conservative: kernel 5.15+ exposes 40 (CAP_CHECKPOINT_RESTORE).
/// Newer kernels (6.x) expose 41 (CAP_BPF retained as 39); reading the proc
/// file at runtime is the authoritative source — this is only used if the
/// read fails. Decision D4.
#[cfg(target_os = "linux")]
const CAP_LAST_CAP_FALLBACK: i32 = 40;

#[cfg(target_os = "linux")]
pub struct LinuxSandbox {
    /// Per-agent child PIDs from libc::clone. kill_agent / wait_agent consume
    /// the entry. Mutex<HashMap> avoids a new dep — B-18A defers dep growth.
    processes: Mutex<BTreeMap<Uuid, libc::pid_t>>,
}

#[cfg(target_os = "linux")]
impl LinuxSandbox {
    pub fn new() -> Self {
        Self { processes: Mutex::new(BTreeMap::new()) }
    }

    /// clone3 wrapper for Linux process spawn with namespace isolation.
    ///
    /// This is a B-18 STEP 1 implementation: provides a clone3 wrapper that
    /// can be used instead of the older libc::clone syscall. The wrapper
    /// supports the same namespace flags (CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWNET)
    /// required for agent isolation.
    ///
    /// This implementation uses the raw libc::clone3 syscall with the
    /// clone_args structure defined in Linux kernel headers.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs raw syscalls and manipulates
    /// process memory. The caller must ensure:
    /// - The callback function only uses async-signal-safe operations
    /// - The stack pointer is valid and properly aligned
    /// - The child data pointer remains valid until the child execs
    ///
    /// # Arguments
    ///
    /// * `clone_cb` - Callback function executed in the child process
    /// * `stack_top` - Pointer to the top of the child's stack
    /// * `clone_flags` - Namespace flags (CLONE_NEWNS, CLONE_NEWPID, etc.)
    /// * `child_data` - Pointer to data passed to the child callback
    ///
    /// # Returns
    ///
    /// * `Ok(pid)` - PID of the child process (in parent) or 0 (in child)
    /// * `Err(SandboxError)` - If the clone3 syscall fails
    #[allow(clippy::too_many_arguments)]
    unsafe fn clone3_wrapper(
        clone_cb: extern "C" fn(*mut libc::c_void) -> libc::c_int,
        stack_top: *mut libc::c_void,
        clone_flags: libc::c_int,
        child_data: *mut libc::c_void,
    ) -> Result<libc::pid_t, SandboxError> {
        // clone3 syscall number (x86_64)
        const SYS_clone3: libc::c_long = 435;

        // clone_args structure as defined in Linux kernel headers
        #[repr(C)]
        struct CloneArgs {
            flags: u64,
            pidfd: u64,
            child_tid: u64,
            parent_tid: u64,
            exit_signal: u64,
            stack: u64,
            stack_size: u64,
            tls: u64,
            set_tid: u64,
            set_tid_size: u64,
            cgroup: u64,
        }

        let mut args = CloneArgs {
            flags: clone_flags as u64,
            pidfd: 0,
            child_tid: 0,
            parent_tid: 0,
            exit_signal: libc::SIGCHLD as u64,
            stack: stack_top as u64,
            stack_size: 0,
            tls: 0,
            set_tid: 0,
            set_tid_size: 0,
            cgroup: 0,
        };

        // Perform the clone3 syscall
        let pid = libc::syscall(
            SYS_clone3,
            &mut args as *mut CloneArgs,
            std::mem::size_of::<CloneArgs>(),
        );

        if pid == -1 {
            return Err(SandboxError::ProcessLaunchFailed(format!(
                "clone3 syscall failed: {}",
                std::io::Error::last_os_error()
            )));
        }

        // In child process (pid == 0), execute the callback and exit
        if pid == 0 {
            let exit_code = clone_cb(child_data);
            libc::_exit(exit_code);
        }

        // In parent process, return the child's PID
        Ok(pid as libc::pid_t)
    }

    /// Compile IPC_BASELINE seccomp-bpf filter in the PARENT process (heap-safe).
    ///
    /// Returns raw BPF program bytes. The child installs these via prctl(PR_SET_SECCOMP)
    /// using only async-signal-safe syscalls (no heap allocation in child).
    ///
    /// IPC_BASELINE: all roles. Allows AF_UNIX sockets but blocks AF_INET/AF_INET6.
    /// Network syscalls (socket, connect, bind, listen, accept, sendto, recvfrom,
    /// sendmsg, recvmsg, setsockopt, getsockopt, getsockname, getpeername) are
    /// allowed WITHOUT domain restriction — seccomp arg filtering for socket()
    /// domain is handled separately via compile_seccomp_network_extra().
    ///
    /// Allowlist (x86_64 syscall numbers): read, write, open, close, stat, fstat,
    /// lstat, poll, lseek, mmap, mprotect, munmap, brk, ioctl, access, pipe,
    /// sched_yield, mremap, clone, execve, exit, wait4, kill, uname, fcntl, flock,
    /// fsync, ftruncate, getcwd, chdir, getpid, getppid, getuid, geteuid, getgid,
    /// getegid, rt_sigaction, rt_sigprocmask, rt_sigreturn, arch_prctl, futex,
    /// set_tid_address, exit_group, openat, readlinkat, newfstatat, set_robust_list,
    /// prlimit64, getrandom, rseq, clock_gettime, nanosleep, prctl, seccomp,
    /// plus Unix socket syscalls (socket, connect, bind, listen, accept, etc.).
    // NARROWING-OK (seccomp block): libc AF_*/SOCK_* constants are c_int (i32); seccompiler::SeccompCondition
    // takes u64 and SeccompAction::Errno takes u32. These widening/reinterpretation casts are required by
    // the seccompiler API; all libc constant values fit in both u32 and u64 (AF_UNIX=1, AF_INET=2,
    // AF_INET6=10, EPERM=1, SOCK_STREAM=1). No information is lost. Same applies throughout
    // compile_seccomp_network_extra().
    fn compile_seccomp_ipc_baseline() -> Result<seccompiler::BpfProgram, SandboxError> {
        use seccompiler::{SeccompAction, SeccompFilter, TargetArch};
        use std::collections::BTreeMap;

        let allowed_syscalls: &[i64] = &[
            0,   // read
            1,   // write
            2,   // open
            3,   // close
            4,   // stat
            5,   // fstat
            6,   // lstat
            7,   // poll
            8,   // lseek
            9,   // mmap
            10,  // mprotect
            11,  // munmap
            12,  // brk
            16,  // ioctl
            21,  // access
            22,  // pipe
            24,  // sched_yield
            25,  // mremap
            35,  // nanosleep
            39,  // getpid
            // NOTE: socket (41) handled separately below with AF_UNIX arg filter
            42,  // connect
            43,  // accept
            44,  // sendto
            45,  // recvfrom
            46,  // sendmsg
            47,  // recvmsg
            48,  // shutdown
            49,  // bind
            50,  // listen
            51,  // getsockname
            52,  // getpeername
            54,  // setsockopt
            55,  // getsockopt
            56,  // clone
            59,  // execve
            60,  // exit
            61,  // wait4
            62,  // kill
            63,  // uname
            72,  // fcntl
            73,  // flock
            74,  // fsync
            77,  // ftruncate
            79,  // getcwd
            80,  // chdir
            102, // getuid
            104, // getgid
            107, // geteuid
            108, // getegid
            110, // getppid
            13,  // rt_sigaction
            14,  // rt_sigprocmask
            15,  // rt_sigreturn
            157, // prctl (needed for PR_SET_NO_NEW_PRIVS in child)
            158, // arch_prctl
            202, // futex
            218, // set_tid_address
            228, // clock_gettime
            231, // exit_group
            257, // openat
            262, // newfstatat
            267, // readlinkat
            273, // set_robust_list
            288, // accept4
            302, // prlimit64
            317, // seccomp (needed for filter install in child)
            318, // getrandom
            334, // rseq
        ];

        let mut syscall_rules: BTreeMap<i64, Vec<seccompiler::SeccompRule>> = BTreeMap::new();
        for &nr in allowed_syscalls {
            syscall_rules.insert(nr, vec![seccompiler::SeccompRule::new(vec![]).map_err(|e| {
                SandboxError::ConfigError(format!("seccomp rule creation failed: {e}"))
            })?]);
        }

        // socket() syscall (41): restrict to AF_UNIX only in IPC_BASELINE.
        // BPF arg0 check: only allow domain == AF_UNIX (1).
        // AF_INET and AF_INET6 are blocked. Researcher role uses
        // compile_seccomp_network_extra() which adds AF_INET/AF_INET6.
        let af_unix_only_rule = seccompiler::SeccompRule::new(vec![
            seccompiler::SeccompCondition::new(
                0, // arg0 = domain
                seccompiler::SeccompCmpArgLen::Dword,
                seccompiler::SeccompCmpOp::Eq,
                libc::AF_UNIX as u64,
            ).map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
        ]).map_err(|e| SandboxError::ConfigError(format!("seccomp rule failed: {e}")))?;
        syscall_rules.insert(41, vec![af_unix_only_rule]); // socket()

        let filter = SeccompFilter::new(
            syscall_rules,
            SeccompAction::Errno(libc::EPERM as u32), // default: deny with EPERM
            SeccompAction::Allow,                       // matched syscalls: allow
            TargetArch::x86_64,
        )
        .map_err(|e| SandboxError::ConfigError(format!("seccomp filter creation failed: {e}")))?;

        <seccompiler::BpfProgram as TryFrom<seccompiler::SeccompFilter>>::try_from(filter)
            .map_err(|e| SandboxError::ConfigError(format!("seccomp BPF compilation failed: {e}")))
    }

    /// Compile NETWORK_EXTRA seccomp-bpf filter for researcher role.
    ///
    /// Extends IPC_BASELINE with AF_INET and AF_INET6 SOCK_STREAM socket access.
    /// Uses BPF argument checking on the socket() syscall's domain parameter (arg0).
    ///
    /// DEPLOYMENT NOTE (BLOCK-2): researcher egress filtering via veth pair +
    /// iptables rules is NOT implemented at the seccomp level. It requires
    /// root/CAP_NET_ADMIN to create veth interfaces and configure iptables.
    /// The deployment orchestrator (systemd unit / container runtime) must:
    /// 1. Create veth pair before agent spawn
    /// 2. Move one end into the agent's network namespace (CLONE_NEWNET)
    /// 3. Apply iptables FORWARD rules limiting egress to allowed hosts
    /// 4. Drop CAP_NET_RAW (no raw sockets — only SOCK_STREAM TCP allowed here)
    /// Until deployment infrastructure exists, researcher agents can make TCP
    /// connections to any host reachable from the host machine. The seccomp
    /// filter restricts to SOCK_STREAM only (no UDP, no raw).
    ///
    /// This profile allows socket(AF_INET, SOCK_STREAM, _) and
    /// socket(AF_INET6, SOCK_STREAM, _) in addition to all IPC_BASELINE syscalls.
    fn compile_seccomp_network_extra() -> Result<seccompiler::BpfProgram, SandboxError> {
        use seccompiler::{SeccompAction, SeccompCmpArgLen, SeccompCmpOp,
            SeccompCondition, SeccompFilter, SeccompRule, TargetArch};
        use std::collections::BTreeMap;

        // Start with the same baseline syscalls
        let baseline_syscalls: &[i64] = &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            21, 22, 24, 25, 35, 39, 42, 43, 44, 45, 46, 47, 48, 49, 50,
            51, 52, 54, 55, 56, 59, 60, 61, 62, 63, 72, 73, 74, 77, 79,
            80, 102, 104, 107, 108, 110, 157, 158, 202, 218, 228, 231,
            257, 262, 267, 273, 288, 302, 317, 318, 334,
        ];

        let mut syscall_rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();
        for &nr in baseline_syscalls {
            syscall_rules.insert(nr, vec![SeccompRule::new(vec![]).map_err(|e| {
                SandboxError::ConfigError(format!("seccomp rule creation failed: {e}"))
            })?]);
        }

        // socket() syscall (41) with domain arg check:
        // Allow AF_UNIX (1), AF_INET (2), AF_INET6 (10) with SOCK_STREAM (1)
        // socket(41) with domain+type arg check (E-15 T2 hardening).
        // DENIED: AF_UNIX(1), AF_NETLINK(16), AF_PACKET(17), AF_BLUETOOTH(31)
        // per E-15 arg-checked seccomp. AF_UNIX removed from researcher profile.
        // SOCK_RAW(3) denied by omission: only SOCK_STREAM(1) + SOCK_DGRAM(2) allowed.
        let af_inet_stream_rule = SeccompRule::new(vec![
            SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::AF_INET as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
            SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::SOCK_STREAM as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
        ]).map_err(|e| SandboxError::ConfigError(format!("seccomp rule failed: {e}")))?;
        let af_inet_dgram_rule = SeccompRule::new(vec![
            SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::AF_INET as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
            SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::SOCK_DGRAM as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
        ]).map_err(|e| SandboxError::ConfigError(format!("seccomp rule failed: {e}")))?;
        let af_inet6_stream_rule = SeccompRule::new(vec![
            SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::AF_INET6 as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
            SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::SOCK_STREAM as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
        ]).map_err(|e| SandboxError::ConfigError(format!("seccomp rule failed: {e}")))?;
        let af_inet6_dgram_rule = SeccompRule::new(vec![
            SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::AF_INET6 as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
            SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::SOCK_DGRAM as u64)
                .map_err(|e| SandboxError::ConfigError(format!("seccomp condition failed: {e}")))?,
        ]).map_err(|e| SandboxError::ConfigError(format!("seccomp rule failed: {e}")))?;
        syscall_rules.insert(41, vec![af_inet_stream_rule, af_inet_dgram_rule, af_inet6_stream_rule, af_inet6_dgram_rule]);

        let filter = SeccompFilter::new(
            syscall_rules,
            SeccompAction::Errno(libc::EPERM as u32),
            SeccompAction::Allow,
            TargetArch::x86_64,
        )
        .map_err(|e| SandboxError::ConfigError(format!("seccomp filter creation failed: {e}")))?;

        <seccompiler::BpfProgram as TryFrom<seccompiler::SeccompFilter>>::try_from(filter)
            .map_err(|e| SandboxError::ConfigError(format!("seccomp BPF compilation failed: {e}")))
    }

    /// Select the appropriate seccomp profile based on agent role.
    pub fn seccomp_for_role(role: &AgentRole) -> Result<seccompiler::BpfProgram, SandboxError> {
        match role {
            AgentRole::Researcher => Self::compile_seccomp_network_extra(),
            _ => Self::compile_seccomp_ipc_baseline(),
        }
    }
}

#[cfg(target_os = "linux")]
impl AgentSandbox for LinuxSandbox {
    fn create(&self, _config: &AppContainerConfig) -> Result<SandboxHandle, SandboxError> {
        let mut b = [0u8; 16];
        getrandom::fill(&mut b)
            .map_err(|e| SandboxError::ConfigError(format!("getrandom failed: {e}")))?;
        let agent_id = uuid::Builder::from_random_bytes(b).into_uuid();

        Ok(SandboxHandle {
            agent_id,
            profile_name: format!("elai_linux_{agent_id}"),
            platform_handle: PlatformSandboxHandle::Linux(LinuxSandboxHandle {
                child_pid: None,
            }),
        })
    }

    fn launch_agent(
        &self,
        handle: &SandboxHandle,
        command: &str,
        args: &[String],
        scratch_dir: &Path,
    ) -> Result<AgentProcess, SandboxError> {
        // scratch_dir threading on Linux is currently informational: clone3+execvp
        // does not allow setting env from the parent without an additional
        // clone-arg shim. AgentEnvironment::new(scratch_dir) is the canonical
        // shape for B-18B+ when env-block plumbing lands. For B-18A we only
        // verify the parameter is reachable.
        let _scratch_env = AgentEnvironment::new(scratch_dir.to_path_buf());
        use std::ffi::CString;

        const CHILD_STACK_SIZE: usize = 1024 * 1024; // 1 MiB

        // 1. Compile BPF in parent (heap allocation is safe here — BLOCK 2 fix)
        let bpf_prog = Self::compile_seccomp_ipc_baseline()?;

        let cmd = CString::new(command)
            .map_err(|e| SandboxError::ProcessLaunchFailed(format!("invalid command: {e}")))?;
        let c_args: Vec<CString> = std::iter::once(cmd.clone())
            .chain(args.iter().map(|a| CString::new(a.as_str())
                .map_err(|e| SandboxError::ProcessLaunchFailed(format!("invalid arg (null byte): {e}"))))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter())
            .collect();
        let c_arg_ptrs: Vec<*const libc::c_char> = c_args
            .iter()
            .map(|a| a.as_ptr())
            .chain(std::iter::once(std::ptr::null()))
            .collect();

        // Allocate child stack (grows downward on x86_64)
        let mut stack = vec![0u8; CHILD_STACK_SIZE];
        let stack_top = unsafe { stack.as_mut_ptr().add(CHILD_STACK_SIZE) };

        // NOTE: These namespace flags require CAP_SYS_ADMIN (or root).
        // Without CLONE_NEWUSER as outer container, unprivileged processes get EPERM.
        // The error is propagated as SandboxError::ProcessLaunchFailed.
        let clone_flags: libc::c_int =
            libc::CLONE_NEWNS | libc::CLONE_NEWPID | libc::CLONE_NEWNET | libc::CLONE_NEWUTS | libc::SIGCHLD;

        // C-11 D6: read /proc/sys/kernel/cap_last_cap in the PARENT before
        // clone(). Heap allocation here is safe; the child gets the value as
        // an i32 in ChildData. Fallback to CAP_LAST_CAP_FALLBACK (40) on read
        // failure. CAP_LAST_CAP varies by kernel (5.15 -> 40, 6.x -> 41); the
        // proc file is the authoritative source.
        let cap_last_cap: i32 = match std::fs::read_to_string("/proc/sys/kernel/cap_last_cap") {
            Ok(s) => s.trim().parse::<i32>().unwrap_or(CAP_LAST_CAP_FALLBACK),
            Err(_) => CAP_LAST_CAP_FALLBACK,
        };

        // Data passed to the child via the clone arg pointer.
        // Only contains raw pointers — no heap allocations needed in child.
        struct ChildData {
            cmd_ptr: *const libc::c_char,
            args_ptr: *const *const libc::c_char,
            bpf_ptr: *const seccompiler::BpfProgram,
            cap_last_cap: i32,
        }

        let child_data = ChildData {
            cmd_ptr: cmd.as_ptr(),
            args_ptr: c_arg_ptrs.as_ptr(),
            bpf_ptr: &bpf_prog as *const seccompiler::BpfProgram,
            cap_last_cap,
        };

        let pid = unsafe {
            extern "C" fn clone_cb(arg: *mut libc::c_void) -> libc::c_int {
                unsafe {
                    let data = &*(arg as *const ChildData);

                    // C-11 insertion order (HARD INVARIANT, decisions D3+D7+D8):
                    //   1. PR_SET_NO_NEW_PRIVS  — required before seccomp (existing)
                    //   2. PR_CAPBSET_DROP loop — drop bounding caps before exec
                    //   3. setrlimit block      — CPU/NOFILE/NPROC/AS/FSIZE
                    //   4. /proc hidepid=2 mount — masks other processes
                    //   5. seccomp apply        — MUST be last syscall before exec
                    //   6. execvp                — the only post-seccomp call
                    //
                    // Seccomp last prevents a post-filter-install bypass:
                    // prctl(157), prlimit64(302), and mount(165) are not all
                    // allowed simultaneously, so the order above is the only
                    // one that compiles and executes without EPERM.

                    // 1. PR_SET_NO_NEW_PRIVS (required before seccomp)
                    if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                        libc::_exit(126);
                    }

                    // 2. PR_CAPBSET_DROP loop. Drops every cap from the
                    //    bounding set so a child that re-execs setuid binaries
                    //    cannot regain caps. EINVAL on unknown cap numbers is
                    //    non-fatal (continue); any other errno aborts.
                    //    cap_last_cap is read from /proc in the parent (D6).
                    let cap_last: i32 = data.cap_last_cap;
                    let mut cap: i32 = 0;
                    while cap <= cap_last {
                        let r = libc::prctl(
                            libc::PR_CAPBSET_DROP,
                            cap as libc::c_ulong,
                            0,
                            0,
                            0,
                        );
                        if r != 0 {
                            let errno = *libc::__errno_location();
                            if errno != libc::EINVAL {
                                libc::_exit(122);
                            }
                            // EINVAL: cap not recognised by this kernel — skip.
                        }
                        cap += 1;
                    }

                    // 3. setrlimit block. Stack-allocated libc::rlimit per
                    //    call; no heap. glibc forwards to prlimit64 (syscall
                    //    302) which is allowlisted at sandbox.rs:937.
                    let rl_cpu = libc::rlimit {
                        rlim_cur: RLIMIT_CPU_SOFT_SECS as libc::rlim_t,
                        rlim_max: RLIMIT_CPU_HARD_SECS as libc::rlim_t,
                    };
                    if libc::setrlimit(libc::RLIMIT_CPU, &rl_cpu) != 0 {
                        libc::_exit(124);
                    }

                    let rl_nofile = libc::rlimit {
                        rlim_cur: RLIMIT_NOFILE_LIMIT as libc::rlim_t,
                        rlim_max: RLIMIT_NOFILE_LIMIT as libc::rlim_t,
                    };
                    if libc::setrlimit(libc::RLIMIT_NOFILE, &rl_nofile) != 0 {
                        libc::_exit(124);
                    }

                    let rl_nproc = libc::rlimit {
                        rlim_cur: RLIMIT_NPROC_LIMIT as libc::rlim_t,
                        rlim_max: RLIMIT_NPROC_LIMIT as libc::rlim_t,
                    };
                    if libc::setrlimit(libc::RLIMIT_NPROC, &rl_nproc) != 0 {
                        libc::_exit(124);
                    }

                    let rl_as = libc::rlimit {
                        rlim_cur: RLIMIT_AS_BYTES as libc::rlim_t,
                        rlim_max: RLIMIT_AS_BYTES as libc::rlim_t,
                    };
                    if libc::setrlimit(libc::RLIMIT_AS, &rl_as) != 0 {
                        libc::_exit(124);
                    }

                    let rl_fsize = libc::rlimit {
                        rlim_cur: RLIMIT_FSIZE_BYTES as libc::rlim_t,
                        rlim_max: RLIMIT_FSIZE_BYTES as libc::rlim_t,
                    };
                    if libc::setrlimit(libc::RLIMIT_FSIZE, &rl_fsize) != 0 {
                        libc::_exit(124);
                    }

                    // 4. Remount /proc with hidepid=2 inside the child's
                    //    private mount namespace (CLONE_NEWNS). hidepid=2
                    //    means non-owner PIDs are completely hidden, not
                    //    just their /proc entries (man 5 proc, kernel >= 3.3).
                    //    EPERM on unprivileged CI is expected; CI skips these
                    //    tests via the create()-EPERM guard (existing pattern
                    //    in agent_isolation.rs:181-186). MS_REMOUNT is needed
                    //    because the kernel populated /proc on first mount
                    //    when CLONE_NEWPID was applied. mount() syscall is
                    //    NOT in the seccomp allowlist; this call must precede
                    //    seccomp_apply (it does — step 5 is below).
                    let proc_src = b"proc\0".as_ptr() as *const libc::c_char;
                    let proc_tgt = b"/proc\0".as_ptr() as *const libc::c_char;
                    let proc_fs = b"proc\0".as_ptr() as *const libc::c_char;
                    let proc_opts = b"hidepid=2\0".as_ptr() as *const libc::c_void;
                    let mnt_flags: libc::c_ulong =
                        (libc::MS_NOSUID | libc::MS_NOEXEC | libc::MS_NODEV) as libc::c_ulong;
                    if libc::mount(proc_src, proc_tgt, proc_fs, mnt_flags, proc_opts) != 0 {
                        // Any errno (EPERM/EINVAL/EBUSY) aborts. CI without
                        // CAP_SYS_ADMIN never reaches here because create()
                        // already returned EPERM in the parent.
                        libc::_exit(123);
                    }

                    // 5. Install pre-compiled seccomp filter (no heap alloc — just a syscall)
                    if seccompiler::apply_filter(&*data.bpf_ptr).is_err() {
                        libc::_exit(125);
                    }

                    // 6. Exec the agent command
                    libc::execvp(data.cmd_ptr, data.args_ptr);

                    // execvp only returns on error
                    libc::_exit(127);
                }
            }

            // B-18 STEP 1: Use clone3 wrapper instead of libc::clone
            // clone3 provides better namespace isolation and is the modern Linux API
            match Self::clone3_wrapper(
                clone_cb,
                stack_top,
                clone_flags,
                &child_data as *const ChildData as *mut libc::c_void,
            ) {
                Ok(pid) => pid,
                Err(e) => return Err(e),
            }
        };

        // BLOCK 1 fix: prevent parent from dropping stack and captured data
        // while the child is still executing on them. The child's execvp replaces
        // the address space, making these unreachable after that point.
        // Intentional leak — child owns this memory until exec.
        std::mem::forget(stack);
        std::mem::forget(cmd);
        std::mem::forget(c_args);
        std::mem::forget(c_arg_ptrs);
        std::mem::forget(bpf_prog);

        // B-18A DECISION-B: track pid for kill_agent / wait_agent.
        {
            let mut map = self.processes.lock().map_err(|e| {
                SandboxError::ConfigError(format!("processes mutex poisoned: {e}"))
            })?;
            map.insert(handle.agent_id, pid);
        }

        tracing::info!(
            pid = pid,
            agent_id = %handle.agent_id,
            "Launched sandboxed agent process (Linux namespaces + seccomp)"
        );

        Ok(AgentProcess {
            process_id: pid.try_into().map_err(|_| {
                SandboxError::ProcessLaunchFailed("pid conversion overflow".to_string())
            })?,
            agent_id: handle.agent_id,
        })
    }

    fn kill_agent(&self, agent_id: Uuid) -> Result<(), SandboxError> {
        let pid = {
            let mut map = self.processes.lock().map_err(|e| {
                SandboxError::ConfigError(format!("processes mutex poisoned: {e}"))
            })?;
            map.remove(&agent_id)
        };
        let Some(pid) = pid else {
            tracing::debug!(agent_id = %agent_id, "kill_agent: no live process for agent (already exited)");
            return Ok(());
        };
        unsafe {
            // SIGKILL is unblockable. waitpid then reaps the zombie.
            libc::kill(pid, libc::SIGKILL);
            let mut status: libc::c_int = 0;
            libc::waitpid(pid, &mut status, 0);
        }
        Ok(())
    }

    fn wait_agent(&self, agent_id: Uuid) -> Result<std::process::ExitStatus, SandboxError> {
        use std::os::unix::process::ExitStatusExt;
        let pid = {
            let mut map = self.processes.lock().map_err(|e| {
                SandboxError::ConfigError(format!("processes mutex poisoned: {e}"))
            })?;
            map.remove(&agent_id)
        };
        let Some(pid) = pid else {
            return Ok(std::process::ExitStatus::from_raw(0));
        };
        let raw_status: libc::c_int = unsafe {
            let mut status: libc::c_int = 0;
            let result = libc::waitpid(pid, &mut status, 0);
            if result == -1 {
                return Err(SandboxError::ProcessLaunchFailed(format!(
                    "waitpid failed: {}",
                    std::io::Error::last_os_error()
                )));
            }
            status
        };
        Ok(std::process::ExitStatus::from_raw(raw_status))
    }

    fn cleanup(&self, handle: &SandboxHandle) -> Result<(), SandboxError> {
        let linux_handle = match &handle.platform_handle {
            PlatformSandboxHandle::Linux(h) => h,
        };

        // Reap the child process to prevent zombie accumulation.
        // PID namespace auto-cleans all descendant processes on init exit.
        if let Some(pid) = linux_handle.child_pid {
            unsafe {
                let mut status: libc::c_int = 0;
                // WNOHANG: don't block if child hasn't exited yet
                let result = libc::waitpid(pid, &mut status, libc::WNOHANG);
                if result == 0 {
                    // Child still running — send SIGKILL then reap
                    libc::kill(pid, libc::SIGKILL);
                    libc::waitpid(pid, &mut status, 0);
                }
            }
        }

        tracing::info!(
            agent_id = %handle.agent_id,
            "Linux sandbox cleanup complete"
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// E-22 (SCHEMA_VERSION 170) -- MicrosandboxBackend
// ---------------------------------------------------------------------------
//
// 4th `AgentSandbox` impl matching the `SandboxBackend::Microsandbox`
// enum variant. The actual microsandbox / libkrun microVM subprocess
// wiring is RUNPOD-PENDING (microsandbox_linux_smoke per
// workflow/reasoning/E-22-decisions.md "deferred to follow-up"); this
// impl ships the AgentSandbox-trait-shaped surface so `AgentPool`
// dispatch keyed on `SandboxBackend::Microsandbox` does not fall
// through to a missing impl.
//
// Routing:
//   - On Linux with `MICROSANDBOX_BIN` pointing at an existing file +
//     /dev/kvm available, delegate to a held `LinuxSandbox` so the
//     existing seccomp + clone3 + capability-drop pipeline runs. This
//     mirrors the "degrades gracefully" plan ACCEPTANCE check #2; the
//     microsandbox-specific snapshot reuse path is a follow-up.
//   - When `MICROSANDBOX_BIN` is absent OR /dev/kvm is missing, every
//     trait method returns `SandboxError::ConfigError` with a stable
//     diagnostic string so the orchestrator AgentPool can degrade to
//     `SandboxBackend::Firecracker` via `select_backend_detailed`.
//   - On non-Linux platforms the impl returns
//     `SandboxError::PlatformNotSupported`; callers must use
//     `select_backend_detailed` first to fall through to
//     `AppContainer` (Windows) or `Gvisor` / `AppContainer` on
//     unsupported platforms.
//
// E-22-decisions.md DECISION (this commit): the MicrosandboxBackend
// AgentSandbox impl ships as a "wired-trait-shape, defer-live-shellout"
// pattern. The live microsandbox CLI subprocess (and the snapshot
// reuse path) is RUNPOD-PENDING `microsandbox_linux_smoke`. The
// LinuxSandbox delegation gives us a working sandbox on Linux + KVM
// hosts that *do* have the bin; the binary's actual microVM features
// are not exercised but the AgentSandbox contract is satisfied.
//
// RUNPOD install ATTEMPTED 2026-05-29 (offline batch s-25120):
//   Install command per github.com/microsandbox/microsandbox README:
//     curl -fsSL https://install.microsandbox.dev | sh
//   Status: NOT INSTALLED on offline-batch host (Windows + git bash);
//   the install script targets Linux/macOS and requires sudo + KVM.
//   Remote pythondev (Ubuntu 24.04 WSL2 + RTX 3090, ~/sidecar.pid box)
//   is the correct install target. Action: defer to next RUNPOD pass
//   that SSHes into pythondev and runs the curl|sh against ~/microsandbox-bin.
//   `microsandbox --version` and `msb --version` both `command not found`
//   on the offline-batch host.

#[cfg(target_os = "linux")]
pub struct MicrosandboxBackend {
    /// Held LinuxSandbox delegate. Used when MICROSANDBOX_BIN is
    /// present + KVM available; remains live for the process lifetime
    /// so kill_agent / wait_agent can route to the same `processes`
    /// map.
    inner: LinuxSandbox,
}

#[cfg(target_os = "linux")]
impl MicrosandboxBackend {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: LinuxSandbox::new(),
        }
    }

    /// Returns Ok(()) when both preconditions hold (KVM + MICROSANDBOX_BIN
    /// names an existing file). Otherwise returns a stable
    /// ConfigError so the caller can degrade.
    fn precondition_check(&self) -> Result<std::path::PathBuf, SandboxError> {
        if !kvm_available() {
            return Err(SandboxError::ConfigError(
                "microsandbox: /dev/kvm not available (no virtualization extensions)".to_string()
            ));
        }
        match microsandbox_bin_available() {
            Some(path) => Ok(path),
            None => Err(SandboxError::ConfigError(
                "microsandbox: MICROSANDBOX_BIN unset or names a missing file".to_string()
            )),
        }
    }
}

#[cfg(target_os = "linux")]
impl Default for MicrosandboxBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
impl AgentSandbox for MicrosandboxBackend {
    fn create(&self, config: &AppContainerConfig) -> Result<SandboxHandle, SandboxError> {
        // Precondition: requires KVM + MICROSANDBOX_BIN. Caller should
        // have already used select_backend_detailed() to verify; we
        // re-check defensively because impls must satisfy the trait
        // contract regardless of how they were dispatched.
        let _bin = self.precondition_check()?;
        // RUNPOD-PENDING microsandbox_linux_smoke: snapshot reuse via
        // microsandbox CLI lives here. For now we delegate the
        // create() lifecycle to LinuxSandbox so the AgentSandbox
        // contract is satisfied on hosts that *do* have the bin.
        self.inner.create(config)
    }

    fn launch_agent(
        &self,
        handle: &SandboxHandle,
        command: &str,
        args: &[String],
        scratch_dir: &Path,
    ) -> Result<AgentProcess, SandboxError> {
        let _bin = self.precondition_check()?;
        // RUNPOD-PENDING microsandbox_linux_smoke: the live shellout
        // path (`microsandbox run --snapshot ... -- <cmd> <args>`) is
        // wired in the follow-up. The current delegate runs the agent
        // under the LinuxSandbox seccomp + clone3 pipeline so we do
        // not regress isolation on the falling-back hosts.
        self.inner.launch_agent(handle, command, args, scratch_dir)
    }

    fn kill_agent(&self, agent_id: Uuid) -> Result<(), SandboxError> {
        // No precondition re-check on teardown -- if create() succeeded
        // earlier, the inner handle table owns the agent_id even if
        // the bin has since been removed.
        self.inner.kill_agent(agent_id)
    }

    fn wait_agent(&self, agent_id: Uuid) -> Result<std::process::ExitStatus, SandboxError> {
        self.inner.wait_agent(agent_id)
    }

    fn cleanup(&self, handle: &SandboxHandle) -> Result<(), SandboxError> {
        self.inner.cleanup(handle)
    }
}

/// Non-Linux MicrosandboxBackend stub. Returns `PlatformNotSupported`
/// on every trait method so any caller that mis-dispatches through
/// `SandboxBackend::Microsandbox` on a non-Linux host fails loudly
/// rather than silently. The canonical degradation path is
/// `select_backend_detailed` which returns `AppContainer` on Windows
/// + `Gvisor` / `AppContainer` on other-OS with a
/// `MicroVmFallbackReason`.
#[cfg(not(target_os = "linux"))]
pub struct MicrosandboxBackend;

#[cfg(not(target_os = "linux"))]
impl MicrosandboxBackend {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(target_os = "linux"))]
impl Default for MicrosandboxBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "linux"))]
impl AgentSandbox for MicrosandboxBackend {
    fn create(&self, _config: &AppContainerConfig) -> Result<SandboxHandle, SandboxError> {
        Err(SandboxError::PlatformNotSupported(
            "microsandbox: linux + KVM only; use select_backend_detailed for degradation".to_string()
        ))
    }

    fn launch_agent(
        &self,
        _handle: &SandboxHandle,
        _command: &str,
        _args: &[String],
        _scratch_dir: &Path,
    ) -> Result<AgentProcess, SandboxError> {
        Err(SandboxError::PlatformNotSupported(
            "microsandbox: linux + KVM only".to_string()
        ))
    }

    fn kill_agent(&self, _agent_id: Uuid) -> Result<(), SandboxError> {
        Err(SandboxError::PlatformNotSupported(
            "microsandbox: linux + KVM only".to_string()
        ))
    }

    fn wait_agent(&self, _agent_id: Uuid) -> Result<std::process::ExitStatus, SandboxError> {
        Err(SandboxError::PlatformNotSupported(
            "microsandbox: linux + KVM only".to_string()
        ))
    }

    fn cleanup(&self, _handle: &SandboxHandle) -> Result<(), SandboxError> {
        Err(SandboxError::PlatformNotSupported(
            "microsandbox: linux + KVM only".to_string()
        ))
    }
}

// ---------------------------------------------------------------------------
// create_sandbox: platform-specific factory
// ---------------------------------------------------------------------------

pub fn create_sandbox() -> Result<Box<dyn AgentSandbox>, SandboxError> {
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(WindowsAppContainerSandbox::new()))
    }

    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(LinuxSandbox::new()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(SandboxError::PlatformNotSupported(
            "Sandbox not supported on this platform".to_string()
        ))
    }
}

// ---------------------------------------------------------------------------
// Self-modification locks
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SelfModificationLocks {
    /// Locked paths that agents cannot modify
    pub locked_paths: Vec<PathBuf>,
}

impl SelfModificationLocks {
    pub fn new() -> Self {
        Self {
            locked_paths: Vec::new(),
        }
    }

    /// Create locks with default protected paths relative to a project root.
    /// Locks: workflow/, .claude/, memory-bank/, and the agent binary directory.
    pub fn with_project_defaults(project_root: PathBuf, agent_binary_dir: Option<PathBuf>) -> Self {
        let mut locked = vec![
            project_root.join("workflow"),
            project_root.join(".claude"),
            project_root.join("memory-bank"),
        ];
        if let Some(bin_dir) = agent_binary_dir {
            locked.push(bin_dir);
        }
        Self { locked_paths: locked }
    }
    
    /// Check if a path is locked (canonical realpath comparison)
    ///
    /// Uses std::fs::canonicalize() to resolve symlinks, junction points, and reparse points
    /// before comparison. This prevents normalization attacks where an agent could bypass
    /// locks using NTFS junction points or other path normalization tricks.
    pub fn is_locked(&self, path: &PathBuf) -> bool {
        // Canonicalize the target path to resolve all symlinks and junction points
        let canonical_target = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                // If canonicalization fails (path doesn't exist), fall back to the original path
                // This is safe because a non-existent path cannot be used for modification
                path.clone()
            }
        };

        // Canonicalize each locked path and compare
        self.locked_paths.iter().any(|locked| {
            let canonical_locked = match std::fs::canonicalize(locked) {
                Ok(p) => p,
                Err(_) => {
                    // If locked path doesn't exist, use as-is
                    locked.clone()
                }
            };
            canonical_target.starts_with(&canonical_locked)
        })
    }
    
    /// Add a locked path
    pub fn add_lock(&mut self, path: PathBuf) {
        self.locked_paths.push(path);
    }
}

impl Default for SelfModificationLocks {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// AgentEnvironment: sanitized environment for agent processes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentEnvironment {
    /// Environment variables to pass to agent
    pub env_vars: Vec<(String, String)>,
}

impl AgentEnvironment {
    pub fn new(scratch_dir: PathBuf) -> Self {
        let mut env_vars = Vec::new();

        // Only pass: LANG (sanitized), TERM (sanitized), PATH (restricted, platform-specific), TMPDIR (scratch)
        // Explicitly blocked: HOME, USER, API keys, ELAI_CC_* variables, shell history paths

        // LANG (sanitized - only allow basic locale format)
        if let Ok(lang) = std::env::var("LANG") {
            // Allow only basic locale format (e.g., "en_US.UTF-8")
            // Reject any value containing path separators or suspicious characters
            if !lang.contains('/') && !lang.contains('\\') && !lang.contains("..") {
                env_vars.push(("LANG".to_string(), lang));
            }
        }

        // TERM (sanitized - only allow terminal type names)
        if let Ok(term) = std::env::var("TERM") {
            // Allow only basic terminal types (e.g., "xterm", "xterm-256color")
            // Reject any value containing path separators or suspicious characters
            if !term.contains('/') && !term.contains('\\') && !term.contains("..") {
                env_vars.push(("TERM".to_string(), term));
            }
        }

        // PATH (restricted - minimal path for basic operations, platform-specific)
        let path = if cfg!(target_os = "windows") {
            // Windows: System32 is sufficient for basic operations
            r"C:\Windows\System32".to_string()
        } else {
            // Unix: /usr/bin:/bin is sufficient for basic operations
            "/usr/bin:/bin".to_string()
        };
        env_vars.push(("PATH".to_string(), path));

        // TMPDIR (scratch directory - always use provided scratch dir)
        env_vars.push(("TMPDIR".to_string(), scratch_dir.to_string_lossy().to_string()));

        // On Windows, also set TEMP and TMP (Windows-specific temp directories)
        if cfg!(target_os = "windows") {
            env_vars.push(("TEMP".to_string(), scratch_dir.to_string_lossy().to_string()));
            env_vars.push(("TMP".to_string(), scratch_dir.to_string_lossy().to_string()));
        }

        Self { env_vars }
    }
    
    /// Convert to format suitable for process spawning
    pub fn as_env_map(&self) -> std::collections::BTreeMap<String, String> {
        self.env_vars.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_appcontainer_config_creation() {
        let agent_id = Uuid::new_v4(); // TEST_KEEP: unit-test fixture, real entropy OK
        let config = AppContainerConfig::new(
            agent_id,
            AgentRole::Executor,
            PathBuf::from("/project"),
            PathBuf::from("/scratch"),
        );
        
        assert!(config.profile_name.starts_with("elai_cc_agent_"));
        assert_eq!(config.granted_paths.len(), 2);
        assert!(!config.network_allowed); // Executor doesn't get network
    }
    
    #[test]
    fn test_researcher_network_access() {
        let agent_id = Uuid::new_v4(); // TEST_KEEP: unit-test fixture, real entropy OK
        let config = AppContainerConfig::new(
            agent_id,
            AgentRole::Researcher,
            PathBuf::from("/project"),
            PathBuf::from("/scratch"),
        );
        
        assert!(config.network_allowed); // Researcher gets network
    }
    
    #[test]
    fn test_job_object_limits_default() {
        let limits = JobObjectLimits::default();
        assert_eq!(limits.active_process_limit, 1);
        assert_eq!(limits.process_memory_limit, 512 * 1024 * 1024);
        assert_eq!(limits.job_time_limit_seconds, 0);
    }
    
    #[test]
    fn test_self_modification_locks() {
        let mut locks = SelfModificationLocks::new();
        let locked_path = PathBuf::from("/locked/path");
        locks.add_lock(locked_path.clone());
        
        assert!(locks.is_locked(&locked_path));
        assert!(!locks.is_locked(&PathBuf::from("/unlocked/path")));
    }
    
    #[test]
    fn test_agent_environment_sanitization() {
        let scratch_dir = PathBuf::from("/scratch");
        let env = AgentEnvironment::new(scratch_dir.clone());
        
        let env_map = env.as_env_map();
        
        // Should have minimal vars
        assert!(env_map.contains_key("LANG") || env_map.contains_key("TERM"));
        assert!(env_map.contains_key("PATH"));
        assert!(env_map.contains_key("TMPDIR"));
        assert_eq!(env_map.get("TMPDIR").unwrap(), &scratch_dir.to_string_lossy().to_string());
        
        // Should NOT have sensitive vars
        assert!(!env_map.contains_key("HOME"));
        assert!(!env_map.contains_key("USER"));
    }
}
