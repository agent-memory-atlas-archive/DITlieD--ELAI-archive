// agent_loop — Phase A2: agent-side ReAct loop.
//
// Drives the (z, a, y) tuple from workflow/papers/swe-world-interface.md:
//   z = observation (InferResponse text + tool results so far)
//   a = action (TOOL_USE marker -> ToolRequest, or end_turn -> AgentResponse)
//   y = outcome (ToolResult bytes appended to conversation)
//
// Bounded iteration per workflow/papers/agentless-2407.01489-interface.md
// ("an incorrect step can be easily amplified"): hard cap at
// AGENT_MAX_ITERATIONS (default 25, mid-point of swe-world's [20, 100]
// range; see workflow/handoff/phase-A2.md acceptance_criteria).
//
// Deviations vs the original orchestrator-prompt A2 wording:
//
//   (a) The agent crate does NOT wrap, import, or share a ConversationRuntime
//       with the orchestrator. crates/agent/Cargo.toml MUST NOT add
//       elai-orchestrator; the invariant is declared at main.rs:1-9. Instead
//       this module reimplements just the ReAct shape on top of
//       elai_ipc_types — the wire-format crate both sides depend on.
//       Rationale: workflow/handoff/design-deviations.md.
//
//   (b) IpcMessageType::InferResponse carries a flat `text: String`, not a
//       Vec<ContentBlock>. Tool-use detection therefore uses an in-text
//       marker (`TOOL_USE: <tool_id>`) rather than a typed block. The
//       orchestrator-side translator that fronts the real model can emit
//       this marker; the agent never sees ContentBlock structs by design.
//
//   (c) The agent has no tokio runtime (Cargo.toml:12-20). The loop is
//       fully synchronous over blocking std::io::{Read, Write} — no async
//       layer is added in A2.

use std::io::{Read, Write};

use elai_ipc_types::{IpcFrame, IpcFrameError, IpcMessageType, ToolId};
use uuid::Uuid;

use crate::{read_frame, send_message, AgentError};

/// Inner-read helper: blocks until a full frame arrives, retrying indefinitely
/// on WouldBlock/TimedOut. The stream's read timeout
/// (AGENT_HEARTBEAT_INTERVAL_SECS, set in run_agent) only paces the OUTER
/// heartbeat loop in run_message_loop; inner reads inside run_react_turn must
/// wait for the matching InferResponse/ToolResult frame, which can take
/// arbitrarily long (e.g. long-running tool execution). Cancellation is the
/// orchestrator's job via AgentKill, which arrives as a normal frame.
fn read_frame_blocking(reader: &mut dyn Read) -> Result<IpcFrame, AgentError> {
    loop {
        match read_frame(reader) {
            Ok(frame) => return Ok(frame),
            Err(AgentError::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // Socket read timeout fired; the matching frame just hasn't
                // arrived yet. Loop back and wait again. Bounded only by the
                // orchestrator sending AgentKill if it wants us to stop.
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Hard cap on agent ReAct iterations per single AgentRequest.
///
/// Sourced from workflow/papers/swe-world-interface.md (max_turns ∈ [20, 100])
/// and tightened to 25 to bias toward the lower bound while leaving headroom
/// for legitimate multi-tool tasks. Override is not exposed in A2 — every
/// agent role inherits this cap.
pub const AGENT_MAX_ITERATIONS: u32 = 25;

/// Marker that signals "the model wants to call a tool" inside a flat
/// InferResponse.text payload. Format (one per line):
///
///   TOOL_USE: <tool_id>
///
/// Any line starting with this prefix is parsed as a single tool-use action.
/// All matching lines on a single InferResponse are dispatched in order.
const TOOL_USE_MARKER: &str = "TOOL_USE: ";

/// Drive one ReAct turn for `task`.
///
/// Caller owns the IPC stream; this function reads InferResponse / ToolResult
/// frames and writes InferRequest / ToolRequest / AgentResponse /
/// AgentMaxIterationsReached frames. Returns `Ok(())` after emitting a
/// terminal frame (AgentResponse on end_turn, AgentMaxIterationsReached on
/// the cap). Any IPC error propagates as `AgentError`.
pub fn run_react_turn(
    task: String,
    _agent_id: &Uuid,
    writer: &mut dyn Write,
    reader: &mut dyn Read,
) -> Result<(), AgentError> {
    // Conversation history is a single flat String that grows by one
    // assistant turn + N tool-result observations per iteration. This
    // matches the InferRequest.prompt shape (it's a String, not a
    // structured message list — see ipc-types/src/lib.rs:144).
    let mut history = format!("USER: {task}\n");
    let mut iterations: u32 = 0;

    loop {
        // ----- Step 1: send InferRequest with current history -----
        let infer_req = IpcMessageType::InferRequest {
            prompt: history.clone(),
            // Conservative default — orchestrator-side capability profile
            // can clamp further. Not configurable in A2.
            max_tokens: 1024,
        };
        send_message(writer, &infer_req)?;

        // ----- Step 2: block-read frames until we see the InferResponse
        // for this turn. Skip any out-of-band frames (heartbeats etc).
        // read_frame_blocking ignores socket-level read timeouts (those exist
        // only to pace the outer heartbeat loop, see run_message_loop).
        let infer_text = loop {
            let frame = read_frame_blocking(reader)?;
            let msg = IpcFrame::decode(&frame.payload)
                .map_err(AgentError::Frame)?;
            match msg {
                IpcMessageType::InferResponse { text, .. } => break text,
                // Heartbeats are agent-outbound only; we should not see any
                // inbound here. Drop other variants with a trace.
                IpcMessageType::AgentKill { reason } => {
                    return Err(AgentError::Killed(reason));
                }
                other => {
                    tracing::trace!(
                        kind = ?std::mem::discriminant(&other),
                        "agent_loop: skipping out-of-band frame while awaiting InferResponse"
                    );
                }
            }
        };

        // Record the assistant turn into the history before parsing tools,
        // so the next iteration's prompt reflects what the model said.
        history.push_str("ASSISTANT: ");
        history.push_str(&infer_text);
        if !infer_text.ends_with('\n') {
            history.push('\n');
        }

        // ----- Step 3: parse tool_use markers -----
        let tool_calls: Vec<ToolId> = infer_text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                trimmed
                    .strip_prefix(TOOL_USE_MARKER)
                    .map(|name| ToolId::new(name.trim()))
            })
            .filter(|tid| !tid.as_str().is_empty())
            .collect();

        // No tool_use lines -> end_turn. Emit AgentResponse and return.
        if tool_calls.is_empty() {
            let response = IpcMessageType::AgentResponse {
                content: infer_text,
                artifacts: Vec::new(),
            };
            send_message(writer, &response)?;
            return Ok(());
        }

        // ----- Step 4: for each tool_use, send ToolRequest, await matching
        // ToolResult, append the (utf8-lossy) output to history.
        for tool_id in tool_calls {
            let tool_req = IpcMessageType::ToolRequest {
                tool_id: tool_id.clone(),
                arguments: Vec::new(),
            };
            send_message(writer, &tool_req)?;

            // Block-read until matching ToolResult arrives. Out-of-band
            // frames are traced and dropped. read_frame_blocking retries
            // through the 5s socket timeout so long-running tools don't
            // kill the agent mid-turn — only AgentKill cancels.
            let (output, success) = loop {
                let frame = read_frame_blocking(reader)?;
                let msg = IpcFrame::decode(&frame.payload)
                    .map_err(AgentError::Frame)?;
                match msg {
                    IpcMessageType::ToolResult {
                        tool_id: got_id,
                        output,
                        success,
                    } if got_id == tool_id => {
                        break (output, success);
                    }
                    IpcMessageType::ToolResult { tool_id: got_id, .. } => {
                        tracing::warn!(
                            expected = %tool_id.as_str(),
                            got = %got_id.as_str(),
                            "agent_loop: tool_id mismatch on ToolResult; skipping frame"
                        );
                    }
                    IpcMessageType::AgentKill { reason } => {
                        return Err(AgentError::Killed(reason));
                    }
                    other => {
                        tracing::trace!(
                            kind = ?std::mem::discriminant(&other),
                            "agent_loop: skipping out-of-band frame while awaiting ToolResult"
                        );
                    }
                }
            };

            // Decode tool output into the history as utf8-lossy to avoid
            // .unwrap() on String::from_utf8 (production paths must not
            // panic on malformed tool stdout).
            let decoded = String::from_utf8_lossy(&output);
            history.push_str(&format!(
                "TOOL_RESULT tool_id={} success={} output={}\n",
                tool_id.as_str(),
                success,
                decoded
            ));
        }

        iterations = iterations.saturating_add(1);

        // ----- Step 5: iteration cap check -----
        if iterations >= AGENT_MAX_ITERATIONS {
            let msg = IpcMessageType::AgentMaxIterationsReached {
                reason: format!(
                    "agent_loop: iteration cap hit (AGENT_MAX_ITERATIONS={AGENT_MAX_ITERATIONS})"
                ),
                iterations_completed: iterations,
            };
            send_message(writer, &msg)?;
            return Ok(());
        }
    }
}

// ---------------------------------------------------------------------------
// Re-export the AgentError type so callers in the same crate keep working.
// Sanity asserts on the marker — caught at compile time if the constant ever
// changes shape (e.g. someone drops the trailing space and breaks the parser).
// ---------------------------------------------------------------------------

const _: () = {
    // The marker must end with ": " so `strip_prefix` consumes the
    // separator and `trim()` cleans the tool id.
    assert!(TOOL_USE_MARKER.len() >= 10);
};

// Silence the unused-import lint when callers don't surface IpcFrameError;
// it appears in AgentError::Frame and is part of the public error surface.
#[allow(dead_code)]
fn _ipc_frame_error_is_in_error_surface(_e: IpcFrameError) {}
