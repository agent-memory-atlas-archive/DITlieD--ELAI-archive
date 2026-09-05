# I-12: evidence-gated playbook curator

- Track 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE | lever STEERING | grounding STRONG | nature BUILD | effort M | WAVE 3 | ID-6
- Premise: playbook and rule entries today promote through human editorial choice without a receipts ledger,
  making demotion invisible and learned content diverge from measured behaviour.
  An evidence-gated curator promotes entries only when linked to >=k SSB-verified successful outcomes,
  carries the measured lift, demotes by the same ledger, and exports verified content to the formats
  consumers (CLAUDE.md, AGENTS.md, skills) actually use.

## Steal/build
The ACE-shape (+10.6% [DESIGN — WebSearch-confirm before using as a target]) curator pattern is the prior
art; ELAI's differentiator is RECEIPTS: promotion requires >=k SSB-verified outcome links, carry a measured
lift from cheap on/off injection, and a merged rule must not regress the C-28 mutation suite (DSPy/MIPROv2
+13% [DESIGN] style metric-driven validation).
Phase 3F rules lifecycle (PROPOSED->SHADOW->ACTIVE) already exists as the promotion machinery; the curator
feeds it — never replaces it (IS-13 store precedent is the Compiled-AI zero-token execution seam; ID-6
governs the playbook as a self-tuned artifact requiring ledger-gate promotion).
Export: verified entries written to CLAUDE.md /AGENTS.md /skills export formats so learned content is
interoperable. Demote by the same ledger: a stale or regressing entry loses its ACTIVE status without
human edit intervention.
F-5 staleness demotion is the predecessor staleness-gate; the curator extends it with the evidence-count
criterion.

## Existing ground truth
- IS-13: compiled_pipeline.rs TemplateCrystallizer:293 + CompiledPipelineStore:669 (C-25 shipped) — the Compiled-AI zero-token execution seam and the storage precedent for I-12's curator (PLAN-asserted; executor re-greps per W-7)
- Phase 3F rules lifecycle: PROPOSED->SHADOW->ACTIVE promotion machinery (PLAN-asserted; executor re-greps per W-7)
- C-25 TemplateCrystallizer: success-count precedent (the pattern I-12 generalises with evidence-gating)
- E-8 GEPA: the optimizer lane whose metric the curator's lift measurement mirrors
- C-21 self-mod data tables: the mutation-test validation surface
- F-5 staleness demotion: the predecessor staleness gate that the curator extends
- ID-6 (I-0_INDEX.md): WEIGHTS-ARE-CONFIGS — any self-tuned artifact promotes ONLY through the ledger gates; evaluator must live outside the loop's write boundary (gate-freeze, R8)
- IS-16 (H Track-3/4 gate substrate): trial_ledger.rs + holdout_budget.rs + deflation_gate.rs — the ledger gates ID-6 references

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/rule_curator.rs (~220 LOC): PlaybookCurator, evidence-count promotion gate (>=k SSB outcome links), lift measurement job (on/off injection, offline tick), demotion by same ledger, C-28 mutation-suite regression check before any promotion
- +PlaybookEvent (Kind: Promoted/Demoted/ValidationRun/Exported) in ssb.rs / action_type_lock.rs
- `elai playbook export` CLI subcommand writing verified entries to CLAUDE.md-formatted / AGENTS.md / skills blocks with embedded evidence row IDs
- integration into the Phase 3F PROPOSED->SHADOW->ACTIVE promotion machinery (feed, not replace)
- offline tick scheduler entry (Layer-1A schedule, configurable interval)

DEFER (Tier-2 / gated):
- cross-corpus transfer validation (verifying exported rules transfer to a held-out corpus; depends on I-38 QDELTA baseline being established first — ID-5)
- automated CLAUDE.md update with a PR-diff preview (user-approval required; gated behind ELAI_PLAYBOOK_AUTOUPDATE_ENABLE registered in wire-dark-allowlist.toml with a promotion plan, per W-2 / R9)

## Integration
New module: crates/orchestrator/src/rule_curator.rs (~220 LOC)
Seams: Phase 3F rules engine (feed the PROPOSED->SHADOW->ACTIVE machine), C-28 mutation runner (IS-8: regression gate), IS-13 CompiledPipelineStore (export target format precedent), IS-16 trial_ledger.rs (the ledger gate surface)
CLI: `elai playbook export` added under the skills_cmd family (PLAN-asserted; executor re-greps per W-7)
frontend_pair: playbook panel rows in RulesDrawer (PLAN-asserted; executor re-greps per W-7)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/rule_curator.rs::PlaybookCurator (evidence-count gated promotion + demotion + lift measurement)
  - +PlaybookEvent (Kind: Promoted/Demoted/ValidationRun/Exported) in ssb.rs / action_type_lock.rs
  - `elai playbook export` CLI subcommand (exports verified entries with evidence row IDs embedded)
consumes_symbols:
  - Phase 3F rules lifecycle promotion machinery (IS from 3F plans — feed, never replace) — GD-1
  - compiled_pipeline.rs::CompiledPipelineStore (IS-13: storage format precedent) — GD-1
  - mutation_testing.rs::MutationRunner + EquivalentMutantCache (IS-8: regression check suite) — GD-1
  - trial_ledger.rs (IS-16: the ID-6 ledger gate surface) — GD-1
  - F-5 staleness demotion gate (predecessor; curator extends, never replaces) — GD-1
  - E-8 GEPA optimizer metric (the lift-measurement shape to mirror) — GD-1
reachable_from:
  - elai playbook export -> PlaybookCurator::export_verified()  [PRODUCTION CLI surface]
  - offline tick scheduler -> PlaybookCurator::run_promotion_tick()  [PRODUCTION background job]
  - (test callers in contracts/ or #[cfg(test)] do NOT satisfy R7 production reachability)
stub_ban:
  - NO zero-evidence promotion (the evidence-count gate is unconditional — bypassing it is R1)
  - NO merged rule that regresses the C-28 mutation suite (validation is mandatory before ACTIVE)
  - NO re-implementation of the Phase 3F promotion machinery (feed only)
  - NO ACE +10.6% headline as a target threshold (DESIGN-tagged; measure ELAI's own lift per ID-5)
  - NO DSPy +13% headline as a target threshold (DESIGN-tagged; same ID-5 constraint)
  - NO .unwrap()/.expect() on production promotion paths
  - NO blake2/sha2 (blake3 only)
  - NO self-evaluator in the loop's write boundary for promotion (R8/ID-6: gate-freeze required)
frontend_pair:
  Playbook panel rows in RulesDrawer — required; evidence-count + lift + demotion events are user-actionable signals (W-4/W-6).
observable_effect:
  trigger: "cargo run -p elai-cli -- playbook export --emit-ssb"
  expect_ssb: "PlaybookEvent{kind: Promoted, outcome_row_ids: [...], lift: N} for a rule meeting the >=k threshold; PlaybookEvent{kind: ValidationRun, regression_count: 0} before promotion; PlaybookEvent{kind: Demoted} for a stale/regressing entry"
  expect_artifact: "the exported AGENTS.md block contains every promoted entry with embedded evidence row IDs (grep-verifiable); a zero-evidence promotion attempt emits PlaybookEvent{kind: ValidationRun, outcome_count: 0} and is REFUSED with no ACTIVE state transition (R1); a rule whose merged form regresses >=1 C-28 case emits ValidationRun(regression_count>0) and is REFUSED (R1)"

## Grounding
- IS-13 (CompiledPipelineStore:293/669 — storage precedent), IS-8 (MutationRunner:354 + EquivalentMutantCache — regression suite), IS-16 (ledger gate substrate)
- Phase 3F rules lifecycle, C-25 TemplateCrystallizer (success-count precedent), E-8 GEPA (lift measurement shape), F-5 staleness demotion (PLAN-asserted)
- ID-6 (I-0_INDEX.md: WEIGHTS-ARE-CONFIGS; ledger-gate promotion requirement; evaluator outside write boundary)
- ACE paper (WebSearch-confirm arXiv ID before any headline number), DSPy/MIPROv2 (WebSearch-confirm +13% figure — tagged [DESIGN] until confirmed on ELAI corpus per ID-5)
- SSB: +PlaybookEvent (Kind: Promoted/Demoted/ValidationRun/Exported) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)

## COMPOSES-WITH
Phase 3F rules lifecycle (PROPOSED->SHADOW->ACTIVE machinery — the curator feeds this; never replaces it)
C-25 TemplateCrystallizer / IS-13 CompiledPipelineStore (storage format precedent)
C-28 mutation suite (IS-8: the regression gate on every promotion)
IS-16 trial_ledger.rs + holdout_budget.rs + deflation_gate.rs (the ID-6 ledger gates)
E-8 GEPA (optimizer metric shape for lift measurement)
F-5 staleness demotion (predecessor gate extended by the evidence-count criterion)

End of I-12.
