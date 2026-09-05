# I-33: mechanical decision governance — CEFL, I6Q, commit-reveal for decision-type LLM calls

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever TRUST | grounding STRONG | nature BUILD | effort M | WAVE 2

- Premise: LLM calls that drive orchestration decisions (mode picks, gate advisories, classification choices)
  are structurally vulnerable to framing effects and vacuous-rationale attacks that verbal governance cannot
  close. Three Santander-derived mechanical primitives decouple compliance from task stress and are
  measurable locally (ID-5): CEFL externalization, I6Q quality validation, commit-reveal integrity.

## Steal/build
Three mechanical governance primitives, applied to all decision-type LLM calls in the orchestrator:

CEFL (Candidate Externalization for Fair Labelling): the orchestrator authors the candidate set; the model
scores or ranks it only. Model-invented options outside the externalized set are structurally unparseable
(the schema enforces the domain) — selection-by-negation is eliminated at the type level, not the prompt
level.

I6Q (rationale quality validation): a deterministic checker validates that rationales meet minimum length
and lexical diversity thresholds before the response is accepted. Vacuous compliant-looking rationales
("this is the best choice because it is best") are REJECTED with a GovernancePrimitiveFired(I6qRejected)
SSB row. The threshold is not model-settable; it is orchestrator-owned config.

Commit-reveal entropy integrity: stochastic gate choices (any gate that selects among options with a
random component) commit a blake3(seed || choice_index) BEFORE the selection is revealed to the model.
The row order in the SSB proves commit-then-reveal; an out-of-order reveal is R2-FAIL.

Model-authored candidate options are the EXCLUDED alternative — they allow the model to construct a
candidate set it can score favorably, which CEFL structurally forecloses.
Reveal-before-commit is the EXCLUDED alternative for stochastic choices — it allows ordering exploitation.

Governance metrics (CDL cosmetic-deadlock rate, DIU deferral information utilisation, FSR framing-success
rate) are ledgered per run under ELAI_SEED (GD-2). The dossier's measured decoupling (governance invariant
under task stress) is the design target; ELAI measures it locally before claiming it (ID-5).

## Existing ground truth
- IN-11: NO CEFL / I6Q / commit-reveal primitives (grounding map, grep-confirmed absent)
- PLAN-asserted: Phase 3F rules engine (the verdict plumbing target); executor re-greps per W-7
- PLAN-asserted: D-16 structured deferral (the refusal target for rejected rationales); executor re-greps per W-7
- PLAN-asserted: B-22 / H-37 gate decision call sites; executor re-greps per W-7
- PLAN-asserted: C-26 constrained decoding (scoring schema host); executor re-greps per W-7
- IS-18: prompt_decision primitive (E-25 surface, present at file level); executor re-greps per W-7

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/decision_governance.rs: CeflRequest (externalized candidate set), I6qValidator
  (deterministic checker), CommitRevealRecord (blake3 commitment + row ordering)
- Wire into the prompt_decision call sites (gate advisory, mode-pick, classification calls)
- GovernancePrimitiveFired SSB rows (CeflExternalized / I6qRejected / CommitRevealSealed)
- Governance metrics tile (CDL/DIU/FSR) in the dashboard — frontend_pair mandatory (W-6)
- Fixture: framing-inversion test (reversed field order), vacuous-rationale test, out-of-order commit test
- decision doc: workflow/reasoning/I-33-decisions.md

DEFER (Tier-2 / gated):
- Cross-gate correlation analysis of DIU trends — needs >=3 real runs (GD-4)
- Adaptive I6Q threshold calibration (post ID-5 local measurement)

## Integration
- new crates/orchestrator/src/decision_governance.rs (~200 LOC): CeflRequest, I6qValidator, CommitRevealRecord
- MODIFY prompt_decision call sites: wrap every gate-advisory / mode-pick / classification call with
  CeflRequest + I6qValidator
- seam from Phase 3F rules engine: GovernancePrimitiveFired rows route through the existing verdict plumbing
- frontend_pair: governance metrics tile (CDL/DIU/FSR) in the existing SettingsWindow / MonitorWindow governance
  panel — a new Svelte chip component; cross_lang_diff.py PASS required (W-4)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/decision_governance.rs::CeflRequest (candidate-externalization wrapper)
  - crates/orchestrator/src/decision_governance.rs::I6qValidator (rationale quality checker)
  - crates/orchestrator/src/decision_governance.rs::CommitRevealRecord (entropy integrity record)
  - ssb action_type::GovernancePrimitiveFired (Kind: CeflExternalized / I6qRejected / CommitRevealSealed) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
consumes_symbols:
  - E-25 prompt_decision primitive (the call site; GD-1 compose)
  - Phase 3F rules engine verdict plumbing (GovernancePrimitiveFired routing; GD-1)
  - D-16 refusal path (rejected rationale route; GD-1)
  - C-26 DecodingMode scoring schema (the CEFL output constraint; GD-1)
reachable_from:
  - prompt_decision -> decision_governance::CeflRequest::apply -> GovernancePrimitiveFired SSB row  [PRODUCTION gate-advisory surface]
  - gate advisory call at commit-reveal sites -> CommitRevealRecord::commit_then_reveal -> CommitRevealSealed row  [PRODUCTION stochastic-gate surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO model-authored candidate options (CEFL forecloses this structurally — if the schema parses a model-invented option, the CEFL wrapper is broken)
  - NO reveal-before-commit (SSB row order is the proof; an out-of-order arm FAILS R2)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
  - NO fabricated CDL/DIU/FSR measurements (GD-2: all metrics recomputable from SSB under ELAI_SEED)
frontend_pair:
  governance metrics tile (CDL/DIU/FSR) in MonitorWindow governance panel — new Svelte chip + Tauri read; cross_lang_diff.py PASS required (W-4).
observable_effect:
  trigger: "elai run --governance-probe --seed 42 (fixture set: framing-inversion, vacuous-rationale, out-of-order-commit)"
  expect_ssb: "GovernancePrimitiveFired rows: CeflExternalized for each decision call (candidate list in payload), I6qRejected for the vacuous-rationale fixture, CommitRevealSealed with commit_row_id < reveal_row_id for every stochastic choice — recomputable under same seed (GD-2)"
  expect_artifact: "framing-inversion fixture produces SAME decision regardless of field order (FSR ~0 on fixture set, measured and ledgered); the out-of-order-commit fixture FAILS (R2 — the SSB out-of-order arm exits nonzero); the CDL/DIU/FSR values render in the governance tile (cross_lang_diff.py PASS)"

## Grounding
- NL-vs-deterministic dossier (docs/prefableresearch/Research_NaturalLanguage-vs-Deterministic-Constraints.*): Santander mechanical-enforcement primitives, MCC 0.43->0.88 governance decoupling
- IN-11 (grounding map): CEFL/I6Q/commit-reveal absent — genuine BUILD
- ID-5 binding: local governance measurement before claiming decoupling holds on ELAI's tasks
- Trust-review A5 / B1 / B7: mechanical governance sequencing
- SSB: +GovernancePrimitiveFired (Kind: CeflExternalized/I6qRejected/CommitRevealSealed) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9)
- arXiv: AgentSpec 2503.18666, Agent-C 2512.23738 — WebSearch-confirm before headline

## COMPOSES-WITH
- E-25 prompt_decision primitive — compose the call site, do NOT re-implement the LLM call machinery
- Phase 3F rules engine — compose the verdict routing, do NOT re-implement the rules plumbing
- D-16 refusal path — the I6Q rejection target; compose, do NOT fork
- C-26 DecodingMode scoring schema — the CEFL constraint domain; compose, do NOT re-implement

End of I-33.
