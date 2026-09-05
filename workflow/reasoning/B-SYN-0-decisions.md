B-SYN-0 VERDICT — novelty-instrument discrimination

DATE 2026-07-18
PREREG workflow/reasoning/synergy-validation-preregistration-2026-07-17.md section 4, BLAKE3
8cbdda737e71df0e601d2d4e62cc01cdcd49fcec6a2885178f4336d12b4d5b1a (verified unchanged at run time)
ARCHIVE workflow/benchmarks/synergy-novelty-b-syn-0-2026-07-18/
  corpus_seal_blake3 f897dd5c19b51fdfd646a21b36aa89916d761f804c319eb29dd01bfebcb29123
  scored_run_json    blake3 0cd9ff6782ab5a44c525eb87768ac00b1e61da41a8f23f4a9a8c877d246ca47f
  manifest_id        9bef922fb2cfc3712306aab87b7d5546cbee76c2bc1e57279c3b1336d9d1c35e

VERDICT: SUPPORTED (producer + independent checker agree, mismatch False, checker exit 0)

This is the first non-REFUTED mechanism result on this project after four straight refutations
(P-25 Family A + B, research-creativity b0v3, B-SYN-2). It is a real result and it is BOUNDED; both
halves matter and neither is laundered below.

THE NUMBERS (glm-5-2 free lane, 200 scored calls, 50 pairs x 4 arms)
- P1 accuracy on the 20 known pairs: 20/20 = 1.000; one-sided exact-binomial p vs 0.50 = 9.54e-07.
  Gate (>=0.90 AND p<=0.05): PASS.
- P2 non-UNCERTAIN lift (instrument A1 minus always-UNCERTAIN bin-coordinate baseline A2), paired
  bootstrap seed 20260712 5000 samples: mean 1.000, CI95 lower 1.000. Gate (mean>=0.50 AND lower>0): PASS.
- Guards: A3 scrambled instrument FAILED closed (20/20 rows displaced, 0 emitted, all A3 journal
  verdicts null); A1/A4 order-swap agreement 48/50 = 0.96 (>=0.80); A1 and A4 valid-output 1.00 each
  (>=0.95); cost envelope 200 benchmark calls <=212, one call per pair/arm, prompt max 1815 bytes
  <=5000; integrity seal ok, no label leak, no path leak. ALL PASS.

WHY IT IS CREDIBLE AND NOT DEGENERATE
The instrument varies its answer by pair, it does not lean on one label: known-SAME 10/10 SAME,
known-DISTINCT 10/10 DISTINCT, paraphrase (guard, label SAME) 10/10 SAME, unrelated (guard, DISTINCT)
10/10 DISTINCT, compositional-extension (guard, DISTINCT) 8 DISTINCT / 2 SAME. Overall 28 DISTINCT,
22 SAME. The only wobble is on the genuinely ambiguous compositional-extension class (an extension
shares its base idea), which is also where both order-swap disagreements land (p04, p09). That is
honest borderline behaviour on the hard class, and that class does NOT feed the P1/P2 accuracy gate.
The auditor blind-validated the 20 known-pair labels 0/20 disagreements, so a strong independent judge
agrees the pairs are correctly labelled and discriminable; the instrument getting them right is the
expected behaviour of a capable judge, not a leak (there is none: the judge never sees labels).

WHAT IT MEANS (prereg 4.6 external-validity boundary, quoted, not softened)
Support establishes discrimination ONLY on this sealed ELAI-archive corpus and authorizes the E-1
harness leg for its NEXT GATED implementation step. It does NOT prove out-of-distribution novelty
judgment, and it does NOT reopen the Q-10 / P-25 creativity mechanisms (those stay REFUTED). The
corpus is a floor-level discrimination (reworded-same vs different-idea), not a hard adversarial
benchmark, so a perfect score reflects an easy-but-valid discrimination task, not a difficult one.
Concretely: E-1's verdicts leg may proceed to its next gated step as a discriminating instrument on
in-distribution ELAI research artifacts; it is NOT yet cleared as a general novelty oracle.

CAVEATS AND DEVIATIONS (disclosed, none laundered)
1. CORPUS IS PARTLY SYNTHESIZED. The archive has no natural same-idea duplicates, so the 10
   known-SAME pairs are authored faithful restatements (same mechanism, same target, reworded). The
   10 known-DISTINCT are natural different-mechanism-family pairs. Only these 20 feed the accuracy/lift
   gates; the paraphrase/extension/unrelated 30 feed the order-consistency and valid-output guards
   (label-free). The auditor blind-validated all 20 known labels (0/20 disagreements).
2. SOLO BUILD (owner-approved). The free-executor Agent lane was down, so the corpus + producer +
   tests were authored by the main agent under RULE ZERO (the test contract verifies machinery, never
   outcome, which removes the rigging pressure that produced two preregistration violations in
   B-SYN-2). Independence was preserved where it gates the verdict: the checker was authored by a
   separate agent that never read the producer (ast import-closure: 0 producer imports) and rederives
   the verdict from the frozen archive alone; the adversarial-auditor validated the machinery through
   a deterministic fake client (5 integration legs) with zero live calls and did the blind label check.
3. TEMPERATURE-0 VIA LANE DETERMINISM. The devin bridge returns a gateway internal error when a
   temperature field is present, so temperature is omitted from the request. Prereg 4.3 requires
   temperature 0 for DETERMINISM; that intent is met and was verified empirically (glm-5-2 byte-
   identical across 3 repeat calls on multiple pairs). Documented deviation from the literal
   "send temperature 0"; the scientific requirement (deterministic scoring) holds.
4. UPSTREAM-ERROR-AS-TRANSPORT. The bridge returns HTTP 200 with an upstream gateway/quota error in
   the message text; the client treats these as transport failures (journaled non-durable, run halts
   and resumes by pair/arm key, never resampled), never as instrument outputs. Without this, lane
   flakiness would have been charged to instrument accuracy. The scored run resumed through several
   transient gateway 500s; final 200/200 durable, one durable call per pair/arm, zero resampling.
5. HEALTH-CALL COUNT. The prereg speced 1 health probe; 6 external health/diagnostic calls were made
   (1 formal probe that hit a transient gateway 500, then 5 diagnostics to confirm transient-not-
   persistent and that glm-5-2 works vs swe-1-7/kimi-k2-7). All USD 0, recorded outside the benchmark.
   The 212-call BENCHMARK cap (one durable call per pair/arm) is intact and is what gates the verdict.
6. NON-BLOCKING AUDIT WARN CARRIED FORWARD. The run_benchmark stage call-cap check became a
   length-identity after resume_plan was wired in (skip+todo == len(selected_pairs)*4 by construction),
   so it no longer catches a corrupted/foreign journal; total calls stay hard-bounded to 212 by the
   fixed 50-pair corpus, and this run started from an empty journal so it could not bite. FOLLOW-UP:
   add an independent "every durable journal row's pair_id is in corpus.json" integrity check.

PROCESS RESULT (the discipline held)
The independent checker caught nothing wrong (it agreed), but the process caught real defects before
they could bias the result: the fake-client audit found the A3 scramble did NOT fail closed under a
perfect instrument (label-agreement canary was seed-dependent) -> rebuilt as identity-binding, seed-
independent; the auditor found resume_plan wire-dark -> wired into the production path; and the
REFUTED-vs-INVALID taxonomy was over-classifying quality-guard misses -> corrected to prereg 4.5. The
health probe caught the gateway-error-as-200 and the temperature incompatibility BEFORE the scored run.
Machinery-not-outcome (RULE ZERO) meant a SUPPORTED result was reached with no test ever asserting the
hypothesis must pass -- the opposite of the B-SYN-2 failure.

FILES
  .claude/tools/synergy_novelty_bench_corpus.py       sealed corpus builder
  .claude/tools/synergy_novelty_instrument_bench.py   producer + live CLI
  .claude/tools/synergy_novelty_instrument_check.py   independent checker (0 producer imports)
  .claude/tools/tests/test_synergy_novelty_instrument.py  23 tests, machinery-not-outcome
  workflow/benchmarks/synergy-novelty-b-syn-0-2026-07-18/  sealed archive (corpus/prompts/smoke/scored/manifest)
  workflow/sessions/s-autoresearch-synergy-2026-07-17/{B-SYN-0-BUILD-SPEC,B-SYN-0-ARCHIVE-SCHEMA,run_b_syn_0_live}.*
