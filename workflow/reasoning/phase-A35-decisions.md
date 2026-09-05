Phase A-35: PI Multi-Language DKG Grammars

What: Contract tests for multi-language DKG grammars + TSX grammar fix + KnowledgeError migration.

Key decisions:
1. Grammars already existed (Python, Go, TS, Java, C#, Ruby) from prior work. A-35 scope = contract tests + audit fixes.
2. Added TsxGrammar with LANGUAGE_TSX (separate from TypeScriptGrammar/LANGUAGE_TYPESCRIPT) so .tsx with JSX parses correctly.
3. Migrated all DkgEngine pub methods from Box<dyn Error> to KnowledgeError (thiserror). map_err for set_language and index_version::write.
4. ext_to_lang_id("tsx") now returns "tsx" not "typescript" to route to TsxGrammar.

Expected test outcome: 24 contract tests GREEN, 442 knowledge unit tests GREEN (1 pre-existing bm25 Windows failure).

Unknowns:
- generate_cross_edges() only resolves Rust "use" imports. Non-Rust import edge resolution deferred (auditor WARN, not in A-35 scope).
- parse_symbol_kind catch-all maps unknown kinds to Function (auditor WARN, deferred).
- N+1 SQL in generate_cross_edges (auditor WARN, perf issue at scale, deferred).
