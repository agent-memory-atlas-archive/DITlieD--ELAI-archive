//! E-42 Tier-A — grammar_tier_contract.
//!
//! Locks the TIER-A grammar set + crate-name invariants + banned-crate list
//! + tree-sitter runtime version pin. Growth of any of these requires
//! coordinated edits to this contract + workflow/reasoning/E-42-decisions.md.

use elai_knowledge::grammar_tier::{
    tier_a_for, GrammarTier, BANNED_GRAMMAR_CRATES, REQUIRED_TREE_SITTER_VERSION,
    TIER_A_GRAMMARS,
};

#[test]
fn tier_a_grammar_count_locked_at_13() {
    assert_eq!(
        TIER_A_GRAMMARS.len(),
        13,
        "TIER-A grammar count locked at 13 \
         (c, cpp, csharp, go, java, javascript, kotlin, python, ruby, rust, svelte, tsx, typescript). \
         Growth requires updating this assertion AND the canonical list."
    );
}

#[test]
fn tier_a_set_includes_canonical_languages() {
    // Pin the canonical TIER-A set. If a grammar leaves TIER-A this test
    // fails loudly before the demotion can ship silently.
    let canonical = [
        "c",
        "cpp",
        "csharp",
        "go",
        "java",
        "javascript",
        "kotlin",
        "python",
        "ruby",
        "rust",
        "svelte",
        "tsx",
        "typescript",
    ];
    for id in canonical {
        assert!(
            tier_a_for(id).is_some(),
            "language '{id}' must be in TIER_A_GRAMMARS"
        );
    }
}

#[test]
fn kotlin_uses_org_ng_variant_not_fwcd() {
    let kotlin = tier_a_for("kotlin").expect("kotlin must be TIER-A");
    assert_eq!(
        kotlin.crate_name, "tree-sitter-kotlin-ng",
        "kotlin grammar MUST be on the org -ng variant per C-17 \
         anti-slopsquatting 2026-05-18 decision. Reverting to the fwcd fork \
         is a regression."
    );
    assert_eq!(kotlin.tier, GrammarTier::A);
}

#[test]
fn svelte_uses_ng_variant_not_abandoned() {
    let svelte = tier_a_for("svelte").expect("svelte must be TIER-A");
    assert_eq!(
        svelte.crate_name, "tree-sitter-svelte-ng",
        "svelte grammar MUST be on the -ng variant. The plain \
         tree-sitter-svelte crate is abandoned."
    );
}

#[test]
fn tier_a_is_alphabetical() {
    // Future additions must be inserted at the alphabetical position.
    let ids: Vec<&str> = TIER_A_GRAMMARS.iter().map(|m| m.language_id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "TIER_A_GRAMMARS must remain alphabetical");
}

#[test]
fn banned_crates_include_language_pack_and_dead_forks() {
    assert!(BANNED_GRAMMAR_CRATES.contains(&"tree-sitter-language-pack"));
    assert!(BANNED_GRAMMAR_CRATES.contains(&"tree-sitter-kotlin"));
    assert!(BANNED_GRAMMAR_CRATES.contains(&"tree-sitter-svelte"));
}

#[test]
fn no_tier_a_crate_is_on_the_banned_list() {
    for meta in TIER_A_GRAMMARS {
        assert!(
            !BANNED_GRAMMAR_CRATES.contains(&meta.crate_name),
            "TIER-A grammar '{}' uses banned crate '{}'",
            meta.language_id,
            meta.crate_name
        );
    }
}

#[test]
fn required_tree_sitter_version_is_pinned() {
    assert_eq!(REQUIRED_TREE_SITTER_VERSION, "0.26");
}

#[test]
fn grammar_tier_str_round_trip() {
    assert_eq!(GrammarTier::A.as_str(), "a");
    assert_eq!(GrammarTier::B.as_str(), "b");
    assert_eq!(GrammarTier::C.as_str(), "c");
}
