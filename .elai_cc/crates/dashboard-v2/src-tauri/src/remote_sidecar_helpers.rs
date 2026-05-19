//! remote_sidecar_helpers.rs — SSH boot + tunnel + health-poll helpers for
//! the Tauri dashboard remote-sidecar overlay.
//!
//! Inlined from crates/cli/src/remote_sidecar.rs so that dashboard-v2 does
//! not take a compile-time dep on the cli crate (CLAUDE.md no-new-deps rule).
//! Keep in sync with the cli version if the boot protocol changes.
//!
//! These helpers are deliberately blocking (std::thread::sleep + TcpStream).
//! They MUST be called via tokio::task::spawn_blocking so they do not stall
//! the Tauri async runtime.

use std::env;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};

// ---------------------------------------------------------------------------
// Connection constants (from CLAUDE.md "Remote sidecar (canonical)")
// ---------------------------------------------------------------------------

const SSH_HOST: &str = "192.0.2.10";
const SSH_PORT: &str = "6465";
const SSH_USER: &str = "maintainer";
const SSH_KEY_ENV: &str = "ELAI_REMOTE_SSH_KEY";

const REMOTE_BOOT_CMD: &str = r#"bash -lc "source ~/.venvs/elai-sidecar/bin/activate && cd ~/elai_inference && nohup env ELAI_FORCE_SDPA=1 python server.py --model huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated --port 8100 --no-dola --turboquant-bits 0 --no-steering --load-in-4bit > ~/sidecar.log 2>&1 & echo $! > ~/sidecar.pid""#;

const HEALTH_TIMEOUT_SECS: u64 = 120;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve SSH private-key path: ELAI_REMOTE_SSH_KEY env override, else
/// ~/.ssh/elai_remote.
pub(crate) fn ssh_key_path() -> PathBuf {
    if let Ok(k) = env::var(SSH_KEY_ENV) {
        return PathBuf::from(k);
    }
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(home).join(".ssh").join("elai_remote")
}

/// Resolve start-tunnel.ps1 path relative to the dashboard executable.
pub(crate) fn tunnel_script_path() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|e| format!("cannot determine exe path: {e}"))?;
    // exe lives at dashboard-v2/src-tauri/target/release/<name> (dev)
    // or at install/<name> — walk up to find the workflow root heuristically.
    // Primary: go up 4 levels (release/ -> target/ -> src-tauri/ -> dashboard-v2/ -> .elai_cc/)
    // then ../../workflow/remote-setup/start-tunnel.ps1
    let maybe_elai_cc = exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf());

    if let Some(elai_cc) = maybe_elai_cc {
        let script = elai_cc
            .parent()
            .unwrap_or(&elai_cc)
            .join("workflow")
            .join("remote-setup")
            .join("start-tunnel.ps1");
        if script.exists() {
            return Ok(script);
        }
    }

    // Fallback: ELAI_WORKFLOW_ROOT env var.
    if let Ok(root) = env::var("ELAI_WORKFLOW_ROOT") {
        let script = PathBuf::from(root)
            .join("workflow")
            .join("remote-setup")
            .join("start-tunnel.ps1");
        if script.exists() {
            return Ok(script);
        }
    }

    Err("start-tunnel.ps1 not found; set ELAI_WORKFLOW_ROOT or run from .elai_cc/".to_owned())
}

// ---------------------------------------------------------------------------
// Boot steps (blocking — call from spawn_blocking)
// ---------------------------------------------------------------------------

/// Step 1: SSH boot. Non-zero exit is a warning, not abort (may already be up).
pub(crate) fn ssh_boot_remote_sidecar() -> Result<(), String> {
    let key = ssh_key_path();
    tracing::info!(
        host = SSH_HOST,
        port = SSH_PORT,
        key = %key.display(),
        "remote-sidecar overlay: sending SSH boot command"
    );

    let status = Command::new("ssh")
        .args([
            "-i",
            key.to_str().unwrap_or("~/.ssh/elai_remote"),
            "-p",
            SSH_PORT,
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "BatchMode=yes",
            &format!("{SSH_USER}@{SSH_HOST}"),
            REMOTE_BOOT_CMD,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("failed to invoke ssh: {e}"))?;

    if !status.success() {
        tracing::warn!(
            exit_code = ?status.code(),
            "remote-sidecar overlay: SSH boot exited non-zero; may already be running"
        );
    } else {
        tracing::info!("remote-sidecar overlay: SSH boot dispatched");
    }
    Ok(())
}

/// Step 2: Start the SSH tunnel as a detached hidden PowerShell process.
pub(crate) fn start_ssh_tunnel() -> Result<(), String> {
    let script = tunnel_script_path()?;
    tracing::info!(script = %script.display(), "remote-sidecar overlay: starting SSH tunnel");

    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-WindowStyle",
        "Hidden",
        "-File",
        script.to_str().unwrap_or("start-tunnel.ps1"),
    ])
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .stdin(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
    }

    cmd.spawn()
        .map_err(|e| format!("failed to spawn start-tunnel.ps1: {e}"))?;
    tracing::info!("remote-sidecar overlay: tunnel process spawned");
    Ok(())
}

/// Step 3: Poll until the SSH tunnel is accepting TCP connections on port 8100.
///
/// Uses a raw TcpStream connect rather than reqwest::blocking (which is not
/// available in this crate — the dashboard-v2 reqwest dep has no "blocking"
/// feature). A successful TCP connect means the tunnel is forwarding traffic;
/// the sidecar HTTP server will respond once the model finishes loading.
///
/// Returns elapsed seconds on success.
pub(crate) fn poll_until_healthy() -> Result<f32, String> {
    let poll_clock = OsClock;
    let start = poll_clock.now();
    let timeout = Duration::from_secs(HEALTH_TIMEOUT_SECS);
    let mut wait_ms = 2_000u64;
    let sock: SocketAddr = "127.0.0.1:8100"
        .parse()
        .map_err(|e| format!("internal: bad sidecar addr: {e}"))?;

    loop {
        if poll_clock.elapsed_since(start) >= timeout {
            return Err(format!(
                "remote sidecar port 8100 did not open within {HEALTH_TIMEOUT_SECS}s; \
                 check ~/sidecar.log on the remote host"
            ));
        }

        let reachable = TcpStream::connect_timeout(&sock, Duration::from_secs(3)).is_ok();

        if reachable {
            let elapsed = poll_clock.elapsed_since(start).as_secs_f32();
            tracing::info!(elapsed_secs = elapsed, "remote-sidecar overlay: health poll passed");
            return Ok(elapsed);
        }

        thread::sleep(Duration::from_millis(wait_ms));
        wait_ms = (wait_ms * 3 / 2).min(8_000);
    }
}
