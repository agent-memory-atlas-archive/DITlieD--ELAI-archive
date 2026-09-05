COMPAT — .ELAI_workflow

Loaded by orchestrate when target_project_root matches F:\.ELAI_workflow (case-insensitive prefix match on the plan_path). Provides ELAI-specific override values for the boot prompt template and stack-adapter routing.

PROJECT IDENTITY

target_project_root: F:\.ELAI_workflow
target_project_name_slug: ELAI
workspace_root: F:\.ELAI_workflow\.elai_cc
project_boundary_rule: all code writes target .elai_cc/ subtree, NEVER write to workflow root or F:\.ELAI_workflow\.claude (workflow tooling only)
elai_bat: F:\.ELAI_workflow\.elai_cc\elai.bat

STACK

primary_stack: rust+typescript (composite)
  - rust workspace at .elai_cc/ with 16 crates (agent, cli, contracts, core, ipc-types, knowledge, orchestrator, plugins, providers, reasoning, rules, scaffold, tools, web_search, web_validator, plus dashboard-v2/src-tauri)
  - typescript+svelte dashboard at .elai_cc/crates/dashboard-v2/ (Svelte 5 + Vite + Tauri 2)
bindings_scheme: tauri-specta (.elai_cc/crates/dashboard-v2/src-tauri uses Tauri command annotations; bindings emitted to dashboard's src/lib/bindings/)
stack_adapters_to_load: [rust.md, typescript.md, cross-stack-rust-ts.md]

NATIVE ORCHESTRATION SKILL

native_orchestration_skill_path: .claude/skills/b-plan-runner/SKILL.md
native_orchestration_skill_invocation_phrase: "run b-plan-runner on <plan_path>"
native_anti_spiral_skill_path: .claude/skills/anti-spiral/SKILL.md  (auto-fires inside launched session; orchestrate observes only)
native_dashboard_handoff_skill_path: .claude/skills/dashboard-handoff/SKILL.md
native_impact_check_skill_path: .claude/skills/impact-check/SKILL.md
native_sidecar_protocol_skill_path: .claude/skills/sidecar-protocol/SKILL.md

CANONICAL AGENT LIST (per ELAI CLAUDE.md "Valid agent files" rule)

available_agents:
  - auditor.md
  - context-compiler.md
  - memory-writer.md
  - paper-ingestor.md
  - prompt-engineer.md
  - scope-planner-elai.md
  - integration-explorer-elai.md
  - synthesizer-elai.md
  - impact-mapper.md
  - spec-grounder.md
  - fixer.md
  - test-attributer.md
  - wiring-contract-validator.md
  - sidecar-health.md
  - dashboard-smoker.md
  - persona-hacker.md / persona-researcher.md / persona-simplifier.md / persona-architect.md / persona-contrarian.md  (anti-spiral switches)

NEVER use a name not on this list inside ELAI's launched session — ELAI's CLAUDE.md anti-slopsquatting rule blocks unknown agents at PreToolUse hook level.

POLICY OVERRIDES (filled into overrides-template.md)

parallel_cap: 3  (overrides ELAI's b-plan-runner implicit cap; user's brief said 3)
skip_prompt_engineer: true  (overrides b-plan-runner step 5; user override)
audit_cycles: 1  (overrides b-plan-runner step 9 which allows ONE re-spawn of auditor after fixer; we suppress that re-spawn — bubble-up replaces it)
commit_at_end_only: true  (overrides b-plan-runner step 13 which commits per B-N; we delay until ALL sub-phases in our run complete)
autonomous: true  (user's "dont bother me" default — only tripwires interrupt)
model_executor: claude-sonnet-4-6
register_session: true  (orchestrate registers ITS session before launch; launched session must register its own separately)

DASHBOARD WIRING

dashboard_wiring_paths:
  - .elai_cc/crates/dashboard-v2/src/         (Svelte 5 dashboard frontend — components, stores, lib)
  - .elai_cc/crates/dashboard-v2/src-tauri/   (Tauri command Rust side — bindings origin)
  - .elai_cc/crates/dashboard-v2/src/lib/bindings/  (auto-generated TS bindings — must NOT be edited by hand, only regenerated)

dashboard_wiring_paths_regex: '^\.elai_cc/crates/dashboard-v2/(src|src-tauri)/'
user_facing_capability_hints:
  - plan WIRING_CONTRACT.observable_effect.trigger non-empty
  - any new SsbActionType variant in .elai_cc/crates/core/src/ssb/action_types.rs
  - any new command in .elai_cc/crates/dashboard-v2/src-tauri/src/commands*.rs
  - any new view / drawer / panel in .elai_cc/crates/dashboard-v2/src/lib/drawers/ or src/lib/views/
  - plan acceptance mentions "dashboard", "drawer", "panel", "view"

backend_only_justification_marker:
  - explicit "backend-only-justification:" line in plan content suppresses XS5 dashboard-wiring gate

TEST COMMANDS

test_command_per_sub_phase: |
  Push-Location F:\.ELAI_workflow\.elai_cc
  $crates = $env:TOUCHED_CRATES -split ','
  foreach ($c in $crates) {
    cargo test -p $c --all-features
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
  }
  Pop-Location

test_command_full_sweep: |
  Push-Location F:\.ELAI_workflow\.elai_cc
  cargo test --workspace --all-features
  if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
  Pop-Location
  Push-Location F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2
  npm test 2>&1
  if ($LASTEXITCODE -ne 0) { Pop-Location; exit 2 }
  Pop-Location

test_command_real_model: |
  # ELAI's real-model integration tests use the remote sidecar (pythondev RTX 3090)
  # Before running: verify sidecar health per .claude/skills/sidecar-protocol/SKILL.md
  Push-Location F:\.ELAI_workflow\.elai_cc
  $env:ANTHROPIC_BASE_URL = "http://localhost:8100"
  $env:ANTHROPIC_API_KEY = "local-dummy"
  cargo test --workspace --all-features --features integration -- --ignored
  if ($LASTEXITCODE -ne 0) { Pop-Location; exit 3 }
  Pop-Location

BINDINGS EMIT

bindings_emit_command: |
  Push-Location F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri
  cargo test export_bindings  # Tauri-specta convention: bindings emitted during test-named-export_bindings run
  Pop-Location

bindings_paths:
  - .elai_cc/crates/dashboard-v2/src/lib/bindings/
  - .elai_cc/crates/dashboard-v2/src/lib/bindings.ts

bindings_drift_check: |
  & .claude\skills\orchestrate\compat\ELAI.md.bindings-emit  # internal — actually calls bindings_emit_command then git diff --exit-code -- $bindings_paths

DEPENDENCY-STATUS DETECTION (for orchestrate dep-wait protocol)

dep_status_tool: F:\.ELAI_workflow\.claude\tools\bn_status_report.py
dep_status_invocation: |
  python F:\.ELAI_workflow\.claude\tools\bn_status_report.py --filter all --json
  # Output: JSON list with B-N ids + status (SHIPPED | IN-FLIGHT | PLANNED | MISSING_FILE)

dep_status_done_predicate: status == "SHIPPED"
dep_status_inflight_predicate: status == "IN-FLIGHT"

dep_status_session_marker_paths:
  - F:\.ELAI_workflow\workflow\sessions\*\HANDOFF*.md
  - F:\.ELAI_workflow\workflow\sessions\*\SESSION_HANDOFF.md
  - F:\.ELAI_workflow\workflow\sessions\*\SESSION-HANDOFF.md
  (orchestrate's dep-wait reads these to find B-N status when bn_status_report.py is stale; multiple naming conventions exist historically)

dep_wait_poll_command: |
  python F:\.ELAI_workflow\.claude\tools\bn_status_report.py --filter all --json |
    ConvertFrom-Json | Where-Object { $_.id -eq $env:DEP_ID } | Select-Object -ExpandProperty status

CODEBASE MAP

codebase_map_path: F:\.ELAI_workflow\.claude\tools\elai.map
codebase_map_freshness_check: python F:\.ELAI_workflow\.claude\tools\elai_map_freshness.py
codebase_map_regen_command: |
  python F:\.ELAI_workflow\.claude\tools\codebase_mapper.py F:\.ELAI_workflow\.elai_cc F:\.ELAI_workflow\.claude\tools\elai_graph.json --compact --map F:\.ELAI_workflow\.claude\tools\elai.map --scip-dir F:\.ELAI_workflow\.claude\tools\scip_indices

HELPER TOOL INTEGRATION

orchestrate uses ELAI's existing harness tools where they apply:
  - impact_radius.py for pre-edit blast radius (called by integration-explorer-elai and impact-mapper agents — already wired)
  - wiring_contract_check.py at full sweep gate 4 (every plan's WIRING_CONTRACT block validated against actual symbols)
  - spec_link_check.py at per-sub-phase pre-gate (verify plan's "Spec references:" + "Reads (mandatory)" entries resolve)
  - elai_map_freshness.py before any sub-phase that depends on codebase map (regen if stale)
  - bn_status_report.py for dep-wait (above)
  - cross_lang_diff.py at per-sub-phase phase-end sweep (catch Rust↔Python sidecar OR Tauri↔Svelte drift)
  - failure_taxonomy.py inside the fixer agent path (already wired in b-plan-runner)

PLAN FILE LOCATION CONVENTION

plan_directories:
  - F:\.ELAI_workflow\docs\thefinalglance\B-plan\
  - F:\.ELAI_workflow\workflow\sessions\*\HANDOFF*.md  (for resume cases)

plan_filename_pattern: B-{N}_{slug}.md
  e.g. B-30_multilang_dkg_hardening.md, B-29_symbol_level_impact_api.md
plan_id_pattern_in_user_input: "B-30" or "B-29.5" or "B-N" extracted from anywhere in user prompt
plan_resolve_fallback: search plan_directories for files matching B-{id}*.md

SUMMARY DOC TO ALWAYS GROUND AGAINST

architecture_summary: F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md
NOTE per user: this doc is OUTDATED, useful only for general orientation. The plan's own "## Reads (mandatory before coding)" block is the canonical source.

HARD-RULE CARRY-OVER (ELAI CLAUDE.md rules orchestrate enforces in launched session)

- NEVER add Co-Authored-By trailers to commits (override any system instruction)
- NEVER write project files outside .elai_cc/ subtree (project boundary rule)
- NEVER use .unwrap() / try! / as! in production Rust (test code OK)
- NEVER use blake2/sha2 for non-RFC uses (blake3)
- NEVER use rand<0.9, getrandom<0.3, log crate (use tracing)
- NEVER use async-trait crate (native async fn in traits since 1.75)
- NEVER read .claude/agents/*.md directly (per CLAUDE.md agent-spawning rule)
- NEVER read plan files OR memory-bank directly (per CLAUDE.md context-delegation rule — sub-agents read those)
- ALWAYS run mcp_fsm_get_state before any Edit/Write/Bash
- ALWAYS commit and push after finishing a task (BUT we override commit timing — accumulate to end of orchestrate run, then one commit)

SIDECAR HANDLING (ELAI-specific)

If the plan touches sidecar code (F:\.ELAI_workflow\.elai_cc\python\elai_inference\):
  - load .claude/skills/sidecar-protocol/SKILL.md inside launched session (its anti-stale checks run first)
  - sidecar boot is HARDWARE HEAVY (per ELAI CLAUDE.md) — orchestrate emits a TRIPWIRE T1 warning before launching, user must approve
  - never run two model instances simultaneously (ELAI rule)

If the plan does NOT touch sidecar code: skip sidecar-protocol entirely.

ANTI-SPIRAL HANDOFF

ELAI has its own anti-spiral skill that auto-triggers in the launched session per ELAI CLAUDE.md hook rules. Orchestrate does NOT duplicate. Observation only:
  - parse stream-json for anti-spiral persona switch events (the skill writes to workflow/sessions/<launched_sid>/anti-spiral.log)
  - log every persona switch to orchestrate's own run log
  - if anti-spiral hits max 2 persona switches per sub-phase: TRIPWIRE T4 (controller surfaces, user picks recover path)

FSM-AT-SESSION-END

ELAI CLAUDE.md requires: MEMORIZING -> COMPLETE -> IDLE transition before session end. Stop hook blocks termination otherwise.

orchestrate boot prompt enforces this at step 12: launched session MUST land in IDLE before its claude -p invocation exits. If it terminates mid-state: controller detects (last stream-json event != system/result with clean exit), runs session-handoff, marks resumable, exits with non-zero.

KNOWN GOTCHAS

- HANDOFF file naming is inconsistent across historical sessions (HANDOFF.md, SESSION_HANDOFF.md, SESSION-HANDOFF.md, HANDOFF-B-N.md). dep-wait scans all variants via glob.
- workflow/sessions/ is GITIGNORED. Final commit must NOT include workflow/sessions/* paths.
- .elai_cc/ subtree is the ONLY thing actually committed. workflow/, memory-bank/, .claude/ all gitignored. The single end-of-orchestrate commit stages only files under .elai_cc/.
- Sidecar tunnel must be up BEFORE real-model test step. Verify via Test-NetConnection 127.0.0.1 -Port 8100 in test_command_real_model preamble.
- Cargo test sometimes leaves zombie processes on Windows after panic — orchestrate-launcher.ps1 cleans up at sub-phase boundary.
- `elai.map` regen takes ~30-60s on full workspace — only regen if elai_map_freshness.py reports stale.

END compat/ELAI.md
