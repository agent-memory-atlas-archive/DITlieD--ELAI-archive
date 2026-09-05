K-2 routing-economics bench -- decisions (K-26 MOVE 6b build)

what built: crates/orchestrator/src/benchmarks/routing_economics_runner.rs
(fixed N=5 ordered fixture corpus, route_task drives the REAL
cloud_mode::CloudModeRouter, pure compose_routing_economics aggregation,
RunnerStatus{Measured,RunnerPresent} gates ONLY the live $ leg). CLI driver
crates/cli/src/bench_routing_economics.rs mirrors bench_gate_tax.rs
(write_json_report + emit_beacon_and_rra, Ssb::new inside rt.block_on).
Dispatcher arm in args_bench.rs (parse_bench_routing_economics_args,
mirrors gate-tax arm), CliAction::BenchRoutingEconomics in args.rs, main.rs
match arm + mod declaration, bench_cmd.rs thin wrapper. SSB three-file
lockstep: action_type.rs (+RoutingEconomicsMeasured variant, as_sql_str
"routing_economics_measured", from_str arm), payloads.rs (SsbPayload
variant + action_type() arm + RoutingEconomicsMeasuredPayload struct),
ssb.rs (SSB_ACTION_TYPES append, schema_version auto-derives to 273).
tests/action_type_lock.rs exhaustive match guard + golden file line
inserted alphabetically (manual insert, NOT run via `cargo test
regen_golden_file` -- Bash/cargo blocked in this FSM state; auditor must
verify the golden diff is exactly the one new row).
crates/contracts/tests/k2_routing_economics_contract.rs: 6 tests (fixture
mix sanity + the 5 plan-mandated a-e cases).

key decision -- hybrid as a call sequence, not a router field
(RECON_NEEDED-3, settled by the K-26 MOVE 6b integration explorer):
CloudModeRouter::route/route_with_spend take only (mode: ExecMode,
credentials, spend); there is no RoutingPolicy/context field distinguishing
force-cloud from hybrid. Force-cloud arm = route(ExecMode::Cloud) every
step (100% cloud, %offloaded==0 by construction). Hybrid arm =
route(ExecMode::Auto) every step first (the router's own documented
contract: Auto ALWAYS returns Local, cloud_mode.rs:317-321) then,
conditionally, escalate_hybrid_step_to_cloud(&router,...) when that task's
fixture-declared `local_succeeds == false`. This is the SAME two-call
protocol the production task_runner HYBRID loop uses. A single route(Auto)
vs route(Cloud) call per arm would make %offloaded always 0 for both arms
(ATTACK B, guarded by a strict-inequality test).

FORK-1 (SSB): +1 RoutingEconomicsMeasured flat beacon at schema_version
273 (auto-derived from SSB_ACTION_TYPES.len(), never hand-typed). Reuse of
RouteDecisionEvent (action_type.rs, SCHEMA_VERSION 191) was rejected: wrong
grain (per-single-route-call tracing log vs K-2's per-bench-run aggregate),
default-OFF kill-switch (ELAI_ROUTE_DECISION_SSB_ENABLE) vs K-2's
always-on posture. Mirrors GateTaxMeasured/UsageTurnEvent shape exactly
(flat, no Kind discriminator).

FORK-2 (live arm): no live cloud+local lane is funded this window (W-10
RunPod/remote-sidecar unfunded). live_lane_status() gates on
ELAI_ROUTING_ECONOMICS_LIVE_ENDPOINT env var presence -- deliberately
NEVER on J-28 UsageTurnEvent presence (always true once J-28 ships; gating
on it would fabricate a live $ off synthetic fixture tokens, ADVERSARIAL
attack A). Both branches of live_lane_status() currently return
live_cost_available=false (even when the env var IS set, no live query leg
against usage_ledger::rollup_usage_turns is wired yet -- that is a later
infra-gated MOVE per the plan's own "Next" section, not this build). The
fixture arms (force_cloud / hybrid, synthetic deterministic tokens priced
via cloud_mode's SHIPPED Sonnet pricing constants) ship unconditionally and
ARE the number this window.

FORK-3 (frontend): CLI-only this window (the number stays RunnerPresent
for the live leg; a dashboard panel would render a placeholder). Deferred
per the plan's grounded default.

per-anchor re-grep results (K-26 MOVE 6b context-package + integration
explorer, re-confirmed at edit):
- cloud_mode.rs:265/284 CloudModeRouter::route, :309 route_with_spend,
  :663 escalate_hybrid_step_to_cloud -- read in full, matches exploration.
- args_bench.rs: parse_bench_subcommand_args if-chain runs lines 29-95;
  gate-tax arm at :64-66; unknown-sub error list at :91-95 (the plan's
  ":52-93"/":93" line numbers were stale, confirmed by direct read, per
  context-package HIGH warning). routing-economics arm inserted directly
  before the gate-tax-adjacent chain entries and the error string updated.
- action_type.rs: max variant was UsageTurnEvent (TOTAL_VARIANTS 581,
  SCHEMA_VERSION 272); RoutingEconomicsMeasured appended as variant 582,
  SCHEMA_VERSION 273 (auto-derived, RECON_NEEDED-1 confirmed).
- action_type_lock.golden: alphabetical insertion point confirmed by
  manual byte comparison ("route_decision_event" < "routing_economics_
  measured" < "row_chain_sealed") and cross-checked against
  runtime_body_lines()'s plain `tags.sort()` (default Rust &str Ord,
  byte-wise) -- matches.
- benchmarks/mod.rs: `pub mod routing_economics_runner;` added (the plan's
  own Files-touched list omitted this registration; flagged by the
  integration explorer as GAP-1, fixed here).

deviation from the plan's literal Files-touched list: benchmarks/mod.rs was
NOT listed in the plan but IS required for the new module to compile into
the crate (confirmed by the K-26 MOVE 6b integration explorer, GAPS/RISKS
section). Added; no other file left the assigned scope.

self-verification: Bash/cargo is BLOCKED in the EXECUTING FSM state for
this session (confirmed by direct tool attempt). production_caller_check.py
and cargo test/check were NOT run by this executor -- they are UNVERIFIED
by construction and must be run off-agent by the auditor. Reasoned-through
wire-check expectation (not a substitute for the real run): fixture_corpus
and compose_routing_economics each have a same-file non-test caller
(run_routing_economics) -> expected verdict INTERNAL/WARN, mirroring
gate_tax_runner::build_coverage_table's identical shape, never DARK.
run_routing_economics has an external non-test caller in
bench_routing_economics.rs::run_inner -> expected verdict WIRED.
bench_routing_economics::run_inner is called only by run() in the same
file (mirrors bench_gate_tax.rs's identical run_inner shape) -> expected
INTERNAL/WARN, an already-accepted precedent pattern, not a new finding.
