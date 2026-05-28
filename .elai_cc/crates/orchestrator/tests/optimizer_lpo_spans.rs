//! E-8 integration test: LPO `<edit>` / `<frozen>` span enforcement
//! per arXiv:2504.20355. Editable: TASK / GROUNDING / PRE_GATE / TDD /
//! ROLE_PROMPT_BODY. Frozen: DETERMINISTIC_CHECK / ACCEPTANCE /
//! INTEGRATION / EXIT_CRITERIA.

use elai_orchestrator::baby_step_spec_lpo::{
    assert_no_frozen_edit, editable_spans, frozen_spans, BabyStepSpec, BlockKind,
    LpoViolation, SpecBlock,
};

fn full_spec() -> BabyStepSpec {
    BabyStepSpec::from_blocks(
        BlockKind::all()
            .iter()
            .map(|k| SpecBlock {
                kind: *k,
                body: format!("body-{}", k.as_str()),
            })
            .collect(),
    )
}

#[test]
fn five_editable_blocks_identified() {
    let spec = full_spec();
    let e = editable_spans(&spec);
    assert_eq!(e.spans.len(), 5);
}

#[test]
fn four_frozen_blocks_identified() {
    let spec = full_spec();
    let f = frozen_spans(&spec);
    assert_eq!(f.spans.len(), 4);
}

#[test]
fn deterministic_check_is_frozen() {
    let spec = full_spec();
    let r = assert_no_frozen_edit(&spec, BlockKind::DeterministicCheck);
    assert!(matches!(
        r,
        Err(LpoViolation::FrozenBlockMutated { .. })
    ));
}

#[test]
fn acceptance_is_frozen() {
    let spec = full_spec();
    let r = assert_no_frozen_edit(&spec, BlockKind::Acceptance);
    assert!(matches!(
        r,
        Err(LpoViolation::FrozenBlockMutated { .. })
    ));
}

#[test]
fn integration_is_frozen() {
    let spec = full_spec();
    let r = assert_no_frozen_edit(&spec, BlockKind::Integration);
    assert!(matches!(
        r,
        Err(LpoViolation::FrozenBlockMutated { .. })
    ));
}

#[test]
fn exit_criteria_is_frozen() {
    let spec = full_spec();
    let r = assert_no_frozen_edit(&spec, BlockKind::ExitCriteria);
    assert!(matches!(
        r,
        Err(LpoViolation::FrozenBlockMutated { .. })
    ));
}

#[test]
fn role_prompt_body_is_editable_in_full_spec() {
    let spec = full_spec();
    let r = assert_no_frozen_edit(&spec, BlockKind::RolePromptBody);
    assert!(r.is_ok());
}

#[test]
fn task_block_editable() {
    let spec = full_spec();
    assert!(assert_no_frozen_edit(&spec, BlockKind::Task).is_ok());
}
