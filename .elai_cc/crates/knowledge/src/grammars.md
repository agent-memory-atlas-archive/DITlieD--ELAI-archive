E-42 — tree-sitter grammar pins, forks, and ABI invariants
============================================================

This is the canonical reference for the elai-knowledge crate's
tree-sitter grammar set. It documents the pin invariants enforced by
`crates/contracts/tests/grammar_tier_contract.rs` and the per-grammar
caveats the audit pipeline at
`workflow/audits/E-42-grammar-versions-2026-05-28.md` aggregates.

The runtime pin
---------------

The workspace pins `tree-sitter = "0.26"` at
`crates/knowledge/Cargo.toml:13`. The design-time string constant
`elai_knowledge::grammar_tier::REQUIRED_TREE_SITTER_VERSION = "0.26"`
is asserted by the contract test
`required_tree_sitter_version_is_pinned`. tree-sitter 0.26 is the
bug-fix-only line over 0.25's ABI 15 bump (per
github.com/tree-sitter/tree-sitter/issues/5013, verified May 2026).
Future bumps need a coordinated grammar-crate rebuild and a contract-
test update in the same commit.

ABI version range
-----------------

The runtime accepts grammars reporting ABI in
`[ABI_VERSION_MIN, ABI_VERSION_MAX] = [14, 15]`. ABI 14 is the 0.24 /
0.25 mature line; many TIER-A grammar crates in the workspace
(e.g. `tree-sitter-typescript = "0.23"`, `tree-sitter-java = "0.23"`,
`tree-sitter-cpp = "0.23"`) still report 14 against the
`tree-sitter = "0.26"` runtime. ABI 15 carries the language name +
version + supertype info + reserved words metadata per
github.com/zed-industries/zed/issues/24632 (verified May 2026). The
probe at `elai_knowledge::tree_sitter_health::abi_probe` rejects
anything outside this range and emits a
`TreeSitterGrammarEventKind::AbiProbeMismatch` SSB row when
`ELAI_TREE_SITTER_GRAMMARS_ENABLE=1`.

TIER-A canonical list (13 entries, locked)
------------------------------------------

The TIER-A list is locked at exactly 13 entries by the contract test
`tier_a_grammar_count_locked_at_13`. Adding a grammar requires
updating BOTH the canonical slice at
`crates/knowledge/src/grammar_tier.rs::TIER_A_GRAMMARS` AND the
contract assertion. Alphabetical order is also enforced. The 13 are:

  c            -- tree-sitter-c "0.24"
  cpp          -- tree-sitter-cpp "0.23"
  csharp       -- tree-sitter-c-sharp "0.23"
  go           -- tree-sitter-go "0.25"
  java         -- tree-sitter-java "0.23"
  javascript   -- tree-sitter-javascript "0.25"
  kotlin       -- tree-sitter-kotlin-ng "1.1"  (NOT fwcd; see below)
  python       -- tree-sitter-python "0.25"
  ruby         -- tree-sitter-ruby "0.23"
  rust         -- tree-sitter-rust "0.24"
  svelte       -- tree-sitter-svelte-ng "1.0"  (NOT plain; see below)
  tsx          -- tree-sitter-typescript "0.23" (via TsxGrammar)
  typescript   -- tree-sitter-typescript "0.23"

Languages used inline but not in TIER-A
---------------------------------------

  html         -- tree-sitter-html "0.23" — registered as the outer
                  parser for Vue / Svelte composite reparse at
                  `dkg/mod.rs:1715-1786 (VueGrammar)`; NOT indexed as a
                  standalone TIER-A language because HTML symbol
                  extraction is brittle compared to embedded
                  JavaScript / TypeScript.
  css          -- tree-sitter-css "0.25" — used for `@import`
                  extraction at `dkg/mod.rs:1623-1690`; same NOT-in-
                  TIER-A rationale.
  scala        -- tree-sitter-scala "0.26" — registered by
                  DkgEngine::new but classified TIER-B (community-
                  curated, may emit ERROR nodes on advanced macro
                  syntax).
  swift        -- tree-sitter-swift "0.7" — TIER-B (single-maintainer
                  upstream).
  php          -- tree-sitter-php "0.24" — TIER-B (PHP 8 attribute
                  syntax brittle).
  bash         -- tree-sitter-bash "0.25" — TIER-B (POSIX-only).

Kotlin fork-selector (LOCKED)
-----------------------------

The Kotlin grammar pin is `tree-sitter-kotlin-ng "1.1"` from the
official `tree-sitter-grammars` GitHub org (anti-slopsquatting
verified 2026-05-18 via `cargo search tree-sitter-kotlin`). The
fwcd/tree-sitter-kotlin fork is BANNED per
`BANNED_GRAMMAR_CRATES` and the contract test
`kotlin_uses_org_ng_variant_not_fwcd`. Reverting to fwcd is a C-17
regression. Any future plan that asserts the fwcd path is canonical
should be treated as a 2025-snapshot drift.

Svelte fork-selector (LOCKED)
-----------------------------

The Svelte grammar pin is `tree-sitter-svelte-ng "1.0"` from the
official `tree-sitter-grammars` GitHub org. The abandoned plain
`tree-sitter-svelte` crate is BANNED per `BANNED_GRAMMAR_CRATES`
and the contract test `svelte_uses_ng_variant_not_abandoned`. As of
May 2026 no upstream tag formally supports Svelte 5 runes
(`$state`, `$derived`, `$effect`, `$props`); the substrate at
`elai_knowledge::tree_sitter_health::svelte_health` ships an honest
"best-effort with health telemetry" path. We do NOT advertise full
Svelte 5 support to users.

Vue composite-reparse (CANONICAL, do not duplicate)
---------------------------------------------------

Vue Single File Components are parsed in two passes:
  1. tree-sitter-html parses the outer structure, exposing
     `script_element` and `style_element` top-level nodes.
  2. Each `script_element`'s `raw_text` child is re-parsed with
     `tree-sitter-typescript::LANGUAGE_TYPESCRIPT` in
     `vue_extract_script_recursive` at `dkg/mod.rs:1717-1782`.
  3. CSS `@import` paths inside `style_element` blocks are extracted
     via `extract_css_at_imports`.

This is the canonical composite-reparse pathway. The plan's demanded
`injections.scm` query file is NOT shipped because it would duplicate
this pathway and route through a parallel scm-file query engine, per
DRIFT-10 in `workflow/reasoning/E-42-decisions.md`. The reference
file at `crates/knowledge/src/tree_sitter_health/queries/injections.scm`
is descriptive only — it names the canonical reconciliation but
should not be read as a tree-sitter Query::new() input.

Composite languages: one-level-of-injection ceiling
---------------------------------------------------

tree-sitter does not solve multi-level injection per
github.com/tree-sitter/tree-sitter/issues/3625. ELAI only ever
re-parses ONE LEVEL DEEP: outer HTML/Svelte/Vue → inner TS/CSS.
Nested HTML → Vue → template-expression → TS is NOT attempted.

MDX is NOT TIER-A
-----------------

MDX (Markdown JSX) is experimental and its embedded-language routing
is incomplete. We do not register `tree-sitter-mdx` and do not
advertise MDX support to users. Markdown registration when it lands
will be CommonMark + GFM only.

Large-file ceiling
------------------

`elai_knowledge::tree_sitter_health::large_file::TREE_SITTER_LARGE_FILE_BYTES = 2 * 1024 * 1024`.
Source files above 2 MiB skip tree-sitter and chunk as raw text. A
`TreeSitterGrammarEventKind::TreeSitterSkippedLargeFile` SSB row is
emitted when `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1`. Reference:
github.com/tree-sitter/tree-sitter/issues/1277 (1.6 MB JSON parse
~1.2 s on commodity hardware).

WASM grammar gate (NOT SUPPORTED)
---------------------------------

tree-sitter's WASM runtime cannot run on V8 today
(github.com/tree-sitter/tree-sitter/issues/4715, verified May 2026).
Wasmtime is the only fully-supported path. ELAI does NOT load
grammars in a Tauri webview hot-path. If a future path attempts
this, the substrate must add a `tree_sitter_wasm_unsupported`
arm to `TreeSitterGrammarEventKind` and SCHEMA bump again.

Banned grammar crates
---------------------

  tree-sitter-language-pack -- bundles all grammars under one
    umbrella crate. Ships ABIs that may not match the workspace's
    tree-sitter 0.26 pin. Carries MIT/Apache license mismatch. A
    future "simplify dep graph" pass might be tempted to swap the
    workspace's 18 individual tree-sitter-* crates for this one;
    the contract test `banned_crates_include_language_pack_and_dead_forks`
    blocks it.

  tree-sitter-kotlin (fwcd fork) -- abandoned. Replaced by
    tree-sitter-kotlin-ng per C-17. Anyone re-introducing it
    (e.g. via a copy-paste from an older Cargo.toml) fails the
    contract test `kotlin_uses_org_ng_variant_not_fwcd`.

  tree-sitter-svelte (plain crate) -- abandoned. Replaced by
    tree-sitter-svelte-ng. Contract test
    `svelte_uses_ng_variant_not_abandoned` blocks regression.

Wire-dark env gate
------------------

The E-42 telemetry substrate ships wire-dark. The
`TreeSitterGrammarEvent` SSB variant + payload + Kind discriminator
are unconditionally compiled, but the producers
(abi_probe / svelte_health / large_file / registration) only EMIT
rows when `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1` is set at the
producer's call site. Default deployments observe zero rows from
this substrate. Mirrors the precedent set by
`ELAI_PEER_SYSTEMS_ENABLE` / `ELAI_LOCAL_CLOUD_ROUTING_ENABLE` /
`ELAI_DOCUMENT_VISION_RAG_ENABLE`.

See also
--------

  - workflow/reasoning/E-42-decisions.md -- full per-task closure
    rationale + Tier-A → Tier-B transition log
  - workflow/audits/E-42-grammar-versions-2026-05-28.md -- per-grammar
    cargo-tree inventory + ABI report
  - crates/contracts/tests/grammar_tier_contract.rs -- 9 locked
    invariants
