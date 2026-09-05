# B-30: Multi-Language DKG Hardening (closes A-35)
Objective: extend DKG beyond Rust to Python / TypeScript / TSX / JavaScript /
Go / Java / Kotlin / Scala / C / C++ / C# / Ruby / PHP / Swift / Bash / Svelte
/ Vue with per-grammar failure isolation, multi-module resolvers
(JVM suffix map, C# namespace map, Go module info), TS path-alias resolution,
and HTML-then-TS re-parse for Svelte/Vue. Closes A-35 which was on the A-list
but unstarted at audit.
Estimated effort: 3-4 sessions (~18-24 hours).
Pre-requisites: B-1 (path unification), B-29 (symbol graph cache) for
end-to-end multi-lang traversal — but extraction can land independently.
Parallel-safe with: B-31, B-32, B-33, B-34, B-35.

## Context

Per DETAILED_SUMMARY_01:117: "Currently Rust-only; Python/TS/Go/Java/C#
grammars pending (1A-MULTILANG)." A-35 (PI Multi-Language DKG Grammars) was
explicitly listed in the implementation order as ~500-700 LOC across Python,
TypeScript, Go, Java, C# but has not landed. Phase 1A audit metric reports
135 files / 8075 nodes / 4148 edges / 9.67s — these are Rust files only.
On any non-Rust project the DKG contains <module>-only synthetic symbols
and the file-import graph collapses to leaves.

Concrete consequence: Designer's Recursive Decomposition cannot ground a
Python/TS/Go project's existing structure. Researcher's /research returns
hollow context for non-Rust target projects. The B-29 symbol-graph cache
will be empty for any non-Rust file. ELAI today is "the system works for
any language" in spec, "Rust-only" in reality.

SocratiCode demonstrates the pattern works at production quality:
13 dynamic ast-grep grammars loaded with per-grammar try/catch so a single
broken grammar does not crash extraction
(`src/services/code-graph.ts:497-515`), plus dedicated multi-module
resolvers — JVM suffix map for Maven/Gradle (`graph-resolution.ts:23-53`),
C# namespace map for `using` declarations (`graph-resolution.ts:78-119`),
Go module info from `go.mod`, TypeScript path aliases from
`tsconfig.json`/`jsconfig.json` `compilerOptions.paths` with `extends`
chain following (`src/services/graph-aliases.ts`), and Svelte/Vue
HTML-then-TS re-parse (`graph-imports.ts:121-149`). Their CHANGELOG shows
multi-language was added incrementally over many releases — the
per-grammar isolation pattern is what made that scalable.

This B-step ports the patterns to ELAI's existing tree-sitter pipeline.
DO NOT swap engines mid-flight: CLAUDE.md anti-stale rules pin to
tree-sitter, and ELAI's DKG SQLite schema is already designed around
tree-sitter Language IDs. The win is the failure-isolation contract,
the multi-module resolvers, and the composite-language re-parse.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\code-graph.ts
  lines 446-572 (ensureDynamicLanguages — per-grammar isolation pattern)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-resolution.ts
  (FULL — JVM suffix map, C# namespace map, Go module info)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-aliases.ts
  (FULL — tsconfig.json paths + extends chain resolution with cycle prevention)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-imports.ts
  (FULL — per-language import extraction; Svelte/Vue HTML-then-TS re-parse
   at lines 121-149)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-symbols.ts
  (FULL — per-language symbol extraction patterns; mirror to tree-sitter)
- F:\.ELAI_workflow\docs\specs\guide\update-implementation-order-summary.txt
  A-35 (PI Multi-Language DKG Grammars)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\dkg\language\rust.rs
  (existing Rust LanguageGrammar reference impl)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\dkg\engine.rs
  (extraction dispatch site)

## Tasks

### Phase 1 — LanguageGrammar trait stabilization (Session 1, half)

1. Audit existing `LanguageGrammar` trait in
   `crates/knowledge/src/dkg/language/mod.rs`. Confirm it has:
   - `extract_imports(source) -> Vec<ImportInfo>`
   - `extract_symbols(source) -> Vec<DkgNode>`
   - `extract_calls(source) -> Vec<RawCall>`
   If methods are missing or mis-shaped, refactor to match B-29's needs
   (which expects `ExtractedSymbols { symbols, raw_calls }` per file).

2. Add `LanguageRegistry` with per-grammar failure isolation:
   - `register(grammar: Box<dyn LanguageGrammar>) -> Result<(), GrammarLoadError>`
   - Per-grammar try/catch: a panic in one grammar's tree-sitter init
     does NOT abort the rest. Use `std::panic::catch_unwind` around
     `tree_sitter::Parser::set_language` for each.
   - Track `loaded: BTreeSet<String>` and
     `failed: BTreeMap<String, String>` (lang name → error)
   - Expose `LanguageRegistryStatus { loaded: Vec<String>, failed: Vec<(String, String)> }`
     to dashboard consumers.

3. Surface registry status in dashboard `KnowledgeDrawer`: line "Languages
   loaded: rust, python, typescript, go, java, ... (13/15 grammars). Failed:
   php (tree-sitter v15+ required)." Add SSB action_type
   `LanguageGrammarStatus` for one-shot publication at startup.

### Phase 2 — Per-language extractors (Sessions 1-2)

4. Implement `LanguageGrammar` for Python in
   `crates/knowledge/src/dkg/language/python.rs`:
   - Imports: `import_statement` (`import foo`, `import foo.bar`),
     `import_from_statement` (`from foo import bar`)
   - Symbols: `function_definition`, `class_definition`,
     `decorated_definition` (decorators preserved)
   - Methods inside `class_definition` get `qualified_name = "Class.method"`
   - Calls: `call` node with `attribute` resolution for `obj.method()`
   - Mirror SocratiCode's `extractFromPython` patterns
     (`graph-symbols.ts` Python section)

5. Implement TypeScript / TSX / JavaScript in
   `crates/knowledge/src/dkg/language/typescript.rs`:
   - Imports: `import_statement`, `export_statement` (re-exports),
     `call_expression` text-match for `require(...)` and dynamic `import(...)`
   - Symbols: `class_declaration`, `function_declaration`,
     `lexical_declaration` for arrow functions assigned to const,
     `interface_declaration`, `type_alias_declaration`,
     `enum_declaration`, `method_definition` inside classes
   - Differentiate `Lang::JavaScript` / `Lang::TypeScript` / `Lang::Tsx`
     since tree-sitter grammars differ per dialect

6. Implement Go in `crates/knowledge/src/dkg/language/go.rs`:
   - Imports: `import_declaration` (single + grouped)
   - Symbols: `function_declaration`, `method_declaration`,
     `type_declaration` (struct, interface, type alias)
   - Module-level constants in `var_declaration` / `const_declaration` —
     skip unless needed for completeness (PageRank already factors edges)

7. Implement JVM family (Java / Kotlin / Scala) in
   `crates/knowledge/src/dkg/language/jvm.rs` (one module, three Language
   IDs since extractors share ~80% logic):
   - Java: `import_declaration`, `class_declaration`,
     `interface_declaration`, `enum_declaration`, `method_declaration`
   - Kotlin: same plus `object_declaration`, `function_declaration`
   - Scala: `class_definition`, `object_definition`, `trait_definition`,
     `function_definition`

8. Implement C / C++ in
   `crates/knowledge/src/dkg/language/cfamily.rs`:
   - Imports: `preproc_include` (handle both `<system>` and `"local"`)
   - Symbols: `function_definition`, `struct_specifier`, `class_specifier`
     (C++ only), `namespace_definition` (C++ only), `enum_specifier`

9. Implement C# in `crates/knowledge/src/dkg/language/csharp.rs`:
   - Imports: `using_directive`
   - Symbols: `class_declaration`, `interface_declaration`,
     `method_declaration`, `namespace_declaration` (block + file-scoped
     `namespace X.Y.Z;` C# 10+)

10. Implement Ruby / PHP / Swift / Bash in language-specific files:
    - Ruby: `class`, `module`, `method`, `singleton_method`
    - PHP: `function_definition`, `class_declaration`,
      `method_declaration`, `trait_declaration`, `namespace_definition`
    - Swift: `function_declaration`, `class_declaration`,
      `struct_declaration`, `protocol_declaration`,
      `extension_declaration`
    - Bash: `function_definition` only

### Phase 3 — Multi-module resolvers (Session 2-3)

11. JVM suffix map in
    `crates/knowledge/src/dkg/resolution/jvm.rs`:
    - On graph build, scan all `.java/.kt/.kts/.scala` files
    - For each file path containing `/src/main/<lang>/`, register
      `key = "com/example/Foo.java"` (everything after the boundary) →
      `value = "module-a/sub/src/main/java/com/example/Foo.java"`
    - First match wins (avoid duplicate-class ambiguity)
    - Lookup integrated into `resolve_import` for JVM imports
    - Mirror `buildJvmSuffixMap` from
      SocratiCode's `graph-resolution.ts:23-53`

12. C# namespace map in
    `crates/knowledge/src/dkg/resolution/csharp.rs`:
    - Regex parse all `.cs` files for `namespace X.Y.Z` declarations
      (block + file-scoped C# 10+ form `namespace X.Y.Z;`)
    - Build `BTreeMap<String, Vec<String>>` mapping namespace → files
      that contribute to it
    - Files processed in lexicographic order so candidate lists are
      deterministic across machines (CRITICAL: regression test must
      assert byte-identical output across runs)
    - `using X.Y.Z;` resolves to candidate file list
    - Mirror `buildCsNamespaceMap` from
      SocratiCode's `graph-resolution.ts:78-119`

13. Go module info in
    `crates/knowledge/src/dkg/resolution/go.rs`:
    - Parse `go.mod` if present at project root
    - Extract `module path/here` directive — gives the canonical import
      prefix
    - When `import "path/here/foo/bar"` is seen, strip prefix and resolve
      `foo/bar` relative to project root
    - If no `go.mod`, log warn and treat all imports as external (current
      behavior)

14. TypeScript path aliases in
    `crates/knowledge/src/dkg/resolution/tsconfig.rs`:
    - Read `tsconfig.json` and `jsconfig.json` (prefer the latter for JS
      projects)
    - Parse `compilerOptions.paths` (e.g. `"@/*": ["src/*"]`,
      `"$lib/*": ["src/lib/*"]`)
    - Follow `extends` chain (referenced `tsconfig.base.json` etc.) with
      cycle prevention via visited set
    - Resolved aliases stored as `Map<String, Vec<PathBuf>>` for
      `resolve_import` to consult before fallback file-system search
    - Mirror SocratiCode's
      `graph-aliases.ts` `loadPathAliases` algorithm

### Phase 4 — Composite-language re-parse (Session 3)

15. Svelte/Vue extractor in
    `crates/knowledge/src/dkg/language/svelte_vue.rs`:
    - Use HTML tree-sitter grammar to find all `script_element` nodes
    - Extract their `raw_text` (the script body)
    - Re-parse each as TypeScript (default — superset of JS)
    - Apply existing TypeScript extractor to the re-parsed AST
    - Symbols qualified: `<file>.svelte::<symbol>` to disambiguate from
      `<file>.ts::<symbol>` if siblings exist
    - Mirror SocratiCode's
      `graph-imports.ts:121-149` Svelte/Vue branch

16. CSS `@import` extraction from `<style>` blocks in same module:
    - HTML tree-sitter finds `style_element` nodes
    - Regex extract `@import "..."` and `@require "..."` from raw text
    - Skip `http:`/`https:` URLs
    - Emit as edges with `is_css_import=true` flag

### Phase 5 — Wire and verify (Session 3-4)

17. Register all 13+ grammars in `LanguageRegistry::default()` at startup.
    Order: rust, python, typescript, javascript, tsx, go, java, kotlin,
    scala, c, cpp, csharp, ruby, php, swift, bash, svelte, vue. Per-grammar
    isolation means a missing tree-sitter v15 grammar (e.g. PHP often
    lags) won't break the others.

18. Update `dkg_db_path` schema to record `language` per node (probably
    already done — verify). Add migration v4 if not.

19. Update Phase 1A metric file
    `.elai_cc/metrics/phase_1A.toml`: add per-language node/edge counts
    and unresolved import percent. Re-run measurement on a multi-lang
    fixture (e.g. checked-in fixture project with Rust + Python + TS).

20. Update `DkgEngine::extract_symbols` and
    `DkgEngine::resolve_imports` dispatch to call the right
    `LanguageGrammar` based on file extension. Use `LanguageRegistry`
    lookup so unloaded grammars fall through to a no-op extractor that
    still creates a `<module>` synthetic symbol.

21. Test on real codebases:
    - SocratiCode itself (TypeScript only) — sanity check the TS extractor
    - A Python test project (e.g. `requests` library) — symbol resolution
    - A Java multi-module project (e.g. open-source Maven layout) — JVM
      suffix map verification
    - A C# project with file-scoped namespaces — namespace map
    - A Svelte project (Tauri dashboard itself uses Svelte 5) — composite
      re-parse

## WIRING_CONTRACT

produces_symbols:
  - knowledge::dkg::language_registry::LanguageRegistry::register
  - knowledge::dkg::PythonGrammar
  - knowledge::dkg::TypeScriptGrammar
  - knowledge::dkg::TsxGrammar
  - knowledge::dkg::JavaScriptGrammar
  - knowledge::dkg::GoGrammar
  - knowledge::dkg::JavaGrammar
  - knowledge::dkg::KotlinGrammar
  - knowledge::dkg::ScalaGrammar
  - knowledge::dkg::CGrammar
  - knowledge::dkg::CppGrammar
  - knowledge::dkg::CSharpGrammar
  - knowledge::dkg::RubyGrammar
  - knowledge::dkg::PhpGrammar
  - knowledge::dkg::SwiftGrammar
  - knowledge::dkg::BashGrammar
  - knowledge::dkg::SvelteGrammar
  - knowledge::dkg::VueGrammar
  - knowledge::dkg::resolution::jvm::JvmResolver
  - knowledge::dkg::resolution::csharp::CsNamespaceResolver
  - knowledge::dkg::resolution::go_module::GoModuleResolver
  - knowledge::dkg::resolution::tsconfig::TsPathAliasResolver
  - orchestrator::ssb::action_type::SsbActionType::LanguageGrammarStatus
consumes_symbols:
  - orchestrator::indexing_service::IndexingService::build_index → uses
    LanguageRegistry to dispatch extraction per file
  - knowledge::dkg::engine::resolve_import → consults JVM suffix map / CS
    namespace map / Go module info / TS aliases
  - dashboard::commands::get_language_status → reads SSB
    LanguageGrammarStatus rows
reachable_from:
  - cli::main::CliAction::IndexBuild
  - dashboard::lib::run (project indexing)
  - orchestrator::indexing_service::IndexingService::build_index
stub_ban:
  - LanguageRegistry.loaded.len() >= 10 on a healthy build (out of 13+
    registered)
  - LanguageRegistry.failed has explicit error per failed grammar (no
    silent fall-through)
  - phase_1A.toml has per_language_node_counts table populated
frontend_pair: B-29 (symbol graph cache consumes per-language extractors)
observable_effect:
  trigger: "F:\\.ELAI_workflow\\.elai_cc\\target\\release\\elai.exe index build --project-root F:\\.ELAI_workflow\\.elai_cc"
  expect_ssb: language_grammar_status
  expect_artifact: phase_1A.toml shows nodes>0 for at least 5 languages

## DETERMINISTIC_CHECK

```bash
# 1. All grammar modules compile
cargo check -p knowledge --tests
# Expected: zero errors.

# 2. LanguageRegistry registers >=13 grammars at startup
cargo test -p knowledge --test language_registry test_default_registry_loads_all
# Expected: passing.

# 3. Per-language extraction smoke test on bundled fixtures
cargo test -p knowledge --test multilang_extraction
# Expected: each language test asserts symbol_count > 0 and matches
#           expected qualified-name set.

# 4. JVM suffix map deterministic across runs
cargo test -p knowledge --test jvm_resolver test_deterministic_first_match
# Expected: 100 runs produce byte-identical maps.

# 5. C# namespace map deterministic across runs
cargo test -p knowledge --test cs_resolver test_lexicographic_ordering
# Expected: candidate lists in lexicographic order.

# 6. End-to-end on multi-lang fixture
elai.bat index docs/fixtures/multilang_project
sqlite3 docs/fixtures/multilang_project/.elai_cc/indexes/dkg.db \
  "SELECT language, COUNT(*) FROM dkg_nodes GROUP BY language ORDER BY 2 DESC;"
# Expected: at least 5 languages with non-zero counts.

# 7. Per-grammar isolation verified
# Manually corrupt one grammar's loader (e.g. wrong tree-sitter version),
# rerun index. Expected: 12/13 grammars load, that one fails, others
# unaffected.
```

## ACCEPTANCE

- [ ] LanguageRegistry implemented with per-grammar try/catch isolation
- [ ] Python / TS / TSX / JS / Go / Java / Kotlin / Scala / C / C++ / C# /
      Ruby / PHP / Swift / Bash / Svelte / Vue extractors implemented
- [ ] JVM suffix map / C# namespace map / Go module info / TS path
      aliases all consulted by resolve_import in correct order
- [ ] Svelte/Vue HTML-then-TS re-parse working with composite-language
      qualified names
- [ ] phase_1A.toml updated with per-language metrics on multilang fixture
- [ ] Dashboard KnowledgeDrawer displays loaded/failed grammar list
- [ ] LanguageGrammarStatus SSB action emitted at startup
- [ ] B-29 symbol graph cache builds non-empty for all supported languages
- [ ] Zero `.unwrap()` in non-test source per CLAUDE.md hard rule
- [ ] No new external crates without explicit user confirmation (use
      existing `tree-sitter` workspace deps)

## Tests to add

- `crates/knowledge/tests/multilang_extraction.rs` — per-language fixture
  asserts symbol+import extraction (one fixture file per language with
  expected node/edge set)
- `crates/knowledge/tests/language_registry.rs` — default registry loads
  expected set, per-grammar isolation simulated by injecting failing
  grammar
- `crates/knowledge/tests/jvm_resolver.rs` — multi-module project
  fixture, deterministic ordering, FQN resolution accuracy
- `crates/knowledge/tests/cs_resolver.rs` — block + file-scoped namespace
  fixtures, lexicographic ordering proof
- `crates/knowledge/tests/go_resolver.rs` — go.mod parsing, prefix
  stripping
- `crates/knowledge/tests/tsconfig_aliases.rs` — paths + extends chain +
  cycle prevention
- `crates/knowledge/tests/svelte_extraction.rs` — composite-language
  re-parse, qualified names disambiguate
- `tests/contracts/multilang_dkg_contract.rs` — regression: phase_1A
  metric per-language counts non-empty

## Files touched

- CREATE: `crates/knowledge/src/dkg/language/python.rs`
- CREATE: `crates/knowledge/src/dkg/language/typescript.rs`
- CREATE: `crates/knowledge/src/dkg/language/go.rs`
- CREATE: `crates/knowledge/src/dkg/language/jvm.rs`
- CREATE: `crates/knowledge/src/dkg/language/cfamily.rs`
- CREATE: `crates/knowledge/src/dkg/language/csharp.rs`
- CREATE: `crates/knowledge/src/dkg/language/ruby.rs`
- CREATE: `crates/knowledge/src/dkg/language/php.rs`
- CREATE: `crates/knowledge/src/dkg/language/swift.rs`
- CREATE: `crates/knowledge/src/dkg/language/bash.rs`
- CREATE: `crates/knowledge/src/dkg/language/svelte_vue.rs`
- CREATE: `crates/knowledge/src/dkg/resolution/jvm.rs`
- CREATE: `crates/knowledge/src/dkg/resolution/csharp.rs`
- CREATE: `crates/knowledge/src/dkg/resolution/go.rs`
- CREATE: `crates/knowledge/src/dkg/resolution/tsconfig.rs`
- CREATE: `crates/knowledge/src/dkg/language/registry.rs`
- MODIFY: `crates/knowledge/src/dkg/language/mod.rs` (export new modules,
  trait stabilization)
- MODIFY: `crates/knowledge/src/dkg/engine.rs` (dispatch via
  LanguageRegistry)
- MODIFY: `crates/core/src/ssb/action_types.rs` (LanguageGrammarStatus
  variant)
- MODIFY: `crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte`
  (V2 path — was V1 `crates/dashboard/svelte/src/drawers/`; grammar
  status panel)
- CREATE: `docs/fixtures/multilang_project/` (multi-language test fixture
  per language)
- CREATE: `workflow/reasoning/B-30-decisions.md`

## Decision doc

Write `workflow/reasoning/B-30-decisions.md` documenting:
- Why ELAI keeps tree-sitter despite SocratiCode using ast-grep (CLAUDE.md
  anti-stale + existing investment)
- Per-grammar isolation strategy: catch_unwind vs Result-only — choice
  depends on whether tree-sitter loader can panic in safe mode (verify
  empirically; default to catch_unwind for safety)
- TypeScript path-alias resolution order: project tsconfig → jsconfig →
  workspace root → fallback file-system search. Document precisely
- Composite-language qualified-name format: `<file>.svelte::<symbol>` vs
  `<file>::<symbol>` — pick one and document. Affects B-29 indexing
- Whether Dart, Lua, R should be added now (regex-only fallback) or
  deferred to a follow-up B-step

## Side benefits unlocked

- B-29 (symbol graph) populates for Python/TS/Go/Java/etc., not just Rust
- Designer's existing-project mode (TechStackDecision::ExistingProject)
  works on non-Rust target codebases — finally a global system
- /research command's project-context grounding works for non-Rust users
- Dashboard's Project Intelligence drawer (when wired) shows accurate
  language breakdown
- Phase 1A metric becomes a more honest representation of indexing
  capability across real-world repos
- Empire integration (B-25) gains Java/Python interop since Empire's
  brains are mixed-language

## Next

B-29 (Symbol-Level Impact API) — should land first or in parallel; B-30
provides the per-language extractors B-29 expects.
B-31 (Resumable Indexing) — independent, parallel-safe.
B-33 (Interactive Viewer) — benefits from multi-lang DKG for graph node
  language coloring.
