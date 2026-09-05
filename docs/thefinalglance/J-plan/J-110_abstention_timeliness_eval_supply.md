# J-110: Abstention timeliness (SPL) + abstention-warranted eval supply — score WHEN the agent stops, not just whether

STATUS: PLANNED (WAVE STEAL-JUL04; S-M; EXTENSION(H-39/J-56/L-20) — a metric + a task-supply leg over the shipped abstention spine; from the agentic-abstention steal, steal-agenticsts-paw-abstention-2026-07-04.md A2+A3)

Objective: ELAI's abstention spine is complete on the WHETHER axis — D-16 conformal refusal decides, I-8 Aletheia aborts, L-7 policy-gates, H-39 scores correct/abstain/confident-wrong at +1/0/-t/(1-t) — but NOTHING scores WHEN. The agentic-abstention paper's measured headline is that lateness, not absence, is the dominant failure (agents abstain too late or never; <40% timely recall in every setting), and a run that burns 9 steps of budget before abstaining currently scores identically in ELAI to one that stops at step 1. J-110 ships two legs: leg A the timeliness metric — SPL = oracle_step / max(first_abstain_step, oracle_step) plus timely-recall (pass@0) and pass@k, computed OFF-AGENT from the SSB trajectory (the agent never authors it — exactly ELAI's gate shape), reusing the H-39 AbstentionScored family; leg B the denominator — a deterministic abstention-warranted task-mutation leg extending the J-56/L-20 eval corpus: derive should_abstain=true variants from mined real tasks via exactly one labeled defect (false_premise / contradiction / underspecified_intent) with a recorded oracle_step, plus a delayed pack whose infeasibility is only observable after acting. Together they give FEASIBILITY_PROBE (WIRED), B-38 ambiguity, B-40 critic, and L-7 their first measured number, in gate-ON-vs-OFF framing (GD-3), and hand J-84's very-hard=escalate-or-abstain tier a calibration source. The I-12 stopping-rules playbook category (distill from late-abstention failures, admit via J-109) is recorded as a RIDER on I-12, not re-authored here.

Estimated effort: ~1-2 sessions (S-M). Leg A is a trajectory fold + reused SSB family; leg B is a deterministic mutator over existing corpus rows + a bench arm on the reused runner family.
Pre-requisites (re-grep, W-7): H-39 AbstentionScored (SCHEMA 218) + abstention_score.rs (the reused verdict family + the RD-1 read-never-re-derive bit); D-16 RefusalDecision (the abstain bit's source); J-56 living eval corpus rows + L-20 commit-mined supply (`elai eval mine-commits` — the rows leg B mutates); K-15/I-27/J-107 bench runner family (REUSE, never fork; GD-1); FEASIBILITY_PROBE (B-21) + B-38 ambiguity score + B-40 critic + L-7 policy gate (the gates the ON-arm measures); I-38 >=5-seed paired floor.
Parallel-safe with: J-109 (disjoint: eval/bench vs memory/admission). Composes: late-abstention episodes flagged here are the input to the I-12 stopping-rules rider, admitted through J-109's pipeline.
Source: workflow/papers/agentic-abstention-summary.md (mechanisms + metrics at file:line) + workflow/reasoning/steal-agenticsts-paw-abstention-2026-07-04.md A2/A3/A4. Doctrine: J-0_INDEX (W/GD/TD/RD/ID inherited; ADDS no delta family). arXiv:2606.28733 verified live 2026-07-04.

## Context
Why timeliness is the missing axis: RD-1 already prices confident-wrong at -t/(1-t) and credits abstention at 0 — but that ledger is per-OUTCOME. Budget is per-STEP. On an infeasible task, every step between the point where infeasibility was knowable and the actual abstention is pure spend (tokens, wall-clock, sometimes side effects) that the current scorers cannot see. The paper's SPL is the right shape because it is (a) computable purely from the trajectory + a per-task oracle constant, (b) bounded [0,1] and 1.0 exactly when the stop was as early as knowable, (c) impossible for the agent to game without actually stopping earlier. ELAI already journals everything needed: FSM turns, tool calls, the D-16 RefusalDecision bit, I-8 acknowledge_infeasible. The only genuinely new data is oracle_step — which leg B supplies BY CONSTRUCTION: a mutated task records, at mutation time, the earliest step at which its injected defect is detectable (instruction-level defects: oracle_step=1, detectable at read time by FEASIBILITY_PROBE/B-38; delayed defects: the deterministic step at which the workspace/tool response first exposes the contradiction). No LLM labels an oracle (TD-3); the mutator that created the defect knows where it lives.

THE TWO LEGS:
  TL-1 METRIC (off-agent fold): for every eval episode on a should_abstain=true row, first_abstain_step = first turn whose terminal signal is D-16 refusal / I-8 abort / L-7 abstain-verdict; SPL, timely-recall (pass@0), pass@k over the abstain-warranted subset ONLY (never diluted by solvable rows); per-(model,lane,gate-arm) aggregation, >=5 seeds paired (I-38).
  TL-2 SUPPLY (deterministic mutator): from J-56/L-20 mined rows derive variants — exactly ONE defect per variant, labeled {false_premise | contradiction | underspecified_intent}, should_abstain=true, oracle_step recorded at construction; DELAYED pack: mutate the base workspace (e.g. the referenced symbol/file removed at base_sha-prep) so the defect is invisible in the instruction and first observable at a deterministic tool-result step. Solvable originals kept as the false-positive arm (a gate that abstains on solvable tasks is measured, not assumed — the honest-refusal SLA <5%/>50% flags, D-16 family).
  THE ARMS (GD-3, gate-ON-vs-OFF): FLAT (FEASIBILITY_PROBE/B-38/B-40/L-7 informative-off where flag-controllable) vs GATES-ON; y-axis = timely-recall + SPL + false-abstain rate on solvable twins + tokens burned past oracle_step; never a leaderboard number (paper numbers are context only, ID-5).

## Existing state (coverage; all treated as built per spec-eval)
- H-39 ABSTENTION-AWARE SCORING [1.2:326] AbstentionScored (218), +1/0/-t/(1-t) — the WHETHER axis + the SSB family TL-1 reuses (+Kind Timeliness).
- D-16 / I-8 / L-7 [1.2:565, 1.17:2677] — the abstain SIGNALS TL-1 folds over (read, never re-derived; RD-1).
- FEASIBILITY_PROBE [WIRED, 1.12:2338] + B-38 ambiguity + B-40 critic — the read-time gates the ON-arm finally measures (C-22 completeness noted "3 rate metrics synthetic proxies" — TL-2 replaces synthetic with constructed-real).
- J-56 LIVING EVAL CORPUS + L-20 COMMIT-MINED SUPPLY [1.13:2466] — the real-task substrate; TL-2 is a MUTATION leg on its rows, not a second miner (GD-1).
- K-15/I-27/J-107 runner family [1.13] — the bench harness TL-1's arms ride (REUSE; J-107 already added the abstention-rate channel; TL-1 adds the timing dimension).
- J-84 effort classifier [1.2:360] very-hard=escalate-or-abstain — consumes the measured knee (when abstention is typically knowable) as calibration.

## Reads (mandatory before coding)
1. workflow/papers/agentic-abstention-summary.md (metrics at wikimedia_evaluation.py:89-160, taxonomy at instruction_level_abstention_prompts.md + __delayed_* specs; the "computed over abstain-warranted samples only" rule at evaluation.py:82) + the steal file A2/A3.
2. abstention_score.rs + the D-16 RefusalDecision surface + I-8's terminal signal — re-grep the exact signal names (W-7).
3. L-20's corpus row schema {task_prompt, base_sha workspace, blake3 reference_diff} + J-56 EvalCorpusEvent — the row shape TL-2 extends.
4. K-15/I-27/J-107 runner + args_bench.rs; I-38 paired floor; CLAUDE.md ID-5/GD-3/TD-3/RD-1.

## Tasks (WAVE STEAL-JUL04)
1. Oracle-carrying corpus extension (~50 LOC): add {should_abstain, defect_class, oracle_step, twin_of} fields to the corpus row (additive; solvable rows default should_abstain=false, oracle_step=None); every abstention variant links its solvable twin.
2. Deterministic mutators (~140 LOC): false_premise (reference a symbol/behavior verifiably absent at base_sha — sourced from the DKG, so the premise is PROVABLY false, not LLM-invented); contradiction (two requirements that cannot co-hold, constructed from the reference diff's actual constraints); underspecified_intent (strip the disambiguating clause, recorded); delayed pack (workspace-prep removes the dependency the instruction implies, defect first observable at the deterministic tool step). Exactly one defect per variant; the mutator emits oracle_step + a machine-checkable defect witness.
3. TL-1 fold (~90 LOC): off-agent trajectory scorer over SSB episodes — first_abstain_step detection from D-16/I-8/L-7 signals; SPL/timely-recall/pass@k over the abstain-warranted subset; false-abstain rate over solvable twins; emit prefer-reuse AbstentionScored +Kind Timeliness (net 0..+1, lockstep W-9).
4. `elai bench abstention` arm (~60 LOC) on the reused runner family + args_bench.rs: FLAT vs GATES-ON arms, per-(model,lane), >=5 seeds paired; curve rows recomputable (GD-2); dashboard: reuse the convergence/eval tile, decide at edit (W-4).
5. Late-abstention flag export (~20 LOC): episodes with SPL < threshold (named const) emit the observe-only rows the I-12 stopping-rules RIDER consumes (H-31-firewalled; admitted via J-109 — recorded, not built here).
6. workflow/reasoning/J-110-decisions.md (oracle-by-construction rule, abstain-warranted-subset-only rule, the false-abstain twin arm, the I-12/J-109 closed-loop pointer).

## WIRING_CONTRACT
produces_symbols:
  - .elai_cc/benchmarks/abstention_runner.rs (TL-1 fold + the two-arm driver) + the corpus mutator module in the L-20 miner area + the `elai bench abstention` subcommand on args_bench.rs
consumes_symbols:
  - H-39 abstention_score.rs / AbstentionScored (+Kind; RD-1 read-never-re-derive) ; D-16 RefusalDecision + I-8 + L-7 terminal signals (the fold inputs)
  - J-56/L-20 corpus rows + EvalCorpusEvent (the mutation substrate; GD-1 no second miner)
  - K-15/I-27/J-107 runner family + I-38 paired floor (REUSE, never fork)
  - DKG (the false_premise witness source — provably-absent symbols)
reachable_from:
  - `elai bench abstention --model <m> --lane <local|cloud> --arms flat,gates --seeds 5` [PRODUCTION CLI bench dispatch, args_bench.rs]
  - `elai eval mine-commits --mutate-abstention` (the supply leg on the real miner entrypoint)
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-labeled oracle_step (mutator-constructed only, TD-3); NO LLM-judge abstention detection (typed D-16/I-8/L-7 signals; a lexical fallback may ANNOTATE, never decide)
  - NO scoring over the full corpus (abstain-warranted subset only — diluting with solvable rows fabricates high recall); NO citing the paper's 26.7/57.4/40% as ELAI numbers (ID-5)
  - NO forking the runner family (GD-1); NO single-seed claims (I-38); NO treating a solvable-twin abstain as free (the false-abstain arm is mandatory); NO .unwrap in prod
frontend_pair:
  the eval/convergence tile renders timely-recall + SPL + false-abstain per arm — reuse vs dedicated tile decided at edit (W-4).
observable_effect:
  trigger: "`elai eval mine-commits --mutate-abstention` emits variant rows with oracle_step + defect witness; then `elai bench abstention --arms flat,gates --seeds 5` on the fixture corpus produces the paired curves"
  expect_ssb: "prefer-reuse AbstentionScored +Kind Timeliness rows (one per episode) + EvalCorpusEvent rows for variants; net 0..+1 from LIVE ~268/577 (re-grep at edit, W-9)"
  expect_artifact: "recomputable per-(model,lane,arm) timely-recall/SPL/false-abstain curves + the late-abstention flag rows; NEGATIVE arms that must FAIL: a timeliness number emitted over rows lacking oracle_step (undefined SPL treated as 1.0 = fabricated earliness), and a variant row whose defect witness does not verify against the DKG/workspace (an LLM-invented premise)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-110_abstention_timeliness_eval_supply.md --run-trigger
rg "oracle_step|first_abstain_step|should_abstain|twin_of" .elai_cc/benchmarks/abstention_runner.rs   # the fold inputs present
rg -c "judge|llm.*label.*oracle" .elai_cc/benchmarks/abstention_runner.rs                              # MUST be 0 (TD-3)
rg "26\\.7|57\\.4|40%" .elai_cc/benchmarks/abstention_runner.rs                                        # MUST be 0 (ID-5, no paper constants)
python .claude/tools/production_caller_check.py --files .elai_cc/benchmarks/abstention_runner.rs
test -f workflow/reasoning/J-110-decisions.md
```

## ACCEPTANCE
- [ ] TL-2 variants carry mutator-constructed oracle_step + a machine-checkable defect witness (DKG-verified for false_premise); exactly one defect per variant; solvable twin linked.
- [ ] TL-1 computes SPL/timely-recall/pass@k off-agent from typed signals, abstain-warranted subset only; false-abstain rate on twins reported alongside (never hidden).
- [ ] Two-arm gate-ON-vs-OFF framing (GD-3); >=5 seeds paired (I-38); every number recomputable from SSB rows (GD-2); ELAI-measured only (ID-5).
- [ ] Late-abstention flags exported observe-only for the I-12 stopping-rules rider (H-31-firewalled; admission via J-109 recorded as the loop).
- [ ] cargo check --workspace GREEN; auditor PASS; production_caller 0-DARK (the two subcommands ARE the production callers).

## WIRING DONE-GATE (clause by clause)
- W-1: done = a real bench run produced timeliness rows + curves, not file-exists. W-2: no new default-OFF runtime flag (CLI bench arms, on-demand); the mutator flag rides the L-20 miner's surface.
- W-3: metric re-implemented from the summary's file:line grounding (Apache-2.0; no code copied). W-7: H-39/D-16/L-20/runner seams re-grepped at edit.
- W-9: prefer-reuse +Kind, net 0..+1, lockstep golden. RD-1: the abstain bit read from D-16/H-39, never re-derived. GD-3: ablation framing. ID-5/HD-1: paper numbers are motivation, never results.

## Tests to add
- crates/contracts/tests/j110_abstention_timeliness_contract.rs (~90 LOC): (a) SPL=1.0 iff first_abstain_step<=oracle_step, monotone-decreasing after; (b) an episode with no abstain signal on a should_abstain row scores 0 (never-abstained), not skipped; (c) rows without oracle_step are EXCLUDED and counted in a residue channel (never defaulted to SPL 1.0 — the negative arm); (d) false_premise witness that fails DKG verification rejects the variant at construction; (e) solvable-twin false-abstain is scored and surfaced; (f) same-seed re-run reproduces identical curves (GD-2); (g) <5 seeds rejected (I-38); (h) delayed-pack oracle_step equals the deterministic first-observable tool step in the fixture.

## Files touched
- CREATE: .elai_cc/benchmarks/abstention_runner.rs (~150 LOC: fold + arms + curve emission)
- MODIFY: the L-20 miner module (+~140 LOC mutators + row fields) + args_bench.rs (+~25 LOC subcommands)
- MODIFY: ssb.rs + .golden ONLY IF +Kind Timeliness added (lockstep, W-9)
- CREATE: crates/contracts/tests/j110_abstention_timeliness_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/J-110-decisions.md (~25 LOC)

## Citations / Grounding
- workflow/papers/agentic-abstention-summary.md: SPL/pass@k at wikimedia_evaluation.py:89-160 + docs/metrics.md; abstain-warranted-subset rule at evaluation.py:82; taxonomy at terminal/prompts/instruction_level_abstention_prompts.md (false_premise/contradiction/underspecified_intent, defect inferable from instruction alone) + __delayed_execution/__delayed_observation specs; measured lateness headline (<40% timely recall, project page). arXiv:2606.28733 verified live 2026-07-04. Apache-2.0 (Meta AbstentionBench derivative).
- ELAI ground truth: feature-map 1.2:326 (H-39/AbstentionScored 218), :565 (I-8), :360-365 (J-84 escalate-or-abstain), 1.12:2338 (FEASIBILITY_PROBE WIRED + C-22 synthetic-proxy note), 1.13:2446/2466 (J-107 abstention-rate channel; J-56/L-20 supply), 1.17:2677 (L-7).
- Doctrine: RD-1 (abstain bit read from D-16); GD-2/GD-3; TD-3; ID-5; I-38; H-31 (bench-lessons firewall, L-20 precedent).

## Open questions / risks
- Oracle honesty on delayed packs: "first observable step" must be deterministic across seeds (tool-order-stable fixtures); if a defect's observability depends on the agent's chosen tool order, record oracle_step as the EARLIEST possible step and label the residue (GD-5) rather than averaging.
- Mutation realism: constructed defects can be easier than organic infeasibility; keep the defect witness machine-checkable and report per-defect-class curves so an easy class cannot inflate the headline.
- Gate-off arm feasibility: FEASIBILITY_PROBE is WIRED with a skip-condition, not a flag in every path — the FLAT arm uses the honest available off-switches (ELAI_DISABLE_FEASIBILITY_PROBE exists per 1.12); where a gate cannot be disabled, the arm is labeled partial (GD-5), never silently approximated.

## Next
J-110 turns "ELAI abstains" from a design property into a measured, seed-paired, gate-attributable number: how OFTEN it stops when it should, how EARLY relative to the knowable point, at what false-abstain cost, and which gate (probe/ambiguity/critic/policy) buys the earliness. Its late-abstention flags feed the I-12 stopping-rules rider through J-109's admission pipeline — the closed measure->distill->prove->admit loop recorded in steal-agenticsts-paw-abstention-2026-07-04.md.

End of J-110.
