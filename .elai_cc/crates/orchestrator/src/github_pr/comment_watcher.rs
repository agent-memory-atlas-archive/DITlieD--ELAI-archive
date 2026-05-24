//! E-41: PR comment poller.
//!
//! Polls `gh pr view --json comments` for new comments since a given
//! timestamp. Pure function shape: takes inputs, returns the parsed list
//! of new comments, emits one SSB row per call summarising the new-count.
//!
//! There is no long-lived background JoinHandle in this module — the
//! ReviewCommentWatcher orchestration loop deferred from C-40 stays
//! deferred. E-41 ships a per-call poll that the CLI / dashboard can
//! invoke on demand (closes the WIRE-DARK gap: CLI subverb is the
//! production caller).

use std::sync::Arc;

use serde::Deserialize;
use tokio::process::Command;
use tracing::warn;

use crate::ssb::{PrCommentWatchedPayload, Ssb, SsbPayload};

use super::error::GithubPrError;
use super::gh_cli::GhCliAdapter;

/// Stable agent_id label for SSB rows emitted by this module.
const WATCHER_AGENT_ID: &str = "github-pr-watcher";

/// `gh` binary, resolved from PATH at each invocation.
const GH_BINARY: &str = "gh";

/// New comment summary returned by [`watch_comments`]. We deliberately
/// surface only the comment's create-timestamp (in milliseconds) plus
/// the comment id; full body retrieval is the caller's job via a
/// follow-up `gh api` call when it actually needs the text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NewComment {
    /// gh comment id (string for forward compat — gh used to emit u64
    /// but is migrating to opaque ids).
    #[serde(default)]
    pub id: String,
    /// `createdAt` field as wall-clock millis since epoch; parsed from
    /// the gh ISO-8601 string at watch-time.
    #[serde(default)]
    pub created_at_ms: i64,
    /// Best-effort author handle. May be empty if gh omits the field.
    #[serde(default)]
    pub author: String,
}

/// Wire-format helper for parsing `gh pr view --json comments`.
#[derive(Debug, Deserialize)]
struct GhCommentsResponse {
    #[serde(default)]
    comments: Vec<GhCommentWire>,
}

#[derive(Debug, Deserialize)]
struct GhCommentWire {
    #[serde(default)]
    id: String,
    #[serde(rename = "createdAt", default)]
    created_at: String,
    #[serde(default)]
    author: GhAuthorWire,
}

#[derive(Debug, Deserialize, Default)]
struct GhAuthorWire {
    #[serde(default)]
    login: String,
}

/// Poll the GitHub API via `gh pr view --json comments` for new comments
/// on `pr_number` that landed strictly after `since_ts_ms`. Emits one
/// `pr_comment_watched` SSB row summarising the new-comment count.
///
/// Returns the list of new comments (possibly empty). Caller decides
/// whether to spawn a follow-up routing action (E-41 does NOT auto-route
/// — that path stays default-OFF per the C-40 deferred-router scope).
///
/// # Errors
/// Standard `GithubPrError` shape — gh missing / unauthenticated /
/// non-zero exit / JSON parse failure.
pub async fn watch_comments(
    ssb: Arc<Ssb>,
    pr_number: u32,
    since_ts_ms: i64,
) -> Result<Vec<NewComment>, GithubPrError> {
    // Build the argv. gh's `--json comments` returns the full thread;
    // we filter by createdAt > since_ts_ms locally.
    let argv = [
        "pr",
        "view",
        &pr_number.to_string(),
        "--json",
        "comments",
    ];
    let output = Command::new(GH_BINARY)
        .args(argv)
        .output()
        .await
        .map_err(GithubPrError::from)?;
    if !output.status.success() {
        let label = format!("gh {}", argv.join(" "));
        return Err(GithubPrError::from_exit(label, output.status, output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let resp: GhCommentsResponse = serde_json::from_str(stdout.trim())
        .map_err(|e| GithubPrError::from_json(&stdout, e))?;

    let mut new_comments: Vec<NewComment> = Vec::new();
    for c in resp.comments {
        let created_ms = parse_iso8601_to_millis(&c.created_at);
        if created_ms > since_ts_ms {
            new_comments.push(NewComment {
                id: c.id,
                created_at_ms: created_ms,
                author: c.author.login,
            });
        }
    }

    let new_count = u32::try_from(new_comments.len()).unwrap_or(u32::MAX);
    let payload = SsbPayload::PrCommentWatched(PrCommentWatchedPayload {
        pr_number_hash: GhCliAdapter::hash_prefix(&pr_number.to_string()),
        new_comments_count: new_count,
        ts_ms: super::clock::now_ms(),
    });
    if let Err(e) = ssb.write_payload(WATCHER_AGENT_ID, 0, payload, vec![]).await {
        warn!(
            target: "github_pr",
            error = %e,
            "ssb write failed for pr_comment_watched row; row dropped",
        );
    }

    Ok(new_comments)
}

/// Best-effort ISO-8601 -> milliseconds-since-epoch conversion using
/// stdlib only. Handles the common shape `YYYY-MM-DDTHH:MM:SSZ` (gh's
/// default). Returns 0 when the input is empty or unparseable so the
/// downstream filter degrades to "include everything" rather than
/// panicking.
fn parse_iso8601_to_millis(input: &str) -> i64 {
    // Quick check for the canonical gh shape: 20 bytes ending in `Z`.
    if input.len() < 20 || !input.ends_with('Z') {
        return 0;
    }
    let bytes = input.as_bytes();
    // Layout: 0123456789012345678 9
    //         YYYY-MM-DDTHH:MM:SS Z
    let year = parse_int_range(bytes, 0, 4);
    let month = parse_int_range(bytes, 5, 7);
    let day = parse_int_range(bytes, 8, 10);
    let hour = parse_int_range(bytes, 11, 13);
    let minute = parse_int_range(bytes, 14, 16);
    let second = parse_int_range(bytes, 17, 19);
    let (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) =
        (year, month, day, hour, minute, second)
    else {
        return 0;
    };
    if !(1970..=2999).contains(&y) || !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return 0;
    }
    days_from_epoch(y, mo, d)
        .map(|days| (days * 86_400 + h * 3600 + mi * 60 + s) * 1000)
        .unwrap_or(0)
}

fn parse_int_range(bytes: &[u8], lo: usize, hi: usize) -> Option<i64> {
    if hi > bytes.len() {
        return None;
    }
    let mut acc: i64 = 0;
    for &b in &bytes[lo..hi] {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc * 10 + i64::from(b - b'0');
    }
    Some(acc)
}

/// Days from 1970-01-01 for (year, month, day). Returns None for
/// obviously bad inputs. Uses the Howard Hinnant civil-from-days
/// algorithm inverted: this implementation iterates years/months because
/// we only call it once per comment (gh thread sizes are tiny).
fn days_from_epoch(year: i64, month: i64, day: i64) -> Option<i64> {
    if year < 1970 {
        return None;
    }
    let mut total_days: i64 = 0;
    for y in 1970..year {
        total_days += if is_leap_year(y) { 366 } else { 365 };
    }
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        let idx = usize::try_from(m - 1).ok()?;
        let mut dim = days_in_month[idx];
        if m == 2 && is_leap_year(year) {
            dim = 29;
        }
        total_days += dim;
    }
    total_days += day - 1;
    Some(total_days)
}

const fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_epoch_zero() {
        assert_eq!(parse_iso8601_to_millis("1970-01-01T00:00:00Z"), 0);
    }

    #[test]
    fn iso8601_known_anchor() {
        // 2024-01-01T00:00:00Z -- canonical value.
        // 54 years * 365.25 ~ 19_724 days approximation; exact value:
        // 1704067200 seconds since epoch.
        let ms = parse_iso8601_to_millis("2024-01-01T00:00:00Z");
        assert_eq!(ms, 1_704_067_200_000);
    }

    #[test]
    fn iso8601_unparseable_returns_zero() {
        assert_eq!(parse_iso8601_to_millis("not-a-date"), 0);
        assert_eq!(parse_iso8601_to_millis(""), 0);
        assert_eq!(parse_iso8601_to_millis("2024-13-99T00:00:00Z"), 0);
    }

    #[test]
    fn leap_year_handling() {
        // 2020-03-01 is day 60 from 2020-01-01 (Jan=31 + Feb=29).
        let mar_1 = parse_iso8601_to_millis("2020-03-01T00:00:00Z");
        let jan_1 = parse_iso8601_to_millis("2020-01-01T00:00:00Z");
        let diff_days = (mar_1 - jan_1) / 86_400_000;
        assert_eq!(diff_days, 60);
    }
}
