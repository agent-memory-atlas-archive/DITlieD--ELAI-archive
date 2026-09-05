D.2 Phase C deferral (plugins_macros + #[authz] proc-macro + 200-tuple corpus generator)

Date: 2026-06-01
Session: s-22600 orchestrate, sub-phase trancheD
Decision: DEFER Phase C to a future tranche / patch dedicated to plugins_macros.

Why deferred:
- First proc-macro crate in the workspace. Adds crate-graph friction: NEW workspace member, [lib] proc-macro = true, 4 NEW direct deps on plugins_macros/Cargo.toml (syn 2 / quote 1 / proc-macro2 1 / darling 0.20). The 4 are already transitively resolved in Cargo.lock (no lock churn), but the new crate boundary plus dev-dependency wiring on plugins/Cargo.toml deserves a clean bisectable commit window.
- Phase C also pulls in the 200+ deterministic-tuple corpus generator (5 tools x 5 FSM states x 8 arg shapes = 200; rand 0.9 seed=42) + the corresponding c42b_authz_macro_contract.rs asserting corpus-count >= MCP_FUZZ_CORPUS_MIN_TUPLES. That is a 4-5 file delta orthogonal to the D.2 Phase A+B wire-through.
- Phase A+B shipping in this tranche already delivers the production-side McpAuthzEvent emission path. The proc-macro is a developer-UX ergonomic shorthand over compose_with_tool_gate_emit, NOT a correctness or security requirement (C-42-tranche-B6-decisions.md "the runtime guard achieves the same SSB-row-per-call outcome as the proc-macro path; the declarative attribute is a developer-UX win, not a security/correctness win.").

Will-resolve-in: future B-N / patch-N dedicated to plugins_macros + corpus.

Stub registry: see workflow/sessions/s-22600/stubs.md row `C42b-phase-c-procmacro` (action: DOCUMENTED-DEFERRAL).

What this defers (delta list, for the future patch author):
  NEW   .elai_cc/crates/plugins_macros/Cargo.toml                                   (proc-macro lib)
  NEW   .elai_cc/crates/plugins_macros/src/lib.rs                                   (#[proc_macro_attribute] authz)
  NEW   .elai_cc/crates/plugins_macros/tests/expand.rs                              (trybuild or integration)
  NEW   .elai_cc/crates/plugins/src/bin/corpusgen.rs OR mcp_fuzz_corpus crate       (200-tuple generator)
  NEW   .elai_cc/crates/contracts/tests/c42b_authz_macro_contract.rs                (macro expansion + corpus-count)
  MOD   .elai_cc/Cargo.toml                                                         (add "crates/plugins_macros" member)
  MOD   .elai_cc/crates/plugins/Cargo.toml                                          (dev-dependency on plugins_macros)
  MOD   .elai_cc/crates/orchestrator/src/ssb.rs                                     (NEW pub const MCP_FUZZ_CORPUS_MIN_TUPLES: usize = 200)

Anti-slopsquatting checklist for the future patch (per D10):
  cargo info syn          # expect 2.0.x latest stable
  cargo info quote        # expect 1.0.x latest stable
  cargo info proc-macro2  # expect 1.0.x latest stable
  cargo info darling      # expect 0.20.x latest stable
All 4 already in Cargo.lock at: syn 2.0.117, quote 1.0.45, proc-macro2 1.0.106, darling 0.20.11. Verify still latest at the time of the patch.

RESOLUTION

Tranche M (s-22600, 2026-06-01) closed this deferral. Delta shipped:
  NEW   .elai_cc/crates/plugins_macros/Cargo.toml                                     (proc-macro lib + 4 direct deps; zero lock churn)
  NEW   .elai_cc/crates/plugins_macros/src/lib.rs                                     (#[authz(gate=..., tool_kind=...)] marker-only attribute per D-M1; darling::ast::NestedMeta::parse_meta_list + FromMeta::from_list per D-M5)
  NEW   .elai_cc/crates/plugins_macros/examples/corpusgen.rs                          (rand 0.9 random_* API per D-M6; 200 line-delimited tuples per D-M4)
  NEW   .elai_cc/crates/plugins_macros/tests/corpus/.gitkeep                          (dir sentinel)
  NEW   .elai_cc/crates/contracts/tests/m_authz_macro_contract.rs                     (3 cases: happy-path, vis+sig preservation, documented negative)
  NEW   .elai_cc/crates/contracts/tests/m_authz_corpus_loop_contract.rs               (4 cases: floor, parse shape, dual-emit rule, const pin)
  MOD   .elai_cc/Cargo.toml                                                           (workspace.members += "crates/plugins_macros")
  MOD   .elai_cc/crates/orchestrator/src/ssb.rs                                       (NEW pub const MCP_FUZZ_CORPUS_MIN_TUPLES = 200 per D-M8)
  MOD   .elai_cc/crates/contracts/Cargo.toml                                          ([dev-dependencies] += elai_plugins_macros = path)

Design decisions captured in workflow/reasoning/M-trancheM-decisions.md (D-M1..D-M8).
Verification: cargo check -p elai_plugins_macros --examples --tests GREEN; cargo check --workspace --tests GREEN (1m06s clean + 3.57s incremental, only pre-existing warnings).
