//! Tree-sitter-backed implementation of `AstEvaluator`.

use rustc_hash::FxHashMap;

use elai_rules::predicate::{AstError, AstEvaluator, AstMatch};
use tree_sitter::StreamingIterator;

/// Evaluates AST queries using tree-sitter grammars.
///
/// Holds a registry of language name -> `tree_sitter::Language`.
/// Supports the original 7 source languages (Rust, Python, TypeScript, Go,
/// Java, C#, Ruby) plus the web-domain grammars added in Phase B
/// (HTML, CSS, Svelte; Astro behind the `web-astro` feature flag).
pub struct TreeSitterAstEvaluator {
    languages: FxHashMap<String, tree_sitter::Language>,
}

impl TreeSitterAstEvaluator {
    pub fn new() -> Self {
        let mut languages = FxHashMap::default();
        languages.insert("rust".to_string(), tree_sitter_rust::LANGUAGE.into());
        languages.insert("python".to_string(), tree_sitter_python::LANGUAGE.into());
        languages.insert(
            "typescript".to_string(),
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        );
        languages.insert("go".to_string(), tree_sitter_go::LANGUAGE.into());
        languages.insert("java".to_string(), tree_sitter_java::LANGUAGE.into());
        languages.insert("c_sharp".to_string(), tree_sitter_c_sharp::LANGUAGE.into());
        languages.insert("ruby".to_string(), tree_sitter_ruby::LANGUAGE.into());
        // Phase B web grammars.
        languages.insert("html".to_string(), tree_sitter_html::LANGUAGE.into());
        languages.insert("css".to_string(), tree_sitter_css::LANGUAGE.into());
        languages.insert(
            "svelte".to_string(),
            tree_sitter_svelte_ng::LANGUAGE.into(),
        );
        #[cfg(feature = "web-astro")]
        languages.insert(
            "astro".to_string(),
            tree_sitter_astro_next::LANGUAGE.into(),
        );
        Self { languages }
    }

    fn get_language(&self, language: &str) -> Result<&tree_sitter::Language, AstError> {
        self.languages
            .get(language)
            .ok_or_else(|| AstError::UnsupportedLanguage(language.to_string()))
    }
}

impl AstEvaluator for TreeSitterAstEvaluator {
    fn evaluate_query(
        &self,
        language: &str,
        source: &str,
        query: &str,
    ) -> Result<Vec<AstMatch>, AstError> {
        let lang = self.get_language(language)?;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(lang)
            .map_err(|e| AstError::EvaluationFailed(format!("set_language: {e}")))?;

        let tree = parser
            .parse(source.as_bytes(), None)
            .ok_or_else(|| AstError::EvaluationFailed("parser returned no tree".to_string()))?;

        let ts_query = tree_sitter::Query::new(lang, query)
            .map_err(|e| AstError::InvalidQuery(e.to_string()))?;

        let mut cursor = tree_sitter::QueryCursor::new();
        let source_bytes = source.as_bytes();
        let mut matches = cursor.matches(&ts_query, tree.root_node(), source_bytes);

        let mut results = Vec::new();
        while let Some(m) = matches.next() {
            for capture in m.captures {
                let node = capture.node;
                let text = node
                    .utf8_text(source_bytes)
                    .map_err(|e| AstError::EvaluationFailed(format!("utf8: {e}")))?;
                results.push(AstMatch {
                    node_text: text.to_string(),
                    start_line: u32::try_from(node.start_position().row).unwrap_or(u32::MAX),
                    end_line: u32::try_from(node.end_position().row).unwrap_or(u32::MAX),
                });
            }
        }

        Ok(results)
    }

    fn validate_query(&self, language: &str, query: &str) -> Result<(), AstError> {
        let lang = self.get_language(language)?;
        tree_sitter::Query::new(lang, query)
            .map_err(|e| AstError::InvalidQuery(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_rust_function_query() {
        let eval = TreeSitterAstEvaluator::new();
        let source = "fn hello() { }\nfn world() { }";
        let query = "(function_item name: (identifier) @name)";
        let matches = eval.evaluate_query("rust", source, query).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].node_text, "hello");
        assert_eq!(matches[1].node_text, "world");
    }

    #[test]
    fn unsupported_language_returns_error() {
        let eval = TreeSitterAstEvaluator::new();
        let result = eval.evaluate_query("haskell", "main = putStrLn", "(identifier) @id");
        assert!(matches!(result, Err(AstError::UnsupportedLanguage(_))));
    }

    #[test]
    fn python_language_supported() {
        let eval = TreeSitterAstEvaluator::new();
        let source = "import os\nx = 1";
        let result = eval.evaluate_query("python", source, "(identifier) @id");
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_query_returns_error() {
        let eval = TreeSitterAstEvaluator::new();
        let result = eval.validate_query("rust", "(((invalid_query_syntax");
        assert!(matches!(result, Err(AstError::InvalidQuery(_))));
    }

    #[test]
    fn validate_good_query_succeeds() {
        let eval = TreeSitterAstEvaluator::new();
        let result = eval.validate_query("rust", "(function_item) @fn");
        assert!(result.is_ok());
    }
}
