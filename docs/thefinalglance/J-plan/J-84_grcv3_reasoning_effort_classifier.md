# J-84: GRC-v3 2-axis reasoning-effort classifier — route reasoning by task TYPE × difficulty COST into zero/depth/width/abstain over the H-42 + I-26 budget

STATUS: PLANNED (WAVE GRC-2; M; EXTENSION(H-42 + I-26) — the deterministic classifier that picks the reasoning budget; the user's "complexity classifier + effort toggle"; depends J-83)

Objective: the reasoning run's C5 answer ("reasoning helps a little / a lot / HURTS depending on task + compute; escalate by difficulty") is fully grounded by test-time-compute (2408.03314) + the task-type×gain table from the deep-research report (ROUND-3 ADD 1). J-84 builds the deterministic 2-axis classifier the user asked for and wires it into the SHIPPED budget substrate (H-42 ReasoningBudgetScaled GS-7, I-26 DynamicThinkingBudgets). TWO axes, not one: (TYPE axis) reuse the WIRED D-10 select_grounding_mode (GS-8) to split reason vs RETRIEVE (factual/recall → DKG/A-RAG, NEVER a chain — it confabulates, the zero-shot-CoT CommonsenseQA -4.2pp result) vs validate; (COST/difficulty axis) a deterministic proxy from signals ELAI already has — the H-39 abstention/confidence score (GS-4), the C-45/C-47 case-base hit rate, the J-38 retrieval-coverage gate, the C-29 IntentPair count — routing: trivial→ZERO reasoning (skip the chain, the CoT-hurts-trivial-tasks rung), easy/medium→serial DEPTH (revise the J-83 carrier), hard→parallel WIDTH (J-86 best-of-N / BSP), very-hard (proxy<~0.05)→escalate model (I-7) or ABSTAIN (I-8). The user's manual "effort toggle" = the lambda override on top of the auto-router (U_lambda = I-27 ConvergenceCurveMeasured + H-9 cost cockpit). HARD BOUNDARY: the LLM NEVER picks its own tier (TD-3, deterministic predicate only); the WINDOW is NEVER the lever (RD-4/RULER — the honest depth-vs-width reconciliation DC-5: window→never, easy/medium→depth, hard→width); the done-gate is ELAI-MEASURED per-(model,task-type) on a gold set (K-15 `elai bench reasoning-budget`), NEVER the paper's quintile bins (ID-5); it is OBSERVE-ONLY first (GD-4) before it gates anything (R-2). It does NOT re-author H-42/I-26/D-10 (composed, GD-1).

Estimated effort: ~1 session (~M). The classifier + the H-42/I-26 wiring + an observe-only telemetry leg. Pre-requisites (re-grep, W-7): J-83 (the carrier it sizes); H-42 h42_serial_depth_budget_contract.rs + ReasoningBudgetScaled (220, GS-7); I-26 BudgetController/effort field; D-10 select_grounding_mode (GS-8); H-39 abstention_score.rs (the proxy input); J-38 retrieval coverage + C-29 IntentPair + C-45/C-47 case-base (the other proxy inputs); I-27 + H-9 (the lambda surface).
Parallel-safe with: J-85 (the guard reads the budget the classifier sets, disjoint seam), K-15 (the bench that measures this). Depends J-83.
Source: reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS §5 + IMPLICATION 3/5/24/25/26 + ROUND-3 ADD 1; _grounding/J-GRC-V3-GROUNDING-MAP.md GA-2 + GS-7/GS-8/GS-4 + DC-5. Doctrine: J-0_INDEX.md (W/GD/RD/ID inherited; ADDS no delta family — RD-4 depth-not-width, TD-3 LLM-not-the-gate, ID-5 measure-your-own already cover it).

## Context
Why this matters: more reasoning is non-monotonic in value (it helps math/symbolic +10..+80, helps medium multi-step +5..+30, and HURTS trivial tasks and factual-recall, ROUND-3 ADD-1 table); routing a recall query to "more reasoning" is exactly the "reasoning hurts" case the user flagged. The classifier proxies difficulty from internal signals (a true pass@K oracle is expensive, IMPLICATION 26) and routes the budget; the effort toggle is the manual lambda on top. On the LOCAL Qwen-7B lane the engine IMPOSES structured decomposition (the small model NEEDS the steps, Cobbe) but the classifier still routes a trivial input to zero-reasoning and a recall input to retrieval. Mechanism: a deterministic function (TYPE from D-10, COST from the proxy bundle) → a budget tier on H-42/I-26; OBSERVE-ONLY first (emit the chosen tier vs the realized verified-outcome, do not gate), calibrate against K-15's measured knee, then gate.

THE 2-AXIS ROUTING (what J-84 ships):
  CC-1 TYPE axis (reuse D-10): reason vs RETRIEVE (recall/factual → DKG/A-RAG, never a chain) vs validate. select_grounding_mode already decides this deterministically (GS-8) — J-84 reads it, does not rebuild it.
  CC-2 COST axis (the new deterministic proxy): difficulty ← f(H-39 confidence, C-45 case-base hit, J-38 coverage, C-29 IntentPair count). NO LLM input (TD-3).
  CC-3 ROUTING (difficulty-keyed, DC-5): trivial→ZERO reasoning; easy/medium→serial DEPTH (revise the carrier, H-42); hard→parallel WIDTH (J-86); very-hard(proxy<~0.05)→I-7 escalate or I-8 abstain. WINDOW is never a tier (RD-4).
  CC-4 EFFORT TOGGLE: a manual lambda override (U_lambda) on top of the auto-tier, surfaced via I-27/H-9; the user dials light↔heavy, the floor/ceiling stays deterministic.

## Existing state (W-7 anchors; GS-* in the grounding map)
- H-42 SERIAL-DEPTH BUDGET [PARTIAL] h42_serial_depth_budget_contract.rs + ReasoningBudgetScaled (220) — the DEPTH tier target (GS-7).
- I-26 DYNAMIC THINKING BUDGETS [DARK(ELAI_DYNAMIC_BUDGET_ENABLE)] reuses H-42 + effort field threaded to anthropic_api; BudgetController — the budget the classifier sets (GS-8 note).
- D-10 MODE-ROUTING PREDICATE [WIRED+LOCKED] select_grounding_mode (modes A-E) — the CC-1 TYPE axis (GS-8).
- H-39 abstention_score.rs [WIRED] (GS-4) + C-45/C-47 case-base [DARK] + J-38 retrieval coverage [PLANNED] + C-29 IntentPair [WIRED] — the CC-2 proxy inputs.
- I-27 BUDGETED CONVERGENCE INSTRUMENTATION [DARK] ConvergenceCurveMeasured + H-9 COST COCKPIT [PARTIAL] — the CC-4 lambda surface (the K-15 bench shares I-27's harness).

## Reads (mandatory before coding)
1. reasoning-engine-grc-v3-2026-06-25.md §5 + IMPLICATION 3/5/24/25/26 + ROUND-3 ADD-1 (the gain table).
2. _grounding/J-GRC-V3-GROUNDING-MAP.md (GA-2, GS-7/GS-8/GS-4, DC-5, R-2).
3. h42_serial_depth_budget + ReasoningBudgetScaled + I-26 BudgetController + select_grounding_mode + abstention_score.rs — re-grep live (W-7).
4. J-83 (the carrier the budget sizes) + I-27/H-9 (the lambda surface) + J-34 (the sibling fill-calibration bench, the measurement pattern K-15 mirrors).
5. CLAUDE.md: TD-3 (LLM never the gate), RD-4 (depth-not-width / window-never), ID-5 (measure-your-own), GD-4 (observe-only first).

## Tasks (WAVE GRC-2 — the 2-axis classifier)
1. The deterministic COST proxy (~70 LOC): difficulty ← f(H-39, C-45 hit, J-38 coverage, C-29 count); a pure function, named constants, no LLM (TD-3). Read GS-8 for the TYPE axis (reuse, don't rebuild).
2. The router (~50 LOC): (TYPE, difficulty) → tier {Zero, Depth, Width, EscalateOrAbstain}; wire the tier into H-42 ReasoningBudgetScaled + I-26 BudgetController. WINDOW is never a tier output (DC-5).
3. The effort toggle (~20 LOC): the lambda override on top of the auto-tier, surfaced via I-27/H-9 (the user dial); the deterministic floor/ceiling holds regardless.
4. OBSERVE-ONLY telemetry (GD-4): emit chosen-tier vs realized verified-outcome (reuse ReasoningBudgetScaled +Kind), do NOT gate until K-15 calibrates the knee. ~20 LOC.
5. workflow/reasoning/J-84-decisions.md (the 2-axis design + the depth-vs-width-vs-window reconciliation DC-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/reasoning_classifier.rs (new) — the deterministic 2-axis classifier + the tier→budget router + the lambda toggle
consumes_symbols:
  - select_grounding_mode (the CC-1 TYPE axis; reuse, GS-8) — NOT rebuilt
  - abstention_score.rs + C-45/C-47 + J-38 + C-29 (the CC-2 proxy inputs)
  - H-42 ReasoningBudgetScaled + I-26 BudgetController (the DEPTH/budget target; EXTEND, GD-1)
  - I-27 ConvergenceCurveMeasured + H-9 cockpit (the CC-4 lambda surface)
  - J-83 BeliefState carrier (the thing the budget sizes the reasoning over)
reachable_from:
  - the GRC reasoning loop / task_runner → the classifier sets the budget per step [PRODUCTION reasoning loop]
  - the effort toggle surfaced via the CLI/dashboard lambda dial [PRODUCTION effort override]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-picked tier (deterministic predicate only; TD-3)
  - NO window-size as a tier output (RD-4; window is never the lever)
  - NO paper-quintile thresholds hard-coded (ELAI-measured knee from K-15; ID-5)
  - NO gating before OBSERVE-ONLY calibration (GD-4); NO re-implementation of H-42/I-26/D-10 (GD-1); NO .unwrap on prod
frontend_pair:
  the H-9 CloudPanel / a cost-effort tile shows the chosen tier + the lambda dial — decide the exact surface at edit (W-4); the classifier's tier-vs-outcome telemetry rides I-27's ConvergencePanel. Justified if no new tile (rides existing cost/convergence panels).
observable_effect:
  trigger: "drive the real reasoning entrypoint on (a) a trivial input, (b) a factual-recall query, (c) a medium multi-step task, (d) a hard branching task — assert the routed tier (integration smoke, R10)"
  expect_ssb: "prefer-reuse ReasoningBudgetScaled (+Kind: TierRouted / EffortOverridden) — net 0..+1 from LIVE 268/577, decide at edit (W-9)"
  expect_artifact: "trivial→ZERO reasoning (journal shows no chain budget); recall→RETRIEVE not a chain; medium→DEPTH; hard→WIDTH; the NEGATIVE arm: a recall query routed to a reasoning chain (the confabulation case) MUST NOT happen, and the LLM proposing its own tier is ignored (TD-3)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-84_grcv3_reasoning_effort_classifier.md --run-trigger
rg "Zero|Depth|Width|EscalateOrAbstain|difficulty|select_grounding_mode" .elai_cc/crates/orchestrator/src/reasoning_classifier.rs   # the 2-axis router present
rg -c "llm.*tier|model.*picks|ask.*model.*budget" .elai_cc/crates/orchestrator/src/reasoning_classifier.rs   # MUST be 0 (TD-3)
rg -c "window|context.*size.*tier|32k|32000" .elai_cc/crates/orchestrator/src/reasoning_classifier.rs        # window is never a tier (RD-4) — MUST be 0 as a tier output
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/reasoning_classifier.rs
test -f workflow/reasoning/J-84-decisions.md
```

## ACCEPTANCE
- [ ] CC-1: the TYPE axis reuses select_grounding_mode (recall→retrieve, not a chain); not rebuilt.
- [ ] CC-2: the COST proxy is a deterministic function of H-39/C-45/J-38/C-29; no LLM input.
- [ ] CC-3: routing is trivial→zero / easy-medium→depth / hard→width / very-hard→escalate-or-abstain; window is never a tier (DC-5).
- [ ] CC-4: the lambda effort toggle overrides the auto-tier; the deterministic floor/ceiling holds.
- [ ] GD-4: OBSERVE-ONLY first; gating waits on K-15's measured knee (ID-5, never the paper bins).
- [ ] GD-1: H-42/I-26/D-10 composed not re-implemented; cargo check GREEN; auditor PASS -p orchestrator; production_caller 0-DARK.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the 4-input integration smoke run-trigger (each input routes to the right tier), not file-exists.
- W-2: the I-26 budget rides its existing flag; the classifier ships OBSERVE-ONLY (no gate) first, then default-ON behind a *_DISABLE with an allowlist row.
- W-3: EXTENDS H-42/I-26, not a summary-port. W-4: rides the H-9/I-27 panels or justified N/A.
- W-5/W-10: in-process at reasoning time. W-6: existing cost/convergence panel or justified N/A.
- W-7: H-42/I-26/D-10/H-39 re-grepped at edit. W-9: reuse ReasoningBudgetScaled (+Kind), net 0..+1 derived.
- RD-4: window is never a tier. ID-5: thresholds are ELAI-measured (K-15), never paper bins. GD-2: tier decisions recomputable from the journal.

## Tests to add
- crates/contracts/tests/j84_reasoning_classifier_contract.rs (~70 LOC): (a) trivial→Zero; (b) recall→Retrieve (never a reasoning chain); (c) medium→Depth; (d) hard→Width; (e) very-hard(proxy<0.05)→EscalateOrAbstain; (f) the lambda override shifts the tier within the deterministic floor/ceiling; (g) an LLM-proposed tier is ignored (TD-3); (h) empty/degenerate proxy input fails-closed to the safe (lower-cost or abstain) tier (R1).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/reasoning_classifier.rs (~160 LOC)
- MODIFY: the GRC reasoning-loop / task_runner caller seam (~30 LOC: invoke the classifier, set the budget)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs + action_type_lock.rs + .golden IF a ReasoningBudgetScaled Kind is added (9-tuple lockstep; decide at edit)
- CREATE: crates/contracts/tests/j84_reasoning_classifier_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/J-84-decisions.md (~25 LOC)

## Citations / Grounding
- reasoning-engine-grc-v3-2026-06-25.md §5; IMPLICATION 3 (zero-reasoning rung), 5 (task-TYPE axis / recall→retrieve), 24 (the difficulty-routed escalation ladder), 25 (depth-vs-width reconciliation), 26 (proxy difficulty from internal signals, measure-your-own); ROUND-3 ADD-1 (the task-type×gain table prior).
- ELAI ground truth (GS-7/GS-8/GS-4): H-42 ReasoningBudgetScaled (220), I-26 BudgetController, select_grounding_mode (D-10), abstention_score.rs (H-39) — re-grepped (W-7).
- arXiv [EXTERNAL]: test-time-compute 2408.03314 (compute-optimal routes by difficulty: easy/medium→revision-depth, hard→parallel-width, very-hard→little progress), zero-shot-CoT 2205.11916 (CommonsenseQA -4.2pp = recall-hurts), RULER 2404.06654 (usable-not-available context = window-never-the-lever).
- CLAUDE.md: TD-3, RD-4 (depth-not-width / window-never), ID-5 (measure-your-own knee), GD-4 (observe-only first).

## Open questions / risks
- The COST proxy weighting (how to combine H-39 + case-base + coverage + IntentPair into one difficulty score): start with a transparent linear/threshold combination, calibrate the weights against K-15's measured outcome curve; do NOT tune by vibes (R-2).
- DC-5 must be stated in J-84-decisions.md explicitly (window→never, easy/medium→depth, hard→width) — the depth-not-width doctrine is REFINED here, not contradicted; hiding it is a documentation defect.
- Sequencing: J-84 reads the J-83 carrier; ship after J-83. The classifier OBSERVES first (GD-4) — do not let it gate before K-15 produces the per-(model,task-type) knee.

## Next
J-84 makes "reasoning helps a little/a lot/hurts" an actionable deterministic route, sizing the J-83 carrier's reasoning by task TYPE × difficulty and exposing the user's effort toggle. It pairs with J-85 (the guard that stops a run that overthinks past the budget) and J-86 (the WIDTH tier's candidate branches). K-15's `elai bench reasoning-budget` is its calibration + done-gate; K-16's doctrine binds the measure-your-own-knee + window-never rules.

End of J-84.
