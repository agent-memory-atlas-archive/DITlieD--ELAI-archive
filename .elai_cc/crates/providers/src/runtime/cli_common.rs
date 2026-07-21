//! Shared helpers for the B-37B CLI subprocess adapters.
//!
//! Contains:
//! - `sanitize_stdout_line` — strip ANSI escape sequences, drop control
//!   characters (except `\n`, `\r`, `\t`), and cap line length at 100KB
//!   to defeat resource-exhaustion / context-smuggling from adversarial
//!   subprocess stdout (paper L40-L60 threat model + spec phase-4M.txt
//!   L264-268 "stdout sanitized").
//! - `default_binary_for` — table mapping `RuntimeBackend` to its
//!   conventional binary name (e.g. `RuntimeBackend::KiroCli ->
//!   "kiro-cli"`, the HYPHEN one — see paper L223+L298-299).
//! - `resolve_binary` — locate a CLI binary either from an explicit
//!   `cli_path` or via a PATH scan, returning a usable absolute path or
//!   a `RuntimeAdapterError::BinaryNotFound`.
//! - `effective_timeout` — translate the `Option<f64>` seconds knobs on
//!   `RuntimeFactoryConfig` into `Option<Duration>` (non-positive
//!   disables, matching the ouroboros pattern at paper L192).
//! - `await_first_line_with_startup_timeout` / `await_next_line_with_idle_timeout`
//!   — uniform timeout wrappers around `Lines::next_line()` that handle
//!   the kill-and-reap dance for poisoned `BufReader` instances.
//! - `kill_and_reap` — best-effort `Child::kill().await` followed by
//!   `wait().await` to avoid zombie subprocesses on timeout.
//! - `permission_mode_to_*_arg` — per-CLI flag mappers consulted by the
//!   per-adapter `complete` builders.
//! - `enforce_permission_opt_in` — central choke-point that refuses
//!   `RuntimePermissionMode::DangerouslySkipPermissions` unless the
//!   factory config carries `allow_danger=true`.
//!
//! No external crates added — pure `std` + `tokio`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncBufRead;
use tokio::process::Child;

use super::event_sink::{redact_argv, runtime_event_sink, RuntimeEvent};
use super::isolation::{TierCSpawn, TierCSpawnError, TierCStdio};
use super::types::{
    RuntimeAdapterError, RuntimeBackend, RuntimeFactoryConfig, RuntimePermissionMode,
};

/// Default startup-output timeout when `RuntimeFactoryConfig.startup_output_timeout_seconds`
/// is `None`. Mirrors the ouroboros hermes / kiro defaults (paper L181-L182, L230).
pub const DEFAULT_STARTUP_TIMEOUT_S: f64 = 60.0;

/// Default stdout-idle timeout when `RuntimeFactoryConfig.stdout_idle_timeout_seconds`
/// is `None`.
pub const DEFAULT_IDLE_TIMEOUT_S: f64 = 300.0;

/// Maximum line length we forward through `sanitize_stdout_line`.
/// Above this we truncate and append a marker.
pub const MAX_LINE_LEN: usize = 100 * 1024;

/// Embedded in `RuntimeAdapterError::NotImplemented` for the few
/// CLI-protocol corners still deferred to B-37C.
pub const HINT_B37B_PARTIAL: &str =
    "feature partially implemented in B-37B; remaining work deferred to B-37C";

/// Conventional binary name for each backend. The kiro_cli entry
/// embeds the hyphen explicitly (paper L223, L298-L299).
#[must_use]
pub fn default_binary_for(backend: RuntimeBackend) -> &'static str {
    match backend {
        RuntimeBackend::ClaudeCode => "claude",
        RuntimeBackend::CodexCli => "codex",
        RuntimeBackend::OpenCode => "opencode",
        RuntimeBackend::Hermes => "hermes",
        RuntimeBackend::GeminiCli => "gemini",
        RuntimeBackend::KiroCli => "kiro-cli",
        RuntimeBackend::CopilotCli => "copilot",
        // Non-CLI backends — value is meaningless but returning an
        // empty string would mask a real bug, so we keep a stable
        // sentinel that grep can find.
        RuntimeBackend::Sidecar
        | RuntimeBackend::AnthropicApi
        | RuntimeBackend::Ollama
        | RuntimeBackend::Vllm
        | RuntimeBackend::Mlx
        | RuntimeBackend::LlamaCpp
        | RuntimeBackend::Ppm
        // C-44 — Composer 2 is HTTP+API-key, not a CLI subprocess.
        | RuntimeBackend::Composer2
        // E-40 — Lsp is a subprocess oracle dispatched via
        // elai_orchestrator::symbol_graph::lsp_oracle, not the generic CLI
        // subprocess shape.
        | RuntimeBackend::Lsp
        // E-26 — Sglang is HTTP /v1/chat/completions, not a CLI subprocess.
        | RuntimeBackend::Sglang
        // E-39 — DeepSeek V4-Flash / Cerebras Free / Groq Free are
        // HTTP-only cloud / free-tier endpoints, not CLI subprocesses.
        | RuntimeBackend::DeepSeekV4Flash
        | RuntimeBackend::CerebrasFree
        | RuntimeBackend::GroqFree
        // I-49 — Bedrock + Azure OpenAI are direct HTTP backends, not CLI
        // subprocesses.
        | RuntimeBackend::Bedrock
        | RuntimeBackend::AzureOpenAi
        | RuntimeBackend::Custom => "<non-cli>",
    }
}

/// Resolve a CLI binary path. If `explicit` is `Some`, return it (as a
/// `PathBuf`). Otherwise scan the `PATH` env var for `default_name`. On
/// miss, return `BinaryNotFound`.
///
/// This is intentionally a small stdlib re-implementation of `which` —
/// CLAUDE.md forbids new external dependencies and the existing
/// codebase has the same pattern in `core::sandbox::command_exists`.
pub fn resolve_binary(
    explicit: Option<&Path>,
    default_name: &str,
) -> Result<PathBuf, RuntimeAdapterError> {
    if let Some(p) = explicit {
        // Explicit path: trust the operator. health_check still spawns
        // and observes failure when the file doesn't actually exist, so
        // we don't reject a non-existent explicit path here — that
        // makes the "missing binary -> Unavailable" path testable
        // without a real CLI on PATH.
        return Ok(p.to_path_buf());
    }
    let paths = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(default_name);
        if candidate.is_file() {
            return Ok(candidate);
        }
        // Windows: try with `.exe` / `.cmd` / `.bat` suffixes.
        #[cfg(windows)]
        for ext in ["exe", "cmd", "bat"] {
            let mut with_ext = candidate.clone();
            with_ext.set_extension(ext);
            if with_ext.is_file() {
                return Ok(with_ext);
            }
        }
    }
    Err(RuntimeAdapterError::BinaryNotFound {
        name: default_name.to_string(),
        searched_path: paths.to_string_lossy().to_string(),
    })
}

/// Translate the `Option<f64>` timeout knobs on `RuntimeFactoryConfig`
/// into `Option<Duration>`. `None` -> apply the supplied default; a
/// non-positive value (`<= 0.0`) -> disable the timeout entirely
/// (returns `None`).
#[must_use]
pub fn effective_timeout(secs: Option<f64>, default_secs: f64) -> Option<Duration> {
    let chosen = secs.unwrap_or(default_secs);
    if chosen <= 0.0 || !chosen.is_finite() {
        return None;
    }
    Some(Duration::from_secs_f64(chosen))
}

/// Strip ANSI escape sequences from `input`. Handles the standard CSI
/// (`ESC[...alpha`) and OSC (`ESC]...BEL`) forms. ~20 LOC stdlib-only
/// port of `cli::render::strip_ansi` so we don't introduce a reverse
/// dep `cli <- providers`.
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // ESC. Look at the following byte.
            match chars.next() {
                Some('[') => {
                    // CSI: consume until an alphabetic byte (terminator).
                    while let Some(&peeked) = chars.peek() {
                        chars.next();
                        if peeked.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC: consume until BEL (0x07) or ST (ESC \).
                    while let Some(&peeked) = chars.peek() {
                        chars.next();
                        if peeked == '\x07' {
                            break;
                        }
                        if peeked == '\x1b' {
                            // ST: consume the trailing `\`.
                            if matches!(chars.peek(), Some('\\')) {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                // Single-character escape (e.g. ESC=, ESCM). Skip the
                // next byte without copying.
                Some(_) => {}
                None => {
                    // Trailing ESC at EOF: drop.
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Drop ASCII control characters (< 0x20) except `\n`, `\r`, `\t`. Also
/// drops DEL (0x7F).
fn drop_control_chars(input: &str) -> String {
    input
        .chars()
        .filter(|&c| c == '\n' || c == '\r' || c == '\t' || (!c.is_control() && c != '\u{7f}'))
        .collect()
}

/// Sanitize a single line of subprocess stdout.
///
/// Pipeline:
/// 1. Strip ANSI escape sequences.
/// 2. Drop ASCII control characters (except `\n`, `\r`, `\t`).
/// 3. Cap at `MAX_LINE_LEN`; truncation marker appended on overflow.
///
/// Defeats the threat-model items enumerated in
/// `context-package-B37B.md` THREAT MODEL section (cursor-positioning
/// ANSI, null bytes, mega-line smuggling).
#[must_use]
pub fn sanitize_stdout_line(line: &str) -> String {
    let stripped = strip_ansi(line);
    let mut cleaned = drop_control_chars(&stripped);
    if cleaned.len() > MAX_LINE_LEN {
        let original_len = cleaned.len();
        // Truncate at MAX_LINE_LEN _bytes_, snapping back to a char
        // boundary so we don't slice mid-UTF-8.
        let mut idx = MAX_LINE_LEN;
        while idx > 0 && !cleaned.is_char_boundary(idx) {
            idx -= 1;
        }
        cleaned.truncate(idx);
        cleaned.push_str(&format!(
            "...[truncated, original {original_len} bytes]"
        ));
    }
    cleaned
}

/// Wait for the FIRST line of stdout, bounded by `timeout`. Returns
/// `Ok(Some(line))` on success, `Ok(None)` if the subprocess closed
/// stdout without writing, or `Err(StartupTimeout)` on timeout.
///
/// Caller is responsible for killing the subprocess on timeout (this
/// helper does NOT touch the `Child`).
pub async fn await_first_line_with_startup_timeout<R>(
    reader: &mut tokio::io::Lines<R>,
    timeout: Option<Duration>,
    binary: &str,
) -> Result<Option<String>, RuntimeAdapterError>
where
    R: AsyncBufRead + Unpin,
{
    match timeout {
        None => reader
            .next_line()
            .await
            .map_err(|e| RuntimeAdapterError::Other(format!("read failed: {e}"))),
        Some(d) => match tokio::time::timeout(d, reader.next_line()).await {
            Ok(Ok(line)) => Ok(line),
            Ok(Err(e)) => Err(RuntimeAdapterError::Other(format!("read failed: {e}"))),
            Err(_) => Err(RuntimeAdapterError::StartupTimeout {
                binary: binary.to_string(),
                seconds: d.as_secs_f64(),
            }),
        },
    }
}

/// Wait for a SUBSEQUENT line of stdout (after at least one line was
/// observed). Returns the same shape as
/// `await_first_line_with_startup_timeout`, but on timeout returns
/// `RuntimeAdapterError::IdleTimeout`.
pub async fn await_next_line_with_idle_timeout<R>(
    reader: &mut tokio::io::Lines<R>,
    timeout: Option<Duration>,
    binary: &str,
) -> Result<Option<String>, RuntimeAdapterError>
where
    R: AsyncBufRead + Unpin,
{
    match timeout {
        None => reader
            .next_line()
            .await
            .map_err(|e| RuntimeAdapterError::Other(format!("read failed: {e}"))),
        Some(d) => match tokio::time::timeout(d, reader.next_line()).await {
            Ok(Ok(line)) => Ok(line),
            Ok(Err(e)) => Err(RuntimeAdapterError::Other(format!("read failed: {e}"))),
            Err(_) => Err(RuntimeAdapterError::IdleTimeout {
                binary: binary.to_string(),
                seconds: d.as_secs_f64(),
            }),
        },
    }
}

/// Best-effort kill + reap so a timed-out subprocess doesn't become a
/// zombie. Failures from `kill()` are logged via `tracing::warn!` and
/// then swallowed — we cannot do anything more.
///
/// This is the legacy entry point retained so existing call sites
/// compile unchanged. New call sites should prefer
/// [`kill_and_reap_with_event`] which also emits a
/// `RuntimeEvent::CliSubprocessKilled` row through the installed sink.
pub async fn kill_and_reap(child: &mut Child) {
    if let Err(e) = child.kill().await {
        tracing::warn!(error = %e, "CLI subprocess kill failed (already exited?)");
    }
    if let Err(e) = child.wait().await {
        tracing::warn!(error = %e, "CLI subprocess wait failed after kill");
    }
}

/// Spawn a CLI subprocess through a `TierCSpawn` and emit a
/// `RuntimeEvent::CliSubprocessSpawned` row on success.
///
/// `argv` is redacted via [`redact_argv`] before reaching the sink —
/// the orchestrator's `SsbPayload` sanitiser also runs on top, but
/// this is the first line of defence against accidental secret
/// leakage. Failures bubble through unchanged so callers can map them
/// to `RuntimeAdapterError` via [`spawn_err_to_adapter_err`].
#[allow(clippy::too_many_arguments)]
pub fn spawn_isolated_with_event(
    spawn: &Arc<dyn TierCSpawn>,
    backend: RuntimeBackend,
    program: &Path,
    args: &[String],
    cwd: &Path,
    scratch_dir: &Path,
    stdin: TierCStdio,
    stdout: TierCStdio,
    stderr: TierCStdio,
    permission_mode: RuntimePermissionMode,
    env_overrides: Vec<(String, String)>,
) -> Result<Child, TierCSpawnError> {
    let child = spawn.spawn_isolated(
        program,
        args,
        cwd,
        scratch_dir,
        stdin,
        stdout,
        stderr,
        permission_mode,
        env_overrides,
    )?;
    // tokio::process::Child exposes .id() returning Option<u32>; None
    // only after wait() has reaped, which we have not done yet.
    let pid = child.id().unwrap_or(0);
    let mut full_argv: Vec<String> = Vec::with_capacity(args.len() + 1);
    full_argv.push(program.display().to_string());
    full_argv.extend(args.iter().cloned());
    runtime_event_sink().record(RuntimeEvent::CliSubprocessSpawned {
        backend,
        pid,
        argv_redacted: redact_argv(&full_argv),
    });
    Ok(child)
}

/// Best-effort kill + reap that also emits a
/// `RuntimeEvent::CliSubprocessKilled` row through the installed sink.
/// `reason` is a short human-readable explanation surfaced into the
/// SSB payload.
pub async fn kill_and_reap_with_event(
    child: &mut Child,
    backend: RuntimeBackend,
    reason: impl Into<String>,
) {
    let pid = child.id().unwrap_or(0);
    if let Err(e) = child.kill().await {
        tracing::warn!(error = %e, "CLI subprocess kill failed (already exited?)");
    }
    let exit_code = match child.wait().await {
        Ok(status) => status.code(),
        Err(e) => {
            tracing::warn!(error = %e, "CLI subprocess wait failed after kill");
            None
        }
    };
    runtime_event_sink().record(RuntimeEvent::CliSubprocessKilled {
        backend,
        pid,
        exit_code,
        reason: reason.into(),
    });
}

/// Emit a `RuntimeEvent::CliSubprocessKilled` row reflecting a natural
/// exit (the child already exited; we report it for SSB symmetry).
/// `exit_code` is the value returned by `Child::wait().await?.code()`.
pub fn record_subprocess_exit(
    backend: RuntimeBackend,
    pid: u32,
    exit_code: Option<i32>,
    reason: impl Into<String>,
) {
    runtime_event_sink().record(RuntimeEvent::CliSubprocessKilled {
        backend,
        pid,
        exit_code,
        reason: reason.into(),
    });
}

/// Central permission-mode gate: refuse `DangerouslySkipPermissions`
/// unless `cfg.allow_danger` is `true`. Returns `Ok(())` on permit,
/// `Err(PermissionDenied)` on refusal. Other modes pass through.
pub fn enforce_permission_opt_in(cfg: &RuntimeFactoryConfig) -> Result<(), RuntimeAdapterError> {
    if matches!(
        cfg.permission_mode,
        RuntimePermissionMode::DangerouslySkipPermissions
    ) && !cfg.allow_danger
    {
        return Err(RuntimeAdapterError::PermissionDenied {
            reason: format!(
                "RuntimePermissionMode::DangerouslySkipPermissions requires RuntimeFactoryConfig.allow_danger = true (backend = {:?})",
                cfg.backend
            ),
        });
    }
    Ok(())
}

/// Translate a `RuntimePermissionMode` into the `claude` CLI's
/// `--permission-mode` argument string. Web-grounded: claude CLI
/// accepts `acceptEdits | bypassPermissions | default` per ouroboros
/// paper L129 + Anthropic claude-code-cli docs.
///
/// `Plan` maps to `default` (no special claude flag exists for plan
/// mode) but is otherwise allowed.
pub fn permission_mode_to_claude_arg(mode: RuntimePermissionMode) -> &'static str {
    match mode {
        RuntimePermissionMode::AcceptEdits => "acceptEdits",
        RuntimePermissionMode::Plan => "default",
        RuntimePermissionMode::DangerouslySkipPermissions => "bypassPermissions",
        RuntimePermissionMode::Default => "default",
    }
}

/// Translate a `RuntimePermissionMode` into the gemini CLI's
/// `--approval-mode` argument string. `auto_edit` for AcceptEdits,
/// `yolo` for DangerouslySkipPermissions.
pub fn permission_mode_to_gemini_approval_mode(mode: RuntimePermissionMode) -> &'static str {
    match mode {
        RuntimePermissionMode::AcceptEdits => "auto_edit",
        RuntimePermissionMode::DangerouslySkipPermissions => "yolo",
        // No explicit gemini "plan" mode — fall back to auto_edit
        // (gemini's most read-only-ish path). Default mirrors that.
        RuntimePermissionMode::Plan | RuntimePermissionMode::Default => "auto_edit",
    }
}

/// Translate a `RuntimePermissionMode` into the copilot CLI's
/// `--allow-tool` / `--allow-all` flag set. Returns an owned vec
/// because `--allow-all` may need to be the only flag.
pub fn permission_mode_to_copilot_args(mode: RuntimePermissionMode) -> Vec<String> {
    match mode {
        RuntimePermissionMode::AcceptEdits | RuntimePermissionMode::Default => {
            vec!["--allow-tool".to_string(), "shell".to_string()]
        }
        RuntimePermissionMode::DangerouslySkipPermissions => vec!["--allow-all".to_string()],
        // Plan mode: no allow flag => copilot stays read-only-ish.
        RuntimePermissionMode::Plan => Vec::new(),
    }
}

/// Compose a flat "system + transcript" prompt from `Message` envelopes
/// + an optional `system_prompt`. Used by every CLI adapter as the
/// fallback transcript renderer; per-adapter overrides may pre-process
/// the result further (e.g. add `--system` flags).
#[must_use]
pub fn compose_prompt(messages: &[super::types::Message], system: Option<&str>) -> String {
    use super::types::MessageRole;
    let mut out = String::new();
    if let Some(s) = system {
        out.push_str("System: ");
        out.push_str(s);
        out.push_str("\n\n");
    }
    for m in messages {
        let role_label = match m.role {
            MessageRole::User => "User",
            MessageRole::Assistant => "Assistant",
            MessageRole::System => "System",
            MessageRole::Tool => "Tool",
        };
        out.push_str(role_label);
        out.push_str(": ");
        out.push_str(&m.content);
        out.push_str("\n\n");
    }
    out
}

/// Map a `TierCSpawnError` (raised by `TierCSpawn::spawn_isolated`)
/// into the `RuntimeAdapterError` surface used by `complete` /
/// `stream`. Adapter call paths use this immediately after a failed
/// `spawn_isolated` to keep the per-file boilerplate down.
pub fn spawn_err_to_adapter_err(
    e: super::isolation::TierCSpawnError,
) -> RuntimeAdapterError {
    match e {
        super::isolation::TierCSpawnError::BinaryNotFound { path } => {
            RuntimeAdapterError::BinaryNotFound {
                name: path.display().to_string(),
                searched_path: std::env::var("PATH").unwrap_or_default(),
            }
        }
        super::isolation::TierCSpawnError::SpawnFailed { binary, source } => {
            RuntimeAdapterError::SubprocessSpawn {
                binary: binary.display().to_string(),
                source,
            }
        }
        super::isolation::TierCSpawnError::ScratchDirInvalid { reason } => {
            RuntimeAdapterError::Other(reason)
        }
    }
}

/// Shared `health_check` body for all 7 CLI adapters: PATH-resolve
/// (or honor explicit `cli_path`) and verify the result is a regular
/// file. Missing -> `RuntimeHealth::Unavailable { reason: ... }`.
pub async fn health_check_cli_binary(
    cfg: &RuntimeFactoryConfig,
    backend: RuntimeBackend,
) -> super::types::RuntimeHealth {
    match resolve_binary(cfg.cli_path.as_deref(), default_binary_for(backend)) {
        Ok(path) => {
            if path.is_file() {
                super::types::RuntimeHealth::Ready
            } else {
                super::types::RuntimeHealth::Unavailable {
                    reason: format!(
                        "CLI path is not a regular file: {}",
                        path.display()
                    ),
                }
            }
        }
        Err(e) => super::types::RuntimeHealth::Unavailable {
            reason: e.to_string(),
        },
    }
}

/// Generate a small temp-file path for the codex `--output-last-message`
/// flag. Uses `getrandom` (already in providers/Cargo.toml) to derive
/// a unique suffix without pulling in `tempfile`.
pub fn codex_output_temp_path() -> Result<PathBuf, RuntimeAdapterError> {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).map_err(|e| {
        RuntimeAdapterError::Other(format!("getrandom for codex temp path failed: {e}"))
    })?;
    let suffix: String = buf.iter().map(|b| format!("{b:02x}")).collect();
    Ok(std::env::temp_dir().join(format!("elai_codex_last_message_{suffix}.txt")))
}

// ---------------------------------------------------------------------
// Inline unit tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_handles_csi() {
        let dirty = "\x1b[31mhello\x1b[0m";
        assert_eq!(strip_ansi(dirty), "hello");
    }

    #[test]
    fn strip_ansi_handles_osc_bell_terminator() {
        let dirty = "\x1b]0;title\x07rest";
        assert_eq!(strip_ansi(dirty), "rest");
    }

    #[test]
    fn drop_control_keeps_tab_newline_cr() {
        let s = "a\tb\nc\rd\x07\x00";
        assert_eq!(drop_control_chars(s), "a\tb\nc\rd");
    }

    #[test]
    fn sanitize_caps_length() {
        let huge = "a".repeat(2 * MAX_LINE_LEN);
        let out = sanitize_stdout_line(&huge);
        assert!(out.len() <= MAX_LINE_LEN + 128);
        assert!(out.contains("[truncated"));
    }

    #[test]
    fn effective_timeout_disables_on_zero() {
        assert_eq!(effective_timeout(Some(0.0), 60.0), None);
        assert_eq!(effective_timeout(Some(-1.0), 60.0), None);
        assert_eq!(
            effective_timeout(Some(0.5), 60.0),
            Some(Duration::from_secs_f64(0.5))
        );
        assert_eq!(
            effective_timeout(None, 60.0),
            Some(Duration::from_secs_f64(60.0))
        );
    }

    #[test]
    fn default_binary_table_includes_hyphenated_kiro() {
        assert_eq!(default_binary_for(RuntimeBackend::KiroCli), "kiro-cli");
        assert_eq!(default_binary_for(RuntimeBackend::ClaudeCode), "claude");
        assert_eq!(default_binary_for(RuntimeBackend::Hermes), "hermes");
    }

    #[test]
    fn enforce_opt_in_refuses_dangerous_without_flag() {
        let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode);
        cfg.permission_mode = RuntimePermissionMode::DangerouslySkipPermissions;
        cfg.allow_danger = false;
        assert!(matches!(
            enforce_permission_opt_in(&cfg),
            Err(RuntimeAdapterError::PermissionDenied { .. })
        ));
    }

    #[test]
    fn enforce_opt_in_permits_when_flag_set() {
        let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode);
        cfg.permission_mode = RuntimePermissionMode::DangerouslySkipPermissions;
        cfg.allow_danger = true;
        assert!(enforce_permission_opt_in(&cfg).is_ok());
    }

    #[test]
    fn enforce_opt_in_permits_safe_modes() {
        for mode in [
            RuntimePermissionMode::AcceptEdits,
            RuntimePermissionMode::Plan,
            RuntimePermissionMode::Default,
        ] {
            let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::ClaudeCode);
            cfg.permission_mode = mode;
            cfg.allow_danger = false;
            assert!(
                enforce_permission_opt_in(&cfg).is_ok(),
                "{mode:?} must not require allow_danger"
            );
        }
    }
}
