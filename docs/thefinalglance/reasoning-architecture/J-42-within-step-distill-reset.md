# Within-step distill-reset reasoning (J-42)

- **ELAI ids:** J-42 / WAVE DD-4 / checkpoint_and_reset / DistillateAdmissionCheck / ReasoningCheckpointEventKind
- **Spec status (as designed):** PLANNED BUILD (extend GRC `reset_topic` + B-39 refine loop). GROUNDED ABSENT as a within-step operator at plan time; live cousin: `crates/reasoning/src/distill.rs` admission enums + `ReasoningCheckpointEvent` SSB, while `reset_topic` still only **removes** the topic (`J-plan/J-42_within_step_distill_reset_reasoning.md:1-5`, `engine.rs:561-563`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:258-260`).
- **Crate / module (spec):** `crates/reasoning` (`checkpoint_and_reset` over `GrcEngine::reset_topic`); B-39 `chat_reflect` distill-reset **mode**; H-50 trigger; H-42 per-pass budget; D-16 uncertainty guard.
- **Purpose of this file:** enough architecture to rebuild InftyThink sawtooth **within one** GRC/planning/research step. Not a status ledger.

## 1. What it is

**Within-step distill-reset** is the DD-wave **reasoning** axis: inside a **single** GRC/planning/research step, the model generates a reasoning segment **and** proposes a compact progress-distillate; a **deterministic** trigger (context-fill / GRC budget) decides **WHEN** to checkpoint; a **frozen admission check** (bounded size + non-empty + **uncertainty-preserved**) decides whether the distillate is acceptable; the harness then **RESETS** working context and re-seeds `{pinned original goal + admitted distillate}`. That is InftyThink’s sawtooth (bounded context, unbounded depth) at **step** grain. **Bound-and-reset, never let-the-model-decide-when.** A distillate that **strips all uncertainty FAILS admission** (`J-plan/J-42_within_step_distill_reset_reasoning.md:4,10`, `J-plan/J-SUMMARY_INDEX.md:67-74,1117-1138`).

Sibling of J-39 (nodes) and J-40 (research steps). E-25/H-50 are **session-grain** distill-and-reset; J-42 is **within-step** (`J-plan/J-42_within_step_distill_reset_reasoning.md:46`).

## 2. Why it exists

Today a step runs straight-line until the local window or GRC budget, then truncates or fails. The model never decides when to compress-and-continue **because it must not** — the harness never reset-and-re-reasoned (`J-plan/J-42_within_step_distill_reset_reasoning.md:4`). B-39 **appends** prior verdicts. `reset_topic` clears budget/steps **only**, no distillate carry-forward (`engine.rs:561-563`). If this organ is missing, long reasoning dies at the wall. If faked (model says “I’ll summarize now”, JSON-constrained distillate, or a distillate that drops every hedge), you get attractor drift (2601.08846) and self-distill **uncertainty suppression** (OOD drop up to 40%, 2603.24472) (`J-plan/J-42_within_step_distill_reset_reasoning.md:10,39-40`).

SWEnergy: FIT more depth into bounded context ≠ CAPABLE. Hard reasoning still I-7 escalates (`J-plan/J-42_within_step_distill_reset_reasoning.md:26`). InftyThink paper gains came **with SFT**; prompted-only may be weaker — EXP-3 decides default-ON (`:12,26,54`).

## 3. How it works (runtime)

1. **Who calls it.** PipelineGrc reasoning-step hot paths; B-39 auditor refine loop; `/reason`; planning/research step loops (`J-plan/J-42_within_step_distill_reset_reasoning.md:31`).
2. **Model proposes** reasoning + **free-form** distillate. HD-3 **bans** constrained-decode / force-JSON of a reasoning-role output. Harness parses **best-effort** (chat_reflect verdict-parser idiom), never constrains decode (`:10,32`).
3. **WHEN (deterministic trigger — not the model).** Fire checkpoint when:
   - topic working-context fill crosses a fraction of the **lane ceiling** (`ELAI_LOCAL_MAX_CONTEXT * trim_fraction` local; `CLOUD_TRIM_FRACTION=0.85` at `context_policy.rs:70` / H-50), **OR**
   - GRC topic budget (`max_chars` / `max_calls`) **nears exhaustion**
   Per-pass budget from H-42 `ReasoningBudgetScaled`. **NEVER** the model saying “I’ll summarize now” (`:9,32`). Live `CheckpointTrigger { ContextFill, GrcBudget }` (`distill.rs:21-28`); plan says GrcEngine operator is later tranche for `GrcBudget` (`distill.rs:26-27`).
4. **Admission (frozen DistillateAdmissionCheck).** Admit iff **all**:
   - bounded to the sawtooth **floor size cap**
   - **non-empty**
   - **uncertainty-preserved**: open questions / flagged hedges / abstention markers present in the **pre-reset trace survive** into the distillate (2603.24472), cross-checked against D-16 `RefusalDecision` / H-39 `AbstentionScored`
   A distillate that **strips all uncertainty FAILS** → **re-distill once**, else **escalate (I-7)** / fall back to straight-line truncation. **Never pass** a failed admission (`:10,32`). Live: `DistillateVerdict { Admitted, RejectedEmpty, RejectedOversize, RejectedUncertaintyStripped }` (`distill.rs:7-16`).
5. **Pin the ORIGINAL goal** in every re-seed (not only the rolling distillate) against directional-attractor drift (`:10,40`).
6. **checkpoint_and_reset(topic).** Capture proposed distillate → admission → `reset_topic` (clear budget/steps) → re-seed `{pinned original goal + admitted distillate}`. `reset_topic` **stays** the budget-clearer; checkpoint is the **superset** (`:8,26`). Live `reset_topic` = `self.topics.remove(topic)` (`engine.rs:561-563`) — no carry-forward yet.
7. **B-39 host.** Today append prior verdict, converge `next_best==prev_best` (plan pin `chat_reflect.rs:409-465` — **re-grep**). J-42 adds distill-reset **mode** as the **deep/local** path: checkpoint between passes **instead of** append (`:11`). RSA population-aggregation remains the **parallel N-best sibling** (B-39 N=4), **not** this operator (`:12`).
8. **Pass-count cap.** Like J-39 `MAX_SPLIT_DEPTH`: cap reset-passes; require measurable per-pass progress or escalate (`:56`).
9. **LOCAL vs CLOUD.** Reset that discards the prefix **burns** cloud cache. J-42 is **LOCAL-lane-first**; cloud **append-mode** / keep-cache-prefix (H-50 opposite policies). Decide split at edit (`:57`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:258-260`).
10. **EXP-3 observe-only first.** Local 3B/7B, MATH500/AIME24/GPQA_diamond + coding-reasoning slice; distill-reset vs straight-line at **EQUAL total token budget**. Win = accuracy ≥ straight-line at **LOWER peak context** **AND** no abstention/uncertainty degradation. Else stay observe-only / context-economy-only. No default-ON until FP/over-reset **and** abstention-quality bounded on ≥3 runs (`:7`).
11. **Commit.** One `ReasoningCheckpointEvent` per checkpoint `{trigger: context_fill|grc_budget, distillate_tokens, uncertainty_preserved: bool, pass_index, outcome: ResetAndContinued|DistillateRejected|EscalatedToCloud}` (`:33-34`). Live Kind labels match (`distill.rs:44-54`, `ssb.rs:1664-1666`).

The model may generate reasoning and a free-form distillate. It may **not** fire the reset, pass its own admission, constrain-decode the distillate, or strip uncertainty and still continue.

## 4. Types, enums, objects

Copy from `J-plan/J-42_within_step_distill_reset_reasoning.md:29-34` and live `distill.rs`.

```
fn checkpoint_and_reset(topic)   // superset of GrcEngine::reset_topic

DistillateAdmissionCheck
  // bounded size + non-empty + uncertainty-preserved

ReasoningCheckpointEventKind { ResetAndContinued, DistillateRejected, EscalatedToCloud }

ReasoningCheckpointEvent {
  trigger: context_fill | grc_budget,
  distillate_tokens,
  uncertainty_preserved: bool,
  pass_index,
  outcome: ReasoningCheckpointEventKind,
}
```

**Live names (same organ, do not invent extras):**

```
DistillateVerdict { Admitted, RejectedEmpty, RejectedOversize, RejectedUncertaintyStripped }
CheckpointTrigger { ContextFill, GrcBudget }
CheckpointOutcome { ResetAndContinued, DistillateRejected, EscalatedToCloud }
fn distill_threshold(budget_cap_tokens, pct) -> u32    // distill.rs:71-72
```

**Consumed:** `GrcEngine::reset_topic`; B-39 `ChatReflectPipeline`; H-50 `ContextPolicy`; H-42 `ReasoningBudgetScaled`; D-16/H-39; I-7 `EscalationCascade`.

## 5. Control flow / state machine

**Owner:** harness trigger + admission. GRC loop still 2A. B-39 still owns refine **when** append-mode.

```
step running (GRC / plan / research / /reason)
     │
     ▼
deterministic trigger?  (fill fraction OR GRC must-converge-near)
     NO → continue straight-line
     YES → model proposes FREE-FORM distillate (parse best-effort)
            │
            ▼
         DistillateAdmissionCheck
            ├─ fail (empty / oversize / uncertainty stripped)
            │     re-distill ONCE
            │     still fail → DistillateRejected (append/truncate) OR EscalatedToCloud
            │     NEVER admit
            └─ pass
                  pin ORIGINAL goal
                  reset_topic (clear budget/steps)
                  re-seed {goal + distillate}
                  ResetAndContinued
                  pass_index++
                  if pass cap or no progress → I-7
LOCAL: reset-mode. CLOUD: append-mode (keep prefix) unless edit decides otherwise.
```

**Forced:** trigger is never model-decided. Uncertainty-stripped cannot continue as ResetAndContinued.

## 6. Grounding, receipts, gates

Admission is a **harness receipt** over the distillate vs the pre-reset trace (hedges must survive). H-39/D-16 bits are **read**, not model self-grade. Reasoning **trace** is monitoring (RR-1); it is **not** the confidence signal for J-83 commit.

**Never allowed:** model-timed reset; force-JSON distillate; silent SKIP of failed admission; treating “shorter summary” as success if uncertainty vanished.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| Local ceilings | 3B ~40K / 7B ~25K | lane (`:23`) |
| Cloud trim fraction | 0.85 | H-50 (`context_policy.rs:70`) |
| GRC default | 5 calls / 800 chars | 2A neighbor |
| Size cap | “sawtooth floor” — numeric **UNCERTAIN** | distillate |
| Re-distill | **once** then escalate/truncate | admission fail |
| Pass cap | required, like MAX_SPLIT_DEPTH; numeric **UNCERTAIN** | `:56` |
| EXP-3 | equal total tokens vs straight-line; ≥3-run FP+abstention bound | default-ON gate |
| SSB | +ReasoningCheckpointEvent; sequential after J-39 if it ships first | W-9 `:34` |
| InftyThink +3–13% | Qwen2.5-Math-7B paper | **EXTERNAL WHY** |

Live `distill_threshold` uses a `pct` of stage cap (`distill.rs:68-72`); plan’s exact fraction vs H-50 trim is **UNCERTAIN**.

## 8. Roles and neighbors

Deep/local reasoning path. Cloud prefers append. Auditor refine is the B-39 host.

**Consumes:** 2A GRC; B-39; E-25/H-50; H-42; D-16/H-39; I-7; J-39/J-40 discipline.

**Produces:** re-seeded topic; checkpoint SSB.

**Must not fork:** a second “please summarize your thoughts” prompt as the trigger.

**Excluded:** InftyThink 333K SFT (RUNPOD-PENDING); RSA as this operator; constrained-decode (`:12`).

## 9. Config / flags / defaults

- Preferred: default-ON **observe-only emit** (no flag); allowlist only if gated (`:34`). Conflicts with EXP-3 no default-ON until abstention holds (`:7,26`) — §16.
- LOCAL reset vs CLOUD append — decide at edit (`:57`).
- `/reason` path reachable (`:31`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `checkpoint_and_reset` | EXTEND `GrcEngine::reset_topic` (`engine.rs:35,561`) |
| `distill.rs` | live admission/trigger/outcome enums |
| B-39 `chat_reflect` refine loop | distill-reset **mode**; re-grep lines |
| `orchestrator/src/grc.rs:194/:199` | plan wrapper pin (re-grep) |
| `ReasoningTraceEntry` | distillate compresses existing per-step trace |
| H-50 `context_policy.rs:70` | fill trigger |
| H-42 `ReasoningBudgetScaled` | per-pass budget |
| `j42` SSB `ReasoningCheckpointEvent` | live lock `action_type_lock.rs:1606-1607` |

## 11. Journal / SSB events

Plan (`:33-34`):

```
ReasoningCheckpointEvent {  // Pattern-2, 3-arm Kind
  trigger: context_fill | grc_budget
  distillate_tokens
  uncertainty_preserved: bool
  pass_index
  outcome: ResetAndContinued | DistillateRejected | EscalatedToCloud
}
```

Live Kind already in `SSB_ACTION_TYPES` (`ssb.rs:1664-1666`). Rebuild: **one row per checkpoint**; `uncertainty_preserved` is load-bearing. Sequential bump after J-39 if needed. Do not rewind 268/577.

## 12. Tests and observables

Implied by plan (no T1–Tn table):

| Assert | |
|---|---|
| Trigger is fill or GRC budget, **not** model speech | stub_ban |
| Empty / oversize distillate | reject |
| Uncertainty stripped | `DistillateRejected` / `RejectedUncertaintyStripped`; **not** reset-continue |
| Failed admission | re-distill once then escalate/truncate; **never pass** |
| Re-seed contains **original goal** | every pass |
| EXP-3 | peak context down; abstention not degraded else observe-only |
| HD-3 | no grammar/JSON on distillate decode |
| Cloud | does not burn cache prefix |

Live enums already encode the uncertainty-stripped reject — rebuild must **wire** it to the GRC path, not only the auditor slice.

Adversarial: model “CHECKPOINT NOW”; JSON schema on distillate; drop “I’m unsure” and still ResetAndContinued.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- **Within one step** (not only session compaction): generate → deterministic when → admit distillate → reset → re-seed goal+distillate.
- Trigger is harness (fill / budget). **Model cannot choose when.**
- Distillate is free-form; **not** constrained-decoded.
- Admission: size bound, non-empty, **uncertainty preserved**. Strip-all-uncertainty **fails**.
- Failed admission: re-distill once, then escalate/truncate, **never silent pass**.
- Original goal pinned every re-seed.
- Local-first reset; cloud respects cache prefix.
- Observe-only until equal-budget accuracy and abstention quality hold.
- `reset_topic` remains; checkpoint is a superset.

Session compaction, B-39 append-only refine, or “summarize when you feel done” is a cousin.

## 14. Non-goals / banned designs

- Model-decided reset trigger (`:12,32`).
- Constrained-decode / force-JSON of reasoning or distillate (HD-3) (`:12`).
- Silent SKIP on failed admission (`:32`).
- InftyThink SFT as a ship gate for the **inference-time** operator (`:12`).
- RSA population-aggregation as this organ (`:12`).
- Default-ON if EXP-3 abstention degrades (`:26`).
- Claiming FIT = CAPABLE (`:26`).
- Cloud reset that busts the cache prefix without an explicit edit decision (`:57`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-42_within_step_distill_reset_reasoning.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:67-74,1117-1138`. Context: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:258-260`. Live: `engine.rs:35,561-563`, `distill.rs`, `context_policy.rs:70`, `ssb.rs:1664-1666`. Neighbor: `02A-grc-state-machine.md` `reset_topic`. Doctrine: model proposes / harness disposes when+admission (`:51`).

**[EXTERNAL, WHY]:** InftyThink 2503.06692 KEYSTONE; RSA 2509.26626 contrast; 2603.24472 uncertainty suppression; 2601.08846 attractors (`:37-40`). VERIFIED 2026-06-16 in-plan.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric sawtooth size cap; fill **fraction** vs H-50 0.75/0.85; “nears exhaustion” vs 2A `must_converge` (`remaining_calls<=1`).
- Hedge/open-question detector (plan calls the static check **heuristic**; EXP-3 is the real guard) (`:55`).
- Pass-count cap number.
- Cloud append vs reset decision (`:57`).
- Observe-only-emit default-ON (`:34`) vs EXP-3 no default-ON (`:7`).
- Whether live auditor-only distill is enough or GRC `checkpoint_and_reset` must ship in the same slice (`distill.rs:26-27` says later tranche).
- `chat_reflect.rs` line drift (`:17,58`).
- How `uncertainty_preserved: bool` is computed vs `DistillateVerdict::RejectedUncertaintyStripped`.
- Progress metric for “measurable per-pass progress” (`:56`).
- MATH500/AIME24/GPQA as ELAI-hosted suites vs paper names only.
