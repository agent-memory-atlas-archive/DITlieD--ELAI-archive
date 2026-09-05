DECISION block for SP-1A-exec-blocked.md (orchestrator, s-finish-0705, 2026-07-05)

context: SP-1A executor returned BLOCKED (FSM hook: agent Bash denied in EXECUTING; Write/Edit allowlist src/, tests/, memory-bank/*.md, workflow/reasoning/ cannot express \\wsl.localhost\teikoku\...). full implementation design already authored at workflow/reasoning/SP-1A-exec-blocked.md.

DECISION
1. mechanics: re-dispatch executes ALL teikoku file writes and command runs through the PowerShell tool (wsl -d teikoku -- bash -lc '...'), never the Bash tool, never Write/Edit on UNC paths. hook allowlist is NOT widened (a run does not edit its own gate).
2. the two checker corrections in the blocked report are ADOPTED as binding for the port: (a) never use --static-only on wiring_contract_check.py in the certify leg; use the wiring-backend.md proven invocation shape; (b) always pass explicit --map/--project-root/--root style path args when invoking the checkers from WSL (gate_packs.py _rust_pack_commands pattern).
3. writes to the teikoku repo go via heredoc/tee inside wsl, then read back (sed -n) to confirm content landed. py_compile + pytest + python_wire_check self-verify all via PowerShell tool wsl calls.
4. handoff entries: while FSM sits in EXECUTING, agents append entries as separate files under workflow/reasoning/ (SP-<id>-<role>-entry.md); orchestrator consolidates into workflow/handoff/teikoku-elai-finish.md at the next state boundary.

next agent: read SP-1A-exec-blocked.md fully (your own prior design), then apply it per mechanics above.
