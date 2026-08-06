//! J-79 contract: signal-triggered grill interlace (Tier-1).

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::grill_interlace::{
    apply_grill_return_path, critic_blocks_cleared, intent_pairs_from_resolved,
    load_intent_pairs_for_gate, maybe_offer_grill_on_critic_block, offer_line,
    select_rung_from_block_count, ClarifyRung, GrillOfferTrigger, GrillResolvedDecisions,
};
use elai_orchestrator::idea_framing::{
    CriticCategory, CriticFinding, CriticRound, FramedIdea, ProjectClass, ResearchSummary,
    Severity, TradeoffCard,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use std::sync::Arc;

#[test]
fn d1_zero_blocks_returns_none() {
    assert!(select_rung_from_block_count(0).is_none());
}

#[test]
fn d2_two_blocks_rung1_offer() {
    let rung = select_rung_from_block_count(2).expect("rung1");
    match &rung {
        ClarifyRung::Rung1OfferGrill {
            branch_count,
            trigger: GrillOfferTrigger::IdeaCriticBlock { block_count },
        } => {
            assert_eq!(*branch_count, 2);
            assert_eq!(*block_count, 2);
        }
        other => panic!("expected Rung1, got {other:?}"),
    }
    let line = offer_line(&rung);
    assert!(line.to_lowercase().contains("grill"));
    assert!(!line.to_lowercase().contains("already ran"));
}

#[test]
fn d3_three_branches_pairs() {
    let d = GrillResolvedDecisions {
        resolved_branches: vec![
            ("a".into(), "do a".into()),
            ("b".into(), "do b".into()),
            ("c".into(), "do c".into()),
        ],
        glossary: vec![],
        contradictions_resolved: true,
    };
    assert!(intent_pairs_from_resolved(&d).len() >= 3);
    assert!(critic_blocks_cleared(&d));
}

#[test]
fn d4_blocks_not_cleared() {
    let d = GrillResolvedDecisions {
        resolved_branches: vec![("a".into(), "do a".into())],
        glossary: vec![],
        contradictions_resolved: false,
    };
    assert!(!critic_blocks_cleared(&d));
}

#[test]
fn d5_public_symbols_reachable() {
    let _ = select_rung_from_block_count as fn(u32) -> Option<ClarifyRung>;
    let _ = offer_line as fn(&ClarifyRung) -> String;
    let _ = intent_pairs_from_resolved as fn(&GrillResolvedDecisions) -> _;
    let _ = critic_blocks_cleared as fn(&GrillResolvedDecisions) -> bool;
    let _ = load_intent_pairs_for_gate as fn(&std::path::Path) -> _;
}

fn blocked_framed() -> FramedIdea {
    let finding = CriticFinding {
        category: CriticCategory::Scope,
        severity: Severity::Block,
        evidence_ref: None,
        proposed_fix: "resolve multi-branch ambiguity via grill".into(),
    };
    let round = CriticRound {
        round_index: 1,
        findings: vec![finding],
    };
    FramedIdea {
        project_class: ProjectClass::Other,
        raw_prompt: "build something ambiguous".into(),
        research_summary: ResearchSummary {
            streams: vec![],
            degraded_streams: vec![],
        },
        key_decisions: vec![],
        known_unknowns: vec![],
        tradeoff_card: TradeoffCard { tradeoffs: vec![] },
        critic_rounds: vec![round],
    }
}

fn converged_framed() -> FramedIdea {
    let mut f = blocked_framed();
    f.critic_rounds = vec![CriticRound {
        round_index: 1,
        findings: vec![],
    }];
    f
}

fn temp_ssb() -> (tempfile::TempDir, Arc<Ssb>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: Some(2),
                project_id: "j79-test".into(),
                session_id: "j79-sess".into(),
            },
            clock,
        )
        .expect("ssb new"),
    );
    (dir, ssb)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d7_offer_when_blocked() {
    let (_dir, ssb) = temp_ssb();
    let offer = maybe_offer_grill_on_critic_block(&ssb, &blocked_framed()).await;
    assert!(offer.is_some());
    assert!(offer.unwrap().to_lowercase().contains("grill"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d7b_none_when_converged() {
    let (_dir, ssb) = temp_ssb();
    assert!(maybe_offer_grill_on_critic_block(&ssb, &converged_framed())
        .await
        .is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d8_apply_return_closes_loop() {
    let (dir, ssb) = temp_ssb();
    let session = dir.path().join("session");
    std::fs::create_dir_all(&session).unwrap();
    let resolved = GrillResolvedDecisions {
        resolved_branches: vec![
            ("a".into(), "do a".into()),
            ("b".into(), "do b".into()),
            ("c".into(), "do c".into()),
        ],
        glossary: vec![],
        contradictions_resolved: true,
    };
    std::fs::write(
        session.join("grill_resolved.toml"),
        toml::to_string_pretty(&resolved).unwrap(),
    )
    .unwrap();

    let mut framed = blocked_framed();
    assert!(!framed.is_converged());
    let effect = apply_grill_return_path(&ssb, &session, Some(&mut framed))
        .await
        .expect("return applied");
    assert!(effect.blocks_cleared);
    assert!(effect.pairs_len >= 3);
    assert!(effect.pairs_path.is_file());
    assert!(framed.is_converged(), "B-40 blocks cleared on framed");
    let gate_pairs = load_intent_pairs_for_gate(&session).expect("pairs for C-29");
    assert!(gate_pairs.len() >= 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d8b_missing_resolved_none() {
    let (dir, ssb) = temp_ssb();
    assert!(apply_grill_return_path(&ssb, dir.path(), None).await.is_none());
}

#[test]
fn d9_skill_mirror_keywords() {
    let skill_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../skills/grill/SKILL.md");
    let text = std::fs::read_to_string(&skill_path).unwrap_or_else(|e| {
        panic!("read {}: {e}", skill_path.display());
    });
    let lower = text.to_lowercase();
    assert!(lower.contains("never auto-run"));
    assert!(lower.contains("offer"));
    assert!(lower.contains("consent"));
}
