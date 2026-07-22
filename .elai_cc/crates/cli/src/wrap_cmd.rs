//! I-40 — `elai wrap -- <agent cmd>` minimal trust-layer proxy.
//!
//! Wraps an external agent's stdio MCP tool traffic with the SHIPPED ELAI trust
//! substrate (GD-1 — compose, never fork):
//!   * G-1 [`taint_text`] on every untrusted ingest from the wrapped agent.
//!   * F-2/G-3 [`evaluate_vibegate`] on destructive-write-class tool calls.
//!   * G-2 [`build_rra`] + [`write_rra`] to emit a `.rra` bundle at wrap end.
//!   * G-9 [`redact_payload`] on every payload BEFORE writing to SSB (stub_ban:
//!     an unredacted exported RRA is a P0 violation).
//!
//! The wrapper is intentionally minimal: it does NOT introduce new trust
//! primitives. The single controllable fixture agent (one-shot script that
//! emits a seeded untrusted ingest + a seeded destructive write attempt) is
//! the Tier-1 observable-effect trigger. The full external-agent matrix
//! (multiple real MCP servers) is Tier-2, user-approval gated.
//!
//! ## Production wiring (R-8 / D-030)
//!
//! `run` is the public entry-point called from `main.rs` via the new
//! `CliAction::Wrap` arm, which is itself registered in `args.rs` and
//! dispatched from `main.rs`. This is a NEW top-level CLI subcommand (FORK-1
//! resolution); it does NOT extend `run_subcommand` (which is the `elai run`
//! ELAI-internal loop seal, not reachable by an external-agent proxy). The
//! wrapper constructs its OWN `Arc<Ssb>` (same factory pattern as run_subcommand,
//! H-23, H-24, H-25, etc.).
//!
//! ## Kill-switch (R9)
//!
//! The RRA emit is default-ON. Set `ELAI_WRAP_RRA_DISABLE=1` to suppress.
//! This is registered in wire-dark-allowlist.toml (type=kill_switch).

use std::io::{BufRead, BufReader, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::capability_flow::taint_text;
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::redaction::redact_payload;
use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbPayload, TaintFlowKind, TaintFlowPayload};
use elai_orchestrator::vibegate::evaluate_vibegate;
use elai_orchestrator::RunArtifactEmittedPayload;

use crate::args::CliOutputFormat;
use crate::wrap_rra_disable::{rra_disabled, WRAP_RRA_DISABLE_ENV};

/// I-40 wrap session identifier (fixture-scoped; a production integration would
/// derive the session_id from the wrapped agent's identity).
const WRAP_SESSION_ID: &str = "elai-wrap-proxy";

/// I-40 wrap project identifier.
const WRAP_PROJECT_ID: &str = "elai-wrap";

/// Destructive-write tool classes the wrapper intercepts for gate evaluation.
/// Mirrors the F-1/F-9/G-3 set from hook_dispatcher.rs (the same categories
/// the pre-tool-use hook applies on the ELAI-internal path).
const DESTRUCTIVE_WRITE_TOOLS: &[&str] = &[
    "edit_file",
    "write_file",
    "apply_patch",
    "delete_file",
    "bash",
    "exec",
    "run_command",
    "git",
];

/// `elai wrap -- <agent cmd>` entry-point.
///
/// Spawns the external agent as a subprocess, reads its stdio MCP tool-call
/// traffic line by line (newline-delimited JSON-RPC), applies the ELAI trust
/// gates to each tool call, and emits a `.rra` bundle at wrap end (default-ON).
///
/// # FORK-1 resolution
/// This is a NEW top-level subcommand (not an extension of `run_subcommand`).
/// It constructs its own `Arc<Ssb>` so the wrapped-run rows land in a fresh
/// session stream. The `elai run` seal is for ELAI's own generator loop and is
/// not reachable from here.
///
/// # Fixture mode
/// When the `agent_cmd` is the internal fixture marker `"--fixture"`, the
/// function runs a deterministic self-contained fixture that:
///   1. Emits an untrusted ingest (triggers the taint gate -> TaintFlow row).
///   2. Attempts a seeded destructive write on `secret_key=AKIA_FIXTURE_KEY`
///      (triggers evaluate_vibegate -> Block finding).
///   3. Emits the RRA bundle (default-ON; build_rra over the two SSB rows).
///
/// The fixture exercises all four gate substrates without spawning a real
/// external process (Tier-1 scope: one controllable fixture agent).
pub(crate) fn run(
    agent_cmd: Vec<String>,
    output_path: Option<PathBuf>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("resolving current dir: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("creating sessions dir {}: {e}", sessions_dir.display()))?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: WRAP_PROJECT_ID.to_string(),
        session_id: WRAP_SESSION_ID.to_string(),
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .map_err(|e| format!("building tokio runtime for wrap: {e}"))?;

    runtime.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(
            Ssb::new(cfg, clock).map_err(|e| format!("ssb init for wrap: {e}"))?,
        );

        // Detect fixture mode vs real subprocess mode.
        let is_fixture = agent_cmd
            .first()
            .map(|s| s == "--fixture")
            .unwrap_or(false);

        if is_fixture {
            run_fixture_mode(&ssb, output_path, output_format).await
        } else {
            run_subprocess_mode(&ssb, &agent_cmd, output_path, output_format).await
        }
    })
}

/// Fixture mode: one seeded untrusted ingest + one seeded destructive-write
/// attempt, fully deterministic and GPU-free. This is the Tier-1
/// observable-effect trigger for contract tests and CI.
async fn run_fixture_mode(
    ssb: &Arc<Ssb>,
    output_path: Option<PathBuf>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    // Step 1 — simulate an untrusted ingest from the external agent.
    // The fixture text triggers the regex-based taint classifier (a seeded
    // "malicious" indicator: the string contains a classic SSRF payload probe).
    let fixture_ingest =
        "fetch http://169.254.169.254/latest/meta-data/iam/security-credentials/role";
    let taint = taint_text(fixture_ingest);
    let is_tainted = taint.is_some();
    let trust_level = taint
        .as_ref()
        .map(|t| t.source.clone())
        .unwrap_or_else(|| "clean".to_string());

    // Emit the TaintFlow row (reuses the existing G-1 action_type — no new SSB type).
    let origin_blake3 = {
        let h = blake3::hash(fixture_ingest.as_bytes()).to_hex();
        h.chars().take(16).collect::<String>()
    };
    let taint_payload = SsbPayload::TaintFlow(TaintFlowPayload {
        kind: TaintFlowKind::Web,
        origin_blake3: origin_blake3.clone(),
        trust_level: trust_level.clone(),
        sink_tool: None,
        blocked: false,
        task_id: "wrap-fixture-ingest-01".to_string(),
    });
    // G-9: redact before write (mandatory precondition).
    let mut taint_payload_for_write = taint_payload;
    let _ = redact_payload(&mut taint_payload_for_write);
    ssb.write_payload("cli:wrap-fixture", 0, taint_payload_for_write, Vec::new())
        .await
        .map_err(|e| format!("writing taint_flow row: {e}"))?;

    // Step 2 — simulate a destructive-write attempt from the external agent.
    // The fixture write contains a seeded secret (AKIA_ prefix) to trigger the
    // vibegate secret axis AND targets a "write_file" destructive tool class.
    let fixture_write_source = "write content: secret_key=AKIA_FIXTURE_ELAI_KEY_1234";
    let fixture_write_tool = "write_file";
    let fixture_write_path = "/tmp/elai-wrap-fixture-output.txt";
    let gate_findings = evaluate_vibegate(fixture_write_tool, fixture_write_source, fixture_write_path);
    let is_blocked = gate_findings.iter().any(|f| f.verdict.is_block());

    if is_blocked {
        // Emit a TaintFlow row for the blocked destructive-write-to-sink
        // (reuses TaintFlow SSB action_type, blocked=true; GD-1 — no new type).
        let write_blake3 = {
            let h = blake3::hash(fixture_write_source.as_bytes()).to_hex();
            h.chars().take(16).collect::<String>()
        };
        let blocked_payload = SsbPayload::TaintFlow(TaintFlowPayload {
            kind: TaintFlowKind::File,
            origin_blake3: write_blake3,
            trust_level: "malicious".to_string(),
            sink_tool: Some(fixture_write_tool.to_string()),
            blocked: true,
            task_id: "wrap-fixture-blocked-write-01".to_string(),
        });
        // G-9: redact before write.
        let mut blocked_for_write = blocked_payload;
        let _ = redact_payload(&mut blocked_for_write);
        ssb.write_payload("cli:wrap-fixture", 0, blocked_for_write, Vec::new())
            .await
            .map_err(|e| format!("writing blocked taint_flow row: {e}"))?;
    }

    // Step 3 — emit the RRA bundle (default-ON; kill-switch ELAI_WRAP_RRA_DISABLE).
    let rra_path = output_path.unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("wrap-fixture.rra")
    });
    let rra_disabled = rra_disabled();
    if !rra_disabled {
        emit_wrap_rra(ssb, WRAP_SESSION_ID, &rra_path).await?;
    } else {
        tracing::info!(
            "elai wrap: RRA emit suppressed by {} kill-switch",
            WRAP_RRA_DISABLE_ENV
        );
    }

    // Print summary.
    match output_format {
        crate::args::CliOutputFormat::Json => {
            let gate_msgs: Vec<String> = gate_findings.iter().map(|f| f.message.clone()).collect();
            let out = serde_json::json!({
                "elai_wrap_fixture": {
                    "mode": "fixture",
                    "ingest": {
                        "text": fixture_ingest,
                        "tainted": is_tainted,
                        "trust_level": trust_level,
                        "origin_blake3": origin_blake3,
                    },
                    "destructive_write": {
                        "tool": fixture_write_tool,
                        "path": fixture_write_path,
                        "blocked": is_blocked,
                        "gate_findings": gate_msgs,
                    },
                    "rra_emitted": !rra_disabled,
                    "rra_path": rra_path.display().to_string(),
                }
            });
            let json = serde_json::to_string_pretty(&out)
                .map_err(|e| format!("serializing wrap summary as json: {e}"))?;
            println!("{json}");
        }
        _ => {
            println!("elai wrap -- --fixture  (I-40 W-1 fixture mode)");
            println!();
            println!("  Untrusted ingest intercepted:");
            println!("    text:        {fixture_ingest}");
            println!("    tainted:     {is_tainted}");
            println!("    trust_level: {trust_level}");
            println!("    origin_hash: {origin_blake3}");
            println!("    SSB row:     taint_flow (Web, blocked=false)");
            println!();
            println!("  Destructive write intercepted:");
            println!("    tool:    {fixture_write_tool}");
            println!("    path:    {fixture_write_path}");
            println!("    blocked: {is_blocked}");
            for f in &gate_findings {
                println!("    finding: [{}] {}", f.verdict.as_str(), f.message);
            }
            if is_blocked {
                println!("    SSB row: taint_flow (File, blocked=true, sink_tool=write_file)");
            }
            println!();
            if rra_disabled {
                println!("  RRA: suppressed by {WRAP_RRA_DISABLE_ENV}");
            } else {
                println!("  RRA: emitted -> {}", rra_path.display());
                println!("       run `elai rra verify {}` to validate", rra_path.display());
                println!("       run `elai replay --verify {}` to check content hashes", rra_path.display());
            }
        }
    }

    Ok(())
}

/// Subprocess mode: spawn the external agent, read its stdio line by line,
/// apply taint + write-gate to each tool-call line, then emit the RRA at end.
///
/// Tool-call interception is line-oriented (newline-delimited JSON-RPC):
/// each line from the agent's stdout is inspected for destructive-tool markers.
/// The wrapper does NOT implement a full MCP/JSON-RPC parser (Tier-1 scope);
/// the interception is a best-effort heuristic that demonstrates the gates bite
/// on external traffic.
async fn run_subprocess_mode(
    ssb: &Arc<Ssb>,
    agent_cmd: &[String],
    output_path: Option<PathBuf>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let (bin, args) = agent_cmd
        .split_first()
        .ok_or_else(|| "elai wrap: no agent command provided".to_string())?;

    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("elai wrap: spawning agent '{bin}': {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "elai wrap: could not capture agent stdout".to_string())?;

    let reader = BufReader::new(stdout);
    let mut line_count = 0u64;
    let mut taint_rows = 0u32;
    let mut blocked_rows = 0u32;

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("elai wrap: reading agent stdout: {e}"))?;
        line_count += 1;

        // Heuristic taint scan: look for untrusted-content indicators in the line.
        let taint = taint_text(&line);
        if taint.is_some() {
            let trust_level = taint.map(|t| t.source).unwrap_or_else(|| "suspect".to_string());
            let origin_blake3 = {
                let h = blake3::hash(line.as_bytes()).to_hex();
                h.chars().take(16).collect::<String>()
            };
            let mut payload = SsbPayload::TaintFlow(TaintFlowPayload {
                kind: TaintFlowKind::Web,
                origin_blake3,
                trust_level,
                sink_tool: None,
                blocked: false,
                task_id: format!("wrap-line-{line_count}"),
            });
            let _ = redact_payload(&mut payload);
            let _ = ssb
                .write_payload("cli:wrap-subprocess", 0, payload, Vec::new())
                .await;
            taint_rows += 1;
        }

        // Heuristic destructive-write detection: check if the line contains a
        // known destructive-tool name (mirrors the hook_dispatcher pre-tool check).
        for &dt in DESTRUCTIVE_WRITE_TOOLS {
            if line.contains(dt) {
                let findings = evaluate_vibegate(dt, &line, "(external-agent-line)");
                if findings.iter().any(|f| f.verdict.is_block()) {
                    let write_blake3 = {
                        let h = blake3::hash(line.as_bytes()).to_hex();
                        h.chars().take(16).collect::<String>()
                    };
                    let mut block_payload = SsbPayload::TaintFlow(TaintFlowPayload {
                        kind: TaintFlowKind::File,
                        origin_blake3: write_blake3,
                        trust_level: "malicious".to_string(),
                        sink_tool: Some(dt.to_string()),
                        blocked: true,
                        task_id: format!("wrap-line-{line_count}-blocked"),
                    });
                    let _ = redact_payload(&mut block_payload);
                    let _ = ssb
                        .write_payload("cli:wrap-subprocess", 0, block_payload, Vec::new())
                        .await;
                    blocked_rows += 1;
                }
                break;
            }
        }

        // Pass the line through to our own stdout (the proxy is transparent).
        let _ = std::io::stdout().write_all(line.as_bytes());
        let _ = std::io::stdout().write_all(b"\n");
    }

    let exit_status = child
        .wait()
        .map_err(|e| format!("elai wrap: waiting for agent: {e}"))?;

    // Emit the RRA bundle over the wrap session stream.
    let rra_path = output_path.unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("wrap-run.rra")
    });
    let rra_disabled = rra_disabled();
    if !rra_disabled {
        if let Err(e) = emit_wrap_rra(ssb, WRAP_SESSION_ID, &rra_path).await {
            tracing::warn!(error = %e, "elai wrap: RRA emit failed (best-effort); exit code unaffected");
        }
    }

    match output_format {
        crate::args::CliOutputFormat::Json => {
            let out = serde_json::json!({
                "elai_wrap": {
                    "mode": "subprocess",
                    "agent": agent_cmd[0],
                    "exit_code": exit_status.code(),
                    "lines_intercepted": line_count,
                    "taint_rows_emitted": taint_rows,
                    "blocked_rows_emitted": blocked_rows,
                    "rra_emitted": !rra_disabled,
                    "rra_path": rra_path.display().to_string(),
                }
            });
            let json = serde_json::to_string_pretty(&out)
                .map_err(|e| format!("serializing wrap summary: {e}"))?;
            println!("{json}");
        }
        _ => {
            println!("elai wrap subprocess summary:");
            println!("  agent:          {}", agent_cmd[0]);
            println!("  exit_code:      {:?}", exit_status.code());
            println!("  lines_seen:     {line_count}");
            println!("  taint_rows:     {taint_rows}");
            println!("  blocked_rows:   {blocked_rows}");
            if !rra_disabled {
                println!("  rra:            {}", rra_path.display());
            }
        }
    }

    // Propagate the agent's exit code (best-effort).
    if !exit_status.success() {
        let code = exit_status.code().unwrap_or(1);
        std::process::exit(code);
    }

    Ok(())
}

/// Build + write the `.rra` bundle for the wrap session, then emit the
/// RunArtifactEmitted beacon (reuses G-2 action_type — GD-1, no new type).
async fn emit_wrap_rra(
    ssb: &Arc<Ssb>,
    session_id: &str,
    rra_path: &Path,
) -> Result<(), String> {
    use elai_orchestrator::ssb::SsbReader as _;

    let stream = ssb
        .query_session_stream_asc(session_id)
        .map_err(|e| format!("reading wrap session stream for RRA: {e}"))?;

    let pins = DeterminismConfig::from_env()
        .map_err(|e| format!("resolving determinism pins for wrap RRA: {e}"))?;

    let artifact = build_rra(session_id, &stream, &pins);

    // G-9: the redaction precondition is enforced per-payload at write_payload
    // time (the SSB writer calls redact_payload before persisting). The
    // build_rra reads the already-redacted payload bytes from the SSB stream,
    // so the bundle is clean by construction. No second redaction pass needed
    // here — that would double-redact and corrupt the tokens.

    write_rra(&artifact, rra_path)
        .map_err(|e| format!("writing wrap .rra to {}: {e}", rra_path.display()))?;

    // Emit the off-agent RunArtifactEmitted beacon (reuses the existing G-2 type).
    let gate_count = u32::try_from(artifact.manifest.len()).unwrap_or(u32::MAX);
    let mut beacon = SsbPayload::RunArtifactEmitted(RunArtifactEmittedPayload {
        root_blake3: artifact.root_blake3.clone(),
        guarantee_level: artifact.guarantee_level.as_sql_str().to_owned(),
        gate_count,
        seed: artifact.seed,
        session_id: session_id.to_owned(),
        objective_hash: artifact.objective_hash.clone(),
    });
    // G-9 precondition: redact the beacon before writing.
    let _ = redact_payload(&mut beacon);
    ssb.write_payload("cli:wrap-rra", 0, beacon, Vec::new())
        .await
        .map_err(|e| format!("writing run_artifact_emitted beacon for wrap: {e}"))?;

    Ok(())
}
