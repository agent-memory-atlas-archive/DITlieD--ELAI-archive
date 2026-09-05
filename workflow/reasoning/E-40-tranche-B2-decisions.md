E-40 Tranche B-2 — SymbolGraph DSL + Dataflow + Refine+Filter + 5th-list RRF + 2 ToolDefinitions — decisions
session: s-22300
date: 2026-05-31
status: SHIPPED-PENDING-AUDIT (cargo check + auditor cycle outstanding)
SCHEMA bump: 192 -> 193 (SymbolGraphEvent SINGLE-VARIANT Pattern-2, 45th application)
TOTAL_VARIANTS: 465 -> 466
Pattern-2 application counter: 44 -> 45

This is the Tranche B-2 addendum to E-40-decisions.md (s-25120, 2026-05-25).
The original decisions doc covered Tasks 1/5/7-9 (SHIPPED via DECISION-1..7).
B-2 covers Tasks 3/4/6/7/9 (DSL / Dataflow / Refine+Filter / 5th-list RRF /
2 ToolDefinitions).

DECISION-8 (addendum to DECISION-2): Pattern-2 single-variant
+ 4-arm SymbolGraphEventKind discriminator, NOT atomic +4 variants.

  - Rationale: B-40, B-41, B-42, B-43, E-31 (sub-phase 6A), E-43,
    E-47 have ALL used Pattern-2 collapse for >=3 related SSB beacons
    (7 consecutive applications since DECISION-2 shipped atomic +3 at
    s-25120). DECISION-2 was the LAST atomic bump in the symbol-graph
    family — every subsequent multi-row beacon has used Pattern-2.
    Continuing atomic here would be the OUTLIER, not the rule.
  - Concrete savings vs atomic +4: 3 SCHEMA slots saved (193 vs 196), 3
    ALL_VARIANTS rows saved, 3 alphabetical insert points saved,
    3 SQL CHECK tag literals saved. Action_type_lock.golden has ONE
    inserted row at ord 0403 instead of 4 rows scattered across the
    file.
  - The 4 Kind arms reserve the wire labels (symbol_dsl_executed /
    dataflow_impact_computed / agent_impact_claim_refuted /
    scope_graph_resolved). ScopeGraphResolved is reserved here even
    though the scope_graph impl lands in Tranche B-3 — reserving the
    slot now avoids a 2nd SCHEMA bump on B-3 landing.
  - Grounding: ssb.rs:2627-2628 documents "Pattern-2 collapse saves
    N-1 SCHEMA slots + N-1 ALL_VARIANTS rows + N-1 alphabetical
    inserts"; this is the canonical rationale text.

DECISION-9 (crate-rooting — confirms DECISION-1, adds adapter trait):
orchestrator-rooted modules + thin knowledge-side `SymbolGraphAdapter`
trait.

  - The DSL parser/executor + dataflow analyzer + refine+filter stage
    all live in `crates/orchestrator/src/symbol_graph/{dsl,dataflow,
    refine_filter}.rs` per DECISION-1 (orchestrator-rooted JSON cache
    surface is the B-2 substrate; the B-29 SQLite-rooted cache is NOT
    touched).
  - The 5th-list RRF integration runs INSIDE
    `elai_knowledge::pipeline::query_with_tier`, so the pipeline needs
    a way to call into the orchestrator-rooted cache without depending
    on `elai_orchestrator` (which would be an illegal reverse-direction
    edge — orchestrator depends on knowledge, never the other way).
  - Solution: knowledge defines a thin
    `SymbolGraphAdapter` trait in
    `crates/knowledge/src/symbol_graph_adapter.rs`; orchestrator
    implements it via `SymbolGraphCacheAdapter` (blanket-style impl in
    `crates/orchestrator/src/symbol_graph/mod.rs`). The pipeline holds
    `Option<Arc<dyn SymbolGraphAdapter>>` so existing call sites
    compile without code change; the live wiring at CLI startup
    installs the concrete adapter.
  - This pattern matches the existing `KnowledgeSsbSink` design where
    knowledge defines the trait and orchestrator implements it.

DECISION-10 (DSL parser — nom 7.x over hand-rolled or new dep):

  - Grammar is tiny (4 selectors + 3 filters + pipe-separator); a
    hand-rolled state machine would be ~50 LOC but lose forward
    composability (adding a 5th filter doubles the state-machine
    surface).
  - nom 7.1.3 is ALREADY resolved in Cargo.lock as a transitive dep
    of tantivy-query-grammar + spm_precompiled (verified
    Cargo.lock:3794, 6090, 6401). Declaring it as a direct
    orchestrator dep adds NO new compiled code to the build graph —
    the artifact is already there. Per the Tranche B-2 prompt
    constraint "use nom which is already in Cargo.lock" + the
    anti-slopsquatting protocol, nom is the canonical name for the
    rust-bakery/nom crate (verified crates.io/crates/nom).
  - Rejected: pest (new dep, would pull a new resolved crate). Rejected:
    chumsky (newer alternative but not in Cargo.lock). Rejected:
    hand-rolled (loses forward composability, harder to add the
    Tranche B-3 ScopeGraph selectors).

DECISION-11 (Refine+Filter weights — fixed v1, config-load deferred):

  - Plan E-40 Task 6 specifies "deterministic walker walks impact_radius
    and rejects if any file downstream is NOT in claim". The
    implementation here adds a re-rank step that combines the agent's
    initial list with the DSL result list via the standard RRF
    formulation (1/(k+rank)) weighted by alpha=0.6 (initial) + beta=0.4
    (DSL) + a small co-occurrence bonus for symbols appearing in both
    lists.
  - Two-iteration schedule matches RANGER (arXiv:2509.25257 sec. 4.2)
    "two-step refinement matches three-step within noise" — beyond 2
    iters the rank list converges and additional passes are pure
    compute cost.
  - Weights are FIXED in v1 (named consts: DEFAULT_ALPHA = 0.6,
    DEFAULT_BETA = 0.4, CO_OCCURRENCE_BONUS = 0.05). Config-loading via
    elai_cc.toml [knowledge.refine] is a follow-on per the
    E-17 ops follow-up pattern.

DECISION-12 (5th-list RRF — wire-dark default, env-gated):

  - Plan-mandated weight RRF_SYMBOL_GRAPH_VEC_WEIGHT = 0.9 (named
    const at pipeline.rs:74).
  - Default OFF: `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE` env gate (matching
    the precedent set by ELAI_ROUTE_DECISION_SSB_ENABLE,
    ELAI_LORA_SHIP_GATE_ENABLE, etc.). Operators opt in.
  - The pipeline holds `Option<Arc<dyn SymbolGraphAdapter>>`; even with
    the env gate ON, an absent adapter still degrades to a
    no-op (empty 5th list). RRF accumulator is order-independent
    across input lists so an empty 5th list is identity.
  - Live adapter wiring at `cli::live_cli::build_local_knowledge_stack`
    is a follow-on (NOT in Tranche B-2 scope). Tranche B-2 ships the
    substrate + env gate; the install site lands when the dashboard
    SymbolGraphWindow needs it (Tranche B-3 or a sibling C-plan per
    DECISION-7 SymbolGraphWindow deferral).

DECISION-13 (dashboard mounting — deferred to C-plan per DECISION-7):

  - The 2 new ToolDefinitions (symbol_dsl_query +
    symbol_dataflow_impact) are AGENT-FACING ONLY in Tranche B-2.
    No new Tauri commands, no Svelte panel updates, no types.ts
    additions. cross-language pair flagged via `// cross-lang-skip:
    dashboard-ts-pair` comment in the orchestrator changeset.
  - Rationale: DECISION-7 specified a new SymbolGraphWindow rather
    than a KnowledgeDrawer extension; that window will mount the DSL
    input form + dataflow visualization. SymbolGraphWindow ship lands
    in a sibling C-plan when C-24 KnowledgeDrawer ships (currently
    PLANNED per the s-25120 cross-check report).

KNOWN-RESIDUAL (intentional, follow-on):

  - SymbolGraphCacheAdapter live wiring at `cli::live_cli` — the
    pipeline knows how to call the adapter when present + the env gate
    is on, but the production CLI does not yet construct the adapter
    instance. Lands in a follow-on patch alongside the SymbolGraphWindow
    Tauri-command pair.
  - production_agent_rpc::translate_tool_request arms for the 2 new
    tools currently return EMPTY responses (typed wire shape but no
    cache backend). The real implementation lands when the adapter is
    wired in — for now the agent sees a consistent wire shape (matches
    the wire-dark precedent set by lsp_oracle in the original E-40).
  - Cross-language sidecar Pydantic mirror: NOT added in B-2 because
    the new ToolDefinitions are not on the sidecar HTTP boundary (the
    sidecar passes tool_use blocks through as Dict[str, Any] inside
    ContentBlock.input; per exploration GAPS-RESOLVED-5 no per-tool
    Pydantic dataclasses exist for any symbol_* tool).
  - SCIP protobuf parsing in scip_import — still COUNTER-ONLY per the
    s-25120 known-residual. Tranche B-3 work.

Files touched (net new + edits, all under .elai_cc/ + workflow/):
  + crates/orchestrator/src/symbol_graph/dsl.rs                  (~330 LOC, nom parser + 4-selector executor + tests)
  + crates/orchestrator/src/symbol_graph/dataflow.rs             (~250 LOC, BFS over reverse-call edges + tests)
  + crates/orchestrator/src/symbol_graph/refine_filter.rs        (~180 LOC, 2-iter RRF combine + tests)
  + crates/knowledge/src/symbol_graph_adapter.rs                 (~75 LOC, trait + error type)
  + crates/contracts/tests/e40_symbol_graph_contract.rs          (~260 LOC, F8-F13 coverage)
  + workflow/reasoning/E-40-tranche-B2-decisions.md              (this file)
  M  crates/ipc-types/src/lib.rs                                 (+2 TOOL consts + 5 wire structs/enums)
  M  crates/orchestrator/Cargo.toml                              (nom = "7" direct dep, transitive promoted)
  M  crates/orchestrator/src/symbol_graph/mod.rs                 (3 new mod decls + adapter impl + notify_symbol_graph_event)
  M  crates/orchestrator/src/ssb.rs                              (SCHEMA 192->193 + SymbolGraphEvent variant + 4-arm Kind + payload + ENV const + 7 lockstep sites)
  M  crates/orchestrator/src/production_agent_rpc.rs             (2 new ToolDefinitions + 2 dispatch arms + 2 postcard arms)
  M  crates/orchestrator/tests/action_type_lock.rs               (TOTAL_VARIANTS 465->466 + SsbActionType arm + exhaustive guard)
  M  crates/orchestrator/tests/action_type_lock.golden           (ord 0403 symbol_graph_event row, +1 ordinal shift 0404+)
  M  crates/knowledge/src/lib.rs                                 (pub mod symbol_graph_adapter)
  M  crates/knowledge/src/pipeline.rs                            (RRF_SYMBOL_GRAPH_VEC_WEIGHT const + env gate + symbol_graph_adapter field + with_symbol_graph_adapter builder + stage push + weight match arm)
  M  python/elai_inference/schemas/messages.py                   (SCHEMA_VERSION 192->193)
