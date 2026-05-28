;; E-42 Tier-B -- composite-language injection reference (DESCRIPTIVE).
;;
;; This .scm file is NOT loaded as a tree-sitter Query::new() input
;; at runtime. The shipped composite-reparse pattern at
;; `crates/knowledge/src/dkg/mod.rs::vue_extract_script_recursive`
;; (lines 1717-1782) and the `SvelteGrammar` reuse of the TypeScript
;; walker (line 218 comment in dkg/mod.rs) ARE the canonical
;; reconciliations for `<script>` / `<style>` injection. Introducing
;; a parallel scm-file query routing pathway would duplicate this
;; pathway and create a maintenance fork (DRIFT-10 in
;; workflow/reasoning/E-42-decisions.md).
;;
;; This file exists as DOCUMENTATION so future maintainers can see at
;; a glance which node kinds get re-parsed by which downstream grammar.
;; If the workspace ever pivots to a scm-file-driven multi-language
;; query engine (e.g. a Helix-style highlighter), this is the seed.
;;
;; One-level-of-injection ceiling: tree-sitter does not solve
;; multi-level injection (github.com/tree-sitter/tree-sitter/issues/3625
;; verified May 2026). We never recurse a level deeper than one.

;; --------------------------------------------------------------------
;; HTML grammar (used as outer parser for Vue + Svelte composite)
;; --------------------------------------------------------------------
;;
;; tree-sitter-html exposes `script_element` and `style_element` as
;; top-level children of the document. Their `raw_text` children are
;; the embed payloads.

(script_element
  (raw_text) @injection.content
  (#set! injection.language "typescript")
  (#set! injection.combined))

(style_element
  (raw_text) @injection.content
  (#set! injection.language "css")
  (#set! injection.combined))

;; --------------------------------------------------------------------
;; Svelte SFC (tree-sitter-svelte-ng) -- canonical reconciliation
;; --------------------------------------------------------------------
;;
;; The Svelte grammar already exposes `<script>` body nodes that the
;; existing TypeScript walker re-traverses via the `SvelteGrammar`
;; impl. Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`)
;; are detected by `tree_sitter_health::svelte_health::detect_runes`
;; (regex pre-scan over the raw source). When runes are present AND
;; the ERROR / MISSING node ratio exceeds
;; `tree_sitter_health::svelte_health::ERROR_RATE_THRESHOLD = 0.05`,
;; the ingest pipeline downgrades to raw-text chunking and emits a
;; `TreeSitterGrammarEventKind::SvelteRunesParseHealth` SSB row.
;;
;; No `injection.language` rules are declared for svelte here because
;; the SvelteGrammar already routes the TS walker internally; adding
;; scm-driven injections would race the inline routing.

;; --------------------------------------------------------------------
;; Vue SFC (tree-sitter-html outer + secondary TS parser)
;; --------------------------------------------------------------------
;;
;; Vue SFCs are parsed in two passes (per dkg/mod.rs:1700-1740):
;;   1. tree-sitter-html parses the outer structure.
;;   2. Each `script_element`'s `raw_text` child is extracted and
;;      re-parsed with `tree-sitter-typescript::LANGUAGE_TYPESCRIPT`.
;;   3. CSS `@import` paths inside `style_element` blocks are
;;      extracted via `extract_css_at_imports`.
;;
;; This is implemented in Rust (vue_extract_script_recursive) rather
;; than in scm queries because the existing walker pattern interleaves
;; tree-sitter node access with the DKG node emission cleaner than a
;; pure-query pass would. The HTML rules above DOCUMENT the embed
;; structure for reference.

;; --------------------------------------------------------------------
;; Astro frontmatter -- DEFERRED to TIER-C
;; --------------------------------------------------------------------
;;
;; Astro `.astro` files are NOT registered. The frontmatter (between
;; `---` lines) is TS; the body is HTML with embedded JS injections.
;; The `tree-sitter-astro-next` crate is single-maintainer and
;; feature-gated behind `web-astro`; the Tier-C status follows
;; from upstream funding uncertainty per `grammars.md`.

;; END injections.scm reference
