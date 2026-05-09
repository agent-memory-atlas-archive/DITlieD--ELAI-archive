use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Binary name of the Tauri dashboard produced by `elai-dashboard-v2` crate.
/// The dashboard-v1 crate has been parked at `crates/.bkp_old/dashboard/`;
/// v2 lives at `crates/dashboard-v2/src-tauri/` and emits `elai-dashboard-v2[.exe]`.
/// Set `ELAI_DASHBOARD_BINARY` to override at runtime.
pub(crate) const DASHBOARD_BINARY_NAME: &str = "elai-dashboard-v2";

/// Launch the Tauri dashboard as a detached child process so the CLI never
/// links the Tauri/WebView runtime into its own binary (phase-4K.2 design
/// decision: separate process to keep CLI lean).
pub(crate) fn run_dashboard() -> Result<(), anyhow::Error> {
    let candidate = locate_dashboard_binary()?;

    if std::env::var("ELAI_NO_SIDECAR").ok().as_deref() != Some("1") {
        let port: u16 = std::env::var("ELAI_SIDECAR_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8100);
        match spawn_sidecar_visible(port) {
            Ok(SidecarSpawnOutcome::Spawned) => tracing::info!(port, "dashboard: sidecar spawned in visible cmd window"),
            Ok(SidecarSpawnOutcome::AlreadyRunning) => tracing::info!(port, "dashboard: sidecar already running"),
            Err(e) => tracing::warn!(error = %e, port, "dashboard: sidecar auto-start failed; model-backed features will degrade (set ELAI_NO_SIDECAR=1 to silence)"),
        }
    } else {
        tracing::info!("dashboard: ELAI_NO_SIDECAR=1, skipping sidecar auto-start");
    }

    tracing::info!(path = %candidate.display(), "launching ELAI dashboard");
    // patch-ui-fix: detach the dashboard from the parent cmd window so the
    // elai.bat terminal closes immediately after elai.exe returns, leaving
    // only (a) the ELAI Sidecar cmd window and (b) the Tauri dashboard
    // window. Null the stdio handles and, on Windows, apply
    // CREATE_NO_WINDOW (0x08000000) so the child does not inherit the parent
    // console. The sidecar cmd window is spawned via `spawn_sidecar_visible`
    // above with a separate `start` command — this change does NOT touch it.
    let mut cmd = std::process::Command::new(&candidate);
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW — prevents the Tauri bootstrap from flashing a
        // console and, combined with the null stdio above, guarantees the
        // parent cmd can close cleanly.
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn()
        .map_err(|e| anyhow::anyhow!("failed to launch dashboard at {}: {e}", candidate.display()))?;
    Ok(())
}

pub(crate) enum SidecarSpawnOutcome { Spawned, AlreadyRunning }

/// Spawn the sidecar in a new visible Windows cmd window via `start-sidecar.bat`.
/// Non-blocking: returns immediately after spawning. The dashboard's SidecarProvider
/// handles "sidecar not yet reachable" gracefully; the user sees the loading progress
/// in the cmd window and the dashboard window appears without delay.
pub(crate) fn spawn_sidecar_visible(port: u16) -> Result<SidecarSpawnOutcome, anyhow::Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .connect_timeout(Duration::from_secs(1))
        .build()?;
    let already_running = client
        .get(format!("http://localhost:{port}/health"))
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    if already_running {
        return Ok(SidecarSpawnOutcome::AlreadyRunning);
    }

    let exe_dir = std::env::current_exe()?
        .parent()
        .and_then(|d| d.parent())
        .and_then(|d| d.parent())
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap_or_default());
    let script = exe_dir.join("start-sidecar.bat");
    if !script.exists() {
        return Err(anyhow::anyhow!("start-sidecar.bat not found at {}", script.display()));
    }

    Command::new("cmd")
        .args([
            "/c", "start", "ELAI Sidecar",
            script.to_str().unwrap_or("start-sidecar.bat"),
            &format!("{port}"),
        ])
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn sidecar cmd window: {e}"))?;

    Ok(SidecarSpawnOutcome::Spawned)
}

pub(crate) fn locate_dashboard_binary() -> Result<PathBuf, anyhow::Error> {
    if let Ok(override_path) = std::env::var("ELAI_DASHBOARD_BINARY") {
        let p = PathBuf::from(&override_path);
        if p.exists() {
            return Ok(p);
        }
        tracing::warn!(
            override = %override_path,
            "ELAI_DASHBOARD_BINARY points at a missing file, falling back to auto-discovery"
        );
    }
    let exe_name = if cfg!(windows) {
        format!("{DASHBOARD_BINARY_NAME}.exe")
    } else {
        DASHBOARD_BINARY_NAME.to_string()
    };
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            let sibling = dir.join(&exe_name);
            if sibling.exists() {
                return Ok(sibling);
            }
        }
    }
    // Fallback: rely on PATH.
    Ok(PathBuf::from(exe_name))
}
