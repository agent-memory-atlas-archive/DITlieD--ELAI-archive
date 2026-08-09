//! C-32 — MiniCritic: cheap per-fill verdict between skeleton blanks.
//!
//! Runs three checks in order on a candidate body string:
//!
//!   (a) T1 lazy-pattern subset (7 patterns borrowed from
//!       [`elai_plugins::tiered_analysis::mini_critic_patterns`]); first
//!       match -> `BLOCK` with reason `"lazy_pattern:<id>"`.
//!   (b) Body line count `< 3` -> `BLOCK` with reason `"too_short:<N>_lines"`.
//!       Single-expression bodies (no `'\n'` in `body.trim()`) are exempt
//!       so one-liners like `Ok(x)` are allowed.
//!   (c) Tree-sitter fragment parse (when enabled). Body is wrapped in a
//!       minimal per-language probe shell and parsed. ERROR node -> `BLOCK`
//!       reason `"parse_error:<kind>"`. Unregistered language -> `WARN`
//!       reason `"ts_unavailable"`.
//!
//! First `BLOCK` short-circuits — subsequent checks are skipped. The verdict
//! is emitted to SSB as a `mini_critic_verdict` row regardless of result.
//!
//! Spec anchors (C-32 plan):
//!   - §Task 2 paragraph (a), (b), (c)
//!   - WIRING_CONTRACT consumes_symbols: T1 LazyLock<Regex> statics, no copy.

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use elai_plugins::tiered_analysis::{mini_critic_patterns, MiniCriticPattern};

use crate::skeleton_generator::BlankId;
use crate::ssb::{MiniCriticVerdictPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Verdict + result types
// ---------------------------------------------------------------------------

/// MiniCritic verdict tri-state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MiniCriticVerdict {
    Pass,
    Warn,
    Block,
}

impl MiniCriticVerdict {
    /// Lowercase wire-form string used in SSB payloads and contract assertions.
    pub fn as_str(self) -> &'static str {
        match self {
            MiniCriticVerdict::Pass => "pass",
            MiniCriticVerdict::Warn => "warn",
            MiniCriticVerdict::Block => "block",
        }
    }
}

impl fmt::Display for MiniCriticVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verdict row produced by [`MiniCritic::check`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniCriticResult {
    pub blank_id: BlankId,
    pub verdict: MiniCriticVerdict,
    /// Short reason tag consumed by the dashboard ReasoningDrawer. Format:
    /// `"lazy_pattern:<id>"`, `"too_short:<N>_lines"`, `"parse_error:<kind>"`,
    /// `"ts_unavailable"`, or `"ok"` for a PASS verdict.
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Language tag — small enum so we can route tree-sitter grammars deterministically.
// ---------------------------------------------------------------------------

/// Tree-sitter language tag that MiniCritic understands.
///
/// Mirrors the per-language patterns and the deterministic_auditor.rs
/// fragment-parse coverage. Unknown languages skip check (c) with `WARN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MiniCriticLanguage {
    Rust,
    Python,
    /// Any other language. Used so callers can carry a tag without having
    /// to invent a separate enum at every call site. Check (a) restricts
    /// language-scoped patterns to those declared as `None` (universal).
    Other,
}

impl MiniCriticLanguage {
    /// Lowercase tag string used in SSB payloads (so the dashboard renders
    /// the verdict row with a stable label).
    pub fn as_str(self) -> &'static str {
        match self {
            MiniCriticLanguage::Rust => "rust",
            MiniCriticLanguage::Python => "python",
            MiniCriticLanguage::Other => "other",
        }
    }

    /// Match against the per-pattern `language` filter from [`MiniCriticPattern`].
    fn matches_pattern(self, pat: &MiniCriticPattern) -> bool {
        match pat.language {
            None => true,
            Some("rust") => matches!(self, MiniCriticLanguage::Rust),
            Some("python") => matches!(self, MiniCriticLanguage::Python),
            // Unknown language entries are never selected; mini_critic_patterns()
            // only ships `None`, `"rust"`, `"python"` today.
            Some(_) => false,
        }
    }
}

// ---------------------------------------------------------------------------
// MiniCritic
// ---------------------------------------------------------------------------

/// Minimum body line count required for a PASS verdict on check (b).
///
/// Plan §Task 2 (b): "body.lines().count() < 3 -> BLOCK". Named so the
/// contract test asserts against the constant instead of a magic literal.
pub const MIN_BODY_LINES: usize = 3;

/// Provisional seed for the syntactic god-object callee threshold. This seed is
/// PROVISIONAL and is replaced by the I-38 QDELTA `--quality` arm measurement
/// in a later slice.
pub const GOD_OBJECT_MIN_DISTINCT_CALLEES: usize = 8;

/// Provisional seed for the syntactic god-object function-item threshold. This
/// seed is PROVISIONAL and is replaced by the I-38 QDELTA `--quality` arm
/// measurement in a later slice.
pub const GOD_OBJECT_MIN_FUNCTION_ITEMS: usize = 3;

/// Provisional seed for the syntactic god-object body-line threshold. This seed
/// is PROVISIONAL and is replaced by the I-38 QDELTA `--quality` arm measurement
/// in a later slice.
pub const GOD_OBJECT_MIN_BODY_LINES: usize = 40;

/// MiniCritic — runs lazy-pattern + line-count + optional tree-sitter checks
/// on a candidate fill body. The struct is cheap; the per-call cost is one
/// regex sweep plus an optional tree-sitter parse.
pub struct MiniCritic {
    /// Whether check (c) is enabled. False on platforms where tree-sitter
    /// is intentionally turned off (we still register the grammars at
    /// construction time so toggling on does not require a rebuild).
    pub tree_sitter_enabled: bool,
}

impl Default for MiniCritic {
    fn default() -> Self {
        Self { tree_sitter_enabled: true }
    }
}

impl MiniCritic {
    /// Construct with tree-sitter check (c) enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct with tree-sitter check (c) disabled. Useful in pure-Rust
    /// unit tests that do not want to link the tree-sitter grammar tables.
    pub fn without_tree_sitter() -> Self {
        Self { tree_sitter_enabled: false }
    }

    /// Run the three checks in order. First `BLOCK` short-circuits.
    pub fn check(
        &self,
        blank_id: BlankId,
        body: &str,
        lang: MiniCriticLanguage,
    ) -> MiniCriticResult {
        // (a) lazy-pattern subset.
        if let Some(id) = self.lazy_pattern_match(body, lang) {
            return MiniCriticResult {
                blank_id,
                verdict: MiniCriticVerdict::Block,
                reason: format!("lazy_pattern:{id}"),
            };
        }

        // (b) line count, with single-expression exemption.
        let n_lines = body.lines().count();
        let trimmed = body.trim();
        let is_single_expression = !trimmed.contains('\n');
        if n_lines < MIN_BODY_LINES && !is_single_expression {
            return MiniCriticResult {
                blank_id,
                verdict: MiniCriticVerdict::Block,
                reason: format!("too_short:{n_lines}_lines"),
            };
        }

        // (c) tree-sitter fragment parse (only when enabled).
        if self.tree_sitter_enabled {
            match self.tree_sitter_check(body, lang) {
                TsOutcome::Pass => {}
                TsOutcome::Warn(reason) => {
                    return MiniCriticResult {
                        blank_id,
                        verdict: MiniCriticVerdict::Warn,
                        reason,
                    };
                }
                TsOutcome::Block(reason) => {
                    return MiniCriticResult {
                        blank_id,
                        verdict: MiniCriticVerdict::Block,
                        reason,
                    };
                }
            }
        }

        // (d) Syntactic god-object proxy. WARN is intentionally the ceiling.
        let (callees, function_items, lines) = self.god_object_signals(body);
        if callees >= GOD_OBJECT_MIN_DISTINCT_CALLEES
            && (function_items >= GOD_OBJECT_MIN_FUNCTION_ITEMS
                || lines >= GOD_OBJECT_MIN_BODY_LINES)
        {
            return MiniCriticResult {
                blank_id,
                verdict: MiniCriticVerdict::Warn,
                reason: format!(
                    "metric_god_object:callees={callees}:function_items={function_items}:lines={lines}"
                ),
            };
        }

        MiniCriticResult {
            blank_id,
            verdict: MiniCriticVerdict::Pass,
            reason: "ok".to_string(),
        }
    }

    /// Returns the matching pattern id when a lazy pattern fires on `body`.
    fn lazy_pattern_match(&self, body: &str, lang: MiniCriticLanguage) -> Option<&'static str> {
        for pat in mini_critic_patterns() {
            if !lang.matches_pattern(pat) {
                continue;
            }
            if pat.regex.is_match(body) {
                return Some(pat.id);
            }
        }
        None
    }

    /// Return `(distinct callees, function items, lines)` for one fill body.
    /// This is a SYNTACTIC proxy over one fill body, NOT a call graph. It is a
    /// generation-time SELF-CORRECTION signal that makes StructuralRetry
    /// re-roll, while the authoritative SOLID verdict stays auditor-owned in
    /// the frozen `.elai_cc/tools/solid_proxy_check.py`. The fill is not in
    /// `elai.map` yet, which is why the audit-time checker is a separate layer.
    fn god_object_signals(&self, body: &str) -> (usize, usize, usize) {
        const KEYWORDS: &[&str] = &[
            "if", "else", "for", "while", "match", "loop", "fn", "def", "return",
            "in", "let", "use", "pub", "struct", "impl", "unsafe",
        ];
        let bytes = body.as_bytes();
        let mut tokens: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
                let start = i;
                i += 1;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    i += 1;
                }
                tokens.push((start, i));
            } else {
                i += 1;
            }
        }

        let mut distinct_callees = std::collections::HashSet::new();
        let mut function_items = 0;
        for (index, &(start, end)) in tokens.iter().enumerate() {
            let token = &body[start..end];
            if token == "fn" || token == "def" {
                function_items += 1;
            }
            if KEYWORDS.contains(&token) {
                continue;
            }
            let mut next = end;
            while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                next += 1;
            }
            if next >= bytes.len() || bytes[next] != b'(' {
                continue;
            }
            let preceded_by_definition = index > 0
                && matches!(&body[tokens[index - 1].0..tokens[index - 1].1], "fn" | "def");
            if !preceded_by_definition {
                distinct_callees.insert(token.to_owned());
            }
        }
        (distinct_callees.len(), function_items, body.lines().count())
    }

    /// Run check (c). Wraps the body in a per-language probe shell, parses
    /// it with tree-sitter, and returns the outcome.
    fn tree_sitter_check(&self, body: &str, lang: MiniCriticLanguage) -> TsOutcome {
        match lang {
            MiniCriticLanguage::Rust => ts_parse_with_grammar(
                &format!("fn __probe() {{\n{body}\n}}\n"),
                tree_sitter_rust::LANGUAGE.into(),
            ),
            MiniCriticLanguage::Python => {
                let indented = body
                    .lines()
                    .map(|line| format!("    {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                ts_parse_with_grammar(
                    &format!("def __probe():\n{indented}\n"),
                    tree_sitter_python::LANGUAGE.into(),
                )
            }
            MiniCriticLanguage::Other => TsOutcome::Warn("ts_unavailable".to_string()),
        }
    }

    /// Emit a `mini_critic_verdict` SSB row. Errors are silently swallowed so
    /// the fill loop is never broken by an SSB backpressure event.
    pub async fn emit_verdict_ssb(
        ssb: &Arc<Ssb>,
        agent_id: &str,
        superstep: i64,
        result: &MiniCriticResult,
        language: MiniCriticLanguage,
    ) {
        let payload = SsbPayload::MiniCriticVerdict(MiniCriticVerdictPayload {
            blank_id: result.blank_id.0,
            verdict: result.verdict.as_str().to_string(),
            reason: result.reason.clone(),
            language: language.as_str().to_string(),
        });
        let _ = ssb
            .write_payload(agent_id, superstep, payload, Vec::new())
            .await;
    }
}

// ---------------------------------------------------------------------------
// Tree-sitter helpers
// ---------------------------------------------------------------------------

enum TsOutcome {
    Pass,
    Warn(String),
    Block(String),
}

fn ts_parse_with_grammar(source: &str, grammar: tree_sitter::Language) -> TsOutcome {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&grammar).is_err() {
        return TsOutcome::Warn("ts_unavailable".to_string());
    }
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return TsOutcome::Warn("ts_unavailable".to_string()),
    };
    let root = tree.root_node();
    if root.has_error() {
        // Walk to surface the first error node kind.
        let mut cursor = root.walk();
        let mut visited = false;
        let mut error_kind = "ERROR".to_string();
        loop {
            let node = cursor.node();
            if node.is_error() {
                error_kind = node.kind().to_string();
                break;
            }
            if cursor.goto_first_child() {
                continue;
            }
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    visited = true;
                    break;
                }
            }
            if visited {
                break;
            }
        }
        return TsOutcome::Block(format!("parse_error:{error_kind}"));
    }
    TsOutcome::Pass
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn b(n: u8) -> BlankId {
        BlankId(n)
    }

    #[test]
    fn block_on_todo_slash_pattern() {
        let critic = MiniCritic::without_tree_sitter();
        let r = critic.check(b(1), "// TODO implement", MiniCriticLanguage::Rust);
        assert_eq!(r.verdict, MiniCriticVerdict::Block);
        assert!(r.reason.starts_with("lazy_pattern:"), "reason={}", r.reason);
    }

    #[test]
    fn block_on_short_body() {
        let critic = MiniCritic::without_tree_sitter();
        let r = critic.check(b(1), "x\ny", MiniCriticLanguage::Rust);
        assert_eq!(r.verdict, MiniCriticVerdict::Block);
        assert_eq!(r.reason, "too_short:2_lines");
    }

    #[test]
    fn single_expression_exempt_from_line_count() {
        let critic = MiniCritic::without_tree_sitter();
        let r = critic.check(b(1), "Ok(())", MiniCriticLanguage::Rust);
        assert_eq!(r.verdict, MiniCriticVerdict::Pass);
    }

    #[test]
    fn pass_on_clean_multiline_body() {
        let critic = MiniCritic::without_tree_sitter();
        let body = "let x = 1;\nlet y = 2;\nx + y";
        let r = critic.check(b(1), body, MiniCriticLanguage::Rust);
        assert_eq!(r.verdict, MiniCriticVerdict::Pass);
    }

    #[test]
    fn warn_on_other_language_with_ts_enabled() {
        let critic = MiniCritic::new();
        let body = "alpha\nbeta\ngamma";
        let r = critic.check(b(1), body, MiniCriticLanguage::Other);
        // Lazy + length both pass; tree-sitter check returns Warn for Other.
        assert_eq!(r.verdict, MiniCriticVerdict::Warn);
        assert_eq!(r.reason, "ts_unavailable");
    }
}
