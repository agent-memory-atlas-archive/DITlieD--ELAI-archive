Session start

If user says workflow/config/agent or "off-topic" changes, transition to CONFIGURING. Otherwise:
1. Spawn context-compiler agent (.claude/agents/context-compiler.md) with task = "session start". Read the context package. Flag stale references or template files.
2. Ask: "Current task: [X]. Blockers: [Y]. What do you need ?"
3. After user gives task: call mcp_session_register(task) + mcp_session_list(). Report active sessions.
4. If task involves live model inference (/research, /task, B-VERIFY, or any call that hits ANTHROPIC_BASE_URL): run the 3-step health check from workflow/remote-setup/REMOTE-SIDECAR.md before other work. Flag as HARDWARE-HEAVY in chat.

Plan resume

Trigger: user says to read a plan file or continue a plan. Follow planning-workflow skill for full protocol.
Short form: mcp_fsm_get_state -> register session -> spawn context-compiler (with session_id + plan path + phase + user notes) -> WAIT for context-compiler to finish (prompt-engineer depends on its output) -> spawn prompt-engineer (with session_id + raw task + context package path) -> present transformed prompt -> wait for confirmation. These two agents are SEQUENTIAL, never parallel.
HARD RULE — context package handoff: when spawning prompt-engineer, you MUST pass the explicit path to the context package file: workflow/sessions/{session_id}/context-package.md. The prompt-engineer CANNOT locate this file from session_id alone. Omitting it = prompt-engineer works blind = garbage output. This is non-negotiable.
HARD RULE — zero main-context reads: NEVER Read/Glob/Grep the plan file, memory-bank files, or context-package.md yourself. Not "after" the sub-agent, not "to check" — NEVER. Pass the plan path and phase to context-compiler; it reads everything. Pass context package path to prompt-engineer; it reads the context. You only see the final transformed prompt. Violation = wasted 800-1500 tokens the sub-agents already summarize for you.
FSM abort: if the user asks to stop the current task or switch to something else, call mcp_fsm_reset to return to IDLE. This is the escape hatch from any state.

E-38 posture-doc-stale check: at session start, check `docs/strategy/ELAI-2026-POSTURE.md` last-modified date. If > 90 days old, surface a `posture_doc_stale` warning to the user and recommend a refresh pass. Wire-dark by default; live emission gated by `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1` on the orchestrator side (drives the `RoadmapPanel` dashboard tile). Source: `crates/orchestrator/src/roadmap_signals.rs::check_posture_doc_age` + `workflow/reasoning/E-38-decisions.md`.
