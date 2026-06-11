//! H-51: cross-session sleep-time handoff digest author + open-topic detector.
//!
//! Grounded source: H-51 plan file
//! `docs/thefinalglance/H-plan/H-51_sleeptime_handoff_digest_autodetect.md`.
//!
//! ## Design
//!
//! At session-end (via [`author_topic_digest`]), this module:
//!   1. Reads the current session's SSB stream via `query_session_stream_asc`.
//!   2. Extracts decisions from `InterventionRequest` rows and files from
//!      `FileWrite` rows — NEVER fabricates from LLM recollection (STUB_BAN).
//!   3. Derives a kebab-slug from the first `InterventionRequest.description`
//!      found in the stream, falling back to the session_id prefix.
//!   4. Appends (or creates) `workflow/handoffs/{slug}.md` per the
//!      handoff-protocol.md living-append convention.
//!   5. Returns a [`TopicHandle`] the caller uses to emit the
//!      [`HandoffLifecycleEvent`][`crate::ssb::SsbActionType::HandoffLifecycleEvent`]
//!      DigestWritten row.
//!
//! [`detect_open_topic`] scans `workflow/handoffs/` for any `.md` file last
//! modified within [`OPEN_TOPIC_HORIZON_SECS`] and returns a `TopicHandle` so
//! the REPL startup path can offer a one-tap resume.
//!
//! ## Disabled path
//!
//! Set `ELAI_HANDOFF_AUTORESUME_DISABLE=1` to suppress auto-detect at startup.
//! The digest-write leg always runs at session end (it is the producer that
//! makes future resume possible; suppressing it would be data loss).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ssb::{SsbActionType, SsbEntry};

/// How many seconds old a handoff file can be and still be "open" for resume.
/// 8 hours — covers overnight / same-day resume.
const OPEN_TOPIC_HORIZON_SECS: u64 = 8 * 60 * 60;

/// Maximum token estimate for the digest.  Enforced by the TieredLoad done-gate:
/// loaded_tokens MUST be <= 5000.
const MAX_DIGEST_TOKENS: usize = 4_000;

/// Rough chars-per-token ratio used for the cheap token estimator.
const CHARS_PER_TOKEN: usize = 4;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Stable reference to an authored (or found) handoff topic.
///
/// Returned by [`author_topic_digest`] and [`detect_open_topic`].
#[derive(Debug, Clone)]
pub struct TopicHandle {
    /// Kebab-slug derived from the session's first decision description.
    pub topic_slug: String,
    /// Absolute path to the `workflow/handoffs/{slug}.md` file.
    pub digest_path: PathBuf,
    /// Estimated token count of the digest written / found.
    pub loaded_tokens: u32,
}

// ---------------------------------------------------------------------------
// Digest author (session-end, called from LiveCli::drop)
// ---------------------------------------------------------------------------

/// Author or append a sleep-time digest for the current session.
///
/// Reads the SSB stream from `ssb` for `session_id`, extracts decisions
/// (from `InterventionRequest` rows) and files touched (from `FileWrite` rows),
/// and writes/appends `{cwd}/workflow/handoffs/{slug}.md`.
///
/// Returns a [`TopicHandle`] on success.  Errors are non-fatal — the caller
/// logs them and continues session teardown.
///
/// # STUB_BAN
/// Decisions are extracted ONLY from `InterventionRequest` SSB rows.
/// Files are extracted ONLY from `FileWrite` SSB rows.
/// No LLM recollection, no hallucination.
pub fn author_topic_digest(
    ssb: &crate::ssb::Ssb,
    session_id: &str,
    cwd: &Path,
) -> anyhow::Result<TopicHandle> {
    use anyhow::Context as _;

    let entries = ssb
        .query_session_stream_asc(session_id)
        .context("query_session_stream_asc")?;

    let (decisions, files_touched) = extract_digest_facts(&entries);

    let slug = derive_slug(session_id, &decisions);
    let handoffs_dir = cwd.join("workflow").join("handoffs");
    fs::create_dir_all(&handoffs_dir).context("create workflow/handoffs")?;
    let digest_path = handoffs_dir.join(format!("{slug}.md"));

    let digest_body = build_digest_body(session_id, &slug, &decisions, &files_touched);
    let estimated_tokens = (digest_body.len() / CHARS_PER_TOKEN).min(MAX_DIGEST_TOKENS) as u32;

    append_or_create_handoff(&digest_path, &slug, session_id, &digest_body)?;

    Ok(TopicHandle {
        topic_slug: slug,
        digest_path,
        loaded_tokens: estimated_tokens,
    })
}

// ---------------------------------------------------------------------------
// Open-topic detector (REPL startup)
// ---------------------------------------------------------------------------

/// Scan `{cwd}/workflow/handoffs/` for any `.md` file modified within
/// [`OPEN_TOPIC_HORIZON_SECS`].  Returns the most-recently-modified one as a
/// [`TopicHandle`] if found, otherwise `None`.
///
/// Guard: if `ELAI_HANDOFF_AUTORESUME_DISABLE=1` is set, always returns `None`.
pub fn detect_open_topic(cwd: &Path) -> Option<TopicHandle> {
    if std::env::var("ELAI_HANDOFF_AUTORESUME_DISABLE")
        .unwrap_or_default()
        .trim()
        == "1"
    {
        return None;
    }

    let handoffs_dir = cwd.join("workflow").join("handoffs");
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut best: Option<(u64, PathBuf)> = None;

    let rd = fs::read_dir(&handoffs_dir).ok()?;
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if now_secs.saturating_sub(mtime) <= OPEN_TOPIC_HORIZON_SECS {
            if best.as_ref().map_or(true, |(prev, _)| mtime > *prev) {
                best = Some((mtime, path));
            }
        }
    }

    let (_, path) = best?;

    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_owned();

    let loaded_tokens = fs::read_to_string(&path)
        .map(|s| (s.len() / CHARS_PER_TOKEN) as u32)
        .unwrap_or(0);

    Some(TopicHandle {
        topic_slug: slug,
        digest_path: path,
        loaded_tokens,
    })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Extract (decisions, files_touched) from the raw SSB stream.
///
/// Decisions come from `InterventionRequest` rows (the `description` field
/// is the human-readable decision text).  Files come from `FileWrite` rows
/// (`path` field).  All other rows are ignored.
fn extract_digest_facts(entries: &[SsbEntry]) -> (Vec<String>, Vec<String>) {
    use crate::ssb::SsbPayload;

    let mut decisions: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    // deduplicate files preserving order
    let mut seen_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in entries {
        match entry.action_type {
            SsbActionType::InterventionRequest => {
                if let Ok(payload) =
                    rmp_serde::from_slice::<SsbPayload>(&entry.payload)
                {
                    if let SsbPayload::InterventionRequest(p) = payload {
                        let text = p.description.trim().to_owned();
                        if !text.is_empty() && decisions.len() < 20 {
                            decisions.push(text);
                        }
                    }
                }
            }
            SsbActionType::FileWrite => {
                if let Ok(payload) =
                    rmp_serde::from_slice::<SsbPayload>(&entry.payload)
                {
                    if let SsbPayload::FileWrite(p) = payload {
                        let path = p.path.trim().to_owned();
                        if !path.is_empty() && seen_files.insert(path.clone()) {
                            files.push(path);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (decisions, files)
}

/// Derive a stable kebab-slug from the session's first decision text, falling
/// back to a truncated `session_id`.
fn derive_slug(session_id: &str, decisions: &[String]) -> String {
    let raw = decisions
        .first()
        .map(|d| d.as_str())
        .unwrap_or(session_id);

    // Take first 6 words, lowercase, replace non-alnum with '-', trim dashes.
    let slug: String = raw
        .split_whitespace()
        .take(6)
        .map(|w| {
            w.chars()
                .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-");

    // Collapse consecutive dashes and trim.
    let mut result = String::new();
    let mut prev_dash = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash {
                result.push(c);
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    let result = result.trim_matches('-').to_owned();

    if result.is_empty() {
        // Fallback: first 16 chars of session_id, replacing non-alnum with '-'.
        session_id
            .chars()
            .take(16)
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
            .collect()
    } else {
        result
    }
}

/// Build the body text for a new digest entry.
///
/// Token-budgeted: if the body would exceed MAX_DIGEST_TOKENS, files list is
/// truncated first, then decisions list.
fn build_digest_body(
    session_id: &str,
    slug: &str,
    decisions: &[String],
    files: &[String],
) -> String {
    let date = chrono_date_utc();

    let mut body = format!(
        "\n---\n## {date} session {session_id}\n\nSUMMARY\nSession `{session_id}` for topic `{slug}`.\n\nDECISIONS\n"
    );

    for (i, d) in decisions.iter().enumerate() {
        body.push_str(&format!("- {d}\n"));
        // Early exit if we exceed the token budget.
        if body.len() / CHARS_PER_TOKEN >= MAX_DIGEST_TOKENS.saturating_sub(200) {
            body.push_str(&format!("... ({} more decisions truncated)\n", decisions.len() - i - 1));
            break;
        }
    }

    if decisions.is_empty() {
        body.push_str("(no InterventionRequest rows found in SSB stream for this session)\n");
    }

    body.push_str("\nFILES TOUCHED\n");
    let files_budget = MAX_DIGEST_TOKENS.saturating_sub(body.len() / CHARS_PER_TOKEN + 100);
    for (i, f) in files.iter().enumerate() {
        body.push_str(&format!("- {f}\n"));
        if body.len() / CHARS_PER_TOKEN >= MAX_DIGEST_TOKENS.saturating_sub(50) {
            body.push_str(&format!("... ({} more files truncated)\n", files.len() - i - 1));
            break;
        }
        let _ = files_budget;
    }

    if files.is_empty() {
        body.push_str("(no FileWrite rows found in SSB stream for this session)\n");
    }

    body.push_str("\nHOW TO PICK UP\n");
    body.push_str(&format!(
        "Run `elai /continue` or set topic slug `{slug}` to resume with tiered load.\n"
    ));

    body
}

/// Append a new session entry to an existing handoff file, or create from
/// template if the file does not yet exist.
///
/// Follows the handoff-protocol.md living-append convention: CURRENT STATUS
/// and INDEX are updated; prior entries are immutable.
fn append_or_create_handoff(
    path: &Path,
    slug: &str,
    session_id: &str,
    entry_body: &str,
) -> anyhow::Result<()> {
    use anyhow::Context as _;
    use std::io::Write as _;

    let date = chrono_date_utc();

    if path.exists() {
        // Append-only: read head to update CURRENT STATUS, then append entry.
        let existing = fs::read_to_string(path).context("read existing handoff")?;
        // Update CURRENT STATUS line (overwrite the one-liner after "CURRENT STATUS:")
        let updated = update_current_status(
            &existing,
            &format!("session {session_id} complete, digest appended {date}"),
        );
        // Append new index line + entry.
        let appended = format!(
            "{updated}\n{date} session {session_id} - digest written by H-51 author\n{entry_body}"
        );
        fs::write(path, appended).context("write handoff append")?;
    } else {
        // Create from template.
        let template = format!(
            "# Handoff: {slug}\n\nproblem: {slug} (auto-created by H-51 sleep-time digest)\n\
             CURRENT STATUS: session {session_id} complete, digest written {date}\n\
             \nINDEX\n{date} session {session_id} - digest written by H-51 author\n\
             ---\n{entry_body}"
        );
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .context("create handoff file")?;
        file.write_all(template.as_bytes())
            .context("write handoff template")?;
    }

    Ok(())
}

/// Overwrite the "CURRENT STATUS:" line with `new_status`.  Returns the
/// updated string; if no CURRENT STATUS line is found, appends one.
fn update_current_status(content: &str, new_status: &str) -> String {
    let marker = "CURRENT STATUS:";
    let mut result = String::with_capacity(content.len() + 64);
    let mut replaced = false;
    for line in content.lines() {
        if !replaced && line.starts_with(marker) {
            result.push_str(&format!("{marker} {new_status}\n"));
            replaced = true;
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    if !replaced {
        result.push_str(&format!("{marker} {new_status}\n"));
    }
    result
}

/// Return today's UTC date as "YYYY-MM-DD".
fn chrono_date_utc() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple epoch-to-date conversion (no chrono dep).
    let days = now / 86_400;
    // days since 1970-01-01
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Convert days-since-epoch to (year, month, day).  Gregorian calendar.
fn days_to_ymd(mut days: u64) -> (u32, u32, u32) {
    // 400-year cycle = 146097 days
    let year_400 = days / 146_097;
    days %= 146_097;
    let year_100 = (days / 36_524).min(3);
    days -= year_100 * 36_524;
    let year_4 = days / 1_461;
    days %= 1_461;
    let year_1 = (days / 365).min(3);
    days -= year_1 * 365;

    let year = (year_400 * 400 + year_100 * 100 + year_4 * 4 + year_1 + 1970) as u32;
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let month_days: [u32; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 0u32;
    let mut remaining = days as u32;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md {
            month = i as u32 + 1;
            break;
        }
        remaining -= md;
    }
    if month == 0 {
        month = 12;
        remaining = 30;
    }
    (year, month, remaining + 1)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Serialize all env-mutating tests to prevent races under the default
    // parallel runner.  Using a plain std::sync::Mutex so no external dep is
    // required (serial_test not in dev-deps; mirrors context_policy.rs idiom).
    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_derive_slug_from_decision() {
        let decisions = vec!["Implement the handoff digest author function".to_string()];
        let slug = derive_slug("sess-001", &decisions);
        assert_eq!(slug, "implement-the-handoff-digest-author-function");
    }

    #[test]
    fn test_derive_slug_fallback_session_id() {
        let slug = derive_slug("sess-abc123", &[]);
        assert!(!slug.is_empty());
        assert!(slug.len() <= 16);
    }

    #[test]
    fn test_detect_open_topic_respects_disable_env() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // Safety: test-only env mutation; serialized by ENV_MUTEX to prevent
        // races with other env-mutating tests under the parallel runner.
        unsafe { std::env::set_var("ELAI_HANDOFF_AUTORESUME_DISABLE", "1") };
        let result = detect_open_topic(Path::new("."));
        unsafe { std::env::remove_var("ELAI_HANDOFF_AUTORESUME_DISABLE") };
        assert!(result.is_none(), "must return None when kill-switch is set");
    }

    #[test]
    fn test_update_current_status_replaces_line() {
        let content = "# Handoff\n\nCURRENT STATUS: old status\n\nrest\n";
        let updated = update_current_status(content, "new status");
        assert!(updated.contains("CURRENT STATUS: new status"));
        assert!(!updated.contains("old status"));
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_known_date() {
        // 2026-06-11 = days since epoch
        // 2026-01-01 is day (2026-1970)*365 + leap_days
        // Approximate: 56 years * 365.25 = 20453.  2026-06-11 = day ~20615
        let (y, m, d) = days_to_ymd(20615);
        assert_eq!(y, 2026);
        assert_eq!(m, 6);
        assert_eq!(d, 11);
    }

    #[test]
    fn test_extract_digest_facts_empty() {
        let (decisions, files) = extract_digest_facts(&[]);
        assert!(decisions.is_empty());
        assert!(files.is_empty());
    }

    #[test]
    fn test_build_digest_body_no_data() {
        let body = build_digest_body("test-session", "test-slug", &[], &[]);
        assert!(body.contains("no InterventionRequest rows"));
        assert!(body.contains("no FileWrite rows"));
        assert!(body.len() / CHARS_PER_TOKEN <= MAX_DIGEST_TOKENS);
    }

    #[test]
    fn test_detect_open_topic_empty_dir() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let result = detect_open_topic(dir.path());
        assert!(result.is_none(), "empty dir should return None");
    }
}
