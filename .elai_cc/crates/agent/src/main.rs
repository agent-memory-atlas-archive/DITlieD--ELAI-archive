// elai_cc_agent — isolated agent binary
//
// This binary runs as a sandboxed child process of the orchestrator.
// It communicates exclusively via IPC (unix socket / named pipe).
//
// INVARIANT: this crate MUST NOT depend on reqwest, providers, tools,
// rules, knowledge, or orchestrator. All LLM calls are proxied via
// IPC InferRequest/InferResponse messages through the orchestrator.

use std::env;
use std::io::{Read, Write};
use std::process::ExitCode;
use std::time::Duration;

use elai_core::{ClockProvider as _, OsClock};

use elai_ipc_types::{
    IpcFrame, IpcFrameError, IpcMessageType,
    AGENT_HEARTBEAT_INTERVAL_SECS, FRAME_HEADER_SIZE,
    IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
};
use uuid::Uuid;

mod agent_loop;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Exit code when socket connection fails.
const EXIT_CONNECT_FAILED: u8 = 2;

/// Exit code when protocol version mismatch is detected.
const EXIT_VERSION_MISMATCH: u8 = 3;

/// Exit code when an unrecoverable IPC error occurs.
const EXIT_IPC_ERROR: u8 = 4;

// ---------------------------------------------------------------------------
// AgentError
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum AgentError {
    Io(std::io::Error),
    Frame(IpcFrameError),
    VersionMismatch { expected: u8, actual: u8 },
    Killed(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Frame(e) => write!(f, "frame error: {e}"),
            Self::VersionMismatch { expected, actual } => {
                write!(f, "version mismatch: expected {expected}, got {actual}")
            }
            Self::Killed(reason) => write!(f, "killed: {reason}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Frame I/O helpers
// ---------------------------------------------------------------------------

fn read_frame(stream: &mut dyn Read) -> Result<IpcFrame, AgentError> {
    let mut header = [0u8; FRAME_HEADER_SIZE];
    stream.read_exact(&mut header).map_err(AgentError::Io)?;

    let version = header[0];
    if version != IPC_PROTOCOL_VERSION {
        return Err(AgentError::VersionMismatch {
            expected: IPC_PROTOCOL_VERSION,
            actual: version,
        });
    }

    let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    if len > MAX_FRAME_SIZE {
        return Err(AgentError::Frame(IpcFrameError::PayloadTooLarge {
            size: len,
            limit: MAX_FRAME_SIZE,
        }));
    }

    let mut payload = vec![0u8; len];
    if len > 0 {
        stream.read_exact(&mut payload).map_err(AgentError::Io)?;
    }

    Ok(IpcFrame { version, payload })
}

fn write_frame(stream: &mut dyn Write, frame: &IpcFrame) -> Result<(), AgentError> {
    let wire = frame.to_wire_bytes().map_err(AgentError::Frame)?;
    stream.write_all(&wire).map_err(AgentError::Io)?;
    stream.flush().map_err(AgentError::Io)?;
    Ok(())
}

fn send_message(stream: &mut dyn Write, msg: &IpcMessageType) -> Result<(), AgentError> {
    let frame = IpcFrame::encode(msg).map_err(AgentError::Frame)?;
    write_frame(stream, &frame)
}

// ---------------------------------------------------------------------------
// Message dispatch
// ---------------------------------------------------------------------------

fn handle_message(
    msg: IpcMessageType,
    agent_id: &Uuid,
    writer: &mut dyn Write,
    reader: &mut dyn Read,
) -> Result<bool, AgentError> {
    match msg {
        IpcMessageType::AgentRequest { task, dispatch_config } => {
            tracing::info!(task = %task, role = ?dispatch_config.role, "received agent request");
            // Phase A2: dispatch into the real ReAct loop instead of the
            // ack-only stub that 4A.5 shipped. run_react_turn drives
            // InferRequest <-> InferResponse <-> ToolRequest <-> ToolResult
            // and emits a terminal AgentResponse (end_turn) or
            // AgentMaxIterationsReached (cap) frame.
            agent_loop::run_react_turn(task, agent_id, writer, reader)?;
            Ok(true) // continue outer dispatch loop for subsequent requests
        }

        IpcMessageType::InferResponse { text: _text, tokens_used } => {
            // The ReAct loop owns InferResponse handling for in-flight turns.
            // Stray frames outside an active turn fall through here for
            // visibility only.
            tracing::trace!(
                tokens = tokens_used,
                "received out-of-turn InferResponse — ignored"
            );
            Ok(true)
        }

        IpcMessageType::ToolResult { tool_id, output, success } => {
            // Likewise, the ReAct loop consumes ToolResult inline. A
            // stray ToolResult outside the loop is logged with the byte
            // length (NOT silently swallowed — that was the A2 stub-ban).
            tracing::trace!(
                tool = %tool_id.as_str(),
                success,
                output_len = output.len(),
                "received out-of-turn ToolResult — logged, not consumed"
            );
            Ok(true)
        }

        IpcMessageType::BashResult { stdout: _, stderr: _, exit_code } => {
            tracing::debug!(exit_code, "received bash result");
            Ok(true)
        }

        IpcMessageType::GroundingResult { topic, evidence, confidence } => {
            tracing::debug!(
                topic = %topic,
                evidence_count = evidence.len(),
                confidence,
                "received grounding result"
            );
            Ok(true)
        }

        IpcMessageType::AgentKill { reason } => {
            tracing::info!(reason = %reason, "received kill signal");
            Err(AgentError::Killed(reason))
        }

        // Messages the agent should not receive (agent -> orchestrator only)
        IpcMessageType::AgentResponse { .. }
        | IpcMessageType::InferRequest { .. }
        | IpcMessageType::ToolRequest { .. }
        | IpcMessageType::BashRequest { .. }
        | IpcMessageType::GroundingRequest { .. }
        | IpcMessageType::ThinkStep { .. }
        | IpcMessageType::CheckpointSync { .. }
        | IpcMessageType::AgentHeartbeat { .. }
        | IpcMessageType::ReplanRequest { .. }
        | IpcMessageType::AgentMaxIterationsReached { .. } => {
            tracing::warn!("received unexpected message type from orchestrator");
            Ok(true)
        }
    }
}

// ---------------------------------------------------------------------------
// Connection and message loop
// ---------------------------------------------------------------------------

/// Connect to the orchestrator's IPC socket and run the message loop.
///
/// On Unix: connects to a Unix domain socket at the given path.
/// On Windows: connects to a named pipe at the given path.
///
/// The agent sends a version byte on connect, then enters a read loop
/// dispatching incoming frames. Heartbeats are sent at regular intervals.
#[cfg(unix)]
fn run_agent(socket_path: &str, agent_id: Uuid) -> Result<(), AgentError> {
    use std::os::unix::net::UnixStream;

    let stream = UnixStream::connect(socket_path).map_err(AgentError::Io)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(AGENT_HEARTBEAT_INTERVAL_SECS)))
        .map_err(AgentError::Io)?;

    tracing::info!(socket_path, "connected to orchestrator");

    let mut reader = stream.try_clone().map_err(AgentError::Io)?;
    let mut writer = stream;

    run_message_loop(&mut reader, &mut writer, agent_id)
}

#[cfg(windows)]
fn run_agent(socket_path: &str, agent_id: Uuid) -> Result<(), AgentError> {
    use std::fs::OpenOptions;

    // On Windows, named pipes are opened as files
    let pipe = OpenOptions::new()
        .read(true)
        .write(true)
        .open(socket_path)
        .map_err(AgentError::Io)?;

    tracing::info!(socket_path, "connected to orchestrator via named pipe");

    let mut reader = pipe.try_clone().map_err(AgentError::Io)?;
    let mut writer = pipe;

    run_message_loop(&mut reader, &mut writer, agent_id)
}

fn run_message_loop(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    agent_id: Uuid,
) -> Result<(), AgentError> {
    let _clock = OsClock;
    let heartbeat_interval = Duration::from_secs(AGENT_HEARTBEAT_INTERVAL_SECS);
    let mut last_heartbeat = _clock.now();
    let uptime_start = _clock.now();

    loop {
        // Send heartbeat if interval elapsed
        if _clock.elapsed_since(last_heartbeat) >= heartbeat_interval {
            let uptime_secs = _clock.elapsed_since(uptime_start).as_secs();
            let hb = IpcMessageType::AgentHeartbeat {
                agent_id,
                uptime_secs,
            };
            send_message(writer, &hb)?;
            last_heartbeat = _clock.now();
        }

        // Try to read a frame (with timeout on Unix)
        match read_frame(reader) {
            Ok(frame) => {
                let msg = IpcFrame::decode(&frame.payload).map_err(AgentError::Frame)?;
                match handle_message(msg, &agent_id, writer, reader)? {
                    true => continue,
                    false => break,
                }
            }
            Err(AgentError::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout on read — just loop back to send heartbeat
                continue;
            }
            Err(AgentError::Io(ref e)) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Windows timeout variant
                continue;
            }
            Err(AgentError::Io(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof
                    || e.kind() == std::io::ErrorKind::ConnectionReset =>
            {
                tracing::info!("orchestrator disconnected");
                break;
            }
            Err(AgentError::Killed(reason)) => {
                tracing::info!(reason = %reason, "agent killed by orchestrator");
                break;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    // patch-R2.2: agent has no tracing_subscriber initialized (would pull a
    // ~200KB workspace dep), so tracing::error! events vanish. Use eprintln!
    // for every error path so the orchestrator's stderr_tail capture surfaces
    // actionable failure info. Bug history: B3 cloud e2e showed exit_code=4
    // (EXIT_IPC_ERROR) with empty stderr because all errors flowed through
    // tracing into a no-op subscriber. Now connect failures (the most common
    // mode) print the OS error verbatim before exit.
    eprintln!(
        "elai_cc_agent: starting with argv={:?}",
        args.iter().skip(1).collect::<Vec<_>>()
    );

    // Expected: elai_cc_agent <socket_path> [agent_id]
    let socket_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!("elai_cc_agent: usage: elai_cc_agent <socket_path> [agent_id]");
            tracing::error!("usage: elai_cc_agent <socket_path> [agent_id]");
            return ExitCode::from(1);
        }
    };

    // Parse agent_id from args or generate one via explicit getrandom entropy
    let agent_id = args
        .get(2)
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(|| {
            let mut b = [0u8; 16];
            getrandom::fill(&mut b)
                .expect("getrandom: OS entropy source unavailable at agent startup");
            uuid::Builder::from_random_bytes(b).into_uuid()
        });

    tracing::info!(
        socket_path = %socket_path,
        agent_id = %agent_id,
        "agent process starting"
    );

    match run_agent(socket_path, agent_id) {
        Ok(()) => {
            eprintln!("elai_cc_agent: exiting cleanly");
            tracing::info!("agent exiting cleanly");
            ExitCode::SUCCESS
        }
        Err(AgentError::Killed(reason)) => {
            eprintln!("elai_cc_agent: killed: {reason}");
            tracing::info!(reason = %reason, "agent killed");
            ExitCode::SUCCESS
        }
        Err(AgentError::VersionMismatch { expected, actual }) => {
            eprintln!(
                "elai_cc_agent: protocol version mismatch (expected {expected}, got {actual}) \
                 — binary may be outdated"
            );
            tracing::error!("protocol version mismatch — binary may be outdated");
            ExitCode::from(EXIT_VERSION_MISMATCH)
        }
        Err(AgentError::Io(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
            eprintln!("elai_cc_agent: connection refused at socket_path={socket_path}: {e}");
            tracing::error!(error = %e, "connection to orchestrator failed");
            ExitCode::from(EXIT_CONNECT_FAILED)
        }
        Err(e) => {
            // patch-R2.2: emit the FULL underlying error to stderr (was tracing
            // only — never reached the orchestrator). This covers the most
            // important diagnostic case for B2b/B3: agent can't open the IPC
            // endpoint (Windows ERROR_FILE_NOT_FOUND from wrong pipe path,
            // missing parent dir on Unix, etc.).
            eprintln!("elai_cc_agent: error: {e} (socket_path={socket_path})");
            tracing::error!(error = %e, "agent error");
            ExitCode::from(EXIT_IPC_ERROR)
        }
    }
}
