//! J-25 T6 completion-gate contract.
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    std::fs::read_to_string(crates_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn d1_module_and_lib_decl() {
    let gate = crates_dir().join("orchestrator/src/completion_gate.rs");
    assert!(gate.is_file(), "missing completion_gate.rs");
    let lib = read_src("orchestrator/src/lib.rs");
    assert!(lib.contains("pub mod completion_gate;"), "lib.rs missing pub mod");
}

#[test]
fn d2_production_caller_in_task_runner() {
    let src = read_src("orchestrator/src/task_runner.rs");
    assert!(
        src.contains("completion_gate::maybe_downgrade_passed")
            || src.contains("crate::completion_gate::maybe_downgrade_passed"),
        "missing production caller"
    );
    assert!(
        src.contains("global_task_registry()"),
        "caller must use global_task_registry"
    );
}

#[test]
fn d3_no_unwrap_expect_in_gate_prod() {
    let src = read_src("orchestrator/src/completion_gate.rs");
    let prod = src.split("#[cfg(test)]").next().unwrap_or(&src);
    assert!(!prod.contains(".unwrap("), "unwrap in prod");
    assert!(!prod.contains(".expect("), "expect in prod");
}

#[test]
fn d4_open_tasks_downgrade_to_partial() {
    use elai_orchestrator::completion_gate::maybe_downgrade_passed;
    use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
    use elai_orchestrator::task_registry::TaskRegistry;
    let reg = TaskRegistry::new();
    let task = reg.create("open", None);
    assert!(reg.set_owner_agent_id(&task.task_id, "agent-1"));
    let open = elai_orchestrator::completion_gate::non_terminal_task_ids_for_agent(&reg, "agent-1");
    assert_eq!(open, vec![task.task_id.clone()]);
    let out = maybe_downgrade_passed(BabyStepOutcome::Passed, open);
    match out {
        BabyStepOutcome::Partial { open_task_ids } => {
            assert_eq!(open_task_ids, vec![task.task_id]);
        }
        other => panic!("expected Partial, got {other:?}"),
    }
}

#[test]
fn d5_no_open_tasks_stays_passed() {
    use elai_orchestrator::completion_gate::maybe_downgrade_passed;
    use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
    let out = maybe_downgrade_passed(BabyStepOutcome::Passed, Vec::new());
    assert!(matches!(out, BabyStepOutcome::Passed));
}

#[test]
fn d6_no_llm_in_gate_source() {
    let src = read_src("orchestrator/src/completion_gate.rs");
    for needle in ["LlmProvider", "invoke_model", "chat_completion", "/v1/messages"] {
        assert!(!src.contains(needle), "ABORT-2 LLM marker {needle}");
    }
}

#[test]
fn d7_kill_switch_symbol_present() {
    let src = read_src("orchestrator/src/completion_gate.rs");
    assert!(
        src.contains("ELAI_J25_COMPLETION_GATE_DISABLE"),
        "kill-switch env name must exist"
    );
    assert!(
        src.contains("pub fn completion_gate_disabled"),
        "completion_gate_disabled must be public for ops"
    );
    // Behavioral disable path is env-dependent; do not mutate process env here (MR-2).
}
