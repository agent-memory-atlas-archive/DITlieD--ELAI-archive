# H-42: Serial-depth-scaled reasoning budget + the depth-not-width guard — give a deep sub-task the CoT steps it provably needs

STATUS: PLANNED (TRACK 6 GROUNDED-REASONING HARDENING; WAVE 6; in-process EXTENSION of the shipped per-role GRC budget + E-19 reflection)

Objective: ELAI's GRC budget is per-ROLE and STATIC (grc.rs:223 AgentGrcConfig.required_steps; reasoning/config.rs:430-580
agent_grc_config_for_role gives executor/auditor/etc. a fixed call+char budget). The formal CoT-expressivity results say
the number of SERIAL reasoning steps a problem needs scales with its serial DEPTH: a single forward pass is bounded
(~TC0), and the autoregressive CoT loop is the only escape to higher complexity classes — polynomial CoT reaches
EXACTLY P (2310.07923), and compositional reasoning needs ~O(n) CoT tokens for input size n with tight step lower
bounds (2503.01544 / 2502.02393). A flat per-role budget therefore UNDER-budgets a deep serial sub-task and FORCES the
truncation failure the budget meant to prevent. H-42 scales the per-role GRC + E-19 reflection budget by an estimated
serial-depth signal already in the system (decomposition-tree depth / DKG reachability-chain length), bounded by a
LOCKED ceiling, and adds the DEPTH-NOT-WIDTH guard: a budget-exhaustion on a high-serial-depth sub-task is classified
depth-bound (needs more steps) vs access-bound (needs more context), so a context-window lift is never mis-applied as
the fix for a depth-class failure (RD-4: a single pass is ~TC0; context buys access, not compute-depth).

Estimated effort: ~1 session (M, ~6-8 h). In-process extension of the shipped GRC budget; reuses the E-19 self-certainty
  + micro-reflection substrate; ONE new signal variant for the scaling decision.
Pre-requisites: NONE hard (GRC budget + E-19 SHIPPED). Composes Phase-2C GRC config + E-19 (self_certainty / micro_reflection)
  + the decomposition tree (B-7) / DKG blast-radius (B-29 / C-30 surgical) depth signal. Reads RS-10/RS-11 + RG-42 + RD-4.
Parallel-safe with: H-39, H-40, H-41 (disjoint files).
Source: the Hallucination-Stations research thread (2026-06-04) + 2310.07923 / 2402.12875 / 2503.01544 / 2502.02393;
  _grounding/H-REASONING-GROUNDING-MAP.md RG-42 + RD-4 + the CITATIONS block. Doctrine: H-0_INDEX.md.

## Context

The video/target paper (2507.07505) claim "more context / more steps buys nothing" and "a bigger window helps." The
formal literature says the OPPOSITE on both counts: (1) CoT/serial steps DO buy strictly more complexity-class reach
(2310.07923 poly CoT = P; 2402.12875 T steps simulate a size-T circuit; 2511.07378 GD provably learns NC1 with CoT) —
the bound is on the single PASS (~TC0), not the loop; (2) a bigger context window buys information ACCESS, not
compute-DEPTH (still one ~TC0 pass per token). So the correct lever for a hard, serial sub-task is MORE CoT STEPS, and
the required count is lower-bounded by the task's serial depth (2503.01544 ~O(n); 2502.02393 tight bounds). ELAI's flat
per-role budget (RS-10) ignores depth; H-42 makes the budget depth-proportional (bounded) and labels depth-class
failures so the remedy is steps, not window. This also FORMALLY grounds ELAI's "smarter orchestration, not a smarter
model" thesis (H-7): the FSM/BSP/CoT loop is exactly the serial-composition that reaches past a single 7B forward pass.

## Existing state (B/C/D/E/F/G coverage)
- grc.rs:223 AgentGrcConfig (required_steps) + reasoning/config.rs:430-580 agent_grc_config_for_role (per-role STATIC
  budgets) + reasoning/types.rs:228 — Phase-2C SHIPPED. The budget H-42 scales (GD-1; extend, do not replace).
- core/reasoning/self_certainty.rs compute_score (the L1-confidence signal) + core/micro_reflection.rs truncate_to_budget
  + micro_reflection_emitter.rs (emit_micro_reflection / parse_reflection_json / SidecarReflectionAdapter) — E-19 SHIPPED.
  The reflection budget H-42 also scales + the confidence signal the depth-bound classifier reads.
- the decomposition tree depth (B-7 tree / TreeNavigatorState) + the DKG reachability-chain length (B-29 impact radius /
  C-30 surgical query_reachability) — the serial-depth SIGNAL source (use the cheapest available for the current sub-task).
- a serial-depth scaling factor + the depth-bound-vs-access-bound classifier — ABSENT (RG-42). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-REASONING-GROUNDING-MAP.md: RG-42, RD-4, RS-10, RS-11, the CITATIONS block.
- workflow/papers/compositional-reasoning-transformers-rnn-cot-2503.01544-interface.md (~O(n) CoT tokens; the resource tradeoff)
  + workflow/papers/expressive-power-transformers-cot-2310.07923-interface.md (single pass ~TC0; poly CoT = P).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grc.rs:200-260 (AgentGrcConfig — the budget struct to scale)
- F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\config.rs:430-580 (agent_grc_config_for_role — the per-role budgets)
- F:\.ELAI_workflow\.elai_cc\crates\core\src\reasoning\self_certainty.rs (compute_score — the depth-bound classifier signal)
- the decomposition-tree depth source (TreeNavigatorState) OR the DKG reachability surface (C-30 query_reachability) — pick the cheapest shipped one.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; GD-2 determinism; RD-4.

## Tasks (WAVE 6 — depth signal + bounded scaling + the depth-not-width classifier)

1. **Serial-depth estimator (~50 LOC).** A deterministic `estimate_serial_depth(sub_task)` reading the cheapest shipped
   signal: the decomposition-tree depth of the current node (concept>phase>step>babystep nesting) OR, for a code task,
   the DKG reachability-chain length (the longest must-traverse call/dep chain from C-30 query_reachability / B-29 impact
   radius). Returns a bounded integer depth d. No wall-clock; seed-stable.

2. **Bounded budget scaling (~40 LOC).** A `scale_grc_budget(base: AgentGrcConfig, d)` that multiplies the per-role
   call+char (and the E-19 reflection) budget by a monotone factor of d, CLAMPED to a LOCKED ceiling (a deep task gets
   more steps; nothing gets an unbounded budget — the ceiling is mutable=0 so a loop cannot inflate its own budget).
   The base per-role config is the floor; scaling only ever RAISES toward the ceiling for genuinely deep sub-tasks.

3. **Depth-not-width classifier (~50 LOC).** On a GRC budget-exhaustion or a low self_certainty.rs::compute_score at
   high d, classify the failure: `DepthBound` (high d + budget exhausted -> the fix is MORE CoT steps / decomposition,
   never more context) vs `AccessBound` (low d + retrieval miss -> the fix is more/better context). Emit the classification
   so a context-window lift is never mis-applied to a DepthBound failure (RD-4). This is a LABELED diagnostic, not a hard gate.

4. **Off-agent measured row (~30 LOC).** Emit `{role, base_calls, serial_depth, scaled_calls, ceiling, failure_class}`
   as a ReasoningBudgetScaled SSB row (Pattern-2; baseline+1 from LIVE 220/497, re-read ssb.rs:2990 + action_type_lock.rs:624;
   first Track-6 bump fixes the stale e45 pin). Recomputable from the depth signal + the LOCKED ceiling under ELAI_SEED (GD-2).

5. **Decision doc workflow/reasoning/H-42-decisions.md (~70 LOC):** the CoT-step-lower-bound grounding (2503.01544 ~O(n));
   the single-pass ~TC0 bound (2310.07923) = why context != depth; the LOCKED ceiling rationale (a loop cannot inflate its
   budget); the honest residue (depth-scaling helps a SERIAL-decomposable task; a task that is genuinely above the model's
   reachable class even with CoT must route to a tool/solver (H-41) or refuse (D-16) — RD-4/GD-5, not "more steps always wins").

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/reasoning_budget.rs::estimate_serial_depth (the depth signal)
  - crates/orchestrator/src/reasoning_budget.rs::scale_grc_budget (the bounded scaler)
  - crates/orchestrator/src/reasoning_budget.rs::classify_failure (DepthBound vs AccessBound — the depth-not-width guard)
  - crates/orchestrator/src/ssb.rs::SsbActionType::ReasoningBudgetScaled (off-agent scaling artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/grc.rs::AgentGrcConfig (the per-role budget — EXTEND, GD-1)
  - crates/core/src/reasoning/self_certainty.rs::compute_score (the depth-bound classifier signal — REUSE)
  - the decomposition-tree depth OR C-30 query_reachability (the serial-depth signal — REUSE the cheapest shipped one)
reachable_from:
  - the per-baby-step dispatch (where agent_grc_config_for_role is resolved) -> scale_grc_budget(base, d) -> ReasoningBudgetScaled row [PRODUCTION]
  - a GRC budget-exhaustion -> classify_failure -> the DepthBound/AccessBound diagnostic on the row
stub_ban:
  - NO context-window lift accepted as the fix for a DepthBound failure (RD-4) — a DepthBound row whose remedy is "raise context" is a FAIL
  - NO unbounded budget; the ceiling is LOCKED mutable=0 (a loop cannot inflate its own budget)
  - NO re-implementation of the GRC config or E-19 reflection (GD-1 P0); extend them
  - the broken `--flat-budget` arm MUST give a deep and a shallow sub-task the SAME budget (R2: proves depth-scaling is load-bearing)
  - blake3 / no .unwrap()/.expect() in prod
frontend_pair:
  the per-sub-task serial-depth + scaled-budget + failure-class in the reasoning drawer (cross_lang_diff PASS).
  W-6 E2E: a deep sub-task shown with a larger budget + a DepthBound diagnostic in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && run a real task with (a) a shallow sub-task (depth ~1) and (b) a deep serial sub-task
    (high decomposition depth / long reachability chain), same role, NO env override"
  expect_ssb: "two ReasoningBudgetScaled rows; scaled_calls(b) > scaled_calls(a) (both <= the LOCKED ceiling),
    recomputable from the depth signal under ELAI_SEED (GD-2)"
  expect_artifact: "the deep sub-task gets proportionally more GRC/reflection steps; a forced budget-exhaustion at high
    depth emits failure_class=DepthBound (remedy = steps, not context); the broken `--flat-budget` arm gives both the same budget (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-42_serial_depth_reasoning_budget.md --run-trigger
rg "estimate_serial_depth|scale_grc_budget|ReasoningBudgetScaled" .elai_cc/crates/orchestrator/src --glob '!*test*'   # produced + wired
rg "AgentGrcConfig|compute_score" .elai_cc/crates/orchestrator/src/reasoning_budget.rs                                # GRC + self-certainty reused (GD-1)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/reasoning_budget.rs                                              # MUST be 0
test -f workflow/reasoning/H-42-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (deep > shallow budget; rows landed). cargo-green != DONE.
- [ ] W-2 default-ON: depth-scaling runs on every sub-task with no env override; kill-switch only; budget floor = the base per-role config.
- [ ] W-3 production entrypoint: scaling fires from the live per-baby-step dispatch, not a #[cfg(test)] caller.
- [ ] RD-4: a DepthBound failure's remedy is steps/decomposition, never context; the ceiling is LOCKED mutable=0.
- [ ] GD-1: AgentGrcConfig + self_certainty + the depth signal REUSED (no re-implementation; P0 if violated).
- [ ] GD-2 reproducible: scaled_calls + failure_class re-compute byte-identically from the depth signal under ELAI_SEED.
- [ ] R2: the --flat-budget broken arm gives deep and shallow the same budget (depth-scaling is falsifiable, not vacuous).
- [ ] GD-5: the decision doc states depth-scaling helps a serial-decomposable task; an out-of-class task routes to a tool/refusal.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the scaler (produced) feeds the live dispatch budget (W-8) reached by a real sub-task (W-3).
- W-2: runs with no override; floor = base config; kill-switch only.
- W-3: live per-baby-step dispatch; not test-only.
- W-4: frontend_pair = the depth/budget/failure-class reasoning drawer; cross_lang_diff PASS.
- W-5: observable_effect is a runnable task -> real journal rows; in-process.
- W-6: a deep sub-task's larger budget + DepthBound diagnostic visible in the dashboard (dashboard-smoker).
- W-7: grc.rs:223 / reasoning/config.rs:430-580 / self_certainty.rs grep-confirmed (RS-10/RS-11); depth-scaling absent (RG-42).
- W-8: scale_grc_budget (produced) consumed by the dispatch budget resolution; classify_failure by the diagnostic.
- W-9: +ReasoningBudgetScaled baseline+1 (Pattern-2); first Track-6 bump fixes e45.
- W-10: in-process, no network/RunPod axis — single-tier.
- GD-1/2 + RD-4: compose; reproducible; depth-scaled, ceiling-LOCKED, depth-not-width.

## Tests to add
- crates/contracts/tests/h42_serial_depth_budget_contract.rs (~140 LOC): (a) estimate_serial_depth on a fixture tree;
  (b) scale_grc_budget raises with depth, clamps at the ceiling; (c) base config is the floor (depth 0/1 unchanged);
  (d) classify_failure = DepthBound at high d + exhaustion, AccessBound at low d + retrieval miss; (e) the --flat-budget
  arm gives equal budgets (R2); (f) the ceiling is mutable=0 (cannot be raised at runtime); (g) ELAI_SEED twice -> identical.
- crates/contracts/tests/fixtures/reasoning_budget/ (a shallow + a deep decomposition-tree fixture).

## Files touched
- CREATE: crates/orchestrator/src/reasoning_budget.rs (~140 LOC: depth estimator + bounded scaler + failure classifier)
- MODIFY: the per-baby-step dispatch site (resolve agent_grc_config_for_role -> scale_grc_budget(base, d), ~20 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (+ReasoningBudgetScaled) + action_type_lock.rs:624 + e45:294 (if first Track-6 bump)
- MODIFY: the reasoning drawer Svelte pair (+depth/budget/failure-class columns)
- CREATE: crates/contracts/tests/h42_serial_depth_budget_contract.rs (~140 LOC)
- CREATE: workflow/reasoning/H-42-decisions.md (~70 LOC)

## Citations / Grounding
- 2310.07923 Merrill & Sabharwal (single pass ~TC0; poly CoT = P — context != depth, steps buy class-reach).
- 2402.12875 Li et al (T CoT steps simulate a size-T circuit — serial steps are the resource).
- 2503.01544 Yehudai et al (~O(n) CoT tokens for compositional reasoning — the scaling law for depth) +
  2502.02393 Amiri/Hahn (tight CoT step lower bounds). WebSearch-confirm all ids before any headline (feedback_paper_citation_verify).
- ELAI ground truth (RS-10/RS-11, grep-confirmed 2026-06-04): grc.rs:223 AgentGrcConfig; reasoning/config.rs:430-580; self_certainty.rs compute_score; micro_reflection_emitter.rs.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); RD-4; GD-5 (depth-scaling is not a cure for an out-of-class task).

## Open questions / risks
- The serial-depth signal is a PROXY (tree depth / reachability-chain length), not the true CoT-step lower bound (which is
  task-specific and generally uncomputable). Calibrate the scaling factor + ceiling on measured budget-exhaustion rates
  over >=3 real runs (GD-4); start conservative (small factor) and raise only on evidence of depth-bound truncation.
- The LOCKED ceiling must bound total cost — depth-scaling that uncaps the budget would let a deep task burn the window.
  The ceiling is the cost guardrail; HD-2/HD-3 (cache + output) still apply to the scaled budget.
- DepthBound vs AccessBound is a heuristic classification; a mixed failure (deep AND retrieval-starved) should surface
  BOTH, not force one — do not over-claim a clean dichotomy.

## Next
Track 6 complete: H-39 (abstention-credited reward) + H-40 (decorrelated-verifier retention) + H-41 (sound-checker
coverage) + H-42 (depth-scaled reasoning). Together they align ELAI's reasoning/verification loop with the rigorous
2024-2026 computational-limits literature: reward honesty, keep decorrelated verifiers, reach sound checkers, and give a
deep sub-task the serial steps it provably needs — none of which the single-pass FLOP-ceiling framing of 2507.07505 captures.

End of H-42.
