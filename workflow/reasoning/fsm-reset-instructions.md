FSM RESET INSTRUCTIONS
======================
The global FSM file is stuck. Copy the content below into the file at the path shown.

TARGET FILE: F:\.ELAI_workflow\workflow\.fsm-state.json

PASTE THIS EXACT CONTENT (replace everything in the file):
{"state":"IDLE","attempts":0,"audit_cycles":0,"phase":"","dead_end_reason":"","agent_substate":"","block_count":0,"last_transition":"2026-05-21T00:00:00+00:00"}

ALSO CREATE THIS FILE: F:\.ELAI_workflow\workflow\sessions\s-1233230\.fsm-state.json
WITH SAME CONTENT AS ABOVE.

If saves are not working, the file may be held open by a Python process.
Kill it: taskkill /F /IM python.exe /T (in an admin CMD)
Then retry the save.
