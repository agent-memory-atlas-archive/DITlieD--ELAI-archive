PROMPT FOR THE NEXT SESSION — BENCHMARK THE CGG REPAIR LOOP (catch -> REPAIR -> re-verify -> 0% emitted hallucination)
======================================================================================================================
Paste everything below the line into a fresh session. It is self-contained; read the listed files FIRST.

----------------------------------------------------------------------------------------------------------------------

GOAL
We already proved (last session) that CGG's DETECTION signal works (PopQA AUROC 0.83 mid / 0.91 weak;
code behavioral-consistency 0.93) and that ABSTENTION cuts emitted hallucination ~75% but COSTS ~50%
coverage. Detection+abstention is a CEILING: it removes wrong answers without producing right ones.

YOUR JOB: build and benchmark the REPAIR LOOP — when CGG catches a high-risk/hallucinated answer, instead
of abstaining, spend bounded effort to ACTUALLY IMPROVE the answer (regenerate with grounded feedback /
retrieve evidence / run the checker / re-reason), RE-VERIFY against an EXTERNAL receipt, and commit ONLY
if it now passes. Measure whether this reaches ~0% EMITTED hallucination AT HIGH COVERAGE (the whole point
is recovering the coverage pure abstention throws away). This is the L-15 RepairController, benchmarked.

THE ONE NUMBER THAT DECIDES IT (the done-gate, per L-15 R4): H_emit at FIXED 90% coverage, three arms:
  (a) baseline (answer everything), (b) abstain-only gate (last session's CGG), (c) REPAIR loop.
The thesis succeeds iff arm (c) holds H_emit near 0 while keeping coverage ~90% — i.e. it REPAIRS to a
correct answer instead of refusing. Report H_emit at 100/90/75/50% coverage for all three, per domain.

READ THESE FIRST (full paths)
  Measured findings (start here):  /workspace/ELAI\workflow\experiments\cgg_bench\CGG-BENCH-FINDINGS.md
  The repair-loop architecture:    /workspace/ELAI\workflow\reasoning\cgg-zero-hallucination-opus-reasoning-2026-06-27.md
  The plan (RepairController):     /workspace/ELAI\docs\thefinalglance\L-plan\L-15_repair_controller_and_evidence_first_reorder.md
  The CGG design (predictors):     /workspace/ELAI\workflow\reasoning\cgg-hallucination-causal-system-2026-06-27.md
  The CGG plan units it composes:  /workspace/ELAI\docs\thefinalglance\L-plan\L-0_INDEX.md (then L-3, L-5, L-6, L-7, L-9, L-10)
  The GRC reasoning system:        /workspace/ELAI\workflow\reasoning\rlm-recursive-agents-grc-2026-06-26.md
  HippoRAG (deterministic multi-hop retriever, REUSE for QA repair): /workspace/ELAI\.elai_cc\crates\memory\src\hippo_rag.rs

REUSE THE EXISTING HARNESS (do NOT rebuild it) — /workspace/ELAI\workflow\experiments\cgg_bench\
  cgg.py           arms, normalize(), clustering, verdict_* (domain-pluggable keyfn)
  code_exec.py     sandboxed executor + behavioral signature + passes_visible/grade_code (THE code receipt)
  domains.py       QA(PopQA)/MATH(GSM8K)/CODE(MBPP) specs (prompts, extraction, deterministic grading)
  data_popqa.py    PopQA loader (popularity-stratified, gold aliases)
  run_domain.py    generation (stores raw_*.jsonl)
  derive_domain.py / derive_code.py   offline verdict + deterministic grade
  analyze.py       risk-coverage / AURC / operating points (audit-hardened: AURC tie-grouping, gate-only arms)
  model.py         Liberty model seam (pinned litellm, retries, think-strip)
  Datasets present: popqa.tsv (QA), mbpp.jsonl (code), math500.jsonl (HARD math — USE THIS, 500 competition
    problems, levels 1-5), gsm8k_test.jsonl (TOO EASY — ceiling, do NOT use for the math test)
  math500_grade.py   Windows-safe deterministic MATH-500 grader (\boxed extraction + sympy; 500/500 self-match)
  Run with: /workspace/ELAI\.liberty\.venv\Scripts\python.exe   (Liberty router; keys in /workspace/ELAI\.liberty\.env)
  Free models that worked: groq/llama-3.1-8b-instant (weak; HITS DAILY TOKEN CAP — use sparingly or another
  provider), mistral/mistral-small-latest (mid, reliable), groq/openai/gpt-oss-120b (capable, fast),
  cerebras/gpt-oss-120b (capable but ~12s/call). Spread load across providers; grade DETERMINISTICALLY
  (alias/numeric/test-execution) — NO LLM grader.

BUILD THE REPAIR ARM PER DOMAIN (the core new work — each is a bounded loop, max ~3 rounds, then abstain)
  CODE (strongest, do FIRST — the executable receipt is ground truth):
    detect: a sample fails the VISIBLE test (assert[0]).  repair: regenerate WITH the failing test +
    error message fed back into the prompt ("your code failed this test: <assert> with <error>; fix it").
    re-verify: run the VISIBLE test again; loop up to N rounds; COMMIT only a version passing the visible
    test. GRADE on HELD-OUT tests (assert[1:]) so it is non-circular. Compare vs A5 (single do-test, no
    repair) and baseline. EXPECT: emitted hallucination -> near-0 at HIGH coverage (repair recovers the
    fails A5 abstained on). This is the headline.
  MATH (use HARD math — GSM8K was a CEILING mistake; see DATASET note):
    USE MATH-500 (math500.jsonl, competition AMC/AIME, 5 difficulty levels) NOT GSM8K. A mid model gets
    ~30-50% here, so there is REAL hallucination to detect/repair. STRATIFY BY LEVEL (1-5) and report CGG's
    AUROC per level — the signal should be strongest on the hard levels (more confabulation).
    RECEIPT NUANCE (important): math does NOT have a free ground-truth oracle like code. You do NOT have the
    gold answer at inference, and a symbolic checker verifies STEP VALIDITY / EXPRESSION EQUIVALENCE, not
    "is the final answer correct". So math's receipt class is BETWEEN QA and code:
      detect: low self-consistency over final answers (semantic entropy on \boxed{} values).
      repair: regenerate with "re-derive step by step and verify each step"; OPTIONAL symbolic step-check
        (sympy: does this algebra step preserve equality) to catch invalid steps; re-verify by
        self-consistency CONVERGENCE across the new samples (+ recompute numeric subresults deterministically).
      commit: on a STABLE, step-consistent answer; abstain if it stays divergent. NOTE this receipt is
        weaker than code's executable one (it cannot certify final correctness without the gold) — so expect
        math repair to land BETWEEN QA and code (recovers some, not all; honest residue = a confidently-wrong
        stable derivation). Run a WEAK model (more errors to repair) on a NON-rate-capped provider.
    GRADER: math500_grade.py (Windows-safe: \boxed{} extraction + sympy equivalence; tested 500/500 self-match)
    OR the GOLD path math-verify (pip-installed; its mp timeout BREAKS on Windows -> run grading under WSL/
    the remote Linux box, or pass timeout_seconds=None and still expect Windows mp errors). math500_grade.grade
    (pred_text, gold_latex)->bool is ready to use as the deterministic grade for the MATH-500 domain spec.
  QA (retrieval receipt — the hard case):
    detect: high semantic entropy / low consistency.  repair: fire DETERMINISTIC retrieval (BM25 over a
    corpus, or HippoRAG-style; for the bench you may use a Wikipedia/Wikidata snippet lookup or the PopQA
    s_uri/o_uri context) and RE-ANSWER grounded on the retrieved text; re-verify the new answer's
    consistency + that it is SUPPORTED by the retrieved snippet (round-trip). COMMIT if grounded, else
    abstain. EXPECT: recovers the "model didn't know but it was retrievable" subset; the unretrievable
    residual still abstains (that is the honest floor).

HARD RULES (from the L-plan / house doctrine — violating these re-introduces hallucination)
  - TD-3 / the TRUST INVARIANT: the COMMIT must be gated on an EXTERNAL receipt the model did NOT author
    (run the held-out test / symbolic check / retrieved-snippet round-trip). NEVER commit on the model's
    own say-so that it "fixed it" — that is Reflexion-style self-correction and it re-hallucinates at the
    verify step. This is the single most important constraint; the whole result is meaningless without it.
  - R3 deterministic repair-retrieval: for QA, the retrieval must be DETERMINISTIC (BM25/HippoRAG), the
    model only reads the snippet; NEVER model-driven free navigation (measured 87% deterministic vs 25%
    model-driven elsewhere).
  - Non-circular code grading: repair against VISIBLE tests, GRADE on HELD-OUT tests (the Goodhart guard).
  - Bound the loop: max ~3 repair rounds, then abstain (credit the abstention, do not penalize honesty).
  - Measurement integrity: an adversarial-auditor found 5 BLOCK + 8 WARN in the last harness (all fixed) —
    re-audit any new measurement code the same way (store raw outputs, deterministic grading, risk-coverage
    not a single tuned point, held-out splits, errors count in denominators). See CGG-BENCH-FINDINGS.md sec 2.

DELIVERABLE
  1. A repair_<domain>.py (or extend derive_*) implementing the bounded repair loop with external-receipt
     commit, reusing code_exec.py / domains.py / model.py.
  2. The three-arm table per domain: H_emit at 100/90/75/50% coverage for baseline vs abstain-only vs
     REPAIR, plus COVERAGE-at-H_emit=0. The decisive cell: H_emit @ 90% coverage, repair vs abstain-only.
  3. A verdict: does the repair loop reach ~0% emitted hallucination at HIGH coverage where a receipt
     exists (expected: YES for code, YES for math, PARTIAL for QA bounded by retrievability)? Name the
     irreducible residual (confident-consistent fabrication of unretrievable facts).
  4. Append results to CGG-BENCH-FINDINGS.md and update L-15 (R4 done-gate) with the measured numbers.
     Docs are gitignored; do not commit unless asked.

WHY THIS MATTERS (the one line): last session proved CGG can DETECT and ABSTAIN well; this session proves
whether it can REPAIR — turn a caught hallucination into a CORRECT answer against an un-authored receipt —
which is the difference between "75% cut at 50% coverage" (a STOP sign) and "near-0% emitted hallucination
at ~90% coverage" (the actual goal). Code is where it should win cleanest; QA is where the honest floor lives.
