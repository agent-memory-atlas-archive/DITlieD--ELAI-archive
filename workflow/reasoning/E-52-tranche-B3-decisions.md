E-52 Tranche B-3 — LSP diagnostic probe producer (LOCAL-ONLY) — decisions
session: s-22300 (Tranche B-3)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-52_lsp_integration.md
prior: workflow/reasoning/E-52-decisions.md (Tier-A + Tier-B)
date: 2026-05-31

================================================================
TRANCHE B-3 — NARROWED SCOPE PRODUCER + GATE SHIP
================================================================

(1) NARROWING — what shipped vs deferred

SHIPPED in this tranche (S-1..S-7):
  S-1  crates/orchestrator/src/diagnostic_probe.rs (NEW, ~400 LOC)
       Public: DiagnosticProbeRequest, ENABLE_ENV, lsp_diagnostic_probe_enabled_from_env,
       probe_kind_for_language, canonical_binary_for_language,
       dispatch_diagnostic_probe (async), notify_diagnostic_probe (async),
       DiagnosticProbeBeacon (3-arm), synthetic_error_outcome.
  S-2  crates/orchestrator/src/lib.rs `pub mod diagnostic_probe;`.
  S-3  crates/orchestrator/src/evidence.rs
       - TransitionContext::diagnostic_probe_outcome: Option<DiagnosticProbeOutcome>
         (back-compat default None).
       - struct DiagnosticProbeGate; impl EvidenceGate (slot 9 in all_gates()).
       - test_gated_transitions_have_gate: (Executing, Auditing) 1 -> 2.
       - test_all_gates_reject_empty_context: skip diagnostic_probe (None-Ok).
       - test_gate_names_unique: total 8 -> 9.
  S-4  crates/orchestrator/src/task_runner.rs:11744 region
       After emit_tool_dispatch in Ok(results) branch, dispatch_diagnostic_probe
       per changed file, then notify Dispatched + Completed beacons. Guarded
       by lsp_diagnostic_probe_enabled_from_env so production stays wire-dark.
  S-5  crates/contracts/tests/e52_diagnostic_probe_contract.rs (NEW, 6 tests):
       None-Ok, clean-Ok, Error+Completed-Err, Error+ToolMissing-Ok,
       Error+TimedOut-Ok, registry-presence.

DEFERRED (with explicit stub rows in workflow/sessions/s-22300/stubs.md):
  E-52-multilspy  : .elai_cc/python/elai_inference/lsp_oracle.py sidecar —
                    needs multilspy pypi anti-slopsquat verify + user dep confirm.
  E-52-tsmorph    : .elai_cc/crates/dashboard-v2/tools/check_ts_bindings.mjs —
                    needs ts-morph npm anti-slopsquat verify + user dep confirm.
  E-52-ScipRefresh: rust-side ScipRefreshCompleted producer — no rust caller
                    exists; codebase_mapper.py owns SCIP path; defer until a
                    rust wrapper materializes.
  E-52-S5         : `elai lsp-verify <file>` CLI subcommand. CliAction enum +
                    args.rs + structured-output-cmd.rs + run_subcommand.rs
                    dispatch arms exceed the 80-LOC trivial threshold. Wire-
                    dark gate semantics already cover the EXECUTING->AUDITING
                    pipe; CLI subcommand is operator convenience only. Will
                    land in a follow-on patch-N if user requests an out-of-FSM
                    invocation path.

(2) PATTERN-2 REUSE — no SCHEMA bump

The 4 already-shipped E-52 SSB variants (DiagnosticProbeDispatched, _Completed,
_Block, ScipRefreshCompleted at ssb.rs:6400+ / 12330+ / 19553+ / 26690+ /
30303+) absorbed the substrate during prior Tier-B work (SCHEMA_VERSION 139 at
the time; today 192 has carried through additional bumps). Tranche B-3 emits
into the existing variants — NO additional variant added, NO SCHEMA bump
required. This is the canonical Pattern-2 reuse: producer ships against an
already-locked substrate.

NOT-EMITTED in B-3 (deferred to atomic-bump tranche): lsp_server_spawned /
lsp_server_idle_shutdown / lsp_server_ram_ceiling_exceeded (multilspy
lifecycle) and ts_bindings_drift_detected. These ride atomically with
E-52-multilspy + E-52-tsmorph deferred items.

(3) HONEST LOCAL-ONLY TOOLMISSING FAST-PATH

LOCAL-ONLY semantics in this tranche:
  - When wire-dark (ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE != "1"):
    dispatch_diagnostic_probe returns a Completed-with-zero outcome
    immediately. Zero SSB rows, zero overhead.
  - When enabled:
    1. probe_kind_for_language maps the language slug -> ProbeKind.
       Unknown slugs fall through to GenericLsp.
    2. canonical_binary_for_language maps slug -> binary name
       (rust-analyzer / pyright / tsc / svelte-check). None -> ToolMissing.
    3. binary_on_path shells out to `which` (unix) / `where` (windows) inside
       a tokio::task::spawn_blocking guarded by tokio::time::timeout. Any
       failure mode (exit non-zero, join error, IO error) collapses to false.
    4. Binary missing -> ToolMissing outcome (gate soft-passes).
    5. Binary present -> Completed-with-zero-diagnostics + a tracing::warn
       that real subprocess invocation is deferred. NO fake diagnostics, NO
       optimistic clean signal — only the wire-shape proof.

This is the honest path E-52-decisions.md (c)/(d) demanded: ToolMissing is
NOT auto-pass, it is a soft-pass that the auditor (downstream) will surface
to the user. Real subprocess invocation is the only addition Tranche B-4
needs to land before production probe coverage activates.

(4) DIAGNOSTIC_PROBE_GATE AT SLOT 9 + transition collision rationale

  Slot 0  KnowledgeReadinessGate    (Idle -> Planning)
  Slot 1  GroundingSweepGate        (Planning -> TestingFirst)
  Slot 2  TestFailReasonsGate       (TestingFirst -> Executing)
  Slot 3  TestFlippedGate           (Executing -> Auditing) -- co-fire
  Slot 4  ReplanFastTrackGate       (Executing -> Planning)
  Slot 5  ZeroBlocksGate            (Auditing -> Memorizing)
  Slot 6  DkgChangelogGate          (Memorizing -> Planning / Complete)
  Slot 7  IntentPairsGate           (Planning -> TestingFirst) -- co-fire
  Slot 8  DiagnosticProbeGate       (Executing -> Auditing)   -- co-fire NEW

Order-stable append. gates_for_transition() filters by transition() pair so
slot order does NOT impact gate routing. The order-stable append rule exists
solely so any future test that indexes into all_gates() by position
keeps working. The Executing -> Auditing pair now resolves to a 2-gate vec
(TestFlipped + DiagnosticProbe); short-circuit semantics preserve TestFlipped
priority because TestFlipped sits earlier in all_gates() and check_transition
iterates in order.

(5) task_runner.rs INSERTION POINT (E-52-S3) AND CONTEXT THREADING TODO

Inserted in run_validate_dispatch at the Ok(results) branch, AFTER
emit_tool_dispatch, BEFORE emit_daira_trace Return. Per-changed-file loop:
  - dispatch_diagnostic_probe (wire-dark guarded inline).
  - notify_diagnostic_probe Dispatched.
  - notify_diagnostic_probe Completed.

NOT done in this tranche:
  - Worst-status outcome aggregation into TransitionContext at the
    EXECUTING -> AUDITING transition site. run_validate_dispatch does not own
    the FSM transition context — the orchestrator FSM driver does. The
    producer surface is in place; the consumer threading is a small follow-on
    edit at the FSM driver site (which calls check_transition).
  - DiagnosticProbeBlock beacon emission. Errors-count > 0 path is rare in
    LOCAL-ONLY mode (only ToolMissing / Completed-clean reachable today). When
    the real subprocess landing flips this path live, the Block beacon emit
    rides on top of the gate's check failing — a 3-line addition.

Both gaps marked with TODO[E-52-S3] in the code so the next tranche has a
clean grep anchor.

(6) CONTRACT TEST MIRROR

mirror selected = crates/contracts/tests/ambiguity_scorer_contract.rs lines
256-336 (TransitionContext { field: Some(...), ..Default::default() } builder
pattern + unwrap_err() + assertion on missing_artifacts / suggestion content).
NOT used:
  - crates/contracts/tests/diagnostic_envelope_contract.rs — that's the
    Tier-A envelope contract (serde + LSP scale + 1-indexed line/column +
    is_clean-error-only). The Tranche B-3 contract is gate-shape, not
    envelope-shape — different concerns, no overlap.
  - crates/contracts/tests/e52_lsp_integration_contract.rs — that's the
    JSON-RPC framing + SSB round-trip contract. Wire shape already locked.

(7) DRIFT FOLLOW-UP

E-52-decisions.md DRIFT-2 listed task_runner.rs line positions 7210 / 8357 /
8385 / 8444. Today's positions are 11615 (emit_tool_dispatch), 11643
(emit_validation_tier), 11702 (run_validate_dispatch entry), 11744
(emit_tool_dispatch call site inside Ok branch). DRIFT-2 re-drifted by
~3300 lines since the Tier-B decisions doc was last refreshed. Logging here
so the next E-52 follow-on does not re-search.

(8) ENV GATES + WIRE-DARK DEFAULTS

  ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE  -- "1" = enable probe dispatch;
                                       unset / any other value = wire-dark.
                                       Default OFF. Honoured by:
                                       - diagnostic_probe::lsp_diagnostic_probe_enabled_from_env
                                       - task_runner::run_validate_dispatch
                                         (inline guard).
  ELAI_LSP_MODE                     -- already wired by E-40 oracle path
                                       at symbol_graph/lsp_oracle.rs:124.
                                       NOT consulted by E-52 producer (E-40 +
                                       E-52 are independent surfaces).

(9) DELIBERATE NON-ACTIONS

  - No new Cargo dep (blake3 + tokio + tracing all already in workspace).
  - No new pyproject dep.
  - No new package.json dep.
  - No SCHEMA bump.
  - No svelte-check / cargo test / cargo run by the executor (auditor only).
  - No CLI subcommand (deferred S-5).
  - No DiagnosticProbePanel.svelte (producer must prove wire before dashboard
    consumer lands; deferred per E-52-decisions.md DRIFT-5).
  - No auditor_mode.rs / deterministic_auditor.rs consumer plumbing (the gate
    blocks first; auditor consumer follows; deferred per DRIFT-3).
  - No .unwrap() in lib code. No Box<dyn Error>. blake3 used (never blake2 /
    sha2). thiserror / anyhow already in the workspace; gate returns the
    existing EvidenceError type.

END E-52 Tranche B-3 decisions.
