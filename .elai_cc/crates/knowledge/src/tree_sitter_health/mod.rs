//! E-42 Tier-B closure — tree-sitter grammar health substrate.
//!
//! Module is named `tree_sitter_health` (not `tree_sitter`) to avoid
//! shadowing the external `tree_sitter` crate. The file system path
//! `crates/knowledge/src/tree_sitter_health/` is the canonical landing
//! per the E-42 launch prompt's `knowledge/src/tree_sitter/ extensions`
//! directive (the extension family lives here under the disambiguated
//! name).
//!
//! Landing zone for E-42's deferred Tier-B work (per
//! `workflow/reasoning/E-42-decisions.md` (b)+(c)). Sits parallel to
//! the runtime `crate::dkg::language_registry::LanguageRegistry` (which
//! holds catch_unwind-isolated registration state) and consumes the
//! design-time metadata from `crate::grammar_tier::TIER_A_GRAMMARS`.
//!
//! Wire-dark by default. Producers under `crate::dkg::DkgEngine` only
//! emit the paired SSB events when the operator sets
//! `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1` at the call site, mirroring the
//! E-20 / E-21 / E-36 wire-dark precedent.
//!
//! Modules:
//!   * [`abi_probe`] — runtime ABI version check per registered grammar
//!     (`tree_sitter::Language::abi_version()`). Rejects ABI outside
//!     `{14, 15}` per the May 2026 maturity table. No `.unwrap()`; all
//!     callers route Result-typed.
//!   * [`svelte_health`] — Svelte 5 runes detection + ERROR / MISSING
//!     node counting + raw-text-fallback classifier. Honest about the
//!     upstream tree-sitter-svelte-ng lag on runes.
//!   * [`large_file`] — 2 MiB ceiling guard for tree-sitter ingest
//!     (skip-as-raw-text when source byte length exceeds the limit).
//!   * [`registration`] — `register_tier_a_languages` /
//!     `register_tier_b_languages` helpers that consume
//!     [`crate::grammar_tier::TIER_A_GRAMMARS`] as the iteration source.
//!     The Tier-B helper is cfg-gated behind the `experimental_grammars`
//!     Cargo feature (see `crates/knowledge/Cargo.toml`).
//!
//! Composite-language injection note: the existing
//! `crate::dkg::VueGrammar::vue_extract_script_recursive` and the
//! `SvelteGrammar` reuse of the TypeScript walker are the canonical
//! reconciliations for `<script>` / `<style>` injection. The bundled
//! `queries/injections.scm` reference doc is descriptive (anchors
//! `script_element` / `style_element` to their downstream parsers) so
//! future maintainers do not re-introduce a parallel scm-file routing
//! pathway that would conflict with the shipped composite-reparse.

pub mod abi_probe;
pub mod large_file;
pub mod registration;
pub mod svelte_health;

/// Wire-dark env var name. When set to a truthy value (`"1"` / `"true"` /
/// `"yes"`), producers in `crate::dkg::DkgEngine` MAY emit
/// `TreeSitterGrammarEvent` SSB rows for ABI probe mismatches, Svelte
/// runes health, Tier-B grammar loads, and large-file skips. Mirrors
/// the precedent set by `ELAI_PEER_SYSTEMS_ENABLE` /
/// `ELAI_DOCUMENT_VISION_RAG_ENABLE` per
/// `workflow/reasoning/E-42-decisions.md` (Tier-B).
pub const ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV: &str =
    "ELAI_TREE_SITTER_GRAMMARS_ENABLE";

/// Returns `true` when the wire-dark env gate is set to a truthy value.
///
/// Truthy values (case-insensitive): `"1"`, `"true"`, `"yes"`, `"on"`.
/// Anything else (including unset) returns `false`. Producer call sites
/// MUST gate emission on this so the substrate stays observation-only
/// in default deployments.
#[must_use]
pub fn is_enabled() -> bool {
    match std::env::var(ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV) {
        Ok(v) => {
            let trimmed = v.trim().to_ascii_lowercase();
            matches!(trimmed.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_enabled_defaults_to_false_when_unset() {
        // SAFETY: scoped to test thread; var name is local to E-42.
        unsafe { std::env::remove_var(ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV); }
        assert!(!is_enabled());
    }

    #[test]
    fn is_enabled_accepts_canonical_truthy_values() {
        for v in ["1", "true", "TRUE", "yes", "ON"] {
            // SAFETY: scoped to test thread.
            unsafe { std::env::set_var(ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV, v); }
            assert!(is_enabled(), "expected truthy for {v}");
        }
        // SAFETY: scoped to test thread.
        unsafe { std::env::remove_var(ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV); }
    }

    #[test]
    fn is_enabled_rejects_non_truthy_values() {
        for v in ["0", "false", "no", "off", ""] {
            // SAFETY: scoped to test thread.
            unsafe { std::env::set_var(ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV, v); }
            assert!(!is_enabled(), "expected falsy for {v}");
        }
        // SAFETY: scoped to test thread.
        unsafe { std::env::remove_var(ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV); }
    }
}
