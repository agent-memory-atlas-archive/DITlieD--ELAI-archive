# Reasoning-method artifact shape (J-44)

- **ELAI ids:** J-44 / WAVE DPW-2 / Track 2 METHOD / C2 / M1–M12 / JD-1
- **Spec status (as designed):** PLANNED BUILD. Rule `reasoning-method.md` and tool `reasoning_artifact_shape_check.py` are **absent** (grep 0). AUX method file **exists**. Depends J-43 (lens/floor sibling — do not re-author here) (`J-plan/J-44_reasoning_method_artifact_shape.md:1-4,18-24`, `J-plan/J-SUMMARY_INDEX.md:1176-1197`).
- **Crate / module (spec):** harness leg — `.claude/rules/reasoning-method.md`; `.claude/tools/reasoning_artifact_shape_check.py`; auditor-hosted random-sample substance-audit panel; injections into spec-eval, planning-workflow, idea-intake, `agent-dispatch-preamble.md`. **Net 0 SSB.**
- **Purpose of this file:** enough architecture to rebuild the METHOD gate (shape + non-author substance audit). Not a status ledger.

## 1. What it is

**Reasoning-method artifact shape** is Track 2 of doctrine-propagation: the **HOW** that produces features, not the WHAT lens (that is DOCTRINE.md / J-43). You cannot regex reasoning **quality**, but the **artifact shape** the method mandates **is** presence-checkable: mandated sections exist, load-bearing claims are tagged. That structure is fail-closed (D3 on the reasoning artifact). **CRITICAL (C2):** a shape gate that only checks sections exist is **theater** unless a **random-sample deep audit of substance by a non-author** exists. Mandated shapes (ADRs, pre-mortems, WHO checklists) improve outcomes **only when audited**; **partial compliance = null effect**. The substance-audit panel is **first-class**, not an afterthought (`J-plan/J-44_reasoning_method_artifact_shape.md:4`, `docs/strategy/ELAI-DETERMINISM-DOCTRINE-AUX.md:1-8,15-28`).

## 2. Why it exists

J-43 propagates the **lens** via code gates. This file propagates the **method** so “determinism” is not pattern-matched as a slogan (`J-plan/J-44_reasoning_method_artifact_shape.md:4`, `ELAI-DETERMINISM-DOCTRINE-AUX.md:5-8`). If this organ is missing, plans and `workflow/reasoning/*.md` can skip ATTACK, stance, falsifier-as-step-1, and honest residue. If it is faked (section headings only, or an LLM **score** on a gating path), C2 theater and D5 self-grade both return (`J-plan/J-44_reasoning_method_artifact_shape.md:16`).

Honest residue (V6): **no** controlled RCT of artifact-shape-alone vs downstream quality in this harness domain. Quality lift is **instrument-and-measure**, not an assertion (`:27,41`). The panel is a real recurring human/cross-family cost (`:27`).

## 3. How it works (runtime)

1. **Who calls it.** AUDITING-state **auditor** (≠ author) runs the shape-check + substance-audit panel over a real reasoning artifact; planning-workflow PRE_GATE on a real plan; spec-eval DECISION-LEDGER on a real design (`J-plan/J-44_reasoning_method_artifact_shape.md:32`).
2. **EXP-DP2 observe-only first.** `reasoning_artifact_shape_check.py` over existing `workflow/reasoning/*.md` + plan files: (a) shape-check **FP rate**, (b) **boilerplate rate** (section PRESENT but empty-of-substance). Boilerplate rate decides whether the panel is load-bearing or optional. **No blocking-default-ON** until FP bounded (GD-4) **and** substance-audit catch-rate observed (`:8`).
3. **Rule `.claude/rules/reasoning-method.md`.** Generalizes planning-workflow plan-improvement to **ALL** reasoning/design/plan artifacts; adds M2/stance/M7/M10 checks that protocol lacks (`:9`). Mandates:

| Id | Artifact requirement |
|---|---|
| M1 | Grounding-before-claim: every **external** claim has a source **or** a tag (`ELAI-DETERMINISM-DOCTRINE-AUX.md:34-39`) |
| M2 | Confidence calibration: every **load-bearing** claim tagged VERIFIED / high-confidence / training-unsearched |
| STANCE | Decision classification: what the **model** decides vs what the **harness** could (`AUX.md:15-28`) |
| M4/M6 | Net-new vs existing: reinvention \| extension \| net-new + named **file:line** seam |
| M7 | Attack-your-own-design: ATTACK section = strongest case against |
| M9 | Falsifier-as-step-1: cheapest falsifying experiment is **literally step 1**; nothing default-ON before FP/over-block ≥3 runs |
| M10 | Honest residue: unverified / provisional / deferred |
| M12 | Write-it-down: reasoning chain + durable lens + handoff |

4. **Tool `reasoning_artifact_shape_check.py`.** Sibling of `doctrine_floor_check.py`. Deterministic **presence/structure**: mandated sections EXIST + load-bearing claims TAGGED. Missing **ATTACK** = FAIL. Untagged load-bearing claim = FAIL. Regex/heading-aware, **necessary-not-sufficient**. Exit-2 fail-closed on missing/empty target. **Observe-only WARN first.** Checks **SHAPE only**; quality is the panel (`:10`).
5. **Random-sample substance-audit panel (C2, load-bearing).** Recurring auditor-side protocol: randomly sample **N** artifacts (+ DP-4 ledger rows); **deep-audit SUBSTANCE** (is ATTACK real vs boilerplate, is the falsifier real, is decision-classification honest). Auditor **≠ author** (measurer ≠ measured). Track **remediation velocity**, not section-present counts. Held-out un-gamed subset. **Partial = null** ⇒ all-or-nothing **within each sampled subset** (`:11`). Flagged cases route to J-45 (advisory, not a gate) (`:48`). Velocity feeds J-46 (`:49`).
6. **Lifecycle injections (extend, never rewrite GD-1)** (`:12-15`):
   - **Design-time:** spec-eval required **DECISION LEDGER** dimension — every decision sorted model-class vs harness-class; a harness-class decision handed to the model is a named **DESIGN WEAKNESS** in spec-eval grammar.
   - **Plan-time:** DOCTRINE PRE_GATE in planning-workflow step 4 (parallel WALKING-SKELETON PRE_GATE) + step 7 (parallel OFF-AGENT PROOF) + idea-intake per-level auditor rubric. Every deliverable declares DECISIONS / MODEL-or-HARNESS / off-agent **DISPOSER** per harness-class decision / DOCTRINE-DEBT row for legitimate-model decisions with no oracle. Producer-only / disposer-less harness-class decision is **REFUSED** at decomposition.
   - **Dispatch-time:** doctrine + method pointer on `agent-dispatch-preamble.md`; R10 **SEAM-DISPOSAL** — a harness-class decision a worker implemented must be disposed by the **harness**, not silently by the model at the seam.
7. **JD-1 done-gate (off-agent).** A real session produced a shape-checked artifact **and** a real non-author substance audit emitted a verdict **and** a remediation-velocity row (`:27`).
8. **Fail-closed.** No section-present-as-quality. No automated LLM quality **SCORE** on a gating path (advisory only, D5/DP-3). Checker WARNs observe-only first; **rule** fails-closed once promoted. No half-enforcement within a sampled subset (`:33`).

The model may **author** the artifact (ATTACK, stance, residue). It may **not** grade its own substance on a gating path, mark the checker as quality, or skip ATTACK.

## 4. Types, enums, objects

Copy from `J-plan/J-44_reasoning_method_artifact_shape.md:9-15,29-33`. Do not invent fields.

```
// mandated presence (shape-check)
ATTACK section
load-bearing claim tags: VERIFIED | high-confidence | training-unsearched
STANCE / decision classification { model-class, harness-class }
DECISIONS / MODEL-or-HARNESS / DISPOSER / DOCTRINE-DEBT

reasoning_artifact_shape_check.py
  exit 2 on missing/empty target
  FAIL: missing ATTACK, untagged load-bearing claim
  observe-only WARN first

substance-audit panel
  sample N artifacts
  auditor != author
  SUBSTANCE-FAIL + remediation-velocity row
  held-out subset; all-or-nothing within subset

DECISION LEDGER          // spec-eval output dimension
DOCTRINE PRE_GATE        // planning-workflow + idea-intake
R10 seam-disposal check  // dispatch
```

**No new SSB action_type** (`:35`).

## 5. Control flow / state machine

**Owner:** auditor for check+panel; planning-workflow/spec-eval/idea-intake/dispatch for injections. J-43 owns the **code** floor. J-45 owns cross-family **advisory**. J-46 owns debt-rate **metric**.

```
author writes reasoning/design/plan artifact
     │
     ▼
EXP-DP2 / observe-only: shape-check WARN
     │
     ▼
once promoted: shape-check FAIL-closed on missing ATTACK / untagged claims
     │
     ├─ PASS shape  — necessary, NOT quality
     └─ FAIL shape  — block (after promotion)
     │
     ▼
random sample → non-author panel
     ├─ boilerplate ATTACK → SUBSTANCE-FAIL + remediation row
     │     (this is the C2 demo: shape-complete-empty PASSES checker, FAILS panel)
     └─ harness-class decision with no disposer
           plan-time PRE_GATE REFUSES
     │
     ▼
flagged substance → J-45 advisory (never gating LLM score)
velocity → J-46
```

**Forced:** negative arm of JD-1 — shape-complete-but-empty **passes the checker** and is caught **only** by the panel (`:34`).

## 6. Grounding, receipts, gates

Shape-check is a **structural receipt** (headings + tags). Substance-audit is a **human/cross-family receipt**, not model self-eval. **LLM scoring quality on a gating path is banned (D5).**

Selector hierarchy for **product claims** remains receipt > vote (receipt-absent) > LLM self-eval banned. This organ must not add “the author is sure the ATTACK is good.”

**Never allowed:** section-present = quality; LLM judge as gate; constrained-decode of the reasoning (HD-3) (`:16`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| EXP-DP2 | FP rate + boilerplate rate over existing reasoning+plan files | observe-only |
| Default-ON blocking | after FP bound **and** panel catch-rate observed | GD-4 |
| Falsifier (M9) | ≥3 runs FP/over-block | before default-ON of a **feature** the artifact designs |
| Sample N | named, numeric **UNCERTAIN** | panel |
| C2 literature | full WHO compliance 18–47% drop; partial = no effect | **PROVISIONAL WHY**, domain-transfer unproven (`:40`) |
| SSB | **net 0** | JD-1 artifacts in audit log |
| V6 | no RCT | do not assert quality lift |

## 8. Roles and neighbors

**Auditor ≠ author** for the panel. Workers doctrine-bound via preamble. Spec-eval / plan-auditor / grill / GRC ledger are **bound surfaces**, not rewritten (`:46`).

**Consumes:** planning-workflow 9-step protocol; spec-eval; idea-intake; agent-dispatch-preamble R-1..R-8 + R10; `auditor.md`; `doctrine_floor_check.py` sibling; AUX M1–M12.

**Produces:** rule, checker, panel protocol, four injections, JD-1 rows (shape FAIL, PRE_GATE refuse, SUBSTANCE-FAIL + remediation).

**Must not fork:** a second plan-improvement rewrite; an LLM rubric that **gates**.

**Depends J-43** (feature-property floor). **Do not implement J-43 in this file.**

## 9. Config / flags / defaults

- Checker observe-only WARN until promotion (`:10,33`).
- Decision ledger **lean inline** in the phase/baby-step file (like PRE_GATE blocks), vs sibling file — decide at edit (`:53`).
- Sampling rate + remediation-velocity SLO are **observed knobs**, not guessed (`:54`).
- Frontend: plan-improvement log + audit log; no new SSB panel required.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.claude/rules/reasoning-method.md` | CREATE |
| `.claude/tools/reasoning_artifact_shape_check.py` | CREATE |
| substance-audit panel protocol | auditor-hosted; CREATE |
| `docs/strategy/ELAI-DETERMINISM-DOCTRINE-AUX.md` | method source (exists) |
| `.claude/skills/planning-workflow/SKILL.md` | EXTEND PRE_GATE + OFF-AGENT PROOF |
| `.claude/skills/spec-eval/SKILL.md` | EXTEND DECISION LEDGER |
| `.claude/skills/idea-intake/SKILL.md` | EXTEND per-level rubric |
| `.claude/rules/agent-dispatch-preamble.md` | EXTEND pointer + R10 seam-disposal |
| `.claude/agents/auditor.md` | host (`:24`) |
| `doctrine_floor_check.py` | sibling shape (J-43) |

## 11. Journal / SSB events

**Net 0** new `action_type`. JD-1 evidence = shape-check **output** + substance-audit **verdict** + remediation-velocity **rows in the audit log** (JC-3/J-27 precedent). Velocity feeds DP-4, not a new SSB stream (`J-plan/J-44_reasoning_method_artifact_shape.md:35`).

## 12. Tests and observables

JD-1 off-agent (`:34`):

| Arm | Assert |
|---|---|
| Real artifact **missing ATTACK** | `reasoning_artifact_shape_check.py` **shape FAIL** |
| Plan deliverable hands sizing/admission/verdict to the model with **no disposer** | planning-workflow DOCTRINE PRE_GATE **refuses** (visible in plan-improvement log) |
| Sampled artifact ATTACK is **boilerplate** | non-author panel **SUBSTANCE-FAIL** + remediation row (audit caught it, **not** the checker) |
| **Negative / C2 demo** | shape-complete-but-empty **PASSES** the checker and is caught **ONLY** by the panel |
| Missing/empty target | checker **exit 2** |
| LLM quality score on gating path | **absent** |

Adversarial: filling every heading with one generic sentence (attack-theater on the checker) — panel hold-out + J-46 implausibly-zero-debt self-test are the named guards, **not solved** (`:55`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A **method** rule (stance + grounding tags + ATTACK + falsifier-as-step-1 + residue) on **all** reasoning artifacts, not only plans.
- A **deterministic** shape checker: sections present + claims tagged; missing ATTACK / untagged load-bearing = FAIL once promoted.
- A **non-author** random-sample **substance** audit with remediation velocity; partial-within-subset = null.
- Shape-complete-empty **passes** the checker and **fails** the panel (C2 demonstrated).
- Plan/design/dispatch seams **extended** so harness-class decisions have an off-agent disposer.
- **No** LLM quality score as a gate. **No** constrained-decode of reasoning.
- Observe-only until FP + boilerplate + catch-rate are measured.

A markdown template without the panel, or an LLM “rate this design 1–5” gate, is a cousin.

## 14. Non-goals / banned designs

- LLM that **scores** reasoning quality on a **gating** path (D5) (`:16`).
- Constrained-decoding the reasoning itself (HD-3) (`:16`).
- Re-authoring planning-workflow plan-improvement (extend only) (`:16`).
- Treating section-present as quality (C2 trap) (`:16,33`).
- Half-enforcement within a sampled subset (`:33`).
- Asserting quality lift without measurement (V6) (`:41`).
- New SSB stream for this harness leg (`:35`).
- Implementing J-43 / J-45 / J-46 in this organ.

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-44_reasoning_method_artifact_shape.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:1176-1197`. Method: `docs/strategy/ELAI-DETERMINISM-DOCTRINE-AUX.md:1-8,15-28,34-39` (confirm remaining M-numbers at edit — plan DG-8). Neighbors: planning-workflow / spec-eval / idea-intake / agent-dispatch-preamble / auditor.md (as cited `:20-24`).

C2 WHO/ADR/pre-mortem literature is **PROVISIONAL** process/medical transfer (`:40`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- AUX M3/M5/M8/M11 full text vs which of those the **shape-check** requires (plan names M1, M2, stance, M4/M6, M7, M9, M10, M12).
- Inline vs sibling DECISION LEDGER (`:53`).
- Sample **N**, sampling cadence, remediation-velocity SLO (`:54`).
- Exact heading strings the regex must match (`ATTACK` vs `## Attack`).
- “Load-bearing claim” detector (untagged FAIL) — heuristic unspecified.
- Whether checker WARNs forever on existing corpus until EXP-DP2, or only on new artifacts.
- Host path of the audit log (no SSB).
- Confirm AUX M-numbering vs plan (plan said UNCONFIRMED this pass; this write confirmed AUX exists and M1+STANCE; remaining M heads **re-grep at implement**) (`:19,56`).
- J-43 floor interaction if a shape FAIL and a doctrine-floor FAIL both fire.
