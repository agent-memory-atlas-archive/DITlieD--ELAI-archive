# Incident Postmortem Template
# E-30: harperfoley.com/blog/ai-agents-destroyed-production-zero-postmortems

Timestamp: {TIMESTAMP_MS}
Trigger event: {TRIGGER_ACTION_TYPE}
(one of: destructive_verb_denied / mcp_pin_violation / trust_dialog_required /
 auditor_lineage_collision / untrusted_memory_write_refused)

## Affected paths
- {PATH_1}
- {PATH_2}

## Agent and model lineage involved
- Agent: {AGENT_ID}
- Executor model: {EXECUTOR_MODEL}
- Auditor model: {AUDITOR_MODEL}
- FSM state at time of incident: {FSM_STATE}

## What happened
{DESCRIPTION}

## Mitigation taken
{MITIGATION}

## Follow-up required
- [ ] {FOLLOWUP_1}
- [ ] {FOLLOWUP_2}

## Root cause
{ROOT_CAUSE}

## Prevention
{PREVENTION}

## References
- workflow/reasoning/E-30-decisions.md
- https://harperfoley.com/blog/ai-agents-destroyed-production-zero-postmortems
- https://theregister.com/2025/07/21/replit_saastr_vibe_coding_incident (Replit DROP DATABASE)
- https://forum.cursor.com/t/154810 (Cursor git restore)
