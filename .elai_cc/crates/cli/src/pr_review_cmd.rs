//! E-41: CLI dispatcher for `elai pr review <action>`.
//!
//! Provides the production callers for every E-41 github_pr extension
//! module — closes the WIRE-DARK gap so each new primitive
//! (PrBodyComposer, ReviewCommentWatcher, verify_package, semgrep_gate,
//! auto_merge_guard, codeowners_gen, git_assist) has at least one
//! non-test caller exercised from the binary.
//!
//! Sync entry — owns its own current-thread tokio runtime (mirrors
//! `github_pr_cmd::run_pr_subcommand`). The SSB writer is shared across
//! all subverbs so emitted rows land in the same DB as the C-40
//! github_pr surface.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::github_pr::{
    apply_branch_protection, assess_merge_safety, compose_pr_body, detect_force_push,
    generate_codeowners, post_review_comment, run_semgrep_gate, suggest_squash_message,
    verify_pr_deps_no_slopsquat, watch_comments, MergePolicy, MergeVerdict, PrContext, PrShape,
};
use elai_orchestrator::{Ssb, SsbConfig};
use serde_json::json;

/// Subverb selector for `elai pr review`.
#[derive(Debug, Clone)]
pub enum PrReviewAction {
    /// Compose a PR body from a template + key=value context.
    ComposeBody {
        pr: u32,
        template: String,
        kv_pairs: Vec<(String, String)>,
    },
    /// Post a review comment to a PR.
    Comment {
        pr: u32,
        body: String,
        file: Option<String>,
        line: Option<u32>,
    },
    /// Poll for new comments since a millisecond watermark.
    Watch { pr: u32, since: Option<i64> },
    /// Run anti-slopsquat verification on the PR's added deps.
    VerifyDeps { pr: u32 },
    /// Run a semgrep gate over the working copy.
    SemgrepGate { pr: u32, ruleset: String },
    /// Assess auto-merge safety against the default policy.
    AutoMergeAssess {
        pr: u32,
        loc_added: u32,
        loc_removed: u32,
        files_changed: u32,
        touches_owned: bool,
    },
    /// Apply ELAI-recommended branch protection settings.
    ApplyBranchProtection { repo: String, branch: String },
    /// Generate a CODEOWNERS body for the supplied touched files.
    GenCodeowners {
        touched: Vec<PathBuf>,
        default_owner: String,
    },
    /// Suggest a maintainer-voice squash message from a PR title.
    SuggestSquash { title: String, body_hint: String },
    /// Detect force-push events on a PR.
    DetectForcePush { pr: u32, repo: Option<String> },
}

/// Entry point invoked from `main.rs::CliAction::PrReview`. `json`
/// controls whether output is machine-readable.
pub fn run_pr_review_subcommand(action: PrReviewAction, json: bool) -> Result<()> {
    // Each branch picks between sync-pure and async-shellout helpers.
    // The pure branches do not need a runtime; the async ones build one.
    match action {
        PrReviewAction::ComposeBody {
            pr,
            template,
            kv_pairs,
        } => {
            let mut ctx = PrContext::new();
            for (k, v) in kv_pairs {
                ctx = ctx.with(k, v);
            }
            let body = compose_pr_body(&template, &ctx).map_err(anyhow::Error::from)?;
            if json {
                println!("{}", json!({ "pr": pr, "body": body }));
            } else {
                println!("{body}");
            }
            Ok(())
        }
        PrReviewAction::SuggestSquash { title, body_hint } => {
            let msg = suggest_squash_message(&title, &body_hint);
            if json {
                println!("{}", json!({ "message": msg }));
            } else {
                println!("{msg}");
            }
            Ok(())
        }
        PrReviewAction::GenCodeowners {
            touched,
            default_owner,
        } => {
            let body = generate_codeowners(&touched, &default_owner);
            if json {
                println!("{}", json!({ "body": body, "count": touched.len() }));
            } else {
                println!("{body}");
            }
            Ok(())
        }
        // All remaining subverbs need an async runtime + SSB writer.
        other => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("building tokio runtime for `elai pr review`")?;
            rt.block_on(async move { run_async(other, json).await })
        }
    }
}

async fn run_async(action: PrReviewAction, json: bool) -> Result<()> {
    let ssb = build_ssb().await.context("opening SSB writer")?;
    match action {
        PrReviewAction::Comment {
            pr,
            body,
            file,
            line,
        } => {
            post_review_comment(Arc::clone(&ssb), pr, &body, file, line)
                .await
                .map_err(anyhow::Error::from)?;
            if json {
                println!("{}", json!({ "pr": pr, "posted": true }));
            } else {
                println!("Comment posted to PR #{pr}.");
            }
            Ok(())
        }
        PrReviewAction::Watch { pr, since } => {
            let since_ms = since.unwrap_or(0);
            let comments = watch_comments(Arc::clone(&ssb), pr, since_ms)
                .await
                .map_err(anyhow::Error::from)?;
            if json {
                let rows: Vec<_> = comments
                    .iter()
                    .map(|c| {
                        json!({
                            "id": c.id,
                            "created_at_ms": c.created_at_ms,
                            "author": c.author,
                        })
                    })
                    .collect();
                println!("{}", json!({ "pr": pr, "new_comments": rows }));
            } else {
                println!("New comments on PR #{pr}: {}", comments.len());
                for c in &comments {
                    println!("  {} {} {}", c.created_at_ms, c.author, c.id);
                }
            }
            Ok(())
        }
        PrReviewAction::VerifyDeps { pr } => {
            let report = verify_pr_deps_no_slopsquat(Arc::clone(&ssb), pr)
                .await
                .map_err(anyhow::Error::from)?;
            if json {
                println!(
                    "{}",
                    json!({
                        "pr": pr,
                        "flagged": report.flagged,
                        "total_inspected": report.total_inspected,
                    })
                );
            } else {
                println!(
                    "PR #{pr} deps: inspected={} flagged={}",
                    report.total_inspected,
                    report.flagged.len()
                );
                for f in &report.flagged {
                    println!("  flag: {f}");
                }
            }
            Ok(())
        }
        PrReviewAction::SemgrepGate { pr, ruleset } => {
            let result = run_semgrep_gate(Arc::clone(&ssb), pr, &ruleset)
                .await
                .map_err(anyhow::Error::from)?;
            if json {
                println!(
                    "{}",
                    json!({
                        "pr": pr,
                        "findings_count": result.findings_count,
                        "severity_max": result.severity_max.label(),
                    })
                );
            } else {
                println!(
                    "PR #{pr} semgrep: findings={} severity_max={}",
                    result.findings_count,
                    result.severity_max.label(),
                );
            }
            Ok(())
        }
        PrReviewAction::AutoMergeAssess {
            pr,
            loc_added,
            loc_removed,
            files_changed,
            touches_owned,
        } => {
            let shape = PrShape {
                loc_added,
                loc_removed,
                files_changed,
                touches_owned_paths: touches_owned,
                is_machine_generated_reversible: false,
            };
            let policy = MergePolicy::default();
            let verdict = assess_merge_safety(Arc::clone(&ssb), pr, &shape, &policy).await;
            let verdict_label = match verdict {
                MergeVerdict::Allow => "allow".to_string(),
                MergeVerdict::Block { reason_label } => format!("block:{reason_label}"),
            };
            if json {
                println!("{}", json!({ "pr": pr, "verdict": verdict_label }));
            } else {
                println!("PR #{pr} auto-merge verdict: {verdict_label}");
            }
            Ok(())
        }
        PrReviewAction::ApplyBranchProtection { repo, branch } => {
            apply_branch_protection(&repo, &branch)
                .await
                .map_err(anyhow::Error::from)?;
            if json {
                println!("{}", json!({ "repo": repo, "branch": branch, "applied": true }));
            } else {
                println!("Branch protection applied to {repo}/{branch}.");
            }
            Ok(())
        }
        PrReviewAction::DetectForcePush { pr, repo } => {
            let found = detect_force_push(Arc::clone(&ssb), pr, repo.as_deref())
                .await
                .map_err(anyhow::Error::from)?;
            if json {
                println!("{}", json!({ "pr": pr, "force_pushed": found }));
            } else {
                println!("PR #{pr} force-push detected: {found}");
            }
            Ok(())
        }
        // Pure branches dispatched above; should not appear here.
        PrReviewAction::ComposeBody { .. }
        | PrReviewAction::SuggestSquash { .. }
        | PrReviewAction::GenCodeowners { .. } => {
            Err(anyhow!("internal dispatch error: pure action reached async path"))
        }
    }
}

/// Open an SSB writer bound to the active project sessions DB.
async fn build_ssb() -> Result<Arc<Ssb>> {
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
        session_id: "cli-pr-review".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(cfg, clock).context("opening SSB writer")?;
    Ok(Arc::new(ssb))
}
