# J-107: `elai bench longcot` — the LongCoT recursive-scaffold-vs-flat measurement arm, and the instrument that gates J-108

STATUS: PLANNED (WAVE RLM-1; S-M; BUILD-on-shipped — a new bench arm reusing the K-15/I-27 runner; the gating instrument for the whole RLM wave)

Objective: the RLM corpus yields exactly ONE clearly-buildable net-new unit (rlm-recursive-agents-grc-2026-06-26.md PART 6 NET): adopt the open MIT LongCoT harness (LongHorizonReasoning/longcot + the HF dataset, RG-2) as an ELAI-measured reasoning-bench arm, two arms — a CONSTRAINED recursive-context scaffold ON vs flat single-shot OFF — ELAI-measured per-(model,lane), receipts on the y-axis. This is BOTH a standing differentiator measurement AND the F4 instrument: it answers the single open question for the RLM steal — does a constrained recursive-slice primitive LIFT the local Qwen-7B lane on long-horizon reasoning WITHOUT tripping the J-55 code-mode collapse (RG-6). Its measured knee is the K-15-class done-gate (ID-5) that GATES J-108: J-108's local-lane default-ON is authored ONLY if this arm clears. Hard boundary: this ships NO new reasoning architecture and NO model-chosen orchestration (RG-1 G6 anti-goal); it is a bench arm + a recomputable curve. GD-1 already-shipped, do NOT re-implement: K-15's `elai bench reasoning-budget` runner shape (RS-5), I-27 ConvergenceCurveMeasured + bench_convergence + ConvergencePanel, J-34 calibration artifact, I-38 QDELTA statistical floor.

Estimated effort: ~1 session (~S-M). A LongCoT task adapter + the two-arm driver over the existing bench runner + the curve emission + the knee extraction. The live local-lane run is RunPod-gated (W-10); the fixture/cloud arm ships now.
Pre-requisites (re-grep, W-7): K-15 `elai bench reasoning-budget` runner (RS-5, feature-map 1.13; reuse the bootstrap-CI/ELAI_SEED/receipt-y-axis shape — do NOT fork it); I-27 ConvergenceCurveMeasured + bench_convergence + ConvergencePanel; J-34 empirical context calibration (the sibling, x-axis = context-fill; this one's x-axis = imposed recursion/attention budget); args_bench.rs (the `elai bench` CLI dispatch); I-38 QDELTA (the >=5-seed paired floor). The constrained-scaffold ARM under test is J-108's primitive — so J-107 ships first with the flat arm + a STUB scaffold arm wired to J-108's seam (or co-sequences with J-108's WAVE RLM-2).
Parallel-safe with: J-108 (disjoint files: this is benchmarks/ + CLI; J-108 is orchestrator/search + memory). Sequence: J-107's flat arm + harness first; the scaffold-ON arm reads J-108's primitive (co-author or J-108-after).
Source: rlm-recursive-agents-grc-2026-06-26.md PART 5 (F2/F4) + PART 6 (LongCoT = NEW, the one clear build) + PART 7/8/9 (the measured shape the bench formalizes); _grounding/J-RLM-GROUNDING-MAP.md RG-2/RG-3/RG-6/RS-5. Doctrine: J-0_INDEX.md (W/GD/HD/ID inherited; ADDS no delta family).

## Context
Why this matters: the empirical RLM finding is ELAI's OWN harness>model thesis MEASURED — a scaffold (no fine-tuning) lifts small open models on long-horizon reasoning (RG-1). That lands directly on ELAI's local Qwen2.5-Coder-7B lane, the exact wedge. But the RLM scaffold works by the model writing Python over a context variable, and J-55 forbids general CodeAct on the sub-14B lane (~12% multi-tool collapse <14B). The two ELAI experiments already measured that a CONSTRAINED named-op/slice path sits BELOW that threshold (RG-6) while free-code fabricates (RG-4). J-107 turns that one-off measurement into a STANDING instrument on an OPEN, license-clean benchmark so the lift is judged per-(model,lane) on every model refresh, never on the paper's single-source-LOW leaderboard (RG-1 landmine; ID-5). The bench is the gate: J-108's local-lane default is contingent on this arm showing a real lift with no collapse.

THE TWO ARMS (what J-107 measures):
  LB-1 FLAT (OFF): single-shot reasoning at the imposed attention/recursion budget W — the baseline.
  LB-2 CONSTRAINED-SCAFFOLD (ON): the J-108 within-step recursive-context primitive (named-op slice + one capped child-spawn + HippoRAG hop) — NOT free-Python RLM (RG-4, REJECTED), NOT model-chosen orchestration (RG-1 G6).
  Y-AXIS: verified-outcome rate (receipts, SAFEDELTA-correct, GD-3) AND abstention-rate (abstention is GOOD when the answer is unreachable — the RG-4 trust property) AND tokens-per-correct.
  X-AXIS: imposed budget bands (the RG-3/RG-4 0.5W/2W/8W shape) and/or recursion depth.
  KNEE: the per-(model,lane) point past which more recursion/steps stop converting abstentions to correct answers (the K-15-class calibration source J-108's step budget + J-84/J-85's overthinking ceiling consume; ELAI-MEASURED, NEVER the paper 6K/7K, ID-5).

## Existing state (B/C/D/E/F/G/H/I/J coverage; RS-* in the grounding map)
- K-15 REASONING-BUDGET BENCH [PLANNED(K-15); BENCH-PENDING] `elai bench reasoning-budget` + the bootstrap-CI/ELAI_SEED/receipt-y-axis runner shape (RS-5, feature-map 1.13:2266-2287) — the runner J-107 reuses (GD-1).
- I-27 BUDGETED CONVERGENCE INSTRUMENTATION [DARK] ConvergenceCurveMeasured + bench_convergence + `elai bench convergence` + ConvergencePanel — the curve emission + tile J-107 rides.
- J-34 EMPIRICAL CONTEXT CALIBRATION [WIRED/PARTIAL] `elai bench context-fill` — the sibling (x-axis = context-fill); J-107 is the reason-budget sibling, NOT a second context bench (GD-1).
- I-38 QDELTA [PLANNED] the >=5-seed paired statistical floor; J-107 inherits it (no single-seed claims).
- LongCoT external: GitHub LongHorizonReasoning/longcot (MIT) + HF LongHorizonReasoning/LongCoT (RG-2, workflow/papers/longcot-bench-2604.14140-interface.md HIGH) — adapted, never vendored wholesale; the dataset is fetched, the harness's scoring re-implemented ELAI-side.

## Reads (mandatory before coding)
1. _grounding/J-RLM-GROUNDING-MAP.md (RG-2/RG-3/RG-4/RG-5/RG-6, RS-5, RC-3, the web landmine).
2. rlm-recursive-agents-grc-2026-06-26.md PART 6 (LongCoT NEW verdict) + PART 7/8/9 (the measured arm shape, the abstention-as-good property) + workflow/papers/longcot-bench-2604.14140-interface.md.
3. workflow/experiments/rlm-vs-elai-retrieval-2026-06-26.md + RLM-VS-ELAI-RESULTS-2026-06-26.md (the exact arm/band/metric shape to mirror) — these ARE the reference implementation of the experiment; the bench is its productionization.
4. The K-15 runner + I-27 bench_convergence + args_bench.rs (the `elai bench` dispatch) + J-34 — re-grep live (W-7). Reuse the runner; do NOT fork.
5. CLAUDE.md: ID-5 (ELAI-measured never paper constant), HD-1 (no multiplied literature figures), GD-2 (recomputable from the journal), GD-3 (receipts on the y-axis), I-38 statistical floor; J-55 (the local-lane CodeAct guard the scaffold arm must respect).

## Tasks (WAVE RLM-1 — the gating instrument)
1. LongCoT task adapter (~80 LOC): fetch/cache the HF LongHorizonReasoning/LongCoT dataset; ELAI-side deterministic scorer (exact-match / receipt-checkable, no LLM-judge, TD-3); the imposed-budget bands (mirror exp-1/exp-2's W/0.5W/2W/8W). NO wholesale vendoring of the MIT harness — adapt the task spec + scoring only (RG-2; license-clean).
2. The two-arm driver (~70 LOC): LB-1 flat vs LB-2 constrained-scaffold, over the K-15/I-27 runner (reuse bench_convergence; do NOT fork). The LB-2 arm calls J-108's primitive behind its seam; if J-108 not yet landed, the arm is a registered STUB wired to J-108's entrypoint (the gate runs flat-only until J-108 ships, and SAYS so — no silent half-measure).
3. Curve + knee emission (~40 LOC): emit ConvergenceCurveMeasured rows (reuse; +Kind LongCotArm only if a discriminator is needed, decide at edit, lockstep) per (model, lane, band, arm); extract the per-(model,lane) knee; the abstention-rate channel (abstention GOOD on unreachable). >=5 seeds paired (I-38 floor).
4. `elai bench longcot` CLI subcommand (~25 LOC) on args_bench.rs (the real entrypoint) + a ConvergencePanel/LongCot tile or justified reuse of the existing convergence tile (W-4, decide at edit).
5. The done-gate read (~10 LOC): the knee + the lift (LB-2 minus LB-1 verified-outcome rate) + the collapse check (LB-2 code-error/fabrication rate on the local lane MUST NOT exceed the flat arm — the RG-6 J-55 gate) become the recorded artifact J-108's authoring decision reads.
6. workflow/reasoning/J-107-decisions.md (the two arms, the abstention-as-good metric, the knee-is-the-gate-for-J-108 rule, the ID-5 never-cite-the-paper-number rule).

## WIRING_CONTRACT
produces_symbols:
  - benchmarks/longcot_runner.rs (the LongCoT adapter + two-arm driver over the K-15/I-27 runner) + the `elai bench longcot` subcommand on args_bench.rs
consumes_symbols:
  - K-15 reasoning-budget runner shape + I-27 bench_convergence / ConvergenceCurveMeasured / ConvergencePanel (REUSE the runner + curve + tile, do NOT fork; RS-5, GD-1)
  - J-34 calibration artifact (the sibling shape) + I-38 QDELTA paired-seed floor
  - J-108 RetrievalNavigator/HippoRAG primitive (the LB-2 arm-under-test seam; co-sequenced)
  - the HF LongHorizonReasoning/LongCoT dataset (adapted task spec + ELAI-side scorer; MIT, no wholesale vendoring)
reachable_from:
  - `elai bench longcot --model <m> --lane <local|cloud> --arms flat,scaffold` [PRODUCTION CLI bench dispatch, args_bench.rs]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-judge scorer (deterministic/receipt scoring only, TD-3); NO citing the paper leaderboard number as a result (ELAI-measured only, ID-5/HD-1)
  - NO forking the K-15/I-27 runner (reuse; GD-1); NO wholesale copy of the MIT harness code (adapt the spec + scorer)
  - NO single-seed claim (I-38 >=5-seed paired floor); NO treating abstention as failure (abstention is GOOD on unreachable answers — the RG-4 trust property)
  - NO routing the LB-2 scaffold arm to free-Python CodeAct on the local lane (RG-4/RG-6 + J-55); NO .unwrap on prod
frontend_pair:
  the existing ConvergencePanel renders the two-arm LongCoT curve + the knee + the abstention channel — decide a dedicated tile vs reuse at edit (W-4); justified reuse of the I-27 convergence tile is acceptable.
observable_effect:
  trigger: "`elai bench longcot --arms flat,scaffold --model <m> --lane local --seeds 5` (a REAL CLI run on a fixture corpus now, RunPod-gated for the live local model, W-10)"
  expect_ssb: "reuse ConvergenceCurveMeasured rows (one per model/lane/band/arm), net 0..+1 from LIVE ~268/577 (a +Kind LongCotArm decided at edit, lockstep, W-9)"
  expect_artifact: "a recomputable two-arm curve + a per-(model,lane) knee + the lift (scaffold minus flat) + the abstention-rate channel; the NEGATIVE arm that must FAIL: a run that emits a verified-outcome number with ZERO ConvergenceCurveMeasured rows behind it (fabricated curve), and a scaffold arm whose local-lane code-error/fabrication rate exceeds the flat arm (the J-55 collapse the gate must catch)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-107_longcot_recursive_scaffold_bench.md --run-trigger
rg "longcot|two.arm|flat.*scaffold|knee|abstention" .elai_cc/benchmarks/longcot_runner.rs   # the LB-1..LB-2 legs + the abstention channel
rg -c "llm.judge|judge_model|model.*scores.*answer" .elai_cc/benchmarks/longcot_runner.rs    # MUST be 0 (deterministic scoring, TD-3)
rg -c "6000|6144|7000|paper.*leaderboard|72\\.5|0->100" .elai_cc/benchmarks/longcot_runner.rs # MUST be 0 (no paper constant as a result, ID-5)
python .claude/tools/production_caller_check.py --files .elai_cc/benchmarks/longcot_runner.rs
test -f workflow/reasoning/J-107-decisions.md
```

## ACCEPTANCE
- [ ] LB-1/LB-2: two arms (flat vs constrained-scaffold) run over the REUSED K-15/I-27 runner (not a fork); the LB-2 arm calls J-108's primitive (or a registered stub that says flat-only until J-108 lands).
- [ ] Y-axis is receipts + abstention-rate + tokens/correct (GD-3); abstention treated as GOOD on unreachable answers (RG-4).
- [ ] knee + lift + the local-lane collapse-check are emitted as the recomputable artifact J-108's authoring reads (the gate); ELAI-measured, NO paper number cited (ID-5/HD-1).
- [ ] >=5 seeds paired (I-38); deterministic scorer, no LLM-judge (TD-3); no wholesale MIT-harness vendoring (RG-2 license).
- [ ] GD-1: K-15/I-27/J-34 reused not re-implemented; cargo check GREEN; auditor PASS -p the bench crate; production_caller 0-DARK (the subcommand IS the production caller).

## WIRING DONE-GATE (clause by clause)
- W-1: done = a real `elai bench longcot` run produced ConvergenceCurveMeasured rows + a knee, not file-exists.
- W-2: no new default-OFF gate (it is a CLI bench arm, invoked on demand); justified — a bench is not a runtime feature flag.
- W-3: BUILD-on-shipped — reuses the K-15/I-27 runner + the LongCoT spec adapted (not a summary-port of the MIT harness).
- W-4: rides ConvergencePanel (dedicated LongCot tile vs reuse decided at edit). W-5/W-10: fixture/cloud arm ships now; the live LOCAL-lane run is RunPod-gated (the separately-gated W-10 checkbox).
- W-6: ConvergencePanel surface. W-7: K-15 runner / I-27 / args_bench.rs / J-34 re-grepped at edit.
- W-9: reuse ConvergenceCurveMeasured; net 0..+1 (a Kind decided at edit, lockstep). ID-5: every number ELAI-measured; HD-1: no multiplied literature figures. GD-2: the curve recomputable from its rows.

## Tests to add
- crates/contracts/tests/j107_longcot_bench_contract.rs (~70 LOC): (a) the two-arm driver emits a ConvergenceCurveMeasured row per (model,lane,band,arm); (b) the scorer is deterministic (same input -> same verdict, no model call); (c) abstention is recorded as a distinct channel, NOT counted as a wrong answer; (d) the knee extraction is monotone-stable across a re-run with the same ELAI_SEED (GD-2 recomputability); (e) a result emitted with zero curve rows is rejected (R1 fabricated-curve negative arm); (f) the local-lane scaffold-arm collapse-check flags when code-error/fabrication exceeds the flat arm (the J-55 gate); (g) <5 seeds is rejected (I-38 floor).

## Files touched
- CREATE: .elai_cc/benchmarks/longcot_runner.rs (~190 LOC: adapter + two-arm driver + curve/knee emission)
- MODIFY: .elai_cc/crates/cli/src/args_bench.rs (~25 LOC: the `elai bench longcot` subcommand)
- MODIFY: the ConvergencePanel tile or a justified reuse note (~20 LOC dashboard) — decide at edit (W-4)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs + the .golden ONLY IF a +Kind LongCotArm is added (lockstep, W-9)
- CREATE: crates/contracts/tests/j107_longcot_bench_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/J-107-decisions.md (~25 LOC)

## Citations / Grounding
- rlm-recursive-agents-grc-2026-06-26.md PART 6 (LongCoT = NEW, the one clear build; F2 license-clean; F4 the bench doubles as the measurement instrument) + PART 7/8/9 (the measured arm/band/metric shape + the abstain-not-fabricate trust property).
- workflow/papers/longcot-bench-2604.14140-interface.md (arXiv:2604.14140, HIGH; the MIT harness + HF dataset are open) — RG-2; arXiv id grounded via the on-disk interface map (RC-3), not training memory.
- ELAI ground truth (RS-5): K-15 `elai bench reasoning-budget` runner + I-27 ConvergenceCurveMeasured/bench_convergence/ConvergencePanel + J-34 + args_bench.rs — re-grepped (W-7).
- OWN MEASURED DATA: workflow/experiments/{rlm-vs-elai-retrieval,RLM-VS-ELAI-RESULTS}-2026-06-26.md (the experiment this bench productionizes; the abstention-as-good + the J-55 collapse-check shapes).
- CLAUDE.md: ID-5 (ELAI-measured, never the paper 6K/7K knee), HD-1 (no multiplied literature figures), TD-3 (deterministic scorer, no LLM-judge), I-38 (>=5-seed paired floor), J-55 (the local-lane CodeAct guard the scaffold arm respects).

## Open questions / risks
- LongCoT task shape: confirm the HF dataset's tasks are receipt/exact-match checkable end-to-end (not requiring an LLM-judge); if a subset needs subjective scoring, EXCLUDE it from the gate (deterministic-only, TD-3) and note the covered scope (GD-5 honest residue).
- The LB-2 scaffold arm depends on J-108's primitive; if J-108 slips, the gate ships flat-only and SAYS so (a registered stub, never a silent pass). Do NOT let J-107 claim a lift number before J-108's primitive is real.
- Local-lane run is RunPod-gated; the fixture/cloud arm measures the SHAPE now, the per-(model,lane) knee for the actual Qwen-7B lane lands when the GPU box is up (W-10). Flag the sidecar boot before any live local run (hardware-heavy house rule).

## Next
J-107 makes the RLM corpus's one clear build real: an ELAI-measured LongCoT arm that both stands as a differentiator instrument (the engine grounds the model — measured) and GATES J-108 (the constrained recursive primitive ships local-lane-default ONLY if this arm shows a real lift with no J-55 collapse). It pairs with K-15 (reasoning-budget) and J-34 (context-fill) as the third calibration axis (reason-budget), and feeds J-84/J-85's overthinking ceiling its knee.

End of J-107.
