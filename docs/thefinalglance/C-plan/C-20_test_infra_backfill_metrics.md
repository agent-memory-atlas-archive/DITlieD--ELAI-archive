# C-20: Test Infrastructure — Metrics TOML Backfill and Measurement Suite
Objective: populate all placeholder/zero metrics TOMLs with real measurements, wire the
ELAI_DEBUG_BATCH_DELAY_SECS measurement loop, and add an `elai measure` subcommand (or
equivalent driver) that executes the full measurement suite including a cloud Sonnet 4.6
live smoke run.
Estimated effort: 1-2 sessions.
Pre-requisites: C-9 (autonomous infrastructure wired), C-13 (IPC dispatcher production
swap), C-17 (measurement harness scaffolding present; assumed prior C-plan).
Parallel-safe with: C-18, C-19.

## Context

Seven phase_*.toml files exist but contain placeholder or zero values for runtime
measurements required by their respective specs. The verification reports confirm this
concretely:

- phase_4G.toml: tests/implementation/cli sections populated; the 7 runtime categories
  from spec §4G.6 (auto_advance_accuracy, false_escalation_rate, mode_switch_latency_ms,
  split_task_overhead_ms, substitute_model_swap_ms, end_to_end_pipeline_time,
  gate_evaluation_overhead_ms) are absent. Report: phase-4G.report.md §Deliverables PARTIAL.

- phase_4F.toml: system_probe_latency_ms, stub_execution_p50/p99, alternatives_explored_avg,
  novel_step_skip_rate all marked unmeasured=true. Six of nine spec metrics absent.
  Report: phase-4F.report.md §Deliverables PARTIAL.

- phase_4B.toml: file does not exist. Required keys per report §Deliverables MISSING:
  ipc_overhead_ms, compile_check_latency_{rust,python,typescript,go}_ms,
  shortcoder_savings_pct, test_count, loc_tools. 4B.5 MEASURE was explicitly deferred.

- phase_4M.toml: counts and acceptance present; per-backend p50/p99 latency, cost,
  fallback-fire rate, uptime % (spec §4M.6 telemetry) absent.
  Report: phase-4M.report.md §Summary.

- phase_4C.toml: qualitative notes only; 9 quantified metric categories from spec absent.
  Report: phase-4C.report.md §Summary.

- phase_b29_5_scale.toml: elai_self synthetic data present; linux/chromium/vscode/postgres
  tier entries absent. The four external-fixture tier runs have never been executed.
  Report: B-29.5_symbol_graph_scale_benchmark.report.md §Claims PARTIAL.

- phase_1F.toml: missing entirely; prompt-lookup decoding latency UNAVAILABLE note needed
  (1F used a different decoding path that was never instrumented).

- phase_1G.toml: missing; requires running scripts/quantize_cross_encoder.py + sweep.

- phase_1H.toml: mostly populated from DROP measurement (decision=DROP recorded);
  verify completeness and fill any remaining gaps.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4B.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4C.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4F.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4G.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4M.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-29.5_symbol_graph_scale_benchmark.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-31.5_resumable_indexing_scale_benchmark.report.md
- .elai_cc/metrics/phase_4G.toml (existing structure to preserve)
- .elai_cc/metrics/phase_4F.toml (existing structure to extend)
- .elai_cc/metrics/phase_4M.toml (existing structure to extend)
- .elai_cc/metrics/phase_4C.toml (existing structure to extend)
- .elai_cc/metrics/phase_b29_5_scale.toml (existing structure to extend)
- .elai_cc/metrics/phase_1H.toml (verify completeness)

## Tasks

1. **Add `elai measure` subcommand** to crates/cli/src/commands.rs.
   - SlashCommand::Measure { scope: MeasureScope } where MeasureScope: All, Phase(String),
     Scale, Cloud.
   - `elai measure --all` runs every measurement pass defined in tasks 2-9 in sequence.
   - `elai measure --phase 4G` runs only the 4G measurement pass.
   - `elai measure --scale` runs scripts/scale_test/run_all_tiers.sh via tokio::process.
   - `elai measure --cloud` runs the hybrid /task smoke + asserts 4M backend entries.
   - On completion writes a run summary to metrics/last_measure_run.toml
     (timestamp, scopes_run, files_written Vec<String>, pass/fail per file).

2. **ELAI_DEBUG_BATCH_DELAY_SECS measurement loop** in task_runner.rs.
   - Read env var once at startup; if set, inject a tokio::time::sleep of that many
     seconds between baby-step dispatch calls in the measurement-mode batch.
   - Purpose: slow down batch execution to allow resource probing at stable intervals.
   - Guard: only active when ELAI_MEASURE_MODE=1 AND ELAI_DEBUG_BATCH_DELAY_SECS is set.
   - Zero effect on production execution paths (env absent = no-op branch, zero overhead).
   - Add unit test: assert delay is applied when both vars set, skipped otherwise.

3. **Populate metrics/phase_4B.toml** (4B.5 MEASURE).
   - Micro-benchmark IpcToolRegistry::dispatch round-trip on 10 tool types via
     criterion or std::time::Instant (whichever matches existing bench style in
     crates/knowledge/benches/).
   - Measure compile_check_latency per language using existing CompileCheckHandler
     against known-good fixture files (one per language: rust, python, typescript, go).
   - Measure ShortCoder savings on a 300-line Rust file (R1-R10 rules, pct reduction).
   - Write: ipc_overhead_ms (p50, p99), compile_check_latency_{rust,python,typescript,go}_ms
     (p50, p99), shortcoder_savings_pct, test_count (read from phase_4B.toml notes),
     loc_tools (wc -l of crates/tools/src/*.rs).
   - File: .elai_cc/metrics/phase_4B.toml (CREATE; does not exist).

4. **Populate runtime measurement sections in metrics/phase_4G.toml**.
   - Add [measurements] section below existing [notes] without modifying existing keys.
   - auto_advance_accuracy: run 20 synthetic gate decisions (10 should auto-advance, 10
     should not); record correct/total.
   - mode_switch_latency_ms: time AutonomousGate::set_mode (p50, p99 over 100 calls).
   - gate_evaluation_overhead_ms: time AutonomousGate::decide with Evidence::all_passed
     pre-seeded (p50, p99 over 200 calls).
   - escalation_count_per_session: synthetic session driving 5 circuit-breaker trips;
     record actual EscalationFiredPayload count.
   - false_escalation_rate: 0/20 by construction (deterministic fixture); record.
   - split_task_overhead_ms: not measurable without C-9 tree wiring; record as
     unmeasured_reason = "requires_tree_wiring_C9".
   - substitute_model_swap_ms: time the config path only (no live model); p50 expected
     < 1ms; record.

5. **Populate runtime measurement sections in metrics/phase_4F.toml**.
   - Add [runtime_measurements] section; preserve all existing keys.
   - system_probe_latency_ms: run DefaultSystemProber::probe() 20 times; record p50, p99.
   - stub_execution_p50_ms, stub_execution_p99_ms: run SyntheticStubExecutor on a
     5-line stub 100 times; record.
   - alternatives_explored_avg: run 10 probe cycles with MAX_ALTERNATIVES_PER_STEP=3;
     count alternatives per run; record mean.
   - novel_step_skip_rate: 0.0 (synthetic fixture has 0 novel steps by construction);
     record with rationale.
   - constraint_check_latency_us: time check_constraints with 3-resource fixture
     (p50 over 1000 calls; expected < 100us).

6. **Populate runtime measurement sections in metrics/phase_4M.toml**.
   - Add [backend_latency] and [backend_health] sections; preserve existing keys.
   - For each of 11 backends: health_check() p50/p99 latency, Unavailable when CLI absent.
   - For Sidecar and AnthropicApi (the two live backends): complete() p50/p99 from a
     single canary prompt ("ok"). Requires live sidecar (use ELAI_MEASURE_MODE=1 guard).
   - fallback_fire_rate_per_1000: run RuntimeOrchestrator::try_complete with primary
     forced Unavailable 100 times; record fallback rows emitted per 1000 ops extrapolation.
   - uptime_pct: derived from health snapshots taken during measurement run.
   - cost_per_1m_tokens: record config-driven static values (not measured dynamically;
     annotate as source=config).
   - Cloud-Sonnet specific: run /task --mode hybrid via CLI integration; assert
     phase_4M.toml gets AnthropicApi latency row and Sidecar latency row populated from
     actual task execution. Fallback-fire-rate from any retries during that run appended.

7. **Populate runtime measurement sections in metrics/phase_4C.toml**.
   - Add [runtime_metrics] section; preserve existing qualitative notes.
   - The 9 categories from spec §4C (hook dispatch latency, tiered analysis p50/p99,
     T1/T2/T3 path split rates, block rate, warn rate, sanitization overhead, cross-session
     lock contention rate, SSB write latency, pre-compact duration):
   - Run IpcHookDispatcher with 100 synthetic PreToolUse events; record per-event p50/p99.
   - Run TieredAnalysis on 30 rule fixtures (10 T1, 10 T2, 10 T3); record tier split pct.
   - SSB write latency: measured from existing ssb.rs or noted as covered by phase_2A.

8. **Populate phase_b29_5_scale.toml with real tier runs**.
   - Pre-condition: ELAI_SCALE_FIXTURE_ROOT must point to a real clone root with
     linux/chromium/vscode/postgres sub-directories (populated by
     scripts/fixtures/clone_scale_fixtures.sh).
   - Driver: scripts/scale_test/run_all_tiers.sh (ADD this script if not present;
     wraps crates/knowledge/benches/symbol_graph_scale.rs for all 4 tiers).
   - For each tier: run bench binary, collect p50/p95/p99 for all 4 traversal tools
     and peak_rss_mb, write to phase_b29_5_scale.toml under the tier key.
   - Assert: p99 <= 500ms per scale_p99_budget_contract.rs (existing contract).
   - Run scale_determinism test (currently #[ignore]) once after measurement as
     regression check.
   - Note: if ELAI_SCALE_FIXTURE_ROOT is not set, task 8 records
     skipped_reason = "fixture_root_not_set" and does not fail the overall measure run.

9. **Write phase_1F.toml and populate phase_1G.toml / verify phase_1H.toml**.
   - phase_1F.toml: record phase=1F, decision=UNAVAILABLE,
     reason="prompt_lookup_decoding_not_instrumented", recommended_action="no_further_work"
     (1F decoding path was replaced; latency measurement is moot).
   - phase_1G.toml: run scripts/quantize_cross_encoder.py against the cross-encoder model
     in models/; record quantization sweep results (model_size_mb, latency_p50_ms,
     latency_p99_ms, quality_delta_pct per quantization level).
   - phase_1H.toml: read existing file; verify all fields present (vocab_coverage_code,
     model_variant_chosen, model_p50_latency_ms, recall_delta_mean_pct, wilcoxon_p_value,
     decision=DROP are confirmed). Add generated_at timestamp if absent.

## WIRING_CONTRACT

produces_symbols:
  - SlashCommand::Measure { scope: MeasureScope } in crates/cli/src/commands.rs
  - MeasureScope enum (All, Phase(String), Scale, Cloud)
  - measure_command_handler in crates/cli/src/measure.rs (new module)
  - ELAI_DEBUG_BATCH_DELAY_SECS env-driven branch in task_runner.rs
  - ELAI_MEASURE_MODE=1 guard in task_runner.rs
  - .elai_cc/metrics/phase_4B.toml (CREATE)
  - .elai_cc/metrics/phase_1F.toml (CREATE)
  - .elai_cc/metrics/phase_1G.toml (CREATE or populate)
  - [measurements] section in .elai_cc/metrics/phase_4G.toml (EXTEND)
  - [runtime_measurements] section in .elai_cc/metrics/phase_4F.toml (EXTEND)
  - [backend_latency] + [backend_health] sections in .elai_cc/metrics/phase_4M.toml (EXTEND)
  - [runtime_metrics] section in .elai_cc/metrics/phase_4C.toml (EXTEND)
  - tier entries in .elai_cc/metrics/phase_b29_5_scale.toml (EXTEND)
  - scripts/scale_test/run_all_tiers.sh (ADD if absent)
  - metrics/last_measure_run.toml (runtime artifact; not committed)
consumes_symbols:
  - AutonomousGate::decide, AutonomousGate::set_mode (crates/orchestrator/src/autonomous_gate.rs)
  - DefaultFeasibilityProber::probe, SyntheticStubExecutor (crates/orchestrator/src/feasibility_probe.rs)
  - IpcHookDispatcher (crates/plugins/src/ipc_hooks.rs)
  - TieredAnalysis (crates/plugins/src/tiered_analysis.rs)
  - RuntimeOrchestrator::try_complete, RuntimeOrchestrator::health_snapshot (crates/providers/src/runtime/)
  - IpcToolRegistry::dispatch (crates/tools/src/ipc_registry.rs)
  - ShortCoder (crates/knowledge/src/shortcoder.rs)
  - CompileCheckHandler (crates/tools/src/compile_check.rs)
  - DefaultSystemProber::probe (crates/orchestrator/src/system_probe.rs)
  - crates/knowledge/benches/symbol_graph_scale.rs (bench binary)
  - scripts/fixtures/clone_scale_fixtures.sh (fixture prep)
reachable_from:
  - CLI `elai measure --all`
  - CI: cargo test -p elai_cc_contracts -- scale_p99_budget (existing contract already reads TOML)
  - Auditor verification step for any phase whose TOML was previously placeholder
stub_ban:
  - measure_command_handler MUST NOT write fabricated numbers; every field written must
    be the result of an actual measurement or explicitly annotated with
    unmeasured_reason = "..."
  - ELAI_DEBUG_BATCH_DELAY_SECS branch MUST be gated by ELAI_MEASURE_MODE=1; it MUST
    NOT activate in any production code path where neither env var is set
observable_effect:
  trigger: ELAI_MEASURE_MODE=1 elai measure --all
  expect_artifact: all 9 phase_*.toml files have non-placeholder values for required
                   fields; metrics/last_measure_run.toml records pass for each scope
  trigger_cloud: elai measure --cloud (requires live sidecar + Anthropic API key)
  expect_cloud: phase_4M.toml [backend_latency.AnthropicApi] and [backend_latency.Sidecar]
                populated with real p50/p99 from hybrid task execution

## DETERMINISTIC_CHECK

```bash
# 1. elai measure subcommand variant present
rg "Measure" .elai_cc/crates/cli/src/commands.rs

# 2. ELAI_DEBUG_BATCH_DELAY_SECS env branch present
rg "ELAI_DEBUG_BATCH_DELAY_SECS" .elai_cc/crates/orchestrator/src/task_runner.rs

# 3. phase_4B.toml created with required keys
grep -E "ipc_overhead_ms|compile_check_latency|shortcoder_savings_pct" \
  .elai_cc/metrics/phase_4B.toml

# 4. phase_4G.toml has [measurements] section with non-placeholder values
grep -E "auto_advance_accuracy|mode_switch_latency_ms|gate_evaluation_overhead_ms" \
  .elai_cc/metrics/phase_4G.toml

# 5. phase_4F.toml has [runtime_measurements] with non-placeholder values
grep -E "system_probe_latency_ms|stub_execution_p50_ms|alternatives_explored_avg" \
  .elai_cc/metrics/phase_4F.toml

# 6. phase_4M.toml has [backend_latency] section
grep -E "\[backend_latency\]" .elai_cc/metrics/phase_4M.toml

# 7. phase_4C.toml has [runtime_metrics] section
grep -E "\[runtime_metrics\]" .elai_cc/metrics/phase_4C.toml

# 8. phase_1F.toml exists with UNAVAILABLE decision
grep "decision" .elai_cc/metrics/phase_1F.toml | grep -i "unavailable"

# 9. scale_p99_budget_contract passes (reads scale.toml; skips gracefully on missing tiers)
cargo test -p elai_cc_contracts --test scale_p99_budget_contract 2>&1 | grep "test result"

# 10. No fabricated numbers (unmeasured fields explicitly annotated)
rg "unmeasured_reason" .elai_cc/metrics/ --include="*.toml"
# expect: every previously-marked unmeasured=true field now has either a real value
#         or an unmeasured_reason string, never a bare 0 or placeholder
```

## ACCEPTANCE

- [ ] `elai measure --all` exits 0; metrics/last_measure_run.toml written with
      scopes_run = ["4B","4C","4F","4G","4M","1F","1G","1H","scale","cloud"] and
      pass=true for each scope (or skipped_reason for scale when fixture root absent)
- [ ] phase_4B.toml exists with ipc_overhead_ms, compile_check_latency_{rust,python,
      typescript,go}_ms (p50 + p99 each), shortcoder_savings_pct, test_count, loc_tools;
      no field value is 0 or placeholder
- [ ] phase_4G.toml [measurements] section present with 6 of 7 spec categories
      populated (split_task_overhead_ms annotated unmeasured_reason when C-9 not done)
- [ ] phase_4F.toml [runtime_measurements] section present with system_probe_latency_ms
      (p50/p99), stub_execution_p50_ms, stub_execution_p99_ms, alternatives_explored_avg,
      novel_step_skip_rate, constraint_check_latency_us all non-placeholder
- [ ] phase_4M.toml [backend_latency] section present; Unavailable returned by all 7
      CLI adapters when binary absent (health_check_when_absent = "Unavailable" for each)
- [ ] phase_4C.toml [runtime_metrics] section present with hook_dispatch_p50_us,
      hook_dispatch_p99_us, tier_split_t1_pct, tier_split_t2_pct, tier_split_t3_pct,
      block_rate, sanitization_overhead_us; no field is 0 unless genuinely measured as 0
- [ ] phase_b29_5_scale.toml: elai_self tier already present and unchanged; real
      linux/chromium/vscode/postgres tiers populated when ELAI_SCALE_FIXTURE_ROOT is set;
      scale_p99_budget_contract passes
- [ ] phase_1F.toml exists with decision = "UNAVAILABLE" and reason field
- [ ] phase_1G.toml exists with quantize sweep results (at least one quantization level)
- [ ] phase_1H.toml verified complete (generated_at timestamp present; all existing
      fields confirmed non-stale)
- [ ] ELAI_DEBUG_BATCH_DELAY_SECS env branch present in task_runner.rs; unit test
      confirms zero-overhead when vars absent; delay applied when both vars set
- [ ] Cloud-Sonnet smoke test: `elai measure --cloud` triggers hybrid task; phase_4M.toml
      [backend_latency.AnthropicApi] and [backend_latency.Sidecar] contain p50/p99 from
      the run; fallback_fire_rate recorded (0.0 if no retries occurred)
- [ ] cargo check --workspace after all edits: zero new errors
- [ ] No .unwrap() or .expect() added in production measure_command_handler code

## Files touched

- ADD: crates/cli/src/measure.rs (MeasureScope enum + measure_command_handler)
- MODIFY: crates/cli/src/commands.rs (SlashCommand::Measure variant)
- MODIFY: crates/orchestrator/src/task_runner.rs
  (ELAI_DEBUG_BATCH_DELAY_SECS + ELAI_MEASURE_MODE guard)
- ADD: .elai_cc/metrics/phase_4B.toml
- ADD: .elai_cc/metrics/phase_1F.toml
- ADD: .elai_cc/metrics/phase_1G.toml (or MODIFY if skeleton exists)
- MODIFY: .elai_cc/metrics/phase_4G.toml (add [measurements] section)
- MODIFY: .elai_cc/metrics/phase_4F.toml (add [runtime_measurements] section)
- MODIFY: .elai_cc/metrics/phase_4M.toml (add [backend_latency] + [backend_health])
- MODIFY: .elai_cc/metrics/phase_4C.toml (add [runtime_metrics] section)
- MODIFY: .elai_cc/metrics/phase_b29_5_scale.toml (add tier entries after scale run)
- MODIFY: .elai_cc/metrics/phase_1H.toml (add generated_at if absent)
- ADD: scripts/scale_test/run_all_tiers.sh (if absent; wraps bench binary for all tiers)
- ADD: workflow/reasoning/C-20-decisions.md

## Next

C-21 (verification report refresh pass — re-run auditor against all phase-4* reports
after metrics are populated to close PARTIAL status items) or fall through to next
outstanding C-plan depending on prioritization.
