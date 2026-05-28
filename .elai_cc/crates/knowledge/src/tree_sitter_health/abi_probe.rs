//! E-42 Tier-B — runtime ABI version probe for registered grammars.
//!
//! `tree_sitter::Language::abi_version()` (tree-sitter 0.26.x binding,
//! verified at `~/.cargo/registry/.../tree-sitter-0.26.8/binding_rust/
//! lib.rs:511`) returns a `usize` corresponding to the ABI version the
//! grammar was generated against. The 0.26 line keeps ABI 15 (per
//! tree-sitter/tree-sitter#5013); older grammars built against ABI 14
//! are still compatible. Anything outside `{ABI_VERSION_MIN, ABI_VERSION_MAX}`
//! is rejected so a stale grammar cannot silently degrade indexing.
//!
//! Pairs with `SsbActionType::TreeSitterGrammarEvent` carrying
//! `TreeSitterGrammarEventKind::AbiProbeMismatch` (see
//! `elai_orchestrator::ssb`). Producers gate emission on
//! [`super::is_enabled`].

use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::{Deserialize, Serialize};

use crate::LanguageGrammar;

/// Minimum acceptable tree-sitter grammar ABI version. ABI 14 is the
/// 0.24 / 0.25 mature line; many TIER-A grammar crates in the workspace
/// (e.g. `tree-sitter-typescript = "0.23"`, `tree-sitter-java = "0.23"`,
/// `tree-sitter-cpp = "0.23"`) still report 14 against the
/// `tree-sitter = "0.26"` runtime.
pub const ABI_VERSION_MIN: usize = 14;

/// Maximum acceptable tree-sitter grammar ABI version. ABI 15 was added
/// in tree-sitter 0.25 (language name + version + supertype info +
/// reserved words per zed-industries/zed#24632); 0.26 is bug-fix-only
/// over that floor.
pub const ABI_VERSION_MAX: usize = 15;

/// A single grammar's reported ABI version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiVersion {
    /// The grammar's language id (e.g. "rust", "kotlin").
    pub language_id: String,
    /// The ABI version reported by `Language::abi_version()`.
    pub version: usize,
}

/// Per-grammar probe outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiProbeRecord {
    /// The language id and reported ABI version.
    pub abi: AbiVersion,
    /// `true` if the version is in `[ABI_VERSION_MIN, ABI_VERSION_MAX]`.
    pub in_range: bool,
}

impl AbiProbeRecord {
    /// Build a record from a language id + reported version.
    #[must_use]
    pub fn new(language_id: impl Into<String>, version: usize) -> Self {
        let in_range = (ABI_VERSION_MIN..=ABI_VERSION_MAX).contains(&version);
        Self {
            abi: AbiVersion {
                language_id: language_id.into(),
                version,
            },
            in_range,
        }
    }
}

/// Errors that can occur when probing a grammar's ABI version.
#[derive(Debug, thiserror::Error)]
pub enum AbiProbeError {
    /// The grammar's `abi_version()` call panicked. Usually means the
    /// underlying native library failed to load or is corrupted.
    #[error("abi probe panicked for grammar '{0}'")]
    ProbePanicked(String),

    /// The reported ABI is outside `[ABI_VERSION_MIN, ABI_VERSION_MAX]`.
    #[error(
        "grammar '{language_id}' reports ABI {version}; expected \
         {ABI_VERSION_MIN}..={ABI_VERSION_MAX}"
    )]
    OutOfRange {
        /// The grammar's language id.
        language_id: String,
        /// The reported ABI version.
        version: usize,
    },
}

/// Probe one grammar's ABI version under `catch_unwind` and return a
/// per-grammar record. Errors are surfaced as `AbiProbeError`; callers
/// MAY choose to log + continue (the index build does not stop on a
/// single ABI mismatch) or hard-fail at startup.
///
/// Returns Ok(record) for both in-range and out-of-range versions
/// (callers inspect `record.in_range`). Returns Err only when the
/// probe itself panicked (which is qualitatively different from a
/// successful "wrong ABI" report).
pub fn probe_one(
    language_id: &str,
    grammar: &dyn LanguageGrammar,
) -> Result<AbiProbeRecord, AbiProbeError> {
    // Wrap the probe in catch_unwind so a panic in the native grammar
    // does not abort the entire DkgEngine init path. AssertUnwindSafe
    // is the correct adapter here (the catch is local; no observer
    // state crosses the unwind boundary).
    let probe = catch_unwind(AssertUnwindSafe(|| {
        let lang = grammar.tree_sitter_language();
        lang.abi_version()
    }));

    match probe {
        Ok(version) => Ok(AbiProbeRecord::new(language_id, version)),
        Err(_panic_payload) => Err(AbiProbeError::ProbePanicked(language_id.to_owned())),
    }
}

/// Walk a slice of `(language_id, &dyn LanguageGrammar)` pairs and
/// return one `AbiProbeRecord` per grammar. Panics during any single
/// grammar's probe are isolated; the corresponding record is
/// substituted by a synthetic out-of-range entry with `version = 0`
/// so callers can still treat the return as Vec-aligned with the
/// input.
///
/// The single canonical entry point producers call. Wire-dark unless
/// [`super::is_enabled`] returns true at the call site (the producer
/// is responsible for the gate; this function is pure data).
#[must_use]
pub fn probe_abi_versions<'a, I>(grammars: I) -> Vec<AbiProbeRecord>
where
    I: IntoIterator<Item = (&'a str, &'a dyn LanguageGrammar)>,
{
    grammars
        .into_iter()
        .map(|(lang_id, g)| match probe_one(lang_id, g) {
            Ok(rec) => rec,
            Err(_panicked) => AbiProbeRecord {
                abi: AbiVersion {
                    language_id: lang_id.to_owned(),
                    version: 0,
                },
                in_range: false,
            },
        })
        .collect()
    // Note: alphabetical ordering is the caller's contract (BTreeMap
    // iteration order via `LanguageRegistry::iter`).
}

/// Returns Ok(()) iff every probed grammar reported a version inside
/// `[ABI_VERSION_MIN, ABI_VERSION_MAX]`. On the first out-of-range
/// grammar, returns `Err(AbiProbeError::OutOfRange)` carrying the
/// offending language id + version.
///
/// Use this when the caller wants startup-fail-fast semantics (e.g.
/// CI lint mode). The default DkgEngine path prefers
/// [`probe_abi_versions`] so partial successes still produce a usable
/// index.
pub fn assert_abi_compatibility(records: &[AbiProbeRecord]) -> Result<(), AbiProbeError> {
    for rec in records {
        if !rec.in_range {
            return Err(AbiProbeError::OutOfRange {
                language_id: rec.abi.language_id.clone(),
                version: rec.abi.version,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DkgNode;

    struct StubGrammar {
        id: &'static str,
    }

    impl LanguageGrammar for StubGrammar {
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

    struct PanickingGrammar;

    impl LanguageGrammar for PanickingGrammar {
        fn language_id(&self) -> &'static str {
            "panicker"
        }

        fn tree_sitter_language(&self) -> tree_sitter::Language {
            panic!("simulated native grammar load failure");
        }

        fn extract_nodes(
            &self,
            _tree: &tree_sitter::Tree,
            _source: &[u8],
        ) -> Vec<DkgNode> {
            Vec::new()
        }
    }

    #[test]
    fn record_in_range_when_version_is_14_or_15() {
        assert!(AbiProbeRecord::new("x", 14).in_range);
        assert!(AbiProbeRecord::new("x", 15).in_range);
    }

    #[test]
    fn record_out_of_range_when_version_is_outside_window() {
        assert!(!AbiProbeRecord::new("x", 0).in_range);
        assert!(!AbiProbeRecord::new("x", 13).in_range);
        assert!(!AbiProbeRecord::new("x", 16).in_range);
    }

    #[test]
    fn probe_one_returns_record_for_good_grammar() {
        let g = StubGrammar { id: "rust" };
        let rec = probe_one("rust", &g).expect("rust grammar must probe");
        assert_eq!(rec.abi.language_id, "rust");
        // tree-sitter-rust 0.24 reports ABI 14 or 15 against tree-sitter 0.26.
        assert!(
            rec.in_range,
            "rust grammar must report in-range ABI; got {}",
            rec.abi.version
        );
    }

    #[test]
    fn probe_one_returns_err_for_panicking_grammar() {
        let result = probe_one("panicker", &PanickingGrammar);
        assert!(matches!(result, Err(AbiProbeError::ProbePanicked(_))));
    }

    #[test]
    fn probe_abi_versions_isolates_panicking_grammar() {
        let g_good = StubGrammar { id: "rust" };
        let g_bad = PanickingGrammar;
        let pairs: Vec<(&str, &dyn LanguageGrammar)> = vec![
            ("rust", &g_good),
            ("panicker", &g_bad),
        ];
        let records = probe_abi_versions(pairs);
        assert_eq!(records.len(), 2);
        assert!(records[0].in_range, "rust should probe in-range");
        // Panicking grammar gets synthetic out-of-range record.
        assert_eq!(records[1].abi.language_id, "panicker");
        assert_eq!(records[1].abi.version, 0);
        assert!(!records[1].in_range);
    }

    #[test]
    fn assert_abi_compatibility_passes_on_all_in_range() {
        let records = vec![
            AbiProbeRecord::new("rust", 14),
            AbiProbeRecord::new("python", 15),
        ];
        assert!(assert_abi_compatibility(&records).is_ok());
    }

    #[test]
    fn assert_abi_compatibility_fails_fast_on_first_out_of_range() {
        let records = vec![
            AbiProbeRecord::new("rust", 14),
            AbiProbeRecord::new("ancient", 10),
            AbiProbeRecord::new("python", 15),
        ];
        let err = assert_abi_compatibility(&records).expect_err("must fail");
        match err {
            AbiProbeError::OutOfRange { language_id, version } => {
                assert_eq!(language_id, "ancient");
                assert_eq!(version, 10);
            }
            other => panic!("expected OutOfRange, got {other:?}"),
        }
    }
}
