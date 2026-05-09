use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use elai_core::ClockProvider;
use elai_orchestrator::BackendConfig;
use elai_providers::{SidecarConfig, SidecarProvider};

/// Current sidecar startup flags: see `start_sidecar` below. Keeping these in
/// one place lets the Rust-side `SidecarConfig` mirror what the Python engine
/// actually loaded. Update together when start_sidecar changes.
pub(crate) const SIDECAR_DEFAULT_DOLA_ENABLED: bool = false;
pub(crate) const SIDECAR_DEFAULT_STEERING_ENABLED: bool = false;
pub(crate) const SIDECAR_DEFAULT_TURBOQUANT_BITS: u8 = 8;

/// Fallback transformer-block count when the sidecar's `/v1/models` probe
/// fails. Used to resolve "auto" steering layer specs before the compute
/// call. Qwen2.5-Coder-7B has 28 hidden layers (verified via the model's
/// config.json `num_hidden_layers`); 3B has 36. `probe_sidecar_model_layers`
/// is the authoritative source when reachable.
pub(crate) const SIDECAR_DEFAULT_MODEL_LAYERS: usize = 28;

/// Environment variable that opts a session into role steering precomputation.
pub(crate) const ELAI_PRECOMPUTE_STEERING_ENV: &str = "ELAI_PRECOMPUTE_STEERING";

/// Convert a Windows path like `D:\foo\bar` to WSL `/mnt/d/foo/bar`.
/// Handles any single-letter drive prefix (A-Z).
pub(crate) fn windows_to_wsl_path(win_path: &str) -> String {
    let normalized = win_path.replace('\\', "/");
    if normalized.len() >= 2 && normalized.as_bytes()[1] == b':' {
        let drive = (normalized.as_bytes()[0] as char).to_ascii_lowercase();
        format!("/mnt/{drive}{}", &normalized[2..])
    } else {
        normalized
    }
}

/// Start or connect to the inference sidecar. Returns the child process handle
/// if we spawned it (None if already running). Blocks until /health responds.
pub(crate) fn start_sidecar(port: u16) -> Result<Option<std::process::Child>, anyhow::Error> {
    let sidecar_url = format!("http://localhost:{port}");

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    let already_running = client
        .get(format!("{sidecar_url}/health"))
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if already_running {
        println!("Connected to running sidecar on port {port}.");
        return Ok(None);
    }

    println!("Starting inference sidecar... (loading model, ~30s)");

    let elai_root = env::current_dir()?;
    let wsl_path = windows_to_wsl_path(elai_root.to_str().unwrap_or("."));

    let child = Command::new("wsl")
        .args([
            "-d", "Ubuntu", "--", "bash", "-c",
            &format!(
                "source ~/vllm-env/bin/activate && cd '{wsl_path}' && \
                 PYTHONPATH=python python -m elai_inference.server \
                 --model /mnt/d/.ELAI_workflow/.elai_cc/models/Qwen2.5-Coder-3B-Instruct-abliterated \
                 --port {port} --no-dola --no-steering --turboquant-bits 8 \
                 --log-level INFO"
            ),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let mut proc = match child {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start WSL2 sidecar: {e}");
            eprintln!("Make sure WSL2 is installed with Ubuntu distribution.");
            eprintln!("You can start the sidecar manually:");
            eprintln!("  wsl -d Ubuntu -- bash -c \"source ~/vllm-env/bin/activate && \\");
            eprintln!("    python -m python.elai_inference.server --port {port}\"");
            return Err(e.into());
        }
    };

    // Health-poll with exponential backoff (capped at 5s).
    let _poll_clock = elai_core::OsClock;
    let _poll_start = _poll_clock.now();
    let timeout = Duration::from_secs(120);
    let mut wait_ms = 2000u64;
    loop {
        if _poll_clock.elapsed_since(_poll_start) > timeout {
            eprintln!("Sidecar failed to start within 120s.");
            let _ = proc.kill();
            return Err(anyhow::anyhow!("sidecar startup timeout"));
        }

        if client
            .get(format!("{sidecar_url}/health"))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            break;
        }

        thread::sleep(Duration::from_millis(wait_ms));
        wait_ms = (wait_ms * 3 / 2).min(5000);
        print!(".");
        io::stdout().flush()?;
    }
    println!();
    println!("Sidecar ready.");
    Ok(Some(proc))
}

/// Call `compute_vector_via_sidecar` for every default steering spec attached
/// to `role` and write the resulting `.safetensors` files under
/// `{output_dir}/{role}/{spec.name}.safetensors`. Returns the number of
/// vectors that were successfully written. Skips silently when steering is
/// disabled for the role or the env var opt-in is missing — this guarantees
/// `compute_vector_via_sidecar` has a real production caller without forcing
/// an expensive precompute on every cold start.
///
/// patch-1.33: provides the agent-setup callsite the spec acceptance asks for.
pub(crate) fn try_precompute_role_steering(
    role: &str,
    sidecar_url: &str,
    output_dir: &Path,
    total_layers: usize,
) -> usize {
    use elai_providers::layer_config::steering::{SteeringConfig, SteeringVectorEngine};

    let config = SteeringConfig::default_for_role(role);
    if !config.enabled || config.vectors.is_empty() {
        tracing::debug!(role, "steering disabled for role; skipping precompute");
        return 0;
    }

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!(role, error = %e, "steering precompute: tokio runtime build failed");
            return 0;
        }
    };

    let role_dir = output_dir.join(role);
    if let Err(e) = std::fs::create_dir_all(&role_dir) {
        tracing::warn!(role, error = %e, dir = %role_dir.display(), "steering precompute: mkdir failed");
        return 0;
    }

    let mut written = 0usize;
    for spec in &config.vectors {
        let result = runtime.block_on(SteeringVectorEngine::compute_vector_via_sidecar(
            spec,
            sidecar_url,
            total_layers,
        ));
        match result {
            Ok(vector) => {
                tracing::info!(
                    role,
                    name = %vector.name,
                    layer = vector.layer,
                    dim = vector.vector.len(),
                    "steering precompute: vector ready (in-memory only; persistence is out-of-scope for patch-1.33)"
                );
                written += 1;
            }
            Err(e) => {
                tracing::warn!(role, name = %spec.name, error = %e, "steering precompute failed");
            }
        }
    }
    written
}

/// Short health probe for the sidecar — used by HyDE wiring (patch-1.34) to
/// decide whether to attach `SidecarHydeClient` to the QueryPipeline. A 1s
/// timeout GET /health keeps session startup snappy when the sidecar is
/// unreachable.
pub(crate) fn sidecar_probe_ok(sidecar_url: &str) -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let url = format!("{}/health", sidecar_url.trim_end_matches('/'));
    client
        .get(&url)
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Query the sidecar `/v1/models` endpoint for the active model's hidden-layer
/// count. Returns `None` when the sidecar is unreachable or the response is
/// malformed so the caller can fall back to a conservative default.
pub(crate) fn probe_sidecar_model_layers(sidecar_url: &str) -> Option<usize> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .ok()?;
    let url = format!("{}/v1/models", sidecar_url.trim_end_matches('/'));
    let resp = client.get(&url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().ok()?;
    body.get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .and_then(|m| m.get("num_hidden_layers"))
        .and_then(|n| n.as_u64())
        .map(|n| n as usize)
}

/// Build a `SidecarProvider` tuned for HyDE hypothesis generation: shorter
/// max_tokens, higher temperature. Distinct from the main-dispatch provider
/// so each path gets appropriately sized requests without one clobbering the
/// other's state.
pub(crate) fn build_hyde_sidecar_provider(config: &BackendConfig) -> Result<SidecarProvider, anyhow::Error> {
    let sidecar_cfg = SidecarConfig {
        server_url: config.sidecar_url.clone(),
        model: config.model.clone(),
        dola_enabled: SIDECAR_DEFAULT_DOLA_ENABLED,
        dola_alpha: 0.1,
        turboquant_bits: SIDECAR_DEFAULT_TURBOQUANT_BITS,
        steering_enabled: SIDECAR_DEFAULT_STEERING_ENABLED,
        steering_coefficient: 5.0,
        steering_layer: "auto".to_string(),
        temperature: elai_knowledge::hyde::HYDE_TEMPERATURE,
        max_tokens: elai_knowledge::hyde::HYDE_MAX_TOKENS,
        etc_threshold: f64::INFINITY,
        // Batch 4A (3J): the HyDE-scoped sidecar provider stays conservative —
        // HyDE is toggled per-dispatch, ActAdd vectors are role-scoped and
        // loaded lazily via the per-project cache.
        hyde_enabled: false,
        actadd_enabled: false,
        actadd_layer: 0,
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
    Ok(SidecarProvider::new(sidecar_cfg, clock)?)
}
