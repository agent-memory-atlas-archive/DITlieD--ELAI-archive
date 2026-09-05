# Verification Report: B-30_multilang_dkg_hardening
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-30_multilang_dkg_hardening.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary

The core multi-language DKG hardening work is implemented but consolidated in
`crates/knowledge/src/dkg/mod.rs` rather than the per-language files the plan
specifies. LanguageRegistry with catch_unwind isolation, all resolution modules
(JVM, C#, Go, TS path aliases), the SSB LanguageGrammarStatus action, and
dashboard wiring are all present. Key gaps: Kotlin has no grammar struct and
`.kt` is absent from `ext_to_lang_id`; the `svelte_extraction.rs` test is
missing; the `docs/fixtures/multilang_project/` fixture tree is absent;
`workflow/reasoning/B-30-decisions.md` was never written; the contract test
lives at a different path than specified; and `phase_1A.toml` per-language
counts are all zero (fixture was Rust-only).

## Claims verified DONE

- LanguageRegistry with per-grammar catch_unwind isolation implemented
  — evidence: crates/knowledge/src/dkg/language_registry.rs:50-60 (GrammarLoadError),
    language_registry.rs:139-193 (register + try_register with catch_unwind)
- GrammarLoadError enum with TreeSitterAbiMismatch / ParserInitPanicked / FactoryPanicked variants
  — evidence: crates/knowledge/src/dkg/language_registry.rs:50 (DEF_ENUM GrammarLoadError)
- LanguageRegistryStatus struct, loaded/failed BTreeMap, status() method
  — evidence: crates/knowledge/src/dkg/language_registry.rs:220 (DEF_FN status),
    language_registry.rs:115-116 (BTreeMap::new for loaded/failed)
- PythonGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:1264 (DEF_STRUCT PythonGrammar),
    mod.rs:1274 (impl LanguageGrammar for PythonGrammar)
- TypeScriptGrammar implemented (covers TS/TSX/JS)
  — evidence: crates/knowledge/src/dkg/mod.rs:1542 (DEF_STRUCT TypeScriptGrammar),
    mod.rs:1557 (impl LanguageGrammar); JavaScriptGrammar at mod.rs:2333
- GoGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:1385 (DEF_STRUCT GoGrammar),
    mod.rs:1397 (impl LanguageGrammar for GoGrammar)
- JavaGrammar implemented (covers Java part of JVM claim)
  — evidence: crates/knowledge/src/dkg/mod.rs:1904 (DEF_STRUCT JavaGrammar),
    mod.rs:1918 (impl LanguageGrammar)
- ScalaGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:2651 (language_id "scala"),
    mod.rs:2660 (impl LanguageGrammar for ScalaGrammar)
- CGrammar and CppGrammar implemented (covers C/C++ claim)
  — evidence: crates/knowledge/src/dkg/mod.rs:2413 (DEF_STRUCT CGrammar),
    mod.rs:2522 (DEF_STRUCT CppGrammar)
- CSharpGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:2040 (DEF_STRUCT CSharpGrammar),
    mod.rs:2056 (impl LanguageGrammar)
- RubyGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:2193 (DEF_STRUCT RubyGrammar)
- PhpGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:2734 (DEF_STRUCT PhpGrammar)
- SwiftGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:2832 (DEF_STRUCT SwiftGrammar)
- BashGrammar implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:2938 (DEF_STRUCT BashGrammar)
- SvelteGrammar implemented (HTML-then-TS re-parse via script_element/raw_text)
  — evidence: crates/knowledge/src/dkg/mod.rs:1606 (DEF_STRUCT SvelteGrammar),
    mod.rs:1739 (SCIP comment: script_element raw_text child re-parse as TypeScript)
- VueGrammar implemented separately (not unified with Svelte as plan specifies)
  — evidence: crates/knowledge/src/dkg/mod.rs:1712 (DEF_STRUCT VueGrammar),
    mod.rs:1714 (impl LanguageGrammar for VueGrammar)
- CSS @import extraction from style blocks implemented
  — evidence: crates/knowledge/src/dkg/mod.rs:1631 (css_at_import_regex),
    tests/css_at_import.rs exists
- JvmResolver (suffix map) implemented
  — evidence: crates/knowledge/src/dkg/resolution/jvm.rs (JvmResolver::new at :151+),
    tests/jvm_resolver.rs, tests/jvm_global_lex.rs
- CsNamespaceResolver implemented with lexicographic determinism
  — evidence: crates/knowledge/src/dkg/resolution/csharp.rs:93 (impl ImportResolver),
    tests/cs_resolver.rs
- GoModuleResolver (go.mod parsing) implemented
  — evidence: crates/knowledge/src/dkg/resolution/go_module.rs (GoModuleResolver::new at :96+),
    tests/go_resolver.rs
- TsPathAliasResolver (tsconfig extends chain + cycle prevention) implemented
  — evidence: crates/knowledge/src/dkg/resolution/tsconfig.rs:90 (DEF_STRUCT TsPathAliasResolver),
    tests/tsconfig_aliases.rs, tests/tsconfig_string_safe.rs
- LanguageGrammarStatus SSB action type defined
  — evidence: crates/orchestrator/src/ssb.rs:807 (LanguageGrammarStatus variant),
    ssb.rs:3446 (LanguageGrammarStatusPayload struct), ssb.rs:3454 (From impl)
- LanguageGrammarStatus emitted at index time (before DkgReady)
  — evidence: crates/cli/src/indexing.rs:54,109,111-120
- Dashboard KnowledgeDrawer displays loaded/failed grammar list
  — evidence: crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte:126-163
    (LANGUAGES LOADED card with loaded count + failed chips)
- get_language_grammar_status Tauri command implemented
  — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:919
- phase_1A.toml [per_language_node_counts] table added
  — evidence: .elai_cc/metrics/phase_1A.toml:15-36 (table present, rust=8075 others=0)
- multilang_extraction.rs test file exists (per-language smoke tests)
  — evidence: crates/knowledge/tests/multilang_extraction.rs:55+ (JS, Go, C++, Scala, PHP, Swift, Bash tests)
- language_registry.rs test file exists (registry isolation, sorted status)
  — evidence: crates/knowledge/tests/language_registry.rs:88,98-103,119
- jvm_resolver.rs + jvm_global_lex.rs test files exist
  — evidence: crates/knowledge/tests/jvm_resolver.rs, jvm_global_lex.rs
- cs_resolver.rs test file exists (lexicographic ordering proof)
  — evidence: crates/knowledge/tests/cs_resolver.rs
- go_resolver.rs test file exists
  — evidence: crates/knowledge/tests/go_resolver.rs
- tsconfig_aliases.rs test file exists (extends chain + cycle prevention)
  — evidence: crates/knowledge/tests/tsconfig_aliases.rs
- vue_extraction.rs test file exists
  — evidence: crates/knowledge/tests/vue_extraction.rs
- ext_to_lang_id covers 16 of 18 required language IDs
  — evidence: crates/knowledge/src/dkg/mod.rs:3233-3258 (rs py go ts tsx svelte vue java cs rb js c cpp scala php swift sh)

## Claims PARTIAL

- Plan specifies unified JvmGrammar covering Java/Kotlin/Scala in `language/jvm.rs`
  - exists: JavaGrammar (mod.rs:1904) and ScalaGrammar (mod.rs:2651) implemented;
    Kotlin routed in clean_import_name (mod.rs:3279) and resolution dispatch (mod.rs:530)
  - gap: KotlinGrammar struct is absent; `.kt` extension is absent from ext_to_lang_id
    (mod.rs:3233-3258); Kotlin files will not be indexed
- Plan specifies unified CFamilyGrammar in `language/cfamily.rs`
  - exists: CGrammar (mod.rs:2413) and CppGrammar (mod.rs:2522) both implemented
  - gap: named CGrammar/CppGrammar not CFamilyGrammar; stored in mod.rs not cfamily.rs
    (naming mismatch vs WIRING_CONTRACT which says CFamilyGrammar)
- Plan specifies unified SvelteVueGrammar in `language/svelte_vue.rs`; composite qualified names `<file>.svelte::<symbol>`
  - exists: SvelteGrammar (mod.rs:1606) and VueGrammar (mod.rs:1712) both present
  - gap: they are separate structs, not one SvelteVueGrammar; plan's WIRING_CONTRACT
    symbol `knowledge::dkg::language::svelte_vue::SvelteVueGrammar` does not resolve
- Plan specifies phase_1A.toml updated with per-language node counts from a multilang fixture
  - exists: [per_language_node_counts] table present in metrics/phase_1A.toml:15
  - gap: all non-Rust values are 0 (comment in file: "Phase 1A fixture was Rust-only");
    the plan's observable_effect "expect_artifact: phase_1A.toml shows nodes>0 for at
    least 5 languages" is not met

## Claims MISSING

- Kotlin grammar (KotlinGrammar struct + `.kt`/`.kts` in ext_to_lang_id)
  — searched: KotlinGrammar, tree_sitter_kotlin, "kt" ext_to_lang_id; no evidence
- `docs/fixtures/multilang_project/` multi-language test fixture tree
  — searched: docs/fixtures/multilang_project, multilang_project; path does not exist
- `crates/knowledge/tests/svelte_extraction.rs` composite-language re-parse test
  — searched: svelte_extraction.rs in tests/; not present (vue_extraction.rs exists but not svelte)
- `workflow/reasoning/B-30-decisions.md` decision doc
  — searched: B-30-decisions in workflow/reasoning/; no file found
- Separate per-language source files as specified in "Files touched"
  (python.rs, typescript.rs, go.rs, jvm.rs, cfamily.rs, csharp.rs, ruby.rs,
  php.rs, swift.rs, bash.rs, svelte_vue.rs, registry.rs under dkg/language/)
  — all grammar code consolidated in dkg/mod.rs + separate language_registry.rs;
  no dkg/language/ subdirectory exists

## Claims UNVERIFIABLE

- `LanguageRegistry.loaded.len() >= 10` stub_ban assertion (runtime behavior)
  — reason: requires cargo test execution; static analysis cannot confirm count at runtime
- End-to-end indexing observable_effect on multi-lang fixture project
  — reason: fixture does not exist; cannot verify SSB emission or per-language node counts > 0
- B-29 symbol graph cache builds non-empty for non-Rust languages
  — reason: depends on runtime fixture execution; no multilang fixture exists to test against

## What must be done (action list)

- [ ] Add KotlinGrammar struct to crates/knowledge/src/dkg/mod.rs (mirrors JavaGrammar;
      uses tree-sitter-kotlin if available, else fallback to java grammar)
- [ ] Add "kt" and "kts" to ext_to_lang_id in crates/knowledge/src/dkg/mod.rs:3233
- [ ] Register kotlin grammar in LanguageRegistry::default() (near line 222)
- [ ] Create crates/knowledge/tests/svelte_extraction.rs — composite-language re-parse,
      qualified name disambiguation test
- [ ] Create docs/fixtures/multilang_project/ with one source file per language
      (or a tests/fixtures/multilang/ under knowledge crate) and re-run measure.rs
      so phase_1A.toml per_language_node_counts shows nodes>0 for >=5 languages
- [ ] Write workflow/reasoning/B-30-decisions.md covering tree-sitter vs ast-grep,
      catch_unwind vs Result-only, TS alias resolution order, qualified-name format,
      Dart/Lua/R deferral decision
- [ ] Rename/alias CFamilyGrammar and SvelteVueGrammar in WIRING_CONTRACT to match
      actual implementation names (CGrammar/CppGrammar, SvelteGrammar/VueGrammar)
      OR create type aliases so the contract symbols resolve

## Notes

The plan specifies individual files under `crates/knowledge/src/dkg/language/` but
all grammar implementations landed in `crates/knowledge/src/dkg/mod.rs`. This is a
structural divergence from the spec's "Files touched" list but does not affect
correctness. The WIRING_CONTRACT symbol paths (e.g.
`knowledge::dkg::language::python::PythonGrammar`) do not match the actual module
paths (`knowledge::dkg::mod::PythonGrammar`), which means wiring_contract_check.py
would flag these as unresolved if run. The contract test that exists is named
`dkg_multilang_contract.rs` (in crates/contracts/tests/), not
`tests/contracts/multilang_dkg_contract.rs` as specified.
