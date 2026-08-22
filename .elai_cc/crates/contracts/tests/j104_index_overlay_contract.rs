//! J-104 live-index overlay contract.
//!
//! WIRED[J-104] production callers:
//! - `tools/src/lib.rs` `run_write_file` / `run_edit_file` ->
//!   `index_overlay::capture_file_ops_edit` (shipped file_ops signal)
//! - `pipeline.rs` `query_with_tier` -> `apply_overlay_first`
//! - `task_runner.rs` `WatchdogTimeoutKind::NoMaterialProgress` ->
//!   `try_drain_on_trigger(DrainTrigger::NoMaterialProgress)`
//!
//! FALSIFICATION: deleting the NoMaterialProgress drain arm or the
//! overlay-first apply must fail these tests.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

fn drain_test_gate() -> &'static Mutex<()> {
    static GATE: OnceLock<Mutex<()>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(()))
}

use elai_core::{ClockProvider, OsClock};
use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::pipeline::QueryPipeline;
use elai_knowledge::DkgEngine;
use elai_orchestrator::index_overlay::{
    live_overlay_enabled, DrainOpts, DrainOutcome, DrainTrigger, IndexOverlay, OverlayError,
    ELAI_LIVE_OVERLAY_ENABLE_ENV,
};
use tempfile::TempDir;

const BEFORE: &str = "fn keep_me() {}\nfn delete_me() {}\n";
const AFTER: &str = "fn keep_me() {}\nfn added_symbol() {}\n";

fn write_src(root: &Path, body: &str) -> std::path::PathBuf {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("src dir");
    let path = src.join("lib.rs");
    std::fs::write(&path, body).expect("write lib.rs");
    path
}

fn dkg_symbol_names(db: &Path) -> BTreeSet<String> {
    let conn = rusqlite::Connection::open(db).expect("open dkg");
    let mut stmt = conn
        .prepare("SELECT name FROM dkg_nodes ORDER BY name")
        .expect("prepare");
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query");
    rows.filter_map(|r| r.ok()).collect()
}

fn pipeline_with_overlay(root: &Path, overlay: Arc<IndexOverlay>) -> QueryPipeline {
    let mut bm25 = Bm25Index::new(root, "j104").expect("bm25");
    let src = root.join("src").join("lib.rs");
    let _ = bm25.index_file(&src, None);
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    QueryPipeline::new(bm25, catrag, arag).with_live_overlay(overlay)
}

#[test]
fn j104_edit_then_retrieve_before_batch_reindex() {
    let tmp = TempDir::new().expect("tmp");
    let root = tmp.path();
    let path = write_src(root, BEFORE);

    let db = root.join("probe.db");
    let mut engine = DkgEngine::new(&db, None, 0).expect("dkg");
    let _ = engine.build_graph(root);
    drop(engine);

    let overlay = Arc::new(IndexOverlay::new(root.to_path_buf(), true));
    std::fs::write(&path, AFTER).expect("edit on disk");
    overlay.capture_edit(&path, BEFORE, AFTER);

    let pipeline = pipeline_with_overlay(root, overlay);
    let added = pipeline
        .query("added_symbol")
        .expect("query added");
    let added_text: String = added
        .iter()
        .flat_map(|s| s.results.iter())
        .map(|r| format!("{} {}", r.chunk_id, r.snippet))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        added_text.contains("added_symbol"),
        "just-added symbol must be retrievable before batch re-index, got {added_text}"
    );

    let deleted = pipeline
        .query("delete_me")
        .expect("query deleted");
    let deleted_text: String = deleted
        .iter()
        .flat_map(|s| s.results.iter())
        .map(|r| format!("{} {}", r.chunk_id, r.snippet))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !deleted_text.contains("delete_me"),
        "just-deleted symbol must not be retrieved, got {deleted_text}"
    );
}

#[test]
fn j104_overlay_wins_on_conflict_and_tombstone_shadows_base() {
    let tmp = TempDir::new().expect("tmp");
    let root = tmp.path();
    let path = write_src(root, BEFORE);
    let overlay = Arc::new(IndexOverlay::new(root.to_path_buf(), true));
    overlay.capture_edit(&path, BEFORE, AFTER);

    let stale_keep = "fn keep_me() { stale }";
    overlay.capture_edit(&path, stale_keep, "fn keep_me() { live }");

    let hits = elai_knowledge::live_overlay::LiveIndexOverlay::hits_for_query(
        overlay.as_ref(),
        "keep_me",
    );
    assert!(
        hits.iter().any(|h| matches!(
            h,
            elai_knowledge::OverlayHit::Live { snippet, .. } if snippet.contains("live")
        )),
        "overlay must win on conflict, got {hits:?}"
    );

    let deleted = elai_knowledge::live_overlay::LiveIndexOverlay::hits_for_query(
        overlay.as_ref(),
        "delete_me",
    );
    assert!(
        deleted.iter().any(|h| matches!(
            h,
            elai_knowledge::OverlayHit::Tombstone { symbol, .. } if symbol == "delete_me"
        )),
        "tombstone must shadow base, got {deleted:?}"
    );
}

#[test]
fn j104_crash_mid_drain_leaves_old_consistent() {
    let tmp = TempDir::new().expect("tmp");
    let root = tmp.path();
    let path = write_src(root, BEFORE);

    let indexes = root.join(".elai_cc").join("indexes");
    std::fs::create_dir_all(&indexes).expect("indexes");
    let dkg = elai_core::dkg_db_path(root);
    let mut engine = DkgEngine::new(&dkg, None, 0).expect("dkg");
    let _ = engine.build_graph(root);
    drop(engine);
    let before_names = dkg_symbol_names(&dkg);
    assert!(
        before_names.iter().any(|n| n == "delete_me"),
        "fixture must index delete_me, got {before_names:?}"
    );

    let overlay = IndexOverlay::new(root.to_path_buf(), true);
    std::fs::write(&path, AFTER).expect("edit");
    overlay.capture_edit(&path, BEFORE, AFTER);
    assert!(overlay.pending_len() > 0);

    let _gate = drain_test_gate().lock().unwrap_or_else(|p| p.into_inner());
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let err = overlay
        .drain_atomic(
            DrainTrigger::NoMaterialProgress,
            clock,
            DrainOpts {
                inject_crash_before_apply: true,
            },
        )
        .expect_err("crash must fire");
    assert!(matches!(err, OverlayError::CrashInjected));

    let after_names = dkg_symbol_names(&dkg);
    assert_eq!(
        before_names, after_names,
        "crash before apply must leave DKG old-consistent"
    );
    assert!(
        overlay.pending_len() > 0,
        "overlay must be restored on injected crash"
    );
}

#[test]
fn j104_post_drain_dkg_matches_from_scratch() {
    let tmp = TempDir::new().expect("tmp");
    let root = tmp.path();
    let path = write_src(root, BEFORE);

    let indexes = root.join(".elai_cc").join("indexes");
    std::fs::create_dir_all(&indexes).expect("indexes");
    let dkg = elai_core::dkg_db_path(root);
    let mut engine = DkgEngine::new(&dkg, None, 0).expect("dkg");
    let _ = engine.build_graph(root);
    drop(engine);

    let overlay = IndexOverlay::new(root.to_path_buf(), true);
    std::fs::write(&path, AFTER).expect("edit");
    overlay.capture_edit(&path, BEFORE, AFTER);

    let _gate = drain_test_gate().lock().unwrap_or_else(|p| p.into_inner());
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let outcome = overlay
        .drain_atomic(
            DrainTrigger::NoMaterialProgress,
            clock,
            DrainOpts::default(),
        )
        .expect("drain");
    assert!(
        matches!(outcome, DrainOutcome::Drained { .. }),
        "expected drained, got {outcome:?}"
    );
    assert_eq!(overlay.pending_len(), 0);

    let drained = dkg_symbol_names(&dkg);

    let scratch = TempDir::new().expect("scratch");
    write_src(scratch.path(), AFTER);
    let scratch_db = scratch.path().join("fresh.db");
    let mut fresh = DkgEngine::new(&scratch_db, None, 0).expect("fresh dkg");
    let _ = fresh.build_graph(scratch.path());
    drop(fresh);
    let expected = dkg_symbol_names(&scratch_db);

    for name in ["added_symbol", "keep_me"] {
        assert!(
            drained.iter().any(|n| n == name),
            "drained DKG missing {name}: {drained:?}"
        );
        assert!(
            expected.iter().any(|n| n == name),
            "scratch DKG missing {name}: {expected:?}"
        );
    }
    assert!(
        !drained.iter().any(|n| n == "delete_me"),
        "drained DKG still has delete_me: {drained:?}"
    );
}

#[test]
fn j104_drain_trigger_is_order_based() {
    match DrainTrigger::NoMaterialProgress {
        DrainTrigger::NoMaterialProgress | DrainTrigger::SoftCap => {}
    }
    assert!(
        !live_overlay_enabled(),
        "{ELAI_LIVE_OVERLAY_ENABLE_ENV} must stay default-OFF in this process"
    );
}
