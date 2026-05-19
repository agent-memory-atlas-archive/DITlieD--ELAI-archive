// C-18 B-32 backfill — IndexingService minified-file SSB emission contract.
//
// The plan named this test `chunking_minified.rs` and located it under
// `crates/knowledge/tests/`. The context-package audit (C-18 §3 DRIFT)
// flagged the wrong crate: `IndexingService` lives in `elai-orchestrator`,
// and `elai-knowledge` cannot depend on it (would form a cycle).  The test
// therefore lives in `crates/orchestrator/tests/` under the corrected name.
//
// What it asserts:
//   - Indexing a project that contains one minified `bundle.min.js` file
//     (avg_line_length > 500) emits at least one
//     `SsbActionType::MinifiedFileChunked` SSB row whose payload's
//     `relative_path` contains "bundle.min.js" and whose `chunk_count > 0`.
//
// The test uses a fresh tempdir for both the project root and the SSB db.
// The indexer is driven via the production `IndexingService::run_full_index`
// entry-point (matches B-32 production flow). `chunking_minified.rs` in the
// knowledge crate is left untouched (it covers chunker unit-level behaviour).

use std::sync::Arc;

use elai_orchestrator::indexing_service::IndexingService;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload};
use elai_orchestrator::ssb_viewer::SsbViewer;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal project tree with one minified JS file:
///   <root>/vendor/bundle.min.js — single-line ~12K-char file
fn setup_minified_project(root: &std::path::Path) {
    let vendor = root.join("vendor");
    std::fs::create_dir_all(&vendor).expect("create vendor/ dir");
    // Single line, > MAX_AVG_LINE_LENGTH (500) so chunking::detect::is_minified
    // classifies it as minified. We make it large enough that chunk_count > 0.
    let mut content = String::with_capacity(12_000);
    for _ in 0..1_200 {
        content.push_str("var a=1;");
    }
    std::fs::write(vendor.join("bundle.min.js"), &content)
        .expect("write bundle.min.js");
    // Add a non-minified file so the BM25 path always has work to do (B-31's
    // resumable indexer skips empty projects).
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create src/");
    std::fs::write(src.join("a.rs"), "pub fn a() {}\n").expect("write a.rs");
}

fn make_ssb_with_db(db_path: std::path::PathBuf) -> Arc<Ssb> {
    let config = SsbConfig {
        db_path,
        project_id: "c18-minified".to_owned(),
        session_id: "s-c18-minified".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(elai_core::clock::OsClock);
    Arc::new(Ssb::new(config, clock).expect("Ssb::new"))
}

async fn wait_for_minified_rows(
    viewer: &SsbViewer,
    expected_at_least: usize,
    max_polls: u32,
) -> Vec<elai_orchestrator::ssb_viewer::SsbActionDetail> {
    let action_str = SsbActionType::MinifiedFileChunked.as_sql_str();
    for _ in 0..max_polls {
        if let Ok(rows) = viewer.recent_rows_of_type(action_str, 32) {
            if rows.len() >= expected_at_least {
                return rows;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    viewer
        .recent_rows_of_type(action_str, 32)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Contract: bundle.min.js triggers exactly one MinifiedFileChunked row with
// chunk_count > 0 and the file path embedded in the payload.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_minified_file_emits_ssb_action() {
    let project_dir = TempDir::new().expect("project tempdir");
    let project_root = project_dir.path().to_path_buf();
    setup_minified_project(&project_root);

    // SSB lives in a separate tempdir so we don't accidentally walk the SSB
    // sqlite file as a "source" file (it lives under .elai_cc/indexes anyway,
    // which is the gitignore-excluded path).
    let ssb_dir = TempDir::new().expect("ssb tempdir");
    let ssb_db_path = ssb_dir.path().join("c18_minified_ssb.db");
    let ssb = make_ssb_with_db(ssb_db_path.clone());
    let viewer = SsbViewer::open(&ssb_db_path).expect("SsbViewer::open");

    let indexes_dir = project_root.join(".elai_cc").join("indexes");
    let mut svc =
        IndexingService::new(project_root.clone(), indexes_dir).with_ssb_writer(Arc::clone(&ssb));

    // Drive the production entry-point so `emit_minified_chunked_rows` fires.
    let summary = svc.run_full_index().expect("run_full_index must succeed");
    assert!(
        summary.bm25_files >= 1,
        "fixture must index at least one file (got {})",
        summary.bm25_files,
    );

    let rows = wait_for_minified_rows(&viewer, 1, 40).await;
    assert!(
        !rows.is_empty(),
        "expected at least 1 MinifiedFileChunked SSB row after indexing a minified file",
    );

    // At least one row's payload must reference bundle.min.js and have
    // chunk_count > 0.
    let mut found = false;
    for detail in &rows {
        let parsed: SsbPayload = match serde_json::from_value(detail.payload_json.clone()) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if let SsbPayload::MinifiedFileChunked(p) = parsed {
            if p.relative_path.contains("bundle.min.js") && p.chunk_count > 0 {
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "no MinifiedFileChunked row with relative_path containing 'bundle.min.js' \
         and chunk_count > 0 found; got rows: {rows:?}",
    );
}
