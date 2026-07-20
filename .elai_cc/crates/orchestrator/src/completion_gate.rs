//! J-25 T6 deterministic completion gate (no LLM).
use crate::pipeline_tree_navigator::BabyStepOutcome;
use crate::task_registry::TaskRegistry;

const KILL_SWITCH: &str = "ELAI_J25_COMPLETION_GATE_DISABLE";

#[must_use]
pub fn completion_gate_disabled() -> bool {
    matches!(
        std::env::var(KILL_SWITCH).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

/// Open non-terminal task ids for the returning agent. Empty when kill-switched.
#[must_use]
pub fn non_terminal_task_ids_for_agent(
    registry: &TaskRegistry,
    owner_agent_id: &str,
) -> Vec<String> {
    if completion_gate_disabled() {
        return Vec::new();
    }
    registry.non_terminal_task_ids_for_agent(owner_agent_id)
}

/// Downgrade Passed -> Partial when open ids remain.
#[must_use]
pub fn maybe_downgrade_passed(
    outcome: BabyStepOutcome,
    open_task_ids: Vec<String>,
) -> BabyStepOutcome {
    if completion_gate_disabled() {
        return outcome;
    }
    match outcome {
        BabyStepOutcome::Passed if !open_task_ids.is_empty() => {
            BabyStepOutcome::Partial { open_task_ids }
        }
        other => other,
    }
}
