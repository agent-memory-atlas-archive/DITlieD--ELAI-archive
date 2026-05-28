//! E-8 integration test: optimized-template admit lifecycle.
//!
//! Asserts the canonical Candidate->Promoted lifecycle entry-point per
//! workflow/reasoning/E-8-decisions.md DECISION-2 ("SHADOW/ACTIVE" is
//! plan-drift; lifecycle is {Candidate, Promoted, Retired, Frozen} at
//! reasoning_template.rs:258-265).

use elai_orchestrator::baby_step_spec_lpo::{BabyStepSpec, BlockKind, SpecBlock};
use elai_orchestrator::optimizer_admit::{
    admit_optimized_template, compute_optimized_content_hash, AdmitError,
    CrossDomainCanaryResult, OptimizedTemplate,
};
use elai_orchestrator::reasoning_template::ArchiveStatus;

fn ok_template() -> (OptimizedTemplate, BabyStepSpec) {
    let content = "improved body".to_string();
    let content_blake3 = compute_optimized_content_hash(&content);
    let parent_template_blake3 = compute_optimized_content_hash("parent");
    let spec = BabyStepSpec::from_blocks(vec![
        SpecBlock {
            kind: BlockKind::RolePromptBody,
            body: "orig".into(),
        },
        SpecBlock {
            kind: BlockKind::DeterministicCheck,
            body: "shell".into(),
        },
    ]);
    let t = OptimizedTemplate {
        content,
        content_blake3,
        parent_template_blake3,
        optimizer_version: "gepa-0.1.1+elai-e8".into(),
        optimization_run_id: "run-lifecycle".into(),
        val_lift_pct: 7.0,
        held_out_lift_pct: 6.0,
        cross_domain_canary_results: vec![],
        target_blocks: vec![BlockKind::RolePromptBody],
    };
    (t, spec)
}

#[test]
fn admit_first_lands_as_candidate() {
    let (t, spec) = ok_template();
    let r = admit_optimized_template(&t, &spec).expect("admit ok");
    assert_eq!(r.archive_status, ArchiveStatus::Candidate);
    assert!(r.admitted);
}

#[test]
fn candidate_can_transition_to_promoted() {
    // Canonical lifecycle invariant -- documented at
    // reasoning_template.rs:282-289.
    assert!(ArchiveStatus::Candidate.can_transition_to(ArchiveStatus::Promoted));
    assert!(ArchiveStatus::Candidate.can_transition_to(ArchiveStatus::Retired));
    assert!(!ArchiveStatus::Promoted.can_transition_to(ArchiveStatus::Candidate));
}

#[test]
fn admit_rejects_frozen_block_edit() {
    let (mut t, spec) = ok_template();
    t.target_blocks = vec![BlockKind::Acceptance];
    let r = admit_optimized_template(&t, &spec);
    assert!(matches!(r, Err(AdmitError::LpoViolation(_))));
}

#[test]
fn admit_rejects_cross_domain_regression() {
    let (mut t, spec) = ok_template();
    t.cross_domain_canary_results = vec![CrossDomainCanaryResult {
        family: "math".into(),
        delta_pct: -7.0,
    }];
    let r = admit_optimized_template(&t, &spec);
    assert!(matches!(r, Err(AdmitError::CrossDomainRegression { .. })));
}
