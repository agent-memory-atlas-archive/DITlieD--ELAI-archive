// elai_cc_agent — isolated agent binary
//
// This binary runs as a sandboxed child process of the orchestrator.
// It communicates exclusively via IPC (unix socket / named pipe).
//
// INVARIANT: this crate MUST NOT depend on reqwest, providers, tools,
// rules, knowledge, or orchestrator. All LLM calls are proxied via
// IPC InferRequest/InferResponse messages through the orchestrator.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    // Expected: elai_cc_agent <socket_path>
    // Socket path is provided by orchestrator at spawn time.
    let socket_path = match args.get(1) {
        Some(p) => p,
        None => {
            tracing::error!("usage: elai_cc_agent <socket_path>");
            return ExitCode::from(1);
        }
    };

    tracing::info!(socket_path = %socket_path, "agent process starting");

    // Connection and message loop implementation deferred to 4A.5.
    // This skeleton establishes the binary target and dependency boundary.

    ExitCode::SUCCESS
}
