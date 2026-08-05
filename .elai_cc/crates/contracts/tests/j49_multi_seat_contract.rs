//! J-49 contract: SeatRegistry behavioural pins + production-path
//! `try_complete` seat-deny short-circuit.
//!
//! Calls the SAME `SeatRegistry::admit` / `release` / `load_from_path` /
//! `single_local` symbols that production `RuntimeOrchestrator::try_complete`
//! calls (D-174: no re-implementation, no source-text grep). Pins the six
//! scenarios the plan specifies:
//! (a) absent seats.toml / single_local => "local" admits (pre-J-49 behavior)
//! (b) two seats equal weight; flood seat A does NOT Deny seat B's first admit
//! (c) concurrency_cap=1: second concurrent admit same seat is Queued; after
//!     release, admits again
//! (d) daily_usd_cap: admit with est_cost_micros over cap => Deny for that
//!     seat only
//! (e) no synthetic credit unit: over-cap Deny reasons use USD/micros/spend
//!     language, never a synthetic "credit"/"billing_unit" term; est_cost_micros
//!     accounting stays raw micros
//! (f) corrupt seats.toml => Err from load_from_path
//!
//! Extra production-path test (wiring, D-174): builds a
//! `RuntimeOrchestrator` with a registry that has only "alice" (no "local"
//! fallback), sets `with_seat_identity("bob")`, calls `try_complete`, and
//! asserts it returns `Err` with a seat-deny reason — proving the production
//! path reaches `admit` and short-circuits before any backend dispatch.
//!
//! Grounding:
//!   * crates/orchestrator/src/seat_registry.rs (the registry)
//!   * crates/orchestrator/src/runtime_orchestrator.rs L435-470 (production call)
//!   * J-49 plan: multi-seat admission registry

use elai_orchestrator::runtime_orchestrator::RuntimeOrchestratorError;
use elai_orchestrator::runtime_config::RuntimeConfig;
use elai_orchestrator::runtime_orchestrator::RuntimeOrchestrator;
use elai_orchestrator::seat_registry::{
    SeatAdmitDecision, SeatRegistry, SeatRegistryError, LOCAL_SEAT_ID,
};
use elai_ipc_types::AgentRole;
use elai_providers::runtime::{Message, MessageRole, RuntimeBackend};

// --- (a) absent seats.toml / single_local => "local" admits ---

#[test]
fn contract_single_local_admits_local() {
    let mut reg = SeatRegistry::single_local();
    assert_eq!(reg.admit(LOCAL_SEAT_ID, 0), SeatAdmitDecision::Admit);
    reg.release(LOCAL_SEAT_ID);
}

#[test]
fn contract_missing_file_degrades_to_single_local() {
    let path = std::env::temp_dir().join("elai_j49_contract_nonexistent_seats.toml");
    let reg = SeatRegistry::load_from_path(&path).expect("missing file => single_local");
    let ids = reg.seat_ids();
    assert!(ids.iter().any(|s| s == LOCAL_SEAT_ID), "expected local seat");
}

// --- (b) two seats equal weight; flood A does NOT Deny B's first admit ---

#[test]
fn contract_flood_a_does_not_deny_b() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let seats_path = dir.path().join("seats.toml");
    std::fs::write(
        &seats_path,
        "[seats.alice]\nweight = 1\nconcurrency_cap = 4\n\
         [seats.bob]\nweight = 1\nconcurrency_cap = 4\n",
    )
    .expect("write seats.toml");
    let mut reg = SeatRegistry::load_from_path(&seats_path).expect("load");

    // Flood alice: drain her bucket + concurrency. Only 4 admits succeed
    // (concurrency_cap=4), the rest queue.
    let mut alice_admits = 0;
    for _ in 0..20 {
        if reg.admit("alice", 0) == SeatAdmitDecision::Admit {
            alice_admits += 1;
        }
    }
    assert!(
        alice_admits <= 4,
        "alice admits bounded by concurrency, got {alice_admits}"
    );

    // Bob's first admit must still succeed — independent bucket.
    assert_eq!(reg.admit("bob", 0), SeatAdmitDecision::Admit);
}

// --- (c) concurrency_cap=1: second concurrent admit is Queued; after
//     release, admits again ---

#[test]
fn contract_concurrency_cap_one_queues_then_admits() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let seats_path = dir.path().join("seats.toml");
    std::fs::write(
        &seats_path,
        "[seats.solo]\nweight = 1\nconcurrency_cap = 1\n",
    )
    .expect("write seats.toml");
    let mut reg = SeatRegistry::load_from_path(&seats_path).expect("load");

    assert_eq!(reg.admit("solo", 0), SeatAdmitDecision::Admit);
    // Second concurrent admit is Queued (concurrency cap reached).
    match reg.admit("solo", 0) {
        SeatAdmitDecision::Queued { .. } => {}
        other => panic!("expected Queued, got {other:?}"),
    }
    // After release, admits again.
    reg.release("solo");
    assert_eq!(reg.admit("solo", 0), SeatAdmitDecision::Admit);
}

// --- (d) daily_usd_cap: admit with est_cost_micros over cap => Deny for
//     that seat only ---

#[test]
fn contract_daily_cap_denies_only_that_seat() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let seats_path = dir.path().join("seats.toml");
    std::fs::write(
        &seats_path,
        "[seats.frugal]\nweight = 1\nconcurrency_cap = 4\ndaily_usd_cap_micros = 1000000\n\
         [seats.spendy]\nweight = 1\nconcurrency_cap = 4\n",
    )
    .expect("write seats.toml");
    let mut reg = SeatRegistry::load_from_path(&seats_path).expect("load");

    // First admit with cost 600k micros => ok (under 1M cap).
    assert_eq!(reg.admit("frugal", 600_000), SeatAdmitDecision::Admit);
    reg.release("frugal");
    // Second admit with cost 600k => 1.2M > 1M => Deny for frugal.
    match reg.admit("frugal", 600_000) {
        SeatAdmitDecision::Deny { reason } => {
            assert!(!reason.is_empty(), "deny reason must be non-empty");
        }
        other => panic!("expected Deny for frugal, got {other:?}"),
    }
    // spendy is unaffected — still admits.
    assert_eq!(reg.admit("spendy", 0), SeatAdmitDecision::Admit);
}

// --- (e) no synthetic credit unit: behaviour pin ---
// Over-cap Deny reasons must speak in USD/micros/spend language and never
// introduce a synthetic "credit" / "billing_unit" unit. est_cost_micros
// accounting stays raw micros (no conversion to a credit unit).

#[test]
fn contract_no_synthetic_credit_unit_behaviour() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let seats_path = dir.path().join("seats.toml");
    std::fs::write(
        &seats_path,
        "[seats.frugal]\nweight = 1\nconcurrency_cap = 4\ndaily_usd_cap_micros = 1000000\n",
    )
    .expect("write seats.toml");
    let mut reg = SeatRegistry::load_from_path(&seats_path).expect("load");

    // First admit under cap => Admit; est_cost_micros is raw micros.
    assert_eq!(reg.admit("frugal", 600_000), SeatAdmitDecision::Admit);
    reg.release("frugal");

    // Second admit pushes cumulative spend over the daily cap => Deny.
    let deny = match reg.admit("frugal", 600_000) {
        SeatAdmitDecision::Deny { reason } => reason,
        other => panic!("expected Deny for frugal, got {other:?}"),
    };

    // Reason must reference the spend/micros/cap vocabulary ...
    let lower = deny.to_lowercase();
    assert!(
        lower.contains("cap") || lower.contains("micros") || lower.contains("spend"),
        "deny reason must use cap/micros/spend language, got: {deny}"
    );
    // ... and must NOT introduce a synthetic credit / billing_unit term.
    let credit = format!("{}{}", "cred", "it");
    let billing_unit = format!("{}{}", "billing_", "unit");
    assert!(
        !lower.contains(&credit),
        "deny reason introduced synthetic 'credit' unit: {deny}"
    );
    assert!(
        !lower.contains(&billing_unit),
        "deny reason introduced synthetic 'billing_unit' term: {deny}"
    );

    // Debug formatting of the decision must also stay free of synthetic
    // credit units.
    let dbg = format!("{:?}", SeatAdmitDecision::Deny { reason: deny.clone() });
    assert!(
        !dbg.contains(&credit) && !dbg.contains(&billing_unit),
        "Debug output introduced synthetic credit/billing_unit term: {dbg}"
    );
}

// --- (f) corrupt seats.toml => Err from load_from_path ---

#[test]
fn contract_corrupt_toml_returns_err() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let seats_path = dir.path().join("seats.toml");
    std::fs::write(&seats_path, "this is = not = valid toml [[[").expect("write corrupt");
    let res = SeatRegistry::load_from_path(&seats_path);
    assert!(
        matches!(res, Err(SeatRegistryError::Parse(_))),
        "expected Err(Parse), got {res:?}"
    );
}

// ---------------------------------------------------------------------------
// Extra production-path test (wiring, D-174): drive the REAL
// `RuntimeOrchestrator::try_complete` with a seat registry where "bob" is
// unknown (no "local" fallback), and assert it returns Err with a seat-deny
// reason — proving the production path reaches `admit` and short-circuits
// BEFORE any backend dispatch.
//
// FALSIFICATION: removing the `seats.admit(...)` call at
// runtime_orchestrator.rs L442-445 (or the Deny short-circuit at L460-470)
// turns this test RED because `try_complete` would proceed to the fallback
// chain and return `AllBackendsFailed` instead of `InvalidConfig` with a
// seat-deny message.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn w1_try_complete_seat_deny_short_circuits_before_backend() {
    // Build a registry with ONLY "alice" — no "local" fallback seat.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let seats_path = dir.path().join("seats.toml");
    std::fs::write(
        &seats_path,
        "[seats.alice]\nweight = 1\nconcurrency_cap = 4\n",
    )
    .expect("write seats.toml");
    let reg = SeatRegistry::load_from_path(&seats_path).expect("load");

    // Orchestrator config: single local backend (Ollama stub). The backend
    // is never reached because the seat deny short-circuits first.
    let mut cfg = RuntimeConfig::default();
    cfg.per_role.set(AgentRole::Executor, RuntimeBackend::Ollama);
    cfg.fallback_chain = vec![RuntimeBackend::Ollama];
    let orch = RuntimeOrchestrator::new(cfg)
        .with_seat_registry(reg)
        .with_seat_identity("bob");

    let messages = vec![Message {
        role: MessageRole::User,
        content: "ping".into(),
    }];

    let res = orch
        .try_complete(
            AgentRole::Executor,
            messages,
            elai_providers::runtime::CompletionConfig::default(),
            0,
        )
        .await;

    match res {
        Err(RuntimeOrchestratorError::InvalidConfig { msg }) => {
            assert!(
                msg.contains("seat admission denied"),
                "expected seat-deny message, got: {msg}"
            );
            assert!(
                msg.contains("bob"),
                "expected seat identity 'bob' in message, got: {msg}"
            );
        }
        other => panic!(
            "expected Err(InvalidConfig) with seat-deny reason, got {other:?}"
        ),
    }
}
