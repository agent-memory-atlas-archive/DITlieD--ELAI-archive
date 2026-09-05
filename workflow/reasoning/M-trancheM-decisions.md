Tranche M (s-22600, 2026-06-01) -- D.2 Phase C decisions log.

Sub-phase: D.2 Phase C (plugins_macros + #[authz] proc-macro + 200-tuple corpus).
Closes:   workflow/reasoning/D2-phase-c-deferral.md (DEFERRED in Tranche D).
Inputs:   workflow/sessions/s-22600/context-package-trancheM.md (8 D-M decisions + HIGH/MED warnings).

Verification verdict: cargo check -p elai_plugins_macros --examples --tests GREEN; cargo check --workspace --tests GREEN (incremental 3.57s, only pre-existing workspace warnings).

D-M1 -- Attribute design (MARKER-ONLY vs FULL-WRAP)
Decision: MARKER-ONLY (context recommendation A).
Rationale: D2-phase-c-deferral.md L10 frames the attribute as a developer-UX shorthand over compose_with_tool_gate_emit, not a correctness or security requirement. Marker-only avoids the framework-wide convention burden of FULL-WRAP (which would require attribute args naming policy / gate / emitter bindings at every call site). Future patch can extend toward FULL-WRAP without breaking the surface (additive named args).
Implementation: src/lib.rs emits the original function body verbatim, prefixed with two const literals (__ELAI_AUTHZ_GATE / __ELAI_AUTHZ_TOOL_KIND) annotated #[allow(dead_code)] so the marker can coexist with the workspace clippy-warn lint level without firing unused-const warnings inside the user fn.

D-M2 -- Attribute arg shape
Decision: #[authz(gate = "...", tool_kind = "...")] with named args parsed via darling::FromMeta (task spec authoritative; matches context M.1 snippet shape).
Implementation: struct AuthzArgs { gate: String, tool_kind: String } derives FromMeta + Debug. Both args mandatory; missing-arg produces compile-time darling error at the attribute site (verified via the documented negative case in m_authz_macro_contract.rs).

D-M3 -- Corpusgen location
Decision: examples/corpusgen.rs inside the plugins_macros crate (task spec authoritative; resolves the context-package D-M2 "examples vs bins" debate the same way).
Rationale: examples auto-see [dev-dependencies], so `rand 0.9` stays out of the production dep graph for the proc-macro crate. Run path: `cargo run --example corpusgen -p elai_plugins_macros`. Co-located with the macro crate so a single git-blame walk surfaces both the attribute surface and the fuzz harness.

D-M4 -- Corpus output format + path
Decision: line-delimited text at crates/plugins_macros/tests/corpus/mcp_authz_200_tuples.txt; one tuple per line, pipe-delimited (`tool_name|authz_decision|expected_emit_count`).
Rationale: tool names carry dots, double underscores, and namespacing colons (`mcp__local-rag__mcp_search_context`), so pipe-delimited is the simplest format that does not require quoting. 200 lines * ~80 chars ≈ 16 KB on disk -- trivially diffable in code review. The skip-when-empty pattern in m_authz_corpus_loop_contract.rs preserves CI green without making corpusgen a prereq.

D-M5 -- syn 2 attribute parse API
Decision: darling::ast::NestedMeta::parse_meta_list(args.into()) -> FromMeta::from_list(&attr_args). Verified via darling 0.20.11 docs (https://docs.rs/darling/0.20.11/darling/ast/enum.NestedMeta.html).
Replaces the deprecated syn::AttributeArgs path the context package M.1 snippet originally suggested. The NestedMeta path is the canonical syn-2 + darling-0.20 idiom -- the Parse trait impl on NestedMeta handles both Meta and Lit variants and yields the Vec<NestedMeta> FromMeta::from_list expects.

D-M6 -- rand 0.9 API
Decision: random_range(0..n) + random_bool(0.5) (anti-stale-training; never the deprecated 0.8 gen_range / gen_bool spellings).
Verified at examples/corpusgen.rs against rand 0.9.2 (Cargo.lock L5018). The .expect() panics inside main() are CLAUDE.md-compliant (bootloader/entrypoint exception -- a panic at example runtime surfaces immediately to the operator, no silent corruption path).

D-M7 -- Anti-slopsquatting verification
Decision: informational comment block at the top of crates/plugins_macros/Cargo.toml citing the Cargo.lock line numbers for all 5 deps (syn 2.0.117 L6232, quote 1.0.45 L4962, proc-macro2 1.0.106 L4798, darling 0.20.11 L942, rand 0.9.2 L5018).
Rationale: all 5 already transitively in Cargo.lock today -- no new external sources, no lock churn. The CLAUDE.md HARD RULE is satisfied by the existing workspace pin trail; a `cargo info` round-trip would add zero new evidence. The header comment makes the verification trail discoverable from inside the new crate.

D-M8 -- MCP_FUZZ_CORPUS_MIN_TUPLES const placement
Decision: added pub const MCP_FUZZ_CORPUS_MIN_TUPLES: usize = 200 to crates/orchestrator/src/ssb.rs immediately after the existing MCP_FUZZ_CORPUS_DIR (near L24256), with a doc-comment cross-linking to the corpus generator + the corpus contract test.
Rationale: keeps the two fuzz-corpus consts physically adjacent (MCP_FUZZ_CORPUS_DIR + MCP_FUZZ_CORPUS_MIN_TUPLES); preserves the existing SCHEMA_VERSION_PUBLIC = 195 floor (no bump per D9-NO-NEW-SSB-VARIANT carry-over from Tranche D). Contract pin at m_authz_corpus_loop_contract.rs case 4 asserts the const stays >= 200.

Warnings resolved (HIGH/MED from context package):
- HIGH (syn 2 AttributeArgs deprecated): resolved via D-M5 NestedMeta path. The starter snippet's `parse_macro_input!(attr as AttributeArgs)` line was REPLACED with the canonical darling::ast::NestedMeta::parse_meta_list pattern before the first cargo check; the build passed first try (~1m06s clean compile of syn 2 + darling + quote).
- HIGH (rand 0.9 random_* names): resolved via D-M6 by using random_range / random_bool directly in examples/corpusgen.rs.
- MED (corpusgen as example sees dev-deps): resolved by the [dev-dependencies] rand entry in plugins_macros/Cargo.toml; example builds via cargo check -p elai_plugins_macros --examples without promoting rand to [dependencies].
- MED (D-M1 attribute design decision): made explicitly before code-write per warning instruction; documented above.
- LOW (workspace check timing -- G.3 type mismatch): cargo check --workspace --tests is GREEN today (the G.3 SymbolGraphCache risk noted in the context package did not surface).
- LOW (elai.map staleness): carry-over warning, not in Tranche M scope.
- LOW (contracts dev-dep on plugins_macros): resolved by the new line in crates/contracts/Cargo.toml [dev-dependencies] block.

File delta summary:
  NEW   .elai_cc/crates/plugins_macros/Cargo.toml
  NEW   .elai_cc/crates/plugins_macros/src/lib.rs
  NEW   .elai_cc/crates/plugins_macros/examples/corpusgen.rs
  NEW   .elai_cc/crates/plugins_macros/tests/corpus/.gitkeep
  NEW   .elai_cc/crates/contracts/tests/m_authz_macro_contract.rs
  NEW   .elai_cc/crates/contracts/tests/m_authz_corpus_loop_contract.rs
  MOD   .elai_cc/Cargo.toml                                                                    (workspace.members += plugins_macros)
  MOD   .elai_cc/crates/orchestrator/src/ssb.rs                                                (NEW pub const MCP_FUZZ_CORPUS_MIN_TUPLES = 200)
  MOD   .elai_cc/crates/contracts/Cargo.toml                                                   (NEW [dev-dependencies] elai_plugins_macros = path)
  MOD   workflow/reasoning/D2-phase-c-deferral.md                                              (RESOLUTION block)
  MOD   workflow/sessions/s-22600/stubs.md                                                     (C42b-phase-c-procmacro: DOCUMENTED-DEFERRAL -> WIRED-Tranche-M)

Test command surface (auditor runs):
  cargo test -p elai_plugins_macros --tests --examples
  cargo test -p elai_cc_contracts --test m_authz_macro_contract
  cargo test -p elai_cc_contracts --test m_authz_corpus_loop_contract
  cargo run --example corpusgen -p elai_plugins_macros   (regen corpus to lift the skip-when-empty path)
