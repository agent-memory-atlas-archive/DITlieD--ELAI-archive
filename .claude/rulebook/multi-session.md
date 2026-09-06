Multi-session protocol

Each Claude Code instance gets a session ID (s-{parent_pid}). Per-session FSM state in workflow/sessions/{id}/.fsm-state.json. Shared registry in workflow/sessions/state.json.

Entering TESTING_FIRST or EXECUTING:
1. Call mcp_session_claim(owns="file1.py,file2.py", locks="file3.py"). owns = files you edit. locks = blast radius (auto-computed if omitted).
2. If conflict returned: tell user, wait for other session to finish, or pick different work.
3. PreToolUse hook silently blocks Write/Edit to files owned/locked by other sessions. Zero token cost during coding.

Build/test errors with multiple sessions:
1. Auditor calls mcp_attribute_build_errors(error_output) after any test failure.
2. Fix only YOUR errors. Other session errors: report as INFO, not BLOCK.

Phase completion (after MEMORIZING):
1. Call mcp_session_release() to free owned/locked files.
2. Other sessions can now claim those files.

Session end:
1. Transition FSM to IDLE. Session entry stays in state.json until heartbeat expires (30 min).
2. Next session start auto-cleans stale entries.
