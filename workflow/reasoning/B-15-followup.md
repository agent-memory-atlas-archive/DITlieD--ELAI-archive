B-15 Phase 0 Debt Burndown -- Followup (regression + out-of-scope items)
==========================================================================

Session: s-25120 (completion-drive 1) closed B-15 against the v2 shipped
state and rebaselined the plan. This file captures the residual items
that COULD NOT be closed in B-15's scope and routes them to follow-up
phases.

Why a followup file
--------------------

B-15 was a debt-burndown plan written against the 2026-04-24 audit
snapshot. Between then and 2026-05-22, six follow-on plans (B-4, B-7,
B-9, B-27, D-* dashboard work) shipped large producer-wire surfaces
that re-introduced determinism leaks faster than B-15 could close them.
The right move is not to expand B-15's scope mid-flight (that would
violate the per-conversion-rationale rule), but to route the regression
to a dedicated burndown phase with fresh audit + per-site decision.

(1) Entropy quarantine regression: B-15-regression-burndown
============================================================

Scope: 82 production .rs files in .elai_cc/crates/ contain
SystemTime::now() or Instant::now() calls outside core/src/clock.rs and
the tests/ tree. Total occurrences: 192 (counted 2026-05-22 in s-25120).

Comparison points:
  - 2026-04-24 audit baseline (phase-0.md):     40+ files (claim was 40+, not measured exactly)
  - 2026-04-29 B-15C-GAPS handoff:              28 residual (after deferring 23 HIGH-risk)
  - 2026-05-08 B-15 batch G close:              16 residual (post script-tightening)
  - 2026-05-22 this completion-drive:           82 files / 192 occurrences

Net regression vs the post-script-tightening floor: +66 files in 14 days.
That's ~4.7 new leak sites per day, all from new producer wires.

Root cause analysis (per cross-check + this session's grep):

  B-4 wires (research_tree producer):
    - orchestrator/research_tree.rs (Instant::now in scheduler tick)
    - orchestrator/intent_pair.rs (timestamp on every emit)
    - orchestrator/empire/factory_line.rs (turn timestamps)

  B-7 wires (ssb + navigator producer):
    - orchestrator/ssb.rs (per-row timestamp)
    - orchestrator/session_dag.rs (per-snapshot timestamp)
    - orchestrator/session_control.rs (control-frame timestamps)

  D-* dashboard wires (tauri command surface):
    - dashboard-v2/src-tauri/src/commands.rs (per-command latency)
    - dashboard-v2/src-tauri/src/remote_sidecar_helpers.rs (RTT)

  E-* / general producers:
    - web_search/src/obscura_fetcher.rs (request RTT measurement)
    - orchestrator/sandbox.rs / sandbox/firecracker.rs (snapshot age)
    - orchestrator/task_runner.rs (task spawn + duration timers)
    - reasoning/src/chat_memory_reflect.rs (reflect duration)
    - knowledge/src/cosmos_worker.rs / startup.rs (boot timestamps)
    - knowledge/src/symbol_graph/build_pipeline.rs (build-step duration)

The fix shape per producer:
  - Constructor accepts Arc<dyn ClockProvider>
  - Direct SystemTime::now() / Instant::now() calls -> self.clock.system_now()
    / self.clock.instant_now()
  - Test sites get a SeededClock (already shipped at core/src/clock.rs:122)
    so deterministic snapshot ordering across runs is restored.

Recommended plan structure for B-15-regression-burndown:

  Phase 1 -- audit: re-run check_entropy_quarantine.sh; produce per-crate
  leak list; group by producer wire (B-4 / B-7 / D-* / general).

  Phase 2 -- per-crate threading: dispatch one fixer agent per crate
  with a per-site decision matrix (constructor inject vs pure-function
  refactor vs accept-as-cfg-test). Cascade rule: cfg(test) and prod
  migrate atomically.

  Phase 3 -- contract tests: add deterministic-build contract tests
  per producer that pin output byte-equality across two runs with the
  same SeededClock seed.

  Phase 4 -- ban tightening: update check_entropy_quarantine.sh allow-list
  + remove the documentation-only entries it currently carries.

Owner: a new B-15-regression-burndown plan (not yet authored). Estimated
effort: 3-5 sessions given the 82-file scope and the need for per-site
rationale.

Out-of-scope here: this followup ONLY records the entropy regression. It
does NOT cover any other producer wire that emits its own clock surface
(e.g. AnimationClock at media_validator/render_adapters/animation.rs,
which is a domain clock, not an entropy leak).

(2) HashMap ban residuals: dashboard-v2 + commands_d26
========================================================

Five production HashMap violations remain after this session, all in
dashboard-v2 src-tauri command modules that coordinate with B-3 / B-4 /
B-7 ships and were explicitly excluded from B-15 completion-drive scope:

  - dashboard-v2/src-tauri/src/commands_d6.rs:22   use std::collections::HashMap
  - dashboard-v2/src-tauri/src/commands_d7.rs:20   use std::collections::HashMap
  - dashboard-v2/src-tauri/src/commands_d9.rs:19   use std::collections::HashMap
  - dashboard-v2/src-tauri/src/commands_e46.rs:21  use std::collections::HashMap
  - dashboard-v2/src-tauri/src/commands_d26.rs:141 std::collections::HashMap::with_capacity

Per executor instructions: "DO NOT touch dashboard-v2 Tauri command
files (commands_d6/d7/d9/e46) -- those are coordinated with B-3/B-4/B-7
ships." commands_d26.rs is in the same coordinate-with-dashboard bucket.

Recommended treatment: tighten check_hashmap_ban.sh to FAIL these in CI
(remove any dashboard-v2 exception) AFTER B-3/B-4/B-7 dashboard work
quiesces. Until then, allow-list by file basename (NOT cargo-warn-only,
because that would silently regress).

(3) Dashboard-v2 frontend TypeScript residuals
================================================

Out of Rust ban scope but worth tracking:

  - dashboard-v2/src/lib/api/mock-data.ts        SystemTime equivalent
                                                  (Date.now() leaks)
  - dashboard-v2/src/lib/api/stub.ts             similar
  - dashboard-v2/src-tauri/src/commands.rs       655, 3110, 3624

A TypeScript-side determinism audit is a separate workstream; the Rust
ban script does not cover TS. Track in a future dashboard-determinism plan.

(4) HashSet ban (not yet enforced)
====================================

Plan B-15 mentioned "84 std::collections::HashMap imports + 10 HashSet"
in the original audit baseline. The HashSet half was never enforced;
no check_hashset_ban.sh exists.

Current state (2026-05-22 grep):
  - 10+ production files import std::collections::HashSet
  - Same iteration-order non-determinism risk as HashMap for any
    HashSet that gets serialized or compared by hash for content
    addressing.

Recommended treatment: extend check_hashmap_ban.sh to also scan for
HashSet (one-line change: add `use std::collections::HashSet` to the
scan loop with the same allow-list mechanism). Per-file decision:
BTreeSet (deterministic iteration) vs FxHashSet (lookup-only hot path).

(5) cli/main.rs size deferral acceptance
==========================================

cli/main.rs sits at 555 LOC vs the spec's 500 LOC target. B-15
completion-drive accepted this as shipped (see B-15-decisions.md
section (e)). No follow-up needed unless the file grows further.

Tripwire: if cli/main.rs >= 600 LOC at any future grep, that's a real
regression and warrants a B-15-cli-shrink follow-up.

(6) C-15 closure-drive snapshot (s-25120, 2026-05-23)
=======================================================

C-15 completion-drive (session s-25120, executor pass) sharply scoped a
final batch of cli + knowledge crate closures and rebaselined the plan
rather than expanding it. Full per-decision rationale in
F:\.ELAI_workflow\workflow\reasoning\C-15-decisions.md "C-15 CLOSURE-DRIVE
SESSION (s-25120)" section.

Measurement snapshot (after closure batch):

  Entropy quarantine : 73 -> 51 leak sites (-22 net)
    - cli/src/measure.rs       : 16 -> 0 (module-scope allow + script glob)
    - cli/src/bench_arc.rs     :  1 -> 0 (script glob `**/bench_*.rs`)
    - cli/src/bench_conformal.rs:  1 -> 0 (script glob `**/bench_*.rs`)
    - cli/src/live_cli.rs      :  2 -> 0 (1 inject self.clock + 1 entropy-allow)
    - knowledge/src/startup.rs :  1 -> 0 (entropy-allow on cfg(test) TimeoutClock)
    - knowledge/src/cosmos_worker.rs: 1 -> 0 (entropy-allow + future trait ext)

  HashMap ban         : 5 violations (unchanged; dashboard-v2 carve-out per
                        section (2) above)

  Residual leak distribution (51 across 22 files; all routed here):
    9  orchestrator/src/compiled_pipeline.rs
    6  tools/src/adaptive_tool.rs
    4  orchestrator/src/sandbox/firecracker.rs
    4  dashboard-v2/src-tauri/src/commands.rs
    3  orchestrator/src/wiring_contract.rs
    3  orchestrator/src/task_runner.rs
    3  orchestrator/src/metamorphic_testing.rs
    2  plugins/src/mcp_schema_pin.rs
    2  orchestrator/src/locked_keys_auditor.rs
    1 each across: safety/trust_dialog, reasoning/chat_memory_reflect,
       plugins/mcp/lazy, orchestrator/{surgical_decomposer, reasoning_template,
       observable_effect, intent_pair, held_out, empire/factory_line,
       determinism, checker_registry, bai, background_task},
       memory/{notes, fact}

Net regression vs the 18-leak floor from C-15 original (s-32952, 2026-05-18):
+33 leaks in 5 days. Continues the producer-wire regression rate trend
noted in section (1) above. B-15-regression-burndown remains the right
home for sustained closure.

Tripwire: if entropy leak count rises above 75 at any future scan, that
is a regression beyond the s-25120 closure baseline and warrants opening
B-15-regression-burndown immediately.

End of followup doc.
