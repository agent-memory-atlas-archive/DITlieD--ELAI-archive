# I-32: structural role scoping — strip personas from factual/code roles, keep where alignment-dependent

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever STEERING-TRUST | grounding STRONG | nature REFACTOR | effort M | WAVE 2 | binds IC-17

- Premise: narrative persona prompting measurably harms factual and code-generation accuracy (MMLU
  -3.6..-5.3pp; MT-Bench coding -0.65) while providing no mechanical compliance guarantee. Alignment-
  dependent roles (hostile critic, anti-spiral lateral, adversary) retain their framing because the
  research's own carve-out applies there. The flip is evidence-gated per ID-5: A/B on fixtures precedes
  any production change.

## Steal/build
The persona evidence splits cleanly: wholesale removal is declined (it would strip the auditor's hostile-
critic framing that drives adversarial coverage). The CORRECT split per IC-17: executor, context-compiler,
researcher, and all factual/code-generation agents lose narrative persona blocks in favor of schema-first
role contracts (typed I/O envelope, explicit output constraints, no "you are a X who Y" opener). Auditor,
critic, adversary, and anti-spiral's five lateral personas KEEP their framing — alignment-dependent task
performance depends on it.

The refactor is controlled: (a) each affected agent's system-prompt template is replaced with a compact
role contract (max-tokens=0 on the persona block, typed constraint list, I/O schema); (b) an A/B harness
runs both the old and new template against the existing E-19 fixture set per role; (c) the flip lands ONLY
if the new contract meets or exceeds the old persona on the fixture pass-rate (evidence-gated per ID-5).
Wholesale persona removal is the EXCLUDED alternative — declined because the alignment-dependent roles
(auditor, critic, anti-spiral) show the opposite effect.

IC-17 is the binding correction: the source dossier's "strip all personas" reading is too strong; the
research paper's own nuance is that structured role definitions outperform personas on factual/code tasks
but alignment-enforcement tasks are the exception. This session encodes that split into the code.

## Existing ground truth
- PLAN-asserted: Phase 2C per-role GRC configs (role substrate in agent dispatch configs); executor re-greps per W-7
- PLAN-asserted: .claude/agents/* prompt templates (each agent has a system-prompt section); executor re-greps per W-7
- PLAN-asserted: C-41 skills routing (the role-dispatch layer); executor re-greps per W-7
- IS-18: streaming_validator token presence in runtime_builder.rs + runtime_client.rs (role plumbing layer)
- IC-17: persona removal bounded to factual/code roles; alignment-dependent roles RETAINED (grounding map)

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- Schema-first role contract templates for executor, context-compiler, researcher (remove narrative persona block)
- A/B harness wired to E-19 probe fixture set; RoleContractApplied(AbDelta) emitted per role under seed
- Auditor/critic/adversary/anti-spiral lateral persona prompts: no change (PersonaRetained row emitted as proof)
- Evidence gate: the production flip requires the fixture pass-rate to be >= baseline on each affected role
- decision doc: workflow/reasoning/I-32-decisions.md

DEFER (Tier-2 / gated):
- Cross-role interaction effects (executor->auditor prompt chain effects) — measure in I-21 sprint context
- Automated per-commit regression A/B on new persona blocks — requires CI harness (post Wave 3)

## Integration
- new crates/orchestrator/src/role_contracts.rs (~120 LOC): RoleContract struct (typed I/O constraints,
  no narrative), RoleContractStore, apply_to_system_prompt()
- MODIFY crates/orchestrator/src/agent_dispatch.rs (or equivalent prompt-builder site): switch persona
  injection to role_contracts::apply for affected roles
- A/B harness: reuse E-19 probe infrastructure; emit RoleContractApplied SSB rows
- frontend_pair: N/A — prompt-internal refactor; no new dashboard surface (W-6 justified: the AbDelta
  rows render in the existing governance metrics stream). Justified.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/role_contracts.rs::RoleContract (schema-first role contract type)
  - crates/orchestrator/src/role_contracts.rs::RoleContractStore (per-role contract registry)
  - ssb action_type::RoleContractApplied (Kind: PersonaStripped / PersonaRetained / AbDelta) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
consumes_symbols:
  - crates/orchestrator/src/agent_dispatch.rs::build_system_prompt (the injection site; GD-1 compose)
  - E-19 probe fixture infrastructure (A/B harness reuse; GD-1)
  - IC-17 binding (persona split posture — grounding map, not re-implemented)
reachable_from:
  - agent_dispatch::dispatch_agent -> role_contracts::apply_to_system_prompt -> RoleContractApplied SSB row  [PRODUCTION agent-dispatch surface]
  - the A/B harness driven from `elai run --role-ab-probe` (real CLI entrypoint)  [PRODUCTION CLI surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO persona narrative in executor / context-compiler / researcher system prompts after the flip (the journal-grepped proof replaces assertion)
  - NO role flip without a preceding AbDelta row showing fixture pass-rate >= baseline (the gate is mechanical, not aesthetic)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only); NO fabricated measurements
frontend_pair:
  N/A — prompt-internal refactor; AbDelta rows surface in the existing governance metrics stream, no new Svelte component needed. Justified.
observable_effect:
  trigger: "elai run --role-ab-probe --seed 42 (or equivalent fixture runner on the affected role set)"
  expect_ssb: "RoleContractApplied rows per role: PersonaStripped for executor/context-compiler/researcher (AbDelta field carries fixture delta >= 0), PersonaRetained for auditor/critic/adversary/anti-spiral lateral — all under ELAI_SEED for GD-2 recomputability"
  expect_artifact: "the executor's outbound system prompt (logged to journal) contains the schema contract and ZERO persona narrative (journal-grepped); a regression in any role's fixture pass-rate FAILS the evidence gate and BLOCKS the production flip (R2 — missing evidence is hard fail)"

## Grounding
- IC-17 (grounding map, canonical binding): factual/code persona removal; alignment-dependent retention
- Persona dossier (docs/prefableresearch/Research_Persona-Prompting.*): MMLU -3.6..-5.3pp, MT-Bench coding -0.65, alignment-task reversal
- ID-5 binding: local fixture measurement before any flip becomes production policy
- E-19 probe infrastructure: A/B harness reuse (PLAN-asserted; executor re-greps per W-7)
- arXiv IDs from the persona dossier: WebSearch-confirm before headline
- SSB: +RoleContractApplied (Kind: PersonaStripped/PersonaRetained/AbDelta) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)

## COMPOSES-WITH
- Phase 2C agent dispatch configs (role substrate) — compose, do NOT re-implement the dispatch machinery
- .claude/agents/* prompt templates — modify in place, do NOT fork
- E-19 probe infrastructure — reuse harness, do NOT re-implement fixture loading
- C-41 skills routing — the role-dispatch key; compose only
- I-21 sprint contracts (evaluator persona kept by I-32's own rule; I-21 adds zero-shared-context)

End of I-32.
