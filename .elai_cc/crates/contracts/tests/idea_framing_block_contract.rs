//! B-40 — Idea-Framer BLOCK contract.
//!
//! Asserts that the critic loop refuses to advance when unresolved C-* BLOCK
//! findings remain at the final round. The deterministic critic emits a
//! BLOCK Missing finding when `raw_prompt` is empty (and we mutate the
//! FramedIdea in-place to force this condition after the initial seed).
//!
//! Spec ref: B-40 plan ACCEPTANCE line 3 ("Critic loop converges in <= 3
//! rounds on 95% of fixture ideas; remaining 5% emit idea_framing_block
//! (acceptable failure mode)") + observable_effect / stub_ban
//! ("run_critic_loop must NOT short-circuit max_rounds=3 enforced even
//! when 0 findings").

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::{
    run_critic_loop, CriticCategory, CriticFinding, FramedIdea, IdeaFramingError, KeyDecision,
    ProjectClass, ResearchSummary, Severity, Tradeoff, TradeoffCard, MAX_CRITIC_ROUNDS,
};

fn os_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn build_framed_with_block_condition() -> FramedIdea {
    FramedIdea {
        project_class: ProjectClass::Other,
        // raw_prompt empty -> deterministic critic emits C-MISSING / BLOCK
        // every round.
        raw_prompt: "".to_owned(),
        research_summary: ResearchSummary {
            streams: Vec::new(),
            degraded_streams: Vec::new(),
        },
        key_decisions: Vec::new(),
        known_unknowns: Vec::new(),
        tradeoff_card: TradeoffCard::default(),
        critic_rounds: Vec::new(),
    }
}

#[tokio::test]
async fn contract_block_on_unresolved_critic_at_round_3() {
    let mut framed = build_framed_with_block_condition();
    let result = run_critic_loop(
        &mut framed,
        MAX_CRITIC_ROUNDS,
        None,
        "sess-block-test",
        None,  // I-41: no advisory runtime in this contract fixture
    )
    .await;
    match result {
        Err(IdeaFramingError::MaxRoundsReachedWithBlocks { block_count }) => {
            assert!(
                block_count >= 1,
                "CONTRACT: BLOCK error must report >= 1 unresolved BLOCK finding"
            );
        }
        Ok(rounds) => panic!(
            "CONTRACT: expected MaxRoundsReachedWithBlocks; got Ok({})",
            rounds
        ),
        Err(other) => panic!("CONTRACT: expected MaxRoundsReachedWithBlocks; got {}", other),
    }
    // Verify that exactly MAX_CRITIC_ROUNDS rounds ran (no auto-advance,
    // no short-circuit).
    assert_eq!(
        framed.critic_rounds.len() as u32,
        MAX_CRITIC_ROUNDS,
        "CONTRACT: must run all {} rounds before emitting BLOCK",
        MAX_CRITIC_ROUNDS
    );
}

#[tokio::test]
async fn contract_no_auto_advance_when_blocks_remain() {
    // Even if max_rounds is requested higher than MAX_CRITIC_ROUNDS, the
    // loop caps at 3. is_converged() must return false at the end.
    let mut framed = build_framed_with_block_condition();
    let _ = run_critic_loop(&mut framed, 99, None, "sess-no-aa", None).await;
    assert!(
        !framed.is_converged(),
        "CONTRACT: framed-idea with persistent BLOCK must NOT report converged"
    );
    assert!(
        framed.critic_rounds.len() as u32 <= MAX_CRITIC_ROUNDS,
        "CONTRACT: critic_rounds.len() <= MAX_CRITIC_ROUNDS (cap is enforced)"
    );
}

#[tokio::test]
async fn contract_critic_loop_emits_at_least_one_round_even_when_zero_findings() {
    // stub_ban: run_critic_loop must NOT short-circuit when zero findings.
    // For a clean FramedIdea the deterministic critic still records the
    // round (with zero BLOCKs) and exits via converge. Per B-40 plan L203
    // ("max_rounds=3 enforced even when 0 findings") this means the loop
    // ALWAYS runs at least round 1, even if the first round is empty.
    let mut framed = FramedIdea {
        project_class: ProjectClass::ConsumerMobileApp,
        raw_prompt: "a long enough idea to avoid C-VAGUE and C-MISSING".to_owned(),
        research_summary: ResearchSummary {
            streams: Vec::new(),
            degraded_streams: Vec::new(),
        },
        key_decisions: vec![KeyDecision {
            name: "k".to_owned(),
            rationale: "r".to_owned(),
            source_refs: Vec::new(),
        }],
        known_unknowns: Vec::new(),
        tradeoff_card: TradeoffCard {
            tradeoffs: vec![Tradeoff {
                name: "t1".to_owned(),
                options: vec!["a".to_owned(), "b".to_owned()],
                default: "a".to_owned(),
                user_facing_question: "?".to_owned(),
            }],
        },
        critic_rounds: Vec::new(),
    };
    let rounds = run_critic_loop(&mut framed, MAX_CRITIC_ROUNDS, None, "sess-zero-findings", None)
        .await
        .expect("clean idea should converge");
    assert!(
        rounds >= 1,
        "CONTRACT: at least one critic round must execute even when zero findings"
    );
    assert_eq!(
        framed.critic_rounds.len() as u32,
        rounds,
        "CONTRACT: critic_rounds.len() must equal reported rounds_used"
    );
}

#[test]
fn contract_critic_finding_block_severity_string() {
    let f = CriticFinding {
        category: CriticCategory::Missing,
        severity: Severity::Block,
        evidence_ref: None,
        proposed_fix: "x".to_owned(),
    };
    assert_eq!(f.severity.as_snake_case(), "block");
    let _ = os_clock(); // touch helper so it's not unused
}
