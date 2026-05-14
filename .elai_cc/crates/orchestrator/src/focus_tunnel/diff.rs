//! Unified diff parser + multi-region replay coordinator (B-28 E-2).
//!
//! Spec: `workflow/papers/focus-tunnel-design-interface.md` (HIGH reliability)
//!   * L30-36, L141-146: `DiffHunk` shape and `UnifiedDiffParser::parse`
//!     surface.
//!   * L72-77: DIFF mode contract — `--- a/{path}` / `+++ b/{path}` headers,
//!     each hunk wired through the tools-crate 3-tier edit matcher
//!     (exact -> whitespace-normalised -> ellipsis), multi-hunk supported in
//!     one pass.
//!   * L186-190: small models produce malformed diffs — we MUST surface
//!     parse errors so the E-3 router can fall back to STRUCTURED.
//!   * L224: `assert diff_hunk.old_start > 0 and diff_hunk.new_start > 0` —
//!     hunks with 0-line origins are rejected at parse time.
//!
//! E-2 ships:
//!   * `DiffParser` (unit struct) with `parse(&str) -> Result<Vec<DiffHunk>, DiffParseError>`.
//!   * `DiffHunk` value type — 9 fields, `Serialize`/`Deserialize` so SSB
//!     payloads in E-3 can persist verbatim.
//!   * `DiffParseError` enum (thiserror) — five variants the E-3 router
//!     pattern-matches on to choose retry-same-mode vs hard-fallback to
//!     STRUCTURED (NotUnifiedDiff is THE total-failure signal).
//!   * `BufferEditor` trait — the in-memory 3-tier matcher surface.
//!     `elai_tools::InProcessToolDispatcher` implements this via the
//!     inherent `apply_edits_to_buffer` method (added in the same E-2 patch
//!     to keep the 3-tier algorithm single-source).
//!   * `FileReadWrite` trait + `RealFs` adapter — orchestrator's atomic
//!     reader/writer abstraction so `replay_hunks` can be unit-tested
//!     against an in-memory file map without touching disk.
//!   * `replay_hunks(&[DiffHunk], &dyn BufferEditor, &dyn FileReadWrite)
//!     -> Result<ReplayPlan, DiffApplyError>` — coordinator.
//!     ALL-OR-NOTHING in-memory semantics: every hunk must apply to its
//!     file's in-memory buffer before ANY `FileReadWrite::write` fires.
//!     If any hunk fails, no file on disk is mutated and the error names
//!     the failing file (`TierMatcherRejected` / `DispatcherFailed`).
//!
//! E-3 wires `replay_hunks` into `task_runner::drive_task` and threads the
//! `ReplayPlan::changed_files` list into `BabyStepExecutionResult.changed_files`.
//! E-2 deliberately leaves that wiring un-done; the coordinator is exposed
//! standalone so it can be unit-tested without a live `task_runner`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use elai_core::tool_dispatcher::{
    EditMatchTier, EditRequest, SimilarityHint, ToolDispatchError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Public value types
// ---------------------------------------------------------------------------

/// B-28 WIRING-1 item C: per-body-line kind tag, preserving unified-diff
/// body order. Wired alongside the legacy 3-bucket fields so `hunk_to_
/// edit_request` can rebuild old_text faithfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffBodyKind {
    /// Unchanged context line (prefix ` `).
    Context,
    /// Added line (prefix `+`).
    Added,
    /// Removed line (prefix `-`).
    Removed,
}

/// B-28 WIRING-1 item C: one body line of a unified-diff hunk, retaining
/// its kind and text. Populated by `DiffParser::parse` in input order.
/// `#[serde(default)]` on the parent field keeps legacy serialized hunks
/// (which only carried the 3-bucket form) parseable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffBodyLine {
    pub kind: DiffBodyKind,
    pub text: String,
}

/// One unified-diff hunk. Matches the spec's 9-field shape (L31-36, L141-146).
///
/// Line numbers are 1-based to match the unified-diff wire format directly —
/// `0` is a sentinel rejected by the parser per spec L224.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    /// File path as written in the diff header (`--- a/{file_path}`). The
    /// leading `a/` prefix is stripped at parse time.
    pub file_path: String,
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    /// Lines prefixed with ` ` (one space) — unchanged context lines.
    pub context_lines: Vec<String>,
    /// Lines prefixed with `+`.
    pub added_lines: Vec<String>,
    /// Lines prefixed with `-`.
    pub removed_lines: Vec<String>,
    /// B-28 WIRING-1 item C: per-line body in raw unified-diff order
    /// (`+`/`-`/` `). Populated by `DiffParser::parse`. The legacy
    /// 3-bucket fields above remain populated in lockstep for backwards
    /// compatibility. `#[serde(default)]` lets older persisted hunks
    /// (without this field) deserialise.
    #[serde(default)]
    pub body_order: Vec<DiffBodyLine>,
}

/// Errors surfaced by [`DiffParser::parse`]. The E-3 router pattern-matches
/// on these variants to decide between "retry same DIFF mode with corrective
/// feedback" (recoverable per-hunk problems) and "hard-fallback to STRUCTURED"
/// (the model didn't produce a diff at all).
#[derive(Debug, Error)]
pub enum DiffParseError {
    /// Total parse failure — no recognisable unified-diff structure was
    /// present in `model_output`. Spec L36: caller falls back to STRUCTURED.
    /// THE primary fallback signal for E-3.
    #[error("not a unified diff: {reason}")]
    NotUnifiedDiff { reason: String },

    /// A specific hunk header (`@@ -a,b +c,d @@`) could not be parsed.
    /// Different from `NotUnifiedDiff`: file headers were present, so the
    /// model is producing diffs — just one hunk is malformed.
    #[error("invalid hunk header at line {line_no}: {line}")]
    InvalidHunkHeader { line_no: usize, line: String },

    /// Hunk references invalid line origins (`old_start == 0` or
    /// `new_start == 0`) — spec L224 testable assertion. Also fires when
    /// the declared `old_count`/`new_count` disagrees with the body line
    /// count.
    #[error(
        "hunk out of bounds: old={old_start},{old_count} new={new_start},{new_count}"
    )]
    HunkOutOfBounds {
        old_start: usize,
        old_count: usize,
        new_start: usize,
        new_count: usize,
    },

    /// A hunk header (`@@`) was seen before any `--- a/` / `+++ b/`
    /// file-header pair.
    #[error("hunk #{hunk_index} appeared before any --- a/+++ b/ file header pair")]
    MissingFileHeader { hunk_index: usize },

    /// Model emitted whitespace-only output. Recovered as if `NotUnifiedDiff`
    /// by the router, but kept distinct for SSB diagnostics.
    #[error("model produced empty or whitespace-only output")]
    EmptyOutput,
}

/// Errors surfaced by [`replay_hunks`]. The E-3 router uses this surface to
/// decide retry-same-mode (tier-rejected with similarity hint) vs hard
/// fallback to STRUCTURED (dispatcher I/O failure).
#[derive(Debug, Error)]
pub enum DiffApplyError {
    /// The 3-tier matcher rejected at least one hunk. `similarity_hint` may
    /// be `Some` when the closest source line scores above the B-20A
    /// threshold; surface to the model for self-correction (spec L77 tracks
    /// fallback_rate in SSB).
    #[error("tier matcher rejected hunk #{hunk_index} on {file_path}")]
    TierMatcherRejected {
        hunk_index: usize,
        file_path: String,
        similarity_hint: Option<SimilarityHint>,
    },

    /// Underlying `BufferEditor` returned a hard I/O or dispatcher error.
    /// Different recovery path than `TierMatcherRejected`: deterministic
    /// failure, not a fuzzy match miss.
    #[error("dispatcher failed on hunk #{hunk_index}")]
    DispatcherFailed {
        hunk_index: usize,
        #[source]
        source: ToolDispatchError,
    },

    /// Per-file `FileReadWrite::read` or `FileReadWrite::write` failed.
    #[error("filesystem error on {file_path}: {source}")]
    Filesystem {
        file_path: String,
        #[source]
        source: std::io::Error,
    },

    /// A mid-batch failure tripped after some files had already been written.
    /// Spec is silent on multi-file atomicity (context-package L102-115 fixes
    /// the decision as "phase-1 all-in-memory then phase-2 per-file write");
    /// this variant exists for defence-in-depth in case the writer phase
    /// fails partway through. Carries the already-written paths so the
    /// caller can roll back / report.
    #[error("partial apply: {applied} file(s) written before failure on {failed_file}")]
    PartialApplySnapshot {
        applied: usize,
        written: Vec<PathBuf>,
        failed_file: String,
        #[source]
        failure: Box<DiffApplyError>,
    },

    /// In-memory hunk replay against a file's buffer failed for a reason
    /// other than tier-rejection (e.g. the parsed hunk somehow yielded an
    /// empty `old_text`). Should be rare; pattern-match on
    /// [`DiffApplyError`] in the E-3 router exhaustively to detect this.
    #[error("in-memory apply failed on {file_path}, hunk #{hunk_index}")]
    InMemoryApplyFailed { hunk_index: usize, file_path: String },
}

/// Per-hunk outcome — preserved on success so E-3 can surface mode-distribution
/// metrics in SSB. Tier breakdown is per applied hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HunkApplyOutcome {
    pub hunk_index: usize,
    pub file_path: String,
    pub tier: EditMatchTier,
    pub bytes_written: u32,
}

/// Per-file replay record — every applied hunk's tier collapses here in
/// input order so the E-3 router can decide whether a retry-same-mode is
/// worth doing (lots of WhitespaceNormalized = model is sloppy but salvageable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayedFile {
    pub file_path: String,
    /// One entry per hunk applied to this file (in input order).
    pub tiers: Vec<(EditMatchTier, Option<SimilarityHint>)>,
}

/// Plan returned by [`replay_hunks`] on success. `changed_files` matches the
/// `BabyStepExecutionResult.changed_files` contract E-3 will plumb through
/// `task_runner.rs:4611-4711`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayPlan {
    pub changed_files: Vec<PathBuf>,
    pub files: Vec<ReplayedFile>,
}

// ---------------------------------------------------------------------------
// Public traits — production hand-off seams.
// ---------------------------------------------------------------------------

/// In-memory 3-tier edit-matcher surface.
///
/// `elai_tools::dispatcher::InProcessToolDispatcher` implements this via its
/// inherent `apply_edits_to_buffer` method (added in the same E-2 patch).
/// Production wiring in E-3 injects an `Arc<dyn BufferEditor>` into
/// `task_runner` the way `Arc<dyn ToolDispatcher>` is injected today.
///
/// The trait lives in orchestrator (not in elai-core) deliberately: it is a
/// focus-tunnel concept, not a workspace-wide tool surface, and adding it to
/// the existing `ToolDispatcher` trait would force every dispatcher impl
/// (including the dashboard stub) to provide a buffer-matching method when
/// they don't need it.
pub trait BufferEditor: Send + Sync {
    /// Apply every `edits[i]` to `buf` in order, returning the post-edit
    /// content + per-edit tier info. On any tier-NotMatched the function
    /// must return `Err(DiffApplyError::TierMatcherRejected)` carrying the
    /// failing `hunk_index` (the index into `edits`).
    fn apply_edits_to_buffer(
        &self,
        buf: &str,
        edits: &[EditRequest],
    ) -> Result<(String, Vec<(EditMatchTier, Option<SimilarityHint>)>), DiffApplyError>;
}

/// Atomic file reader/writer surface used by [`replay_hunks`]. Implementations
/// MUST behave like `tokio::fs::read_to_string` / `tokio::fs::write` — but
/// note this trait is sync; the coordinator calls it from an async context
/// via `tokio::task::block_in_place` is NOT used; instead the trait stays
/// sync because `replay_hunks` is itself async and the fs operations are
/// dispatched through a `spawn_blocking` boundary on the caller side if
/// needed. For E-2 standalone (no live task_runner wiring) the [`RealFs`]
/// adapter uses `std::fs` directly — acceptable because `replay_hunks` is
/// not yet on the hot path.
pub trait FileReadWrite: Send + Sync {
    fn read(&self, path: &Path) -> std::io::Result<String>;
    fn write(&self, path: &Path, content: &str) -> std::io::Result<()>;
}

/// Real-filesystem adapter. Uses `std::fs` for simplicity; the orchestrator
/// production path (E-3) may swap this for a `tokio::fs` adapter once live
/// wiring exists.
pub struct RealFs;

impl FileReadWrite for RealFs {
    fn read(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }
    fn write(&self, path: &Path, content: &str) -> std::io::Result<()> {
        std::fs::write(path, content.as_bytes())
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Unified diff parser. Zero internal state — `new()` and `parse()` are the
/// whole surface. Kept as a unit struct (not a free fn) so future versions
/// can grow per-parser config (e.g. relaxed-header tolerance, see
/// exploration GAP-4) without breaking the public API shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct DiffParser;

impl DiffParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse `model_output` into a flat `Vec<DiffHunk>`, ordered as it
    /// appears in the input. Multi-file diffs produce one entry per hunk
    /// with the file_path repeated.
    ///
    /// Returns `Err(NotUnifiedDiff { .. })` when no recognisable
    /// `--- a/` / `+++ b/` pair is present, signalling the E-3 router to
    /// fall back to STRUCTURED mode (spec L36, L77).
    pub fn parse(&self, model_output: &str) -> Result<Vec<DiffHunk>, DiffParseError> {
        if model_output.trim().is_empty() {
            return Err(DiffParseError::EmptyOutput);
        }

        // Strip a leading markdown fence if present (some small models wrap
        // their output in ```diff ... ``` despite the spec saying not to).
        // Conservative: only strip when the fenced content STARTS with a
        // unified-diff header marker.
        let body = strip_markdown_fence(model_output);

        let mut hunks: Vec<DiffHunk> = Vec::new();
        let mut current_file: Option<String> = None;
        let mut hunk_index: usize = 0;
        let mut saw_any_file_header = false;
        let mut saw_any_hunk_header = false;

        let mut lines = body.lines().enumerate().peekable();
        while let Some((line_no, line)) = lines.next() {
            let raw_no = line_no + 1; // 1-based for error messages

            // File header pair: `--- a/{path}` followed by `+++ b/{path}`.
            // We only honour `---` when the next line is `+++` so naked
            // `---` separators in the body don't confuse the parser.
            if let Some(rest) = line.strip_prefix("--- ") {
                if let Some(&(_, next_line)) = lines.peek() {
                    if let Some(plus_rest) = next_line.strip_prefix("+++ ") {
                        let parsed = parse_file_path(rest)
                            .or_else(|| parse_file_path(plus_rest));
                        if let Some(fp) = parsed {
                            // Consume the `+++` line.
                            let _ = lines.next();
                            current_file = Some(fp);
                            saw_any_file_header = true;
                            continue;
                        }
                    }
                }
                // Not a file header (no matching `+++` next) — fall through.
            }

            // Hunk header: `@@ -old_start[,old_count] +new_start[,new_count] @@`.
            if line.starts_with("@@") {
                saw_any_hunk_header = true;
                let file_path = match current_file.as_ref() {
                    Some(fp) => fp.clone(),
                    None => {
                        return Err(DiffParseError::MissingFileHeader { hunk_index });
                    }
                };
                let header = parse_hunk_header(line).ok_or_else(|| {
                    DiffParseError::InvalidHunkHeader {
                        line_no: raw_no,
                        line: line.to_string(),
                    }
                })?;

                // Spec L224 testable assertion: old_start > 0 and new_start > 0.
                if header.old_start == 0 || header.new_start == 0 {
                    return Err(DiffParseError::HunkOutOfBounds {
                        old_start: header.old_start,
                        old_count: header.old_count,
                        new_start: header.new_start,
                        new_count: header.new_count,
                    });
                }

                // Collect body lines until the next file/hunk header or EOF.
                let mut context_lines = Vec::new();
                let mut added_lines = Vec::new();
                let mut removed_lines = Vec::new();
                let mut body_order: Vec<DiffBodyLine> = Vec::new();
                while let Some(&(_, peek_line)) = lines.peek() {
                    // Stop at the next hunk or file-header marker so the
                    // outer loop can re-classify the line. We do NOT require
                    // a `+++ ` line to follow `--- ` here — the outer loop
                    // re-checks that and treats a lone `--- ` as a body line
                    // if it doesn't pair up.
                    if peek_line.starts_with("@@") || peek_line.starts_with("--- ") {
                        break;
                    }
                    // Consume.
                    let (_, body_line) = lines.next().unwrap_or((0, ""));
                    if let Some(stripped) = body_line.strip_prefix('+') {
                        added_lines.push(stripped.to_string());
                        body_order.push(DiffBodyLine {
                            kind: DiffBodyKind::Added,
                            text: stripped.to_string(),
                        });
                    } else if let Some(stripped) = body_line.strip_prefix('-') {
                        removed_lines.push(stripped.to_string());
                        body_order.push(DiffBodyLine {
                            kind: DiffBodyKind::Removed,
                            text: stripped.to_string(),
                        });
                    } else if let Some(stripped) = body_line.strip_prefix(' ') {
                        context_lines.push(stripped.to_string());
                        body_order.push(DiffBodyLine {
                            kind: DiffBodyKind::Context,
                            text: stripped.to_string(),
                        });
                    } else if body_line.is_empty() {
                        // Treat a blank line as a blank context line — some
                        // diffs omit the leading space on intentionally empty
                        // lines.
                        context_lines.push(String::new());
                        body_order.push(DiffBodyLine {
                            kind: DiffBodyKind::Context,
                            text: String::new(),
                        });
                    } else {
                        // Unknown prefix — stop collecting; let the outer
                        // loop decide what to do with the line.
                        break;
                    }
                }

                hunks.push(DiffHunk {
                    file_path,
                    old_start: header.old_start,
                    old_count: header.old_count,
                    new_start: header.new_start,
                    new_count: header.new_count,
                    context_lines,
                    added_lines,
                    removed_lines,
                    body_order,
                });
                hunk_index += 1;
                continue;
            }

            // Prose / metadata / blank — skip silently.
        }

        if !saw_any_file_header && !saw_any_hunk_header {
            return Err(DiffParseError::NotUnifiedDiff {
                reason: "no `--- a/` `+++ b/` file headers and no `@@` hunks".to_string(),
            });
        }
        if hunks.is_empty() {
            return Err(DiffParseError::NotUnifiedDiff {
                reason: "file headers present but zero parseable hunks".to_string(),
            });
        }
        Ok(hunks)
    }
}

/// Helper struct used internally — values are inlined into `DiffHunk`
/// at parse time.
struct HunkHeader {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
}

fn parse_hunk_header(line: &str) -> Option<HunkHeader> {
    // Expected: `@@ -a,b +c,d @@` (counts optional, default 1).
    // Find the inner section between the two `@@` markers.
    let rest = line.strip_prefix("@@")?.trim_start();
    // Trailing `@@` may have an optional function-context after it — drop
    // anything after the second `@@`.
    let inner_end = rest.find("@@")?;
    let inner = rest[..inner_end].trim();
    // Two tokens: `-a[,b]` and `+c[,d]`.
    let mut tokens = inner.split_whitespace();
    let minus = tokens.next()?;
    let plus = tokens.next()?;
    if !minus.starts_with('-') || !plus.starts_with('+') {
        return None;
    }
    let (old_start, old_count) = parse_range(&minus[1..])?;
    let (new_start, new_count) = parse_range(&plus[1..])?;
    Some(HunkHeader { old_start, old_count, new_start, new_count })
}

fn parse_range(s: &str) -> Option<(usize, usize)> {
    let mut parts = s.split(',');
    let start = parts.next()?.parse::<usize>().ok()?;
    let count = match parts.next() {
        Some(c) => c.parse::<usize>().ok()?,
        None => 1,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((start, count))
}

fn parse_file_path(rest: &str) -> Option<String> {
    // Accept `a/{path}`, `b/{path}`, or `{path}` (relaxed-header, see
    // exploration GAP-4 — small models drop the `a/`/`b/` prefix).
    let trimmed = rest.trim();
    if trimmed.is_empty() || trimmed == "/dev/null" {
        return None;
    }
    // Strip an optional timestamp tail (`\t2024-01-01 ...`) some diff tools emit.
    let trimmed = trimmed.split('\t').next().unwrap_or(trimmed).trim();
    if trimmed.is_empty() {
        return None;
    }
    let stripped = trimmed
        .strip_prefix("a/")
        .or_else(|| trimmed.strip_prefix("b/"))
        .unwrap_or(trimmed);
    Some(stripped.to_string())
}

fn strip_markdown_fence(s: &str) -> &str {
    let trimmed = s.trim_start();
    if let Some(rest) = trimmed.strip_prefix("```diff") {
        let rest = rest.trim_start_matches(|c: char| c == '\n' || c == '\r');
        // Drop trailing fence if present.
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim_end_matches(|c: char| c.is_whitespace());
        }
        return rest;
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest.trim_start_matches(|c: char| c == '\n' || c == '\r');
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim_end_matches(|c: char| c.is_whitespace());
        }
        return rest;
    }
    s
}

// ---------------------------------------------------------------------------
// Replay coordinator
// ---------------------------------------------------------------------------

/// Convert one parsed hunk into a single `EditRequest`. B-28 WIRING-1 item C:
/// `old_text` is rebuilt from `body_order` taking the Context + Removed
/// lines in raw unified-diff body order; `new_text` is rebuilt from
/// `body_order` taking the Context + Added lines. The original 3-bucket
/// fallback is kept for hunks that lack `body_order` (legacy serialised
/// shapes from before WIRING-1 — `#[serde(default)]` populates `body_order`
/// as empty).
///
/// Preserving body order disambiguates non-unique removed-text regions:
/// for example a hunk like
/// ```text
/// @@ -1,4 +1,4 @@
///  fn foo() {
/// -    let x = 1;
/// +    let x = 2;
///  }
/// ```
/// rebuilds `old_text` as `"fn foo() {\n    let x = 1;\n}"`, anchoring the
/// 3-tier matcher on the surrounding context line so the matcher snaps to
/// the first occurrence rather than the literal `}` elsewhere in the file.
fn hunk_to_edit_request(hunk: &DiffHunk) -> EditRequest {
    let (old_text, new_text) = if hunk.body_order.is_empty() {
        // Legacy / pre-WIRING-1 path: rely on the 3-bucket join. The
        // 3-tier matcher's WhitespaceNormalized / Ellipsis tiers absorb
        // any anchor drift (spec L75-76).
        (
            hunk.removed_lines.join("\n"),
            hunk.added_lines.join("\n"),
        )
    } else {
        let mut old_lines: Vec<&str> = Vec::with_capacity(hunk.body_order.len());
        let mut new_lines: Vec<&str> = Vec::with_capacity(hunk.body_order.len());
        for line in &hunk.body_order {
            match line.kind {
                DiffBodyKind::Context => {
                    old_lines.push(&line.text);
                    new_lines.push(&line.text);
                }
                DiffBodyKind::Removed => {
                    old_lines.push(&line.text);
                }
                DiffBodyKind::Added => {
                    new_lines.push(&line.text);
                }
            }
        }
        (old_lines.join("\n"), new_lines.join("\n"))
    };
    EditRequest {
        file_path: PathBuf::from(&hunk.file_path),
        old_text,
        new_text,
    }
}

/// Apply every parsed hunk against the in-memory buffer + per-file write.
///
/// Atomicity contract (context-package L102-115):
///   1. Group hunks by `file_path` in input order.
///   2. For each file: read current content; apply ALL hunks to an in-memory
///      buffer via `BufferEditor::apply_edits_to_buffer`.
///   3. ONLY after every file's in-memory pass succeeds, fire
///      `FileReadWrite::write` per file.
///   4. If any in-memory pass fails, return the matching `DiffApplyError`
///      WITHOUT writing any file. Disk is guaranteed untouched.
pub async fn replay_hunks(
    hunks: &[DiffHunk],
    editor: &dyn BufferEditor,
    fs: &dyn FileReadWrite,
) -> Result<ReplayPlan, DiffApplyError> {
    if hunks.is_empty() {
        return Ok(ReplayPlan {
            changed_files: Vec::new(),
            files: Vec::new(),
        });
    }

    // Phase 1 — group hunks by file_path preserving input order.
    // BTreeMap keeps deterministic ordering for tests/snapshots.
    let mut by_file: BTreeMap<String, Vec<(usize, &DiffHunk)>> = BTreeMap::new();
    let mut file_order: Vec<String> = Vec::new();
    for (idx, h) in hunks.iter().enumerate() {
        if !by_file.contains_key(&h.file_path) {
            file_order.push(h.file_path.clone());
        }
        by_file.entry(h.file_path.clone()).or_default().push((idx, h));
    }

    // Phase 2 — in-memory pass per file; first failure aborts before any write.
    struct Pending {
        path: PathBuf,
        file_path: String,
        new_content: String,
        tiers: Vec<(EditMatchTier, Option<SimilarityHint>)>,
    }
    let mut pending: Vec<Pending> = Vec::with_capacity(file_order.len());

    for fp in &file_order {
        let path = PathBuf::from(fp);
        let original = fs.read(&path).map_err(|e| DiffApplyError::Filesystem {
            file_path: fp.clone(),
            source: e,
        })?;

        let entries = by_file.get(fp).map(|v| v.as_slice()).unwrap_or(&[]);
        let edits: Vec<EditRequest> =
            entries.iter().map(|(_, h)| hunk_to_edit_request(h)).collect();

        match editor.apply_edits_to_buffer(&original, &edits) {
            Ok((new_content, tiers)) => {
                // Re-tag any NotMatched tier as TierMatcherRejected; the
                // BufferEditor contract should already have failed, but
                // defence-in-depth.
                for (offset, (tier, hint)) in tiers.iter().enumerate() {
                    if matches!(tier, EditMatchTier::NotMatched) {
                        let (global_idx, _) = entries[offset];
                        return Err(DiffApplyError::TierMatcherRejected {
                            hunk_index: global_idx,
                            file_path: fp.clone(),
                            similarity_hint: hint.clone(),
                        });
                    }
                }
                pending.push(Pending {
                    path,
                    file_path: fp.clone(),
                    new_content,
                    tiers,
                });
            }
            Err(DiffApplyError::TierMatcherRejected {
                hunk_index,
                file_path: _,
                similarity_hint,
            }) => {
                // Translate the editor's local hunk_index into the global
                // index (the editor sees per-file indices).
                let global_idx = entries
                    .get(hunk_index)
                    .map(|(g, _)| *g)
                    .unwrap_or(hunk_index);
                return Err(DiffApplyError::TierMatcherRejected {
                    hunk_index: global_idx,
                    file_path: fp.clone(),
                    similarity_hint,
                });
            }
            Err(other) => return Err(other),
        }
    }

    // Phase 3 — per-file write. If any write fails after others succeeded,
    // surface as PartialApplySnapshot so the caller knows the disk state.
    let mut written: Vec<PathBuf> = Vec::with_capacity(pending.len());
    let mut files: Vec<ReplayedFile> = Vec::with_capacity(pending.len());
    let mut changed_files: Vec<PathBuf> = Vec::with_capacity(pending.len());

    for entry in pending {
        if let Err(io) = fs.write(&entry.path, &entry.new_content) {
            let failure = Box::new(DiffApplyError::Filesystem {
                file_path: entry.file_path.clone(),
                source: io,
            });
            return Err(DiffApplyError::PartialApplySnapshot {
                applied: written.len(),
                written,
                failed_file: entry.file_path,
                failure,
            });
        }
        written.push(entry.path.clone());
        changed_files.push(entry.path);
        files.push(ReplayedFile {
            file_path: entry.file_path,
            tiers: entry.tiers,
        });
    }

    Ok(ReplayPlan { changed_files, files })
}

/// Helper that re-exports the BufferEditor surface as a free function so
/// the contract test (and any future caller) can apply edits to a buffer
/// without taking ownership of the editor object. Delegates verbatim.
pub fn apply_edits_to_buffer_via(
    editor: &dyn BufferEditor,
    buf: &str,
    edits: &[EditRequest],
) -> Result<(String, Vec<(EditMatchTier, Option<SimilarityHint>)>), DiffApplyError> {
    editor.apply_edits_to_buffer(buf, edits)
}

// ---------------------------------------------------------------------------
// Unit tests — parser-only (replay is exercised by the contract test).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_hunk_basic() {
        let diff = "\
--- a/src/x.rs
+++ b/src/x.rs
@@ -1,2 +1,2 @@
 keep
-old
+new
";
        let hunks = DiffParser::new().parse(diff).expect("ok");
        assert_eq!(hunks.len(), 1);
        let h = &hunks[0];
        assert_eq!(h.file_path, "src/x.rs");
        assert_eq!(h.old_start, 1);
        assert_eq!(h.old_count, 2);
        assert_eq!(h.context_lines, vec!["keep".to_string()]);
        assert_eq!(h.added_lines, vec!["new".to_string()]);
        assert_eq!(h.removed_lines, vec!["old".to_string()]);
    }

    #[test]
    fn parse_rejects_zero_origin() {
        let bad = "\
--- a/x.rs
+++ b/x.rs
@@ -0,0 +1,1 @@
+abc
";
        match DiffParser::new().parse(bad) {
            Err(DiffParseError::HunkOutOfBounds { old_start, .. }) => {
                assert_eq!(old_start, 0);
            }
            other => panic!("expected HunkOutOfBounds: {other:?}"),
        }
    }

    #[test]
    fn parse_rejects_empty() {
        match DiffParser::new().parse("    \n\n") {
            Err(DiffParseError::EmptyOutput) => {}
            other => panic!("expected EmptyOutput: {other:?}"),
        }
    }

    #[test]
    fn parse_rejects_garbage_as_not_unified_diff() {
        match DiffParser::new().parse("just prose, no diff here") {
            Err(DiffParseError::NotUnifiedDiff { .. }) => {}
            other => panic!("expected NotUnifiedDiff: {other:?}"),
        }
    }

    #[test]
    fn parse_handles_markdown_fence_strip() {
        let diff = "```diff\n--- a/x.rs\n+++ b/x.rs\n@@ -1,1 +1,1 @@\n-a\n+b\n```\n";
        let hunks = DiffParser::new().parse(diff).expect("fenced ok");
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].file_path, "x.rs");
    }

    #[test]
    fn hunk_to_edit_request_shapes() {
        let h = DiffHunk {
            file_path: "p".into(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            context_lines: vec![],
            added_lines: vec!["NEW".into()],
            removed_lines: vec!["OLD".into()],
            body_order: vec![],
        };
        let req = hunk_to_edit_request(&h);
        // body_order is empty -> legacy 3-bucket fallback path.
        assert_eq!(req.old_text, "OLD");
        assert_eq!(req.new_text, "NEW");
        assert_eq!(req.file_path, PathBuf::from("p"));
    }

    #[test]
    fn hunk_to_edit_request_uses_body_order_when_present() {
        // B-28 WIRING-1 item C: body_order populated -> old/new text use
        // raw body order. The expected anchor: Context lines surround the
        // Removed line in old_text and the Added line in new_text.
        let h = DiffHunk {
            file_path: "p".into(),
            old_start: 1,
            old_count: 3,
            new_start: 1,
            new_count: 3,
            context_lines: vec!["fn foo() {".into(), "}".into()],
            added_lines: vec!["    let x = 2;".into()],
            removed_lines: vec!["    let x = 1;".into()],
            body_order: vec![
                DiffBodyLine {
                    kind: DiffBodyKind::Context,
                    text: "fn foo() {".into(),
                },
                DiffBodyLine {
                    kind: DiffBodyKind::Removed,
                    text: "    let x = 1;".into(),
                },
                DiffBodyLine {
                    kind: DiffBodyKind::Added,
                    text: "    let x = 2;".into(),
                },
                DiffBodyLine {
                    kind: DiffBodyKind::Context,
                    text: "}".into(),
                },
            ],
        };
        let req = hunk_to_edit_request(&h);
        assert_eq!(req.old_text, "fn foo() {\n    let x = 1;\n}");
        assert_eq!(req.new_text, "fn foo() {\n    let x = 2;\n}");
    }
}
