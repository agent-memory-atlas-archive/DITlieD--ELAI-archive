Model Refresh Cadence — Operator Reference (K-9)

Overview

`elai bench model-refresh` is the deterministic quarterly cadence check that
answers one question per hardware lane: is the model we currently run still the
one the catalog would pick, or has a stronger candidate appeared. It composes
J-9's HardwareModelCatalog (it re-implements none of the ranking) and emits one
decision row per lane: coding, vision, ocr, embedding_reranker.

The load-bearing property is that it NEVER swaps a model on its own. A better
candidate is only flagged, never adopted. Adoption is a separate, human-gated
step (W-10) that needs a live fixture eval plus owner ratification. This leg is
the cheap, always-runnable half: it tells you where to look, it does not act.

The Decision Rule

Each lane produces a verdict by comparing the catalog's top eligible candidate
against a recorded incumbent baseline (model_id + fit_score from the last
cadence snapshot, held in the INCUMBENTS table in the runner):

  candidate.fit_score >  incumbent baseline  ->  re_evaluate
  candidate.fit_score <= incumbent baseline  ->  keep

`keep` is the safe arm: an equal or regressing candidate leaves the incumbent in
place, and the incumbent_model field on that row is always non-empty (the
retention is explicit, never a silent drop). `re_evaluate` is a flag, not a
swap: it says a live W-10 eval is worth funding for that lane, nothing more.
This deterministic leg emits ONLY keep and re_evaluate. `adopt` is reserved for
the W-10 live leg and is never produced here.

Determinism

The runner does not call the live SystemProber. It builds against a fixed 24 GB
probe fixture (probed_at_ms: 0), so the catalog it scores and the verdicts it
derives are byte-reproducible run to run. `run_model_refresh(None, 0)` called
twice yields identical reports. The measured fields that need a real run
(pass_rate, tok_per_s, dollars_per_verified_task) are None on every row this
leg, and fixture_run is false, because no fixture has been executed yet.

The Live Leg (deferred, W-10)

`live_eval_pass_rate` is wired into the runner but returns None until the W-10 /
RunPod eval is funded (marked WIRE-DARK[K-9-W10] in the runner). When that leg
lands it fills pass_rate / tok_per_s / dollars_per_verified_task from a real
fixture run, sets fixture_run true, and only then can a lane earn the `adopt`
verdict, still behind owner ratification. Until then the cadence check stays a
pure planning signal.

Layering

The runner is pure: it builds decision rows and returns a report, it does not
touch SSB. The CLI handler (`elai bench model-refresh`, bench_model_refresh.rs)
owns the side effects: it writes one ModelRefreshCadence SSB row per decision
(IncumbentKept for a keep, DecisionRecorded for a re_evaluate) and seals one RRA
over the session stream. This mirrors the shipped routing-economics benchmark,
where the runner computes and the CLI persists. The contract test
k9_model_refresh_contract.rs locks both surfaces: the SSB payload round-trips
and the runner's safe-arm behavior.

Usage

  elai bench model-refresh                 all four lanes
  elai bench model-refresh --lane coding   one lane (coding|vision|ocr|embedding_reranker)
  elai bench model-refresh --seed 7        seed passed through to the report
  elai bench model-refresh --json          pretty-printed report to stdout

Grounding

  SSB action type ModelRefreshCadence lands at SCHEMA_VERSION 276, auto-derived
  from SSB_ACTION_TYPES.len() (never hand-edited). The payload struct
  ModelRefreshCadencePayload and its ModelRefreshCadenceKind live in
  crates/orchestrator/src/ssb/payloads.rs.

  J-9 HardwareModelCatalog (crates/orchestrator/src/hardware_catalog.rs) is the
  ranking source this bench composes. The cadence check adds no ranking of its
  own; it reads the catalog's eligible/fit_score output and compares to the
  incumbent baseline.
