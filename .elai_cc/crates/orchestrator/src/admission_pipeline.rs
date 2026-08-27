//! J-109 UNIT A1: learned-artifact admission hardening -- the deterministic
//! AD-2 immutable-surface dedup leg, ported as ONE pure decision function in
//! front of the existing learned-artifact write seams.
//!
//! WHAT THIS IS (plan J-109, UNITS A1 / grounding: AgenticSTS write-gate cascade,
//! `write_gate.py::WriteGate.check` L3a over StaticSpanIndex :323): a candidate
//! learned artifact (`/remember` text, and from UNIT A2 a rule body) is compared
//! lexically against spans built from the J-102 admission-manifest surface.
//! Candidates that substantially restate doctrine get `RejectImmutableSpan` --
//! they are rules already sitting on disk, not new lessons.
//!
//! COMPOSITION LAW (GD-1, binding):
//!   * The span index is built STRICTLY FROM [`elai_core::config::ConfigLoader::
//!     admission_scan()`] rows -- the J-102 manifest enumeration IS the only path
//!     list; this module never enumerates paths itself. Spans derive ONLY from
//!     those rows' files' texts.
//!   * AD-1 exact/near-exact match stays owned upstream by the E-5 memory
//!     dispatcher's ADD/supersede/noop arms -- no second exact-match arm exists
//!     here (FORKS item 5, permanent descope).
//!   * Staleness is checked and the index rebuilt-if-stale BEFORE any verdict is
//!     computed ([`rebuild_if_stale`], mirroring the steal's rebuild_if_stale).
//!   * Fully deterministic, observe-only: no inference calls anywhere in this
//!     file, ever (TD-3), and every verdict carries receipts sufficient to
//!     recompute the decision from the emitted SSB row alone (GD-2).

use std::fs;

use elai_core::config::AdmissionScanRow;

/// AD-2 lexical reject threshold for the immutable-surface leg. Starting value
/// transferred from the AgenticSTS immutable-reject band boundary
/// (`write_gate.py:43-52`, summary: "reject if candidate cosine-overlaps an
/// IMMUTABLE span (>=0.70)"); the lexical containment replaces cosine until
/// UNIT A3 lands the embedder leg. ID-5: this is a STARTING value recalibrated
/// from observe-only rows once they accumulate.
pub const T_IMMUTABLE_LEXICAL: f32 = 0.70;

/// Maximum manifest span size, in tokens. Manifest row text is chunked into
/// sliding windows of at most this many tokens so a one-line doctrine statement
/// inside a large CLAUDE.md-class file cannot be diluted below the threshold by
/// unrelated surrounding text (row granularity proved too coarse for AD-2; the
/// chunking choice per plan A1.1 ON_FAIL runs over row TEXT ONLY).
pub const MAX_SPAN_TOKENS: usize = 48;

/// Sliding-window stride between consecutive chunks of one manifest file, in
/// tokens. Half the window so a duplicated statement straddling a boundary is
/// still fully contained in some chunk.
const CHUNK_STRIDE_TOKENS: usize = MAX_SPAN_TOKENS / 2;

/// surface_leg label carried on rows produced by this pure lexical leg (UNIT A3
/// adds the embedder leg with its own label; infrastructure failure degrades TO
/// this leg labeled honestly -- never a silent pass).
pub const SURFACE_LEG_LEXICAL: &str = "lexical";

// ---------------------------------------------------------------------------
// Verdict enum + receipts
// ---------------------------------------------------------------------------

/// The full admission-pipeline verdict space. UNIT A1 arms exercised through the
/// lexical AD-2 leg: `AdmitClean` + `RejectImmutableSpan`. The remaining arms are
/// reached progressively by later units (AD-3 dup/merge/coexist band, UNIT A3) --
/// the enum shape is pinned NOW so rows emitted today stay re-interpretable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionCheckVerdict {
    /// No leg matched -- candidate may proceed (observe-only here either way).
    AdmitClean,
    /// Candidate substantially restates a manifest (doctrine) span.
    RejectImmutableSpan,
    /// Near-duplicate of an existing store entry (>= T_DUP; UNIT A3).
    RejectDuplicate,
    /// Inside the dup/merge band with matching trigger context (UNIT A3).
    Merge,
    /// Inside the dup/merge band but a distinct variant (UNIT A3).
    Coexist,
}

impl AdmissionCheckVerdict {
    /// Stable snake_case receipt string for the SSB verdict field (GD-2: the row
    /// alone must carry enough to recompute/audit the decision).
    #[must_use]
    pub fn as_receipt_str(&self) -> &'static str {
        match self {
            Self::AdmitClean => "admit_clean",
            Self::RejectImmutableSpan => "reject_immutable_span",
            Self::RejectDuplicate => "reject_duplicate",
            Self::Merge => "merge",
            Self::Coexist => "coexist",
        }
    }
}

/// Receipts carried on EVERY verdict row (GD-2): hash identifies the exact
/// candidate bytes probed, leg + threshold identify the pure function that ran,
/// verdict states the outcome -- recomputable from the emitted row alone.
#[derive(Debug, Clone, PartialEq)]
pub struct AdmissionReceipts {
    /// blake3 hex (64 chars) over the exact candidate text that was probed.
    /// Raw text never rides the wire (SSB stores blake3 digests BY DESIGN).
    pub candidate_hash: String,
    /// Which leg produced the verdict (`"lexical"` in UNIT A1).
    pub surface_leg: &'static str,
    /// Threshold value in force when the decision was made.
    pub threshold_used: f32,
    /// Stable snake_case verdict string (see [`AdmissionCheckVerdict::as_receipt_str`]).
    pub verdict: &'static str,
}

/// Verdict + receipts bundle returned by [`AdmissionPipeline::evaluate_candidate`].
#[derive(Debug, Clone, PartialEq)]
pub struct AdmissionEvaluation {
    /// The pipeline verdict.
    pub verdict: AdmissionCheckVerdict,
    /// Receipts to journal beside the verdict (GD-2).
    pub receipts: AdmissionReceipts,
}

// ---------------------------------------------------------------------------
// Span index (built STRICTLY from J-102 admission_scan rows)
// ---------------------------------------------------------------------------

/// One chunk of one manifest artifact. `text`/`tokens` derive ONLY from the
/// scan row's file content; `source_path` echoes the row's path verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionSpan {
    /// Path exactly as enumerated by `admission_scan()` (row authority).
    pub source_path: String,
    /// Raw chunk text (<= [`MAX_SPAN_TOKENS`] tokens, sliding window).
    pub text: String,
    /// Tokenized chunk used by the lexical scorer (see [`tokenize`]).
    pub tokens: Vec<String>,
}

/// blake3 fingerprint over the manifest rows: hex digest of the rows'
/// `content_hash` values (one per row, `\n`-joined in SORTED order). Sorting
/// makes the fingerprint independent of `read_dir` enumeration order, so the
/// same manifest snapshot always hashes identically across processes; any
/// membership or content change still moves it. The staleness contract
/// compares THIS against a cached index before any verdict.
#[must_use]
pub fn rows_fingerprint(rows: &[AdmissionScanRow]) -> String {
    let mut hashes: Vec<&str> = rows.iter().map(|row| row.content_hash.as_str()).collect();
    hashes.sort_unstable();
    let joined = hashes.join("\n");
    blake3::hash(joined.as_bytes()).to_hex().to_string()
}

/// `true` iff the cached index no longer matches the fresh manifest rows.
#[must_use]
pub fn index_is_stale(index: &StaticSpanIndex, rows: &[AdmissionScanRow]) -> bool {
    index.fingerprint != rows_fingerprint(rows)
}

/// Cached-index front door mirroring the steal's `rebuild_if_stale`: check
/// staleness FIRST, rebuild if stale, and only then let callers reach any
/// verdict (the guarantee contract arm (c) pins). Returns whether a rebuild
/// happened. After the call, `cache` holds `Some(index)` whenever `rows`
/// contain at least one readable manifest artifact.
///
/// An empty manifest leaves `cache` untouched-or-empty: there is no immutable
/// surface to dedup against, and emitting an `admit_clean` row against a
/// surface that was never scanned would fake coverage (GD-5 honesty), so the
/// CALLER treats `None` as "no probe possible", never as a clean bill.
pub fn rebuild_if_stale(cache: &mut Option<StaticSpanIndex>, rows: &[AdmissionScanRow]) -> bool {
    let stale = cache
        .as_ref()
        .map_or(true, |idx| index_is_stale(idx, rows));
    if stale {
        *cache = Some(StaticSpanIndex::from_scan_rows(rows));
    }
    stale
}

/// The static span index over the J-102 instruction-surface manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticSpanIndex {
    /// blake3 over the rows' content_hash values (see [`rows_fingerprint`]).
    fingerprint: String,
    /// Chunks derived from the rows' texts (sliding windows over each file).
    spans: Vec<AdmissionSpan>,
}

impl StaticSpanIndex {
    /// Build the index STRICTLY from `admission_scan()` rows (GD-1). Each row's
    /// file is read back for its text; unreadable/non-UTF-8 artifacts contribute
    /// no spans (the same visibility rule `admission_scan()` itself applies) and
    /// still count toward the fingerprint via their row hashes. The fingerprint
    /// stays row-authoritative even in the race where a file changes between the
    /// two reads -- staleness is decided by the ROWS, the texts are their best-
    /// effort projection.
    #[must_use]
    pub fn from_scan_rows(rows: &[AdmissionScanRow]) -> Self {
        let mut spans = Vec::new();
        for row in rows {
            let Ok(text) = fs::read_to_string(&row.path) else {
                continue;
            };
            spans.extend(chunk_into_spans(row.path.to_string_lossy().as_ref(), &text));
        }
        Self {
            fingerprint: rows_fingerprint(rows),
            spans,
        }
    }

    /// Row-authoritative fingerprint of the manifest this index was built from.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// The manifest spans (chunks over row texts, in scan order).
    #[must_use]
    pub fn spans(&self) -> &[AdmissionSpan] {
        &self.spans
    }
}

/// Split one manifest file's text into sliding windows of at most
/// [`MAX_SPAN_TOKENS`] tokens (stride [`CHUNK_STRIDE_TOKENS`]; a short file is a
/// single chunk). Chunking runs ONLY over the row's own text -- never any other
/// source (GD-1/A2).
fn chunk_into_spans(source_path: &str, text: &str) -> Vec<AdmissionSpan> {
    let tokens = tokenize(text);
    let last_start = tokens.len().saturating_sub(1);
    let stride = CHUNK_STRIDE_TOKENS.max(1);
    let mut starts: Vec<usize> = (0..=last_start).step_by(stride).collect();
    if tokens.is_empty() {
        starts.clear();
    } else if starts.last().copied() != Some(last_start) || tokens.len() <= MAX_SPAN_TOKENS {
        // Pin the tail window so trailing doctrine lines always anchor a full span.
        let tail_start = last_start.saturating_sub(MAX_SPAN_TOKENS.saturating_sub(1)).min(last_start);
        starts.push(tail_start);
    }
    let mut out = Vec::with_capacity(starts.len());
    let mut prev: Option<usize> = None;
    for start in starts {
        // Drop windows fully contained in the previous one (short-file tail pin).
        if prev.is_some_and(|p| start <= p) {
            continue;
        }
        prev = Some(start);
        let end = (start + MAX_SPAN_TOKENS).min(tokens.len());
        let slice = &tokens[start..end];
        out.push(AdmissionSpan {
            source_path: source_path.to_string(),
            text: slice.join(" "),
            tokens: slice.to_vec(),
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Pure decision function
// ---------------------------------------------------------------------------

/// Lowercase alphanumeric token stream over `text` (splitting on any
/// non-alphanumeric run). Deterministic; shared by index build and scoring so
/// both sides of the comparison see identical tokenization.
#[must_use]
pub fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch.to_ascii_lowercase());
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Token-set overlap of a candidate against one span: |intersection| divided by
/// the CANDIDATE token count ("how much of what you propose is already here").
/// Empty candidate scores 0.0 (nothing asserted => nothing to reject).
#[must_use]
pub fn lexical_overlap(candidate_tokens: &[String], span_tokens: &[String]) -> f32 {
    if candidate_tokens.is_empty() {
        return 0.0;
    }
    let mut span_set = std::collections::BTreeSet::new();
    span_set.extend(span_tokens.iter().cloned());
    let hits = candidate_tokens.iter().filter(|t| span_set.contains(*t)).count();
    hits as f32 / candidate_tokens.len() as f32
}

/// The admission pipeline entry point (stateless; the index is passed in so
/// staleness is managed explicitly by [`rebuild_if_stale`] BEFORE any batch).
#[derive(Debug, Clone, Copy, Default)]
pub struct AdmissionPipeline;

impl AdmissionPipeline {
    /// Pure AD-2 lexical leg: token-set overlap vs manifest spans under
    /// [`T_IMMUTABLE_LEXICAL`]. Deterministic end to end; UNIT A3 extends the
    /// same signature with the embedder leg + AD-3 store band behind it.
    #[must_use]
    pub fn check_candidate(
        &self,
        index: &StaticSpanIndex,
        candidate_text: &str,
    ) -> AdmissionCheckVerdict {
        let candidate_tokens = tokenize(candidate_text);
        let best = index
            .spans()
            .iter()
            .map(|span| lexical_overlap(&candidate_tokens, &span.tokens))
            .fold(0.0_f32, f32::max);
        if best >= T_IMMUTABLE_LEXICAL {
            AdmissionCheckVerdict::RejectImmutableSpan
        } else {
            AdmissionCheckVerdict::AdmitClean
        }
    }

    /// [`Self::check_candidate`] plus the GD-2 receipts journal-ready bundle.
    #[must_use]
    pub fn evaluate_candidate(&self, index: &StaticSpanIndex, candidate_text: &str) -> AdmissionEvaluation {
        let verdict = self.check_candidate(index, candidate_text);
        let receipts = AdmissionReceipts {
            candidate_hash: blake3::hash(candidate_text.as_bytes()).to_hex().to_string(),
            surface_leg: SURFACE_LEG_LEXICAL,
            threshold_used: T_IMMUTABLE_LEXICAL,
            verdict: verdict.as_receipt_str(),
        };
        AdmissionEvaluation { verdict, receipts }
    }
}
