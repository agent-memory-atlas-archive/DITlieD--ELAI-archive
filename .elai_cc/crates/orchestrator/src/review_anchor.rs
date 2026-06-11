//! H-52 — deterministic snippet->hunk finding-anchoring + line-less finding schema.
//!
//! Ports the core primitive from alibaba/open-code-review (Apache-2.0,
//! internal/diff/resolver.go + internal/diff/hunk.go) with ELAI-specific fixes:
//!
//! - Duplicate-snippet ambiguity guard: >1 normalized match in scope -> `Ambiguous`
//!   (first-occurrence-wins in the upstream mis-anchors duplicates — we never guess).
//! - Hunk-proximity tiebreak: when `scope_hunk` is supplied, matches inside that
//!   hunk are preferred over matches in other hunks of the same file.
//! - `Unanchorable` on total miss (NOT a guessed line — the caller renders at file
//!   level or defers to the Tier-2 LLM re-location; that path is DEFERRED here).
//! - ELAI's existing [`crate::focus_tunnel::diff::DiffHunk`] / `body_order` is
//!   reused; no new diff parser is authored (GD-1).
//!
//! DETERMINISM (GD-2): `anchor_finding` is a PURE function of (snippet, hunks,
//! scope_hunk). Given the same pinned diff bytes the anchor re-computes
//! byte-identically. The SSB `FindingAnchored` row carries `snippet_blake3`
//! (the blake3 hash of the normalized snippet) so the compute is verifiable.
//!
//! KILL-SWITCH (W-2 / R9): default-ON; the only opt-out is the env var
//! `ELAI_REVIEW_ANCHOR_DISABLE` (truthy). No wire-dark-allowlist row is required
//! for a DISABLE kill-switch (it is NOT a default-OFF enable gate).
//!
//! STUB_BAN ASSERTION: `ReviewFinding` exposes NO model-settable line/verdict
//! field. The `anchor` field is populated only by `anchor_finding` (deterministic
//! code), never by the model. A finding carrying a model-emitted line number is a
//! WIRING FAIL.
//!
//! EXCLUDED (Tier-2, DEFERRED): the RE-LOCATION LLM retry
//! (internal/diff/relocation.go pattern — a one-LLM-call snippet-regen on total
//! miss). `AnchorKind::Relocated` is reserved for that path; this file never
//! produces it.

use serde::{Deserialize, Serialize};

use crate::focus_tunnel::diff::{DiffBodyKind, DiffHunk};

// ────────────────────────────────────────────────────────────────────────────
// Public types
// ────────────────────────────────────────────────────────────────────────────

/// Why a finding was (or was not) pinned to a hunk.
///
/// The model cannot choose this value — deterministic code sets it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorKind {
    /// Snippet found EXACTLY ONCE in the diff's new (added) side, or uniquely
    /// on the removed side when the new side has no match. `hunk_id` and
    /// `new_line_range` are populated.
    Anchored,
    /// Snippet found but the diff is unavailable or the hunk body is empty; the
    /// anchor falls back to file-level. `hunk_id` and `new_line_range` are None.
    FileFallback,
    /// Snippet occurs MORE THAN ONCE in the scope: anchoring to a specific line
    /// would be a guess. Falls back to file-level.
    Ambiguous,
    /// Snippet not found anywhere in the diff after normalization.
    Unanchorable,
    /// RESERVED for the Tier-2 LLM re-location retry (DEFERRED). Never produced
    /// by this module; `FindingAnchored` rows from this module will never carry
    /// this variant.
    Relocated,
}

impl AnchorKind {
    /// Stable snake_case wire string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anchored => "anchored",
            Self::FileFallback => "file_fallback",
            Self::Ambiguous => "ambiguous",
            Self::Unanchorable => "unanchorable",
            Self::Relocated => "relocated",
        }
    }
}

/// Result of `anchor_finding`. All fields except `kind` and `candidate_match_n`
/// may be absent (None) when the anchor is file-level or unresolvable.
///
/// The model CANNOT set any of these fields — the resolver owns the anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorResult {
    /// The stable hunk id (`<path>#<old_start>-<new_start>`) from
    /// `crate::review_surface::hunk_id_for`. `None` for file-level fallbacks.
    pub hunk_id: Option<String>,
    /// Inclusive `(first_line, last_line)` range on the NEW side of the diff,
    /// 1-based. `None` when hunk_id is None.
    pub new_line_range: Option<(u32, u32)>,
    /// Why this result was produced.
    pub kind: AnchorKind,
    /// Number of normalized matches found in the search scope. 0 = Unanchorable,
    /// 1 = Anchored (or FileFallback on empty body), >1 = Ambiguous.
    pub candidate_match_n: u32,
}

/// Why a file was excluded from the DSR denominator (typed, GD-denominator-integrity).
///
/// Stolen from alibaba/open-code-review `internal/preview.go whyExcluded`.
/// Skip-not-truncate: an oversize file surfaces here rather than being silently
/// truncated, so the DSR changed-line denominator stays explainable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcludeReason {
    /// File is binary (no line-addressable diff).
    Binary,
    /// File matches a user-supplied exclude pattern.
    UserExclude,
    /// File extension is not supported by the diff parser.
    UnsupportedExt,
    /// File matches a default-excluded path pattern (e.g. `*.lock`, `vendor/`).
    DefaultPath,
    /// File was deleted; no new-side content to anchor against.
    Deleted,
    /// File was skipped because its diff size exceeded the configured limit.
    /// Surfaced as `Skipped` (never silently truncated) to keep the denominator
    /// explainable.
    Skipped,
}

/// A reviewer finding carrying a verbatim snippet instead of a model-asserted
/// line number. The `anchor: Option<AnchorResult>` field is set by
/// `anchor_finding` (deterministic code), never by the model.
///
/// STUB_BAN: no `line` or `verdict` field may be added here that a model caller
/// could populate. The resolver owns the anchor; a gate owns the verdict.
///
/// This is a re-export of `crate::review_surface::ReviewFinding` (the single
/// unified struct). The H-52 `snippet` + `anchor` fields live on the shared
/// struct; no separate "AnchoredReviewFinding" type is needed.
/// `pub anchor: Option<AnchorResult>` — resolver-owned, not model-settable.
pub type ReviewFinding = crate::review_surface::ReviewFinding;

// ────────────────────────────────────────────────────────────────────────────
// Kill-switch
// ────────────────────────────────────────────────────────────────────────────

/// Default-ON kill-switch check. Anchoring is ON unless
/// `ELAI_REVIEW_ANCHOR_DISABLE` is a truthy value.
/// No wire-dark-allowlist row is required — this is a DISABLE kill-switch
/// (not a default-OFF enable flag), per W-2 / R9.
#[must_use]
pub fn anchor_disabled() -> bool {
    match std::env::var("ELAI_REVIEW_ANCHOR_DISABLE") {
        Ok(raw) => {
            let v = raw.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Core resolver
// ────────────────────────────────────────────────────────────────────────────

/// Normalize one diff line for snippet matching.
///
/// Strips leading `+`/`-`/` ` diff prefix characters and collapses
/// leading/trailing whitespace, mirroring `normalizeLine` in alibaba/
/// open-code-review `internal/diff/resolver.go`.
#[must_use]
fn normalize_line(s: &str) -> String {
    // Strip a single leading diff-prefix character (+/-/space) if present.
    let stripped = match s.chars().next() {
        Some('+') | Some('-') | Some(' ') => &s[1..],
        _ => s,
    };
    stripped.trim().to_string()
}

/// Split a multi-line snippet into normalized lines, dropping blank lines.
fn normalize_snippet(snippet: &str) -> Vec<String> {
    snippet
        .lines()
        .map(normalize_line)
        .filter(|l| !l.is_empty())
        .collect()
}

/// One resolved candidate location within a hunk.
#[derive(Debug)]
struct CandidateMatch {
    hunk_index: usize,
    /// 1-based new-side line number of the first matched line.
    new_line_start: u32,
    /// 1-based new-side line number of the last matched line.
    new_line_end: u32,
    hunk_id: String,
}

/// Attempt to match a consecutive normalized-line run against the NEW (added)
/// side of a single hunk's `body_order`, then fall back to the OLD (removed)
/// side if not found.
///
/// Returns ALL `CandidateMatch` entries found within this hunk so the caller's
/// ambiguity accumulator sees the exact total count. The new-side scan returns
/// every non-overlapping occurrence; if that yields >= 1 results the old-side
/// scan is skipped. The old-side scan similarly returns every occurrence.
///
/// `new_start` is the 1-based line number of the first line on the new side
/// in this hunk (from `DiffHunk::new_start`).
fn matches_in_hunk(
    pattern: &[String],
    hunk: &DiffHunk,
    hunk_index: usize,
) -> Vec<CandidateMatch> {
    if pattern.is_empty() || hunk.body_order.is_empty() {
        return Vec::new();
    }

    // Build the new-side line sequence from body_order (Added + Context lines).
    // We carry the 1-based new-side line counter alongside each normalized line.
    let mut new_lines: Vec<(u32, String)> = Vec::new();
    let mut new_line_no = hunk.new_start as u32;
    for bl in &hunk.body_order {
        match bl.kind {
            DiffBodyKind::Added | DiffBodyKind::Context => {
                new_lines.push((new_line_no, normalize_line(&bl.text)));
                new_line_no += 1;
            }
            DiffBodyKind::Removed => {
                // Removed lines do NOT advance the new-side counter.
            }
        }
    }

    // Scan for ALL non-overlapping occurrences on the new side.
    let hunk_id = crate::review_surface::hunk_id_for(hunk);
    let new_hits = scan_all_runs(&new_lines, pattern, hunk_index, &hunk_id);
    if !new_hits.is_empty() {
        return new_hits;
    }

    // Fall back to the OLD (removed) side — mirrors resolveFromHunk OLD-side
    // pass in the upstream. Uses the old_start counter.
    let mut old_lines: Vec<(u32, String)> = Vec::new();
    let mut old_line_no = hunk.old_start as u32;
    for bl in &hunk.body_order {
        match bl.kind {
            DiffBodyKind::Removed | DiffBodyKind::Context => {
                old_lines.push((old_line_no, normalize_line(&bl.text)));
                old_line_no += 1;
            }
            DiffBodyKind::Added => {}
        }
    }
    // For old-side hits we report new_line_start = hunk.new_start (conservative):
    // the line is removed and has no new-side number; the cockpit renders it at the
    // hunk boundary. We still track the full count so ambiguity is detected.
    let old_hit_count = scan_all_runs(&old_lines, pattern, hunk_index, &hunk_id).len();
    (0..old_hit_count)
        .map(|_| CandidateMatch {
            hunk_index,
            new_line_start: hunk.new_start as u32,
            new_line_end: hunk.new_start as u32,
            hunk_id: hunk_id.clone(),
        })
        .collect()
}

/// Scan `lines` (a slice of (1-based-line-no, normalized-text) pairs) for ALL
/// non-overlapping consecutive runs matching `pattern`. Returns every hit as a
/// `CandidateMatch`. The caller uses the count to detect ambiguity.
fn scan_all_runs(
    lines: &[(u32, String)],
    pattern: &[String],
    hunk_index: usize,
    hunk_id: &str,
) -> Vec<CandidateMatch> {
    let mut results = Vec::new();
    if lines.len() < pattern.len() {
        return results;
    }
    let last_start = lines.len() - pattern.len();
    let mut start = 0;
    while start <= last_start {
        let window = &lines[start..start + pattern.len()];
        if window.iter().zip(pattern.iter()).all(|((_, l), p)| l == p) {
            let first_line = window[0].0;
            let last_line = window[window.len() - 1].0;
            results.push(CandidateMatch {
                hunk_index,
                new_line_start: first_line,
                new_line_end: last_line,
                hunk_id: hunk_id.to_string(),
            });
            // Advance past this match (non-overlapping).
            start += pattern.len();
        } else {
            start += 1;
        }
    }
    results
}

/// Resolve a verbatim code snippet to its hunk position in a set of diff hunks.
///
/// # Arguments
/// - `snippet`: the verbatim code snippet the model emitted. Must match the diff
///   content after normalization (whitespace collapse + leading `+`/`-` strip).
///   An empty snippet returns `AnchorKind::FileFallback` (file-level finding).
/// - `file_changes`: the parsed diff hunks for the file the finding refers to.
///   Typically a subset of all hunks filtered to a single `file_path`; can be
///   all hunks (the resolver ignores non-matching files).
/// - `scope_hunk`: optional preferred hunk id (from a `C-7 ChangeDecision` row
///   or the G-5 cockpit context). Used as a proximity tiebreak: if the snippet
///   matches in both `scope_hunk` AND another hunk, the `scope_hunk` match wins.
///   Does NOT change the ambiguity rule: two matches WITHIN `scope_hunk` still
///   return `Ambiguous`.
///
/// # Returns
/// An [`AnchorResult`] with:
/// - `kind == Anchored` + populated `hunk_id` + `new_line_range` on unique match.
/// - `kind == Ambiguous` + `candidate_match_n > 1` when multiple matches found.
/// - `kind == Unanchorable` when no match found.
/// - `kind == FileFallback` when `snippet` is empty or all hunk bodies are empty.
///
/// # No panics
/// All Result/Option paths return a conservative fallback. No `.unwrap()` or
/// `.expect()` anywhere in this function (production code policy).
#[must_use]
pub fn anchor_finding(
    snippet: &str,
    file_changes: &[DiffHunk],
    scope_hunk: Option<&str>,
) -> AnchorResult {
    // Empty snippet = file-level finding (the model did not provide a code referent).
    if snippet.trim().is_empty() {
        return AnchorResult {
            hunk_id: None,
            new_line_range: None,
            kind: AnchorKind::FileFallback,
            candidate_match_n: 0,
        };
    }

    let pattern = normalize_snippet(snippet);
    if pattern.is_empty() {
        return AnchorResult {
            hunk_id: None,
            new_line_range: None,
            kind: AnchorKind::FileFallback,
            candidate_match_n: 0,
        };
    }

    // Collect all hunk matches. We gather ALL matches across all hunks (including
    // multiple occurrences within a single hunk) so the ambiguity count is exact.
    let mut matches: Vec<CandidateMatch> = Vec::new();
    for (idx, hunk) in file_changes.iter().enumerate() {
        matches.extend(matches_in_hunk(&pattern, hunk, idx));
    }

    let total = matches.len();

    match total {
        0 => AnchorResult {
            hunk_id: None,
            new_line_range: None,
            kind: AnchorKind::Unanchorable,
            candidate_match_n: 0,
        },
        1 => {
            // Exactly one match — Anchored.
            let m = &matches[0];
            AnchorResult {
                hunk_id: Some(m.hunk_id.clone()),
                new_line_range: Some((m.new_line_start, m.new_line_end)),
                kind: AnchorKind::Anchored,
                candidate_match_n: 1,
            }
        }
        n => {
            // Multiple matches — apply the hunk-proximity tiebreak if scope_hunk
            // was supplied and exactly one of the matches is inside it.
            if let Some(scope) = scope_hunk {
                let scope_matches: Vec<&CandidateMatch> =
                    matches.iter().filter(|m| m.hunk_id == scope).collect();
                if scope_matches.len() == 1 {
                    // Unique match in the preferred hunk — tiebreak wins.
                    let m = scope_matches[0];
                    return AnchorResult {
                        hunk_id: Some(m.hunk_id.clone()),
                        new_line_range: Some((m.new_line_start, m.new_line_end)),
                        kind: AnchorKind::Anchored,
                        candidate_match_n: n as u32,
                    };
                }
            }
            // Ambiguous: multiple matches with no tiebreak resolution.
            AnchorResult {
                hunk_id: None,
                new_line_range: None,
                kind: AnchorKind::Ambiguous,
                candidate_match_n: n as u32,
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focus_tunnel::diff::{DiffBodyKind, DiffBodyLine, DiffHunk};

    fn make_hunk(
        file_path: &str,
        old_start: usize,
        new_start: usize,
        lines: &[(&str, DiffBodyKind)],
    ) -> DiffHunk {
        let body_order: Vec<DiffBodyLine> = lines
            .iter()
            .map(|(text, kind)| DiffBodyLine {
                kind: *kind,
                text: text.to_string(),
            })
            .collect();
        let added_lines: Vec<String> = lines
            .iter()
            .filter(|(_, k)| *k == DiffBodyKind::Added)
            .map(|(t, _)| t.to_string())
            .collect();
        let removed_lines: Vec<String> = lines
            .iter()
            .filter(|(_, k)| *k == DiffBodyKind::Removed)
            .map(|(t, _)| t.to_string())
            .collect();
        let context_lines: Vec<String> = lines
            .iter()
            .filter(|(_, k)| *k == DiffBodyKind::Context)
            .map(|(t, _)| t.to_string())
            .collect();
        DiffHunk {
            file_path: file_path.to_string(),
            old_start,
            old_count: removed_lines.len() + context_lines.len(),
            new_start,
            new_count: added_lines.len() + context_lines.len(),
            context_lines,
            added_lines,
            removed_lines,
            body_order,
        }
    }

    /// Case (a): single-occurrence snippet -> Anchored with resolved hunk_id +
    /// new_line_range. This is the W-1 observable_effect case (a).
    #[test]
    fn single_match_is_anchored() {
        let hunk = make_hunk(
            "src/main.rs",
            10,
            10,
            &[
                ("+fn foo() {", DiffBodyKind::Added),
                ("+    bar();", DiffBodyKind::Added),
                ("+}", DiffBodyKind::Added),
            ],
        );
        let result = anchor_finding("fn foo() {\n    bar();\n}", &[hunk], None);
        assert_eq!(result.kind, AnchorKind::Anchored, "expected Anchored");
        assert!(result.hunk_id.is_some(), "hunk_id must be populated");
        assert!(result.new_line_range.is_some(), "new_line_range must be populated");
        assert_eq!(result.candidate_match_n, 1);
    }

    /// Case (b): duplicate snippet -> Ambiguous + file-fallback (NOT a guessed
    /// line). This is the W-1 observable_effect case (b) + stub_ban.
    #[test]
    fn duplicate_match_is_ambiguous_file_fallback() {
        let hunk = make_hunk(
            "src/main.rs",
            10,
            10,
            &[
                ("+fn foo() {}", DiffBodyKind::Added),
                ("+fn bar() {}", DiffBodyKind::Added),
                ("+fn foo() {}", DiffBodyKind::Added),
            ],
        );
        let result = anchor_finding("fn foo() {}", &[hunk], None);
        assert_eq!(result.kind, AnchorKind::Ambiguous, "expected Ambiguous");
        assert!(result.hunk_id.is_none(), "hunk_id must be None for Ambiguous");
        assert!(result.new_line_range.is_none(), "new_line_range must be None for Ambiguous");
        assert!(result.candidate_match_n > 1, "candidate_match_n must reflect duplicate count");
    }

    /// Case (c): absent snippet -> Unanchorable + file-fallback. W-1 case (c).
    #[test]
    fn absent_snippet_is_unanchorable() {
        let hunk = make_hunk(
            "src/main.rs",
            10,
            10,
            &[("+fn foo() {}", DiffBodyKind::Added)],
        );
        let result = anchor_finding("fn totally_absent() {}", &[hunk], None);
        assert_eq!(result.kind, AnchorKind::Unanchorable, "expected Unanchorable");
        assert!(result.hunk_id.is_none());
        assert_eq!(result.candidate_match_n, 0);
    }

    /// Empty snippet -> FileFallback (no model-asserted line, file-level finding).
    #[test]
    fn empty_snippet_is_file_fallback() {
        let result = anchor_finding("", &[], None);
        assert_eq!(result.kind, AnchorKind::FileFallback);
        assert!(result.hunk_id.is_none());
    }

    /// Hunk-proximity tiebreak: two matches in different hunks, scope_hunk
    /// picks the preferred one -> Anchored (tiebreak wins).
    #[test]
    fn scope_hunk_tiebreak_resolves_ambiguity() {
        let hunk_a = make_hunk(
            "src/main.rs",
            1,
            1,
            &[("+fn dup() {}", DiffBodyKind::Added)],
        );
        let hunk_b = make_hunk(
            "src/main.rs",
            20,
            20,
            &[("+fn dup() {}", DiffBodyKind::Added)],
        );
        let scope_id = crate::review_surface::hunk_id_for(&hunk_a);
        let result = anchor_finding("fn dup() {}", &[hunk_a, hunk_b], Some(&scope_id));
        assert_eq!(result.kind, AnchorKind::Anchored, "tiebreak must resolve to Anchored");
        assert_eq!(result.hunk_id.as_deref(), Some(scope_id.as_str()));
    }

    /// Normalize strips leading +/- and whitespace. Matches despite diff prefix.
    #[test]
    fn normalize_strips_diff_prefix() {
        assert_eq!(normalize_line("+    let x = 1;"), "let x = 1;");
        assert_eq!(normalize_line("-    let x = 1;"), "let x = 1;");
        assert_eq!(normalize_line(" context line  "), "context line");
        assert_eq!(normalize_line("no prefix"), "no prefix");
    }

    /// Empty body_order (legacy hunk without body) -> Unanchorable (not a panic).
    #[test]
    fn empty_body_order_returns_unanchorable_not_panic() {
        let hunk = DiffHunk {
            file_path: "src/main.rs".into(),
            old_start: 1,
            old_count: 0,
            new_start: 1,
            new_count: 1,
            context_lines: Vec::new(),
            added_lines: vec!["fn foo() {}".into()],
            removed_lines: Vec::new(),
            body_order: Vec::new(), // legacy hunk without body_order
        };
        let result = anchor_finding("fn foo() {}", &[hunk], None);
        // An empty body_order means the resolver cannot match; must not panic.
        assert_eq!(result.kind, AnchorKind::Unanchorable);
    }

    /// STUB_BAN: ReviewFinding has no model-settable `line` or `verdict` field.
    /// This test asserts by construction: we verify the struct fields at compile
    /// time (no `line: u32` or `verdict: String` field exists).
    #[test]
    fn review_finding_has_no_model_settable_line_field() {
        let finding = ReviewFinding {
            file_path: "src/main.rs".into(),
            kind: crate::review_surface::FindingKind::TestFailed,
            snippet: "fn foo() {}".into(),
            anchor: None,
        };
        // The only way to set the anchor is via anchor_finding (deterministic code).
        // Verifying the struct can be constructed without a `line` field is the test.
        let _ = finding;
    }
}
