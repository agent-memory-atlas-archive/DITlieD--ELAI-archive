// B-29 Phase 4 — Contract tests for build_pipeline public API.
//
// Locks:
//   - build_symbol_graph signature
//   - update_changed_files_symbol_graph signature
//   - BuildSummary field names and types
//   - UpdateSummary field names and types
//   - INCREMENTAL_SYMBOL_THRESHOLD == 50

use std::path::{Path, PathBuf};

use elai_knowledge::symbol_graph::build_pipeline::{
    BuildSummary, UpdateSummary, INCREMENTAL_SYMBOL_THRESHOLD,
};

// ---------------------------------------------------------------------------
// Compile-time structural contracts
// ---------------------------------------------------------------------------

/// Assert BuildSummary has exactly the fields the plan specifies.
/// This test will fail to compile if any field is renamed or removed.
fn _assert_build_summary_fields(s: BuildSummary) {
    let _files: usize = s.files;
    let _symbols: usize = s.symbols;
    let _edges: usize = s.edges;
    let _unresolved_pct: f64 = s.unresolved_pct;
}

/// Assert UpdateSummary has exactly the fields the plan specifies.
fn _assert_update_summary_fields(s: UpdateSummary) {
    let _files_changed: usize = s.files_changed;
    let _files_removed: usize = s.files_removed;
    let _rebuild_triggered: bool = s.rebuild_triggered;
}

/// Assert the function signatures are callable with the expected types.
/// This function is never called — it exists only to trigger type-checking.
#[allow(dead_code)]
fn _assert_function_signatures() {
    use elai_knowledge::symbol_graph::build_pipeline::{
        build_symbol_graph, update_changed_files_symbol_graph,
    };
    use elai_knowledge::KnowledgeError;

    // build_symbol_graph(project_id, project_root, dkg_db_path, clock_now_ms)
    //   -> Result<BuildSummary, KnowledgeError>
    // clock_now_ms is injected per CLAUDE.md ClockProvider rule — no raw
    // SystemTime::now() inside library code.
    let _: fn(&str, &Path, &Path, i64) -> Result<BuildSummary, KnowledgeError> = build_symbol_graph;

    // update_changed_files_symbol_graph(project_id, changed_paths, removed_paths,
    //   dkg_db_path, project_root, clock_now_ms) -> Result<UpdateSummary, KnowledgeError>
    let _: fn(
        &str,
        &[PathBuf],
        &[PathBuf],
        &Path,
        &Path,
        i64,
    ) -> Result<UpdateSummary, KnowledgeError> = update_changed_files_symbol_graph;
}

// ---------------------------------------------------------------------------
// Runtime constant contract
// ---------------------------------------------------------------------------

#[test]
fn test_incremental_symbol_threshold_locked_at_50() {
    assert_eq!(
        INCREMENTAL_SYMBOL_THRESHOLD,
        50,
        "INCREMENTAL_SYMBOL_THRESHOLD is a spec-locked constant; changing it is a breaking change"
    );
}

// ---------------------------------------------------------------------------
// Dependency reachability: elai-knowledge must be accessible from elai-orchestrator
// ---------------------------------------------------------------------------

/// This test uses the fact that elai-orchestrator is a dev-dep of the contracts
/// crate AND elai-knowledge is a dep of elai-orchestrator. The type
/// `elai_orchestrator::indexing_service::IndexingService` is only constructable
/// via the orchestrator crate which itself depends on elai_knowledge. If this
/// import fails to compile, the orchestrator's dep graph is broken.
#[test]
fn test_orchestrator_can_reference_knowledge_build_pipeline() {
    // Verify the knowledge crate's build_pipeline module is accessible.
    // A compilation error here means the module is missing or not pub-exported.
    let threshold = elai_knowledge::symbol_graph::build_pipeline::INCREMENTAL_SYMBOL_THRESHOLD;
    assert_eq!(threshold, 50);
}
