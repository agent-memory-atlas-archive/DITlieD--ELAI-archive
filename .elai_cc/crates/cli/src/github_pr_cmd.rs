//! C-40: CLI dispatcher for `elai pr <open|list|status|close>`.
//!
//! Sync entry — owns its own current-thread tokio runtime (mirrors
//! `archive_verify::run` + `wip_squash_subcommand::run_wip_squash_subcommand`).
//! Each subverb constructs a `GhCliAdapter` over the active project's
//! SSB writer, awaits the trait call, and prints either a human-readable
//! summary (default) or a machine-readable JSON line (`--json`).
//!
//! The handler does NOT bypass risk classification — `gh pr create`,
//! `gh pr close`, and `gh pr merge` are already classified
//! `RiskLevel::SharedState` by `tools::action_classifier`
//! (BASH_SHARED_STATE_PATTERNS at action_classifier.rs:128-141). The
//! orchestrator's existing gate machinery applies; this handler only
//! handles the user-facing presentation layer.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::github_pr::{
    GhCliAdapter, GithubPrAdapter, PrOpenRequest, PrState, PrStatus, PrSummary,
};
use elai_orchestrator::{Ssb, SsbConfig};
use serde_json::json;

/// Entry point invoked from `main.rs::CliAction::Pr`. Returns
/// `Ok(())` on success or an anyhow error on any failure (CLI dispatcher
/// surfaces the error message to stderr and exits 1). Sync entry — owns
/// its own tokio runtime.
#[allow(clippy::too_many_arguments)]
pub fn run_pr_subcommand(
    sub: String,
    repo: Option<String>,
    title: Option<String>,
    body: Option<String>,
    head: Option<String>,
    base: Option<String>,
    draft: bool,
    pr_number: Option<u32>,
    state_filter: Option<String>,
    limit: u32,
    json: bool,
) -> Result<()> {
    // Build a current-thread tokio runtime — gh shell-out is async via
    // tokio::process::Command, and the SSB writer requires a runtime
    // context for write_payload. Sync entry pattern matches
    // archive_verify::run + wip_squash_subcommand::run.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("building tokio runtime for `elai pr`")?;

    rt.block_on(async move {
        let adapter = build_adapter()
            .await
            .context("constructing GhCliAdapter")?;
        match sub.as_str() {
            "open" => {
                let title = title.ok_or_else(|| anyhow!("--title is required for `pr open`"))?;
                let head = head.ok_or_else(|| anyhow!("--head is required for `pr open`"))?;
                let req = PrOpenRequest {
                    repo,
                    title,
                    body: body.unwrap_or_default(),
                    head,
                    base,
                    draft,
                };
                let summary = adapter.open(req).await.map_err(anyhow::Error::from)?;
                print_open_result(&summary, json);
                Ok(())
            }
            "list" => {
                let summaries = adapter
                    .list(repo, state_filter, limit)
                    .await
                    .map_err(anyhow::Error::from)?;
                print_list_result(&summaries, json);
                Ok(())
            }
            "status" => {
                let pr = pr_number.ok_or_else(|| {
                    anyhow!("PR number positional argument is required for `pr status`")
                })?;
                let status = adapter
                    .status(repo, pr)
                    .await
                    .map_err(anyhow::Error::from)?;
                print_status_result(pr, &status, json);
                Ok(())
            }
            "close" => {
                let pr = pr_number.ok_or_else(|| {
                    anyhow!("PR number positional argument is required for `pr close`")
                })?;
                adapter
                    .close(repo, pr)
                    .await
                    .map_err(anyhow::Error::from)?;
                if json {
                    println!("{}", json!({"pr_number": pr, "closed": true}));
                } else {
                    println!("PR #{pr} closed.");
                }
                Ok(())
            }
            other => Err(anyhow!(
                "unknown `pr` sub-command '{other}' (open|list|status|close)"
            )),
        }
    })
}

/// Build the GhCliAdapter against the current project's SSB DB.
/// Resolves the SSB path via `core::paths::sessions_dir_path` so the
/// same DB that the dashboard reads is the one we write to. `Ssb::new`
/// is itself synchronous but must be called inside the tokio runtime
/// because it spawns the single-writer task.
async fn build_adapter() -> Result<GhCliAdapter> {
    let project_root: PathBuf =
        std::env::current_dir().context("cannot determine current working directory")?;
    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    let parent = db_path
        .parent()
        .ok_or_else(|| anyhow!("ssb.db has no parent dir; cannot prepare path"))?;
    std::fs::create_dir_all(parent).context("creating ssb sessions dir")?;
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-pr".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(cfg, clock).context("opening SSB writer")?;
    Ok(GhCliAdapter::with_agent_id(
        Arc::new(ssb),
        "cli:pr".to_string(),
    ))
}

fn print_open_result(summary: &PrSummary, json: bool) {
    if json {
        println!(
            "{}",
            json!({
                "number": summary.number,
                "title": summary.title,
                "state": pr_state_label(summary.state),
                "url": summary.url,
                "head": summary.head_ref_name,
                "base": summary.base_ref_name,
                "draft": summary.is_draft,
            })
        );
    } else {
        println!("PR opened");
        println!("  Number  #{}", summary.number);
        println!("  Title   {}", summary.title);
        println!("  URL     {}", summary.url);
        println!("  Head    {}", summary.head_ref_name);
        if !summary.base_ref_name.is_empty() {
            println!("  Base    {}", summary.base_ref_name);
        }
        if summary.is_draft {
            println!("  Draft   yes");
        }
    }
}

fn print_list_result(summaries: &[PrSummary], json: bool) {
    if json {
        let rows: Vec<_> = summaries
            .iter()
            .map(|s| {
                json!({
                    "number": s.number,
                    "title": s.title,
                    "state": pr_state_label(s.state),
                    "url": s.url,
                    "head": s.head_ref_name,
                    "base": s.base_ref_name,
                    "draft": s.is_draft,
                })
            })
            .collect();
        println!("{}", serde_json::Value::Array(rows));
    } else if summaries.is_empty() {
        println!("(no PRs matched)");
    } else {
        for s in summaries {
            let draft_tag = if s.is_draft { " [draft]" } else { "" };
            println!(
                "#{:<5} {:<10} {:<8} {}{}",
                s.number,
                pr_state_label(s.state),
                s.head_ref_name,
                s.title,
                draft_tag,
            );
        }
    }
}

fn print_status_result(pr: u32, status: &PrStatus, json: bool) {
    if json {
        println!(
            "{}",
            json!({
                "pr_number": pr,
                "state": pr_state_label(status.state),
                "review_decision": status.review_decision,
                "ci_rollup_label": status.ci_rollup_label(),
                "review_state_label": status.review_state_label(),
                "check_runs": status.check_runs.iter().map(|c| json!({
                    "name": c.name,
                    "status": c.status,
                    "conclusion": c.conclusion,
                })).collect::<Vec<_>>(),
            })
        );
    } else {
        println!("PR #{pr} status");
        println!("  State          {}", pr_state_label(status.state));
        let review = if status.review_decision.is_empty() {
            "(none)"
        } else {
            status.review_decision.as_str()
        };
        println!("  Review         {review}");
        println!("  CI rollup      {}", status.ci_rollup_label());
        if !status.check_runs.is_empty() {
            println!("  Check runs");
            for c in &status.check_runs {
                let conclusion = if c.conclusion.is_empty() {
                    "(pending)"
                } else {
                    c.conclusion.as_str()
                };
                println!("    {:<24} {:<14} {}", c.name, c.status, conclusion);
            }
        }
    }
}

fn pr_state_label(state: PrState) -> &'static str {
    match state {
        PrState::Open => "open",
        PrState::Closed => "closed",
        PrState::Merged => "merged",
        PrState::Unknown => "unknown",
    }
}
