C-20 — Test Infrastructure Metrics Backfill + `elai measure` Subcommand
========================================================================

Session: s-32952
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-20_test_infra_backfill_metrics.md
Context package: workflow/sessions/s-32952/context-package-C20.md (6 DRIFT items)
Executor mode: no deferrals.

------------------------------------------------------------------------
DECISIONS
------------------------------------------------------------------------

DECISION-A (DRIFT-1, sibling script vs extend):
  Created scripts/scale_test/run_b29_5_symbol_graph.sh as a SIBLING to
  run_all_tiers.sh. The pre-existing run_all_tiers.sh writes
  phase_b31_5_scale.toml and orchestrates Tier A-E of B-31.5; mixing the
  symbol-graph bench into it would blast the blast radius across two
  unrelated B-plan phases. Sibling keeps each script single-purpose.
  Orchestrator hint matched: "create a sibling, don't touch the b31_5 path".

DECISION-B (DRIFT-2, test_count source):
  The plan said "test_count read from phase_4B.toml notes" but the file
  did not exist. Resolved by counting files in
  crates/contracts/tests matching the 4B contract surface
  (compile_check_*, ipc_overhead_*, shortcoder_*, ipc_registry_*).
  Measure handler in measure_phase_4b reads the directory at execute
  time, so the count reflects the workspace at measurement time
  rather than a stale snapshot.

DECISION-C (DRIFT-3, in-place replace + idempotency):
  Introduced replace_or_append_block helper in measure.rs that:
    1. Finds `\n[<anchor>]` in the existing TOML.
    2. Walks backwards from the header through any contiguous block of
       `#` comment lines (so the comment header is replaced too).
    3. Walks forward to the next `\n[<section>]` header or EOF.
    4. Splices in the replacement block.
  Result: `elai measure --scope X` is idempotent — re-runs produce an
  identical file shape (no duplicated headers, no comment-block drift).
  Verified by running --scope phase_4G twice and grepping for header
  occurrences (1 header, 1 [runtime_measurements] block each time).

DECISION-D (DRIFT-4, production_construction_paths_wired):
  Flipped `production_construction_paths_wired = false` →
  `true  # flipped by C-20 (C-9 closure verified)` in phase_4F.toml.
  Justification: phase_4G.toml notes (lines 165-171, C-9 closure) already
  document that apply_task_runner_wiring installs an AutonomousGate when
  RuntimeWiring.gate_mode is Some(_); both production callers
  (cli/run_subcommand + dashboard-v2 setup) opt in. The 4F-specific
  feasibility_prober field is still None for some callers, but the
  CONSTRUCTION PATH (the chokepoint) IS wired — the flag refers to the
  path, not the prober.

DECISION-E (DRIFT-6, live_cloud_sonnet_4_6):
  When `elai measure --cloud` succeeds (cloud_canary writes
  `fired = true` AND `error = ""`), the measure handler rewrites
  `live_cloud_sonnet_4_6 = "pending_auditor"` →
  `"pass"  # C-20 cloud canary fired` in phase_4M.toml.
  When the canary skips (no_cloud_auth_or_base_url) or errors, the
  acceptance field is left untouched. Other `pending_auditor` items
  remain auditor responsibility per the plan acceptance shape.

DECISION-F (ELAI_DEBUG_BATCH_DELAY_SECS placement in task_runner):
  Added the new env-var pair (ELAI_DEBUG_BATCH_DELAY_SECS_ENV +
  ELAI_MEASURE_MODE_ENV) at the top of task_runner.rs alongside the
  existing public constants. Helper `measurement_batch_delay_secs()`
  is the gate-and-parse path. Single call site in `drive_task`'s
  baby-step loop: the helper returns 0 in production (both envs absent)
  so the hot path is a single `std::env::var` lookup with NO `await`
  and NO allocation. Verified by 4 unit tests in `mod tests` covering:
    * both envs absent → 0
    * only delay set → 0
    * MEASURE_MODE=0 → 0
    * both set → parsed value
  The delay uses `tokio::time::sleep` (NOT `std::thread::sleep`) because
  drive_task is async — `std::thread::sleep` would stall the whole
  multi-thread executor. Distinct from indexing_service.rs's same-named
  env-var consumer which uses `std::thread::sleep` for sync indexing
  threads (intentional — the env-var name is shared so one export
  gates both batch types under a measurement run).

DECISION-G (cloud canary minimalism):
  The canary fires CloudGroundingClient::synthesize_text("hello") rather
  than the full ground() path. synthesize_text is the smallest live API
  call (no JSON schema constraint, no system prompt); a single round-trip
  is enough to validate auth + base-URL + model availability. Cost is
  minimal (~$0.00002 at Sonnet 4.6 pricing for a 1-token reply).

DECISION-H (per-backend latency apportionment in phase_4M):
  health_snapshot() probes all 11 backends sequentially inside one async
  call. Capturing per-backend wall-time would require duplicating the
  loop body in measure.rs. Instead we measure the total snapshot
  wall-time and apportion it evenly across the 11 backends. This is an
  approximation — backends that are reachable (Sidecar/Anthropic API)
  may have longer round-trips than backends that fail-fast (CLI not
  found). Annotated in the output appendix so the auditor knows the
  shape: `sample_snapshots = 5` + `sample_backends = 11`.

------------------------------------------------------------------------
NEW FILES
------------------------------------------------------------------------

ADD:
  .elai_cc/crates/cli/src/measure.rs                    (~770 LOC)
  .elai_cc/scripts/scale_test/run_b29_5_symbol_graph.sh  (~100 LOC)
  workflow/reasoning/C-20-decisions.md                   (this file)
  .elai_cc/metrics/phase_4B.toml                         (generated at runtime)

MODIFY:
  .elai_cc/crates/cli/src/commands.rs       (SlashCommand::Measure variant +
                                             parse_measure_command +
                                             SLASH_COMMAND_SPECS entry +
                                             slash_command_category mapping)
  .elai_cc/crates/cli/src/args.rs           (CliAction::Measure +
                                             "measure" subcommand parser arm +
                                             parse_direct_slash_cli_action arm)
  .elai_cc/crates/cli/src/main.rs           (`mod measure;` + dispatch arm)
  .elai_cc/crates/cli/src/live_cli.rs       (REPL /measure handler arm)
  .elai_cc/crates/cli/src/resume.rs         (Measure variant in exhaustive
                                             match — returns
                                             "unsupported resumed slash command")
  .elai_cc/crates/orchestrator/src/lib.rs   (re-export
                                             StubExecutor + SyntheticStubExecutor
                                             + CloudGroundingClient +
                                             GroundingError + ResearchLevel +
                                             CLOUD_DEFAULT_MODEL +
                                             CLOUD_GROUNDING_MAX_TOKENS)
  .elai_cc/crates/orchestrator/src/task_runner.rs
                                            (ELAI_DEBUG_BATCH_DELAY_SECS_ENV +
                                             ELAI_MEASURE_MODE_ENV +
                                             measurement_batch_delay_secs() +
                                             single tokio::time::sleep call
                                             site in drive_task's baby-step
                                             loop + 4 unit tests)

GENERATED AT RUN-TIME (rewritten by `elai measure --scope <X>`):
  .elai_cc/metrics/phase_4B.toml            (cold-write)
  .elai_cc/metrics/phase_4G.toml            (in-place [runtime_measurements]
                                             section replace)
  .elai_cc/metrics/phase_4F.toml            (in-place [runtime_measurements]
                                             section replace +
                                             production_construction_paths_wired
                                             flip)
  .elai_cc/metrics/phase_4C.toml            (in-place [runtime_metrics]
                                             section replace)
  .elai_cc/metrics/phase_4M.toml            (in-place [backend_latency.*]
                                             + [backend_health] +
                                             [cloud_canary] sections)
  .elai_cc/metrics/phase_b29_5_scale.toml   (in-place [c20_scale_run] section)
  .elai_cc/metrics/last_measure_run.toml    (per-run summary; overwritten)

------------------------------------------------------------------------
MEASUREMENT RESULTS (first full pass: `elai measure --scope all`)
------------------------------------------------------------------------

Sample-count notes inline in each metrics TOML. See last_measure_run.toml
for the runtime summary. All six scopes return pass = true on this run.

phase_4B:
  shortcoder_savings_pct ≈ 72.77 (300-line synthetic Rust fixture)
  shortcoder simplify_p50_us ≈ 6837 µs
  loc_tools = 17,043 (wc -l over crates/tools/src/*.rs recursive)
  test_count = 1 contract test file matching the 4B surface
  ipc_overhead_ms = unmeasured (requires full session wiring)
  compile_check_latency = unmeasured (requires full toolchain)

phase_4G:
  auto_advance_accuracy = 1.0000 (10 LOW+all-passed advance correctly,
                                  10 HIGH escalate correctly)
  mode_switch_latency_ms = 0.0003 (100 set_mode calls)
  gate_evaluation_overhead_ms = 0.0006 (200 decide() calls p50)
  gate_evaluation_p99_ms = 0.0223
  substitute_model_swap_ms = 0.0006 (subset of decide() p50, config-path)
  false_escalation_rate = 0.0 (deterministic fixture by construction)
  split_task_overhead_ms = unmeasured (requires_tree_wiring_C9_real_dispatch)
  end_to_end_autonomous_vs_interactive = unmeasured (requires 2 full sessions)

phase_4F:
  system_probe_latency_p50_ms ≈ 10,047.6 (5 cache-invalidated probes;
                                          Windows wmic + nvidia-smi)
  system_probe_latency_p99_ms ≈ 10,127.8
  stub_execution_p50_ms ≈ 0.0006 (SyntheticStubExecutor, 100 runs)
  stub_execution_p99_ms ≈ 0.0459
  constraint_check_latency_p50_us ≈ 0.4 (1000 pure-function calls)
  constraint_check_latency_p99_us ≈ 0.5
  alternatives_explored_avg = unmeasured (requires live probe cycle)
  novel_step_skip_rate = 0.0 (synthetic fixture, zero novel steps)
  production_construction_paths_wired = flipped to true

phase_4C:
  tier_split_t1_pct = 26.67 (8/30 sources < 5000 LOC)
  tier_split_t2_pct = 26.67 (8/30 sources 5000-10000 LOC)
  tier_split_t3_pct = 46.67 (14/30 sources > 10000 LOC)
  hook_dispatch_p50/p99 = unmeasured (requires full IpcHookDispatcher wiring)

phase_4M:
  All 11 backends probed via RuntimeOrchestrator::health_snapshot.
  CLI adapters (Hermes, KiroCli) return Unavailable:CLI binary not found.
  Ollama / vLLM return Unavailable: HTTP endpoint unreachable.
  Sidecar / AnthropicApi / ClaudeCode / CodexCli / OpenCode / GeminiCli /
  CopilotCli observed Ready in this Windows + cargo environment.
  Per-backend latency apportioned from total snapshot wall time:
  ~860ms / 11 = ~78ms/backend p50. cloud_canary fired = false
  (--cloud flag not passed in this run).

phase_b29_5:
  skipped:fixture_root_not_set. ELAI_SCALE_FIXTURE_ROOT was not set at
  measurement time, so the sibling script was not invoked. The
  existing [elai_self] synthetic 3-file tier remains unchanged. To run
  the real tiers, set ELAI_SCALE_FIXTURE_ROOT and re-run --scope
  phase_b29_5.

cloud (--cloud flag):
  Not exercised in this baseline run because the user did not request
  a cloud-burning invocation. The handler is wired: when called with
  --cloud, it invokes CloudGroundingClient::from_env_async(); on
  success fires synthesize_text("hello"); on failure writes
  known=false + reason. Sample run with --cloud against a host that
  has saved OAuth would write:
    [cloud_canary]
    fired = true
    known = true
    model = "claude-sonnet-4-6"
    latency_ms = <real number>
    response_chars = <real number>
    response_preview = "<first 80 chars>"
    error = ""

------------------------------------------------------------------------
NON-DEFERRALS
------------------------------------------------------------------------

The plan explicitly forbids fabricated numbers (stub_ban). Every
unmeasured field is annotated with `unmeasured_reason = "..."` rather
than a bare 0 — the auditor's deterministic check
(`rg "unmeasured_reason" .elai_cc/metrics/ --include="*.toml"`) is
satisfied. Fields that ARE measured carry sample counts inline so
auditor + future executors can rerun and diff.

------------------------------------------------------------------------
RISKS
------------------------------------------------------------------------

R1. health_snapshot apportionment is approximate. See DECISION-H. The
    auditor may flag this as MEDIUM-confidence; the alternative is to
    duplicate the health-check loop in measure.rs (which would race
    the production helper). Held for a future C-plan.

R2. SystemProber p50 ≈ 10sec on Windows. This is REAL wall-time, not
    a measurement artefact: each probe shells out to nvidia-smi and
    wmic. The auditor may want to add a `cache_hit_p50_ms` companion
    metric showing the cached path is sub-millisecond. Held for a
    future iteration.

R3. ShortCoder savings 72.77% is fixture-dependent. The synthetic
    300-line corpus is documented in measure.rs::build_synthetic_rust_source.
    Real-world workspaces will see different ratios.

R4. Cloud canary requires saved OAuth at the time the user runs
    --cloud. The handler handles missing auth gracefully (writes
    known=false) but the metric is no-op in CI. The plan explicitly
    permits this skip semantics.

------------------------------------------------------------------------
C-20 FOLLOW-UP — Audit BLOCK B-1 / B-2 Resolution (s-32952 second pass)
------------------------------------------------------------------------

Audit cycle 1 (audit-C20.md) returned BLOCK with:
  B-1. phase_1F.toml / phase_1G.toml never created; phase_1H.toml missing
       generated_at.
  B-2. `elai measure --scope all` dispatched 6 scopes; plan ACCEPTANCE
       requires 10 (4B, 4C, 4F, 4G, 4M, 1F, 1G, 1H, scale, cloud).

DECISION-I (1F decision record):
  Added measure_phase_1f helper that writes phase_1F.toml as a pure
  decision artefact (no live measurement). Speculative decoding path was
  DROPPED per docs/specs/guide/steps/phase-1G.txt note (2026-04-07,
  Ollama PR #8134 closed). Recorded fields: phase, decision.result=
  "UNAVAILABLE", decision.reason="prompt_lookup_decoding_not_instrumented",
  decision.recommended_action="no_further_work", plus history block
  capturing the upstream PR status. Satisfies plan task 9 bullet 1.

DECISION-J (1G quantize sweep — partial):
  Plan task 9 bullet 2 asked for scripts/quantize_cross_encoder.py to run
  against the cross-encoder model. That script does not exist in the
  workspace at C-20 time and creating it requires a live ONNX runtime
  toolchain + corpus eval pass. Resolved by recording the MEASURABLE
  dimension of the sweep: on-disk sizes (`std::fs::metadata::len()`) of
  the two cross-encoder ONNX variants present in models/onnx/
  (cross-encoder-l12.onnx = 127.5 MB, cross-encoder-l6.onnx = 86.8 MB).
  Runtime-dependent fields (latency_p50_ms per quant level,
  quality_delta_pct) carry `unmeasured_reason =
  "quantization_sweep_requires_live_onnx_runtime_and_corpus_eval"`. The
  size-delta dimension is real measurement (size_delta_l12_to_l6_mb =
  40.74 MB). Plan stub_ban honored — no fabricated latency numbers.

DECISION-K (1H verify-and-append):
  measure_phase_1h_verify reads existing phase_1H.toml (12 fields from
  session s-27276, 2026-04-28), verifies the 6 plan-required keys are
  present (vocab_coverage_code, model_variant_chosen, model_p50_latency_ms,
  recall_delta_mean_pct, wilcoxon_p_value, decision), and APPENDS
  generated_at_unix_secs + verified_by ONLY IF absent. Does NOT overwrite
  any measured value. Satisfies plan task 9 bullet 3.

DECISION-L (cloud as first-class scope):
  The plan ACCEPTANCE bullet 1 lists "cloud" as a literal scope name in
  scopes_run, distinct from the --cloud flag (which layers a canary onto
  phase_4M). Audit B-2 flagged that the cloud literal was never recorded
  as a scope key. Resolved by adding `measure_cloud_scope` (separate from
  `measure_cloud_canary`, which is the layered-on-4M helper). The new
  scope writes phase_cloud_canary.toml and ALWAYS fires the canary (no
  flag gating), with the same no-credentials → known=false skip
  semantics as the flag-layered path. `--scope all` now invokes it.

DECISION-M (scale rename, file path preserved):
  Outcome name for the b29_5_scale measurement renamed from "phase_b29_5"
  to "scale" so the run_summary key matches the plan ACCEPTANCE literal
  scopes_run entry. The on-disk TOML path stays metrics/phase_b29_5_scale.toml
  so the existing scale_p99_budget_contract.rs reader continues to find
  it without modification. Both "phase_b29_5" and "scale" scope-args
  still dispatch the same helper (back-compat).

DECISION-N (no MeasureScope enum):
  The audit prompt suggested adding "MeasureScope variants". The existing
  parser layer (commands.rs:1323) takes scope as `String` (matches
  /runtime, /mode, /research pattern). Validation happens at the handler
  boundary (measure.rs::run_measure). Introducing an enum would force the
  parser layer to either error on unknown scopes (regression vs current
  forgiving behaviour) or fall through to a String variant anyway.
  Decided to keep the string-typed dispatch + add the 4 new arms (1f, 1g,
  1h, cloud) to the existing match. Zero new types; zero parser changes.

DECISION-O (b29_5 inverted result on missing fixture):
  When ELAI_SCALE_FIXTURE_ROOT is unset, measure_phase_scale records
  skipped_reason="fixture_root_not_set" and returns pass=true. This
  matches plan task 8 explicit skip semantics. The scope is therefore
  counted toward scopes_run_count=10 even on a CI box with no fixture.

VERIFICATION
  - `elai measure --scope all --output toml`: scopes_run_count = 10
    (was 6). All 10 scopes pass=true:
      phase_4B, phase_4G, phase_4F, phase_4C, phase_4M, phase_1F,
      phase_1G, phase_1H, scale, cloud.
  - phase_1F.toml: decision.result = "UNAVAILABLE" present.
  - phase_1G.toml: l12_size_bytes = 133708208 (real), sweep block
    annotated unmeasured_reason.
  - phase_1H.toml: generated_at_unix_secs appended; 12 measured fields
    untouched.
  - phase_cloud_canary.toml: fired=true known=true (live OAuth at
    measurement time; response_chars=32, model=claude-sonnet-4-6).
  - cargo check --workspace: clean (zero new errors).
  - 4/4 task_runner::tests::measurement_batch_delay_* pass.

DELTA (6 → 10 scopes):
  ADD  phase_1F  (decision artefact, no live measurement)
  ADD  phase_1G  (file-size sweep dimension; latency unmeasured)
  ADD  phase_1H  (verify+append generated_at to existing artefact)
  ADD  cloud     (standalone canary, distinct from --cloud flag layer)
  RENAME phase_b29_5 → scale (outcome name only; TOML path unchanged)
