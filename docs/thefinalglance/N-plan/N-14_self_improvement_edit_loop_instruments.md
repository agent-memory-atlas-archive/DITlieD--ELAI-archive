# N-14: self-improvement edit-loop instruments — EditKind classifier + task-demand routing + two-stage promotion + improvement-timing, from EvoPolicyGym

STATUS: PLANNED (WAVE 3/4-adjacent; Tier-2/3; nature BUILD + ACTIVATION — feeds N-1 spine / N-3 promotion / N-5 intensity, all ADVISORY per TD-3)

Objective: ship the four small, deterministic, high-leverage instruments that arXiv:2607.02440 (EvoPolicyGym) hands ELAI for measuring and routing its own self-improvement edit loop, bound to the N substrates they feed: (1) a stripped-AST EditKind classifier {synthesis-edit vs parametric-edit} joined to the existing verdict stream; (2) synthesis-vs-tuning TASK-DEMAND as a routing/intensity axis; (3) a two-stage hidden SELECT-then-REPORT promotion rule; (4) improvement-event-timing fields on the convergence curve. All ADVISORY signals (TD-3) — none is a blocker. Hard boundary: the EditKind classifier is a deterministic diagnostic, NEVER a gate (the paper admits the AST-topology lens is conservative — two topologies can implement the same behavior); it enters as an ADVISORY axis only.

Estimated effort: ~1-2 sessions (~M-L). Tier-1 in-process (tree-sitter AST hashing + journal reads; no network). The four legs share one seam (the score-bearing attempt row) so they ship as one vertical slice with one done-gate.
Pre-requisites (re-greped at edit, W-7): the existing verdict/attempt SSB rows, anti-spiral C-10 StagnationPattern + the B-22b persona rotation, I-4 search receipts, I-22 progress shaping, J-10 insight backprop, I-27 ConvergenceCurveMeasured, H-45/H-47 self-improvement promotion, D-6 held-out store, L-21 weak-lane intensity, H-7 architect/editor split, J-84 effort classifier, N-1 spine, N-3 deflation, N-5 intensity resolver. tree-sitter already in crates/knowledge (PLAN-asserted; re-grep). All PLAN-asserted from ELAI-FEATURE-MAP + the source steal file.
Parallel-safe with: N-13, N-15.
Source: workflow/reasoning/four-source-steal-2026-07-05.md (Artifact 2 verdicts S2-1, S2-2, S2-3, S2-5). Paper: arXiv:2607.02440 "EvoPolicyGym: Evaluating Autonomous Policy Evolution in Interactive Environments" (WebSearch-verified 2026-07-05, cs.AI/cs.CL, 24pp). Doctrine: N-0_INDEX.md.

## Context
Why this matters: EvoPolicyGym's deterministic edit-type classifier is finer than anything in ELAI's stagnation stack. A SYNTHESIS edit introduces a previously-unseen policy-bundle AST TOPOLOGY after numeric constants are stripped; a PARAMETRIC edit changes the bundle while preserving the stripped topology. The paper's headline measured result: the capability gap between models opens on synthesis-dominant tasks — leaders convert synthesis edits at 41-48% hit rate, weak models churn structure at 3-10% while staying competitive on tuning (25-38%); "complex code is not necessarily a task-adapted mechanism." ELAI's anti-spiral (C-10) detects Spinning/Oscillation/NoDrift/DiminishingReturns off content hashes and score plateaus but CANNOT say "you are churning STRUCTURE without traction" vs "you are tuning constants inside the WRONG structure" — the paper's two named failure modes. The weak-model collapse (MiniMax/DeepSeek 0.19/0.03 normalized on synthesis; 0.00 on all three locked-door MiniGrid tasks) is direct evidence for ELAI's H-7 architect/editor split and L-21 weak-lane intensity: a 7B-class local model must never OWN structure discovery. Two-stage SELECT-then-REPORT (select on hidden validation, report on disjoint hidden held-out) prevents even the selection step from leaking into the reported number. Improvement-event-timing makes "budget-to-first-improvement" (the paper's Figure-3 best-so-far curve read) a per-config number at near-zero cost since I-27 already owns the curve.

## Existing state (B..M coverage — PLAN-asserted, executor re-greps per W-7)
- Stagnation: anti-spiral C-10 StagnationPattern (Spinning/Oscillation/NoDrift/DiminishingReturns off content hashes + score plateaus) + B-22b persona rotation (BLIND rotation today). No edit-topology signal.
- Search/progress: I-4 search receipts, I-22 progress shaping (deterministic-observable rewards), J-10 insight backprop (labels wins). None labels the edit KIND that produced a win.
- Curve: I-27 ConvergenceCurveMeasured (owns the best-so-far curve; the paper's Figure-3 shape). No per-improvement-event timing fields.
- Promotion: H-45 worst-case scoring + H-47 date-gate/template-regen/budget binding; D-6 held-out store with per-family tagging. Promotion gates on held-out EVALUATION but has no explicit SELECT-on-hidden-validation-then-REPORT-on-disjoint-held-out two-stage split.
- Routing/intensity: L-21 weak-lane intensity, H-7 architect/editor split, J-84 effort classifier. No synthesis-vs-tuning task-demand axis.
- N substrates this binds to: N-1 spine (the outcome-label fold), N-3 deflation, N-5 intensity resolver (the six-decider table).

## Reads (mandatory before coding)
- N-0_INDEX.md (ND-1..ND-5), N-SUMMARY_INDEX.md (N-1/N-3/N-5 done-gates the legs feed).
- workflow/reasoning/four-source-steal-2026-07-05.md Artifact 2 (S2-1/S2-2/S2-3/S2-5 + the ADVERSARIAL READ: n=1, AST-topology-conservative, memorization-confound).
- C-10 anti-spiral + B-22b persona (the consumer that becomes evidence-driven).
- I-27 (curve), H-45/H-47 (promotion), L-21/H-7/J-84 (routing) — the seams each leg wires.

## Tasks (WAVE 3/4-adjacent)
1. EditKind classifier (~90 LOC): blake3 of the changed file-set's AST topology with numeric constants + identifiers normalized (tree-sitter, already in crates/knowledge — do NOT add a parser dep). unchanged topology + changed bundle => ParametricEdit; new topology => SynthesisEdit; rollbacks/retests are candidate-management events, not edit kinds. Join {edit_kind, improved: bool} onto every score-bearing attempt row. DETERMINISTIC + byte-stable under ELAI_SEED. NEGATIVE space: a whitespace/rename-only diff must NOT flip topology (constant + identifier normalization test).
2. Wire EditKind as ADVISORY evidence (~40 LOC) into: anti-spiral C-10/B-22b persona choice (structure-churn-without-traction -> Architect/Simplifier; wrong-structure-tuning -> Contrarian/rethink, replacing blind rotation), I-4 receipts ("branch 2 won via synthesis edit"), I-22 progress shaping (decay faster on repeated no-hit synthesis churn), J-10 insight backprop (label which edit KIND produced the win). ADVISORY only (TD-3) — never a blocker.
3. Synthesis-vs-tuning TASK-DEMAND axis (~40 LOC): classify the upcoming step's edit intent (introduces new topology vs tunes existing) and feed it as an axis into N-5's intensity resolver + H-7 architect/editor split + L-21 intensity + J-84 effort: synthesis-demand -> architect lane + full verification stack; tuning-demand -> editor lane eligible + parameter-sweep-shaped retries. ELAI-MEASURED thresholds only (ID-5); grounded by the paper's Table 4/8 gap but never IMPORTING its constants.
4. Two-stage hidden SELECT-then-REPORT (~30 LOC on H-45/H-47): candidate SELECTION happens on a hidden validation slice (never agent-visible, budget-bound via H-19/N-3); the REPORTED fitness comes from a DISJOINT held-out slice; ties resolved deterministically. Re-arrangement of shipped pieces (D-6 already tags per-family) — closes the "selection leaks into the reported number" residue.
5. Improvement-event-timing fields on I-27 (~20 LOC): per improvement event, {budget-consumed-at-jump, jump-magnitude, plateau-length}. Makes budget-to-first-improvement a per-config number.
6. Emit the join via one SSB field on the existing attempt row (prefer-reuse) OR +edit_kind_classified (Pattern-2) if no attempt row carries it; route the labels through the N-1 fold (ND-4).
7. Decision doc: workflow/reasoning/N-14-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/edit_kind.rs::classify_edit_kind (stripped-AST topology hash -> Synthesis|Parametric)
  - crates/orchestrator/src/edit_kind.rs::classify_task_demand (upcoming-step synthesis-vs-tuning axis)
  - the two-stage SELECT-then-REPORT rule on the H-45/H-47 promotion path
  - improvement-event-timing fields on I-27 ConvergenceCurveMeasured
  - SSB: reuse the score-bearing attempt row's {edit_kind, improved} field where present, else +edit_kind_classified (Pattern-2; Kind: Synthesis / Parametric / CandidateMgmt)
consumes_symbols:
  - anti-spiral C-10 StagnationPattern + B-22b persona rotation (the ADVISORY consumer; do NOT re-implement the detector, feed it)
  - I-4 receipts / I-22 progress shaping / J-10 insight backprop (label consumers)
  - I-27 ConvergenceCurveMeasured (EXTEND with timing fields, do NOT re-author the curve)
  - H-45/H-47 promotion + D-6 held-out + H-19/N-3 budget (the two-stage rule composes these)
  - N-5 intensity resolver + H-7 architect/editor + L-21 + J-84 (task-demand axis consumers)
  - N-1 spine (the outcome-label sink, ND-4); tree-sitter in crates/knowledge
reachable_from:
  - the score-bearing attempt/verdict emission in the orchestrator agent loop [PRODUCTION anti-spiral + promotion path]
  - the routing decision seam in task_runner (task-demand axis) [PRODUCTION routing]
  - a #[cfg(test)]/contracts caller NEVER satisfies this (R7)
stub_ban:
  - NO EditKind as a BLOCKER (ADVISORY only, TD-3 — the paper's own conservative-lens caveat)
  - NO imported paper constants for the routing thresholds (ID-5; ELAI-measured only)
  - NO re-implementation of the C-10 detector / I-27 curve / the promotion machinery (GD-1)
  - NO topology flip on whitespace/rename-only diffs (constant + identifier normalization required)
  - NO .unwrap()/.expect() on prod paths; blake3 only for the topology hash
frontend_pair:
  an edit-kind chip on the existing anti-spiral / search-receipt drawer, OR N/A justified (internal advisory signal) if no surface consumes it yet.
observable_effect:
  trigger: "run a real agent-loop attempt that applies (a) a constant-only edit then (b) a restructuring edit, seeded under ELAI_SEED"
  expect_ssb: "the (a) attempt carries edit_kind=Parametric, the (b) attempt edit_kind=Synthesis; a repeated no-hit synthesis churn shifts the anti-spiral persona to Architect/Simplifier (not blind rotation)"
  expect_artifact: "the improvement-timing fields land on the I-27 curve; a SELECT-then-REPORT promotion reports fitness from the disjoint held-out slice; NEGATIVE arm — a whitespace/rename-only diff that flips topology to Synthesis FAILS the normalization test; a promotion that reports the SELECTION-slice number FAILS the two-stage gate"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/N-plan/N-14_self_improvement_edit_loop_instruments.md --run-trigger
rg "fn classify_edit_kind" crates/orchestrator/src/edit_kind.rs        # the classifier exists
rg -c "block|reject|deny" crates/orchestrator/src/edit_kind.rs         # MUST be 0 (advisory, never a gate)
rg "blake3" crates/orchestrator/src/edit_kind.rs                       # topology hash uses blake3
test -f workflow/reasoning/N-14-decisions.md
```

## ACCEPTANCE
- [ ] TD-3: EditKind + task-demand are ADVISORY; no gate consumes them as a blocker.
- [ ] ID-5: routing thresholds are ELAI-measured, no paper constant imported.
- [ ] Determinism: byte-stable classification under ELAI_SEED; whitespace/rename-only never flips topology.
- [ ] ND-4: labels route through the N-1 spine.
- [ ] SELECT-then-REPORT: reported fitness comes from a slice disjoint from the selection slice.
- [ ] W-2: default ON behind ELAI_EDIT_KIND_LEGACY (blind persona rotation survives one release as fallback).
- [ ] cargo check GREEN; auditor PASS on -p elai_cc_orchestrator + contracts. No new dep (tree-sitter already present).

## WIRING DONE-GATE (clause by clause)
- W-1 --run-trigger PASS. W-2 kill-switch, default ON. W-3 reachable from the real agent-loop + routing seams. W-4 frontend_pair chip or N/A justified. W-5 LIVE trigger. W-6 dashboard item or N/A justified. W-7 anchors re-greped. W-8 producers have in-plan consumers (the four wired seams). W-9 SSB via golden lock; SCHEMA baseline+0..1 from LIVE 267/576 as of 2026-07-04, re-read at edit. W-10 in-process Tier-1. ND-1: net-new concern (no existing edit-topology classifier), not a fork. ND-4: measured through the spine. ND-5: the routing axis is a thin parameter into N-5, not a new resolver.

## Tests to add
- crates/contracts/tests/n14_edit_kind_contract.rs (~90 LOC): (a) constant-only diff -> Parametric; (b) restructure -> Synthesis; (c) whitespace/rename-only -> topology UNCHANGED (normalization); (d) repeated no-hit synthesis churn -> anti-spiral persona shifts to Architect/Simplifier; (e) SELECT-then-REPORT reports the disjoint held-out number, not the selection number; (f) a malformed/unparseable diff degrades to CandidateMgmt, never a false Synthesis.

## Files touched
- CREATE: crates/orchestrator/src/edit_kind.rs (~140 LOC: classifier + task-demand)
- MODIFY: anti-spiral C-10 persona seam (~25 LOC: evidence-driven choice)
- MODIFY: I-27 curve (~20 LOC: timing fields), H-45/H-47 promotion (~30 LOC: two-stage split)
- MODIFY: N-5 intensity resolver + H-7/L-21/J-84 seams (~30 LOC: task-demand axis)
- MODIFY: crates/orchestrator/src/ssb.rs (~8 LOC: reuse attempt field or +edit_kind_classified, re-read baseline)
- CREATE: crates/contracts/tests/n14_edit_kind_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/N-14-decisions.md (~40 LOC)

## Citations / Grounding
- Paper: arXiv:2607.02440 EvoPolicyGym (WebSearch-verified 2026-07-05). The stripped-AST synthesis/parametric classifier + the measured weak-model synthesis-collapse (Table 4/8) are the grounded mechanism + demand feature. ADVERSARIAL caveats carried (steal file Artifact 2): n=1 per cell (no seeds/CIs — ELAI must not import the practice, only the protocol; ELAI runs its own I-38 floor); the AST-topology lens is admitted-conservative (ADVISORY only); memorization/recall confound on famous environments (irrelevant here — ELAI classifies ITS OWN edits, not policy discovery).
- ELAI ground truth: all code anchors PLAN-asserted from ELAI-FEATURE-MAP.md; executor re-greps per W-7.
- CLAUDE.md rules: TD-3 deterministic-advisory, ID-5 no-imported-constants, GD-1 compose-not-restate, blake3-only.

## Open questions / risks
- Which attempt/verdict SSB row already carries a per-attempt payload the {edit_kind, improved} join can ride (decide at authoring; prefer reuse over +edit_kind_classified).
- Whether task-demand classification is cheap enough to run per-step or should be cached per plan node (decide at authoring; it reads the plan's edit intent, not the model).
- The AST-topology normalization set (constants + identifiers at minimum; decide whether string literals normalize too — a source of false Parametric if a mechanism is encoded in a literal).

## Next
Closes the "churning structure vs tuning the wrong structure" blind spot in the stagnation stack and protects the local-7B lane from owning structure discovery (H-7/L-21). Pairs with N-1 (label sink), N-3 (deflation on the two-stage promotion), N-5 (the task-demand axis). Sequences behind the EvoPolicyGym bench candidate (S2-4, recorded as a K-2 bench note) which measures ELAI-harness-with-the-same-models against the paper's Claude-Code numbers.

End of N-14.
