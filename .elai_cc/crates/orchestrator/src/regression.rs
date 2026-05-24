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
//!
//! E-30 failure taxonomy additions (5 new categories mirrored in
//! .claude/tools/failure_taxonomy.py CATEGORY_PATTERNS):
//!   "argv_denied"         -- check_argv() Deny hit; tool blocked pre-dispatch.
//!   "trust_dialog_block"  -- ensure_session_trust() refused hook/env load.
//!   "mcp_pin_refused"     -- MCP server version refused by exact-pin policy.
//!   "lineage_collision"   -- auditor+executor share same lineage group (autonomous=BLOCK).
//!   "untrusted_mem_write" -- TextSource::ExternalTool attempted a memory write.

use crate::self_mod_types::{
    AcRegression, FullPipelineOutcome, PipelineId, RegressionReport, TestOutcomeReport,
};

// ---------------------------------------------------------------------------
// E-30: Named failure-taxonomy entries (Rust-side parallel to
// .claude/tools/failure_taxonomy.py CATEGORY_PATTERNS).
//
// The 5 plan-named symptom categories surfaced by the public-incident
// hardening work. Each entry pairs the canonical category name (used by the
// .py classifier and by downstream counter consumers) with the SSB action
// string emitted when the safeguard fires. Keep this list in lock-step with
// failure_taxonomy.py and with the SsbActionType variants -- a category
// without a producer is a tombstone, and a producer without a category line
// here is invisible to the regression detector.
// ---------------------------------------------------------------------------

/// One named taxonomy entry: the (category_name, ssb_action_type, description)
/// triple referenced by both Rust-side counters and the Python classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct E30TaxonomyEntry {
    /// Canonical category name surfaced by counters and reports.
    pub category: &'static str,
    /// SSB `action_type` sql_str emitted by the producing safeguard.
    pub ssb_action: &'static str,
    /// Short human-readable description used by reports.
    pub description: &'static str,
}

/// The 5 E-30 named taxonomy entries. Keep aligned with
/// `.claude/tools/failure_taxonomy.py` (each `category` field MUST appear as a
/// regex name there) and with `SsbActionType` (each `ssb_action` MUST be one
/// of the shipped sql_str values at ssb.rs).
///
/// Producer wiring per entry:
///   DestructiveVerbBlocked     -- crates/safety::check_argv() Deny path
///                                 -> hook_dispatcher.rs::dispatch_pre_tool_use
///   TrustDialogDeferredLoad    -- crates/safety::ensure_session_trust() refusal
///                                 -> boot path (cli/runtime_config)
///   McpServerPinMismatch       -- crates/plugins::mcp::pins enforcement refusal
///                                 -> mcp loader call site
///   AuditorLineageCollision    -- per_role_runtime::auditor_executor_lineage_collision == true
///                                 -> agent_pool::spawn_auditor
///   UntrustedMemoryWriteRefused -- conversation::TextSource::is_trusted_for_memory_write == false
///                                 -> mcp_memorize call site
pub const E30_FAILURE_TAXONOMY: &[E30TaxonomyEntry] = &[
    E30TaxonomyEntry {
        category: "DestructiveVerbBlocked",
        ssb_action: "argv_destructive_denied",
        description: "argv-regex allowlist denied a destructive verb (rm -rf, git restore ., DROP TABLE, ...).",
    },
    E30TaxonomyEntry {
        category: "TrustDialogDeferredLoad",
        ssb_action: "trust_dialog_hook_deferred",
        description: "trust-dialog ordering deferred a hook/env load until the user minted a session trust token (CVE-2025-59536 / CVE-2026-21852 fix pattern).",
    },
    E30TaxonomyEntry {
        category: "McpServerPinMismatch",
        ssb_action: "mcp_pin_version_refused",
        description: "MCP server version did not match the exact pin in mcp_pins.toml; semver float refused per postmark-mcp v1.0.16 backdoor precedent.",
    },
    E30TaxonomyEntry {
        category: "AuditorLineageCollision",
        ssb_action: "auditor_lineage_collision",
        description: "auditor and executor agents resolved to the same model lineage; correlated reward-hacking bias per arXiv:2511.18397.",
    },
    E30TaxonomyEntry {
        category: "UntrustedMemoryWriteRefused",
        ssb_action: "untrusted_memory_write_blocked",
        description: "mcp_memorize call originated from an untrusted TextSource (external tool / web fetch / OCR / clipboard); refused per SpAIware ChatGPT memory-injection precedent.",
    },
];

/// Lookup the taxonomy entry for a given category name. Returns `None` when
/// the category is unknown — caller should fall back to a generic
/// `unknown` bucket rather than panicking.
#[must_use]
pub fn e30_taxonomy_entry_for_category(category: &str) -> Option<&'static E30TaxonomyEntry> {
    E30_FAILURE_TAXONOMY.iter().find(|e| e.category == category)
}

/// Lookup the taxonomy entry for a given SSB action_type sql_str. Used by
/// the SSB row -> regression-report mapper to bin rows by named category.
#[must_use]
pub fn e30_taxonomy_entry_for_ssb_action(ssb_action: &str) -> Option<&'static E30TaxonomyEntry> {
    E30_FAILURE_TAXONOMY
        .iter()
        .find(|e| e.ssb_action == ssb_action)
}

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

    // -----------------------------------------------------------------
    // E-30: named taxonomy entries
    // -----------------------------------------------------------------

    #[test]
    fn e30_taxonomy_has_five_named_entries() {
        // Plan Task 7 promises exactly 5 named symptom categories.
        assert_eq!(E30_FAILURE_TAXONOMY.len(), 5);
    }

    #[test]
    fn e30_taxonomy_includes_all_plan_named_categories() {
        let expected = [
            "DestructiveVerbBlocked",
            "TrustDialogDeferredLoad",
            "McpServerPinMismatch",
            "AuditorLineageCollision",
            "UntrustedMemoryWriteRefused",
        ];
        for name in expected {
            assert!(
                e30_taxonomy_entry_for_category(name).is_some(),
                "E-30 taxonomy missing plan-named category: {name}"
            );
        }
    }

    #[test]
    fn e30_taxonomy_ssb_actions_match_shipped_sql_strs() {
        // These sql_str values are LOCKED by ssb.rs and the action_type_lock
        // golden. If any of them change, this test trips and forces an
        // explicit rename across regression.rs + failure_taxonomy.py.
        let expected_ssb_actions = [
            "argv_destructive_denied",
            "trust_dialog_hook_deferred",
            "mcp_pin_version_refused",
            "auditor_lineage_collision",
            "untrusted_memory_write_blocked",
        ];
        for ssb_action in expected_ssb_actions {
            assert!(
                e30_taxonomy_entry_for_ssb_action(ssb_action).is_some(),
                "E-30 taxonomy missing entry for SSB action: {ssb_action}"
            );
        }
    }

    #[test]
    fn e30_taxonomy_category_names_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for entry in E30_FAILURE_TAXONOMY {
            assert!(
                seen.insert(entry.category),
                "duplicate E-30 taxonomy category: {}",
                entry.category
            );
        }
    }
}
