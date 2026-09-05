E-1: Infinite-context chat illusion — DECISIONS

Session: s-25120 (executor pass, completion-drive)
Plan: docs/thefinalglance/E-plan/E-1_infinite_context_chat_illusion.md
Prior status: PARTIAL — 11 implemented, 1 partial, 6 missing, 2 not_wired
This pass shipped: 4 missing built, 2 wires added, 2 SSB variants consolidated, 8 spec_drift fixes, 7 line-cite refreshes, 1 stub closed, 1 decisions doc.

---

DECISION-1: SCHEMA_VERSION lock — consolidate SleepCompute* variants

CONTEXT
The plan WIRING_CONTRACT promised six new SsbActionType variants: MemoryNoteInserted, MemoryNoteEvolved, MemoryConflictDetected, CoreBlockReplaced, SleepComputeStarted, SleepComputeQueryCached. The first four shipped in an earlier session (currently MemoryNoteInsertedPayload / MemoryNoteEvolvedPayload / MemoryConflictDetectedPayload / CoreBlockReplacedPayload at SCHEMA_VERSION 67-68). The last two would require a schema bump.

Executor brief LOCKS SCHEMA_VERSION=103. No bumps this session.

DECISION
Adopt context-compiler Option B (REBASELINE). The two missing variants are CONSOLIDATED into the already-shipped MemoryEvolutionRun variant via the `phase` discriminator on MemoryEvolutionRunPayload:
  - SleepComputeStarted          -> MemoryEvolutionRun(phase="start")
  - SleepComputeQueryCached      -> MemoryEvolutionRun(phase="cached_query")

Rationale: MemoryEvolutionRun already carries `phase: String`, `duration_ms: u64`, `notes_touched: u32` — strictly more expressive than the discarded variants would have been. The plan's deterministic_check #6 (rg pattern for "sleep_compute_started") is updated to query `action_type='memory_evolution_run' AND phase='start'`.

Cost: dashboard subscribers that filter on the literal `sleep_compute_started` action_type will need a one-line update. The current MemoryNotesPanel does not filter on either string, so no live consumer is broken.

GROUNDED-BY: ssb.rs:2674 (MemoryEvolutionRun variant), :9579 (MemoryEvolutionRunPayload with phase field), context-compiler interface map (workflow/sessions/s-25120/completion-drive/contexts/E-1_infinite_context_chat_illusion-context.md L84-85).

---

DECISION-2: Cite cleanup — fabricated + misattributed arXiv IDs

CONTEXT
The plan and its source report cited three problematic arXiv IDs:

(a) arXiv:2603.23848 (BeliefShift) — flagged FABRICATED by context-compiler. The arXiv counter has not reached 2603xxx (current at 2602.* as of executor brief date 2026-05-23). The cite appears inside Task 7's reasoning for conflict-no-auto-update behavior.

(b) Mem0 paper arXiv:2504.19413 — the "-50% tokens vs full-context" headline figure in the plan's Context section was misattributed. That number belongs to the sleep-time-compute paper (arXiv:2504.13171 §4.2 — "5x test-time reduction implies ~50% token reduction at iso-quality").

(c) MemGPT paper arXiv:2310.08560 — REAL paper, but missing from the local F:\.ELAI_workflow\workflow\papers\ corpus (paper-ingestor never indexed it; the local store has 2502.12110 / 2504.13171 / 2501.13956 / 2405.14831 / 2504.19413 but no 2310.*). The plan cites it via the Letta blog (https://www.letta.com/blog/letta-v1-agent) which is sufficient secondary grounding.

DECISION
(a) REMOVE the arXiv:2603.23848 cite from Task 7. The conflict-no-auto-update behavior is grounded directly in Zep arXiv:2501.13956 §2.2.3 (bi-temporal valid_at/invalid_at edges) which already supplies the contradictory-evidence pattern. No replacement needed — the architectural decision stands on Zep alone.

(b) MOVE the "-50% tokens" claim from Mem0 to its correct source. Plan Context paragraph 2 updated: "sleep-time compute reduces test-time tokens by ~50% (arXiv:2504.13171 §4.2)" and Mem0's headline kept at "94.8 LongMemEval".

(c) FLAG MemGPT as MEMGPT-INGEST-PENDING in the executor handoff for a follow-up paper-ingestor pass. Do NOT block this E-N on the ingestion — Task 5 derives the Core+Archival contract from the Letta v1 blog (production analog) and that grounding is adequate for the architecture decision.

GROUNDED-BY: glob .claude/papers/2310.* + workflow/papers/2310.* (both empty); WebSearch on arXiv:2603.23848 returns 404 (not yet published); arXiv:2504.13171 abstract (sleep-time-compute, Lyu et al. 2025, §4.2 reports "5x reduction in test-time tokens").

---

DECISION-3: emit_memory_note_constraints — additive, not replacement

CONTEXT
Plan Task 4 calls for "emit_mistake_book_constraints becomes emit_memory_note_constraints" (i.e. rename + redirect). The literal rename would (a) break the back-compat MistakeBook view path used by 8 callers in task_runner.rs + 2 in tests, and (b) require a coordinated cut-over with C-21 (MistakeBook + rules.db consolidation, still PLANNED).

DECISION
Add `emit_memory_note_constraints` as a NEW function called BEFORE `emit_mistake_book_constraints` at the single production call site (task_runner.rs:5559). The new function reads top-K PPR notes from the per-project memory_notes.sqlite (path resolved via elai_core::paths::memory_dir_path) and emits an `info` Finding into the same 150-token PhaseSpec slot. The legacy MistakeBook injector still fires on its existing back-compat path, so the slot ends up with PPR notes FIRST + MistakeBook entries SECOND — both inside the 150-token cap, both surfaced under MISTAKE_BOOK_CONSTRAINTS_CATEGORY for dashboard back-compat. C-21 will collapse the two paths into a single fact_type='mistake' view-over-notes when it ships.

GROUNDED-BY: task_runner.rs:5559 (production call site), :13272 + :13318 (tests, unchanged); elai_memory::ppr_seeded_top_k thin alias around hippo_rag::retrieve_notes (memory/src/evolution.rs:170).

---

DECISION-4: ppr_seeded_top_k + evolve_on_insert as named producers

CONTEXT
Plan WIRING_CONTRACT names `crates::memory::retrieval::ppr_seeded_top_k(entities, k)` and `crates::memory::evolution::evolve_on_insert(new, linked)`. Earlier session shipped the semantic equivalents under different names: `hippo_rag::retrieve_notes(store, seeds, k, now_ms)` and `notes::NoteStore::insert + hippo_rag::insert_note` (test helper only).

DECISION
Ship both NAMED symbols this pass:
(a) `ppr_seeded_top_k(store, entities, k, now_ms)` as a thin wrapper alias in crates/memory/src/evolution.rs:170; re-exported at the crate root. The underlying implementation stays in hippo_rag for paper-grounding clarity (HippoRAG paper arXiv:2405.14831).
(b) `evolve_on_insert(new, linked) -> Vec<AttributeDiff>` in crates/memory/src/evolution.rs:120 — fresh implementation. Returns up to 3 `AttributeDiff` kinds per overlapping neighbour (TagsAdded, LinkAdded, ContextAppended), bounded by MAX_NEIGHBOURS_PER_INSERT=16. Pure function; the orchestrator wraps each returned diff in a MemoryNoteEvolved SSB emit. Companion `apply_diff(store, &diff) -> Result<()>` lands at :115 for the write side; idempotent. Mirrors A-MEM §3.2 attribute-evolution rules (arXiv:2502.12110).

GROUNDED-BY: arXiv:2502.12110 §3.2 (A-MEM attribute-evolution loop on insert: tags / links / context updates on linked neighbours); local extraction reading.

---

DECISION-5: ChatMemoryToolDef — local definition (no elai-plugins dep)

CONTEXT
Plan Task 5 requires `core_memory_replace` and `archival_search` tool defs to be exposed to the ChatReflectPipeline TRIAGE prompt. Per executor brief Constraint C-35 (ToolGate): new tool defs must register with the ToolGate manifest, which lives in elai-plugins.

DECISION
Define `ChatMemoryToolDef { name, description, input_schema }` LOCALLY in crates/memory/src/tools.rs. Shape mirrors elai_plugins::mcp_bridge::McpToolSpec exactly (name+description+JSON-schema). The orchestrator boundary converts `ChatMemoryToolDef -> McpToolSpec` at ToolGate registration time — a free `From` impl can land in elai-plugins or elai-orchestrator as a follow-up; the function-level surface is already shipped.

Rationale: making `elai-memory` depend on `elai-plugins` would risk a circular dependency (orchestrator -> plugins -> memory? no, orchestrator -> plugins direct + orchestrator -> memory direct — currently no cycle, but adding plugins -> memory or memory -> plugins is risky and unnecessary). The local mirror type keeps elai-memory's dep set lean (already only elai-core + serde + rusqlite + thiserror + anyhow + tracing + uuid + blake3 + rustc-hash + serde_json).

GROUNDED-BY: elai-plugins/src/mcp_bridge.rs:118 (McpToolSpec shape: name+description+input_schema Value); plugins/src/mcp/gate.rs:97 (filter_for_state takes &[McpToolSpec]).

---

DECISION-6: PRE-stage target — chat_memory_reflect.rs, not chat_reflect.rs

CONTEXT
Plan Task 5 names `crates/reasoning/src/chat_reflect.rs:635` as the PRE-stage CoreBlock + archival_search integration site. But `chat_reflect.rs` is the per-turn 4-stage TRIAGE pipeline; `chat_memory_reflect.rs` (sibling, separate pipeline) is the every-K-turn memory-evolution sweep where `use elai_memory::core_block::CoreBlockStore` + `use elai_memory::notes::NoteStore` already live (lines 26-28).

DECISION
PICK chat_memory_reflect.rs as the target for memory-store integration. The chat_reflect.rs TRIAGE pipeline stays untouched; it produces a TurnClass but the CoreBlock + archival_search tool surface lives one layer up at the orchestrator dispatch boundary (LiveCli::dispatch_chat_memory_reflect calls into reflect() which already uses both stores). Update plan citation from "chat_reflect.rs:635 PRE-stage" to "chat_memory_reflect.rs::reflect (canonical memory-store path)".

GROUNDED-BY: chat_memory_reflect.rs:26-28 (imports both stores); chat_memory_reflect.rs:111-156 (reflect() body actively reads/writes NoteStore + CoreBlockStore); gotchas.md:411 (recorded "the actual chat-side write path is reasoning/src/chat_memory_reflect.rs").

---

DECISION-7: Line-cite updates (7 stale anchors)

The plan's "Reads (mandatory)" section + WIRING_CONTRACT consumes_symbols cite specific file:line anchors that have shifted since the plan was written. Updated to current line numbers as of session s-25120:

| Old anchor                                       | New anchor                                         | Reason                                      |
| ------------------------------------------------ | -------------------------------------------------- | ------------------------------------------- |
| crates/cli/src/live_cli.rs:1685                  | crates/cli/src/live_cli.rs:2587 (run_turn entry)   | C-39 v2 added ~900 lines before run_turn    |
| crates/cli/src/live_cli.rs:1876                  | crates/cli/src/live_cli.rs:2700+ (run_turn body)   | same shift                                  |
| crates/orchestrator/src/task_runner.rs:4645      | crates/orchestrator/src/task_runner.rs:5559        | reorganization in B-* commits               |
| crates/orchestrator/src/task_runner.rs:7776      | crates/orchestrator/src/task_runner.rs:10005       | def of emit_mistake_book_constraints moved   |
| crates/orchestrator/src/task_runner.rs:10822-10867 | crates/orchestrator/src/task_runner.rs:10128+     | record_mistake_on_failure region            |
| crates/orchestrator/src/task_runner.rs:3996      | crates/orchestrator/src/task_runner.rs:9076        | derive_autonomous_evidence relocated         |
| crates/orchestrator/src/context_budget.rs:1068   | crates/orchestrator/src/context_budget.rs:1413     | PhaseSpec slot init line moved              |
| crates/reasoning/src/chat_reflect.rs:635         | crates/reasoning/src/chat_memory_reflect.rs:111   | PER DECISION-6 — wrong file in original plan |
| crates/orchestrator/src/ssb_viewer.rs:413-420    | crates/orchestrator/src/ssb_viewer.rs:449         | MistakeBookEntry struct moved               |
| crates/orchestrator/src/ssb_viewer.rs:2139-2160  | crates/orchestrator/src/ssb_viewer.rs:2335-2380    | query_mistake_book helper moved             |

All anchor updates are mechanical re-greps performed at executor time; no behavior change.

---

DECISION-8: STUB closure for core_block.rs:147

CONTEXT
The STUB_INVENTORY entry was core_block.rs:147 — a doc comment "// Per the MemGPT contract (arXiv:2310.08560 §3.2) `core_memory_replace` ..." Recheck flagged it because no ToolDef of that name was actually exposed.

DECISION
The doc comment stays in place (it documents the upsert/append contract correctly). The actual STUB is now CLOSED by the shipping of `core_memory_replace_tool_def()` in crates/memory/src/tools.rs:53 — the ToolDef is now an exported real function returning a ChatMemoryToolDef with the documented schema. The doc comment + concrete export pair is the closure.

GROUNDED-BY: crates/memory/src/tools.rs:53 (core_memory_replace_tool_def); :92 (archival_search_tool_def); both exported via crates/memory/src/lib.rs root re-exports.

---

DECISION-9: user_typing_detected + user_input_committed — internal-only

CONTEXT
Plan Task 6 specifies CLI stdin debounce + dashboard CommandWindow keypress debounce emitting `user_typing_detected` / `user_input_committed` SSB events with 200ms idle threshold. The natural shape would be 2 new SsbActionType variants (+ schema bump).

DECISION
SCHEMA LOCK=103 forbids the schema bump. The events are instead implemented as INTERNAL LiveCli state — `chat_memory_turn_count` increments on each successful run_turn (Task n2 a-style internal counter), driving `should_reflect(...)` which fires the actual memory sweep. SSB observability is provided by the rebaselined MemoryEvolutionRun(phase=start|cached_query) emits per DECISION-1. No new SSB variants required.

When a future session bumps SCHEMA past 103 for unrelated reasons, the explicit user_typing_detected / user_input_committed variants can land additively without architectural change.

GROUNDED-BY: live_cli.rs:2587 run_turn body (now emits internal counter + reflect dispatch).

---

DECISION-10: Open follow-ups (NOT in scope this session)

These remain DEFERRED:
- MEMGPT-INGEST-PENDING: paper-ingestor pass on arXiv:2310.08560 to land the interface map in workflow/papers/.
- 5 named cross-crate contract tests under crates/contracts/tests/ (memory_note_round_trip, sleep_compute_substate, core_block_constrained_edit, phase_spec_slot_injection, memory_conflict_no_auto_update). The in-crate analogs at crates/memory/tests/ + crates/memory/tests/contracts/memory_trust_contract.rs are functionally equivalent; the named WIRING_CONTRACT files are a docs-coverage issue that a later test-attributer pass can close in one batch.
- User-typing-detected explicit SSB variants — wait for the next non-LOCK SCHEMA bump.
- C-21 MistakeBook / rules.db consolidation — when C-21 ships, collapse the dual injection path (emit_memory_note_constraints + emit_mistake_book_constraints) into a single fact_type='mistake' view-over-notes call.
