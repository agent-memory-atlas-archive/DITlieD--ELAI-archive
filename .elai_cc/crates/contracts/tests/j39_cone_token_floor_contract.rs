// J-39 T3 contract: derive_cone_token_floor empty-degrade + wiring.
// Exercises the SAME production fn wired at
//   crates/cli/src/symbol_graph.rs run_cone_token_floor (CLI production call site)
// No reimplemented cone math in the test body.

use elai_orchestrator::derive_cone_token_floor;
use elai_knowledge::dkg::DkgEngine;
use elai_knowledge::symbol_graph::cache::SymbolGraphCache;
use elai_knowledge::symbol_graph::store;
use elai_knowledge::symbol_graph::types::SymbolGraphMeta;
use rusqlite::Connection;
use tempfile::TempDir;

fn build_minimal_db(dir: &TempDir, project_id: &str) -> std::path::PathBuf {
    let db_path = dir.path().join(format!("{}_dkg.db", project_id.replace('-', "_")));
    let _e = DkgEngine::new(&db_path, None, 1).expect("DkgEngine");
    drop(_e);

    let mut conn = Connection::open(&db_path).expect("open");
    store::upsert_meta(
        &mut conn,
        &SymbolGraphMeta {
            project_id: project_id.to_string(),
            symbol_count: 0,
            edge_count: 0,
            file_count: 0,
            unresolved_edge_pct: 0.0,
            built_at: 1,
            schema_version: 3,
        },
    )
    .expect("meta");

    db_path
}

#[test]
fn empty_entry_files_returns_zero() {
    let dir = TempDir::new().expect("tempdir");
    let project_id = "j39-contract-empty";
    let db_path = build_minimal_db(&dir, project_id);
    let mut cache = SymbolGraphCache::new(&db_path, project_id, 64)
        .expect("SymbolGraphCache::new with valid db must not fail");
    let floor = derive_cone_token_floor(&[], &mut cache, 2);
    assert_eq!(floor, 0, "empty entry_files must return 0 without using cache");
}

#[test]
fn non_existent_target_degrades_to_zero() {
    let dir = TempDir::new().expect("tempdir");
    let project_id = "j39-contract-missing";
    let db_path = build_minimal_db(&dir, project_id);
    let mut cache = SymbolGraphCache::new(&db_path, project_id, 64)
        .expect("SymbolGraphCache::new with valid db must not fail");
    let entry = vec!["nonexistent_file_xyz.rs".to_string()];
    let floor = derive_cone_token_floor(&entry, &mut cache, 2);
    assert_eq!(floor, 0, "non-existent target must degrade to 0, never panic");
}
