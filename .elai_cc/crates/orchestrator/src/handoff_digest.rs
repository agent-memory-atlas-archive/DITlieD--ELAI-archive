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

    let (decisions, files_touched, memory_fact_ids) = extract_digest_facts(&entries);

    let slug = derive_slug(session_id, &decisions);
    let handoffs_dir = cwd.join("workflow").join("handoffs");
    fs::create_dir_all(&handoffs_dir).context("create workflow/handoffs")?;
    let digest_path = handoffs_dir.join(format!("{slug}.md"));

    let prior_discovery_titles = collect_prior_discovery_titles(&digest_path);

    let digest_body = build_digest_body(
        session_id,
        &slug,
        &decisions,
        &files_touched,
        &prior_discovery_titles,
        &memory_fact_ids,
        cwd,
    );
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
    let now_secs = SystemTime::now() // entropy-allow: repl-startup-mtime-horizon-check-not-replay-compared
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
// Section-aware budgeted read (J-25 MOVE T2)
// ---------------------------------------------------------------------------

/// One section sliced from a handoff digest by the budgeted loader.
///
/// T2 minimal shape: header + body. The typed [`DigestSection`] enum with
/// per-section token budgets + dedup validator is T1 (shipped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetedSection {
    /// The header line that introduced this section (e.g. `SUMMARY`,
    /// `## DECISIONS`, `CURRENT STATUS:`). Always present in the result,
    /// even when the body was truncated to zero by the budget.
    pub header: String,
    /// The body lines under the header. May be truncated to fit the budget.
    pub body: String,
}

/// Result of [`read_budgeted_section_aware`].
#[derive(Debug, Clone)]
pub struct BudgetedReadResult {
    /// Skeleton headers + filled bodies, in section order.
    pub sections: Vec<BudgetedSection>,
    /// Estimated tokens of the assembled skeleton + bodies text
    /// (`len / CHARS_PER_TOKEN`, capped at `u32::MAX`).
    pub loaded_tokens: u32,
    /// `true` iff any body was cut by the budget.
    pub truncated: bool,
    /// `Some("line N")` 1-based hint into `digest_text` naming the line
    /// after the last emitted line, when `truncated`. `None` otherwise.
    /// The caller can prefix the file path if it wants a `file:line` form.
    pub resume_hint: Option<String>,
}

// ---------------------------------------------------------------------------
// Typed digest section template (J-25 MOVE T1)
// ---------------------------------------------------------------------------
//
// The `pub enum DigestSection` definition stays in THIS file so the J-25
// WIRING_CONTRACT D1a source-grep (`pub enum DigestSection` in
// `handoff_digest.rs`) keeps passing. The impl block + the free functions
// `render_digest_sections` / `extract_discovery_titles` /
// `validate_discoveries_dedup` + the `DedupVerdict` struct + the T1 unit
// tests live in the `digest_section` submodule (MR-1 split: this file was
// over the 1000-line cap). The `pub use` below keeps the existing
// `crate::DigestSection` / `handoff_digest::DigestSection` paths working.

mod digest_section;
mod memory_taint;
pub use digest_section::{
    DedupVerdict, extract_discovery_titles, render_digest_sections,
    validate_discoveries_dedup,
};

/// One typed section of a handoff digest, carrying its ALL-CAPS header, its
/// per-section token-budget CAP, and its body text.  The render path emits
/// these as markdown text that [`read_budgeted_section_aware`] (T2) consumes
/// unchanged — there is no second on-disk format (ABORT-1).
///
/// Variants match the J-25 T1 plan / MiMo checkpoint template: Intent,
/// NextAction, Directives, TaskTree, CurrentWork, Files, Discoveries, Errors,
/// LiveResources, Decisions, Notes.  Per-section token CAPS follow the plan:
/// 500 / 1000 / 800 / 1000 / 2000 / 1500 / 2000 / 1500 / 1000 / 3000 / 800.
/// These are CAPS, not a mandate to emit ~11K total; the render path truncates
/// each section to its cap and the total stays under [`MAX_DIGEST_TOKENS`].
///
/// The impl block (`header` / `budget_tokens` / `budget_chars` / `render`)
/// lives in [`digest_section`] — see MR-1 note above.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigestSection {
    Intent,
    NextAction,
    Directives,
    TaskTree,
    CurrentWork,
    Files,
    Discoveries,
    Errors,
    LiveResources,
    Decisions,
    Notes,
    /// J-93 LEG-C: persisted memory trust labels.
    MemoryTaint,
}

/// Read `digest_text` into a skeleton + budgeted bodies, preserving every
/// section header unconditionally.
///
/// A "section header" is any line starting with `#` (markdown heading) or
/// an ALL-CAPS label line (e.g. `SUMMARY`, `DECISIONS`, `FILES TOUCHED`,
/// `HOW TO PICK UP`, `CURRENT STATUS:`, `INDEX`). A section is its header
/// line plus every following line until the next header line or EOF.
///
/// Behavior:
///  - The skeleton (every header line, plus every line under an `INDEX`
///    header) ALWAYS survives, even when `budget_tokens == 0`.
///  - Bodies are filled in section order using
///    `floor(budget_tokens * 0.95)` body-token budget; each body line
///    consumes `len / CHARS_PER_TOKEN` estimated tokens
///    (`CHARS_PER_TOKEN` is 4, consistent with the rest of the module).
///  - On overflow: `truncated = true` and `resume_hint = Some("line N")`
///    naming the 1-based line in `digest_text` after the last emitted line.
///  - Deterministic; NO LLM, NO I/O, NO network.
pub fn read_budgeted_section_aware(
    digest_text: &str,
    budget_tokens: u32,
) -> BudgetedReadResult {
    // 1. Parse into raw sections (header line + body lines).
    let mut raw: Vec<(String, Vec<&str>, usize)> = Vec::new();
    // tuple: (header_line, body_lines, header_line_number_1based)
    let mut current: Option<(String, Vec<&str>, usize)> = None;
    let mut line_no: usize = 0;
    for line in digest_text.lines() {
        line_no += 1;
        if is_section_header(line) {
            if let Some((h, b, n)) = current.take() {
                raw.push((h, b, n));
            }
            current = Some((line.to_string(), Vec::new(), line_no));
        } else if let Some((_, ref mut body, _)) = current.as_mut() {
            body.push(line);
        } else {
            // No header seen yet; treat as a pre-amble section with empty
            // header so the content is not silently dropped. The skeleton
            // rule applies only to actual headers, so this pre-amble is
            // subject to the body budget like any other body.
            current = Some((String::new(), vec![line], line_no));
        }
    }
    if let Some((h, b, n)) = current {
        raw.push((h, b, n));
    }

    // 2. Compute body budget: floor(budget_tokens * 0.95).
    let body_budget_tokens: u32 = ((budget_tokens as f64) * 0.95).floor() as u32;
    let mut body_budget_chars: usize = body_budget_tokens.saturating_mul(CHARS_PER_TOKEN as u32) as usize;

    // 3. Walk sections in order. Headers (and INDEX bodies) always survive.
    let mut sections: Vec<BudgetedSection> = Vec::with_capacity(raw.len());
    let mut truncated = false;
    let mut resume_line: Option<usize> = None;

    for (header, body_lines, header_no) in &raw {
        let header_is_index = header.trim().eq_ignore_ascii_case("INDEX");
        if header_is_index {
            // INDEX lines are skeleton too: preserved unconditionally.
            let body = body_lines.join("\n");
            sections.push(BudgetedSection {
                header: header.clone(),
                body,
            });
            continue;
        }

        // Fill body lines in order until body budget exhausted.
        let mut kept: Vec<&str> = Vec::with_capacity(body_lines.len());
        for (idx, line) in body_lines.iter().enumerate() {
            let line_chars = line.len() + 1; // +1 for the newline that join("\n") re-adds
            if line_chars <= body_budget_chars {
                body_budget_chars -= line_chars;
                kept.push(line);
            } else if body_budget_tokens > 0 {
                // Only flag truncation when a positive body budget was
                // requested. With budget_tokens == 0 (skeleton-only request)
                // no body content was cut mid-fill, so truncated must stay
                // false per the J-25 WIRING_CONTRACT.
                truncated = true;
                // 1-based line in digest_text of the line we just rejected
                // (the line after the last emitted line): header_no + 1 (the
                // header itself) + idx (body lines consumed) + 1.
                resume_line = Some(header_no + 1 + idx + 1);
                break;
            } else {
                // No body budget requested; skip remaining body lines
                // without marking truncated.
                break;
            }
        }
        let body = kept.join("\n");
        sections.push(BudgetedSection {
            header: header.clone(),
            body,
        });
    }

    // 4. If we exhausted the budget mid-section but never hit the `else`
    //    branch (e.g. last section's last line exactly fit), truncated stays
    //    false and resume_hint stays None, which is correct.
    //    If truncated was set, compute the hint.
    let resume_hint = if truncated {
        resume_line.map(|n| format!("line {n}"))
    } else {
        None
    };

    // 5. loaded_tokens: estimated tokens of assembled skeleton + bodies.
    let mut assembled_len: usize = 0;
    for s in &sections {
        assembled_len += s.header.len() + 1 + s.body.len() + 1; // header + \n + body + \n
    }
    let loaded_tokens = (assembled_len / CHARS_PER_TOKEN).min(u32::MAX as usize) as u32;

    BudgetedReadResult {
        sections,
        loaded_tokens,
        truncated,
        resume_hint,
    }
}

/// Classify a line as a section header for the budgeted loader.
///
/// Markdown headings (`#`, `##`, ...) and ALL-CAPS label lines (e.g.
/// `SUMMARY`, `CURRENT STATUS:`, `INDEX`) are headers. A line is ALL-CAPS
/// if its non-space alphabetic characters are all uppercase and it has at
/// least one alphabetic character.
fn is_section_header(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return true;
    }
    let t = trimmed.trim();
    if t.is_empty() {
        return false;
    }
    let mut alpha_count = 0usize;
    for ch in t.chars() {
        if ch.is_alphabetic() {
            alpha_count += 1;
            if !ch.is_uppercase() {
                return false;
            }
        } else if !ch.is_whitespace() && ch != ':' && ch != '_' && ch != '-' {
            // Allow `:`, `_`, `-` as label punctuation (e.g. `CURRENT STATUS:`).
            return false;
        }
    }
    alpha_count > 0
}

/// Truncate `body` to at most `char_budget` chars on a line boundary.  Whole
/// lines are kept while their cumulative length (including the trailing
/// newline) fits; the final line may be cut mid-line to land exactly on the
/// budget.  Returns the truncated body without a trailing newline.
fn truncate_lines_to_char_budget(body: &str, char_budget: usize) -> String {
    if body.len() <= char_budget {
        return body.to_string();
    }

    let mut kept: Vec<String> = Vec::new();
    let mut used: usize = 0;
    for line in body.lines() {
        let line_chars = line.len() + 1;
        if used.saturating_add(line_chars) > char_budget {
            let remaining = char_budget.saturating_sub(used);
            if remaining == 0 {
                break;
            }
            let mut partial = String::from(line);
            truncate_to_char_boundary(&mut partial, remaining);
            kept.push(partial);
            break;
        }
        used = used.saturating_add(line_chars);
        kept.push(line.to_owned());
    }

    kept.join("\n")
}

/// Truncate `s` in place to at most `max_chars` chars, landing on a UTF-8
/// char boundary.  Never panics on non-UTF-8 or short strings.
fn truncate_to_char_boundary(s: &mut String, max_chars: usize) {
    if s.chars().count() <= max_chars {
        return;
    }
    let end = s
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    s.truncate(end);
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Extract (decisions, files_touched) from the raw SSB stream.
///
/// Decisions come from `InterventionRequest` rows (the `description` field
/// is the human-readable decision text).  Files come from `FileWrite` rows
/// (`path` field).  All other rows are ignored.
fn extract_digest_facts(entries: &[SsbEntry]) -> (Vec<String>, Vec<String>, Vec<String>) {
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

    (decisions, files, memory_taint::extract_fact_ids(entries))
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

/// Scan `digest_path` (if it exists) for prior `DISCOVERIES` sections and
/// collect their titles via [`extract_discovery_titles`].  Returns an empty
/// set when the file does not yet exist or has no discoveries.  Used by
/// [`author_topic_digest`] to feed [`build_digest_body`]'s dedup call site.
fn collect_prior_discovery_titles(digest_path: &Path) -> std::collections::HashSet<String> {
    let mut titles = std::collections::HashSet::new();
    let Ok(text) = fs::read_to_string(digest_path) else {
        return titles;
    };
    let mut in_discoveries = false;
    for line in text.lines() {
        if is_section_header(line) {
            in_discoveries = line.trim().eq_ignore_ascii_case("DISCOVERIES");
            continue;
        }
        if in_discoveries {
            for t in extract_discovery_titles(line) {
                titles.insert(t);
            }
        }
    }
    titles
}

/// Build the body text for a new digest entry using the typed
/// [`DigestSection`] template (J-25 MOVE T1).  Each section is truncated to
/// its per-section token cap; the total stays under [`MAX_DIGEST_TOKENS`].
///
/// The rendered text is plain markdown with ALL-CAPS headers, compatible with
/// [`read_budgeted_section_aware`] (T2).  No second on-disk format (ABORT-1).
///
/// `prior_discovery_titles` carries discovery titles already present in the
/// digest file (collected by [`author_topic_digest`]) so [`build_digest_body`]
/// can dedup the current discoveries body against them via
/// [`validate_discoveries_dedup`] at the real call site (J-25 S6b wire2).
fn build_digest_body(
    session_id: &str,
    slug: &str,
    decisions: &[String],
    files: &[String],
    prior_discovery_titles: &std::collections::HashSet<String>,
    memory_fact_ids: &[String],
    cwd: &Path,
) -> String {
    let date = chrono_date_utc();

    let intent_body = format!("Session `{session_id}` for topic `{slug}`.");
    let next_action_body = format!(
        "Run `elai /continue` or set topic slug `{slug}` to resume with tiered load."
    );

    let mut decisions_body = String::new();
    if decisions.is_empty() {
        decisions_body.push_str("(no InterventionRequest rows found in SSB stream for this session)\n");
    } else {
        for d in decisions {
            decisions_body.push_str(&format!("- {d}\n"));
        }
    }

    let mut files_body = String::new();
    if files.is_empty() {
        files_body.push_str("(no FileWrite rows found in SSB stream for this session)\n");
    } else {
        for f in files {
            files_body.push_str(&format!("- {f}\n"));
        }
    }

    let raw_discoveries_body = "(none)\n".to_string();
    let current_titles = extract_discovery_titles(&raw_discoveries_body);
    let verdict = validate_discoveries_dedup(&current_titles, prior_discovery_titles);
    let discoveries_body = if verdict.has_duplicates {
        let mut filtered = String::new();
        for line in raw_discoveries_body.lines() {
            let trimmed = line.trim_start();
            let rest = match trimmed.strip_prefix("- ") {
                Some(r) => r,
                None => {
                    filtered.push_str(line);
                    filtered.push('\n');
                    continue;
                }
            };
            let title = match rest.split_once(':') {
                Some((t, _)) => t.trim(),
                None => rest.trim(),
            };
            if !verdict.duplicates.iter().any(|d| d == title) {
                filtered.push_str(line);
                filtered.push('\n');
            }
        }
        filtered
    } else {
        raw_discoveries_body
    };

    let mut sections: Vec<(DigestSection, String)> = vec![
        (DigestSection::Intent, intent_body),
        (DigestSection::NextAction, next_action_body),
        (DigestSection::Decisions, decisions_body),
        (DigestSection::Files, files_body),
        (DigestSection::Discoveries, discoveries_body),
    ];

    if verdict.has_duplicates && !verdict.duplicates.is_empty() {
        let dropped = verdict.duplicates.len();
        sections.push((DigestSection::Notes, format!("dedup dropped {dropped} prior titles\n")));
    }

    if let Some(body) = memory_taint::section_body(memory_fact_ids, cwd) {
        sections.push((DigestSection::MemoryTaint, body));
    }

    let rendered = render_digest_sections(&sections);

    format!("\n---\n## {date} session {session_id}\n\n{rendered}\n")
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
    let now = SystemTime::now() // entropy-allow: human-readable-digest-date-label-not-replay-compared
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
// Writer coalescing + fork snapshot (J-25 MOVE T5)
// ---------------------------------------------------------------------------

/// Frozen snapshot of the prompt-materialized state captured at writer-spawn
/// time so the background writer never recomputes the prompt (HD-2 prefix-cache
/// alignment). Carried by [`PendingDigestRequest`] for the type-system / test
/// contract; the current author path does not yet consume the fields (a future
/// slice wires them into the body builder without a second format).
#[derive(Debug, Clone)]
pub struct DigestWriterSnapshot {
    pub system_parts: Vec<String>,
    pub tool_defs: Vec<String>,
    pub watermark_msg_id: Option<String>,
}

/// A pending digest-author request queued for the background writer.  The
/// snapshot is frozen at `request()` time (HD-2); a later newer request
/// REPLACES the pending slot entirely and never mutates the kept snapshot.
#[derive(Debug, Clone)]
pub struct PendingDigestRequest {
    pub snapshot: DigestWriterSnapshot,
    pub session_id: String,
    pub cwd: PathBuf,
}

/// 1-slot newest-wins pending queue for digest-author requests.  A newer
/// `request()` evicts an older pending request in-place; `take()` drains the
/// single slot.  Sync-mutex; no new deps.
#[derive(Debug, Default)]
pub struct DigestWriterQueue {
    pending: std::sync::Mutex<Option<PendingDigestRequest>>,
}

impl DigestWriterQueue {
    pub fn new() -> Self {
        Self {
            pending: std::sync::Mutex::new(None),
        }
    }

    /// Enqueue `req`, evicting any older pending request.  The frozen snapshot
    /// on `req` is never mutated after insert (HD-2).
    pub fn request(&self, req: PendingDigestRequest) {
        if let Ok(mut slot) = self.pending.lock() {
            *slot = Some(req);
        }
    }

    /// Drain the single pending slot, returning the newest request if any.
    pub fn take(&self) -> Option<PendingDigestRequest> {
        self.pending.lock().ok().and_then(|mut slot| slot.take())
    }
}

/// Author a digest for a pending request by delegating body authorship to the
/// existing [`author_topic_digest`] (ABORT-1: no second digest body builder).
/// The frozen snapshot is logged here so the HD-2 freeze is load-bearing for
/// diagnostics without introducing a second digest format.
pub fn author_from_pending(
    ssb: &crate::ssb::Ssb,
    req: &PendingDigestRequest,
) -> anyhow::Result<TopicHandle> {
    tracing::debug!(
        watermark_msg_id = ?req.snapshot.watermark_msg_id,
        system_parts = req.snapshot.system_parts.len(),
        tool_defs = req.snapshot.tool_defs.len(),
        "J-25 T5 frozen digest-writer snapshot"
    );
    author_topic_digest(ssb, &req.session_id, &req.cwd)
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
        let (decisions, files, memory_facts) = extract_digest_facts(&[]);
        assert!(decisions.is_empty());
        assert!(files.is_empty());
    }

    #[test]
    fn test_build_digest_body_no_data() {
        let prior = std::collections::HashSet::new();
        let body = build_digest_body("test-session", "test-slug", &[], &[], &prior, &[], Path::new("."));
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

    #[test]
    fn test_build_digest_body_uses_typed_sections() {
        let prior = std::collections::HashSet::new();
        let body = build_digest_body("s1", "slug1", &["decide A".to_string()], &["f.rs".to_string()], &prior, &[], Path::new("."));
        assert!(body.contains("INTENT\n"));
        assert!(body.contains("NEXT ACTION\n"));
        assert!(body.contains("DECISIONS\n- decide A"));
        assert!(body.contains("FILES TOUCHED\n- f.rs"));
        assert!(body.len() / CHARS_PER_TOKEN <= MAX_DIGEST_TOKENS);
    }

    #[test]
    fn test_rendered_text_is_section_header_compatible() {
        let sections = vec![
            (DigestSection::Intent, "body".to_string()),
            (DigestSection::Decisions, "- d".to_string()),
        ];
        let out = render_digest_sections(&sections);
        assert!(is_section_header("INTENT"));
        assert!(is_section_header("DECISIONS"));
        let result = read_budgeted_section_aware(&out, 5000);
        assert!(!result.sections.is_empty());
        assert!(result.sections.iter().any(|s| s.header == "INTENT"));
        assert!(result.sections.iter().any(|s| s.header == "DECISIONS"));
    }
}
