D-21 Mode-routing coverage benchmark — decisions

Phase: D-21 (mode-routing coverage benchmark)
Date: 2026-05-20

What
----

Land an orchestrator-side benchmark that replays a hand-labeled fixture
of synthetic claims through D-10's pure predicate `select_grounding_mode`,
computes per-mode F1 + macro F1 + 5x5 confusion matrix, and gates pass
iff macro F1 >= 0.85 AND every per-mode F1 >= 0.70. CLI subcommand
`elai bench mode-routing` drives CI; a Tauri command + Svelte panel
surface the runs on the dashboard.

Key decisions
-------------

(a) Mini-fixture size: 25 rows (5 per mode), NOT the 200-row full
benchmark called for in the D-21 plan §1. Rationale: the plan's full
fixture set is scoped for the cloud test A live run, but the CI gate
needs an in-tree fixture small enough for a deterministic smoke test
that runs sub-second. 5 rows per mode is the smallest count that gives
F1 = 1.0 against a perfect predicate (one TP per mode); the 40-per-mode
expansion is deferred per plan §Out-of-scope and tracked separately.

(b) F1 macro 0.85 + per-mode 0.70 floors: shipped as
`DEFAULT_MACRO_F1_FLOOR` + `DEFAULT_PER_MODE_F1_FLOOR`. The macro floor
guards against total-system regression; the per-mode floor guards
against class collapse (an aggregate F1 of 0.85 with a 0.0 per-mode
score is a false PASS the gate must refuse). CLI exposes both as
`--macro-floor` / `--per-mode-floor` overrides so the cloud live run can
tighten or loosen the gate without touching the runner.

(c) Hand-labeled fixture (not LLM-generated): plan §4 stub_ban makes
this binding. Fixtures live at
`.elai_cc/benchmarks/mode_routing/fixtures/mode_routing_mini.jsonl`
with a README documenting the schema + provenance. Lines beginning with
`#` are skipped by the loader so authors can leave inline comments.

(d) Drawer key `mode-routing-bench` (not `mode-routing`): the D-10
phase already shipped a drawer keyed `mode-routing` for the predicate's
runtime distribution panel. The benchmark surface uses a distinct key
so the existing panel is unaffected and both can be open
simultaneously.

(e) SSB schema bump 77 → 78. Single new variant
`ModeRoutingBenchRunCompleted` carrying `(total, accuracy, f1_macro,
gating_verdict, fixture_blake3, fixture_row_count)`. action_type CHECK
list and `all_test_payloads()` length 166 → 167 updated atomically.

(f) Synthetic canary record (not boolean flag): when a fixture row
sets `canary_blocks: true`, the runner constructs a full
`CanaryStatusSummary { decision_kind: "fail", ... }` so the routing
predicate sees a real blocking record. Defends against future drift in
`OrchContext::canary_blocks()` semantics — the runner sees what
production sees.

Unknowns
--------

- Cloud test A (200-row live run) is still pending; the mini fixture
  cannot validate cross-domain coverage at scale.
- D-22 conformal calibration will tighten the gate on
  `ConformalRefusal` once UniCR is wired (current fixture relies on the
  `conformal_calibration_ready` boolean only).

Expected test outcome
---------------------

`cargo check --workspace`: zero new errors.
`cargo test -p elai_orchestrator -- mode_routing`: all new tests pass.
`cargo test -p elai_cli --test bench_mode_routing_smoke`: PASS;
  observable_effect smoke runs in <1s.
`svelte-check` on `crates/dashboard-v2/`: zero new errors.
Mini fixture: macro F1 = 1.0, every per-mode F1 = 1.0, verdict = Pass.

Files touched
-------------

ADD:
- .elai_cc/crates/orchestrator/src/benchmarks/mode_routing_runner.rs
- .elai_cc/crates/cli/src/bench_mode_routing.rs
- .elai_cc/crates/cli/tests/bench_mode_routing_smoke.rs
- .elai_cc/benchmarks/mode_routing/fixtures/mode_routing_mini.jsonl
- .elai_cc/benchmarks/mode_routing/fixtures/README.md
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d21.rs
- .elai_cc/crates/dashboard-v2/src/lib/drawers/ModeRoutingBenchPanel.svelte
- workflow/reasoning/D-21-decisions.md

MODIFY (additive only):
- .elai_cc/crates/orchestrator/src/benchmarks/mod.rs (+1 `pub mod`)
- .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 77 → 78,
  +1 SsbActionType variant, +1 payload struct, +1 CHECK entry,
  +1 from_str arm, +1 SsbPayload variant, +1 action_type arm,
  +1 test fixture row, count 166 → 167)
- .elai_cc/crates/cli/src/args.rs (+1 CliAction variant, +1 parser
  function, +1 bench sub-verb dispatch)
- .elai_cc/crates/cli/src/main.rs (+1 mod, +1 dispatch arm)
- .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (+1 mod, +1
  invoke_handler entry)
- .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (+1 DrawerKey)
- .elai_cc/crates/dashboard-v2/src/lib/drawers/DrawerHost.svelte (+1 import + route)
- .elai_cc/crates/dashboard-v2/src/lib/layout/Drawer.svelte (+1 label)
