//! Batch 3A (Phase 4F) — SystemProbe surface.
//!
//! Detects GPU VRAM, system RAM, and free disk for the project drive.
//!
//! GPU detection (3-tier fallback per E-22 phase-4F M1, 2026-05-22):
//!   1. `nvml-wrapper` (`Nvml::init() -> device.memory_info()` +
//!      `device.uuid()`) — canonical, mirrors the pattern at
//!      `media_validator/types.rs:493-519`. Graceful fallback when
//!      `Nvml::init()` returns `Err` (no NVIDIA driver / no GPU); spec
//!      phase-4F.txt risk R-4F-1.
//!   2. `nvidia-smi --query-gpu=...` shell-out — kept as fallback for
//!      hosts where the NVML loader is present but blocked (CI sandboxes
//!      that whitelist the binary but not the library).
//!   3. `EnvFallback` — read `ELAI_VRAM_GB`, `ELAI_VRAM_FREE_GB`,
//!      `ELAI_FREE_RAM_GB`, `ELAI_FREE_DISK_GB` when neither GPU path
//!      produces a signal.
//!   4. `Unavailable` — hardcoded to `Vram12Gb` (the dashboard's prior
//!      hardcode per the MVP audit 2026-04-18 dashboard/src/lib.rs:338).
//!
//! RAM/disk paths continue to use `wmic`/`/proc/meminfo`/`df` shell-out;
//! sysinfo remains rejected (CLAUDE.md no-new-deps rule, user has not
//! re-approved).
//!
//! Cached for 60 seconds via a sync `Mutex` so dashboard polling does not
//! thrash the NVML loader / `nvidia-smi`. The cache uses
//! `std::time::Instant` (monotonic); test code can force a fresh probe by
//! constructing a new `DefaultSystemProber`.

use std::env;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use elai_core::clock::ClockProvider;
use elai_core::VramTier;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::timeout;

/// Environment variable toggling the probe entirely. Recognised values:
/// "1", "true", "TRUE", "on", "ON". Any other value (including empty) leaves
/// the probe enabled.
pub const ELAI_DISABLE_SYSTEM_PROBE_ENV: &str = "ELAI_DISABLE_SYSTEM_PROBE";

/// Env-fallback variable names (Option D from batch-3a prompt).
pub const ELAI_VRAM_GB_ENV: &str = "ELAI_VRAM_GB";
pub const ELAI_VRAM_FREE_GB_ENV: &str = "ELAI_VRAM_FREE_GB";
pub const ELAI_FREE_RAM_GB_ENV: &str = "ELAI_FREE_RAM_GB";
pub const ELAI_FREE_DISK_GB_ENV: &str = "ELAI_FREE_DISK_GB";

/// Unavailable-path default VRAM (GiB) — matches the dashboard's prior
/// hardcoded `VramTier::Vram12Gb` so behaviour never regresses below the
/// pre-4F baseline. See audit section "VRAM auto-detect absent".
const DEFAULT_VRAM_TOTAL_MB: u32 = 12_288;

/// Default nvidia-smi / platform-tool timeout. 5s is generous for a local
/// binary and short enough that a hung process can never stall the dashboard
/// poll loop.
const SHELL_OUT_TIMEOUT: Duration = Duration::from_secs(5);

/// Cache TTL. Matches the 60 s value requested by the batch-3a prompt.
const CACHE_TTL: Duration = Duration::from_secs(60);

/// Returns `true` when `ELAI_DISABLE_SYSTEM_PROBE` is set to a truthy value.
pub fn system_probe_disabled_via_env() -> bool {
    env::var(ELAI_DISABLE_SYSTEM_PROBE_ENV)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
        .unwrap_or(false)
}

/// Per-probe GPU descriptor. `tier` is held as an enum for callers that
/// want to pattern-match directly; `Serialize` / `Deserialize` are NOT
/// derived on `GpuInfo` because `elai_core::VramTier` does not (yet) derive
/// serde — adding derives there is a cross-crate change that CLAUDE.md
/// requires user confirmation for. `SystemProbeResult` round-trips go via
/// `SystemProbePayload` instead (see crates/orchestrator/src/ssb.rs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInfo {
    pub name: String,
    pub vram_total_mb: u32,
    pub vram_free_mb: u32,
    /// VRAM tier classification. Derived from `vram_total_mb` via
    /// [`classify_vram_tier`].
    pub tier: VramTier,
    pub driver_version: Option<String>,
    /// C-22: stable hardware UUID exposed by `nvidia-smi --query-gpu=uuid`.
    /// Format is the canonical `GPU-<32-hex with 4 dashes>` per NVML.
    /// `None` when the shell-out failed or the value did not match the
    /// expected pattern (env-fallback / Unavailable paths). Read once per
    /// probe; no separate shell-out — uuid is fetched in the same
    /// query-gpu CSV row as the other GPU fields per C-22 decision (one-shell).
    pub uuid: Option<String>,
}

/// How the probe produced its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    /// Platform tools (`nvidia-smi`, `wmic`, `df`, `/proc/meminfo`) returned
    /// parsable output.
    ShellOut,
    /// One or more shell-outs failed; values read from env vars
    /// (`ELAI_VRAM_GB`, `ELAI_FREE_RAM_GB`, `ELAI_FREE_DISK_GB`).
    EnvFallback,
    /// Neither shell nor env produced a signal; values are hardcoded.
    Unavailable,
}

impl DetectionMethod {
    /// Stable SSB / dashboard label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShellOut => "shell_out",
            Self::EnvFallback => "env_fallback",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Aggregate snapshot produced by [`SystemProber::probe`]. Not
/// `Serialize`/`Deserialize` because the embedded `GpuInfo` carries a
/// non-serde `VramTier`. SSB emission goes through `SystemProbePayload`
/// which holds a string tier label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemProbeResult {
    pub gpu: Option<GpuInfo>,
    pub ram_total_mb: u64,
    pub ram_free_mb: u64,
    pub disk_total_mb: u64,
    pub disk_free_mb: u64,
    pub detection_method: DetectionMethod,
    /// Unix-epoch milliseconds at which the probe ran. Identity marker for
    /// cache equality checks in the contract tests (two probes inside the
    /// 60 s window share the same `probed_at_ms`).
    pub probed_at_ms: i64,
}

/// Classify `total_mb` into a [`VramTier`]. Rounds DOWN so a GPU with
/// 11.7 GiB reported (RTX 3060) lands on `Vram12Gb`; 10.2 GiB reported
/// (mobile 2080) also lands on `Vram12Gb` because we prefer optimistic tier
/// assignment to avoid starving Layer-3 branch budgets. Cloud tier is
/// never returned here — set it via env override.
pub fn classify_vram_tier(total_mb: u32) -> VramTier {
    // Tier edges chosen to match existing constants in
    // `crates/core/src/vram_tier.rs` plus the 2x3090 tier target noted in
    // memory:project_vram_tier_plan.
    if total_mb >= 44_000 {
        VramTier::Vram48Gb
    } else if total_mb >= 22_000 {
        VramTier::Vram24Gb
    } else if total_mb >= 14_500 {
        VramTier::Vram16Gb
    } else if total_mb >= 10_000 {
        VramTier::Vram12Gb
    } else {
        VramTier::Vram8Gb
    }
}

/// Runtime interface for platform probing. Dyn-safe via `Arc<dyn ...>`.
/// Uses the hand-rolled `Pin<Box<dyn Future>>` return-type pattern so the
/// orchestrator crate does not take on `async-trait` (CLAUDE.md anti-stale-
/// training hard rule).
pub trait SystemProber: Send + Sync {
    fn probe<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = SystemProbeResult> + Send + 'a>>;
}

// ---------------------------------------------------------------------------
// DefaultSystemProber — shell-out implementation with env fallback
// ---------------------------------------------------------------------------

/// Default probe implementation. Reads from:
///   * GPU: `nvidia-smi --query-gpu=name,memory.total,memory.free,driver_version --format=csv,noheader,nounits`.
///   * RAM: `wmic OS get FreePhysicalMemory,TotalVisibleMemorySize /value` on
///     Windows, `/proc/meminfo` on Linux.
///   * Disk: `wmic LogicalDisk where "DeviceID='<drive>'" get FreeSpace,Size /value`
///     on Windows, `df -Pk <path>` on Linux.
///
/// All shell commands are wrapped in a 5 s tokio timeout so a hung platform
/// tool never stalls the dashboard.
pub struct DefaultSystemProber {
    /// Optional project directory used to pick the right drive / mount.
    /// When `None`, defaults to `std::env::current_dir()`.
    project_dir: Option<PathBuf>,
    /// Clock used for the `probed_at_ms` field.
    clock: Arc<dyn ClockProvider>,
    /// Cache guarded by a sync `Mutex` (tiny critical section; acquiring the
    /// lock is cheaper than awaiting a second shell-out).
    cache: Mutex<Option<CachedReport>>,
}

struct CachedReport {
    report: SystemProbeResult,
    cached_at: elai_core::LogicalInstant,
}

impl DefaultSystemProber {
    /// Construct a new prober bound to the given project directory. `None`
    /// resolves to `current_dir()` at probe time.
    pub fn new(project_dir: Option<PathBuf>, clock: Arc<dyn ClockProvider>) -> Self {
        Self {
            project_dir,
            clock,
            cache: Mutex::new(None),
        }
    }

    /// Clear the cache so the next `probe()` call re-queries the platform.
    /// Used by contract tests that assert cache-miss behaviour explicitly.
    pub fn invalidate_cache(&self) {
        if let Ok(mut guard) = self.cache.lock() {
            *guard = None;
        }
    }

    fn now_ms(&self) -> i64 {
        let ms_u128 = self
            .clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        i64::try_from(ms_u128).unwrap_or(i64::MAX)
    }

    fn effective_project_dir(&self) -> PathBuf {
        self.project_dir
            .clone()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}

impl SystemProber for DefaultSystemProber {
    fn probe<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = SystemProbeResult> + Send + 'a>> {
        Box::pin(async move {
            // Fast-return on env kill switch — emit Unavailable so downstream
            // code always has a report and never has to special-case `Option`.
            if system_probe_disabled_via_env() {
                return build_unavailable_report(self.now_ms());
            }

            // Cache hit inside the 60 s TTL.
            if let Ok(guard) = self.cache.lock() {
                if let Some(cached) = guard.as_ref() {
                    if self.clock.elapsed_since(cached.cached_at) < CACHE_TTL {
                        return cached.report.clone();
                    }
                }
            }

            // Attempt shell-out. On ANY failure fall back to env vars. On env
            // failure fall back to hardcoded defaults.
            let gpu_opt = probe_gpu_shell_out().await;
            let ram_opt = probe_ram_shell_out().await;
            let project_dir = self.effective_project_dir();
            let disk_opt = probe_disk_shell_out(&project_dir).await;

            let all_shell_ok =
                gpu_opt.is_some() && ram_opt.is_some() && disk_opt.is_some();

            let report = if all_shell_ok {
                // Unwraps justified: `all_shell_ok` is the invariant. Kept as
                // `.unwrap_or` fallbacks anyway so a racing env change cannot
                // panic the probe under any code path.
                let gpu = gpu_opt.unwrap_or_else(|| GpuInfo {
                    name: "unavailable".to_owned(),
                    vram_total_mb: DEFAULT_VRAM_TOTAL_MB,
                    vram_free_mb: 0,
                    tier: VramTier::Vram12Gb,
                    driver_version: None,
                    uuid: None,
                });
                let (ram_total, ram_free) = ram_opt.unwrap_or((0, 0));
                let (disk_total, disk_free) = disk_opt.unwrap_or((0, 0));
                SystemProbeResult {
                    gpu: Some(gpu),
                    ram_total_mb: ram_total,
                    ram_free_mb: ram_free,
                    disk_total_mb: disk_total,
                    disk_free_mb: disk_free,
                    detection_method: DetectionMethod::ShellOut,
                    probed_at_ms: self.now_ms(),
                }
            } else {
                match build_env_fallback_report(self.now_ms(), gpu_opt) {
                    Some(r) => r,
                    None => build_unavailable_report(self.now_ms()),
                }
            };

            // Store in cache. Lock failure is non-fatal (cache is a performance
            // optimisation, not a correctness boundary).
            if let Ok(mut guard) = self.cache.lock() {
                *guard = Some(CachedReport {
                    report: report.clone(),
                    cached_at: self.clock.now(),
                });
            }
            report
        })
    }
}

// ---------------------------------------------------------------------------
// Platform probe helpers — shell-out only, no new crates
// ---------------------------------------------------------------------------

/// E-22 phase-4F M1 (2026-05-22): primary GPU probe via nvml-wrapper.
/// Mirrors the canonical pattern at
/// `crates/orchestrator/src/media_validator/types.rs:493-519`. Returns
/// `None` on any NVML failure so the caller falls through to the
/// nvidia-smi shell-out path. Graceful by design — phase-4F.txt risk
/// R-4F-1: "On systems without NVIDIA GPU, Nvml::init() returns Err.
/// Must handle gracefully: gpu_count=0, gpu_info=vec![]." Equivalent here:
/// returns `None` so the upper layer records EnvFallback / Unavailable.
fn probe_gpu_nvml() -> Option<GpuInfo> {
    use nvml_wrapper::Nvml;
    // Nvml::init() loads libnvidia-ml.so / nvml.dll. Wrapped in a closure
    // so we can propagate Option without unwrap.
    let nvml = Nvml::init().ok()?;
    let device_count = nvml.device_count().ok()?;
    if device_count == 0 {
        return None;
    }
    // First GPU only per batch-3a spec (single shell-out parity).
    let device = nvml.device_by_index(0).ok()?;
    let mem = device.memory_info().ok()?;
    let name = device.name().ok()?;
    // sys_driver_version returns the cluster-wide driver string (e.g.
    // "552.22"); fall back to None on error rather than failing the whole
    // probe (downstream consumers tolerate Option<String>).
    let driver_version = nvml.sys_driver_version().ok();
    let uuid = device.uuid().ok().and_then(|raw| parse_gpu_uuid(&raw));
    // mem.{total,free} are in bytes; convert to MB (1024*1024) to match
    // the existing GpuInfo schema. Saturating casts keep the u32 field
    // safe on >4 PB GPUs (theoretical only; current max GPU is 192 GB).
    let total_mb = u32::try_from(mem.total / (1024 * 1024)).unwrap_or(u32::MAX);
    let free_mb = u32::try_from(mem.free / (1024 * 1024)).unwrap_or(u32::MAX);
    let tier = classify_vram_tier(total_mb);
    Some(GpuInfo {
        name,
        vram_total_mb: total_mb,
        vram_free_mb: free_mb,
        tier,
        driver_version,
        uuid,
    })
}

async fn probe_gpu_shell_out() -> Option<GpuInfo> {
    // E-22 phase-4F M1: try NVML first; fall through to nvidia-smi if
    // the library isn't loadable or the device query fails. NVML is a
    // single C-FFI call so this stays well under the 5s SHELL_OUT_TIMEOUT
    // budget.
    if let Some(gpu) = probe_gpu_nvml() {
        return Some(gpu);
    }
    // Shell-out fallback. First GPU only per batch-3a spec.
    // `--format=csv,noheader,nounits` yields lines like
    // "NVIDIA GeForce RTX 3060, 12288, 11392, 552.22, GPU-xxxx".
    // C-22: uuid added as the 5th column (single shell-out per C-22 decision —
    // avoids the doubled p50 latency of a second `-q -d UUID` invocation).
    let fut = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,memory.free,driver_version,uuid",
            "--format=csv,noheader,nounits",
        ])
        .output();
    let output = match timeout(SHELL_OUT_TIMEOUT, fut).await {
        Ok(Ok(o)) => o,
        _ => return None,
    };
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().find(|l| !l.trim().is_empty())?;
    let fields: Vec<&str> = first_line.split(',').map(str::trim).collect();
    if fields.len() < 3 {
        return None;
    }
    let name = fields[0].to_owned();
    let vram_total_mb: u32 = fields[1].parse().ok()?;
    let vram_free_mb: u32 = fields[2].parse().ok()?;
    let driver_version = fields.get(3).map(|s| s.to_string());
    // uuid: validate format `GPU-` + 32 hex with 4 dashes. Anything else
    // (e.g. nvidia-smi returned `[Not Supported]` on an older driver)
    // becomes None — never propagate a malformed UUID to consumers.
    let uuid = fields.get(4).and_then(|raw| parse_gpu_uuid(raw));
    let tier = classify_vram_tier(vram_total_mb);
    Some(GpuInfo {
        name,
        vram_total_mb,
        vram_free_mb,
        tier,
        driver_version,
        uuid,
    })
}

/// C-22: stdlib-only parser for an NVML UUID line. Expected shape:
///   `GPU-<8 hex>-<4 hex>-<4 hex>-<4 hex>-<12 hex>` (36 chars + `GPU-`).
/// Returns the canonical string when the input matches; `None` otherwise.
/// No regex crate dependency (CLAUDE.md no-new-deps).
fn parse_gpu_uuid(raw: &str) -> Option<String> {
    let s = raw.trim();
    let rest = s.strip_prefix("GPU-")?;
    // Expect 32 hex chars + 4 dashes = 36 chars; segment lengths 8-4-4-4-12.
    let segs: Vec<&str> = rest.split('-').collect();
    if segs.len() != 5 {
        return None;
    }
    let expected_lens = [8usize, 4, 4, 4, 12];
    for (seg, &expected) in segs.iter().zip(expected_lens.iter()) {
        if seg.len() != expected {
            return None;
        }
        if !seg.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
    }
    Some(s.to_owned())
}

#[cfg(target_os = "windows")]
async fn probe_ram_shell_out() -> Option<(u64, u64)> {
    // wmic emits KiB values in lines like:
    //   FreePhysicalMemory=10485760
    //   TotalVisibleMemorySize=33554432
    // The `/value` flag keeps the format stable across Windows 10/11.
    let fut = Command::new("wmic")
        .args(["OS", "get", "FreePhysicalMemory,TotalVisibleMemorySize", "/value"])
        .output();
    let output = match timeout(SHELL_OUT_TIMEOUT, fut).await {
        Ok(Ok(o)) => o,
        _ => return None,
    };
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut free_kb: Option<u64> = None;
    let mut total_kb: Option<u64> = None;
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("FreePhysicalMemory=") {
            free_kb = v.trim().parse::<u64>().ok();
        } else if let Some(v) = line.strip_prefix("TotalVisibleMemorySize=") {
            total_kb = v.trim().parse::<u64>().ok();
        }
    }
    // Convert KiB -> MiB (divide by 1024).
    Some((total_kb? / 1024, free_kb? / 1024))
}

#[cfg(target_os = "linux")]
async fn probe_ram_shell_out() -> Option<(u64, u64)> {
    // /proc/meminfo line format: "MemTotal:       32939740 kB"
    let bytes = tokio::fs::read("/proc/meminfo").await.ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let mut total_kb: Option<u64> = None;
    let mut avail_kb: Option<u64> = None;
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let label = parts[0];
        let value: u64 = parts[1].parse().unwrap_or(0);
        match label {
            "MemTotal:" => total_kb = Some(value),
            "MemAvailable:" => avail_kb = Some(value),
            _ => {}
        }
    }
    Some((total_kb? / 1024, avail_kb? / 1024))
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
async fn probe_ram_shell_out() -> Option<(u64, u64)> {
    // macOS / BSD path — no portable probe without a new dep. Returning
    // None funnels into the env fallback, which is documented behaviour.
    None
}

#[cfg(target_os = "windows")]
async fn probe_disk_shell_out(project_dir: &Path) -> Option<(u64, u64)> {
    // Extract drive letter from the project_dir (e.g. "D:\\...").
    let drive = project_dir
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("C:")
        .to_string();
    let filter = format!("DeviceID='{drive}'");
    let fut = Command::new("wmic")
        .args(["LogicalDisk", "where", &filter, "get", "FreeSpace,Size", "/value"])
        .output();
    let output = match timeout(SHELL_OUT_TIMEOUT, fut).await {
        Ok(Ok(o)) => o,
        _ => return None,
    };
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut free_bytes: Option<u64> = None;
    let mut total_bytes: Option<u64> = None;
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("FreeSpace=") {
            free_bytes = v.trim().parse::<u64>().ok();
        } else if let Some(v) = line.strip_prefix("Size=") {
            total_bytes = v.trim().parse::<u64>().ok();
        }
    }
    const MB: u64 = 1024 * 1024;
    Some((total_bytes? / MB, free_bytes? / MB))
}

#[cfg(target_os = "linux")]
async fn probe_disk_shell_out(project_dir: &Path) -> Option<(u64, u64)> {
    let path_str = project_dir.to_string_lossy().to_string();
    let fut = Command::new("df")
        .args(["-Pk", &path_str])
        .output();
    let output = match timeout(SHELL_OUT_TIMEOUT, fut).await {
        Ok(Ok(o)) => o,
        _ => return None,
    };
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Header line + one data line. Columns: Filesystem, 1024-blocks, Used,
    // Available, Capacity, Mounted on. We want 1024-blocks and Available.
    let data = stdout.lines().nth(1)?;
    let cols: Vec<&str> = data.split_whitespace().collect();
    if cols.len() < 4 {
        return None;
    }
    let total_kb: u64 = cols[1].parse().ok()?;
    let avail_kb: u64 = cols[3].parse().ok()?;
    Some((total_kb / 1024, avail_kb / 1024))
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
async fn probe_disk_shell_out(_project_dir: &Path) -> Option<(u64, u64)> {
    None
}

// ---------------------------------------------------------------------------
// Env-fallback + Unavailable builders
// ---------------------------------------------------------------------------

fn read_env_gb(name: &str) -> Option<u64> {
    env::var(name).ok().and_then(|v| v.parse::<u64>().ok())
}

fn build_env_fallback_report(
    probed_at_ms: i64,
    shell_gpu: Option<GpuInfo>,
) -> Option<SystemProbeResult> {
    let vram_total_gb = read_env_gb(ELAI_VRAM_GB_ENV);
    let vram_free_gb = read_env_gb(ELAI_VRAM_FREE_GB_ENV);
    let ram_free_gb = read_env_gb(ELAI_FREE_RAM_GB_ENV);
    let disk_free_gb = read_env_gb(ELAI_FREE_DISK_GB_ENV);

    // At least one env var must be populated to justify the EnvFallback
    // method — otherwise we fall all the way through to Unavailable.
    if vram_total_gb.is_none()
        && vram_free_gb.is_none()
        && ram_free_gb.is_none()
        && disk_free_gb.is_none()
        && shell_gpu.is_none()
    {
        return None;
    }

    let gpu = shell_gpu.or_else(|| {
        let total_mb = u32::try_from(vram_total_gb? * 1024).ok()?;
        let free_mb = u32::try_from(vram_free_gb.unwrap_or(vram_total_gb?) * 1024).ok()?;
        Some(GpuInfo {
            name: "env-provided".to_owned(),
            vram_total_mb: total_mb,
            vram_free_mb: free_mb,
            tier: classify_vram_tier(total_mb),
            driver_version: None,
            uuid: None,
        })
    });

    let ram_free_mb = ram_free_gb.map(|g| g * 1024).unwrap_or(0);
    let disk_free_mb = disk_free_gb.map(|g| g * 1024).unwrap_or(0);

    Some(SystemProbeResult {
        gpu,
        ram_total_mb: ram_free_mb,
        ram_free_mb,
        disk_total_mb: disk_free_mb,
        disk_free_mb,
        detection_method: DetectionMethod::EnvFallback,
        probed_at_ms,
    })
}

fn build_unavailable_report(probed_at_ms: i64) -> SystemProbeResult {
    SystemProbeResult {
        gpu: Some(GpuInfo {
            name: "unavailable".to_owned(),
            vram_total_mb: DEFAULT_VRAM_TOTAL_MB,
            vram_free_mb: 0,
            tier: VramTier::Vram12Gb,
            driver_version: None,
            uuid: None,
        }),
        ram_total_mb: 0,
        ram_free_mb: 0,
        disk_total_mb: 0,
        disk_free_mb: 0,
        detection_method: DetectionMethod::Unavailable,
        probed_at_ms,
    }
}

// ---------------------------------------------------------------------------
// Tests (co-located so internal helpers stay crate-private)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_classification_rounds_down_to_safe_tier() {
        assert_eq!(classify_vram_tier(48_000), VramTier::Vram48Gb);
        assert_eq!(classify_vram_tier(24_000), VramTier::Vram24Gb);
        assert_eq!(classify_vram_tier(16_000), VramTier::Vram16Gb);
        assert_eq!(classify_vram_tier(12_288), VramTier::Vram12Gb);
        assert_eq!(classify_vram_tier(8_192), VramTier::Vram8Gb);
        // Below 10 GB lands on 8 GB tier (conservative).
        assert_eq!(classify_vram_tier(6_000), VramTier::Vram8Gb);
    }

    #[test]
    fn detection_method_strings_stable() {
        assert_eq!(DetectionMethod::ShellOut.as_str(), "shell_out");
        assert_eq!(DetectionMethod::EnvFallback.as_str(), "env_fallback");
        assert_eq!(DetectionMethod::Unavailable.as_str(), "unavailable");
    }

    #[test]
    fn unavailable_report_matches_prior_hardcode() {
        let r = build_unavailable_report(0);
        let gpu = r.gpu.expect("unavailable report retains gpu stub");
        assert_eq!(gpu.vram_total_mb, DEFAULT_VRAM_TOTAL_MB);
        assert_eq!(gpu.tier, VramTier::Vram12Gb);
        assert_eq!(r.detection_method, DetectionMethod::Unavailable);
    }

    #[test]
    fn env_fallback_requires_any_env_var() {
        // With no env vars AND no shell gpu, returns None so caller falls
        // through to Unavailable.
        let r = build_env_fallback_report(123, None);
        assert!(r.is_none());
    }
}
