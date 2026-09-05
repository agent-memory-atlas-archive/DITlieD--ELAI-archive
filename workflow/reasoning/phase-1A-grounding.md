Phase 1A Grounding Results
Date: 2026-04-06 (v2 — substantive improvement pass)
Session: s-1499300

SPEC VERIFICATION

READ directive: docs/specs/crates/knowledge.txt Sections 1-2
  Status: EXISTS, RELEVANT
  Evidence: L7 "1. RESPONSIBILITIES", L18-19 "Layer 1 - DKG"

READ directive: docs/specs/crates/knowledge.txt Section 8 (parse failure fallback)
  Status: EXISTS, RELEVANT (added in v2)
  Evidence: L168-177 defines fallback behavior for parse failures

READ directive: docs/specs/crates/knowledge.txt Section 10 (index versioning)
  Status: EXISTS, RELEVANT (added in v2)
  Evidence: L193-211 defines index_version.toml and migration logic

READ directive: docs/specs/architecture-overview.txt "Section 4 (KnowledgeEngine trait)"
  Status: SPEC_GAP
  Evidence: No "Section 4" numbering. KnowledgeEngine at L94-99 under TRAIT CONTRACTS.
  Action: Corrected to "TRAIT CONTRACTS section, knowledge::KnowledgeEngine (~L94)".

CLAIM GROUNDING

"First layer of knowledge pipeline" | VERIFIED | knowledge.txt L18-19
"tree-sitter -> SQLite graph" | VERIFIED | knowledge.txt L19-20
"Rust, Python, TypeScript, Go" | VERIFIED | knowledge.txt L21, risk-register.txt L45-46
"concept TEXT column" | VERIFIED | knowledge.txt L24-25
"file content hash for staleness" | VERIFIED | knowledge.txt L31
"KnowledgeEngine trait" | VERIFIED | architecture-overview.txt L94-99
"ReadinessStatus enum" | VERIFIED | architecture-overview.txt L99, knowledge.txt L154-164
"Incremental updates" | VERIFIED | knowledge.txt L20
"Parse failure fallback" | VERIFIED | knowledge.txt L168-177 (Sec 8)
"index_version.toml" | VERIFIED | knowledge.txt L193-211 (Sec 10)
"Edge/adjacency API for Phase 1D" | VERIFIED | phase-1D.txt L29-30 PRE_GATE
"DkgSnapshot built from DkgEngine" | VERIFIED | phase-1D.txt L35-37
"tree-sitter crate 0.26.8" | VERIFIED | crates.io 2026-04-06
"tree-sitter-rust 0.24.2" | VERIFIED | crates.io 2026-04-06
"tree-sitter-python 0.25.0" | VERIFIED | crates.io 2026-04-06
"tree-sitter-go 0.25.0" | VERIFIED | crates.io 2026-04-06
"tree-sitter-typescript 0.23.2" | VERIFIED | crates.io 2026-04-06 (aging — last update 2024-11)
"rusqlite 0.39.0" | VERIFIED | crates.io 2026-04-06
"tree-sitter-rust node types" | VERIFIED | node-types.json on GitHub: function_item, struct_item, impl_item, use_declaration, mod_item, trait_item, enum_item all confirmed
"Edge types: imports/contains/implements" | DERIVED | knowledge.txt L67 (edge-weight boost), L79 (CatRAG traversal), phase-1D.txt L35-37. No explicit list in specs.
"LanguageGrammar trait" | DERIVED | knowledge.txt L20-21 (100% coverage, 4 languages). Without trait dispatch, adding grammars requires parser refactor.
"<15s build time" | UNGROUNDED | knowledge.txt L20. Phase 1A.5 establishes baseline.
"Query latency" | UNGROUNDED | no target in specs. Phase 1A.5 records baseline.
"SQLite schema" | DESIGNED | not in specs. Schema in step 1A.1.
"Section 4 of architecture-overview" | SPEC_GAP | corrected to TRAIT CONTRACTS
"7-layer pipeline vs FIVE LAYERS" | SPEC_CONFLICT | knowledge.txt L16 vs architecture-overview.txt L100

SUBSTANTIVE IMPROVEMENTS (v2)

1. file_hash BLOB column added to dkg_nodes schema
   Reason: knowledge.txt L31 mandates content hash for staleness tracking.
   Without it, incremental updates cannot detect unchanged files.

2. Typed edge semantics defined: "imports", "contains", "implements"
   Reason: Phase 1D (PageRank) needs edge types for weight boosting (knowledge.txt L67).
   Phase 1E (CatRAG) needs typed traversal (knowledge.txt L79).
   Without types, downstream phases must re-parse source to infer relationships.

3. Public node/edge iteration API added to DkgEngine
   Reason: Phase 1D PRE_GATE (phase-1D.txt L29) requires "DKG edge/adjacency API exists".
   Phase 1D builds DkgSnapshot by iterating DkgEngine (phase-1D.txt L35-37).
   Without this, Phase 1D is blocked at entry.

4. LanguageGrammar trait for multi-language dispatch
   Reason: knowledge.txt L20-21 requires 4-language support.
   Trait dispatch isolates grammar-specific logic. Without it, adding Python/TS/Go
   requires refactoring the Rust-specific parser core.

5. Parse failure handling (graceful skip + logging)
   Reason: knowledge.txt Sec 8 (L168-177) mandates: parse failure -> log + skip.
   Without it, unsupported or malformed files crash or silently disappear.

6. readiness_check() implementation (READY/DEGRADED/UNAVAILABLE)
   Reason: orchestrator gates FSM transitions on readiness (knowledge.txt L163).
   Without it, orchestrator cannot determine if knowledge engine is usable.

7. index_version.toml creation
   Reason: knowledge.txt Sec 10 (L193-211) migration logic depends on this file.
   Without it, first upgrade hits "missing version file" edge case.

8. Canonical path disambiguation
   Reason: Phase 0 creates crates at .elai_cc/crates/ but current code is at .elai_cc/rust/crates/.
   Phase file now explicitly states canonical path: .elai_cc/crates/knowledge/.

BLOCK SUMMARY
Pre-gates: 5 (one per step)
Deterministic checks: 5 (one per step, expanded to 8+ individual checks in 1A.1 and 1A.4)
Adversarial tests: 12 (across all steps)
Acceptance criteria: 5 (one per step)
Test count target: 15+ (up from 6 in v1)
Ungrounded assumptions: 3
Spec gaps: 1 corrected
Spec conflicts: 1 flagged
Derived decisions: 2 (edge types, LanguageGrammar trait)
New risks: 4 (R-1A-1 through R-1A-4)
