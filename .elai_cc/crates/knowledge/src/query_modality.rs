//! Deterministic query-modality routing for retrieval channel selection.
//!
//! This module is intentionally pure: it uses the existing query-class
//! heuristic and query text only, without model or orchestrator dependencies.

use crate::query_class::{classify, has_code_symbol_signal, QueryClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalChannel {
    Bm25,
    Dense,
    Graph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryModality {
    /// Identifier / path / exact-symbol lookup
    ExactSymbol,
    /// Conceptual / "where is the logic" NL
    ConceptualLogic,
    /// Ambiguous or low-confidence
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelSubset {
    pub modality: QueryModality,
    /// Ordered unique channels to run. Widen-on-doubt => all three.
    pub channels: Vec<RetrievalChannel>,
    /// true when classifier is unsure and returned the full set
    pub widened_on_doubt: bool,
}

const FULL: [RetrievalChannel; 3] = [
    RetrievalChannel::Bm25,
    RetrievalChannel::Dense,
    RetrievalChannel::Graph,
];

fn interrogative_count(query: &str) -> usize {
    query
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|word| matches!(*word, "how" | "why" | "what" | "where" | "which"))
        .count()
}

/// Deterministically select the retrieval channel subset before fusion.
/// Uncertain inputs widen to the complete channel set rather than guessing.
pub fn classify_query_modality(query: &str) -> ChannelSubset {
    let trimmed = query.trim();
    let lower = trimmed.to_ascii_lowercase();
    let interrogatives = interrogative_count(&lower);
    let code_signal = has_code_symbol_signal(trimmed);

    if trimmed.is_empty() {
        return ambiguous();
    }

    let token_count = trimmed.split_whitespace().count();
    let exact_shape = (token_count == 1 || code_signal)
        && classify(trimmed) == QueryClass::CodeSymbol
        && interrogatives < 2;
    if exact_shape {
        return ChannelSubset {
            modality: QueryModality::ExactSymbol,
            channels: vec![RetrievalChannel::Bm25],
            widened_on_doubt: false,
        };
    }

    let phrase = lower.contains("where is")
        || lower.contains("logic that")
        || lower.contains("how does")
        || lower.contains("architecture")
        || lower.contains("design of");
    let nl_question = interrogatives > 0
        && token_count > 1
        && !code_signal
        && classify(trimmed) == QueryClass::NaturalLanguage;
    if phrase || nl_question {
        return ChannelSubset {
            modality: QueryModality::ConceptualLogic,
            channels: vec![RetrievalChannel::Dense, RetrievalChannel::Graph],
            widened_on_doubt: false,
        };
    }

    ambiguous()
}

fn ambiguous() -> ChannelSubset {
    ChannelSubset {
        modality: QueryModality::Ambiguous,
        channels: FULL.to_vec(),
        widened_on_doubt: true,
    }
}

pub fn channel_enabled(subset: &ChannelSubset, ch: RetrievalChannel) -> bool {
    subset.channels.contains(&ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_widens() {
        let result = classify_query_modality("  ");
        assert_eq!(result.modality, QueryModality::Ambiguous);
        assert_eq!(result.channels, FULL);
        assert!(result.widened_on_doubt);
    }

    #[test]
    fn exact_symbols_use_bm25_only() {
        for query in ["foo::bar", "should_retrieve"] {
            let result = classify_query_modality(query);
            assert_eq!(result.modality, QueryModality::ExactSymbol);
            assert_eq!(result.channels, vec![RetrievalChannel::Bm25]);
            assert!(!result.widened_on_doubt);
        }
    }

    #[test]
    fn conceptual_logic_uses_dense_and_graph() {
        let result = classify_query_modality("where is the logic that handles retries");
        assert_eq!(result.modality, QueryModality::ConceptualLogic);
        assert_eq!(result.channels, vec![RetrievalChannel::Dense, RetrievalChannel::Graph]);
        assert!(!result.widened_on_doubt);
    }

    #[test]
    fn ambiguous_input_widens() {
        let result = classify_query_modality("%%% ??? 91 / / unrelated");
        assert_eq!(result.modality, QueryModality::Ambiguous);
        assert!(result.widened_on_doubt);
        assert_eq!(result.channels, FULL);
    }

    #[test]
    fn channel_enabled_respects_subset() {
        let result = classify_query_modality("foo::bar");
        assert!(channel_enabled(&result, RetrievalChannel::Bm25));
        assert!(!channel_enabled(&result, RetrievalChannel::Dense));
    }

    #[test]
    fn classification_is_deterministic() {
        assert_eq!(classify_query_modality("where is the logic"), classify_query_modality("where is the logic"));
    }
}
