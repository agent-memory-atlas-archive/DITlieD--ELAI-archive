# Tree-sitter grammars + parsing for ELAI (May 2026)

Scope: state of tree-sitter 0.25/0.26, per-grammar maturity for B-30's 13+ language target, alternatives (oxc, libcst, ts-morph, ANTLR, SCIP indexers), composite-language (Svelte/Vue/Astro/RSC) reality. Web-only research; no local ELAI source read.

## TL;DR
- Pin core at `tree-sitter 0.26.x` (current 0.26.8, March 2026). Stay on ABI 15 (introduced 0.25.0). B-30 should NOT chase pre-release 0.26.x bumps; bug-fix-only stream, no breaking changes to wire shape. (tree-sitter releases.)
- Ship grammars in two tiers: TIER-A (vendor as crates, treat as production) = rust, python, typescript, tsx, javascript, go, java, c, cpp, c-sharp, ruby, php, bash, html, css, json. TIER-B (defer or wrap in stub fallback) = kotlin (fork ambiguity fwcd vs official), scala (Scala 3 brace-less bugs open 2025), swift (community grammar, no Apple endorsement), svelte (Svelte 5 runes not formally supported by any tree-sitter-svelte tag), vue (Vue 3 SFC parser stack fragmented across `ikatyang`, `tree-sitter-grammars/tree-sitter-vue`, `tree-sitter-vue3`).
- For ELAI's symbol-graph use case, do NOT replace tree-sitter wholesale. Reach for `oxc` ONLY if a per-file TypeScript/TSX/JSX accuracy or type-aware analysis bottleneck shows up; `oxc_parser` + `oxc_ast` are now production-ready and 3x faster than swc, 5x faster than Biome. For Python deep refactors, `libcst` (Python 3.0->3.14, Instagram-maintained) beats tree-sitter's lossy AST on whitespace/comment fidelity. For TS workspace-wide type queries, `ts-morph` 28.x is the canonical wrapper.
- Composite-language strategy already in ELAI (Svelte/Vue parsed as HTML, `<script>` re-parsed as TS) matches industry practice (tree-sitter language injection via `injections.scm`). It is fragile post-Svelte 5 runes / Vue 3 `<script setup>` / Astro `---` frontmatter — flag known unparseable regions rather than asserting "all langs supported."
- Use `kreuzberg-dev/tree-sitter-language-pack` (formerly grantjenks `tree-sitter-languages`, now 306 langs, MIT/Apache only, releases through May 2026) as the *reference catalogue* — but ELAI should still vendor specific crates, not import the meta-pack, because the pack ships Python wheels not Rust crates.

## Key sources
| URL | What | Takeaway |
|---|---|---|
| https://github.com/tree-sitter/tree-sitter/releases | tree-sitter core releases | 0.26.8 March 2026 latest; 0.26.x is bug-fix stream over 0.25.0's ABI 15 bump |
| https://github.com/tree-sitter/tree-sitter/issues/5013 | 0.26.x release checklist | confirms 0.26 line is patch-level, no ABI bump from 0.25 |
| https://github.com/zed-industries/zed/issues/24632 | Zed ABI 15 support | ABI 15 added language name, version, supertype info, reserved words |
| https://github.com/tree-sitter/tree-sitter-scala/issues/466 | Scala 3 brace-less coloring bug | Scala 3 still rough as of April 2025 in Metals/Zed pipeline |
| https://github.com/tree-sitter/tree-sitter-scala/releases | tree-sitter-scala v0.24 June 2025 | Active but lagging Scala 3 syntax |
| https://github.com/fwcd/tree-sitter-kotlin | fwcd Kotlin fork | Community fork (140 stars, v0.3.7) — competes with official `tree-sitter/kotlin-tree-sitter` bindings repo (these are bindings, not the grammar) |
| https://github.com/Himujjal/tree-sitter-svelte | Himujjal svelte | Original Svelte grammar, NOT Svelte 5 runes ready |
| https://github.com/tree-sitter-grammars/tree-sitter-svelte | tree-sitter-grammars svelte | Org-adopted variant; tagged 1.x in 2024, runes coverage unclear |
| https://github.com/xiaoxin-sky/tree-sitter-vue3 | Vue3 fork | Created because upstream vue grammar not updated for Vue 3 |
| https://github.com/tree-sitter-grammars/tree-sitter-vue | tsg Vue | Org-adopted Vue grammar |
| https://github.com/ast-grep/ast-grep | ast-grep | Rust-native structural search; uses tree-sitter parsers under the hood; CST not AST |
| https://github.com/oxc-project/oxc | oxc | `oxc_parser` + `oxc_ast` production-ready; 3x swc, 5x Biome; used by Rolldown, Nuxt, Preact, Shopify, ByteDance, Shopee |
| https://oxc.rs/docs/guide/usage/parser.html | oxc parser docs | Test262 100%, Babel 99%, TS 99%; native Rust no tree-sitter |
| https://github.com/Instagram/LibCST | LibCST | Python 3.0->3.14 lossless CST; sits on parso; canonical Python codemod tool |
| https://github.com/dsherret/ts-morph | ts-morph 28.x | TypeScript Compiler API wrapper for static analysis; April-May 2026 release |
| https://github.com/sourcegraph/scip | SCIP | indexers: scip-java, scip-typescript, scip-clang, scip-ruby, scip-python, scip-dotnet, scip-dart, scip-php, rust-analyzer (Rust) — only Reprolang uses tree-sitter; production SCIP indexers use language-native compilers |
| https://sourcegraph.com/blog/how-cody-understands-your-codebase | Cody parser stack | Cody uses tree-sitter for chunking/completion shape detection, NOT for semantic analysis; semantic done via SCIP precise indexers |
| https://github.com/orgs/sheeptechnologies/discussions/4 | RFC: drop SCIP for tree-sitter | community pushback on SCIP — tree-sitter queries argued as enabling file-incremental indexing without build tools |
| https://github.com/kreuzberg-dev/tree-sitter-language-pack | language-pack | 306 langs, polyglot bindings, MIT/Apache only, May 2026 active |
| https://github.com/tree-sitter/tree-sitter-cpp/issues/341 | C++20 modules | C++20 keywords merged but tree-sitter-cpp not re-released — known stale |
| https://github.com/tree-sitter/tree-sitter/issues/1277 | large-file perf | 1.6MB JSON: 1.2s full parse / 0.7s incremental; soft cap above ~1MB |
| https://github.com/tree-sitter/tree-sitter/issues/4715 | wasm-c-api | tree-sitter WASM runtime can't use V8 yet; Wasmtime preferred for perf |
| https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html | query syntax | S-expr query language stable (captures, predicates, fields, quantifiers, anchors) but injection cross-language story remains a known sore spot |
| https://github.com/tree-sitter/tree-sitter/issues/3625 | injection proposal | Multi-language node injection is the unsolved problem — Svelte/Vue/Astro pain |
| https://blog.jez.io/tree-sitter-limitations/ | tree-sitter limits | independent critique of tree-sitter for compiler-grade work |

(28 sources; 12+ web calls completed.)

## Grammar maturity table (B-30 target list)
Legend: known-good = parses representative samples without ERROR; fork-graveyard = HIGH (3+ competing forks), MED (2 forks), LOW (1 canonical).

| Grammar | Repo | Last release (approx) | Known-good | Fork-graveyard | Source |
|---|---|---|---|---|---|
| rust | tree-sitter/tree-sitter-rust | 0.24.x 2025 | yes | LOW | crates.io/crates/tree-sitter-rust |
| python | tree-sitter/tree-sitter-python | 0.25 line, active | yes (Py 3.12 reliable, 3.13 match-stmt edge cases) | LOW | github.com/tree-sitter/tree-sitter-python |
| typescript | tree-sitter/tree-sitter-typescript | 0.23.x 2025 | mostly (decorators stage 3 spotty) | LOW | tree-sitter/tree-sitter-typescript |
| tsx | tree-sitter/tree-sitter-typescript (tsx variant) | same as TS | yes | LOW | same |
| javascript | tree-sitter/tree-sitter-javascript | 0.23.x 2025 | yes | LOW | same |
| go | tree-sitter/tree-sitter-go | 0.23.x 2025 | yes (generics OK, type-set rare bugs) | LOW | github.com/tree-sitter/tree-sitter-go |
| java | tree-sitter/tree-sitter-java | 0.23.x 2025 | yes (sealed classes OK) | LOW | github.com/tree-sitter/tree-sitter-java |
| kotlin | fwcd/tree-sitter-kotlin OR tree-sitter/kotlin-tree-sitter | fwcd 0.3.7 / org bindings 2025-01 | partial (KMP coroutines syntax fragile) | HIGH (fwcd, fwcd-fork at nvim-treesitter, org bindings — bindings are not the grammar) | fwcd + tree-sitter/kotlin-tree-sitter |
| scala | tree-sitter/tree-sitter-scala | 0.24.0 June 2025 | NO for Scala 3 brace-less syntax | MED | tree-sitter/tree-sitter-scala (issue #466) |
| c | tree-sitter/tree-sitter-c | 0.24.x 2025 | yes (C23 partial) | LOW | tree-sitter/tree-sitter-c |
| cpp | tree-sitter/tree-sitter-cpp | stale — C++20 merged not released | partial (C++20 modules not in release) | LOW | tree-sitter-cpp #341 |
| c-sharp | tree-sitter/tree-sitter-c-sharp | 0.23.5 April 2026 | yes (C# 12 records OK; C# 13 unknown) | LOW | pypi tree-sitter-c-sharp |
| ruby | tree-sitter/tree-sitter-ruby | active 2025-01-30 | yes | LOW | tree-sitter parser list |
| php | tree-sitter/tree-sitter-php | active 2025 | yes (PHP 8.3 OK) | LOW | tree-sitter/tree-sitter-php |
| swift | alex-pinkus/tree-sitter-swift (community); no Apple endorsement | 2025 commits | partial (macro syntax, result builders fragile) | MED | swiftpackageindex/tree-sitter/tree-sitter |
| bash | tree-sitter/tree-sitter-bash | active 2025 | yes (POSIX OK; bash-isms OK; zsh-isms NO) | LOW | tree-sitter parser list |
| svelte | tree-sitter-grammars/tree-sitter-svelte (org) + Himujjal/tree-sitter-svelte (original) + tree-sitter-svelte-next | Sept 2024 | NO for Svelte 5 runes ($state, $derived, $effect, $props) without source patch | HIGH | github + npm package pages |
| vue | tree-sitter-grammars/tree-sitter-vue + ikatyang/tree-sitter-vue (Vue 2.6 only) + xiaoxin-sky/tree-sitter-vue3 | varies | partial; Vue 3 `<script setup>` parse OK on org variant, NOT on ikatyang | HIGH | tree-sitter-vue repos |
| html | tree-sitter/tree-sitter-html | 0.23.x 2025 | yes | LOW | tree-sitter/tree-sitter-html |
| css | tree-sitter-grammars/tree-sitter-css | 0.23.x 2025 | yes | LOW | tree-sitter-grammars/tree-sitter-css |
| json | tree-sitter/tree-sitter-json | stable | yes; 1MB+ files slow | LOW | tree-sitter/tree-sitter-json + issue 1277 |
| yaml | tree-sitter-grammars/tree-sitter-yaml | 0.7.x | yes (anchors/aliases OK) | LOW | tree-sitter-grammars |
| toml | tree-sitter-grammars/tree-sitter-toml | active | yes | LOW | tree-sitter-grammars |
| markdown | tree-sitter-grammars/tree-sitter-markdown | 0.5.2 docs.rs | yes (CommonMark + GFM); MDX NO | LOW | docs.rs/tree-sitter-md |
| zig | tree-sitter-grammars/tree-sitter-zig | active | partial (Zig 0.13+ async syntax) | LOW | tree-sitter-grammars |
| elixir | elixir-lang/tree-sitter-elixir | active | yes | LOW | tree-sitter parser list |
| elm | elm-tooling/tree-sitter-elm | 2024 last | yes but Elm itself frozen | LOW | tree-sitter parser list |
| haskell | tree-sitter/tree-sitter-haskell | 0.21.x | partial (extensions soup) | MED | tree-sitter parser list |
| ocaml | tree-sitter/tree-sitter-ocaml | active | yes | LOW | tree-sitter parser list |
| erlang | WhatsApp/tree-sitter-erlang | active 2025 (WhatsApp maintained) | yes | LOW | github.com/WhatsApp/tree-sitter-erlang |
| fortran | stadelmanma/tree-sitter-fortran | sporadic | partial | LOW | tree-sitter parser list |
| r | r-lib/tree-sitter-r | active | yes | LOW | tree-sitter parser list |

Source for status column: github.com/tree-sitter/tree-sitter/wiki/List-of-parsers (community maintained), kreuzberg-dev/tree-sitter-language-pack readme, and each grammar's GitHub releases tab.

## Composite-language parse strategy (Svelte/Vue/Astro/RSC)
Tree-sitter's official solution is **language injection** via `injections.scm` queries. Parent grammar (HTML or Svelte/Vue dialect) leaves `<script>` body as raw_text or an injection-capture node; consumer is responsible for invoking the JS/TS parser on those byte ranges (tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html; tree-sitter-html issue #103).

This is precisely ELAI's current approach per the report intro ("composite languages parsed as HTML, `<script>` blocks re-parsed as TypeScript, `<style>` blocks for CSS @import"). It is correct and industry-standard. Known fragility 2025-2026:

- **Svelte 5 runes** (`$state`, `$derived`, `$effect`, `$props`) are JS expressions inside Svelte syntactic positions the grammar didn't expect. tree-sitter-svelte and tree-sitter-svelte-next both predate Svelte 5 stable (Oct 2024). No upstream tagged "Svelte 5 ready" grammar exists as of May 2026. ELAI should detect Svelte 5 runes and either bypass to raw text chunking OR fork-patch the grammar. (npmjs.com/package/@tree-sitter-grammars/tree-sitter-svelte; openreplay.com Svelte summer 2025 recap.)
- **Vue 3 `<script setup>`** including `defineProps`, `defineEmits`, `defineExpose` macros: org variant (`tree-sitter-grammars/tree-sitter-vue`) handles SFC structure; macro CALLS are valid TS so the TS injection captures them, but cross-block reference resolution (template referencing setup-scope identifiers) is invisible to tree-sitter and requires Volar (the Vue language server, TS-based) for semantic links.
- **Astro `.astro` files**: frontmatter delimited by `---` lines, code-fence-like, with TS body then HTML/JSX-with-curlies template. No officially endorsed tree-sitter-astro grammar in `tree-sitter/` or `tree-sitter-grammars/` orgs. virchau13/tree-sitter-astro exists but unfunded. ELAI should treat .astro as: frontmatter -> TS parser, body -> tree-sitter-html with embedded JS injections.
- **React Server Components (RSC)**: NOT a syntactic distinction. RSC files are still .tsx/.jsx — the server/client boundary is the `"use server"` / `"use client"` directive string literal at top of file. Tree-sitter-typescript handles them natively. No grammar work needed; semantic understanding requires Next/Vite metadata. (alvarlagerlof/rsc-parser handles only the over-the-wire serialised RSC payload format, not source.)
- **Multi-language injection still has open proposal-stage problems** (tree-sitter issue #3625): nested injections (HTML > Vue template > template expression > TS), region splitting across injection sites, and re-entrancy are unsolved. Production code-intel tools either constrain to one level of injection or fall back to language-server-per-file.

## Tree-sitter vs alternatives
**Stay on tree-sitter for:**
- Symbol-graph chunking, DEF/CALL extraction, syntax-shape detection, language-agnostic AST traversal across 20+ langs.
- Error-recovery tolerance: tree-sitter generates partial trees on broken code (LLM-generated half-files included). GLR + ERROR/MISSING nodes mean the indexer never crashes on syntactically invalid input. (tree-sitter discussions #1205, #1870.)

**Reach for `oxc` instead when:**
- Per-file TS/TSX/JSX accuracy matters and you want full type-aware analysis later. `oxc_parser` is production-ready (oxc.rs), 3x faster than swc, supports Stage 3 decorators, 100% Test262, 99% Babel/TS tests. Used by Rolldown, Nuxt, Preact, Shopify, ByteDance, Shopee. If ELAI grows a "fix this TS file" pipeline, oxc is the right substrate; tree-sitter-typescript is for indexing not transformation.
- You need a linter-grade pass: Oxlint v1.0 (August 2025 stable, infoq) ships 500+ rules at 50-100x ESLint speed.

**Reach for `libcst` instead when:**
- Doing Python codemods that must preserve whitespace, comments, paren placement, trailing commas — i.e. anything that round-trips source. Tree-sitter-python emits a lossy AST; libcst is a true lossless CST. (github.com/Instagram/LibCST, Python 3.0->3.14, sits on parso.)

**Reach for `ts-morph` instead when:**
- You need workspace-wide TS type queries: "what implements this interface", "all call sites of X with their resolved types", "is this property used after refactor". ts-morph 28.x wraps the official TypeScript compiler API; tree-sitter has no type info at all. (github.com/dsherret/ts-morph, npm releases May 2026.)

**Reach for SCIP indexers (scip-java, scip-typescript, scip-clang, scip-python, rust-analyzer) when:**
- You want PRECISE cross-file definition/reference resolution. tree-sitter-based heuristic resolvers (ELAI's JVM suffix map, TS path aliases) are 80% solutions; SCIP indexers consume the actual compiler/language-server output and emit symbol IDs with full canonical paths. ELAI already references SCIP indices for rust-analyzer / scip-python / scip-typescript output in the elai.map — that's the right hybrid. (sourcegraph/scip, sourcegraph/scip-python.)

**Reach for `ast-grep` for:**
- Structural search/refactor at the CLI level by humans. Same tree-sitter parsers under the hood; not a replacement for ELAI's indexing layer but a complementary tool for one-off refactor scripts. (github.com/ast-grep/ast-grep.)

**Skip ANTLR:** Grammar coverage exists (mature) but runtime perf, error recovery, and incremental parsing are worse than tree-sitter for an indexing workload. ANTLR shines for DSL authors and compiler implementers, not code-intel.

## Best B-30 sequence
Ranked by ROI (impact * inverse-risk):

1. **rust, python, typescript, tsx, javascript, go, java, c, html, css, json, bash** — TIER-A grammars. Land all in one cargo `[dependencies]` add. Per-grammar `std::panic::catch_unwind` (already planned) covers the residual risk. Grounding: each grammar 0.23+ stable, LOW fork-graveyard.
2. **c-sharp, ruby, php** — TIER-A, second wave. Same maturity but smaller user surface in ELAI's likely use cases. Grounding: April 2026 c-sharp release, active 2025 ruby/php.
3. **yaml, toml, markdown** — TIER-A configuration langs. Useful for project-config indexing (Cargo.toml, tsconfig.json, package.json). Markdown CommonMark+GFM only; MDX rejected.
4. **svelte (org variant only), vue (org variant only)** — TIER-B with explicit "Svelte 5 runes parse may produce ERROR nodes" caveat surfaced to user. Use injection queries for `<script>` -> TS, `<style>` -> CSS. Grounding: org variants in `tree-sitter-grammars/` are the only viable forks; fwcd-style multi-fork grammars (kotlin) should be wrapped behind a fork-selector if shipped.
5. **kotlin (pick fwcd/tree-sitter-kotlin as the GRAMMAR; the org's `kotlin-tree-sitter` is BINDINGS, not a grammar — easy to confuse)** — TIER-B. Document the pick in `.elai_cc/crates/.../grammars.md` so future maintainers don't swap to the bindings repo.
6. **swift, scala** — TIER-B, last priority. Swift macros and Scala 3 brace-less syntax are known broken; ship a min-viable parse with a feature flag `experimental_grammars = ["swift", "scala"]`.
7. **DEFER: cpp until upstream tree-sitter-cpp re-releases with C++20 modules merged** (issue #341 open since 2024). Use older 0.23.x release in the meantime with a known-stale note.
8. **DEFER: astro, mdx, .glimmer, .nunjucks, .liquid** — not in B-30 13+ list, not worth the injection complexity right now.

Anti-pattern guard: do NOT import `tree-sitter-language-pack` as a meta-dependency. It's Python-wheel-shaped. Vendor specific Rust crates per grammar, control versions, control ABI alignment.

## Anti-patterns
- **Stale C++ grammar pretending to support C++20**: tree-sitter-cpp merged C++20 keyword support but did NOT cut a release; users importing 0.23.x get pre-C++20 grammar. Document the version pin, do not advertise "C++20 supported." (tree-sitter-cpp issue #341.)
- **Pinning ABI 14 while pulling new grammars**: ABI 15 (0.25.0) added language name, version, supertype info, reserved words. Pulling new grammars built against ABI 15 into a core pinned at <0.25.0 will fail at parser load. ELAI is on 0.26.x; safe — but contracts must enforce no downgrade. (zed-industries/zed #24632, doomemacs/doomemacs #8503.)
- **Multi-level injection chains (HTML > Svelte template > expression > TS)** assuming round-tripping works: tree-sitter issue #3625 documents this as unsolved. Limit ELAI to at most 1 level of injection (Svelte/Vue/HTML -> JS/TS/CSS). Beyond that, fall back to per-block raw chunking.
- **Treating tree-sitter as a precise indexer**: Cody itself documents using tree-sitter for *chunking and completion shape*, not symbol resolution. ELAI should not promise "find all references" semantics from tree-sitter alone; that's an SCIP indexer's job. (sourcegraph.com/blog/how-cody-understands-your-codebase.)
- **Forks-as-truth for kotlin**: `tree-sitter/kotlin-tree-sitter` looks authoritative by org but is BINDINGS only. The grammar lives at `fwcd/tree-sitter-kotlin`. Easy to misread; pin the URL in the build.rs.
- **Vue grammar selection**: `ikatyang/tree-sitter-vue` is Vue 2.6 ONLY; `tree-sitter-grammars/tree-sitter-vue` is the org-adopted version. Vue 3 SFC users on the ikatyang grammar silently get ERROR nodes on `<script setup>`. (github.com/ikatyang/tree-sitter-vue README.)
- **Assuming WASM tree-sitter perf matches native**: WASM runtime perf is acceptable but `tree-sitter` cannot use V8 as its wasm runtime today; Wasmtime is the only fully-supported path (issue #4715). If ELAI ever runs grammars in Tauri webview, expect 1.5-2x slower than native; tolerable for offline indexing not for hot-path completion.
- **Reading 1MB+ JSON / minified files through tree-sitter**: known O(n) memory blowup, 1.2s parse for 1.6MB JSON (issue #1277, #3473). ELAI's existing minified detection (avg_line_length > 500) is the right guard; keep it.

## Open questions
1. Does ELAI's current ABI version (declared in `tree-sitter` 0.26 crate's bindgen) match every vendored grammar's ABI claim? Measurable: add a startup probe that calls `ts_language_version()` on each loaded language and panics if < 14 or > 15.
2. What's the actual ERROR-node rate on real-world Svelte 5 / Vue 3 / Astro source from the GitHub trending sample? Measurable: 100-file sample per framework, count ERROR / MISSING nodes per 1k LoC.
3. Should we adopt `oxc` for TS/TSX as a primary parser and use tree-sitter only for non-JS langs? Measurable: A/B perf benchmark of indexing one large monorepo (vite source tree, ~200K LoC TS) with tree-sitter-typescript vs `oxc_parser` walking the same DEF/CALL extraction.
4. Is the cross-file resolution in JVM suffix map / TS path aliases / Go modules accurate enough that ELAI doesn't need SCIP for those languages? Measurable: precision/recall vs scip-java + scip-typescript ground truth on a tagged sample (10-20 cross-file refs per lang).
5. When the user upgrades tree-sitter core from 0.26.x to 0.27 (future), can we gate it behind a contract test that all 20+ grammars still load? Measurable: a `cargo test -p contracts grammars_load` that imports and parses one canonical sample per grammar at build time.
