//! E-17 SP2 task 5 — Layer A query-class classifier (heuristic, no model).
//!
//! Distinguishes three classes:
//! - [`QueryClass::CodeSymbol`] — identifier-shaped: `::`, `.`, snake_case,
//!   CamelCase, parens, file extensions, or a single non-space token.
//! - [`QueryClass::NaturalLanguage`] — sentence-shaped: contains >= 4 of the
//!   English stop-words / interrogatives the gate watches for, OR the
//!   query is structurally English-like (multiple words, no identifier
//!   signal).
//! - [`QueryClass::Mixed`] — neither bucket dominates. Defaults to
//!   HyDE-OFF (conservative) when paired with the `ELAI_HYDE_NL_ONLY`
//!   gate; downstream callers can override.
//!
//! Honest framing: heuristic classifier; tests in
//! `crates/knowledge/tests/query_class_contract.rs` use a fixture set
//! whose labels match the heuristic itself (not an aspirational ML F1).
//! Real F1 measurement is deferred to SP6 task 10.

/// Three-way query class for the HyDE-gate (E-17 SP2 task 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryClass {
    CodeSymbol,
    NaturalLanguage,
    Mixed,
}

impl QueryClass {
    /// Stable string identifier used in SSB payloads.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CodeSymbol => "code_symbol",
            Self::NaturalLanguage => "natural_language",
            Self::Mixed => "mixed",
        }
    }
}

/// E-17 SP2: stop-word count threshold for the NaturalLanguage bucket.
/// At least 4 hits across the english-stopword set marks the query as NL.
const NL_STOPWORD_HITS_MIN: usize = 4;

/// English stop-words + interrogatives that signal NL phrasing. Pure
/// lowercase ASCII match against whitespace-split tokens (no morphology).
const NL_STOPWORDS: &[&str] = &[
    "how", "why", "what", "when", "where", "which", "who",
    "does", "do", "is", "are", "was", "were", "the", "a", "an",
    "in", "on", "of", "to", "for", "and", "or", "with", "by",
    "explain", "describe", "show", "find", "tell",
];

/// E-17 SP2: classify a query string into a [`QueryClass`].
///
/// Decision tree (deterministic, order-sensitive):
///   1. Single token + no whitespace -> CodeSymbol
///   2. Identifier-shape signal (`::`, `->`, fn/class/def, snake_case,
///      CamelCase, `.ext`, parens) -> CodeSymbol unless NL stopwords also fire
///   3. >= NL_STOPWORD_HITS_MIN stopwords -> NaturalLanguage
///   4. Otherwise -> Mixed (default HyDE-off when combined with NL-only gate)
pub fn classify(query: &str) -> QueryClass {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return QueryClass::Mixed;
    }

    let token_count = trimmed.split_whitespace().count();
    let has_code_signal = has_code_symbol_signal(trimmed);
    let nl_hits = natural_language_stopword_hits(trimmed);

    // Rule 1: bare identifier with no whitespace is unambiguously code.
    if token_count == 1 && !trimmed.contains(char::is_whitespace) {
        return QueryClass::CodeSymbol;
    }

    // Rule 2: identifier-shape dominates UNLESS the query is also dense
    // with natural-language stop-words (handles "what does foo::bar do").
    if has_code_signal && nl_hits < NL_STOPWORD_HITS_MIN {
        return QueryClass::CodeSymbol;
    }

    // Rule 3: enough stop-words -> NL.
    if nl_hits >= NL_STOPWORD_HITS_MIN {
        return QueryClass::NaturalLanguage;
    }

    // Rule 4: ambiguous.
    QueryClass::Mixed
}

/// Returns `true` when the query carries an identifier-shape signal:
/// path-operator `::`, arrow `->`, an explicit `fn` / `class` / `def` /
/// `impl` keyword, a `snake_case` or `CamelCase` token, a `.ext` filename
/// signal, or a parenthesized call site.
pub fn has_code_symbol_signal(query: &str) -> bool {
    if query.contains("::") || query.contains("->") || query.contains("=>") {
        return true;
    }
    if query.contains('(') && query.contains(')') {
        return true;
    }
    for keyword in ["fn ", "class ", "def ", "impl ", "struct ", "trait ", "enum "] {
        if query.contains(keyword) {
            return true;
        }
    }
    for ext in [".rs", ".py", ".ts", ".tsx", ".js", ".svelte", ".toml", ".onnx"] {
        if query.contains(ext) {
            return true;
        }
    }
    for token in query.split_whitespace() {
        if is_identifier_shape(token) {
            return true;
        }
    }
    false
}

/// A token is identifier-shaped when it (a) contains `_` between alnum
/// chars (snake_case), or (b) contains an inner uppercase letter
/// (CamelCase / mixedCase), or (c) contains digits adjacent to letters,
/// or (d) starts with `_` or uppercase and is all-alnum.
fn is_identifier_shape(token: &str) -> bool {
    let bytes = token.as_bytes();
    if bytes.is_empty() || bytes.len() < 2 {
        return false;
    }
    let mut has_underscore = false;
    let mut has_inner_upper = false;
    let mut has_inner_digit_letter = false;
    let mut all_alnum_or_underscore = true;
    let mut prev_is_letter = false;
    let mut prev_is_digit = false;
    for (i, &b) in bytes.iter().enumerate() {
        let is_upper = b.is_ascii_uppercase();
        let is_lower = b.is_ascii_lowercase();
        let is_digit = b.is_ascii_digit();
        let is_letter = is_upper || is_lower;
        if b == b'_' {
            has_underscore = true;
        } else if !is_letter && !is_digit {
            all_alnum_or_underscore = false;
        }
        if i > 0 {
            if is_upper {
                has_inner_upper = true;
            }
            if (is_digit && prev_is_letter) || (is_letter && prev_is_digit) {
                has_inner_digit_letter = true;
            }
        }
        prev_is_letter = is_letter;
        prev_is_digit = is_digit;
    }
    if !all_alnum_or_underscore {
        return false;
    }
    has_underscore || has_inner_upper || has_inner_digit_letter
}

/// Count NL stop-word hits across whitespace-split tokens (case-insensitive).
fn natural_language_stopword_hits(query: &str) -> usize {
    let lower = query.to_ascii_lowercase();
    let mut hits: usize = 0;
    for token in lower.split(|c: char| !c.is_alphanumeric()) {
        if NL_STOPWORDS.iter().any(|&sw| sw == token) {
            hits = hits.saturating_add(1);
        }
    }
    hits
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_identifier_is_code() {
        assert_eq!(classify("rrf_rerank"), QueryClass::CodeSymbol);
        assert_eq!(classify("RuntimeAdapter"), QueryClass::CodeSymbol);
        assert_eq!(classify("foo"), QueryClass::CodeSymbol);
    }

    #[test]
    fn path_operator_is_code() {
        assert_eq!(classify("BTreeMap::remove"), QueryClass::CodeSymbol);
        assert_eq!(classify("crate::pipeline::QueryPipeline"), QueryClass::CodeSymbol);
    }

    #[test]
    fn file_extension_is_code() {
        assert_eq!(classify("models/onnx/cross-encoder-l12.onnx"), QueryClass::CodeSymbol);
        assert_eq!(classify("crates/knowledge/src/hyde.rs"), QueryClass::CodeSymbol);
    }

    #[test]
    fn natural_language_question_is_nl() {
        assert_eq!(
            classify("how does the degradation cascade work in the knowledge layer"),
            QueryClass::NaturalLanguage,
        );
        assert_eq!(
            classify("explain why the auditor fails on missing tests"),
            QueryClass::NaturalLanguage,
        );
    }

    #[test]
    fn mixed_short_phrase_is_mixed() {
        assert_eq!(classify("knowledge layer"), QueryClass::Mixed);
        assert_eq!(classify("anti spiral detector"), QueryClass::Mixed);
    }

    #[test]
    fn mixed_question_about_symbol_falls_through_to_nl_when_enough_stopwords() {
        // "what does the RerankerConfig do" has 5 NL stopwords:
        // what / does / the / do (plus implicit). CodeSymbol is signalled by
        // CamelCase but NL stopwords >= 4 overrides per the decision tree.
        let q = "what does the RerankerConfig default do in production";
        // 4+ stopwords -> NL wins.
        assert_eq!(classify(q), QueryClass::NaturalLanguage);
    }

    #[test]
    fn identifier_shape_camel_case() {
        assert!(is_identifier_shape("RuntimeAdapter"));
        assert!(is_identifier_shape("camelCase"));
        assert!(is_identifier_shape("foo_bar"));
        assert!(is_identifier_shape("var_42"));
        assert!(!is_identifier_shape("plain"));
        assert!(!is_identifier_shape("a"));
    }

    #[test]
    fn empty_query_is_mixed() {
        assert_eq!(classify(""), QueryClass::Mixed);
        assert_eq!(classify("   "), QueryClass::Mixed);
    }
}
