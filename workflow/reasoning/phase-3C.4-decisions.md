Phase 3C.4 Recovery Protocol - Decision Document

1. Death Detection Stub (heartbeat absent)
AgentDeathClassification enum with 4 variants: Crash, CleanExit, Timeout, BudgetExceeded.
Caller provides the classification directly (no heartbeat polling).
Timeout variant is accepted but treated identically to Crash for recovery purposes.
Comment: // STUB(3C.2): heartbeat-based death detection not yet implemented
No unimplemented!() or todo!() anywhere.

2. SSB Action Type Strategy
Option A chosen: schema migration to SCHEMA_VERSION=2.
Rationale: run_migration() already drops+recreates tables on version mismatch (ssb.rs L667-676).
SSB is session-scoped, data loss acceptable for old sessions.
Add "death_event" and "recovery_event" to CHECK constraint in CREATE_SSB_TABLE.
Extend SsbActionType enum with DeathEvent and RecoveryEvent.
Extend SsbPayload enum with DeathEvent(DeathEventPayload) and RecoveryEvent(RecoveryEventPayload).
Extend as_sql_str(), ssb_action_type_from_str(), and action_type() accordingly.

3. Module Boundary
recovery.rs as standalone pub module in orchestrator/src/.
RecoveryManager<C: ClockProvider> as plain struct (not trait).
Single public method: pub async fn recover_agent().
3C.5 SupervisionTree will call RecoveryManager directly, no trait indirection needed.

4. Expected Tests
Unit tests (7 minimum in recovery.rs):
  - recover_agent_crash_with_checkpoint: checkpoint present, Crash -> Ok(handle), same agent_id, SSB has both events
  - recover_agent_crash_no_checkpoint: Ok(None) checkpoint -> Ok(handle), fresh grc_history, degraded=true
  - recover_agent_budget_exceeded: BudgetExceeded -> Err(RecoveryError::BudgetExceeded), no SSB writes
  - recovery_preserves_agent_id: returned handle.agent_id == input handle.agent_id
  - ssb_step_index_monotonic: recovery_event superstep > prior_step_index
  - schema_migration_v2: fresh DB has death_event and recovery_event in CHECK constraint
  - schema_migration_idempotent: creating Ssb twice on same DB does not error
Contract tests (3 in contracts/tests/recovery_contract.rs):
  - contract_recovered_handle_preserves_agent_id
  - contract_corrupt_checkpoint_is_fresh_start_not_error
  - contract_budget_exceeded_returns_err_no_ssb_write

5. Unknowns
  - 3C.2 agent lifecycle not yet implemented. Recovery stubs Timeout classification.
  - Concurrent recovery for same agent_id not guarded (out of scope per spec; callers serialize).
  - first_grounding_source field on RecoveryContext: recovery.rs populates it but does not write ReasoningTrace entries (agent runtime responsibility).
