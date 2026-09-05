# J-58: pre-execution sprint-contract negotiation loop — generator and evaluator argue "done" into a GRANULAR contract BEFORE building (the one Anthropic long-running-agents mechanism I-21 does not have)

STATUS: PLANNED (WAVE PV-1; effort S-M; EXTENSION of I-21 — adds a bounded multi-round pre-execution re-propose loop in front of I-21's single-shot approval; reuses SprintContractEvent, +1 Kind arm; composes I-21/D-13, re-implements none)

Objective: the Anthropic long-running-agents talk's single distinct mechanism not already in ELAI is the CONTRACT NEGOTIATION — before the generator writes a line, the generator and the (isolated) evaluator go back and forth via files on disk over what "done" means and what tests are adequate, until both agree; the NEGOTIATION PRESSURE is what makes the contract GRANULAR ("their retro-game run settled on 27 criteria; granular criteria -> actionable critique, vague criteria -> the generator shrugs"). I-21 (SPRINT CONTRACTS + RALPH LOOP, WIRED) already ships the contract STRUCT, the ISOLATED evaluator (zero generator context, +0.12 self-review bias removed), the SINGLE pre-exec Approve/Reject, and a POST-execution Ralph loop on output failure — but its approval is single-shot: a Rejected contract returns a structured objection and that is the end of the pre-exec phase; there is NO re-propose-until-agree. J-58 adds exactly that bounded loop: on Reject, the objection is fed back to the generator, which re-proposes a refined contract (typically with MORE, more-specific acceptance criteria), the isolated evaluator re-judges, bounded to N rounds; convergence = Approved -> execution proceeds; non-convergence at the cap = escalate to the D-13 cross-family adversary or surface. HARD BOUNDARY (freeze-the-gate R8 preserved): the evaluator stays ISOLATED — zero generator scratchpad; negotiation flows ONLY through the sealed contract payload + the structured objection field (the talk's "markdown files on disk", here the I-1 supervisor's sealed-payload path), NEVER a shared context window. GD-1 do-NOT-re-implement: I-21 SprintContract struct + isolated-evaluator Directive + SprintContractEvent + the post-exec Ralph loop; D-13 adversary; I-1 supervisor context-pruning.

Estimated effort: ~1 session (~5 h). In-process (the loop is a bounded counter + a re-dispatch of the already-built isolated evaluator over a re-proposed payload); no new model machinery.
Pre-requisites (re-grep at edit, W-7): I-21 SprintContract + isolated-evaluator Directive + SprintContractEvent + approval gate SHIPPED (this wraps its pre-exec approval); D-13 cross-family adversary SHIPPED (the cap-escalation target); I-1 supervisor SHIPPED-or-shim (the context-pruning isolation substrate I-21 already rides).
Parallel-safe with: J-56, J-59. Pairs with J-57 (both harden the generate/verify loop).
Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (the Anthropic talk: contract negotiation as the bridge from high-level spec to testable assertions; granularity -> actionable critique). Doctrine: J-0_INDEX.md.

## Context
Why this matters (external evidence, scope-tagged per HD-1): the Anthropic talk (FIRST-PARTY on their own harness — the strongest tier here) calls the contract negotiation "the glue": the planner stays deliberately HIGH-LEVEL (a low-level planner error cascades and magnifies over a multi-hour horizon, so they ABANDONED the granular up-front spec), and the granular, testable contract is negotiated by the generator and evaluator at execution time — "generator proposes I'll build X, verify by Y; evaluator pushes back scope is too big, those tests are too weak, you missed edge case XYZ; back and forth until both agree; the evaluator then grades against the NEGOTIATED contract, not the planner's one-shot spec". [DESIGN, first-party — directional; the "27 criteria" figure is one run, not a target.] the talk is explicit that this is what the Ralph loop lacked — "a fixed plan.md with nobody on the other side arguing". I-21 brought the isolated adversarial evaluator and the post-failure Ralph reinjection; J-58 brings the PRE-execution argument that I-21's single-shot approval skips. correctness boundary: every contract version is still validated by I-21's existing rule (acceptance_criteria must be deterministically checkable — symbol refs / compile status / wiring_contract fields, never "feels complete"); the negotiation can only make criteria MORE specific, never replace the deterministic-checkability requirement (TD-3).

## Existing state (B/C/D/E/F/G/H/I/J coverage — W-7 anchors, nothing from memory)
- SPRINT CONTRACTS + RALPH LOOP [WIRED, I-21]: SprintContract {goal, in_scope_files, acceptance_criteria, bounded_iteration_count, out_of_scope_marker}; isolated evaluator (zero generator context); SprintContractEvent (Proposed/Approved/Rejected/RalphReinjected/AdversaryEscalated); the Ralph loop is POST-execution; the pre-exec approval is SINGLE-SHOT (feature-map line 669-670; I-21 detail file). J-58 inserts the pre-exec re-propose loop between Proposed and Approved.
- the I-21 acceptance-criteria validation: vague/open-ended criteria REJECTED at approval time (must reference a test symbol, compile status, or wiring_contract field). J-58 reuses it on every re-proposed version.
- D-13 cross-family adversary [WIRED]: the AdversaryEscalated target at the negotiation cap (mirrors I-21's 3rd-Ralph-failure escalation). GD-1 do NOT re-implement.
- I-1 supervisor [PLANNED/shim]: the context-pruning isolation substrate I-21 already depends on; J-58 routes the re-proposed payload through the SAME sealed path (no new isolation mechanism).

## Reads (mandatory before coding)
1. workflow/reasoning/steal-production-ai-talks-2026-06-20.md (the negotiation/glue mechanism + the granularity claim).
2. docs/thefinalglance/I-plan/I-21_sprint_contracts_ralph_loop.md (the struct, the isolated evaluator Directive, the approval gate, the SprintContractEvent arms — the seam J-58 wraps).
3. crates/orchestrator/src/sprint_contract.rs (I-21's module — re-grep; the approval gate + the SprintContractEvent emit are here).
4. the D-13 adversary call surface; the I-1 supervisor sealed-payload spawn path.
5. CLAUDE.md: TD-3 (deterministic acceptance criteria only), freeze-the-gate R8 (evaluator isolation), W-2, no-.unwrap-in-prod.

## Tasks (WAVE PV-1 — the bounded pre-exec negotiation loop)
1. NegotiationLoop (~80 LOC) in sprint_contract.rs: a bounded counter (NAMED const cap, default e.g. 3, SEPARATE from the post-exec Ralph counter). On SprintContractEvent::Rejected, feed the evaluator's structured objection back to the generator, which re-proposes a refined SprintContract; the isolated evaluator re-judges the NEW payload (same sealed-payload isolation path). Loop until Approved or the cap. NOT: an unbounded loop; NOT: leaking the generator's prior turns into the evaluator (the objection is the ONLY thing carried back; the evaluator still sees only the sealed contract + task spec).
2. Granularity-monotonicity guard (~40 LOC): a re-proposed contract that does NOT strictly increase acceptance-criteria specificity (criteria count must not decrease, and each must still pass the I-21 deterministic-checkability validation) is itself Rejected — the negotiation can only tighten, never loosen, the contract. NOT: accepting a re-proposal that drops criteria to get past the evaluator (gaming the gate).
3. Cap-escalation (~30 LOC): non-convergence at the cap escalates to the D-13 cross-family adversary (reuse I-21's AdversaryEscalated arm + path) OR surfaces to the user — the adversary produces a concrete counter-contract, not a score. NOT: silently proceeding to execution on a never-Approved contract.
4. SSB: reuse I-21's SprintContractEvent, ADD a `Renegotiated{round, criteria_count_before, criteria_count_after}` Kind arm (golden-lock lockstep — additive arm, TOTAL_VARIANTS +1, no new action_type). NOT: a new event type.
5. Decision doc: workflow/reasoning/J-58-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/sprint_contract.rs::NegotiationLoop (the bounded pre-exec re-propose loop)
  - the granularity-monotonicity guard on each re-proposed contract
  - SprintContractEvent::Renegotiated Kind arm (additive to I-21's event)
consumes_symbols:
  - I-21 SprintContract struct + the isolated-evaluator Directive + the approval gate (the seam this wraps; GD-1 do NOT fork)
  - I-21 SprintContractEvent (extended with one arm, NOT replaced; GD-1)
  - I-21 acceptance-criteria deterministic-checkability validation (re-run on every re-proposal; GD-1)
  - D-13 cross-family adversary (the cap-escalation target; GD-1 do NOT re-implement)
  - I-1 supervisor sealed-payload path (the isolation the re-proposed payload rides; GD-1)
reachable_from:
  - the I-21 sprint-contract dispatch in task_runner / search ladder (PRODUCTION — the negotiation loop runs in the pre-exec approval phase of a real sprint, before the generator writes code)
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO shared context between generator and evaluator across the negotiation (R8 freeze-the-gate — only the sealed contract + the structured objection cross; the evaluator never sees generator scratchpad)
  - NO unbounded negotiation (the cap is a NAMED const, SSB-tracked; an overflow is a BLOCK -> D-13 escalation)
  - NO loosening re-proposal admitted (criteria count must not decrease; each must pass I-21 deterministic-checkability)
  - NO proceeding to execution on a never-Approved contract (cap = escalate, not proceed)
  - NO new SSB action_type (reuse SprintContractEvent + the Renegotiated arm)
  - NO re-implementation of the I-21 struct / evaluator / D-13 adversary / I-1 pruning (compose)
  - NO .unwrap()/.expect() on the negotiation prod path
frontend_pair:
  the I-21 contract card (PlanApprovalCard sibling, I-21 frontend_pair) gains a negotiation-round counter + the per-round criteria-count delta; Tauri read of the Renegotiated rows. cross_lang_diff PASS (W-4). (Minimal extension of the existing card; justified.)
observable_effect:
  trigger: "elai run --sprint --task-fixture fixtures/vague_contract.toml --emit-ssb  (a fixture whose first proposal has a single vague criterion)"
  expect_ssb: "SprintContractEvent::Rejected (round 0, the vague criterion) -> ::Renegotiated{round=1, criteria_count_before < criteria_count_after} -> ::Approved (the tightened contract); the evaluator's incoming payload on EVERY round contains ZERO generator turns (journal row-count join)"
  expect_artifact: "the Approved contract has strictly MORE / more-specific acceptance criteria than the first proposal. NEGATIVE that must FAIL: a negotiation with no cap (unbounded); a re-proposal that DROPS criteria getting Approved; any evaluator payload containing generator scratchpad (R8 isolation breach); execution starting on a never-Approved contract."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-58_sprint_contract_negotiation_loop.md --run-trigger
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/sprint_contract.rs   # NegotiationLoop WIRED, not DARK
rg -n "NegotiationLoop|Renegotiated|criteria_count|NEGOTIATION_CAP" crates/orchestrator/src/sprint_contract.rs
rg -c "generator_context|scratchpad|prior_turns" crates/orchestrator/src/sprint_contract.rs   # MUST be 0 in the evaluator payload path (R8 isolation)
rg -c "unwrap\(\)|expect\(" crates/orchestrator/src/sprint_contract.rs   # MUST be 0 (prod path)
test -f workflow/reasoning/J-58-decisions.md
```

## ACCEPTANCE
- [ ] R8 ISOLATION: across every negotiation round the evaluator sees only the sealed contract + structured objection, zero generator scratchpad (contract-tested row-count).
- [ ] BOUNDED: the negotiation cap is a NAMED const, SSB-tracked; overflow -> D-13 escalation, never silent proceed (contract-tested).
- [ ] MONOTONE: a loosening re-proposal (fewer/vaguer criteria) is Rejected; only tightening re-proposals converge (contract-tested).
- [ ] TD-3: every contract version's acceptance criteria pass I-21's deterministic-checkability validation.
- [ ] R7/D-030: NegotiationLoop reachable from the production sprint dispatch, not a test.
- [ ] GD-1: I-21 struct/evaluator/event + D-13 adversary + I-1 pruning consumed, not restated; SprintContractEvent extended by ONE arm, not replaced.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: entrypoint = the I-21 sprint pre-exec approval phase; artifact = SprintContractEvent::Renegotiated/Approved rows. Met.
- W-2: no new flag (it tightens an existing WIRED loop). Met.
- W-3: the talk verdict in-repo (steal doc). Met.
- W-4: measures the objective (the Approved contract's criteria count strictly rose + the evaluator stayed isolated), not stdout. Met.
- W-5: production_caller_check; NegotiationLoop called from the sprint dispatch, not a test. Met.
- W-6: the I-21 contract card gains the round counter + criteria delta. cross_lang_diff PASS.
- W-7: I-21/D-13/I-1 anchors re-grepped at edit.
- W-9: SCHEMA baseline re-read at edit (SprintContractEvent += Renegotiated arm = TOTAL_VARIANTS +1, no new action_type).
- W-10: in-process loop; no network leg. (the per-round re-dispatch is an in-harness evaluator call.)
- GD-1: composition above. GD-5 honest residue: the granularity-monotonicity heuristic (criteria count must rise) is a PROXY for "the contract got more specific" — a re-proposal could add a redundant criterion to satisfy the count; the deterministic-checkability validation bounds the abuse but does not eliminate it (named in the file, the cap stops infinite gaming).

## Tests to add
- crates/contracts/tests/j58_negotiation_loop_contract.rs (~90 LOC): (a) a vague first proposal -> Rejected -> Renegotiated -> Approved within the cap, criteria count strictly higher; (b) the evaluator payload on every round has zero generator rows (R8); (c) a loosening re-proposal (fewer criteria) is Rejected; (d) non-convergence at the cap -> AdversaryEscalated, never execution; (e) the cap is enforced (an N+1th round is impossible); (f) SprintContractEvent::Renegotiated carries the before/after criteria counts.

## Files touched
- MODIFY: crates/orchestrator/src/sprint_contract.rs (~150 LOC: NegotiationLoop + monotonicity guard + cap escalation) — re-grep at edit
- MODIFY: crates/orchestrator/src/ssb.rs (SprintContractEvent += Renegotiated arm + golden lock) (~30 LOC)
- MODIFY: dashboard-v2 the I-21 contract card (~25 LOC: round counter + criteria delta) + Tauri read — re-grep
- CREATE: crates/contracts/tests/j58_negotiation_loop_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/J-58-decisions.md (~25 LOC)

## Citations / Grounding
- Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Anthropic long-running-agents talk: contract negotiation as the glue; granular criteria -> actionable critique). [DESIGN, FIRST-PARTY directional — the "27 criteria" is one run, not a target; the done-gate is "criteria count strictly rose + isolation held", not a borrowed number.]
- ELAI ground truth (PLAN-asserted, executor re-greps per W-7): I-21 sprint_contract.rs (struct, isolated evaluator, approval gate, SprintContractEvent); D-13 adversary; I-1 supervisor pruning.
- CLAUDE.md: TD-3 (deterministic acceptance criteria), freeze-the-gate R8 (evaluator isolation is the structural invariant), W-2, no-.unwrap-in-prod, named-constants.
- LIVE SCHEMA baseline ~268/577 (Tier-B derived, 2026-06-15); SprintContractEvent += Renegotiated arm => TOTAL_VARIANTS +1, NO new action_type; re-read at edit (R-c/W-9).

## Open questions / risks
- Whether the generator re-proposes autonomously or the loop is driven by the orchestrator re-dispatching it — decide at edit; the I-1 supervisor seam (which spawns the isolated evaluator) is the natural driver. PLAN-asserted (W-7).
- The negotiation cap value (3 mirrors the Ralph cap) — a NAMED const, tunable; default 3, never unbounded.
- Honest residue (GD-5): "criteria count rose" is a granularity PROXY (named above); the deterministic-checkability validation + the cap bound the gaming, they do not prove the contract is genuinely better — that only shows at execution via I-21's post-exec Ralph loop.

## Next
J-58 closes the gap between I-21's single-shot pre-exec approval and the Anthropic harness's negotiated, granular contract — the generator and evaluator now argue "done" into a specific, testable contract BEFORE building, which is what makes downstream critique actionable. It pairs with I-21 (the post-exec Ralph loop it precedes) and J-57 (the prompts the contract executes under). Freeze-the-gate R8 is preserved: the argument flows through sealed payloads, never a shared context.

End of J-58.
