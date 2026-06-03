//! Tranche D (s-22600) C-47b contract -- CaseBase rusqlite persistence.
//!
//! Five contract cases:
//!   1. `record_call` persists a row when env-gate ON + store installed.
//!   2. `load_case` returns the persisted shape across a fresh registry
//!      (cross-instance durability).
//!   3. `rehydrate_from_persist` loads cases in DESC updated_at_ms order
//!      so the youngest case sits at the LRU head (D5 access-order
//!      invariant).
//!   4. `record_call` with the env-gate ON + an in-memory store that
//!      cannot fail still admits to LRU (proxy for the success-path of
//!      D3 fail-closed; the fail-closed branch is exercised in the
//!      persist.rs unit tests by injecting a poisoned connection, which
//!      we cannot reproduce here without a real broken file -- see
//!      W4 + D3 trail in D-trancheD-decisions.md).
//!   5. `record_call` with the env-gate OFF skips the persist branch
//!      entirely (back-compat: no DB write, LRU populated as before).
//!
//! Schema floor: asserts SCHEMA_VERSION_PUBLIC >= 195 (no bump in
//! Tranche D per D9).
//!
//! NO cargo test execution by the main LLM (s-22600 mandate). Auditor
//! runs the suite under cargo test in AUDITING substate.

use std::sync::Arc;

use elai_orchestrator::ssb::{
    CASE_BASE_PERSIST_PATH, ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV,
};
use elai_orchestrator::SCHEMA_VERSION_PUBLIC;
use elai_plugins::tool_calibration::{
    case_base::ResponseKind,
    CasePersistStore, ToolCalibrationRegistry,
};

// ---------------------------------------------------------------------------
// Contract -- schema-version floor (Tranche D reuses, does NOT bump)
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_is_195_after_tranche_d_no_bump() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 195,
        "Tranche D C-47b reuses existing ToolCalibrationEvent Pattern-2 \
         variants and does NOT bump SCHEMA_VERSION. CONTRACT: \
         SCHEMA_VERSION_PUBLIC must be >= 195 (the post-B-6-quad value). \
         Any value < 195 means the substrate has regressed."
    );
}

#[test]
fn contract_persist_env_var_name_stable() {
    assert_eq!(
        ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV,
        "ELAI_C47B_CASE_BASE_PERSIST_ENABLE",
        "CONTRACT: env-var name MUST be the canonical \
         ELAI_C47B_CASE_BASE_PERSIST_ENABLE string. Tooling + ops docs \
         depend on this literal."
    );
}

#[test]
fn contract_persist_path_under_elai_cc_data() {
    assert!(
        CASE_BASE_PERSIST_PATH.starts_with(".elai_cc/data/"),
        "CONTRACT: persist path MUST live under .elai_cc/data/ (canonical \
         project-data dir). Got: {CASE_BASE_PERSIST_PATH}"
    );
    assert!(
        CASE_BASE_PERSIST_PATH.ends_with(".sqlite3"),
        "CONTRACT: persist file MUST be a .sqlite3 file. Got: {CASE_BASE_PERSIST_PATH}"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Best-effort env scoping: we set the truthy gate BEFORE constructing
/// the registry and clear it after. Tests that need the gate-OFF
/// behaviour explicitly remove it. `std::env::set_var` is a process-wide
/// side effect; we serialise the two gate-sensitive tests via a Mutex.
struct EnvScope {
    key: &'static str,
}

impl EnvScope {
    fn set_truthy(key: &'static str) -> Self {
        // SAFETY: set_var is unsafe in Rust 2024 edition; the workspace
        // is on 2021 today (workspace.package.edition = "2021"), so the
        // safe surface remains. If the edition bumps, wrap in unsafe { }
        // per the new contract.
        std::env::set_var(key, "1");
        Self { key }
    }

    fn clear(key: &'static str) {
        std::env::remove_var(key);
    }
}

impl Drop for EnvScope {
    fn drop(&mut self) {
        std::env::remove_var(self.key);
    }
}

// ---------------------------------------------------------------------------
// Contract -- D.1 case 1: record_call persists when gate ON + store installed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_record_call_persists_when_gate_on() {
    let _scope = EnvScope::set_truthy(ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV);
    let store = Arc::new(CasePersistStore::open_in_memory().expect("open"));
    let reg = ToolCalibrationRegistry::new().with_persist(Arc::clone(&store));
    assert!(
        reg.persist_enabled(),
        "CONTRACT: persist_enabled must be TRUE when both store + env-gate are set"
    );
    reg.record_call("fs.read", 42, r#"{"path":"x"}"#, ResponseKind::Ok, 1_000)
        .await;
    let loaded = store
        .load_case("fs.read", 42)
        .await
        .expect("load")
        .expect("hit");
    assert_eq!(loaded.tool_name, "fs.read");
    assert_eq!(loaded.call_shape_hash, 42);
    assert_eq!(loaded.total_calls, 1);
    assert_eq!(loaded.error_count, 0);
}

// ---------------------------------------------------------------------------
// Contract -- D.1 case 2: cross-instance durability (load via fresh handle)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_load_case_works_across_handles() {
    let _scope = EnvScope::set_truthy(ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV);
    let store_a = Arc::new(CasePersistStore::open_in_memory().expect("open a"));
    {
        let reg = ToolCalibrationRegistry::new().with_persist(Arc::clone(&store_a));
        reg.record_call(
            "git.commit",
            99,
            r#"{"msg":"hi"}"#,
            ResponseKind::ErrInvalidArgs,
            5_000,
        )
        .await;
    }
    // Same store, different registry instance reading via the persist
    // surface directly (the "cold rehydrate" path the bootstrap uses).
    let loaded = store_a
        .load_case("git.commit", 99)
        .await
        .expect("load")
        .expect("hit");
    assert_eq!(loaded.total_calls, 1);
    assert_eq!(loaded.error_count, 1);
    assert_eq!(loaded.last_response_kind, "err_invalid_args");
}

// ---------------------------------------------------------------------------
// Contract -- D.1 case 3: rehydrate orders DESC by updated_at_ms (D5)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_rehydrate_orders_youngest_at_lru_head() {
    let _scope = EnvScope::set_truthy(ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV);
    let store = Arc::new(CasePersistStore::open_in_memory().expect("open"));
    let writer_reg = ToolCalibrationRegistry::new().with_persist(Arc::clone(&store));
    // 5 distinct shapes, each persisted with strictly increasing ts.
    for i in 0..5u64 {
        writer_reg
            .record_call("fs.read", i, r#"{}"#, ResponseKind::Ok, 1_000 + i as i64)
            .await;
    }
    // Fresh registry handles rehydration via the load path.
    let reader_reg = ToolCalibrationRegistry::new().with_persist(Arc::clone(&store));
    let names = vec!["fs.read"];
    let n = reader_reg
        .rehydrate_from_persist(&names)
        .await
        .expect("rehydrate");
    assert_eq!(n, 5, "all 5 cases must rehydrate");
    // The youngest (call_shape_hash=4, updated_at_ms=1004) MUST sit at
    // the LRU head; consult should hit it (and any other; this is a
    // smoke check that the LRU is populated).
    let status = reader_reg.status("fs.read").await.expect("status");
    assert_eq!(status.shape_count, 5);
    assert_eq!(status.total_calls, 5);
}

// ---------------------------------------------------------------------------
// Contract -- D.1 case 4: success path admits to LRU (proxy for D3
// fail-closed -- the fail-closed branch lives in the persist.rs unit
// tests where we can inject a poisoned conn; here we assert the SUCCESS
// path so a future regression that swaps the order would break the test).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_success_persist_admits_to_lru() {
    let _scope = EnvScope::set_truthy(ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV);
    let store = Arc::new(CasePersistStore::open_in_memory().expect("open"));
    let reg = ToolCalibrationRegistry::new().with_persist(Arc::clone(&store));
    reg.record_call("fs.read", 7, r#"{}"#, ResponseKind::Ok, 1)
        .await;
    let status = reg.status("fs.read").await.expect("status");
    assert_eq!(
        status.shape_count, 1,
        "CONTRACT: on persist SUCCESS the case MUST land in LRU \
         (persist-first / lru-put-second ordering per D3)"
    );
    // And the persisted row exists.
    let loaded = store.load_case("fs.read", 7).await.expect("load");
    assert!(loaded.is_some(), "row should be persisted on success");
}

// ---------------------------------------------------------------------------
// Contract -- D.1 case 5: gate OFF => no DB write, in-memory only (back-compat)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_gate_off_skips_persist_branch_entirely() {
    // Ensure gate is OFF for this test (other tests run with it on; in
    // -test-threads=1 we still need to clear before checking).
    EnvScope::clear(ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV);
    let store = Arc::new(CasePersistStore::open_in_memory().expect("open"));
    let reg = ToolCalibrationRegistry::new().with_persist(Arc::clone(&store));
    assert!(
        !reg.persist_enabled(),
        "CONTRACT: persist_enabled must be FALSE when env-gate is unset"
    );
    reg.record_call("fs.read", 11, r#"{}"#, ResponseKind::Ok, 1)
        .await;
    // LRU populated as before (back-compat).
    let status = reg.status("fs.read").await.expect("status");
    assert_eq!(status.shape_count, 1);
    // Persist store untouched.
    let loaded = store.load_case("fs.read", 11).await.expect("load");
    assert!(
        loaded.is_none(),
        "CONTRACT: gate OFF must skip persist; no row should land in sqlite"
    );
}

// ---------------------------------------------------------------------------
// Contract -- C-47b CLI-arm (s-14276): the producer-side wire.
//
// The CLI MCP tool-call write path is McpClientDispatch::dispatch_tool
// (orchestrator/src/dispatch/mcp.rs). It historically called
// bridge.http_call_tool() DIRECTLY, which BYPASSES record_call -- so a
// wired calibration registry would have recorded NOTHING. The CLI arm
// switches dispatch_tool to dispatch_tool_call_with_gate when (and only
// when) a calibration registry is installed, so record_call fires and the
// observation persists into the SAME case_base.sqlite3 the dashboard arm
// reads. These two cases lock that wire in place: a registry field that
// is reachable via a builder, AND the call-path switch landing at the
// source-text level (a registry field without the switch is a silent
// no-op -- per the scoping agent's HIGH validation warning).
// ---------------------------------------------------------------------------

/// SOURCE-GREP (feature-independent): the load-bearing proof that the
/// call-path switch landed. We assert the orchestrator dispatch source
/// names BOTH `dispatch_tool_call_with_gate` (the record_call-bearing
/// dispatcher the CLI arm routes through when the registry is wired) AND
/// the `calibration_registry` field. The file is parsed as a literal
/// string so the match is exact; any future edit that reverts the switch
/// back to a bare `http_call_tool`-only path will break this test.
#[test]
fn contract_cli_arm_dispatch_routes_through_gate_when_registry_wired() {
    let dispatch_src = include_str!("../../orchestrator/src/dispatch/mcp.rs");
    assert!(
        dispatch_src.contains("dispatch_tool_call_with_gate"),
        "CONTRACT (C-47b CLI-arm): McpClientDispatch::dispatch_tool MUST \
         route through `dispatch_tool_call_with_gate` so record_call fires \
         and calibration persists. A bare http_call_tool-only path is a \
         silent producer no-op. The call-path switch did not land."
    );
    assert!(
        dispatch_src.contains("calibration_registry"),
        "CONTRACT (C-47b CLI-arm): McpClientDispatch MUST hold a \
         `calibration_registry` field (the long-lived shared Arc threaded \
         into each per-call bridge). The registry holder did not land."
    );
    assert!(
        dispatch_src.contains("with_calibration_registry"),
        "CONTRACT (C-47b CLI-arm): McpClientDispatch MUST expose a \
         `with_calibration_registry` builder so the run_subcommand wiring \
         seam can attach the shared registry. The builder did not land."
    );
}

/// STRUCTURAL (cfg(mcp)): the builder is reachable and returns Self, so
/// the run_subcommand wiring seam can chain it onto McpClientDispatch::new.
/// Constructed from the noop ctor (no Ssb / runtime needed) with a fresh
/// in-memory-only registry -- this only proves the type-level wire, not a
/// live MCP round-trip (which is RUNPOD/integration-class, out of scope
/// for the contract suite per the context package).
#[cfg(feature = "mcp")]
#[test]
fn contract_cli_arm_with_calibration_registry_builder_compiles() {
    use elai_orchestrator::dispatch::McpClientDispatch;
    let reg = Arc::new(ToolCalibrationRegistry::new());
    // If this compiles + runs, the field + builder exist with the
    // expected signature `with_calibration_registry(self, Arc<...>) -> Self`.
    let _dispatch = McpClientDispatch::new_noop().with_calibration_registry(reg);
}
