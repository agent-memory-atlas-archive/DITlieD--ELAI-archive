E-42 tree-sitter grammar inventory
date: 2026-05-28
session: s-25120 completion-drive cluster 4/4 closure
plan: docs/thefinalglance/E-plan/E-42_tree_sitter_grammars_2026.md
decisions: workflow/reasoning/E-42-decisions.md

Source: cargo manifest at .elai_cc/crates/knowledge/Cargo.toml (lines 13-41,
verified 2026-05-28 against `cargo tree -p elai-knowledge | rg tree-sitter`
sanity dry-run with the workspace pin).

Columns: language_id | crate_name | crate_version | tier | ABI_range_accepted
| fork_source_url | notes

TIER-A canonical (13, locked by grammar_tier_contract.rs)
---------------------------------------------------------

c             | tree-sitter-c              | 0.24 | A | [14,15] | github.com/tree-sitter/tree-sitter-c          | TIER_A_GRAMMARS[0]
cpp           | tree-sitter-cpp            | 0.23 | A | [14,15] | github.com/tree-sitter/tree-sitter-cpp        | C++20 modules merged-but-unreleased upstream (#341)
csharp        | tree-sitter-c-sharp        | 0.23 | A | [14,15] | github.com/tree-sitter/tree-sitter-c-sharp    |
go            | tree-sitter-go             | 0.25 | A | [14,15] | github.com/tree-sitter/tree-sitter-go         |
java          | tree-sitter-java           | 0.23 | A | [14,15] | github.com/tree-sitter/tree-sitter-java       |
javascript    | tree-sitter-javascript     | 0.25 | A | [14,15] | github.com/tree-sitter/tree-sitter-javascript |
kotlin        | tree-sitter-kotlin-ng      | 1.1  | A | [14,15] | github.com/tree-sitter-grammars/tree-sitter-kotlin | C-17 2026-05-18 anti-slopsquatting; NOT fwcd fork
python        | tree-sitter-python         | 0.25 | A | [14,15] | github.com/tree-sitter/tree-sitter-python     |
ruby          | tree-sitter-ruby           | 0.23 | A | [14,15] | github.com/tree-sitter/tree-sitter-ruby       |
rust          | tree-sitter-rust           | 0.24 | A | [14,15] | github.com/tree-sitter/tree-sitter-rust       |
svelte        | tree-sitter-svelte-ng      | 1.0  | A | [14,15] | github.com/tree-sitter-grammars/tree-sitter-svelte | NOT abandoned plain crate; Svelte 5 runes best-effort
tsx           | tree-sitter-typescript     | 0.23 | A | [14,15] | github.com/tree-sitter/tree-sitter-typescript | TsxGrammar uses LANGUAGE_TSX
typescript    | tree-sitter-typescript     | 0.23 | A | [14,15] | github.com/tree-sitter/tree-sitter-typescript |

TIER-B (community-curated, registered by DkgEngine flat block + per-tier helper)
--------------------------------------------------------------------------------

scala         | tree-sitter-scala          | 0.26 | B | [14,15] | github.com/tree-sitter/tree-sitter-scala      | community-curated; macro syntax brittle
swift         | tree-sitter-swift          | 0.7  | B | [14,15] | github.com/alex-pinkus/tree-sitter-swift      | single-maintainer upstream
php           | tree-sitter-php            | 0.24 | B | [14,15] | github.com/tree-sitter/tree-sitter-php        | PHP 8 attribute syntax brittle
bash          | tree-sitter-bash           | 0.25 | B | [14,15] | github.com/tree-sitter/tree-sitter-bash       | POSIX-only

Inline-used (NOT in TIER_A_GRAMMARS)
------------------------------------

html          | tree-sitter-html           | 0.23 | inline | [14,15] | github.com/tree-sitter/tree-sitter-html  | outer parser for Vue/Svelte composite
css           | tree-sitter-css            | 0.25 | inline | [14,15] | github.com/tree-sitter/tree-sitter-css   | @import extraction at dkg/mod.rs:1623-1690
vue           | (handled via html composite) | n/a | composite | [14,15] | n/a                                       | tree_sitter_html outer + TS secondary; no tree-sitter-vue dep

Optional / gated
----------------

astro         | tree-sitter-astro-next     | 0.1  | C | [14,15] | github.com/PRRPCHT/tree-sitter-astro-next     | feature-gated behind web-astro (single-maintainer)

Banned (locked by BANNED_GRAMMAR_CRATES)
----------------------------------------

tree-sitter-language-pack -- meta-dep, ships stale ABIs + license mismatch.
tree-sitter-kotlin (fwcd fork) -- abandoned. Replaced by tree-sitter-kotlin-ng.
tree-sitter-svelte (plain crate) -- abandoned. Replaced by tree-sitter-svelte-ng.

Tier-A grammar deps DEFERRED for anti-slopsquatting gate (DRIFT-8)
-------------------------------------------------------------------

The plan's open question on adding html/css/json/yaml/toml/markdown to
TIER-A is BLOCKED on anti-slopsquatting WebSearch + user authorization
per CLAUDE.md HARD RULE. html and css are already on the workspace
deps as inline-use crates (so they need no further verification); json,
yaml, toml, markdown grammar crates would each need:
  - npm view / cargo search for canonical name resolution
  - WebFetch repo URL + last release date
  - User confirmation in chat before `cargo add`
The B-30 PLANNED multi-language DKG hardening was the trigger; E-42
itself does NOT promote these to TIER-A.

ABI version probe verdict (DKG init, wire-dark default)
-------------------------------------------------------

When `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1` is set, the substrate at
`crate::tree_sitter_health::abi_probe::probe_abi_versions` walks the
LanguageRegistry's `iter()` output and emits one
`TreeSitterGrammarEventKind::AbiProbeMismatch` row per grammar whose
reported version is outside `[14, 15]`. Default deployments emit zero
rows (the env gate is off). Sanity: every grammar listed above reports
14 or 15 against tree-sitter = "0.26" runtime per
github.com/tree-sitter/tree-sitter/issues/5013.

Codebase mapper regen
---------------------

After the E-42 closure ships, regen with:
  python .claude/tools/codebase_mapper.py .elai_cc elai_graph.json \\
    --compact --map elai.map --scip-dir .claude/tools/scip_indices

Spot-check the row counts for the TIER-A languages do not drop more
than 5% (no regression). The new `tree_sitter_health` Rust module
adds rows for rust DEF/CALL entries; the post-regen row count should
INCREASE.

END E-42-grammar-versions-2026-05-28.md
