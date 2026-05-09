//! REPL entry points: standard (`run_repl`) and local-sidecar (`run_local_repl`).
//!
//! Extracted from main.rs in B-15D Phase A4.

use std::time::Duration;

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
