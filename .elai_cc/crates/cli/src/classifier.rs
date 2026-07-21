// A-47: Bare-message fallback classifier for routing unslashed user input.
// J-32: the T1 matcher logic now lives in elai_orchestrator::front_door (the
// single classifier of record). This module is a thin facade: it calls the
// shared matchers from front_door and maps onto SlashCommand. When
// ELAI_FRONT_DOOR_ENABLE is OFF the public behavior is bit-identical to the
// pre-J-32 classifier; when ON it routes via FrontDoorRouter::resolve.

use std::sync::Arc;

use crate::commands::SlashCommand;
use elai_orchestrator::front_door::{
    self, FrontDoorDecision, FrontDoorLane, FrontDoorRouter,
};
use elai_orchestrator::front_door_emit::{emit_intent_routed, risk_for_lane};
use elai_orchestrator::ssb::Ssb;

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

/// Classify a bare message and, when the front door is enabled, emit the
/// IntentRouted SSB row for the resolved decision. This is the slice-D
/// production-wired variant of `classify_bare_message`: the repl bare-message
/// path calls this with `cli.ssb.as_ref()` so the routing observability row
/// lands on the same code path that drives SlashCommand routing.
///
/// When `ELAI_FRONT_DOOR_ENABLE` is OFF the behavior is bit-identical to
/// `classify_bare_message` (no SSB row; legacy route table). When ON and
/// `ssb` is `None`, the decision is still resolved and mapped; only the
/// emit is skipped (a missing SSB handle is not a correctness failure).
pub fn classify_bare_message_with_ssb(
    input: &str,
    known_names: &[&str],
    ssb: Option<&Arc<Ssb>>,
) -> SlashCommand {
    if front_door::enabled() {
        let decision = FrontDoorRouter.resolve(input, known_names, &[]);
        if let Some(ssb) = ssb {
            emit_intent_routed(ssb, &decision, risk_for_lane(decision.lane));
        }
        return map_decision_to_slash_command(&decision, input);
    }
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

    // J-32 slice-D: the repl bare-message path calls
    // classify_bare_message_with_ssb, which must emit an IntentRouted
    // Classified SSB row when ELAI_FRONT_DOOR_ENABLE is set and an Ssb
    // handle is supplied. This is the production-wire proof for
    // emit_intent_routed (R7: a real caller, not a test-only caller).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn j32_slice_d_with_ssb_emits_intent_routed_row() {
        std::env::set_var("ELAI_FRONT_DOOR_ENABLE", "1");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
        let ssb = Arc::new(
            elai_orchestrator::ssb::Ssb::new(
                elai_orchestrator::ssb::SsbConfig {
                    db_path: dir.path().join("j32-slice-d-ssb.db"),
                    pool_size: Some(2),
                    project_id: "j32-slice-d".into(),
                    session_id: "j32-slice-d".into(),
                },
                clock,
            )
            .expect("ssb open"),
        );

        let _ = classify_bare_message_with_ssb(
            "fix the failing test in sum.rs",
            &[],
            Some(&ssb),
        );

        tokio::time::sleep(std::time::Duration::from_millis(120)).await;

        use elai_orchestrator::ssb::SsbReader;
        let rows = ssb
            .query_by_type(elai_orchestrator::ssb::SsbActionType::IntentRouted)
            .expect("query_by_type IntentRouted");
        assert!(
            !rows.is_empty(),
            "expected >= 1 IntentRouted row from classify_bare_message_with_ssb; \
             R1 missing evidence = FAIL"
        );
        let decoded = elai_orchestrator::ssb::decode_ssb_payload(&rows[0].payload)
            .expect("decode_ssb_payload");
        match decoded {
            elai_orchestrator::ssb::SsbPayload::IntentRouted(p) => {
                assert_eq!(
                    p.kind,
                    elai_orchestrator::ssb::IntentRoutedKind::Classified
                );
                assert_eq!(p.lane, elai_orchestrator::front_door::FrontDoorLane::Fix);
                assert!(p.ts_ms > 0);
            }
            other => panic!("expected IntentRouted; got {:?}", other),
        }

        std::env::remove_var("ELAI_FRONT_DOOR_ENABLE");
    }
}
