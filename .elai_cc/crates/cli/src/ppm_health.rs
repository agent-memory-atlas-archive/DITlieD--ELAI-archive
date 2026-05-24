//! D-2: `elai ppm health` — three-step PPM sidecar health probe.
//!
//! Mirrors the 3-step pattern from REMOTE-SIDECAR.md (and the existing
//! `sidecar-health` agent at .claude/agents/sidecar-health.md):
//!
//!   1. TCP probe of the configured port (default 8101). Confirms tunnel /
//!      local sidecar listens.
//!   2. `GET <endpoint>/health` — confirms FastAPI is up and the model
//!      load_status is reported. Surfaces `model_id` if exposed.
//!   3. `POST <endpoint>/v1/ppm/score` with a tiny one-step canary payload.
//!      End-to-end round trip through the wire contract; catches a stale
//!      / wrong-model / shape-misconfigured sidecar that step 2 wouldn't.
//!
//! Exit code 0 = all 3 steps OK. Exit code 1 = any step fails. JSON output
//! (via `--json`) mirrors `archive verify --json` shape so CI can consume.
//!
//! Owns its own tokio runtime (sync entrypoint pattern, same as
//! `archive_verify::run`).

use std::net::TcpStream;
use std::time::Duration;

use elai_orchestrator::{
    HttpSidecarPpmHook, PpmHookError, PpmMode, PpmScoreRequest, SidecarPpmHook,
    PPM_DEFAULT_BASE_URL, PPM_DEFAULT_MAX_RETRIES, PPM_DEFAULT_TIMEOUT_SECS,
};
use serde::Serialize;

/// Canonical default port for the PPM sidecar (matches
/// `http_sidecar_ppm_hook::PPM_DEFAULT_BASE_URL`).
const PPM_DEFAULT_PORT: u16 = 8101;

/// Short timeout for the TCP probe (step 1). The full request timeout is
/// configurable via `--timeout-secs`; step 1 only checks the port is open.
const TCP_PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// JSON report emitted via `--json`. Mirrors the `archive verify --json`
/// shape: a single object with per-step booleans + per-step detail.
#[derive(Debug, Serialize)]
struct PpmHealthReport {
    overall: &'static str,
    endpoint: String,
    tcp_open: bool,
    health_ok: bool,
    health_status: Option<String>,
    health_model_id: Option<String>,
    health_load_status: Option<String>,
    score_ok: bool,
    score_detail: Option<String>,
    failure_reason: Option<String>,
}

/// Entry point. Owns a tokio current-thread runtime so the call stays sync
/// from the CLI dispatcher (mirrors `archive_verify::run`).
pub fn run(
    endpoint: Option<String>,
    timeout_secs: Option<u64>,
    json: bool,
) -> anyhow::Result<()> {
    let base_url = endpoint.unwrap_or_else(|| PPM_DEFAULT_BASE_URL.to_string());
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(PPM_DEFAULT_TIMEOUT_SECS));

    // Build runtime + probe. Errors flow through `anyhow::Error` so the
    // top-level dispatcher (main.rs run() -> anyhow::Result) can apply `?`
    // without `Sync` / `Sized` complaints on `Box<dyn StdError>`.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let report = runtime.block_on(probe(base_url, timeout));

    if json {
        let serialized = serde_json::to_string_pretty(&report)?;
        println!("{serialized}");
    } else {
        print_human(&report);
    }

    if report.overall != "ok" {
        // Surface non-zero exit via std::process::exit; the CLI dispatcher
        // expects the handler to either return Ok or short-circuit the
        // process on failure (same as `archive_verify::run`).
        std::process::exit(1);
    }
    Ok(())
}

/// Async core. Steps 1-3 in order; the first failure short-circuits.
async fn probe(base_url: String, timeout: Duration) -> PpmHealthReport {
    let mut report = PpmHealthReport {
        overall: "ok",
        endpoint: base_url.clone(),
        tcp_open: false,
        health_ok: false,
        health_status: None,
        health_model_id: None,
        health_load_status: None,
        score_ok: false,
        score_detail: None,
        failure_reason: None,
    };

    // Step 1 — TCP probe.
    let port = extract_port(&base_url).unwrap_or(PPM_DEFAULT_PORT);
    let host = extract_host(&base_url).unwrap_or_else(|| "127.0.0.1".to_string());
    report.tcp_open = tcp_probe(&host, port, TCP_PROBE_TIMEOUT);
    if !report.tcp_open {
        report.overall = "fail";
        report.failure_reason = Some(format!("step 1: TCP {host}:{port} closed"));
        return report;
    }

    // Step 2 — /health GET.
    let hook = HttpSidecarPpmHook::with_endpoint(
        base_url.clone(),
        timeout,
        // Disable retries for health probes; we want the operator to see
        // the immediate failure without 3 backoff cycles.
        0,
    );
    match hook.health().await {
        Ok(h) => {
            report.health_ok = true;
            report.health_status = Some(h.status);
            report.health_model_id = h.model_id;
            report.health_load_status = h.load_status;
        }
        Err(err) => {
            report.overall = "fail";
            report.failure_reason = Some(format!("step 2: /health failed: {err}"));
            return report;
        }
    }

    // Step 3 — canary /v1/ppm/score POST (one-step, deterministic shape).
    let req = PpmScoreRequest {
        candidate_id: "elai-ppm-health-canary".into(),
        verifier_id: "elai-ppm-health-canary".into(),
        problem_statement: "ELAI PPM health canary".into(),
        reasoning_steps: vec!["1 == 1".into()],
        mode: PpmMode::Math,
    };
    match hook.score(req).await {
        Ok(resp) => {
            report.score_ok = true;
            report.score_detail = Some(format!(
                "score_mean={:.3} num_steps={} latency_ms={}",
                resp.score_mean, resp.num_steps, resp.latency_ms
            ));
        }
        Err(PpmHookError::Unavailable) => {
            report.overall = "fail";
            report.failure_reason = Some(
                "step 3: /v1/ppm/score reported Unavailable (sidecar not loaded?)"
                    .to_string(),
            );
        }
        Err(other) => {
            report.overall = "fail";
            report.failure_reason = Some(format!("step 3: /v1/ppm/score failed: {other}"));
        }
    }
    let _ = PPM_DEFAULT_MAX_RETRIES; // retain anchor reference for grep
    report
}

/// Open a TCP connection with a bounded timeout. Returns true if the
/// socket connects. Used by step 1 to confirm the tunnel / local sidecar
/// is listening before paying HTTP latency for a 404.
fn tcp_probe(host: &str, port: u16, timeout: Duration) -> bool {
    let addr = format!("{host}:{port}");
    match addr.parse() {
        Ok(socket) => TcpStream::connect_timeout(&socket, timeout).is_ok(),
        Err(_) => {
            // Hostname-form (e.g. localhost) — fall back to a regular
            // connect (no timeout knob on `connect`, but the resolver is
            // local + fast).
            TcpStream::connect(&addr).is_ok()
        }
    }
}

/// Crude `host` extractor that handles `http://host:port[/path]` and
/// `host:port` shapes. Returns `None` if no port is present.
fn extract_port(url: &str) -> Option<u16> {
    let after_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host_port = after_scheme.split('/').next().unwrap_or(after_scheme);
    let port_str = host_port.rsplit(':').next()?;
    port_str.parse().ok()
}

fn extract_host(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host_port = after_scheme.split('/').next().unwrap_or(after_scheme);
    let host = host_port.rsplit_once(':').map(|(h, _)| h).unwrap_or(host_port);
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

fn print_human(report: &PpmHealthReport) {
    println!("PPM sidecar health probe ({})", report.endpoint);
    println!(
        "  step 1: TCP probe        : {}",
        if report.tcp_open { "OPEN" } else { "CLOSED" }
    );
    println!(
        "  step 2: /health GET      : {}",
        if report.health_ok {
            format!(
                "OK (status={}{}{})",
                report.health_status.as_deref().unwrap_or("?"),
                report
                    .health_model_id
                    .as_deref()
                    .map(|m| format!(", model={m}"))
                    .unwrap_or_default(),
                report
                    .health_load_status
                    .as_deref()
                    .map(|s| format!(", load={s}"))
                    .unwrap_or_default(),
            )
        } else {
            "FAIL".to_string()
        }
    );
    println!(
        "  step 3: /v1/ppm/score    : {}",
        if report.score_ok {
            format!("OK ({})", report.score_detail.as_deref().unwrap_or(""))
        } else {
            "FAIL".to_string()
        }
    );
    println!(
        "Overall: {}",
        if report.overall == "ok" {
            "OK"
        } else {
            "FAIL"
        }
    );
    if let Some(reason) = &report.failure_reason {
        println!("Reason : {reason}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_port_parses_standard_url() {
        assert_eq!(
            extract_port("http://127.0.0.1:8101/v1/ppm/score"),
            Some(8101)
        );
        assert_eq!(extract_port("http://localhost:9999"), Some(9999));
        assert_eq!(extract_port("127.0.0.1:8101"), Some(8101));
    }

    #[test]
    fn extract_port_returns_none_when_missing() {
        assert_eq!(extract_port("http://127.0.0.1"), None);
        assert_eq!(extract_port("localhost"), None);
    }

    #[test]
    fn extract_host_parses_standard_url() {
        assert_eq!(
            extract_host("http://127.0.0.1:8101/v1/ppm/score"),
            Some("127.0.0.1".to_string())
        );
        assert_eq!(
            extract_host("http://localhost:9999"),
            Some("localhost".to_string())
        );
    }

    #[test]
    fn tcp_probe_on_closed_port_returns_false() {
        // Port 1 is reserved + closed on every modern host.
        assert!(!tcp_probe("127.0.0.1", 1, Duration::from_millis(50)));
    }

    #[tokio::test]
    async fn probe_against_closed_port_fails_at_step_1() {
        let report = probe(
            "http://127.0.0.1:1".to_string(),
            Duration::from_millis(200),
        )
        .await;
        assert_eq!(report.overall, "fail");
        assert!(!report.tcp_open);
        assert!(report.failure_reason.unwrap().contains("TCP"));
    }
}
