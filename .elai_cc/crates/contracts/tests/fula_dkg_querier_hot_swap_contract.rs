//! FUL-A.1 — DkgQuerierAdapter hot-swap contract.
//!
//! Pins the behaviour of the new `HotSwappableDkgQuerier` wrapper and the
//! `dkg_querier_swap` slot on `TaskRunnerConfig` / `ResolvedConfig`:
//!
//! 1. `HotSwappableDkgQuerier` is `DkgQuerier + Send + Sync`, dyn-safe, and
//!    reads through an `Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>>`.
//! 2. A fresh project starts with `EmptyDkgQuerier` (inner returns zero
//!    reachability matches).
//! 3. After the swap, the inner is replaced and reachability queries see the
//!    new querier's results.
//! 4. `DkgQuerierSwappedPayload` round-trips through rmp_serde and is listed
//!    on `SsbActionType::DkgQuerierSwapped`.
//! 5. `TaskRunnerConfig::dkg_querier_swap` defaults to `None` and is
//!    preserved by `new_minimal` — no existing caller is forced to update.
//! 6. Concurrent read-path access during a write-swap does not deadlock
//!    (timed probe — 200ms upper bound).

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use elai_core::GitBranchPolicy;
use elai_orchestrator::ssb::{DkgQuerierSwappedPayload, SsbActionType, SsbPayload};
use elai_orchestrator::TaskRunnerConfig;
use elai_rules::predicate::{
    DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams,
};
use elai_rules::stubs::{EmptyDkgQuerier, HotSwappableDkgQuerier};

fn sample_params() -> ReachabilityParams {
    ReachabilityParams {
        source_pattern: "src%".to_string(),
        sink_pattern: "sink%".to_string(),
        must_traverse: None,
        must_not_exist: false,
        max_depth: 3,
        edge_types: vec![],
    }
}

// ---------------------------------------------------------------------------
// Contract 1: HotSwappableDkgQuerier is dyn-safe as Arc<dyn DkgQuerier + Send + Sync>.
// ---------------------------------------------------------------------------

#[test]
fn hot_swappable_is_dyn_safe_send_sync() {
    let inner: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(EmptyDkgQuerier)));
    let wrapper = HotSwappableDkgQuerier::new(inner.clone());
    let _obj: Arc<dyn DkgQuerier + Send + Sync> = Arc::new(wrapper);
    // Compile-time proof that the trait object accepts Send + Sync bounds.
}

// ---------------------------------------------------------------------------
// Contract 2: Empty default returns empty reachability (pre-swap semantics).
// ---------------------------------------------------------------------------

#[test]
fn empty_default_returns_empty_reachability() {
    let inner: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(EmptyDkgQuerier)));
    let wrapper = HotSwappableDkgQuerier::new(inner);
    let params = sample_params();
    let matches = wrapper
        .query_reachability(&params)
        .expect("empty querier must not error");
    assert!(matches.is_empty(), "EmptyDkgQuerier returns no matches");
}

// ---------------------------------------------------------------------------
// Contract 3: After swap, reachability queries see the new querier.
// ---------------------------------------------------------------------------

struct FakeAdapter {
    symbol: String,
}

impl DkgQuerier for FakeAdapter {
    fn query_reachability(
        &self,
        _params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(vec![ReachabilityMatch {
            path: vec![self.symbol.clone()],
            edge_labels: vec!["calls".to_string()],
        }])
    }
    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

#[test]
fn swap_replaces_inner_querier() {
    let inner: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(EmptyDkgQuerier)));
    let wrapper = HotSwappableDkgQuerier::new(inner.clone());

    // Pre-swap: empty.
    let before = wrapper.query_reachability(&sample_params()).unwrap();
    assert!(before.is_empty());

    // Swap.
    wrapper
        .swap(Box::new(FakeAdapter {
            symbol: "adapter_sym".to_string(),
        }))
        .expect("swap must succeed");

    // Post-swap: adapter's result.
    let after = wrapper.query_reachability(&sample_params()).unwrap();
    assert_eq!(after.len(), 1, "after swap expect 1 match from FakeAdapter");
    assert_eq!(after[0].path, vec!["adapter_sym".to_string()]);
}

// ---------------------------------------------------------------------------
// Contract 4: DkgQuerierSwappedPayload msgpack round-trip + ActionType map.
// ---------------------------------------------------------------------------

#[test]
fn dkg_querier_swapped_payload_roundtrips() {
    let payload = DkgQuerierSwappedPayload {
        project_id: "proj-42".to_string(),
        from: "empty".to_string(),
        to: "adapter".to_string(),
    };
    let wire = SsbPayload::DkgQuerierSwapped(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wire).expect("serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    match decoded {
        SsbPayload::DkgQuerierSwapped(p) => {
            assert_eq!(p, payload);
        }
        other => panic!("expected DkgQuerierSwapped, got {other:?}"),
    }
    assert_eq!(
        SsbActionType::DkgQuerierSwapped.as_sql_str(),
        "dkg_querier_swapped"
    );
}

// ---------------------------------------------------------------------------
// Contract 5: TaskRunnerConfig::dkg_querier_swap defaults to None and
// new_minimal preserves it.
// ---------------------------------------------------------------------------

#[test]
fn task_runner_config_accepts_dkg_querier_swap() {
    use std::path::PathBuf;

    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/cursor.json"));
    assert!(cfg.dkg_querier_swap.is_none(), "default is None");

    let slot: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(EmptyDkgQuerier)));

    let populated = TaskRunnerConfig {
        cursor_path: PathBuf::from("/tmp/cursor.json"),
        dev_server_launcher: None,
        project_dir_resolver: None,
        dev_server_registry: None,
        web_research_runner: None,
        scaffold_applier: None,
        rule_evaluator: None,
        interrogation_store: None,
        checkpoint_scratch_dir: None,
        baby_step_executor: None,
        anti_spiral_enabled: false,
        llm_provider: None,
        backend_config: None,
        indexing_service: None,
        hook_dispatcher: None,
        signal_reader: None,
        tool_dispatcher: None,
        git_branch_policy: GitBranchPolicy::default(),
        system_prober: None,
        feasibility_prober: None,
        autonomous_gate: None,
        plan_interactive_gate: None,
        layer1a_processor: None,
        dispatch_context_profile: None,
        reflection_client: None,
        irreversible_classifier: None,
        dkg_querier_swap: Some(slot.clone()),
        task_splitter: None,
        research_tree: None,
        pipeline_grc: None,
        model_capability_profile: None,
        cloud_router: None,
        cloud_spend_cap_daily_usd: None,
        crystallizer: None,
        compiled_pipeline_store: None,
        background_task_manager: None,
        // F-6 / Slice A3: legacy contract fixture -- no learned-skill store.
        skill_store: None,
        // C-39: legacy contract fixture -- no Dispatch wired.
        mcp_dispatch: None,
        // C.5: legacy contract fixture -- no live tracker (sentinel fallback).
        token_tracker: None,
        // B-28 WIRING-2: legacy contract fixture -- no production buffer editor.
        buffer_editor: None,
        // E-46 D-8: contract fixture — module-constant defaults.
        planning_config: Default::default(),
    };
    assert!(populated.dkg_querier_swap.is_some());
}

// ---------------------------------------------------------------------------
// Contract 6: Concurrent read path does not deadlock during a swap.
// Timed probe — a RwLock::read during swap should complete under 200ms.
// ---------------------------------------------------------------------------

#[test]
fn concurrent_read_does_not_deadlock_during_swap() {
    let inner: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(EmptyDkgQuerier)));
    let wrapper = Arc::new(HotSwappableDkgQuerier::new(inner.clone()));

    // Spawn a reader loop.
    let reader_wrapper = wrapper.clone();
    let reader_done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let reader_done_sig = reader_done.clone();
    let h = thread::spawn(move || {
        let start = Instant::now();
        while !reader_done_sig.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = reader_wrapper.query_reachability(&sample_params());
            if start.elapsed() > Duration::from_secs(2) {
                break;
            }
        }
    });

    // Give the reader a moment to spin.
    thread::sleep(Duration::from_millis(10));

    // Swap should complete within 200ms even with the reader spinning.
    let swap_start = Instant::now();
    wrapper
        .swap(Box::new(FakeAdapter {
            symbol: "concurrent_swap".to_string(),
        }))
        .expect("swap under reader load");
    let elapsed = swap_start.elapsed();
    assert!(
        elapsed < Duration::from_millis(200),
        "swap took {elapsed:?}, expected <200ms — RwLock correctness in doubt"
    );

    reader_done.store(true, std::sync::atomic::Ordering::Relaxed);
    h.join().expect("reader join");
}

// ---------------------------------------------------------------------------
// Contract 7: ELAI_DISABLE_INDEXING path ⇒ swap slot unused.
// The dashboard construction code wires `indexing_service = None` when the env
// var is set. Without an IndexingService, the task_runner never hits the
// post-index hook, so the swap slot stays in its initial Empty state. We
// assert the structural invariant: a wrapper created with Empty remains Empty
// unless explicitly swapped.
// ---------------------------------------------------------------------------

#[test]
fn no_swap_without_explicit_trigger() {
    let inner: Arc<RwLock<Box<dyn DkgQuerier + Send + Sync>>> =
        Arc::new(RwLock::new(Box::new(EmptyDkgQuerier)));
    let wrapper = HotSwappableDkgQuerier::new(inner);
    for _ in 0..3 {
        let r = wrapper.query_reachability(&sample_params()).unwrap();
        assert!(r.is_empty(), "no swap was triggered — must stay empty");
    }
}
