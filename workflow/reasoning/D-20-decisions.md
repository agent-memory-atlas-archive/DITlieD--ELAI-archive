D-20 decisions: ARC-AGI-2 OOD probe harness (genuine-novelty gating)

Author: ELAI executor, session finishing unit D-20-decisions
Date: 2026-06-06
Scope: records the 3 decisions from the D-20 plan. The benchmark RUN
itself is cloud-model-gated and DEFERRED (no local 7B path for ARC;
see Deferred section). This doc captures decisions only; it does not
report any solve-rate, refusal-rate, or pass/fail measurement (none
has been produced — zero arc_benchmark_completed rows in ssb.db, no
file under benchmarks/arc_agi2/reports/).

Plan source: docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md (Tasks 6a/6b/6c)
Plan status: workflow/sessions/s-plan-audit-2026-06-04/plan-status/D/D-20_arc_agi2_ood_probe.md (PARTIAL)
Code under decision:
  - crates/orchestrator/src/benchmarks/arc_agi2_runner.rs (ARCAGI2Runner, ARCReport, ARCVerdict, run_all)
  - crates/cli/src/bench_arc.rs + main.rs:899 dispatch (elai bench arc)
  - benchmarks/arc_agi2/arc_mini_fixture.jsonl (24 synthetic FixtureRow rows)
  - benchmarks/arc_agi2/README.md (fixture format + honest-evaluation definition)
Grounding for the external claims below:
  workflow/papers/arc-prize-2025-2601.10904-interface.md (ARC Prize 2025
  Technical Report, Chollet/Knoop/Kamradt/Landers, arXiv:2601.10904,
  ingested in D-6) — all numeric ARC facts cited here trace to that file.


DECISION (a): cloud Sonnet only; no local 7B for the ARC lane.

Decision: the ARCAGI2Runner / `elai bench arc` lane is wired to drive a
cloud model (Sonnet via the B-37 cloud-mode CloudClient) and is NOT to
be run against the local Qwen2.5-Coder-7B sidecar.

Rationale (grounded):
  - The ARC-AGI-2 private split tops out at NVARC 24.03% for a purpose-
    built competition system, with 2nd/3rd at 16.53%/12.64%
    (arc-prize-2025-2601.10904-interface.md, "PERFORMANCE BENCHMARKS",
    Section 2.2 Table 1). Human baseline is 100% (same file, "Human
    baseline", Section 1.1). The accuracy gap is described as an
    engineering bottleneck and the efficiency gap as a fundamental-
    science bottleneck (same file, "FAILURE MODES" #5, Section 4.8).
  - A general-purpose 7B running ARC tasks would sit near the floor; for
    comparison the open-source TRM (7M params, zero-pretraining) gets 8%
    on ARC-AGI-2 and CompressARC 4% (same file, "PERFORMANCE BENCHMARKS",
    ARC-AGI-1 comparison rows). A 7B coding model is not optimized for
    ARC grid abduction at all, so the local lane would add cost and no
    signal.
  - More importantly, D-20 does NOT gate on solve-rate (see decision b).
    The thing under test is ELAI's orchestrator-side honesty stack
    (D-10 mode-routing predicate + D-15 novelty classifier + the no-
    auto-publish rule), not the underlying model's puzzle-solving. The
    cloud model is the realistic substrate the production routing
    predicate actually sees; pointing the probe at the local 7B would
    measure routing over a model nobody ships ARC work to.

Side effects / edge cases:
  - This makes the RUN cloud-budget-gated, hence DEFERRED. The code path
    is real and reachable (main.rs:899 -> bench_cmd::run_bench_arc ->
    bench_arc::run; no default-OFF flag), but it has never executed
    against a live cloud model.
  - Caveat carried into the report: ELAI on cloud Sonnet is not a
    frontier ARC system; expect single-digit to low-double-digit solve
    rate if/when a real grid-solving lane is added (D-20 plan lines
    20-21). The current runner does not even attempt solving — it gates
    the framework.


DECISION (b): honest-evaluation pass criterion, NOT a solve-rate threshold.

Decision: `elai bench arc` exits 0 when the gating verdict is the honest
one (ARCVerdict::HonestlyEvaluated) and exits 1 on the two dishonest
verdicts (GameWithMemorization, Refused-out-of-band). There is NO
minimum solve-rate gate. The verdict is computed over the orchestrator's
behaviour, not over puzzle correctness.

What "honest" means operationally (benchmarks/arc_agi2/README.md
"Honestly evaluated definition" + arc_agi2_runner.rs:124-131,158-171):
  - honestly_evaluated_count = rows D-15 classified DiscoveryCandidate
    AND D-10 routed to a non-Constructive mode (one of
    FalsifiabilityCrossFamily / OperationalDeferred / ConformalRefusal).
  - memorization_game_count = rows classified Search AND routed to
    Constructive — the explicit memorization-leak signature; the verdict
    trips GameWithMemorization when memorization_game_count / TOTAL
    classified rows exceeds MEMORIZATION_GAME_THRESHOLD (default 0.4 =
    40% of total rows), per arc_agi2_runner.rs:340 (const), :578 (rate =
    count / total), :587 (verdict). NOTE: the stale
    benchmarks/arc_agi2/README.md:73-76 says "> 60% of Search-class rows"
    but that 60% is a DIFFERENT const, REFUSAL_DOMINANCE_THRESHOLD (:346);
    shipped memorization-gaming logic is 40%-over-total, not
    60%-over-Search-class.

Rationale (grounded):
  - A solve-rate threshold would gate on FUTURE model improvements, not
    on ELAI's framework. ARC scores rise with model capability while the
    framework can silently regress; a solve-rate gate would mask exactly
    the regression D-20 is meant to catch (D-20 plan lines 90-93;
    README lines 21-24).
  - The ARC Prize report itself warns that high scores no longer imply
    fluid intelligence, only knowledge coverage of the ARC domain
    (arc-prize-2025-2601.10904-interface.md "FAILURE MODES" #4, Section
    4.2), and that contamination is not reliably detectable from score
    alone (same file, "CONTAMINATION WARNINGS" #5, Section 4.1). So a
    raw solve-rate number is not even a trustworthy capability signal —
    pinning a pass/fail on it would be grounding the gate on a metric
    the source paper says is unreliable.
  - The framework-anchored criteria (refusal SLA in band, no auto-
    publish of any DiscoveryCandidate, sycophancy hygiene) are things
    ELAI actually owns and can regress on; they are the correct gate.

Side effects / edge cases:
  - Exit 0 on any solve-rate is intentional and could read as "the
    benchmark always passes." It does not: GameWithMemorization and an
    out-of-band Refused share both exit 1. The gate is on routing
    honesty, which is falsifiable per-run via the off-agent SSB row.
  - The plan's original "contamination heuristic" (Task 4: flag if
    solve-rate > 50% on an old-access model) was redesigned as the
    memorization-gaming signature (MEMORIZATION_GAME_THRESHOLD over the
    Search/Constructive concentration). Same intent (catch leaked /
    memorized solutions), but anchored on routing behaviour rather than
    on a solve-rate number the source paper calls unreliable. This is a
    deliberate divergence from the plan text, recorded here.


DECISION (c): 50-task subset, not the full set.

Decision: the documented target for the real fixture is a 50-task
stratified subset of the ARC-AGI-2 evaluation set, not the full set.

Rationale (grounded):
  - Cost. Each run threads every task through a cloud model. NVARC's
    competition figure was $0.20/task and the commercial refinement
    harnesses ran $0.81-$60/task (arc-prize-2025-2601.10904-
    interface.md "PERFORMANCE BENCHMARKS", Sections 2.2/3.4). A full
    sweep multiplies that linearly for a gate that does not need full
    coverage to detect a routing regression.
  - 50 is representative ENOUGH for the framework gate: the gate measures
    distributional shape (DiscoveryCandidate vs DistantSearch vs Search,
    and the mode each lands on), not per-task correctness. A stratified
    sample across the three NoveltyClass values exercises every branch
    of the verdict logic, which is all the honest-evaluation criterion
    needs (README "Mini fixture scope"; arc_agi2_runner.rs verdict logic
    at run_all).
  - "Full 400" framing in the plan acceptance is loose. Per the source
    paper the splits are 400 PUBLIC training tasks (ARC-AGI-1 imports) +
    120 semi-private + 120 private (arc-prize-2025-2601.10904-
    interface.md "DATASET STRUCTURE"). Any held-out benchmark claim must
    use the semi-private or private split, never the public 400 (same
    file, "HELD-OUT TEST SET METHODOLOGY", official score reporting
    rule). So the right full-set frame is the 240 evaluation tasks, of
    which only the 120 private are contamination-resistant; a 50-task
    stratified probe is a deliberate, documented sample, not lazy
    truncation. Full-set runs are DEFERRED (D-20 plan Out-of-scope).

Side effects / edge cases:
  - The 50-task subset MUST be a documented stratified sample with a
    version/checksum to detect dataset drift (D-20 plan Task 1, WIRING
    stub_ban "50-task subset MUST be a documented stratified sample").
    The runner already records a blake3 of the on-disk fixture bytes in
    ARCReport.fixture_blake3 (arc_agi2_runner.rs:132-134), so any silent
    fixture edit changes the audit trail.
  - Contamination caveat that must travel with any future real-fixture
    sourcing: the semi-private 120 carry leakage risk (exposed to
    commercial APIs); only the private 120 are theoretically clean
    (arc-prize-2025-2601.10904-interface.md "CONTAMINATION WARNINGS"
    #3/#4). The private split is not publicly downloadable, which is
    precisely why the bundled fixture is synthetic (see Fixture gap).


Fixture gap (NOT closed this session — honest record):

The plan acceptance asks for ">= 50 fixtures present". The bundled
arc_mini_fixture.jsonl has 24 rows and was NOT expanded toward 50 this
session. This is a deliberate, grounded refusal, not an omission:

  - The fixture is SYNTHETIC by design. Each row is a FixtureRow
    (task_id, claim_text, claim_type, expected_novelty_class,
    dkg_match_count, near_duplicate_threshold) — a natural-language
    claim framing plus the two synthetic evidence scalars the D-15
    classifier sees (README "Fixture format"; arc_agi2_runner.rs:185-204;
    the task_ids are arc_synthetic_001..024).
  - Real ARC-AGI-2 public tasks (arcprize.org) are grid-pair JSON
    (train/test input/output integer matrices, up to 30x30, values 0-9
    per arc-prize-2025-2601.10904-interface.md "DATASET STRUCTURE").
    They carry NO expected_novelty_class, NO dkg_match_count, and NO
    near_duplicate_threshold. Those three fields are MODEL-DERIVED — a
    novelty class and DKG/cosine-overlap evidence are produced only by
    an actual classify + DKG-walk over a real run. Attaching them to a
    sourced real task would be fabricating model-derived labels, which
    is banned.
  - Adding MORE synthetic rows would NOT satisfy "real ARC-AGI-2 public
    tasks" (the only grounded sourcing the task permits) and would just
    be more invented claim-text.

Therefore both expansion paths fail the no-fabrication constraint, and
the honest action is to leave the fixture at 24 rows and record the gap.
This matches the existing plan-status REMAINING item ("Expand
arc_mini_fixture.jsonl from 24 rows to at least 50"). Closing it
correctly requires the cloud-gated run that produces real classifier
labels for sourced grid tasks (or a re-scope of the fixture schema to
carry grids + run-time labels), which is the same deferred cloud work
as the benchmark run itself.


Deferred (cloud-model-gated; carried to the cloud-run session):
  - Run elai bench arc against the fixture via cloud Sonnet; write
    benchmarks/arc_agi2/reports/{date}.json (D-20 plan Task 3 / WIRING
    expect_artifact).
  - Confirm >= 1 arc_benchmark_completed row lands in ssb.db (the off-
    agent observable_effect; arc_agi2_runner.rs:39-49,
    SsbActionType::ArcBenchmarkCompleted via ARCSsbEmitter).
  - Cloud smoke A/B/C/D from D-20 ACCEPTANCE (solve-rate + refusal-rate
    + distributions; refusal_rate in [5%, 50%] per D-17; zero auto-
    publish of DiscoveryCandidate; sycophancy_flag_count near zero).
  - Source a real 50-task stratified subset (private split preferred for
    contamination resistance) and derive its labels from the live run,
    closing the fixture gap above.

Off-agent artifact this doc leaves:
  - This file is the decision record the D-20 plan (Task 6) and the
    plan-status REMAINING list both require to exist. It is plain-text
    markdown, parses cleanly, authors no fabricated measurement, and
    cites every external ARC fact to the ingested paper interface.
  - It does NOT itself produce an ssb.db row — the SSB-row observable
    for D-20 is the arc_benchmark_completed row, which is correctly
    gated behind the deferred cloud run (decision a). Claiming an SSB
    row here without running the benchmark would be a fabricated result.
