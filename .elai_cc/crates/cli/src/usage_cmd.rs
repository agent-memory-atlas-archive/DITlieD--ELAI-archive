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

/// Renders `cost_micros` as a dollar string, or an explicit `n/a` marker
/// when the group's cost is `None` (S-02 Option-semantics: every folded
/// row was an `EstimateApplied` token-only estimate, so no dollar figure
/// is trustworthy for this group -- tokens-only display, S-12 rule).
fn cost_display(cost_micros: Option<u64>) -> String {
    cost_micros.map_or_else(
        || "n/a (tokens only)".to_string(),
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
            cost_display(entry.cost_micros),
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
    let rows_json: Vec<String> = rollup
        .iter()
        .map(|(key, entry)| {
            let cost_json = entry
                .cost_micros
                .map_or_else(|| "null".to_string(), |m| m.to_string());
            format!(
                r#"{{"lane":"{lane}","identity":"{identity}","model":"{model}","task_id":{task},"session_id":{session},"input_tokens":{input},"output_tokens":{output},"cache_creation_tokens":{cache_creation},"cache_read_tokens":{cache_read},"cost_micros":{cost},"row_count":{rows}}}"#,
                lane = lane_label(key.lane),
                identity = key.identity,
                model = key.model,
                task = key
                    .task_id
                    .as_ref()
                    .map_or_else(|| "null".to_string(), |t| format!(r#""{t}""#)),
                session = key
                    .session_id
                    .as_ref()
                    .map_or_else(|| "null".to_string(), |s| format!(r#""{s}""#)),
                input = entry.input_tokens,
                output = entry.output_tokens,
                cache_creation = entry.cache_creation_tokens,
                cache_read = entry.cache_read_tokens,
                cost = cost_json,
                rows = entry.row_count,
            )
        })
        .collect();
    println!("[{}]", rows_json.join(","));
}
