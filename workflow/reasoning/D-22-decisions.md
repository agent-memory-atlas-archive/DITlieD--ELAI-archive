D-22 — Conformal calibration validation (closure decisions)

Session: s-25120 completion-drive cluster, D-22 closure
Substrate state at closure: SCHEMA_VERSION = 141 (HEAD ssb.rs:1718); D-22 floor = 86
Plan: docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md

Scope of this closure
The D-22 substrate is already shipped:
  - conformal_validation_runner.rs (orchestrator)
  - ConformalValidationCompleted SSB variant + ConformalValidationCompletedPayload
  - ConformalValidationPanel.svelte (dashboard)
  - commands_d22.rs + get_conformal_validation_recent invoke_handler at lib.rs:757
  - bench_conformal.rs CLI driver + tokio runtime + JSON report renderer
  - conformal_calibration_cli.rs handler for `elai conformal-calibration <status|update>`
  - conformal_refusal_contract.rs (D-16 contract test, SCHEMA floor pin)
This closure ships the 4 remaining gaps the plan called out: a 300-row JSONL
smoke corpus, a per-plan decisions doc, an opt-in CLI module shim, and a
D-22-specific contract test.

DECISION-1 — NO SCHEMA bump.
The substrate is at SCHEMA_VERSION = 141 with the D-22 floor pinned at 86 (per
the D-22 contract test below). The plan never required a SCHEMA bump for the
closure step; the SSB variant landed at SCHEMA = 86 in the original D-22 ship.
Nothing in this closure modifies any SsbPayload variant or wire shape, so
SCHEMA stays at 141.

DECISION-2 — ConformalValidationCompleted SSB variant preserved.
The variant is shipped at ssb.rs:4109 and round-trips through
SsbPayload::ConformalValidationCompleted at ssb.rs:8923. No new variants;
no payload field changes. The contract test pins the round-trip + the
variant tag so future refactors cannot silently drift the wire shape.

DECISION-3 — Fixture layout = 3 JSONL files (NOT plan's per-file JSON).
The plan section "CLI `elai bench conformal-validate`" implied a per-file
JSON layout. The shipped runner reads JSONL fixtures via
`ValidationFixtureRow` (one record per line). This closure ships 3 JSONLs
(low_risk, boundary, high_risk) at 100 rows each = 300 total, matching the
runner-expected read shape. Documented as drift; no plan-shape rewrite to
match the substrate.

Additional brief drift: the closure task brief lists a per-row schema of
`{claim_score: f64, ground_truth_correct: bool, source: String}` (the
CalibrationCorpus tuple shape, NOT the runner's fixture row shape). The
fixtures match the shipped runner schema (`{row_id, distribution, domain,
predicted_score, ground_truth}`). Calibration corpus rows live under
`.elai_cc/calibration/seed/` and use the CalibrationTuple shape; validation
fixtures live under `.elai_cc/benchmarks/conformal_validation/fixtures/` and
use ValidationFixtureRow. The two corpora are paper-distinct artefacts per
UniCR §3 (calibration vs held-out validation).

DECISION-4 — CLI dispatch via `elai bench conformal`.
The plan names the command `elai bench conformal-validate`. The shipped
CliAction enum carries `BenchConformal { dataset, corpus, output, epsilon }`
at args.rs:572. This closure does NOT rename the CLI surface (which would
break operators + CI); the drift is documented here. Future plan refresh
should pin the plan name to `bench conformal` for terminology coherence.

DECISION-5 — No SlashCommand variant added.
The plan section "MODIFY: .elai_cc/crates/cli/src/commands.rs
(SlashCommand::BenchConformalValidate)" never landed and is not needed:
dispatch goes via CliAction::BenchConformal (a top-level CLI subcommand),
not through the slash-command dispatcher. This closure does NOT add a
SlashCommand variant; the drift is documented here.

DECISION-6 — tau.txt persisted reads.
Per D-16-decisions.md DECISION-8, the tau scalar is durable on disk at
`.elai_cc/data/conformal_tau.txt` (default per `tau_path_default()`).
D-22's runner currently recomputes tau in-memory from the loaded
CalibrationCorpus (`compute_tau` in `run_all`). When the production
calibration pipeline lands a non-synthetic corpus, the runner should read
the persisted tau via `TauUpdater::read_persisted_tau` rather than
recomputing. Recorded as a future refinement; NO code change in this
closure (the in-memory path is correct for the synthesised default corpus
used by `bench_conformal`).

DECISION-7 — Paper citation reconciliation.
Canonical UniCR paper of record: arXiv:2509.01455 (per
workflow/papers/unicr-2509.01455-interface.md, D-16 ingestion). The legacy
inline citation `arXiv:2503.01345` appearing in:
  - crates/cli/src/bench_conformal.rs:8 (module docstring)
  - crates/orchestrator/src/benchmarks/conformal_validation_runner.rs (docstring)
  - crates/orchestrator/src/ssb.rs:14387 (ConformalValidationCompletedPayload docstring)
  - .elai_cc/benchmarks/conformal/README.md (line 26)
is a cite-drift artefact from the original D-22 ship (pre-D-16 ingestion of
the canonical paper). Treat as STALE / WRONG citation. This closure does NOT
modify those docstrings (out of scope; would touch core code per the
"no core code changes" constraint). Cite-drift recorded here for a future
docs-only cleanup pass.

DECISION-8 — Selective risk budget invariant.
Per UniCR Eq.5: tau is the (1 - alpha) empirical quantile of the non-conformity
scores `{1 - predicted_score_i}` over the calibration corpus. The runtime
selective-risk invariant is:

    empirical_sel_risk <= alpha + finite_sample_slack

where `finite_sample_slack` is the standard 1/(n+1) Vovk bound (n = corpus
size). For the balanced 20-pair synthetic corpus (n=40, alpha=0.1), slack
~= 0.025; the contract test asserts the slack budget holds against the
target band `[target - epsilon, target + epsilon]` the runner emits. Per
D-16-decisions.md DECISION-4, alpha (miscoverage rate) and rho (selective
risk budget) are paper-distinct quantities; only alpha is on the SSB row.
The empirical selective-risk number is the runner's
`empirical_refusal_rate` field on `ConformalValidationCompletedPayload`.

Ground truth sources
- workflow/papers/unicr-2509.01455-interface.md — UniCR ingestion (D-16),
  alpha=0.1 CRC violation point + rho=0.05 selective-risk budget
- D-16-decisions.md DECISION-2 (RefusalDecision 2-arm lock) +
  DECISION-4 (alpha vs rho distinction) + DECISION-8 (tau.txt persistence)
- D-19-decisions.md (fixture pattern under .elai_cc/benchmarks/prmbench/)
- Vovk, Gammerman, Shafer — Algorithmic Learning in a Random World §2.2

Files touched by this closure
- F:/.ELAI_workflow/.elai_cc/benchmarks/conformal_validation/fixtures/low_risk.jsonl   (NEW, 100 rows)
- F:/.ELAI_workflow/.elai_cc/benchmarks/conformal_validation/fixtures/boundary.jsonl   (NEW, 100 rows)
- F:/.ELAI_workflow/.elai_cc/benchmarks/conformal_validation/fixtures/high_risk.jsonl  (NEW, 100 rows)
- F:/.ELAI_workflow/.elai_cc/benchmarks/conformal_validation/fixtures/README.md        (NEW)
- F:/.ELAI_workflow/.elai_cc/crates/cli/src/bench_conformal.rs                         (extended w/ status()+validate() helper fns)
- F:/.ELAI_workflow/.elai_cc/crates/contracts/tests/conformal_validation_contract.rs   (NEW, D-22-specific contract test)
- F:/.ELAI_workflow/workflow/reasoning/D-22-decisions.md                               (this file)

Files explicitly NOT touched (per scope guard)
- conformal_calibration.rs (D-16 shipped)
- ConformalValidationPanel.svelte (shipped)
- commands_d22.rs base (shipped)
- ConformalValidationCompleted SSB variant + payload (shipped)
- Stale arXiv:2503.01345 citations (cite-drift; future docs-only pass)
