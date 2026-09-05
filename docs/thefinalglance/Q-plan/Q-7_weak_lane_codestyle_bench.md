# Q-7: weak-lane code-style bench — measure whether a code-style policy raises weak-model task success, promote to the L gate only on a measured win

STATUS: PLANNED (WAVE 2; NEW (bench-first); the objective is weak-lane TASK SUCCESS as the measured target of a code-style policy)

- Track: ENGINEERING-TASTE | lever: structure-code-so-weaker-models-still-succeed | source: bellard-taste V6 (Carmack Jun-2026, verified 2026-07-10) | grounding: MODERATE (the mechanism is grounded; the effect size is a HYPOTHESIS, bench-first) | nature: NEW | effort: M | WAVE 2
- PROOF_SPINE: feeds PS-4 (every point of weak-lane success rate moves $/verified-task directly — a style policy that lifts weak-lane pass-rate lets more tasks resolve on the cheap lane).

- Premise (bellard-taste V6): Carmack (x.com/ID_AA_Carmack, Jun 2026, WebSearch-confirmed 2026-07-10): "LLMs could optimize coding style by exploring ways of structuring code so weaker and weaker models can still successfully perform tasks in a codebase." ELAI builds adjacent machinery (C-plan code-completeness with per-file line/function budgets + skeleton-first + mini-critic; J model-strength router; L output-quality 4-seam gate) but NO unit treats weak-lane TASK SUCCESS as the measured OBJECTIVE of a code-STYLE policy. The effect size is a HYPOTHESIS (a tweet, not a result; no paper here claims the delta) — so this is BENCH-FIRST with a cheap falsifier, and a style rule is promoted into the L gate ONLY on a measured pass-rate win.

## Steal/build
Run the SAME task shard (the D-6/E-54 eval corpus) on the shipped WEAK local lane against controlled STYLE VARIANTS of the same module — file-size cap on/off, explicit types vs inferred, colocated doc-comments vs bare, flat vs deep nesting — and measure weak-lane pass-rate per variant. A style rule is promoted into the L output-quality gate ONLY on a measured pass-rate win beyond noise (`WeakLaneStyleBench{variant, shard_id, pass_rate, n}`). The cheapest falsifier ships FIRST: 2 style variants x 1 task shard on the shipped local lane; if the pass-rate delta is inside noise, the idea is dropped for free and the negative result is recorded (a measured null is a valid, valuable outcome).
EXCLUDED: promoting a style rule into the L gate WITHOUT a measured win (the whole discipline — Carmack's conjecture is not evidence; L-gate rules are earned by pass-rate, GD-4/R4); asserting a style→success delta as a claim (KD-1 — the bench MEASURES it, the plan never asserts it); a bench that grades on the STRONG lane (the objective is WEAK-lane success — a strong-lane bench measures nothing about the hypothesis).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- D-6/E-54 eval corpus (the task shards) — the fixed input the variants are measured against.
- the shipped weak LOCAL lane (Qwen2.5-Coder local sidecar) — the model under test; J router picks it.
- C-plan code-completeness (per-file line/function budgets + skeleton-first + mini-critic) — the adjacent style machinery a winning variant folds into.
- L output-quality 4-seam gate — the promotion target; a style rule lands here only on a measured win.
- K-2 routing-economics bench harness — the sibling bench; Q-7 reuses its shard-run + measured-row idiom.
- SSB measured-number row idiom — the `WeakLaneStyleBench` pattern.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the bench runner (`elai bench weak-lane-style`) over the D-6/E-54 shard with the 2-variant cheapest falsifier (file-size-cap on/off) on the shipped local lane + `WeakLaneStyleBench` journaling + the measured pass-rate per variant.
- DEFER (Tier-2): the additional variants (explicit-types, colocated-docs, nesting) + the L-gate promotion of any WINNING rule (only after a variant shows a beyond-noise win; a null result promotes nothing and closes the unit honestly).

## Integration
- crates/cli: the `elai bench weak-lane-style` runner reusing the K-2 shard harness; the per-variant pass-rate recompute + `WeakLaneStyleBench` emit.
- crates/orchestrator (only IF a variant wins): the L-gate style-rule addition — deferred, gated on the measured win.
- frontend_pair: N/A with justification — the bench result is a CLI + SSB artifact; a dashboard tile is a later job. Justified.

## MOVES (wargame)
M1 the bench runner: run the D-6/E-54 shard on the shipped weak local lane under 2 style variants (file-size cap on/off), same shard, same seed; journal `WeakLaneStyleBench{variant, shard_id, pass_rate, n}` per variant.
EXPECTED_OBSERVATION: the runner produces a pass-rate per variant recomputable from the journal; both variants run the SAME shard on the SAME weak lane (only the style differs).
ON_FAIL: the variants differ in shard or lane, not just style -> the comparison is confounded -> hold shard+lane+seed fixed, vary only the style knob; assert identical shard_id across variants.
M2 compute the pass-rate delta with an n-aware noise band; report WIN / NULL / LOSS against the band; a NULL closes the falsifier honestly (idea dropped for free).
EXPECTED_OBSERVATION: the report classifies the delta vs the noise band; a delta inside the band is reported NULL, not spun as a win.
ON_FAIL: a within-noise delta is reported as a win -> KD-1 overclaim -> require the delta to clear the n-aware band before WIN; the claims lint blocks an asserted style→success delta with no band.
M3 (Tier-2, gated) promote a WINNING style rule into the L output-quality gate; a NULL/LOSS promotes NOTHING and records the null result.
EXPECTED_OBSERVATION: only a beyond-noise WIN adds an L-gate rule; the L-gate change cites the `WeakLaneStyleBench` win row; a null run adds no rule and journals the null.
ON_FAIL: a rule is promoted on a hunch / a null -> unearned L-gate rule -> gate promotion on a recorded WIN row only (R4/GD-4).

## FORK
FORK: if the D-6/E-54 shard cannot be run cleanly on the shipped local lane in the bench harness (harness assumes the cloud lane) -> ship the falsifier on the SMALLEST runnable local shard + record the coverage gap; do NOT substitute the strong lane (the objective is weak-lane success — a strong-lane run measures the wrong thing and the unit ABORTs before that).

## RECON_NEEDED
RECON_NEEDED-1: the D-6/E-54 shard runner API + whether it targets the local lane -> settle via `grep -rn "eval.corpus\|shard\|D6\|E54\|bench" .elai_cc/crates/cli/src .elai_cc/crates/orchestrator/src`.
RECON_NEEDED-2: the C-plan file-size-cap style knob (how to toggle it as a variant) -> settle via `grep -rn "file.size\|line.budget\|completeness\|skeleton" .elai_cc/crates`.
RECON_NEEDED-3: the L output-quality gate rule-addition seam (for the deferred promotion) -> settle via `grep -rn "output.quality\|L.gate\|quality.seam" .elai_cc/crates` (Tier-2 only).

## ABORT
ABORT: if the only feasible bench runs on the STRONG lane (the weak local lane is unavailable in the harness) -> STOP + handoff. The entire hypothesis is about WEAK-lane success; a strong-lane bench cannot measure it and would produce a meaningless number that must never be reported as evidence for the style policy.

## WIRING_CONTRACT
produces_symbols:
  - `elai bench weak-lane-style` runner (2-variant falsifier on the shipped local lane)
  - `WeakLaneStyleBench` SSB row + the n-aware WIN/NULL/LOSS classifier (baseline+1; re-read at edit W-9)
  - (Tier-2, gated on a WIN) the L-gate style-rule addition
consumes_symbols:
  - D-6/E-54 eval corpus (the shard); the shipped weak local lane (Qwen local sidecar); K-2 shard-run harness; C-plan code-completeness style knobs; L output-quality gate (promotion target); SSB measured-number idiom
reachable_from:
  - `elai bench weak-lane-style` runs the shard on the weak lane [PRODUCTION bench CLI]; (Tier-2) the L gate reads a promoted rule [PRODUCTION output-quality gate]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO L-gate rule promotion without a recorded beyond-noise WIN row (GD-4/R4)
  - NO asserted style→success delta as a claim (bench measures; KD-1)
  - NO strong-lane substitution for the weak lane (measures the wrong objective)
  - NO within-noise delta reported as a win (n-aware band required)
observable_effect:
  trigger: "elai bench weak-lane-style over the D-6/E-54 shard with file-size-cap on/off on the local lane"
  expect_ssb: "WeakLaneStyleBench{variant, shard_id, pass_rate, n} per variant, same shard_id, recomputable pass-rates"
  expect_artifact: "a WIN/NULL/LOSS classification vs an n-aware noise band; NEGATIVE: a within-noise delta reported as a win, or an L-gate rule promoted with no WIN row, must FAIL"

PRODUCTION_HOOK: producer the bench runner + `WeakLaneStyleBench` emit -> consumer `elai bench weak-lane-style` (the bench CLI), wired same-changeset; (Tier-2) producer a winning style rule -> consumer the L output-quality gate; the measured row is the off-agent artifact.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-7_weak_lane_codestyle_bench.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
`python .claude/tools/claims_policy_lint.py --files <the bench report copy> --json`   # no asserted style-delta without the band
pass = `WeakLaneStyleBench` rows landed off-agent from a real local-lane shard run under both variants with identical shard_id, and the WIN/NULL/LOSS classification cleared the n-aware band (a NULL result is a valid pass — the falsifier ran honestly).

INTEGRATION_SMOKE: an independent agent runs the full 2-variant falsifier on the REAL shipped local lane over a real D-6/E-54 shard, asserts the pass-rates recompute from the journal and the delta is classified against the noise band — a real weak-lane run, not a mocked pass-rate (R10).

## REDTEAM
REDTEAM: attack that FAILED — "manufacture a style win by running the 'better' variant on the strong lane": FAILS because the ABORT + stub_ban pin the bench to the SHIPPED weak lane and assert identical shard_id+lane across variants; a lane swap is refused. Attack that SUCCEEDED and its patch — "promote a favorite style rule off a within-noise flutter": patched by M2's n-aware noise band + the claims lint, so only a beyond-band WIN row can promote an L-gate rule; a null promotes nothing and is recorded as a null.

## Grounding
- bellard-taste-elai-2026-07-10.md V6 + the Carmack tweet (x.com/ID_AA_Carmack/status/2065138674950414428, WebSearch-confirmed 2026-07-10) — mechanism grounded, effect size a hypothesis (hence bench-first + the cheapest falsifier).
- D-6/E-54 eval corpus, the shipped weak local lane, C-plan code-completeness, L output-quality gate, K-2 harness — confirmed at authoring, executor re-greps (W-7).

## COMPOSES-WITH
D-6/E-54 eval corpus, the weak local lane, K-2 shard harness, C-plan code-completeness style knobs, L output-quality gate (promotion target), SSB measured-number rows. Re-implements NONE (GD-1). Feeds PS-4 alongside Q-2/Q-3; a measured WIN is the only path to an L-gate style rule.

End of Q-7.
