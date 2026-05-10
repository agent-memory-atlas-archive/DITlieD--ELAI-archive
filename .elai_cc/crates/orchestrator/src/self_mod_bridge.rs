//! B-23 Phase 4H Group C — bridge from [`FullPipelineOutcome`] to the
//! Layer 1A [`DataTableProposal`] stream (4H.3B).
//!
//! Spec phase-4H.txt section 4H.3B paths:
//!   A. `stale_api_catches`        -> `DataTable::DeprecatedPatterns`
//!   B. `failure_taxonomy_misses`  -> `DataTable::FailureTaxonomy`
//!   C. lazy-code findings         -> stays in [`crate::self_mod_layer1a::DefaultLayer1aProcessor::process_outcome`]
//!      (the rich `FullPipelineOutcome` does NOT carry per-finding records;
//!      the lazy-code path consumes the batch-3C `PipelineOutcome.findings`).
//!
//! Group C also exposes a stub [`regression_pre_screen`] consumer that the
//! REG module (Group E, 4H.7C) will fill with real logic. Today it simply
//! returns `false` when `RegressionReport.has_regressions` is true so
//! callers can wire the gate now and ship the algorithm later.

use crate::self_mod_layer1a::{
    build_pattern_regex, estimate_fpr, DataTable, DataTableProposal, ProposalSource,
};
use crate::self_mod_types::{
    FullPipelineOutcome, ProjectScope, RegressionReport, StaleApiCatch, UnclassifiedError,
};
use std::collections::BTreeMap;

/// Bridge: convert a [`FullPipelineOutcome`] into the Layer 1A proposal
/// stream. Each output proposal is **scoped** to the catch's ecosystem (or
/// the failure's language hint) so the apply path enforces cross-ecosystem
/// isolation.
///
/// `min_occurrences` mirrors `DefaultLayer1aProcessor::min_occurrences`
/// (default 3) so the bridge respects the same threshold the spec uses
/// for Phase-3F self-discovery (rules.txt section 5A).
pub fn proposals_from_full_outcome(
    outcome: &FullPipelineOutcome,
    min_occurrences: u32,
) -> Vec<DataTableProposal> {
    let mut out = Vec::new();

    // Path A — deprecated_patterns from stale_api_catches. Group key is
    // (deprecated_name, ecosystem) per Group C audit BLOCK fix: pooling
    // across ecosystems produces incorrect threshold totals and a scope
    // tied to whichever ecosystem happened to land first in the bucket.
    let mut by_name_eco: BTreeMap<(&str, &str), Vec<&StaleApiCatch>> = BTreeMap::new();
    for catch in &outcome.stale_api_catches {
        by_name_eco
            .entry((catch.deprecated_name.as_str(), catch.ecosystem.as_str()))
            .or_default()
            .push(catch);
    }
    for ((deprecated_name, ecosystem), catches) in by_name_eco {
        let count = catches.len();
        if count < min_occurrences as usize {
            continue;
        }
        let canonical = catches[0];
        let new_row = serde_json::json!({
            "deprecated_name": canonical.deprecated_name,
            "replacement_name": canonical.replacement_name,
            "ecosystem": canonical.ecosystem,
            "first_seen_in": canonical.file_path.to_string_lossy(),
            // Group C audit WARN fix: spec phase-4H.txt:455 specifies
            // severity="BLOCK" for deprecated_patterns rows. Adding it
            // here keeps the row schema-complete on first apply.
            "severity": "BLOCK",
        });
        let scope = ProjectScope {
            ecosystem: Some(ecosystem.to_string()),
            language: ecosystem_to_language(ecosystem),
            pipeline_mode: Some(outcome.pipeline_mode),
        };
        out.push(DataTableProposal {
            proposal_id: format!("dep-{}-{}", ecosystem, sanitize_id(deprecated_name)),
            table: DataTable::DeprecatedPatterns,
            new_row,
            source: ProposalSource::AutoFromOutcome,
            fpr_estimate: 0.0, // name-equality, not regex — corpus FPR is degenerate.
            occurrences_seen: u32::try_from(count).unwrap_or(u32::MAX),
            pipeline_mode: outcome.pipeline_mode,
            scope: Some(scope),
        });
    }

    // Path B — failure_taxonomy from unclassified errors.
    let mut by_text: BTreeMap<&str, Vec<&UnclassifiedError>> = BTreeMap::new();
    for err in &outcome.failure_taxonomy_misses {
        by_text.entry(err.error_text.as_str()).or_default().push(err);
    }
    for (error_text, errors) in by_text {
        let total: usize = errors
            .iter()
            .map(|e| e.occurrence_count)
            .sum();
        let total_u32 = u32::try_from(total).unwrap_or(u32::MAX);
        if total_u32 < min_occurrences {
            continue;
        }
        let canonical = errors[0];
        let new_row = serde_json::json!({
            "error_text": canonical.error_text,
            "first_seen_in": canonical.file_path.to_string_lossy(),
            "baby_step_id": canonical.baby_step_id,
            "occurrences": total,
        });
        let fpr = build_pattern_regex(error_text)
            .map(|rx| estimate_fpr(&rx))
            .unwrap_or(0.0);
        out.push(DataTableProposal {
            proposal_id: format!("ftax-{}", sanitize_id(error_text)),
            table: DataTable::FailureTaxonomy,
            new_row,
            source: ProposalSource::AutoFromOutcome,
            fpr_estimate: fpr,
            occurrences_seen: total_u32,
            pipeline_mode: outcome.pipeline_mode,
            // Group C audit BLOCK fix: spec phase-4H.txt:499 requires every
            // auto-proposal to carry a non-None scope. FailureTaxonomy is
            // language-agnostic, but we can still scope to pipeline_mode
            // so the proposal_id + apply path can attribute it to the
            // generating mode without restricting which ecosystem applies.
            scope: Some(ProjectScope {
                ecosystem: None,
                language: None,
                pipeline_mode: Some(outcome.pipeline_mode),
            }),
        });
    }

    out
}

/// Stub for the REG (4H.7C) gate — Group E will fill in the real logic.
/// Returns `true` when the proposal may proceed, `false` when a regression
/// blocks it. Today: `false` iff `report.has_regressions` is set.
pub fn regression_pre_screen(report: &RegressionReport) -> bool {
    !report.has_regressions
}

fn ecosystem_to_language(eco: &str) -> Option<String> {
    match eco {
        "cargo" => Some("Rust".into()),
        "pip" => Some("Python".into()),
        "npm" => Some("TypeScript".into()),
        "go" => Some("Go".into()),
        "maven" => Some("Java".into()),
        _ => None,
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(48)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_mod_layer1a::PipelineMode;
    use crate::self_mod_types::{
        AcRegression, CatchLayer, RegressionReport, StaleApiCatch, TestOutcomeReport,
        UnclassifiedError,
    };
    use std::path::PathBuf;

    fn make_outcome(mode: PipelineMode) -> FullPipelineOutcome {
        FullPipelineOutcome {
            template_id: "t".into(),
            project_id: "p".into(),
            pipeline_mode: mode,
            phases_total: 0,
            phases_passed_first_try: 0,
            audit_findings_total: 0,
            audit_findings_block: 0,
            test_coverage_final: 0.0,
            time_to_complete_seconds: 0,
            tokens_consumed_total: 0,
            human_interventions: 0,
            rules_triggered: vec![],
            test_outcomes: TestOutcomeReport::default(),
            etc_trigger_events: 0,
            stale_api_catches: vec![],
            context_budget_truncations: vec![],
            failure_taxonomy_misses: vec![],
        }
    }

    #[test]
    fn deprecated_patterns_emits_when_threshold_met() {
        let mut outcome = make_outcome(PipelineMode::Surgical);
        for _ in 0..3 {
            outcome.stale_api_catches.push(StaleApiCatch {
                file_path: PathBuf::from("src/lib.rs"),
                deprecated_name: "blake2".into(),
                replacement_name: "blake3".into(),
                ecosystem: "cargo".into(),
                caught_by: CatchLayer::Auditor,
            });
        }
        let proposals = proposals_from_full_outcome(&outcome, 3);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].table, DataTable::DeprecatedPatterns);
        let scope = proposals[0].scope.as_ref().expect("scope");
        assert_eq!(scope.ecosystem.as_deref(), Some("cargo"));
        assert_eq!(scope.language.as_deref(), Some("Rust"));
    }

    #[test]
    fn deprecated_patterns_below_threshold_no_emit() {
        let mut outcome = make_outcome(PipelineMode::Surgical);
        outcome.stale_api_catches.push(StaleApiCatch {
            file_path: PathBuf::from("src/lib.rs"),
            deprecated_name: "blake2".into(),
            replacement_name: "blake3".into(),
            ecosystem: "cargo".into(),
            caught_by: CatchLayer::Auditor,
        });
        let proposals = proposals_from_full_outcome(&outcome, 3);
        assert!(proposals.is_empty());
    }

    #[test]
    fn failure_taxonomy_threshold_summed_across_records() {
        let mut outcome = make_outcome(PipelineMode::Greenfield);
        outcome.failure_taxonomy_misses.push(UnclassifiedError {
            error_text: "MISSING_FEATURE_FLAG_X".into(),
            file_path: PathBuf::from("src/foo.rs"),
            baby_step_id: "bs1".into(),
            occurrence_count: 2,
        });
        outcome.failure_taxonomy_misses.push(UnclassifiedError {
            error_text: "MISSING_FEATURE_FLAG_X".into(),
            file_path: PathBuf::from("src/bar.rs"),
            baby_step_id: "bs2".into(),
            occurrence_count: 2,
        });
        let proposals = proposals_from_full_outcome(&outcome, 3);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].table, DataTable::FailureTaxonomy);
        // FailureTaxonomy carries a mode-only scope (audit BLOCK fix:
        // spec phase-4H.txt:499 requires scope != None on every auto-
        // proposal). Ecosystem/language stay None to preserve language-
        // agnosticism.
        let scope = proposals[0].scope.as_ref().expect("mode-only scope expected");
        assert!(scope.ecosystem.is_none());
        assert!(scope.language.is_none());
        assert_eq!(scope.pipeline_mode, Some(PipelineMode::Greenfield));
        assert_eq!(proposals[0].pipeline_mode, PipelineMode::Greenfield);
    }

    #[test]
    fn regression_pre_screen_blocks_on_regression() {
        let mut report = RegressionReport::default();
        assert!(regression_pre_screen(&report), "clean report passes");
        report = RegressionReport::from_regressions(vec![AcRegression {
            ac_index: 0,
            ac_text: "ac".into(),
            passed_in_pipeline: "p1".into(),
            failed_in_pipeline: "p5".into(),
            consecutive_failures: 1,
        }]);
        assert!(!regression_pre_screen(&report), "regression should block");
    }

    #[test]
    fn ecosystem_to_language_known_pairs() {
        assert_eq!(ecosystem_to_language("cargo"), Some("Rust".into()));
        assert_eq!(ecosystem_to_language("pip"), Some("Python".into()));
        assert_eq!(ecosystem_to_language("npm"), Some("TypeScript".into()));
        assert_eq!(ecosystem_to_language("go"), Some("Go".into()));
        assert_eq!(ecosystem_to_language("unknown"), None);
    }
}
