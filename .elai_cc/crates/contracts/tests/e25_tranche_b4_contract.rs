//! E-25 Tranche B-4 — concrete-impl regression contracts.
//!
//! Four negative-space guards + three structural shape guards. Pairs with the
//! existing `e25_compaction_architecture_contracts.rs` substrate guards by
//! enforcing that the tranche-B4 concrete impls (compaction_ladder,
//! skill_templating, opusplan overlay, session_fork emit) do not grow
//! parallel modules / duplicated overlays. Uses the shared walker from
//! `tests/common/mod.rs` (E-12.A factoring).

mod common;

use common::{collect_hits, format_hits, workspace_rs_files};

use elai_orchestrator::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload, CompactionTierId, ALL_COMPACTION_TIERS,
};
use elai_orchestrator::compaction_ladder::{
    compaction_ladder_enabled, CompactionLadder, LadderRunResult, NullEmitter,
    run_compaction_ladder, ELAI_E25_COMPACTION_LADDER_ENABLE_VAR,
};
use elai_orchestrator::skill_templating::{
    expand_skill_template, skill_templating_enabled, SkillTemplate,
    ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR,
};
use elai_orchestrator::fsm::{
    opus_plan_overlay_enabled, ELAI_E25_OPUSPLAN_OVERLAY_ENABLE_VAR,
};

// ---------------------------------------------------------------------------
// Negative-space guards
// ---------------------------------------------------------------------------

#[test]
fn no_parallel_compaction_ladder_module() {
    // E-25 D1: only ONE compaction_ladder.rs file in the workspace. A parallel
    // _v2 module fragments the ladder traversal + SSB emit surface.
    let banned = [
        "struct CompactionLadderV2",
        "fn run_compaction_ladder_v2",
        "trait LadderTierV2",
        "struct LadderRunResultV2",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel compaction-ladder modules are banned — extend CompactionLadder.\nfound:\n  {}",
        format_hits(&hits)
    );

    // Also assert there is exactly one compaction_ladder.rs in the orchestrator
    // crate (not counting tests/contracts).
    let mut prod_files = 0;
    for path in workspace_rs_files() {
        let s = path.to_string_lossy();
        if s.ends_with("compaction_ladder.rs")
            && s.contains("orchestrator")
            && s.contains("src")
        {
            prod_files += 1;
        }
    }
    assert_eq!(
        prod_files, 1,
        "expected exactly one orchestrator/src/compaction_ladder.rs, found {prod_files}"
    );
}

#[test]
fn no_parallel_skill_template_loader() {
    // E-25 D2: only ONE skill_templating module. A parallel loader bypasses
    // the allowlist + emit cadence.
    let banned = [
        "fn execute_skill_template_v2",
        "struct ShellInjectionV2",
        "trait SkillTemplateLoaderV2",
        "fn expand_skill_template_v2",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel skill-template loaders are banned — extend skill_templating.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_parallel_opusplan_overlay() {
    // E-25 D3: opusplan overlay is a FsmInner field accessed via
    // set_opus_plan_active / opus_plan_active. A parallel SubstateOverlayV2
    // would re-introduce the FsmSubstate variant minted by stale-plan callers.
    let banned = [
        "fn route_opusplan_v2",
        "struct SubstateOverlayV2",
        "struct OpusplanSubstateOverlay",
        "fn opus_plan_route_v2",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel opusplan overlays are banned — extend Fsm::set_opus_plan_active.\nfound:\n  {}",
        format_hits(&hits)
    );
}

// ---------------------------------------------------------------------------
// Structural / shape guards
// ---------------------------------------------------------------------------

#[test]
fn compaction_ladder_tier_round_trip_msgpack() {
    // Round-trip every CompactionTierId through msgpack to assert the
    // serde derives on the LOCKED CompactionTierId enum stay green. Mirrors
    // architecture/claude_code_arch.rs:585 (taxonomy_count_is_locked).
    for &tier in ALL_COMPACTION_TIERS {
        let bytes = rmp_serde::to_vec_named(&tier).expect("serialize tier");
        let round_tripped: CompactionTierId =
            rmp_serde::from_slice(&bytes).expect("deserialize tier");
        assert_eq!(round_tripped, tier);
    }
}

#[test]
fn session_fork_event_details_json_shape() {
    // E-25 D4: SessionForked emit details_json carries fork_event +
    // parent_session + child_session + branch_point. Assert the shape via a
    // constructed payload + msgpack round-trip.
    let payload = ClaudeCodeArchitectureEventPayload {
        kind: ClaudeCodeArchitectureEventKind::SessionForked,
        primitive: ClaudeCodeArchPrimitive::SessionFork,
        session_id: Some("s-parent".into()),
        details_json: serde_json::json!({
            "fork_event": "session_forked_dag",
            "parent_session": "s-parent",
            "child_session": "s-child",
            "branch_point": "snap-abc",
        }),
        ts_ms: 1_700_000_000_000,
    };
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: ClaudeCodeArchitectureEventPayload =
        rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded.kind, ClaudeCodeArchitectureEventKind::SessionForked);
    assert_eq!(decoded.primitive, ClaudeCodeArchPrimitive::SessionFork);
    let fork_event = decoded
        .details_json
        .get("fork_event")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(matches!(
        fork_event,
        "session_forked_dag" | "session_forked_fallback" | "session_forked_branch"
    ));
}

#[test]
fn opusplan_overlay_default_off() {
    // E-25 D3: wire-dark default OFF. With the env-gate unset the helper
    // returns false.
    let prior = std::env::var(ELAI_E25_OPUSPLAN_OVERLAY_ENABLE_VAR).ok();
    // SAFETY: tests scope env mutation + restore.
    unsafe { std::env::remove_var(ELAI_E25_OPUSPLAN_OVERLAY_ENABLE_VAR); }
    assert!(!opus_plan_overlay_enabled());
    if let Some(v) = prior {
        unsafe { std::env::set_var(ELAI_E25_OPUSPLAN_OVERLAY_ENABLE_VAR, v); }
    }
}

#[test]
fn compaction_ladder_off_via_killswitch() {
    // H-6: the E-25 ladder is now DEFAULT-ON; the kill-switch is
    // ELAI_E25_COMPACTION_LADDER_ENABLE="0". The gate-off path still returns a
    // zero-effect LadderRunResult — exercised here via the kill-switch (was: env
    // removed, when default was OFF).
    let prior = std::env::var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR).ok();
    unsafe { std::env::set_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR, "0"); }
    assert!(!compaction_ladder_enabled());
    let ladder = CompactionLadder::default_order(20_000);
    let out: LadderRunResult =
        run_compaction_ladder(&ladder, 30_000, Some("s-test".into()), &NullEmitter);
    assert!(out.tiers_run.is_empty());
    assert_eq!(out.bytes_freed, 0);
    assert_eq!(out.final_budget_tokens, 30_000);
    match prior {
        Some(v) => unsafe { std::env::set_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR, v); },
        None => unsafe { std::env::remove_var(ELAI_E25_COMPACTION_LADDER_ENABLE_VAR); },
    }
}

#[test]
fn skill_templating_default_off() {
    // E-25 D2: wire-dark default OFF; gate-off path returns body verbatim.
    let prior = std::env::var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR).ok();
    unsafe { std::env::remove_var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR); }
    assert!(!skill_templating_enabled());
    let skill = SkillTemplate {
        body: "x !`pwd` y".into(),
        source_path: std::path::PathBuf::from("/tmp/SKILL.md"),
    };
    let out = expand_skill_template(
        &skill,
        std::path::Path::new("."),
        &elai_orchestrator::skill_templating::NullEmitter,
    )
    .expect("body verbatim under default-off");
    assert_eq!(out, "x !`pwd` y");
    if let Some(v) = prior {
        unsafe { std::env::set_var(ELAI_E25_SKILL_TEMPLATING_ENABLE_VAR, v); }
    }
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let files = workspace_rs_files();
    assert!(
        !files.is_empty(),
        "e25 tranche B-4 contract walker found zero .rs files"
    );
}
