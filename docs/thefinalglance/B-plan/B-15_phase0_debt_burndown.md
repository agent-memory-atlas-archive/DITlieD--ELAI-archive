# B-15: Phase 0 Debt Burndown
Status (2026-07-03 rebaseline, session s-25120 completion-drive 2):
  11 of 18 tasks SHIPPED-ON-V2 (Tasks 4, 7, 9, 10, 11, 12, 13, 14, 16, 17, 18).
  5 of 18 tasks DEFERRED BY HANDOFF to follow-up plans (Tasks 1, 2, 3, 5, 15).
  1 of 18 tasks ACTIVE-CLOSED prior session (Task 6 HashMap ban, 8 of 17 prod
    files closed; residuals regrew to 21 production violations routed to
    B-15-followup).
  1 of 18 tasks REGRESSED + DEFERRED (Task 8 entropy; reduced from 82 files
    at prior rebaseline to 1 leak site (task_runner.rs:15311) via Tranche H
    + C-15; residual routed to B-15-regression-burndown at reduced scope).
  NEW TRIPWIRE FIRED: cli/main.rs at 1632 LOC vs spec 500 (was 555 at
    prior rebaseline; regrew via H/I/J/K/L dispatch arms); cli-shrink
    follow-up routed.

Decision doc: workflow/reasoning/B-15-decisions.md (completion-drive 1
rebaseline supersedes prior s-38324 entries).
Followup: workflow/reasoning/B-15-followup.md (routes regression +
out-of-scope items to dedicated burndown plan).

Original objective (preserved): strip the dead code Phase 0 was supposed
to remove, replace 84 std HashMap imports, fix 40+ entropy quarantine
leaks, decompose cli/main.rs (currently 22x over spec target), restore
missing scripts.

Original estimate: 2-3 sessions (broad cross-crate refactor).
Actual to date: 5 sub-phases (B-15A / B-15B / B-15B-FX / B-15C / B-15
batch G) plus this completion-drive 1.

Pre-requisites: none.
Parallel-safe with: B-1..B-14, B-16..B-20.

## Context (rebaseline 2026-07-03)

Per audit-implementation/phase-0.md and _AGGREGATE.md PATTERN 9, plus
all sub-phase handoffs and gotchas.md "B-15:DEAD-CODE-STALE" entry:

Phase 0 said STRIP these. Status across SHIPPED + DEFERRED:
- crates/orchestrator/src/compact.rs (819 LOC) -- DEFERRED to B-9
    (6 live callers + /compact REPL + ConversationRuntime::compact)
- crates/core/src/usage.rs (98 LOC, was 279 LOC) -- DEFERRED to B-27
    (pricing helpers ALREADY STRIPPED; 4-field residual is load-bearing
    on /cost + JSON "estimated_cost" until B-27 SSB CLOUD_SPEND lands)
- crates/providers/src/oauth.rs (652 LOC) -- DEFERRED to B-27
    (15 callers + /login Anthropic OAuth flow)
- crates/orchestrator/src/remote.rs -- SHIPPED (deleted in B-15A s-28972)
- crates/orchestrator/src/prompt.rs (829 LOC) -- DEFERRED to B-15D
    (6 system-prompt-build callers + ConversationRuntime construction)

Plus violations (current state, re-verified 2026-07-03):
- HashMap imports: was 84 at plan time, was 5 at prior rebaseline, NOW
    21 production violations (16 use-stmt + 5 qualified-path) across
    plugins/knowledge/providers/safety/orchestrator/dashboard-v2.
    HashSet ban not separately enforced (10+ residuals; routed to followup).
- Entropy quarantine leaks: was 40+ at plan time, was 82 at prior
    rebaseline, NOW 1 leak site (orchestrator/src/task_runner.rs:15311
    Instant::now) -- regression largely closed by Tranche H + C-15.
- cli/src/main.rs: was 11,181 LOC at plan time, was 555 LOC at prior
    rebaseline, NOW 1632 LOC (regrew via H/I/J/K/L dispatch arms;
    tripwire at 600 CROSSED; cli-shrink follow-up routed).
- file_ops.rs / bash_validation.rs / stale_branch.rs:
    SHIPPED (moved orchestrator/ -> tools/ in B-15 batch G s-38324).
- bash_validation.rs duplicate: SHIPPED (resolved in batch G).
- Session persistence msgpack (Phase 0.7b): DEFERRED to B-9.
- Scripts: SHIPPED (check_entropy_quarantine.sh + determinism_proof.sh
    + smoke_test_ollama.sh + check_hashmap_ban.sh).
- pre_phase_0.toml: SHIPPED.
- hooks.rs:634 Windows "sh" bug: SHIPPED (cfg-gated at hooks.rs:758-771).

## Reads

- audit-implementation/phase-0.md (original audit baseline 2026-04-24)
- audit-implementation/pre-phase-0.md
- audit-implementation/_AGGREGATE.md PATTERN 9
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-0.txt
- F:\.ELAI_workflow\docs\specs\guide\steps\pre-phase-0.txt
- workflow/reasoning/B-15-decisions.md (completion-drive rebaseline)
- workflow/reasoning/B-15-followup.md (regression routing)
- workflow/memory-bank/gotchas.md ("B-15:DEAD-CODE-STALE" + "B-15:HASHMAP-BAN-CASCADE"
    + "B-15:JSON-MAP-ORDER" + "B-15:LATENT-WINDOWS-OAUTH")

## Tasks

### Strip dead code

1. **[DEFERRED -> B-9] Delete compact.rs (819 LOC).**
   Caller-audit: 6 live production callers identified
   (orchestrator/conversation.rs::compact_history,
    orchestrator/lib.rs::pub mod compact, /compact REPL command,
    ConversationRuntime::compact, plus 2 transitive sites).
   Defer rationale: B-9 SessionDag is the proper replacement; B-9
   takeover will retire the callers first, then this file can go.
   Evidence: ls .elai_cc/crates/orchestrator/src/compact.rs returns
   24432-byte file dated 2026-04-07.

2. **[DEFERRED -> B-27] Reduce usage.rs to spec minimum.**
   Status: PARTIAL (pricing helpers ModelPricing, UsageCostEstimate,
   pricing_for_model, format_usd ALREADY STRIPPED per file header
   comment lines 1-11). Residual: 4-field TokenUsage{input_tokens,
   output_tokens, cache_creation_input_tokens, cache_read_input_tokens}
   + UsageTracker. Current LOC 98; spec target was <30 LOC with
   only 2-field TokenUsage.
   Defer rationale: 12 importers + live /cost slash command + JSON
   turn-summary "estimated_cost" key still consume the 4-field
   struct. B-27 Cloud Mode replaces ad-hoc cost tracking with SSB
   CLOUD_SPEND; spec minimum unblocks after that ships.

3. **[DEFERRED -> B-27] Delete oauth.rs.**
   Caller-audit: 15 production callers (providers/client.rs +
   providers/lib.rs + providers/providers/anthropic.rs + entire
   /login OAuth flow). Latent bug at oauth.rs:~388 credentials_home_dir
   PowerShell-HOME bug (gotchas.md) proves the file is live.
   Current LOC: 652.
   Defer rationale: B-27 Cloud Mode owns the auth surface;
   coordinate removal with that plan.

4. **[SHIPPED-ON-V2] Delete remote.rs.**
   Shipped in B-15A s-28972. File absent: ls returns "No such file
   or directory" for .elai_cc/crates/orchestrator/src/remote.rs.

5. **[DEFERRED -> B-15D] Delete prompt.rs.**
   Caller-audit: 6 system-prompt-build callers + ConversationRuntime
   construction (orchestrator/conversation.rs + orchestrator/lib.rs +
   SystemPromptBuilder).
   Current LOC: 829.
   Defer rationale: spec said replace with elai_cc.toml + Rust code;
   B-15D will migrate consumers first.

### HashMap ban

6. **[ACTIVE-CLOSED] Replace HashMap imports (re-scoped + closed for 8 of 17 prod files).**

   Citation correction: the prior text "Per CLAUDE.md Anti-Stale-Training"
   was UNGROUNDED. CLAUDE.md contains zero HashMap / BTreeMap / IndexMap /
   FxHashMap tokens (grep verified). The actual citation chain is:
   (1) audit-implementation/_AGGREGATE.md PATTERN 9 -- baseline audit
   identified 84 imports as a determinism risk;
   (2) memory-bank/gotchas.md "B-15:JSON-MAP-ORDER" -- pins
   HashMap/FxHashMap as NON-DETERMINISTIC for JSON serialization.

   Per-file decision rules:
   - Iteration order matters OR cross-process serialization -> BTreeMap
   - Lookup-only hot path with no iteration -> FxHashMap (rustc-hash;
     already in workspace at Cargo.toml:46)
   - Insertion order preservation -> IndexMap (not used this session)

   Closed this session (8 conversions):
   - core/file_plan.rs LanguageLimitsMap -> BTreeMap
   - knowledge/deep.rs scores -> BTreeMap (cascade: pipeline.rs +
     contracts/tests/deep_pool_cap_contract.rs +
     contracts/tests/c31_file_plan_contract.rs)
   - orchestrator/observable_effect.rs fixtures -> BTreeMap
   - orchestrator/sandbox/firecracker.rs snapshots -> BTreeMap
   - orchestrator/checker_registry.rs entries -> BTreeMap
   - orchestrator/compiled_pipeline.rs observation_log / pipeline_cache /
     consecutive_failures / CompiledPipelineStore.entries -> BTreeMap
   - orchestrator/oracle_cache.rs LRU map -> FxHashMap
   - orchestrator/background_task.rs active_tasks -> FxHashMap
   - plugins/mcp/lazy.rs registry param -> FxHashMap (cascade:
     plugins/tests/tool_gate.rs)
   - cli/live_cli.rs prev_breaker_states -> BTreeMap
   - orchestrator/dispatch/mod.rs -> unused import deleted

   Intentional retentions (allow-listed in check_hashmap_ban.sh):
   - orchestrator/dual_ledger.rs (IPC boundary; serde needs HashMap)
   - orchestrator/reasoning_template.rs (explicit doc-link)
   - providers/runtime/sidecar_xgrammar_hook.rs (cfg(test) inside src/)

   Residual 21 violations (16 use-stmt + 5 qualified-path across
   plugins/knowledge/providers/safety/orchestrator/dashboard-v2):
   routed to B-15-followup per executor instruction
   ("DO NOT touch dashboard-v2 Tauri command files -- coordinated
   with B-3/B-4/B-7 ships").

   Per-conversion rationale in workflow/reasoning/B-15-decisions.md
   section (c).

7. **[SHIPPED-ON-V2] Add CI lint.**
   .elai_cc/.github/workflows/windows.yml:54-56 has step "HashMap ban
   check" running scripts/check_hashmap_ban.sh; script is 6191 bytes,
   executable. Shipped in B-15B s-27276 and B-15 batch G s-38324.

### Entropy quarantine

8. **[LARGELY-CLOSED + DEFERRED -> B-15-regression-burndown] Entropy leak burndown.**

   Original target: 40+ leaks at plan time (2026-04-24 audit).
   B-15C-GAPS handoff (2026-04-29): 28 production residual after
   deferring 23 HIGH-risk sites (sandbox.rs / task_runner.rs /
   dashboard/commands.rs) to B-15C-HIGH.
   B-15 batch G close (2026-05-08): 16 production residual after
   script tightening + checkpoint TEST_KEEP fix.
   Prior rebaseline (2026-05-22): 82 production files / 192 occurrences
   -- REGRESSED 5x over the post-batch-G floor.
   This session (2026-07-03): 1 leak site remains
   (orchestrator/src/task_runner.rs:15311 Instant::now) -- regression
   largely closed by Tranche H + C-15 producer-wire ClockProvider
   thread-through.

   Root cause (historical): B-4 / B-7 / D-* shipped producer wires that
   emit SystemTime::now() / Instant::now() in hot paths without
   ClockProvider injection. ~4.7 new leak sites per day at peak.

   This session's scope: ZERO new conversions. Reasons:
   (1) The residual is now a single site (task_runner.rs:15311); a
       targeted fix is the proper scope of B-15-regression-burndown
       at reduced effort (1 session, not 3-5).
   (2) Dashboard-v2 Tauri command files explicitly excluded per
       executor instruction (coordinate with B-3 / B-4 / B-7 ships).

   Forward path: targeted ClockProvider thread-through for the single
   residual site in B-15-regression-burndown (1-session effort given
   the 1-file scope). Full root-cause analysis + per-producer fix
   shape lives in workflow/reasoning/B-15-followup.md section (1).

   ClockProvider trait at .elai_cc/crates/core/src/clock.rs:43 is
   stable + complete (now() / elapsed_since() / system_now() /
   instant_now() / now_ms() / now_secs()); SeededClock production
   implementation at line 122.

9. **[SHIPPED-ON-V2] scripts/check_entropy_quarantine.sh.**
   .elai_cc/scripts/check_entropy_quarantine.sh (4954 bytes,
   executable). Shipped in B-15A s-28972. W4 ELAI_DETERMINISM_SEED
   unused -- doc-only note, not breaking.

10. **[SHIPPED-ON-V2] scripts/determinism_proof.sh.**
    .elai_cc/scripts/determinism_proof.sh (4043 bytes, executable).
    Shipped in B-15A s-28972.

11. **[SHIPPED-ON-V2] scripts/smoke_test_ollama.sh.**
    .elai_cc/scripts/smoke_test_ollama.sh (2772 bytes, executable).
    Shipped in B-15A s-28972. W2 fallback-substring fragility note
    is doc-only, not breaking.

### CLI decomposition

12. **[SHIPPED-ON-V2 -> TRIPWIRE-FIRED] Decompose cli/src/main.rs.**
    Prior rebaseline state: 555 LOC (target was <=500; +55 delta = +11%
    over, accept-as-shipped).
    Current state (2026-07-03): 1632 LOC -- TRIPWIRE FIRED at 600.
    main.rs regrew via H/I/J/K/L dispatch arms added after the prior
    rebaseline. The 6 sibling modules remain present:
    - crates/cli/src/repl.rs (8692 bytes)
    - crates/cli/src/commands.rs (197989 bytes)
    - crates/cli/src/sidecar.rs (10230 bytes)
    - crates/cli/src/indexing.rs (14023 bytes)
    - crates/cli/src/handoff.rs (26475 bytes)
    - crates/cli/src/dashboard.rs (5501 bytes)
    The accept-as-shipped rationale (B-15-decisions.md section (e))
    applied to the 555-LOC state and is superseded by the tripwire
    firing. A new B-15-cli-shrink follow-up plan is warranted to
    re-decompose main.rs back under 600 LOC.

### File reorganization

13. **[SHIPPED-ON-V2] Move file_ops.rs / bash_validation.rs /
    stale_branch.rs from orchestrator/ to tools/.**
    .elai_cc/crates/tools/src/{file_ops,bash_validation,stale_branch}.rs
    all present (27486 / 29833 / 12758 bytes). orchestrator/src/
    counterparts ABSENT. Shipped in B-15 batch G s-38324.

14. **[SHIPPED-ON-V2] Delete duplicate bash_validation.rs.**
    Only tools/src/bash_validation.rs remains. Subsumed by Task 13
    (move-not-copy semantics). Shipped in B-15 batch G s-38324.

### Session msgpack

15. **[DEFERRED -> B-9] Phase 0.7b msgpack for disk sessions.**
    Plan itself acknowledges partial defer ("Already covered partially
    by B-9 (SessionDag activation), but for legacy session.rs that's
    deprecated, keep JSONL only behind --export-jsonl flag"). B-9
    SessionDag takeover deprecates the legacy path entirely.

### Other

16. **[SHIPPED-ON-V2] Fix hooks.rs Windows "sh" bug.**
    .elai_cc/crates/orchestrator/src/hooks.rs:758-771 shell_command()
    uses #[cfg(windows)] Command::new("cmd").arg("/C") and
    #[cfg(not(windows))] Command::new("sh").arg("-c"). Shipped in
    B-15A s-28972. Line drift from spec line 634 to 758-771 noted.
    W1 sh -c vs sh -lc divergence with plugins/hooks.rs documented
    but not breaking.

17. **[SHIPPED-ON-V2] Generate .elai_cc/metrics/pre_phase_0.toml.**
    File present (2683 bytes, dated 2026-04-26). Shipped in B-15A
    s-28972. W3 warnings field naming nit is doc-only.

18. **[SHIPPED-ON-V2] Add CI workflow.**
    .elai_cc/.github/workflows/windows.yml self-tagged "B-15A Task 18"
    in the header. Steps: cargo check --workspace + HashMap ban check
    + entropy quarantine check + anti-stale lint + hooks compat test
    + wiring contract audit. Shipped in B-15A s-28972 + B-15 batch G
    s-38324. Placement disputed in s-38324 audit; re-check this
    session UPGRADED to IMPLEMENTED.

## WIRING_CONTRACT (rebaselined)

produces_symbols:
  - crates/cli/src/repl.rs::run_repl
  - crates/cli/src/commands.rs
  - crates/cli/src/remote_sidecar.rs::run_remote_sidecar
  - crates/cli/src/indexing.rs
  - crates/cli/src/handoff.rs::list_managed_sessions
  - crates/cli/src/dashboard.rs::run_dashboard
consumes_symbols:
  - crates/cli/src/main.rs::run (1632 LOC; tripwire FIRED at 600;
    cli-shrink follow-up routed).
  - tests/contracts/ updated module paths (B-15 batch G).
reachable_from:
  - crates/cli/src/main.rs::main
  - .github/workflows/windows.yml (CI runner).
stub_ban (rebaselined):
  - compact.rs / oauth.rs / prompt.rs PRESENT but DEFERRED with documented
    live-caller counts (6 + 15 + 6) and named follow-up phases (B-9 / B-27
    / B-15D). Per gotchas.md B-15:DEAD-CODE-STALE, blind deletion is
    documented as a recurring failure pattern.
  - usage.rs PRESENT (98 LOC) with pricing helpers already stripped;
    spec-minimum reduction DEFERRED to B-27.
  - remote.rs ABSENT -- SHIPPED.
  - 21 production HashMap violations remain (16 use-stmt + 5 qualified;
    across plugins/knowledge/providers/safety/orchestrator/dashboard-v2)
    -- routed to B-15-followup.
  - 1 production entropy leak site remains
    (orchestrator/src/task_runner.rs:15311 Instant::now) -- regression
    largely closed by Tranche H + C-15; residual routed to
    B-15-regression-burndown follow-up plan at reduced scope.
  - cli/main.rs 1632 LOC vs spec 500 -- tripwire FIRED at 600;
    accept-as-shipped superseded; cli-shrink follow-up routed.
frontend_pair: null
observable_effect:
  trigger: cd F:\.ELAI_workflow\.elai_cc && cargo test -p elai_cc_contracts --test fsm_phase_transition_smoke_ssb -- --nocapture
  expect_artifact: workspace builds green; touched crates compile clean;
                   HashMap ban check shows residuals only in routed-defer
                   set; entropy check reports 1 leak site (task_runner.rs).
  expect_ssb: phase_transition

## DETERMINISTIC_CHECK (rebaselined to current paths)

```bash
# 1. Stripped files status (mix of SHIPPED + DEFERRED-with-rationale).
test ! -f .elai_cc/crates/orchestrator/src/remote.rs                 # SHIPPED
test -f .elai_cc/crates/orchestrator/src/compact.rs && echo "DEFERRED B-9"
test -f .elai_cc/crates/providers/src/oauth.rs && echo "DEFERRED B-27"
test -f .elai_cc/crates/orchestrator/src/prompt.rs && echo "DEFERRED B-15D"

# 2. usage.rs status (98 LOC; pricing helpers stripped).
wc -l .elai_cc/crates/core/src/usage.rs
# Expected: 98 lines (pricing helpers removed; 4-field TokenUsage
# + UsageTracker remain; spec-min reduction DEFERRED to B-27).

# 3. HashMap ban (residuals across plugins/knowledge/providers/safety/
#    orchestrator/dashboard-v2).
bash .elai_cc/scripts/check_hashmap_ban.sh
# Expected: 21 production violations (16 use-stmt + 5 qualified-path)
# routed to B-15-followup.

# 4. Entropy quarantine (largely closed -- 1 leak site remains).
bash .elai_cc/scripts/check_entropy_quarantine.sh
# Expected: 1 leak site (orchestrator/src/task_runner.rs:15311
# Instant::now) routed to B-15-regression-burndown follow-up plan.

# 5. cli/main.rs size (TRIPWIRE FIRED at 600; was 555, now 1632).
wc -l .elai_cc/crates/cli/src/main.rs
# Expected: 1632 lines (regrew via H/I/J/K/L dispatch arms; tripwire
# at 600 CROSSED; cli-shrink follow-up plan warranted).

# 6. Scripts present + executable (SHIPPED).
test -x .elai_cc/scripts/check_entropy_quarantine.sh
test -x .elai_cc/scripts/determinism_proof.sh
test -x .elai_cc/scripts/smoke_test_ollama.sh
test -x .elai_cc/scripts/check_hashmap_ban.sh

# 7. Pre-phase-0 metrics TOML (SHIPPED).
test -f .elai_cc/metrics/pre_phase_0.toml

# 8. tools/ relocation (SHIPPED).
test -f .elai_cc/crates/tools/src/file_ops.rs
test -f .elai_cc/crates/tools/src/bash_validation.rs
test -f .elai_cc/crates/tools/src/stale_branch.rs
test ! -f .elai_cc/crates/orchestrator/src/file_ops.rs
test ! -f .elai_cc/crates/orchestrator/src/bash_validation.rs
test ! -f .elai_cc/crates/orchestrator/src/stale_branch.rs

# 9. CI workflow (SHIPPED).
test -f .elai_cc/.github/workflows/windows.yml

# 10. hooks.rs Windows fix (SHIPPED).
grep -q 'cfg(windows)' .elai_cc/crates/orchestrator/src/hooks.rs
```

## ACCEPTANCE (rebaselined 2026-07-03)

- [x] Task 4 remote.rs DELETED -- SHIPPED.
- [DEFERRED] Tasks 1/2/3/5 (compact/usage/oauth/prompt) deferred to
  B-9 / B-27 / B-15D with documented live-caller counts.
- [PARTIAL] Task 6 HashMap ban: 8 of 17 prod files closed in prior
  session; 3 intentional retentions allow-listed; 21 production
  residuals (16 use-stmt + 5 qualified) routed to B-15-followup.
- [LARGELY-CLOSED] Task 8 entropy: reduced from 82 files at prior
  rebaseline to 1 leak site (task_runner.rs:15311) via Tranche H +
  C-15; residual routed to B-15-regression-burndown at reduced scope.
- [TRIPWIRE-FIRED] cli/main.rs <=500 LOC: was 555 (accept-as-shipped),
  now 1632 (regrew via H/I/J/K/L dispatch arms); tripwire at 600
  CROSSED; cli-shrink follow-up plan warranted.
- [x] file_ops/bash_validation/stale_branch in tools/, not orchestrator/.
- [x] No duplicate bash_validation.rs.
- [x] hooks.rs Windows "sh" bug fixed.
- [x] Pre-phase-0 metrics TOML generated.
- [x] CI workflow runs on Windows.
- [x] All 3 scripts present and executable.
- [x] check_hashmap_ban.sh present and CI-wired (Task 7).

## Files touched (this completion-drive)

This completion-drive modified ONLY this plan file (markdown rebaseline + WIRING_CONTRACT fix). The list below is retained from prior batch G (s-38324) for reference only.

- MODIFIED: 10 .rs files for HashMap conversions
  - core/file_plan.rs (HashMap -> BTreeMap)
  - knowledge/deep.rs + knowledge/pipeline.rs (HashMap -> BTreeMap; cascade)
  - knowledge/tests/deep_pool_cap_contract.rs (cascade)
  - contracts/tests/c31_file_plan_contract.rs (cascade)
  - orchestrator/observable_effect.rs (HashMap -> BTreeMap)
  - orchestrator/sandbox/firecracker.rs (HashMap -> BTreeMap; SnapshotKey
    gains Ord)
  - orchestrator/checker_registry.rs (HashMap -> BTreeMap)
  - orchestrator/compiled_pipeline.rs (HashMap -> BTreeMap, 4 sites +
    CompiledPipelineStore)
  - orchestrator/oracle_cache.rs (HashMap -> FxHashMap)
  - orchestrator/background_task.rs (HashMap -> FxHashMap)
  - plugins/mcp/lazy.rs + plugins/tests/tool_gate.rs (HashMap -> FxHashMap;
    cascade)
  - orchestrator/dispatch/mod.rs (unused HashMap import removed)
  - cli/live_cli.rs (qualified HashMap path -> BTreeMap)
- MODIFIED: 1 script
  - scripts/check_hashmap_ban.sh (allow-list extended with dual_ledger,
    reasoning_template, sidecar_xgrammar_hook)
- MODIFIED: 1 plan file
  - docs/thefinalglance/B-plan/B-15_phase0_debt_burndown.md
    (this file -- full rebaseline)
- CREATED: workflow/reasoning/B-15-decisions.md (rewritten)
- CREATED: workflow/reasoning/B-15-followup.md (new)

## Decision doc

workflow/reasoning/B-15-decisions.md captures:
  (a) SHIPPED-ON-V2 markers across 11 tasks with file:line evidence.
  (b) DEFERRED-BY-HANDOFF markers for Tasks 1/2/3/5/15 with cross-plan
      handoff notes (B-9 / B-27 / B-15D).
  (c) Per-file HashMap conversion rationale (BTreeMap vs FxHashMap)
      and per-file false-positive / intentional-retention analysis.
  (d) Entropy regression scope-lock (zero conversions this session;
      1 residual leak site (task_runner.rs:15311) routed to
      B-15-regression-burndown follow-up at reduced scope).
  (e) cli/main.rs 1632 LOC tripwire-fired rationale (was 555 at prior
      rebaseline; regrew via H/I/J/K/L dispatch arms; cli-shrink
      follow-up routed).
  (f) Followup handoff pointer to B-15-followup.md.

## Next

- B-15-regression-burndown (reduced scope): targeted ClockProvider
  thread-through for the single residual entropy leak site
  (orchestrator/src/task_runner.rs:15311 Instant::now). 1-session effort.
- B-15-cli-shrink (new plan warranted): re-decompose cli/main.rs from
  1632 LOC back under 600 LOC (tripwire fired). Move H/I/J/K/L dispatch
  arms into sibling modules.
- Coordinate with B-3 / B-4 / B-7 maintainers on the 21 HashMap
  residual closure (16 use-stmt + 5 qualified across
  plugins/knowledge/providers/safety/orchestrator/dashboard-v2).
- B-16 (Anti-stale rule enforcement) per original B-15 sequencing.
