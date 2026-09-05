# I-26: dynamic thinking budgets — serial-depth-scaled compute with 85% collapse and effort-parameter cloud mapping

- Track 16 — ROUTING + BUDGETS + MEASUREMENT | lever STEERING | grounding STRONG | nature EXTENSION | effort M | WAVE 4
- Premise: H-42 serialized GRC token budgets for the main reasoning chain; I-26 applies the same depth-scaled
  discipline to search-node exploration. a node's compute budget scales with its depth in the DAG; exploration
  halts at 85% budget consumed (score frontier, synthesize best); a plateau rule (L2 scores flat across 5
  supersteps) collapses early. cloud reasoning roles map to the EFFORT parameter — Anthropic Opus 4.7+
  explicitly rejects manual budget_tokens (carried landmine #6) — and reasoning roles are never force-JSON'd
  (HD-3).

## Steal/build
H-42 ships the ReasoningBudgetScaled SSB row and the serial-depth concept for GRC; I-26 extends the same
idea to search-node budgets (each I-4 expansion gets a depth-proportional compute envelope). mechanism: a
budget_controller.rs computes node_budget(depth) = base * depth_factor^depth, capped at E-39's hard ceiling
(supreme). the 85% collapse rule: when accumulated node-budget spend across all active siblings exceeds
85% of the total search envelope, halt new expansions, score the frontier with I-17's L2 pass, synthesize
the best result, emit ConvergenceCurveMeasured (I-27's instrument). the plateau rule: if the top-L2-ranked
surviving branch's score does not improve by delta_min across 5 consecutive supersteps, fire early collapse.
cloud reasoning mapping: when the node calls a cloud architecture role, the request uses effort (auto/expert)
not budget_tokens; the field budget_tokens is structurally absent from the cloud request builder for Opus
4.7+ (a contract test greps the builder to prove it). never force-JSON a reasoning role (HD-3 verbatim).
the emitted SSB row is the H-42 ReasoningBudgetScaled family — NO new action_type; if the existing payload
lacks a depth_level or plateau_fired field, extend the struct in place (decide at edit; prefer reuse). EXCLUDED:
manual budget_tokens on any Opus 4.7+ code path (landmine #6 — the contract test is the enforcement);
force-JSON wrapping of extended-thinking responses (HD-3 hard rule).

## Existing ground truth
- H-42 ReasoningBudgetScaled SSB row ships (I-0 table: "NO new action_type — reuse H-42 ReasoningBudgetScaled"). PLAN-asserted; executor re-greps the ssb.rs row definition per W-7.
- E-39 hard cost ceiling: the supreme cap; I-26 budget_controller.rs never exceeds it. PLAN-asserted; executor re-greps per W-7.
- E-19 confidence gate: the convergence prior that plateau detection composes with. PLAN-asserted; executor re-greps per W-7.
- I-4 (WAVE 4 predecessor): the search-node expansions whose budgets I-26 governs. depends I-4.
- IS-19: wiring tooling live; R8 frozen-gate applies to the budget controller (it is not a gate itself, but the collapse rule must not be reachable from the generator).
- HD-2: cache prefix stability must hold across depth-budget variations (the prefix must not encode a mutable budget token).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/routing/budget_controller.rs (~100 LOC — depth-scaled budget, 85% collapse, plateau
  detector, effort-parameter mapping for cloud roles)
- modification of cloud request builder to replace any budget_tokens field with effort for Opus 4.7+ models
- contract test: i26_dynamic_thinking_budgets_contract.rs — greps the builder for absence of budget_tokens
  on Opus 4.7+ paths; verifies ReasoningBudgetScaled rows carry depth + collapse metadata
- decision doc workflow/reasoning/I-26-decisions.md

DEFER (Tier-2 / gated):
- adaptive depth_factor tuning from journal data (consumes I-27's curve, requires sufficient trajectory volume)
- per-role effort-level calibration beyond auto/expert (once ELAI-measured cost/quality delta is available)

## Integration
- MODIFY: crates/orchestrator/src/search/ (budget_controller hook at node expansion and collapse point)
- MODIFY: cloud request builder (providers/src/anthropic_api.rs or equivalent — remove budget_tokens field for Opus 4.7+, add effort field)
- CREATE: crates/orchestrator/src/routing/budget_controller.rs (~100 LOC)
- seams from the map: Track 16 anchor — H-42 serial-depth + E-39 cost gate; cloud request builder for effort-parameter mapping
- frontend_pair: budget bar on the branch board (I-4 branch board; the depth + budget_remaining fields from ReasoningBudgetScaled rows render as a progress bar per node).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/routing/budget_controller.rs::BudgetController (depth-scaled compute, 85% collapse, plateau detection)
  - cloud request builder modification: effort field replaces budget_tokens for Opus 4.7+ reasoning roles

consumes_symbols:
  - H-42 ReasoningBudgetScaled SSB row (reuses; payload extended if depth_level field absent) — GD-1
  - E-39 hard cost ceiling (BudgetController never exceeds it; supreme gate) — GD-1
  - E-19 confidence gate (plateau detection composes the convergence prior) — GD-1
  - I-4 search module (the expansion loop that BudgetController governs) — GD-1

reachable_from:
  - crates/orchestrator/src/search/ladder.rs (I-4) -> BudgetController at each node expansion  [PRODUCTION search surface]
  - `elai run <task>` triggers I-4 which calls BudgetController; ReasoningBudgetScaled rows land in SSB as the off-agent artifact

stub_ban:
  - NO budget_tokens field on Opus 4.7+ cloud requests — use effort parameter (carried landmine #6; contract test enforces)
  - NO force-JSON wrapping of a reasoning/extended-thinking role (HD-3 hard rule — never negotiable)
  - NO paper-derived budget constants as thresholds (ID-5 — ELAI-measured baselines from I-27 curve only)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2

frontend_pair:
  budget bar on the branch board (H-15/I-4 branch board UI). ReasoningBudgetScaled rows carry depth + budget_remaining + plateau_fired; the bar renders remaining budget per active node. cross_lang_diff.py covers the Tauri -> Svelte pair. Justified: reuses the existing branch board surface, no new dashboard crate.

observable_effect:
  trigger: "elai run --task <deep-fixture>"
  expect_ssb: "ReasoningBudgetScaled rows per search node with depth_level; on a seeded over-budget fixture, a collapse row (plateau_fired=true or budget_pct>=85) followed by NO further expansion rows; on a cloud Opus 4.7+ node, zero budget_tokens fields in the journaled request bytes (journal shows effort field only)"
  expect_artifact: "NEGATIVE: a budget_tokens field on an Opus 4.7+ request is absent (contract test rg 'budget_tokens' the cloud request builder path — MUST count 0 on Opus 4.7+ branches); NEGATIVE: a post-collapse expansion row is structurally impossible (journal grep shows no SearchNodeEvent(Expanded) after the collapse marker)"

## Grounding
- H-42 ReasoningBudgetScaled (the reused SSB row family; I-0 CANONICAL CONSTANTS table).
- Anthropic Opus 4.7+ effort parameter: carried landmine #6 in I-0_INDEX ("Opus 4.7+ REJECTS manual thinking budget_tokens — use the effort parameter"). WebSearch-confirm the current effort parameter API shape before editing the cloud request builder.
- HD-3 (reasoning roles never force-JSON'd): H-0_INDEX canonical.
- ID-5: no paper budget-scaling constant becomes a gate threshold; ELAI measures its own depth-factor from the I-27 convergence curve.
- RD-1..RD-4 (H-0 reasoning-budget delta family): bind here — budget governs the reasoning budget within the search.
- SSB: NO new action_type — reuse H-42 ReasoningBudgetScaled. baseline+0 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- H-42 ReasoningBudgetScaled: do NOT re-implement the row or the serial-depth concept; extend the payload in place.
- E-39 hard cost ceiling: do NOT re-implement the ceiling; call it as the supreme cap.
- E-19 confidence gate: do NOT re-implement the confidence estimator; compose the plateau detection on top of it.
- I-4 search ladder: BudgetController is a governed module inside the ladder; do NOT fork the ladder's expansion loop.

End of I-26.
