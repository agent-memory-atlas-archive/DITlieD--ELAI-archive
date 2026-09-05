# C-22: FeasibilityProber Completeness — B-21 / Phase-4F Gap Closure
Objective: close all MISSING and PARTIAL items from B-21_phase_4F_feasibility_probe.report.md
and phase-4F.report.md: rename SystemProbeReport -> SystemProbeResult, add GpuInfo.uuid,
ResourceDelta struct, StubResult extension fields, wire real DefaultFeasibilityProber into
production construction paths, author FeasibilityPanel.svelte, reconcile SSB action_type
names, remove kill_on_drop(true) from production path, and populate phase_4F.toml metrics.
Estimated effort: 2-3 sessions.
Pre-requisites: C-19 (adversarial test scaffolding — A1-A8 tests reference types added here;
C-19 gates those tests with #[cfg(feature="stub_result_v2")] if needed before C-22 ships);
C-9 (autonomous gate to dispatch decisions — required for skip path trigger in smoke test).
Parallel-safe with: C-21, C-23 (non-overlapping file scope; C-21 touches session_dag,
C-23 touches cloud-mode hardening; no shared files with this plan).

## Context

Verification report B-21_phase_4F_feasibility_probe.report.md (2026-05-17) assigns
MOSTLY_DONE with the following gaps:

1. SystemProbeReport vs SystemProbeResult name mismatch: spec says SystemProbeResult;
   code uses SystemProbeReport (system_probe.rs:112). DETERMINISTIC_CHECK grepping for
   SystemProbeResult will find zero matches and fail the check.

2. GpuInfo struct (system_probe.rs:72) lacks uuid field. The spec (phase-4F.txt) requires
   uuid populated via `nvidia-smi -q -d UUID` shell-out. Shell-out pattern is already
   established in system_probe.rs (no new deps needed per system_probe.rs:12 policy).

3. StubResult (feasibility_probe.rs:141) lacks alternative_index, stdout_hash, and
   stderr_summary fields. T16 contract test (C-19) depends on alternative_index=0 when
   malformed IPC occurs; A3 adversarial depends on stderr_summary for diagnostics.

4. ResourceDelta struct is absent. StubResult should carry vram_bytes_before/after and
   ram_bytes_before/after to enable FeasibilityPanel to display pre/post resource usage.

5. Production construction: cli/main.rs and runtime_setup.rs pass None where
   DefaultFeasibilityProber should be constructed. The entire probe path is dead in the
   live binary.

6. SSB action_type reconciliation: code emits system_probe/feasibility_report; WIRING_CONTRACT
   names feasibility_probe_started/feasibility_probe_complete. One canonical set must be
   chosen and all sites aligned.

7. kill_on_drop(true) at feasibility_probe.rs:685 conflicts with WIRING_CONTRACT stub_ban
   (explicit child.kill().await at lines 818/832 is the intended mechanism; kill_on_drop
   is redundant defense-in-depth that the contract disallows).

8. FeasibilityPanel.svelte is absent; ReasoningDrawer.svelte:259 explicitly notes
   "FeasibilityProbe not yet implemented"; d.unwired.feasibility = true in drawers.svelte.ts.

9. phase_4F.toml is missing 6 metric fields: system_probe_latency_ms, stub_execution_p50,
   stub_execution_p99, alternatives_explored_avg, novel_step_skip_rate, escalation_rate.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4F.txt (full spec, §4F.1-4F.6)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-21_phase_4F_feasibility_probe.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4F.report.md
- .elai_cc/crates/orchestrator/src/feasibility_probe.rs (full: ~1100 LOC)
- .elai_cc/crates/orchestrator/src/system_probe.rs (struct definitions, shell-out pattern)
- .elai_cc/crates/orchestrator/src/ssb.rs:222,531-540 (existing action_type constants)
- .elai_cc/crates/cli/src/main.rs (construction path for None feasibility prober)
- .elai_cc/crates/orchestrator/src/runtime_setup.rs (same)
- .elai_cc/crates/dashboard-v2/src/lib/windows/ReasoningDrawer.svelte:255-280
- .elai_cc/crates/dashboard-v2/src/lib/drawers.svelte.ts:232
- .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs (23 existing tests)
- .elai_cc/metrics/phase_4F.toml (current content — populate after tasks)

## Tasks

1. **Rename SystemProbeReport -> SystemProbeResult in system_probe.rs**.
   Find struct SystemProbeReport at system_probe.rs:112. Rename struct and all uses in the
   same file. grep elai.map for SystemProbeReport to locate all call sites across the codebase
   (impact_radius.py --symbols SystemProbeReport). Update every caller. The rename must be
   mechanical: no logic changes. Verify with: `grep -rn "SystemProbeReport"
   .elai_cc/crates/ | wc -l` returns 0 after rename.

2. **Add GpuInfo.uuid field** in system_probe.rs at the GpuInfo struct definition (line 72).
   Field: `uuid: Option<String>`. Populate in the existing nvidia-smi shell-out block by
   running `nvidia-smi -q -d UUID` and parsing the "GPU UUID" line via regex
   `GPU-[0-9a-f-]{36}`. Pattern: run once per GpuInfo, store result or None on parse fail.
   No new crate dependency — existing Command/String stdlib only (per system_probe.rs:12 policy).

3. **Add ResourceDelta struct** in feasibility_probe.rs (add after existing StubResult struct):
   ```
   pub struct ResourceDelta {
       pub vram_bytes_before: u64,
       pub vram_bytes_after: u64,
       pub ram_bytes_before: u64,
       pub ram_bytes_after: u64,
   }
   ```
   Derive: Debug, Clone, Default, serde::Serialize, serde::Deserialize.

4. **Extend StubResult** in feasibility_probe.rs:141 with three fields:
   - `alternative_index: u8` (0-indexed; 0 = first alternative or error path)
   - `stdout_hash: Option<String>` (blake3 hex digest of stub stdout, None if empty)
   - `stderr_summary: Option<String>` (first 256 chars of stderr, None if empty)
   - `resource_delta: Option<ResourceDelta>` (captured before/after probe execution)
   Use `#[serde(default)]` on each new field so existing deserialized msgpack payloads remain
   valid (additive schema change). No removal of existing fields.

5. **Capture ResourceDelta in stub execution path**. In feasibility_probe.rs, locate the stub
   execution block (near lines 818-843 where child.kill().await lives). Before spawning the
   stub subprocess: snapshot vram_bytes_before and ram_bytes_before via existing SystemProbeResult
   (now renamed) query. After stub returns: re-query for _after values. Populate
   StubResult.resource_delta. Use the same shell-out approach already present in system_probe.rs
   (no new sysinfo dep). If re-query fails: resource_delta = None, not a fatal error.

6. **Remove kill_on_drop(true) at feasibility_probe.rs:685**. The explicit child.kill().await
   at lines 818 and 832 is the production mechanism. kill_on_drop(true) violates WIRING_CONTRACT
   stub_ban and creates double-kill risk. Remove only that one call. Verify: `grep -n
   "kill_on_drop" .elai_cc/crates/orchestrator/src/feasibility_probe.rs` returns 0 matches.

7. **Reconcile SSB action_type names**. Decision: keep code-side names (system_probe,
   feasibility_report, novel_step_detected, stub_run) as canonical because they are already
   emitted in production and changing them would break existing SSB consumers. Update
   WIRING_CONTRACT in phase-4F.txt and B-21 plan to declare the actual names. Add two new
   action_types: alternative_explored (emitted once per NovelStep alternative accepted,
   payload: step_id + alternative_index) and feasibility_escalated (emitted on Escalate
   decision, payload: reason + escalation variant). Add both to ssb.rs action_type constants
   and emit them from the relevant decision points in feasibility_probe.rs.

8. **Wire real DefaultFeasibilityProber in production construction paths**.
   - cli/main.rs: locate the None passed where FeasibilityProber is expected; replace with
     `Some(Box::new(DefaultFeasibilityProber::new()))`.
   - runtime_setup.rs: same pattern. If DefaultFeasibilityProber::new() requires config
     (e.g., timeout constants), pass the existing AppConfig reference already in scope.
   Grep elai.map for DefaultFeasibilityProber to verify it is exported; if not, add pub.
   Impact: cargo check --workspace after this change must still pass.

9. **Add FeasibilityPanel.svelte** at
   .elai_cc/crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte.
   Props: `report: FeasibilityReport | null`. Renders:
   - probe_count: number of concepts probed
   - novel_step_count: total novel steps across all concepts
   - alternatives_explored: sum of alternatives per concept
   - skip_reason: if present, display skip notice
   - Per-concept accordion: concept name, decision (Proceed/ProceedWithConstraints/Escalate),
     stub results (stdout_hash, stderr_summary if present), resource_delta before/after vram.
   - If report is null: render "Feasibility probe not run" placeholder.
   Data source: calls existing get_feasibility_report Tauri command (commands.rs:433).
   Mount in ReasoningDrawer.svelte: replace the "FeasibilityProbe not yet implemented" block
   at line 259 with `<FeasibilityPanel {report} />`. Update drawers.svelte.ts:232 to set
   d.unwired.feasibility = false.

10. **Populate phase_4F.toml with 6 metric fields**. After tasks 1-9 compile and at least one
    full probe run completes (cloud smoke test in ACCEPTANCE), measure and write:
    - system_probe_latency_ms = measured p50 from smoke test (nvidia-smi round-trip)
    - stub_execution_p50 = measured p50 stub run time in ms
    - stub_execution_p99 = measured p99 stub run time in ms
    - alternatives_explored_avg = average alternatives per concept across smoke run
    - novel_step_skip_rate = fraction of concepts where skip fired (0.0-1.0)
    - escalation_rate = fraction of probes that returned Escalate
    Values must be non-zero floats from real measurement; not placeholder zeros.

11. **Author missing T9/T13-T16 contract tests** (if not already authored by C-19 when
    C-19 runs first). Check feasibility_probe_contract.rs for existing T9/T13-T16 functions.
    If absent, add them per C-19 task 7 specification. These tests are listed in C-19 as
    targets but C-19 depends on C-22 shipping ResourceDelta and alternative_index first.
    C-22 ships the types; C-19 authors the tests. Cross-reference: if C-19 ran before C-22
    and gated tests with #[cfg(feature="stub_result_v2")], remove the feature gate after
    this plan ships. Document which C-plan ran first in workflow/reasoning/C-22-decisions.md.

## WIRING_CONTRACT

produces_symbols:
  - SystemProbeResult (rename from SystemProbeReport; system_probe.rs:112)
  - GpuInfo.uuid: Option<String> (system_probe.rs:72)
  - ResourceDelta struct (feasibility_probe.rs, after StubResult)
  - StubResult.alternative_index / stdout_hash / stderr_summary / resource_delta
    (feasibility_probe.rs:141 extension)
  - SSB action_types: alternative_explored, feasibility_escalated (ssb.rs new constants)
  - FeasibilityPanel.svelte (dashboard-v2/src/lib/components/)
  - phase_4F.toml 6 metric fields (all non-zero)
consumes_symbols:
  - SystemProbeReport (consumed by rename — all callers updated)
  - DefaultFeasibilityProber::new() (feasibility_probe.rs — must be pub)
  - get_feasibility_report Tauri command (dashboard-v2/src-tauri/src/lib.rs:433)
  - child.kill().await (feasibility_probe.rs:818, 832 — sole kill mechanism post-C-22)
  - AppConfig (runtime_setup.rs — passed to DefaultFeasibilityProber::new if required)
reachable_from:
  - elai /research --mode cloud (triggers DefaultFeasibilityProber via production path)
  - dashboard ReasoningDrawer -> FeasibilityPanel (get_feasibility_report command)
  - SSB viewer: alternative_explored + feasibility_escalated rows visible in live run
stub_ban:
  - kill_on_drop(true) MUST NOT appear in production feasibility_probe.rs after C-22
  - resource_delta capture MUST use the same SystemProbeResult shell-out (no sysinfo dep)
  - FeasibilityPanel MUST call get_feasibility_report; no mock/hardcoded data in prod
frontend_pair: FeasibilityPanel.svelte + ReasoningDrawer.svelte + drawers.svelte.ts
observable_effect:
  trigger: elai /research --mode cloud "implement using a novel library candle-rs that I
    haven't used before"
  expect_ssb: SSB contains system_probe row (not null payload) + feasibility_report row
    (not null payload) + alternative_explored row(s) if alternatives found; if
    RESOURCE_MARGIN_THRESHOLD_PCT < 20% on any concept, skip path fires (novel_step_skip_rate
    > 0.0 in phase_4F.toml after run)
  expect_artifact: FeasibilityPanel renders probe_count >= 1, alternatives_explored >= 0,
    resource_delta vram fields non-zero; ReasoningDrawer shows panel (not placeholder text)
  expect_deterministic: grep -n "kill_on_drop" .elai_cc/crates/orchestrator/src/feasibility_probe.rs
    returns no matches

## DETERMINISTIC_CHECK

```bash
# 1. SystemProbeReport name eradicated
grep -rn "SystemProbeReport" .elai_cc/crates/
# Expected: 0 matches

# 2. SystemProbeResult present
grep -n "struct SystemProbeResult" .elai_cc/crates/orchestrator/src/system_probe.rs
# Expected: 1 match

# 3. GpuInfo.uuid field
grep -n "uuid" .elai_cc/crates/orchestrator/src/system_probe.rs
# Expected: >= 2 matches (field decl + parser line)

# 4. ResourceDelta struct present
grep -n "struct ResourceDelta" .elai_cc/crates/orchestrator/src/feasibility_probe.rs
# Expected: 1 match

# 5. StubResult new fields
grep -n "alternative_index\|stdout_hash\|stderr_summary\|resource_delta" \
    .elai_cc/crates/orchestrator/src/feasibility_probe.rs
# Expected: >= 4 matches (one per field in struct)

# 6. kill_on_drop removed from production path
grep -n "kill_on_drop" .elai_cc/crates/orchestrator/src/feasibility_probe.rs
# Expected: 0 matches

# 7. New SSB action_types
grep -n "alternative_explored\|feasibility_escalated" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: >= 2 matches (constant definitions)

# 8. Production wiring — not None
grep -n "DefaultFeasibilityProber" .elai_cc/crates/cli/src/main.rs
grep -n "DefaultFeasibilityProber" .elai_cc/crates/orchestrator/src/runtime_setup.rs
# Expected: >= 1 match per file

# 9. FeasibilityPanel component exists
test -f .elai_cc/crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte && echo OK

# 10. drawers.svelte.ts unwired flag cleared
grep -n "unwired.feasibility" .elai_cc/crates/dashboard-v2/src/lib/drawers.svelte.ts
# Expected: line shows false (not true)

# 11. phase_4F.toml 6 metric fields non-zero
grep -E "system_probe_latency_ms|stub_execution_p50|stub_execution_p99|\
alternatives_explored_avg|novel_step_skip_rate|escalation_rate" .elai_cc/metrics/phase_4F.toml
# Expected: 6 matches; none equal 0

# 12. Workspace compiles
cargo check --workspace 2>&1 | tail -3
# Expected: 0 errors
```

## ACCEPTANCE

- [ ] SystemProbeReport renamed to SystemProbeResult; 0 grep matches for old name in .elai_cc/
- [ ] GpuInfo.uuid: Option<String> present; nvidia-smi -q -d UUID parsing in system_probe.rs
- [ ] ResourceDelta struct present with vram_bytes_before/after, ram_bytes_before/after
- [ ] StubResult carries alternative_index, stdout_hash, stderr_summary, resource_delta;
      #[serde(default)] on each; existing deserialization of old msgpack does not panic
- [ ] resource_delta captured during stub execution (before/after SystemProbeResult query)
- [ ] kill_on_drop(true) removed from feasibility_probe.rs; grep returns 0 matches
- [ ] alternative_explored and feasibility_escalated added to ssb.rs and emitted from
      correct decision points in feasibility_probe.rs
- [ ] DefaultFeasibilityProber wired in cli/main.rs and runtime_setup.rs (not None)
- [ ] FeasibilityPanel.svelte created; renders probe_count, alternatives_explored,
      resource_delta vram, skip_reason, per-concept accordion
- [ ] ReasoningDrawer.svelte mounts FeasibilityPanel; "not yet implemented" text removed
- [ ] drawers.svelte.ts: d.unwired.feasibility = false
- [ ] phase_4F.toml: all 6 metric fields present, all non-zero after smoke run
- [ ] T9/T13-T16 tests present in feasibility_probe_contract.rs (added here or by C-19;
      compile-gate #[cfg] removed if previously gated)
- [ ] cargo check --workspace: 0 errors after all changes
- [ ] Cloud smoke test:
      elai /research --mode cloud "implement using a novel library candle-rs that I haven't
      used before"
      SSB contains system_probe + feasibility_report rows with non-null payloads;
      ResourceDelta vram fields non-zero in at least one StubResult;
      FeasibilityPanel renders probe_count >= 1 and alternatives_explored in ReasoningDrawer;
      if RESOURCE_MARGIN_THRESHOLD_PCT < 20%, skip path fires (visible in SSB novel_step row
      with skip_reason populated);
      grep -n "kill_on_drop" .elai_cc/crates/orchestrator/src/feasibility_probe.rs = 0

## Files touched

- MODIFY: .elai_cc/crates/orchestrator/src/system_probe.rs
    (rename SystemProbeReport -> SystemProbeResult; add GpuInfo.uuid; add uuid parsing)
- MODIFY: .elai_cc/crates/orchestrator/src/feasibility_probe.rs
    (add ResourceDelta; extend StubResult; remove kill_on_drop; emit new SSB types;
     capture ResourceDelta in stub execution; update all SystemProbeReport references)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (add alternative_explored + feasibility_escalated constants)
- MODIFY: .elai_cc/crates/cli/src/main.rs
    (replace None with Some(Box::new(DefaultFeasibilityProber::new())))
- MODIFY: .elai_cc/crates/orchestrator/src/runtime_setup.rs
    (same wiring fix as main.rs)
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/ReasoningDrawer.svelte
    (mount FeasibilityPanel; remove placeholder text at line 259)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers.svelte.ts
    (d.unwired.feasibility = false at line 232)
- MODIFY: .elai_cc/metrics/phase_4F.toml
    (add 6 metric fields with measured values from smoke run)
- MODIFY: .elai_cc/crates/contracts/tests/feasibility_probe_contract.rs
    (add T9/T13-T16 if not already authored by C-19; remove compile-gate if gated)
- CREATE: workflow/reasoning/C-22-decisions.md

## Next

C-19 (if run before C-22: remove #[cfg(feature="stub_result_v2")] gates in
feasibility_adversarial.rs after this plan ships the missing types).
C-23 (cloud-mode hardening — independent; parallel-safe).
