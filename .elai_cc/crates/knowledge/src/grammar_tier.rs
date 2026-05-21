//! E-42 Tier-A — tree-sitter grammar tier metadata.
//!
//! Types-first foundation (E-36.A + E-52.A pattern) for the grammar-tier
//! classification scheme. The existing LanguageRegistry at
//! `knowledge::dkg::language_registry` registers 18+ grammars as a flat list
//! at `dkg/mod.rs:209-234`. This module adds the TIER metadata as a
//! parallel surface so future producers + dashboards + audits can route
//! per-tier without changing the registration logic itself.
//!
//! Tier-B will refactor the dkg/mod.rs registration into `register_tier_a` /
//! `register_tier_b` helpers, add the 4 GrammarHealth SSB variants, and
//! land a GrammarHealthPanel.svelte. All explicitly deferred per
//! workflow/reasoning/E-42-decisions.md.
//!
//! GRAMMAR PIN INVARIANTS this module locks (enforced by
//! crates/contracts/tests/grammar_tier_contract.rs):
//!   - kotlin grammar is on `tree-sitter-kotlin-ng` (the org variant), NOT
//!     the fwcd fork. Reason: C-17 anti-slopsquatting 2026-05-18 decision.
//!   - svelte grammar is on `tree-sitter-svelte-ng` (the active fork), NOT
//!     the abandoned `tree-sitter-svelte` crate.
//!   - tree-sitter runtime is pinned at exactly 0.26 across the workspace.
//!   - tree-sitter-language-pack is BANNED — it bundles all grammars under
//!     one MIT/Apache mismatch + ships stale ABIs.
//!   - MDX (Markdown JSX) is NOT a TIER-A target — its grammar is
//!     experimental and embedded-language-routing for it is incomplete.

use serde::{Deserialize, Serialize};

/// Tier classification for a registered grammar.
///
/// Tier-A = first-party langs ELAI actively indexes for code projects.
/// Tier-B = community-curated langs supported with best-effort + caveats.
/// Tier-C = experimental / incubating — may panic, may have ABI drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrammarTier {
    /// First-party langs indexed for every project that contains them.
    A,
    /// Community langs supported best-effort.
    B,
    /// Experimental — may degrade gracefully (catch_unwind) rather than fail
    /// the index build.
    C,
}

impl GrammarTier {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
        }
    }
}

/// Static per-grammar metadata. The set is canonical at compile time; growth
/// requires bumping `crates/contracts/tests/grammar_tier_contract.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarMeta {
    /// LanguageRegistry id (e.g. "rust", "kotlin"). Lower-case canonical.
    pub language_id: &'static str,
    /// The Cargo crate name shipping the grammar (e.g.
    /// "tree-sitter-rust", "tree-sitter-kotlin-ng"). Pinned by the tier
    /// contract so a future "consolidate to tree-sitter-language-pack" pass
    /// fails the test loudly.
    pub crate_name: &'static str,
    /// Tier classification.
    pub tier: GrammarTier,
}

/// Canonical TIER-A grammar list. Order is alphabetical (not registration
/// order). Adding a TIER-A grammar requires updating this slice AND the
/// contract test.
pub const TIER_A_GRAMMARS: &[GrammarMeta] = &[
    GrammarMeta {
        language_id: "c",
        crate_name: "tree-sitter-c",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "cpp",
        crate_name: "tree-sitter-cpp",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "csharp",
        crate_name: "tree-sitter-c-sharp",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "go",
        crate_name: "tree-sitter-go",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "java",
        crate_name: "tree-sitter-java",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "javascript",
        crate_name: "tree-sitter-javascript",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "kotlin",
        // C-17 anti-slopsquatting 2026-05-18: org variant, NOT fwcd fork.
        crate_name: "tree-sitter-kotlin-ng",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "python",
        crate_name: "tree-sitter-python",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "ruby",
        crate_name: "tree-sitter-ruby",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "rust",
        crate_name: "tree-sitter-rust",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "svelte",
        // The active fork at tree-sitter-grammars/tree-sitter-svelte-ng,
        // not the abandoned tree-sitter-svelte.
        crate_name: "tree-sitter-svelte-ng",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "tsx",
        crate_name: "tree-sitter-typescript",
        tier: GrammarTier::A,
    },
    GrammarMeta {
        language_id: "typescript",
        crate_name: "tree-sitter-typescript",
        tier: GrammarTier::A,
    },
];

/// Look up TIER-A metadata by language id. None for non-TIER-A.
#[must_use]
pub fn tier_a_for(language_id: &str) -> Option<&'static GrammarMeta> {
    TIER_A_GRAMMARS.iter().find(|m| m.language_id == language_id)
}

/// Banned grammar crate names. If any of these appears in workspace deps the
/// grammar_tier_contract should fail. See module docs for rationale.
pub const BANNED_GRAMMAR_CRATES: &[&str] = &[
    // tree-sitter-language-pack bundles all grammars under one umbrella +
    // ships stale ABIs + carries MIT/Apache mismatch with workspace policy.
    "tree-sitter-language-pack",
    // The abandoned fwcd kotlin fork. Use tree-sitter-kotlin-ng instead.
    "tree-sitter-kotlin",
    // The abandoned svelte crate. Use tree-sitter-svelte-ng.
    "tree-sitter-svelte",
];

/// Tree-sitter runtime version this workspace expects. The contract test
/// asserts Cargo.toml pins this exact value.
pub const REQUIRED_TREE_SITTER_VERSION: &str = "0.26";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_a_list_is_sorted_by_language_id() {
        // Lexicographic invariant — additions must be inserted at the
        // alphabetical position.
        let ids: Vec<&str> = TIER_A_GRAMMARS.iter().map(|m| m.language_id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted, "TIER_A_GRAMMARS must stay alphabetical");
    }

    #[test]
    fn tier_a_kotlin_uses_org_variant() {
        let kotlin = tier_a_for("kotlin").expect("kotlin must be TIER-A");
        assert_eq!(
            kotlin.crate_name, "tree-sitter-kotlin-ng",
            "kotlin grammar MUST be on the org variant -ng, not the fwcd fork"
        );
    }

    #[test]
    fn tier_a_svelte_uses_ng_variant() {
        let svelte = tier_a_for("svelte").expect("svelte must be TIER-A");
        assert_eq!(svelte.crate_name, "tree-sitter-svelte-ng");
    }

    #[test]
    fn banned_grammar_crates_include_language_pack() {
        assert!(BANNED_GRAMMAR_CRATES.contains(&"tree-sitter-language-pack"));
    }
}
