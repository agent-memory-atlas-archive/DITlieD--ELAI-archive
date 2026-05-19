//! C-25 — handler for `elai pipeline [list|reset]`.
//!
//! - `list` (default): print every entry in
//!   `.elai/cache/compiled_pipelines.msgpack` as one line per entry
//!   showing concept_class, plan_skeleton_hash, use_count,
//!   first_try_success_rate, created_at.
//! - `reset`: clear the in-memory store + delete the msgpack file.
//! - `help`: print the usage line.
//!
//! Pure synchronous entrypoint (no tokio runtime needed — the store's
//! load_from_disk / clear_all are blocking std::fs).

use std::path::Path;

use anyhow::Result;
use elai_orchestrator::compiled_pipeline::CompiledPipelineStore;

pub(crate) fn run_pipeline_subcommand(action: Option<&str>, project_root: &Path) -> Result<()> {
    let normalized = action.unwrap_or("list");
    match normalized {
        "list" | "show" => run_list(project_root),
        "reset" | "clear" => run_reset(project_root),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(anyhow::anyhow!(
            "unknown /pipeline action '{other}'. Use list, reset, or help."
        )),
    }
}

fn run_list(project_root: &Path) -> Result<()> {
    let cache_path = CompiledPipelineStore::standard_path(project_root);
    let mut store = CompiledPipelineStore::new(cache_path.clone());
    store.load_from_disk();
    let entries = store.entries();
    if entries.is_empty() {
        println!(
            "No compiled pipelines cached at {}.",
            cache_path.display()
        );
        return Ok(());
    }
    println!(
        "Compiled pipelines at {} (count={}):",
        cache_path.display(),
        entries.len()
    );
    let mut rows: Vec<_> = entries.values().collect();
    rows.sort_by(|a, b| a.last_used_at.cmp(&b.last_used_at).reverse());
    for p in rows {
        println!(
            "  concept_class={concept:<18} hash={hash:#018x} use_count={uses:<4} success_rate={rate:.3} created_at={created} last_used={used}",
            concept = p.concept_class,
            hash = p.plan_skeleton_hash,
            uses = p.use_count,
            rate = p.first_try_success_rate,
            created = p.created_at,
            used = p.last_used_at,
        );
    }
    Ok(())
}

fn run_reset(project_root: &Path) -> Result<()> {
    let cache_path = CompiledPipelineStore::standard_path(project_root);
    let mut store = CompiledPipelineStore::new(cache_path.clone());
    store.load_from_disk();
    let before = store.entries().len();
    store.clear_all().map_err(|e| anyhow::anyhow!("clear_all failed: {e}"))?;
    println!(
        "Cleared {} compiled pipeline entries from {}.",
        before,
        cache_path.display()
    );
    Ok(())
}

fn print_help() {
    println!("Usage: elai pipeline [list|reset|help]");
    println!("       /pipeline [list|reset|help] (REPL)");
    println!("  list   Print all compiled pipeline cache entries (default).");
    println!("  reset  Clear the cache + delete the on-disk msgpack file.");
    println!("  help   This message.");
}
