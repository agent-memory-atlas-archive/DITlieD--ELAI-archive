// B-30 Pass A — pending lib.rs integration by B-29 release.
// This integration test file becomes active when language_registry is
// exposed from the crate root (pub mod dkg::language_registry + pub use).
//
// ENABLE AFTER B-29 LIB.RS RELEASE:
//   1. Remove the `#[cfg(all(test, feature = "__b30_registry_enabled__"))]` gate.
//   2. Ensure `pub mod language_registry;` exists in crates/knowledge/src/dkg/mod.rs
//      (or lib.rs after B-29 releases the lock).
//   3. Ensure `pub use dkg::language_registry::{LanguageRegistry, LanguageRegistryStatus,
//      GrammarLoadError};` is in lib.rs.

#[cfg(test)]
mod active {
    use elai_knowledge::{DkgNode, LanguageGrammar, LanguageRegistry};

    // ---------------------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------------------

    /// A grammar that successfully loads (uses tree-sitter-rust which is always present).
    struct OkGrammar {
        id: &'static str,
    }

    impl LanguageGrammar for OkGrammar {
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

    /// A grammar whose `tree_sitter_language()` always panics.
    struct PanickingGrammar(pub &'static str);

    impl LanguageGrammar for PanickingGrammar {
        fn language_id(&self) -> &'static str {
            self.0
        }

        fn tree_sitter_language(&self) -> tree_sitter::Language {
            panic!("simulated ABI-incompatible grammar: {}", self.0);
        }

        fn extract_nodes(
            &self,
            _tree: &tree_sitter::Tree,
            _source: &[u8],
        ) -> Vec<DkgNode> {
            Vec::new()
        }
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_register_real_rust_grammar() {
        let mut reg = LanguageRegistry::new();
        let result = reg.register(Box::new(OkGrammar { id: "rust" }));
        assert!(result.is_ok(), "rust grammar should load successfully");
        assert_eq!(reg.loaded_count(), 1);
        assert_eq!(reg.failed_count(), 0);
        assert!(reg.get("rust").is_some());
    }

    #[test]
    fn test_register_isolated_failure() {
        let mut reg = LanguageRegistry::new();

        // Register a panicking grammar first — should fail but not abort the registry.
        let first_result = reg.register(Box::new(PanickingGrammar("broken_grammar")));
        assert!(first_result.is_err(), "panicking grammar should return Err");

        // Register a valid grammar after — must succeed.
        let second_result = reg.register(Box::new(OkGrammar { id: "rust" }));
        assert!(second_result.is_ok(), "valid grammar after panicking one should succeed");

        assert_eq!(reg.loaded_count(), 1, "exactly one grammar should be loaded");
        assert_eq!(reg.failed_count(), 1, "exactly one grammar should be in failed");
    }

    #[test]
    fn test_status_is_sorted_deterministic() {
        let mut reg = LanguageRegistry::new();
        // Register in reverse-alphabetical order to verify BTreeMap sorts.
        reg.register(Box::new(OkGrammar { id: "rust" })).unwrap();
        reg.register(Box::new(OkGrammar { id: "python" })).unwrap();
        reg.register(Box::new(OkGrammar { id: "go" })).unwrap();

        let status = reg.status();
        assert_eq!(
            status.loaded,
            vec!["go", "python", "rust"],
            "loaded list must be alphabetically sorted"
        );
    }

    #[test]
    fn test_failed_status_is_sorted_deterministic() {
        let mut reg = LanguageRegistry::new();
        // Register in non-alphabetical order to verify sort.
        let _ = reg.register(Box::new(PanickingGrammar("php")));
        let _ = reg.register(Box::new(PanickingGrammar("kotlin")));
        let _ = reg.register(Box::new(PanickingGrammar("swift")));

        let status = reg.status();
        let failed_ids: Vec<&str> = status.failed.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            failed_ids,
            vec!["kotlin", "php", "swift"],
            "failed list must be alphabetically sorted by language id"
        );
    }
}
