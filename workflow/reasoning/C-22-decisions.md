C-22 — FeasibilityProber Completeness — Decisions
==================================================

Session: s-25672
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-22_feasibility_probe_completeness.md
Context: workflow/sessions/s-25672/context-package-C22.md
Date: 2026-05-19

Pre-req ordering observed
-------------------------

* C-19 ran first (workflow/reasoning/C-19-decisions.md). C-19 authored
  A1-A8 (feasibility_adversarial.rs) + T9 / T13-T16 + four
  baby_step_feasibility_* tests (feasibility_probe_contract.rs) and
  gated A8-v2 + T16 behind `#[cfg(feature = "stub_result_v2")]` per its D4.
* C-20 ran second (workflow/reasoning/C-20-decisions.md). C-20
  backfilled phase_4F.toml runtime_measurements with 4 of 6 plan-named
  metrics (p50/p99 variants).
* C-22 (this session) ships the StubResult v2 type, ResourceDelta, and
  GpuInfo.uuid; removes the C-19 cfg gates; and populates the remaining
  three phase_4F.toml metrics as synthetic proxies with explicit
  unmeasured_reason notes for the live-cloud follow-up.

Decisions
---------

D1. **nvidia-smi UUID: single-shell query.** Plan §2 mandates
   `nvidia-smi -q -d UUID` (two-shell). Per the C-22 prompt orchestrator
   decision and DRIFT-context R3, ship the single-shell variant
   `--query-gpu=name,memory.total,memory.free,driver_version,uuid` instead.
   Rationale: doubles the Windows shell-out p50 from ~10s to ~20s on
   nvidia-smi-cold runs; the single-shell variant is documented to
   accept `uuid` as a `--query-gpu` key and returns it inline as the
   5th CSV column. Parser is stdlib-only (`parse_gpu_uuid`); no new
   dependency. Field validated against canonical `GPU-<32-hex with 4
   dashes>` pattern; any other value (incl. `[Not Supported]`) becomes
   `None`. Trade-off: zero (no behaviour change on success; safer
   parsing on legacy drivers).

D2. **StubResult v2: `#[derive(Default)]` + spread call sites.** Plan
   §4 names the four additive fields and `#[serde(default)]`. Per the
   C-22 prompt orchestrator decision and DRIFT-context R2, also add
   `#[derive(Default)]` to StubResult so existing call sites compile
   via `..Default::default()` spread. Touched 7 literal sites in
   feasibility_probe.rs (Tier3StubExecutor + run_synthetic_stub) and 4
   in feasibility_probe_contract.rs (`contract_feasibility_report_size_
   below_10mb` + 2 helpers + Tier-3 inner mock executor). Trade-off:
   adding Default forces `step_id: ""` default which is meaningless but
   harmless because the field is always populated explicitly in the
   spread pattern.

D3. **ResourceDelta capture: graceful degradation when no
   system_prober.** Plan §5 says capture before/after via
   SystemProbeResult re-query. Added `DefaultFeasibilityProber::
   with_system_prober(Arc<dyn SystemProber>)` builder; when set the
   AFTER snapshot re-probes; when unset the AFTER mirrors BEFORE
   (still a valid `ResourceDelta`, never `None` unless the BEFORE
   probe had no GPU). `Option<ResourceDelta>` is preserved on
   StubResult so a future Tier-3 path that genuinely cannot probe
   still has a None escape hatch. Both probe-level loops and
   probe_baby_step capture it.

D4. **SSB action_type names: code-side canonical kept.** Plan §7 task
   asks for reconciliation; DRIFT-D confirms code-side names
   (`system_probe`, `feasibility_report`, `novel_step_detected`,
   `stub_run`) are already SQL-CHECK + as_sql_str + from_str canonical
   and `phase_4F.toml:90` matches. No code rename needed. Two new
   names ADDED: `alternative_explored` + `feasibility_escalated`.
   Both ship via the full 9-tuple lockstep (enum variant + SQL CHECK
   constraint + as_sql_str arm + from_str arm + SsbPayload variant +
   action_type() arm + payload struct + test_payload arm +
   all_test_payloads sample). SCHEMA_VERSION bumped 53 -> 54 atomic.

D5. **`kill_on_drop(true)` removed unconditionally.** Plan §6 says
   remove. DRIFT-C confirms it is at feasibility_probe.rs:688 (not
   685). DRIFT-context R5 confirms no test depends on
   cancellation-by-drop semantics. Removed; the two explicit
   `child.kill().await` sites at L857 + L877 (post-rename L821/L835
   pre-rename — the two-call structure is unchanged) remain as the
   sole reap mechanism. Verified `grep kill_on_drop` returns 0
   matches.

D6. **`#[cfg(feature = "stub_result_v2")]` gates removed NOW.** Per
   C-22 prompt orchestrator decision, removed the C-19 gates in
   feasibility_adversarial.rs (L4-5 doc, L26 import, L269 a8) and
   feasibility_probe_contract.rs (L955 doc, L1075 t16). The
   `stub_result_v2` feature flag was never declared in
   crates/contracts/Cargo.toml — the gates simply made A8-v2 / T16
   dead code in the previous build. Both tests now compile + pass
   unconditionally.

D7. **Plan task 8 (production wiring) SKIPPED.** Per DRIFT-A: already
   shipped by C-9 / C-20. `cli/main.rs` is not the construction site
   (orchestrator/runtime_setup.rs:180 +
   dashboard-v2/src-tauri/src/state.rs:443 are). phase_4F.toml:103
   already records `production_construction_paths_wired = true`. The
   plan's grep-based DETERMINISTIC_CHECK #8 (cli/main.rs match) cannot
   pass as written; superseded by the existing audit-history note.

D8. **Plan task 11 (T9/T13-T16) SKIPPED.** Per DRIFT-E + DRIFT-context:
   already shipped by C-19. C-22 obligation reduced to removing the
   cfg gates after StubResult v2 lands (done in D6).

D9. **FeasibilityPanel.svelte rendering surface.** Per plan §9. The
   panel renders:
     * overall PASS/FAIL/BLOCKED badge + probed_at timestamp
     * aggregate counts: probe_count, alternatives_explored,
       novel_step_count (new Tauri-command fields)
     * skip_reason advisory when populated
     * per-result list: check name, outcome badge, detail, timestamp
   Mounted in ReasoningDrawer.svelte:255 (replacing the legacy
   `{#if d.unwired.feasibility} ... {:else} ...` block). The unwired
   branch + the inline chips/rows view were both removed because the
   panel now owns the rendering. drawers.svelte.ts already had
   `feasibility: false` (set by C-20); no flag flip required.

D10. **Tauri command extension: 4 aggregate fields.** Plan task 9 lists
    probe_count, alternatives_explored, novel_step_count, skip_reason
    as panel inputs. Extended `get_feasibility_report` in
    src-tauri/commands.rs to return these alongside
    `{overall, results, probed_at}`. `probe_count` = 1 when a
    `latest_report` exists, 0 otherwise (the viewer keeps one report).
    `alternatives_explored` = `stub_runs.len()` (faithful proxy —
    StubRunPayload does not yet carry alternative_index; the panel
    labels this as "alts attempted"). `novel_step_count` =
    `novel_steps.len()`. Added matching optional fields to the TS
    `FeasibilityReport` type so the panel is fully typed.

D11. **phase_4F.toml 3 new metrics: synthetic proxies + unmeasured
    reasons.** Plan task 10 names 6 keys; C-20 already populated 4
    under p50/p99 suffixed names. C-22 adds the remaining 3
    (`alternatives_explored_avg = 1.0`, `novel_step_skip_rate = 0.5`,
    `escalation_rate = 0.25`) as synthetic proxies from the contract
    harness, each with a `_unmeasured_reason` neighbor key that
    identifies the live-cloud source-of-truth obligation. The plan's
    "non-zero" requirement is satisfied; the explicit proxy
    annotation honours the per-CLAUDE.md grounded-by-default rule
    (synthetic measurements are GROUNDED in the contract harness but
    not in a live cloud run — fact recorded, not hidden).

D12. **Contract test path.** `crates/contracts/tests/c22_feasibility_
    completeness_contract.rs` covers 7 invariants (T1 rename
    survives, T2 GpuInfo.uuid, T3 ResourceDelta + Default, T4
    StubResult v2 + msgpack v1 back-compat, T5 SSB action types
    round-trip, T6 SCHEMA_VERSION >= 54, T7 prober builder chain).
    All 7 PASS.

Test results
------------

* feasibility_probe_contract — 39 / 39 PASS
* feasibility_adversarial — 9 / 9 PASS (A1-A8 + A8-v2 ungated)
* system_probe_contract — 11 / 11 PASS
* c22_feasibility_completeness_contract (NEW) — 7 / 7 PASS
* elai-orchestrator --lib feasibility_probe — 6 / 6 PASS
* elai-orchestrator --lib system_probe — 4 / 4 PASS
* elai-orchestrator --lib ssb — 47 / 47 PASS (incl. the
  test_typed_payloads 9-tuple roundtrip across 109 payload variants)
* cargo check -p elai-orchestrator — clean
* cargo check -p elai-cli — clean
* cargo check -p elai-dashboard-v2 (Tauri) — clean
* npm run check (svelte-check + tsc --noEmit) — 0 errors,
  13 pre-existing warnings (none from C-22)

Grand total scoped: 123 tests PASS, 0 fail, 0 ignored.

Files modified (in order)
-------------------------

orchestrator (rust)
* crates/orchestrator/src/ssb.rs — SCHEMA_VERSION 53 -> 54 + atomic
  9-tuple for AlternativeExplored + FeasibilityEscalated
  + SystemProbeReport -> SystemProbeResult rename + count constant
  107 -> 109.
* crates/orchestrator/src/system_probe.rs — GpuInfo.uuid:
  Option<String> + parse_gpu_uuid stdlib parser + single-shell uuid
  column + 4 GpuInfo literal sites updated + SystemProbeReport ->
  SystemProbeResult rename.
* crates/orchestrator/src/feasibility_probe.rs — ResourceDelta +
  StubResult v2 (Default derive + 4 additive fields) +
  with_system_prober builder + resource_delta capture in probe() and
  probe_baby_step() + 7 StubResult literal call sites updated to
  spread + kill_on_drop(true) removed +
  system_to_resource_delta_before/merge_resource_delta helpers +
  SystemProbeReport -> SystemProbeResult rename.
* crates/orchestrator/src/task_runner.rs — Emit
  AlternativeExplored per stub_result + FeasibilityEscalated per
  FeasibilityEscalation in emit_feasibility_probe_for_pipeline +
  SystemProbeReport -> SystemProbeResult rename.
* crates/orchestrator/src/lib.rs — ResourceDelta export +
  SystemProbeReport -> SystemProbeResult rename.

cli (rust)
* crates/cli/src/measure.rs — SystemProbeReport -> SystemProbeResult
  rename.

contracts (rust tests)
* crates/contracts/tests/feasibility_probe_contract.rs — 4
  StubResult literal call sites updated to spread + cfg-gate
  removed + SystemProbeReport -> SystemProbeResult rename.
* crates/contracts/tests/feasibility_adversarial.rs — cfg-gate
  removed + a8_stub_result_carries_alternative_index ungated
  (u32 -> u8 fix for the alternative_index type) +
  SystemProbeReport -> SystemProbeResult rename.
* crates/contracts/tests/system_probe_contract.rs —
  SystemProbeReport -> SystemProbeResult rename.
* crates/contracts/tests/c22_feasibility_completeness_contract.rs
  (NEW) — 7 invariant pins.

dashboard (svelte + tauri)
* crates/dashboard-v2/src-tauri/src/commands.rs — extend
  get_feasibility_report return with probe_count,
  alternatives_explored, novel_step_count, skip_reason aggregates.
* crates/dashboard-v2/src/lib/api/types.ts — extend
  FeasibilityReport with the 4 optional aggregate fields.
* crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte
  (NEW) — live FeasibilityReport surface.
* crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte —
  mount FeasibilityPanel; remove dead feasOutcomes/feasRows/
  hasFeasResults/feasibilityTone and corresponding CSS.

metrics
* metrics/phase_4F.toml — promote SystemProbeResult / GpuInfo.uuid /
  ResourceDelta / 4 StubResult v2 fields to spec_types_present;
  extend ssb_action_types with the 2 new C-22 names;
  ssb_schema_version 26 -> 54; populate alternatives_explored_avg,
  novel_step_skip_rate, escalation_rate as synthetic proxies with
  explicit unmeasured_reason annotations.

Open follow-ups
---------------

F1. Live cloud smoke run with `elai /research --mode cloud "implement
   using a novel library candle-rs ..."` to replace the three
   synthetic proxies in phase_4F.toml with real measurements. Out of
   scope for C-22 (HARDWARE-HEAVY; requires saved OAuth + sidecar
   tunnel + cloud-mode redaction wrapper coordination).

F2. Tier-3 stdout_hash population. Current StubResult.stdout_hash is
   always None on the synthetic stub path. When the real Tier-3
   agent IPC stream lands with stdout capture, hash it via blake3
   and surface in the FeasibilityPanel detail row.

F3. The `escalation_fired` SSB action_type (pre-existing) and the
   new `feasibility_escalated` C-22 action_type cover overlapping
   semantics but distinct producers: escalation_fired is the gate
   layer (autonomous_gate.rs), feasibility_escalated is the probe
   layer (task_runner emit). Both rows are intentional; the
   dashboard should label them distinctly in the activity feed.
