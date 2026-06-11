//! REPL entry points: standard (`run_repl`) and local-sidecar (`run_local_repl`).
//!
//! Extracted from main.rs in B-15D Phase A4.

use std::time::Duration;

/// H-51 TieredLoad gate: the CLI inject path must not push a digest that exceeds
/// this token estimate.  Mirrors the dashboard gate in commands_h51.rs and the
/// WIRING_CONTRACT stub_ban: "a resume that loads >5K by default = FAIL."
const MAX_TIERED_LOAD_TOKENS: u32 = 5_000;

use elai_orchestrator::{BackendConfig, BackendMode, PermissionMode};

use crate::commands::SlashCommand;
use crate::input;
use crate::{classifier, sidecar};
use crate::{
    AllowedToolSet, LiveCli,
    DEGRADED_PROMPT_PREFIX, OLLAMA_DEFAULT_URL, OLLAMA_PROBE_TIMEOUT,
    default_permission_mode,
};

pub(crate) fn run_repl(
    model: String,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    export_jsonl: bool,
) -> Result<(), anyhow::Error> {
    let mut cli = LiveCli::new_with_options(
        model,
        true,
        allowed_tools,
        permission_mode,
        export_jsonl,
    )?;
    let mut editor =
        input::LineEditor::new("> ", cli.repl_completion_candidates().unwrap_or_default());
    println!("{}", cli.startup_banner());

    // H-51: detect an open handoff topic and offer one-tap resume.
    // Guard: ELAI_HANDOFF_AUTORESUME_DISABLE=1 suppresses this offer.
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Some(handle) = elai_orchestrator::handoff_digest::detect_open_topic(&cwd) {
        println!(
            "[H-51] Open topic detected: '{}' (digest: {}). Resume? [y/N]",
            handle.topic_slug,
            handle.digest_path.display()
        );
        // Emit ResumeOffered SSB row (best-effort; non-fatal on failure).
        if let Some(ssb) = cli.ssb.as_ref() {
            let payload = elai_orchestrator::ssb::SsbPayload::HandoffLifecycleEvent(
                elai_orchestrator::ssb::HandoffLifecyclePayload {
                    kind: elai_orchestrator::ssb::HandoffLifecycleEventKind::ResumeOffered,
                    topic_slug: handle.topic_slug.clone(),
                    digest_path: Some(handle.digest_path.to_string_lossy().into_owned()),
                    loaded_tokens: None,
                    session_id: cli.session.id.clone(),
                },
            );
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                let _ = rt.block_on(ssb.write_payload(
                    "repl-startup",
                    0,
                    payload,
                    vec![],
                ));
            }
        }
        // Read one-line y/N response from stdin (non-interactive-safe: if stdin
        // is not a tty the read returns empty / error and we skip).
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_ok() {
            if answer.trim().eq_ignore_ascii_case("y") {
                // Emit ResumeConfirmed + load digest as context preamble.
                if let Some(ssb) = cli.ssb.as_ref() {
                    let confirm_payload =
                        elai_orchestrator::ssb::SsbPayload::HandoffLifecycleEvent(
                            elai_orchestrator::ssb::HandoffLifecyclePayload {
                                kind: elai_orchestrator::ssb::HandoffLifecycleEventKind::ResumeConfirmed,
                                topic_slug: handle.topic_slug.clone(),
                                digest_path: Some(
                                    handle.digest_path.to_string_lossy().into_owned(),
                                ),
                                loaded_tokens: Some(handle.loaded_tokens),
                                session_id: cli.session.id.clone(),
                            },
                        );
                    let tiered_payload =
                        elai_orchestrator::ssb::SsbPayload::HandoffLifecycleEvent(
                            elai_orchestrator::ssb::HandoffLifecyclePayload {
                                kind: elai_orchestrator::ssb::HandoffLifecycleEventKind::TieredLoad,
                                topic_slug: handle.topic_slug.clone(),
                                digest_path: Some(
                                    handle.digest_path.to_string_lossy().into_owned(),
                                ),
                                loaded_tokens: Some(handle.loaded_tokens),
                                session_id: cli.session.id.clone(),
                            },
                        );
                    if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        let _ = rt.block_on(ssb.write_payload(
                            "repl-startup",
                            0,
                            confirm_payload,
                            vec![],
                        ));
                        let _ = rt.block_on(ssb.write_payload(
                            "repl-startup",
                            0,
                            tiered_payload,
                            vec![handle.digest_path.to_string_lossy().into_owned()],
                        ));
                    }
                }
                // Inject digest content as an additional system prompt entry so
                // the first turn benefits from cross-session context (tiered load).
                // Guard: skip and warn if the digest has grown beyond the TieredLoad
                // token cap (5000).  A bloated digest defeats tiered load and violates
                // the WIRING_CONTRACT stub_ban (R1/R4).
                if handle.loaded_tokens > MAX_TIERED_LOAD_TOKENS {
                    eprintln!(
                        "[H-51] TieredLoad gate: digest for topic '{}' is {} tokens \
                         (> {MAX_TIERED_LOAD_TOKENS}); skipping CLI inject to preserve \
                         tiered-load invariant. Use /continue to load on demand.",
                        handle.topic_slug, handle.loaded_tokens
                    );
                } else if let Ok(digest_text) = std::fs::read_to_string(&handle.digest_path) {
                    let preamble = format!(
                        "[H-51 Tiered Load] Resuming topic '{}'. Prior session digest follows:\n\n{}",
                        handle.topic_slug, digest_text
                    );
                    cli.system_prompt.push(preamble);
                }
            }
        }
    }

    loop {
        editor.set_completions(cli.repl_completion_candidates().unwrap_or_default());
        match editor.read_line()? {
            input::ReadOutcome::Submit(input) => {
                let trimmed = input.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                if matches!(trimmed.as_str(), "/exit" | "/quit") {
                    cli.persist_session()?;
                    break;
                }
                match SlashCommand::parse(&trimmed) {
                    Ok(Some(command)) => {
                        if cli.handle_repl_command(command)? {
                            cli.persist_session()?;
                        }
                        continue;
                    }
                    Ok(None) => {
                        // Bare message: run fallback classifier for routing.
                        let routed = classifier::classify_bare_message(&trimmed, &[]);
                        match routed {
                            SlashCommand::Research { .. } => {
                                // Route to research mode via handle_repl_command.
                                if cli.handle_repl_command(SlashCommand::Research {
                                    question: Some(trimmed.clone()),
                                })? {
                                    cli.persist_session()?;
                                }
                                editor.push_history(input);
                                continue;
                            }
                            SlashCommand::Explore { .. } => {
                                tracing::debug!(route = "explore", input = %trimmed, "classifier routed to explore");
                            }
                            SlashCommand::Task { surgical, new_project, .. } => {
                                tracing::debug!(
                                    route = "task",
                                    surgical = surgical,
                                    new_project = new_project,
                                    input = %trimmed,
                                    "classifier routed to task"
                                );
                            }
                            _ => {}
                        }
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        continue;
                    }
                }
                editor.push_history(input);
                cli.run_turn(&trimmed)?;
            }
            input::ReadOutcome::Cancel => {}
            input::ReadOutcome::Exit => {
                cli.persist_session()?;
                break;
            }
        }
    }

    Ok(())
}

pub(crate) fn run_local_repl(port: u16) -> Result<(), anyhow::Error> {
    let sidecar_url = format!("http://localhost:{port}");
    let backend_config = BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: sidecar_url.clone(),
        ..Default::default()
    };

    println!("╔══════════════════════════════════════════╗");
    println!("║  ELAI Local Inference                    ║");
    println!("║  DoLa + TurboQuant on Gemma4 E2B         ║");
    println!("║  ConversationRuntime pipeline active      ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    let (mut sidecar_process, backend_config, degraded) = match sidecar::start_sidecar(port) {
        Ok(proc) => (proc, backend_config, false),
        Err(sidecar_err) => {
            eprintln!("warn: sidecar failed to start: {sidecar_err}");
            eprintln!("Probing ollama at {OLLAMA_DEFAULT_URL}...");
            let probe_url = format!("{OLLAMA_DEFAULT_URL}/api/tags");
            let probe_result = reqwest::blocking::Client::builder()
                .timeout(OLLAMA_PROBE_TIMEOUT)
                .build()
                .map_err(anyhow::Error::from)
                .and_then(|client| {
                    client.get(&probe_url).send()
                        .map_err(anyhow::Error::from)
                });
            match probe_result {
                Ok(resp) if resp.status().is_success() => {
                    eprintln!("Ollama detected. Entering degraded mode.");
                    let mut cfg = backend_config;
                    cfg.fallback_url = Some(OLLAMA_DEFAULT_URL.to_string());
                    (None, cfg, true)
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "sidecar failed ({sidecar_err}) and ollama not reachable at {OLLAMA_DEFAULT_URL}"
                    ));
                }
            }
        }
    };

    // Build a LiveCli with local backend routed through ConversationRuntime.
    let model = backend_config.model.clone();
    let mut cli = LiveCli::new_local(
        model,
        backend_config,
        default_permission_mode(),
    )?;
    let prompt_prefix = if degraded { DEGRADED_PROMPT_PREFIX } else { "local" };
    let prompt_label = format!("{prompt_prefix}> ");
    let mut editor =
        input::LineEditor::new(&prompt_label, cli.repl_completion_candidates().unwrap_or_default());
    println!("{}", cli.startup_banner());
    // Advance FSM from IDLE into EXECUTING for interactive single-agent mode.
    cli.fsm_enter_executing();

    loop {
        editor.set_completions(cli.repl_completion_candidates().unwrap_or_default());
        match editor.read_line()? {
            input::ReadOutcome::Submit(input) => {
                let trimmed = input.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                if matches!(trimmed.as_str(), "/exit" | "/quit") {
                    cli.fsm_shutdown();
                    cli.persist_session()?;
                    break;
                }
                match SlashCommand::parse(&trimmed) {
                    Ok(Some(command)) => {
                        if cli.handle_repl_command(command)? {
                            cli.persist_session()?;
                        }
                        continue;
                    }
                    Ok(None) => {}
                    Err(error) => {
                        eprintln!("{error}");
                        continue;
                    }
                }
                editor.push_history(input);
                cli.run_turn(&trimmed)?;
            }
            input::ReadOutcome::Cancel => {}
            input::ReadOutcome::Exit => {
                cli.fsm_shutdown();
                cli.persist_session()?;
                break;
            }
        }
    }

    // Always shut down sidecar to free VRAM, regardless of who spawned it.
    println!("Shutting down sidecar...");
    let shutdown_client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok();
    if let Some(client) = shutdown_client {
        let _ = client.post(format!("{sidecar_url}/shutdown")).send();
    }
    // Also kill the child process if we spawned it (belt + suspenders).
    if let Some(mut proc) = sidecar_process.take() {
        let _ = proc.kill();
        let _ = proc.wait();
    }

    println!("Goodbye.");
    Ok(())
}
