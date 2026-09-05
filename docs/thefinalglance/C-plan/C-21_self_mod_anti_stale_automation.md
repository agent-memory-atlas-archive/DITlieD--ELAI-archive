# C-21: Self-Modification + Anti-Staleness Automation Closure
Objective: close wiring gaps in B-16, B-23, phase-4H, and anti-staleness-enforcement
reports: wire check_anti_stale.sh into CI, complete B-16B Chunk C (user-approval gate),
install git pre-commit hook for LOCKED_KEYS, connect dep-audit to stale_api_catches,
wire steering vector refresh on deprecated_patterns promote, trigger nightly dep-audit
from Layer1A DIVERGING, replace AllowedConfigKey::Parameter(String) with closed 12-
variant ParameterKey enum, auto-populate ConstraintsBlock from rules.db, create
benchmarks/swe_bench/ fixtures, wire MistakeBook from T1/T2 BLOCK events.
Estimated effort: 2 sessions.
Pre-requisites: C-15 (entropy quarantine baseline).
Parallel-safe with: C-16, C-17, C-18, C-19, C-20.

## Context

B-16 report (2026-05-17): check_anti_stale.sh exists (scripts/check_anti_stale.sh,
273 lines) but zero CI reference. B-16B Chunk C ~12 .unwrap() sites in
deterministic_auditor + dkg deferred pending user approval (B-16B-decisions.md:24).

B-23 report: nightly dep-audit unwired (SettingsWindow.svelte:519 — "pending PART12");
steering vector refresh has no automated trigger from Layer1A promote.

phase-4H report: AllowedConfigKey::Parameter(String) uses runtime KNOWN_PARAMETER_KEYS
check (self_mod_safety.rs:38) not compile-time enum. benchmarks/swe_bench/ absent.
Pre-commit hook documented at self_mod_safety.rs:13-14 but not installed.

anti-staleness report: ConstraintsBlock populated from TOML only (project_profile.rs:
1117-1126); modern_api.npz absent; no automated trigger from deprecated_patterns change.

task_runner.rs (L5352-5364, L5977): hook_pre_blocked synthesizes Failed result feeding
record_mistake_on_failure but FailureCategory carries no T1/T2 tier discriminant.

self_mod_safety.rs: LOCKED_KEYS at L70 (12 entries), KNOWN_PARAMETER_KEYS at L38
(12 entries) — both string slices to be replaced by ParameterKey enum.

USER APPROVAL REQUIRED for Task 2 (B-16B Chunk C). Hard gate — do not proceed without
explicit "Y" reply.

## Reads (mandatory)

- .elai_cc/scripts/check_anti_stale.sh (exit codes, STRICT mode)
- .elai_cc/.github/workflows/windows.yml (step format)
- .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs (AllowedConfigKey L197, DIVERGING L891)
- .elai_cc/crates/orchestrator/src/self_mod_safety.rs (LOCKED_KEYS L70, KNOWN_PARAMETER_KEYS L38)
- .elai_cc/crates/orchestrator/src/self_mod_types.rs (FullPipelineOutcome L185, StaleApiCatch L127)
- .elai_cc/crates/orchestrator/src/self_mod_bridge.rs (deprecated_patterns promote path L43-79)
- .elai_cc/crates/cli/src/dep_audit.rs (dep_audit output shape)
- .elai_cc/metrics/compute_steering_vector.py (trigger contract)
- .elai_cc/crates/orchestrator/src/task_runner.rs (hook_pre_blocked L5352-5364, L5977)
- .elai_cc/crates/knowledge/src/project_profile.rs (constraints field L1117-1126)
- .elai_cc/crates/rules/src/schema.rs (deprecated_patterns query L900-931)

## Tasks

1. **Wire check_anti_stale.sh into CI**
   Add step to .elai_cc/.github/workflows/windows.yml after the HashMap ban check:
   `bash scripts/check_anti_stale.sh` in STRICT mode. Non-zero exit fails job.
   Match existing step format (working-directory: .elai_cc, shell: bash).
   Files: .elai_cc/.github/workflows/windows.yml

2. **B-16B Chunk C — CONDITIONAL on user approval**
   GATE: prompt "Chunk C: ~12 .unwrap() in deterministic_auditor.rs + dkg. Signature
   cascade required. Proceed? Y/N." Do not proceed on N or silence.
   If Y: read B-16B-decisions.md for site list. Convert using ?/.ok_or(). No .expect().
   Files: .elai_cc/crates/orchestrator/src/deterministic_auditor.rs, src/dkg/*.rs

3. **Install git pre-commit hook for LOCKED_KEYS**
   Create .elai_cc/scripts/install_pre_commit_hook.sh. Hook written to .git/hooks/pre-commit.
   Logic: grep staged files against LOCKED_KEYS-owning paths (elai_cc.toml + any file
   matching *safety_locks* *auditor_config* *cloud_monthly_budget* *benchmark_harness*
   *test_suite* *score_weights* *gate_definitions* *lock_list* *cloud_opus_allowed*
   *cloud_escalation_threshold* *config_variants_schema* patterns). On match: exit 1
   with "BLOCKED: commit modifies LOCKED_KEYS file: <path> (spec phase-4H.txt:201-204)".
   Hook is not committed; installer script is.
   Files: .elai_cc/scripts/install_pre_commit_hook.sh

4. **Wire dep-audit output to FullPipelineOutcome.stale_api_catches**
   Add dep_audit_to_stale_catches(report: &DepAuditReport) -> Vec<StaleApiCatch> in
   .elai_cc/crates/cli/src/dep_audit.rs. Populate stale_api_catches in
   FullPipelineOutcome builder sites (self_mod_types.rs:562, 587, 612) when dep_audit
   results are available. Field stays empty if dep_audit not run for this pipeline.
   Files: .elai_cc/crates/cli/src/dep_audit.rs,
   .elai_cc/crates/orchestrator/src/self_mod_types.rs

5. **Wire steering vector refresh after deprecated_patterns Layer1A promote**
   In self_mod_bridge.rs after DataTable::DeprecatedPatterns promote (L79): spawn
   std::process::Command for metrics/compute_steering_vector.py (fire-and-forget, log
   PID). Emit SSB row action_type "steering_vector_refresh_triggered". Gate: only on
   successful promote. tracing::warn! if script not found — never panic.
   Files: .elai_cc/crates/orchestrator/src/self_mod_bridge.rs,
   .elai_cc/crates/orchestrator/src/ssb.rs

6. **Wire nightly dep-audit trigger from Layer1A DIVERGING state**
   At DIVERGING pause site (self_mod_layer1a.rs:891), call trigger_dep_audit_background()
   that spawns `elai dep-audit --output-format toml` detached. Output to
   .elai_cc/metrics/dep_audit_latest.toml. Rate-limit: sentinel file
   .elai_cc/metrics/.dep_audit_last_trigger (skip if mtime < 24h). No blocking.
   Files: .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs

7. **Replace AllowedConfigKey::Parameter(String) with ParameterKey closed enum**
   Add ParameterKey enum (12 variants) to self_mod_layer1a.rs:
     EtcThreshold, FreshnessHalfLife, FreshnessDocHalfLife, FreshnessBoostNorm,
     BabyStepMaxLoc { vram_tier: u8 }, ContextSlotWeight { slot: u8 },
     RetryBudget, AbMctsMaxBranches, AbMctsTemperatures, MicroReflectionMaxTokens,
     SteeringVectorCoefficient, SelfModFrequency.
   Replace AllowedConfigKey::Parameter(String) with Parameter(ParameterKey).
   Update all match arms in self_mod_layer1a.rs, self_mod_safety.rs, self_mod_bridge.rs,
   self_mod_orchestrator.rs, self_mod_types.rs (SelfModProposal.target_config_key).
   AllowedConfigKey::Parameter("cloud_monthly_budget") must not compile.
   Files: .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs,
   .elai_cc/crates/orchestrator/src/self_mod_safety.rs,
   .elai_cc/crates/orchestrator/src/self_mod_types.rs,
   .elai_cc/crates/orchestrator/src/self_mod_bridge.rs,
   .elai_cc/crates/orchestrator/src/self_mod_orchestrator.rs

8. **Implement ConstraintsBlock auto-population from rules.db**
   In .elai_cc/crates/knowledge/src/project_profile.rs (L1117-1126), query
   deprecated_patterns WHERE severity='BLOCK' (rules/src/schema.rs:900-931). Convert
   each row to ConstraintEntry: "DO NOT USE {deprecated_name}. Use {replacement_name}."
   Append to TOML-override constraints. De-duplicate on deprecated_name. Silent fallthrough
   if rules.db unavailable.
   Files: .elai_cc/crates/knowledge/src/project_profile.rs,
   .elai_cc/crates/rules/src/predicate.rs

9. **Create benchmarks/swe_bench/ with >=10 SWE-bench fixtures**
   Create .elai_cc/benchmarks/swe_bench/. Contents:
   - trigger_task.toml: canary with known-answer, gates StageReport.trigger_task_passed.
   - task_01.toml .. task_10.toml: SWE-bench Lite problem statements (MIT,
     github.com/princeton-nlp/SWE-bench). Fields: id, description, expected_output,
     timeout_s, stage (1|2|3), is_canary. No solution patches included.
   - README.txt: one plain paragraph on fixture format and canary role.
   Files: .elai_cc/benchmarks/swe_bench/{trigger_task.toml, task_01..10.toml, README.txt}

10. **Wire MistakeBook recording from T1/T2 BLOCK events**
    In task_runner.rs synthetic Failed reason at L5352-5363, embed tier discriminant
    string "T1_BLOCK" / "T2_BLOCK" / "PRE_TOOL_BLOCK" based on which hook returned the
    block (T1Hook::evaluate vs T2Hook::check_deprecated_ast_imports vs generic PreToolUse).
    Ensures record_mistake_on_failure (L5977) writes a categorized MistakeEntry. Update
    production_agent_rpc.rs if hook tier is surfaced through RPC response.
    Files: .elai_cc/crates/orchestrator/src/task_runner.rs,
    .elai_cc/crates/orchestrator/src/production_agent_rpc.rs

## WIRING_CONTRACT

produces_symbols:
  - anti-stale CI step in .elai_cc/.github/workflows/windows.yml
  - install_pre_commit_hook.sh + .git/hooks/pre-commit behavior
  - dep_audit_to_stale_catches() in cli/src/dep_audit.rs
  - trigger_dep_audit_background() in self_mod_layer1a.rs
  - ParameterKey enum (12 variants)
  - ConstraintsBlock auto-population from rules.db
  - .elai_cc/benchmarks/swe_bench/ (11 TOML files)
  - hook_block_tier discriminant in task_runner synthetic failure reason
consumes_symbols:
  - check_anti_stale.sh STRICT exit-code contract
  - LOCKED_KEYS const L70, KNOWN_PARAMETER_KEYS L38 (both replaced by ParameterKey)
  - FullPipelineOutcome.stale_api_catches (self_mod_types.rs:185)
  - DataTable::DeprecatedPatterns promote path (self_mod_bridge.rs:79)
  - MSI_DIVERGING_THRESHOLD (self_mod_layer1a.rs:80)
  - compute_steering_vector.py, deprecated_patterns query (schema.rs:900-931)
  - ConstraintsBlock::from_entries (context_budget.rs:1426)
  - record_mistake_on_failure (task_runner.rs:7909)
reachable_from:
  - GitHub Actions PR (anti-stale CI step)
  - git commit (pre-commit hook)
  - Layer1A DIVERGING state, deprecated_patterns promote, task_runner failure path
stub_ban:
  - ParameterKey must have exactly 12 variants — no String fallback
  - pre-commit hook exit 1 (not 0) on LOCKED_KEYS match
  - dep_audit_to_stale_catches uses real dep_audit output, not mock data
  - trigger_dep_audit_background must not block calling thread
observable_effect:
  trigger: cloud Sonnet 4.6 promotes deprecated_patterns entry
  expect: self_mod_bridge spawns compute_steering_vector.py; modern_api.npz mtime changes
  trigger: `git commit` with staged LOCKED_KEYS file change
  expect: pre-commit exits 1, message contains "BLOCKED: commit modifies LOCKED_KEYS file"
  trigger: cargo build with AllowedConfigKey::Parameter("cloud_monthly_budget")
  expect: compile error (ParameterKey enum has no String variant)
  trigger: dep-audit in DIVERGING state pipeline
  expect: dep_audit_latest.toml written; stale_api_catches non-empty

## DETERMINISTIC_CHECK

```bash
# 1. Anti-stale step wired
grep -c "check_anti_stale" .elai_cc/.github/workflows/windows.yml
# Expected: >= 1

# 2. ParameterKey 12 variants present
rg "EtcThreshold|FreshnessHalfLife|FreshnessDocHalfLife|FreshnessBoostNorm|BabyStepMaxLoc|ContextSlotWeight|RetryBudget|AbMctsMaxBranches|AbMctsTemperatures|MicroReflectionMaxTokens|SteeringVectorCoefficient|SelfModFrequency" .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs | wc -l
# Expected: 12

# 3. No raw string Parameter construction in production
rg 'AllowedConfigKey::Parameter\("' .elai_cc/crates/orchestrator/src/
# Expected: 0

# 4. Hook installer present
test -f .elai_cc/scripts/install_pre_commit_hook.sh && echo OK

# 5. dep_audit_to_stale_catches defined
rg "fn dep_audit_to_stale_catches" .elai_cc/crates/cli/src/dep_audit.rs
# Expected: 1 hit

# 6. trigger_dep_audit_background defined
rg "fn trigger_dep_audit_background" .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs
# Expected: 1 hit

# 7. benchmarks/swe_bench/ has 11 TOML files
ls .elai_cc/benchmarks/swe_bench/*.toml | wc -l
# Expected: 11

# 8. Constraints auto-population touches deprecated_patterns
rg "deprecated_patterns|query_block_patterns" .elai_cc/crates/knowledge/src/project_profile.rs
# Expected: >= 1

# 9. Compile clean
cargo check -p elai-orchestrator --locked 2>&1 | grep "^error" | wc -l
# Expected: 0

# 10. T1/T2 discriminant in task_runner
rg "T1_BLOCK|T2_BLOCK|PRE_TOOL_BLOCK" .elai_cc/crates/orchestrator/src/task_runner.rs
# Expected: >= 2
```

## ACCEPTANCE

- [ ] Promote deprecated_patterns entry via cloud Sonnet 4.6 task; assert modern_api.npz
      mtime changes within 60s of promote; SSB row "steering_vector_refresh_triggered" present.
- [ ] `git commit` with intentional LOCKED_KEYS change; assert pre-commit exits 1 with
      "BLOCKED: commit modifies LOCKED_KEYS file" message.
- [ ] `cargo build` with AllowedConfigKey::Parameter("cloud_monthly_budget") in test file;
      assert compile error (closed enum rejects unknown key).
- [ ] DIVERGING-state pipeline triggers dep_audit_latest.toml write;
      FullPipelineOutcome.stale_api_catches non-empty when stale deps exist.
- [ ] Open PR; anti-stale CI job runs, exits 0 in Actions log.
- [ ] `cargo test -p elai-orchestrator -- parameter_key 2>&1 | grep "test result: ok"`.
- [ ] B-16B Chunk C (conditional): if approved, zero .unwrap() in production paths of
      deterministic_auditor.rs; tests pass.

## Files touched

- MODIFY: .elai_cc/.github/workflows/windows.yml
- ADD: .elai_cc/scripts/install_pre_commit_hook.sh
- MODIFY: .elai_cc/crates/cli/src/dep_audit.rs
- MODIFY: .elai_cc/crates/orchestrator/src/self_mod_types.rs
- MODIFY: .elai_cc/crates/orchestrator/src/self_mod_bridge.rs
- MODIFY: .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs
- MODIFY: .elai_cc/crates/orchestrator/src/self_mod_safety.rs
- MODIFY: .elai_cc/crates/orchestrator/src/self_mod_orchestrator.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
- MODIFY: .elai_cc/crates/knowledge/src/project_profile.rs
- MODIFY: .elai_cc/crates/rules/src/predicate.rs
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs
- MODIFY: .elai_cc/crates/orchestrator/src/production_agent_rpc.rs
- ADD: .elai_cc/benchmarks/swe_bench/trigger_task.toml
- ADD: .elai_cc/benchmarks/swe_bench/task_01.toml .. task_10.toml
- ADD: .elai_cc/benchmarks/swe_bench/README.txt
- CONDITIONAL: .elai_cc/crates/orchestrator/src/deterministic_auditor.rs (Chunk C)
- CONDITIONAL: .elai_cc/crates/orchestrator/src/dkg/*.rs (Chunk C cascade)
- CREATE: workflow/reasoning/C-21-decisions.md

## Next

C-22 (TBD) or B-plan follow-up per bn_status_report.py punch list after C-21 ships.
