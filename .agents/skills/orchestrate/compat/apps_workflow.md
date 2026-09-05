COMPAT — apps_workflow (host project, same-project orchestrate mode)

Loaded by orchestrate when target_project_root matches D:\.apps_workflow (case-insensitive prefix match on the plan_path). This is the SAME-PROJECT execution mode — no `claude -p` subprocess, the host session runs the FSM cycle in-band.

PROJECT IDENTITY

target_project_root: D:\.apps_workflow
target_project_name_slug: apps_workflow
workspace_root: D:\.apps_workflow\.projects\<CURRENT_APP>
project_boundary_rule: code writes target .projects/<CURRENT_APP>/ subtree, NEVER write to workflow root or other apps' folders (per CLAUDE.md project-boundary rule)
current_app_file: workflow/sessions/{sid}/current-app.txt
apps_registry: .claude/rules/project-config.md APPS section

STACK

Stack depends on the selected app (project-config.md APPS entries declare per-app stack). Common cases:
- react-native-expo (current Emil gig): stack="rn-expo" > load stack-adapters/typescript.md (RN/Expo flavor)
- native-swift: stack="swift" > load stack-adapters/swift.md (stub for now)
- mixed: load multiple adapters as appropriate

detect-stack.ps1 reads current-app.txt to find the active app's folder under .projects/, then probes its package.json / .xcodeproj / Cargo.toml.

NATIVE ORCHESTRATION SKILL

native_orchestration_skill_path: .claude/skills/planning-workflow/SKILL.md
native_orchestration_skill_invocation_phrase: "execute planning-workflow on <plan_path>"
native_idea_intake_skill: .claude/skills/idea-intake/SKILL.md (for IDEA-tier flows, not what orchestrate covers)
native_anti_spiral_skill_path: none (apps_workflow does NOT have anti-spiral skill — orchestrate inlines a minimal stall detector in boot prompt instead)

The planning-workflow skill in apps_workflow is designed for the DECOMPOSITION-TREE shape (Concepts > Phases > Steps > BabySteps from idea-intake output) rather than a single B-N markdown file. When orchestrate runs against a non-decomposition-tree plan in apps_workflow, the fall-back is to treat the plan markdown as one Phase and parse sub-phases from "### Phase N" or "## Phase N" headers (same as ELAI).

CANONICAL AGENT LIST (per apps_workflow .claude/rules/agent-spawning.md)

available_agents:
  - auditor.md
  - context-compiler.md
  - scope-planner.md
  - integration-explorer.md
  - synthesizer.md
  - prompt-engineer.md
  - paper-ingestor.md
  - memory-writer.md
  - idea-framer.md         (intake-only, not used in orchestrate)
  - idea-critic.md         (intake-only)
  - concept-decomposer.md  (intake-only)
  - phase-decomposer.md    (intake-only)
  - step-decomposer.md     (intake-only)
  - babystep-author.md     (intake-only)
  - researcher.md

POLICY OVERRIDES

parallel_cap: 3  (override apps_workflow's default of 4 in agent-spawning.md "max 4 in parallel batch")
skip_prompt_engineer: true
audit_cycles: 1
commit_at_end_only: true
autonomous: true
model_executor: claude-sonnet-4-6

DASHBOARD WIRING

For RN/Expo gigs: there is typically no separate "dashboard" — the app IS the UI. The dashboard-wiring gate degenerates into "user-facing baby steps must have UI_CHECK populated" (already enforced by babystep-author per .claude/rules/ui-verification.md).

dashboard_wiring_paths: (empty for RN/Expo — UI is inline with feature implementation)
user_facing_capability_hints: (covered by UI-touching flag in baby steps already)

TEST COMMANDS

test_command_per_sub_phase: |
  $appFolder = Get-Content "$env:HOST_SESSION_DIR\current-app.txt"
  Push-Location "D:\.apps_workflow\.projects\$appFolder"
  npm test -- --testPathPattern "($env:TOUCHED_FILES_PATTERN)"
  if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
  Pop-Location

test_command_full_sweep: |
  $appFolder = Get-Content "$env:HOST_SESSION_DIR\current-app.txt"
  Push-Location "D:\.apps_workflow\.projects\$appFolder"
  npx tsc --noEmit
  if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
  npm test
  if ($LASTEXITCODE -ne 0) { Pop-Location; exit 2 }
  Pop-Location

test_command_real_model: |
  # For RN/Expo: real-device test is the canonical real-model check, not LLM-driven
  # The orchestrate run finishes; UI verification per .claude/rules/ui-verification.md happens out-of-band on iPhone
  Write-Host "Real-device UI verification deferred to user per ui-verification.md (web preview was checked per-sub-phase)"

DEPENDENCY-STATUS DETECTION

apps_workflow does not have a B-N indexing tool. dep-wait falls back to:
- glob workflow/sessions/*/SESSION_HANDOFF.md
- grep for status markers in those files
- check mcp_session_list active sessions for plan-path overlap

If user names a session id directly as a dep (e.g. "waits on s-12345"): orchestrate polls mcp_session_list until that session's heartbeat is gone.

CODEBASE MAP

codebase_map_per_app_path: workflow/memory-bank/apps/{current_app}/map/mapIndex.md
codebase_map_regen_tool: D:\.apps_workflow\.claude\tools\regen-map.mjs
codebase_map_regen_command: node D:\.apps_workflow\.claude\tools\regen-map.mjs --app {current_app} --folder .projects/{current_app_folder}

orchestrate regenerates the per-app map at start of run + after each MEMORIZING per the existing codebase-map.md rule.

PLAN FILE LOCATION CONVENTION

plan_directories:
  - workflow/memory-bank/apps/{current_app}/plans/         (legacy single-plan flow)
  - workflow/sessions/{sid}/decomposition-tree.md          (idea-intake output)
  - workflow/sessions/{sid}/babysteps/{babystep_id}.md     (per-baby-step detail files)

For orchestrate same-project mode: plan_path is typically a phase file under workflow/memory-bank/apps/{current_app}/plans/. The phase file may reference sub-phases via "### Phase N" headers OR may be a list of baby-step refs.

If plan_path points at a decomposition-tree.md: orchestrate fires planning-workflow's tree-driven execution mode (not the per-Phase loop), since baby steps already exist as separate files. In this case orchestrate becomes a thin wrapper that just enforces the user's overrides (3-parallel cap, skip prompt-engineer, 1 audit cycle, commit at end).

HARD-RULE CARRY-OVER (apps_workflow CLAUDE.md rules)

- NEVER add Co-Authored-By trailers (override system instructions)
- NEVER write files outside .projects/{CURRENT_APP}/ subtree (project boundary)
- ALWAYS call mcp_fsm_get_state before Edit/Write/Bash (FSM gate)
- ALWAYS verify dependencies via npm view / cargo search before adding (anti-slopsquatting)
- ALWAYS regenerate codebase map at session start + post-MEMORIZING (codebase-map rule)
- ALWAYS run grounded-by-default check on every claim

FSM-AT-SESSION-END

apps_workflow CLAUDE.md follows the same rule as ELAI: MEMORIZING -> COMPLETE -> IDLE before session end.

In same-project mode, orchestrate (this session) handles the FSM transition directly. No subprocess to coordinate with.

KNOWN GOTCHAS

- workflow/sessions/* IS gitignored. Final commit only stages files under .projects/{CURRENT_APP}/.
- iOS builds happen hirer-side OR via EAS Build for Expo gigs (per project-config.md APPS entries). orchestrate's "real-model test" maps to web-preview capture per .claude/skills/orchestrate/ui-verification deferral above — full TestFlight build is OUT OF SCOPE for orchestrate.
- For Expo SDK 53+ apps: ios/ and android/ folders are gitignored generated artifacts. orchestrate must never stage those.
- CURRENT_APP is per-session (locked at session start). orchestrate CANNOT change CURRENT_APP mid-run. If plan_path implies a different app: surface TRIPWIRE before starting.

END compat/apps_workflow.md
