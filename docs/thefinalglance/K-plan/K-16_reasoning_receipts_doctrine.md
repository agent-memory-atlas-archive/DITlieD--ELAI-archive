# K-16: reasoning-receipts discipline doctrine — make "a reasoning trace is never admissible as confidence; only an external receipt is" a binding pre-gate for any future reasoning-engine work

STATUS: PLANNED (WAVE KW-3; CANT-LIE+DISCIPLINE; E-23 + doctrine class, K-8/K-14-shaped; a .claude/rules/ doctrine file + a conditional-routing trigger; NO runtime code, NO SSB; reuses the K-1 claims-policy lint)

Objective: the GRC-v3 reasoning run (workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md) surfaced one finding that recurs FOUR independent times across the 2024-2026 papers and is the single load-bearing correction to the user's framing: a model cannot reliably ground itself or judge which of its own parts is "sure" (the self-correction blind spot + the faithfulness results — Reflexion/Tsui, ToT self-eval, Lanham faithfulness, "Consensus is Not Verification"). So a reasoning TRACE is for monitoring/UX ONLY; the confidence/selection signal MUST come from an EXTERNAL RECEIPT. ELAI already lives this in its spine (F-4 cant-lie, D-13 cross-family, I-4 select-by-receipts, IC-6) — but it is NOT written down as a binding constraint AT THE REASONING-STEP GRAIN, so a future executor of the GRC-v3 capability (J-83..J-86) or any later reasoning-engine work would re-discover it at audit (or, worse, silently fall back to LLM self-eval when receipts run out). K-16 promotes the rule into the planning doctrine as a binding pre-gate (exactly the K-8/KD-4 + K-14/TZ pattern): .claude/rules/reasoning-receipts.md, four clauses RR-1..RR-4, loaded by conditional-routing for any reasoning-engine / GRC / reasoning-budget / candidate-selection task. This is near-zero new capability — it is the DISCIPLINE that makes the J GRC-v3 track (and everything after it) built correctly from the start. HARD BOUNDARY it must NOT cross: it does NOT author the capability (J-83..J-86 = J); it does NOT introduce a runtime gate (a .claude/rules/ doctrine + a conditional-routing trigger + a reuse of the K-1 lint, auditor-applied; the runtime enforcement is the SHIPPED F-4/D-13/I-4 the rule cites); it does NOT re-state F-4/D-13/I-4/IC-6 mechanics (it BINDS them at a new grain, GD-1/IC-16); it does NOT claim a measured number (E-23/doctrine class, KD-1).

Estimated effort: ~0.5 session (~2 h). A .claude/rules/ doctrine file + a conditional-routing trigger + a K-1 claims-policy recogniser extension + a negative fixture. No build, no SSB, no new dependency.
Pre-requisites (re-grep at authoring, W-7): the K-1 claims-policy lint (K-16 reuses + extends it, never forks; GD-1); the SHIPPED runtime spine the rule cites — F-4 TEST-INTEGRITY CANT-LIE GATE (AntiFabricationGate, blake3 stdout_hash), D-13 ADVERSARY CROSS-FAMILY, I-4 EVIDENCE-RANKED SEARCH LADDER (select-by-receipts), H-39 abstention_score.rs (the receipt-sourced confidence bit); CLAUDE.md IC-6 / RD-1 / TD-3 (the doctrines RR-1..RR-4 extend to the reasoning-step grain). The J GRC-v3 track J-83..J-86 (the capability the rule pre-gates) + K-15 (the measure-your-own-knee bench whose ID-5 rule RR-3 states).
Parallel-safe with: K-13, K-14, K-15 (disjoint files). Soft-depends K-1 (lint). Pre-gates the J GRC-v3 track.
Source: reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS (the one-paragraph answer + §3 selector hierarchy + the DONE-GATE) + IMPLICATION 7/8/13/20 + ROUND-3 ADD-2 (faithfulness, the third independent grounding) + the HONEST RESIDUE T1; _grounding/J-GRC-V3-GROUNDING-MAP.md DC-3/DC-5 + GW. Doctrine: K-0_INDEX.md (KD-1, E-23/K-8/K-14 rule precedent); IC-6 / RD-1 / TD-3 / RD-4 / ID-5 (the spine clauses RR-1..RR-4 bind).

## Context
Why this matters: the user's "the part the model is more sure about proceeds, the rest stays" is right in shape but wrong in one word — "sure" must mean "receipt-confirmed", not "chain-self-confident". A CoT trace can be a post-hoc rationalization (a green trace does not mean correct reasoning; 46% near-miss arithmetic / 54% semantic in the CoT failure analysis); larger models can be LESS faithful (the visible rationale is not the real causal process); a model fails to correct its OWN errors while successfully correcting identical errors presented as EXTERNAL input (the blind spot). So the engine grounds the model by feeding it external receipts; the model does not ground itself. Without this written as doctrine, the most natural failure mode is: when no deterministic receipt exists, the system silently falls back to an LLM grading its own output (the banned self-judge), or it cites a paper's reasoning-budget knee instead of measuring ELAI's own. K-16 forecloses both, at the reasoning-step grain, BEFORE the J capability is built. Mechanism: one .claude/rules/ doctrine file (conditional-routing loads it on reasoning-engine tasks) + a K-1 lint recogniser so positioning copy cannot ship a "self-verifying reasoning" over-claim.

THE FOUR-CLAUSE REASONING-RECEIPTS INVARIANT (the rule K-16 ships; binds future GRC-v3 / reasoning-engine work):
  RR-1 TRACE-NEVER-ADMISSIBLE: a reasoning trace / chain-of-thought is for monitoring + UX ONLY; it is NEVER admissible as the confidence or selection signal. The confidence/abstention bit on a reasoning step comes from an EXTERNAL receipt (test / checker / pixel-diff / retrieval verdict) read by the engine (F-4 / H-39, RD-1), never set by the chain self-grading. (the self-correction blind spot + faithfulness; three independent papers.)
  RR-2 SELECTOR HIERARCHY: when selecting among reasoning candidates, deterministic RECEIPT > statistical VOTE over INDEPENDENT samples (receipt-ABSENT fallback ONLY, and NEVER for factual-recall — which routes to retrieval, shared-training-bias) > LLM self-eval (BANNED). A vote is never the selector when a receipt exists; LLM self-eval is never the selector at all (IC-6, DC-3, the ToT 8B-can't-self-evaluate rider). (the receipt-absent vote rung is the only legal place consensus appears.)
  RR-3 MEASURE-YOUR-OWN-BUDGET: a reasoning-budget / overthinking ceiling / difficulty threshold is ELAI-MEASURED per-(model,task-type) on an ELAI gold set (the K-15 bench), NEVER an imported paper constant (ID-5; the overthinking paper itself forbids transferring its knee — the local lane overthinks earliest/lowest). The WINDOW is never the lever for a depth-class reasoning failure (RD-4 / RULER).
  RR-4 COMMIT-AND-FREEZE: a receipt-confirmed reasoning step is COMMITTED and FROZEN — the generator does not re-open it (it cannot reliably re-judge it anyway, the blind spot). Commit is simultaneously the grounding, the safe-compression boundary (preserve receipts losslessly, fold prose lossily; never an LLM re-summarization, HD-2; ciphers banned, IC-3), and the overthinking defense.

## Existing state (A/B..J coverage — what the rule cites, do NOT re-author, GD-1/IC-16)
- F-4 TEST-INTEGRITY CANT-LIE GATE [WIRED] AntiFabricationGate.check() task_runner.rs:9565; blake3 stdout_hash — the RR-1 archetype (the auditor cannot emit passed=true without the orchestrator-captured hash). The rule BINDS it at the reasoning-step grain, does not restate its mechanics.
- D-13 ADVERSARY CROSS-FAMILY [DARK] different pretraining family, never same-model self-critique — the RR-2 "no self-eval" runtime archetype.
- I-4 EVIDENCE-RANKED SEARCH LADDER [DARK] select BY RECEIPTS not LLM-judge — the RR-2 selector runtime (J-86 activates it; K-16 states the rule it obeys).
- H-39 abstention_score.rs [WIRED] reads correct/wrong/abstain from the EVAL LOG (a receipt) — the RR-1 receipt-sourced confidence bit (RD-1).
- the J GRC-v3 track J-83..J-86 (this run's capability half) — the work the rule pre-gates; K-15 (the RR-3 measure-your-own-knee bench).
- K-1 dossier + claims-policy lint (the KD-1 enforcement K-16 reuses); K-8 (the .claude/rules/ + conditional-routing pre-gate precedent); K-14 (the TZ doctrine sibling — same shape: a four-clause invariant binding a future J track); E-23 positioning-artifact precedent (prose + a rule, near-zero code, no SSB).
- CLAUDE.md doctrines RR-1..RR-4 extend: IC-6 (receipts-not-votes / ladder-not-MCTS), RD-1 (abstention bit from D-16/H-39, never re-derived), TD-3 (LLM advisory, never the load-bearing gate), RD-4 (depth-not-width / window-never), ID-5 (ELAI-measured baselines never paper constants), HD-2 (no LLM re-summarization), IC-3 (ciphers banned).

## Reads (mandatory before coding)
1. workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS (the one-paragraph answer + §3 selector hierarchy + the DONE-GATE) + IMPLICATION 7/8/13/20 + ROUND-3 ADD-2 + the HONEST RESIDUE T1.
2. _grounding/J-GRC-V3-GROUNDING-MAP.md (DC-3 selector hierarchy, DC-5 depth-vs-width-vs-window, GW the paper grounding) + the K-1 claims-policy lint (reuse, GD-1).
3. the SHIPPED spine the rule cites — F-4 AntiFabricationGate, D-13 adversary, I-4 ladder, H-39 abstention_score.rs — re-grep live to cite correctly (W-7), do NOT re-implement.
4. K-8's plan + .claude/rules/ + conditional-routing.md (the pre-gate shape) + K-14's plan (the TZ doctrine sibling K-16 mirrors).
5. CLAUDE.md: KD-1, IC-6, RD-1, RD-4, ID-5, TD-3, HD-2, IC-3 (the clauses RR-1..RR-4 bind at a new grain).

## Tasks (WAVE KW-3 — the reasoning-receipts doctrine + the claim recogniser)
1. [DISCIPLINE] Author .claude/rules/reasoning-receipts.md: the four-clause invariant RR-1..RR-4 above, as a HARD RULE binding any task that builds/audits reasoning-engine machinery (a reasoning-state carrier, a reasoning-effort/budget classifier, an overthinking guard, a candidate selector); cite F-4/D-13/I-4/H-39 as the SHIPPED runtime archetypes (BIND, don't restate; GD-1/IC-16). Add a conditional-routing.md trigger ("building/auditing a reasoning-engine / GRC / reasoning-budget / candidate-selection unit: read .claude/rules/reasoning-receipts.md"). ~55-70 lines.
2. [PACKAGE] one-line cross-link so the GRC-v3 differentiator claim (banked by K-15) is governed by RR-1 (trace-never-admissible) — the lint already enforces it; K-16 names the rule it enforces. No new dossier row (K-15 owns it; K-16 binds it).
3. EXTEND the K-1 claims-policy lint recogniser (NOT a fork; GD-1) so a "self-verifying reasoning / the model checks its own reasoning / chain-of-thought confidence" claim FAILS unless it carries the receipt-not-self-eval correction, and a reasoning-budget threshold cited from a paper (not ELAI-measured) FAILS (RR-3/ID-5). ~10-15 LOC (composes the K-15 recogniser if K-15 lands first; else K-16 adds it).
4. NEGATIVE arm: a fixtures file with (a) "ELAI's chain-of-thought self-verifies" untagged, (b) a candidate-selection design that uses an LLM grading its own paths, (c) a reasoning-budget set to the paper's 7K knee — the lint fails (a)/(c) and the rule (auditor-applied) catches (b).
5. workflow/reasoning/K-16-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - .claude/rules/reasoning-receipts.md (the RR-1..RR-4 invariant; the binding pre-gate for future GRC-v3 / reasoning-engine work)
  - the conditional-routing.md trigger that loads it for reasoning-engine / GRC / reasoning-budget / candidate-selection tasks
  - the self-verifying-reasoning + paper-knee recogniser extension in .claude/tools/claims_policy_lint.py (K-1's lint; extended)
consumes_symbols:
  - the K-1 claims-policy lint (EXTEND, do NOT fork; KD-1; GD-1)
  - F-4 / D-13 / I-4 / H-39 (the SHIPPED runtime archetypes the rule CITES; bound at a new grain, not re-implemented; IC-16)
  - the GRC-v3 reasoning run + DC-3/DC-5 (the rule's source; promoted, not re-derived)
  - CLAUDE.md IC-6 / RD-1 / TD-3 / RD-4 / ID-5 / HD-2 / IC-3 (the doctrines RR-1..RR-4 extend)
reachable_from:
  - conditional-routing loads .claude/rules/reasoning-receipts.md when a reasoning-engine / GRC task runs [PRODUCTION harness gate; JD-1 = the rule fires on a relevant task]
  - the auditor runs claims_policy_lint.py over any positioning copy carrying a reasoning-grounding claim [PRODUCTION audit gate]
  - the J GRC-v3 track (J-83..J-86) executor MUST obey the rule (the pre-gate binds it, like K-8/KD-4 binds efficiency work) [PRODUCTION pre-gate]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO trace/chain-of-thought admitted as a confidence or selection signal (RR-1; only an external receipt)
  - NO LLM self-eval as a candidate selector, and NO vote-when-a-receipt-exists / NO vote-for-recall (RR-2)
  - NO paper-constant reasoning-budget / overthinking-knee (ELAI-measured per-(model,task-type); RR-3/ID-5)
  - NO fork of the K-1 lint (extend the recogniser; GD-1); NO main-LLM self-run of the lint (auditor-applied); NO re-implementation of F-4/D-13/I-4 (cite; IC-16); NO authoring of the J capability here
frontend_pair:
  N/A — a .claude/rules/ doctrine file + a positioning/claims recogniser, no dashboard surface. Justified (W-6).
observable_effect:
  trigger: "a session building/auditing a reasoning-engine unit (e.g. any of J-83..J-86) loads .claude/rules/reasoning-receipts.md (conditional-routing), and the auditor runs claims_policy_lint.py over the reasoning-grounding copy"
  expect_ssb: "none (E-23/doctrine class; JD-1 harness evidence — the rule fires via conditional-routing; the lint is a deterministic check)"
  expect_artifact: "the rule file exists and conditional-routing loads it on a reasoning-engine task (a live session shows it firing); the K-1 lint PASSES a receipt-grounded-and-tagged claim and FAILS (the NEGATIVE arm that MUST fail) a 'self-verifying reasoning' untagged over-claim + a paper-knee-as-ELAI's threshold; the rule (auditor-applied) catches an LLM-self-eval candidate selector in a reviewed design"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-16_reasoning_receipts_doctrine.md
test -f .claude/rules/reasoning-receipts.md
rg -n "RR-1|RR-2|RR-3|RR-4" .claude/rules/reasoning-receipts.md            # the four clauses present
rg -n "reasoning-receipts.md" .claude/rules/conditional-routing.md         # the trigger is wired
python .claude/tools/claims_policy_lint.py --files docs/thefinalglance/K-plan/_fixtures/reasoning_selfverify_negative.md ; test $? -ne 0   # the over-claims MUST fail
rg -ci "self.verif|chain.*confidence|model.*grades.*own" docs/strategy/*.md   # MUST be 0 (no self-verifying-reasoning over-claim ships)
test -f workflow/reasoning/K-16-decisions.md
```

## ACCEPTANCE
- [ ] the four-clause invariant RR-1..RR-4 is written as a binding rule + conditional-routing loads it on reasoning-engine / GRC / reasoning-budget / candidate-selection tasks.
- [ ] RR-1/RR-2: the rule forbids a trace as a confidence signal and an LLM self-eval selector; cites F-4/D-13/I-4 as the shipped archetypes (bound, not restated).
- [ ] RR-3: the rule forbids a paper-constant reasoning budget (ELAI-measured per-(model,task-type), K-15; ID-5) and window-as-the-lever (RD-4).
- [ ] the K-1 lint is EXTENDED not forked (GD-1); a "self-verifying reasoning" / paper-knee claim FAILS; no runtime gate is introduced (auditor-applied; the runtime enforcement is the cited F-4/D-13/I-4).
- [ ] no runtime code, no SSB, no new dependency (E-23/doctrine class); the J capability (J-83..J-86) is NOT authored here.

## WIRING DONE-GATE (clause by clause)
- W-1: done = conditional-routing actually loads the rule on a reasoning-engine task (JD-1) + the lint negative fixture FAILS, not "the file exists".
- W-2: N/A (no feature flag). W-3: the design is PROMOTED into a rule, not summary-ported. W-4/W-6: N/A dashboard; justified.
- W-5: trigger = a live reasoning-engine task firing the rule + the real lint over real copy. W-7: F-4/D-13/I-4/H-39 re-grepped at edit (cited correctly).
- GD-1/IC-16: bind the wired spine (F-4/D-13/I-4), never restate its mechanics; extend the K-1 lint, never fork. KD-1: the lint IS the enforcement (no reasoning-grounding claim without the receipt-not-self-eval correction).
- JD-1: harness evidence = the rule fires via conditional-routing; the lint is a deterministic check (no SSB).

## Tests to add
- a Python test for the lint extension (or crates/contracts/tests/k16_reasoning_receipts_claims_contract.rs ~40 LOC): (a) a receipt-grounded-and-tagged reasoning claim passes; (b) "ELAI's chain-of-thought self-verifies" fails; (c) "the model checks its own reasoning" fails; (d) a reasoning-budget set to the paper's 7K knee fails (RR-3); (e) empty/missing input fails-loud (R1). The RR-clauses are auditor-applied (no runnable code check until J-83..J-86 exist), so the rule's test is the conditional-routing load (JD-1) + a reviewed-design catch, not a code assertion.

## Files touched
- CREATE: .claude/rules/reasoning-receipts.md (~65 LOC, the RR-1..RR-4 invariant)
- MODIFY: .claude/rules/conditional-routing.md (~2 LOC: load the rule on reasoning-engine / GRC / reasoning-budget / candidate-selection tasks)
- MODIFY: .claude/tools/claims_policy_lint.py (+~12 LOC self-verifying-reasoning + paper-knee recogniser; K-1's lint, extended — composes the K-15 recogniser if K-15 landed first)
- CREATE: docs/thefinalglance/K-plan/_fixtures/reasoning_selfverify_negative.md (~12 LOC, the negative arm)
- CREATE: workflow/reasoning/K-16-decisions.md (~25 LOC)

## Citations / Grounding
- reasoning-engine-grc-v3-2026-06-25.md FINAL SYNTHESIS (the one-paragraph answer: "'sure' must mean receipt-confirmed, not chain-self-confident") + §3 (the selector hierarchy = RR-2) + the DONE-GATE (ELAI-measured, never a paper number = RR-3); IMPLICATION 7 (select-by-receipt), 8 (the receipt-absent vote rung), 13 (no self-eval, the 8B rider), 20 (the self-correction blind spot — the engine grounds the model = RR-1); ROUND-3 ADD-2 (faithfulness, the third independent grounding).
- ELAI ground truth (the SHIPPED archetypes the rule binds): F-4 AntiFabricationGate task_runner.rs:9565 (blake3 stdout_hash) [WIRED]; D-13 adversary cross-family [DARK]; I-4 select-by-receipts [DARK]; H-39 abstention_score.rs (receipt-sourced bit) [WIRED] — re-grepped at edit (W-7).
- arXiv [EXTERNAL, grounding for WHY not an ELAI number]: Reflexion 2303.11366 + Tsui (the self-correction blind spot — fix EXTERNAL not self-critique), ToT 2305.10601 (self-eval is the weak link, 8B can't self-evaluate), self-consistency 2203.11171 + "Consensus is Not Verification" 2603.06612 (receipts > vote; vote fails for shared-bias), Lanham faithfulness (the visible rationale is not the causal process), overthinking 2604.10739 (the knee is model-specific, do NOT transfer = RR-3).
- CLAUDE.md: KD-1, IC-6 (receipts-not-votes/ladder-not-MCTS), RD-1 (abstention from D-16/H-39), RD-4 (depth-not-width/window-never), ID-5 (measure-your-own), TD-3, HD-2 (no LLM re-summarization), IC-3 (ciphers banned), E-23/K-8/K-14 rule precedent.

## Open questions / risks
- Scope discipline (the K-14 lesson): K-16 ships the RULE + the CLAIM-governance only. The temptation is to start authoring J-83 (the carrier) — resist it; that is the technical J track (authored as J-83..J-86 this same wave, but as J files, not here). K-16 is the pre-gate that makes them correct.
- the rule must BIND the shipped spine, not RESTATE it (GD-1/IC-16) — F-4/D-13/I-4 already enforce receipts-not-self-eval at runtime; K-16 names the rule at the reasoning-STEP grain so the J executor obeys it from the start. A K-16 that re-implements F-4's logic is a P0 (GD-1).
- the K-1 lint recogniser for RR-1/RR-3 overlaps the K-15 differentiator recogniser — coordinate so they compose (one recogniser, two claim shapes), never two forks of the lint (GD-1). If K-15 lands first, K-16 extends its recogniser; if K-16 lands first, K-15 extends K-16's.
- TECHNICAL COMPANION WORK, OUT-OF-K (recorded; the J GRC-v3 capability track, authored this wave as J-83..J-86 per the user's 2026-06-26 ruling, all BOUND by RR-1..RR-4): J-83 the typed BeliefState reasoning carrier [KEYSTONE, RR-1/RR-4]; J-84 the 2-axis reasoning-effort classifier [RR-3]; J-85 the deterministic overthinking guard [RR-3/RR-4]; J-86 the receipt-gated candidate ladder [RR-2]; the I-14 preserve-receipts/fold-prose compression RIDER. K-16 is the discipline that gates them; K-15 is the proof.

## Next
K-16 makes the receipts-not-traces rule binding at the reasoning-step grain BEFORE the GRC-v3 capability is built (so the J executor builds it correctly, not corrected at audit), and governs the K-15 differentiator claim. It pairs with K-1 (the lint it extends), K-15 (the bench + differentiator it governs), K-8/K-14 (the .claude/rules/ pre-gate siblings), and the J GRC-v3 track (the capability it pre-gates). The reasoning trace stays monitoring/UX; only the external receipt is admissible — the whole trust spine, applied one level down to reasoning itself.

End of K-16.
