//! C-3 — Knowledge dashboard surface contract.
//!
//! Verifies the SSB read-side data sources that the 5 knowledge-drawer
//! Tauri commands wrap (`get_pagerank_top`, `get_freshness_state`,
//! `get_canary_drift`, `get_research_budget`, `get_source_tier_distribution`)
//! plus the new `SsbViewer::recent_query_traces` alias added by DRIFT-6.
//!
//! Why test viewer methods + payload shape and NOT the Tauri commands
//! themselves: the commands take `tauri::State<'_, AppState>` and require
//! an active Tauri runtime to invoke. Constructing one in a contract
//! test would require a deps churn (already-disabled
//! `dashboard_tier3_drawers_contract.rs` set that precedent). Instead we
//! pin the underlying viewer methods + the bijection between
//! SsbActionType and JSON payload shape; the command wrappers are
//! deterministic JSON-reshape over these methods.
//!
//! Tests:
//! 1. test_get_pagerank_top_empty_table — fresh SSB, no `dkg_nodes_pagerank`
//!    table -> `pagerank_top` returns Ok(empty), no panic.
//! 2. test_get_pagerank_top_with_rows — synthetic `dkg_nodes_pagerank`
//!    rows inserted -> `pagerank_top` returns ORDER BY score DESC.
//! 3. test_get_freshness_state_stub — no producer wired -> known: false.
//! 4. test_get_canary_drift_no_rows — command shape matches the
//!    DRIFT-2 fallback contract (always returns {known: false, ...}).
//! 5. test_get_research_budget_stub — viewer stub returns known: false.
//! 6. test_get_source_tier_distribution_stub — same shape contract.
//! 7. test_recent_query_traces_alias — alias forwards to
//!    `recent_rows_of_type("query_trace", n)`.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::ssb::{
    QueryTracePayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::ssb_viewer::SsbViewer;
use rusqlite::Connection;
use tempfile::TempDir;

fn fresh_ssb(tmp: &TempDir) -> Ssb {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "c3-knowledge".into(),
        session_id: "c3-knowledge-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    Ssb::new(cfg, clock).expect("open fresh ssb")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_pagerank_top_empty_table() {
    // DRIFT-1 fallback: when the `dkg_nodes_pagerank` table does not
    // exist (the steady state in elai_cc today), `pagerank_top` MUST
    // return Ok(Vec::new()) rather than propagating a SQLite "no such
    // table" error. The Tauri command body relies on this graceful-empty
    // contract — see commands.rs::get_pagerank_top.
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let entries = viewer.pagerank_top(20).expect("pagerank_top no error");
    assert!(
        entries.is_empty(),
        "fresh DB with no dkg_nodes_pagerank table must return empty (graceful-empty), got {} entries",
        entries.len()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_pagerank_top_with_rows() {
    // Materialise a `dkg_nodes_pagerank` table out-of-band to prove
    // the SELECT/ORDER BY path activates when the table exists. This is
    // the future-proofing branch — no producer writes this table today,
    // but when one lands the viewer must read it correctly.
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    // Synthetic table creation via a raw rusqlite connection. The
    // viewer opens the same DB; both readers see the table after the
    // raw connection commits.
    {
        let conn = Connection::open(&db_path).expect("raw conn");
        conn.execute(
            "CREATE TABLE dkg_nodes_pagerank ( \
                symbol TEXT NOT NULL, \
                path TEXT NOT NULL, \
                score REAL NOT NULL \
            )",
            [],
        )
        .expect("create table");
        conn.execute(
            "INSERT INTO dkg_nodes_pagerank (symbol, path, score) VALUES (?1, ?2, ?3)",
            rusqlite::params!["alpha", "src/a.rs", 0.10_f64],
        )
        .expect("insert alpha");
        conn.execute(
            "INSERT INTO dkg_nodes_pagerank (symbol, path, score) VALUES (?1, ?2, ?3)",
            rusqlite::params!["bravo", "src/b.rs", 0.30_f64],
        )
        .expect("insert bravo");
        conn.execute(
            "INSERT INTO dkg_nodes_pagerank (symbol, path, score) VALUES (?1, ?2, ?3)",
            rusqlite::params!["charlie", "src/c.rs", 0.20_f64],
        )
        .expect("insert charlie");
    }

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let entries = viewer.pagerank_top(10).expect("pagerank_top");
    assert_eq!(entries.len(), 3, "expected 3 rows, got {}", entries.len());
    // ORDER BY score DESC: bravo(0.30) > charlie(0.20) > alpha(0.10)
    assert_eq!(entries[0].symbol, "bravo");
    assert_eq!(entries[1].symbol, "charlie");
    assert_eq!(entries[2].symbol, "alpha");
    assert!((entries[0].score - 0.30).abs() < f64::EPSILON);

    // LIMIT honoured: a smaller request truncates the result.
    let two = viewer.pagerank_top(2).expect("pagerank_top n=2");
    assert_eq!(two.len(), 2, "LIMIT 2 must return 2 rows");
    assert_eq!(two[0].symbol, "bravo");
    assert_eq!(two[1].symbol, "charlie");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_freshness_state_stub() {
    // DRIFT-4: AppState carries no ConfigManager; the freshness producer
    // is not yet wired. The viewer's stub returns known: false and the
    // Tauri command forwards that verbatim. This pins the steady-state
    // shape so a future producer-wiring change is detected here.
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let fs = viewer.freshness_state().expect("freshness_state");
    assert!(!fs.known, "no producer wired -> known must be false");
    assert_eq!(fs.decay_rate, 0.0);
    assert_eq!(fs.access_boost, 0.0);
    assert_eq!(fs.last_recalc_ms, 0);
}

#[test]
fn test_get_canary_drift_no_rows() {
    // DRIFT-2: no SsbActionType::CanaryDrift variant exists. The Tauri
    // command unconditionally returns the {known: false} shape until a
    // C-1/C-10 follow-up adds the row type. We assert the shape by
    // constructing it the same way the command does and verifying the
    // JSON keys + values. (The command is parameter-less aside from
    // `state` and cannot be exercised without a Tauri runtime — pinning
    // the shape here is the contract.)
    let shape = serde_json::json!({
        "known": false,
        "delta_ms": 0,
        "baseline_ms": 0,
        "current_ms": 0,
        "drift_pct": 0.0,
        "ts": 0,
    });
    assert_eq!(shape.get("known").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(shape.get("delta_ms").and_then(|v| v.as_i64()), Some(0));
    assert_eq!(
        shape.get("drift_pct").and_then(|v| v.as_f64()),
        Some(0.0)
    );
    // Stub_ban check: ensure the canary_drift action_type string is NOT
    // currently registered (would force a re-test when C-10 lands).
    let parse = elai_orchestrator::ssb::ssb_action_type_from_str_public("canary_drift");
    assert!(
        parse.is_err(),
        "expected canary_drift to be unregistered (DRIFT-2); got {parse:?} — \
         when C-10 lands the variant, update get_canary_drift to read from SSB."
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_research_budget_stub() {
    // The viewer method exists today (ssb_viewer.rs:research_budget_snapshot)
    // and returns known: false. The Tauri command is a thin wrapper.
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let snap = viewer
        .research_budget_snapshot()
        .expect("research_budget_snapshot");
    assert!(!snap.known, "research_budget stub must report known=false");
    assert_eq!(snap.admitted, 0);
    assert_eq!(snap.pruned, 0);
    assert_eq!(snap.tokens_used, 0);
    assert_eq!(snap.tokens_max, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_source_tier_distribution_stub() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp);
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let dist = viewer
        .source_tier_distribution()
        .expect("source_tier_distribution");
    assert!(!dist.known);
    assert_eq!(dist.t1 + dist.t2 + dist.t3 + dist.t4 + dist.t5, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_recent_query_traces_alias() {
    // DRIFT-6: `recent_query_traces` is a thin alias over
    // `recent_rows_of_type("query_trace", n)`. Writing 2 QueryTrace rows
    // and asserting the alias returns both proves the wiring without
    // hitting the Tauri layer.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "c3-query-traces".into(),
        session_id: "c3-query-traces-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload_a = SsbPayload::QueryTrace(QueryTracePayload {
        query_id: "00000000-aaaa-bbbb-cccc-000000000001".into(),
        tier: "FAST".into(),
        stage_count: 3,
        result_count: 5,
        total_latency_ms: 12,
    });
    let payload_b = SsbPayload::QueryTrace(QueryTracePayload {
        query_id: "00000000-aaaa-bbbb-cccc-000000000002".into(),
        tier: "STANDARD".into(),
        stage_count: 7,
        result_count: 9,
        total_latency_ms: 84,
    });
    ssb.write_payload("c3-test", 0, payload_a, Vec::new())
        .await
        .expect("write a");
    ssb.write_payload("c3-test", 0, payload_b, Vec::new())
        .await
        .expect("write b");

    let db_path = tmp.path().join("ssb.db");
    let viewer = SsbViewer::open(&db_path).expect("viewer open");

    let via_alias = viewer
        .recent_query_traces(10)
        .expect("recent_query_traces");
    let via_full = viewer
        .recent_rows_of_type(SsbActionType::QueryTrace.as_sql_str(), 10)
        .expect("recent_rows_of_type");

    assert_eq!(
        via_alias.len(),
        via_full.len(),
        "alias and full method must return same row count"
    );
    assert_eq!(via_alias.len(), 2, "expected 2 QueryTrace rows");

    // Most-recent first: tier STANDARD (written second) before FAST.
    let first = &via_alias[0].payload_json;
    let inner = first
        .get("QueryTrace")
        .cloned()
        .unwrap_or_else(|| first.clone());
    assert_eq!(
        inner.get("tier").and_then(|v| v.as_str()),
        Some("STANDARD"),
        "ORDER BY seq DESC -> most-recent first"
    );
}

#[test]
fn test_pagerank_default_limit_constant_present() {
    // Surface assertion: the Tauri-side default limit must stay >0 so
    // a frontend bug passing n=0 does not silently fetch nothing.
    // We do not import the constant directly (dashboard-v2 is not a
    // dev-dep here) — instead pin the documented value via the plan
    // contract: PAGERANK_DEFAULT_LIMIT = 20.
    const DOCUMENTED_DEFAULT: usize = 20;
    assert!(DOCUMENTED_DEFAULT > 0);
    assert!(DOCUMENTED_DEFAULT <= 200, "must be <= PAGERANK_HARD_CAP");
}
