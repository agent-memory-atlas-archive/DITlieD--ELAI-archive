//! J-89 L1: typed, taint-tracked objective specification bundles.
//!
//! This module is deliberately observe-only.  It provides deterministic coverage
//! and an advisory (non-verdict) jury for the planning path.

/// Kill-switch env (default OFF).
pub const ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV: &str = "ELAI_SPEC_BUNDLE_GATE_ENABLE";

pub fn spec_bundle_gate_enabled() -> bool {
    std::env::var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV)
        .map(|v| !v.trim().is_empty() && v != "0")
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpecBundle {
    pub purpose: String,
    pub scope: String,
    pub domain_assumptions: Vec<String>,
    pub non_functional_constraints: Vec<String>,
    pub competency_questions: Vec<String>,
    pub obstacle_log: Vec<String>,
    pub bundle_digest: String,
    pub inferred_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageReport {
    pub ok: bool,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JuryReport {
    pub challenges_survived: u32,
    pub challenges_raised: Vec<String>,
    pub escalated: bool,
    pub families_used: Vec<String>,
    pub refused_reason: Option<String>,
}

fn nonempty(value: Option<&str>) -> String {
    value.unwrap_or_default().trim().to_owned()
}

/// Build a typed bundle from free-text objective and optional structured fields.
pub fn build_spec_bundle(
    objective: &str,
    purpose: Option<&str>,
    scope: Option<&str>,
    domain_assumptions: &[String],
    non_functional_constraints: &[String],
    competency_questions: &[String],
    obstacle_log: &[String],
) -> SpecBundle {
    let objective = objective.trim();
    let mut inferred_fields = Vec::new();
    let purpose = match purpose.map(str::trim).filter(|v| !v.is_empty()) {
        Some(value) => value.to_owned(),
        None => {
            inferred_fields.push("purpose".to_owned());
            objective.to_owned()
        }
    };
    let scope = match scope.map(str::trim).filter(|v| !v.is_empty()) {
        Some(value) => value.to_owned(),
        None => {
            inferred_fields.push("scope".to_owned());
            String::new()
        }
    };
    let mut questions: Vec<String> = competency_questions
        .iter()
        .map(|q| q.trim().to_owned())
        .filter(|q| !q.is_empty())
        .collect();
    for line in objective.lines() {
        if let Some(question) = line.trim().strip_prefix("CQ:") {
            let question = question.trim();
            if !question.is_empty() && !questions.iter().any(|q| q == question) {
                questions.push(question.to_owned());
                inferred_fields.push("competency_questions".to_owned());
            }
        }
    }
    let mut bundle = SpecBundle {
        purpose,
        scope,
        domain_assumptions: domain_assumptions.to_vec(),
        non_functional_constraints: non_functional_constraints.to_vec(),
        competency_questions: questions,
        obstacle_log: obstacle_log.to_vec(),
        bundle_digest: String::new(),
        inferred_fields,
    };
    let canonical = serde_json::to_vec(&bundle).unwrap_or_default();
    bundle.bundle_digest = blake3::hash(&canonical).to_hex().to_string();
    bundle
}

/// Check purpose, competency-question, and assumption coverage deterministically.
pub fn check_bundle_coverage(bundle: &SpecBundle) -> CoverageReport {
    let mut missing = Vec::new();
    if bundle.purpose.trim().is_empty() {
        missing.push("purpose_empty".to_owned());
    }
    if bundle.competency_questions.is_empty() {
        missing.push("competency_questions_empty".to_owned());
    }
    let searchable = [
        bundle.purpose.as_str(),
        bundle.scope.as_str(),
        &bundle.competency_questions.join(" "),
        &bundle.obstacle_log.join(" "),
    ]
    .join(" ")
    .to_lowercase();
    for assumption in &bundle.domain_assumptions {
        let snippet = assumption.trim();
        if !snippet.is_empty() && !searchable.contains(&snippet.to_lowercase()) {
            missing.push(format!("assumption_unreferenced:{snippet}"));
        }
    }
    CoverageReport { ok: missing.is_empty(), missing }
}

/// Run an advisory cross-family jury; this never asserts objective correctness.
pub fn run_spec_jury(bundle: &SpecBundle, coverage: &CoverageReport, families: &[&str]) -> JuryReport {
    let families_used: Vec<String> = families.iter().map(|family| (*family).to_owned()).collect();
    if families.len() < 2 {
        return JuryReport {
            challenges_survived: 0,
            challenges_raised: Vec::new(),
            escalated: false,
            families_used,
            refused_reason: Some("single-family panel refused".to_owned()),
        };
    }
    let mut challenges = coverage.missing.clone();
    for assumption in &bundle.domain_assumptions {
        let lower = assumption.to_lowercase();
        if !["must", "shall", "require", "constraint", "only", "cannot", "should"]
            .iter().any(|word| lower.contains(word))
        {
            challenges.push(format!("assumption_constraint_missing:{assumption}"));
        }
    }
    JuryReport {
        challenges_survived: if coverage.ok { challenges.len() as u32 } else { 0 },
        challenges_raised: challenges,
        escalated: !coverage.ok,
        families_used,
        refused_reason: None,
    }
}

/// Evaluate an objective only when the kill-switch is enabled; never blocks callers.
pub fn evaluate_objective_observe_only(objective: &str) -> Option<(SpecBundle, CoverageReport, JuryReport)> {
    if !spec_bundle_gate_enabled() {
        return None;
    }
    let bundle = build_spec_bundle(objective, None, None, &[], &[], &[], &[]);
    let coverage = check_bundle_coverage(&bundle);
    let jury = run_spec_jury(&bundle, &coverage, &["glm", "swe"]);
    if coverage.ok {
        tracing::info!(digest = %bundle.bundle_digest, ok = coverage.ok, escalated = jury.escalated,
            "J-89 L1: spec bundle evaluated");
    } else {
        tracing::warn!(digest = %bundle.bundle_digest, ok = coverage.ok, escalated = jury.escalated,
            "J-89 L1: spec bundle coverage incomplete");
    }
    Some((bundle, coverage, jury))
}

#[cfg(test)]
mod j89_spec_bundle_tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn coverage_reports_missing_questions()  {
        let bundle = build_spec_bundle("purpose", Some("purpose"), None, &[], &[], &[], &[]);
        let report = check_bundle_coverage(&bundle);
        assert!(!report.ok);
        assert!(report.missing.iter().any(|v| v == "competency_questions_empty"));
    }

    #[test]
    fn coverage_reports_unreferenced_assumptions()  {
        let assumption = vec!["encoding".to_owned()];
        let question = vec!["What is the result?".to_owned()];
        let report = check_bundle_coverage(&build_spec_bundle("purpose", Some("purpose"), None, &assumption, &[], &question, &[]));
        assert!(report.missing.iter().any(|v| v == "assumption_unreferenced:encoding"));
    }

    #[test]
    fn full_bundle_passes_coverage() {
        let assumption = vec!["encoding".to_owned()];
        let question = vec!["How is encoding validated?".to_owned()];
        let bundle = build_spec_bundle("encoding purpose", Some("encoding purpose"), None, &assumption, &[], &question, &[]);
        assert!(check_bundle_coverage(&bundle).ok);
    }

    #[test]
    fn jury_refuses_single_family() {
        let bundle = build_spec_bundle("p", Some("p"), None, &[], &[], &["q".to_owned()], &[]);
        let report = check_bundle_coverage(&bundle);
        assert!(run_spec_jury(&bundle, &report, &["only"]).refused_reason.is_some());
    }

    #[test]
    fn jury_two_family_raises_on_bad_coverage() {
        let bundle = build_spec_bundle("p", Some("p"), None, &[], &[], &[], &[]);
        let coverage = check_bundle_coverage(&bundle);
        let jury = run_spec_jury(&bundle, &coverage, &["a", "b"]);
        assert!(jury.escalated);
        assert!(!jury.challenges_raised.is_empty());
    }

    #[test]
    fn flag_default_off() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prior = std::env::var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV).ok();
        std::env::remove_var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV);
        assert!(!spec_bundle_gate_enabled());
        assert!(evaluate_objective_observe_only("objective").is_none());
        if let Some(value) = prior { std::env::set_var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV, value); }
    }

    #[test]
    fn evaluate_on_when_flag_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prior = std::env::var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV).ok();
        std::env::set_var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV, "1");
        assert!(evaluate_objective_observe_only("objective").is_some());
        match prior { Some(value) => std::env::set_var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV, value), None => std::env::remove_var(ELAI_SPEC_BUNDLE_GATE_ENABLE_ENV) }
    }
}
