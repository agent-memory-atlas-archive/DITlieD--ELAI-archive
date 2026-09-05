# H-9: Cost cockpit dashboard — activate the dark spend ledger; the believe-the-number instrument

STATUS: PLANNED (WAVE 1; ACTIVATE + BUILD; the instrument that makes H-1/H-2/H-3 falsifiable)

Objective: The cloud spend ledger is DARK END-TO-END (C10, grep-confirmed 2026-06-03): SpendAccumulator +
CumulativeTokenTracker + the CloudSpend SSB emit (grounding_client.rs:887-940) are gated `if let Some(ssb) =
self.ssb_writer`, and from_env()/from_env_async() NEVER chain .with_ssb_writer() — the sole cloud client ctor
(task_runner.rs:5840) attaches no writer. CloudPanel.svelte is MOUNTED and polls get_cloud_spend every 5s but
renders ALL-ZEROS in production; cost_per_task_cents is always None (the only feeder is #[cfg(test)]-only); there is
NO dedicated token-tracker UI component (absent). H-9 attaches .with_ssb_writer on the cloud client (the pattern
TemplateCrystallizer already uses correctly), builds the cost feeder, and adds the absent component: a per-response
input/output/CACHED breakdown + a cache-expiry timer + $/resolved-task + cache-hit-rate (consuming H-1's
measure_cache_hit_rate). This is the instrument that turns H-1/H-2/H-3's claims into a MEASURED, journal-recomputable
number — the believe-the-number control that pairs with G-6 SAFEDELTA (same measurement discipline).

Estimated effort: ~1-2 sessions (~6-10 h). In-process activation + a dashboard component; W-6 dashboard E2E via dashboard-smoker.
Pre-requisites: C-5 CloudPanel SHIPPED (CloudPanel.svelte, get_cloud_spend commands.rs:5088); the CloudSpend SSB emit
  SHIPPED but dark (grounding_client.rs:887-940); H-1 (the measured hit-rate producer) — coordinate. TemplateCrystallizer
  with-writer pattern SHIPPED (runtime_setup.rs:371) — the correct attach to replicate.
Parallel-safe with: H-5/H-6/H-7/H-8. Coordinate with H-1 (the hit-rate source) and H-3 (the input:output ratio).
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §13 (F-E9) + §14 (the measurement) + §12 (Devin per-response UX);
  _grounding/H-GROUNDING-MAP.md C10 + the H-9 table. Doctrine: H-0_INDEX.md.

## Context

The claim "ELAI is more efficient than Claude Code / Cursor on the same model" is a MEASURED $/resolved-task delta at
the model-harness-CONFIG level (Harness-Bench arXiv:2605.27922 — capability/cost vary per pairing; HD-1), never a
SWE-bench rank. The efficiency tuple per config is ($/resolved-task, cache-read fraction, input:output token ratio,
tokens-per-resolved-task) with a paired bootstrap CI and a ~6pp noise floor, recomputable from the SSB journal under
ELAI_SEED (GD-2). ELAI has the instrumentation wired (E-49 MetricsCollector, cache_check.rs hit-rate, SpendAccumulator
cache_read_tokens, CumulativeTokenTracker) — but the SSB-emit leg is dark, so none of it reaches the dashboard. Devin
Desktop ships a per-response token breakdown (input / output / CACHED) + a prompt-cache-expiry timer in the context
indicator (the single most directly stealable UX — ELAI's edge is to beat them on RIGOR, a journal-recomputable
paired-CI number). H-9 is the activation + the UI that makes the efficiency program visible and falsifiable.

## Existing state (B/C/D/E/F/G coverage)
- SpendAccumulator (cloud_mode.rs:646) — WIRE-DARK. Owned by CloudGroundingClient (grounding_client.rs:411);
  accumulate/accumulate_cache called live (891-895) + CloudSpend SSB built (922-932) from snapshot — BUT the entire
  block is gated `if let Some(ssb) = self.ssb_writer` (:887) and from_env/from_env_async (602/626) return Self::new with
  ssb_writer:None and NEVER chain .with_ssb_writer().
- CloudGroundingClient (grounding_client.rs:394) — WIRE-DARK emit. with_ssb_writer :483 NEVER called on a CloudGroundingClient
  in non-test code; the sole prod ctor task_runner.rs:5840 (from_env_async, env-gated) attaches no writer; the daily-cap check :820 dormant.
- CumulativeTokenTracker (context_budget.rs:774) — WIRE-DARK emit, live record. Constructed grounding_client.rs:462-464;
  record_response live (906-913); consumed by TaskRunnerConfig.token_tracker (task_runner.rs:2844) for the PreCompact ladder budget — but only on the cloud branch, inside the same ssb_writer-gated block.
- cost_per_task_cents (metrics_cmd.rs:53; source coding_metrics/types.rs:32) — WIRE-DARK. CLI plumbing shipped
  (`elai metrics capture`) but the VALUE is always None: the only feeder record_cost_sample_cents (collector.rs:88) is #[cfg(test)]-only.
- CloudPanel.svelte (screens/CloudPanel.svelte) — SHIPPED frontend, dark data. Polls get_cloud_spend every 5s ->
  Tauri get_cloud_spend (commands.rs:5088) -> SsbViewer::cloud_spend_summary over cloud_spend rows; renders ALL-ZEROS;
  commands.rs:5077-5080 admits zero "because the CLI --mode cloud flag bridge is still under follow-up"; redaction_count hardcoded 0.
- C-25 CumulativeTokenTracker frontend — ABSENT. No dedicated UI component; MUST be CREATED.
- TemplateCrystallizer (compiled_pipeline.rs:293, constructed runtime_setup.rs:371 WITH an SSB writer) — SHIPPED. The
  correct .with_ssb_writer attach pattern H-9 replicates for the cloud client.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C10 + the H-9 symbol table.
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §13 (F-E9) + §14 (the measurement protocol + comparators) + §12 (Devin per-response UX + cache-expiry timer).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grounding_client.rs:380-500,880-940 (CloudGroundingClient + the dark CloudSpend block + with_ssb_writer)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:640-735 (SpendAccumulator)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_setup.rs:365-390 (the TemplateCrystallizer with-writer pattern to replicate)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:5070-5095 (get_cloud_spend) + src/lib/screens/CloudPanel.svelte
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\metrics_cmd.rs:50-75 (cost_per_task_cents / run_capture)
- CLAUDE.md HARD RULES + HD-1 (measured-not-multiplied) + GD-2 (reproducible-metric).

## Tasks (WAVE 1 — activate + build)

1. **Attach .with_ssb_writer on the cloud client (~50 LOC) [the activation].** At the sole cloud ctor (task_runner.rs:5840)
   and in from_env/from_env_async (grounding_client.rs:602/626), chain .with_ssb_writer(ssb) so the CloudSpend emit block
   (:887-940) actually fires. Replicate the TemplateCrystallizer pattern (runtime_setup.rs:371). The cloud_spend rows now land.

2. **Build the cost_per_task_cents feeder (~50 LOC).** Add a non-test production feeder of record_cost_sample_cents
   (collector.rs:88, today #[cfg(test)]-only) so `elai metrics capture` reports a real cost per task (today always None).
   The number is recomputable from the SSB journal (GD-2).

3. **Build the C-25 token-tracker dashboard component (~280 LOC svelte + ~60 LOC tauri).** A per-response input/output/CACHED
   token breakdown + a prompt-cache-expiry timer (Devin-style) + $/resolved-task + cache-hit-rate (consuming H-1's
   measure_cache_hit_rate). Reads the now-live CloudSpend rows + the hit-rate. cross_lang_diff PASS.

4. **Fix the CloudPanel zero-rendering + redaction_count (~30 LOC).** With the writer attached, CloudPanel renders real
   numbers; remove the redaction_count hardcoded 0 (commands.rs) and wire the real count (from G-9 if shipped, else 0 with an honest label).

5. **Decision doc workflow/reasoning/H-9-decisions.md (~100 LOC).** Records: the writer-never-attached end-to-end dark
   finding; the TemplateCrystallizer with-writer pattern; HD-1 (measured per config, never multiplied); the Devin UX steal;
   the efficiency tuple ($/RT, cache-read fraction, input:output ratio, tokens/RT) is the believe-the-number deliverable.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/task_runner.rs (the .with_ssb_writer attach on the cloud client) + grounding_client.rs (from_env/from_env_async chain the writer)
  - crates/cli/src/metrics_cmd.rs OR coding_metrics/collector.rs (a non-test feeder of record_cost_sample_cents)
  - crates/dashboard-v2/src/lib/... (the C-25 token-tracker component: input/output/cached + cache-expiry timer + $/RT + hit-rate)
  - crates/dashboard-v2/src-tauri/src/commands.rs (the real redaction_count + the token-tracker read command)
consumes_symbols:
  - crates/orchestrator/src/grounding_client.rs::CloudGroundingClient::with_ssb_writer + the CloudSpend emit block (887-940)
  - crates/orchestrator/src/cloud_mode.rs::SpendAccumulator (accumulate/accumulate_cache/snapshot)
  - crates/orchestrator/src/context_budget.rs::CumulativeTokenTracker (cache_read_tokens)
  - crates/orchestrator/src/runtime_setup.rs::TemplateCrystallizer (the with-writer pattern to replicate, do NOT reuse its writer)
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_cloud_spend (the read path CloudPanel polls)
  - crates/orchestrator/src/cache_check.rs::measure_cache_hit_rate (H-1's hit-rate producer)
reachable_from:
  - crates/orchestrator/src/task_runner.rs:5840 (the cloud client ctor — attach the writer) -> CloudSpend SSB rows  [PRODUCTION cloud lane]
  - crates/dashboard-v2/src-tauri/src/commands.rs:5088 (get_cloud_spend) -> CloudPanel + the C-25 component  [dashboard]
stub_ban:
  - NO CloudGroundingClient constructed WITHOUT .with_ssb_writer on the cloud lane (the whole point — the rows must land)
  - NO cost_per_task_cents feeder left #[cfg(test)]-only (a non-test feeder)
  - NO redaction_count hardcoded 0 with a "redacted" claim (wire the real count or label it honestly — GD-5)
  - NO multiplied headline number (HD-1: report the tuple per config; cache + output wins are separate stacks)
  - blake3; no .unwrap()/.expect() in prod
frontend_pair:
  the C-25 token-tracker component + the activated CloudPanel (svelte) + the token-tracker read command (tauri). cross_lang_diff PASS; W-6 dashboard E2E.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a real cloud-mode task, NO env override; open the dashboard CloudPanel>"
  expect_ssb: "cloud_spend SSB rows land (input/output/cache_read tokens + spend) — where pre-H-9 the writer was never
    attached and zero rows landed; every number recomputes byte-identically from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "CloudPanel + the C-25 token-tracker component render NON-zero input/output/cached counts + a live
    $/resolved-task + cache-hit-rate + a cache-expiry timer (verified through the dashboard by dashboard-smoker, W-6),
    where pre-H-9 they rendered all-zeros"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-9_cost_cockpit_dashboard.md --run-trigger
rg "with_ssb_writer" .elai_cc/crates/orchestrator/src/task_runner.rs .elai_cc/crates/orchestrator/src/grounding_client.rs  # writer now attached on the cloud lane
rg "record_cost_sample_cents" .elai_cc/crates/cli/src .elai_cc/crates/orchestrator/src --glob '!*test*'  # a non-test feeder exists
test -f .elai_cc/crates/dashboard-v2/src/lib/screens/TokenTrackerPanel.svelte  # the C-25 component exists
rg "redaction_count" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs    # no longer hardcoded 0 with a false claim
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-9-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (cloud_spend rows land; CloudPanel + the C-25 component render non-zero).
- [ ] W-2 default-ON: the writer is attached + the rows emit with no env override (beyond the existing ELAI_GROUNDING_BACKEND=cloud lane selector).
- [ ] W-3 production entrypoint: the writer attach fires from the live cloud client ctor (task_runner.rs:5840); not test-only.
- [ ] W-4: the C-25 component + CloudPanel cross-lang mirrored; cross_lang_diff PASS.
- [ ] W-6: the panel renders real numbers THROUGH the dashboard (dashboard-smoker).
- [ ] GD-2: every reported number ($/RT, cache-read fraction, input:output ratio, tokens/RT) recomputes from the journal under ELAI_SEED.
- [ ] HD-1: the numbers are reported as a tuple per config, never multiplied into a headline.
- [ ] GD-5: redaction_count is real or honestly labelled (no fabricated "redacted" claim).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + dashboard svelte-check.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; the writer attach (produced) makes the CloudSpend emit (consumed) fire (W-8) on the live cloud lane (W-3).
- W-2: writer attached + rows emit by default on the cloud lane.
- W-3: task_runner.rs:5840 (the cloud ctor); not test.
- W-4: the C-25 component + CloudPanel; cross_lang_diff PASS.
- W-5: a runnable cloud task -> real cloud_spend rows + a rendering panel; in-process + dashboard.
- W-6: the panel renders real numbers via dashboard-smoker (the W-6 item).
- W-7: with_ssb_writer / SpendAccumulator / CumulativeTokenTracker / get_cloud_spend / cost_per_task_cents grep-confirmed 2026-06-03 (C10).
- W-8: the writer attach (produced) is consumed by the CloudSpend emit; the C-25 component (produced) consumes get_cloud_spend.
- W-9: NO new action_type (CloudSpend exists; cost_per_task advisory optional).
- W-10: Tier-1 in-process (the row emit) + the dashboard render (W-6).
- HD-1/GD-2: numbers measured per config + journal-reproducible.

## Tests to add
- crates/contracts/tests/h9_cost_cockpit_contract.rs (~170 LOC): (a) the cloud client is constructed WITH a writer on
  the cloud lane (was None); (b) a cloud inference emits a cloud_spend row with input/output/cache_read tokens + spend;
  (c) the row recomputes the same $/RT + cache-read fraction byte-identically from the journal under ELAI_SEED; (d)
  cost_per_task_cents has a non-test feeder and reports a real value; (e) redaction_count is the real count or honestly labelled.
- a dashboard-smoker check that CloudPanel + the C-25 component render non-zero from real rows (W-6).

## Files touched
- MODIFY: crates/orchestrator/src/task_runner.rs (~30 LOC: .with_ssb_writer on the cloud ctor)
- MODIFY: crates/orchestrator/src/grounding_client.rs (~20 LOC: from_env/from_env_async chain the writer)
- MODIFY: crates/cli/src/metrics_cmd.rs OR coding_metrics/collector.rs (~50 LOC: a non-test cost feeder)
- CREATE: crates/dashboard-v2/src/lib/screens/TokenTrackerPanel.svelte (~280 LOC: input/output/cached + cache-expiry timer + $/RT + hit-rate)
- MODIFY: crates/dashboard-v2/src-tauri/src/commands.rs (~60 LOC: the token-tracker read command + the real redaction_count)
- MODIFY: crates/dashboard-v2/src/lib/screens/CloudPanel.svelte (~30 LOC: render the now-live numbers)
- CREATE: crates/contracts/tests/h9_cost_cockpit_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-9-decisions.md (~100 LOC)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §13 (F-E9: steal Devin's per-response breakdown + cache-expiry timer);
  §14 (the efficiency tuple per config, paired bootstrap CI, ~6pp noise floor, journal-recomputable; Harness-Bench
  arXiv:2605.27922; SWE-Effi arXiv:2509.09853 — efficiency is cost-AND-correctness); §12 (Cursor 85% / Claude Code 98.2% cache-read fractions — the bars).
- ELAI ground truth (C10): the CloudSpend emit gated on ssb_writer (grounding_client.rs:887); from_env never chains
  the writer (:602/:626); the sole cloud ctor task_runner.rs:5840 attaches none; CloudPanel renders all-zeros (commands.rs:5077-5080);
  cost_per_task_cents test-only feeder; TemplateCrystallizer with-writer pattern runtime_setup.rs:371.
- CLAUDE.md HARD RULES + HD-1 + GD-2.

## Open questions / risks
- The CloudPanel data source is the cloud_spend rows; if ELAI_GROUNDING_BACKEND defaults to sidecar, the cloud lane (and
  thus the rows) only run when cloud mode is selected — the panel honestly shows "no cloud spend this session" rather than
  zeros that look like a bug. Label the empty state.
- HD-1 is the discipline: the cockpit must show the TUPLE (cache-read fraction, input:output ratio, $/RT, tokens/RT)
  per config, NOT a single multiplied "X% cheaper" headline — that is the exact sin the efficiency doc was written to stop.
- redaction_count is hardcoded 0; until G-9 ships the redactor, label it honestly ("redaction not yet wired") rather than implying zero secrets.

## Next
H-9 completes the Wave-1 efficiency instrument: with H-1 (hit-rate) + H-2 (cache-stable) + H-3 (output ratio) feeding
it, the measured $/resolved-task tuple is the believe-the-number control that pairs with G-6 SAFEDELTA on the SAME
measurement discipline (paired CI, ~6pp floor, journal-recomputable). Wave 2 (the canvas) opens with H-10's blocking B0 measurement.

End of H-9.
