# N-16: governance closure — evaluator-refutation verdict-collapse + a lambda exogenous-grounding floor + adjudicator-ritual metrics

- Track: EXTERNAL-STEAL (DeepReason 2026-07-08) | lever: is-our-own-adjudication-still-real | verdicts S4+S6+S7 | grounding: STRONG (steal-deepreason-2026-07-08.md; ELAI anchors PLAN-asserted, re-grep W-7) | nature: EXTENSION(N-8) + BUILD | effort: M | WAVE 9

PROOF_SPINE (PSR-2): feeds PS-2 (composed gate-tax vs locked ceiling, K-4) TRANSITIVELY and DECLARED (not vibes): S7 measures the fraction of shipped units whose declared VERIFY legs never actually ran, and S4 keeps every gate verdict from standing on a since-refuted evaluator — a locked gate-tax ceiling is only a credible number if its gates fire and their verdicts are not stale. N-16 is the honesty check ON the number PS-2 sells.

- Premise: DeepReason names two capture surfaces its spec §11 treats as first-class (steal §4.2 S4/S6/S7): GENERATOR capture is covered by ELAI's anti-spiral + L-23 orbit detector; ADJUDICATOR capture — criticism ritualizes while statuses stay formally correct, and verdicts keep standing after the standard that issued them is refuted — is the residue N-8's five liveness/coverage views do NOT catch. N-16 adds the three missing views over the SAME "what part of our own governance is decorative" question N-8 asks, so it is an N-8 EXTENSION (a sixth/seventh/eighth view over one join), not a new governance framework (ND-1 unify-not-fork). All three are ADVISORY alarms that force retrieval/re-audit/escalation, never PASS signals (TD-3, HG-4 posture); measured through the N-1 spine (ND-4), ELAI-measured thresholds only (ID-5).

## Steal/build
THREE views bolted onto the N-8 governance-coverage substrate + the N-1 spine:
- S4 EVALUATOR-REFUTATION => VERDICT-COLLAPSE (case-law closure): every SSB verdict row carries an evaluator-id + version-hash (cheap, additive field); a registered evaluator-refutation event — a failed H-27 mutant corpus, an I-24/N-15 bias detection, a frozen-tool bugfix — triggers an F-5-style staleness cascade over exactly the verdicts that evaluator issued: QUARANTINE them into an advisory re-audit queue, NEVER delete (append-only preserved). Closes the gap DeepReason's rubric-closure names: today a miscalibrated checker/judge/rule found later leaves its past verdicts silently standing; invalidation is manual. This is the one valuable consequence of his Dung reinstatement, taken WITHOUT the argument-graph adjudicator (REJECTED, steal §4.3).
- S6 LAMBDA EXOGENOUS-GROUNDING FLOOR: a windowed fraction of accepted claims/verdicts whose support bottoms out in an EXOGENOUS anchor (a program check, an off-agent artifact, retrieval evidence, a human ruling) vs a model-judged rubric. ELAI enforces off-agent evidence PER-GATE (R4) but has NO aggregate closed-loop alarm — a long session can drift toward rubric/advisory-heavy operation with every individual gate still green. lambda below a floor forces retrieval/research/escalation (a BRAKE), and is NEVER a pass signal (his honest framing survives: it detects STALLED/closed-loop dynamics, not wrong-but-stable consensus). One SSB-derived tile + one threshold; journal-recomputable.
- S7 ADJUDICATOR-RITUAL METRICS: entropy over auditor finding-CLASSES per window (is criticism probing new surfaces or re-litigating the same nodes = rubber-stamp drift beyond the existing anti-rubber-stamp minimum) + the fraction of shipped units whose declared VERIFY legs never actually ran (criticism debt) + the rate at which the gates THEMSELVES get challenged/amended (his validity-node-attack-rate ~= 0 translation: nobody ever challenges a test). Feeds N-8's marketable governance number.
EXCLUDED: the Dung grounded-extension adjudicator + reinstatement as a general mechanism (REJECTED §4.3 — ELAI verdicts are execution-grounded; S4 takes only the closure consequence); any of the three ACTING as a gate that emits PASS (TD-3 advisory-only); claiming a governance number before it is MEASURED through N-1 (ND-4).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- N-8 governance-coverage substrate [PLANNED] — five views (B-17 gate-liveness / K-22 doctrine-enforcer / J-46 debt-rate / J-70 traceability / H-27 mutant-kill) over one join; N-16 adds S4/S6/S7 as further views on the SAME substrate.
- F-5 staleness cascade [WIRED-class] — the quarantine-not-delete machinery S4's verdict-collapse reuses (never re-implement).
- H-27 kill-your-own-mutant-corpus + I-24/N-15 bias detection — the evaluator-refutation EVENTS that trigger S4's cascade.
- SSB verdict rows (ssb.rs) — carry the new evaluator-id + version-hash field; R4 off-agent evidence is the exogenous-anchor signal S6 counts.
- .claude/gate-manifest.txt + gate_freeze_check.py (R8) — the frozen gate registry whose amend-rate S7 measures.
- N-1 measurement spine [PLANNED] — the sink the three numbers route through (ND-4); D-17 refusal SLA — the escalation S6's brake fires into.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1, in-process): the evaluator-id+version-hash field on verdict rows + the S4 quarantine cascade on a registered refutation event; the S6 lambda tile + floor + brake (advisory); the S7 finding-class entropy + VERIFY-leg-ran rate. All observe-only (GD-4) until their FP bound is measured on >=3 runs.
- DEFER (Tier-2): back-filling version-hashes onto historical verdict rows (the field stops the bleed forward; the back-fill is a rolling task); the gate-amend-rate leg of S7 (needs >=1 quarter of gate-manifest history).

## Integration
- crates/orchestrator (the verdict-row field + the F-5 cascade hook + the lambda/ritual derivations) + .claude/tools (the governance-coverage checker gains the three views); the N-1 spine sink.
- frontend_pair: the three numbers on the N-8 / G-5 / M-3 governance tile (the lambda gauge + the quarantine-queue count + the ritual-entropy band).

## WIRING_CONTRACT
produces_symbols:
  - the evaluator-id + version-hash field on SSB verdict rows + the S4 evaluator-refutation => quarantine cascade
  - the S6 lambda exogenous-grounding-ratio tile + floor + brake (advisory)
  - the S7 adjudicator-ritual view (finding-class entropy + VERIFY-leg-ran rate + gate-amend rate) on the N-8 substrate
consumes_symbols:
  - N-8 governance-coverage substrate (EXTEND — never re-author the five views); F-5 staleness/quarantine (reuse the cascade); H-27 + I-24/N-15 (the refutation events); gate-manifest + gate_freeze_check.py (R8); N-1 spine (the sink, ND-4); D-17 refusal SLA (the brake target); R4 off-agent evidence (the exogenous anchor S6 counts)
reachable_from:
  - a registered evaluator-refutation event (mutant-corpus fail / bias detection / gate bugfix) > the F-5 verdict-collapse sweep  [PRODUCTION governance seam]
  - the CI/authoring governance-coverage checker invocation > the S6/S7 derivations  [PRODUCTION coverage seam]
  - a #[cfg(test)]/contracts caller NEVER satisfies this (R7)
stub_ban:
  - NO Dung/argument-graph adjudicator (REJECTED §4.3); NO deletion of a quarantined verdict (append-only, F-5)
  - NO lambda/ritual number emitting PASS (TD-3 advisory-only, HG-4 posture); NO number claimed before it is measured through N-1 (ND-4)
  - NO re-implementation of N-8's five views, F-5, or the H-27/I-24 detectors (GD-1/ND-1 P0); NO .unwrap on prod; blake3 not sha2
frontend_pair: the lambda gauge + quarantine-queue count + ritual-entropy band on the N-8 / G-5 / M-3 governance tile
observable_effect:
  trigger: "register an evaluator-refutation event (a frozen-checker bugfix or an I-24 bias detection) then run the governance-coverage checker over the rulebook + verdict stream"
  expect_ssb: "+evaluator_closure_swept (Pattern-2; Kind: EvaluatorRefuted/VerdictQuarantined/Reinstated) + +grounding_ratio_measured (Pattern-2; Kind: AboveFloor/BelowFloorBraked); S7 prefer-reuse governance_coverage_measured (+Kind AdjudicatorRitual); baseline+2 from LIVE 267/576 as of 2026-07-04 (re-grep ssb.rs:1519 + action_type_lock.rs at edit, W-9)"
  expect_artifact: "the refuted evaluator's past verdicts land in the advisory re-audit queue (NOT deleted); lambda under the floor fires a retrieval/escalation brake; a shipped unit whose VERIFY leg never ran is counted as criticism debt. NEGATIVE: a quarantined verdict silently deleted FAILS (F-5 append-only); a lambda/ritual number used as a PASS gate FAILS (TD-3); a governance number asserted without an N-1 measurement FAILS (ND-4)"

## Grounding
- workflow/reasoning/steal-deepreason-2026-07-08.md S4 (evaluator-refutation => verdict-collapse, case-law closure), S6 (lambda exogenous-grounding floor + brake), S7 (adjudicator-capture metrics: attack-target entropy, criticism debt, validity-node-attack rate); DeepReason spec §11 (adjudicator capture) [EXTERNAL/DESIGN-RATIONALE, ID-5].
- ELAI ground truth (re-grepped at edit, W-7): N-8 substrate, F-5 cascade, H-27/I-24/N-15 detectors, gate-manifest + gate_freeze_check.py, N-1 spine, D-17, R4 off-agent doctrine, ssb.rs:1519.
- CLAUDE.md / wiring-enforcement: TD-3 (advisory never adjudicates), ID-5 (ELAI-measured), R4 (off-agent artifact), R8 (frozen gate manifest), GD-2 (journal-recomputable); ND-1 (unify-not-fork), ND-4 (measured-through-the-spine).

## COMPOSES-WITH
N-8 (the substrate EXTENDED with three views), F-5 (quarantine cascade), H-27 + I-24 + N-15 (refutation events), gate-manifest + gate_freeze_check.py, N-1 (measurement sink), D-17 (escalation brake), R4 (exogenous anchor). Re-implements NONE (GD-1/ND-1). Upgrades N-8's "is the gate real" from generator-side (does it fire / catch) to ALSO adjudicator-side (do its verdicts still stand, is criticism still probing new surfaces, is our grounding still exogenous) — the DeepReason adjudicator-capture surface ELAI's five views missed.

End of N-16.
