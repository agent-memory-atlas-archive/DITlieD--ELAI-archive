# J-85: GRC-v3 deterministic overthinking guard — a per-call reasoning ceiling + 3-signal early-stop that catches the chain drifting OFF a correct answer (distinct from anti-spiral)

STATUS: PLANNED (WAVE GRC-2; M; BUILD — the ONE genuinely-new component of the GRC-v3 wave; depends J-83; measured by K-15)

Objective: the reasoning run's C5 ("more reasoning can HURT") is empirically proven by the overthinking paper (2604.10739, current, DeepSeek-R1-32B + s1-32B): accuracy is NON-MONOTONIC in reasoning-token budget — it rises, PEAKS, then FALLS, with a "negative flip" where extending the chain makes the model ABANDON a previously-correct answer. Three deterministic-to-compute signals predict it: answer OSCILLATION (r=0.78, the lead), hesitation markers "wait/but/actually" (r=0.71), declining confidence trajectory (r=0.63); flip_ratio>1.0 (negative flips exceed positive) is the regime. J-85 ships the deterministic guard: a per-call reasoning CEILING + an early-stop MONITOR on those three signals. It is DISTINCT from anti-spiral (GA-3): anti-spiral (phase-3C blake3 3-identical-hash stop, anti_spiral_extended.rs) catches REPETITION (same hypothesis/file 3×); the overthinking guard catches the chain DRIFTING off a correct answer — two different "stop" detectors, both deterministic (TD-3, string-scan + intermediate-conclusion tracking, NO LLM judge), running side by side. It composes the J-83 commit-and-freeze: once a candidate's external receipt PASSES, that entry is FROZEN — the guard stops the generator from oscillating away from it (the IMPLICATION-31 triple-payoff, the overthinking-defense leg). HARD BOUNDARY: the guard acts on the LEAD signal (oscillation), accepting some false-positive early stops (76.3% precision in the paper, R-3), tuned on ELAI's OWN data (K-15), NEVER the paper's 6K/7K knee (the paper ITSELF forbids transferring it — the local Qwen-7B lane overthinks EARLIEST and LOWEST, R-4, ID-5); the guard is deterministic-observable, never an LLM deciding when to stop (TD-3); it does NOT duplicate anti-spiral (a separate detector, composed); after ~3 failed receipted retries the right move is a MODALITY CHANGE (retrieve / escalate I-7 / abstain I-8), never more reasoning (IMPLICATION 23 — the top-of-ladder rung J-84/J-86 own).

Estimated effort: ~1 session (~M). The 3-signal monitor + the per-call ceiling + the J-83 freeze composition + the K-15 measurement seam. Pre-requisites (re-grep, W-7): J-83 (commit-and-freeze, the entries it protects); the GRC Verify step (crates/reasoning, phase-2A/2D) where the monitor sits; anti_spiral_extended.rs (the DISTINCT detector it sits beside — read it to NOT duplicate); I-26 BudgetController (the per-call ceiling); J-84 (the classifier that sets the budget the ceiling caps).
Parallel-safe with: J-84 (disjoint — classifier sets the budget UP FRONT, the guard stops DURING), K-15 (the bench that finds the knee). Depends J-83.
Source: reasoning-engine-grc-v3-2026-06-25.md §6 + IMPLICATION 28/29/30/31 + ROUND-2 paper 2604.10739; _grounding/J-GRC-V3-GROUNDING-MAP.md GA-3 + R-3/R-4 + DC-3. Doctrine: J-0_INDEX.md (W/GD/TD/ID inherited; ADDS no delta family — TD-3 deterministic-not-LLM, ID-5 measure-your-own already cover it).

## Context
Why this matters: this is the deterministic mechanism for the user's "without spiraling" AND "reasoning can hurt". A run can keep generating past its useful peak and oscillate away from a correct intermediate answer — invisible to anti-spiral (which only sees repetition) and to the up-front budget (J-84, which sizes but does not watch). The guard watches the chain DURING the call: rising oscillation in the running answer + hesitation-marker density + a declining-confidence trajectory → flip_ratio>1.0 → STOP and commit the last receipt-confirmed state. The knee is MODEL-SPECIFIC and LOWER for small models (the paper's R1-32B ~7K / s1-32B ~5K, "do NOT apply to smaller models"), so ELAI MEASURES its own per-(model,task-type) reasoning-budget→verified-outcome peak via K-15's `elai bench reasoning-budget` (a J-34 sibling) and the guard's ceiling tracks THAT, never the paper's number. Mechanism: a deterministic monitor over the running generation (oscillation = intermediate-answer changes; hesitation = a frozen marker word-list scan; confidence trajectory = the H-39 bit over steps) + a hard per-call token ceiling from I-26.

THE GUARD (what J-85 ships):
  OG-1 PER-CALL CEILING: a hard reasoning-token ceiling per call, set from the J-84 tier and bounded by the K-15-measured per-(model,task-type) knee (NOT the paper's number, ID-5). Hitting the ceiling = stop+commit-last-confirmed.
  OG-2 3-SIGNAL EARLY-STOP: oscillation (r=0.78, lead — count intermediate-answer flips), hesitation markers (r=0.71 — a FROZEN word-list scan, gate-manifest R8), declining confidence (r=0.63 — the H-39 bit trajectory); flip_ratio>1.0 = STOP. Deterministic (TD-3, no LLM).
  OG-3 DISTINCT-FROM-ANTI-SPIRAL: a SEPARATE detector beside anti_spiral_extended.rs — anti-spiral=repetition, overthinking=drift-off-correct; both run, neither subsumes the other (GA-3).
  OG-4 FREEZE COMPOSITION (the IMPLICATION-31 leg): on a receipt-confirmed J-83 commit, the guard FREEZES the entry so the generator cannot oscillate it away — commit IS the overthinking defense.
  OG-5 MODALITY-CHANGE RUNG (IMPLICATION 23): after ~3 failed receipted retries, the guard signals MODALITY CHANGE (retrieve / I-7 escalate / I-8 abstain), never "reason more" — the top-of-ladder rung J-84/J-86 own (the guard signals, they act).

## Existing state (W-7 anchors; GS-*/GA-* in the grounding map)
- ANTI-SPIRAL PRIMITIVES [PARTIAL] anti_spiral_extended.rs + phase-3C blake3 3-identical-hash stop + 5 personas + GoalDriftCanary — the DISTINCT repetition detector (OG-3; read to not duplicate).
- GRC STATE MACHINE [WIRED] GrcState Verify step (crates/reasoning) — where the monitor sits (OG-2).
- I-26 DYNAMIC THINKING BUDGETS [DARK] BudgetController + effort field — the OG-1 per-call ceiling host.
- H-39 abstention_score.rs [WIRED] AbstentionScored (218) — the OG-2 confidence-trajectory signal (GS-4).
- J-83 BeliefState carrier (this wave) — the commit-and-freeze the guard protects (OG-4).
- I-7 i7_escalation_cascade + I-8 ALETHEIA voluntary abort [PLANNED] — the OG-5 modality-change targets.
- (overthinking guard itself: ABSENT — grep "oscillation" hit only anti_spiral_extended.rs + telemetry, GA-3, the one net-new component.)

## Reads (mandatory before coding)
1. reasoning-engine-grc-v3-2026-06-25.md §6 + IMPLICATION 28 (the guard, distinct from anti-spiral) / 29 (model-specific lower knee, the J-34 sibling) / 30 (U_lambda) / 31 (commit = overthinking defense); ROUND-2 paper 2604.10739 block.
2. _grounding/J-GRC-V3-GROUNDING-MAP.md (GA-3, R-3/R-4, DC-3) + the K-15 plan (the measurement seam).
3. anti_spiral_extended.rs IN FULL (the distinct detector — do NOT duplicate it) + the GrcState Verify step + I-26 BudgetController + abstention_score.rs — re-grep live (W-7).
4. J-83 (the commit-and-freeze the guard composes) + I-7/I-8 (the modality-change rung targets).
5. CLAUDE.md: TD-3 (deterministic, no LLM stop), ID-5 (measure-your-own knee, never the paper's), anti-spiral skill (the 3-hypothesis-then-change rule OG-5 extends), gate-manifest R8 (the hesitation word-list is frozen).

## Tasks (WAVE GRC-2 — the overthinking guard)
1. The 3-signal monitor (~80 LOC): oscillation counter (intermediate-answer flips), hesitation-marker scan (a FROZEN word-list, gate-manifest R8), confidence-trajectory (the H-39 bit over steps); flip_ratio>1.0 = STOP. Deterministic (TD-3). Sits at the GRC Verify step.
2. The per-call ceiling (~30 LOC): a hard reasoning-token cap from the J-84 tier, bounded by the K-15-measured knee (a per-(model,task-type) value, NOT the paper's; read from the calibration artifact, ID-5).
3. Distinct-detector wiring (~20 LOC): run beside anti_spiral_extended.rs, NOT merged; both can fire; record which fired (OG-3).
4. Freeze composition (~20 LOC): on a J-83 receipt-confirmed commit, freeze the entry; a post-commit generator rewrite is dropped (OG-4, the overthinking-defense leg).
5. Modality-change signal (~15 LOC): after ~3 failed receipted retries, signal MODALITY_CHANGE to J-84/J-86 (retrieve/escalate/abstain), never "reason more" (OG-5, IMPLICATION 23).
6. workflow/reasoning/J-85-decisions.md (the guard design + the distinct-from-anti-spiral rationale + the measure-your-own-knee binding).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/overthinking_guard.rs (new) — the 3-signal monitor + per-call ceiling + modality-change signal (the one net-new mechanism)
  - the frozen hesitation-marker word-list (a fixtures file under the gate-manifest R8 frozen set)
consumes_symbols:
  - anti_spiral_extended.rs (run BESIDE it; do NOT duplicate or merge; GA-3)
  - I-26 BudgetController (the per-call ceiling host) + the J-84 tier (the budget it caps)
  - abstention_score.rs (the OG-2 confidence-trajectory signal; H-39)
  - J-83 BeliefState commit-and-freeze (the OG-4 entries it protects)
  - the K-15-measured per-(model,task-type) knee artifact (the OG-1 ceiling value; ID-5)
  - I-7 / I-8 (the OG-5 modality-change targets)
reachable_from:
  - the GRC Verify step / task_runner reasoning loop → the monitor runs per call [PRODUCTION reasoning loop]
  - the J-83 commit path → the freeze [PRODUCTION commit]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM deciding when to stop (deterministic 3-signal monitor + hard ceiling; TD-3)
  - NO paper-knee hard-coded (the ceiling is the K-15-measured per-(model,task-type) value; ID-5 — the paper forbids transferring it)
  - NO merge with anti-spiral (a distinct detector; GA-3); NO .unwrap on prod; the hesitation word-list is FROZEN (R8), not LLM-generated
frontend_pair:
  the chosen-stop-reason (ceiling / oscillation / hesitation / confidence) surfaces on the existing GRC/reasoning telemetry panel + the I-27 ConvergencePanel (the budget-vs-outcome curve) — decide the tile at edit (W-4); justified N/A if it rides the existing reasoning panel.
observable_effect:
  trigger: "drive the real reasoning entrypoint on a task tuned to overthink past its peak (integration smoke, R10): assert the guard STOPS at/near the measured knee and commits the last receipt-confirmed state, NOT the drifted one"
  expect_ssb: "+OverthinkingGuardFired (Pattern-2; Kind: CeilingHit / OscillationStopped / HesitationStopped / ConfidenceDeclined) OR reuse ReasoningBudgetScaled (+Kind) — net 0..+1 from LIVE 268/577, decide at edit (W-9)"
  expect_artifact: "a run that would oscillate off a correct answer is STOPPED at the lead signal and the receipt-confirmed state is committed (the win); the NEGATIVE arm: anti-spiral and the overthinking guard are DISTINCT (a repetition-only case fires anti-spiral but NOT the overthinking guard, and vice-versa); an LLM-proposed stop is NEVER the gate"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-85_grcv3_overthinking_guard.md --run-trigger
rg "oscillation|flip_ratio|hesitation|confidence.*traject|per.call.*ceiling" .elai_cc/crates/orchestrator/src/overthinking_guard.rs   # the 3 signals + ceiling present
rg -c "llm.*stop|model.*decides.*stop|ask.*model.*continue" .elai_cc/crates/orchestrator/src/overthinking_guard.rs   # MUST be 0 (TD-3)
rg -c "7000|7K|5000|paper.*knee|R1.32B.*threshold" .elai_cc/crates/orchestrator/src/overthinking_guard.rs            # MUST be 0 (no transferred paper knee; ID-5)
rg "anti_spiral" .elai_cc/crates/orchestrator/src/overthinking_guard.rs                                              # references it as DISTINCT, does not merge
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/overthinking_guard.rs
test -f workflow/reasoning/J-85-decisions.md
```

## ACCEPTANCE
- [ ] OG-1: a hard per-call reasoning ceiling from the K-15-measured knee (per model+task-type), never the paper's 6K/7K (ID-5).
- [ ] OG-2: the 3-signal monitor (oscillation lead / hesitation markers / declining confidence), flip_ratio>1.0 = stop, deterministic (TD-3).
- [ ] OG-3: distinct from anti-spiral (a separate detector beside it; both can fire; GA-3).
- [ ] OG-4: a J-83 receipt-confirmed commit freezes the entry against generator oscillation (the IMPLICATION-31 defense leg).
- [ ] OG-5: ~3 failed receipted retries → MODALITY_CHANGE signal (retrieve/escalate/abstain), never "reason more" (IMPLICATION 23).
- [ ] cargo check GREEN; auditor PASS -p orchestrator; production_caller 0-DARK; the hesitation word-list is frozen (R8).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the overthink-past-peak integration smoke run-trigger (the guard stops + commits the confirmed state), not file-exists.
- W-2: default-ON behind a *_DISABLE kill-switch with an allowlist row (it is a safety guard, not an experiment).
- W-3: a net-new deterministic component (the one true build in the wave), composing J-83/I-26/anti-spiral, not a summary-port.
- W-4: rides the GRC/I-27 panels (decide the tile at edit) or justified N/A. W-5/W-10: in-process at reasoning time. W-6: existing reasoning panel or justified N/A.
- W-7: anti_spiral_extended.rs / GrcState Verify / I-26 / abstention_score.rs re-grepped at edit.
- W-9: +OverthinkingGuardFired (Pattern-2) OR reuse ReasoningBudgetScaled; net 0..+1 from LIVE 268/577 derived; 9-tuple lockstep if added.
- TD-3: deterministic monitor, no LLM stop. ID-5: the knee is ELAI-measured (K-15), never the paper's. GD-2: the stop decision is recomputable from the journal.

## Tests to add
- crates/contracts/tests/j85_overthinking_guard_contract.rs (~70 LOC): (a) flip_ratio>1.0 (rising oscillation) STOPS the run; (b) the per-call ceiling stops at the configured (measured) knee and commits the last confirmed state; (c) the guard is DISTINCT from anti-spiral (a repetition case fires anti-spiral only; a drift case fires the guard only); (d) a J-83 committed entry is frozen against a post-commit rewrite (OG-4); (e) 3 failed receipted retries emit MODALITY_CHANGE not more-reasoning (OG-5); (f) an LLM "keep going / stop now" signal is ignored (TD-3); (g) the hesitation word-list is the frozen fixture (R8), not regenerated.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/overthinking_guard.rs (~160 LOC)
- CREATE: a frozen hesitation-marker fixture (under the gate-manifest R8 frozen set)
- MODIFY: the GRC Verify-step / task_runner reasoning loop (~25 LOC: run the monitor + the ceiling beside anti-spiral)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs + action_type_lock.rs + .golden IF +OverthinkingGuardFired (9-tuple lockstep; decide at edit)
- CREATE: crates/contracts/tests/j85_overthinking_guard_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/J-85-decisions.md (~25 LOC)

## Citations / Grounding
- reasoning-engine-grc-v3-2026-06-25.md §6; IMPLICATION 28 (the 3-signal guard, distinct from anti-spiral), 29 (model-specific LOWER knee for small models = the J-34/K-15 sibling bench), 30 (U_lambda = the cost/accuracy balance), 31 (commit = overthinking defense, the OG-4 leg).
- arXiv [EXTERNAL, grounding for WHY not an ELAI number]: overthinking 2604.10739 (non-monotonic accuracy / negative flip; oscillation r=0.78 / hesitation r=0.71 / declining-confidence r=0.63; 76.3% precision @ 80% recall; "do NOT apply the R1-32B threshold to smaller models" — the load-bearing reason the knee is ELAI-measured).
- ELAI ground truth (GA-3/GS-4): anti_spiral_extended.rs (the DISTINCT detector), GrcState Verify step, I-26 BudgetController, abstention_score.rs — re-grepped (W-7).
- CLAUDE.md: TD-3 (deterministic, no LLM stop), ID-5 (measure-your-own knee), anti-spiral 3-hypothesis-then-change rule (OG-5), gate-manifest R8 (frozen word-list).

## Open questions / risks
- R-3: the flip-ratio is a LAGGING indicator; oscillation is the LEAD. The guard acts on the lead and accepts false-positive early stops (76.3% precision) — tune recall-vs-precision on ELAI's own data (K-15), OBSERVE-ONLY first if a false-stop rate is a concern.
- R-4: the local Qwen-7B lane overthinks EARLIEST/LOWEST — its ceiling is the lowest and its self-eval is noise; the guard matters MOST there. Do NOT share one knee across lanes (per-(model,task-type), ID-5).
- "intermediate answer" extraction for oscillation counting must be deterministic and cheap (a structured-output field or a regex over the running answer region, NOT an LLM re-read). Decide the extraction at authoring; if it needs the model, it is a material fork (TD-3 violation), STOP + handoff.

## Next
J-85 is the GRC-v3 wave's one genuinely-new mechanism: a deterministic guard that stops a run from reasoning past its useful peak and protects receipt-confirmed state from the generator's own second-guessing. It pairs with J-84 (which sets the budget the ceiling caps) and J-83 (whose commit-and-freeze it enforces), and is calibrated + done-gated by K-15's `elai bench reasoning-budget`. K-16's doctrine binds the measure-your-own-knee + deterministic-not-LLM-stop rules.

End of J-85.
