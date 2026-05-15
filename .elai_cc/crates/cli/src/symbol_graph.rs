// B-29 Phase 4 — CLI runner for `elai symbol_graph` subcommand.
//
// Supports two sub-sub-commands:
//   elai symbol_graph build [--project-root <path>]  — full build
//   elai symbol_graph stats [--project-root <path>]  — print SymbolGraphMeta
//
// This module calls the knowledge crate directly (no IndexingService).
// IndexingService auto-wiring is the preferred path (B-29 Phase 4 Task 10),
// but is gated on s-21092 releasing the indexing_service.rs lock.

use std::path::PathBuf;

use elai_knowledge::{
    dkg::DkgEngine,
    symbol_graph::{
        build_pipeline::{build_symbol_graph, INCREMENTAL_SYMBOL_THRESHOLD},
        cache::SymbolGraphCache,
        sharding::SYMBOL_FILE_LRU_SIZE,
    },
};
use elai_core::paths::dkg_db_path as core_dkg_db_path;

/// Run `elai symbol_graph build` for `project_root`.
///
/// 1. Opens a DkgEngine and calls build_graph to ensure DKG is up-to-date.
/// 2. Drops DkgEngine to release the WAL write-lock.
/// 3. Calls build_symbol_graph.
/// 4. Prints a one-line summary.
pub(crate) fn run_symbol_graph_build(project_root: PathBuf) -> Result<(), anyhow::Error> {
    let db_path = core_dkg_db_path(&project_root);

    // Derive project_id from the root directory name (same heuristic used by
    // the indexing pipeline for projects without an explicit id).
    let project_id = match project_root.file_name().and_then(|n| n.to_str()) {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => {
            return Err(anyhow::anyhow!(
                "symbol_graph: cannot derive project_id from project_root={:?} (no file_name component — pass a non-root path)",
                project_root
            ));
        }
    };

    eprintln!(
        "[symbol_graph] building DKG for project '{}' ...",
        project_id
    );

    // Capture a single timestamp at entrypoint init (CLAUDE.md exemption:
    // CLI entrypoint allowed to call SystemTime::now once for built_at).
    let clock_now_ms = entrypoint_now_ms();

    {
        let mut dkg = DkgEngine::new(&db_path, None, clock_now_ms)
            .map_err(|e| anyhow::anyhow!("DkgEngine::new failed: {e}"))?;
        dkg.build_graph(&project_root)
            .map_err(|e| anyhow::anyhow!("dkg.build_graph failed: {e}"))?;
        // DkgEngine drops here, releasing WAL lock.
    }

    eprintln!("[symbol_graph] building symbol graph ...");

    let summary = build_symbol_graph(&project_id, &project_root, &db_path, clock_now_ms)
        .map_err(|e| anyhow::anyhow!("build_symbol_graph failed: {e}"))?;

    println!(
        "Symbol graph built: files={} symbols={} edges={} unresolved_pct={:.1}%",
        summary.files, summary.symbols, summary.edges, summary.unresolved_pct
    );

    Ok(())
}

/// Run `elai symbol_graph stats` for `project_root`.
///
/// Reads SymbolGraphMeta from the database and prints the key fields.
pub(crate) fn run_symbol_graph_stats(project_root: PathBuf) -> Result<(), anyhow::Error> {
    let db_path = core_dkg_db_path(&project_root);

    let project_id = match project_root.file_name().and_then(|n| n.to_str()) {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => {
            return Err(anyhow::anyhow!(
                "symbol_graph: cannot derive project_id from project_root={:?} (no file_name component — pass a non-root path)",
                project_root
            ));
        }
    };

    let cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
        .map_err(|e| anyhow::anyhow!("SymbolGraphCache::new failed: {e}"))?;

    let meta = cache.meta();
    println!("Symbol graph stats for '{}':", project_id);
    println!("  files          : {}", meta.file_count);
    println!("  symbols        : {}", meta.symbol_count);
    println!("  edges          : {}", meta.edge_count);
    println!("  unresolved_pct : {:.1}%", meta.unresolved_edge_pct);
    println!("  built_at_ms    : {}", meta.built_at);
    println!("  schema_version : {}", meta.schema_version);

    Ok(())
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Capture the wall-clock timestamp once at entrypoint init.
///
/// CLAUDE.md ClockProvider rule exempts `main()/entrypoint/bootloader init`.
/// This function is called exactly once per CLI subcommand invocation, from
/// the subcommand entrypoint — never from library code. The library functions
/// (`build_symbol_graph`, `update_changed_files_symbol_graph`) accept the
/// timestamp as a parameter.
fn entrypoint_now_ms() -> i64 {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0u128);
    i64::try_from(millis).unwrap_or(i64::MAX)
}
