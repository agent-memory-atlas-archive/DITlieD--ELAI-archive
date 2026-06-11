//! H-36 -- NotificationDispatcher: completion / needs-input dev-event alerts.
//!
//! Fires a terminal-escape (OSC 9 / BEL) and an optional OS notification when a
//! task COMPLETES (PhaseComplete) or NEEDS INPUT (AutonomyDecision::RequestHuman).
//! Writes one `agent_notification_emitted` SSB row as the off-agent wiring artifact.
//!
//! Mount points (verified against live source 2026-06-07):
//!   - Completion: task_runner.rs post-PhaseComplete write (after AntiFabricationGate
//!     passes, inside drive_task).
//!   - NeedsInput: task_runner.rs after emit_autonomous_decision for RequestHuman
//!     (fix_and_rerun cap exhausted, ~:1163).
//!
//! OS notification is gated by `ELAI_NOTIFY_ENABLE` (default-OFF kill_switch;
//! see wire-dark-allowlist.toml). Terminal escape and SSB row fire unconditionally.
//!
//! personal_agent_risk FAIL-CLOSED: exactly two code-defined trigger kinds
//! (Completion, NeedsInput). No per-user thresholds, no learned preferences, no
//! quiet-hours store. Adding a third trigger kind requires a code change + this
//! file's revision.
//!
//! Anti-slopsquatting: no `notify-rust` crate. Platform dispatch via
//! `std::process::Command` + `std::io::IsTerminal` (both stdlib, no new dep).

use std::io::IsTerminal;
use std::process::Command;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use tracing::{info, warn};

use elai_core::clock::{ClockProvider, OsClock};

use crate::ssb::{AgentNotificationEmittedPayload, AgentNotificationKind, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// Which event triggered this notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    /// Task reached PhaseComplete (terminal row written, anti-fabrication gate passed).
    Completion,
    /// Autonomous decision escalated to human (fix_and_rerun cap exhausted,
    /// AutonomyDecision::RequestHuman emitted).
    NeedsInput,
}

/// Dispatches completion / needs-input dev-event alerts.
///
/// Created once (via [`NotificationDispatcher::from_env`]) and stored in the
/// component that owns the task lifecycle. Thread-safe: `Clone + Send + Sync`.
///
/// Owns the `ClockProvider` so callers (task_runner, tests) do not need to
/// thread a separate clock reference to the notify call site.
#[derive(Clone)]
pub struct NotificationDispatcher {
    /// Whether to spawn the OS-level notification binary (ELAI_NOTIFY_ENABLE).
    /// Terminal escape and SSB row always fire; this only gates the platform spawn.
    os_enabled: bool,
    /// Clock for ts_ms field in the SSB payload. Injected for test determinism.
    clock: Arc<dyn ClockProvider>,
}

impl std::fmt::Debug for NotificationDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotificationDispatcher")
            .field("os_enabled", &self.os_enabled)
            .finish_non_exhaustive()
    }
}

impl NotificationDispatcher {
    /// Read configuration from the environment using `OsClock` as the clock.
    /// `ELAI_NOTIFY_ENABLE=1` (or `true`/`yes`/`on`) enables the OS notification spawn.
    pub fn from_env() -> Self {
        Self {
            os_enabled: env_truthy("ELAI_NOTIFY_ENABLE"),
            clock: Arc::new(OsClock),
        }
    }

    /// Construct with an explicit clock (for testing / deterministic ts_ms).
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    /// Emit a dev-event notification.
    ///
    /// Order of operations (fail-graceful at every step):
    ///   1. Write the `agent_notification_emitted` SSB row (off-agent artifact, R1/R4).
    ///   2. Emit `\x1b]9;<text>\x07` (OSC 9) when stdout is a TTY.
    ///   3. Emit `\x07` BEL always (unconditional urgency fallback).
    ///   4. Spawn the platform binary when `ELAI_NOTIFY_ENABLE=1` (best-effort).
    ///
    /// No `.unwrap()` / `.expect()` in non-test code; all fallible paths warn via
    /// `tracing::warn` (mirrors hook_dispatcher.rs:1302-1306).
    pub async fn notify(
        &self,
        kind: NotificationKind,
        ssb: &Arc<Ssb>,
        session_id: &str,
    ) {
        let now_ms = self
            .clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let text = match kind {
            NotificationKind::Completion => "ELAI -- Task Complete",
            NotificationKind::NeedsInput => "ELAI -- Needs Input: Run blocked, human decision required",
        };

        let ssb_kind = match kind {
            NotificationKind::Completion => AgentNotificationKind::Completion,
            NotificationKind::NeedsInput => AgentNotificationKind::NeedsInput,
        };

        // surfaces_attempted bit 0 = terminal escape, bit 1 = OS notification.
        let mut surfaces_attempted: u8 = 0b01; // escape always attempted
        let mut surfaces_succeeded: u8 = 0;

        // --- Terminal escape ------------------------------------------------
        // Bit 0: terminal escape (OSC 9 + BEL). OSC 9 fires when stdout is a
        // TTY; BEL fires unconditionally (inert in non-TTY environments).
        let is_tty = std::io::stdout().is_terminal();
        if is_tty {
            // OSC 9 desktop notification escape (iTerm2 / WezTerm / Kitty / Ghostty).
            print!("\x1b]9;{text}\x07");
        }
        // BEL fallback: always fire regardless of TTY state.
        print!("\x07");
        // Flush so BEL reaches the terminal synchronously.
        use std::io::Write;
        if std::io::stdout().flush().is_ok() {
            surfaces_succeeded |= 0b01;
        }

        // --- OS notification spawn ------------------------------------------
        // Bit 1: OS notification (platform binary, gated by ELAI_NOTIFY_ENABLE).
        if self.os_enabled {
            surfaces_attempted |= 0b10;
            let spawned = spawn_os_notify(text);
            if spawned {
                surfaces_succeeded |= 0b10;
            }
            info!(
                kind = ?ssb_kind,
                spawned,
                "notify_dispatch: OS notification dispatch"
            );
        }

        // --- SSB row (off-agent artifact, R1/R4) ----------------------------
        let payload = SsbPayload::AgentNotificationEmitted(AgentNotificationEmittedPayload {
            kind: ssb_kind,
            surfaces_attempted,
            surfaces_succeeded,
            session_id: session_id.to_owned(),
            ts_ms: now_ms,
        });

        if let Err(e) = ssb
            .write_payload(
                "notify_dispatch",
                0,
                payload,
                vec![],
            )
            .await
        {
            // Mirrors hook_dispatcher.rs:1302-1306: warn-on-failure, never panic.
            warn!(
                error = %e,
                kind = ?ssb_kind,
                "notify_dispatch: write agent_notification_emitted SSB row failed"
            );
        } else {
            info!(
                kind = ?ssb_kind,
                session_id,
                surfaces_attempted,
                surfaces_succeeded,
                "notify_dispatch: agent_notification_emitted row written"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Platform dispatch (no new crate dep, anti-slopsquatting)
// ---------------------------------------------------------------------------

/// Returns `true` when the platform binary was successfully spawned (not when
/// the notification was displayed -- that happens asynchronously in the OS).
/// A missing binary (e.g. `notify-send` absent) returns `false`; caller records
/// the surfaces_succeeded bit accordingly.
fn spawn_os_notify(text: &str) -> bool {
    os_notify_impl(text)
}

#[cfg(target_os = "linux")]
fn os_notify_impl(text: &str) -> bool {
    Command::new("notify-send")
        .arg("ELAI")
        .arg(text)
        .spawn()
        .is_ok()
}

#[cfg(target_os = "macos")]
fn os_notify_impl(text: &str) -> bool {
    let script = format!(
        "display notification \"{}\" with title \"ELAI\"",
        text.replace('"', "'")
    );
    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .is_ok()
}

#[cfg(target_os = "windows")]
fn os_notify_impl(text: &str) -> bool {
    // Dependency-free toast via Windows Forms MessageBox (non-blocking approach:
    // use -WindowStyle Hidden to keep the PowerShell window invisible and
    // Background to avoid blocking the main thread wait).
    // anti-slopsquatting: no BurntToast / notify-rust crate.
    let cmd = format!(
        "Add-Type -AssemblyName System.Windows.Forms; \
         [System.Windows.Forms.MessageBox]::Show('{}', 'ELAI', 'OK', 'Information') | Out-Null",
        text.replace('\'', "`'")
    );
    Command::new("powershell")
        .args([
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &cmd,
        ])
        .spawn()
        .is_ok()
}

// Fallback for other targets (e.g. FreeBSD, unknown): no-op, returns false.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn os_notify_impl(_text: &str) -> bool {
    false
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true when an environment variable is set to a truthy value.
fn env_truthy(var: &str) -> bool {
    match std::env::var(var) {
        Ok(raw) => {
            let v = raw.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}
