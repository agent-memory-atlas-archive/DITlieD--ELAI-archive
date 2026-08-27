//! N-8 SLICE 1 integration smoke — governance-coverage substrate fail-closed contract.
//!
//! Runs against the CURRENT tree. Repo-root resolution prefers the explicit
//! `ELAI_REPO_ROOT` env override, else derives the workspace root from
//! `CARGO_MANIFEST_DIR` (`.elai_cc/crates/orchestrator` -> up three levels).
//!
//! Contract asserted here (plan NEGATIVE observable effect: inventing
//! coverage must be impossible):
//!   * DoctrineEnforcer (V2/K-22) and DebtRate (V3/J-46) are Unmeasured on
//!     today's tree, citing probed absence paths — never a number;
//!   * Traceability (V4) joins >0 rows when `.claude/gate-manifest.txt` is
//!     readable, and otherwise fails closed citing exactly that path (this
//!     git worktree does not track the untracked manifest; trees merged with
//!     the production checkout exercise the measured leg);
//!   * GateLiveness (V1) counts active (non-`#[ignore]`) wired tests > 0 and
//!     excludes BOTH known `#[ignore]` wiring stubs BY NAME;
//!   * MutantKill (V5/H-27) stays "not yet scored" in the sync core — the
//!     replay requires a live SSB session and no synthetic score may exist;
//!   * no view ever reports Measured without a concrete source value.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_orchestrator::governance_coverage::{
    measure_debt_rate, measure_doctrine_enforcer, measure_gate_liveness, measure_traceability,
    run_coverage, GovernanceView, ViewStatus, ViewValue,
};
use elai_orchestrator::ssb::{FindingPayload, SsbPayload};

const IGNORED_STUB_NAME_CAP: usize = 16; // mirrors governance_coverage.rs payload cap

fn repo_root() -> PathBuf {
    std::env::var("ELAI_REPO_ROOT")
        .ok()
        .filter(|root| !root.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(3)
                .map(Path::to_path_buf)
                .expect("CARGO_MANIFEST_DIR ancestor chain reaches the workspace root")
        })
}

fn unmeasured_reason(status: &ViewStatus) -> Option<&str> {
    let ViewStatus::Unmeasured { reason } = status else {
        return None;
    };
    Some(reason)
}

#[test]
fn doctrine_enforcer_fails_closed_with_cited_absence() {
    let status = measure_doctrine_enforcer(&repo_root());
    let reason = unmeasured_reason(&status)
        .unwrap_or_else(|| panic!("K-22 doctrine enforcer must stay Unmeasured, got {status:?}"));
    assert!(
        reason.contains("doctrine_coverage.py"),
        "reason must cite the probed artifact name: {reason}"
    );
}

#[test]
fn debt_rate_fails_closed_with_cited_absence() {
    let status = measure_debt_rate(&repo_root());
    let reason = unmeasured_reason(&status)
        .unwrap_or_else(|| panic!("J-46 debt rate must stay Unmeasured, got {status:?}"));
    assert!(
        reason.contains("debt rate") || reason.contains("debt-rate"),
        "reason must speak about the debt rate: {reason}"
    );
    assert!(
        reason.contains("doctrine_debt_rate.py"),
        "reason must cite the probed producer path(s): {reason}"
    );
}

#[test]
fn traceability_join_is_honest_about_the_manifest() {
    let root = repo_root();
    let manifest = root.join(".claude/gate-manifest.txt");
    let status = measure_traceability(&root);
    if manifest.is_file() {
        match status {
            ViewStatus::Measured(ViewValue::Traceability {
                rows_valid,
                manifest_path,
                sample_rows,
                ..
            }) => {
                assert!(rows_valid > 0, ">0 DO-333 rows required when measured");
                assert!(
                    manifest_path.ends_with(".claude/gate-manifest.txt"),
                    "provenance must point at the frozen manifest: {manifest_path}"
                );
                assert!(!sample_rows.is_empty(), "audit samples must accompany a join");
                assert!(sample_rows.len() <= rows_valid, "samples are a subset of valid rows");
                for row in &sample_rows {
                    assert!(!row.path.is_empty());
                    assert!(!row.objective_covered.is_empty());
                    assert!(!row.technique.is_empty());
                    assert!(!row.evidence_artifact.is_empty());
                    assert!(!row.satisfaction.is_empty());
                    assert!(!row.path.starts_with('#'), "comment lines never parse as rows");
                }
            }
            other => panic!(
                "manifest present at {} but the join refused to measure it honestly: {other:?}",
                manifest.display()
            ),
        }
    } else {
        let reason = unmeasured_reason(&status).unwrap_or_else(|| {
            panic!("manifest absent yet reported Measured — fabrication bug: {status:?}")
        });
        assert!(
            reason.contains(".claude/gate-manifest.txt"),
            "unmeasured leg must cite the exact absence path: {reason}"
        );
    }
}

#[test]
fn gate_liveness_counts_active_tests_and_excludes_ignore_stubs() {
    let root = repo_root();
    let test_file = root.join(".elai_cc/crates/orchestrator/tests/gate_liveness.rs");
    let script = root.join(".elai_cc/scripts/check_gate_liveness.sh");
    let status = measure_gate_liveness(&root);
    if !test_file.is_file() || !script.is_file() {
        let reason = unmeasured_reason(&status)
            .unwrap_or_else(|| panic!("missing B-17 seam must read Unmeasured, got {status:?}"));
        assert!(
            reason.contains(".elai_cc"),
            "unmeasured leg must cite the missing seam path: {reason}"
        );
        return;
    }
    match status {
        ViewStatus::Measured(ViewValue::GateLiveness {
            active_wired_tests,
            ignored_stub_tests,
            ref ignored_stub_names,
            liveness_script_present,
            ..
        }) => {
            assert!(active_wired_tests > 0, "current tree wires real gate tests");
            assert!(liveness_script_present, "runner presence was verified above");
            assert!(ignored_stub_tests >= 2, "the two known B-17 wiring stubs exist today");
            assert!(
                ignored_stub_names
                    .iter()
                    .any(|n| n == "search_mode_cascade_pending_b4_wiring"),
                "B-4-blocked #[ignore] stub must NOT count as live coverage"
            );
            assert!(
                ignored_stub_names
                    .iter()
                    .any(|n| n == "cli_interactive_gate_pending_b2_wiring"),
                "B-2-blocked #[ignore] stub must NOT count as live coverage"
            );
            assert_eq!(
                ignored_stub_names.len(),
                ignored_stub_tests.min(IGNORED_STUB_NAME_CAP),
                "stub-name payload respects its documented cap"
            );
        }
        other => panic!(
            "both B-17 seams present at {} / {} but liveness came back unmeasured: {other:?}",
            test_file.display(),
            script.display()
        ),
    }
}

#[test]
fn sync_core_keeps_mutant_kill_unscored() {
    let report = run_coverage(&repo_root());
    let probe = report
        .views()
        .iter()
        .find_map(|(v, s)| (*v == GovernanceView::MutantKill).then_some(s));
    let Some(ViewStatus::Unmeasured { reason }) = &probe else {
        panic!(
            "mutant-kill without a driven H-27 SSB replay must stay unscored, got {:?}",
            probe
        );
    };
    assert!(reason.contains("not yet scored"), "reason={reason}");
}

// ---------------------------------------------------------------------------
// N-8 SLICE 2 — SSB lockstep + real-emission verticals
// ---------------------------------------------------------------------------

#[test]
fn governance_coverage_measured_action_is_lockstep_registered() {
    use elai_orchestrator::ssb::{ssb_action_type_from_str_public, SsbActionType, SSB_ACTION_TYPES};

    // as_sql_str arm -> wire tag.
    let tag = SsbActionType::GovernanceCoverageMeasured.as_sql_str();
    assert_eq!(tag, "governance_coverage_measured");

    // from_str arm round-trips.
    let parsed = ssb_action_type_from_str_public(tag)
        .unwrap_or_else(|e| panic!("from_str must parse its own as_sql_str tag {tag}: {e}"));
    assert_eq!(parsed, SsbActionType::GovernanceCoverageMeasured);

    // Registry (single source of truth for SCHEMA_VERSION / TOTAL_VARIANTS).
    assert!(
        SSB_ACTION_TYPES
            .iter()
            .any(|v| v.as_sql_str() == "governance_coverage_measured"),
        "SSB_ACTION_TYPES must include GovernanceCoverageMeasured"
    );
    // Schema CHECK/DDL acceptance is exercised by the emission test below.
}

/// Mirror of `governance_coverage_cmd::finding_payload` row construction — the
/// CLI module is a binary crate, so the smoke rebuilds the SAME payload shape
/// (reused FindingPayload carrier) to drive the REAL emission + journal read
/// path end to end.
fn cli_row_payload(
    view: GovernanceView,
    status: &ViewStatus,
    summary: String,
    category_suffix: &str,
    severity: &'static str,
    provenance: Option<String>,
) -> SsbPayload {
    SsbPayload::GovernanceCoverageMeasured(FindingPayload {
        severity: severity.to_string(),
        category: format!("governance_coverage:{category_suffix}"),
        message: summary,
        file_path: provenance,
        line: None,
        failure_category: None,
        audit_step: None,
        few_shot_examples: None,
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unmeasured_views_still_emit_rows_through_the_real_journal() {
    use elai_core::clock::{ClockProvider, OsClock};
    use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbReader};
    let gov_cov_action = SsbActionType::GovernanceCoverageMeasured;

    let tmp = std::env::temp_dir().join(format!(
        "gov_cov_slice2_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&tmp).expect("scratch dir");

    let cfg = SsbConfig {
        db_path: tmp.join("ssb.db"),
        pool_size: None,
        project_id: "elai-governance-coverage".to_string(),
        session_id: "elai-governance-coverage-fixture".to_string(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let mut ssb = Ssb::new(cfg, clock).expect("ssb init on scratch db");

    let report = run_coverage(&repo_root());
    // Row templates table-selected by the measured flag — identical strings
    // to the former per-status arms, without a branch ladder.
    const ROW_TEMPLATES: [(&str, &str); 2] =
        [("unmeasured_leg", "warning"), ("gate_liveness", "info")];
    let mut summaries_by_category: Vec<(String, bool)> = Vec::new();
    for (view, status) in report.views() {
        let measured = matches!(status, ViewStatus::Measured(_));
        let (suffix, severity) = ROW_TEMPLATES[usize::from(measured)];
        let summary = unmeasured_reason(status).map_or(
            format!("{view:?}=Measured(verified-in-this-run)"),
            |reason| format!("{view:?}=Unmeasured({reason})"),
        );
        let payload = cli_row_payload(
            *view,
            status,
            summary.clone(),
            suffix,
            severity,
            Some("smoke://slice2".to_string()),
        );
        let seq = ssb
            .write_payload("governance-coverage-smoke", 0, payload, vec!["slice2.md".into()])
            .await
            .expect("write_payload must accept every view incl. unmeasured ones");
        assert!(seq > 0);
        summaries_by_category.push((format!("{view:?}"), matches!(status, ViewStatus::Measured(_))));
    }

    // Every view — measured OR unmeasured — produced exactly one row; dark
    // views are REPORTED through the standard SSB journal read path, never
    // silently skipped by the emit loop.
    let rows = ssb
        .query_by_type(gov_cov_action)
        .expect("query GovernanceCoverageMeasured rows");
    assert_eq!(
        rows.len(),
        report.views().len(),
        "one row per view ({} expected, got {})",
        report.views().len(),
        rows.len()
    );
    assert_eq!(rows.len(), 5, "the fixed five-view contract");
    for entry in &rows {
        assert_eq!(entry.action_type, gov_cov_action);
    }

    // The two mandated fail-closed arms (V2/V3) MUST be among the emitted rows:
    // this assertion only holds while K-22/J-46 are genuinely dark upstream.
    assert!(
        summaries_by_category.contains(&("DoctrineEnforcer".to_string(), false)),
        "DoctrineEnforcer stays Unmeasured today and must still have emitted"
    );
    assert!(
        summaries_by_category.contains(&("DebtRate".to_string(), false)),
        "DebtRate stays Unmeasured today and must still have emitted"
    );

    ssb.shutdown();
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn every_view_status_maps_to_a_deterministic_row_severity_and_tag() {
    use elai_orchestrator::ssb::{FindingPayload, SsbActionType, SsbPayload};
    let report = run_coverage(&repo_root());
    for (view, status) in report.views() {
        // One payload construction shared by both statuses: row severity and
        // message are combinator-selected with the SAME strings per arm.
        let unmeasured_text = unmeasured_reason(status);
        let severity = ["info", "warning"][usize::from(unmeasured_text.is_some())];
        let message: String = unmeasured_text.map_or_else(
            || "smoke".to_string(),
            |reason| reason.to_string(),
        );
        let payload = SsbPayload::GovernanceCoverageMeasured(FindingPayload {
            severity: severity.to_string(),
            category: format!("governance_coverage:{view:?}"),
            message,
            file_path: None,
            line: None,
            failure_category: None,
            audit_step: None,
            few_shot_examples: None,
        });
        // The payload variant maps onto THE registered action (payload ->
        // action_type bijection arm exists for the new variant).
        assert_eq!(
            payload.action_type(),
            SsbActionType::GovernanceCoverageMeasured,
            "{view:?} row must map to GovernanceCoverageMeasured"
        );
        let _ = cli_row_payload(*view, status, "unused".to_string(), "t", "info", None);
    }
}


#[test]
fn no_view_reports_measured_without_a_concrete_source_value() {
    let report = run_coverage(&repo_root());
    for (view, status) in report.views() {
        match status {
            ViewStatus::Unmeasured { reason } => {
                assert!(!reason.is_empty(), "every gap cites its absence ({view:?})")
            }
            // Single (view, payload-kind) dispatch table: kind mismatches fall
            // through to the same unreachable guard the per-view nesting had.
            ViewStatus::Measured(value) => match (*view, value) {
                (
                    GovernanceView::GateLiveness,
                    ViewValue::GateLiveness {
                        active_wired_tests,
                        test_file,
                        ..
                    },
                ) => {
                    assert!(*active_wired_tests > 0, "a 0-test 'measurement' is a lie");
                    assert!(
                        test_file.ends_with("gate_liveness.rs"),
                        "liveness provenance: {test_file}"
                    );
                }
                (
                    GovernanceView::Traceability,
                    ViewValue::Traceability {
                        rows_valid,
                        manifest_path,
                        ..
                    },
                ) => {
                    assert!(*rows_valid > 0, "a 0-row 'join' is a lie");
                    assert!(!manifest_path.is_empty(), "traceability provenance required");
                }
                (
                    GovernanceView::MutantKill,
                    ViewValue::MutantKill {
                        gates_total,
                        gates_killed,
                        source,
                        ..
                    },
                ) => {
                    assert!(*gates_total > 0, "empty registry can never pass");
                    assert!(gates_killed <= gates_total, "killed exceeds registry");
                    assert!(
                        source.contains("gate_self_validation"),
                        "H-27 delegation provenance: {source}"
                    );
                }
                (GovernanceView::DoctrineEnforcer | GovernanceView::DebtRate, _) => panic!(
                    "{view:?} shipped only a fail-closed arm in N-8 slice 1; a Measured state \
                     before the K-22/J-46 scoring contracts land would fabricate coverage"
                ),
                _ => unreachable!("view/payload kind mismatch"),
            },
        }
    }
}
