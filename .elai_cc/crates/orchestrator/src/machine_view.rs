//! Machine-derived projection for the optional binary audit lane (J-88 E3).
//!
//! This module deliberately contains only structural and numeric inputs.  The
//! source-bearing deterministic findings remain available to the advisory SSB
//! path, but are never copied into `MachineView`.

/// Kill-switch env (default OFF).
pub const ELAI_MACHINE_AUDIT_LANE_ENABLE_ENV: &str = "ELAI_MACHINE_AUDIT_LANE_ENABLE";

/// Return whether the machine-derived binary lane is enabled.
pub fn machine_audit_lane_enabled() -> bool {
    std::env::var(ELAI_MACHINE_AUDIT_LANE_ENABLE_ENV)
        .map(|v| !v.trim().is_empty() && v != "0")
        .unwrap_or(false)
}

/// Machine-derived-only binary-lane input.  No source text fields are stored.
#[derive(Debug, Clone, PartialEq)]
pub struct MachineView {
    /// Count of Block-severity checker findings (structural).
    pub block_count: u64,
    /// Count of Warn-severity checker findings.
    pub warn_count: u64,
    /// Check kinds that contributed, never finding messages.
    pub check_kinds: Vec<String>,
    /// Optional SARIF rule IDs (empty if none provided).
    pub sarif_rule_ids: Vec<String>,
    /// Optional coverage branch percent 0.0..100.0.
    pub coverage_branch_pct: Option<f64>,
    /// Optional unmatched-observer count from the J-87 feed.
    pub unmatched_observer_count: Option<u64>,
    /// Optional attestation or bundle digest.
    pub attestation_digest: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum MachineViewError {
    #[error("machine_view binary lane rejected source-bearing field: {0}")]
    SourcePresent(String),
}

/// Project a machine-only view from a deterministic audit result.
pub fn project_from_audit_result(
    result: &crate::deterministic_auditor::DeterministicAuditResult,
) -> MachineView {
    let mut check_kinds: Vec<String> = result
        .findings
        .iter()
        .map(|finding| finding.check.to_string())
        .collect();
    check_kinds.sort();
    check_kinds.dedup();
    MachineView {
        block_count: result.block_count() as u64,
        warn_count: result.warn_count() as u64,
        check_kinds,
        sarif_rule_ids: Vec::new(),
        coverage_branch_pct: None,
        unmatched_observer_count: None,
        attestation_digest: None,
    }
}

fn validate_string(label: &str, value: &str) -> Result<(), MachineViewError> {
    if value.contains('\n') || value.contains('\r') || value.contains("fn ") {
        return Err(MachineViewError::SourcePresent(label.to_owned()));
    }
    Ok(())
}

/// Fail closed when a string field contains an obvious source-bearing payload.
pub fn validate_no_source(view: &MachineView) -> Result<(), MachineViewError> {
    for kind in &view.check_kinds {
        validate_string("check_kinds", kind)?;
    }
    for rule_id in &view.sarif_rule_ids {
        validate_string("sarif_rule_ids", rule_id)?;
    }
    if let Some(digest) = &view.attestation_digest {
        validate_string("attestation_digest", digest)?;
    }
    Ok(())
}

/// Binary verdict from machine view only: pass iff no Block findings exist.
pub fn binary_passed(view: &MachineView) -> bool {
    view.block_count == 0
}

#[cfg(test)]
mod j88_machine_view_tests {
    use super::*;
    use crate::deterministic_auditor::{
        CheckKind, DeterministicAuditResult, DeterministicFinding, DeterministicSeverity,
    };

    fn result(message: &str, severity: DeterministicSeverity) -> DeterministicAuditResult {
        DeterministicAuditResult {
            findings: vec![DeterministicFinding {
                check: CheckKind::LazyCode,
                severity,
                file: None,
                line: None,
                message: message.to_owned(),
            }],
            checks_run: 1,
            passed: severity != DeterministicSeverity::Block,
            mutation: None,
            mutation_budget: None,
        }
    }

    #[test]
    fn project_drops_finding_messages() {
        let view = project_from_audit_result(&result(
            "fn safe_delete() { std::fs::remove_dir_all(\"/\"); }",
            DeterministicSeverity::Block,
        ));
        let rendered = format!("{view:?}");
        assert_eq!(view.block_count, 1);
        assert!(!rendered.contains("safe_delete"));
        assert!(!rendered.contains("remove_dir"));
    }

    #[test]
    fn binary_verdict_ignores_finding_message_prose() {
        let first = project_from_audit_result(&result("safe_delete", DeterministicSeverity::Warn));
        let second = project_from_audit_result(&result("x9", DeterministicSeverity::Warn));
        assert_eq!(first, second);
        assert_eq!(binary_passed(&first), binary_passed(&second));
    }

    #[test]
    fn validate_rejects_source_bearing_view() {
        let mut view = MachineView {
            block_count: 0,
            warn_count: 0,
            check_kinds: vec!["build".to_owned()],
            sarif_rule_ids: Vec::new(),
            coverage_branch_pct: None,
            unmatched_observer_count: None,
            attestation_digest: Some("fn foo() { source }".to_owned()),
        };
        assert!(validate_no_source(&view).is_err());
        view.attestation_digest = None;
        view.check_kinds.push("fn foo".to_owned());
        assert!(validate_no_source(&view).is_err());
    }

    #[test]
    fn flag_default_off() {
        if std::env::var(ELAI_MACHINE_AUDIT_LANE_ENABLE_ENV).is_err() {
            assert!(!machine_audit_lane_enabled());
        }
    }
}
