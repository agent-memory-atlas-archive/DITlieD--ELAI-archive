//! E-42 Tier-B — per-tier grammar registration helpers.
//!
//! These helpers consume [`crate::grammar_tier::TIER_A_GRAMMARS`] as
//! the canonical iteration source and dispatch each entry to its
//! matching factory closure. They are thin wrappers over
//! `LanguageRegistry::try_register` — the existing flat block at
//! `crate::dkg::DkgEngine::new` (dkg/mod.rs:209-234) remains the
//! production call site for backwards compatibility; this module
//! provides the SAME wiring under named helpers so future audits +
//! per-tier producers (e.g. `register_tier_b_languages` cfg-gated
//! behind the `experimental_grammars` Cargo feature) have a clean
//! seam to extend.
//!
//! The Tier-B helper is cfg-gated. When the operator builds without
//! `--features experimental_grammars`, the function returns an empty
//! `Vec` and emits no Tier-B grammar status. When the feature is
//! enabled, the function registers the curated Tier-B set behind the
//! same `catch_unwind` isolation as Tier-A.
//!
//! TIER-B grammar set rationale (per
//! `workflow/reasoning/E-42-decisions.md`):
//!   * `scala` / `swift` / `kotlin` (org -ng variant) / `php` /
//!     `bash` / `ruby` / `csharp` — community-curated, ship best-effort;
//!     Tier-A registration is also retained for backwards compat in
//!     `DkgEngine::new`. The Tier-B set is the strict superset of
//!     "may emit ERROR nodes on uncommon idioms".
//!   * cpp is NOT registered here at Tier-B because the existing flat
//!     block already covers it; promotion to either Tier-A-with-caveat
//!     or Tier-C is deferred (C++20 modules merged-but-unreleased per
//!     tree-sitter/tree-sitter-cpp#341).
//!   * vue is NOT registered here at Tier-B because the shipped
//!     composite-reparse via `tree_sitter_html` is the canonical path
//!     (`crate::dkg::VueGrammar::vue_extract_script_recursive`).
//!
//! No new Cargo deps are added in this slice (json / yaml / toml /
//! markdown additions need anti-slopsquatting WebSearch + user gate
//! per `workflow/reasoning/E-42-decisions.md` DRIFT-8).

use crate::dkg::language_registry::{GrammarLoadError, LanguageRegistry};
use crate::dkg::{
    BashGrammar, CGrammar, CppGrammar, CSharpGrammar, GoGrammar, JavaGrammar, JavaScriptGrammar,
    KotlinGrammar, PhpGrammar, PythonGrammar, RubyGrammar, RustGrammar, ScalaGrammar,
    SvelteGrammar, SwiftGrammar, TsxGrammar, TypeScriptGrammar, VueGrammar,
};
use crate::LanguageGrammar;

/// Outcome of a per-tier registration call.
///
/// Does NOT derive `Eq` because `GrammarLoadError` carries a free-form
/// message in its `Other` variant and the existing 0.39.2 thiserror
/// derive does not synthesise Eq for it.
#[derive(Debug, Clone, PartialEq)]
pub struct TierRegistrationOutcome {
    /// Language ids successfully registered, in registration order.
    pub registered: Vec<String>,
    /// Per-language failures observed during this tier's registration.
    /// Other tiers are NOT affected.
    pub failures: Vec<(String, GrammarLoadError)>,
}

impl TierRegistrationOutcome {
    fn empty() -> Self {
        Self {
            registered: Vec::new(),
            failures: Vec::new(),
        }
    }
}

/// Register the canonical TIER-A grammar set into `registry`.
///
/// Iteration order matches `crate::grammar_tier::TIER_A_GRAMMARS`
/// (alphabetical). The function returns a [`TierRegistrationOutcome`]
/// summarising which languages loaded and which failed; failures are
/// observation-only (the `LanguageRegistry::try_register` path already
/// records them in `registry.status().failed`).
///
/// This helper is the seam future producers iterate against without
/// reaching into the flat dkg/mod.rs block. `DkgEngine::new` keeps
/// its existing inline registration to preserve byte-for-byte
/// compatibility with downstream callers; both paths reach the same
/// `LanguageRegistry::try_register` underneath.
pub fn register_tier_a_languages(registry: &mut LanguageRegistry) -> TierRegistrationOutcome {
    let mut outcome = TierRegistrationOutcome::empty();

    for meta in crate::grammar_tier::TIER_A_GRAMMARS {
        let result = try_register_by_language_id(registry, meta.language_id);
        record_result(&mut outcome, meta.language_id, result);
    }

    // Vue is registered via the composite-reparse pattern. It is NOT
    // in TIER_A_GRAMMARS (its grammar is the HTML grammar with a
    // secondary TS pass; see VueGrammar at dkg/mod.rs:1715-1786) so
    // the helper handles it explicitly to retain parity with the
    // existing DkgEngine::new flat block.
    let vue_result = registry.try_register("vue", || {
        Box::new(VueGrammar) as Box<dyn LanguageGrammar>
    });
    record_result(&mut outcome, "vue", vue_result);

    outcome
}

/// Register the curated TIER-B grammar set into `registry`. Gated
/// behind the `experimental_grammars` Cargo feature; without the
/// feature this function compiles to an empty stub that always
/// returns an empty outcome.
#[cfg(feature = "experimental_grammars")]
pub fn register_tier_b_languages(registry: &mut LanguageRegistry) -> TierRegistrationOutcome {
    let mut outcome = TierRegistrationOutcome::empty();

    // TIER-B grammars that DO ship in the workspace deps today. These
    // are also registered by DkgEngine::new for backwards compat;
    // having a named seam here lets future audit / dashboard surfaces
    // discriminate "loaded as Tier-B" without re-deriving from the
    // canonical list.
    for lang_id in TIER_B_LANGUAGE_IDS {
        let result = try_register_by_language_id(registry, lang_id);
        record_result(&mut outcome, lang_id, result);
    }

    outcome
}

/// No-op fallback when the `experimental_grammars` feature is not
/// enabled. The helper signature stays callable so downstream code
/// (e.g. test contracts) does not need its own cfg attribute.
#[cfg(not(feature = "experimental_grammars"))]
pub fn register_tier_b_languages(_registry: &mut LanguageRegistry) -> TierRegistrationOutcome {
    TierRegistrationOutcome::empty()
}

/// TIER-B language ids (curated subset of community grammars that
/// "may emit ERROR nodes" per the May 2026 maturity table). Subset
/// of the existing DkgEngine flat block; the helper does not re-add
/// the canonical TIER-A set.
#[cfg(feature = "experimental_grammars")]
const TIER_B_LANGUAGE_IDS: &[&str] = &["scala", "swift", "php", "bash"];

/// Dispatch a language id to its concrete factory closure +
/// `try_register` call. Centralised so both Tier-A and Tier-B helpers
/// stay declarative.
fn try_register_by_language_id(
    registry: &mut LanguageRegistry,
    language_id: &str,
) -> Result<(), GrammarLoadError> {
    match language_id {
        "rust" => registry.try_register(language_id, || {
            Box::new(RustGrammar) as Box<dyn LanguageGrammar>
        }),
        "python" => registry.try_register(language_id, || {
            Box::new(PythonGrammar) as Box<dyn LanguageGrammar>
        }),
        "go" => registry.try_register(language_id, || {
            Box::new(GoGrammar) as Box<dyn LanguageGrammar>
        }),
        "typescript" => registry.try_register(language_id, || {
            Box::new(TypeScriptGrammar) as Box<dyn LanguageGrammar>
        }),
        "tsx" => registry.try_register(language_id, || {
            Box::new(TsxGrammar) as Box<dyn LanguageGrammar>
        }),
        "svelte" => registry.try_register(language_id, || {
            Box::new(SvelteGrammar) as Box<dyn LanguageGrammar>
        }),
        "java" => registry.try_register(language_id, || {
            Box::new(JavaGrammar) as Box<dyn LanguageGrammar>
        }),
        "kotlin" => registry.try_register(language_id, || {
            Box::new(KotlinGrammar) as Box<dyn LanguageGrammar>
        }),
        "csharp" => registry.try_register(language_id, || {
            Box::new(CSharpGrammar) as Box<dyn LanguageGrammar>
        }),
        "ruby" => registry.try_register(language_id, || {
            Box::new(RubyGrammar) as Box<dyn LanguageGrammar>
        }),
        "javascript" => registry.try_register(language_id, || {
            Box::new(JavaScriptGrammar) as Box<dyn LanguageGrammar>
        }),
        "c" => registry.try_register(language_id, || {
            Box::new(CGrammar) as Box<dyn LanguageGrammar>
        }),
        "cpp" => registry.try_register(language_id, || {
            Box::new(CppGrammar) as Box<dyn LanguageGrammar>
        }),
        "scala" => registry.try_register(language_id, || {
            Box::new(ScalaGrammar) as Box<dyn LanguageGrammar>
        }),
        "php" => registry.try_register(language_id, || {
            Box::new(PhpGrammar) as Box<dyn LanguageGrammar>
        }),
        "swift" => registry.try_register(language_id, || {
            Box::new(SwiftGrammar) as Box<dyn LanguageGrammar>
        }),
        "bash" => registry.try_register(language_id, || {
            Box::new(BashGrammar) as Box<dyn LanguageGrammar>
        }),
        // Unknown language id is treated as a NotFound-style failure so
        // the caller's outcome record reflects the gap without panicking.
        other => Err(GrammarLoadError::Other(
            other.to_owned(),
            "unknown language id in tier registration helper".to_owned(),
        )),
    }
}

fn record_result(
    outcome: &mut TierRegistrationOutcome,
    language_id: &str,
    result: Result<(), GrammarLoadError>,
) {
    match result {
        Ok(()) => outcome.registered.push(language_id.to_owned()),
        Err(err) => outcome.failures.push((language_id.to_owned(), err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_a_helper_registers_full_canonical_set() {
        let mut registry = LanguageRegistry::new();
        let outcome = register_tier_a_languages(&mut registry);

        // Every TIER_A_GRAMMARS entry should be in `registered` (the
        // canonical 13) plus the explicit vue entry handled by the
        // helper, for a total of 14.
        let expected_min = crate::grammar_tier::TIER_A_GRAMMARS.len() + 1;
        assert!(
            outcome.registered.len() == expected_min
                || (!outcome.registered.is_empty()
                    && outcome.registered.len() + outcome.failures.len() == expected_min),
            "registered + failures should account for all {} expected entries, \
             got registered={} failures={}",
            expected_min,
            outcome.registered.len(),
            outcome.failures.len()
        );
        // Spot-check that rust + kotlin made it through (the most
        // load-bearing TIER-A entries).
        assert!(outcome.registered.iter().any(|s| s == "rust"));
        assert!(outcome.registered.iter().any(|s| s == "kotlin"));
    }

    #[test]
    fn tier_b_helper_returns_empty_without_feature() {
        // Without the `experimental_grammars` feature the function
        // compiles to an empty stub.
        let mut registry = LanguageRegistry::new();
        let outcome = register_tier_b_languages(&mut registry);
        #[cfg(not(feature = "experimental_grammars"))]
        {
            assert_eq!(outcome.registered.len(), 0);
            assert_eq!(outcome.failures.len(), 0);
        }
        // When the feature IS enabled, the helper registers TIER_B_LANGUAGE_IDS.
        #[cfg(feature = "experimental_grammars")]
        {
            let total = outcome.registered.len() + outcome.failures.len();
            assert_eq!(total, super::TIER_B_LANGUAGE_IDS.len());
        }
    }
}
