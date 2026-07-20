//! J-28 T8 — `elai usage` CLI handler.
//!
//! Reads `usage_turn_event` SSB rows from the canonical `ssb.db`, folds
//! them via `elai_orchestrator::usage_ledger::rollup_usage_turns` into a
//! per-(model,task,session,lane) grouped total, and prints cloud and local
//! totals SEPARATELY LABELED (stub_ban: no naive cloud+local sum).
//!
//! SSB read path: `usage_turn_event` rows are written at TWO production
//! seams -- `grounding_client.rs::CloudGroundingClient::send_message`
//! (cloud lane) and `runtime_orchestrator.rs::RuntimeOrchestrator::try_complete`
//! (local lane). This is the sole production caller of
//! `usage_ledger::rollup_usage_turns` (R7: a unit-test caller alone would
//! leave the rollup wire-dark).
//!
//! Grounding: docs/thefinalglance/J-plan/J-28_token_ledger_usage_unification.md

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::format_usd;
use elai_orchestrator::ssb::UsageLane;
use elai_orchestrator::ssb_viewer::SsbViewer;
use elai_orchestrator::usage_ledger::{rollup_usage_turns, UsageRollupEntry, UsageRollupKey};

/// Rows scanned per `elai usage` invocation. Mirrors the bound already
/// enforced by `SsbViewer::recent_rows_of_type`'s `DRAWER_QUERY_HARD_CAP`;
/// stated here explicitly so a caller reading this file sees the ceiling
/// without chasing it into ssb_viewer.rs.
const USAGE_SCAN_LIMIT: usize = 10_000;

/// Run `elai usage [--json]`.
///
/// Resolves the project root from `std::env::current_dir()`. Opens the
/// canonical `ssb.db` read-only; returns cleanly when the file does not
/// exist (no usage rows recorded yet).
pub fn run_usage(json: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("could not determine cwd: {e}"))?;

    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    if !db_path.exists() {
        if json {
            println!("[]");
        } else {
            println!("usage: no ssb.db found — no usage rows recorded yet.");
        }
        return Ok(());
    }

    let viewer = SsbViewer::open(&db_path)
        .map_err(|e| anyhow::anyhow!("usage: could not open ssb.db: {e}"))?;
    let rows = viewer
        .usage_turn_events(USAGE_SCAN_LIMIT)
        .map_err(|e| anyhow::anyhow!("usage: could not read usage_turn_event rows: {e}"))?;
    let rollup = rollup_usage_turns(&rows);

    if json {
        print_json(&rollup);
    } else {
        print_table(&rollup);
    }
    Ok(())
}

fn lane_label(lane: UsageLane) -> &'static str {
    match lane {
        UsageLane::Cloud => "cloud",
        UsageLane::Local => "local",
    }
}

/// Renders the cost column for a rollup row. Local lanes have no dollar
/// figure (S-02: every folded row was an `EstimateApplied` token-only
/// estimate), so the column shows the raw token split `in {N} / out {M}`
/// instead of fabricating a USD value. Cloud lanes keep `format_usd` when
/// `cost_micros` is present, or an explicit `n/a (no rate for {model})`
/// when the provider returned no rate for that model.
fn cost_display(lane: UsageLane, model: &str, entry: &UsageRollupEntry) -> String {
    if lane == UsageLane::Local {
        return format!("in {} / out {}", entry.input_tokens, entry.output_tokens);
    }
    entry.cost_micros.map_or_else(
        || format!("n/a (no rate for {model})"),
        // u64 -> f64 is a widening conversion (no truncation risk for any
        // realistic dollar-micros value; f64's 52-bit mantissa covers up
        // to ~9e15 exactly, far past any plausible cost).
        |micros| format_usd((micros as f64) / 1_000_000.0),
    )
}

fn print_table(rollup: &std::collections::BTreeMap<UsageRollupKey, UsageRollupEntry>) {
    if rollup.is_empty() {
        println!("No usage rows recorded yet (elai run / cloud+local turns emit usage_turn_event).");
        return;
    }
    println!(
        "{:<8} {:<8} {:<24} {:<14} {:<14} {:>10} {:>10} {:>14}",
        "LANE", "IDENTITY", "MODEL", "TASK", "SESSION", "INPUT_TOK", "OUTPUT_TOK", "COST"
    );
    println!("{}", "-".repeat(110));
    for (key, entry) in rollup {
        println!(
            "{:<8} {:<8} {:<24} {:<14} {:<14} {:>10} {:>10} {:>14}",
            lane_label(key.lane),
            key.identity,
            key.model,
            key.task_id.as_deref().unwrap_or("-"),
            key.session_id.as_deref().unwrap_or("-"),
            entry.input_tokens,
            entry.output_tokens,
            cost_display(key.lane, &key.model, entry),
        );
    }

    // Cloud vs local totals, SEPARATELY LABELED (stub_ban: no naive sum).
    let cloud_input: u64 = rollup
        .iter()
        .filter(|(k, _)| k.lane == UsageLane::Cloud)
        .map(|(_, v)| v.input_tokens + v.output_tokens)
        .sum();
    let local_input: u64 = rollup
        .iter()
        .filter(|(k, _)| k.lane == UsageLane::Local)
        .map(|(_, v)| v.input_tokens + v.output_tokens)
        .sum();
    println!(
        "\nTotals (never summed across lanes)\n  cloud total tokens  {cloud_input}\n  local total tokens  {local_input}"
    );
}

fn print_json(rollup: &std::collections::BTreeMap<UsageRollupKey, UsageRollupEntry>) {
    let rows_json: Vec<serde_json::Value> = rollup
        .iter()
        .map(|(key, entry)| {
            serde_json::json!({
                "lane": lane_label(key.lane),
                "identity": key.identity,
                "model": key.model,
                "task_id": key.task_id,
                "session_id": key.session_id,
                "input_tokens": entry.input_tokens,
                "output_tokens": entry.output_tokens,
                "cache_creation_tokens": entry.cache_creation_tokens,
                "cache_read_tokens": entry.cache_read_tokens,
                "cost_micros": entry.cost_micros,
                "row_count": entry.row_count,
            })
        })
        .collect();
    println!("{}", serde_json::Value::Array(rows_json));
}
