//! dkg-stream-rigorous Task 2: `elai index build` — non-interactive full index
//! build.
//!
//! # Why this exists
//!
//! Before this command the only way to populate the on-disk index was the
//! implicit first-run build inside `init_indexing_service` (driven from a live
//! chat/REPL session). The `bench rag` eval seam needs the index populated at
//! the CANONICAL `.elai_cc/indexes/projects/default/bm25` + `dkg.db` +
//! `vector_index.json` paths WITHOUT entering an interactive session. This
//! command is that scriptable seam: it constructs the same `IndexingService`
//! production uses, points it at the canonical indexes dir, and runs the full
//! build (BM25 + DKG + optional streaming vectors).
//!
//! # Wiring discipline
//!
//! producer : [`elai_orchestrator::indexing_service::IndexingService::run_full_index`]
//!            (the exact constructor + entrypoint `init_indexing_service` uses).
//! reader   : `crate::live_cli::LiveCli::build_local_knowledge_stack` +
//!            `crate::bench_rag` (both open BM25 via `bm25_index_root` +
//!            `BM25_PROJECT_ID`, the same canonical location this writes).
//!
//! # Memory
//!
//! The DKG build is now index-time memory-bounded (the dense-PageRank-matrix
//! OOM was replaced with a sparse transition in
//! `elai_knowledge::pagerank`). The optional vector stage (`--vectors`) embeds
//! ONE file at a time via `VectorIndexer::index_file`, so peak RAM is bounded
//! by the embedding model + a single file's chunks, never the whole corpus.

use std::path::PathBuf;

use elai_orchestrator::indexing_service::{IndexingService, ELAI_BULK_VECTOR_BUILD_ENV};

/// Run a full, non-interactive index build at `project_root`.
///
/// When `vectors` is set, `ELAI_BULK_VECTOR_BUILD=1` is exported for this
/// process so `run_full_index` populates the (streaming, memory-bounded)
/// vector stage. Writes to the canonical `.elai_cc/indexes` layout.
pub(crate) fn run_index_build(
    project_root: PathBuf,
    vectors: bool,
    json: bool,
) -> anyhow::Result<()> {
    if vectors {
        // Opt the streaming vector stage into this build. `bulk_vector_build_enabled`
        // reads this var when `build_index_resumable` reaches the vector step.
        // SAFETY: single-threaded CLI entrypoint, set before any worker spawns.
        std::env::set_var(ELAI_BULK_VECTOR_BUILD_ENV, "1");
    }

    let indexes_dir = elai_core::paths::indexes_dir_path(&project_root);
    let mut service = IndexingService::new(project_root.clone(), indexes_dir.clone());

    let summary = service
        .run_full_index()
        .map_err(|e| anyhow::anyhow!("index build failed: {e}"))?;

    if json {
        let body = serde_json::json!({
            "project_root": project_root.display().to_string(),
            "indexes_dir": indexes_dir.display().to_string(),
            "vectors_requested": vectors,
            "dkg_files_parsed": summary.dkg_files_parsed,
            "dkg_nodes": summary.dkg_nodes,
            "dkg_edges": summary.dkg_edges,
            "dkg_files_failed": summary.dkg_files_failed,
            "bm25_files": summary.bm25_files,
            "bm25_chunks": summary.bm25_chunks,
            "bm25_skipped_binary": summary.bm25_skipped_binary,
        });
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("elai index build");
        println!("  project_root = {}", project_root.display());
        println!("  indexes_dir  = {}", indexes_dir.display());
        println!("  vectors      = {vectors}");
        println!(
            "  dkg          = {} files / {} nodes / {} edges ({} failed)",
            summary.dkg_files_parsed, summary.dkg_nodes, summary.dkg_edges, summary.dkg_files_failed
        );
        println!(
            "  bm25         = {} files / {} chunks ({} binary skipped)",
            summary.bm25_files, summary.bm25_chunks, summary.bm25_skipped_binary
        );
    }

    Ok(())
}
