//! E-25 Tranche B-4 (D2) -- skill-template runtime expansion.
//!
//! Implements the runtime expansion of `!`shell-cmd`` blocks inside SKILL.md
//! bodies before the body reaches the model. Grounded in the public Claude-
//! Code docs at code.claude.com/docs/en/skills which document the `!`cmd``
//! convention. This module is the minimal sync expander; live-reload via the
//! `notify` crate is DEFERRED pending explicit user dep confirmation +
//! anti-slopsquatting verification (CLAUDE.md HARD RULE).
//!
//! Wire-dark by default per the E-25 substrate posture; the
//! `ELAI_E25_SKILL_TEMPLATING_ENABLE` env-gate must be `1` for the expander
//! to run any command. With the gate OFF the function returns the body
//! verbatim. Each expansion (or rejection) emits one
//! `ClaudeCodeArchitectureEvent` SSB row (kind = `SkillTemplateExecuted`)
//! via a caller-supplied [`SkillTemplateEmitter`].
//!
//! Shell-injection surface: commands are dispatched via `std::process::Command`
//! WITHOUT a shell so `;` / `&&` / `|` / `$()` are NOT special. Only commands
//! whose first token matches the allowlist below are dispatched; everything
//! else returns `SkillTemplateError::ShellInjectionForbidden`. The allowlist
//! is intentionally tiny on first ship (`git`, `ls`, `pwd`, `date`); growth is
//! gated on user confirm.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};

use crate::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload,
};

/// Environment-variable name controlling the wire-dark gate on D2's
/// expansion + SSB emission path. Set to `1` to enable. Default OFF.
pub const ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR: &str = "ELAI_E25_SKILL_TEMPLATING_ENABLE";

/// Maximum wall-time allowed for a single template command. Tight enough that
/// a runaway `git log` cannot stall the model-side load path.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);

/// Allowed first-token commands. Kept intentionally small on first ship.
/// Growth requires explicit user confirmation.
const ALLOWED_COMMANDS: &[&str] = &["git", "ls", "pwd", "date"];

/// Returns true iff D2 skill-template expansion is wire-enabled.
#[must_use]
pub fn skill_templating_enabled() -> bool {
    std::env::var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// In-memory SKILL.md template under expansion.
#[derive(Debug, Clone)]
pub struct SkillTemplate {
    /// Raw SKILL.md body including any `!`cmd`` blocks.
    pub body: String,
    /// Canonical source path for telemetry. Used as `skill_path` in the
    /// emitted `details_json`; not opened by the expander.
    pub source_path: PathBuf,
}

/// Failure modes of [`expand_skill_template`]. No `Box<dyn Error>` (CLAUDE.md
/// HARD RULE); each variant carries enough scalar context for a structured
/// log + an emitted `ClaudeCodeArchitectureEvent` row.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SkillTemplateError {
    /// Command's first token not in [`ALLOWED_COMMANDS`].
    #[error("skill template shell-injection forbidden: {0}")]
    ShellInjectionForbidden(String),
    /// Command exited non-zero.
    #[error("skill template command failed: {cmd} exit={exit}")]
    CommandFailed { cmd: String, exit: i32 },
    /// Backslash-escape patterns reserved for future syntax extensions.
    #[error("skill template unknown backslash escape: {0}")]
    BackslashUnknown(String),
    /// Command exceeded [`COMMAND_TIMEOUT`].
    #[error("skill template command timed out: {0}")]
    CommandTimeout(String),
}

/// Emitter sink for per-block `ClaudeCodeArchitectureEvent` rows. Mirrors the
/// [`crate::compaction_ladder::LadderEventEmitter`] pattern so the sync
/// expander can run without dragging in a tokio runtime.
pub trait SkillTemplateEmitter {
    fn emit(&self, payload: ClaudeCodeArchitectureEventPayload);
}

/// No-op emitter for tests + wire-dark callers.
pub struct NullEmitter;

impl SkillTemplateEmitter for NullEmitter {
    fn emit(&self, _payload: ClaudeCodeArchitectureEventPayload) {}
}

/// Expand all `!`cmd`` blocks in `skill.body`. Returns the expanded body on
/// success. Wire-dark: when the env-gate is OFF the original body is
/// returned verbatim without scanning.
pub fn expand_skill_template<E: SkillTemplateEmitter>(
    skill: &SkillTemplate,
    project_root: &std::path::Path,
    emitter: &E,
) -> Result<String, SkillTemplateError> {
    if !skill_templating_enabled() {
        return Ok(skill.body.clone());
    }

    let blocks = scan_template_blocks(&skill.body);
    if blocks.is_empty() {
        return Ok(skill.body.clone());
    }

    // Build the expanded body by replacing each block in left-to-right order.
    let mut out = String::with_capacity(skill.body.len());
    let mut last_end: usize = 0;
    for block in &blocks {
        out.push_str(&skill.body[last_end..block.start]);
        let started = OsClock.instant_now();
        let result = run_template_command(&block.command, project_root);
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        match result {
            Ok(stdout_trimmed) => {
                emit_expanded(emitter, skill, &block.command, elapsed_ms, true, None);
                out.push_str(&stdout_trimmed);
            }
            Err(err) => {
                emit_expanded(
                    emitter,
                    skill,
                    &block.command,
                    elapsed_ms,
                    false,
                    Some(short_error_label(&err)),
                );
                return Err(err);
            }
        }
        last_end = block.end;
    }
    out.push_str(&skill.body[last_end..]);
    Ok(out)
}

fn emit_expanded<E: SkillTemplateEmitter>(
    emitter: &E,
    skill: &SkillTemplate,
    command: &str,
    elapsed_ms: u64,
    success: bool,
    reject_reason: Option<&'static str>,
) {
    let ts_ms = current_unix_millis();
    let details = serde_json::json!({
        "templating_event": if success { "expanded" } else { "rejected" },
        "skill_path": skill.source_path.to_string_lossy(),
        "command": redact_command(command),
        "elapsed_ms": elapsed_ms,
        "reason": reject_reason,
    });
    emitter.emit(ClaudeCodeArchitectureEventPayload {
        kind: ClaudeCodeArchitectureEventKind::SkillTemplateExecuted,
        primitive: ClaudeCodeArchPrimitive::SkillTemplating,
        session_id: None,
        details_json: details,
        ts_ms,
    });
}

/// A single `!`cmd`` block discovered inside a SKILL.md body.
#[derive(Debug, Clone, PartialEq)]
struct TemplateBlock {
    start: usize,
    end: usize,
    command: String,
}

/// Scan the body for `!`...`` blocks. The bang must immediately precede the
/// backtick with no whitespace (anti-pattern guard against literal "!" in
/// prose followed by an unrelated backtick block).
fn scan_template_blocks(body: &str) -> Vec<TemplateBlock> {
    let bytes = body.as_bytes();
    let mut out: Vec<TemplateBlock> = Vec::new();
    let mut i: usize = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'!' && bytes[i + 1] == b'`' {
            let cmd_start = i + 2;
            // Find the closing backtick.
            let mut j = cmd_start;
            let mut found = false;
            while j < bytes.len() {
                if bytes[j] == b'`' {
                    found = true;
                    break;
                }
                if bytes[j] == b'\n' {
                    // Bang-tick blocks do not span lines.
                    break;
                }
                j += 1;
            }
            if found {
                if let Ok(cmd) = std::str::from_utf8(&bytes[cmd_start..j]) {
                    out.push(TemplateBlock {
                        start: i,
                        end: j + 1,
                        command: cmd.to_owned(),
                    });
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn run_template_command(
    raw: &str,
    cwd: &std::path::Path,
) -> Result<String, SkillTemplateError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SkillTemplateError::ShellInjectionForbidden(String::new()));
    }
    if trimmed.starts_with('\\') {
        return Err(SkillTemplateError::BackslashUnknown(trimmed.to_owned()));
    }
    // Reject shell metacharacters outright -- we never go through a shell so
    // these would only appear as literal args; rejecting them keeps the
    // surface obvious.
    for ch in trimmed.chars() {
        if matches!(ch, ';' | '&' | '|' | '$' | '<' | '>' | '`') {
            return Err(SkillTemplateError::ShellInjectionForbidden(
                trimmed.to_owned(),
            ));
        }
    }
    let mut parts = trimmed.split_whitespace();
    let head = parts.next().unwrap_or("");
    if !ALLOWED_COMMANDS.contains(&head) {
        return Err(SkillTemplateError::ShellInjectionForbidden(
            trimmed.to_owned(),
        ));
    }
    let args: Vec<&str> = parts.collect();
    let mut child = Command::new(head);
    child.args(&args).current_dir(cwd);
    let started = OsClock.instant_now();
    let output_result = match child.output() {
        Ok(out) => out,
        Err(_) => {
            return Err(SkillTemplateError::CommandFailed {
                cmd: head.to_owned(),
                exit: -1,
            });
        }
    };
    if started.elapsed() > COMMAND_TIMEOUT {
        return Err(SkillTemplateError::CommandTimeout(head.to_owned()));
    }
    if !output_result.status.success() {
        return Err(SkillTemplateError::CommandFailed {
            cmd: head.to_owned(),
            exit: output_result.status.code().unwrap_or(-1),
        });
    }
    let stdout = String::from_utf8_lossy(&output_result.stdout).trim().to_owned();
    Ok(stdout)
}

fn short_error_label(err: &SkillTemplateError) -> &'static str {
    match err {
        SkillTemplateError::ShellInjectionForbidden(_) => "shell_injection_forbidden",
        SkillTemplateError::CommandFailed { .. } => "command_failed",
        SkillTemplateError::BackslashUnknown(_) => "backslash_unknown",
        SkillTemplateError::CommandTimeout(_) => "command_timeout",
    }
}

/// Redact arguments after the first token in case the SKILL.md body embeds
/// path-like or token-like args.
fn redact_command(raw: &str) -> String {
    let mut iter = raw.split_whitespace();
    match iter.next() {
        Some(head) => {
            if iter.next().is_some() {
                format!("{head} <redacted>")
            } else {
                head.to_owned()
            }
        }
        None => String::new(),
    }
}

fn current_unix_millis() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct CollectingEmitter {
        rows: RefCell<Vec<ClaudeCodeArchitectureEventPayload>>,
    }

    impl CollectingEmitter {
        fn new() -> Self {
            Self {
                rows: RefCell::new(Vec::new()),
            }
        }
    }

    impl SkillTemplateEmitter for CollectingEmitter {
        fn emit(&self, payload: ClaudeCodeArchitectureEventPayload) {
            self.rows.borrow_mut().push(payload);
        }
    }

    struct EnvGuard {
        prior: Option<String>,
    }
    impl EnvGuard {
        fn set(value: &str) -> Self {
            let prior = std::env::var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR).ok();
            unsafe { std::env::set_var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR, value); }
            Self { prior }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prior {
                Some(v) => unsafe { std::env::set_var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR, v); },
                None => unsafe { std::env::remove_var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR); },
            }
        }
    }

    #[test]
    fn gate_off_returns_body_verbatim() {
        let _g = EnvGuard::set("0");
        let skill = SkillTemplate {
            body: "hello !`pwd` world".into(),
            source_path: PathBuf::from("/tmp/SKILL.md"),
        };
        let out = expand_skill_template(&skill, std::path::Path::new("."), &NullEmitter)
            .expect("body verbatim");
        assert_eq!(out, "hello !`pwd` world");
    }

    #[test]
    fn shell_metacharacters_rejected() {
        let _g = EnvGuard::set("1");
        let emitter = CollectingEmitter::new();
        let skill = SkillTemplate {
            body: "x !`pwd; rm -rf /` y".into(),
            source_path: PathBuf::from("/tmp/SKILL.md"),
        };
        let err = expand_skill_template(&skill, std::path::Path::new("."), &emitter)
            .expect_err("metacharacters must be rejected");
        assert!(matches!(
            err,
            SkillTemplateError::ShellInjectionForbidden(_)
        ));
        assert_eq!(emitter.rows.borrow().len(), 1);
    }

    #[test]
    fn unlisted_command_rejected() {
        let _g = EnvGuard::set("1");
        let emitter = CollectingEmitter::new();
        let skill = SkillTemplate {
            body: "x !`rm -rf /` y".into(),
            source_path: PathBuf::from("/tmp/SKILL.md"),
        };
        let err = expand_skill_template(&skill, std::path::Path::new("."), &emitter)
            .expect_err("rm not allowed");
        assert!(matches!(
            err,
            SkillTemplateError::ShellInjectionForbidden(_)
        ));
    }

    #[test]
    fn scanner_handles_no_blocks() {
        let blocks = scan_template_blocks("no template blocks at all");
        assert!(blocks.is_empty());
    }

    #[test]
    fn scanner_finds_single_block() {
        let blocks = scan_template_blocks("a !`pwd` b");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].command, "pwd");
    }

    #[test]
    fn scanner_rejects_block_with_newline() {
        let blocks = scan_template_blocks("a !`pwd\n` b");
        assert!(blocks.is_empty());
    }

    #[test]
    fn redact_hides_args() {
        assert_eq!(redact_command("git status"), "git <redacted>");
        assert_eq!(redact_command("pwd"), "pwd");
    }
}
