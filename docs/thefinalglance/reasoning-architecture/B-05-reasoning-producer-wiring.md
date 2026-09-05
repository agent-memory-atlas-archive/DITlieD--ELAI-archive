# Reasoning Producer Wiring (B-5)

- **ELAI ids:** B-5 / B5-E01..B5-E22 / SUM-0032..SUM-0036
- **Spec status (as designed):** WIRED producers; live per-agent budget emission PARTIAL (couples B-8)
- **Crate / module (spec):** `crates/orchestrator` (`task_runner.rs` `emit_reasoning`, `ssb/action_type.rs`); `crates/dashboard-v2` (`src-tauri` commands, `drawers/ReasoningDrawer.svelte`, `components/CsqBlockCard.svelte`); consumes `crates/orchestrator/src/grc.rs` `PipelineGrc` and `crates/reasoning` `BasicGrcEngine`
- **Purpose of this file:** enough architecture to rebuild the reasoning telemetry organ — real per-agent GRC gauges, grounding-sweep / CSQ / feasibility / anti-rubber-stamp surfaces — so another harness can emit and render the same spine. Not a status ledger.

## 1. What it is

**Reasoning producer wiring** is the observability organ that replaces synthetic `emit_reasoning` with live values from a `PipelineGrc` handle, journals five typed SSB action types, and exposes them through Tauri commands into `ReasoningDrawer` (eight per-role budget gauges, grounding-sweep pie, CSQ list, feasibility counts, per-role `AgentGrcConfig` table, anti-rubber-stamp counter) plus a red CSQ intervention card (`B-5_reasoning_producer_wiring.md:1-8`, `:61-103`, `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B5-E01). Producer code is dashboard-version agnostic; V2 paths are the rebuild targets (`B-5_reasoning_producer_wiring.md:11-33`). Without B-8 hot-path callers the producers exist but do not fire on every reasoning event (`B-5_reasoning_producer_wiring.md:6-8`, `:173-176`).

## 2. Why it exists

ELAI’s trust spine is unreadable if the dashboard paints literals. The audited baseline hardcodes `agent_id="task_runner"`, `topic="task_runner"`, `budget_remaining=0`, `grounding_level="medium"`, `confidence=0.9`, `elapsed_ms=0` at `task_runner.rs:3985-4004`; `BasicGrcEngine::think()` is test-only (`B-5_reasoning_producer_wiring.md:37-43`, B5-E04). Phases 2B/2C/2D have zero Tauri commands; 2B/2D have zero SSB `action_type`s; CSQ Critical blocks and `FeasibilityOutcome::Blocked` are silent — a safety-UX failure (`B-5_reasoning_producer_wiring.md:45-47`). If the organ is faked (stub.ts forever, or a second synthetic emitter), operators cannot see remaining GRC budget, ungrounded claims, or a blocked `delete_file` (`ELAI-SPEC-OVERVIEW.md:70-72`, `ELAI-SYSTEMS-OVERVIEW.md:67-80`).

## 3. How it works (runtime)

1. **Who calls it.** `task_runner.rs::start_task_runner` / the production `emit_reasoning` site. B-8 later makes `PipelineGrc::{think,ground_plan,verify_claim,generate_csq,check_inline_feasibility}` fire so these producers have real events (`B-5_reasoning_producer_wiring.md:122-124`, `B-8_pipelinegrc_activation.md:1-8`).
2. **Input (deterministic pull, no model).** A live `PipelineGrc` handle plus the current agent/topic. Fields: `agent_id`, `topic`, `budget_remaining_calls`, `budget_remaining_chars`, `grounding_level` **enum** (not the string `"medium"`), `confidence: f32`, `elapsed_ms: u64` (`B-5_reasoning_producer_wiring.md:61-64`, B5-E05).
3. **Ban synthetic constants (deterministic).** `emit_reasoning` must not emit `agent_id="task_runner"`, `grounding_level="medium"`, `confidence=0.9`, or `elapsed_ms=0` (`B-5_reasoning_producer_wiring.md:125-128`, `:139-141`, B5-E19). `GroundingLevel::Medium` does not exist; 2A/2C levels are `Uncertain | High | Verified` (`B-39_chat_reflect_pipeline.md:93-95`, `02A-grc-state-machine.md` GroundingLevel).
4. **Per-GRC-step journal.** After an accepted think, emit `grc_step_emitted` with `{agent_id, topic, step_type, grounding_level, char_count}` (`B-5_reasoning_producer_wiring.md:66-67`, B5-E06). Neighbor 3C already writes a richer `ReasoningTraceEntry` (content_summary max 200 chars, grounding_source/level, confidence, elapsed_ms, budget_remaining) — this organ’s row is the **dashboard-facing** step event, not a fork of that schema (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` P3C-E032).
5. **Grounding sweep journal.** After `ground_plan` (B-8 / 2B), emit `grounding_sweep_result` `{plan_id, claims_total, grounded, verify_needed, ungrounded[]}` (`B-5_reasoning_producer_wiring.md:68`, B5-E07). Classification GROUNDED / VERIFY_NEEDED / UNGROUNDED is owned by 2B `PlanGroundingResult`; this organ **copies the counts**, it does not re-score (`02B-grounding-verification.md` §4).
6. **CSQ journal + intervention.** When `generate_csq` runs (B-8 pre-Write/Edit/Bash): emit `csq_evaluated` `{function_name, severity (CRITICAL|WARNING), guard_present, blocked}` (`B-5_reasoning_producer_wiring.md:69`, B5-E08). If Critical-without-guard already blocks in `BasicGrcEngine`, surface a red card: function name, CSQ (example `delete_file` / `data_loss`), actions `[View details] [Override] [Decline]`. Decline bubbles `ReplanRequest` to the FSM (`B-5_reasoning_producer_wiring.md:99-103`, B5-E17).
7. **Feasibility journal.** Emit `feasibility_check` `{claim, outcome (Pass|Fail|Blocked), source}` (`B-5_reasoning_producer_wiring.md:70`, B5-E09). WIRING_CONTRACT names the Rust variant `SsbActionType::FeasibilityReport` (`B-5_reasoning_producer_wiring.md:111`) — same organ, two names; see §16.
8. **Anti-rubber-stamp counter.** Engine rejection of auditor `Converge` without `FINDING:` (`engine.rs:297-315` as cited) must emit `auditor_rubber_stamp_rejected` so the drawer can show “Auditor N rubber stamps rejected this session” (`B-5_reasoning_producer_wiring.md:94-97`, B5-E16). Silent reject is banned.
9. **Dashboard read path (deterministic).** Tauri commands query SSB / live engine state; frontend keeps `stub.ts` names 1:1 (`B-5_reasoning_producer_wiring.md:33`, B5-E03). Existing V2 stubs: `get_grc_budgets`, `get_grounding_sweep`, `get_csq_pending`, `get_gg_signals` (`B-5_reasoning_producer_wiring.md:21-25`).
10. **Commit.** Real values on the bus + drawer. **Abstain.** None — missing telemetry is a fail for this organ (synthetic is not an allowed degraded mode). **Fail-closed for safety UX:** CSQ Critical-without-guard must be visible, not only blocked in-process (`B-5_reasoning_producer_wiring.md:45-47`).

The model may generate GRC `content` / CSQ guard code. It may **not** invent budget remaining, grounding_level, or a “medium” confidence label.

## 4. Types, enums, objects

Copy from B-5. Do not invent extra payload fields.

```
# emit_reasoning live tuple (B-5_reasoning_producer_wiring.md:61-64)
agent_id: String
topic: String
budget_remaining_calls: u32          // from GrcBudget.remaining_calls()
budget_remaining_chars: u32
grounding_level: GroundingLevel      // Uncertain | High | Verified
confidence: f32
elapsed_ms: u64

# SSB payloads (tasks §2)
grc_step_emitted:        { agent_id, topic, step_type, grounding_level, char_count }
grounding_sweep_result:  { plan_id, claims_total, grounded, verify_needed, ungrounded[] }
csq_evaluated:           { function_name, severity: CRITICAL|WARNING, guard_present, blocked }
feasibility_check:       { claim, outcome: Pass|Fail|Blocked, source }

# Rust SsbActionType variants (WIRING_CONTRACT)
SsbActionType::GrcStepEmitted
SsbActionType::GroundingSweepResult
SsbActionType::CsqEvaluated
SsbActionType::FeasibilityReport          // payload name vs action string: see §16
SsbActionType::AuditorRubberStampRejected

# Tauri DTOs
get_grc_budgets_per_agent() -> Vec<{ agent_id, budgets_per_topic[] }>
get_grounding_sweep(plan_id) -> last sweep result
get_csq_log(limit) -> recent CSQ events
get_feasibility_log(limit) -> recent feasibility events
get_agent_grc_config() -> AgentGrcConfig table (makes B-15 divergence visible)
get_auditor_rubber_stamp_rejections     // WIRING_CONTRACT sixth command

# Drawer rows (exactly 8 roles)
Executor | Auditor | ContextCompiler | PromptEngineer |
MemoryWriter | Designer | Researcher | MediaValidator

# CSQ intervention card
CsqBlockCard { function, csq_kind, actions: ViewDetails | Override | Decline }
Decline -> ReplanRequest  (FSM)
```

`AgentGrcConfig` is the 8-field 2C type (`role`, `budget`, `required_steps`, `grounding_level`, `csq_required`, `allowed_methods`, `transition_overrides`, `require_scope_enumeration`) — this organ **displays** it; it does not redefine it (`02C-per-agent-grc-config.md` §4).

## 5. Control flow / state machine

This organ does **not** own GRC transitions. Owner of the think loop remains `GrcEngine::think` / `PipelineGrc` (`02A-grc-state-machine.md` §5). B-5 owns the **emit → journal → query → render** loop:

```
think / ground_plan / generate_csq / check_inline_feasibility / anti-rubber-stamp reject
    → SSB append (single-writer)
    → Tauri get_* 
    → ReasoningDrawer / CsqBlockCard
Decline on CSQ card → ReplanRequest → FSM
```

**Forced:** every production `emit_reasoning` must take the `PipelineGrc` handle; no parallel synthetic path (`B-5_reasoning_producer_wiring.md:125-128`).

**Dashboard-only states:** gauge red when remaining/max `< 20%`; pie GROUNDED green / VERIFY_NEEDED amber / UNGROUNDED red; CSQ Critical-unguarded red badge (`B-5_reasoning_producer_wiring.md:80-92`).

## 6. Grounding, receipts, gates

B-5 does not verify claims. It **surfaces** 2B sweep classes and 2D CSQ/feasibility outcomes.

**Never allowed as a confidence signal:** hardcoded `confidence=0.9`, literal `grounding_level="medium"`, remaining budget as a quality score, LLM self-rating of “how grounded this step was”. The `confidence: f32` field on `emit_reasoning` is a **telemetry copy** of whatever the engine already stored (3C `ReasoningTraceEntry.confidence`); this organ must not compute a new 0–1 score (`B-5_reasoning_producer_wiring.md:61-64`, `:37-43`, `phase-2A.txt:13` as cited in 02A — 2A has no confidence field).

CSQ Critical-without-guard is a **gate** (blocks) plus a **receipt** (`csq_evaluated`). Override is a human intervention, not an engine self-pass (`B-5_reasoning_producer_wiring.md:99-103`).

## 7. Budgets, limits, ceilings

B-5 does not set GRC `max_calls` / `max_chars`. It **displays** remaining vs max per agent/topic.

| Knob | Value | Scope |
|---|---|---|
| drawer role rows | 8 (list in §4) | per session view |
| gauge red threshold | remaining/max `< 20%` | per gauge |
| feasibility panel | Pass/Fail/Blocked counts + last **5** events | drawer |
| CSQ log / feasibility log | `limit` argument on get_* | query |
| `char_count` on `grc_step_emitted` | step content length | per step |

Neighbor budgets this drawer must be able to show (not owned here): 2A default 5/800; 2C executor 8/1200, auditor 10/1200, memory_writer 2/400, prompt_engineer 5/800, designer 8/1200; 4D Research Agent 10/1200, Designer 8/1200 (`02C-per-agent-grc-config.md` §7, `phase-4D.txt:27-30`).

## 8. Roles and neighbors

**Consumes:** `PipelineGrc` live budgets and step results (B-8 must call the engine); `AgentGrcConfig` table (2C); CSQ / feasibility outcomes (2D); anti-rubber-stamp `InsufficientFindings` (2C).

**Produces:** five SSB action types; Tauri get_*; ReasoningDrawer + CsqBlockCard.

**Must not fork:** a second hardcoded role table for the eight gauges that disagrees with `agent_grc_config_for_role` (B-15 divergent-table bug is **highlighted red**, not duplicated as truth) (`B-5_reasoning_producer_wiring.md:77-78`, `:91-92`). Must not keep V1 `crates/dashboard/` as the live surface — V2 remap is load-bearing (`B-5_reasoning_producer_wiring.md:15-32`, B5-E02).

**Eight-gauge set vs 2C six roles:** 2C specifies executor/auditor/context_compiler/memory_writer/prompt_engineer/designer. B-5’s drawer **adds** Researcher and MediaValidator (`B-5_reasoning_producer_wiring.md:82-83`, `ELAI-SYSTEMS-OVERVIEW.md:105-106`). Those two extra rows are this organ’s surface contract; their GRC numeric tables are **not** in B-5 (see §16).

**Neighbors:** B-8 (activation), B-6 (orchestration drawer, not this one), B-14 (may reuse CsqBlockCard), B-36 (`get_grc_budgets` live handler), B-39 (seventh REFLECT panel **below** these gauges).

## 9. Config / flags / defaults

- No B-5-named kill switch. Visibility is on whenever producers fire.
- `stub.ts` names stay 1:1 with live invokes (`B-5_reasoning_producer_wiring.md:33`).
- Frontend + producer are the **same B-step** (`B-5_reasoning_producer_wiring.md:129`, B5-E19).
- Decision doc required before EXECUTING: `workflow/reasoning/B-5-decisions.md` (`B-5_reasoning_producer_wiring.md:171`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `task_runner.rs` `emit_reasoning` (real handle) | `B-5_reasoning_producer_wiring.md:61-64`, `:166` |
| `ssb/action_type.rs` five `SsbActionType` variants | `:107-112` |
| `dashboard-v2/src-tauri/src/commands.rs` (or `lib.rs` per V2) `get_grc_budgets` … | `:113-118`, `:15-16` |
| `drawers/ReasoningDrawer.svelte` | `:120`, `:80-92` |
| `components/CsqBlockCard.svelte` | `:121`, `:27-32` |
| `start_task_runner` reachability | `:122-124` |
| `BasicGrcEngine` / `PipelineGrc` consumed, not reimplemented | `:54-55` |

V1 paths in “Files touched” (`crates/dashboard/src/commands.rs`, `src-frontend/...`) are **superseded** by the V2 rebaseline block (`B-5_reasoning_producer_wiring.md:15-32` vs `:166-170`). Rebuild V2.

## 11. Journal / SSB events

| action_type (string) | payload | when |
|---|---|---|
| `grc_step_emitted` | agent_id, topic, step_type, grounding_level, char_count | each GRC step |
| `grounding_sweep_result` | plan_id, claims_total, grounded, verify_needed, ungrounded[] | after sweep |
| `csq_evaluated` | function_name, severity, guard_present, blocked | CSQ eval |
| `feasibility_check` | claim, outcome Pass\|Fail\|Blocked, source | inline feasibility |
| `auditor_rubber_stamp_rejected` | (session counter consumer) | Converge rejected |

Smoke: `SELECT * FROM ssb WHERE action_type='csq_evaluated'` after a delete-file task (`B-5_reasoning_producer_wiring.md:148-150`). Observable trigger uses `elai-cli -- run --task "delete_file test.txt"` (`:131-134`).

## 12. Tests and observables

Deterministic checks (`B-5_reasoning_producer_wiring.md:136-151`):

| Id | Assert |
|---|---|
| D1 | `rg agent_id.*=.*"task_runner"` in `task_runner.rs` → **zero** |
| D2 | `SsbActionType::GrcStepEmitted\|GroundingSweepResult\|CsqEvaluated\|FeasibilityReport` → **4 hits** in `action_type.rs` (rubber-stamp is the fifth; grep list is four) |
| D3 | sqlite `csq_evaluated` LIMIT 5 after destructive-file smoke |

Acceptance (`:153-162`, B5-E21): real emit_reasoning; 4–5 new SSB types; **five** live-data Tauri commands (WIRING_CONTRACT lists **six** including rubber-stamp reader — §16); eight gauges; pie; CSQ + feasibility panels; anti-rubber-stamp counter visible; Critical-without-guard intervention card.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Production `emit_reasoning` reads a real `PipelineGrc` handle; the four synthetic literals are absent.
- Five SSB types exist with the payloads in §4 (string names as in tasks §2).
- Eight budget gauges for the eight named roles, remaining/max calls **and** chars, red `<20%`.
- Grounding pie GROUNDED / VERIFY_NEEDED / UNGROUNDED with click-through ungrounded list.
- CSQ list + Critical-unguarded badge; feasibility Pass/Fail/Blocked + last 5.
- Collapsible `AgentGrcConfig` table with cells red where a divergent table disagrees with spec.
- Session rubber-stamp rejection counter driven by `auditor_rubber_stamp_rejected`.
- `CsqBlockCard` with Decline → `ReplanRequest`.
- stub.ts invoke names 1:1 with live commands (`get_grc_budgets` is the V2-stable name — B5-E03 / B-36).

A dashboard that animates fake remaining=0, or a prompt that says “show the user the budget”, is a cousin, not this organ.

## 14. Non-goals / banned designs

- Implementing `think` / `ground_plan` / CSQ / feasibility — those are 2A–2D + B-8 (`B-5_reasoning_producer_wiring.md:1-8`).
- Keeping V1 ReasoningDrawer as canonical (`:15-32`).
- `GroundingLevel::Medium` or string `"medium"` (`B-39_chat_reflect_pipeline.md:93-95`).
- Inventing a new confidence formula in the producer.
- Mock-data forever in production paths (B-3 CLASS A; stub.ts is the **name** contract, not the live payload) (`B-PLAN-SUMMARY.md:126-141`).
- A second GRC engine inside the dashboard.
- Skipping CSQ intervention because the engine already blocked (silent safety).

## 15. Source citations

Primary: `docs/thefinalglance/B-plan/B-5_reasoning_producer_wiring.md` (full file). Index: `B-PLAN-SUMMARY.md:161-172`, `:972`. Overview: `ELAI-SPEC-OVERVIEW.md:140-156`. Feature map neighbors: `ELAI-FEATURE-MAP.md:563-577`, `:639-641`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:67-80`. Exhaustive: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B5-E01..B5-E22, SUM-0032..SUM-0036. Engine types: `02A-grc-state-machine.md`, `02C-per-agent-grc-config.md`, `02D-csq-inline-feasibility.md`. Activation neighbor: `B-8_pipelinegrc_activation.md`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `get_grc_budgets_per_agent` (tasks) vs `get_grc_budgets` (stub.ts / WIRING_CONTRACT / B-36). Rebuild the **stub-stable** name `get_grc_budgets`; payload is still `Vec<{agent_id, budgets_per_topic[]}>` (B5-E10, B5-E03, `B-5_reasoning_producer_wiring.md:72-78` vs `:113`).
- Tasks say **5** Tauri commands; WIRING_CONTRACT lists **6** (`get_auditor_rubber_stamp_rejections`). Exhaustive B-004/X-C03: rubber-stamp reader inventory unconfirmed (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-004/X-C03).
- `feasibility_check` string vs `SsbActionType::FeasibilityReport` variant (`B-5_reasoning_producer_wiring.md:70` vs `:111`).
- `get_csq_pending` (existing stub) vs `get_csq_log(limit)` (task). Whether both exist or one renames the other is unspecified (`:21-24` vs `:75`).
- Payload fields of `auditor_rubber_stamp_rejected` (only the user-visible counter sentence is specified) (`:94-97`).
- Who writes `confidence: f32` / `elapsed_ms` — 3C trace vs PipelineGrc vs ClockProvider. B-5 names the fields; it does not define the formula (`:61-64`).
- Researcher / MediaValidator numeric GRC budgets for the two extra gauge rows (not in B-5 or 2C).
- Whether `grc_step_emitted` **replaces** or **sits beside** 3C `reasoning_trace` (`P3C-E032`).
- `budgets_per_topic[]` inner fields beyond remaining/max calls/chars implied by the gauge (`:84-85`).
- Override on CsqBlockCard: who authorizes, whether it journals, autonomous-mode behavior (B-8 leaves absent-user CSQ policy open; B-5 only specifies Decline → ReplanRequest) (`B-8_pipelinegrc_activation.md:144-147`).
