# I-37: EARS requirement objects — typed, hash-keyed, mutation-covered, verified-status in the cockpit

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever VERIFIED-INTENT | grounding STRONG | nature BUILD | effort M-L | WAVE 3

- Premise: the field's named open problem in specification-driven testing is the behavior harness —
  linking a natural-language requirement to a deterministic verified/unverified status without a human
  reviewer in the loop. IntentPair (IS-15, C-29) is the nearest substrate but captures one
  intent-evidence pair, not a structured typed requirement with bound tests and coverage tracking.
  I-37 builds the missing layer: EARS-pattern requirement objects as typed records, each carrying a
  content hash, bound tests, I-20 mutation coverage on those tests, metamorphic relation bindings, and
  a live VERIFIED/UNVERIFIED status rendered per hunk in the G-5 cockpit.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule).

## Steal/build
EARS (Easy Approach to Requirements Syntax) provides five stereotyped patterns (Ubiquitous, Event-driven,
State-driven, Option, Unwanted-behaviour) that structurally decompose natural-language requirements into
machine-parseable intent shapes. Each I-37 EARS record carries:
  - content_hash: blake3 of the canonical requirement text (drift detector)
  - pattern: the EARS stereotype (enum, not freeform string)
  - bound_tests: Vec<TestSymbol> — the tests that the record claims covers this requirement
  - mutation_coverage: the I-20 sampled mutation score on the bound_tests at the time of the last audit
  - metamorphic_relations: Vec<MrId> — the IS-8 metamorphic relations that bind to this requirement
  - verified_status: Satisfied / Unverified / Regressed (Satisfied = mutation_coverage >= floor AND MRs hold)

Import interop: spec-kit / Kiro-style spec files (YAML/markdown frontmatter requirement lists) round-trip
into typed EarsRequirement records at PLANNING time; ELAI proves satisfaction, the spec-kit captures intent.
This composes IntentPair (IS-15) as the simplest EARS record (an IntentPair has an intent and an evidence
gate — it becomes an Ubiquitous-pattern record with one bound test and one metamorphic relation).

The open problem: "satisfied = covered by T1,T2 at 88% sampled mutation + MR-3 holds" — or honestly
UNVERIFIED. An unverifiable requirement is SHOWN as unverified in the cockpit, never silently dropped (R1).

The G-5 cockpit renders per-requirement status chips per hunk via the H-52 anchoring mechanism. Review time
is the measured bottleneck (landmine #14: METR 19% slowdown; AI-PR review waits 2.5-5.3x); the cockpit
surface directly targets reviewer attention efficiency.

## Existing ground truth
- IS-15 (grounding map): intent_pair.rs IntentPair:86 / IntentPairSet:163; evidence.rs IntentPairsGate:658 (C-29 shipped) — the substrate this generalizes (GD-1; IntentPair becomes the simplest EARS record)
- IN-7 (grounding map): NO EARS requirement object — confirms BUILD; IntentPair is the nearest substrate
- PLAN-asserted: C-50 ContractSpec (the PLANNING gate extension point); executor re-greps per W-7
- PLAN-asserted: G-5 cockpit (the render surface — per-hunk chips); executor re-greps per W-7
- PLAN-asserted: H-52 anchoring mechanism (the cockpit chip anchor pattern); executor re-greps per W-7
- IS-8 (grounding map): metamorphic_testing.rs RelationLibrary:181 — the MR binding source (GD-1)
- I-20 dependency: mutation_coverage field populated by I-20's sampled score (WAVE 3 ordering)

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/requirements.rs: EarsRequirement (typed record per above), EarsRequirementSet, verify_requirement() (mutation coverage + MR check)
- Import interop: parse_speckit_file() -> Vec<EarsRequirement> (YAML/markdown frontmatter)
- PLANNING gate extension: requirements authored/imported before TESTING_FIRST (composes C-50 ContractSpec)
- RequirementVerified SSB rows (Satisfied / Unverified / Regressed)
- Cockpit render: per-requirement status chips in G-5 via H-52 anchoring
- Fixture: 3 EARS requirements with bound tests and mutation scores; deleting a bound test -> Regressed; unverifiable requirement -> Unverified (shown, not dropped)
- decision doc: workflow/reasoning/I-37-decisions.md

DEFER (Tier-2 / gated):
- Formal obligation closure (H-54 sibling) — the formal-lane verifier integration; post Wave 3
- Cross-requirement dependency graphs (hierarchical EARS) — research-grade; post I-20 full-corpus run
- Automated spec-kit spec sync (webhook-triggered re-import) — infrastructure, not correctness

## Integration
- new crates/orchestrator/src/requirements.rs (~250 LOC): EarsRequirement, EarsRequirementSet, verify_requirement(), parse_speckit_file()
- MODIFY PLANNING gate (ContractSpec seam at C-50): requirements imported before TESTING_FIRST FSM transition
- seam to IS-15 IntentPair: EarsRequirement wraps IntentPair as its Ubiquitous-pattern base case
- seam to IS-8 RelationLibrary: metamorphic_relation binding lookup
- seam to I-20 mutation coverage: mutation_coverage field populated from I-20 sampled scores
- frontend_pair: per-requirement status chips in G-5 cockpit — Svelte chip via H-52 anchoring + Tauri read; cross_lang_diff.py PASS (W-4)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/requirements.rs::EarsRequirement (typed EARS record)
  - crates/orchestrator/src/requirements.rs::EarsRequirementSet (the per-task registry)
  - crates/orchestrator/src/requirements.rs::verify_requirement() (mutation + MR check)
  - crates/orchestrator/src/requirements.rs::parse_speckit_file() (import interop)
  - ssb action_type::RequirementVerified (Kind: Satisfied / Unverified / Regressed) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
consumes_symbols:
  - IS-15 intent_pair.rs IntentPair:86 (the base case; GD-1 — compose, the simplest EARS record)
  - IS-15 evidence.rs IntentPairsGate:658 (the gate seam; GD-1)
  - IS-8 metamorphic_testing.rs RelationLibrary:181 (the MR binding source; GD-1)
  - C-50 ContractSpec (the PLANNING gate extension point; GD-1)
  - G-5 cockpit + H-52 anchoring (the render target; GD-1)
  - I-20 mutation scores (the coverage field source; WAVE 3 ordering dependency)
reachable_from:
  - PLANNING gate -> ContractSpec -> requirements::parse_speckit_file -> EarsRequirementSet created  [PRODUCTION planning-gate surface]
  - audit cycle -> verify_requirement -> RequirementVerified SSB row  [PRODUCTION audit surface]
  - cockpit render -> EarsRequirementSet -> per-requirement chip in G-5  [PRODUCTION cockpit surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO silent drop of an unverifiable requirement (R1: missing evidence is hard fail — it renders as Unverified, always)
  - NO fabricated mutation_coverage values (GD-2: the coverage is recomputable from I-20's SSB rows under ELAI_SEED)
  - NO freeform-string pattern field (the EARS stereotype is an enum; a freeform string is a type violation)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  per-requirement status chips in G-5 cockpit via H-52 anchoring — Svelte chips + Tauri read; cross_lang_diff.py PASS required (W-4).
observable_effect:
  trigger: "elai run --ears-fixture --seed 42 (fixture: task with 3 EARS requirements, bound tests, I-20 mutation scores; delete one bound test then re-audit)"
  expect_ssb: "RequirementVerified rows: Satisfied for requirements with coverage >= floor AND MR holds (evidence pointers resolve to real test+mutation rows — join recomputable GD-2); Regressed row when a bound test is deleted (re-audit triggers status flip); Unverified for a requirement with no bound tests (shown, not dropped — R1)"
  expect_artifact: "an imported spec-kit file round-trips into typed EarsRequirement records (the round-trip is idempotent on the content_hash); per-requirement status chips render in G-5 cockpit (cross_lang_diff.py PASS); the Unverified requirement is visible in the cockpit, never silently absent (R2)"

## Grounding
- IS-15 (grounding map): IntentPair:86 / IntentPairsGate:658 — the substrate; confirms GENERALIZATION not BUILD-from-scratch
- IN-7 (grounding map): EARS requirement object absent — the BUILD scope
- Verification-economics review 2.2: requirement-harness open problem, per-requirement verified status
- Landmine #14 (I-0): review time is the bottleneck (METR 19% slowdown); cockpit targets this
- IS-8 (grounding map): RelationLibrary:181 — the MR binding source
- EARS pattern: Mavin et al. (Requirements Engineering Conference); the five stereotypes are public domain
- arXiv: METR 2507.09089 (AI-PR review waits) — WebSearch-confirm before headline
- SSB: +RequirementVerified (Kind: Satisfied/Unverified/Regressed) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9)

## COMPOSES-WITH
- IS-15 IntentPair / IntentPairsGate — the base case substrate; do NOT re-implement intent capture
- IS-8 RelationLibrary — the MR binding source; do NOT re-implement metamorphic testing
- C-50 ContractSpec — the PLANNING gate extension; do NOT re-implement the gate machinery
- G-5 cockpit + H-52 anchoring — the render mechanism; do NOT re-implement the anchoring protocol
- I-20 mutation scheduler — the coverage source; compose the field, do NOT re-implement mutation scoring

End of I-37.
