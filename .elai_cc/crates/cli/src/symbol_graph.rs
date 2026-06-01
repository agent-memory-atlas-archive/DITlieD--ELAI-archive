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
        entry_points::{detect_entry_points, EntryPointReason, EntryPointSummary},
        sharding::SYMBOL_FILE_LRU_SIZE,
        traversal::{compute_impact_radius, compute_symbol_context},
    },
};
use elai_core::paths::dkg_db_path as core_dkg_db_path;
use elai_core::project_id_from_path;

use crate::args::CliOutputFormat;
// E-21 Tier-A: PageRank top-N readback via the existing SsbViewer surface
// (C-3 ship at ssb_viewer.rs:2897). pagerank_top reads from the
// dkg_nodes_pagerank table when present, returns Vec::new() when absent.
use elai_orchestrator::{default_ssb_path_for_base, SsbViewer};

/// Run `elai symbol_graph build` for `project_root`.
///
/// 1. Opens a DkgEngine and calls build_graph to ensure DKG is up-to-date.
/// 2. Drops DkgEngine to release the WAL write-lock.
/// 3. Calls build_symbol_graph.
/// 4. Prints a one-line summary.
pub(crate) fn run_symbol_graph_build(project_root: PathBuf) -> Result<(), anyhow::Error> {
    let project_root = normalize_workspace_root(project_root);
    let db_path = core_dkg_db_path(&project_root);

    // B-34 DECISION-9 / production_agent_rpc.rs:1682 — use the canonical
    // blake3-based project_id so the indexer-written shards stay consistent
    // with the readback handlers (symbol_impact / symbol_context /
    // entry_points all use project_id_from_path). The previous basename
    // heuristic caused cache misses across the build/query boundary.
    let project_id = project_id_from_path(&project_root);

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
    let project_root = normalize_workspace_root(project_root);
    let db_path = core_dkg_db_path(&project_root);

    // Same blake3 project_id as `run_symbol_graph_build` so stats reads the
    // shards the builder wrote.
    let project_id = project_id_from_path(&project_root);

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

/// E-21 Tier-A — `elai symbol_graph rank --top N [--project-root <path>]`.
///
/// Reads the existing dkg_nodes_pagerank table (populated by
/// `knowledge::pipeline::compute_repo_map` / `knowledge::pagerank`) via
/// `SsbViewer::pagerank_top` and prints the top-N symbols by score.
///
/// Returns gracefully (exit 0 with a hint message) when the table is absent —
/// the same shape the dashboard's get_pagerank_top Tauri command uses for
/// "no data yet". This matches the spec: a fresh project that hasn't run
/// `knowledge::pipeline::compute_repo_map` yet should not error.
pub(crate) fn run_symbol_graph_rank(
    project_root: PathBuf,
    top: u32,
) -> Result<(), anyhow::Error> {
    let project_root = normalize_workspace_root(project_root);
    let ssb_path = default_ssb_path_for_base(&project_root);
    if !ssb_path.exists() {
        // No SSB DB yet — emit a hint, exit 0. Matches dashboard graceful-empty.
        println!(
            "[symbol_graph rank] no SSB DB at {} — run a build/index first",
            ssb_path.display()
        );
        return Ok(());
    }
    let viewer = SsbViewer::open(&ssb_path)
        .map_err(|e| anyhow::anyhow!("SsbViewer::open({}) failed: {e}", ssb_path.display()))?;
    let limit = if top == 0 { 20_usize } else { top as usize };
    let entries = viewer
        .pagerank_top(limit)
        .map_err(|e| anyhow::anyhow!("pagerank_top({limit}) failed: {e}"))?;
    if entries.is_empty() {
        println!(
            "[symbol_graph rank] dkg_nodes_pagerank table empty or absent — \
             trigger knowledge::pipeline::compute_repo_map to populate"
        );
        return Ok(());
    }
    println!("PageRank top {} symbols:", entries.len());
    for (idx, e) in entries.iter().enumerate() {
        println!(
            "  {:>3}. {:<60} {:>9.5}  ({})",
            idx + 1,
            truncate_symbol(&e.symbol, 60),
            e.score,
            e.path,
        );
    }
    Ok(())
}

fn truncate_symbol(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    // Walk back to the last char boundary at-or-before `max` so non-ASCII
    // symbols (UTF-8 multibyte) never panic. `is_char_boundary(0)` is always
    // true so the loop terminates.
    let mut idx = max;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    &s[..idx]
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Normalize a `--project-root` argument so the workspace root is the parent
/// of `.elai_cc/`, never `.elai_cc/` itself. Without this, a user running
/// `elai symbol_graph build` from inside `.elai_cc/` (or passing
/// `--project-root <repo>/.elai_cc`) would compute a double-nested DKG path
/// like `<repo>/.elai_cc/.elai_cc/indexes/dkg.db` via `core_dkg_db_path`,
/// which appends `.elai_cc/indexes/dkg.db` unconditionally.
///
/// Detection is purely segment-based (no IO): if the final path component is
/// `.elai_cc`, strip it. Anything else is returned unchanged.
fn normalize_workspace_root(project_root: PathBuf) -> PathBuf {
    if project_root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == elai_core::paths::ELAI_DATA_DIR_NAME)
        .unwrap_or(false)
    {
        if let Some(parent) = project_root.parent() {
            return parent.to_path_buf();
        }
    }
    project_root
}

/// Capture the wall-clock timestamp once at entrypoint init.
///
/// CLAUDE.md ClockProvider rule exempts `main()/entrypoint/bootloader init`.
/// This function is called exactly once per CLI subcommand invocation, from
/// the subcommand entrypoint — never from library code. The library functions
/// (`build_symbol_graph`, `update_changed_files_symbol_graph`) accept the
/// timestamp as a parameter.
// C-15 entropy-allow manifest:
//   line 119 — entrypoint-only one-shot now-ms read; ClockProvider exempted
//              per CLAUDE.md "main()/entrypoint" rule, see fn doc above.
fn entrypoint_now_ms() -> i64 {
    let millis = std::time::SystemTime::now() // entropy-allow: cli-entrypoint-bootloader-init
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0u128);
    i64::try_from(millis).unwrap_or(i64::MAX)
}

// ---------------------------------------------------------------------------
// B-29 Phase 5b — fork-exec shell wrappers for SymbolGraphCache traversal.
//
// These three handlers expose the existing knowledge::symbol_graph traversal
// + entry-point detection surfaces to external scripts (Python eval harness,
// agents) via direct CLI invocation. No IPC bridge, no daemon. They follow
// the same SymbolGraphCache + project_id_from_path pattern as the
// production_agent_rpc TOOL_SYMBOL_IMPACT / TOOL_SYMBOL_CONTEXT /
// TOOL_ENTRY_POINTS arms (production_agent_rpc.rs:1673-1750).
//
// Exit codes:
//   0 — success (including "no results")
//   non-zero — DKG missing, cache open failure, symbol/file not in DB
//              (propagated as anyhow::Error from main.rs)
// ---------------------------------------------------------------------------

/// Run `elai symbol_impact --target <X> [--depth N] [--output-format ...]`.
///
/// Calls `compute_impact_radius` against the project's SymbolGraphCache.
/// `target` is auto-classified as file vs symbol per
/// `looks_like_file_path` (graph-impact.ts:268-270 in the source paper).
pub(crate) fn run_symbol_impact(
    project_root: PathBuf,
    target: String,
    depth: u32,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let project_root = normalize_workspace_root(project_root);
    let db_path = core_dkg_db_path(&project_root);
    if !db_path.exists() {
        return Err(anyhow::anyhow!(
            "symbol_impact: DKG database not found at {} — run `elai symbol_graph build --project-root {}` first",
            db_path.display(),
            project_root.display(),
        ));
    }
    // B-34 DECISION-9 / production_agent_rpc.rs:1682 — use the canonical
    // blake3-based project_id so the cache hits the same shards the
    // indexer wrote (NOT the file_name() basename heuristic).
    let project_id = project_id_from_path(&project_root);

    let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
        .map_err(|e| anyhow::anyhow!("SymbolGraphCache::new failed: {e}"))?;

    let result = compute_impact_radius(&mut cache, &target, depth)
        .map_err(|e| anyhow::anyhow!("compute_impact_radius failed: {e}"))?;

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string(&result)
                .map_err(|e| anyhow::anyhow!("serialise impact result to JSON: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("Impact radius for '{}':", result.target);
            println!("  total_files       : {}", result.total_files);
            println!("  max_depth_reached : {}", result.max_depth_reached);
            println!("  truncated         : {}", result.truncated);
            if result.total_files == 0 {
                println!("  (no files found — target may be unknown or DKG stale)");
            } else {
                println!("  files_by_depth:");
                for (hop, files) in &result.files_by_depth {
                    println!("    hop {hop}: {} files", files.len());
                    for f in files {
                        println!("      - {f}");
                    }
                }
            }
        }
    }
    Ok(())
}

/// Run `elai symbol_context --name <X> [--file PATH] [--output-format ...]`.
///
/// Calls `compute_symbol_context` for all definitions of `name` (filtered
/// to `file` when supplied). Returns 360° view (definition + callers +
/// callees) per definition.
pub(crate) fn run_symbol_context(
    project_root: PathBuf,
    name: String,
    file: Option<String>,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let project_root = normalize_workspace_root(project_root);
    let db_path = core_dkg_db_path(&project_root);
    if !db_path.exists() {
        return Err(anyhow::anyhow!(
            "symbol_context: DKG database not found at {} — run `elai symbol_graph build --project-root {}` first",
            db_path.display(),
            project_root.display(),
        ));
    }
    let project_id = project_id_from_path(&project_root);

    let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
        .map_err(|e| anyhow::anyhow!("SymbolGraphCache::new failed: {e}"))?;

    let contexts = compute_symbol_context(&mut cache, &name, file.as_deref())
        .map_err(|e| anyhow::anyhow!("compute_symbol_context failed: {e}"))?;

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string(&contexts)
                .map_err(|e| anyhow::anyhow!("serialise contexts to JSON: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            if contexts.is_empty() {
                println!(
                    "Symbol context for '{}': no definitions found{}",
                    name,
                    match &file {
                        Some(f) => format!(" (filtered to file={f})"),
                        None => String::new(),
                    }
                );
                return Ok(());
            }
            println!("Symbol context for '{}': {} definition(s)", name, contexts.len());
            for (idx, ctx) in contexts.iter().enumerate() {
                println!(
                    "  [{idx}] def: {} at {}:{} (kind={:?})",
                    ctx.definition.qualified_name,
                    ctx.definition.file,
                    ctx.definition.line,
                    ctx.definition.kind,
                );
                println!("       callers : {}", ctx.callers.len());
                for c in &ctx.callers {
                    println!(
                        "         - {} from {}:{}",
                        c.caller_id, c.call_site_file, c.call_site_line
                    );
                }
                println!("       callees : {}", ctx.callees.len());
                for c in &ctx.callees {
                    println!("         - {} ({})", c.callee_id, c.callee_name);
                }
            }
        }
    }
    Ok(())
}

/// Run `elai entry_points [--limit N] [--output-format ...]`.
///
/// Mirrors `TOOL_ENTRY_POINTS` in production_agent_rpc.rs:1677-1750.
/// Loads the reverse index once, applies the same coarse code-extension
/// filter, loads candidate payloads, and runs `detect_entry_points`. Test
/// entries are excluded by default (matching `include_tests = false`).
pub(crate) fn run_entry_points(
    project_root: PathBuf,
    limit: usize,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let project_root = normalize_workspace_root(project_root);
    let db_path = core_dkg_db_path(&project_root);
    if !db_path.exists() {
        return Err(anyhow::anyhow!(
            "entry_points: DKG database not found at {} — run `elai symbol_graph build --project-root {}` first",
            db_path.display(),
            project_root.display(),
        ));
    }
    let project_id = project_id_from_path(&project_root);

    let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
        .map_err(|e| anyhow::anyhow!("SymbolGraphCache::new failed: {e}"))?;

    // Step 1: load reverse index (lazy, ~5ms first call).
    let reverse_index = cache
        .ensure_reverse_index()
        .map_err(|e| anyhow::anyhow!("ensure_reverse_index failed: {e}"))?
        .clone();

    // Step 2: collect all known files from the reverse index.
    let mut all_files: std::collections::BTreeSet<String> =
        reverse_index.keys().cloned().collect();
    for callers in reverse_index.values() {
        all_files.extend(callers.iter().cloned());
    }

    // Step 3: coarse extension filter (mirrors production_agent_rpc.rs:1712).
    // BLOCK-5 / DECISION-B34-9: do NOT pre-filter on orphan+filename, just
    // exclude non-code files for performance — Heuristic 3 needs imported
    // route files in the payload set.
    const CODE_EXTENSIONS: &[&str] = &[
        "ts", "tsx", "js", "jsx", "py", "rb", "go", "rs", "java", "kt", "php",
        "c", "cpp", "cs", "swift", "scala", "ex", "exs",
    ];
    let candidate_files: Vec<String> = all_files
        .into_iter()
        .filter(|f| {
            let ext = std::path::Path::new(f)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            CODE_EXTENSIONS.contains(&ext)
        })
        .collect();

    // Step 4: selectively load payloads for candidates only.
    let mut payloads = Vec::with_capacity(candidate_files.len());
    for file in &candidate_files {
        match cache.get_or_load_payload(file) {
            Ok(Some(arc)) => payloads.push((*arc).clone()),
            Ok(None) => {} // file not in DB — skip
            Err(e) => {
                tracing::warn!(
                    file = %file,
                    error = %e,
                    "entry_points: payload load failed — skipping"
                );
            }
        }
    }

    // Step 5: run detection (pure fn, no I/O).
    let mut entries = detect_entry_points(&reverse_index, &payloads);

    // Step 6: filter tests (mirrors include_tests=false default in TOOL_ENTRY_POINTS).
    entries.retain(|e| !matches!(e.reason, EntryPointReason::Test));

    // Step 7: apply --limit cap (CLI-only — TOOL_ENTRY_POINTS has no limit).
    if limit > 0 && entries.len() > limit {
        entries.truncate(limit);
    }

    let summary = EntryPointSummary::from_entries(&entries);

    match output_format {
        CliOutputFormat::Json => {
            let payload = serde_json::json!({
                "entries": entries,
                "summary": summary,
            });
            let json = serde_json::to_string(&payload)
                .map_err(|e| anyhow::anyhow!("serialise entry_points to JSON: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("Entry points: {} total", summary.total);
            for (reason, count) in &summary.by_reason {
                println!("  {reason}: {count}");
            }
            if !entries.is_empty() {
                println!("Entries:");
                for e in &entries {
                    let line_part = e.line.map(|l| format!(":{l}")).unwrap_or_default();
                    println!(
                        "  [{}] {} ({}{})",
                        e.reason.key(),
                        e.name,
                        e.file,
                        line_part
                    );
                }
            }
        }
    }
    Ok(())
}

