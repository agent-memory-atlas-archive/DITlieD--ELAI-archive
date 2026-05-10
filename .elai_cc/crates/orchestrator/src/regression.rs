//! B-23 Phase 4H Group E — Cross-Pipeline AC Regression Detector (4H.7C).
//!
//! Port from `github.com/Q00/ouroboros` (MIT, regression.py:39-112). Spans
//! per-AC pass/fail history across all pipeline outcomes; catches slow
//! regressions that single-eval `TestOutcomeReport.pass_to_fail` misses.
//!
//! Spec: docs/specs/guide/steps/phase-4H.txt section 4H.7C (lines 897-970).
//!
//! Algorithm (per latest pipeline outcome):
//!   For each AC index i in latest test_outcomes:
//!     if AC_i is in pass_to_pass or fail_to_pass: skip (passed in latest).
//!     else AC_i FAILED in latest.
//!       search history backward for last pipeline where AC_i passed.
//!       if NOT found: skip (persistent failure, not a regression).
//!       else record regression with consecutive_failures count.
//!
//! Per-mode: caller filters history by pipeline_mode before calling detect.

use crate::self_mod_types::{
    AcRegression, FullPipelineOutcome, PipelineId, RegressionReport, TestOutcomeReport,
};

/// Detect cross-pipeline AC regressions. `history` is ordered **oldest
/// first** with the latest pipeline at `history.last()`. Returns an empty
/// report when fewer than 2 entries are available.
pub fn detect(history: &[FullPipelineOutcome]) -> RegressionReport {
    if history.len() < 2 {
        return RegressionReport::default();
    }
    let latest = &history[history.len() - 1];
    let prior = &history[..history.len() - 1];
    let latest_id = synthetic_pipeline_id(history, history.len() - 1);

    let mut regressions: Vec<AcRegression> = Vec::new();

    // ACs that PASSED in latest — extracted from fail_to_pass + pass_to_pass.
    let latest_passed: std::collections::BTreeSet<&str> = latest
        .test_outcomes
        .fail_to_pass
        .iter()
        .map(String::as_str)
        .chain(latest.test_outcomes.pass_to_pass.iter().map(String::as_str))
        .collect();

    // Group E audit BLOCK fix: build a STABLE per-AC index from the union of
    // all categories sorted lexicographically. Positional iterator indices
    // are unstable across runs — a `Vec<String>` with no ordering guarantee
    // can shuffle the same AC into a different `idx` and downstream
    // deduplication breaks. Sorted text gives a stable canonical position.
    let mut all_acs: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for s in latest
        .test_outcomes
        .fail_to_pass
        .iter()
        .chain(latest.test_outcomes.pass_to_pass.iter())
        .chain(latest.test_outcomes.fail_to_fail.iter())
        .chain(latest.test_outcomes.pass_to_fail.iter())
    {
        all_acs.insert(s.as_str());
    }
    let canonical_index: std::collections::BTreeMap<&str, usize> = all_acs
        .iter()
        .copied()
        .enumerate()
        .map(|(i, s)| (s, i))
        .collect();

    // ACs failing in latest — fail_to_fail + pass_to_fail. Deduplicated
    // via BTreeSet so a single AC listed in both buckets is reported once.
    let latest_failed: std::collections::BTreeSet<&str> = latest
        .test_outcomes
        .fail_to_fail
        .iter()
        .map(String::as_str)
        .chain(latest.test_outcomes.pass_to_fail.iter().map(String::as_str))
        .collect();

    for ac in latest_failed.iter() {
        if latest_passed.contains(ac) {
            continue;
        }
        // Walk history backward looking for a pipeline where `ac` passed.
        let mut last_passed: Option<PipelineId> = None;
        let mut consecutive_failures: usize = 1; // include latest
        for (i, outcome) in prior.iter().enumerate().rev() {
            if ac_passed(&outcome.test_outcomes, ac) {
                last_passed = Some(synthetic_pipeline_id(history, i));
                break;
            }
            if ac_failed(&outcome.test_outcomes, ac) {
                consecutive_failures += 1;
            }
        }
        if let Some(passed_in) = last_passed {
            let canonical_idx = canonical_index
                .get(ac)
                .copied()
                .unwrap_or(0);
            regressions.push(AcRegression {
                ac_index: canonical_idx,
                ac_text: (*ac).to_string(),
                passed_in_pipeline: passed_in,
                failed_in_pipeline: latest_id.clone(),
                consecutive_failures,
            });
        }
    }
    // Sort by canonical index so the report ordering is deterministic.
    regressions.sort_by_key(|r| r.ac_index);

    RegressionReport::from_regressions(regressions)
}

fn ac_passed(report: &TestOutcomeReport, ac: &str) -> bool {
    report.fail_to_pass.iter().any(|s| s == ac) || report.pass_to_pass.iter().any(|s| s == ac)
}

fn ac_failed(report: &TestOutcomeReport, ac: &str) -> bool {
    report.fail_to_fail.iter().any(|s| s == ac) || report.pass_to_fail.iter().any(|s| s == ac)
}

/// Synthetic pipeline-id derived from the history index. Avoids requiring
/// `FullPipelineOutcome` to carry an explicit pipeline_id (none in the
/// spec struct as of Group A) — sufficient for archive bookkeeping.
fn synthetic_pipeline_id(history: &[FullPipelineOutcome], idx: usize) -> PipelineId {
    format!("p{idx}/{}", history[idx].template_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_mod_layer1a::PipelineMode;

    fn outcome(template: &str, fail_to_pass: &[&str], pass_to_pass: &[&str], fail_to_fail: &[&str], pass_to_fail: &[&str]) -> FullPipelineOutcome {
        FullPipelineOutcome {
            template_id: template.into(),
            project_id: "p".into(),
            pipeline_mode: PipelineMode::Surgical,
            phases_total: 0,
            phases_passed_first_try: 0,
            audit_findings_total: 0,
            audit_findings_block: 0,
            test_coverage_final: 0.0,
            time_to_complete_seconds: 0,
            tokens_consumed_total: 0,
            human_interventions: 0,
            rules_triggered: vec![],
            test_outcomes: TestOutcomeReport {
                fail_to_pass: fail_to_pass.iter().map(|s| s.to_string()).collect(),
                pass_to_pass: pass_to_pass.iter().map(|s| s.to_string()).collect(),
                fail_to_fail: fail_to_fail.iter().map(|s| s.to_string()).collect(),
                pass_to_fail: pass_to_fail.iter().map(|s| s.to_string()).collect(),
            },
            etc_trigger_events: 0,
            stale_api_catches: vec![],
            context_budget_truncations: vec![],
            failure_taxonomy_misses: vec![],
        }
    }

    #[test]
    fn empty_history_no_regressions() {
        let r = detect(&[]);
        assert!(!r.has_regressions);
    }

    #[test]
    fn single_outcome_no_regressions() {
        let r = detect(&[outcome("a", &[], &["t1"], &[], &[])]);
        assert!(!r.has_regressions);
    }

    #[test]
    fn ac_passed_then_failed_is_regression() {
        let history = vec![
            outcome("g0", &[], &["t1"], &[], &[]),  // pass at gen 0
            outcome("g1", &[], &[], &["t1"], &[]),  // fail at gen 1
        ];
        let r = detect(&history);
        assert!(r.has_regressions);
        assert_eq!(r.regressions.len(), 1);
        assert_eq!(r.regressions[0].consecutive_failures, 1);
    }

    #[test]
    fn never_passed_is_not_regression() {
        let history = vec![
            outcome("g0", &[], &[], &["t1"], &[]),
            outcome("g1", &[], &[], &["t1"], &[]),
            outcome("g2", &[], &[], &["t1"], &[]),
        ];
        let r = detect(&history);
        assert!(!r.has_regressions, "persistent failure should NOT be regression");
    }

    #[test]
    fn three_consecutive_failures_counted() {
        // pass at g0, fail at g1/g2/g3.
        let history = vec![
            outcome("g0", &[], &["t1"], &[], &[]),
            outcome("g1", &[], &[], &["t1"], &[]),
            outcome("g2", &[], &[], &["t1"], &[]),
            outcome("g3", &[], &[], &["t1"], &[]),
        ];
        let r = detect(&history);
        assert_eq!(r.regressions.len(), 1);
        assert_eq!(r.regressions[0].consecutive_failures, 3);
    }

    #[test]
    fn ac_passed_after_failure_resets_window() {
        // pass at g0, fail at g1, pass at g2, fail at g3.
        let history = vec![
            outcome("g0", &[], &["t1"], &[], &[]),
            outcome("g1", &[], &[], &["t1"], &[]),
            outcome("g2", &["t1"], &[], &[], &[]),
            outcome("g3", &[], &[], &["t1"], &[]),
        ];
        let r = detect(&history);
        assert_eq!(r.regressions.len(), 1);
        assert_eq!(r.regressions[0].consecutive_failures, 1);
    }

    #[test]
    fn pass_to_pass_in_latest_excludes_regression() {
        // AC passed in latest (in pass_to_pass) — must not appear in regressions.
        let history = vec![
            outcome("g0", &[], &[], &["t1"], &[]),
            outcome("g1", &[], &["t1"], &[], &[]),
        ];
        let r = detect(&history);
        assert!(!r.has_regressions);
    }
}
