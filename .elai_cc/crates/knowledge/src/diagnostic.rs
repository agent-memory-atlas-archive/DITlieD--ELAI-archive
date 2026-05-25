//! E-52 Tier-A — diagnostic envelope types for LSP-as-verifier surface.
//!
//! Defines the wire shape any external verifier (rust-analyzer / pyright /
//! tsc --noEmit / svelte-check / future LSPs) emits when it produces a
//! diagnostic about a file. Producer-agnostic: a Tier-B implementation can
//! parse rust-analyzer JSON-RPC into this shape, OR shell out to `tsc
//! --noEmit` and parse stdout into this shape, OR any future LSP.
//!
//! Types-first foundation (E-36.A pattern): wire shape locks NOW so future
//! producers + consumers cannot diverge. Producer wiring + SSB variants +
//! task_runner integration + dashboard panel ship in E-52.B.
//!
//! The full E-52 plan would land:
//!   - DiagnosticProbe trait + 3 concrete probes (rust-analyzer, pyright, tsc)
//!   - 7 new SsbActionType variants (diagnostic_probe_started, _completed,
//!     _failed, _stalled, lsp_oracle_called, ts_bindings_drift, etc.)
//!   - EvidenceGate impl wiring + task_runner emit sites in
//!     orchestrator/{root} (grey-zone)
//!   - dashboard DiagnosticProbePanel.svelte
//! All deferred per workflow/reasoning/E-52-decisions.md.

use serde::{Deserialize, Serialize};

/// Severity tier emitted alongside every diagnostic. Mirrors the LSP 1.x
/// DiagnosticSeverity numeric scale but as named variants so JSON output is
/// stable across LSP-vendor quirks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Compile-time / type-check fatal. Equivalent to LSP severity 1.
    Error,
    /// Probable bug. Equivalent to LSP severity 2.
    Warning,
    /// Style / convention. Equivalent to LSP severity 3.
    Information,
    /// Editor-side suggestion (e.g. unused import). Equivalent to LSP severity 4.
    Hint,
}

impl DiagnosticSeverity {
    /// Stable lower-case tag mirroring serde rename_all.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Information => "information",
            Self::Hint => "hint",
        }
    }

    /// LSP numeric scale (1 = Error, 4 = Hint). Used for filtering thresholds.
    #[must_use]
    pub const fn lsp_severity(&self) -> u8 {
        match self {
            Self::Error => 1,
            Self::Warning => 2,
            Self::Information => 3,
            Self::Hint => 4,
        }
    }
}

/// The kind of verifier that produced a diagnostic. Stable taxonomy so the
/// dashboard + auditor can render / route by kind without parsing the
/// source string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeKind {
    /// rust-analyzer JSON-RPC (textDocument/diagnostics, textDocument/hover).
    RustAnalyzer,
    /// Microsoft Pyright LSP.
    Pyright,
    /// TypeScript compiler --noEmit pass (NOT the language server).
    TscNoEmit,
    /// svelte-check (wraps svelte2tsx + tsc).
    SvelteCheck,
    /// Generic LSP probe — falls through when the verifier is not on the
    /// canonical short list above.
    GenericLsp,
}

impl ProbeKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::RustAnalyzer => "rust_analyzer",
            Self::Pyright => "pyright",
            Self::TscNoEmit => "tsc_no_emit",
            Self::SvelteCheck => "svelte_check",
            Self::GenericLsp => "generic_lsp",
        }
    }
}

/// A single diagnostic — one entry inside a [`DiagnosticProbeOutcome`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// File path relative to project_root (forward-slash separator on all
    /// platforms — Windows producers must normalize).
    pub path: String,
    /// 1-indexed line where the diagnostic anchors (LSP positions are
    /// 0-indexed; producers normalize to 1-indexed for stdout / dashboard
    /// readability).
    pub line: u32,
    /// 1-indexed column (same normalization as `line`).
    pub column: u32,
    /// Severity tier.
    pub severity: DiagnosticSeverity,
    /// Producer-emitted message. Free-form prose; consumers do NOT parse it.
    pub message: String,
    /// Optional error / lint code (e.g. "E0599", "TS2304"). None when the
    /// verifier does not emit one.
    pub code: Option<String>,
}

/// E-52 Tier-B (DRIFT-14 closure) — sibling enum that surfaces probe-run
/// exit conditions WITHOUT breaking the locked Tier-A
/// `DiagnosticProbeOutcome` struct shape (pinned by
/// `crates/contracts/tests/diagnostic_envelope_contract.rs`). The Tier-A
/// outcome reports per-severity diagnostic counts and assumes the probe
/// ran to completion. Tier-B subprocess invocations can also TimeOut or
/// fail because the underlying tool is missing on PATH (RUNPOD-PENDING);
/// this sibling enum carries that information alongside the outcome.
///
/// Serde: snake_case rename so the wire tag matches LSP-canonical
/// shorthand (`completed`, `timed_out`, `tool_missing`).
///
/// Back-compat: this is a NEW enum (additive). The existing
/// `DiagnosticProbeOutcome` gets an optional `run_status` field with
/// `#[serde(default)]` so older payloads (pre-E-52 Tier-B) deserialize
/// cleanly with `Completed` as the implicit default — matching the
/// Tier-A semantics ("counters are valid because the probe completed").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticProbeRunStatus {
    /// Probe subprocess exited cleanly within budget. The per-severity
    /// counters on the carrying [`DiagnosticProbeOutcome`] are
    /// authoritative.
    #[default]
    Completed,
    /// Probe wall-clock budget exceeded. The carrying outcome's counters
    /// may be partial; the auditor MUST treat this as a soft-fail
    /// (anti-rubber-stamp per `workflow/reasoning/E-52-decisions.md`).
    TimedOut,
    /// Toolchain binary not on PATH (e.g. `rust-analyzer` absent on CI
    /// lane). The outcome's counters are zero by construction; the
    /// auditor MUST treat this as a soft warning, NOT a hard pass —
    /// missing toolchain != clean run. RUNPOD-PENDING marker.
    ToolMissing,
}

impl DiagnosticProbeRunStatus {
    /// Stable lower-snake-case tag mirroring serde rename_all. Used by
    /// SSB beacon hashing (`DiagnosticProbeCompletedPayload.run_status_hash`).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::TimedOut => "timed_out",
            Self::ToolMissing => "tool_missing",
        }
    }

    /// True only for `Completed`. The auditor uses this to discriminate
    /// the "probe is authoritative" path from the soft-fail paths.
    #[must_use]
    pub const fn is_authoritative(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// One probe run's full output. Producers emit exactly one
/// `DiagnosticProbeOutcome` per probe invocation (per file or per project
/// depending on probe).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticProbeOutcome {
    /// Which verifier produced this.
    pub kind: ProbeKind,
    /// Number of diagnostics emitted, by severity. Convenience aggregate so
    /// the dashboard can show "3 errors, 7 warnings" without iterating the
    /// full diagnostics vec.
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    pub hint_count: u32,
    /// Wall-clock duration of the probe (ms). Used for the latency budget
    /// check the auditor will run (Tier-B) — "did the probe time out?".
    pub duration_ms: u32,
    /// The diagnostics themselves. Bounded by the probe's own cap (Tier-B
    /// probes will truncate at 1000 / probe to avoid OOM on broken
    /// codebases).
    pub diagnostics: Vec<Diagnostic>,
    /// E-52 Tier-B (DRIFT-14 closure) — exit condition of the probe run.
    /// Defaults to `Completed` for back-compat with Tier-A payloads that
    /// pre-date this field (older producers + the locked Tier-A envelope
    /// contract test do not emit `run_status`; serde fills it with
    /// `Completed` so the counters above are honoured as authoritative).
    /// Tier-B subprocess producers MUST set this explicitly.
    #[serde(default)]
    pub run_status: DiagnosticProbeRunStatus,
}

impl DiagnosticProbeOutcome {
    /// True when no errors were emitted. The Tier-B auditor verdict gates on
    /// this for the diagnostic-as-evidence rule (rust-analyzer + tsc must
    /// both return error_count == 0 to pass).
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.error_count == 0
    }

    /// Total diagnostic count across all severities. Equivalent to
    /// `self.diagnostics.len()` but constant-time from the counters.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.error_count + self.warning_count + self.info_count + self.hint_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_lsp_scale_round_trips() {
        for sev in [
            DiagnosticSeverity::Error,
            DiagnosticSeverity::Warning,
            DiagnosticSeverity::Information,
            DiagnosticSeverity::Hint,
        ] {
            // LSP scale is 1..=4, monotonically increasing in "less severe".
            assert!((1..=4).contains(&sev.lsp_severity()));
            assert!(!sev.as_str().is_empty());
            assert_eq!(sev.as_str().to_ascii_lowercase(), sev.as_str());
        }
        // Order check: Error < Warning < Information < Hint on the LSP scale.
        assert!(
            DiagnosticSeverity::Error.lsp_severity()
                < DiagnosticSeverity::Warning.lsp_severity()
        );
        assert!(
            DiagnosticSeverity::Warning.lsp_severity()
                < DiagnosticSeverity::Information.lsp_severity()
        );
        assert!(
            DiagnosticSeverity::Information.lsp_severity()
                < DiagnosticSeverity::Hint.lsp_severity()
        );
    }

    #[test]
    fn probe_kind_str_is_lower_snake_case() {
        for kind in [
            ProbeKind::RustAnalyzer,
            ProbeKind::Pyright,
            ProbeKind::TscNoEmit,
            ProbeKind::SvelteCheck,
            ProbeKind::GenericLsp,
        ] {
            let s = kind.as_str();
            assert!(s.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
        }
    }

    #[test]
    fn outcome_aggregates_are_consistent() {
        let outcome = DiagnosticProbeOutcome {
            kind: ProbeKind::RustAnalyzer,
            error_count: 2,
            warning_count: 5,
            info_count: 1,
            hint_count: 0,
            duration_ms: 420,
            diagnostics: vec![],
            run_status: DiagnosticProbeRunStatus::Completed,
        };
        assert_eq!(outcome.total(), 8);
        assert!(!outcome.is_clean());
        let clean = DiagnosticProbeOutcome {
            kind: ProbeKind::TscNoEmit,
            error_count: 0,
            warning_count: 3,
            info_count: 0,
            hint_count: 0,
            duration_ms: 200,
            diagnostics: vec![],
            run_status: DiagnosticProbeRunStatus::Completed,
        };
        assert!(clean.is_clean());
        assert_eq!(clean.total(), 3);
    }

    #[test]
    fn diagnostic_serde_round_trip() {
        let d = Diagnostic {
            path: "crates/cli/src/main.rs".to_string(),
            line: 42,
            column: 7,
            severity: DiagnosticSeverity::Error,
            message: "expected `,`, found `;`".to_string(),
            code: Some("E0001".to_string()),
        };
        let json = serde_json::to_string(&d).expect("serialize ok");
        let back: Diagnostic = serde_json::from_str(&json).expect("deserialize ok");
        assert_eq!(back, d);
    }
}
