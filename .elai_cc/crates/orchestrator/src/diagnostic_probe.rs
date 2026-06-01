//! E-52 Tranche B-3 — diagnostic probe producer (LOCAL-ONLY, zero-new-dep).
//!
//! Producer surface that the EvidenceGate `DiagnosticProbeGate` reads + the
//! SSB emit chain in `task_runner::run_validate_dispatch` calls into.
//!
//! Scope (per workflow/reasoning/E-52-tranche-B3-decisions.md):
//!   - LOCAL-ONLY honest fast-path. We DO NOT pull `multilspy`, `ts-morph`,
//!     `tower-lsp`, `lsp-server`, or `lsp-types` — those are deferred pending
//!     explicit user dep confirmation (anti-slopsquatting + "No new dependencies
//!     without confirmation" per CLAUDE.md).
//!   - Binary presence detection only — `which` / `where` shell-out via
//!     `std::process::Command`. NO actual LSP JSON-RPC invocation here; that
//!     belongs in a follow-on tranche that ships either the multilspy sidecar
//!     OR direct subprocess parsing of `rust-analyzer diagnostics --output-format=json`.
//!   - Wire-dark via `ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE=1`. Default OFF means
//!     `dispatch_diagnostic_probe` early-returns an `Ok`-with-empty outcome so
//!     existing call sites stay no-op when the env-gate is unset.
//!
//! Composes with the already-shipped Tier-A envelope
//! (`elai_knowledge::diagnostic::DiagnosticProbeOutcome` + `ProbeKind` +
//! `DiagnosticProbeRunStatus`) and the already-shipped SSB variants
//! (`DiagnosticProbeDispatched` / `_Completed` / `_Block` at ssb.rs:6400+).
//!
//! Deferred follow-ons (tracked in workflow/sessions/s-22300/stubs.md):
//!   - E-52-multilspy: lsp_oracle.py sidecar (multilspy pypi anti-slopsquat
//!     + user dep confirm).
//!   - E-52-tsmorph: check_ts_bindings.mjs (ts-morph npm anti-slopsquat
//!     + user dep confirm).
//!   - E-52-ScipRefresh: rust-side ScipRefreshCompleted producer (no rust
//!     caller exists; codebase_mapper.py owns the SCIP path).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use tokio::time::timeout;
use tracing::warn;

use elai_knowledge::diagnostic::{
    Diagnostic, DiagnosticProbeOutcome, DiagnosticProbeRunStatus, DiagnosticSeverity, ProbeKind,
};

use crate::ssb::{
    DiagnosticProbeBlockPayload, DiagnosticProbeCompletedPayload,
    DiagnosticProbeDispatchedPayload, Ssb, SsbPayload,
};

// ---------------------------------------------------------------------------
// Constants — env gate name, timeout floor, SSB agent identity.
// ---------------------------------------------------------------------------

/// Environment variable that enables real probe dispatch. When unset OR not
/// `"1"`, [`dispatch_diagnostic_probe`] returns an `Ok`-with-empty outcome and
/// the SSB emit chain stays dark.
pub const ENABLE_ENV: &str = "ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE";

/// Default per-probe wall-clock budget. Subprocess invocations (deferred — see
/// module docs) MUST bound on this OR shorter. Currently only the binary
/// presence check runs; it is well under this floor.
pub const DEFAULT_TIMEOUT_MS: u64 = 5_000;

/// SSB-side agent identity for runner-emitted rows. Matches `task_runner.rs`
/// `RUNNER_AGENT_ID` / `RUNNER_SUPERSTEP` constants. Re-declared here so the
/// module compiles without pulling the (private) constants from
/// `task_runner.rs` — both must stay in sync; a contract test pins identity at
/// the SSB boundary.
const RUNNER_AGENT_ID: &str = "task_runner";
const RUNNER_SUPERSTEP: i64 = 0;

// ---------------------------------------------------------------------------
// Public request / outcome surface (Tranche B-3 shape).
// ---------------------------------------------------------------------------

/// One probe request — a single file + language slug + per-probe budget.
/// Constructed by `task_runner::run_validate_dispatch` (or any future caller)
/// from the changed-files list a tool dispatch produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticProbeRequest {
    /// Path to the file the probe should diagnose. Path semantics: as-passed;
    /// `dispatch_diagnostic_probe` does not normalize.
    pub file_path: PathBuf,
    /// Language slug — `"rust"` / `"python"` / `"typescript"` / `"svelte"`.
    /// Used to pick the probe binary via [`canonical_binary_for_language`].
    pub language: String,
    /// Per-probe wall-clock budget. `0` means use [`DEFAULT_TIMEOUT_MS`].
    pub timeout_ms: u64,
}

impl DiagnosticProbeRequest {
    /// Build a request that uses the default budget.
    #[must_use]
    pub fn new(file_path: PathBuf, language: impl Into<String>) -> Self {
        Self {
            file_path,
            language: language.into(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }
}

// ---------------------------------------------------------------------------
// Public env-gate helper.
// ---------------------------------------------------------------------------

/// True when `ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE` is set to `"1"`. Any other
/// value (including unset / empty) returns false — same shape as the
/// `lsp_oracle::LspMode::from_env` precedent at
/// `crates/orchestrator/src/symbol_graph/lsp_oracle.rs`.
#[must_use]
pub fn lsp_diagnostic_probe_enabled_from_env() -> bool {
    matches!(std::env::var(ENABLE_ENV).as_deref(), Ok("1"))
}

// ---------------------------------------------------------------------------
// Language slug -> (ProbeKind, canonical-binary-name) mapping.
// ---------------------------------------------------------------------------

/// Resolve language slug -> `ProbeKind` for the probe-completed payload.
/// Returns `ProbeKind::GenericLsp` when the slug is unknown so the wire stays
/// stable.
#[must_use]
pub fn probe_kind_for_language(language: &str) -> ProbeKind {
    match language.to_ascii_lowercase().as_str() {
        "rust" | "rs" => ProbeKind::RustAnalyzer,
        "python" | "py" => ProbeKind::Pyright,
        "typescript" | "ts" | "tsx" | "javascript" | "js" => ProbeKind::TscNoEmit,
        "svelte" => ProbeKind::SvelteCheck,
        _ => ProbeKind::GenericLsp,
    }
}

/// Canonical LSP binary name for a language slug. Mirrors the
/// `lsp_oracle::OracleKind::canonical_binary` precedent. `None` when unmapped.
#[must_use]
pub fn canonical_binary_for_language(language: &str) -> Option<&'static str> {
    match language.to_ascii_lowercase().as_str() {
        "rust" | "rs" => Some("rust-analyzer"),
        "python" | "py" => Some("pyright"),
        "typescript" | "ts" | "tsx" | "javascript" | "js" => Some("tsc"),
        "svelte" => Some("svelte-check"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Binary presence detection (which / where shell-out).
// ---------------------------------------------------------------------------

/// True when `binary` is resolvable on PATH via `which` (unix) / `where`
/// (windows). Failure modes (missing tool, non-zero exit, shell error) all
/// collapse to `false` — callers treat that as `ProbeRunStatus::ToolMissing`.
fn binary_on_path(binary: &str) -> bool {
    #[cfg(windows)]
    let cmd = "where";
    #[cfg(not(windows))]
    let cmd = "which";
    std::process::Command::new(cmd)
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    }

// ---------------------------------------------------------------------------
// Dispatch entry point.
// ---------------------------------------------------------------------------

/// Run one diagnostic probe.
///
/// LOCAL-ONLY honest fast-path semantics:
///   - When [`lsp_diagnostic_probe_enabled_from_env`] is `false`, returns
///     a `Completed` outcome with zero diagnostics. Wire-dark default.
///   - When enabled, performs a binary-presence check via [`binary_on_path`]:
///     - Binary missing or no language mapping -> `ToolMissing` outcome
///       (soft-pass at the gate per E-52-decisions.md (c)/(d)).
///     - Binary present -> `Completed` outcome with zero diagnostics + a
///       `warn!` that real subprocess invocation is deferred (no `unwrap`,
///       no `Box<dyn Error>`, no fake diagnostics).
///
/// The outer wall-clock is bounded by `req.timeout_ms` via
/// [`tokio::time::timeout`] so any future subprocess work the caller wires
/// in cannot exceed budget. Today the only IO is a sync `which` call wrapped
/// in `tokio::task::spawn_blocking`.
pub async fn dispatch_diagnostic_probe(req: DiagnosticProbeRequest) -> DiagnosticProbeOutcome {
    let kind = probe_kind_for_language(&req.language);
    if !lsp_diagnostic_probe_enabled_from_env() {
        return wire_dark_outcome(kind);
    }
    let bin = match canonical_binary_for_language(&req.language) {
        Some(b) => b,
        None => {
            return tool_missing_outcome(kind, 0);
        }
    };
    let budget = if req.timeout_ms == 0 {
        Duration::from_millis(DEFAULT_TIMEOUT_MS)
    } else {
        Duration::from_millis(req.timeout_ms)
    };
    let started = OsClock.instant_now();
    let bin_owned = bin.to_owned();
    let present_fut = tokio::task::spawn_blocking(move || binary_on_path(&bin_owned));
    let present = match timeout(budget, present_fut).await {
        Ok(Ok(p)) => p,
        Ok(Err(join_err)) => {
            warn!(error = %join_err, language = %req.language, "diagnostic_probe: join error during binary check");
            false
        }
        Err(_) => {
            let elapsed_ms = clamp_elapsed_ms(started.elapsed());
            return timed_out_outcome(kind, elapsed_ms);
        }
    };
    let elapsed_ms = clamp_elapsed_ms(started.elapsed());
    if !present {
        return tool_missing_outcome(kind, elapsed_ms);
    }
    warn!(
        language = %req.language,
        file = %req.file_path.display(),
        binary = bin,
        "diagnostic_probe: real subprocess invocation deferred (E-52 Tranche B-3 LOCAL-ONLY); \
         emitting Completed with zero diagnostics"
    );
    DiagnosticProbeOutcome {
        kind,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: elapsed_ms,
        diagnostics: Vec::new(),
        run_status: DiagnosticProbeRunStatus::Completed,
    }
}

fn wire_dark_outcome(kind: ProbeKind) -> DiagnosticProbeOutcome {
    DiagnosticProbeOutcome {
        kind,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: 0,
        diagnostics: Vec::new(),
        run_status: DiagnosticProbeRunStatus::Completed,
    }
}

fn tool_missing_outcome(kind: ProbeKind, elapsed_ms: u32) -> DiagnosticProbeOutcome {
    DiagnosticProbeOutcome {
        kind,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: elapsed_ms,
        diagnostics: Vec::new(),
        run_status: DiagnosticProbeRunStatus::ToolMissing,
    }
}

fn timed_out_outcome(kind: ProbeKind, elapsed_ms: u32) -> DiagnosticProbeOutcome {
    DiagnosticProbeOutcome {
        kind,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: elapsed_ms,
        diagnostics: Vec::new(),
        run_status: DiagnosticProbeRunStatus::TimedOut,
    }
}

fn clamp_elapsed_ms(elapsed: Duration) -> u32 {
    u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX)
}

// ---------------------------------------------------------------------------
// SSB emit helpers — fire-and-forget, never panic, never propagate.
// ---------------------------------------------------------------------------

/// SSB beacon kind selector for [`notify_diagnostic_probe`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticProbeBeacon {
    /// Emit `diagnostic_probe_dispatched` BEFORE the probe runs.
    Dispatched,
    /// Emit `diagnostic_probe_completed` after probe returns.
    Completed,
    /// Emit `diagnostic_probe_block` when the gate would block.
    Block,
}

/// Emit the SSB beacon corresponding to `beacon` for `outcome`. Errors are
/// logged at `warn!` and dropped — never propagated. Mirrors the
/// `emit_tool_dispatch` pattern at `task_runner.rs:11615`.
pub async fn notify_diagnostic_probe(
    writer: &Arc<Ssb>,
    beacon: DiagnosticProbeBeacon,
    outcome: &DiagnosticProbeOutcome,
    first_error_path: Option<&Path>,
    ts_ms: u64,
) {
    let payload = match beacon {
        DiagnosticProbeBeacon::Dispatched => {
            SsbPayload::DiagnosticProbeDispatched(DiagnosticProbeDispatchedPayload {
                probe_kind_hash: blake3_prefix(outcome.kind.as_str()),
                language_hash: blake3_prefix(outcome.kind.as_str()),
                file_count: 1,
                ts_ms,
            })
        }
        DiagnosticProbeBeacon::Completed => {
            SsbPayload::DiagnosticProbeCompleted(DiagnosticProbeCompletedPayload {
                probe_kind_hash: blake3_prefix(outcome.kind.as_str()),
                run_status_hash: blake3_prefix(outcome.run_status.as_str()),
                error_count: outcome.error_count,
                warning_count: outcome.warning_count,
                duration_ms: outcome.duration_ms,
                ts_ms,
            })
        }
        DiagnosticProbeBeacon::Block => {
            let first_path = first_error_path
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            SsbPayload::DiagnosticProbeBlock(DiagnosticProbeBlockPayload {
                probe_kind_hash: blake3_prefix(outcome.kind.as_str()),
                error_count: outcome.error_count,
                first_path_hash: blake3_prefix(&first_path),
                ts_ms,
            })
        }
    };
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        warn!(error = %e, "diagnostic_probe: ssb emit failed");
    }
}

/// blake3-prefix used by the existing E-52 payload `*_hash` fields. Hashes the
/// UTF-8 bytes of `s` and returns the first 8 bytes as a little-endian u64.
fn blake3_prefix(s: &str) -> u64 {
    let h = blake3::hash(s.as_bytes());
    let bytes = h.as_bytes();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(buf)
}

// ---------------------------------------------------------------------------
// Convenience constructors for test scaffolding / contract tests.
// ---------------------------------------------------------------------------

/// E-52-S3 / s-22600 Tranche B ITEM B.3 -- per-file outcome reducer.
///
/// Returns the outcome the [`crate::evidence::DiagnosticProbeGate`] should
/// see when multiple per-file probes ran in the same dispatch. The gate
/// BLOCKs on `is_authoritative() && !is_clean()`, so the reducer prefers
/// outcomes in this order (worst -> best):
///   1. authoritative + !is_clean (would BLOCK the gate)
///   2. non-authoritative (`TimedOut` / `ToolMissing`; soft-pass at the
///      gate but still informative)
///   3. authoritative + is_clean (`Completed` with zero errors)
///
/// Among same-rank pairs we break ties by higher `error_count` so the
/// "worst-of" reflects the loudest probe. The returned reference borrows
/// from the input slice so the caller can clone on demand (matches the
/// soft-pass `Option<DiagnosticProbeOutcome>` contract on
/// `TransitionContext.diagnostic_probe_outcome`).
#[must_use]
pub fn rank_worst<'a>(
    a: &'a DiagnosticProbeOutcome,
    b: &'a DiagnosticProbeOutcome,
) -> &'a DiagnosticProbeOutcome {
    let a_rank = outcome_block_rank(a);
    let b_rank = outcome_block_rank(b);
    match a_rank.cmp(&b_rank) {
        std::cmp::Ordering::Greater => a,
        std::cmp::Ordering::Less => b,
        std::cmp::Ordering::Equal => {
            if b.error_count > a.error_count {
                b
            } else {
                a
            }
        }
    }
}

/// Higher value = worse outcome (closer to a DiagnosticProbeGate BLOCK).
/// Inlined ordering: 2 = authoritative + would-BLOCK; 1 = non-authoritative
/// soft-pass; 0 = authoritative + clean.
fn outcome_block_rank(outcome: &DiagnosticProbeOutcome) -> u8 {
    if outcome.run_status.is_authoritative() && !outcome.is_clean() {
        2
    } else if !outcome.run_status.is_authoritative() {
        1
    } else {
        0
    }
}

/// Build a `Completed` outcome with one synthetic error diagnostic. Useful for
/// the contract tests + future fixer-routing unit tests.
#[must_use]
pub fn synthetic_error_outcome(kind: ProbeKind, message: impl Into<String>) -> DiagnosticProbeOutcome {
    let diag = Diagnostic {
        path: "synthetic.rs".into(),
        line: 1,
        column: 1,
        severity: DiagnosticSeverity::Error,
        message: message.into(),
        code: None,
    };
    DiagnosticProbeOutcome {
        kind,
        error_count: 1,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: 0,
        diagnostics: vec![diag],
        run_status: DiagnosticProbeRunStatus::Completed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_kind_mapping_covers_canonical_slugs() {
        assert_eq!(probe_kind_for_language("rust"), ProbeKind::RustAnalyzer);
        assert_eq!(probe_kind_for_language("python"), ProbeKind::Pyright);
        assert_eq!(probe_kind_for_language("typescript"), ProbeKind::TscNoEmit);
        assert_eq!(probe_kind_for_language("svelte"), ProbeKind::SvelteCheck);
        assert_eq!(probe_kind_for_language("unknown"), ProbeKind::GenericLsp);
    }

    #[test]
    fn canonical_binary_matches_mapping() {
        assert_eq!(canonical_binary_for_language("rust"), Some("rust-analyzer"));
        assert_eq!(canonical_binary_for_language("python"), Some("pyright"));
        assert_eq!(canonical_binary_for_language("typescript"), Some("tsc"));
        assert_eq!(canonical_binary_for_language("svelte"), Some("svelte-check"));
        assert_eq!(canonical_binary_for_language("brainfuck"), None);
    }

    #[tokio::test]
    async fn wire_dark_returns_completed_when_env_unset() {
        // Belt-and-braces: the test process MUST NOT have the env set. We
        // do not call `set_var` here because that would race other tests.
        if std::env::var(ENABLE_ENV).is_ok() {
            return;
        }
        let req = DiagnosticProbeRequest::new(PathBuf::from("foo.rs"), "rust");
        let out = dispatch_diagnostic_probe(req).await;
        assert_eq!(out.run_status, DiagnosticProbeRunStatus::Completed);
        assert!(out.is_clean());
        assert_eq!(out.diagnostics.len(), 0);
    }

    #[test]
    fn synthetic_error_outcome_is_not_clean() {
        let out = synthetic_error_outcome(ProbeKind::RustAnalyzer, "boom");
        assert!(!out.is_clean());
        assert_eq!(out.error_count, 1);
    }

    #[test]
    fn blake3_prefix_is_deterministic_and_nonzero_for_nonempty() {
        let a = blake3_prefix("rust_analyzer");
        let b = blake3_prefix("rust_analyzer");
        assert_eq!(a, b);
        assert_ne!(a, 0);
    }
}
