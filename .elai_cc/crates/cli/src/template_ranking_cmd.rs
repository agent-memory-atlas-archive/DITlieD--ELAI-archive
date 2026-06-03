//! F-7 / Slice A4 — handler for `elai template-ranking list [--json]`.
//!
//! W-3 entrypoint + W-5 live trigger for the CBR template-ranking surface
//! (HALF B: Laplace-utility outcome ranking). Reads the PERSISTED
//! [`CompiledPipelineStore`] (the same on-disk msgpack cache the C-25
//! `elai pipeline list` surface reads — `.elai/cache/compiled_pipelines.msgpack`),
//! computes the Laplace add-one smoothed outcome utility per crystallized
//! template via [`laplace_utility_from_fields`], sorts DESC by utility, emits
//! ONE `template_utility_ranked` SSB beacon row per template, and prints a JSON
//! list of `{concept_class, utility, first_try_ratio, sample_n,
//! chosen_pipeline_id}`.
//!
//! Both `utility` (the smoothed score that now gates reuse at the
//! `try_execute` seam) and `first_try_ratio` (the raw recorded baseline kept
//! under W-2 no-removal) are printed so the before/after is directly
//! measurable. A 1/1 template scores `2/3 ~= 0.667` rather than `1.0`, so a
//! low-sample template stops being over-trusted relative to a 95/100 one
//! (`~0.941`).
//!
//! Empty store (e.g. a fresh process whose in-memory crystallizer
//! observation_log has not been persisted) is NOT a failure: print an empty
//! list and emit NOTHING (no false beacon row).
//!
//! Default-ON: the SSB emit runs unless the [`ELAI_TEMPLATE_CBR_DISABLED_ENV`]
//! kill-switch is set truthy. There is intentionally NO default-off `*_ENABLE`
//! flag (W-2: the live emit path must not be wire-dark by default). The list
//! itself always prints regardless of the kill-switch; only the durable beacon
//! emission is gated.
//!
//! The Springdrift +10.7% figure is UNGROUNDED and is deliberately NOT a
//! target: F-7 gates on the wiring done-gate + smoothing-correctness, not a
//! numeric uplift.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;

use elai_core::OsClock;
use elai_orchestrator::compiled_pipeline::CompiledPipelineStore;
use elai_orchestrator::template_utility::laplace_utility_from_fields;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload, TemplateUtilityRankedPayload};

/// Kill-switch env var for the F-7 template-ranking SSB emit. Default-ON: the
/// beacon is written unless this var is set truthy. There is NO default-off
/// enable flag (W-2).
pub(crate) const ELAI_TEMPLATE_CBR_DISABLED_ENV: &str = "ELAI_TEMPLATE_CBR_DISABLED";

/// One ranked template row, printed as JSON and carried on the
/// `template_utility_ranked` beacon.
///
/// `utility` is the Laplace-smoothed score (the value that gates reuse);
/// `first_try_ratio` is the raw recorded baseline (W-2 no-removal). Both are
/// surfaced so the before/after is measurable.
#[derive(Debug, Clone, Serialize)]
struct TemplateRankingRow {
    concept_class: String,
    utility: f32,
    first_try_ratio: f32,
    sample_n: u32,
    chosen_pipeline_id: String,
    /// The store-key plan-skeleton hash, carried for the beacon's
    /// `plan_skeleton_hash` field. NOT printed in the JSON list (the brief's
    /// list shape is the 5 fields above); `#[serde(skip)]` keeps the JSON
    /// contract stable while still carrying the REAL hash to the beacon (a 0
    /// here would be a CSQ-class lie — the hash is available from the store key
    /// so there is no excuse to zero-fill it).
    #[serde(skip)]
    plan_skeleton_hash: u64,
}

/// Sync entrypoint for `elai template-ranking <action> [--json]`.
///
/// `action` is the lower-cased subcommand (`"list"` | None => list). The store
/// load is blocking std::fs; the SSB emit owns a current-thread tokio runtime
/// (the `Ssb::new` no-reactor gotcha is handled inside [`emit_ranking_rows`]).
pub(crate) fn run_template_ranking_subcommand(
    action: Option<&str>,
    json: bool,
    project_root: &Path,
) -> Result<()> {
    let normalized = action.unwrap_or("list");
    match normalized {
        "list" | "show" => run_list(json, project_root),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(anyhow::anyhow!(
            "unknown `template-ranking` action '{other}'. Use list or help."
        )),
    }
}

fn run_list(json: bool, project_root: &Path) -> Result<()> {
    let cache_path = CompiledPipelineStore::standard_path(project_root);
    let mut store = CompiledPipelineStore::new(cache_path.clone());
    store.load_from_disk();
    let entries = store.entries();

    // Compute the smoothed utility per template and sort DESC by utility. An
    // empty store yields an empty list + NO beacon — that is correct (a fresh
    // process whose crystallizer log was never persisted has nothing to rank),
    // not a failure.
    let mut rows: Vec<TemplateRankingRow> = entries
        .values()
        .map(|p| TemplateRankingRow {
            concept_class: p.concept_class.clone(),
            utility: laplace_utility_from_fields(p.first_try_success_rate, p.use_count),
            first_try_ratio: p.first_try_success_rate,
            sample_n: p.use_count,
            chosen_pipeline_id: p.pipeline_id.to_string(),
            plan_skeleton_hash: p.plan_skeleton_hash,
        })
        .collect();
    rows.sort_by(|a, b| {
        b.utility
            .partial_cmp(&a.utility)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // W-5 live trigger: emit one durable beacon row per ranked template
    // (default-ON; kill-switch gated). Empty list => no rows emitted.
    if !rows.is_empty() && !is_env_truthy(ELAI_TEMPLATE_CBR_DISABLED_ENV) {
        emit_ranking_rows(&rows, project_root)
            .map_err(|e| anyhow::anyhow!("template-ranking: SSB emit failed: {e}"))?;
    }

    if json {
        let out = serde_json::to_string_pretty(&rows)
            .map_err(|e| anyhow::anyhow!("template-ranking: json serialise failed: {e}"))?;
        println!("{out}");
    } else if rows.is_empty() {
        println!(
            "No compiled pipeline templates to rank at {}.",
            cache_path.display()
        );
    } else {
        println!(
            "Template ranking (Laplace-smoothed utility) at {} (count={}):",
            cache_path.display(),
            rows.len()
        );
        for r in &rows {
            // utility (the gating score) printed alongside first_try_ratio (the
            // raw baseline) — the before/after compare.
            println!(
                "  concept_class={concept:<18} utility={util:.3} first_try_ratio={ratio:.3} sample_n={n:<4} pipeline_id={id}",
                concept = r.concept_class,
                util = r.utility,
                ratio = r.first_try_ratio,
                n = r.sample_n,
                id = r.chosen_pipeline_id,
            );
        }
    }
    Ok(())
}

/// Open the canonical session `ssb.db` and write one `template_utility_ranked`
/// beacon per ranked template, awaited (durable).
///
/// Mirrors the `safety_cmd::write_tool_filter_ssb_rows` runtime pattern:
/// [`Ssb::new`] spawns the single-writer task and MUST be constructed INSIDE
/// `rt.block_on`, else it panics "there is no reactor running" (gotcha:110).
fn emit_ranking_rows(rows: &[TemplateRankingRow], project_root: &Path) -> Result<(), String> {
    let db_path = sessions_db_path(project_root)?;
    let parent = db_path
        .parent()
        .ok_or_else(|| "template-ranking: ssb.db has no parent dir".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("template-ranking: create sessions dir failed: {e}"))?;

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-template-ranking".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("template-ranking: build async runtime failed: {e}"))?;

    // Ssb::new spawns the single-writer task, so it MUST be constructed inside
    // the runtime context. Building it before rt.block_on panics with
    // "there is no reactor running, must be called from the context of a Tokio
    // 1.x runtime" (gotcha:110).
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| format!("template-ranking: open SSB writer failed: {e}"))?;
        for r in rows {
            let payload = SsbPayload::TemplateUtilityRanked(TemplateUtilityRankedPayload {
                concept_class: r.concept_class.clone(),
                // REAL store-key hash (not 0) — sourced from the persisted
                // CompiledPipeline.plan_skeleton_hash via the row. Zero-filling
                // an absent signal is the CSQ footgun F-7 explicitly forbids;
                // here the hash IS present, so it is carried faithfully.
                plan_skeleton_hash: r.plan_skeleton_hash,
                utility: r.utility,
                first_try_ratio: r.first_try_ratio,
                sample_n: r.sample_n,
                chosen_pipeline_id: r.chosen_pipeline_id.clone(),
            });
            ssb.write_payload("cli:template-ranking", 0, payload, Vec::new())
                .await
                .map_err(|e| format!("template-ranking: SSB write failed: {e}"))?;
        }
        Ok::<(), String>(())
    })
}

/// Resolve the canonical `<project_root>/workflow/sessions/ssb.db` path the CLI
/// SSB emitters write to. Mirrors `safety_cmd`'s `sessions_dir_path(...).join("ssb.db")`.
fn sessions_db_path(project_root: &Path) -> Result<PathBuf, String> {
    Ok(elai_core::paths::sessions_dir_path(project_root).join("ssb.db"))
}

fn is_env_truthy(var: &str) -> bool {
    std::env::var(var)
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on" | "TRUE" | "YES" | "ON"))
        .unwrap_or(false)
}

fn print_help() {
    println!("Usage: elai template-ranking [list|help] [--json]");
    println!("  list   Rank compiled-pipeline templates by Laplace-smoothed");
    println!("         outcome utility (default). Prints utility alongside the");
    println!("         raw first_try_ratio baseline + emits a template_utility_ranked");
    println!("         SSB row per template.");
    println!("  help   This message.");
    println!("  --json Emit the ranking as a JSON list.");
}
