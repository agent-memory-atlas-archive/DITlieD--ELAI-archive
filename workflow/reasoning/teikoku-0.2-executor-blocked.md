teikoku 0.2 executor — BLOCKED on FSM-state/gate mismatch (controller action needed)

session: s-de7e4c60 (executor)  date: 2026-06-19  task_id: teikoku-0.2-exec
This file is a controller-facing note written from EXECUTING (workflow/reasoning/ is the only doc path the EXECUTING gate permits). The canonical handoff append (workflow/handoff/teikoku-build.md) must be done by the controller, who owns the FSM and per handoff-protocol.md should not have the FSM reset out from under it by a peer/worker session.

SUMMARY
Zero 0.2 work was possible. No build files written, no deps installed, no bun build run, no credential copied, no Opus probe, no cc-auth-flow.md. The executor is structurally unable to do ANY of the 0.2 work — or even file its own R-4 handoff at the canonical path — from the current FSM state. This is NOT the T-build or T-auth tripwire; the leaked-source build was never attempted. It is a tooling/state-model mismatch the controller must resolve before re-dispatch.

THE OBSTACLE (verified from the gate source, not inferred)
The FSM is EXECUTING. The PreToolUse hook C:\Users\user\.claude\rag\hooks\pre_tool_check.py gates every Bash/Write/Edit:
- _check_executing (lines 616-621): hard-denies ALL Bash in EXECUTING, no exceptions ("the auditor runs tests"). So every command this task needs is blocked: npm view <pkg> version time (anti-slopsquat dep verification, mandatory R-6), bun install, bun build, cp /mnt/c/.../.credentials.json, chmod 600, and the Opus probe (the DONE gate). All are wsl bash. All denied.
- _check_executing (lines 624-648): denies Write/Edit to any path NOT under src/ , tests/ , memory-bank/*.md , workflow/reasoning/ , or memory-bank/plans/. The build files must be written into the WSL distro at ~/teikoku/base-cc (reachable as the UNC path \\wsl.localhost\teikoku\home\teikoku\teikoku\base-cc\...). That path matches none of the allowed prefixes, so writing package.json / tsconfig.json / build.sh into the distro is also denied.
- The same gate even blocks the canonical handoff: workflow/handoff/ is NOT in the EXECUTING allow-list (only workflow/reasoning/ is), so the executor cannot append to workflow/handoff/teikoku-build.md from EXECUTING — which is why this note lives in workflow/reasoning/ instead.

Net: there is NO FSM state in which both (a) Write into the WSL distro and (b) build/probe Bash are permitted. EXECUTING blocks all Bash. AUDITING (the only legal transition target from EXECUTING) blocks non-scratch Write and allows only pytest/ruff/mypy Bash. PLANNING blocks Bash. The gate was authored for ELAI's own Rust+Python tree where the auditor is the sole test-runner; it does not model "reconstruct a leaked foreign build and run a live probe," which interleaves authoring scaffolding files with iterative builds and a network call.

WHY THE EXECUTOR DID NOT SELF-RESOLVE
- dangerouslyDisableSandbox does NOT bypass the PreToolUse hook (the hook runs regardless of sandbox mode), so it cannot unblock Bash.
- The only always-allowed FSM levers the executor holds are mcp_fsm_transition (legal target from EXECUTING = AUDITING only, wrong) and mcp_fsm_reset (-> IDLE). The executor declined to call mcp_fsm_reset: handoff-protocol.md states a peer/controller session holding a shared FSM is not a blocker and "the FSM is never reset for it"; resetting the controller's FSM out from under it is a state mutation outside the executor's scope (R-3) and adjacent to gate-evasion (R-8 freeze spirit). The controller owns the FSM and is the right party to change it.

THE FORK (one concrete question for the controller)
Pick the state/gate resolution, then re-dispatch the 0.2 executor:
  Option A (executor's lean) — mcp_fsm_reset to IDLE, then re-dispatch. VERIFIED cheapest + sufficient: IDLE ungates Bash entirely (pre_tool_check.py:1135-1139) and gates Write only to project-source prefixes .elai_cc/ , .projects/ , src/ , tests/ (lines 185-205) — the WSL distro paths (~/teikoku/base-cc, \\wsl.localhost\teikoku\...) and workflow/handoff/ match NONE of those, so all 0.2 work (wsl bash + distro Writes + canonical handoff append) becomes possible. 0.2 mutates only WSL-distro files; it does not touch .elai_cc/ or any F: project source the IDLE gate protects. Downside: IDLE is "ad-hoc mode" — the controller loses the EXECUTING->AUDITING cycle bookkeeping for this sub-phase and must re-establish whatever state tracking it wants for the audit step.
  Option B — run the 0.2 executor in a session NOT under this FSM gate (e.g. a claude -p launched at a CWD outside the .ELAI_workflow root, or with the pre_tool_check hook disabled for that run). Keeps this controller session's FSM untouched. Downside: more orchestration plumbing; must still honor agent-dispatch-preamble + secret hygiene.
  Option C — add a sanctioned, narrow wsl-bash allow-prefix for ~/teikoku/base-cc (and a Write allow for the distro UNC path + workflow/handoff/) to the EXECUTING branch of pre_tool_check.py. This is the most "correct" long-term fix for TEIKOKU build sub-phases, but the gate is FROZEN and editing it is out-of-scope for the executor and should be a deliberate, human-reviewed change, not done mid-dispatch.
Executor lean: A for THIS sub-phase (unblocks immediately, zero source risk), with C as the proper durable fix the controller may schedule separately so future TEIKOKU build/probe sub-phases are not re-blocked.

HOW TO PICK UP (after the controller resolves the state)
Re-dispatch the 0.2 executor with the SAME prompt; the runbook is unchanged (workflow/teikoku-build/context/0.2-context.md). The first real steps the next executor runs, in order:
  1. ENV HYGIENE in the probe shell: ensure NO ANTHROPIC_API_KEY / ANTHROPIC_AUTH_TOKEN / apiKeyHelper / CLAUDE_CODE_USE_BEDROCK|_VERTEX|_FOUNDRY; never pass --bare (any silently suppresses plan-OAuth, context Part B).
  2. CRED: wsl mkdir -p ~/.claude; cp /mnt/c/Users/user/.claude/.credentials.json ~/.claude/.credentials.json (independent copy, NOT a symlink to /mnt/c); chmod 600. Field names only, never echo a value.
  3. BUILD: at ~/teikoku/base-cc author package.json + tsconfig.json (src/* path alias) + a build.sh; npm-verify EVERY external dep before install (anti-slopsquat); bun install; bun build ./src/main.tsx --target=bun --outfile=cli.js. Resolve bun:bundle (main.tsx:21) via bun build's bundler + the feature() flags; stub/define build-time macros USER_TYPE / MACRO.FEEDBACK_CHANNEL honestly (never a fake that breaks auth). If a clean OAuth-capable build is unreachable in budget -> T-build, STOP + handoff (Plan B1/B2 decision).
  4. AUTH SANITY: confirm getAuthTokenSource() resolves to claude.ai (plan), not a key. A 401/refresh-failure on the freshly-copied file -> T-auth, STOP + handoff (D-03 auth tripwire), do not thrash.
  5. PROBE (DONE gate): one real Opus turn via --print on a trivial prompt ("Reply with exactly one word: teikoku"); capture model id (must be Opus), OAuth-bearer evidence (source=claude.ai, not a key), response word. Save evidence under ~/teikoku/docs/.
  6. DOC: write ~/teikoku/docs/cc-auth-flow.md per the context-package spec (no secret values).
Use the GOTCHA from 0.1: PowerShell->wsl->bash mangles $ and lost --exclude once; write distro files via the Write tool to the \\wsl.localhost\teikoku\... UNC path (cat back to confirm) or a quoted heredoc / an in-distro .sh; avoid embedded $ that PS/wsl will mangle.

KNOWN LIMITS / NEXT
- The owner's 0.2 build + auth decisions are UNCHANGED and still in force (reconstruct from leaked src/ now; autonomous .credentials.json copy). Only the FSM state needs fixing.
- Residual 0.2 risks remain exactly as the context package framed them (T-build = bun:bundle/feature()/macro reconstruction; T-auth = copied refresh_token can't refresh -> owner interactive login).

GROUNDING / VERIFICATION
- Gate behavior above is read directly from C:\Users\user\.claude\rag\hooks\pre_tool_check.py (lines cited), not inferred. Verified by three live denials this session: a wsl bash probe, an `ls` via Bash, and the Edit to workflow/handoff/teikoku-build.md — all returned the EXECUTING-state deny.
- NO build, NO credential handling, NO probe was attempted; nothing to verify there. No F:/D: source was git-mutated (git read-only honored; only this reasoning note + the decision-ledger were written).
