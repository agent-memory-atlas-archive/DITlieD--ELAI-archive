Handoff: glm-orchestrator-phase-0 (written to workflow/reasoning/ because the EXECUTING path gate blocks workflow/glm-orchestrator/handoff/; controller should relocate to workflow/glm-orchestrator/handoff/PHASE-0.md)

problem: PHASE 0 of the GLM-5.2 orchestrator build (harden the devin-bridge GLM worker harness: new worker system prompt, CLAUDE.md filter fix, two-lane concurrency, rate-limit retry, capability matrix, emergency git guardrail).

CURRENT STATUS: BLOCKED at first Write. The shared FSM is in EXECUTING, whose PreToolUse path gate (C:/Users/user/.claude/rag/hooks/pre_tool_check.py::_check_executing, lines 681-713) denies Write/Edit outside src/ tests/ .elai_cc/ .claude/tools/ memory-bank/*.md plans/ workflow/reasoning/, and denies ALL Bash. This phase's scope is "docs/.source/claude-code-main/claude code build/devin-bridge/**" plus a worker PreToolUse guardrail hook — the devin-bridge tree is NOT on the EXECUTING allowlist, and no FSM state opens docs/.source/. Worker cannot mutate the hook/settings/FSM (preamble R-2/R-3). Needs a controller scope exemption before any of the 6 deliverables can land.

INDEX
2026-06-29 session GLM-ORCH-PHASE-0 - blocked at first write: EXECUTING path gate excludes the devin-bridge scope and blocks Bash needed for the matcher unit test + capability probes.

----------------------------------------------------------------
2026-06-29 session GLM-ORCH-PHASE-0

SUMMARY
Fully read the binding preamble, the build plan (F1/F2/F3, D1-D5, PHASE 0), and the entire devin-bridge harness (claudedevin.ps1, devin_proxy.py, devin_wire.py, claudedevin-system.md, devin-system.md, RUN-GLM-5.2.md) plus the guardrail spec (.claude/skills/glm-orchestrate/guardrail.md) and the harness PreToolUse hook I/O contract (pre_tool_check.py). All grounding for the 6 edits is in hand and the exact edit points are pinned. Hit a hard environment blocker on the FIRST Write: the shared FSM EXECUTING-state path gate denies every path in this phase's scope and also blocks Bash (needed for the 0.6 matcher unit test and 0.5 capability probes). Drafted the new worker system prompt content but could not commit it or any of the other 5 deliverables to disk under the current gate.

WORK DONE (all grounding verified against source; NO edits landed)
- Proxy system-prompt loader confirmed: devin_wire.py:53-70 _override_system() concatenates claudedevin-power.md (DEVIN_POWER_FILE, ABSENT on disk) then claudedevin-system.md (DEVIN_SYSTEM_FILE). So 0.1 = REPLACE claudedevin-system.md (the loaded file). devin-system.md is a LEGACY TEMPLATE the proxy NEVER loads; retire it by repointing its header at claudedevin-system.md. VERIFIED by reading both files + the loader.
- F1 (CLAUDE.md filter) confirmed: proxy swaps the SYSTEM prompt (devin_wire.py:172-179) but forwards message bodies verbatim (devin_wire.py:182-220); the user-text append is devin_wire.py:216-220. The knob to mirror is DEVIN_TOOL_DESC at devin_wire.py:34 + 246-249. 0.2b = a DEVIN_CLAUDEMD=pass|strip knob keyed on the "# claudeMd" marker, default pass (behaviour unchanged unless set). UNVERIFIED: that CLAUDE_CODE_DISABLE_CLAUDE_MDS is the exact env var this bundle's getUserContext reads — Glob/Grep over docs/.source/ TIME OUT (huge bundled cli.js) and Bash is gated, so I could not re-confirm the cli.js anchors (449506-449526 / 163422-163437 / 163425). Treated F1's env-var name as plan ground-truth, not re-derived.
- F2 (concurrency) confirmed: devin_proxy.py:39 _CONV_ID = uuid4().hex once per process, stamped via devin_wire.py:166-168 (build_request conv_id). claudedevin.ps1:19 already reads $env:CLAUDEDEVIN_PORT (default 8810); :36-39 Proxy-Healthy hits /health on $Port; :49-58 auto-starts the proxy parameterized by $Port (passing DEVIN_PROXY_PORT). So 0.3 = a NEW glm-worker.ps1 that sets $env:CLAUDEDEVIN_PORT per lane (8810/8811) then delegates to claudedevin.ps1 — no edit to the working launcher needed, cleanest per the plan's "glm-worker.ps1 if cleaner".
- F3 (retry) confirmed: proxy downgrades gateway errors to a 200 text_delta in _iter_devin (devin_proxy.py:93-95 HTTP non-200) and _translate (:146-166), so the CLI's SDK 429 retry never fires. 0.4 backoff goes in _gateway_stream/_iter_devin (:79-107): 429/503/throttle only, parse retry-after if present, NEVER retry content-policy (deterministic), log each retry to proxy.log.err under DEVIN_LOG.
- Guardrail (0.6) confirmed: 10 banned-op classes + allow-list + match-hardening in guardrail.md; harness PreToolUse contract = read JSON {tool_name,tool_input} on stdin, emit {"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny"|"allow","permissionDecisionReason":...}} (pre_tool_check.py:562-584). 0.6 = a stdlib-python hook with the patterns as named constants + a deterministic matcher unit test asserting DENY on `git reset --hard HEAD~5`/`git push --force` and ALLOW on `git commit`/`git push`/single-file checkout. NOTE: the guardrail hook can legitimately live under .claude/tools/ which IS writable in EXECUTING (in _SRC_PREFIXES) — only the devin-bridge writes + Bash are gate-blocked.

FILES TOUCHED
- workflow/reasoning/GLM-ORCH-PHASE-0-BLOCKED-handoff.md (this handoff; the canonical handoff path was itself gate-denied).
- NONE in scope landed (all denied by the EXECUTING path gate).

USER CHOICES / DECISIONS (from the plan, for continuity)
- D1 lean guardrail (NOT the full git ban). D2 two proxies, one per worker, ports 8810/8811. D3 CLAUDE_CODE_DISABLE_CLAUDE_MDS=1 primary + proxy sanitizer fallback. D4 orchestrate autonomy envelope. D5 max 2 GLM instances total (conductor shares the free-model rate limit). Live proofs are honesty-over-green: record BLOCKED-on-external with the exact error if the gateway is down, never fabricate a pass; the 429-retry proof may be a deterministic backoff unit test instead of a real 429.

QUESTION FOR ORCHESTRATOR (ONE concrete question)
The shared FSM is in EXECUTING and its PreToolUse path gate blocks this entire phase (devin-bridge writes + all Bash). How do you want PHASE 0 unblocked?
  Option A (LEAN, my lean): add the phase-0 scope to _check_executing's allow set in pre_tool_check.py — the prefix "docs/.source/claude-code-main/claude code build/devin-bridge/" for Write/Edit — AND ungate Bash for this dispatched session (the 0.6 matcher unit test + 0.5 capability probes must run). The guardrail hook can go under .claude/tools/ which is already allowed. WHY: keeps the plan's Files list intact, minimal change, fail-closed elsewhere.
  Option B: re-dispatch PHASE 0 under a state/flag whose gate permits docs/.source/ + Bash (e.g. a dedicated worker-session exemption, since this is a sub-agent doing harness tooling, not .elai_cc product code). WHY: cleaner separation, but needs a new gate branch.
  (Rejected: relocating the deliverables into .elai_cc/ or .claude/tools/ to dodge the gate — it fights the plan's explicit devin-bridge Files list and splits the harness across two trees. Flipping to AUDITING does NOT help: AUDITING only allows workflow/audit-scratch/ writes.)
I will not pick between A and B myself (material fork affecting the FSM/hook the controller owns). Append a DECISION block under this entry and re-dispatch.

HOW TO PICK UP
After the controller opens the scope (Option A or B) and ungates Bash, the executor proceeds in this order with everything already grounded above:
  0.1 Write devin-bridge/claudedevin-system.md (orchestration-tuned worker prompt: scoped-executor + obey-dispatch-preamble + filter-clean + the verbatim "EDIT existing files, NEVER recreate a module that already exists" line). Repoint devin-system.md's header to it. Confirm devin_wire.py:53-54 SYSTEM_FILE still resolves to claudedevin-system.md.
  0.2 Edit claudedevin.ps1 worker launch path: $env:CLAUDE_CODE_DISABLE_CLAUDE_MDS = "1" (in the non-longcat branch, near the other ANTHROPIC_* env sets ~:76-82). Add DEVIN_CLAUDEMD=pass|strip knob in devin_wire.py at the user-branch :216-220, keyed on the "# claudeMd" marker, default pass, mirroring DEVIN_TOOL_DESC.
  0.3 NEW devin-bridge/glm-worker.ps1: param -Lane (a|b) -> port 8810/8811, set $env:CLAUDEDEVIN_PORT, set CLAUDE_CODE_DISABLE_CLAUDE_MDS=1, delegate @args to claudedevin.ps1. Two lanes => two proxies => two _CONV_IDs (fixes F2).
  0.4 devin_proxy.py _gateway_stream/_iter_devin: bounded exp backoff on 429/503/throttle only, parse retry-after, cap retries (e.g. 5), never retry content-policy, log to proxy.log.err.
  0.5 Capability matrix: live GLM run exercising Task/TodoWrite/MCP(codegraph-elai+local-rag)/WebSearch/WebFetch + Read/Edit/Bash/Grep/Glob; record PASS/FAIL/BLOCKED-on-external to workflow/glm-orchestrator/evidence/PHASE-0-capability-matrix.json. Respect max-2-GLM (D5).
  0.6 Guardrail hook (stdlib python) under .claude/tools/ (writable now) implementing guardrail.md, named-constant patterns, + a deterministic matcher unit test. Register for worker sessions (NOT the main session) — the controller wires the registration in glm-worker.ps1 / a worker settings file.
  Evidence ledger: workflow/glm-orchestrator/evidence/PHASE-0.json (JSON array of {deliverable,status,files,verified_how,notes}).

KNOWN LIMITS / NEXT
- cli.js anchors for F1 are UNVERIFIED (bundle Glob/Grep timeout + Bash gated). Confirm CLAUDE_CODE_DISABLE_CLAUDE_MDS is read by this exact bundle before relying on 0.2a as the sole fix; 0.2b sanitizer is the in-repo fallback regardless.
- Live proofs need the gateway live + the max-2-GLM cap (conductor running). 0.4 backoff proof can be a deterministic unit test of the backoff path (no real 429 required).

GROUNDING / VERIFICATION
- Read + grounded this session: agent-dispatch-preamble.md, GLM-ORCHESTRATOR-BUILD-PLAN.md, RUN-GLM-5.2.md, claudedevin.ps1, devin_proxy.py, devin_wire.py, claudedevin-system.md, devin-system.md, guardrail.md, pre_tool_check.py (gate + hook contract), settings.local.json (hook registration shape).
- Blocker REPRODUCED (not transient): Write to devin-bridge/claudedevin-system.md and to workflow/glm-orchestrator/handoff/PHASE-0.md both DENIED by _check_executing; Bash DENIED by _check_executing. All three are the EXECUTING gate.
- Nothing implemented yet; 0 files in scope written; 0 tests run. All six deliverables UNVERIFIED pending the scope unblock.
