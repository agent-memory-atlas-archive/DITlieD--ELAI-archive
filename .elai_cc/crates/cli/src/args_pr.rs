// Tranche O.3 (T10): PR parser block extracted from args.rs.
// Hosts parse_pr_subcommand_args, parse_pr_review_subcommand_args,
// and the require_value helper (used only by pr-review).

#![allow(dead_code)]

use crate::args::CliAction;

pub(crate) fn parse_pr_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            "missing `pr` sub-command (expected one of: open, list, status, close)".to_string()
        })?
        .to_ascii_lowercase();
    if !matches!(sub.as_str(), "open" | "list" | "status" | "close") {
        return Err(format!(
            "unknown `pr` sub-command: '{sub}' (expected one of: open, list, status, close)"
        ));
    }

    let mut repo: Option<String> = None;
    let mut title: Option<String> = None;
    let mut body: Option<String> = None;
    let mut head: Option<String> = None;
    let mut base: Option<String> = None;
    let mut draft = false;
    let mut pr_number: Option<u32> = None;
    let mut state_filter: Option<String> = None;
    let mut limit: u32 = 0;
    let mut json = false;

    let mut i = 1;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--repo" => {
                repo = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --repo".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--title" => {
                title = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --title".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--body" => {
                body = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --body".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--head" => {
                head = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --head".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--base" => {
                base = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --base".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--draft" => {
                draft = true;
                i += 1;
            }
            "--state" => {
                state_filter = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --state".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--limit" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --limit".to_string())?;
                limit = v
                    .parse::<u32>()
                    .map_err(|e| format!("invalid --limit value '{v}': {e}"))?;
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other if !other.starts_with("--") && pr_number.is_none() => {
                // Positional PR number for status / close.
                pr_number = Some(
                    other
                        .parse::<u32>()
                        .map_err(|e| format!("invalid PR number '{other}': {e}"))?,
                );
                i += 1;
            }
            other => {
                return Err(format!("unknown `pr {sub}` argument: {other}"));
            }
        }
    }

    Ok(CliAction::Pr {
        sub,
        repo,
        title,
        body,
        head,
        base,
        draft,
        pr_number,
        state_filter,
        limit,
        json,
    })
}

/// E-41 — parse `elai pr-review <sub> [args]`.
///
/// Sub-action is validated downstream in `pr_review_cmd::run_pr_review_subcommand`.
/// Flag parsing is permissive: per-subverb required flags are checked by
/// the handler so the args layer stays free of orchestrator types.
pub(crate) fn parse_pr_review_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .map(String::as_str)
        .ok_or_else(|| "pr-review requires a sub-command".to_string())?
        .to_ascii_lowercase();
    let mut pr: Option<u32> = None;
    let mut body: Option<String> = None;
    let mut file: Option<String> = None;
    let mut line: Option<u32> = None;
    let mut template: Option<String> = None;
    let mut kv_pairs: Vec<(String, String)> = Vec::new();
    let mut since: Option<i64> = None;
    let mut ruleset: Option<String> = None;
    let mut loc_added: Option<u32> = None;
    let mut loc_removed: Option<u32> = None;
    let mut files_changed: Option<u32> = None;
    let mut touches_owned = false;
    let mut repo: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut touched_files: Vec<String> = Vec::new();
    let mut default_owner: Option<String> = None;
    let mut title: Option<String> = None;
    let mut json = false;

    let mut i = 1;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--json" => {
                json = true;
                i += 1;
            }
            "--touches-owned" => {
                touches_owned = true;
                i += 1;
            }
            "--body" => {
                body = Some(require_value(rest, i, "--body")?);
                i += 2;
            }
            "--file" => {
                file = Some(require_value(rest, i, "--file")?);
                i += 2;
            }
            "--line" => {
                let v = require_value(rest, i, "--line")?;
                line = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --line '{v}': {e}"))?,
                );
                i += 2;
            }
            "--template" => {
                template = Some(require_value(rest, i, "--template")?);
                i += 2;
            }
            "--kv" => {
                let v = require_value(rest, i, "--kv")?;
                let (k, val) = v
                    .split_once('=')
                    .ok_or_else(|| format!("--kv expects key=value, got '{v}'"))?;
                kv_pairs.push((k.to_string(), val.to_string()));
                i += 2;
            }
            "--since" => {
                let v = require_value(rest, i, "--since")?;
                since = Some(
                    v.parse::<i64>()
                        .map_err(|e| format!("invalid --since '{v}': {e}"))?,
                );
                i += 2;
            }
            "--ruleset" => {
                ruleset = Some(require_value(rest, i, "--ruleset")?);
                i += 2;
            }
            "--loc-added" => {
                let v = require_value(rest, i, "--loc-added")?;
                loc_added = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --loc-added '{v}': {e}"))?,
                );
                i += 2;
            }
            "--loc-removed" => {
                let v = require_value(rest, i, "--loc-removed")?;
                loc_removed = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --loc-removed '{v}': {e}"))?,
                );
                i += 2;
            }
            "--files-changed" => {
                let v = require_value(rest, i, "--files-changed")?;
                files_changed = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --files-changed '{v}': {e}"))?,
                );
                i += 2;
            }
            "--repo" => {
                repo = Some(require_value(rest, i, "--repo")?);
                i += 2;
            }
            "--branch" => {
                branch = Some(require_value(rest, i, "--branch")?);
                i += 2;
            }
            "--touched" => {
                touched_files.push(require_value(rest, i, "--touched")?);
                i += 2;
            }
            "--default-owner" => {
                default_owner = Some(require_value(rest, i, "--default-owner")?);
                i += 2;
            }
            "--title" => {
                title = Some(require_value(rest, i, "--title")?);
                i += 2;
            }
            other if !other.starts_with("--") && pr.is_none() => {
                pr = Some(
                    other
                        .parse::<u32>()
                        .map_err(|e| format!("invalid PR number '{other}': {e}"))?,
                );
                i += 1;
            }
            other => {
                return Err(format!("unknown `pr-review {sub}` argument: {other}"));
            }
        }
    }

    Ok(CliAction::PrReview {
        sub,
        pr,
        body,
        file,
        line,
        template,
        kv_pairs,
        since,
        ruleset,
        loc_added,
        loc_removed,
        files_changed,
        touches_owned,
        repo,
        branch,
        touched_files,
        default_owner,
        title,
        json,
    })
}

fn require_value(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}
