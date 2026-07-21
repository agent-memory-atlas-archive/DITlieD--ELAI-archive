//! J-32 slice-I — GD-1 anti-duplication contract for the FrontDoorRouter.
//!
//! Grounded scope (conductor/opus_walk/evidence/J-32/S5-sliceI-context.md):
//! the orchestrator's `front_door.rs` is the single T1 matcher of record
//! (ACTION_VERBS + has_action_verb + the other has_* primitives live there
//! exactly once). The CLI `classifier.rs` is a thin facade: it composes
//! `FrontDoorRouter::resolve` + `map_decision_to_slash_command` onto
//! `SlashCommand` and re-exports the shared matchers, but it MUST NOT
//! redefine the heuristic body. This contract enforces that boundary via
//! source-grep assertions plus runtime route-table assertions driven through
//! the real `elai_orchestrator::front_door::FrontDoorRouter` entrypoint.
//!
//! Assertions 1-5 mirror the digest's REQUIRED ASSERTIONS verbatim. No
//! #[ignore], no stub asserts, no edits to front_door.rs / classifier.rs /
//! dashboard / Cargo.toml.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::front_door::{
    FrontDoorLane, FrontDoorRouter, FrontDoorTier,
};

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// A1: the orchestrator's front_door.rs is the single matcher of record —
/// exactly one `fn has_action_verb` definition AND the ACTION_VERBS const
/// live there. The CLI facade MUST NOT redefine the heuristic body
/// (`fn has_action_verb` count in classifier.rs == 0; it may only re-export).
/// GD-1 single-implementation: no parallel heuristic.
#[test]
fn front_door_is_single_matcher_of_record() {
    let front = read_src("orchestrator/src/front_door.rs");
    let cli = read_src("cli/src/classifier.rs");

    let front_defs = front.matches("fn has_action_verb").count();
    assert_eq!(
        front_defs, 1,
        "front_door.rs must define has_action_verb exactly once (GD-1 single matcher), found {front_defs}"
    );
    assert!(
        front.contains("ACTION_VERBS"),
        "front_door.rs must own the ACTION_VERBS const (single matcher of record)"
    );

    let cli_defs = cli.matches("fn has_action_verb").count();
    assert_eq!(
        cli_defs, 0,
        "classifier.rs must NOT redefine has_action_verb (facade re-export only); found {cli_defs} definitions"
    );
    assert!(
        !cli.contains("const ACTION_VERBS"),
        "classifier.rs must NOT redefine ACTION_VERBS (facade re-export only)"
    );
}

/// A2 + A3: classifier.rs composes SlashCommand from the router decision —
/// it must reference `FrontDoorRouter.resolve` AND `map_decision_to_slash_command`
/// AND the `SlashCommand::Task` / `SlashCommand::Research` / `SlashCommand::Explore`
/// variants. The orchestrator's front_door.rs MUST NOT contain `enum SlashCommand`
/// (the lane enum is the closed set; SlashCommand lives in the CLI layer).
#[test]
fn classifier_composes_slash_command_from_router() {
    let front = read_src("orchestrator/src/front_door.rs");
    let cli = read_src("cli/src/classifier.rs");

    assert!(
        cli.contains("FrontDoorRouter.resolve")
            || cli.contains("FrontDoorRouter.resolve("),
        "classifier.rs must call FrontDoorRouter.resolve (compose, not re-implement)"
    );
    assert!(
        cli.contains("map_decision_to_slash_command"),
        "classifier.rs must define/call map_decision_to_slash_command (the SlashCommand composition seam)"
    );
    assert!(
        cli.contains("SlashCommand::Task"),
        "classifier.rs must map onto SlashCommand::Task (Task|Fix|Chat lanes)"
    );
    assert!(
        cli.contains("SlashCommand::Research"),
        "classifier.rs must map onto SlashCommand::Research"
    );
    assert!(
        cli.contains("SlashCommand::Explore"),
        "classifier.rs must map onto SlashCommand::Explore"
    );

    assert!(
        !front.contains("enum SlashCommand"),
        "front_door.rs must NOT own SlashCommand (CLI-layer concern; lane enum is the closed set)"
    );
}

/// A3 runtime: arg_spans are byte-offsets into the ORIGINAL input — the span
/// fragment equals the trimmed message, mirroring the front_door unit test.
#[test]
fn runtime_arg_spans_are_byte_offsets_into_original() {
    let input = "  implement the login page  ";
    let decision = FrontDoorRouter.resolve(input, &[], &[]);
    assert!(!decision.arg_spans.is_empty(), "router must produce at least one arg_span");
    let span = decision.arg_spans[0];
    let fragment = &input[span.start..span.end];
    assert_eq!(
        fragment, "implement the login page",
        "arg_spans must be byte offsets into the original input (trimmed fragment)"
    );
}

/// A4 runtime: the flag-independent route table. FrontDoorRouter always
/// resolves to the same lane for these inputs regardless of the
/// ELAI_FRONT_DOOR_ENABLE kill-switch (the facade is the flag gate, not the
/// router). Bare default is Task (parity with the shipped pre-J-32
/// classifier), NOT Chat.
#[test]
fn runtime_route_table_flag_independent() {
    let router = FrontDoorRouter;

    let task = router.resolve("implement the login page", &[], &[]);
    assert_eq!(task.lane, FrontDoorLane::Task, "action-verb input must route to Task");

    let fix = router.resolve("fix the broken tests", &[], &[]);
    assert_eq!(fix.lane, FrontDoorLane::Fix, "fix-verb input must route to Fix");

    let research = router.resolve("how does the authentication work?", &[], &[]);
    assert_eq!(
        research.lane, FrontDoorLane::Research,
        "question without code-intent/project-ref must route to Research"
    );

    let known = &["auth_module"];
    let explore = router.resolve("how does auth_module handle tokens?", known, &[]);
    assert_eq!(
        explore.lane, FrontDoorLane::Explore,
        "project-ref + question must route to Explore"
    );

    let bare = router.resolve("the tests are broken", &[], &[]);
    assert_eq!(
        bare.lane, FrontDoorLane::Task,
        "bare default must route to Task (shipped parity, NOT Chat)"
    );
}

/// A5 runtime: T0 slash bypass — when the input leads with a registered slash
/// name, the router returns tier == T0 and a slash_bypass payload whose
/// `name` matches the registered slash command. The caller (facade) delegates
/// to SlashCommand::parse unchanged.
#[test]
fn runtime_t0_slash_bypass_when_name_registered() {
    let decision = FrontDoorRouter.resolve("/fix the bug", &[], &["fix", "task"]);
    assert_eq!(decision.tier, FrontDoorTier::T0, "registered slash input must bypass to T0");
    let bypass = decision
        .slash_bypass
        .as_ref()
        .expect("T0 bypass must carry a SlashBypass payload");
    assert_eq!(bypass.name, "fix", "slash_bypass.name must match the registered slash command");
}
