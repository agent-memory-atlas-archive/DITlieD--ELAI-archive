# B-10: ContextBudget Profile Activation + 3E.5 file_overlap_score
Objective: stop passing `profile=None` from TaskRunner (5 production
sites that fall back to zeroed [0,0,0] slots). Pass real
ModelCapabilityProfile so the A-10 machinery actually drives
production prompt assembly. Implement the missing 3E.5 file_overlap_score
function for SSB entry filtering.
Estimated effort: 1 session.
Pre-requisites: B-1 (path unification — model profiles read from
canonical models dir).
Parallel-safe with: B-4..B-9.

## Context

Per audit-implementation/phase-3E.md:
- ModelCapabilityProfile (A-10) STRUCTURALLY YES, OPERATIONALLY NO
- 4 tier constructors implemented + tested
- task_runner.rs at 5 sites passes profile=None: 2751, 15609, 15743,
  15948, 16726 (refreshed 2026-05-31; the L4059 hit on
  appcontainer_profile_id is a sibling AppContainer field, NOT a
  dispatch_context_profile site — 5 canonical sites confirmed) —
  falls through to ContextBudget::compute(raw_max, [0, 0, 0], 4096)
  which is exactly the zeroed [0,0,0] stub 3J.WIRE.BUDGET was meant
  to replace
- CLI REPL path at cli/main.rs:7680-7727 sets all three slots correctly
- Net: REPL works; TaskRunner-routed plan dispatches do NOT
- 3E.5 file_overlap_score MISSING ENTIRELY (zero matches anywhere for
  file_overlap_score / compute_overlap / filter_ssb / per-entry intersection)

## Reads

- audit-implementation/phase-3E.md
- audit-implementation/_AGGREGATE.md PATTERN 4 (CONFIG STUB FALLBACK)
- crates/orchestrator/src/context_budget.rs:260-288 (ModelCapabilityProfile)
  + L768-872 (compute_with_profile)
- crates/orchestrator/src/task_runner.rs:2751, 15609, 15743, 15948, 16726
  (all 5 dispatch_context_profile=None sites — refreshed 2026-05-31;
  the 2992-2999 / 1842 / 6448 / 6565 / 6759 / 7513 baseline cited at
  authoring drifted as task_runner.rs grew through B-7 / B-37 / E-25 /
  B-6 / B-15 work. L4059 appcontainer_profile_id is NOT a dispatch
  site.)
- crates/cli/src/main.rs:7680-7727 (correct REPL pattern)
- spec: token-efficiency-design.txt section 2 (per-tier table)

## Tasks

1. **Detect model tier.** At session bootstrap, detect which model is
   loaded by querying the active provider:
   - Local sidecar → query /v1/models or read sidecar config
   - vLLM → /v1/models
   - Ollama → /api/tags
   - Cloud → from elai_cc.toml [cloud] config
   Map to ModelCapabilityProfile via tier table:
   - 12GB / Qwen2.5-Coder-7B → micro_7b_12gb
   - 12GB / Qwen2.5-Coder-3B → micro_3b_12gb
   - 48GB / Qwen2.5-Coder-32B → large_32b_48gb
   - 8GB → micro_8gb
   - cloud → cloud profile (per cloud-mode-design.txt section)

2. **Plumb profile into all 5 task_runner sites.** Replace
   `profile=None` with `profile=Some(self.model_capability_profile.clone())`.
   The profile is determined once at TaskRunner construction, stored as
   field, propagated to all dispatch_context_profile call sites.

3. **Implement compute_overlap(agent_files, ssb_entries) → ranked_entries.**
   For each SSB entry, compute file_overlap_score = |intersection| /
   |agent.files|. Filter entries below threshold (e.g. 0.1) before they
   compete for SsbEntries slot tokens.

4. **Wire compute_overlap into context_budget.rs slot allocation.**
   When SsbEntries slot is being filled, sort candidate entries by
   file_overlap_score descending; truncate at slot capacity; emit SSB
   `ssb_filtered_by_overlap` payload {entries_total, entries_kept,
   overlap_threshold} for dashboard transparency (B-6 producer wires this).

5. **Per-tier slot-weight customization.** Today the spec table values
   (per token-efficiency-design.txt §2) for 7B-12GB are 12.5/66.7/12.5/8.3
   for slots 4-7 (RepoMap/CodeContext/SsbEntries/GrcHistory). Confirm
   ModelCapabilityProfile constructors match this table; if not, fix.

6. **Adaptive reduction events.** When 3-level adaptive reduction
   fires (FilteredByRelevance → ReducedGranularity → ReducedScope →
   Terminal), emit SSB `context_reduction` per level (B-6 producer
   wires the dashboard-side renderer).

7. **TaskRunner-vs-REPL parity test.** Add a contract test that asserts
   ContextBudget produced via TaskRunner pathway matches the budget produced
   via REPL pathway for an equivalent task input. Catches regressions
   where one path drifts back to None.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/context_budget.rs::detect_model_capability_profile
  - crates/orchestrator/src/context_budget.rs::compute_overlap
  - crates/orchestrator/src/context_budget.rs::file_overlap_score
  - crates/orchestrator/src/context_budget.rs::emit_ssb_filtered_by_overlap
  - crates/orchestrator/src/context_budget.rs::emit_context_budget_computed
  - crates/orchestrator/src/task_runner.rs::TaskRunnerConfig
  - crates/orchestrator/src/task_runner.rs::ResolvedConfig
  - crates/providers/src/vram_tier.rs::detect_vram_tier
  - crates/orchestrator/src/dispatcher.rs::build_dispatch_context
consumes_symbols:
  - crates/orchestrator/src/context_budget.rs::compute_with_profile
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_context_state
reachable_from:
  - crates/orchestrator/src/task_runner.rs::start_task_runner
  - crates/orchestrator/src/runtime_setup.rs::apply_task_runner_wiring
  - crates/dashboard-v2/src-tauri/src/commands.rs::submit_task
stub_ban:
  - "dispatch_context_profile: None disallowed in production task_runner"
  - "file_overlap_score function exists (3E.5 closed)"
  - "ContextBudget::compute(raw_max, [0,0,0], 4096) call disallowed in production"
frontend_pair: B-6 surfaces the budget viz
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && cargo test -p elai_cc_contracts --test context_budget_smoke_ssb -- --nocapture
  expect_ssb: context_budget_computed
  expect_artifact: ContextBudget per-slot bar visible in StatusBar / drawer

## DETERMINISTIC_CHECK

```bash
# 1. No profile=None in production task_runner
rg "dispatch_context_profile:\\s*None" .elai_cc/crates/orchestrator/src/task_runner.rs
# Expected: zero hits.

# 2. compute_overlap exists
rg "fn compute_overlap|fn file_overlap_score" .elai_cc/crates/orchestrator/src
# Expected: ≥1 hit.

# 3. Smoke: submit task, verify budget bar non-zero
sqlite3 .elai_cc/.elai_cc/sessions/<sid>/ssb.db \
  "SELECT json_extract(payload, '$.allocations') FROM ssb \
   WHERE action_type='context_budget_computed' ORDER BY seq DESC LIMIT 1;"
# Expected: 8 slots with non-zero values.
```

## ACCEPTANCE

- [ ] ModelCapabilityProfile detected at session bootstrap from active provider
- [ ] All 5 task_runner profile=None sites replaced
- [ ] compute_overlap implemented and callable (3E.5 closed)
- [ ] SsbEntries slot ranks by overlap before truncation
- [ ] context_reduction events emit per adaptive level fire
- [ ] Per-tier slot weights match token-efficiency-design.txt §2 table
- [ ] TaskRunner-vs-REPL parity contract test passes
- [ ] No regression in 3E unit tests (100-config zero-sum invariant)

## Files touched

- MODIFY: crates/orchestrator/src/task_runner.rs (5+ sites + new field)
- MODIFY: crates/orchestrator/src/context_budget.rs (compute_overlap +
  adaptive emit)
- MODIFY: crates/orchestrator/src/dispatcher.rs (if needed)
- MODIFY: crates/cli/src/main.rs (model tier detection at bootstrap)
- MODIFY: crates/providers/src/* (expose loaded model identity)
- ADD: tests/contracts/budget_parity.rs (TaskRunner vs REPL parity)
- CREATE: workflow/reasoning/B-10-decisions.md

## Decision doc

- Whether tier detection happens at session bootstrap (preferred) or
  per-task (more robust if user swaps models mid-session)
- file_overlap threshold default (0.1? 0.2?) — measure on real data
- Whether to emit per-turn ContextBudgetComputed SSB row for dashboard

## GATE FOR PART I

After B-1 through B-10, the user should see:
- Live FSM badge transitions (B-2)
- Real engine telemetry per query (B-4, B-7)
- GRC budgets per agent (real values) (B-5, B-8)
- DAG visualization for executing tasks (B-7)
- Branch UI in session panel (B-9)
- Per-slot context budget bar chart (B-10, B-6)
- No DEAD UI elements (B-3)

MVP wiring complete. Move to PART II (B-11..14) for visual restoration.

## Next

B-11 (Design system restoration).
