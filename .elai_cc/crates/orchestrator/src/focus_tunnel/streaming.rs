//! Streaming validation — B-28 E-4.
//!
//! Library-tier streaming validator that inspects each emitted token from a
//! sidecar generation stream and decides whether to allow generation to
//! continue or to terminate early. Wired into [`crate::focus_tunnel::orchestrator::run_fim_step`]
//! and [`crate::focus_tunnel::orchestrator::run_diff_step`] in a dedicated
//! post-E-7 WIRING phase; E-4 only defines the library surface.
//!
//! Spec: `workflow/papers/focus-tunnel-design-interface.md`
//!   * L30-37 Streaming validation signal types.
//!   * L92-99 token-level vs chunk-level contracts.
//!   * L153-155 TokenPosition shape.
//!   * L290-340 streaming gate failure modes.
//!   * L309 false-positive avoidance — lazy patterns fire ONLY in code body.
//!
//! Two gate kinds:
//!   * Token gates run on every emitted token (cheap state-machine update +
//!     literal regex match).
//!   * Chunk gates run every CHUNK_TOKENS (default 50 ~ 200 chars per
//!     workspace `CHARS_PER_TOKEN` constant at
//!     `crates/core/src/micro_reflection.rs:36`). Tree-sitter partial parse +
//!     deprecated-import scan live here.
//!
//! All gates synchronous and allocation-light. The validator owns one
//! tree-sitter `Parser` per language (reused across chunks) and one
//! [`TokenPositionTracker`] (reused across tokens). Per locked decision #6
//! the trait is synchronous; no async, no I/O.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use elai_plugins::import_extractor::{
    check_deprecated_imports, extract_imports, DeprecatedImport, ImportLanguage,
};

use crate::ssb::SsbPayload;

// ---------------------------------------------------------------------------
// Constants — load-bearing thresholds
// ---------------------------------------------------------------------------

/// Default tokens per "chunk" inspection. ~200 chars at 4 chars/token (see
/// `crates/core/src/micro_reflection.rs:36`).
const DEFAULT_CHUNK_TOKENS: u32 = 50;

/// Default brace divergence threshold (per spec L309 / explorer recommendation):
/// trigger when |open - close| > THRESHOLD after at least MIN_TOKENS.
const DEFAULT_BRACE_DIVERGENCE_THRESHOLD: u32 = 8;

/// Default minimum tokens emitted before the brace-balance gate is allowed
/// to fire — avoids false positives on short outputs that legitimately open
/// many braces in a row (struct literals, match arms).
const DEFAULT_BRACE_MIN_TOKENS: u32 = 100;

/// Default factor: trigger OutputTooLong when token count exceeds expected *
/// factor (spec L98). Factor of 2 = "we asked for 256 tokens, generator
/// produced 513".
const DEFAULT_EXPECTED_FACTOR: u32 = 2;

/// Persistence count for tree-sitter ERROR-node debouncing. A single chunk
/// with an ERROR node does not trigger — the same flag must fire on TWO
/// consecutive chunks before the gate fires. Avoids transient debounce
/// false-positives mid-syntax.
const PARSE_ERROR_DEBOUNCE: u32 = 2;

// ---------------------------------------------------------------------------
// Language hint
// ---------------------------------------------------------------------------

/// Static language hint passed by the caller. Drives the lazy-pattern
/// language filter, the import extractor language enum, and the tree-sitter
/// language selection. Unknown / unsupported languages fall through to
/// [`LanguageHint::Generic`] which uses a conservative C-style state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageHint {
    Rust,
    Python,
    /// Catch-all: line comments `//`, block comments `/* */`, double-quote
    /// strings, escape via backslash. Covers JS/TS/Go/Java/C# adequately for
    /// the streaming gate.
    Generic,
}

impl LanguageHint {
    fn ssb_wire_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Generic => "generic",
        }
    }

    fn as_lazy_filter(self) -> Option<&'static str> {
        match self {
            Self::Rust => Some("rust"),
            Self::Python => Some("python"),
            Self::Generic => None,
        }
    }

    fn as_import_language(self) -> Option<ImportLanguage> {
        match self {
            Self::Rust => Some(ImportLanguage::Rust),
            Self::Python => Some(ImportLanguage::Python),
            Self::Generic => None,
        }
    }
}

// ---------------------------------------------------------------------------
// EarlyTermReason / ValidationDecision
// ---------------------------------------------------------------------------

/// Why generation was halted early. The library variant carries structured
/// detail; the SSB row that gets emitted by the WIRING phase translates this
/// into a flat-string wire form (`reason` + `reason_detail`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EarlyTermReason {
    /// Lazy-code regex matched in a code-body position.
    LazyPattern { pattern_id: &'static str },
    /// Brace count diverged past the configured threshold after sufficient
    /// tokens were emitted.
    UnbalancedBraces { final_depth: i32 },
    /// Token count exceeded expected * configured factor.
    OutputTooLong { expected: u32, actual: u32 },
    /// A new deprecated import surfaced mid-stream.
    DeprecatedImport { name: String, replacement: String },
    /// String literal opened but never closed within the inspected chunk
    /// (the chunk-level parser would flag this too).
    UnboundedStringLiteral { opener_token_index: u32 },
}

impl EarlyTermReason {
    /// SSB wire-form reason string (snake_case). Matches the convention used by
    /// `SelectionRule` in `mode.rs`.
    #[must_use]
    pub fn ssb_wire_str(&self) -> &'static str {
        match self {
            Self::LazyPattern { .. } => "lazy_pattern",
            Self::UnbalancedBraces { .. } => "unbalanced_braces",
            Self::OutputTooLong { .. } => "output_too_long",
            Self::DeprecatedImport { .. } => "deprecated_import",
            Self::UnboundedStringLiteral { .. } => "unbounded_string_literal",
        }
    }

    /// Optional detail string captured into `StreamingEarlyTerminatedPayload.reason_detail`.
    #[must_use]
    pub fn ssb_detail(&self) -> Option<String> {
        match self {
            Self::LazyPattern { pattern_id } => Some((*pattern_id).to_string()),
            Self::UnbalancedBraces { final_depth } => Some(format!("depth={final_depth}")),
            Self::OutputTooLong { expected, actual } => {
                Some(format!("expected={expected};actual={actual}"))
            }
            Self::DeprecatedImport { name, replacement } => {
                Some(format!("{name}->{replacement}"))
            }
            Self::UnboundedStringLiteral { opener_token_index } => {
                Some(format!("opener_token={opener_token_index}"))
            }
        }
    }
}

/// Decision returned by every gate and surfaced at the validator boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationDecision {
    /// Generation may continue.
    Continue,
    /// Generation must stop. Carries the structured reason and the token
    /// count at the moment of termination.
    EarlyTerm {
        reason: EarlyTermReason,
        token_count: u32,
    },
}

// ---------------------------------------------------------------------------
// TokenPositionTracker — byte-by-byte state machine
// ---------------------------------------------------------------------------

/// Lightweight per-byte state machine. Mirrors the spec L153-155 shape
/// extended with comment + string + bracket depth tracking so the lazy
/// gate only fires `in_code_body`.
#[derive(Debug, Clone)]
pub struct TokenPositionTracker {
    language: LanguageHint,
    in_block_comment: bool,
    in_line_comment: bool,
    /// Active string-literal opener byte. `None` outside any literal.
    in_string: Option<u8>,
    /// Python triple-quote tracking. When set, `in_string` carries the quote
    /// character and `triple` indicates whether to look for """ / '''.
    triple_string: bool,
    pending_backslash: bool,
    /// Cross-call pending byte for 2-char comment opener detection (`/` then
    /// `/` or `*`). Required because the validator is driven char-by-char
    /// from the sidecar; the comment opener can be split across two `feed`
    /// calls.
    pending_slash: bool,
    /// Same as `pending_slash` but for `*/` closer detection in block comments.
    pending_block_star: bool,
    brace_depth: i32,
    /// Max brace depth seen since stream start — used by the brace-balance
    /// gate to detect divergence relative to a high-water mark.
    max_brace_depth: i32,
    /// Count of `}` bytes that arrived while `brace_depth <= 0` (i.e.
    /// closer-than-opens). Tracked independently of `brace_depth` so the
    /// brace-balance gate can detect over-closes even if `brace_depth`
    /// goes negative — the i32 `unsigned_abs()` test catches symmetric
    /// divergence but this counter makes the over-close path explicit
    /// and resistant to future refactors that might change the field
    /// type. Fixes B-28 E-4 BLOCK 2.
    close_excess: u32,
    /// Accumulated bytes seen in the current logical line; reset on newline.
    /// Used by the python pass-body heuristic.
    line_buffer: String,
    /// Last full non-empty line — used by the python pass-body gate.
    prev_line: String,
}

impl TokenPositionTracker {
    fn new(language: LanguageHint) -> Self {
        Self {
            language,
            in_block_comment: false,
            in_line_comment: false,
            in_string: None,
            triple_string: false,
            pending_backslash: false,
            pending_slash: false,
            pending_block_star: false,
            brace_depth: 0,
            max_brace_depth: 0,
            close_excess: 0,
            line_buffer: String::new(),
            prev_line: String::new(),
        }
    }

    /// True when the next byte is inside source code (not comment, not string).
    pub fn in_code_body(&self) -> bool {
        !self.in_block_comment && !self.in_line_comment && self.in_string.is_none()
    }

    /// Advance the tracker over the byte slice. Returns the set of bytes
    /// that were classified `in_code_body` AT THE MOMENT they were
    /// processed — the caller can append these to a code-body-only buffer
    /// that the lazy-pattern regex scans against. Comments and string
    /// literals contribute zero bytes to that output (spec L309).
    ///
    /// State is carried across calls via `pending_slash` / `pending_block_star`
    /// so a `//` or `*/` split across two feeds is still recognised. This
    /// matters because the validator is driven char-by-char from the
    /// sidecar's SSE token stream.
    fn feed(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut code_body_out = Vec::with_capacity(bytes.len());
        for &b in bytes {
            let in_code_at_byte = self.in_code_body();
            self.feed_byte(b);
            // Push only bytes that were code-body BOTH at entry and exit of
            // feed_byte. The transition byte (e.g. the second `/` that
            // commits us to a comment) flips the state mid-byte; we elect
            // to exclude it from the code-body buffer so the lazy regex
            // never sees a `//` head fragment that could overlap a `//`
            // pattern (defensive; none of our patterns target `//` but we
            // keep the contract clean).
            if in_code_at_byte && self.in_code_body() {
                code_body_out.push(b);
            }
        }
        code_body_out
    }

    fn feed_byte(&mut self, b: u8) {
        // 1) Resolve pending block-comment closer (`*/` split across calls).
        if self.in_block_comment {
            if self.pending_block_star {
                self.pending_block_star = false;
                if b == b'/' {
                    self.in_block_comment = false;
                    return;
                }
                // The `*` was a non-closer; we lose its sequence but the
                // block remains open. Fall through to re-arm pending_star
                // if this byte is also `*`.
            }
            if b == b'*' {
                self.pending_block_star = true;
            }
            return;
        }

        // 2) Resolve pending line-comment opener — i.e. previous byte was
        // `/` and this byte may complete `//` or `/*`.
        if self.pending_slash {
            self.pending_slash = false;
            match b {
                b'/' => {
                    self.in_line_comment = true;
                    return;
                }
                b'*' => {
                    self.in_block_comment = true;
                    return;
                }
                _ => {
                    // The pending `/` was a real `/` operator. Process the
                    // current byte normally (falls through below).
                }
            }
        }

        // 3) Inside line comment: terminate on newline.
        if self.in_line_comment {
            if b == b'\n' {
                self.in_line_comment = false;
                self.flush_line();
            }
            return;
        }

        // 4) Inside string literal: escape + closer detection.
        if let Some(opener) = self.in_string {
            if self.pending_backslash {
                self.pending_backslash = false;
                return;
            }
            if b == b'\\' {
                self.pending_backslash = true;
                return;
            }
            if b == opener {
                // Triple-string requires the closer to be 3 consecutive
                // openers. We approximate by closing on a single closer —
                // this can produce a brief false code-body window inside a
                // triple-quoted string but is harmless for the lazy gate.
                self.in_string = None;
                self.triple_string = false;
                return;
            }
            // Unterminated python single-line string falls back to closed
            // on newline.
            if b == b'\n' && matches!(self.language, LanguageHint::Python) && !self.triple_string {
                self.in_string = None;
                self.flush_line();
            }
            return;
        }

        // 5) Code-body byte. Detect openers.
        match b {
            b'/' if matches!(self.language, LanguageHint::Rust | LanguageHint::Generic) => {
                self.pending_slash = true;
            }
            b'#' if matches!(self.language, LanguageHint::Python) => {
                self.in_line_comment = true;
            }
            b'"' => {
                self.in_string = Some(b'"');
            }
            b'\'' if !matches!(self.language, LanguageHint::Rust) => {
                self.in_string = Some(b'\'');
            }
            b'`' if matches!(self.language, LanguageHint::Generic) => {
                self.in_string = Some(b'`');
            }
            b'{' | b'(' | b'[' => {
                if b == b'{' {
                    self.brace_depth = self.brace_depth.saturating_add(1);
                    if self.brace_depth > self.max_brace_depth {
                        self.max_brace_depth = self.brace_depth;
                    }
                }
                self.append_line_byte(b);
            }
            b'}' | b')' | b']' => {
                if b == b'}' {
                    // Track over-closes BEFORE decrementing so the counter
                    // increments only when there is no matching opener at
                    // the current depth. Defensive against future type
                    // changes — see `close_excess` doc comment.
                    if self.brace_depth <= 0 {
                        self.close_excess = self.close_excess.saturating_add(1);
                    }
                    self.brace_depth = self.brace_depth.saturating_sub(1);
                }
                self.append_line_byte(b);
            }
            b'\n' => {
                self.flush_line();
            }
            _ => {
                self.append_line_byte(b);
            }
        }
    }

    fn append_line_byte(&mut self, b: u8) {
        // ASCII bytes are appended directly. Multi-byte UTF-8 fragments are
        // skipped — the line buffer is only used for prefix string matching.
        if b.is_ascii() {
            self.line_buffer.push(b as char);
        }
    }

    fn flush_line(&mut self) {
        let trimmed = self.line_buffer.trim();
        if !trimmed.is_empty() {
            self.prev_line = trimmed.to_string();
        }
        self.line_buffer.clear();
    }
}

// ---------------------------------------------------------------------------
// LazyPattern set (lifted from plugins::tiered_analysis at L216-284)
// ---------------------------------------------------------------------------

/// One lazy-pattern entry: ID, the regex object, and the language scope.
struct LazyPattern {
    id: &'static str,
    pattern: &'static LazyLock<regex::Regex>,
    /// `None` = universal; otherwise restricted to the named language.
    language: Option<&'static str>,
}

// CLAUDE-md exception: static-init-with-literal-regex. The `expect(...)` here is
// on a const literal pattern compiled at first use; per CLAUDE.md
// "const/static init with provably valid literals" exception, this is exempt
// from the production-path unwrap ban.
//
// PATTERN SCOPE — spec L309 false-positive avoidance:
// Only patterns that target CODE-BODY tokens (the literal macro / keyword) are
// kept here. Comment-content patterns (// TODO, # TODO, etc.) are DROPPED from
// the streaming validator because by definition they only appear inside a
// comment — and the streaming gate must ignore comment content (test contract
// "// TODO inside comment must NOT trigger"). Comment-content lazy detection
// remains the responsibility of the post-write `plugins::tiered_analysis::T1Hook`
// pass at file-write time; the streaming gate is a STRICT subset focused on
// "tokens that betray a lazy completion mid-stream".
//
// The pattern set is duplicated from `plugins::tiered_analysis::LAZY_PATTERNS`
// (L216-284) — the regex strings are kept verbatim so future updates can
// cross-reference both sites. This duplication is the cost of honoring the
// read-only lock on `plugins/src/tiered_analysis.rs` (locked decision #2).
static RE_RUST_TODO: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\btodo!\(").expect("literal regex"));
static RE_RUST_UNIMPL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\bunimplemented!\(").expect("literal regex"));
static RE_PY_PASS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?m)\bpass\s*$").expect("literal regex"));
static RE_PY_NOT_IMPL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\braise NotImplementedError\b").expect("literal regex"));

static LAZY_PATTERNS: LazyLock<Vec<LazyPattern>> = LazyLock::new(|| {
    vec![
        // Rust.
        LazyPattern { id: "rust_todo_macro", pattern: &RE_RUST_TODO, language: Some("rust") },
        LazyPattern {
            id: "rust_unimplemented",
            pattern: &RE_RUST_UNIMPL,
            language: Some("rust"),
        },
        // Python.
        LazyPattern { id: "py_bare_pass", pattern: &RE_PY_PASS, language: Some("python") },
        LazyPattern {
            id: "py_not_implemented",
            pattern: &RE_PY_NOT_IMPL,
            language: Some("python"),
        },
    ]
});

// ---------------------------------------------------------------------------
// Validator config
// ---------------------------------------------------------------------------

/// Construction parameters for [`StreamingValidator`].
#[derive(Debug, Clone)]
pub struct StreamingValidatorConfig {
    pub language: LanguageHint,
    /// `step.max_gen_tokens` from the originating BabyStepSpec — used by
    /// the OutputTooLong gate to derive the trigger threshold.
    pub max_gen_tokens: u32,
    /// `baby_step_{id}` formatted at construction time so the SSB row this
    /// validator triggers can be linked to its step.
    pub step_id: String,
    /// `"fim"` / `"diff"` / `"structured"` — emitted with the SSB row.
    pub mode: String,
    /// OutputTooLong threshold = `max_gen_tokens * expected_factor`.
    pub expected_factor: u32,
    /// Brace divergence threshold.
    pub brace_divergence_threshold: u32,
    /// Minimum tokens emitted before the brace gate is allowed to fire.
    pub brace_min_tokens_before_trigger: u32,
    /// Deprecated-import list (filtered by ecosystem at chunk time). Owned
    /// by the caller; cloned in for the validator's lifetime.
    pub deprecated_imports: Vec<DeprecatedImport>,
}

impl Default for StreamingValidatorConfig {
    fn default() -> Self {
        Self {
            language: LanguageHint::Generic,
            max_gen_tokens: 1_024,
            step_id: String::new(),
            mode: "structured".into(),
            expected_factor: DEFAULT_EXPECTED_FACTOR,
            brace_divergence_threshold: DEFAULT_BRACE_DIVERGENCE_THRESHOLD,
            brace_min_tokens_before_trigger: DEFAULT_BRACE_MIN_TOKENS,
            deprecated_imports: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Validation outcome (returned by on_complete)
// ---------------------------------------------------------------------------

/// Final outcome of a streamed generation. `Ok` means no gate fired through
/// `on_complete`; the variant carries the final token count for telemetry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationOutcome {
    Ok { total_tokens: u32 },
    Terminated { reason: EarlyTermReason, token_count: u32 },
}

// ---------------------------------------------------------------------------
// StreamingValidator
// ---------------------------------------------------------------------------

/// Synchronous streaming validator. Library-tier — no I/O, no async. Holds
/// one tree-sitter parser per supported language plus a token-position
/// tracker that walks every byte fed via [`Self::on_token`].
///
/// Lifecycle:
///   1. `new(config)`.
///   2. Call `on_token(text)` for every emitted token (or text fragment).
///   3. Call `on_chunk(partial_output)` every ~50 tokens (~200 chars).
///   4. Call `on_complete()` after the stream finishes naturally.
///
/// `on_token` and `on_chunk` return [`ValidationDecision`]. The caller is
/// expected to halt generation on `EarlyTerm`.
pub struct StreamingValidator {
    config: StreamingValidatorConfig,
    tracker: TokenPositionTracker,
    /// Cumulative buffer of every emitted byte. Used by the chunk-level
    /// tree-sitter parser + the import extractor.
    accum: String,
    /// Cumulative buffer of ONLY the bytes classified as code-body at the
    /// moment they were consumed. Spec L309: lazy patterns must NEVER fire
    /// inside comments or string literals. This buffer is what the lazy
    /// regex set scans against.
    code_body_buffer: String,
    token_count: u32,
    /// Set once a terminating gate has fired; subsequent calls return
    /// `Continue` rather than re-triggering.
    terminated: bool,
    /// Captured reason at the moment `terminate()` fired. `None` while the
    /// stream is still live. `on_complete()` reads this to construct the
    /// `Err(ValidationOutcome::Terminated{..})` branch with the real reason
    /// rather than a hardcoded placeholder. Fixes B-28 E-4 BLOCK 1.
    termination_reason: Option<EarlyTermReason>,
    /// Number of consecutive chunks where tree-sitter reported an ERROR node.
    consecutive_parse_errors: u32,
    /// Set of already-reported deprecated import names so we never trigger
    /// twice on the same import line.
    reported_imports: HashMap<String, ()>,
    /// One-time language→parser construction. `None` if the chunk gate has
    /// not yet had to instantiate the parser, OR if the language has no
    /// tree-sitter grammar in the orchestrator crate.
    parser: Option<tree_sitter::Parser>,
}

impl StreamingValidator {
    /// Construct a new validator. `parser` is left `None` and built lazily
    /// inside `on_chunk` only when a chunk actually arrives — keeps the
    /// per-token path allocation-free.
    #[must_use]
    pub fn new(config: StreamingValidatorConfig) -> Self {
        let tracker = TokenPositionTracker::new(config.language);
        Self {
            config,
            tracker,
            accum: String::new(),
            code_body_buffer: String::new(),
            token_count: 0,
            terminated: false,
            termination_reason: None,
            consecutive_parse_errors: 0,
            reported_imports: HashMap::new(),
            parser: None,
        }
    }

    /// Default chunk size, in tokens — exposed so callers can drive the
    /// chunk gate at the same cadence the validator expects.
    #[must_use]
    pub fn chunk_tokens(&self) -> u32 {
        DEFAULT_CHUNK_TOKENS
    }

    /// Process one token. Returns the decision; once terminated, returns
    /// `Continue` so the caller's stream loop can drain cleanly.
    pub fn on_token(&mut self, text: &str) -> ValidationDecision {
        if self.terminated {
            return ValidationDecision::Continue;
        }
        // Advance the position tracker over every byte (UTF-8 safe — only
        // ASCII control bytes drive transitions; multi-byte chars fall
        // through the default branch). Capture only the code-body bytes for
        // the lazy regex gate (spec L309 false-positive avoidance).
        let code_body_bytes = self.tracker.feed(text.as_bytes());
        if let Ok(extra) = std::str::from_utf8(&code_body_bytes) {
            self.code_body_buffer.push_str(extra);
        }
        self.accum.push_str(text);
        self.token_count = self.token_count.saturating_add(1);

        // OutputTooLong gate — token-level for fast trip.
        let cap = self
            .config
            .max_gen_tokens
            .saturating_mul(self.config.expected_factor);
        if cap > 0 && self.token_count > cap {
            return self.terminate(EarlyTermReason::OutputTooLong {
                expected: self.config.max_gen_tokens,
                actual: self.token_count,
            });
        }

        // LazyPatternGate — token-level. Scans only the code-body-filtered
        // buffer so comment/string content can never trigger (spec L309).
        if let Some(pattern_id) = self.scan_lazy_patterns() {
            return self.terminate(EarlyTermReason::LazyPattern { pattern_id });
        }

        // Brace-balance gate — fires when EITHER the absolute
        // depth-divergence OR the explicit over-close counter exceeds the
        // configured threshold. The `unsigned_abs` test catches
        // open-only and (via i32 negative depth) close-only divergence;
        // the explicit `close_excess` test makes the over-close path
        // unambiguous and resistant to type changes. Fixes B-28 E-4
        // BLOCK 2.
        if self.token_count >= self.config.brace_min_tokens_before_trigger
            && (self.tracker.brace_depth.unsigned_abs()
                > self.config.brace_divergence_threshold
                || self.tracker.close_excess > self.config.brace_divergence_threshold)
        {
            return self.terminate(EarlyTermReason::UnbalancedBraces {
                final_depth: self.tracker.brace_depth,
            });
        }

        ValidationDecision::Continue
    }

    /// Process one chunk-level inspection. Caller is responsible for cadence
    /// (typically every `chunk_tokens()` tokens). `partial_output` is the
    /// current cumulative generation; if empty, the validator uses its own
    /// accumulator. The chunk gate runs:
    ///   * tree-sitter partial parse → ERROR node detection (debounce 2);
    ///   * deprecated-import scan on use/import statements.
    pub fn on_chunk(&mut self, partial_output: &str) -> ValidationDecision {
        if self.terminated {
            return ValidationDecision::Continue;
        }
        let source = if partial_output.is_empty() {
            self.accum.as_str()
        } else {
            partial_output
        };
        // Lazy-init parser per configured language.
        if self.parser.is_none() {
            self.parser = build_parser(self.config.language);
        }
        // Tree-sitter ERROR-node check with debouncing.
        let mut has_error = false;
        if let Some(parser) = self.parser.as_mut() {
            if let Some(tree) = parser.parse(source, None) {
                has_error = tree.root_node().has_error();
            }
        }
        if has_error {
            self.consecutive_parse_errors =
                self.consecutive_parse_errors.saturating_add(1);
            if self.consecutive_parse_errors >= PARSE_ERROR_DEBOUNCE {
                return self.terminate(EarlyTermReason::UnbalancedBraces {
                    final_depth: self.tracker.brace_depth,
                });
            }
        } else {
            self.consecutive_parse_errors = 0;
        }

        // Deprecated-import scan — runs against the cumulative source so a
        // mid-stream `use blake2::*` is caught even though the validator
        // already passed `use` itself.
        if !self.config.deprecated_imports.is_empty() {
            if let Some(language) = self.config.language.as_import_language() {
                if let Ok(imports) = extract_imports(source, language) {
                    let violations = check_deprecated_imports(
                        &imports,
                        &self.config.deprecated_imports,
                        language,
                    );
                    for v in violations {
                        if self.reported_imports.contains_key(&v.import_name) {
                            continue;
                        }
                        self.reported_imports.insert(v.import_name.clone(), ());
                        // BLOCK severity terminates; WARN does not. WARN
                        // surfaces in telemetry via the SSB row would
                        // remain Continue here — but the library-tier
                        // validator escalates everything to EarlyTerm and
                        // lets the WIRING phase decide.
                        return self.terminate(EarlyTermReason::DeprecatedImport {
                            name: v.import_name,
                            replacement: v.replacement_name,
                        });
                    }
                }
            }
        }
        ValidationDecision::Continue
    }

    /// Finalize the validator. Returns the final outcome. Calling
    /// `on_token` / `on_chunk` after this is a no-op (returns `Continue`).
    ///
    /// The `Err(Terminated{..})` branch carries the structured reason that
    /// was captured at the moment `terminate()` fired (whichever gate
    /// tripped — LazyPattern, UnbalancedBraces, DeprecatedImport, etc.).
    /// If `terminated` is `true` but `termination_reason` is somehow
    /// `None` (a defensive branch — every code path that flips
    /// `terminated` also sets the reason), this falls back to
    /// `OutputTooLong` as a structural sentinel. Fixes B-28 E-4 BLOCK 1.
    pub fn on_complete(&self) -> Result<ValidationOutcome, ValidationOutcome> {
        if self.terminated {
            let reason = self.termination_reason.clone().unwrap_or_else(|| {
                EarlyTermReason::OutputTooLong {
                    expected: self.config.max_gen_tokens,
                    actual: self.token_count,
                }
            });
            Err(ValidationOutcome::Terminated {
                reason,
                token_count: self.token_count,
            })
        } else {
            Ok(ValidationOutcome::Ok {
                total_tokens: self.token_count,
            })
        }
    }

    /// Read-only access to the language-tagged step metadata so the WIRING
    /// phase can build SSB rows without re-deriving them.
    #[must_use]
    pub fn step_id(&self) -> &str {
        &self.config.step_id
    }

    /// Read-only access to the mode string.
    #[must_use]
    pub fn mode(&self) -> &str {
        &self.config.mode
    }

    /// Wire-form language hint string.
    #[must_use]
    pub fn language_wire_str(&self) -> &'static str {
        self.config.language.ssb_wire_str()
    }

    /// Current cumulative token count.
    #[must_use]
    pub fn token_count(&self) -> u32 {
        self.token_count
    }

    fn scan_lazy_patterns(&self) -> Option<&'static str> {
        let lang_filter = self.config.language.as_lazy_filter();
        let source = &self.code_body_buffer;
        for pat in LAZY_PATTERNS.iter() {
            if let Some(needle_lang) = pat.language {
                let matches_lang = match lang_filter {
                    Some(active) => active == needle_lang,
                    None => false,
                };
                if !matches_lang {
                    continue;
                }
            }
            if pat.pattern.is_match(source) {
                return Some(pat.id);
            }
        }
        None
    }

    fn terminate(&mut self, reason: EarlyTermReason) -> ValidationDecision {
        self.terminated = true;
        // Capture the structured reason so on_complete() can surface the
        // real cause to the WIRING phase rather than a hardcoded variant.
        // Fixes B-28 E-4 BLOCK 1 (on_complete was always returning
        // EarlyTermReason::OutputTooLong regardless of actual cause).
        self.termination_reason = Some(reason.clone());
        ValidationDecision::EarlyTerm {
            reason,
            token_count: self.token_count,
        }
    }
}

fn build_parser(language: LanguageHint) -> Option<tree_sitter::Parser> {
    let mut parser = tree_sitter::Parser::new();
    let lang: tree_sitter::Language = match language {
        LanguageHint::Rust => tree_sitter_rust::LANGUAGE.into(),
        LanguageHint::Python => tree_sitter_python::LANGUAGE.into(),
        LanguageHint::Generic => return None,
    };
    parser.set_language(&lang).ok()?;
    Some(parser)
}

// ---------------------------------------------------------------------------
// Reader-side aggregation — parallels mode_distribution_for_payloads
// ---------------------------------------------------------------------------

/// Aggregated statistics derived from a slice of SSB payloads. Mirrors the
/// reader-side aggregation pattern from
/// [`crate::focus_tunnel::mode::mode_distribution_for_payloads`] so the
/// dashboard can render "streaming early term · 7 events" without re-running
/// SQL. The downstream caller computes `tokens_saved` ratios by joining
/// `total_tokens_saved` against each step's originating `max_gen_tokens`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamingEarlyTermStats {
    /// Number of `StreamingEarlyTerminated` rows observed.
    pub total_terminations: u32,
    /// Reason wire-string → count.
    pub reason_breakdown: HashMap<String, u32>,
    /// Sum of `token_count_at_term` across rows — a coarse "tokens we
    /// stopped emitting" proxy.
    pub total_tokens_saved: u64,
}

/// Aggregate a slice of [`SsbPayload`] borrows into a
/// [`StreamingEarlyTermStats`]. Mirrors
/// [`crate::focus_tunnel::mode::mode_distribution_for_payloads`].
#[must_use]
pub fn streaming_early_termination_stats_for_payloads(
    payloads: &[&SsbPayload],
) -> StreamingEarlyTermStats {
    let mut total = 0u32;
    let mut by_reason: HashMap<String, u32> = HashMap::new();
    let mut tokens_saved: u64 = 0;
    for payload in payloads {
        if let SsbPayload::StreamingEarlyTerminated(p) = payload {
            total = total.saturating_add(1);
            *by_reason.entry(p.reason.clone()).or_insert(0) += 1;
            tokens_saved = tokens_saved.saturating_add(u64::from(p.token_count_at_term));
        }
    }
    StreamingEarlyTermStats {
        total_terminations: total,
        reason_breakdown: by_reason,
        total_tokens_saved: tokens_saved,
    }
}

// ---------------------------------------------------------------------------
// Serde-friendly EarlyTermReason wire-form (consumed by WIRING phase)
// ---------------------------------------------------------------------------

/// Compact wire-form record used by the WIRING phase to build the
/// `StreamingEarlyTerminatedPayload` SSB row. Library-tier validator does
/// not emit the SSB row itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarlyTermWireForm {
    pub reason: String,
    pub reason_detail: Option<String>,
    pub token_count_at_term: u32,
    pub language_hint: String,
}

impl EarlyTermWireForm {
    /// Project a [`ValidationDecision`] into the wire form. Returns `None`
    /// for `Continue`.
    #[must_use]
    pub fn from_decision(
        decision: &ValidationDecision,
        language_hint: &'static str,
    ) -> Option<Self> {
        match decision {
            ValidationDecision::Continue => None,
            ValidationDecision::EarlyTerm { reason, token_count } => Some(Self {
                reason: reason.ssb_wire_str().to_string(),
                reason_detail: reason.ssb_detail(),
                token_count_at_term: *token_count,
                language_hint: language_hint.to_string(),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Inline unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_in_code_body_by_default() {
        let t = TokenPositionTracker::new(LanguageHint::Rust);
        assert!(t.in_code_body());
    }

    #[test]
    fn tracker_detects_line_comment() {
        let mut t = TokenPositionTracker::new(LanguageHint::Rust);
        t.feed(b"fn foo() {\n// hi\n");
        assert!(t.in_code_body(), "after newline, comment should be closed");
    }

    #[test]
    fn tracker_string_literal_disables_code_body() {
        let mut t = TokenPositionTracker::new(LanguageHint::Rust);
        t.feed(b"let s = \"open");
        assert!(!t.in_code_body(), "inside open string literal");
    }

    #[test]
    fn lazy_gate_fires_on_rust_todo() {
        let mut v = StreamingValidator::new(StreamingValidatorConfig {
            language: LanguageHint::Rust,
            max_gen_tokens: 1_000,
            step_id: "s".into(),
            mode: "fim".into(),
            expected_factor: 200,
            brace_divergence_threshold: 64,
            brace_min_tokens_before_trigger: 10_000,
            deprecated_imports: Vec::new(),
        });
        let src = "fn foo() {\n    todo!()\n}\n";
        let mut d = ValidationDecision::Continue;
        for ch in src.chars() {
            let mut buf = [0u8; 4];
            d = v.on_token(ch.encode_utf8(&mut buf));
            if !matches!(d, ValidationDecision::Continue) {
                break;
            }
        }
        assert!(
            matches!(d, ValidationDecision::EarlyTerm { reason: EarlyTermReason::LazyPattern { .. }, .. }),
            "expected LazyPattern, got {d:?}",
        );
    }

    #[test]
    fn lazy_gate_skips_comment() {
        let mut v = StreamingValidator::new(StreamingValidatorConfig {
            language: LanguageHint::Rust,
            max_gen_tokens: 1_000,
            step_id: "s".into(),
            mode: "fim".into(),
            expected_factor: 200,
            brace_divergence_threshold: 64,
            brace_min_tokens_before_trigger: 10_000,
            deprecated_imports: Vec::new(),
        });
        let src = "// todo!() inside a comment must not trigger\n";
        let mut d = ValidationDecision::Continue;
        for ch in src.chars() {
            let mut buf = [0u8; 4];
            d = v.on_token(ch.encode_utf8(&mut buf));
            if !matches!(d, ValidationDecision::Continue) {
                break;
            }
        }
        assert!(matches!(d, ValidationDecision::Continue), "comment must not trigger");
    }

    #[test]
    fn brace_depth_increments_per_open_brace() {
        let mut t = TokenPositionTracker::new(LanguageHint::Rust);
        for _ in 0..50 {
            t.feed(b"{");
        }
        assert_eq!(t.brace_depth, 50, "brace_depth should be 50 after 50 open braces");
    }

    #[test]
    fn brace_gate_triggers_after_min_tokens() {
        let mut v = StreamingValidator::new(StreamingValidatorConfig {
            language: LanguageHint::Rust,
            max_gen_tokens: 1_000_000,
            step_id: "s".into(),
            mode: "fim".into(),
            expected_factor: 200,
            brace_divergence_threshold: 8,
            brace_min_tokens_before_trigger: 100,
            deprecated_imports: Vec::new(),
        });
        let mut d = ValidationDecision::Continue;
        for _ in 0..200 {
            d = v.on_token("{");
            if !matches!(d, ValidationDecision::Continue) {
                break;
            }
        }
        assert!(
            matches!(d, ValidationDecision::EarlyTerm { reason: EarlyTermReason::UnbalancedBraces { .. }, .. }),
            "brace gate must fire after 100+ open braces: {d:?}",
        );
    }

    #[test]
    fn reader_aggregator_counts_payloads() {
        use crate::ssb::StreamingEarlyTerminatedPayload;
        let p1 = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
            step_id: "s1".into(),
            mode: "fim".into(),
            reason: "lazy_pattern".into(),
            reason_detail: None,
            token_count_at_term: 12,
            total_tokens_received: 12,
            language_hint: "rust".into(),
        });
        let p2 = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
            step_id: "s2".into(),
            mode: "diff".into(),
            reason: "output_too_long".into(),
            reason_detail: None,
            token_count_at_term: 50,
            total_tokens_received: 50,
            language_hint: "python".into(),
        });
        let payloads: Vec<&SsbPayload> = vec![&p1, &p2];
        let stats = streaming_early_termination_stats_for_payloads(&payloads);
        assert_eq!(stats.total_terminations, 2);
        assert_eq!(stats.total_tokens_saved, 62);
    }
}
