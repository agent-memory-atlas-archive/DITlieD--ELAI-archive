# Chat REFLECT Pipeline (B-39)

- **ELAI ids:** B-39 / B39-001..B39-077 / SUM-0311..SUM-0322
- **Spec status (as designed):** WIRED (`ELAI-FEATURE-MAP.md:610-612`)
- **Crate / module (spec):** `crates/reasoning/src/chat_reflect.rs` (`ChatReflectPipeline`); roles in `reasoning/src/config.rs` `agent_grc_config_for_role`; wire at `crates/cli/src/live_cli.rs` `LiveCli::run_turn`; SSB `Reflect*` + SCHEMA 28→29; `ReasoningDrawer` REFLECT panel
- **Purpose of this file:** enough architecture to rebuild the per-message TRIAGE → REASONER N=4 → AUDITOR REFINE cap=2 → SYNTHESIZE organ. Not a status ledger.

## 1. What it is

**Chat REFLECT** is a per-message, four-stage reasoning pipeline on the chat path so every **non-trivial** user turn is classified, sampled (best-of-N=4), audited with a bounded REFINE loop, then locally synthesized — reusing `PipelineGrc`, `BasicGrcEngine`, and `AgentGrcConfig` with **no new model weights** (`B-39_chat_reflect_pipeline.md:1-6`, `ELAI-FEATURE-MAP.md:610-612`, `ELAI-SYSTEMS-OVERVIEW.md:200-201`). Opt-in via `/reason on|off` (default off). Each model-calling stage is a **separate** `/v1/messages` POST so the KV cache resets (Agentless arXiv:2407.01489) (`B-39_chat_reflect_pipeline.md:16-19`, `:36-39`). SYNTHESIZE is **local string selection**, not a fifth inference call (`:239-243`, `:387-388`).

**Designed REFINE cap = 2** (RefineBench arXiv:2511.22173 diminishing returns past iteration 2; feature map; this file’s rebuild contract) (`B-39_chat_reflect_pipeline.md:31-32`, `:172`, `:228`, `ELAI-FEATURE-MAP.md:610-611`). A later C-33 note ships cap=3 — **not** this organ’s design; see §16.

## 2. Why it exists

`LiveCli::run_turn` is a passthrough: user → sidecar → streamed reply. No grounding, sampling, or audit on chat turns (`B-39_chat_reflect_pipeline.md:43-48`). Plan-execution GRC is already wired; chat is the hole (`:50-55`). If REFLECT is missing, chat answers are unsampled single-pass. If it is faked (one mega-prompt “think then verify” sharing KV), prompt injection from user content pollutes the verifier (AISLE) (`:36-39`). If SYNTHESIZE is another sidecar call, the 20K budget contract is broken (`:387-388`). Errors must **fall through** to plain chat so `/reason` cannot brick the REPL (`:245-247`).

## 3. How it works (runtime)

1. **Who calls it.** Top of `LiveCli::run_turn` (cited `live_cli.rs:1659`) **before** `runtime.run_turn`, when `reflect_enabled == true` (`B-39_chat_reflect_pipeline.md:257-276`, `:378-380`). `ConversationRuntime` is a field type, not a `send_message` namespace (`:44-46`).
2. **Gate.** `CliSession` (or equivalent) holds `reflect_enabled: bool` default **false** and `reflect_config: ChatReflectConfig` (`:265-266`). Slash spec: **one** entry `name="reason"` `argument_hint=Some("<on|off>")`; on/off validated in `handle_repl_command`. Two specs named `reason` cannot coexist (`:267-271`, B39-044). `slash_command_specs().len()` 146 → 147 (`:277-278`).
3. **`pipeline.run(turn_id, user_message, provider)`.** On `skipped=false` success, stream `ReflectOutput.final_reply`. On `skipped=true` **or** `Err`, fall through unchanged (`:272-276`). Stage errors are `ReflectError::StageFailed { stage, source }` — no panic, no `.unwrap()` (`:245-249`).

### Stage 1 — TRIAGE (~1.5K tok)

4. Single `/v1/messages` with a **fixed** system prompt; seed = **turn content only**. Return one of `"chat" | "problem" | "code-fix"` (`:211-214`). Map to `TurnClass { Chat, Problem, CodeFix }` (`:166`).
5. If `Chat`: `skipped=true`, jump to SYNTHESIZE with `user_message` as-is (zero overhead); stages 2–4 token slots = 0 (`:214-216`, contract test 1).
6. Emit `ReflectTriaged` (`:217`). Exactly one per run (test 9).

### Stage 2 — REASONER (N=4, clean ctx)

7. For `i in 0..n_candidates` (default 4, range `1..=8`): POST `/v1/messages` with role `chat_reasoner` `AgentGrcConfig`, system prompt `"reason about the following problem step by step"`, **no prior chat history** (`:219-223`, arXiv:2407.01489). N=4 is Liu et al. 2502.06703 inflection for 3B–7B (`:20-22`).
8. Emit one `ReflectCandidate` per candidate (`:224`). Token guard: if running total exceeds `budget_cap_tokens * 0.85`, **abort remaining candidates**, proceed with what was collected (`:225-226`).
9. Estimate “~8K tok each” **cannot** fit 4×8K into 20K; contract allows overshoot by **at most one candidate** — 20K is a launch guard, not a never-exceed hard cap (`:219`, `:335-337`, B39-034).

### Stage 3 — AUDITOR (REFINE cap=2)

10. Clean-ctx POST with role `chat_auditor`. Prompt includes **all N candidates**, grounded evidence (tool stdout from `contracts/tools/` if accessible, **else empty**), scoring instruction (Generative Verifiers arXiv:2408.15240 — score traces, do not re-solve) (`:228-232`, `:27-30`). Returns `best_index` + `confidence`.
11. If `confidence < 0.6` AND `refine_iteration < max_refine_iterations` (designed **2**): one REASONER retry with rejected candidates as **negative examples**, then re-audit; increment `refine_iteration` (`:234-236`). Emit `ReflectAuditVerdict` after **each** audit call (`:237`). Loop never exceeds cap regardless of confidence (`:464-465`).
12. Evidence-free auditing is theoretically resampling (Kamoi TACL 2406.01297) but the stage **allows empty evidence** when tools are inaccessible (`:27-30` vs `:228-233`, B39-006) — see §16.

### Stage 4 — SYNTHESIZE (local)

13. `final_reply = candidates[best_index]`. Format for existing streaming render. **No sidecar call** (`:239-243`). Emit `ReflectRefined` with full token summary (`:243`). `stage_tokens[3]` may be 0; “all four > 0” on problem turns is an acceptance line that conflicts with local synth — see §16.
14. **B-39C neighbor (answer substitution):** when `!skipped`, render Auditor-selected Markdown via `TerminalRenderer::render_markdown` and **bypass** `runtime.run_turn` entirely (text-only, no tools). When skipped, fall through. `ConversationRuntime` gains `push_user_text`, `push_assistant_text`, `record_pipeline_usage` (`B-PLAN-SUMMARY.md:838-843`).

The model generates triage class, candidate text, auditor best_index/confidence. It may **not** disable isolation, raise N above config, skip the 0.85 abort, or exceed REFINE cap=2.

## 4. Types, enums, objects

Copy from `B-39_chat_reflect_pipeline.md:87-203`, `:140-149`. No new GRC enums.

```
TurnClass { Chat, Problem, CodeFix }

ChatReflectConfig {
    enabled: bool,                 // false = bypass all stages
    n_candidates: usize,           // default 4, must be 1..=8
    budget_cap_tokens: u32,        // default 20_000
    max_refine_iterations: u8,     // designed default 2
}

ReflectOutput {
    final_reply: String,
    turn_class: TurnClass,
    token_summary: ReflectTokenSummary,
    skipped: bool,                 // Chat triage bypassed stages 2-4
}

ReflectTokenSummary {
    stage_tokens: [u32; 4],        // TRIAGE, REASONER, AUDITOR, SYNTHESIZE
    total: u32,                    // == sum of stage_tokens
}

ChatReflectPipeline {
    config: ChatReflectConfig,
    reasoner_grc: PipelineGrc,
    auditor_grc: PipelineGrc,
    ssb: Option<Arc<Ssb>>,         // None in tests
}

run(turn_id, user_message, provider) -> Result<ReflectOutput, ReflectError>
ReflectError::StageFailed { stage, source }

# chat_reasoner AgentGrcConfig (production lookup agent_grc_config_for_role)
budget: GrcBudget::new(4, 8192)
required_steps: [Observe, Hypothesize, PlanAction]
grounding_level: High              // Medium DOES NOT EXIST
csq_required: false
allowed_methods: [FileCheck, SourceCheck]

# chat_auditor
budget: GrcBudget::new(4, 5120)
required_steps: [Observe, Verify]
grounding_level: High
csq_required: false
allowed_methods: [FileCheck, SourceCheck, TestCheck]
transition_overrides: Observe -> [Verify]   // media_validator pattern

# SSB payloads (all Serialize, Deserialize, Debug, Clone)
ReflectTriagedPayload { turn_id, class, token_count, skipped_to_synthesize }
ReflectCandidatePayload { turn_id, candidate_index: u8, token_count, summary } // summary ≤ 200 chars
ReflectAuditVerdictPayload { turn_id, best_index, confidence: f32, refine_iteration: u8, evidence_sources: Vec<String> }
ReflectRefinedPayload { turn_id, total_tokens_used, stage_tokens: [u32; 4], budget_remaining }

SsbActionType::{ ReflectTriaged, ReflectCandidate, ReflectAuditVerdict, ReflectRefined }
SCHEMA_VERSION 28 -> 29   // designed bump; later integers exist — §16
```

**WIRING DIVERGENCE (specified, not invented):** implementation uses local `ChatInferenceClient` + CLI `SidecarChatClient` instead of `elai-providers` on the reasoning crate (`B-39_chat_reflect_pipeline.md:381`, `:365-367`, B39-023). Rebuild either the original `&dyn LlmProvider` **or** this equivalent adapter; do not add a circular orchestrator↔reasoning↔providers cycle.

`get_reflect_state(): ReflectState { enabled, last_triage, last_verdict, last_summary }` (`:310-313`).

## 5. Control flow / state machine

**Owner:** `ChatReflectPipeline::run`. Orchestrator/FSM is **not** in this path unless B-39E restores tools.

```
/reason off  --> runtime.run_turn (B-14 passthrough)
/reason on:
  TRIAGE
    Chat     --> skipped=true --> SYNTHESIZE(user_message) --> ReflectRefined
    Problem | CodeFix
      REASONER i=0..N-1 (abort rest at 85% cap)
      AUDITOR
        confidence >= 0.6 OR refine_iteration == cap --> SYNTHESIZE(candidates[best])
        else REASONER retry (negatives) --> AUDITOR  (cap=2 designed)
  any StageFailed --> passthrough runtime.run_turn
```

**Forced:** per-stage KV reset = separate POST for TRIAGE, each REASONER candidate, each AUDITOR call. SYNTHESIZE is not a POST (`:16-19` vs `:239-243`, B39-039).

**B-39C:** non-skipped success **does not** call `runtime.run_turn` (text-only). That is a specified follow-on, not a cousin, but it means B-20/B-41 tool/audit paths do not run on substitute turns (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-001/X-010).

## 6. Grounding, receipts, gates

REASONER/AUDITOR steps still walk 2A GRC under their `AgentGrcConfig` (High floor; auditor Observe→Verify override). AUDITOR is supposed to receive test/tool/file evidence before scoring (`:27-30`); empty evidence is allowed if inaccessible (`:230-231`).

**Never allowed as a confidence signal:** auditor `confidence` as a pass to skip GRC; using chat history as shared KV; treating skipped Chat as “already reasoned”; scalar self-eval instead of best-of-N.

`/reason` is the activation gate (default off) (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-002/X-019). Failures **fall through** (observation-like), they do not veto the chat (`B-002/X-016`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `n_candidates` | default **4**, range 1..=8 | per problem/code-fix turn |
| AUDITOR REFINE cap | **2** (designed) | per turn |
| confidence retry threshold | `< 0.6` | per audit |
| `budget_cap_tokens` | **20_000** | per pipeline run |
| local hard ceiling | 25_000 (`ELAI_LOCAL_MAX_CONTEXT`) | leaves ≥ 5K headroom (`:33-35`) |
| abort remaining candidates | running total `> 0.85 * cap` | REASONER |
| TRIAGE size | ~1.5K tok | one call |
| `chat_reasoner` GRC | **4 / 8192** | per reasoner engine |
| `chat_auditor` GRC | **4 / 5120** | per auditor engine |
| candidate summary | ≤ 200 chars | SSB |
| remaining gauge red | `< 10%` of 20_000 | drawer |
| slash spec count | 147 (was 146) | CLI |
| later: `REFLECT_BUDGET_SOFT_WARN_PCT=85` + gated `ReflectBudgetSoftWarn` | B0-081 | not in original tasks |

## 8. Roles and neighbors

**New roles (string keys, not AgentRole enum variants):** `chat_reasoner`, `chat_auditor` on **`agent_grc_config_for_role`** (production path). Adding only `TomlConfigLoader::load_config` **silently misses** live wiring (`:79-85`, B39-011). Mirror `media_validator` arm (`:87-88`).

**Consumes:** `PipelineGrc` (B-8 `new_from_default`), `BasicGrcEngine`, 2C types, SSB, `LiveCli::run_turn`, B-14 streaming (`:371-377`).

**Produces:** pipeline types; two role configs; four SSB variants; `/reason`; REFLECT panel; `get_reflect_state`.

**Must not fork:** a second chat GRC inside CLI that bypasses `ChatReflectPipeline`; shared-context “one call does all four stages”; AB-MCTS as the default REASONER (deferred B-39b, 3× cost) (`:520-523`).

## 9. Config / flags / defaults

- `/reason` default **off** (`enabled: false`).
- `n_candidates=4`, `budget_cap_tokens=20_000`, `max_refine_iterations=2` (designed Default).
- Live `get_reflect_state` Tauri: stub in B-39 Task 7; live invoke deferred Track D (`:301-304`, `:531-532`). Rebuild: typed stub shape is in-contract; live IPC is neighbor.
- Soft-warn `ELAI_B39_SOFTWARN_ENABLE` is a later B0-081 add-on, not original tasks.
- Decision doc `workflow/reasoning/B-39-decisions.md` before EXECUTING (`:537-552`).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `reasoning/src/chat_reflect.rs` | `:152-203` |
| `pub mod chat_reflect` in `lib.rs` | `:251-255` |
| `agent_grc_config_for_role` arms | `:77-101` |
| `orchestrator/src/ssb.rs` 4 variants + payloads + SCHEMA 29 + DDL CHECK + `test_payload()` | `:106-136` |
| `cli/src/live_cli.rs` gate | `:257-276` |
| `cli/src/commands.rs` SlashCommand reason | `:267-278` |
| `ChatInferenceClient` / `SidecarChatClient` | `:366-367`, `:381` |
| `ReasoningDrawer.svelte` REFLECT section | `:280-304` |
| `stub.ts` `get_reflect_state` | `:308-313` |
| `contracts/tests/chat_reflect_contract.rs` | `:320-351` |

Cargo: original wanted `elai-providers` on reasoning; shipped equivalent is `elai-reasoning` on cli (`:157-162`, `:486`, `:381`).

## 11. Journal / SSB events

| variant | payload | cardinality |
|---|---|---|
| `ReflectTriaged` | turn_id, class, token_count, skipped_to_synthesize | exactly 1 / run |
| `ReflectCandidate` | turn_id, candidate_index 0..N-1, token_count, summary≤200 | exactly N on problem (or fewer if 85% abort) |
| `ReflectAuditVerdict` | turn_id, best_index, confidence, refine_iteration, evidence_sources | ≥1 per problem; after **each** audit |
| `ReflectRefined` | turn_id, total_tokens_used, stage_tokens[4], budget_remaining | exactly 1 / run |

SCHEMA_VERSION designed 28→29; update CHECK constraint + exhaustive `test_payload()` in the **same** change (`:132-135`). Observable timing: Triaged ≤500ms; 4 candidates ≤30s; Audit ≤60s; Refined ≤65s (`:390-395`).

## 12. Tests and observables

Minimum **12** contract tests, `MockLlmProvider`, no live sidecar (`:318-351`):

| # | Name | Assert |
|---|---|---|
| 1 | `triage_chat_class_skips_stages_2_to_4` | skipped; stage_tokens[1..3] all 0 |
| 2 | `triage_problem_class_runs_all_stages` | skipped=false; all four stage_tokens > 0 |
| 3 | `triage_code_fix_class_runs_all_stages` | code-fix runs all |
| 4 | `n_candidates_produces_n_ssb_rows` | exactly N ReflectCandidate |
| 5 | `auditor_selects_best_index_in_range` | best_index < n_candidates |
| 6 | `refine_loop_capped_at_2` | confidence always <0.6 → refine_iteration ≤ 2 |
| 7 | `budget_cap_aborts_remaining_candidates` | does not exceed cap by more than one candidate |
| 8 | `stage_tokens_sum_matches_total` | total == sum |
| 9 | `reflect_triaged_ssb_row_emitted` | exactly 1 |
| 10 | `reflect_audit_verdict_ssb_row_emitted` | ≥1 on problem |
| 11 | `reflect_refined_ssb_row_emitted` | exactly 1 |
| 12 | `pipeline_error_falls_back_cleanly` | REASONER HTTP 500 → StageFailed, no panic |

Unit: `TurnClass` From<&str>; `ChatReflectConfig::default()`; token-sum invariant; `StageFailed` Display (`:478-482`). Deterministic: zero `.unwrap()` in `chat_reflect.rs`; SCHEMA 29; ELAI_SEED=42 triage stability (`:399-441`). UI: Svelte 5 runes only, no `$:` (`:300`).

B-39C named contracts: `substitute_final_reply_when_not_skipped`, `runtime_history_includes_pipeline_reply`, `skipped_falls_through_to_runtime`, `no_tool_use_blocks_in_pipeline_reply` (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B0-044).

## 13. Rebuild contract

Another harness has **this** organ iff:

- Pipeline TRIAGE → REASONER **N=4** → AUDITOR REFINE **cap=2** → SYNTHESIZE (local).
- Per-stage KV reset on every model call; no shared chat history into REASONER.
- `chat_reasoner` 4/8192 High FileCheck+SourceCheck; `chat_auditor` 4/5120 High FileCheck+SourceCheck+TestCheck Observe→Verify; lookup via `agent_grc_config_for_role`.
- `/reason on|off` single spec, default off; intercept `run_turn`; Chat skip; error passthrough; no production unwrap/TODO.
- 20K cap, 85% candidate abort, 25K ceiling headroom.
- Four SSB payloads/variants; token total = sum of four stages.
- Tests 1–12 pass with cap=2 asserted.
- Drawer REFLECT: toggle teal, class badges (CHAT gray / PROBLEM amber / CODE-FIX blue), N bars, confidence, refine count, 4-cell tokens, remaining vs 20K red <10%.

A single “chain of thought then answer” call, or MCTS-first REASONER, is a cousin (AB-MCTS is explicitly deferred).

## 14. Non-goals / banned designs

- New weights / verifier-only 7B model (`:5-6`).
- SYNTHESIZE as inference (`:387-388`).
- Two slash specs named `reason` (`:270-271`).
- TomlConfigLoader-only role arms (`:82-85`).
- AISLE/Mythos eval suite, `/think hard` s1 forcing, prefix-cache 30% save, B-39D streaming UI, B-39E tool-loop hint, B-39F compaction (`:509-535`, `B-PLAN-SUMMARY.md:844-847`).
- Using REFLECT substitute path as if tools/B-41 ran (they do not until B-39E).

## 15. Source citations

Primary: `docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md` (full file). Index: `B-PLAN-SUMMARY.md:817-847`, `:991`. Feature map: `ELAI-FEATURE-MAP.md:610-612`. Overview: `ELAI-SPEC-OVERVIEW.md:154-156`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:200-201`. Exhaustive: B39-001..B39-077, SUM-0311..SUM-0322. Neighbors: B-5, B-8, B-14. Papers as cited in B-39 spec references block (`:16-39`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- **REFINE cap 2 vs 3.** Designed/feature-map/tests 6 + acceptance = 2 (`:31-32`, `:172`, `:333-334`, `:464-465`, `ELAI-FEATURE-MAP.md:610-611`). Decision doc C-33 says shipped cap=3 and contract asserts 3 (`:543-546`, B39-037). Rebuild the **designed organ at 2**; do not silently switch to 3 without a spec amendment in this file’s contract.
- Test 2 “all four stage_tokens > 0” vs SYNTHESIZE local (stage[3] may be 0) (`:327-328` vs `:239-243`).
- Evidence mandatory vs empty-if-inaccessible (B39-006).
- SCHEMA 28→29 vs later bus integers (B0-035).
- `ChatReflectPipeline` fields list `PipelineGrc` from `grc.rs` while the module lives in `reasoning/` — how reasoning depends on orchestrator GRC (providers dep vs ChatInferenceClient) is the documented divergence (`:188-192`, `:381`).
- Whether TRIAGE itself is GRC-stepped or a raw classifier (`:211-214` — no GRC role for triage).
- `best_index` type u8 vs usize N.
- B-39C bypass vs original “use final_reply as content to stream” through runtime (`:274-275` vs B-39C). Both specified; substitute is the later chat behavior.
- Soft-warn event / `ELAI_B39_SOFTWARN_ENABLE` (B0-081) not in original tasks.
- How `class: String` on `ReflectTriagedPayload` canonicalizes vs `TurnClass`.
