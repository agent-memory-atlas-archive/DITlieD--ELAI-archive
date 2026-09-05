# H-41: Live checker-coverage + checker-gap INSTRUMENT — measure the Mode-A fraction and rank where a sound checker is missing

STATUS: PLANNED (TRACK 6 GROUNDED-REASONING HARDENING; WAVE 6; in-process instrument over the shipped D-10 routing + D-11 checker registry)

Objective: D-10 ALREADY routes a Factual/Computational claim "constructive when possible, cross-family when not,
refusal otherwise" (grounding_mode.rs:157, Rule 5) and ALREADY firewalls the auditor (Auditor/Meta ->
ConservativeFallback) — so the sound-checker-FIRST preference is SHIPPED (RS-1). What is ABSENT (RG-41) is the
MEASUREMENT that turns "prefer a sound checker" into a managed program: there is no live production signal for (a)
the fraction of real claims this session that actually reached a sound constructive checker (Mode A) vs fell to a
cross-family LLM (Mode B) or refusal, and (b) a ranked CHECKER-GAP backlog naming the claim-classes that fell back
for LACK of a constructive checker. Per Kambhampati 2402.08115 (sound external verifiers work; LLM self/peer
verification collapses on reasoning), maximizing Mode-A coverage is THE high-value verification lever — every new
checker that moves a claim-class from Mode B to Mode A is a measured win. H-41 is the instrument that measures the
live Mode-A coverage ratio (journal-recomputable) and emits a CheckerGapDetected signal — classified by the Xu Table-3
complexity taxonomy so NP-complete / undecidable claim-classes are flagged "route to a sound solver or refuse, never
LLM-grade" at top priority (RD-3).

Estimated effort: ~1 session (M, ~6-8 h). In-process instrument; reuses the shipped grounding-mode + CheckerResultEmitted
  rows for the coverage roll-up (no variant for the ratio); ONE new signal variant for the gap.
Pre-requisites: NONE hard (D-10 select_grounding_mode + D-11 checker_registry SHIPPED). Composes D-10/D-11/D-21 + D-13.
  Reads RS-1/RS-2/RS-3 + RG-41 + RD-3 + the Xu-taxonomy note.
Parallel-safe with: H-39, H-40, H-42 (disjoint files).
Source: the Hallucination-Stations research thread (2026-06-04) + Kambhampati 2402.08115 + Xu 2401.11817 Table 3;
  _grounding/H-REASONING-GROUNDING-MAP.md RG-41 + RD-3 + the CITATIONS block. Doctrine: H-0_INDEX.md.

## Context

Kambhampati 2402.08115 finds LLM self/critique COLLAPSES on reasoning/planning (Game-of-24, graph-coloring, STRIPS)
and explicitly rejects "verification is easier than generation" for LLMs — while SOUND EXTERNAL verifiers DO work. Xu
2401.11817 (Table 3) names the task classes that FORCE hallucination for an LLM (NP-complete: SAT/SubsetSum;
combinatorial-Omega(2^n); undecidable: first-order entailment) — for those, the only formally sound escape is an
external solver or refusal, NEVER an LLM grade. ELAI's routing already encodes the preference (RS-1); the gap is that
it is not MEASURED, so "where do we most need a new checker?" is answered heuristically. H-41 instruments the live
Mode-A fraction and turns each Mode-B/refusal fallback-for-lack-of-checker into a ranked, taxonomy-classified backlog
item. This is an INSTRUMENT-FIRST keystone in the H-30 mold: measure the real objective (sound-coverage), do not just
assert the preference exists.

## Existing state (B/C/D/E/F/G coverage)
- grounding_mode.rs:110 select_grounding_mode + Rule 5 (:157 constructive-when-possible) + Rule 1 (Auditor/Meta ->
  ConservativeFallback firewall) — D-10 SHIPPED + CALLED at agent_pool.rs:2030. GroundingMode = {Constructive,
  FalsifiabilityCrossFamily, ConformalRefusal, OperationalDeferred, ConservativeFallback}. The preference H-41 MEASURES (does not add).
- checker_registry.rs CheckerResult + agent_pool.rs:2153 CheckerResultEmitter + recent_checker_results +
  commands_d11.rs checker_results_recent + CheckerResultRow (types.ts:3149) — D-11 SHIPPED. The constructive-checker lane.
- benchmarks/mode_routing_runner.rs (D-21) — SHIPPED: per-mode F1 over a FIXTURE corpus (offline accuracy). H-41 adds
  the LIVE production coverage signal D-21 does not provide.
- a live Mode-A coverage ratio + a checker-gap backlog signal — ABSENT (RG-41). The pure-BUILD surface (an instrument + one signal).

## Reads (mandatory before coding)
- _grounding/H-REASONING-GROUNDING-MAP.md: RG-41, RD-3, RS-1/RS-2/RS-3, the Xu-taxonomy note, the CITATIONS block.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grounding_mode.rs:110-200 (the routing rules + GroundingMode variants)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\checker_registry.rs (CheckerResult — the checker lane)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\agent_pool.rs:2030 (the live select_grounding_mode call site) + :2153 (CheckerResultEmitter)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\benchmarks\mode_routing_runner.rs (the offline F1 runner — distinct from this live signal)
- workflow/papers/hallucination-inevitable-2401.11817-interface.md (Table 3 task taxonomy) + the Kambhampati 2402.08115 finding.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; GD-2 determinism; RD-3.

## Tasks (WAVE 6 — the live coverage roll-up + the ranked gap signal)

1. **Live coverage roll-up (~40 LOC).** A read-only roll-up over the shipped grounding-mode decisions +
   CheckerResultEmitted rows for a session: `mode_a_fraction = #Constructive / #(Factual|Computational claims)`,
   plus the Mode-B / refusal / deferred fractions. Recomputable from the journal under ELAI_SEED (GD-2) — NO new
   action_type for the ratio (GD-1; it is a roll-up of shipped rows).

2. **CheckerGapDetected signal (~50 LOC).** At the select_grounding_mode call site (agent_pool.rs:2030), when a
   Factual/Computational claim routes to FalsifiabilityCrossFamily or ConformalRefusal BECAUSE no constructive checker
   in the D-11 registry matched it, emit a CheckerGapDetected SSB row (Pattern-2; baseline+1 from LIVE 220/497, re-read
   ssb.rs:2990 + action_type_lock.rs:624; first Track-6 bump fixes the stale e45 pin) carrying {claim_class,
   complexity_taxon, fell_to_mode}. The agent did not author this row — it is the off-agent backlog artifact.

3. **Xu Table-3 complexity taxonomy (~40 LOC).** A deterministic classifier mapping a claim to a complexity_taxon
   {DecidableHasChecker, DecidableNoChecker, NpCompleteOrCombinatorial, Undecidable, Unknown}. NP-complete/combinatorial
   and Undecidable taxa are flagged HIGH-priority: those MUST route to a sound solver or refuse, NEVER an LLM grade
   (Xu Corollary 1) — a CheckerGapDetected on those is a stronger backlog signal (a missing solver, not a missing checker).

4. **Ranked backlog read (~30 LOC).** `elai checker-gaps` (or a dashboard read) aggregates CheckerGapDetected rows into
   a backlog ranked by frequency x taxon-priority — the answer to "which sound checker buys the most Mode-A coverage next?".

5. **Decision doc workflow/reasoning/H-41-decisions.md (~70 LOC):** the Kambhampati sound>LLM rationale; the Xu Table-3
   taxonomy + the never-LLM-grade flag for NP-complete/undecidable; why this is an instrument not a new gate (the
   preference is already shipped — RS-1); the honest residue (coverage measures REACH to a sound checker, not that the
   checker is correct — a checker bug is H-26/H-27's domain; GD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/checker_coverage.rs::mode_a_coverage (the live roll-up)
  - crates/orchestrator/src/checker_coverage.rs::classify_complexity_taxon (the Xu Table-3 classifier)
  - crates/orchestrator/src/ssb.rs::SsbActionType::CheckerGapDetected (off-agent backlog artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/grounding_mode.rs::select_grounding_mode (the shipped routing decision — REUSE, GD-1)
  - crates/orchestrator/src/checker_registry.rs::CheckerResult (the checker lane — REUSE)
  - crates/orchestrator/src/agent_pool.rs (the live select_grounding_mode call site :2030 — the emit point)
reachable_from:
  - a real claim-grading turn (agent_pool::... -> select_grounding_mode) -> CheckerGapDetected when a Factual/Computational
    claim falls back for lack of a checker [PRODUCTION]
  - `elai checker-gaps` -> the ranked backlog roll-up over CheckerGapDetected rows
stub_ban:
  - NO NP-complete / undecidable claim (Xu Table 3) routed to an LLM grade as if verified (RD-3; route to solver or refuse) — FAIL
  - NO re-implementation of select_grounding_mode or the checker registry (GD-1 P0); the preference is consumed, not restated
  - the broken `--ignore-checker-gaps` arm MUST make mode_a_fraction mis-report (R2: an instrument that stays right when broken is a FAIL)
  - blake3 / no .unwrap()/.expect() in prod
frontend_pair:
  the Mode-A coverage ratio + the ranked checker-gap backlog in the trust cockpit (cross_lang_diff PASS).
  W-6 E2E: a checker-gap on an uncovered claim-class visible in the dashboard backlog.
observable_effect:
  trigger: "cd .elai_cc && drive a real task with (a) a Factual claim that HAS a matching D-11 checker and (b) a
    Factual claim with NO matching checker, NO env override"
  expect_ssb: "claim (a) routes Constructive and emits NO gap row; claim (b) emits a CheckerGapDetected row carrying
    {claim_class, complexity_taxon, fell_to_mode}; mode_a_fraction recomputes from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "`elai checker-gaps` ranks (b)'s class in the backlog; an NP-complete claim with no solver is flagged
    HIGH (never-LLM-grade); the broken `--ignore-checker-gaps` arm mis-reports the coverage (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md --run-trigger
rg "mode_a_coverage|CheckerGapDetected|classify_complexity_taxon" .elai_cc/crates/orchestrator/src --glob '!*test*'   # produced + wired
rg "select_grounding_mode" .elai_cc/crates/orchestrator/src/checker_coverage.rs                                        # the shipped router reused (GD-1)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/checker_coverage.rs                                               # MUST be 0
test -f workflow/reasoning/H-41-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (uncovered claim emits CheckerGapDetected; covered routes Constructive). cargo-green != DONE.
- [ ] W-2 default-ON: the coverage roll-up + gap signal run with no env override; kill-switch only.
- [ ] W-3 production entrypoint: the gap signal fires from the live agent_pool routing site, not a #[cfg(test)] caller.
- [ ] RD-3: a Factual/Computational claim with a checker routes Constructive (already shipped); NP-complete/undecidable flagged never-LLM-grade.
- [ ] GD-1: select_grounding_mode + checker_registry CONSUMED, not restated (P0 if re-implemented).
- [ ] GD-2 reproducible: mode_a_fraction re-computes byte-identically from the journal under ELAI_SEED.
- [ ] R2: the --ignore-checker-gaps broken arm mis-reports the coverage (the instrument is falsifiable, not vacuous).
- [ ] GD-5: the decision doc states coverage measures REACH to a sound checker, not the checker's correctness.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the gap signal (produced) fires at the live routing site (W-8) reached by a real grading turn (W-3).
- W-2: runs with no override; kill-switch only.
- W-3: live agent_pool routing; not test-only.
- W-4: frontend_pair = the coverage ratio + backlog cockpit; cross_lang_diff PASS.
- W-5: observable_effect is a runnable task -> a real journal row; in-process.
- W-6: a checker-gap backlog item visible in the dashboard (dashboard-smoker).
- W-7: grounding_mode.rs:110/:157 + checker_registry + agent_pool.rs:2030 grep-confirmed (RS-1/RS-2); the live coverage signal absent (RG-41).
- W-8: mode_a_coverage + CheckerGapDetected (produced) consumed by the backlog read + the dashboard.
- W-9: +CheckerGapDetected baseline+1 (Pattern-2); coverage ratio is a roll-up (no variant); first Track-6 bump fixes e45.
- W-10: in-process, no network/RunPod axis — single-tier.
- GD-1/2 + RD-3: compose; reproducible; sound-checker-coverage measured, taxonomy-classified.

## Tests to add
- crates/contracts/tests/h41_checker_coverage_contract.rs (~140 LOC): (a) a covered Factual claim routes Constructive,
  no gap row; (b) an uncovered one emits CheckerGapDetected with the right claim_class; (c) an NP-complete claim is
  taxon=NpCompleteOrCombinatorial + HIGH priority; (d) mode_a_fraction matches a hand-computed fixture; (e) the
  --ignore-checker-gaps arm mis-reports (R2); (f) ELAI_SEED twice -> identical ratio + backlog order.
- crates/contracts/tests/fixtures/checker_gap/ (a frozen claim set spanning covered / uncovered / NP-complete / undecidable).

## Files touched
- CREATE: crates/orchestrator/src/checker_coverage.rs (~130 LOC: roll-up + taxon classifier + gap emit)
- MODIFY: crates/orchestrator/src/agent_pool.rs (~20 LOC: emit CheckerGapDetected at the :2030 routing site)
- MODIFY: crates/orchestrator/src/ssb.rs (+CheckerGapDetected) + action_type_lock.rs:624 + e45:294 (if first Track-6 bump)
- MODIFY: crates/cli (+`elai checker-gaps` backlog read) + the trust-cockpit Svelte pair
- CREATE: crates/contracts/tests/h41_checker_coverage_contract.rs (~140 LOC)
- CREATE: workflow/reasoning/H-41-decisions.md (~70 LOC)

## Citations / Grounding
- Kambhampati 2402.08115 "On the Self-Verification Limitations of LLMs on Reasoning and Planning" (self-critique
  collapse; sound external verifiers work — the reason Mode-A coverage is the lever).
- Xu 2401.11817 Table 3 (NP-complete / combinatorial / undecidable task classes force LLM hallucination; external
  solver or refusal is the only sound escape — the never-LLM-grade flag). WebSearch-confirm both ids (feedback_paper_citation_verify).
- ELAI ground truth (RS-1/RS-2/RS-3, grep-confirmed 2026-06-04): grounding_mode.rs:110/:157; checker_registry CheckerResult;
  agent_pool.rs:2030 the live call site; mode_routing_runner.rs the offline benchmark.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); RD-3; GD-5 (coverage = reach, not checker-correctness).

## Open questions / risks
- The complexity_taxon classifier is heuristic on claim text/type; misclassifying a hard claim as DecidableNoChecker
  (low priority) under-ranks it. Calibrate the classifier on a labelled claim fixture; surface Unknown rather than guessing.
- mode_a_fraction is only meaningful once D-11 has a non-trivial checker set; on a near-empty registry it reads ~0 and
  the backlog is the whole claim distribution — that is the CORRECT (honest) signal ("invest in checkers"), not a bug.
- This is an instrument, not a gate: it does not BLOCK a Mode-B fallback; it MEASURES it. A gate that forces refusal on
  an uncovered NP-complete claim is a follow-on (compose D-16) once the backlog/coverage numbers justify it (GD-4).

## Next
H-39 fixes the reward an LLM verdict feeds; H-40 keeps decorrelated verifiers; H-41 measures how often we can avoid an
LLM verdict entirely by reaching a SOUND checker — the strongest verification class. Investing in D-11 checkers against
this backlog is the highest-leverage verification work the literature (Kambhampati/Xu) points at.

End of H-41.
