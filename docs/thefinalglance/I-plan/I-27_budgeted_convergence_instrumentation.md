# I-27: budgeted convergence instrumentation — verified-outcome-rate vs compute curve, Pareto cost/accuracy cockpit

- Track 16 — ROUTING + BUDGETS + MEASUREMENT | lever PROVABLE | grounding STRONG | nature BUILD | effort S-M | WAVE 4
- Premise: the search bet is only credible if more compute demonstrably yields more verified outcomes. I-27
  builds the public instrument: a verified-outcome-rate (W-1-grade, not pass@k) vs total test-time compute
  curve per config, plus a Pareto cost/accuracy cockpit tile so an operator can pick an operating point.
  monotonicity is MEASURED and reported honestly — any non-monotonic region is named, not smoothed (GD-5).

## Steal/build
the benchmark harness (bench_convergence.rs) drives `elai run` across a fixture suite at increasing
compute envelopes (nodes / tokens / $) and records verified-outcome counts from the journal at each
increment. the curve is the mean verified-outcome rate with bootstrap confidence intervals (paired seeds >=5,
per I-38's statistical floor) plotted against cumulative compute spend. the Pareto view overlays cost per
additional verified task as a second axis. the knee (where marginal gain collapses) is identified and named
in the report. monotonicity claim: if the rate ever dips between adjacent compute levels on the fixture set,
the report flags it explicitly — "non-monotonic at [knee]; investigate." nothing is claimed beyond what the
curve shows (GD-5 residue honesty). the SSB row is ConvergenceCurveMeasured (flat beacon; baseline+1).
EXCLUDED: pass@k as a metric (not W-1-grade; the curve uses only W-1 verified-outcome rows). SWE-bench as
a label (landmine #11 — it is retired; ELAI's fixture suite is the corpus). any monotonicity guarantee
beyond the measured sample (GD-5).

## Existing ground truth
- E-49 metrics tuple ($/CTS, STABLE-Pass): the cost denominator and the pass definition. PLAN-asserted; executor re-greps per W-7.
- E-9 bench harnesses: the runner shape to mirror (seccodeplt-runner / include_str! fixtures pattern). PLAN-asserted; executor re-greps per W-7.
- H-9 cost cockpit tile family: the dashboard surface where the convergence curve and Pareto view land. PLAN-asserted; executor re-greps per W-7.
- I-4 telemetry: the SearchNodeEvent rows that supply compute spend per run. depends I-4 (WAVE 4).
- IS-19: wiring tooling live; R8 frozen-gate (the bench harness is a measurement artifact, not a gate; the harness must not be modifiable by the measured session).
- IN-9: NO QDELTA harness yet (I-38 is the enforcement instrument; I-27 is the raw convergence instrument, pre-QDELTA).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/cli/src/bench_convergence.rs (~120 LOC — fixture sweep, compute-level stepping, verified-outcome
  aggregation, ConvergenceCurveMeasured emission)
- cockpit tile (dashboard-v2 Svelte component or extension of H-9 cost tile) rendering curve + Pareto view
- bootstrap CI computation (>=5 paired seeds, per I-38 statistical floor)
- contract test crates/contracts/tests/i27_convergence_instrumentation_contract.rs
- decision doc workflow/reasoning/I-27-decisions.md

DEFER (Tier-2 / gated):
- live (non-batch) curve refresh as I-4 trajectory volume grows (requires sustained search-lane deployment)
- cross-config comparative overlay (requires >=3 configs with sufficient runs; RunPod gated for GPU configs)

## Integration
- CREATE: crates/cli/src/bench_convergence.rs (~120 LOC)
- CREATE or MODIFY: dashboard-v2 convergence tile (Svelte component; hooks into H-9 cost tile family)
- seams from the map: Track 16 anchor — E-49 metrics + E-9 bench shape + H-9 cockpit; I-4 SearchNodeEvent rows as compute-spend source
- frontend_pair: convergence curve tile in the H-9 cost cockpit family. The curve and Pareto view are the W-6 dashboard surface. cross_lang_diff.py covers Tauri -> Svelte. Justified: the tile is the primary user-facing output of I-27; W-6 is not waivable here.

## WIRING_CONTRACT
produces_symbols:
  - crates/cli/src/bench_convergence.rs::ConvergenceBench (fixture sweep + curve computation + SSB emission)
  - dashboard-v2 convergence tile (Svelte — renders ConvergenceCurveMeasured beacon rows)

consumes_symbols:
  - E-49 metrics tuple definitions ($/CTS, STABLE-Pass as the cost and pass denominators) — GD-1
  - E-9 bench harness pattern (include_str! fixtures, runner shape) — GD-1
  - H-9 cost cockpit tile family (dashboard surface; tile added to the family) — GD-1
  - I-4 SearchNodeEvent rows (compute-spend telemetry per node) — GD-1

reachable_from:
  - `elai bench convergence --suite <fixture>` (CLI entrypoint)  [PRODUCTION CLI bench surface]
  - ConvergenceCurveMeasured beacon row in SSB is the off-agent artifact; the cockpit tile reads it for display

stub_ban:
  - NO pass@k as the metric — only W-1-grade verified-outcome rows count
  - NO fabricated monotonicity guarantee — if the curve dips, report it (GD-5)
  - NO paper-derived compute-efficiency numbers as targets (ID-5)
  - NO SWE-bench headline (landmine #11 — retired benchmark)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2

frontend_pair:
  convergence curve + Pareto view tile in the H-9 cost cockpit (dashboard-v2). cross_lang_diff.py covers the Tauri -> Svelte pair. Justified (not N/A): the tile IS the W-6 surface for I-27; the cockpit is the operator's operating-point picker.

observable_effect:
  trigger: "elai bench convergence --suite <fixture-set> --seeds 5"
  expect_ssb: "ConvergenceCurveMeasured beacon row whose curve_points array recomputes byte-identically from the journal under ELAI_SEED (GD-2); bootstrap CI bounds present; knee_compute_level named in the row"
  expect_artifact: "workflow/benchmarks/convergence_<fixture>_<date>.toml (curve points + CI + Pareto data + knee label + any non-monotonic flags); NEGATIVE: a --skip-verification broken arm inflates the curve (higher rate without W-1 gate) — the contract test asserts the broken arm's 'verified_count' exceeds the real count and FAILS (R2)"

## Grounding
- E-49 ($/CTS, STABLE-Pass metrics), E-9 bench shape, H-9 cost cockpit: Track 16 anchor table in I-GROUNDING-MAP.
- GD-5 (honest residue): non-monotonic regions reported, not smoothed; the instrument is the honest signal for the search bet.
- ID-5 (local-baseline-before-paper-threshold): the compute-scaling claim is measured on ELAI's own fixture suite; no external paper number becomes a gate.
- I-38 statistical floor (paired seeds >=5, SEM, bootstrap CI): the ConvergenceBench inherits the floor; I-38 is the enforcement instrument.
- SSB: +ConvergenceCurveMeasured (flat beacon). baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- E-49 metrics: do NOT redefine $/CTS or STABLE-Pass; import the definitions.
- E-9 bench harnesses: do NOT re-implement the runner skeleton; mirror the include_str! fixture pattern.
- H-9 cost cockpit: do NOT fork the cockpit; add the convergence tile to the existing family.
- I-4 telemetry: the SearchNodeEvent rows are the compute-spend source; do NOT re-instrument the search module.
- I-38 QDELTA statistical floor: the CI computation methodology is shared; do NOT re-implement the bootstrap; import or mirror.

End of I-27.
