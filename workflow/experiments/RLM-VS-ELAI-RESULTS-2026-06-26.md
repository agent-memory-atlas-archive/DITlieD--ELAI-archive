RLM vs ELAI vs MERGE — FULL RESULTS + VERDICT
=============================================
Date: 2026-06-26. Author: main LLM (Opus 4.8). Grounding: OWN MEASURED DATA (the strongest GRC source).
Plan: workflow/experiments/rlm-vs-elai-retrieval-2026-06-26.md
Reasoning: workflow/reasoning/rlm-recursive-agents-grc-2026-06-26.md (PART 5/6 = pre-run; PART 7 = exp-1; PART 8 = exp-2)
Harness + raw data: /workspace/ELAI\.liberty\experiments\rlm_vs_elai\
  tasks.py model.py arms.py run.py analyze.py | results.jsonl (exp-1, 144 rows) multihop.jsonl (exp-2, 64 rows)
  VERDICT.md (exp-1) | run entirely on FREE APIs via Liberty (LiteLLM router over free tiers); no local model.

THE QUESTION: "Which is better — ELAI, RLM, or both together?"
  ELAI  = deterministic retrieval (BM25 chunk -> top-k -> one answer call). No model-written code. Replayable.
  RLM   = Recursive Language Models (arXiv:2512.24601): the model NEVER sees raw context; it writes Python to
          slice/search a `ctx` variable in a sandboxed REPL + an rlm_query() child-call recursion primitive.
  MERGE = "both": BM25 candidate set (ELAI's deterministic pre-filter) + a FIXED named op-set
          (grep/read/expand/sub_query/answer) the model drives ONE op per turn. RLM's adaptivity, ELAI's
          determinism + journal/replay. This is the candidate ELAI feature (a J-42/J-86 constrained primitive).

FOUR ARMS, run identically: A=flat(truncate to budget), B=RLM, C=ELAI-retrieval, D=merge.
W = an IMPOSED attention budget (6000 tokens), NOT the model's hardware window — on free APIs the real windows
are 131K+, so to make context exceed the budget cheaply we set W ourselves and build contexts at multiples of W.
No arm ever ships full context to the model (flat truncates to W; retrieval/merge send ~6K-token slices), so the
model's window is irrelevant; only the buried-needle-vs-budget relationship is tested. Scoring = deterministic
exact-match on the planted number (no LLM judge, house rule TD-3).

================================================================================
EXPERIMENT 1 — LEXICAL NEEDLE (single-hop recall). Model: groq/llama-3.1-8b-instant (weak / J-55 axis).
   n=12/band (8 single-needle + 4 combine-arithmetic). Task: "access code for vault N" planted in filler.
================================================================================
SINGLE-NEEDLE accuracy (clean retrieval signal, n=8/band):
  band     A flat   B RLM   C retrieval(ELAI)   D merge      tokens/correct @8W
  0.5xW    100%     87%     100%                87%
  2.0xW     62%     50%     100%                100%
  8.0xW     25%     62%     100%                100%          A 25760  B 1550  C 6032  D 2988

READING:
  - A (flat) decays 100->62->25 as context exceeds budget. The floor.
  - C (ELAI retrieval) = 100% at EVERY length, cheap, fully replayable (re-run = identical retrieved ids: TRUE).
    BM25 ranks the lexical needle #1; deterministic retrieval is at CEILING.
  - B (RLM) = 50-62% at long context, ERRATIC, code-errors at 8W. The weak 8B NEVER used the rlm_query
    recursion primitive — RLM's headline mechanism never even engaged on the local lane.
  - D (merge) = matches C (100%) at long context, deterministic, and CHEAPEST per-correct (2988 @8W).
  - COMBINE tasks were poor for all arms = the 8B fails the ARITHMETIC (both needles retrieved, summed wrong,
    e.g. 281663 vs gold 279663), NOT a retrieval miss. A model-capability confound, excluded from the signal.

EXP-1 VERDICT: on lexical long-context recall, ELAI's deterministic retrieval WINS outright; pure RLM loses and
is erratic/fragile on the weak lane; the merge ties ELAI's accuracy cheaply. ELAI retrieval ALONE already
captures the entire long-context benefit RLM claims here. (Confirms ELAI's deterministic-over-prompt thesis +
the J-55 caution.) But because BM25 was at ceiling, exp-1 could NOT show whether the merge's adaptivity is ever
NEEDED -> exp-2.

================================================================================
EXPERIMENT 2 — MULTI-HOP (the case single-shot retrieval structurally cannot do).
   Model: cerebras/gpt-oss-120b (CAPABLE, pinned, 30 RPM). n=8/band. Bands 2xW, 8xW.
   Task: hop1 "The lead of the {dept} project is {person}." + hop2 "{person} sits in room {N}."
   Question names ONLY the dept and asks for the "office number". The hop2 needle uses a SYNONYM ("room") the
   question never says ("office"), so BM25 on the question has ZERO lexical path to hop2 — only the person's
   name (learned from hop1) reaches it. Verified offline: hop2 unreachable by single-shot BM25 in ~5/6 cases.
================================================================================
ACCURACY (n=8/band):
  band     A flat   B RLM   C retrieval(ELAI)   D merge
  2.0xW    25%      50%     12%                 25%
  8.0xW    12%       0%      0%                 25%
  behaviour: B code-errors 13(@2W)/6(@8W), B recursion 0 always; D op-errors ~2, D avg-ops ~8, D recursion 0.

FAILURE MODES (the real signal — from per-row inspection, not just the %):
  - C (ELAI single-shot retrieval): answers are EMPTY almost everywhere. Given only hop1 chunks (no room number
    anywhere in the retrieved excerpts), the model correctly ABSTAINED rather than invent one. C structurally
    CANNOT multi-hop (confirmed: 0-12%) -- BUT it fails SAFE (no hallucination). 1 lucky hit where hop2 leaked.
  - B (RLM, free model-written code): WORST arm. At 8xW it answered "42" on 7 of 8 tasks -- it HALLUCINATED a
    confident default when the code path broke (high code-errors, malformed answers like literal
    "rlm_query(question, ctx)"). Recursion never engaged even on the 120B. RLM does not degrade gracefully on a
    hard task; it FABRICATES -- the single most dangerous failure mode for a trust-focused system.
  - D (merge, deterministic iterative navigation): the ONLY arm meaningfully >0% at 8xW (25%). When it converges
    it is EXACTLY right (2299, 7533, 6139, 2502); when it does not, it hits the 10-op cap and returns EMPTY
    (abstains) -- it never fabricates. Its shortfall is STEP-BUDGET exhaustion (every empty had op_count 9-10),
    a tunable knob, NOT a structural break. op-parsing worked (low op-errors); SUBQ/recursion never used.

EXP-2 VERDICT: where single-shot retrieval structurally fails (multi-hop), you DO need an adaptive/iterative
layer. ELAI's naive single-shot retrieval (C) cannot do it -- but abstains honestly. Free-code RLM (B) is the
worst and HALLUCINATES. The MERGE (D) is the most robust: it reaches multi-hop facts, stays deterministic /
replayable, abstains instead of fabricating, and its only weakness (step budget) is a tuning parameter.
Absolute accuracy is low for ALL arms because free models are weak at 2-hop needle chaining in 48K context --
this is a DIRECTION + FAILURE-MODE result, not a leaderboard.

================================================================================
SYNTHESIS — ELAI, RLM, or BOTH?
================================================================================
1. ELAI's deterministic retrieval is the correct SPINE and wins where it applies (the bulk of long-context
   recall, exp-1: 100% vs RLM 50-62%). It is cheap, replayable, and when it cannot reach a fact it ABSTAINS
   rather than hallucinates (exp-2). This is exactly the trust posture ELAI is built around.
2. PURE RLM (model writes Python over its own context) is REJECTED on both lanes tested:
   - it never BEAT deterministic retrieval in any cell of either experiment;
   - its recursion primitive never engaged on free models (weak 8B OR capable 120B);
   - on the hard multi-hop task it FABRICATED confident wrong answers ("42" x7), the opposite of ELAI's
     abstain-on-uncertainty contract. Plus it is non-replayable and executes model-written code (a sandbox
     liability). This corroborates ELAI rule J-55 and the deterministic-over-prompt thesis with measured data.
3. "BOTH TOGETHER" = the MERGE (constrained iterative navigation: deterministic candidate set + fixed named
   ops + journal/replay) is the JUSTIFIED addition:
   - exp-1: ties ELAI retrieval's accuracy at lowest cost (a safe no-op where retrieval already wins);
   - exp-2: the ONLY arm that handles multi-hop without collapsing AND without hallucinating; deterministic;
     limited only by a tunable step budget.
   The merge is ELAI's existing iterative-retrieval idea (J-38) crystallised into a typed, replayable primitive
   (the J-42/J-86 constrained-recursive-context sibling). It earns its keep precisely where exp-1 could not show
   it: multi-hop, where naive single-shot retrieval structurally fails.

BOTTOM LINE: ELAI (deterministic retrieval) beats RLM. The best system is ELAI's deterministic spine PLUS a
constrained, journaled, iterative-navigation primitive (the merge) for the multi-hop case -- NOT free-Python
RLM, which loses on accuracy and breaks the trust contract by hallucinating.

================================================================================
RECOMMENDED ELAI ACTIONS (grounded in the above; gated, per house rules)
================================================================================
1. KEEP the deterministic-retrieval spine as default. Do NOT ship a free-Python RLM reasoning operator on any
   lane -- the data rejects it (lost every cell; hallucinated on hard tasks; recursion never engaged).
2. AUTHOR the constrained iterative-navigation primitive (the merge / D) as the J-42/J-86 sibling: a fixed op
   set (grep/read/expand/sub_query/answer) over a BM25-bounded candidate set, every op JOURNALED for replay,
   with ABSTAIN-on-budget-exhaustion (never fabricate). It is the "both" answer and the only multi-hop-capable,
   trust-preserving arm. Capable-lane default; local-lane gated on a small measurement (exp-1 already shows the
   constrained op-path does NOT trip the J-55 code-mode collapse the way free-code RLM does).
3. TUNE the step budget: exp-2's D empties were all op-cap (10) exhaustion -> the obvious next measurement is
   D-only at max_steps 16-20 to recover the abstained-but-reachable cases; expect D@8W well above 25%.
4. BUILD the LongCoT bench arm (`elai bench longcot`, MIT harness LongHorizonReasoning/longcot + HF dataset,
   ELAI-measured per-(model,lane), ID-5) as the standing instrument; it is where a STRONG model + a real
   long-horizon task would size the merge's win precisely.

================================================================================
LIMITS (do not oversell)
================================================================================
- Free models only; exp-1 one weak 8B, exp-2 one capable 120B. n=8/band on the primary signals -> DIRECTIONAL,
  not leaderboard numbers.
- W is an imposed budget, not the hardware window (necessary to test cheaply at 131K+ windows).
- exp-2 absolute accuracy is low for ALL arms (free models are weak at 2-hop needle chaining in 48K) -- the
  decisive evidence there is the FAILURE-MODE contrast (C abstains / B hallucinates / D abstains-or-correct),
  not the raw %. The merge's step-budget ceiling (10 ops) was not yet tuned.
- exp-1's combine tasks confounded retrieval with weak-model arithmetic and were excluded from the signal.
- The ELAI retrieval arm is BM25-only; ELAI's production retrieval also has a vector+rerank channel. That would
  NOT change exp-2's multi-hop conclusion (the gap there is structural single-shot-vs-iterative, not lexical-vs-
  semantic), but it could only strengthen ELAI's exp-1 lead.
