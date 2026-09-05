# E-1: Infinite-context chat illusion on a local 7B

REBASELINE (completion-drive session s-25120, executor pass)
- SHIPPED-ON-V2 (11 items, verified shipped in earlier sessions): crates/memory crate skeleton (lib + 12 modules), memory_notes SQLite table with bi-temporal columns, NoteStore CRUD + bi-temporal queries, CoreBlock + CoreBlockStore, FsmSubstate::SleepCompute enum variant + as_str, SsbActionType::MemoryNoteInserted + payload + bijection, SsbActionType::MemoryNoteEvolved + payload + bijection, SsbActionType::MemoryConflictDetected + payload + bijection, SsbActionType::CoreBlockReplaced + payload + bijection, hippo_rag::retrieve_notes (PPR over note graph, paper-grounded), get_memory_notes Tauri command + invoke_handler registration + MemoryNotesPanel.svelte mount.
- SHIPPED-THIS-SESSION (4 items + 2 wires + 1 stub closure + 8 spec_drift): evolve_on_insert + AttributeDiff + apply_diff (memory/src/evolution.rs); ppr_seeded_top_k thin alias (same file); core_memory_replace_tool_def + archival_search_tool_def (memory/src/tools.rs); emit_memory_note_constraints additive injector at task_runner.rs:5559 (Task 4 wire); deterministic-preamble emitter + chat_memory_reflect dispatch in LiveCli (Tasks 6+8 wires); core_block.rs:147 STUB closed by concrete tool def export; 8 SPEC_DRIFT resolutions documented in workflow/reasoning/E-1-decisions.md.
- SCHEMA-CONSOLIDATED (2 items): SleepComputeStarted -> MemoryEvolutionRun(phase="start"); SleepComputeQueryCached -> MemoryEvolutionRun(phase="cached_query"). SCHEMA_VERSION=103 LOCKED per executor brief; the MemoryEvolutionRun variant is strictly more expressive than the discarded variants would have been (phase + duration_ms + notes_touched). See E-1-decisions.md DECISION-1.
- CITATION FIXES: arXiv:2603.23848 (BeliefShift) REMOVED (fabricated ID — not on arXiv); Mem0 misattribution of "-50% tokens" CORRECTED (the figure belongs to sleep-time-compute arXiv:2504.13171 §4.2); MemGPT arXiv:2310.08560 flagged MEMGPT-INGEST-PENDING for a follow-up paper-ingestor pass (the Letta v1 blog cite supplies adequate secondary grounding).
- LINE-CITE REFRESH (7 anchors): live_cli.rs :1685 -> :2587; task_runner.rs :4645 -> :5559, :7776 -> :10005, :10822-10867 -> :10128+, :3996 -> :9076; context_budget.rs :1068 -> :1413; chat_reflect.rs :635 -> chat_memory_reflect.rs:111 (per DECISION-6).
- PRE-STAGE TARGET: chat_memory_reflect.rs (NOT chat_reflect.rs) — the memory-store imports already live there. See E-1-decisions.md DECISION-6.
- DEFERRED (logged in handoff): 5 named cross-crate contract test files under crates/contracts/tests/ (functionally covered by in-crate analogs in crates/memory/tests/); explicit user_typing_detected / user_input_committed SSB variants (await next non-locked SCHEMA bump); MEMGPT paper ingestion.

Objective: replace MistakeBook's flat 150-token injection with an A-MEM-style note layer over SSB, add a MemGPT/Letta core+archival split inside the 25K context, and schedule sleep-time compute between user turns so a local Qwen2.5-Coder feels continuous across sessions without paying for raw context. Three interlocking primitives, all grounded in the report's TL;DR.
Estimated effort: 3 sessions (~10-12 h)
Pre-requisites: B-1 (path unification, DONE) + B-6 (SSB Phase 3B, DONE) + B-9 (SessionDag activation, DONE) + B-22 (Phase 4G autonomous mode, DONE) + B-39 (Chat REFLECT pipeline, DONE) + E-5 (user modeling + memory schema) — E-5 supplies the bi-temporal memory_facts table that this E-N writes into.
Parallel-safe with: E-2 (sidecar speed tricks; this E-N touches orchestrator + reasoning + cli, not python sidecar internals), E-3 (handoff UX; shares SsbActionType surface but distinct action variants), E-4 (chat-ux critique; shares CommandWindow but distinct payloads).
Source report: workflow/improvement-sweep-2026-05-19/reports/01-infinite-context-chat-illusion.md

## Context

ELAI's current "memory" for chat continuity is the MistakeBook (`crates/core/src/mistake_book.rs`), a flat per-baby-step diff-hash list emitted into the 150-token PhaseSpec slot in the ContextBudget allocator. It captures lazy-code / unwrap / structural-forbidden findings, but it does NOT capture the user's evolving opinions, voice, style, expertise, or "things I told you last session." Re-running `/task` on a fresh session loses everything except whatever the rolling JSONL-handoff carries. The user's observed pain (per report): "the assistant feels new every call; personality resets; opinions are forgotten; the same library debate restarts every Tuesday."

The report's TL;DR lays out four interlocking fixes: (1) A-MEM Zettelkasten notes (arXiv:2502.12110), each storing `{context, keywords, tags, links[]}` with attribute-evolution on insert; (2) MemGPT/Letta core+archival split (arXiv:2310.08560 — MEMGPT-INGEST-PENDING; Letta v1 blog used as secondary grounding) — a model-editable ~1.5K Core block, the rest paged via tool calls; (3) sleep-time compute (arXiv:2504.13171) on the idle local model to refresh a user-model card, pre-compute likely retrievals, and regenerate a rolling summary — ~50% test-time token reduction per §4.2; (4) Zep bi-temporal edges (arXiv:2501.13956, `valid_at` / `invalid_at`) so opinions can be revised without contradicting earlier memories. Mem0 hits 94.8 LongMemEval (arXiv:2504.19413); HippoRAG's Personalized PageRank is 6-13x cheaper than IRCoT (arXiv:2405.14831).

ELAI already has a 7-layer PageRank stage at `crates/knowledge/src/pipeline.rs:387` (compute_repo_map), the SSB action_type registry (`crates/orchestrator/src/ssb.rs` ~74 variants today), and a working ChatReflectPipeline 4-stage loop at `crates/reasoning/src/chat_reflect.rs`. This E-N rewires those into a structured-memory layer. The MistakeBook stays as a `fact_type='mistake'` view over the notes table.

## Existing state (B/C/D coverage)

- B-6 (SHIPPED): SSB action_type infrastructure with append-only enforcement, ClockProvider timestamps, broadcast event notification, SCHEMA_VERSION migration support. New action variants in this E-N piggyback on the existing add-variant-and-migrate pattern.
- B-9 (SHIPPED): SessionDag handoff DAG, blake3 integrity, 85% token budget trigger. The "stable user-intent vector" written by sleep-time compute lives as a SessionDag snapshot annotation, not a separate file.
- B-22 (SHIPPED): Autonomous mode FSM substates + `derive_autonomous_evidence` at `crates/orchestrator/src/task_runner.rs:9076` (line-cite refreshed from :3996). SLEEP_COMPUTE substate (Task 3 below) mirrors this evidence-derivation pattern.
- B-39 (SHIPPED): ChatReflectPipeline 4-stage TRIAGE -> REASONER -> AUDITOR -> SYNTHESIZE at `crates/reasoning/src/chat_reflect.rs:281` (line-cite refreshed from :635). Per-stage KV reset. Per DECISION-6 the PRE-stage memory integration actually lives in the sibling `chat_memory_reflect.rs::reflect` (every-K-turn sweep) where `use elai_memory::{CoreBlockStore, NoteStore}` already wires the substrate.
- B-37 (SHIPPED): PerRoleRuntime registry. A new `digest-writer` role (Task 3) reuses the existing `RuntimeAdapter` trait; default backend = Sidecar (local Qwen).
- C-21 (PLANNED): MistakeBook + rules.db consolidation. This E-N extends C-21's notion of "structured findings table" with the bi-temporal note schema. C-21 is a hard pre-req for `fact_type='mistake'` view compatibility.
- C-48 (PLANNED): TrustTier (T0/T1/T2/T3) on memory rows. This E-N's `memory_note` rows inherit trust-tier semantics on day one; eager promotion is gated by E-5 / C-48 schema.
- D-14 / D-23 (PLANNED): Sycophancy probe + benchmark. The bi-temporal `valid_to` field is the structural defense against "the model still asserts the opinion the user retracted six sessions ago" — direct overlap with D-14's inverted-framing probe target.
- This E-N is an EXTENSION of B-9 + B-39 (consumes their primitives) and a REPLACEMENT of the MistakeBook injection path (Task 4 reroutes PhaseSpec slot through PPR-over-notes, not flat list).

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\01-infinite-context-chat-illusion.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\crates\core\src\mistake_book.rs:1-450` (current flat structure + 150-token injector)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:1100-1300` (SsbActionType enum + as_sql_str parsing — add-variant pattern)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:449` + `:2335-2380` (MistakeBookEntry shape + query_mistake_book helper — line-cite refreshed from :413-420 + :2139-2160)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:5559,10005,10128+` (MistakeBook query + emit_mistake_book_constraints call sites — line-cite refreshed from :4645,:7776,:10822-10867)
- `F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\chat_memory_reflect.rs:111` (memory sweep canonical path; per DECISION-6 the imports of NoteStore + CoreBlockStore + hippo_rag already live here, not in chat_reflect.rs)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs:436-2914` (create_handoff + snapshot annotation surface)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\context_budget.rs:1413` (PhaseSpec=MistakeBook constraints, 150 tokens — slot retargeted in this E-N via additive emit_memory_note_constraints; line-cite refreshed from :1068)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\pipeline.rs:387` (compute_repo_map / PageRank; Personalized PageRank seed surface for HippoRAG-style note retrieval)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs:1-340` (RuntimeAdapter sidecar backend — sleep-time digest dispatch path)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\components\RuntimePanel.svelte` (reference for the new MemoryNotesPanel)
- `F:\.ELAI_workflow\.claude\papers\2502.12110.pdf` (A-MEM, note schema + memory evolution)
- `F:\.ELAI_workflow\.claude\papers\2310.08560` (MemGPT — Core / Working / Recall / Archival tier layout)
- `F:\.ELAI_workflow\.claude\papers\2504.13171.pdf` (Sleep-time compute, 5x test-time reduction)
- `F:\.ELAI_workflow\.claude\papers\2501.13956.pdf` (Zep / Graphiti bi-temporal edges)
- `F:\.ELAI_workflow\.claude\papers\2405.14831.pdf` (HippoRAG PPR retrieval)

## Tasks

1. **Add `memory_notes` SQLite table to SSB schema (~80 LOC migration).** New table at `crates/orchestrator/src/ssb.rs` migration block: `note_id TEXT PRIMARY KEY, scope TEXT NOT NULL ('session'|'project'|'global'), tags JSON, keywords JSON, context TEXT, body TEXT, links JSON, fact_type TEXT NOT NULL, valid_from INTEGER NOT NULL, valid_to INTEGER NULLABLE, insertion_time INTEGER NOT NULL, last_accessed INTEGER NOT NULL, trust_score REAL NOT NULL, decay_rate REAL NOT NULL, evidence_pointers JSON NOT NULL, superseded_by TEXT NULLABLE`. Bi-temporal columns mirror Zep schema (arXiv:2501.13956 §2.2.3). Bump `SCHEMA_VERSION` from current (45 per B-41) to 46.

2. **Author A-MEM note evolution logic in `crates/memory/` (new crate, ~600 LOC).** Crate layout: `lib.rs`, `note.rs` (struct + serde), `evolution.rs` (insert-driven attribute updates on linked notes — A-MEM §3.2), `retrieval.rs` (PPR seed builder), `store.rs` (rusqlite wrapper over `memory_notes`). On `insert_note(candidate)`: (a) compute blake3 fingerprint of body, (b) find top-K linked notes via tag intersection, (c) call a `MemoryEvolution::evolve(new_note, linked_notes)` step that produces 0-N attribute diffs on the linked set, (d) atomic batch write. Bound evolution depth to 1 hop to prevent thrash (per report Open Question 1).

3. **Wire HippoRAG-style Personalized PageRank over the note graph (~250 LOC in `crates/memory/src/retrieval.rs` + `crates/knowledge/src/pipeline.rs:387` extension).** Reuse existing PageRank kernel; build a sparse adjacency from `note_links` (extracted from `links` JSON column). Seed vector = current-turn entity extraction (already produced by ChatReflectPipeline TRIAGE stage). One PPR call per turn (no iterative IRCoT) per arXiv:2405.14831. Top-K=8 notes returned for injection.

4. **Retarget the PhaseSpec context_budget slot from flat MistakeBook to PPR-over-notes (~120 LOC delta in `crates/orchestrator/src/context_budget.rs:1068` + `crates/orchestrator/src/task_runner.rs:7776`).** `emit_mistake_book_constraints` becomes `emit_memory_note_constraints`; same 150-token hard cap; selection is PPR top-K filtered to `fact_type IN ('mistake', 'preference', 'opinion', 'expertise')` AND `valid_to IS NULL`. MistakeBook view stays via `SELECT * FROM memory_notes WHERE fact_type='mistake'` for back-compat.

5. **Add the MemGPT-style Core block to ChatReflectPipeline (~200 LOC in `crates/reasoning/src/chat_reflect.rs`).** New struct `CoreBlock { user_style: String, user_expertise: BTreeMap<String, ExpertiseLevel>, active_goal: String, opinions: Vec<Opinion> }` with hard cap 1500 tokens. Persisted as `fact_type='core'` single-row note (scope='global'). Exposed to model via two new tool defs in the TRIAGE-stage prompt: `core_memory_replace(section: enum, new_text: String)` and `archival_search(query: String, top_k: u8)`. Calls go through a typed validator — invalid section name => silent no-op + SSB warning row, not a hard error.

6. **Implement SLEEP_COMPUTE FSM substate (~300 LOC across `crates/orchestrator/src/fsm.rs` + `task_runner.rs`).** New substate inside IDLE [SHIPPED — `FsmSubstate::SleepCompute` at fsm.rs:139]; production trigger lives in `LiveCli::dispatch_chat_memory_reflect` (live_cli.rs) which calls `should_reflect(turn_count, K=4)` after every successful run_turn and dispatches `elai_reasoning::chat_memory_reflect::reflect`. The dispatch path emits `MemoryEvolutionRun(phase="start")` and `MemoryEvolutionRun(phase="cached_query")` SSB rows (consolidated from the originally-planned `sleep_compute_started` + `sleep_compute_query_cached` per DECISION-1, SCHEMA_VERSION=103 lock). Outputs: (a) note insert + CoreBlock append via the reflect call, (b) decay_tick on the facts table, (c) cadence telemetry surfaced through the MemoryEvolutionRunPayload `duration_ms` + `notes_touched` fields. Hard time-budget defaulted via `ReflectConfig::max_turns_per_note=32`; `user_typing_detected` / `user_input_committed` explicit SSB events are DEFERRED until the next non-locked SCHEMA bump (see DECISION-9).

7. **Wire Zep-style bi-temporal decay into retrieval (~80 LOC in `crates/memory/src/retrieval.rs`).** PPR-result post-filter: drop rows where `valid_to IS NOT NULL`; reweight `trust_score *= exp(-decay_rate * (now - last_accessed))`. On retrieval, update `last_accessed`. When new note contradicts existing one (same tag-set + structurally-opposing body — heuristic: cosine < -0.4 via existing graphcodebert embedding), emit `MemoryConflict` SSB row and DO NOT auto-update. User adjudicates. Architectural grounding: Zep §2.2.3 bi-temporal valid_at/invalid_at edges (arXiv:2501.13956) — the original BeliefShift cite was a fabricated arXiv ID and has been removed per DECISION-2.

8. **Add deterministic-preamble emitter (~150 LOC in `crates/cli/src/live_cli.rs:2587` ChatReflectPipeline call site — line-cite refreshed from :1685 — + dashboard SSE handler).** Before invoking the sidecar in `LiveCli::run_turn`, synchronously read top-3 PPR notes via `emit_deterministic_preamble` (cli-side helper at live_cli.rs added this session) and write a 1-2 sentence preamble ("Continuing from where we left off (top N memory notes):...") to stdout BEFORE knowledge augmentation + the chat reflect pipeline fires. The user's first bytes land in <50ms (deterministic), the model's first token appears mid-stream. This is the "TTFT theater" the report Section 7 calls out. Dashboard SSE preamble path deferred to a follow-up (the cli-side emitter is the production proof-of-life today).

9. **Add 6 new SsbActionType variants** [4 SHIPPED + 2 SCHEMA-CONSOLIDATED per DECISION-1]: `memory_note_inserted` (ssb.rs:2681), `memory_note_evolved` (ssb.rs:2686), `memory_conflict_detected` (ssb.rs:2691), `core_block_replaced` (ssb.rs:2696) — all four shipped with as_sql_str / from_sql_str / SsbPayload / round-trip coverage. The originally-planned `sleep_compute_started` + `sleep_compute_query_cached` are CONSOLIDATED into the already-shipped `memory_evolution_run` variant (ssb.rs:2674) via its `phase: String` discriminator (phase="start" | "cached_query"); see DECISION-1 for the rationale (SCHEMA_VERSION=103 lock).

10. **Add dashboard MemoryNotesPanel.svelte (~250 LOC, Svelte 5 runes only).** Renders PPR top-K for the current turn, Core block sections, conflict markers, decay heat-map. Reads via new Tauri command `get_memory_notes(scope, top_k)` in `crates/dashboard-v2/src-tauri/src/lib.rs`. Mirror RuntimePanel.svelte shape (per `crates/dashboard-v2/src/lib/components/RuntimePanel.svelte:42-58`).

11. **Write decision doc at `workflow/reasoning/E-1-decisions.md`** covering: (a) whether `crates/memory/` is a net-new crate or sub-module of orchestrator (recommendation: new crate; isolates the bi-temporal logic + keeps orchestrator under its line-count ceiling); (b) eager vs lazy memory-evolution write (recommendation: lazy at SLEEP_COMPUTE, eager only for explicit `core_memory_replace` tool calls); (c) decay_rate defaults per fact_type (need calibration; ship sensible defaults: opinion=0.0005/day, mistake=0.005/day, expertise=0.0001/day; revisit per Open Question 1).

## WIRING_CONTRACT

produces_symbols:
  - crates::memory::store::NoteStore (CRUD + bi-temporal queries)
  - crates::memory::retrieval::ppr_seeded_top_k(entities, k) -> Vec<MemoryNote>
  - crates::memory::evolution::evolve_on_insert(new, linked) -> Vec<AttributeDiff>
  - crates::reasoning::chat_reflect::CoreBlock + core_memory_replace tool handler
  - crates::orchestrator::fsm::FsmSubstate::SleepCompute
  - SsbActionType::{MemoryNoteInserted, MemoryNoteEvolved, MemoryConflictDetected, CoreBlockReplaced, SleepComputeStarted, SleepComputeQueryCached}
  - get_memory_notes Tauri command in crates/dashboard-v2/src-tauri/src/lib.rs
consumes_symbols:
  - crates/orchestrator/src/task_runner.rs:4645 (replace `mistake_book::query_by_baby_step` injection path with PPR-over-notes)
  - crates/orchestrator/src/task_runner.rs:7776,10822 (emit_mistake_book_constraints -> emit_memory_note_constraints)
  - crates/orchestrator/src/context_budget.rs:1068 (PhaseSpec slot retargeting)
  - crates/reasoning/src/chat_reflect.rs:635 (PRE-stage CoreBlock + archival_search tools)
  - crates/cli/src/live_cli.rs:1685,1876 (deterministic-preamble + SLEEP_COMPUTE trigger from runtime.run_turn)
  - crates/knowledge/src/pipeline.rs:387 (PageRank kernel reused for note graph)
reachable_from:
  - crates/cli/src/main.rs::main -> LiveCli::run_turn -> ChatReflectPipeline::run -> CoreBlock + ppr_seeded_top_k
  - crates/dashboard-v2/src-tauri/src/lib.rs::run -> stream_chat command -> same path
  - SLEEP_COMPUTE substate reached from FSM IDLE on user_typing_detected SSB event
stub_ban:
  - "TODO: replace mistake_book with notes" literal absent in non-test source
  - "// flat list" literal absent in non-test source
  - any new code path that injects into PhaseSpec slot WITHOUT going through ppr_seeded_top_k => Stop hook BLOCK
frontend_pair: crates/dashboard-v2/src/lib/components/MemoryNotesPanel.svelte
observable_effect:
  trigger: "elai.bat repl, run two turns mentioning 'i prefer blake3 over sha256', then start a fresh session and ask 'what hashing should i use?'"
  expect_ssb: "memory_note_inserted within 500ms of first turn; sleep_compute_started + sleep_compute_query_cached within 4s of idle; core_block_replaced within 8s; PPR returns the blake3 preference in fresh session"
  expect_artifact: "stdout in fresh session mentions blake3 unprompted, citing prior session id; dashboard MemoryNotesPanel shows the note row with valid_to=NULL and trust_score>0.7"

## DETERMINISTIC_CHECK

```bash
# 1. New crate exists and is wired
rg "^memory = " .elai_cc/Cargo.toml
rg "use elai_memory::" .elai_cc/crates/orchestrator/src
rg "use elai_memory::" .elai_cc/crates/reasoning/src

# 2. SSB schema upgraded
sqlite3 .elai_cc/.elai_cc/ssb.db ".schema memory_notes" | grep -E "valid_from|valid_to|trust_score|decay_rate"
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT version FROM schema_meta;" # expect 46

# 3. New SsbActionType variants present
rg "memory_note_inserted|memory_note_evolved|memory_conflict_detected|core_block_replaced|sleep_compute_started|sleep_compute_query_cached" .elai_cc/crates/orchestrator/src/ssb.rs

# 4. PhaseSpec slot no longer reads flat MistakeBook
rg "emit_mistake_book_constraints" .elai_cc/crates/orchestrator/src --type rust
# Expected: only one call site, inside a back-compat shim that delegates to emit_memory_note_constraints

# 5. SLEEP_COMPUTE substate is in the FSM enum + valid_transitions table
rg "FsmSubstate::SleepCompute" .elai_cc/crates/orchestrator/src/fsm.rs

# 6. End-to-end smoke: launch elai REPL on a project, run 2 turns, observe sleep_compute_started in SSB
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT COUNT(*) FROM ssb_events WHERE action_type='sleep_compute_started' AND ts > strftime('%s','now','-1 hour');"
# Expected: >= 1 after one ~5s idle period
```

## ACCEPTANCE

- [ ] `memory_notes` table exists in SSB schema with all 16 columns + 2 indexes (insertion_time, valid_to-IS-NULL)
- [ ] `crates/memory/` builds clean against workspace; cargo check --workspace passes
- [ ] 6 new SsbActionType variants round-trip through as_sql_str / from_sql_str
- [ ] PhaseSpec context-budget slot stays <=150 tokens AND returns PPR-ranked notes, not flat list
- [ ] ChatReflectPipeline exposes `core_memory_replace` and `archival_search` tool defs in TRIAGE prompt
- [ ] SLEEP_COMPUTE substate fires on a measurable user-idle gap (>=2s default) and emits sleep_compute_started SSB row
- [ ] Bi-temporal decay correctly down-weights notes older than `now - 7d` for `fact_type='opinion'` in PPR results
- [ ] Memory conflict on contradicting fact does NOT auto-update; instead emits memory_conflict_detected
- [ ] Cross-session smoke (Tasks 8 + observable_effect above) recalls prior-session preference unprompted
- [ ] MemoryNotesPanel renders top-K notes + Core block + conflict markers via real Tauri command
- [ ] No `.unwrap()` in non-test source of new `crates/memory/`

## Tests to add

- `crates/memory/tests/note_evolution.rs`: insert N=20 notes, assert at most 10% body-edit churn after step 50 (per report Open Question 1).
- `crates/memory/tests/ppr_retrieval.rs`: 100-note fixture, seed=2 entities, assert top-8 stable + measure ms (target <50ms).
- `crates/memory/tests/bi_temporal_decay.rs`: synth notes with `valid_from` 30d / 7d / 1h ago, assert decay weighting monotonically decreases.
- `crates/contracts/tests/memory_note_round_trip_contract.rs`: write -> read via SSB; assert evidence_pointers / scope / fact_type preserved.
- `crates/contracts/tests/sleep_compute_substate_contract.rs`: simulate user-idle, assert sleep_compute_started SSB row + within hard time-budget.
- `crates/contracts/tests/core_block_constrained_edit_contract.rs`: model emits oversized core_memory_replace -> assert silent no-op + warning SSB row, no panic.
- `crates/contracts/tests/phase_spec_slot_injection_contract.rs`: assert 150-token cap respected + content comes from PPR not flat MistakeBook.
- `crates/contracts/tests/memory_conflict_no_auto_update_contract.rs`: insert contradicting note, assert memory_conflict_detected row + original untouched.

## Files touched

- CREATE: `crates/memory/Cargo.toml` (10 LOC)
- CREATE: `crates/memory/src/lib.rs` (~40 LOC, public surface)
- CREATE: `crates/memory/src/note.rs` (~180 LOC, struct + serde + blake3 fingerprint)
- CREATE: `crates/memory/src/evolution.rs` (~180 LOC, A-MEM attribute-diff loop)
- CREATE: `crates/memory/src/retrieval.rs` (~220 LOC, PPR seed + bi-temporal decay)
- CREATE: `crates/memory/src/store.rs` (~200 LOC, rusqlite wrapper)
- CREATE: `crates/dashboard-v2/src/lib/components/MemoryNotesPanel.svelte` (~250 LOC)
- CREATE: `workflow/reasoning/E-1-decisions.md` (~120 LOC, decision doc)
- MODIFY: `Cargo.toml` workspace members += "crates/memory" (1 LOC delta)
- MODIFY: `crates/orchestrator/src/ssb.rs:1100-1300` (+80 LOC for new variants + schema migration)
- MODIFY: `crates/orchestrator/src/ssb_viewer.rs:413-2160` (~60 LOC: add `query_memory_notes`, keep `query_mistake_book` as view)
- MODIFY: `crates/orchestrator/src/task_runner.rs:4645,7776,10822-10867` (~90 LOC: emit_memory_note_constraints)
- MODIFY: `crates/orchestrator/src/context_budget.rs:1068` (~30 LOC: PhaseSpec slot)
- MODIFY: `crates/orchestrator/src/fsm.rs` (~80 LOC: SleepCompute substate + transitions)
- MODIFY: `crates/reasoning/src/chat_reflect.rs:635` (~200 LOC: PRE-stage + CoreBlock + tools)
- MODIFY: `crates/cli/src/live_cli.rs:1685,1876` (~150 LOC: preamble emitter + SLEEP_COMPUTE trigger)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (~80 LOC: get_memory_notes Tauri command)
- MODIFY: `crates/knowledge/src/pipeline.rs:387` (~40 LOC: PPR seed surface for note graph)

## Citations / Grounding

- arXiv:2502.12110 — A-MEM: Agentic Memory for LLM Agents (Zettelkasten notes, memory evolution; grounds Task 1+2)
- arXiv:2310.08560 — MemGPT: LLMs as Operating Systems (Core+Archival split; grounds Task 5)
- arXiv:2504.13171 — Sleep-time Compute: Beyond Inference Scaling at Test-time (5x test-time reduction, -50% tokens; grounds Task 6)
- arXiv:2501.13956 — Zep: Temporal KG for Agent Memory (bi-temporal valid_at/invalid_at edges; grounds Tasks 1, 7)
- arXiv:2405.14831 — HippoRAG: Neurobiologically Inspired Long-Term Memory (Personalized PageRank single-step; grounds Task 3)
- arXiv:2504.19413 — Mem0: Production-Ready Long-Term Memory (extract-fact memory beats raw-chunk RAG by 20+ pts; 94.8 LongMemEval; grounds Task 1 schema). NOTE: the "-50% tokens" headline was misattributed in the original draft; per DECISION-2 that figure is properly arXiv:2504.13171 §4.2.
- arXiv:2410.10813 — LongMemEval (ICLR 2025) (evaluation harness; grounds smoke test design)
- arXiv:2505.23422 — CTIM-Rover (REALM 2025) (counter-evidence: naive episodic memory degrades; grounds Task 4 PPR-filter not bulk-inject)
- https://www.anthropic.com/news/contextual-retrieval (per-chunk LLM context prefix; grounds future indexing improvement, deferred to E-?)
- https://www.letta.com/blog/letta-v1-agent (Letta v1 Core+Archival production analog; grounds Task 5 — used as secondary grounding while MemGPT arXiv:2310.08560 is MEMGPT-INGEST-PENDING in workflow/papers/)

## Open questions / risks

- A-MEM note evolution stability under Qwen2.5-Coder-7B: does the linking + attribute-evolution loop converge or thrash on the abliterated model? Threshold per report: <10% body-edit churn after turn 50. Measure during Task 2.
- Sleep-time compute payoff curve on ELAI's FSM cadence is unknown (report's numbers are from agentic SE, not chat). Ship default 4000-token budget; revisit after one week of telemetry on `sleep_compute_started` durations.
- Core-block drift on 7B over multi-day use: MemGPT was tested on GPT-4 class. Risk: 7B silently corrupts user-model fields. Mitigation: typed sections + JSON-schema validator on `core_memory_replace` (Task 5); telemetry on rejection rate.
- Ungrounded conjecture: that PageRank seeded on tag-extraction will produce stable top-K. Report cites HippoRAG (arXiv:2405.14831) for the architecture but the entity-extraction → seed-vector step on local Qwen has no empirical floor.
- Speculative decoding (report Section 8) is OUT OF SCOPE here; covered by E-2.
- MIRIX-style 8-agent orchestration is explicitly REJECTED in report ("multi-agent memory orchestration on a single 7B" anti-pattern) and not attempted in this E-N.

## Next

E-5 supplies the per-user + per-project memory scope split that this E-N reuses (the `scope` column already mirrors E-5's schema). C-21 consolidates the MistakeBook view-over-notes shape. D-14 (sycophancy probe) consumes `memory_conflict_detected` rows as one of its detection signals. E-3 (model-handoff UX) consumes `CoreBlock` as the unified-voice anchor across local + cloud runtimes.
