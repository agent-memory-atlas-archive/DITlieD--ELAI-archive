# E-42: Tree-sitter pin-and-tier (TIER-A vs TIER-B) + Svelte 5 runes injection patch + composite-language guardrails
Objective: pin tree-sitter core to `0.26.x` (ABI 15) with a startup probe that asserts every loaded grammar's `ts_language_version()` is in {14, 15}, split the 17+ B-30 target grammars into TIER-A production (15 langs) vs TIER-B experimental (2-5 langs, feature-gated), wire injection queries for composite languages (Svelte 5 runes, Vue 3 SFC, Astro frontmatter) with explicit ERROR-node telemetry, and document the canonical fork-selector for ambiguous grammars (kotlin: `fwcd/tree-sitter-kotlin` is the GRAMMAR; `tree-sitter/kotlin-tree-sitter` is BINDINGS only).
Estimated effort: 3 sessions, ~10-12 h
Pre-requisites: B-30 PLANNED (Multi-Language DKG Hardening — closes A-35), B-1 SHIPPED (path unification), B-29 IN-FLIGHT (chunk-level tree-sitter, per B-PLAN-SUMMARY:553)
Parallel-safe with: E-41 (CI), E-43 (skill systems), E-44 (durability), E-45 (drift gate) — none touch `crates/knowledge/src/dkg/`
Source report: workflow/improvement-sweep-2026-05-19/reports/42-tree-sitter-grammars-2026.md

## Context

Tree-sitter core is at `0.26.8` as of March 2026 (github.com/tree-sitter/tree-sitter/releases). The 0.26.x line is bug-fix-only over `0.25.0`'s ABI 15 bump (issue #5013); pulling new grammars built against ABI 15 into a core pinned at <0.25 fails at parser load (zed-industries/zed#24632). ELAI's `crates/knowledge/src/dkg/mod.rs` already imports `tree_sitter::Tree` and defines per-language `tree_sitter_language()` shims for at least 4-6 langs (per elai.map hits at mod.rs lines 1248, 1277, 1400, 1560, 1611-1615). B-30 plans 13+ grammars; E-42 is the pin-and-tier refinement that makes that target shippable without grammar drift bricking the dkg.

The report's grammar maturity table partitions 30+ tree-sitter grammars by (a) fork-graveyard risk (LOW / MED / HIGH) and (b) last-release recency. TIER-A (LOW fork-graveyard, active 2025 releases, syntactically stable): rust, python, typescript, tsx, javascript, go, java, c, c-sharp, ruby, php, bash, html, css, json, yaml, toml, markdown. TIER-B (MED-HIGH fork-graveyard OR known partial syntax): kotlin, scala, swift, svelte, vue, c++ (stale C++20), zig, haskell. Sticking the experimental tier behind a `experimental_grammars` Cargo feature with explicit "may emit ERROR nodes" telemetry is the right ship discipline.

Composite-language parsing (Svelte/Vue/Astro/RSC) is unsolved at the multi-level injection layer (tree-sitter issue #3625). ELAI's existing approach — parse outer as HTML / Svelte / Vue dialect, re-parse `<script>` body as TS — is the industry standard (tree-sitter.github.io 3-syntax-highlighting + tree-sitter-html issue #103). The fragility is Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`) which no upstream tree-sitter-svelte tag formally supports as of May 2026 (npmjs.com/package/@tree-sitter-grammars/tree-sitter-svelte + openreplay.com Svelte summer 2025 recap). E-42 ships a runes detection + ERROR-node-rate telemetry path; it does NOT promise full runes parsing.

The report explicitly warns against `tree-sitter-language-pack` as a meta-dependency — it ships Python wheels, not Rust crates. Vendor individual Rust grammar crates per language with pinned versions in `Cargo.toml`. The kotlin fork confusion (`fwcd/tree-sitter-kotlin` GRAMMAR vs `tree-sitter/kotlin-tree-sitter` BINDINGS) is the documented trap and must be pinned in a `grammars.md` reference doc in the knowledge crate.

## Existing state (B/C/D coverage)

- B-30 PLANNED (Multi-Language DKG Hardening, closes A-35), per B-PLAN-SUMMARY lines 606-618: 13+ tree-sitter grammars with per-grammar try/catch isolation; Svelte/Vue composite-language re-parse (HTML tree-sitter finds `<script>` then re-invokes TS parser). EXTENSION: E-42 is the pin-and-tier refinement that grounds B-30's grammar list with the May 2026 maturity table, adds the ABI version probe, and adds the Svelte 5 runes injection path. B-30 still ships the integration; E-42 supplies the version pins, tier split, and injection.scm files.
- B-29 IN-FLIGHT (chunk-level tree-sitter partial parse every ~50 tokens, per B-PLAN-SUMMARY:553): E-42 does not modify the chunking layer; it stabilises the underlying grammar set the chunker relies on.
- B-31 / B-31.5 (per B-PLAN-SUMMARY pre-req graph): grammar tier changes do not cascade to these.
- Existing dkg.rs surface at `crates/knowledge/src/dkg/mod.rs:1248-2347` defines `tree_sitter_language()` for at least 4-6 langs (rust, python, typescript, javascript, java, c, cpp, php, per elai.map hits). EXTENSION: E-42 adds 7-10 more TIER-A languages behind a single `add_languages()` registration site, and gates TIER-B behind a Cargo feature flag.
- C-Plan: no current C-N owns grammars. E-42 is the gating spec for B-30 execution.
- `.claude/tools/elai.map` regen script (codebase_mapper.py) consumes the same grammar set indirectly; ABI bump or grammar swap must not break the mapper. Verify by re-running `python .claude/tools/codebase_mapper.py .elai_cc elai_graph.json --compact --map elai.map` post-change.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\42-tree-sitter-grammars-2026.md` (full)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 606-618 (B-30 scope), line 553 (B-29 chunker)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\dkg\mod.rs` lines 1200-1620 (existing `tree_sitter_language()` shims — verify which langs are wired before adding to TIER-A)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\Cargo.toml` (current tree-sitter-* dep set)
- `F:\.ELAI_workflow\.claude\tools\codebase_mapper.py` (regen script — confirm it doesn't depend on a specific grammar that E-42 might tier-down)
- WebFetch: github.com/tree-sitter/tree-sitter/releases (confirm 0.26.8 March 2026)
- WebFetch: github.com/tree-sitter/tree-sitter/issues/5013 (0.26 line bug-fix-only)
- WebFetch: github.com/tree-sitter/tree-sitter/issues/3625 (multi-level injection unsolved)
- WebFetch: github.com/tree-sitter-grammars/tree-sitter-svelte (confirm release tags + Svelte 5 status)
- WebFetch: github.com/fwcd/tree-sitter-kotlin (confirm canonical grammar URL, NOT bindings)
- WebFetch: github.com/tree-sitter/tree-sitter-cpp/issues/341 (C++20 modules merged-but-unreleased)

## Tasks

1. Pin tree-sitter core to `0.26` with patch-floor. In `crates/knowledge/Cargo.toml` (and any other crate importing tree-sitter), set `tree-sitter = "=0.26"` (exact-pin pattern) and document the rationale in a `crates/knowledge/src/grammars.md` reference file. Verify via `cargo tree -p knowledge | rg tree-sitter` that no transitive bumps occur. Reference: tree-sitter/tree-sitter#5013.

2. ABI version probe at startup. Add `crates/knowledge/src/dkg/abi_probe.rs` (NEW) with `fn assert_abi_compatibility(langs: &[(&str, Language)]) -> Result<()>` that calls `language.version()` (or the 0.26 equivalent — `ts_language_version` C ABI) for each loaded grammar and panics with an actionable error if version ∉ {14, 15}. Wire this into `DkgEngine::build_graph` or wherever grammars are registered. SSB row: `grammar_abi_probe_result` with per-grammar version. Cited: zed-industries/zed#24632 (ABI 15 added language name, version, supertype info, reserved words).

3. TIER-A grammar registration. Add the canonical 15 TIER-A grammars (rust, python, typescript, tsx, javascript, go, java, c, c-sharp, ruby, php, bash, html, css, json, yaml, toml, markdown) to a single registration function `register_tier_a_languages() -> Vec<(&'static str, tree_sitter::Language)>`. Each grammar is a separate Cargo dep (e.g., `tree-sitter-rust = "0.24"`) — verify each crate exists and is the canonical maintainer per the report's table 1 (LOW fork-graveyard). Anti-slopsquatting: confirm crate name, version, last-published date via `cargo search <name>` before adding. Markdown is CommonMark + GFM only — do NOT advertise MDX support.

4. TIER-B grammar registration (feature-gated). Behind Cargo feature `experimental_grammars`, register kotlin, scala, swift, svelte, vue (and optionally cpp until upstream re-releases per #341). Each TIER-B grammar emits a `grammar_tier_b_loaded` SSB row at startup with a one-line "may emit ERROR nodes" caveat. Kotlin fork-selector pin: use `tree-sitter-kotlin` crate from `fwcd/tree-sitter-kotlin` (the GRAMMAR), NOT bindings from `tree-sitter/kotlin-tree-sitter`. Document the pin in `grammars.md` with a literal-URL anchor so future maintainers don't swap.

5. Svelte 5 runes detection + ERROR-node-rate telemetry. Add a post-parse pass over `.svelte` files that walks the tree-sitter tree, counts ERROR / MISSING nodes, and emits `svelte_runes_parse_health` SSB row with `{file, error_node_count, missing_node_count, runes_detected}`. Runes detection is a regex pre-check on `\$state\(|\$derived\(|\$effect\(|\$props\(` (cheap, no parser). If runes are detected AND ERROR rate > 5%, downgrade to raw-text chunking for that file and emit a warning. Cited: tree-sitter-svelte upstream lag (no Svelte 5 ready tag as of May 2026), tree-sitter issue #3625 (multi-level injection unsolved).

6. Composite-language injection scheme. Add `injections.scm` queries for `tree-sitter-html`, `tree-sitter-svelte`, `tree-sitter-vue` that capture `<script>` body as `injection.language = "typescript"` and `<style>` body as `injection.language = "css"`. Honor tree-sitter's one-level-of-injection ceiling — do NOT attempt nested HTML > Vue > template-expression > TS. Reference: tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html + tree-sitter-html issue #103.

7. Astro frontmatter + RSC handling. `.astro` files: frontmatter (between `---` lines) → TS parser; body → tree-sitter-html with embedded JS injections. No officially endorsed `tree-sitter-astro` per report; document the split-parse path in `grammars.md` and gate it behind `experimental_grammars` since virchau13/tree-sitter-astro is unfunded. RSC (`"use server"` / `"use client"` directive): NOT a syntactic distinction — tree-sitter-typescript already handles. Add a test asserting RSC files parse cleanly with TIER-A grammar set.

8. Large-file + minified-file guard. tree-sitter is O(n) memory + ~1.2s for 1.6MB JSON (tree-sitter issue #1277). ELAI's existing minified detection (`avg_line_length > 500`) is the right guard per the report — confirm it lives at `crates/knowledge/src/dkg/mod.rs` or similar, and add a hard ceiling of 2MB file-size beyond which the file is chunked as raw text (no tree-sitter pass). Emit `tree_sitter_skipped_large_file` SSB row. Reference: tree-sitter issues #1277, #3473.

9. WASM grammar gate. Per tree-sitter issue #4715, the WASM runtime cannot use V8 today; Wasmtime is the only fully-supported path. If ELAI ever runs grammars in the Tauri webview (dashboard-v2 hot-path completion), expect 1.5-2x slower than native — tolerable for offline indexing, not for real-time. Document this in `grammars.md` and add a `tree_sitter_wasm_unsupported` SSB row guard if a webview path attempts grammar load.

10. Regenerate elai.map after grammar set change. Run `python .claude/tools/codebase_mapper.py .elai_cc elai_graph.json --compact --map elai.map --scip-dir .claude/tools/scip_indices` post-change to confirm the mapper still produces the same row count (±5%) for langs that were already on TIER-A; verify no regression in SCIP_REF / SCIP_DEF rows.

## WIRING_CONTRACT

produces_symbols:
  - knowledge::dkg::abi_probe::assert_abi_compatibility(langs: &[(&str, tree_sitter::Language)]) -> Result<()>
  - knowledge::dkg::abi_probe::AbiVersion { lang: String, version: u16 }
  - knowledge::dkg::register_tier_a_languages() -> Vec<(&'static str, tree_sitter::Language)>
  - knowledge::dkg::register_tier_b_languages() -> Vec<(&'static str, tree_sitter::Language)> (cfg-gated: experimental_grammars)
  - knowledge::dkg::svelte_health::detect_runes(source: &str) -> bool
  - knowledge::dkg::svelte_health::compute_health(tree: &tree_sitter::Tree) -> ErrorHealth { error: usize, missing: usize }
  - orchestrator::ssb::SsbActionType::GrammarAbiProbeResult
  - orchestrator::ssb::SsbActionType::GrammarTierBLoaded
  - orchestrator::ssb::SsbActionType::SvelteRunesParseHealth
  - orchestrator::ssb::SsbActionType::TreeSitterSkippedLargeFile
consumes_symbols:
  - tree_sitter = "=0.26" (pinned dep)
  - tree-sitter-rust, tree-sitter-python, tree-sitter-typescript, tree-sitter-javascript, tree-sitter-go, tree-sitter-java, tree-sitter-c, tree-sitter-c-sharp, tree-sitter-ruby, tree-sitter-php, tree-sitter-bash, tree-sitter-html, tree-sitter-css, tree-sitter-json, tree-sitter-yaml, tree-sitter-toml, tree-sitter-md (TIER-A)
  - tree-sitter-kotlin (fwcd fork), tree-sitter-scala, tree-sitter-swift, tree-sitter-svelte (org variant), tree-sitter-vue (org variant) — TIER-B, cfg-gated
  - .claude/tools/codebase_mapper.py (regen post-change)
reachable_from:
  - crates/knowledge/src/dkg/mod.rs::DkgEngine::build_graph (registers all TIER-A; TIER-B if feature enabled)
  - crates/cli/src/indexing.rs (per elai.map indexing hits) — flows through DkgEngine
stub_ban:
  - no `tree-sitter-language-pack` import (HARD RULE — per report; vendor crates individually)
  - no `tree-sitter` version range > "0.26" or < "0.26"
  - no `.unwrap()` on `ts_language_version()` calls (use Result/Option)
  - no TIER-B grammar registered without the `experimental_grammars` cfg gate
  - no Svelte 5 "supported" advertising in user-facing strings — only "best-effort with health telemetry"
  - no MDX in markdown registration (CommonMark + GFM only per report)
frontend_pair: crates/dashboard-v2/src/lib/windows/GrammarHealthPanel.svelte (NEW, optional — shows per-grammar ERROR-rate over last N parses; gated behind B-26 dashboard completion)
observable_effect:
  trigger: "elai index --root .elai_cc" on a tree containing one .rs, one .py, one .ts, one .svelte (with Svelte 5 runes), one large .json (>2MB)
  expect_ssb: "grammar_abi_probe_result" once per loaded grammar at startup; "svelte_runes_parse_health" with runes_detected=true; "tree_sitter_skipped_large_file" for the big JSON
  expect_artifact: elai_graph.json contains DEF rows for the .rs/.py/.ts file symbols; no rows for the >2MB JSON; per-grammar version recorded in workflow/audits/E-42-grammar-versions-2026-05-19.md

## DETERMINISTIC_CHECK

```bash
# 1. tree-sitter pinned exactly
rg '^tree-sitter\s*=\s*"=0\.26"' .elai_cc/crates/knowledge/Cargo.toml
# Expected: 1 hit (or equivalent exact-pin form)

# 2. ABI probe module present and called
rg "fn assert_abi_compatibility" .elai_cc/crates/knowledge/src/dkg --type rust
rg "assert_abi_compatibility\(" .elai_cc/crates/knowledge/src/dkg/mod.rs
# Expected: 1 def + ≥1 call

# 3. TIER-A registration function exists
rg "fn register_tier_a_languages" .elai_cc/crates/knowledge/src/dkg --type rust
# Expected: 1 hit

# 4. TIER-B is cfg-gated
rg '#\[cfg\(feature\s*=\s*"experimental_grammars"' .elai_cc/crates/knowledge/src/dkg --type rust
# Expected: ≥1 hit (gates register_tier_b_languages or the call site)

# 5. No language-pack meta-dep
! rg "tree-sitter-language-pack|tree_sitter_language_pack" .elai_cc/crates/knowledge --type toml
# Expected: zero hits

# 6. No MDX advertising
! rg "MDX|mdx" .elai_cc/crates/knowledge/src/dkg/mod.rs
# Expected: zero hits in body (test file exceptions OK)

# 7. New SSB action types registered
rg "GrammarAbiProbeResult|GrammarTierBLoaded|SvelteRunesParseHealth|TreeSitterSkippedLargeFile" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: ≥4 hits

# 8. Kotlin fork-selector pinned in dep
rg 'tree-sitter-kotlin.*fwcd|package\s*=\s*"tree-sitter-kotlin"' .elai_cc/crates/knowledge/Cargo.toml
# Expected: ≥1 hit (or git URL pinned to fwcd)

# 9. Grammar versions audit artifact present
test -f workflow/audits/E-42-grammar-versions-2026-05-19.md
# Expected: file exists with one row per registered grammar
```

## ACCEPTANCE

- [ ] `cargo build -p knowledge` succeeds without `experimental_grammars` feature; only TIER-A grammars loaded
- [ ] `cargo build -p knowledge --features experimental_grammars` succeeds; TIER-B grammars also loaded
- [ ] Startup probe emits `grammar_abi_probe_result` for every loaded grammar with `version ∈ {14, 15}`; mismatched grammar fails fast with actionable error
- [ ] Indexing a `.svelte` file with Svelte 5 runes emits `svelte_runes_parse_health` with `runes_detected=true`; if ERROR rate >5%, raw-text fallback is used and a warning is logged
- [ ] Indexing a >2MB JSON file emits `tree_sitter_skipped_large_file` and produces no tree-sitter rows for it (raw-text chunk only)
- [ ] `grammars.md` reference doc exists at `crates/knowledge/src/grammars.md` documenting: tree-sitter core pin rationale, per-grammar version pin, kotlin fork-selector, Svelte 5 / Vue 3 / Astro caveats, WASM unsupported note
- [ ] `python .claude/tools/codebase_mapper.py` regen produces an elai.map with ±5% row count for previously-supported languages (no regression)
- [ ] No `tree-sitter-language-pack` import anywhere in the workspace
- [ ] No `.unwrap()` in `crates/knowledge/src/dkg/abi_probe.rs` (production code)
- [ ] `workflow/audits/E-42-grammar-versions-2026-05-19.md` records one row per grammar: name / crate / version / ABI / tier / fork-source URL

## Tests to add

- `crates/knowledge/tests/abi_probe_passes_on_tier_a.rs`: every TIER-A grammar reports ABI ∈ {14,15}
- `crates/knowledge/tests/abi_probe_rejects_invalid.rs`: synthetic grammar with ABI=10 is rejected with the right error
- `crates/knowledge/tests/svelte_runes_detect.rs`: detects `$state(`, `$derived(`, `$effect(`, `$props(`; doesn't false-positive on Svelte 4 stores
- `crates/knowledge/tests/svelte_health_fallback.rs`: synthetic .svelte with >5% ERROR nodes triggers raw-text fallback path
- `crates/knowledge/tests/large_file_skip.rs`: 3MB JSON fixture skipped; SSB row emitted
- `crates/knowledge/tests/composite_html_injection.rs`: `<script>` body in HTML re-parsed as TS via injection query
- `crates/knowledge/tests/no_mdx_in_markdown.rs`: MDX content parses as plain markdown without crash (we don't claim support)
- `tests/contracts/grammar_tier_contract.rs`: TIER-A registration count ≥15; TIER-B registration count ≥2; both gates compile
- `tests/contracts/grammar_fork_pin_contract.rs`: kotlin crate URL is the fwcd fork (not the bindings repo)

## Files touched

- CREATE: `.elai_cc/crates/knowledge/src/dkg/abi_probe.rs` (~80 LOC — ABI version check + error type)
- CREATE: `.elai_cc/crates/knowledge/src/dkg/svelte_health.rs` (~120 LOC — runes detector + ERROR-rate walk)
- CREATE: `.elai_cc/crates/knowledge/src/grammars.md` (~120 LOC — reference doc for pins, forks, caveats)
- MODIFY: `.elai_cc/crates/knowledge/src/dkg/mod.rs` (+~80 LOC — register_tier_a_languages, register_tier_b_languages, abi_probe call, large-file guard hook)
- MODIFY: `.elai_cc/crates/knowledge/Cargo.toml` (+~30 LOC — pin tree-sitter to "=0.26", add TIER-A grammar deps, gate TIER-B behind `experimental_grammars` feature)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~40 LOC for 4 new action types — bump SCHEMA_VERSION per memory project_b42_shipped.md trajectory)
- CREATE: `.elai_cc/crates/knowledge/src/dkg/queries/injections.scm` (~60 LOC — html → script→ts, style→css; svelte/vue variants)
- CREATE: `workflow/audits/E-42-grammar-versions-2026-05-19.md`
- CREATE: `workflow/reasoning/E-42-decisions.md`
- OPTIONAL: `.elai_cc/crates/dashboard-v2/src/lib/windows/GrammarHealthPanel.svelte` (~100 LOC — gated behind B-26)

## Citations / Grounding

- github.com/tree-sitter/tree-sitter/releases (0.26.8 March 2026) — grounds Task 1 pin
- github.com/tree-sitter/tree-sitter/issues/5013 (0.26 bug-fix-only over 0.25 ABI 15) — grounds Task 1 rationale
- github.com/zed-industries/zed/issues/24632 (ABI 15 added language name, version, supertype info, reserved words) — grounds Task 2 probe
- github.com/tree-sitter/tree-sitter/issues/3625 (multi-level injection unsolved) — grounds Task 5 + Task 6 one-level ceiling
- github.com/tree-sitter/tree-sitter-cpp/issues/341 (C++20 modules merged-but-unreleased) — grounds C++ TIER-B defer
- github.com/fwcd/tree-sitter-kotlin (canonical GRAMMAR) — grounds Task 4 fork-selector
- github.com/tree-sitter-grammars/tree-sitter-svelte (org variant) — grounds Task 5 Svelte path
- github.com/tree-sitter/tree-sitter/issues/1277 (large-file 1.6MB JSON 1.2s parse) — grounds Task 8
- github.com/tree-sitter/tree-sitter/issues/4715 (WASM can't use V8) — grounds Task 9
- tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html — grounds Task 6 injection.scm
- B-30 PLANNED (B-PLAN-SUMMARY:606-618) — grounds the multi-language target

## Open questions / risks

- ABI version range: the report says ABI 15 was introduced in 0.25; older grammar releases may still report ABI 14. Probe accepts {14, 15} per the report's "pin core at 0.26.x, stay on ABI 15" guidance, but if a grammar reports ABI 13 the probe fails — confirm at startup which grammars in the chosen pin set still claim 14.
- Kotlin coverage: KMP coroutines syntax is "fragile" per the table. The `fwcd` fork is the right pick BUT may still ERROR on some idioms. Document explicitly that kotlin is TIER-B.
- Svelte 5 runes: no upstream-tagged grammar exists as of May 2026. The detection + fallback path is honest but not a parse fix; if a user requests "full Svelte 5 support", route to a fork-patch task (out of scope for E-42).
- Vue ikatyang trap: `ikatyang/tree-sitter-vue` is Vue 2.6 ONLY; org variant is required for Vue 3 `<script setup>`. Hard pin in Cargo.toml and contract test must assert the URL.
- Astro: virchau13/tree-sitter-astro is unfunded. The split-parse fallback (frontmatter → TS, body → HTML) is the right architecture but adds complexity; gated behind `experimental_grammars` is the safe path.
- Re-running codebase_mapper after grammar set change may shift elai.map by more than 5% if a previously-missing grammar now contributes; threshold may need tuning per language added.
- Tree-sitter is fundamentally heuristic for cross-file refs — the report cites Cody as using tree-sitter for chunking + completion shape, NOT semantic analysis. ELAI's SCIP-enriched elai.map already does this hybrid; E-42 should not regress it.

## Next

E-42 is the foundation for B-30 execution: once the tier split + ABI probe + Svelte5 health telemetry are in place, B-30 ships the multi-language DKG extractor work on a stable substrate. C-N follow-ons that depend on richer language coverage (cross-language refactor tasks, multi-language symbol unification) should consume the TIER-A list as their language matrix. If/when an `oxc` substitution is considered for TS/TSX (report Open Question 3 — A/B perf), it should land as a separate E-N — do not bundle into E-42's scope. Astro/MDX/glimmer/nunjucks/liquid grammars are explicitly deferred per the report; revisit in 12+ months.
