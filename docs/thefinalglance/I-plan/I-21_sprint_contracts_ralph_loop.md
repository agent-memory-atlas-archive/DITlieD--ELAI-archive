# I-21: sprint contracts + ralph loop

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever TRUST-ANTI-BIAS | grounding STRONG | nature BUILD | effort M | WAVE 3 | IC-17 consistency
- Premise: self-review bias is measured at +0.12 on structured tasks — the generator evaluating its own output
  inflates quality scores. IN-4 confirms no sprint_contract module exists. i-21 builds the structural separation:
  the generator proposes a bounded Sprint Contract; an isolated evaluator (zero shared context with the generator)
  approves or rejects it; on exit, failed checks reinject the trace back to the generator (ralph loop, <=3 iterations);
  the 3rd failure escalates to the D-13 cross-family adversary. the evaluator persona is KEPT (hostile-critic role
  at tau=0) per IC-17 consistency: alignment-dependent tasks are exactly where persona prompting helps.

## Steal/build
SPRINT CONTRACT STRUCT: a bounded, structured proposal from the generator: {goal, in_scope_files, acceptance_criteria,
bounded_iteration_count, out_of_scope_marker}. the contract is STRUCTURED (machine-readable fields), not a free-text
plan. the acceptance_criteria must be deterministically checkable (reference to test symbols, compile status, or
wiring_contract fields — NOT "the implementation feels complete"). a vague or open-ended acceptance criterion is
REJECTED at approval time.
ISOLATED EVALUATOR: the evaluator receives ONLY the Sprint Contract and the task spec — ZERO generator context (no
prior turns, no intermediate outputs, no scratchpad). isolation is enforced by the i-1 supervisor (the Directive that
spawns the evaluator carries only the sealed contract payload; the generator's context is pruned before the spawn).
the evaluator runs the D-13 adversary posture at tau=0 with a hostile-critic role (IC-17: personas HELP on
alignment-dependent tasks; self-review bias removal is exactly that; the persona is KEPT here).
APPROVAL GATE: the evaluator emits SprintContractEvent(Proposed) on receive and SprintContractEvent(Approved) or
(Rejected) on verdict. Rejected returns a concrete objection (what is missing, what is ambiguous, what the scope
creep risk is) as a structured field — not a generic "needs improvement".
RALPH LOOP (bounded trace reinjection): on exit, if the generated output fails the acceptance criteria (checked by
the i-17 TriLayerStack), the trace of the failure (the specific failing gate + line) is REINJECTED to the generator
for one retry (SprintContractEvent(RalphReinjected)). the Ralph loop is bounded at 3 total iterations. the 3rd
failure (no fix after 2 reinjects) escalates to the D-13 cross-family adversary (AdversaryEscalated) which produces
a concrete counterexample rather than just a score. the loop counter is SSB-tracked; an overflowing loop is a BLOCK.
IC-17 CONSISTENCY: the evaluator persona is a hostile-critic with alignment mandate — exactly the case where the
persona research supports retention. this is NOT in tension with i-32's persona removal; i-32 removes narrative
personas from FACTUAL/CODE roles; the evaluator here is an alignment-dependent role (bias removal, adversarial
scrutiny). the summary explicitly notes "Evaluator runs tau=0 with a hostile-critic role — the persona research
SUPPORTS personas exactly here (IC-17 consistency)."
EXCLUDED: any shared context between generator and evaluator (the isolation is the entire point); any unbounded
Ralph loop (the 3-iteration cap is non-negotiable); any acceptance criterion that cannot be deterministically checked.

## Existing ground truth
- IN-4: NO sprint_contract module — CONFIRMED absent; i-21 is a genuine BUILD
- H-37 plan-approval gate seam (run_interactive_gate machinery) — SHIPPED; the Sprint Contract rides this existing gate machinery (compose, do not fork; the contract is a new payload type, not a new gate framework)
- H-38 stop gate — SHIPPED; i-21 composes it for the loop-cap enforcement (third failure triggers stop + escalation)
- D-13 cross-family adversary — SHIPPED; the AdversaryEscalated arm calls into it (GD-1 do NOT re-implement)
- C-32 skeleton-first (MiniCritic shape) — SHIPPED; the evaluator posture mirrors this (compose the posture, not the module)
- i-1 supervisor — the isolation substrate (the Directive spawn carries only the sealed contract; generator context is pruned by the supervisor before the evaluator sees anything — this requires i-1 to ship first, or a temporary isolation shim that gets replaced)
- i-17 TriLayerStack — the acceptance-criteria checker (the Ralph loop trigger is a TriLayerStack verdict, not an ad-hoc check)
- "PLAN-asserted; executor re-greps per W-7": run_interactive_gate exact file:line; the H-37 contract-payload type; the D-13 adversary call surface

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- sprint_contract.rs: SprintContract struct (goal, in_scope_files, acceptance_criteria, iteration_count, out_of_scope_marker)
- contract validation: vague/open-ended acceptance criteria rejected pre-execution (structured check: must reference a test symbol, compile status, or wiring_contract field)
- isolated evaluator Directive (spawned via i-1 supervisor with pruned context; fallback isolation shim if i-1 not yet landed)
- approval gate: Proposed / Approved / Rejected with concrete objection fields
- Ralph loop: trace reinjection on TriLayerStack failure, bounded to 3 iterations (loop counter in SSB)
- 3rd-failure escalation to D-13 adversary (AdversaryEscalated row + concrete counterexample)
- SprintContractEvent SSB action_type (Kind: Proposed / Approved / Rejected / RalphReinjected / AdversaryEscalated)
DEFER (Tier-2 / gated):
- multi-contract sprint (a sprint that sequences multiple contracts with cross-contract dependencies): the seam for contract chaining ships now as a future extension point; the runtime is deferred
- automated acceptance-criteria generation (the generator proposes deterministically checkable criteria given the task spec): a quality-of-life addition; the gate ships with manual criteria first

## Integration
new module: crates/orchestrator/src/sprint_contract.rs (the struct, validation, Directive builder, loop counter).
modify: the H-37 run_interactive_gate dispatch seam (add SprintContract as a new payload type).
modify: search/ladder.rs or task_runner.rs (the outer retry loop replaced or supplemented by the Ralph loop within the sprint boundary — PLAN-asserted; executor confirms the exact seam at EXECUTING, W-7).
SSB action_type: +SprintContractEvent (Kind: Proposed / Approved / Rejected / RalphReinjected / AdversaryEscalated) — baseline+1.
frontend_pair: contract card as a sibling to the existing PlanApprovalCard (H-37 surface). the contract card shows
the structured fields, the evaluator verdict, and the Ralph-loop iteration counter. cross_lang_diff PASS required (W-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/sprint_contract.rs::SprintContract (the proposal struct + validation)
  - crates/orchestrator/src/sprint_contract.rs::RalphLoop (bounded loop counter + escalation logic)
  - SprintContractEvent SSB action_type with Proposed / Approved / Rejected / RalphReinjected / AdversaryEscalated discriminators
consumes_symbols:
  - H-37 run_interactive_gate machinery (why: the contract rides the existing gate seam; GD-1 do NOT fork the gate framework)
  - D-13 adversary (why: AdversaryEscalated calls into it; GD-1 do NOT re-implement the adversary)
  - i-1 supervisor (why: the isolated evaluator Directive is spawned via the supervisor's context-pruning path; GD-1)
  - i-17 TriLayerStack (why: the Ralph loop trigger is a TriLayerStack verdict; GD-1 do NOT duplicate the gate check)
  - H-38 stop gate (why: 3rd-failure loop-cap enforcement; GD-1)
reachable_from:
  - task_runner or search/ladder.rs sprint dispatch (PLAN-asserted; executor re-greps at EXECUTING, W-7; this is the production entrypoint that creates and evaluates a SprintContract)  [PRODUCTION task runner / search ladder]
  (contracts/ callers never satisfy R7 — the production task runner / search ladder is the reachable_from chain)
stub_ban:
  - NO shared context between generator and evaluator: the isolation is the structural invariant; any evaluator prompt that contains generator turns or scratchpad is a BLOCK
  - NO unbounded Ralph loop: the iteration cap is 3 (hard limit, SSB-enforced); a loop without a counter is a BLOCK
  - NO acceptance criterion that is free-text / non-deterministic: "the implementation looks correct" is REJECTED at validation time; only symbol references, compile status, wiring_contract fields pass
  - NO re-implementation of D-13 adversary logic inside sprint_contract.rs (compose via the D-13 interface — GD-1)
  - NO .unwrap()/.expect() on production paths; NO fabricated evaluator verdicts or loop counts
frontend_pair:
  contract card on the plan UI (PlanApprovalCard sibling). shows: goal, in_scope_files, acceptance_criteria (with determinism badge), evaluator verdict (Approved/Rejected + concrete objection), Ralph-loop iteration counter. Tauri IPC: SprintContractEvent SSB rows feed the card via the existing SSB subscription. cross_lang_diff PASS required (W-4). Justified.
observable_effect:
  trigger: "elai run --sprint --task-fixture fixtures/vague_contract.toml --emit-ssb"
  expect_ssb: "SprintContractEvent(Rejected) for a fixture with a vague acceptance criterion ('implementation feels complete') — evaluator verdict includes a concrete objection field naming the ambiguity. SprintContractEvent(RalphReinjected) on the first failed exit (TriLayerStack failure). SprintContractEvent(AdversaryEscalated) on the 3rd loop failure. journal shows evaluator's incoming payload contains ZERO generator turns (context isolation verified by row count)."
  expect_artifact: "the evaluator context provably contains no generator rows (journal join on session + role shows only the sealed contract payload). approval and execution rows byte-recompute under ELAI_SEED (GD-2). a --shared-context broken arm (evaluator sees full generator history) changes the verdict distribution and FAILS the contract test asserting zero generator rows in evaluator context (R2)."

## Grounding
- IN-4: NO sprint_contract module — CONFIRMED; genuine BUILD
- H-37 gate seam, D-13 adversary, i-1 supervisor: all PLAN-asserted with re-grep required at EXECUTING (W-7)
- IC-17 (persona correction): hostile-critic evaluator persona is KEPT here — alignment-dependent task, exactly where persona prompting helps per the persona research; this is NOT in tension with i-32 (which targets factual/code roles)
- +0.12 self-review bias figure: from the Consolidated draft / research dossiers; WebSearch-confirm the source paper before citing as a headline gate number (ID-5 applies to threshold derivation)
- Ralph loop pattern: from the RLHF / iterative refinement literature; no specific arXiv ID required for the bounded-retry concept; the bound (3) is a design choice, not a paper constant
- Consolidated Part IV + review 1.1 "sprint contracts + bounded reinject"
- SSB: +SprintContractEvent (Kind: Proposed/Approved/Rejected/RalphReinjected/AdversaryEscalated) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
H-37 run_interactive_gate (rides the existing machinery — do NOT fork it); H-38 stop gate (loop-cap enforcement —
do NOT re-implement the stop logic); D-13 adversary (AdversaryEscalated target — do NOT re-implement); i-1 supervisor
(isolation substrate — do NOT re-implement context pruning); i-17 TriLayerStack (Ralph loop trigger — do NOT
duplicate gate logic); C-32 MiniCritic skeleton-first (posture reference — do NOT absorb C-32 logic).

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule) — effort M.

End of I-21.
