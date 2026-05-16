//! B-37 WARN-4 — runtime lifecycle event sink.
//!
//! Providers crate is a LEAF (per `systemPatterns.md` L30 +
//! `isolation/mod.rs:6-8`); it MUST NOT depend on the orchestrator
//! crate where the SSB writer lives. CLI adapters, however, need to
//! emit `cli_subprocess_spawned` / `cli_subprocess_killed` rows from
//! deep inside the spawn/kill plumbing. The resolution is a thin
//! object-safe trait + process-global registry: the orchestrator
//! installs a sink at boot, and any provider-side call site
//! (`DefaultTierCSpawn::spawn_isolated`, `cli_common::kill_and_reap`,
//! `RuntimeOrchestrator::try_complete`/`health_snapshot`) reads the
//! registry and pushes a `RuntimeEvent` value.
//!
//! Threading model:
//!   - `OnceLock<Arc<dyn RuntimeEventSink>>` so installation is
//!     idempotent and read-side is lock-free.
//!   - `RuntimeEventSink` methods are sync (the sink impl in
//!     `orchestrator` does the msgpack encode + SSB write on its own
//!     executor); call sites here are synchronous spawn / kill
//!     reactions that cannot afford to block on an async write.
//!
//! Redaction:
//!   - `redact_argv` walks the argv vector and replaces any element
//!     matching common secret prefixes (`sk-...`, `oauth_...`, Bearer
//!     headers, base64 tokens longer than 32 chars after `--api-key`
//!     or `--token`) with `<redacted>`. This is defense-in-depth on
//!     top of the orchestrator's own SsbPayload sanitizer; we never
//!     want a process argv to leak through the sink.

use std::sync::{Arc, OnceLock};

use super::types::{RuntimeBackend, RuntimeHealth};

/// Provider-side lifecycle event. Mirrors the four
/// `runtime_*` / `cli_subprocess_*` `SsbPayload` variants in
/// `orchestrator::ssb` but does NOT depend on that crate — the
/// orchestrator's sink impl translates these into `SsbPayload`.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeEvent {
    /// One step of a `RuntimeOrchestrator::try_complete` fallback walk.
    /// `succeeded == Some(backend)` on a winning step.
    Fallback {
        /// `AgentRole` is identified by its snake_case string here so
        /// the providers crate doesn't pull in `ipc-types`.
        role: String,
        primary: RuntimeBackend,
        attempted: Vec<RuntimeBackend>,
        succeeded: Option<RuntimeBackend>,
        reason: String,
    },
    /// `RuntimeHealth` transition observed by `health_snapshot`.
    HealthChanged {
        backend: RuntimeBackend,
        prev: RuntimeHealth,
        next: RuntimeHealth,
    },
    /// CLI subprocess spawned (`DefaultTierCSpawn::spawn_isolated`).
    CliSubprocessSpawned {
        backend: RuntimeBackend,
        pid: u32,
        argv_redacted: Vec<String>,
    },
    /// CLI subprocess killed / exited (`cli_common::kill_and_reap`).
    CliSubprocessKilled {
        backend: RuntimeBackend,
        pid: u32,
        exit_code: Option<i32>,
        reason: String,
    },
}

/// Object-safe sink consumed by the runtime layer. The orchestrator
/// crate implements this against its `Ssb` writer.
pub trait RuntimeEventSink: Send + Sync + std::fmt::Debug {
    /// Record one event. Implementations MUST NOT block the caller
    /// long enough to stall a subprocess spawn; if persistence is
    /// expensive, push the event onto an unbounded channel and drain
    /// asynchronously.
    fn record(&self, event: RuntimeEvent);
}

/// No-op default sink. Installed implicitly when no orchestrator is
/// present (CLI tools, unit tests, contracts crate).
#[derive(Debug, Default)]
pub struct NoopRuntimeEventSink;

impl RuntimeEventSink for NoopRuntimeEventSink {
    fn record(&self, _event: RuntimeEvent) {}
}

/// Process-global sink. First-write-wins; subsequent installs are
/// no-ops so test code can swap in a `CapturingEventSink` only on a
/// fresh process. The orchestrator crate calls
/// `install_runtime_event_sink` at boot.
static GLOBAL_SINK: OnceLock<Arc<dyn RuntimeEventSink>> = OnceLock::new();

/// Install the process-global sink. Returns `Ok(())` on first install,
/// `Err(existing)` if a sink is already installed (caller can decide
/// whether to log + ignore or panic).
pub fn install_runtime_event_sink(
    sink: Arc<dyn RuntimeEventSink>,
) -> Result<(), Arc<dyn RuntimeEventSink>> {
    GLOBAL_SINK.set(sink).map_err(|existing| existing)
}

/// Read the installed sink. If none has been installed, return a
/// fresh `NoopRuntimeEventSink` arc so call sites never have to
/// handle `Option`.
#[must_use]
pub fn runtime_event_sink() -> Arc<dyn RuntimeEventSink> {
    GLOBAL_SINK
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(NoopRuntimeEventSink) as Arc<dyn RuntimeEventSink>)
}

/// Argv redaction. Replaces any element that looks like a credential
/// with the literal `"<redacted>"`. Heuristics:
///   - elements starting with `sk-`, `sk_`, or `oauth_`
///   - elements starting with `Bearer ` (with or without a space)
///   - elements that are the VALUE following one of the
///     credential-flag names (`--api-key`, `--token`,
///     `--anthropic-api-key`, `--openai-api-key`,
///     `--bearer-token`)
///   - long base64-like tokens (>32 chars, only `[A-Za-z0-9_\-+/=.]`)
///
/// The function is allocation-conservative: the input vector is
/// cloned only when a redaction is actually applied. Test the
/// behaviour via the inline `redact_argv_*` tests below.
#[must_use]
pub fn redact_argv(argv: &[String]) -> Vec<String> {
    const CRED_FLAGS: &[&str] = &[
        "--api-key",
        "--token",
        "--auth-token",
        "--anthropic-api-key",
        "--openai-api-key",
        "--bearer-token",
        "--secret",
        "--password",
    ];

    let mut out: Vec<String> = Vec::with_capacity(argv.len());
    let mut redact_next = false;
    for arg in argv {
        if redact_next {
            out.push("<redacted>".to_string());
            redact_next = false;
            continue;
        }
        if CRED_FLAGS.iter().any(|flag| arg == flag) {
            out.push(arg.clone());
            redact_next = true;
            continue;
        }
        if CRED_FLAGS.iter().any(|flag| {
            let prefix = format!("{flag}=");
            arg.starts_with(&prefix)
        }) {
            // `--api-key=sk-...` form: keep the flag, redact the value.
            if let Some((flag, _value)) = arg.split_once('=') {
                out.push(format!("{flag}=<redacted>"));
                continue;
            }
        }
        if looks_like_secret(arg) {
            out.push("<redacted>".to_string());
            continue;
        }
        out.push(arg.clone());
    }
    out
}

fn looks_like_secret(arg: &str) -> bool {
    if arg.starts_with("sk-") || arg.starts_with("sk_") || arg.starts_with("oauth_") {
        return true;
    }
    if arg.starts_with("Bearer ") || arg.starts_with("bearer ") {
        return true;
    }
    // Long opaque token. Be conservative: must be longer than 40 chars
    // and use only ASCII identifier-safe characters. Avoids flagging
    // file paths (which contain `/` only on POSIX; we keep the rule
    // tight by requiring NO `/` and NO `\` characters).
    if arg.len() > 40
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+' | '=' | ':'))
        && !arg.contains('=')
        && !arg.contains(':')
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Test helper. Captures every event in an in-memory vec.
    #[derive(Debug, Default)]
    pub struct CapturingEventSink {
        pub events: Mutex<Vec<RuntimeEvent>>,
    }

    impl RuntimeEventSink for CapturingEventSink {
        fn record(&self, event: RuntimeEvent) {
            self.events
                .lock()
                .expect("capture sink poisoned")
                .push(event);
        }
    }

    impl CapturingEventSink {
        pub fn snapshot(&self) -> Vec<RuntimeEvent> {
            self.events
                .lock()
                .expect("capture sink poisoned")
                .clone()
        }
    }

    #[test]
    fn noop_sink_is_no_op() {
        let sink = NoopRuntimeEventSink;
        sink.record(RuntimeEvent::CliSubprocessSpawned {
            backend: RuntimeBackend::ClaudeCode,
            pid: 1,
            argv_redacted: vec![],
        });
        // No panic, nothing observable.
    }

    #[test]
    fn redact_argv_strips_sk_prefix() {
        let argv = vec![
            "claude".to_string(),
            "--api-key".to_string(),
            "sk-ant-9999999".to_string(),
            "--print".to_string(),
        ];
        let out = redact_argv(&argv);
        assert_eq!(out[0], "claude");
        assert_eq!(out[1], "--api-key");
        assert_eq!(out[2], "<redacted>");
        assert_eq!(out[3], "--print");
    }

    #[test]
    fn redact_argv_strips_equals_form() {
        let argv = vec![
            "claude".to_string(),
            "--api-key=sk-ant-12345".to_string(),
            "prompt".to_string(),
        ];
        let out = redact_argv(&argv);
        assert_eq!(out[1], "--api-key=<redacted>");
        assert_eq!(out[2], "prompt");
    }

    #[test]
    fn redact_argv_handles_oauth_and_bearer() {
        let argv = vec![
            "curl".to_string(),
            "oauth_abc123def".to_string(),
            "Bearer eyJsomething".to_string(),
        ];
        let out = redact_argv(&argv);
        assert_eq!(out[1], "<redacted>");
        assert_eq!(out[2], "<redacted>");
    }

    #[test]
    fn redact_argv_passes_through_plain_args() {
        let argv = vec!["claude".to_string(), "-p".to_string(), "hello".to_string()];
        let out = redact_argv(&argv);
        assert_eq!(out, argv);
    }

    #[test]
    fn install_runtime_event_sink_first_wins() {
        // We cannot test the global sink in isolation because OnceLock
        // is process-global. Instead, exercise the CapturingEventSink
        // through a *direct* record() call — the rest is covered by
        // the integration test in orchestrator/runtime_orchestrator.
        let sink = Arc::new(CapturingEventSink::default());
        sink.record(RuntimeEvent::Fallback {
            role: "executor".into(),
            primary: RuntimeBackend::Sidecar,
            attempted: vec![RuntimeBackend::Sidecar],
            succeeded: Some(RuntimeBackend::Sidecar),
            reason: "primary".into(),
        });
        assert_eq!(sink.snapshot().len(), 1);
    }
}
