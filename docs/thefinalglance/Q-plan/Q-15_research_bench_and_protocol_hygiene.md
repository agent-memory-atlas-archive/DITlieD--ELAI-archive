# Q-15: research-bench + measurement-protocol hygiene — pareto specialist retention, a refutation power floor, a verified-provenance bit, a canonical variance helper (plus one handoff-doctrine rider)

STATUS: PLANNED, S-1 REJECTED-by-measurement 2026-07-17 (WAVE AUTORESEARCH-2; research-bench + protocol hygiene wave; S-1 is CLOSED and must NOT be built — B-SYN-2 refuted it at the preregistered effect size: P2 mean delta 0.1015625 vs the locked 0.25 floor, CI95 [0.055, 0.156] clearing only the CI>0 leg; per the permission ladder this closes the bench lane AND the research-lane variant is not authored at all; do not re-open the design to chase a passing number, that is the exact violation the B-SYN-2 session caught twice — see workflow/handoff/autoresearch-synergy-validation-plan-2026-07-17.md. S-2 is now UNBLOCKED by B-SYN-1 SUPPORTED. Remaining live: three code units + one doctrine rider — S-2 refutation power floor, S-3 verified-flag bench/catalog provenance, S-4 repeats + pass@k/std_err variance-helper shape, S-6 handoff mismatch-logging. Cohesive theme: the research-explore bench + the measurement protocols around it get the same de-slopping discipline Q applies elsewhere.)

Authoring provenance: paper-to-plan PHASE 5 over the autoresearch-synergy steal (conjecture engine run workflow/reasoning/conjecture-run-autoresearch-synergy-2026-07-17.md; survivors C5/S-1 + C6/S-2 + C7/S-3 + C8/S-4 as units + C10/S-6 doctrine rider, refuted=0; C9/S-5 recorded PENDING, no consumer). Auditors do NOT read the engine run artifact (SF-3) — anchors are re-greppable in-tree (W-7). Sources: Evo frontier_strategies.py (GEPA port), NeMo-RL + NeMo-Gym skills/servers (all vendor-reported, [EXTERNAL], never an ELAI number).

- Track: AUTORESEARCH-SYNERGY | lever: harden-the-research-bench-and-its-measurement-protocols | source: Evo frontier_strategies.py:251-377 (S-1), NeMo-RL underpowered-smoke guard + session-memory recovery (S-2, S-6), NeMo-Gym server_metadata.py:25 + reward_profile.py:310 (S-3, S-4) | grounding: STRONG for every gap (grep-confirmed in-tree/in-source), [EXTERNAL] for every vendor mechanism | nature: BUILD-on-shipped (S-1 EXTENSION of Q-12; S-2/S-3/S-4 harden the shipped bench/protocol; S-6 doctrine) | effort: M
- PROOF_SPINE: none (measurement-protocol + research-bench hygiene class). User-directed authoring 2026-07-17 (the instruction to author these plans from the synergy reasoning file IS the authoring authority); flagged for explicit ack review per PSR-2. S-1's bench-lane consumer touches the PS-4-adjacent explore-selection surface but the file claims no PS feed of its own.

- Premise: four measurement gaps the study exposed, each grep-confirmed. (1) Every ELAI frontier/selection surface is aggregate-scalar (research_tree.rs info_gain greedy; the B-3 bench portfolio; Q-12's planned bandit arms are aggregate-reward) — nothing retains a candidate that wins SOME task while losing the aggregate (`grep -icE "pareto|per_task|specialist|tasks_won" research_tree.rs` = 0), which is the brief's "discard intermediate high-performing checkpoints" anti-pattern at selection grain; Evo ships exactly this as a GEPA port (`_pick_pareto_per_task` at frontier_strategies.py:251). (2) The conjecture/bench protocol distinguishes INVALID-instrument from REFUTED-mechanism but nothing structurally requires a refutation to record its power (`grep -icE "power|minimal_detectable|underpowered" frontier_strategy_bench.py` = 0); NeMo-RL's underpowered-smoke guard is the anti-false-negative. (3) The growing bench/fixture/canary catalog has no per-artifact reviewed/baselined bit; NeMo-Gym stamps `verified: bool = False` (server_metadata.py:25). (4) The benches recompute variance ad hoc per instrument (P-25 + B-CREATIVITY hand-rolled bootstrap plumbing); NeMo-Gym threads `num_repeats` to pass@k/std_err through one helper (`compute_pass_majority_metrics`, reward_profile.py:310, `grep -icE "num_repeats|pass_at_k|std_err|majority" frontier_strategy_bench.py` = 0). Plus (5, doctrine) the handoff protocol re-verifies notes on resume but never records where they disagreed with reality (`grep -icE "mismatch|disagree|drift" handoff-protocol.md` = 0), losing the exact data that would let ELAI measure handoff drift (S-6).

## Steal/build

S-1 — pareto_per_task specialist retention (EXTENSION of Q-12's policy-selector seam; research_tree.rs + frontier_strategy_bench.py, bench-lane FIRST). A working port of GEPA candidate selection (per-task best -> tie collection -> set-cover dominance prune -> sample weighted by tasks-won, honoring per-task metric direction), added as a THIRD selectable policy in the same seam Q-12 builds for its bandit (Q-12 ships the policy selector + deterministic control arm; S-1 is a policy arm there, NOT a new selector). Benched under Q-11's equal-budget rule. The BENCH lane (frontier_strategy_bench.py with per-task scores) is the safe first consumer, because research-lane tasks may not decompose into stable per-task scores; the research_tree.rs policy arm lands with Q-12's seam. Ports Evo's `_pick_pareto_per_task` ranking logic (~130 lines pure ranking), NOT its runtime.

S-2 — refutation power floor (frontier_strategy_bench.py analysis + the preregistration protocol). A required POWER block on preregistrations and on REFUTED closeouts: K, paired-sample count, minimal detectable effect at CI95, and whether the observed CI width could have detected the preregistered effect. A refutation whose CI is too wide to have detected the claimed effect is recorded NULL-UNDERPOWERED, not REFUTED. Reuses the CI95 widths the analysis engine already computes. Composes Q-14's E-2: a within-noise kill IS a NULL-UNDERPOWERED candidate.

S-3 — verified-flag bench/catalog provenance (the bench/fixture/canary catalog). A per-artifact `verified: bool = False` provenance bit + a stamp discipline keeping every new bench/fixture/canary untrusted until human-baselined. Composes the Q-1 standing-goal re-check cadence (which re-checks verified invariants). A field + a default-false convention, not an effect claim.

S-4 — repeats + pass@k/std_err variance-helper shape (frontier_strategy_bench.py). Repeats-as-first-class-config threaded through to ONE canonical variance helper (pass@k, pass@1[avg-of-k], majority@k where applicable, std_dev/std_err, binary-vs-continuous handling), so the benches stop hand-rolling bootstrap plumbing and Q-11's per-dollar columns become comparable across benches. A design-shape steal rewritten in ELAI's stack; NO dependency on NeMo-Gym.

S-6 — handoff mismatch-logging (doctrine RIDER; .claude/rules/handoff-protocol.md + optionally a mismatch field in the handoff_state resume path). One protocol clause: on resume, after re-verifying load-bearing claims against the live tree, RECORD where the notes disagreed (a mismatch timeline entry), not just re-verify silently — that record is the handoff-drift data ELAI currently discards. Harness/doctrine; the optional field-add is the only code.

EXCLUDED: any claim that pareto retention is Weco's (it is Evo's GEPA port, arXiv:2507.19457, cited as ELAI's own port); making S-1 a NEW selector rather than a policy arm in Q-12's seam (it composes that seam); adaptive/rescaling anything in S-2 (the power block is declared, not tuned to reach a verdict); an LLM judge as the S-3 provenance oracle (the bit is a human-baseline stamp); a bench dependency on NeMo-Gym for S-4 (design-shape only); building S-5 campaign stop conditions (recorded PENDING — no live multi-session research-campaign consumer; see the coda).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)

- `crates/orchestrator/src/research_tree.rs::ResearchTree / ResearchNode / info_gain / ConvergenceStatus` — aggregate-scalar selection, breadth/depth caps only, no per-task/pareto/specialist retention (`grep -icE "pareto|per_task|specialist|tasks_won|set_cover"` = 0).
- `.claude/tools/frontier_strategy_bench.py` — paired-CI95 within-noise gate; NO power block (`grep -icE "power|minimal_detectable|underpowered"` = 0); NO first-class repeats/pass@k/std_err (`grep -icE "num_repeats|pass_at_k|std_err|majority"` = 0). NOT gate-frozen (safe to extend).
- The bench/fixture/canary catalog (`.claude/tools` + workflow/benchmarks) — no per-artifact reviewed/baselined bit today.
- `.claude/rules/handoff-protocol.md` — says re-verify load-bearing claims on resume; does NOT require recording the disagreement (`grep -icE "mismatch|disagree|drift"` = 0).
- Q-12 (Q-plan): the policy-selector seam in research_tree.rs (the deterministic control arm + the bandit arm) that S-1's pareto arm extends. Do NOT re-implement the selector (GD-1).
- Q-11 (Q-plan): the fixed-budget bench + `unbounded_best_of_n`/`parallel_widening` control arms + baseline — S-1's equal-budget apparatus. Do NOT re-implement (GD-1).
- Evo `_pick_pareto_per_task` (docs/.source/evo/plugins/evo/src/evo/frontier_strategies.py:251-377), NeMo-Gym `verified: bool = False` (server_metadata.py:25) + `compute_pass_majority_metrics` (reward_profile.py:310), NeMo-RL underpowered-smoke guard + session-memory recovery (docs/.source/nemo-rl-SUMMARY.md) — source anchors, verified present.

## REVISED scope (W-10 tiered)

- SHIP FIRST (S-4, cheapest + unblocks comparability): the canonical variance helper + repeats-as-config threaded through the bench. No behavior change to selection.
- SHIP SECOND (S-2): the required power block on preregistrations + REFUTED closeouts + the NULL-UNDERPOWERED reclassification. Reuses existing CI95 widths.
- SHIP THIRD (S-3): the per-artifact verified:false provenance bit + stamp discipline on the catalog.
- SHIP FOURTH (S-1, GATED on Q-12's selector seam existing): the pareto_per_task policy arm, benched on Q-11's equal-budget apparatus in the BENCH lane first. If Q-12's seam is not yet landed, S-1 ships as a bench-only ranking policy and the research_tree.rs arm lands with Q-12 (RECON-1).
- SHIP FIFTH (S-6): the handoff-protocol mismatch-logging clause + the optional resume-path field. Doctrine.

## Integration

- `crates/orchestrator/src/research_tree.rs`: the pareto_per_task policy arm added to Q-12's policy selector (S-1); the arm reads per-task scores where they exist, else the bench lane supplies them.
- `.claude/tools/frontier_strategy_bench.py`: the per-task-score bench consumer of the pareto policy (S-1), the power-block computation + NULL-UNDERPOWERED classification (S-2), the canonical variance helper + repeats config (S-4). Each new function is consumed by its bench CLI arm / report path in the same changeset (the production caller for a harness tool).
- the bench/fixture/canary catalog: the verified:false provenance bit + stamp check (S-3).
- `.claude/rules/handoff-protocol.md` (+ optionally the handoff_state resume path): the mismatch-logging clause + field (S-6).
- frontend_pair: N/A with justification — research-bench + measurement-protocol tooling, no dashboard surface this wave. Justified.

## MOVES (wargame)

M1 build the S-4 canonical variance helper (repeats-as-config -> pass@k / pass@1[avg-of-k] / std_err) and thread it through the frontier bench on a multi-repeat run.
EXPECTED_OBSERVATION: the bench report carries pass@k + std_err from the single helper; a prior hand-rolled variance path is removed (not duplicated); Q-11's per-dollar columns render comparably across two benches using the same helper.
ON_FAIL: two variance code paths now coexist (the helper was added beside the old one) -> delete the hand-rolled path and route it through the helper; a binary-vs-continuous score mis-handled -> branch the helper on score type per the NeMo-Gym shape.

M2 add the S-2 power block to a preregistration + a REFUTED closeout, then re-run the P-25 analysis leg through it.
EXPECTED_OBSERVATION: the P-25 families (real locked caps, real CIs) still classify REFUTED; a synthetic HALF-K run classifies NULL-UNDERPOWERED (its CI could not have detected the preregistered effect).
ON_FAIL: the power block reclassifies a genuinely-refuted family as NULL-UNDERPOWERED (too lenient) -> tighten the minimal-detectable-effect vs CI-width comparison to the preregistered effect size, not an inflated one; a synthetic half-K run still reads REFUTED -> the CI-width check is not wired into the verdict.

M3 add the S-3 verified:false provenance bit + stamp discipline and register a new bench fixture.
EXPECTED_OBSERVATION: a newly-added bench/fixture/canary artifact defaults `verified=false` until a human-baseline stamp flips it; a catalog load surfaces the unverified set.
ON_FAIL: the bit defaults true (untrusted artifacts read as baselined) -> default it false; the stamp is settable by the plan run that added the artifact (self-verification) -> the stamp is a separate human-review action, never the authoring run's own write.

M4 add the S-1 pareto_per_task policy arm to Q-12's selector seam and bench it (BENCH lane) against greedy/bandit at equal budget with per-task scores.
EXPECTED_OBSERVATION: on the Q-11 fixed-budget bench with per-task scores, pareto retention preserves at least one later-winning specialist that greedy/bandit prune, at equal spend, beyond the CI95 noise band; the arm reads per-task metric direction correctly.
ON_FAIL: pareto wins only by spending more -> Q-11's spend_leak flag fires and the win is void; re-cap at equal budget; if research-lane per-task scores do not exist, the arm stays bench-lane-only and the research_tree.rs consumer lands with Q-12 (FORK below).

M5 add the S-6 handoff mismatch-logging clause + optional resume-path field and resume a handoff whose notes disagree with the live tree.
EXPECTED_OBSERVATION: the resume records a mismatch timeline entry naming where the notes disagreed with git/fs, not just a silent re-verify; a clean resume (no disagreement) records an empty mismatch set, not nothing.
ON_FAIL: the resume re-verifies but records no mismatch entry (the drift data is still lost) -> add the required record clause; the record is prose-only with no greppable marker -> add a stable marker so drift is measurable over time.

## FORK

FORK: if Q-12's policy-selector seam in research_tree.rs is NOT yet landed when S-1 executes, take route B — S-1 ships as a BENCH-LANE-ONLY pareto ranking policy in frontier_strategy_bench.py (its safe first consumer), and the research_tree.rs policy arm is deferred to land WITH Q-12's seam (cross-linked); do NOT build a second selector in research_tree.rs that duplicates Q-12's (that is the composition breach the ABORT list forbids).

FORK: if the bench cannot supply stable per-task scores for S-1 (the tasks are aggregate-only), take route B — S-1 reduces to the bench lane with a SYNTHETIC per-task-score fixture (the frontier_strategy_bench DSL) as the falsifier apparatus, and the research-lane consumer is recorded as blocked-on per-task scoring; do NOT fabricate per-task scores on the live research lane.

## RECON_NEEDED

RECON_NEEDED-1: before S-1, confirm whether Q-12's policy-selector seam has landed (grep research_tree.rs for the policy-selector + control-arm symbols Q-12 adds); if absent, S-1 is bench-lane-only this wave (FORK-1). Confirm `info_gain` and any per-task score field are readable for the pareto ranking.
RECON_NEEDED-2: before S-2, re-read frontier_strategy_bench.py's CI95 computation to confirm the observed CI width is available per comparison (the power block reuses it, does not recompute); confirm the preregistration format has a place to record K + minimal-detectable-effect.
RECON_NEEDED-3: before S-4, re-read the existing bench variance/bootstrap paths to enumerate every hand-rolled site the helper must REPLACE (not sit beside); confirm binary-vs-continuous score handling matches the NeMo-Gym shape.
RECON_NEEDED-4: before S-6, re-read the handoff_state resume path to confirm whether a mismatch record needs a code field or is a pure doctrine clause; a pure-doctrine clause needs no code change, only the protocol edit.

## ABORT

ABORT: any proposal that (a) makes S-1 a NEW selector rather than a policy arm in Q-12's seam, (b) lets pareto retention win on more spend rather than better retention, (c) attributes the pareto/GEPA mechanism to Weco (it is Evo's port, arXiv:2507.19457), (d) makes the S-2 power block adaptive so a favored verdict is reachable, (e) defaults the S-3 verified bit to true or lets the authoring run stamp its own artifact verified, (f) leaves two variance code paths after S-4, or (g) makes S-6 a silent re-verify with no recorded mismatch -> STOP and flag. Each breaks a composition boundary, a funnel-discipline rule, or the exact gap the unit closes.

## WIRING_CONTRACT
produces_symbols:
  - research_tree.rs — the pareto_per_task policy arm in Q-12's selector seam (S-1)
  - frontier_strategy_bench.py — the per-task-score pareto bench consumer (S-1), the power-block + NULL-UNDERPOWERED classification (S-2), the canonical variance helper + repeats config (S-4)
  - the bench/fixture/canary catalog — the verified:false provenance bit + stamp check (S-3)
  - .claude/rules/handoff-protocol.md (+ optional handoff_state field) — the mismatch-logging clause (S-6, doctrine)
consumes_symbols:
  - research_tree.rs::ResearchTree / ResearchNode / info_gain / ConvergenceStatus (extend via Q-12's selector, do NOT re-implement, GD-1)
  - Q-12 policy-selector seam + deterministic control arm (S-1 is a policy ARM there, do NOT build a second selector)
  - Q-11 fixed-budget bench + control arms + baseline (S-1's equal-budget apparatus; S-2 reuses the CI95 widths; extend, do NOT restate)
  - Evo _pick_pareto_per_task ranking logic (port, do NOT vendor-depend), NeMo-Gym compute_pass_majority_metrics shape (S-4 design-shape, do NOT vendor-depend)
reachable_from:
  - `python .claude/tools/frontier_strategy_bench.py` selecting the pareto policy arm [PRODUCTION explore-bench CLI] (S-1 bench lane)
  - the research_tree.rs allocation/policy-selection site choosing the pareto policy [PRODUCTION research selector] (S-1 research lane, with Q-12's seam)
  - `python .claude/tools/frontier_strategy_bench.py` REFUTED-verdict path invoking the power block [PRODUCTION bench CLI] (S-2)
  - the catalog-load / stamp-check path reading the verified bit [PRODUCTION catalog gate] (S-3)
  - the bench metric-report path calling the variance helper [PRODUCTION bench CLI] (S-4)
  - the handoff_state resume path recording the mismatch [PRODUCTION handoff CLI] (S-6, if the code field is added)
  - a #[cfg(test)] / unit-test-only caller NEVER satisfies reachability (R7)
stub_ban:
  - NO new selector for S-1 — it is a policy arm in Q-12's seam; NO pareto win on higher spend (Q-11 spend_leak enforced, equal budget)
  - NO attributing the GEPA/pareto mechanism to Weco (it is Evo's port, arXiv:2507.19457)
  - NO adaptive/rescaling power block; the minimal-detectable-effect is compared to the PREREGISTERED effect, not a reachable target
  - NO verified bit defaulting true; NO authoring run stamping its own artifact verified (human-review only)
  - NO two variance code paths after S-4 (the helper REPLACES the hand-rolled sites); NO NeMo-Gym runtime dependency
  - NO silent re-verify in S-6 (the mismatch is recorded); NO producer-only fn (each bench/selector fn is wired to its CLI/selector site same changeset, R6); NO blake2/sha2
frontend_pair:
  N/A with justification — research-bench + measurement-protocol tooling, no dashboard surface this wave. Justified.
observable_effect:
  trigger: "python .claude/tools/frontier_strategy_bench.py on a multi-repeat, per-task-scored, equal-budget run selecting the pareto arm; re-run the P-25 analysis leg through the power block; register a new bench fixture; resume a handoff with disagreeing notes"
  expect_ssb: "N/A (harness bench + protocol tooling; the off-agent artifacts are the bench report rows, the frozen-checker exit codes, and the recorded mismatch entry, not SSB rows)"
  expect_artifact: "the bench report shows pass@k+std_err from one helper, a pareto specialist preserved at equal budget beyond noise, a NULL-UNDERPOWERED reclassification of a half-K run, a new fixture defaulting verified=false, and a recorded handoff mismatch entry. NEGATIVE: a pareto win with spend_leak, a genuinely-refuted family read as NULL-UNDERPOWERED, a fixture defaulting verified=true, two variance paths coexisting, or a silent re-verify with no mismatch record must FAIL"

PRODUCTION_HOOK: producer S-1 pareto ranking -> consumer Q-12's research_tree.rs policy selector AND the frontier_strategy_bench.py bench arm, wired same changeset (bench lane first per FORK-1); producer S-2 power block -> consumer the bench REFUTED-verdict path; producer S-3 verified bit -> consumer the catalog-load/stamp-check path; producer S-4 variance helper -> consumer the bench metric-report path; producer S-6 mismatch record -> consumer the handoff_state resume path (or the protocol clause if doctrine-only). No producer-only fn and no unit-test-only caller satisfies the hook (R6/R7).

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-15_research_bench_and_protocol_hygiene.md --run-trigger
rg -c "pareto|per_task|tasks_won" .elai_cc/crates/orchestrator/src/research_tree.rs   # MUST be >0 (S-1, with Q-12 seam)
rg -c "num_repeats|pass_at_k|std_err|power|minimal_detectable|verified" .claude/tools/frontier_strategy_bench.py  # MUST be >0 (S-2/S-3/S-4)
rg -c "mismatch|disagree" .claude/rules/handoff-protocol.md                             # MUST be >0 (S-6)
python .claude/tools/production_caller_check.py --files <changed .rs> --json            # no DARK new fn (S-1)
python .claude/tools/python_wire_check.py --files .claude/tools/frontier_strategy_bench.py --json  # no DARK new def
test -f workflow/reasoning/Q-15-decisions.md
```

## VERIFY

VERIFY: before declaring Q-15 done, the auditor runs off-agent —
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-15_research_bench_and_protocol_hygiene.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json` (S-1's pareto fn has a non-test caller in Q-12's selector or the bench, or a WIRE-DARK[Q-15] row)
`python .claude/tools/python_wire_check.py --files .claude/tools/frontier_strategy_bench.py --json`
plus the M2 synthetic half-K run classifying NULL-UNDERPOWERED while the real P-25 families stay REFUTED, and the M4 pareto arm preserving a specialist at equal budget beyond noise. Pass = the wire-checkers clean, one variance path (not two), the verified bit default-false and human-stamped, the pareto arm a policy in Q-12's seam not a new selector, and the mismatch recorded. Cargo/bench-green alone is necessary-not-sufficient.

INTEGRATION_SMOKE: an INDEPENDENT agent drives the FULL frontier bench end-to-end on a HARD input — a multi-repeat, per-task-scored, equal-budget run with a plateau-and-specialist structure — asserting: (a) the pareto arm preserves a later-winning specialist greedy/bandit prune, at equal budget (no spend_leak); (b) a synthetic half-K refutation reclassifies NULL-UNDERPOWERED while a well-powered one stays REFUTED; (c) variance comes from ONE helper across two benches; (d) a freshly-registered fixture defaults verified=false; and separately (e) a handoff resume with disagreeing notes records a mismatch entry. A single-repeat aggregate-only toy run does not fire the pareto/plateau path and does not satisfy R10.

## REDTEAM

REDTEAM: attack that SUCCEEDED against an earlier framing — "ship S-1 as its own second selector in research_tree.rs": patched by making it a policy ARM in Q-12's existing selector seam (bench-lane first), so no duplicate selector exists and the composition boundary holds. Attack that FAILED — "win the pareto bench by spending more": FAILS because Q-11's fixed-budget bench caps both arms and flags a spend-more win as spend_leak. Attack that FAILED — "record a genuinely-refuted family as NULL-UNDERPOWERED to reopen it": FAILS because the power block compares the CI width to the PREREGISTERED effect size and the P-25 families (real caps, real CIs) stay REFUTED in the M2 check. Attack that SUCCEEDED against an earlier framing — "let the plan run stamp its own new fixture verified=true": patched by defaulting the bit false and making the stamp a separate human-review action (S-3 stub_ban). Attack that FAILED — "add the variance helper beside the old bootstrap path and call it done": FAILS because the M1 check requires the hand-rolled path REPLACED, not duplicated (one variance path is the acceptance).

## Grounding

- Evo `_pick_pareto_per_task` (docs/.source/evo/plugins/evo/src/evo/frontier_strategies.py:251-377; GEPA port, arXiv:2507.19457 per evo-SUMMARY.md; agent-level, mechanism not isolated — [EXTERNAL], never an ELAI number). NeMo-RL underpowered-smoke guard + typed session-memory recovery (docs/.source/nemo-rl-SUMMARY.md — [EXTERNAL], vendor skill). NeMo-Gym `verified: bool = False` (server_metadata.py:25) + `compute_pass_majority_metrics` (reward_profile.py:310) — [EXTERNAL], NeMo-Gym server code. All ground the DESIGN need, never an ELAI effect number.
- ELAI ground truth: research_tree.rs (aggregate-scalar, no pareto), frontier_strategy_bench.py (CI95 gate, no power block, no first-class repeats, NOT gate-frozen), the bench catalog (no verified bit), handoff-protocol.md (re-verify but no mismatch record). Re-greppable (W-7).
- CLAUDE.md hard rules: no .unwrap/.expect on prod; scoped tests; GD-2 journal/report-recomputable numbers; the pareto arm composes Q-12 (GD-1 compose-not-restate); [EXTERNAL] tags never asserted or summed (HD-1/KD-1); every arXiv id (2507.19457) verified via the source summary.

## COMPOSES-WITH

Q-12 policy-selector seam (S-1 is a policy arm there, EXTENSION), Q-11 fixed-budget bench + control arms + baseline (S-1's apparatus; S-2 reuses the CI95 widths), Q-1 standing-goal verified-invariant re-check (S-3 provenance cadence), Q-14 E-2 quantified KILL-EDGE tail (S-2's within-noise kill = a NULL-UNDERPOWERED candidate), B-40 ResearchReport/SubQuestionResult, frontier_strategy_bench.py, the handoff protocol + handoff_state. Re-implements NONE (GD-1).

## DEFERRED-CONDITIONAL CANDIDATE (recorded here, NOT a build unit)

DEFERRED-S5 campaign-grain typed stop conditions (crates/orchestrator TaskSpec substrate + Q-1 standing goals): convert a user budget to an absolute deadline + target_experiment_count + per_experiment_timeout at campaign start, count attempts including crashes, re-check before AND after each run (NeMo-RL stop-condition pattern, [EXTERNAL]). BLOCKED-ON: a real multi-session RESEARCH-CAMPAIGN consumer (a Cosmos/explore effort spanning sessions) to enroll the typed stop object — ELAI has per-run spend caps (Q-11) but no campaign-grain consumer today, so the object would be wire-dark. EXTENSION of existing budget machinery; re-promote the moment a campaign consumer lands (its falsifier flips from "no consumer" to "enroll into that consumer"). User-ack required if promoted.

## Open questions / risks

- S-1's research-lane consumer depends on Q-12's seam landing AND per-task scores existing on the research lane; the bench lane is the safe first consumer and the honest fallback (FORK-1/2), declared (GD-5).
- The S-2 power block's minimal-detectable-effect must be compared to the PREREGISTERED effect, not an inflated one, or it becomes a reopening loophole; the M2 P-25 check is the guard.
- S-3's stamp discipline must resist self-stamping by the authoring run; the human-review separation is the acceptance, not a convenience to relax.
- S-6 is doctrine-first; the code field is optional (RECON-4) and only added if a pure clause cannot make drift measurable.

## Next

Hardens the research-explore bench and its measurement protocols: pareto retention stops discarding specialists the aggregate hides (S-1, composing Q-12), a refutation must prove it had the power to refute (S-2, composing Q-14's E-2), the bench catalog records what was ever human-baselined (S-3), and one variance helper makes Q-11's per-dollar columns comparable (S-4); plus the handoff protocol starts measuring its own drift (S-6). Pairs with Q-11 (the apparatus), Q-12 (the selector seam), and Q-14 (the quantified-kill sibling). S-5 waits for a campaign consumer.

End of Q-15.
