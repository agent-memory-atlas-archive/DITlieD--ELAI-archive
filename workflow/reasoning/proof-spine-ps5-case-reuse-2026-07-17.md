PS-5 CASE-REUSE — FIRST MEASURED OBSERVE-ARM BASELINE (2026-07-17)

Ruling: workflow/reasoning/proof-spine-2026-07-05.md (PS-5 row). Rule: .claude/rules/proof-spine.md
(PSR-4). This is a dated ledger row off a REAL off-agent benchmark artifact the controller did not
author (D-6 / wiring-enforcement R4). KD-1/GD-2/GD-3 framing: this is a receipt, not a superlative.
Read the CAVEATS section before citing this number anywhere. In particular this is the OBSERVE-arm
recurrence-hit-rate over the SHIPPED retrieval stack, NOT the compounding reuse-into-generation moat
number (that is K-18 Task 2, still BENCH-PENDING on the unbuilt VCG J-track — see CAVEATS).

HEADLINE NUMBER (fixture-grain, static retrieval, dated 2026-07-17)
  recurrence_hit_rate@1 = 0.95  (19/20 recurrences: paired proven case retrieved at rank 1)
  recurrence_hit_rate@3 = 1.00  (20/20 at rank <= 3)
  recurrence_hit_rate@5 = 1.00  (20/20 at rank <= 5)
  false_match_rate@{1,3,5} = 0.00 / 0.00 / 0.00  (REDTEAM Attack-A false-positive control: no
    distractor scored at/above the score floor into any top-k)
  retrieval_mode = bm25_only
  95% Wilson CI at ref_k=1: [0.85, 1.00]

UNIT OF MEASURE + SCOPE (state it loudly)
  Unit: a single "recurrence" query against the embedded fixture corpus. n_recurrences = 20 queries,
  each paired to exactly one of n_proven = 20 proven cases, mixed with n_distractors = 20 unrelated
  cases across class_count = 5 problem classes. A "hit@k" = the query's PAIRED proven case appears in
  the top-k retrieval AND at/above the runner's SCORE_FLOOR. false_match@k = a distractor (no paired
  recurrence) surfaced at/above the floor into top-k (a false-positive; must stay 0 for the number to
  mean anything).
  Retrieval stack exercised: the SHIPPED elai_knowledge BM25 index (tantivy), retrieval_mode
  bm25_only. The ONNX/RRF hybrid legs are NOT staged in this run (the artifact self-reports
  bm25_only), so this is a pure lexical-retrieval baseline, not the hybrid-reranked number.
  Corpus: embedded in the runner binary (include_str! of case_reuse_fixtures/corpus_v1.json),
  blake3-pinned, filesystem-independent — the number is byte-reproducible from the binary + seed.

NUMERATOR / DENOMINATOR (raw, from the artifact JSON)
  suite:            case_reuse_fixture_v1
  n_proven:         20
  n_recurrences:    20   (the denominator for recurrence_hit_rate@k)
  n_distractors:    20   (the denominator for false_match_rate@k)
  class_count:      5
  seed:             0
  corpus_blake3:    7b9d68a3e98b08981f628a0cb2c6be67c62ae2a06e1207850cd3199425853a11
  match_everything: false
  hit_rate@1 = 19/20 = 0.95 ; hit_rate@3 = 20/20 = 1.00 ; hit_rate@5 = 20/20 = 1.00

OFF-AGENT EVIDENCE (D-6 / R4 — the artifact the controller did not author)
  Primary: .elai_cc/benchmarks/case_reuse/case-reuse-1784295862167.json (the JSON above, written by
    the runner, blake3-pinned corpus).
  Receipt: .elai_cc/benchmarks/case_reuse/case-reuse-1784295862167.rra — a receipts-audit record
    carrying a RecurrenceHitRateMeasured event row (action_type recurrence_hit_rate_measured) with the
    per-k rates, the corpus_blake3, and a root_blake3 chain (8d94554fc861ed59). This IS the off-agent
    beacon; it landed in the .rra receipt, NOT in a sessions/ssb.db table (see CAVEAT on the SSB row).
  Produced off-agent via a free-lane swe-1-7 worker (D-6 non-author) driving the real CLI subcommand;
  the controller read the on-disk artifact, never the worker's stdout (see CAVEAT: worker-return
  divergence — the worker's reported numbers were WRONG; the artifact is authoritative).

RECOMPUTE (GD-2 — exact command, byte-reproducible)
  cd .elai_cc && ./target/debug/elai bench case-reuse --suite fixture --seed 0 --json
  (writes a fresh benchmarks/case_reuse/<ts>.json; corpus is embedded + blake3-pinned so the per-k
  numbers reproduce exactly for seed 0. CLI parse: crates/cli/src/args_bench.rs
  parse_bench_case_reuse_args; dispatch: crates/cli/src/bench_cmd.rs run_bench_case_reuse; runner:
  crates/orchestrator/src/benchmarks/case_reuse_runner.rs.)
  Layer-0 status read (advisory, PSR-4): python .claude/tools/proof_spine_status.py --json

CAVEATS (what this number does and does NOT capture)
  - OBSERVE-ARM ONLY, NOT THE MOAT NUMBER. This is recurrence-hit-rate: "does the shipped retrieval
    stack surface the right proven case when a known recurrence comes in". It is ZERO reuse-into-
    generation. The compounding number PS-5 ultimately wants — does reusing a retrieved proven case
    measurably improve a downstream generation vs not reusing it (the case-reuse moat / K-18 Task 2) —
    is BENCH-PENDING on the unbuilt VCG J-track by design. Do not cite 0.95@1 as "case-reuse works";
    cite it as "the retrieval leg that case-reuse depends on hits 0.95@1 on the fixture". The moat
    delta is a separate, harder, still-unmeasured number.
  - FIXTURE, NOT PRODUCTION CORPUS. n=20 recurrences over an embedded 20-proven/20-distractor/5-class
    fixture (corpus_v1.json). A ceiling of 1.00@3 on a hand-built fixture is expected and is NOT
    evidence of production retrieval quality; it is evidence the retrieval leg + score-floor + the
    false-positive control are WIRED and behave on a controlled input. The CI [0.85,1.00] is a
    small-n Wilson interval — 20 queries is a smoke-grade sample, not a population estimate.
  - BM25_ONLY. The artifact self-reports retrieval_mode = bm25_only; the ONNX/RRF hybrid rerank legs
    were not staged in this run. A hybrid run is a different (and separately-worth-measuring) number.
    NOTE: an early worker return claimed retrieval_mode "hybrid" with much lower rates
    (0.1429/0.3809/0.4762) — that return was FABRICATED / did not match any on-disk artifact; the
    authoritative artifact is bm25_only at 0.95/1.00/1.00. This is exactly the R4 failure mode
    (never trust worker stdout; read the off-agent artifact) working as designed. Logged.
  - NO ssb.db BEACON ROW. The RecurrenceHitRateMeasured event landed in the .rra receipt file, not in
    a queryable sessions/ssb.db table (a census of all 22 ssb.db on this box carries no
    recurrence_hit_rate_measured rows). The .rra receipt is a valid off-agent artifact (R4 satisfied),
    but a downstream tool expecting an ssb.db row will not find one. If PS-5 later needs an ssb.db
    beacon (e.g. for a dashboard tile), that is a follow-up wiring item, not a defect in this number.
  - FROZEN-TOOL STATUS. This is a shipped runner (bench case-reuse subcommand), not a one-off scratch
    script — the RECOMPUTE command is a real, repeatable CI-able artifact, stronger than PS-6's
    scratch-sweep provenance.

PROOF-SPINE STATUS FLIP (advisory, PSR-4)
  proof_spine_status.py's PS-5 heuristic looks for a FILENAME containing "case_reuse" or "case-reuse"
  (artifact_patterns), searched SHALLOWLY in .elai_cc/metrics/, workflow/status/, or
  workflow/reasoning/. This file (proof-spine-ps5-case-reuse-2026-07-17.md, directly in
  workflow/reasoning/) matches "case-reuse", so proof_spine_status.py flips PS-5 from RUNNER_PRESENT
  to MEASURED (2/6 measured: PS-5 + PS-6). This is a Layer-0 ADVISORY flip only (PSR-3/PSR-4): it
  NEVER discharges a done-gate, and the MEASURED verdict rests on the artifact + RECOMPUTE above, not
  on the filename match. The remaining four: PS-2 gate-tax is journal-gated (needs an e2e-populated
  ssb.db with EvidenceGateEvaluated/TieredAnalysis rows — Q-16/GO-2 downstream); PS-1 safety-delta is
  RunPod-gated; PS-4 routing-economics is cloud-$/J-28-gated; PS-3 hallucination-delta is PLANNED.
