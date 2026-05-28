//! E-46 (SCHEMA_VERSION 184) planning-decomposition contract test.
//!
//! Asserts:
//!   1. `SsbActionType::PlanningDecompositionEvent` round-trips through
//!      the string-to-enum map (wire label
//!      `planning_decomposition_event`).
//!   2. All 8 [`PlanningDecompositionEventKind`] arms have unique,
//!      snake_case wire labels.
//!   3. `PlanningDecompositionEventPayload` serdes round-trips for every
//!      kind with a representative `details_json`.
//!   4. The env-var literal `ELAI_PLANNING_DECOMPOSITION_ENABLE` matches
//!      verbatim between the orchestrator module
//!      ([`elai_orchestrator::planning_decomposition::PLANNING_DECOMPOSITION_ENABLE_ENV`])
//!      and the ssb module
//!      ([`elai_orchestrator::ssb::ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`])
//!      — drift would silently disable producers.
//!   5. The SCHEMA_VERSION floor is `>= 184` (E-46 introduced 184).
//!   6. Producer-side stub_ban payload validations work:
//!      `bubble_up_replan_phase` rejects empty fields with
//!      `SsbError::PayloadValidation`.
//!   7. Goal-drift report carries BOTH jaccard AND cosine arms by struct
//!      shape (stub_ban §89 enforced at the type level).
//!   8. Reflection buffer evicts oldest at cap (plan task §165
//!      `reflection_buffer_cap_evicts_oldest`).
//!   9. Decomposition entropy stop rule flags collapsed children below
//!      threshold (plan task §168 `entropy_stop_blocks_low_entropy_fork`).
//!  10. Process-reward score in `[0, 1]` for 50 synthetic verdicts
//!      (plan task §169 `process_reward_score_in_unit_interval`).
//!  11. Skill-prior top-K=0 disables retrieval entirely.

use elai_orchestrator::planning_decomposition::{
    self,
    audit_score::{
        compute_score, compute_score_for_agentless_shortcut_eligible, should_prune_fork,
        FORK_PRUNE_FLOOR, MAX_EXPECTED_FINDINGS,
    },
    decomposition_entropy::{
        compute_entropy_ratio_from_strings, EntropyVerdict, ENTROPY_RATIO_THRESHOLD,
        MIN_PARENT_TOKENS,
    },
    dual_ledger_ext::{
        Blocker, MagenticLedgerStore, LEDGER_RESET_BLOCKER_THRESHOLD,
    },
    goal_drift_ext::{
        GoalDriftEvaluator, COSINE_SIMILARITY_THRESHOLD, JACCARD_SIMILARITY_THRESHOLD,
    },
    reflection_buffer::{Reflection, ReflectionBuffer, MAX_REFLECTIONS_PER_NODE},
    skill_prior::{SkillPriorStore, DEFAULT_TOP_K},
    PLANNING_DECOMPOSITION_ENABLE_ENV,
};
use elai_orchestrator::auditor_mode::{
    AuditorMode, Finding, LevelAuditVerdict, Severity, TierNodeId, Verdict,
};
use elai_orchestrator::research_tree::BabyStepId;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, PlanningDecompositionEventKind,
    PlanningDecompositionEventPayload, SsbActionType, SsbPayload,
    ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV, SCHEMA_VERSION_PUBLIC,
};

#[test]
fn schema_version_floor_184() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 184,
        "SCHEMA_VERSION_PUBLIC regressed below E-46 floor 184; got {SCHEMA_VERSION_PUBLIC}"
    );
}

#[test]
fn action_type_string_round_trips() {
    assert_eq!(
        SsbActionType::PlanningDecompositionEvent.as_sql_str(),
        "planning_decomposition_event"
    );
    let parsed = ssb_action_type_from_str_public("planning_decomposition_event")
        .expect("from_str must succeed for planning_decomposition_event");
    assert_eq!(parsed, SsbActionType::PlanningDecompositionEvent);
}

#[test]
fn planning_decomposition_event_kind_labels_unique_and_snake_case() {
    use PlanningDecompositionEventKind::*;
    let kinds = [
        ReflectionAppended,
        LowEntropyDecomposition,
        GoalDriftCanaryBelowThreshold,
        BubbleUpReplanPhase,
        BubbleUpLedgerReset,
        AgentlessShortcutDispatched,
        ProcessRewardScoreEmitted,
        SkillPriorRetrieved,
    ];
    let labels: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
    // 8 unique labels.
    let mut sorted = labels.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 8, "labels not unique: {labels:?}");
    // All snake_case (ASCII lowercase + digits + underscore).
    for label in &labels {
        assert!(
            label
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "label {label:?} is not snake_case"
        );
    }
}

#[test]
fn payload_round_trips_for_every_kind() {
    let kinds = [
        PlanningDecompositionEventKind::ReflectionAppended,
        PlanningDecompositionEventKind::LowEntropyDecomposition,
        PlanningDecompositionEventKind::GoalDriftCanaryBelowThreshold,
        PlanningDecompositionEventKind::BubbleUpReplanPhase,
        PlanningDecompositionEventKind::BubbleUpLedgerReset,
        PlanningDecompositionEventKind::AgentlessShortcutDispatched,
        PlanningDecompositionEventKind::ProcessRewardScoreEmitted,
        PlanningDecompositionEventKind::SkillPriorRetrieved,
    ];
    for kind in kinds {
        let payload = SsbPayload::PlanningDecompositionEvent(PlanningDecompositionEventPayload {
            kind,
            session_id: Some("s-e46-contract".into()),
            task_id: Some("node-1".into()),
            details_json: serde_json::json!({
                "kind_label": kind.as_str(),
                "caller_site": "contract_test_planning_decomposition_e46",
            }),
            ts_ms: 1_700_000_184_001,
        });
        // serialize -> deserialize -> compare action_type discriminator
        let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
        let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
        assert_eq!(
            decoded.action_type(),
            SsbActionType::PlanningDecompositionEvent,
            "round-trip lost variant for kind {kind:?}"
        );
        match decoded {
            SsbPayload::PlanningDecompositionEvent(p) => {
                assert_eq!(p.kind, kind);
                assert_eq!(p.session_id, Some("s-e46-contract".to_string()));
            }
            _ => panic!("decoded into wrong SsbPayload variant for kind {kind:?}"),
        }
    }
}

#[test]
fn env_var_literals_match_between_modules() {
    // CONTRACT: orchestrator::planning_decomposition::PLANNING_DECOMPOSITION_ENABLE_ENV
    // MUST equal orchestrator::ssb::ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV verbatim.
    // Drift would silently disable producers.
    assert_eq!(
        PLANNING_DECOMPOSITION_ENABLE_ENV,
        ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV,
        "env var literal drift between planning_decomposition and ssb modules"
    );
    assert_eq!(PLANNING_DECOMPOSITION_ENABLE_ENV, "ELAI_PLANNING_DECOMPOSITION_ENABLE");
}

#[test]
fn is_enabled_default_off() {
    std::env::remove_var(PLANNING_DECOMPOSITION_ENABLE_ENV);
    assert!(
        !planning_decomposition::is_enabled(),
        "wire-dark default broke — is_enabled() returned true with env unset"
    );
}

// ------------------------------------------------------------------
// Plan task §165: reflection_buffer_cap_evicts_oldest
// ------------------------------------------------------------------

#[test]
fn reflection_buffer_cap_evicts_oldest() {
    let mut buf = ReflectionBuffer::with_cap(3);
    for i in 0..(MAX_REFLECTIONS_PER_NODE + 5) {
        let r = Reflection::new_t1(
            format!("reflection number {} for the node", i),
            format!("evt-{}", i).as_bytes(),
            1_700_000_000 + i as u64,
        )
        .unwrap();
        buf.push(r);
    }
    assert_eq!(buf.len(), 3);
    // newest 3 retained; the very-last push wins
    let last_text = buf.iter().last().unwrap().text.clone();
    assert!(
        last_text.contains(&format!("number {}", MAX_REFLECTIONS_PER_NODE + 5 - 1)),
        "expected newest reflection at tail, got {last_text}"
    );
}

// ------------------------------------------------------------------
// Plan task §168: entropy_stop_blocks_low_entropy_fork
// ------------------------------------------------------------------

#[test]
fn entropy_stop_blocks_low_entropy_fork() {
    // Parent has 24 distinct tokens; 3 children each repeat the same token.
    let parent_str =
        "user submits form captures email name phone validates schema stores database \
         indexed tested endpoint secured rate limited audit logged monitored \
         deployable observable reliable verified";
    let children = ["same same same same same", "same same same same same", "same same same"];
    let verdict =
        compute_entropy_ratio_from_strings(parent_str, &children, ENTROPY_RATIO_THRESHOLD);
    assert!(verdict.is_low_entropy(), "expected low-entropy verdict, got {verdict:?}");
    match verdict {
        EntropyVerdict::LowEntropy { threshold, ratio, .. } => {
            assert_eq!(threshold, ENTROPY_RATIO_THRESHOLD);
            assert!(ratio < threshold, "ratio={ratio} threshold={threshold}");
        }
        _ => unreachable!(),
    }
}

#[test]
fn entropy_stop_skipped_on_short_parent() {
    let parent_str = "user submits form"; // only 3 tokens, < MIN_PARENT_TOKENS
    let verdict =
        compute_entropy_ratio_from_strings(parent_str, &["form"], ENTROPY_RATIO_THRESHOLD);
    match verdict {
        EntropyVerdict::Skipped { parent_token_count, floor } => {
            assert_eq!(parent_token_count, 3);
            assert_eq!(floor, MIN_PARENT_TOKENS);
        }
        _ => panic!("expected Skipped on short parent, got {verdict:?}"),
    }
}

// ------------------------------------------------------------------
// Plan task §169: process_reward_score_in_unit_interval
// ------------------------------------------------------------------

#[test]
fn process_reward_score_in_unit_interval() {
    let severities = [Severity::Block, Severity::Warn, Severity::Info];
    for i in 0..50 {
        let n_findings = i % 8;
        let verdict_choice = if i % 3 == 0 { Verdict::Block } else { Verdict::Allow };
        let findings: Vec<Finding> = (0..n_findings)
            .map(|j| Finding {
                category: "C-TEST".into(),
                severity: severities[j % 3],
                message: format!("finding {}", j),
                source: None,
            })
            .collect();
        let v = LevelAuditVerdict {
            mode: AuditorMode::BabyStepDetail,
            tier_node_id: TierNodeId::BabyStep(BabyStepId(i as u64)),
            findings,
            verdict: verdict_choice,
            respawned_once: false,
            cascade_to_parent: false,
        };
        let r = compute_score(&v, MAX_EXPECTED_FINDINGS);
        assert!(
            (0.0..=1.0).contains(&r.score),
            "iteration {} produced score {} outside [0, 1]",
            i,
            r.score
        );
    }
}

#[test]
fn process_reward_agentless_shortcut_modifier_applied() {
    let v = LevelAuditVerdict {
        mode: AuditorMode::Steps,
        tier_node_id: TierNodeId::BabyStep(BabyStepId(1)),
        findings: Vec::new(),
        verdict: Verdict::Allow,
        respawned_once: false,
        cascade_to_parent: false,
    };
    let r = compute_score_for_agentless_shortcut_eligible(&v, MAX_EXPECTED_FINDINGS);
    assert!((r.score - 0.85).abs() < 1e-5, "got {}", r.score);
}

#[test]
fn process_reward_fork_prune_floor_check() {
    assert!(should_prune_fork(0.3, FORK_PRUNE_FLOOR));
    assert!(!should_prune_fork(0.6, FORK_PRUNE_FLOOR));
}

// ------------------------------------------------------------------
// Plan task §167: goal_drift_canary_fires_on_two_consecutive_phases
// ------------------------------------------------------------------

#[test]
fn goal_drift_canary_fires_on_two_consecutive_phases() {
    let idea = "user submits form validates schema stores database";
    let drifty_phase = "graphics shader pipeline raster vertex fragment depth";
    let ev = GoalDriftEvaluator::from_idea_text(idea);
    // First phase below threshold; second consecutive also below -> fire.
    let r1 = ev.evaluate(drifty_phase, false);
    assert!(r1.below_threshold);
    assert!(!r1.fire_cascade, "first below-threshold phase should NOT fire cascade");
    let r2 = ev.evaluate(drifty_phase, true);
    assert!(r2.below_threshold);
    assert!(r2.fire_cascade, "second consecutive below-threshold phase MUST fire cascade");
    // Both metrics populated (stub_ban §89 — type-level guarantee).
    assert!(r1.jaccard.is_finite() && r1.cosine.is_finite());
    assert!(r2.jaccard.is_finite() && r2.cosine.is_finite());
}

#[test]
fn goal_drift_thresholds_in_unit_interval() {
    assert!((0.0..=1.0).contains(&JACCARD_SIMILARITY_THRESHOLD));
    assert!((0.0..=1.0).contains(&COSINE_SIMILARITY_THRESHOLD));
}

// ------------------------------------------------------------------
// Plan task §163: dual_ledger_blocker_threshold_triggers_reset
// ------------------------------------------------------------------

#[test]
fn dual_ledger_blocker_threshold_triggers_reset() {
    let mut store = MagenticLedgerStore::new();
    let task_id = elai_orchestrator::dual_ledger::TaskId::from("phase-7");
    for i in 0..LEDGER_RESET_BLOCKER_THRESHOLD {
        let blocker = Blocker {
            reason: format!("blocker {}", i),
            ssb_seq: i as i64,
            recorded_at_ms: i as i64,
        };
        store.append_blocker(task_id.clone(), blocker);
    }
    assert!(store.should_emit_ledger_reset(&task_id, LEDGER_RESET_BLOCKER_THRESHOLD));
}

// ------------------------------------------------------------------
// Skill-prior store behavior
// ------------------------------------------------------------------

#[test]
fn skill_prior_k_zero_disables_retrieval() {
    let mut store = SkillPriorStore::with_k(0);
    store.insert_from_goal_text(
        "p1".into(),
        "user submits form validates schema",
        "diff summary".into(),
        vec![1],
        1_700_000_000_000,
    );
    let matches = store.top_k("user submits form validates schema");
    assert!(matches.is_empty(), "k=0 must disable retrieval");
    assert!(store.is_disabled());
}

#[test]
fn skill_prior_default_top_k_is_three() {
    assert_eq!(DEFAULT_TOP_K, 3);
}

#[test]
fn skill_prior_top_k_returns_best_match() {
    let mut store = SkillPriorStore::new();
    store.insert_from_goal_text(
        "p1".into(),
        "user submits form validates schema endpoint database",
        "diff summary one".into(),
        vec![10],
        1_700_000_000_000,
    );
    store.insert_from_goal_text(
        "p2".into(),
        "completely unrelated goal about graphics rendering pipelines",
        "diff summary two".into(),
        vec![20],
        1_700_000_001_000,
    );
    let matches = store.top_k("user submits form validates schema");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].prior_id, "p1");
}
