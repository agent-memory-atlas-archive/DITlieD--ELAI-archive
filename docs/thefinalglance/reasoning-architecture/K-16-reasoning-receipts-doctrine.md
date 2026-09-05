# Reasoning-receipts discipline doctrine (RR-1..RR-4)

- **ELAI ids:** K-16 / WAVE KW-3 / DC-3 / DC-5 / KD-1 / RR-1..RR-4
- **Spec status (as designed):** PLANNED (`ELAI-FEATURE-MAP.md:693-704`). E-23 / doctrine class. **No** runtime gate, **no** new SSB. Auditor-applied. Pre-gates J-83..J-86.
- **Crate / module (spec):** `.claude/rules/reasoning-receipts.md`; trigger in `.claude/rules/conditional-routing.md`; recogniser extension in `.claude/tools/claims_policy_lint.py`. Runtime enforcement is the **cited** shipped F-4 / D-13 / I-4 / H-39 — bound at a new grain, not re-implemented.
- **Purpose of this file:** enough architecture to rebuild the four-clause reasoning-step invariant so another harness cannot silently fall back to LLM self-eval. Not a status ledger.

## 1. What it is

The **reasoning-receipts doctrine** is a binding pre-gate: `.claude/rules/reasoning-receipts.md` with four clauses **RR-1..RR-4**, loaded by conditional-routing on any reasoning-engine / GRC / reasoning-budget / candidate-selection task. It promotes one load-bearing correction to the planning surface: a reasoning **TRACE** is monitoring/UX **only**; the confidence/selection signal must be an **external receipt**. ELAI already lives this in the spine (F-4 cant-lie, D-13 cross-family, I-4 select-by-receipts, H-39 receipt-sourced abstain bit, IC-6) but it was **not** written as a constraint **at the reasoning-step grain**. Without that, a GRC-v3 executor rediscovers it at audit — or silently uses LLM self-eval when receipts run out (`K-plan/K-16_reasoning_receipts_doctrine.md:1-5,16-19`, `K-plan/K-SUMMARY_INDEX.md:413-437`).

## 2. Why it exists

The user’s shape “the part the model is more sure about proceeds” is right except for one word: **sure** must mean **receipt-confirmed**, not chain-self-confident (`:12`). Independent 2024–2026 results (Reflexion/Tsui self-correction blind spot, ToT self-eval as the weak link, Lanham faithfulness, “Consensus is Not Verification”) say a model cannot reliably ground itself or judge which of its own parts is sure (`:5,107-109`). A CoT can be post-hoc rationalization; larger models can be **less** faithful; a model fails to correct its **own** errors while correcting identical errors presented as **external** input (`:12`). If this organ is missing, the natural failure is: no deterministic receipt → LLM grades its own output, or a paper’s 7K knee is imported as ELAI’s ceiling (`:12`). If it is faked (a rule that restates F-4’s mechanics, or a runtime cousin that lets the trace vote), that is a GD-1/IC-16 P0 (`:115`). Near-zero new capability: it is the **discipline** that makes the J GRC-v3 track built correctly from the start (`:5`).

## 3. How it works (runtime)

This organ has **no** product runtime loop. The “runtime” is the **harness pre-gate** plus the **cited** spine.

1. **Who loads it.** `conditional-routing.md` loads `.claude/rules/reasoning-receipts.md` when the task is building/auditing a reasoning-engine / GRC / reasoning-budget / candidate-selection unit (K-8 / K-14 pre-gate pattern) (`K-plan/K-16_reasoning_receipts_doctrine.md:38,55`).
2. **Input.** The task kind (routing trigger) and any positioning copy that claims reasoning-grounding (lint).
3. **RR-1 TRACE-NEVER-ADMISSIBLE (binding).** A reasoning trace / chain-of-thought is monitoring + UX **only**. It is **never** admissible as the confidence or selection signal. The confidence/abstention bit on a reasoning step comes from an **external** receipt (test / checker / pixel-diff / retrieval verdict) read by the engine (F-4 / H-39, RD-1), never set by the chain self-grading (`:17`).
4. **RR-2 SELECTOR HIERARCHY (binding).** Deterministic **RECEIPT** > statistical **VOTE** over **independent** samples (receipt-**absent** fallback **only**, and **never** for factual-recall — recall routes to retrieval; shared-training-bias) > **LLM self-eval (BANNED)**. A vote is never the selector when a receipt exists. LLM self-eval is never the selector at all (IC-6, DC-3, ToT 8B-can’t-self-evaluate rider). The receipt-absent vote rung is the **only** legal place consensus appears (`:18`).
5. **RR-3 MEASURE-YOUR-OWN-BUDGET (binding).** A reasoning-budget / overthinking ceiling / difficulty threshold is **ELAI-measured** per-(model,task-type) on an ELAI gold set (**K-15**), never an imported paper constant (ID-5; the overthinking paper forbids transferring its knee; local lane overthinks earliest/lowest). The **WINDOW** is never the lever for a depth-class reasoning failure (RD-4 / RULER) (`:19`).
6. **RR-4 COMMIT-AND-FREEZE (binding).** A receipt-confirmed reasoning step is **committed and frozen** — the generator does not re-open it (it cannot reliably re-judge it, the blind spot). Commit is simultaneously the grounding, the safe-compression boundary (preserve receipts losslessly, fold prose lossily; never an LLM re-summarization, HD-2; ciphers banned, IC-3), and the overthinking defense (`:20`).
7. **Lint (auditor).** Extend K-1 `claims_policy_lint.py` (compose K-15’s recogniser if K-15 landed first; else K-16 adds it): “self-verifying reasoning / the model checks its own reasoning / chain-of-thought confidence” **FAILS** unless it carries the receipt-not-self-eval correction; a reasoning-budget threshold cited from a paper **FAILS** (RR-3/ID-5) (`:40`).
8. **No new dossier row.** K-15 owns the differentiator claim; K-16 **names the rule** it enforces (RR-1) (`:39`).
9. **Commit (harness).** Rule file exists; conditional-routing trigger wired; lint extended; `workflow/reasoning/K-16-decisions.md` (`:38-42,99-104`).
10. **Fail-closed.** Main-LLM must not self-run the lint. No runtime gate invented here. No re-implementation of F-4/D-13/I-4. No authoring of J-83..J-86 (`:59-63`).

The model may **generate** traces for UX. It may **not** use those traces as confidence, select candidates by self-eval, set a paper knee, or re-open a receipt-frozen step.

## 4. Types, enums, objects

Copy from `K-plan/K-16_reasoning_receipts_doctrine.md:16-19,45-48`. Do not invent runtime structs — this organ is clauses + a lint.

```
// Four closed clauses. Names are load-bearing.

RR-1 TRACE-NEVER-ADMISSIBLE
RR-2 SELECTOR HIERARCHY
     RECEIPT
       > VOTE over INDEPENDENT samples   // receipt-absent ONLY; NEVER factual-recall
       > LLM self-eval                   // BANNED
RR-3 MEASURE-YOUR-OWN-BUDGET             // K-15 per-(model,task-type); NEVER paper constant
                                         // WINDOW is never the lever (RD-4)
RR-4 COMMIT-AND-FREEZE                   // receipt-confirmed step frozen;
                                         // preserve receipts losslessly, fold prose lossily

// Cited runtime archetypes (BIND, do not restate mechanics — IC-16):
F-4  AntiFabricationGate.check()         // task_runner.rs:9565; blake3 stdout_hash [WIRED]
D-13 Adversary cross-family              // never same-model self-critique [DARK]
I-4  evidence-ranked search ladder       // select BY RECEIPTS [DARK]; J-86 activates
H-39 abstention_score.rs                 // abstain bit from EVAL LOG receipt [WIRED]

// Harness artifacts:
.claude/rules/reasoning-receipts.md
conditional-routing.md trigger
claims_policy_lint.py recogniser         // EXTEND K-1; compose K-15 if present
K-plan/_fixtures/reasoning_selfverify_negative.md
```

**Vote rung scope:** independent samples, receipt-absent only, never recall. J-108’s 3-sample vote on the **atomic read** is the same grain (fuzzy read, not hop selector) — K-16 does not re-specify J-108; a rebuild that uses vote **for recall** or **when a receipt exists** is a RR-2 violation.

## 5. Control flow / state machine

**Owner of the loop:** conditional-routing + auditor. Not `GrcEngine`. Not the model.

**When the rule must fire:**

| Task kind | Loads RR-1..RR-4 |
|---|---|
| reasoning-state carrier (J-83) | yes |
| reasoning-effort / budget classifier (J-84) | yes |
| overthinking guard (J-85) | yes |
| candidate selector / ladder (J-86, I-4) | yes |
| reasoning-budget bench (K-15) | yes |
| any later GRC / reasoning-engine unit | yes |

**Forced reject (auditor):**

- Trace used as confidence/selection → RR-1 fail.
- LLM self-eval selector, or vote-when-receipt-exists, or vote-for-recall → RR-2 fail.
- Paper-constant budget / window-as-depth-lever → RR-3 fail.
- Generator re-opens a receipt-confirmed step; LLM re-summarizes receipts; ciphered compression → RR-4 fail.

**No product state machine.** F-4/D-13/I-4/H-39 remain the runtime machines. K-16 does not wrap them in a new FSM (`:5,87`).

## 6. Grounding, receipts, gates

This organ **is** the selector-hierarchy gate at doctrine grain.

**Never allowed as a confidence / selection signal:** reasoning TRACE / CoT, LLM self-rating, “the chain looks sure,” same-family self-critique as if it were D-13, vote when a receipt exists, vote for factual recall, paper 6K/7K/7K-as-ELAI, window fill as a quality score (`:17-19,59-62`).

**Receipt examples named:** test / checker / pixel-diff / retrieval verdict (`:17`). Do not treat that list as closed if a later plan adds a typed receipt; do not invent new receipt kinds here.

**H-39 / RD-1:** abstain bit is **read** from the eval log, never re-derived from the trace (`:25`, `reasoning-architecture/H-39-abstention-aware-scoring.md` §3 step 3).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| clauses | exactly RR-1..RR-4 | the rule file |
| rule file size (plan) | ~55–70 lines / ~65 LOC | `.claude/rules/reasoning-receipts.md` |
| routing trigger | ~2 LOC | `conditional-routing.md` |
| lint add | ~10–15 LOC | compose K-15 if first |
| SSB | **none** | E-23/doctrine; JD-1 |
| runtime code | **none** | auditor-applied |
| effort | ~0.5 session | plan |

RR-3 **defers numeric ceilings to K-15**. This organ must not embed 7K/6K/0.75-window as a reasoning budget. (H-39’s locked `t` is a **scoring** constant for abstention-aware reward, not a reasoning-token knee — do not conflate.)

## 8. Roles and neighbors

Doctrine is role-agnostic. It binds **work on** reasoning engines, not a single agent role.

**Consumes (cite, do not re-author):** K-1 lint; F-4, D-13, I-4, H-39; GRC-v3 run + DC-3/DC-5; CLAUDE.md IC-6 / RD-1 / TD-3 / RD-4 / ID-5 / HD-2 / IC-3; K-8 / K-14 pre-gate shape; K-15 (RR-3 bench + differentiator) (`:49-53`).

**Produces:** the rule, the trigger, the lint extension, the negative fixture, `K-16-decisions.md`.

**Must not fork:** K-1 lint (two recognisers that both rewrite the tool); a runtime “RR engine”; a restatement of F-4’s hash check.

**Pre-gates:** J-83 (RR-1/RR-4), J-84 (RR-3), J-85 (RR-3/RR-4), J-86 (RR-2), I-14 preserve-receipts rider (`:117`).

**Parallel-safe with** K-13, K-14, K-15. If K-15 lands first, K-16 extends its recogniser; if K-16 first, K-15 extends K-16’s — **one** recogniser, two claim shapes (`:116`).

**Frontend:** N/A, W-6 justified (`:64-65`).

## 9. Config / flags / defaults

- No feature flag (W-2 N/A) (`:91`).
- No dashboard.
- Conditional-routing **must** actually load the file on a live reasoning-engine task (JD-1; W-1 is that fire + lint negative FAIL, not file-exists) (`:90`).
- Decision doc `workflow/reasoning/K-16-decisions.md` (`:42`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.claude/rules/reasoning-receipts.md` | RR-1..RR-4 (`:46,99`) |
| `.claude/rules/conditional-routing.md` | load trigger (`:47,100`) |
| `.claude/tools/claims_policy_lint.py` | self-verify + paper-knee recogniser (`:48,101`) |
| `K-plan/_fixtures/reasoning_selfverify_negative.md` | (`:102`) |
| `workflow/reasoning/K-16-decisions.md` | (`:103`) |
| F-4 `AntiFabricationGate` `task_runner.rs:9565` | cited archetype (`:22,108`) |
| D-13 adversary | cited (`:23`) |
| I-4 ladder | cited (`:24`) |
| H-39 `abstention_score.rs` | cited (`:25`) |

PLAN-asserted line `task_runner.rs:9565` re-grep at edit (W-7) (`:108`).

## 11. Journal / SSB events

**None.** E-23/doctrine class. JD-1 harness evidence = the rule fires via conditional-routing; the lint is a deterministic check (`:68,94`).

Do not add `ReasoningReceiptsLoaded` as a “helpful” action_type. That would be a cousin.

## 12. Tests and observables

Observable (`:66-69,82-87`):

| Id | Assert |
|---|---|
| load | a live J-83..J-86 (or equivalent) task loads `reasoning-receipts.md` via conditional-routing |
| clauses present | `rg RR-1\|RR-2\|RR-3\|RR-4` hits the rule file |
| trigger wired | `rg reasoning-receipts.md` in `conditional-routing.md` |
| lint pass | receipt-grounded-and-tagged claim passes |
| lint fail (a) | “ELAI’s chain-of-thought self-verifies” untagged fails |
| lint fail (c) | “the model checks its own reasoning” fails |
| lint fail (d) | reasoning-budget set to paper 7K fails (RR-3) |
| lint fail (e) | empty/missing input fails-loud (R1) |
| auditor (b) | a candidate-selection design that uses an LLM grading its own paths is caught by the **rule** (not necessarily the lint) (`:41,97`) |
| copy hygiene | `rg -ci "self.verif\|chain.*confidence\|model.*grades.*own"` over `docs/strategy/*.md` MUST be 0 (`:78`) |

Contract file optional: `k16_reasoning_receipts_claims_contract.rs` ~40 LOC **or** a Python lint test (`:97`). RR-clauses are auditor-applied until J-83..J-86 exist — no runnable product assertion yet (`:97`).

W-1: conditional-routing **actually loads** + negative fixture **FAILS**, not “the file exists” (`:90`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Four named clauses equivalent to RR-1..RR-4, loaded automatically on reasoning-engine work.
- TRACE is UX/monitoring only; never confidence/selection.
- Selector: external receipt > vote over independent samples **only when no receipt**, never for recall > LLM self-eval banned.
- Budgets/knees are **this harness’s** per-(model,task-type) measurements; paper constants and window-as-lever are forbidden.
- Receipt-confirmed steps freeze; receipts survive compression losslessly; no LLM re-summary of receipts; no ciphers.
- Claims lint fails untagged self-verify and imported paper knees.
- Runtime enforcement is the existing receipt/cant-lie/cross-family/ladder organs, **cited not cloned**.
- Zero new product SSB for the doctrine itself.

A prompt “don’t trust your own CoT,” an LLM-as-judge rubric, or restating a stdout-hash gate inside the rule file as new code is a **cousin**.

## 14. Non-goals / banned designs

- Authoring J-83..J-86 here (`:5,63,113`).
- Introducing a **runtime** gate in this organ (`:5,87`).
- Re-stating F-4/D-13/I-4 mechanics (GD-1/IC-16 P0) (`:5,115`).
- Claiming a measured number (E-23; KD-1) (`:5`).
- Fork of the K-1 lint (`:62`).
- Main-LLM self-run of the lint (`:62`).
- Trace as confidence (RR-1) (`:60`).
- LLM self-eval selector; vote-when-receipt-exists; vote-for-recall (RR-2) (`:61`).
- Paper-constant reasoning budget / overthinking knee (RR-3) (`:62`).
- Window as the depth lever (RD-4) (`:19`).
- LLM re-summarization of receipts (HD-2); ciphers (IC-3) (`:20`).

## 15. Source citations

Primary: `docs/thefinalglance/K-plan/K-16_reasoning_receipts_doctrine.md` (full file). Index: `K-plan/K-SUMMARY_INDEX.md:413-437`. Feature map: `ELAI-FEATURE-MAP.md:693-704`. Reasoning doc: `workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md` FINAL SYNTHESIS (one-paragraph + §3 selector hierarchy + DONE-GATE) + IMPLICATION 7/8/13/20 + ROUND-3 ADD-2 + T1. Grounding map: `_grounding/J-GRC-V3-GROUNDING-MAP.md` DC-3, DC-5, GW. Spine: F-4 `task_runner.rs:9565`; D-13; I-4; H-39. ArXiv EXTERNAL: Reflexion 2303.11366 + Tsui; ToT 2305.10601; self-consistency 2203.11171; “Consensus is Not Verification” 2603.06612; Lanham faithfulness; overthinking 2604.10739. CLAUDE.md: KD-1, IC-6, RD-1, RD-4, ID-5, TD-3, HD-2, IC-3. Siblings: K-8, K-14, K-15, K-1. Neighbor organ: `reasoning-architecture/H-39-abstention-aware-scoring.md`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact conditional-routing trigger **string** (plan gives the task kinds, not the matcher regex) (`:38`).
- Composition order with K-15’s lint recogniser if both land in one changeset (`:40,116`).
- Whether `reasoning_selfverify_negative.md` and K-15’s `reasoning_grounding_negative.md` should share a fixture or stay two files (`:41` vs K-15 `:35`).
- Closed list of receipt kinds (`:17` is exemplary).
- How an auditor **mechanically** catches design (b) (LLM-self-eval selector) before J-86 exists — “reviewed-design catch,” not a code assertion (`:97`).
- `task_runner.rs:9565` live line (PLAN-asserted, W-7).
- Whether RR-4’s compression rider is in-scope for I-14 only (recorded OUT-OF-K) vs any summarizer (`:117`).
- Interaction with J-108’s atomic 3-sample vote (legal under RR-2 if scoped to receipt-absent fuzzy read; this file does not name J-108).
