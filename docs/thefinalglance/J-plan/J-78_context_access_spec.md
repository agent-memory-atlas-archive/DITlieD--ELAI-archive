# J-78: context-access spec — formalize the Conductor's access_list as a first-class, journaled per-dispatch context-access contract

- Track J (vision-orchestration wave) | lever CANT-MISREAD | grounding STRONG (Conductor arXiv:2512.04388 access_list primitive verified live 2026-06-23; the Conductor paper's own ablation grounds it) | nature EXTENSION (I-5 + auditor firewall + 3E allocator) | effort M | WAVE VO-2 | depends none (EXTENDS I-5)

- Premise: the Sakana "Fugu" orchestration intake's STRONGEST genuine code-steal (O1). Fugu Ultra's Conductor (arXiv:2512.04388, 7B GRPO) generates a workflow spec as three lists — model_id / subtasks / ACCESS_LIST (who sees which prior outputs). ELAI has scoped context as an orchestrator HABIT (orchestrate/auto-orchestrate "a worker receives ONLY what its task needs" + the CONTEXT BUDGET ALLOCATOR 8 slots + the PRISTINE CONTEXT INJECTOR I-5 hard-excluding failed-sibling tracebacks + the AUDITOR-OBJECTIVITY FIREWALL giving the auditor ZERO memory) but "who sees which prior outputs" is an implicit habit, not a DECLARED, AUDITABLE list. J-78 turns it into an explicit per-agent access_spec — which prior SSB rows / sibling outputs this dispatch may read — composing I-5 + the auditor firewall + the 3E slots. Deterministic (TD-3), journaled (GD-2 recomputable), upgrading a habit into a verifiable contract. The cleanest genuine net-new from the whole Fugu intake.

## Steal/build
1. A DECLARED per-dispatch CONTEXT-ACCESS SPEC: each agent dispatch carries an explicit access_spec naming which prior SSB rows / sibling outputs / memory namespaces it MAY read. The orchestrator builds the dispatch context from the access_spec ONLY — a read outside the spec is a violation, not a silent inclusion. This makes I-5's pristine-context hard-exclude (failed-sibling tracebacks) and the auditor firewall (auditor gets ZERO memory) DECLARED rather than implicit.
2. JOURNALED + RECOMPUTABLE (GD-2): the access_spec for each dispatch is journaled so the context a worker actually saw is reproducible from the SSB stream under ELAI_SEED — the access decision becomes auditable (the RRA can show "this dispatch was allowed to read rows X,Y,Z and no others").
3. DETERMINISTIC (TD-3): the access_spec is computed by the orchestrator (a pure function of the dispatch role + sub-phase + the I-5/firewall rules), NOT proposed by the LLM. The Conductor LEARNS the access_list; ELAI DECLARES it deterministically — stronger on reproducibility, and ELAI's coordinator is a pure function (zero orchestration-model tokens) where Fugu pays a 0.6B coordinator / $5-$30 markup (O6: ELAI is ahead, no steal there).
EXCLUDED: O5 TRINITY latent hidden-state routing (REJECTED — requires white-box access to worker internals; ELAI's cloud wedge is text APIs with no hidden-state access, same structural ineligibility that rejected RecursiveMAS latent comms — the portable arm is the TEXT-level access/role/topology spec only); a LEARNED router that proposes the access_list (TD-3 — the spec is declared deterministically, not learned); a new orchestration topology engine (O3: novel-task topology discovery is answered by the SHIPPED/PLANNED I-4 evidence-ranked search ladder — select-by-receipts, NOT a trained router; recorded as grounding not a build).

## Existing ground truth (grep-confirmed 2026-06-23; re-grep at edit, W-7/W-9)
- PRISTINE CONTEXT INJECTOR [PLANNED(I-5)] hard-exclude failed-sibling tracebacks, contrastive constraint only (feature map 1.3). J-78 makes I-5's exclusion a DECLARED access_spec; EXTENDS I-5.
- CONTEXT BUDGET ALLOCATOR [WIRED] 8 fixed slots + floors+steal + cache breakpoints (phase-3E, context_budget.rs). The access_spec governs which prior-output content fills the SsbEntries / GrcHistory / CodeContext slots.
- AUDITOR-OBJECTIVITY FIREWALL (H-31): auditor/critic/adversary get ZERO memory injection (arXiv:2510.09905). The strongest existing access rule — J-78 makes it one case of the declared access_spec.
- orchestrate / auto-orchestrate scoped-context discipline (.claude skills): "a worker receives ONLY what its task needs" — the HABIT J-78 formalizes.
- SSB SHARED STATE BOARD [WIRED] single-writer WAL, append-only — the journal the access_spec is recorded in (phase-3B); the prior SSB rows the spec gates access to.

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (in-process, deterministic): the access_spec struct + the orchestrator builds dispatch context from it ONLY + the journaled access-spec row + a contract test that a dispatch cannot read outside its spec (the auditor-firewall + I-5 exclusion become declared cases). OBSERVE-ONLY first (GD-4 — journal the spec + flag would-be out-of-spec reads, before hard-enforcing, until the friction/FP is bounded on >=3 real orchestrate runs).
- No Tier-2 GPU dependency (pure orchestration plumbing).

## Integration
- crates/orchestrator (the dispatch path + I-5 pristine-context injector) — the access_spec struct + context-build-from-spec-only (~120 LOC; compose I-5, do not re-author the hard-exclude).
- crates/orchestrator/src/context_budget.rs (3E) — the access_spec gates which prior-output content fills the SsbEntries/GrcHistory/CodeContext slots.
- the auditor dispatch path — the auditor firewall (zero memory) becomes the canonical access_spec example (compose H-31, do not re-author).
- SSB: +ContextAccessSpecEvent (Pattern-2 single-variant; Kind: SpecDeclared / OutOfSpecReadFlagged / SpecEnforced). baseline+1 from LIVE ~269/578 (post-J-66; grep 2026-06-22), re-read at edit (W-9). PREFER-REUSE: if an agent-dispatch SSB row already exists, add the access-spec field there (net 0) — decide at edit.
- cross_lang pair: none (Rust-internal orchestrator); frontend_pair: the per-dispatch access-spec on the agent-pool / orchestrate surface (or W-6-justified absence).

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator access_spec struct + per-dispatch context-build-from-spec-only
- the journaled access-spec record + the out-of-spec-read flag
consumes_symbols:
- I-5 pristine-context injector (the hard-exclude becomes a declared access_spec case; NOT re-authored)
- H-31 auditor-objectivity firewall (the zero-memory rule becomes a declared case; NOT re-authored)
- 3E context_budget.rs 8-slot allocator (the spec gates slot content; NOT re-authored)
- SSB single-writer journal (the access-spec record + the prior rows it gates; NOT re-authored)
reachable_from:
- a real `elai` orchestrate / auto-orchestrate / task dispatch building a worker's context from its access_spec  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO LLM-proposed access_spec (TD-3 — the orchestrator computes it deterministically from role+sub-phase+I-5/firewall rules)
- NO latent / hidden-state routing (O5 REJECTED — text-level access spec only; no white-box worker access)
- NO silent context inclusion outside the spec (a read outside the spec is a flagged violation, not a default)
- NO re-authoring of I-5's hard-exclude or H-31's firewall (GD-1 — they become declared cases)
- NO enforced block before the friction/FP is bounded on >=3 real runs (GD-4 — OBSERVE-ONLY first)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
frontend_pair: per-dispatch access-spec on the agent-pool/orchestrate surface (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a real `elai` orchestrate run dispatching an auditor (firewall: zero memory) and an executor after a failed sibling (I-5: exclude the failed-sibling traceback), with the access_spec governing each dispatch context"
- expect_ssb: "ContextAccessSpecEvent Kind=SpecDeclared for each dispatch carrying the allowed-rows/outputs set; Kind=OutOfSpecReadFlagged if a build attempts a read outside the spec"
- expect_artifact: "the auditor's context provably contains ZERO memory rows; the executor's context provably EXCLUDES the failed-sibling traceback; the access_spec is journaled so the RRA reproduces exactly what each worker saw. NEGATIVE (R1 fail-closed): a dispatch whose access_spec is missing/empty gets the MINIMAL safe context (not a silent full-context dump); an out-of-spec read is flagged (OBSERVE) then blocked (ENFORCE), never silently allowed."

## Grounding
- Conductor arXiv:2512.04388 "Learning to Orchestrate Agents in Natural Language with the Conductor" (Sakana): emits model_id/subtasks/access_list -> dynamic topology; pool-randomized training => model-swap robust; ~3 steps; 735 tok/sample vs consensus 1412; GPQA-D 87.5 vs best single 84.8. The access_list IS the steal (O1). arXiv ID re-verify at edit; benchmark numbers are provider-reported / DESIGN-rationale (HD-1).
- TRINITY arXiv:2512.04695 (the latent-routing sibling, O5 REJECTED — white-box only). O6: TRINITY's "coordinator emits zero tokens, all generation in workers" = ELAI's main-llm-role strategist-never-executor, and ELAI's router is a PURE function (zero model tokens) — strictly cheaper than even the 0.6B coordinator; ELAI is ahead, no steal.
- O2 (CONFIRMATION, recorded not built): Thinker/Worker/Verifier triad + always-end-with-Verifier at a <=5-turn cap = ELAI's generator/evaluator (I-21) + mandatory AUDITING + BSP/cascade caps; the empirical 3-step-default/5-ceiling GROUNDS ELAI's existing turn/agent caps.
- O3 (RIDER, recorded not built): Fugu's only honest edge (novel/unclassifiable topology discovery, GPQA-D +5.2) is answered by I-4 (select-by-receipts best-of-N -> tournament -> PUCT) — a receipt-selected topology is reproducible (RRA) where a learned router is not. Record the Conductor's GPQA +5.2 as the grounding for WHY I-4 matters.
- O4 (GTM, not a code change): Fugu proves the orchestration-as-product market AND that it can be cheaper than a single frontier model; ELAI's deterministic orchestration is structurally cheaper (zero orchestration-model tokens) IF it matches quality — the quality number is BENCH-PENDING (SAFEDELTA G-6 / E-31), a bench priority not a unit.
- the "5x slower AND 5x pricier" premise is REFUTED as stated (grounded-by-default): the Conductor paper's own figure is 735 vs 1412 tok/sample (MORE efficient than naive ensembles); a real build test (paddo.dev 2026-06-23) had Fugu Ultra CHEAPER+FASTER than Opus 4.8 (Opus won on quality). The thing to beat is the learned router's novel-task topology discovery, answered by I-4.
- full synthesis: workflow/reasoning/vision-and-orchestration-improvements-2026-06-23.md PART B. LIVE SSB ~269/578 (post-J-66; grep 2026-06-22). re-grep at edit (W-9).

## COMPOSES-WITH
I-5 (the pristine-context injector — the hard-exclude becomes a declared access_spec case; EXTENDED), H-31 auditor-objectivity firewall (the zero-memory rule becomes a declared case), 3E context_budget allocator (the spec gates slot content), I-4 evidence-ranked search ladder (the deterministic answer to Fugu's novel-task edge — grounding O3, not duplicated), I-21 generator/evaluator (O2 confirmation), main-llm-role strategist-never-executor (O6 — ELAI's zero-token coordinator). Restates none.

## Open questions / risks
- spec granularity vs friction: too fine an access_spec is friction (every dispatch hand-specifies rows); the spec should be ROLE+SUB-PHASE-derived by default (the orchestrator computes it), with explicit overrides only where I-5/firewall already differ — measure the friction on >=3 real orchestrate runs before enforcing (GD-4).
- prefer-reuse vs new SSB: if an agent-dispatch row already journals the context window, add the access-spec field there rather than a new event — decide at edit (W-9).

End of J-78.
