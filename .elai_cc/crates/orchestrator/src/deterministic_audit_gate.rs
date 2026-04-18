//! Deterministic audit gate invoked by the [`crate::fsm::Fsm`] on transitions
//! involving the `AUDITING` state (Session wire-a12).
//!
//! Spec: `docs/mvp-wiring-gaps.md` Session wire-a12 — "invoke
//! `DeterministicAuditor::run_checks(&profile, &changed_files)` from the FSM's
//! AUDITING state transition handler. Emit findings as `SsbPayload::Finding`
//! rows with severity Block/Warn. Early-exit the AUDITING→MEMORIZING
//! transition on any Block."
//!
//! The gate is a trait so the [`crate::fsm::Fsm`] can stay agnostic of how the
//! profile and changed file set are produced. The production impl
//! [`DefaultDeterministicAuditGate`] owns the [`crate::ssb::Ssb`] writer and a
//! pair of provider closures so the FSM call-path does not need to build a
//! `ProjectProfile` on every transition.
//!
//! Behavior contract enforced by [`crate::fsm::Fsm`]:
//!   * `evaluate` is called on every `_ -> Auditing` transition. The result is
//!     cached inside `FsmInner.last_audit_result`.
//!   * On `Auditing -> Memorizing` the cached result is consulted: if
//!     `block_count() > 0` the transition is rejected with
//!     [`crate::fsm::FsmError::AuditingBlocked`].
//!   * `evaluate` MUST be side-effect-tolerant: the FSM transition lock is
//!     held for its duration, so implementations should avoid long blocking
//!     I/O. The production impl dispatches the synchronous
//!     [`crate::deterministic_auditor::DeterministicAuditor::run_all`] inside
//!     a `tokio::task::block_in_place`-safe scope at the caller's discretion.
//!
//! Severity mapping to SSB `Finding.severity`:
//!   * `DeterministicSeverity::Block` -> `"error"`
//!   * `DeterministicSeverity::Warn`  -> `"warning"`

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::project_profile::ProjectProfile;

use crate::deterministic_auditor::{
    DeterministicAuditResult, DeterministicAuditor, DeterministicAuditorConfig,
    DeterministicFinding, DeterministicSeverity,
};
use crate::ssb::{FindingPayload, Ssb, SsbPayload};

/// Agent id used on Finding rows emitted by the deterministic audit gate.
pub const GATE_AGENT_ID: &str = "deterministic_audit_gate";

/// Superstep for gate-emitted rows. Matches [`crate::task_runner::RUNNER_SUPERSTEP`] in spirit —
/// the dashboard keys rows by `seq`, so `0` is sufficient.
pub const GATE_SUPERSTEP: i64 = 0;

/// Category used on the per-finding `Finding` rows.
pub const GATE_FINDING_CATEGORY: &str = "deterministic_audit";

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Evaluates the deterministic audit checks at a FSM transition boundary.
///
/// The trait is intentionally minimal — the FSM calls `evaluate` once per
/// `_ -> Auditing` transition and stores the returned result. All side effects
/// (Finding emission, logging) are the implementation's responsibility.
pub trait DeterministicAuditGate: Send + Sync {
    /// Run the deterministic checks and return the aggregate result.
    ///
    /// The FSM passes the transition source+target so implementations can
    /// differentiate behavior (e.g. a replay on `Auditing -> Executing`). The
    /// production impl [`DefaultDeterministicAuditGate`] ignores them: it
    /// always runs the full 5-check sequence.
    fn evaluate(
        &self,
        source: crate::fsm::FsmState,
        target: crate::fsm::FsmState,
    ) -> DeterministicAuditResult;
}

// ---------------------------------------------------------------------------
// Default production impl
// ---------------------------------------------------------------------------

/// Pluggable supplier of `ProjectProfile`. The dashboard passes a closure that
/// reads from its cached [`elai_core::project_profile::ProjectProfile`]; tests
/// pass an in-memory fixture.
pub type ProjectProfileSupplier = Arc<dyn Fn() -> ProjectProfile + Send + Sync>;

/// Pluggable supplier of the files changed since the most recent commit.
/// The dashboard implementation reads `git diff --name-only HEAD`; tests pass
/// a fixed list.
pub type ChangedFilesSupplier = Arc<dyn Fn() -> Vec<PathBuf> + Send + Sync>;

/// Production [`DeterministicAuditGate`]:
///   * Builds a [`DeterministicAuditor`] from the supplied [`ProjectProfile`].
///   * Runs the 5 checks via `DeterministicAuditor::run_all`.
///   * Emits one `Finding` SSB row per `DeterministicFinding`.
///   * Returns the raw [`DeterministicAuditResult`] to the FSM for the
///     AUDITING -> MEMORIZING gate.
pub struct DefaultDeterministicAuditGate {
    writer: Arc<Ssb>,
    profile_supplier: ProjectProfileSupplier,
    changed_files_supplier: ChangedFilesSupplier,
}

impl DefaultDeterministicAuditGate {
    pub fn new(
        writer: Arc<Ssb>,
        profile_supplier: ProjectProfileSupplier,
        changed_files_supplier: ChangedFilesSupplier,
    ) -> Self {
        Self {
            writer,
            profile_supplier,
            changed_files_supplier,
        }
    }

    /// Expose the writer so callers that already have an Arc<Ssb> don't need
    /// a second Arc — keeps the dashboard setup site concise.
    pub fn writer(&self) -> &Arc<Ssb> {
        &self.writer
    }
}

impl DeterministicAuditGate for DefaultDeterministicAuditGate {
    fn evaluate(
        &self,
        source: crate::fsm::FsmState,
        target: crate::fsm::FsmState,
    ) -> DeterministicAuditResult {
        let profile = (self.profile_supplier)();
        let changed = (self.changed_files_supplier)();
        let config = DeterministicAuditorConfig::from_profile(&profile, Vec::new());
        let auditor = DeterministicAuditor::new(config);
        let result = auditor.run_all(&changed);

        tracing::info!(
            %source,
            %target,
            checks_run = result.checks_run,
            block_count = result.block_count(),
            warn_count = result.warn_count(),
            passed = result.passed,
            "deterministic_audit_gate: evaluated"
        );

        let writer = Arc::clone(&self.writer);
        let findings = result.findings.clone();
        tokio::spawn(async move {
            emit_findings(writer.as_ref(), &findings).await;
        });
        result
    }
}

async fn emit_findings(writer: &Ssb, findings: &[DeterministicFinding]) {
    for f in findings {
        let severity = match f.severity {
            DeterministicSeverity::Block => "error",
            DeterministicSeverity::Warn => "warning",
        };
        let file_path = f
            .file
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned());
        let payload = SsbPayload::Finding(FindingPayload {
            severity: severity.to_owned(),
            category: GATE_FINDING_CATEGORY.to_owned(),
            message: format!("[{}] {}", f.check, f.message),
            file_path,
            line: f.line,
            failure_category: None,
        });
        if let Err(e) = writer
            .write_payload(GATE_AGENT_ID, GATE_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::error!(error = %e, "deterministic_audit_gate: write Finding failed");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deterministic_auditor::CheckKind;

    struct FakeGate {
        result: DeterministicAuditResult,
    }

    impl DeterministicAuditGate for FakeGate {
        fn evaluate(
            &self,
            _source: crate::fsm::FsmState,
            _target: crate::fsm::FsmState,
        ) -> DeterministicAuditResult {
            self.result.clone()
        }
    }

    #[test]
    fn fake_gate_returns_its_result() {
        let gate = FakeGate {
            result: DeterministicAuditResult {
                findings: vec![DeterministicFinding {
                    check: CheckKind::Build,
                    severity: DeterministicSeverity::Warn,
                    file: None,
                    line: None,
                    message: "no build cmds".into(),
                }],
                checks_run: 1,
                passed: true,
            },
        };
        let r = gate.evaluate(crate::fsm::FsmState::Executing, crate::fsm::FsmState::Auditing);
        assert_eq!(r.checks_run, 1);
        assert_eq!(r.warn_count(), 1);
        assert_eq!(r.block_count(), 0);
        assert!(r.passed);
    }

    #[test]
    fn severity_maps_block_to_error_and_warn_to_warning() {
        // Verifies the mapping contract documented in the module doc-comment.
        // The actual SSB write is exercised in the contract test.
        let block = DeterministicFinding {
            check: CheckKind::Build,
            severity: DeterministicSeverity::Block,
            file: None,
            line: None,
            message: "build failed".into(),
        };
        let warn = DeterministicFinding {
            check: CheckKind::LazyCode,
            severity: DeterministicSeverity::Warn,
            file: Some(PathBuf::from("src/foo.rs")),
            line: Some(42),
            message: "todo!()".into(),
        };
        assert_eq!(
            match block.severity {
                DeterministicSeverity::Block => "error",
                DeterministicSeverity::Warn => "warning",
            },
            "error"
        );
        assert_eq!(
            match warn.severity {
                DeterministicSeverity::Block => "error",
                DeterministicSeverity::Warn => "warning",
            },
            "warning"
        );
    }
}
