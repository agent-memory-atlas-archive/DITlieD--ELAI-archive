// A-47: Bare-message fallback classifier for routing unslashed user input.
// J-32: the T1 matcher logic now lives in elai_orchestrator::front_door (the
// single classifier of record). This module is a thin facade: it calls the
// shared matchers from front_door and maps onto SlashCommand. When
// ELAI_FRONT_DOOR_ENABLE is OFF the public behavior is bit-identical to the
// pre-J-32 classifier; when ON it routes via FrontDoorRouter::resolve.

use crate::commands::SlashCommand;
use elai_orchestrator::front_door::{
    self, FrontDoorDecision, FrontDoorLane, FrontDoorRouter,
};

// Re-export the shared matcher primitives so existing direct callers (and
// tests) keep compiling without a parallel heuristic.
pub use elai_orchestrator::front_door::{
    has_action_verb, has_code_intent, has_project_ref, has_question,
};

/// Classify a bare (non-slash-prefixed) message into a routed SlashCommand.
///
/// Route table (priority order, per pipeline-modes-design.txt §1):
/// 1. `HAS_ACTION_VERB` → `/task`
/// 2. `HAS_PROJECT_REF && HAS_QUESTION` → `/explore`
/// 3. `HAS_QUESTION && !HAS_CODE_INTENT && !HAS_PROJECT_REF` → `/research`
/// 4. default → `/task`
///
/// `known_names`: project-known identifiers from DKG. Pass `&[]` if DKG
/// unavailable.
///
/// When `ELAI_FRONT_DOOR_ENABLE` is set, the FrontDoorRouter drives the
/// decision and the lane is mapped onto SlashCommand. The route table is
/// preserved (Fix lane maps to Task for SlashCommand parity since
/// SlashCommand has no Fix variant).
pub fn classify_bare_message(input: &str, known_names: &[&str]) -> SlashCommand {
    if front_door::enabled() {
        let decision = FrontDoorRouter.resolve(input, known_names, &[]);
        return map_decision_to_slash_command(&decision, input);
    }

    // Kill-switch OFF: exact pre-J-32 behavior using the shared matchers.
    classify_bare_message_legacy(input, known_names)
}

/// Map a FrontDoorDecision onto SlashCommand. The router's lane enum is the
/// closed set; SlashCommand is the existing route target. Fix maps to Task
/// (SlashCommand has no Fix variant; the existing classifier routed "fix" to
/// Task, so this preserves parity).
fn map_decision_to_slash_command(decision: &FrontDoorDecision, input: &str) -> SlashCommand {
    match decision.lane {
        FrontDoorLane::Task | FrontDoorLane::Fix => SlashCommand::Task {
            description: Some(input.to_string()),
            surgical: false,
            new_project: false,
            via_mcp: false,
        },
        FrontDoorLane::Research => SlashCommand::Research {
            question: Some(input.to_string()),
        },
        FrontDoorLane::Explore => SlashCommand::Explore {
            target: Some(input.to_string()),
            via_mcp: false,
        },
        FrontDoorLane::Chat => SlashCommand::Task {
            description: Some(input.to_string()),
            surgical: false,
            new_project: false,
            via_mcp: false,
        },
    }
}

/// Pre-J-32 route table using the shared matchers (identical behavior to the
/// classifier.rs that shipped before J-32).
fn classify_bare_message_legacy(input: &str, known_names: &[&str]) -> SlashCommand {
    let action = has_action_verb(input);
    let question = has_question(input);
    let code_intent = has_code_intent(input);
    let project_ref = has_project_ref(input, known_names);

    if action {
        return SlashCommand::Task {
            description: Some(input.to_string()),
            surgical: false,
            new_project: false,
            via_mcp: false,
        };
    }

    if project_ref && question {
        return SlashCommand::Explore {
            target: Some(input.to_string()),
            via_mcp: false,
        };
    }

    if question && !code_intent && !project_ref {
        return SlashCommand::Research {
            question: Some(input.to_string()),
        };
    }

    SlashCommand::Task {
        description: Some(input.to_string()),
        surgical: false,
        new_project: false,
        via_mcp: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_verb_routes_to_task() {
        let result = classify_bare_message("implement the login page", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn question_without_code_intent_routes_to_research() {
        let result = classify_bare_message("how does the authentication work?", &[]);
        assert!(matches!(result, SlashCommand::Research { .. }));
    }

    #[test]
    fn question_prefix_without_mark_routes_to_research() {
        let result = classify_bare_message("what is the difference between X and Y", &[]);
        assert!(matches!(result, SlashCommand::Research { .. }));
    }

    #[test]
    fn question_with_code_intent_routes_to_task() {
        let result = classify_bare_message("change how does this work?", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn project_ref_plus_question_routes_to_explore() {
        let known = &["auth_module"];
        let result = classify_bare_message("how does auth_module handle tokens?", known);
        assert!(matches!(result, SlashCommand::Explore { .. }));
    }

    #[test]
    fn project_ref_without_question_routes_to_task() {
        let known = &["auth_module"];
        let result = classify_bare_message("auth_module needs cleanup", known);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn empty_dkg_degrades_gracefully() {
        let result = classify_bare_message("how does auth_module handle tokens?", &[]);
        assert!(matches!(result, SlashCommand::Research { .. }));
    }

    #[test]
    fn bare_statement_routes_to_task() {
        let result = classify_bare_message("the tests are broken", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn action_verb_takes_priority_over_question() {
        let result = classify_bare_message("fix the broken tests?", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn action_verb_with_trailing_punctuation() {
        let result = classify_bare_message("fix! how does this work?", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn action_verb_with_comma() {
        let result = classify_bare_message("implement, test, and deploy the feature", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }
}
