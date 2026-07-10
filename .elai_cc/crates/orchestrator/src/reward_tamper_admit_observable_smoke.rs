//! H-28-WIRE-1 integration smoke: reward-tampering gate in darwin admit loop.
//!
//! Drives the REAL entrypoint run_one_generation with ELAI_H28_REWARD_GATE_ENABLE=1
//! on a candidate set including one reward-implausible candidate, and asserts:
//! (a) that candidate is NOT admitted, (b) a DarwinianAdmit SSB row with kind
//! RejectedByGuard and guard_rejected_by == "reward-tampering" lands for it,
//! (c) with the flag unset the same set admits as on main.

use elai_core::SeededRng;
use crate::darwin_admit_loop::{
    run_one_generation, MutantCandidate, EvalFirewallContext,
};
use crate::config_archive::ConfigArchive;
use crate::promotion_gate::PromotionConfig;
use crate::ssb::{Ssb, SsbConfig, SsbActionType, SsbReader};
use crate::ssb::DarwinianAdmitKind;
use crate::self_mod_layer1a::PipelineMode;
use crate::held_out::{HeldOutStore, ModelTrainingCutoff};
use crate::holdout_budget::{HoldoutBudgetLedger, HOLDOUT_BUDGET_TOTAL, provision_fresh_holdout};
use std::sync::Arc;

/// Clean candidate with fitness PLAUSIBLE from 0.0 baseline (force_alpha=0.35 < DEFAULT_PLAUSIBLE_REWARD_BAND=0.50).
/// This candidate ADMITS when flag is ON and becomes the prior_reward for the second candidate.
fn clean_candidate() -> MutantCandidate {
    MutantCandidate {
        variant_label: "clean-001".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "force_alpha": 0.35 }),
        eval_log: "suite=math-floors score=0.35 clean".to_string(),
    }
}

/// Implausible candidate with fitness that JUMPS beyond DEFAULT_PLAUSIBLE_REWARD_BAND (0.50)
/// relative to the clean prior (0.35 -> 0.95 = 0.60 jump > 0.50 band).
/// This candidate is REJECTED by the reward-tampering guard when flag is ON.
fn reward_implausible_candidate() -> MutantCandidate {
    MutantCandidate {
        variant_label: "implausible-001".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "force_alpha": 0.95 }),
        eval_log: "suite=math-floors score=0.95".to_string(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reward_tamper_gate_rejects_implausible_when_flag_on() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let archive = ConfigArchive::open(&dir.path().join("self_mod.db"))
        .await
        .expect("archive open");
    let ssb = Ssb::new(
        SsbConfig {
            db_path: dir.path().join("ssb.db"),
            pool_size: None,
            project_id: "test".to_string(),
            session_id: "reward-tamper-smoke".to_string(),
        },
        Arc::new(elai_core::OsClock),
    )
    .expect("ssb");

    let ssb_arc = Arc::new(ssb);
    let clock = Arc::new(elai_core::OsClock) as Arc<dyn elai_core::clock::ClockProvider>;

    let held_out_store = HeldOutStore::open_in_memory().expect("held_out_store");
    let budget_ledger = HoldoutBudgetLedger::open(
        &dir.path().join("ssb.db"),
        "reward-tamper-smoke",
    ).expect("budget_ledger");

    provision_fresh_holdout(
        &budget_ledger,
        &ssb_arc,
        &clock,
        "reward-tamper-holdout",
        "fitness",
        "0..9999999999999",
        HOLDOUT_BUDGET_TOTAL,
        42,
        "reward-tamper-run",
    )
    .await
    .expect("provision holdout");

    let cutoff = ModelTrainingCutoff {
        model_id: "test-model".into(),
        date_ms: 0,
    };

    let firewall_ctx = EvalFirewallContext {
        held_out_store: &held_out_store,
        budget_ledger: &budget_ledger,
        clock: &clock,
        ssb_arc: &ssb_arc,
        cutoff: &cutoff,
        holdout_id: "reward-tamper-holdout",
        objective: "fitness",
        data_window: "0..9999999999999",
        budget_seed: 42,
        generation_id: 0,
        workload_ngrams: &[],
        ignore_dates: false,
    };

    let candidates = vec![clean_candidate(), reward_implausible_candidate()];
    let cfg = PromotionConfig::default();
    let mut rng = SeededRng::new(42);

    std::env::set_var("ELAI_H28_REWARD_GATE_ENABLE", "1");

    let summary = run_one_generation(
        &archive,
        ssb_arc.as_ref(),
        &candidates,
        PipelineMode::Surgical,
        42,
        "math-floors",
        &cfg,
        &mut rng,
        Some(&firewall_ctx),
    )
    .await
    .expect("generation");

    std::env::remove_var("ELAI_H28_REWARD_GATE_ENABLE");

    assert_eq!(summary.admitted, 1, "only clean candidate should be admitted (FACT 1: one admit per generation)");
    assert!(summary.rejected >= 1, "implausible candidate should be rejected by reward-tampering guard");

    let darwin_rows = ssb_arc.query_by_type(SsbActionType::DarwinianAdmit).expect("query DarwinianAdmit rows");
    let reward_rejected_rows: Vec<_> = darwin_rows
        .iter()
        .filter_map(|e| {
            rmp_serde::from_slice::<crate::ssb::SsbPayload>(&e.payload)
                .ok()
                .and_then(|p| match p {
                    crate::ssb::SsbPayload::DarwinianAdmit(inner) => Some(inner),
                    _ => None,
                })
        })
        .filter(|p| {
            p.kind == DarwinianAdmitKind::RejectedByGuard
                && p.guard_rejected_by.as_deref() == Some("reward-tampering")
        })
        .collect();

    assert_eq!(
        reward_rejected_rows.len(),
        1,
        "exactly ONE DarwinianAdmit row with kind RejectedByGuard and guard_rejected_by == \"reward-tampering\" must exist (load-bearing SSB assertion, R4)"
    );
    assert_eq!(
        reward_rejected_rows[0].variant_id,
        "implausible-001",
        "the reward-tampering rejection must be for the implausible candidate"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reward_tamper_gate_dormant_when_flag_off() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let archive = ConfigArchive::open(&dir.path().join("self_mod.db"))
        .await
        .expect("archive open");
    let ssb = Ssb::new(
        SsbConfig {
            db_path: dir.path().join("ssb.db"),
            pool_size: None,
            project_id: "test".to_string(),
            session_id: "reward-tamper-smoke".to_string(),
        },
        Arc::new(elai_core::OsClock),
    )
    .expect("ssb");

    let ssb_arc = Arc::new(ssb);

    let candidates = vec![clean_candidate()];
    let cfg = PromotionConfig::default();
    let mut rng = SeededRng::new(42);

    let summary = run_one_generation(
        &archive,
        ssb_arc.as_ref(),
        &candidates,
        PipelineMode::Surgical,
        42,
        "math-floors",
        &cfg,
        &mut rng,
        None, // firewall_ctx = None (byte-for-byte production main; reward gate is behind the flag anyway)
    )
    .await
    .expect("generation");

    assert_eq!(summary.admitted, 1, "clean candidate should be admitted (FACT 1: one admit per generation)");
    assert_eq!(summary.rejected, 0, "no rejections when flag is off");
}
