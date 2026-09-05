---
name: dashboard-handoff
description: Post-edit dashboard verification for B-N work touching F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\. Triggers after Write/Edit on dashboard files when transitioning to AUDITING. Spawns dashboard-smoker agent for svelte-check + cargo check + optional manual tauri:dev launch. Aggregates results into the audit context.
license: internal
compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"
metadata:
  version: "1.0.0"
  author: "elai"
---

Dashboard handoff

Trigger: in EXECUTING -> AUDITING transition, any of these were edited
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\*.rs
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\**\*.svelte
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\api\*.ts
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\*.svelte

Skip if no dashboard-v2 file was edited.

PROTOCOL

1. Aggregate the windows / components changed in this EXECUTING pass. Use git diff or scan the session's edit log:
   - windows_changed: from src/lib/windows/*.svelte and src/screens/*.svelte
   - components_changed: from src/lib/drawers/*.svelte, src/lib/orbit/*, src/lib/layout/*
   - tauri_commands_changed: from src-tauri/src/*.rs

2. Cross-language pair sanity check. For each src-tauri/src/*.rs change, verify the corresponding src/lib/api/stub.ts has been updated (or has cross-lang-skip annotation). Use cross_lang_diff.py:
     python F:\.ELAI_workflow\.Codex\tools\cross_lang_diff.py --git-staged
   If drift: surface to user before spawning the smoker.

3. Spawn dashboard-smoker agent (.Codex/agents/dashboard-smoker.md) with:
     session_id: <session_id>
     windows_changed: <list>
     components_changed: <list>
     trigger: "post-edit"

4. Read its return + the dashboard-smoke.md it writes. Possible verdicts:
   PASS: svelte-check + cargo check both clean. Proceed to AUDITING.
   FAIL: errors present. Treat as a BLOCK pre-audit. Trigger fixer pre-audit (skip the auditor; go straight to fixer).
   DEGRADED: tool missing (pnpm/svelte-check). Surface to user — they may need to install pnpm or skip dashboard checks.

5. If user requests visual verification (e.g., "check the orbit visualizer"):
   Re-spawn dashboard-smoker with trigger="manual". The smoker will start `pnpm tauri:dev` in background. The smoker returns the localhost URL.
   Tell user: "Tauri dev running at <URL>. Open in browser, verify the changes manually. Reply 'done' when finished."
   Cleanup: when user replies done, kill the background tauri process.

6. Append the smoke report path to the auditor's input. The auditor reads dashboard-smoke.md as evidence in REVIEW_CODE sub-state.

INTEGRATION WITH B-PLAN-RUNNER
b-plan-runner skill step 9 (AUDITING) checks if dashboard files were touched. If yes, runs dashboard-handoff before spawning auditor.

QUALITY RULES
- Never run pnpm install or npm install — Anti-Slopsquatting.
- Never modify dashboard source from this skill — read-only verification.
- Cleanup: always kill background tauri:dev when done. Use Bash run_in_background tracking.
