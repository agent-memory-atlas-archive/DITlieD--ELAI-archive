// Template-based multi-query expansion (Phase 3K.2).
//
// Strategies:
//   1. Keyword extraction: strip question words, keep nouns/verbs
//   2. Code symbol expansion: CamelCase/snake_case variants of concepts
//   3. Synonym expansion: common programming synonyms
//
// Queries < MIN_WORDS_FOR_EXPANSION words skip expansion entirely.

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Queries with fewer words than this skip expansion.
const MIN_WORDS_FOR_EXPANSION: usize = 5;

/// Maximum number of expansion variants (excluding the original query).
const MAX_EXPANSIONS: usize = 3;

/// Question words to strip during keyword extraction.
const QUESTION_WORDS: &[&str] = &[
    "how", "what", "where", "when", "why", "which", "who", "does", "do",
    "is", "are", "can", "could", "would", "should", "the", "a", "an",
    "in", "on", "at", "to", "for", "of", "with", "from", "by", "about",
    "this", "that", "it", "its", "my", "our", "their",
];

/// Programming synonym groups. Each inner slice maps interchangeable terms.
const SYNONYM_GROUPS: &[&[&str]] = &[
    &["error", "failure", "exception", "fault", "panic"],
    &["function", "method", "fn", "def", "func"],
    &["struct", "class", "type", "record"],
    &["test", "spec", "check", "verify", "assert"],
    &["config", "configuration", "settings", "options", "preferences"],
    &["async", "asynchronous", "concurrent", "parallel"],
    &["module", "crate", "package", "mod"],
    &["import", "use", "require", "include"],
    &["variable", "var", "field", "attribute", "property"],
    &["search", "query", "find", "lookup", "retrieve"],
    &["create", "new", "init", "initialize", "construct"],
    &["delete", "remove", "drop", "destroy"],
    &["update", "modify", "change", "mutate", "set"],
    &["return", "result", "output", "response"],
    &["handle", "handler", "callback", "listener"],
    &["parse", "deserialize", "decode", "read"],
    &["serialize", "encode", "write", "format"],
    &["database", "db", "store", "storage", "repository"],
    &["index", "indexer", "indexing"],
    &["pipeline", "chain", "workflow", "stage"],
];

// ---------------------------------------------------------------------------
// ExpandedQuery
// ---------------------------------------------------------------------------

/// A single query variant produced by expansion.
#[derive(Debug, Clone)]
pub struct ExpandedQuery {
    pub text: String,
    pub strategy: ExpansionStrategy,
}

/// Which expansion strategy produced this variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpansionStrategy {
    /// Original query, passed through unchanged.
    Original,
    /// Stop-word-stripped keyword extraction.
    Keywords,
    /// CamelCase / snake_case code symbol guessing.
    CodeSymbol,
    /// Synonym substitution of programming terms.
    Synonym,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Expand a query into 1 + N variants (original + up to MAX_EXPANSIONS).
///
/// Returns a vec where index 0 is always the original query.
/// Short queries (< MIN_WORDS_FOR_EXPANSION words) return only the original.
pub fn expand_query(query: &str) -> Vec<ExpandedQuery> {
    let words: Vec<&str> = query.split_whitespace().collect();

    let mut result = vec![ExpandedQuery {
        text: query.to_string(),
        strategy: ExpansionStrategy::Original,
    }];

    if words.len() < MIN_WORDS_FOR_EXPANSION {
        return result;
    }

    // Strategy 1: keyword extraction
    let keywords = extract_keywords(&words);
    if !keywords.is_empty() && keywords != query {
        result.push(ExpandedQuery {
            text: keywords,
            strategy: ExpansionStrategy::Keywords,
        });
    }

    // Strategy 2: code symbol expansion
    if result.len() <= MAX_EXPANSIONS {
        let symbols = code_symbol_variants(&words);
        if !symbols.is_empty() && !result.iter().any(|e| e.text == symbols) {
            result.push(ExpandedQuery {
                text: symbols,
                strategy: ExpansionStrategy::CodeSymbol,
            });
        }
    }

    // Strategy 3: synonym expansion
    if result.len() <= MAX_EXPANSIONS {
        let synonyms = synonym_expand(&words);
        if !synonyms.is_empty() && !result.iter().any(|e| e.text == synonyms) {
            result.push(ExpandedQuery {
                text: synonyms,
                strategy: ExpansionStrategy::Synonym,
            });
        }
    }

    // Clamp: original + at most MAX_EXPANSIONS
    result.truncate(1 + MAX_EXPANSIONS);
    result
}

// ---------------------------------------------------------------------------
// Strategy implementations
// ---------------------------------------------------------------------------

/// Strip question/stop words, return remaining terms joined by space.
fn extract_keywords(words: &[&str]) -> String {
    let stop: HashSet<&str> = QUESTION_WORDS.iter().copied().collect();
    let kept: Vec<&str> = words
        .iter()
        .filter(|w| {
            let lower = w.to_ascii_lowercase();
            !stop.contains(lower.as_str()) && lower.len() > 1
        })
        .copied()
        .collect();
    kept.join(" ")
}

/// Convert concept words to plausible CamelCase and snake_case identifiers.
///
/// Example: ["FSM", "handle", "transitions"] -> "FsmState handle_transitions transition_to"
fn code_symbol_variants(words: &[&str]) -> String {
    let stop: HashSet<&str> = QUESTION_WORDS.iter().copied().collect();
    let meaningful: Vec<&str> = words
        .iter()
        .filter(|w| {
            let lower = w.to_ascii_lowercase();
            !stop.contains(lower.as_str()) && lower.len() > 1
        })
        .copied()
        .collect();

    if meaningful.is_empty() {
        return String::new();
    }

    let mut parts: Vec<String> = Vec::new();

    // CamelCase: join all meaningful words with first-letter capitalized
    let camel: String = meaningful
        .iter()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => {
                    let mut s = c.to_uppercase().to_string();
                    s.push_str(&chars.as_str().to_lowercase());
                    s
                }
                None => String::new(),
            }
        })
        .collect();
    parts.push(camel);

    // snake_case: join all meaningful words with underscores, lowercased
    let snake: String = meaningful
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_");
    parts.push(snake);

    // Individual words as potential symbol names (lowercased)
    for w in &meaningful {
        let lower = w.to_lowercase();
        if !parts.contains(&lower) {
            parts.push(lower);
        }
    }

    parts.join(" ")
}

/// Replace one term with its synonym group alternatives (first match only).
fn synonym_expand(words: &[&str]) -> String {
    let mut result_words: Vec<String> = words.iter().map(|w| w.to_string()).collect();
    let mut added: Vec<String> = Vec::new();

    for (idx, word) in words.iter().enumerate() {
        let lower = word.to_ascii_lowercase();
        for group in SYNONYM_GROUPS {
            if group.contains(&lower.as_str()) {
                // Replace the word with the first synonym that differs
                for &syn in group.iter() {
                    if syn != lower {
                        result_words[idx] = syn.to_string();
                        // Also append one more synonym to broaden recall
                        if let Some(&extra) = group.iter().find(|&&s| s != lower && s != syn) {
                            added.push(extra.to_string());
                        }
                        break;
                    }
                }
                // Only expand first matching term to avoid noise
                result_words.extend(added);
                return result_words.join(" ");
            }
        }
    }

    // No synonyms matched — return empty to signal "no expansion"
    String::new()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_query_skips_expansion() {
        let result = expand_query("find main");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].strategy, ExpansionStrategy::Original);
    }

    #[test]
    fn four_word_query_skips_expansion() {
        let result = expand_query("how does FSM work");
        assert_eq!(result.len(), 1, "4-word query should skip expansion");
    }

    #[test]
    fn five_word_query_gets_expanded() {
        let result = expand_query("how does the FSM handle transitions");
        assert!(result.len() > 1, ">=5 word query should expand");
        assert_eq!(result[0].strategy, ExpansionStrategy::Original);
        assert_eq!(result[0].text, "how does the FSM handle transitions");
    }

    #[test]
    fn keyword_extraction_strips_question_words() {
        let words: Vec<&str> = "how does the FSM handle transitions".split_whitespace().collect();
        let keywords = extract_keywords(&words);
        assert!(!keywords.contains("how"));
        assert!(!keywords.contains("does"));
        assert!(!keywords.contains("the"));
        assert!(keywords.contains("FSM"));
        assert!(keywords.contains("handle"));
        assert!(keywords.contains("transitions"));
    }

    #[test]
    fn code_symbol_variants_produces_camel_and_snake() {
        let words: Vec<&str> = "how does the FSM handle transitions".split_whitespace().collect();
        let symbols = code_symbol_variants(&words);
        // Should contain CamelCase variant
        assert!(symbols.contains("FsmHandleTransitions"), "got: {symbols}");
        // Should contain snake_case variant
        assert!(symbols.contains("fsm_handle_transitions"), "got: {symbols}");
    }

    #[test]
    fn synonym_expand_replaces_term() {
        let words: Vec<&str> = "how does the error handler work in pipeline".split_whitespace().collect();
        let expanded = synonym_expand(&words);
        assert!(!expanded.is_empty());
        // "error" should be replaced with "failure" (first different synonym)
        assert!(expanded.contains("failure"), "got: {expanded}");
    }

    #[test]
    fn synonym_expand_no_match_returns_empty() {
        let words: Vec<&str> = "hello world foo bar baz".split_whitespace().collect();
        let expanded = synonym_expand(&words);
        assert!(expanded.is_empty());
    }

    #[test]
    fn max_expansions_capped() {
        // A query that triggers all 3 strategies
        let result = expand_query("how does the error handler work in the pipeline");
        // original + at most MAX_EXPANSIONS
        assert!(result.len() <= 1 + MAX_EXPANSIONS, "got {} variants", result.len());
    }

    #[test]
    fn no_duplicate_expansions() {
        let result = expand_query("how does the FSM handle state transitions here");
        let texts: Vec<&str> = result.iter().map(|e| e.text.as_str()).collect();
        let unique: HashSet<&str> = texts.iter().copied().collect();
        assert_eq!(texts.len(), unique.len(), "duplicate expansion detected");
    }

    #[test]
    fn original_always_first() {
        let query = "how does the FSM handle transitions in the pipeline";
        let result = expand_query(query);
        assert_eq!(result[0].text, query);
        assert_eq!(result[0].strategy, ExpansionStrategy::Original);
    }

    #[test]
    fn structural_check_constants() {
        assert_eq!(MIN_WORDS_FOR_EXPANSION, 5);
        assert_eq!(MAX_EXPANSIONS, 3);
        assert!(QUESTION_WORDS.contains(&"how"));
        assert!(!SYNONYM_GROUPS.is_empty());
    }
}
