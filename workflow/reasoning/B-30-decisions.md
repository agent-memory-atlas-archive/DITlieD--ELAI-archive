B-30 Phase 1 Decision Document
Session: s-21092
Date: 2026-05-16
Pass: A (standalone registry + SSB + dashboard)

DECISION 1 — Why ELAI keeps tree-sitter (not ast-grep)
-------------------------------------------------------
CLAUDE.md anti-stale rule explicitly pins tree-sitter for Rust code. The existing
DKG engine (dkg/mod.rs) already has 9 grammar implementations against the
tree_sitter::Language API. Swapping to ast-grep (@ast-grep/napi) would require
rewriting the extraction pipeline, changing the DKG SQLite schema (tree-sitter node
cursors produce different traversal APIs than ast-grep SgNode), and introducing a
Node.js FFI boundary into a Rust-native crate. The SocratiCode explorer (explorer-
socraticode.md) explicitly notes that tree-sitter and ast-grep use the SAME underlying
grammar files and the AST node kinds are IDENTICAL — so the pattern ports directly
without switching engines. No evidence of ast-grep providing a correctness advantage
for ELAI's extract_nodes use case (cursor walk returning DkgNode structs). Decision:
keep tree-sitter, port the per-grammar isolation pattern.

DECISION 2 — Per-grammar isolation: catch_unwind + AssertUnwindSafe
--------------------------------------------------------------------
Problem: tree_sitter::Parser is not UnwindSafe (it holds raw pointers to C grammar
state). Standard catch_unwind requires UnwindSafe bounds; passing Parser directly
would fail to compile.
Solution: wrap the entire probe closure in AssertUnwindSafe(|| { ... }).
Rationale: The catch is LOCAL — if the closure panics, we record the failure and
discard the parser. We never observe the Parser's internal state after an unwind, so
the UnwindSafe guarantee is not semantically violated. AssertUnwindSafe is the
canonical Rust pattern for "I know this is safe even though the type is not tagged".
Grounded in: std::panic::AssertUnwindSafe docs + SocratiCode's per-grammar try/catch
(each grammar is initialized inside its own try block before joining survivors).
Alternative considered: wrapping each grammar crate's language() call separately (only
catch_unwind around language() not set_language). Rejected: set_language can also
panic on ABI mismatch in some grammar crate versions (grammar calls tree-sitter
internal functions that panicked in pre-ts-language shim versions). Wrapping both is
safer at negligible overhead — grammars are registered once at startup.

DECISION 3 — TypeScript path-alias resolution order
-----------------------------------------------------
Per explorer-socraticode.md PATTERN_TSCONFIG_PATH_ALIASES and SocratiCode's
graph-aliases.ts implementation:
1. Try `tsconfig.json` in the project root.
2. If not found or has no `paths`, try `jsconfig.json` in the project root.
3. Follow the `extends` chain from whichever config was found (up to MAX_EXTENDS_DEPTH=10).
4. Within the chain, `paths` in the child config take priority over the parent (short-
   circuit: stop following extends if `paths` is non-empty at the current level).
5. Cycle prevention: track visited absolute paths in a BTreeSet; break on first revisit.
6. Package references in `extends` (e.g. `@tsconfig/node20/tsconfig.json`) are resolved
   via the workspace's `node_modules/`.
7. Fallback: if neither config exists or the chain has no `paths`, fall back to
   file-system import search (extension probing: .ts, .tsx, .js, .jsx, index.ts, etc.).
Determinism: `paths` entries are stored in BTreeMap (insertion order of Object.entries
is JS-engine-specific; BTreeMap guarantees alphabetical for ELAI's Rust port).
Maximum extends depth: 10 (same constant as SocratiCode, prevents cycle runaway even
if BTreeSet cycle detection misses a mutual-extends pair across symlinks).

DECISION 4 — Composite-language qualified-name format
------------------------------------------------------
Convention: `<file_path>::<symbol>` where `<file_path>` is the project-relative path
including the file extension (e.g. `src/lib/Drawer.svelte::MyComponent`).
This matches the existing SvelteGrammar implementation in dkg/mod.rs:1454 which
uses file-path-based naming via DkgNode parent/file linkage and the DkgNode struct
fields (name, kind, file_path, parent_idx). The file_path for Svelte files already
includes the `.svelte` extension — no special escaping or alternate format is needed.
Alternative considered: `<file>.svelte::<symbol>` (duplicate extension in the key).
Rejected: the existing Rust grammar produces `<file>.rs::<symbol>` without special
treatment; consistency requires the same pattern. A special case for Svelte would
add complexity with no benefit. Decision: use the existing convention for all grammars.

DECISION 5 — Dart, Lua, R deferred to follow-up B-step
--------------------------------------------------------
DEFERRED. Rationale:
- tree-sitter-dart: no stable 0.26-compatible Rust crate confirmed on crates.io as of
  2026-05-16. The explorer-treesitter.md LANGUAGE_TABLE does not list dart — no version
  confirmed. Adding an unverified crate violates the anti-slopsquatting rule.
- tree-sitter-lua: similar status — not in the explorer-treesitter verified list. Lua
  usage in ELAI target projects is minimal; not in the Phase 1 priority set.
- tree-sitter-r: no production-quality Rust crate with ABI-verified tree-sitter-language
  ^0.1 dependency exists. R is a low-priority language for code-graph extraction.
Path forward: confirm crate names and ABI compatibility via `cargo info` for each, get
user confirmation per CLAUDE.md "no new external crates without human validation", then
add to knowledge/Cargo.toml and implement grammar structs in a follow-up B-step.

CROSS-SESSION COORDINATION NOTE
---------------------------------
B-30 Pass A splits work because session s-22200 (B-29) holds locks on:
  crates/knowledge/src/dkg/mod.rs  — contains the 9 grammar impls + DkgEngine
  crates/knowledge/src/lib.rs      — contains LanguageGrammar trait definition
  crates/knowledge/src/symbol_graph/**

Consequences for this pass:
1. language_registry.rs is created as a standalone file in dkg/ but NOT wired via
   `pub mod language_registry;` in lib.rs (that line must be added by Pass A.2).
2. The integration test file (tests/language_registry.rs) is gated behind
   `#[cfg(all(test, feature = "__b30_registry_enabled__"))]` so it compiles to a no-op
   without the mod wiring.
3. The Tauri command get_language_grammar_status is registered in invoke_handler! but
   will only return the empty-status fallback until the LanguageRegistry emits real SSB
   rows (which requires DkgEngine wiring in dkg/mod.rs, also gated behind B-29).
4. The KnowledgeDrawer LANGUAGES LOADED card shows UnwiredNote (languageStatus: true)
   until the emitter lands in Pass A.2.

Pass A.2 checklist (after B-29 releases locks):
  [ ] Add `pub mod language_registry;` to crates/knowledge/src/dkg/mod.rs
  [ ] Add `pub use dkg::language_registry::{LanguageRegistry, LanguageRegistryStatus,
      GrammarLoadError};` to crates/knowledge/src/lib.rs
  [ ] Remove `#[cfg(all(test, feature = "__b30_registry_enabled__"))]` gate from tests/
  [ ] Wire DkgEngine::new() to use LanguageRegistry instead of the inline grammars map
  [ ] Emit LanguageGrammarStatus SSB row in DkgEngine after registration cycle
  [ ] Set `unwired.languageStatus: false` in drawers.svelte.ts once emitter is live
