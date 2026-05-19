/// remote_sidecar.rs — boot the friend's GPU box sidecar via SSH, start the
/// local SSH tunnel, poll for health, then launch the V2 dashboard.
///
/// Invoked by `elai.exe --remote-sidecar` / `elai.bat --remote`.
///
/// Boot sequence
///   0. (Idempotency probe) TCP-probe 127.0.0.1:8100. If already open, POST
///      /v1/messages with 5s timeout. If HTTP <500, sidecar is already healthy
///      via an existing tunnel — skip SSH boot + tunnel start entirely.
///   1. SSH into 192.0.2.10:6465 with key ~/.ssh/elai_remote and start the
///      Python sidecar in a detached nohup background process.
///   2. Spawn the SSH tunnel (start-tunnel.ps1) as a detached hidden PowerShell.
///   3. Poll http://127.0.0.1:8100/v1/messages with exponential backoff for up
///      to 120 s. The tunnel forwards local:8100 → remote:8100.
///   4. Set ANTHROPIC_BASE_URL=http://localhost:8100 in the child env.
///   5. Launch the V2 Tauri dashboard via dashboard::run_dashboard().
use std::env;
use std::io::{self, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::Context as _;
use elai_core::clock::{ClockProvider, OsClock};

// SSH connection details (from CLAUDE.md "Remote sidecar (canonical)").
const SSH_HOST: &str = "192.0.2.10";
const SSH_PORT: &str = "6465";
const SSH_USER: &str = "maintainer";
const SSH_KEY_ENV: &str = "ELAI_REMOTE_SSH_KEY"; // override at runtime if needed

/// Full nohup sidecar boot command — verbatim from REMOTE-SIDECAR.md.
/// Stored as a raw string to avoid escaping pain.
const REMOTE_BOOT_CMD: &str = r#"bash -lc "source ~/.venvs/elai-sidecar/bin/activate && cd ~/elai_inference && nohup env ELAI_FORCE_SDPA=1 python server.py --model huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated --port 8100 --no-dola --turboquant-bits 0 --no-steering --load-in-4bit > ~/sidecar.log 2>&1 & echo $! > ~/sidecar.pid""#;

const HEALTH_URL: &str = "http://127.0.0.1:8100/v1/messages";
const HEALTH_TIMEOUT_SECS: u64 = 120;

/// Probe TCP 127.0.0.1:{port} with a short connect timeout.
/// Returns true if the port is accepting connections.
fn tcp_probe(port: u16) -> bool {
    let addr = format!("127.0.0.1:{port}");
    TcpStream::connect_timeout(
        &addr.parse().unwrap_or_else(|_| "127.0.0.1:0".parse().expect("static addr")),
        Duration::from_secs(2),
    )
    .is_ok()
}

/// Probe the sidecar HTTP endpoint. Returns true if HTTP status < 500.
fn http_probe_healthy(client: &reqwest::blocking::Client) -> bool {
    let probe_body = serde_json::json!({
        "model": "qwen",
        "max_tokens": 1,
        "messages": [{"role": "user", "content": "ping"}]
    });
    client
        .post(HEALTH_URL)
        .header("Content-Type", "application/json")
        .json(&probe_body)
        .send()
        .map(|r| r.status().as_u16() < 500)
        .unwrap_or(false)
}

/// FIX 5 — Diagnostic probe: print TCP status for key local ports at startup.
fn print_port_diagnostics() {
    println!("[remote-sidecar] === port diagnostics ===");
    for port in [8100u16, 8889, 8080] {
        let state = if tcp_probe(port) { "OPEN" } else { "closed" };
        println!("[remote-sidecar]   127.0.0.1:{port} -> {state}");
    }
    println!("[remote-sidecar] === end diagnostics ===");
    io::stdout().flush().ok();
}

/// Resolve the SSH private-key path.
/// Prefer the ELAI_REMOTE_SSH_KEY env var, then fall back to ~/.ssh/elai_remote.
fn ssh_key_path() -> PathBuf {
    if let Ok(override_key) = env::var(SSH_KEY_ENV) {
        return PathBuf::from(override_key);
    }
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".ssh").join("elai_remote")
}

/// Resolve the start-tunnel.ps1 path relative to the CLI executable.
/// Canonical location: <elai_root>/workflow/remote-setup/start-tunnel.ps1.
fn tunnel_script_path() -> Result<PathBuf, anyhow::Error> {
    // Walk up from exe dir: target/release/ -> target/ -> .elai_cc/ -> root.
    let exe = env::current_exe().context("cannot determine exe path")?;
    // exe is .elai_cc/target/release/elai.exe → go up 3 levels to .elai_cc/
    let elai_cc = exe
        .parent()       // release/
        .and_then(|p| p.parent()) // target/
        .and_then(|p| p.parent()) // .elai_cc/
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // .elai_cc/../workflow/remote-setup/start-tunnel.ps1
    let script = elai_cc
        .parent()
        .unwrap_or(&elai_cc)
        .join("workflow")
        .join("remote-setup")
        .join("start-tunnel.ps1");

    if script.exists() {
        return Ok(script);
    }

    // Fallback: maybe we're running from the project root.
    let alt = elai_cc.join("workflow").join("remote-setup").join("start-tunnel.ps1");
    if alt.exists() {
        return Ok(alt);
    }

    Err(anyhow::anyhow!(
        "start-tunnel.ps1 not found; checked {} and {}",
        script.display(),
        alt.display()
    ))
}

/// Step 1 — SSH boot: fire-and-forget the remote sidecar startup.
/// The remote process is backgrounded via nohup so the SSH connection exits
/// immediately once the shell has forked the Python process.
fn ssh_boot_remote_sidecar() -> Result<(), anyhow::Error> {
    let key = ssh_key_path();
    let key_str = key.to_string_lossy().into_owned();
    println!("[remote-sidecar] step 1: SSH boot -> {SSH_USER}@{SSH_HOST}:{SSH_PORT} key={key_str}");
    io::stdout().flush().ok();
    tracing::info!(
        host = SSH_HOST,
        port = SSH_PORT,
        key = %key.display(),
        "remote-sidecar: sending boot command via SSH"
    );

    let status = Command::new("ssh")
        .args([
            "-i", &key_str,
            "-p", SSH_PORT,
            "-o", "StrictHostKeyChecking=no",
            "-o", "BatchMode=yes",
            &format!("{SSH_USER}@{SSH_HOST}"),
            REMOTE_BOOT_CMD,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("failed to invoke ssh — is OpenSSH in PATH?")?;

    if !status.success() {
        // Non-zero exit may mean the sidecar was already running (PID already
        // written) or a transient error. Log a warning but do not abort — the
        // health-poll in step 3 is the definitive gate.
        let code = status.code().unwrap_or(-1);
        println!("[remote-sidecar] step 1: SSH exited code={code} (sidecar may already be running — continuing)");
        tracing::warn!(
            exit_code = ?status.code(),
            "remote-sidecar: SSH boot command exited with non-zero status; \
             the sidecar may already be running — continuing to tunnel + poll"
        );
    } else {
        println!("[remote-sidecar] step 1: SSH boot dispatched OK");
        tracing::info!("remote-sidecar: SSH boot command dispatched successfully");
    }
    io::stdout().flush().ok();
    Ok(())
}

/// Step 2 — Tunnel: spawn start-tunnel.ps1 as a hidden, detached PowerShell.
fn start_ssh_tunnel() -> Result<(), anyhow::Error> {
    let script = tunnel_script_path()?;
    println!("[remote-sidecar] step 2: starting SSH tunnel via {}", script.display());
    io::stdout().flush().ok();
    tracing::info!(script = %script.display(), "remote-sidecar: starting SSH tunnel");

    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-WindowStyle", "Hidden",
        "-File",
        script.to_str().unwrap_or("start-tunnel.ps1"),
    ])
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .stdin(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS (0x00000008) + CREATE_NO_WINDOW (0x08000000)
        // so the tunnel runs silently in the background.
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
    }

    cmd.spawn()
        .context("failed to spawn start-tunnel.ps1 via PowerShell")?;

    println!("[remote-sidecar] step 2: tunnel process spawned");
    io::stdout().flush().ok();
    tracing::info!("remote-sidecar: tunnel process spawned");
    Ok(())
}

/// Step 3 — Health poll: POST to the Anthropic-protocol /v1/messages endpoint
/// with exponential backoff, max 120 s. The endpoint is used rather than
/// /health because the tunnel may forward only the Anthropic-protocol path.
fn poll_until_healthy(client: &reqwest::blocking::Client) -> Result<(), anyhow::Error> {
    // C-15: OsClock-backed `start` so the entropy scanner stays clean. Health
    // polling is wall-clock-driven (HEALTH_TIMEOUT_SECS budget) and is not part
    // of any deterministic test.
    let poll_clock = OsClock;
    let start = poll_clock.now();
    let timeout = Duration::from_secs(HEALTH_TIMEOUT_SECS);
    let mut wait_ms = 2_000u64;
    let mut attempt = 0u32;

    println!("[remote-sidecar] step 3: polling {HEALTH_URL} (max {HEALTH_TIMEOUT_SECS}s)...");
    io::stdout().flush().ok();

    loop {
        if poll_clock.elapsed_since(start) >= timeout {
            return Err(anyhow::anyhow!(
                "remote sidecar did not become healthy within {HEALTH_TIMEOUT_SECS}s; \
                 check ~/sidecar.log on the remote host"
            ));
        }

        let probe_body = serde_json::json!({
            "model": "qwen",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "ping"}]
        });

        let reachable = client
            .post(HEALTH_URL)
            .header("Content-Type", "application/json")
            .json(&probe_body)
            .send()
            .map(|r| {
                // 200 OK or 4xx (model error / bad request) both mean the
                // server is reachable. 5xx might mean the model is still
                // loading — treat as not-yet-ready.
                let s = r.status().as_u16();
                s < 500
            })
            .unwrap_or(false);

        if reachable {
            println!("\n[remote-sidecar] step 3: healthy after {:.1}s ({} attempts)", poll_clock.elapsed_since(start).as_secs_f32(), attempt);
            io::stdout().flush().ok();
            tracing::info!(
                elapsed_secs = poll_clock.elapsed_since(start).as_secs_f32(),
                attempts = attempt,
                "remote-sidecar: health poll passed"
            );
            return Ok(());
        }

        attempt += 1;
        print!(".");
        io::stdout().flush().ok();

        thread::sleep(Duration::from_millis(wait_ms));
        // Exponential backoff capped at 8 s.
        wait_ms = (wait_ms * 3 / 2).min(8_000);
    }
}

/// Entry point called by `CliAction::RemoteSidecar` in main.rs.
///
/// Executes the full boot sequence with idempotency guard:
///   diagnostics → (skip if already healthy) → ssh boot → tunnel → health poll
///   → set env → launch dashboard.
pub(crate) fn run_remote_sidecar() -> Result<(), anyhow::Error> {
    println!("[remote-sidecar] boot sequence starting");
    io::stdout().flush().ok();
    tracing::info!("remote-sidecar: boot sequence starting");

    // FIX 5: print port diagnostics so the user sees what's already running.
    print_port_diagnostics();

    // Build HTTP client once (reused for idempotency probe + health poll).
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .build()
        .context("failed to build HTTP client")?;

    // FIX 1: Idempotency guard — if 8100 is already open AND healthy, skip
    // SSH boot + tunnel start entirely.
    let already_healthy = if tcp_probe(8100) {
        println!("[remote-sidecar] port 8100 is open — probing sidecar health...");
        io::stdout().flush().ok();
        let h = http_probe_healthy(&client);
        if h {
            println!("[remote-sidecar] remote sidecar already healthy via existing tunnel — skipping SSH+tunnel");
            io::stdout().flush().ok();
        } else {
            println!("[remote-sidecar] port 8100 open but sidecar not healthy — will re-run boot");
            io::stdout().flush().ok();
        }
        h
    } else {
        false
    };

    if !already_healthy {
        // Step 1: SSH boot.
        if let Err(e) = ssh_boot_remote_sidecar() {
            eprintln!("[remote-sidecar] FAIL: SSH boot step failed: {e}");
            eprintln!("[remote-sidecar] Continuing — the sidecar may already be running.");
            tracing::warn!(error = %e, "remote-sidecar: SSH boot failed — will still attempt tunnel + poll");
        }

        // Brief pause to let the remote shell fork and background the process
        // before we try to open the tunnel. 2 s is enough for nohup to set up.
        thread::sleep(Duration::from_secs(2));

        // Step 2: SSH tunnel (only if port not already open).
        if tcp_probe(8100) {
            println!("[remote-sidecar] step 2: port 8100 already bound — skipping tunnel start");
            io::stdout().flush().ok();
        } else {
            if let Err(e) = start_ssh_tunnel() {
                eprintln!("[remote-sidecar] FAIL: tunnel start failed: {e}");
                return Err(e).context("remote-sidecar: failed to start SSH tunnel");
            }
            // Brief pause for the tunnel to negotiate the SSH handshake before poll.
            thread::sleep(Duration::from_secs(3));
        }

        // Step 3: Health poll.
        if let Err(e) = poll_until_healthy(&client) {
            eprintln!("[remote-sidecar] FAIL: health poll timed out: {e}");
            return Err(e).context("remote-sidecar: health check failed");
        }
    } else {
        println!("[remote-sidecar] step 3: skipped (already healthy)");
        io::stdout().flush().ok();
    }

    // Step 4: Export ANTHROPIC_BASE_URL so dashboard child process inherits it.
    // Single-threaded at this point (no other threads spawned yet), so set_var
    // is safe in practice.
    println!("[remote-sidecar] step 4: setting ANTHROPIC_BASE_URL=http://localhost:8100");
    io::stdout().flush().ok();
    env::set_var("ANTHROPIC_BASE_URL", "http://localhost:8100");
    tracing::info!("remote-sidecar: ANTHROPIC_BASE_URL=http://localhost:8100 set");

    // Step 5: Launch dashboard. Set ELAI_NO_SIDECAR=1 to prevent dashboard.rs
    // from attempting to spawn a local sidecar on top of the remote one.
    env::set_var("ELAI_NO_SIDECAR", "1");

    println!("[remote-sidecar] step 5: launching V2 dashboard...");
    io::stdout().flush().ok();
    tracing::info!("remote-sidecar: launching V2 dashboard");

    crate::dashboard::run_dashboard().context("remote-sidecar: dashboard launch failed")?;

    println!("[remote-sidecar] dashboard launched, you can close this window");
    io::stdout().flush().ok();
    tracing::info!("remote-sidecar: boot sequence complete");
    Ok(())
}
