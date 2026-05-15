//! Per-grammar failure-isolated LanguageRegistry.
//!
//! # Design rationale
//!
//! Tree-sitter grammar crates may panic during language initialization if:
//!   - The native `.so` / `.dll` is missing (dlopen failure manifests as panic, not Err).
//!   - The grammar ABI version does not match the host tree-sitter runtime.
//!   - `tree_sitter::Parser::set_language` returns a `LanguageError` (non-matching ABI).
//!
//! We wrap EACH grammar's probe in `std::panic::catch_unwind` so a single broken
//! grammar does not abort registration of the others. This mirrors the pattern in
//! SocratiCode `ensureDynamicLanguages` (code-graph.ts:497-515) adapted for Rust.
//!
//! # Determinism
//!
//! `BTreeMap` is used for all internal storage so `status()` returns sorted,
//! deterministic output across runs and platforms. This satisfies the project-wide
//! HASHMAP_BAN rule for serialized maps.
//!
//! # Integration note (Pass A.2)
//!
//! This module is self-contained and only uses public types from the crate root.
//! It CANNOT be `pub mod`-included from `lib.rs` until session s-22200 (B-29)
//! releases the `lib.rs` lock. The `pub mod language_registry;` line and a
//! corresponding `pub use dkg::language_registry::*;` re-export must be added
//! by the Pass A.2 integration pass.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use crate::LanguageGrammar;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of language entries to display in summary contexts.
/// Does NOT truncate the internal registry — only advisory for UI consumers.
pub const MAX_DISPLAY_LANGUAGES: usize = 8;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when registering a grammar into the registry.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum GrammarLoadError {
    /// The grammar's tree-sitter ABI version is incompatible with the host runtime.
    /// The string carries the language id.
    #[error("tree-sitter ABI mismatch for grammar '{0}'")]
    TreeSitterAbiMismatch(String),

    /// `tree_sitter::Parser::set_language` panicked (native library load failure,
    /// dlopen error, or grammar-internal initialization panic).
    #[error("grammar parser init panicked for '{0}'")]
    ParserInitPanicked(String),

    /// Factory function (in `try_register`) panicked during grammar construction.
    #[error("grammar factory panicked for '{0}'")]
    FactoryPanicked(String),

    /// Any other non-ABI error returned by `set_language`.
    #[error("grammar load failed for '{0}': {1}")]
    Other(String, String),
}

// ---------------------------------------------------------------------------
// Status struct (serializable, sent to dashboard)
// ---------------------------------------------------------------------------

/// Snapshot of the registry state, suitable for JSON serialization.
///
/// Fields are sorted (BTreeMap source) so dashboard consumers receive
/// alphabetically ordered lists across every platform and run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageRegistryStatus {
    /// Language IDs successfully loaded, in alphabetical order.
    pub loaded: Vec<String>,
    /// Language IDs that failed to load, paired with the error message,
    /// in alphabetical order of language ID.
    pub failed: Vec<(String, String)>,
}

impl Default for LanguageRegistryStatus {
    fn default() -> Self {
        Self { loaded: Vec::new(), failed: Vec::new() }
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Registry of [`LanguageGrammar`] implementations with per-grammar
/// `catch_unwind` isolation.
///
/// Build the registry at DKG engine startup by calling [`LanguageRegistry::register`]
/// or [`LanguageRegistry::try_register`] for each grammar. Failed grammars are
/// recorded in the `failed` map and do not prevent successful grammars from loading.
///
/// Query with [`LanguageRegistry::get`]; iterate with [`LanguageRegistry::iter`];
/// export status with [`LanguageRegistry::status`].
pub struct LanguageRegistry {
    loaded: BTreeMap<String, Box<dyn LanguageGrammar>>,
    failed: BTreeMap<String, String>,
}

impl LanguageRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            loaded: BTreeMap::new(),
            failed: BTreeMap::new(),
        }
    }

    /// Register a pre-constructed grammar with per-grammar `catch_unwind` isolation.
    ///
    /// Runs a probe: `grammar.tree_sitter_language()` is called, then
    /// `tree_sitter::Parser::new().set_language(&lang)` is called inside
    /// `catch_unwind(AssertUnwindSafe(...))`. On panic or set_language error the
    /// grammar is recorded in `failed`; on success it is inserted into `loaded`.
    ///
    /// Returns `Ok(())` on success; `Err(GrammarLoadError)` (and records the failure)
    /// on any error.
    ///
    /// NOTE: `catch_unwind` does NOT intercept SIGABRT or SIGSEGV raised by native
    /// grammar C libraries (e.g. dlopen segfault, corrupted heap in grammar init).
    /// Signal-level failures will still abort the process. This is acceptable for
    /// grammar load errors in production; callers must not rely on `catch_unwind`
    /// for signal-safety guarantees beyond Rust panic isolation.
    pub fn register(
        &mut self,
        grammar: Box<dyn LanguageGrammar>,
    ) -> Result<(), GrammarLoadError> {
        let lang_id = grammar.language_id().to_owned();

        // Wrap the entire probe in catch_unwind. Parser is not UnwindSafe;
        // AssertUnwindSafe is the correct adapter here (the catch is local and
        // we never observe the Parser's state after an unwind).
        let probe_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let lang = grammar.tree_sitter_language();
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang)
        }));

        match probe_result {
            Ok(Ok(_)) => {
                self.loaded.insert(lang_id, grammar);
                Ok(())
            }
            Ok(Err(lang_err)) => {
                let msg = lang_err.to_string();
                warn!(grammar = %lang_id, error = %msg, "grammar ABI mismatch");
                self.failed.insert(lang_id.clone(), msg.clone());
                Err(GrammarLoadError::TreeSitterAbiMismatch(lang_id))
            }
            Err(_panic_payload) => {
                let msg = "parser init panicked".to_owned();
                warn!(grammar = %lang_id, "grammar registration panicked during set_language probe");
                self.failed.insert(lang_id.clone(), msg);
                Err(GrammarLoadError::ParserInitPanicked(lang_id))
            }
        }
    }

    /// Register a grammar via a factory closure, with `catch_unwind` wrapping
    /// both the factory call AND the set_language probe.
    ///
    /// Use this when the grammar crate's `language()` / `LANGUAGE.into()` call
    /// may itself panic (e.g. ABI-incompatible native library loaded at startup).
    ///
    /// The factory `F` must be `UnwindSafe`. Plain function pointers (e.g.
    /// `|| Box::new(MyGrammar)`) are `UnwindSafe` by default.
    pub fn try_register<F>(
        &mut self,
        lang_id: &str,
        factory: F,
    ) -> Result<(), GrammarLoadError>
    where
        F: FnOnce() -> Box<dyn LanguageGrammar> + std::panic::UnwindSafe,
    {
        // Phase 1: catch panics in the factory itself.
        let grammar = match std::panic::catch_unwind(factory) {
            Ok(g) => g,
            Err(_) => {
                let msg = "grammar factory panicked".to_owned();
                warn!(grammar = %lang_id, "grammar factory panicked");
                self.failed.insert(lang_id.to_owned(), msg);
                return Err(GrammarLoadError::FactoryPanicked(lang_id.to_owned()));
            }
        };

        // Verify the factory-returned grammar identifies itself correctly.
        debug_assert_eq!(
            grammar.language_id(),
            lang_id,
            "try_register lang_id mismatch: caller passed '{}' but grammar.language_id() returns '{}'",
            lang_id,
            grammar.language_id()
        );

        // Phase 2: probe with set_language.
        self.register(grammar)
    }

    /// Look up a loaded grammar by language id.
    pub fn get(&self, lang_id: &str) -> Option<&dyn LanguageGrammar> {
        self.loaded.get(lang_id).map(|b| b.as_ref())
    }

    /// Return a snapshot of the current registry state.
    ///
    /// Both `loaded` and `failed` lists are in alphabetical order (BTreeMap
    /// iteration order). This is the canonical deterministic representation
    /// used for SSB `LanguageGrammarStatus` payloads and dashboard rendering.
    pub fn status(&self) -> LanguageRegistryStatus {
        LanguageRegistryStatus {
            loaded: self.loaded.keys().cloned().collect(),
            failed: self
                .failed
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    /// Number of successfully loaded grammars.
    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    /// Number of grammars that failed to load.
    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }

    /// Iterate over all loaded grammars in alphabetical order of language id.
    ///
    /// Used by `DkgEngine` to dispatch parsing across all registered languages.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &dyn LanguageGrammar)> {
        self.loaded.iter().map(|(k, v)| (k.as_str(), v.as_ref()))
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// In-module unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DkgNode, LanguageGrammar};

    // ---- minimal stub grammars ----

    struct GoodGrammar {
        id: &'static str,
    }

    impl LanguageGrammar for GoodGrammar {
        fn language_id(&self) -> &'static str {
            self.id
        }

        fn tree_sitter_language(&self) -> tree_sitter::Language {
            tree_sitter_rust::LANGUAGE.into()
        }

        fn extract_nodes(
            &self,
            _tree: &tree_sitter::Tree,
            _source: &[u8],
        ) -> Vec<DkgNode> {
            Vec::new()
        }
    }

    struct PanickingGrammar {
        id: &'static str,
    }

    impl LanguageGrammar for PanickingGrammar {
        fn language_id(&self) -> &'static str {
            self.id
        }

        fn tree_sitter_language(&self) -> tree_sitter::Language {
            panic!("simulated ABI panic for testing");
        }

        fn extract_nodes(
            &self,
            _tree: &tree_sitter::Tree,
            _source: &[u8],
        ) -> Vec<DkgNode> {
            Vec::new()
        }
    }

    // ---- tests ----

    #[test]
    fn test_empty_registry_zero_loaded() {
        let reg = LanguageRegistry::new();
        assert_eq!(reg.loaded_count(), 0);
        assert_eq!(reg.failed_count(), 0);
    }

    #[test]
    fn test_get_missing_returns_none() {
        let reg = LanguageRegistry::new();
        assert!(reg.get("rust").is_none());
    }

    #[test]
    fn test_status_returns_sorted_loaded_keys() {
        let mut reg = LanguageRegistry::new();
        // Insert in deliberately non-alphabetical order.
        reg.register(Box::new(GoodGrammar { id: "rust" })).unwrap();
        reg.register(Box::new(GoodGrammar { id: "go" })).unwrap();
        reg.register(Box::new(GoodGrammar { id: "python" })).unwrap();

        let status = reg.status();
        // BTreeMap gives alphabetical order.
        assert_eq!(status.loaded, vec!["go", "python", "rust"]);
    }

    #[test]
    fn test_failed_record_includes_lang_id() {
        let mut reg = LanguageRegistry::new();
        let result = reg.register(Box::new(PanickingGrammar { id: "bad_lang" }));
        assert!(result.is_err());
        assert_eq!(reg.failed_count(), 1);
        let status = reg.status();
        assert_eq!(status.failed[0].0, "bad_lang");
    }

    #[test]
    fn test_isolation_good_grammar_after_bad() {
        let mut reg = LanguageRegistry::new();
        // A panicking grammar should NOT abort subsequent registrations.
        let _ = reg.register(Box::new(PanickingGrammar { id: "bad_first" }));
        reg.register(Box::new(GoodGrammar { id: "rust" })).unwrap();

        assert_eq!(reg.loaded_count(), 1);
        assert_eq!(reg.failed_count(), 1);
        assert!(reg.get("rust").is_some());
    }

    #[test]
    fn test_iter_returns_all_loaded() {
        let mut reg = LanguageRegistry::new();
        reg.register(Box::new(GoodGrammar { id: "rust" })).unwrap();
        let pairs: Vec<_> = reg.iter().collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "rust");
    }

    #[test]
    fn test_try_register_panicking_factory() {
        let mut reg = LanguageRegistry::new();
        // Use explicit return type so the panic!()'s never-type coerces to
        // Box<dyn LanguageGrammar> via the closure signature — avoids
        // `unreachable_code` warnings without needing a feature gate.
        let result = reg.try_register("phantom", || -> Box<dyn LanguageGrammar> {
            panic!("factory explosion")
        });
        assert!(result.is_err());
        assert_eq!(reg.failed_count(), 1);
        assert_eq!(reg.loaded_count(), 0);
    }
}
