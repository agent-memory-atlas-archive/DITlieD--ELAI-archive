//! J-48 contract: LocalServingRouter::decide() behavioural pins + W-1
//! off-agent RouteDecisionEvent row from a real try_complete drive.
//!
//! Calls the SAME `LocalServingRouter::decide()` that production
//! `RuntimeOrchestrator::try_complete` calls (D-174: no re-implementation,
//! no source-text grep). Pins the six decision branches the plan specifies:
//! (a) ServeLocal warm under cap, (b) AdmitQueued at cap, (c)
//! HibernateThenServe EXCLUSIVE, (d) SpillToCloud non-empty reason when
//! busy/cold + cloud_available, (e) never spill when !cloud_available,
//! (f) corrupt inflight > Wait, never panic.
//!
//! W-1 (off-agent artifact): drives the REAL `try_complete` dispatch path
//! with ELAI_ROUTE_DECISION_SSB_ENABLE=1, first warming the High tier then
//! requesting Low (which triggers SpillToCloud because High is warm). Asserts
//! a RouteDecisionEvent SSB row with kind=CloudRouteChosen + non-empty reason
//! was written by the orchestrator's own emit path (not by the test).
//!
//! Grounding:
//!   * crates/orchestrator/src/local_serving.rs (the router)
//!   * crates/orchestrator/src/runtime_orchestrator.rs L384-404 (production call)
//!   * J-48 plan: local multi-model serving tier router

use std::sync::Arc;
use std::sync::Mutex;

use elai_orchestrator::local_serving::{
    DecideInput, LocalServingDecision, LocalServingRouter, NamedClass, TierResidencySnapshot,
    TierState, TIER_CAP_LOW,
};
use elai_orchestrator::role_predict::VramTier;
use elai_orchestrator::runtime_orchestrator::RuntimeOrchestrator;
use elai_orchestrator::runtime_config::RuntimeConfig;
use elai_orchestrator::ssb::{
    RouteDecisionEventKind, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    ELAI_ROUTE_DECISION_SSB_ENABLE_ENV,
};
use elai_core::clock::OsClock;
use elai_ipc_types::AgentRole;
use elai_providers::runtime::{Message, MessageRole, RuntimeBackend};

fn empty_residency() -> TierResidencySnapshot {
    TierResidencySnapshot::new()
}

fn router() -> LocalServingRouter {
    LocalServingRouter::new()
}

// --- (a) ServeLocal: tier warm and inflight < cap ---

#[test]
fn contract_serve_local_warm_under_cap() {
    let mut r = empty_residency();
    r.insert(
        VramTier::Medium,
        TierState {
            warm: true,
            inflight: 1,
        },
    );
    let input = DecideInput {
        requested_tier: VramTier::Medium,
        residency: &r,
        cloud_available: true,
        spend: None,
    };
    let d = router().decide(&input);
    assert_eq!(d, LocalServingDecision::ServeLocal { tier: VramTier::Medium });
}

// --- (b) AdmitQueued: tier warm but inflight >= cap ---

#[test]
fn contract_admit_queued_at_cap() {
    let mut r = empty_residency();
    r.insert(
        VramTier::Low,
        TierState {
            warm: true,
            inflight: TIER_CAP_LOW,
        },
    );
    let input = DecideInput {
        requested_tier: VramTier::Low,
        residency: &r,
        cloud_available: true,
        spend: None,
    };
    let d = router().decide(&input);
    assert_eq!(
        d,
        LocalServingDecision::AdmitQueued {
            tier: VramTier::Low,
            class: NamedClass::Autonomous,
        }
    );
}

// --- (c) HibernateThenServe: big tier requested, small tier warm (EXCLUSIVE) ---

#[test]
fn contract_hibernate_then_serve_exclusive() {
    let mut r = empty_residency();
    r.insert(VramTier::High, TierState::warm());
    r.insert(VramTier::Low, TierState::warm());
    let input = DecideInput {
        requested_tier: VramTier::High,
        residency: &r,
        cloud_available: false,
        spend: None,
    };
    let d = router().decide(&input);
    assert_eq!(
        d,
        LocalServingDecision::HibernateThenServe {
            evict: VramTier::Low,
            serve: VramTier::High,
        }
    );
}

// --- (d) SpillToCloud: non-empty reason when tier cold with concurrent
//     load + cloud_available (cold-start goes ServeLocal, not spill) ---

#[test]
fn contract_spill_to_cloud_non_empty_reason() {
    let mut r = empty_residency();
    r.insert(
        VramTier::Medium,
        TierState {
            warm: false,
            inflight: 1,
        },
    );
    let input = DecideInput {
        requested_tier: VramTier::Medium,
        residency: &r,
        cloud_available: true,
        spend: None,
    };
    let d = router().decide(&input);
    match d {
        LocalServingDecision::SpillToCloud { reason } => {
            assert!(!reason.is_empty(), "spill reason must be non-empty");
        }
        other => panic!("expected SpillToCloud, got {other:?}"),
    }
}

// --- (d1) Cold-start: empty residency => ServeLocal (admit to warm) ---

#[test]
fn contract_cold_start_serve_local() {
    let r = empty_residency();
    let input = DecideInput {
        requested_tier: VramTier::Medium,
        residency: &r,
        cloud_available: true,
        spend: None,
    };
    let d = router().decide(&input);
    assert_eq!(d, LocalServingDecision::ServeLocal { tier: VramTier::Medium });
}

// --- (d2) SpillToCloud: big tier warm forces small tier cold + cloud ---

#[test]
fn contract_spill_to_cloud_big_tier_warm() {
    let mut r = empty_residency();
    r.insert(VramTier::High, TierState::warm());
    let input = DecideInput {
        requested_tier: VramTier::Medium,
        residency: &r,
        cloud_available: true,
        spend: None,
    };
    let d = router().decide(&input);
    match d {
        LocalServingDecision::SpillToCloud { reason } => {
            assert!(!reason.is_empty(), "spill reason must be non-empty");
        }
        other => panic!("expected SpillToCloud, got {other:?}"),
    }
}

// --- (e) Never spill when !cloud_available (air-gapped) ---

#[test]
fn contract_never_spill_when_air_gapped() {
    let r = empty_residency();
    let input = DecideInput {
        requested_tier: VramTier::Medium,
        residency: &r,
        cloud_available: false,
        spend: None,
    };
    let d = router().decide(&input);
    assert!(
        !matches!(d, LocalServingDecision::SpillToCloud { .. }),
        "must never spill when cloud unavailable"
    );
}

// --- (f) Corrupt inflight -> Wait, never panic ---

#[test]
fn contract_corrupt_inflight_safe_wait() {
    let mut r = empty_residency();
    r.insert(
        VramTier::Medium,
        TierState {
            warm: true,
            inflight: u32::MAX / 2 + 1,
        },
    );
    let input = DecideInput {
        requested_tier: VramTier::Medium,
        residency: &r,
        cloud_available: true,
        spend: None,
    };
    let d = router().decide(&input);
    assert_eq!(d, LocalServingDecision::Wait);
}

// --- (f2) Cold tier + no cloud -> cold-start ServeLocal (not Wait, not
//     panic); the router admits cold tiers to warm locally ---

#[test]
fn contract_cold_tier_no_cloud_serve_local() {
    let r = empty_residency();
    let input = DecideInput {
        requested_tier: VramTier::Low,
        residency: &r,
        cloud_available: false,
        spend: None,
    };
    let d = router().decide(&input);
    assert_eq!(d, LocalServingDecision::ServeLocal { tier: VramTier::Low });
}

// ---------------------------------------------------------------------------
// W-1 off-agent artifact: RouteDecisionEvent CloudRouteChosen row from a
// real try_complete drive.
//
// This test drives the REAL production dispatch method `try_complete` (NOT
// `decide()` directly) with ELAI_ROUTE_DECISION_SSB_ENABLE=1. The first
// call sets ELAI_VRAM_TIER=high so the orchestrator warms the High tier in
// its internal tier_residency map. The second call sets ELAI_VRAM_TIER=low
// so the router sees High warm + Low requested + cloud_available and returns
// SpillToCloud. The orchestrator's own emit path (runtime_orchestrator.rs
// L449-493) writes a RouteDecisionEvent SSB row with kind=CloudRouteChosen.
//
// FALSIFICATION: deleting the `write_payload(SsbPayload::RouteDecisionEvent(
// ...))` block inside try_complete (L477-484) turns this test RED because no
// RouteDecisionEvent row would be written. The test kills the mutation
// "remove the RouteDecisionEvent emit from try_complete's pre-step."
//
// Env mutation: static Mutex (poison-tolerant) + EnvRestoreGuard per the
// canonical R2 pair (j35_intent_control_contract.rs /
// j40_stub_ban_contract.rs). The test is serial to avoid env races.

static W1_ENV_MUTEX: Mutex<()> = Mutex::new(());

struct W1EnvGuard {
    keys: [(&'static str, Option<String>); 2],
}

impl W1EnvGuard {
    fn capture() -> Self {
        W1EnvGuard {
            keys: [
                (ELAI_ROUTE_DECISION_SSB_ENABLE_ENV, std::env::var(ELAI_ROUTE_DECISION_SSB_ENABLE_ENV).ok()),
                ("ELAI_VRAM_TIER", std::env::var("ELAI_VRAM_TIER").ok()),
            ],
        }
    }
}

impl Drop for W1EnvGuard {
    fn drop(&mut self) {
        for (key, prior) in &self.keys {
            match prior {
                Some(val) => std::env::set_var(key, val),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn w1_try_complete_emits_cloud_route_chosen_off_agent() {
    let _mutex = W1_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = W1EnvGuard::capture();

    // --- SSB setup ---
    let dir = tempfile::TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("j48_w1.db"),
                pool_size: Some(2),
                project_id: "test-j48".into(),
                session_id: "s-j48-w1".into(),
            },
            Arc::new(OsClock),
        )
        .expect("ssb init"),
    );

    // --- orchestrator config: single local backend (Ollama stub) ---
    let mut cfg = RuntimeConfig::default();
    cfg.per_role.set(AgentRole::Executor, RuntimeBackend::Ollama);
    cfg.fallback_chain = vec![RuntimeBackend::Ollama];
    let orch = RuntimeOrchestrator::new(cfg).with_ssb(Arc::clone(&ssb));

    let messages = vec![Message {
        role: MessageRole::User,
        content: "ping".into(),
    }];

    // Step 1: ELAI_VRAM_TIER=high + SSB enabled. try_complete warms the
    // High tier in tier_residency. The Ollama stub fails on complete, but
    // the route decision emit runs BEFORE the fallback chain, so the SSB
    // row is written regardless of dispatch outcome.
    std::env::set_var(ELAI_ROUTE_DECISION_SSB_ENABLE_ENV, "1");
    std::env::set_var("ELAI_VRAM_TIER", "high");
    let _ = orch
        .try_complete(AgentRole::Executor, messages.clone(), elai_providers::runtime::CompletionConfig::default(), 0)
        .await;

    // Step 2: ELAI_VRAM_TIER=low. The High tier is still warm in
    // tier_residency from step 1. The router sees High warm + Low requested
    // + cloud_available (CloudConfig::default().air_gapped=false) and
    // returns SpillToCloud. The orchestrator emits a RouteDecisionEvent
    // with kind=CloudRouteChosen.
    std::env::set_var("ELAI_VRAM_TIER", "low");
    let _ = orch
        .try_complete(AgentRole::Executor, messages, elai_providers::runtime::CompletionConfig::default(), 0)
        .await;

    // --- read the off-agent artifact back ---
    let rows = ssb
        .query_by_type(SsbActionType::RouteDecisionEvent)
        .expect("query_by_type");

    let mut found_cloud_chosen = false;
    for entry in &rows {
        if let Ok(SsbPayload::RouteDecisionEvent(p)) =
            rmp_serde::from_slice::<SsbPayload>(&entry.payload)
        {
            if p.kind == RouteDecisionEventKind::CloudRouteChosen {
                found_cloud_chosen = true;
                assert!(
                    !p.reason.is_empty(),
                    "CloudRouteChosen reason must be non-empty, got empty string"
                );
            }
        }
    }
    assert!(
        found_cloud_chosen,
        "expected a RouteDecisionEvent CloudRouteChosen off-agent SSB row from try_complete, \
         found none in {} route_decision_event rows",
        rows.len()
    );
}
