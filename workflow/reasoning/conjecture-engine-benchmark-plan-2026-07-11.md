# Conjecture-engine benchmark — plan-gate BLOCK-rate A/B (2026-07-11)

Purpose: answer the ONLY question that matters about the improvement-conjecture engine —
does generating improvement ideas WITH the machinery (diverge -> deterministic triage ->
executed-leg test -> ranked pick) produce plans that are MEASURABLY more build-ready than
freestyle ideas, or not? The plan's own OPEN LIMITS names this the "honest one": a
before/after on plan-audit BLOCK rates for engine-picked vs freestyle ideas. This doc is the
protocol; it is built to be able to return NO (SF-4: a null result is a real result, reported,
never buried).

## HYPOTHESIS + NULL (falsifiable)

H1: plans authored from ENGINE picks have a LOWER total auditor BLOCK rate than plans authored
    from FREESTYLE ideas, on the same triggers, same author, same template.
NULL: no difference (engine BLOCK rate >= freestyle within noise).
FALSIFICATION: if A3 (engine) does not beat A1 (freestyle-1) on the deterministic primary
metric by more than the bootstrap CI, the engine does not improve plan-readiness at this K —
that is the honest answer and it ships in the report. A partial win (engine reduces the
categories it filters but total is flat) is reported AS SUCH, not spun.

## ARMS (three, to control the best-of-N confound)

A1 FREESTYLE-1: the base generator produces ONE improvement idea per trigger, freestyle.
A2 FREESTYLE-N: the base generator produces 10 freestyle ideas; a NAIVE judge (single "pick the
   best" call, NO triage / NO MAP-Elites / NO executed leg) selects one.
A3 ENGINE: the full 4-stage engine pick (stance-rotated diverge with empty-bin forcing ->
   conjecture_triage.py stage-2 kills -> executed-leg/estimate stage-3 -> judge-ranked pick).
Comparisons:
- A3 vs A1 = the production value proposition (engine vs one-shot freestyle).
- A3 vs A2 = isolates the MACHINERY from the mere best-of-N effect (both see 10 candidates;
  only A3 applies triage + MAP-Elites + the executed-leg requirement).
- A1 vs A2 = does more sampling ALONE help (the control that keeps A3 vs A1 honest).

## CONFOUND CONTROLS (this is what makes it a test, not a demo)

1. SAME BASE GENERATOR for all three arms (Devin swe-1-7). We are testing the MACHINERY, not a
   model swap. A1/A2/A3 all diverge from the same model; only the selection differs.
2. SAME PLAN AUTHOR + SAME TEMPLATE for all arms (Devin swe-1-7, one fixed authoring prompt).
   The author receives ONLY the idea text, BLIND to which arm produced it. The user accepted
   the author-quality confound; holding the author constant makes it at least fair across arms.
3. RECEIPTS-STRIPPED: before authoring, the engine pick's falsifier/anchor/receipt annotations
   are STRIPPED to the same idea-only shape as the freestyle arms. This kills the "engine wins
   because its output LOOKS like a plan" confound — the author works from idea substance only.
4. BLIND, CROSS-FAMILY AUDITOR: the auditor panel is a DIFFERENT family than the author (Devin
   glm-5-2 + kimi-k2-7, majority). It never sees the arm label and never sees the engine run
   artifact (SF-3, structurally — the score harness passes only the plan file). Order/label
   anonymized; a verbosity-length penalty is declared in the audit prompt (I-24/N-15).
5. GOODHART GUARD (the critical one): the auditor's BLOCK taxonomy includes categories the
   engine does NOT filter — design-soundness, decomposition quality, integration-smoke presence,
   spaghetti/maintainability. The HEADLINE is TOTAL BLOCK rate, so the engine cannot win merely
   by pre-filtering its own proxy (anchor/ledger/demarcation). The engine-filtered categories
   and the NON-filtered categories are reported SEPARATELY so a win is attributable and a
   proxy-only win is visible as such (wiring-enforcement R4: measure the objective not the proxy).

## METRICS

PRIMARY (deterministic, reproducible, un-gameable, free):
- P-A PRODUCTION_HOOK-anchor validity: does the authored plan's named seam (file+fn) grep to a
  REAL symbol in the FRESH elai.map (or codegraph-elai)? A hallucinated seam = FAIL. This is the
  un-gameable core of the engine's anchor value-prop and it is a frozen ELAI checker, not the
  engine's own advisory tool.
- P-B plan_gate_check.py pass rate + which of the 10 points fail per plan.
SECONDARY (substantive, panel-judged):
- S total auditor BLOCK rate, and BLOCK rate BY CATEGORY:
  engine-filtered {anchor, ledger-dup, demarcation, composition} vs
  non-filtered {design-soundness, decomposition, integration-smoke, maintainability, other}.
Report per arm: P-A rate, P-B rate, S total, S by category; plus A3-vs-A1 and A3-vs-A2 deltas
with a bootstrap 95% CI and a sign test across triggers.

## TRIGGERS (K=10, REAL — sourced from the corpus so ledger/anchor checks have teeth)

Drawn from named ELAI weaknesses / bench gaps / OPEN-LIMITS / unmeasured proof-spine numbers
(final list pinned at run time from the feature map + proof-spine ruling; must be REAL, never
invented — an invented trigger has no ledger/anchor ground truth). Candidate pool:
context-lifecycle economics; wire-dark rate reduction (PS-6); gate-tax composition (PS-2);
case-reuse compounding (PS-5); routing $/verified-task (PS-4); hallucination-delta CGG ablation
(PS-3); the stage-2 lexical-unsoundness of triage tools; token-ledger coverage; the perception
gateway; the compaction-resume seam. Exactly 10 pinned in the run report's TRIGGERS section.

## PIPELINE per (trigger, arm) — all via the Devin bridge (:8810), zero Claude subagents

1. IDEA: POST to :8810 /v1/messages (swe-1-7). A1: 1 idea. A2: 10 ideas + 1 naive-pick call.
   A3: the scripted 4-stage engine (>=3 stance-rotated diverge calls with empty-bin forcing ->
   conjecture_triage.py --run -> executed micro-experiment where cheap / estimate else ->
   cross-family judge call -> deterministic pick).
2. PLAN: POST (swe-1-7, fixed template, idea-only, arm-blind) -> a short wargame plan file.
3. GATE: python .claude/tools/plan_gate_check.py (P-B) + the PRODUCTION_HOOK-anchor grep (P-A).
4. AUDIT: POST (glm-5-2 + kimi-k2-7, arm-blind, plan-only) -> per-category BLOCK verdict; majority.
All steps are orchestrated by two scripts (built after this doc is approved):
  .claude/tools/conjecture_bench_run.py   (drives arms via Devin, writes per-plan artifacts)
  .claude/tools/conjecture_bench_score.py (P-A/P-B/S aggregation + the A/B deltas + CIs)

## FIDELITY CAVEAT (stated, not hidden)

A3 is a SCRIPTED faithful implementation of the engine's 4-stage protocol, not the Claude-run
skill. This trades a little ecological validity for full reproducibility + zero paid tokens. The
UNIT-5 integration smoke (Claude-run, hard trigger) is the separate check that the SKILL itself
runs; if the scripted A3 and the skill diverge in behavior, that is a finding to report.

## POWER (honest)

K=10 x 3 arms = 30 plans is a PILOT. Report effect size + bootstrap CI + sign test, NEVER a bare
p<0.05 claim. A directional result at K=10 justifies a larger K; it does not settle the number.
This matches the plan's OPEN LIMITS ("a new plan, not this one" for a real number).

## EXECUTION BUDGET (for approval — free Devin, no paid tokens)

- ~ K*(A1:1 idea + A2:11 + A3:~5 engine calls) generation = ~170 Devin calls
  + K*3 plan-author calls = 30
  + K*3*2 auditor-panel calls = 60
  ~= 260 Devin completions total, all FREE (no token $), concurrency-capped at 3 (the bridge's
  limit), so wall-clock is the cost: order of 1-3 hours depending on Devin latency.
- ZERO Claude subagents (the fan-out rule does not bind — these are external HTTP calls to a free
  endpoint, orchestrated by a deterministic script). The only Claude cost is building the two
  harness scripts + reading the final report.
- Prerequisite: fresh elai.map (regenerating now) for P-A + A3 anchor checks; codegraph-elai is
  the FORK-2 fallback.

## DELIVERABLES

1. conjecture_bench_run.py + conjecture_bench_score.py (harness, built after approval).
2. workflow/reasoning/conjecture-bench-run-2026-07-1X.md — the run report: TRIGGERS pinned,
   per-plan P-A/P-B/S, the arm table, A3-vs-A1 + A3-vs-A2 deltas + CIs, and a plain verdict
   (improves / partially / does-not, at this K).
3. one-line outcome appended to the engine plan's OPEN LIMITS.
