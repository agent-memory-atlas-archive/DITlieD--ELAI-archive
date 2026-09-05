TEIKOKU P4-D CONTEXT SNAPSHOT (context-compiler, task P4-D-CC2)

Scope: the SNAPSHOT + RAG FALLBACK + AUTO-CARRY + MIGRATING-INDICATOR + zero-loss proof half of
P4-D auto-migration (D-09). Peer CC-1 owns the token counter + threshold trigger; NOT duplicated here.
Done-criterion (master plan :200): ONE interactive chat survives an auto-migration with ZERO info loss
(snapshot + RAG verified), continuing in the SAME conversation on a fresh model context, carrying
skills/rules/toggles, with a "migrating" indicator.
All file:line refs are in the WSL distro `teikoku` at /home/user/teikoku/ (~ = /home/user).
Grounding: every claim below was read directly from the distro source this session (no training memory).

================================================================================
1. EXISTING SNAPSHOT / CHECKPOINT / MEMORY MACHINERY (what exists vs what the executor adds)
================================================================================

EVENT LOG (the spine, P2). conductor/conductor_db.py:
 - events table is APPEND-ONLY (triggers events_no_update + events_no_delete enforced; check() at
   conductor_db.py asserts both present). Columns: event_seq (monotonic AUTOINCREMENT), event_id,
   task_id, event_type (OPEN TEXT, no CHECK enum / no SCHEMA_VERSION treadmill), payload_json,
   created_at, project_id, session_id (migrations/005_events_project_session.sql).
 - append_event(event_type, task_id, payload, event_id, created_at, project_id, session_id) is the
   write primitive (conductor_db.py ~ append_event). project_id defaults DEFAULT_PROJECT_ID="conductor".
 - query_events(project_id, session_id, event_type, task_id, limit) + recent_events() are the typed
   readers; events.py iter_events(db_path, after_seq) is the full ordered replay.

RAG MEMORY (P2, the index over the log). conductor/memory_store.py + memory_semantic.py + memory_server.py:
 - memory_store.append_event_memory(project_id, event_type, payload, session_id, task_id) is the
   PRODUCTION write seam: it appends the event AND best-effort semantic-indexes it (memory_semantic.index_event
   on the just-written event_seq). This is the single call that puts a payload into BOTH the grep floor
   and the vector store. The executor's snapshot SHOULD emit through THIS, not raw append_event, so the
   snapshot is RAG-recallable for free.
 - memory_store.memory_search(needle, project_id, limit, semantic=True) -> (hits, mode): grep-primary
   (SQL LIKE over event payload prose + ripgrep over workflow/handoff/*.md) with an OPTIONAL semantic leg
   (nomic-embed-text-v1.5, sqlite-vec vec0 in a SEPARATE state/memory_vec.db). Semantic degrades to grep
   on any failure; NEVER hard-fails. This is the RAG RETRIEVAL seam (section 3).
 - memory_server.py is the MCP tool surface; slice_api.py memory_search_http is the HTTP surface
   (GET /memory/search?q=). handoff_indexer.index_handoffs ingests handoff markdown as memory events
   (the pattern the snapshot's "index a structured doc into RAG" mirrors).

EPOCH COMPACTION (the closest existing "snapshot a run" precedent, NOT a chat snapshot).
conductor/conductor_loop.py:795 _compact_epoch(db, artifact_root, run_id, label, payload):
 - writes a JSON summary to artifact_root/epochs/<run_id>/<label>.json AND emits an EPOCH_COMPACTED
   event. THIS IS THE SHAPE TO COPY for the structured snapshot: persist a JSON file on disk + emit an
   event. It is run/wave-scoped (task_counts + results), NOT session/conversation-scoped, so it is a
   PRECEDENT not a reuse.

WAKE HUB (the indicator transport). conductor/wake.py + slice_api.py:
 - wake.publish(event_seq, event_type, task_id, project_id, recipient=) broadcasts to the in-process hub
   AND POSTs /events/notify cross-process; best-effort, never raises. slice_api GET /events/wait long-poll
   (events_wait, 30s server hold) is how the dashboard/another process BLOCKS for new events. A migrating
   indicator is just an event + a publish (section 5).

TRANSCRIPT/JOURNAL OF THE CHAT TO SNAPSHOT? Partial, reconstructable, no dedicated table.
 - There is NO edit_checkpoints / session_dag / transcript / snapshot table in this distro (migrations
   001-006 are: initial, verdicts, control_state, active_worker_pid, events_project_session,
   worker_pool_and_harness). Those ELAI-runtime concepts (the ones the agent brief asked about) DO NOT
   EXIST here. The executor MUST NOT assume them.
 - The chat "transcript" is the set of session-scoped events: mailbox session_message rows (addressed,
   wake.unread_for watermark model), SESSION_SPAWNED rows (spawn.children_of), and whatever the
   chat-turn path records. slice_driver.py records SLICE_TURN_DONE with cc_session_id + result + token
   numbers; the interactive chat-turn (slice_api /chat/turn -> chat_orchestrator) currently records the
   spawn/dropin RESULT but does NOT yet persist the user/assistant message text as events.
 - GAP THE EXECUTOR MUST CLOSE: to snapshot a chat with zero loss, the message history must be CAPTURED.
   base-cc runs as the full CC TUI/`--print` subprocess (slice_driver.run_cc_turn: bun dist/cli.js
   --print --output-format json returns {result, usage, modelUsage, session_id}); base-cc's own transcript
   lives in its history store (base-cc/src/history.ts), keyed by its cc_session_id. So the snapshot's
   message-history field is sourced EITHER from (a) appending each chat turn's user text + assistant
   result as session_message-class events going forward (cheap, conductor-side, the recommended path), OR
   (b) reading base-cc's --output-format json transcript per turn. The structured snapshot should record
   turns as events from the FIRST turn so the history exists to snapshot; this is a small wiring add to
   the /chat/turn path, not a new store.

NET: REUSE append_event_memory (write+index), memory_search (RAG recall), wake.publish/events.wait
(indicator), control_state via session_toggles (carry), spawn.children_of (children), model_pool.migrate_at
(threshold data, from CC-1). ADD: the snapshot builder, the migrate orchestration (snapshot->respawn->carry),
the migrating event, per-turn message capture, and the zero-loss round-trip test.

================================================================================
2. STRUCTURED SNAPSHOT DESIGN (what the executor builds)
================================================================================

A new module conductor/session_snapshot.py (name follows the P4-x one-concept-per-module convention:
session_toggles.py / model_pool.py / chat_orchestrator.py). It captures, for a chat session_id, a single
structured dict with these fields (every field is what a migrated chat must NOT lose):

 SNAPSHOT FIELDS (the zero-loss contract):
  - schema_version + snapshot_id (uuid) + session_id + project_id + created_at
  - message_history: the ordered user/assistant turns for this session_id (from the per-turn events,
    section 1 gap). The load-bearing field; this is the conversation.
  - objective: the active run goal (session_toggles.objective).
  - toggles: the full SessionToggles.to_dict() — orchestration, model_pool, model_choice, skills,
    objective, always_grill (session_toggles.get_toggles(session_id)). This single read carries the
    P4-C control_state KV.
  - children: spawn.children_of(db, session_id) — the live + terminal spawned sessions (one chat = N
    sessions lineage) so the migrated chat still owns its sub-sessions.
  - pool_model: the active (pool, model) + its window/migrate_at (model_pool.get_pool_model(pool,model)
    .to_dict()) so the snapshot is MODEL-POOL-AWARE (revision :142-143 requirement) and the post-migration
    session knows its own threshold.
  - cc_session_id: base-cc's own session id for the pre-migration turns (continuity reference).
  - token_state: the last recorded ctx_used/ctx_cap/migrate_at that TRIGGERED the migration (handed in by
    CC-1's trigger; the snapshot records WHY it migrated; do not recompute — consume CC-1's number).

 PERSISTENCE (two writes, mirroring _compact_epoch):
  (a) a JSON file at conductor/state/snapshots/<session_id>/<snapshot_id>.json (durable, full-fidelity,
      the round-trip source of truth). Use artifact_root from ConductorConfig, NOT a hardcoded path.
  (b) an event via memory_store.append_event_memory(project_id, event_type="session_snapshot",
      payload={snapshot_id, session_id, path, summary, fields...}, session_id=session_id). This is BOTH
      the durable record AND the automatic RAG index (it flows through the semantic+grep index). The
      payload prose is what RAG recalls; include a compact text rendering of the message_history so a
      pre-migration FACT is grep/semantic-findable post-migration (section 6).

 RESTORE: session_snapshot.restore(snapshot_id|session_id) reads the JSON (authoritative) and returns the
 typed snapshot; the migrate orchestration (section 4/5) re-applies toggles + re-seeds the message history
 into the new (same) session. GENERALITY: every field is pure data (no ELAI literal); a non-ELAI chat
 snapshots identically (objective may be empty, project_id is the chat's own).

================================================================================
3. RAG FALLBACK (how the snapshot is recallable if the structured restore is incomplete)
================================================================================

The fallback is ALREADY the seam — no new store. Because the snapshot is emitted through
append_event_memory (section 2b), its payload prose is in BOTH the grep floor and (when the embed stack is
up) the vector store. The post-migration session recalls pre-migration content via:
 - memory_store.memory_search(needle, project_id=<chat project>, limit) -> (hits, mode): semantic+grep.
   A pre-migration fact (a decision, a file path, a user constraint) embedded in the snapshot payload prose
   (and in any per-turn session_message events) is retrievable by content even if a specific structured
   field was dropped from the JSON.
 - The HTTP/MCP surfaces already exist: slice_api GET /memory/search?q=&project=, memory_server MCP
   memory_search tool. The migrated session (or the dashboard) queries these.
 RECOMMENDED EXECUTOR WIRING: at migration time, also drive memory_store.reindex(project_id) (or rely on
 the per-event index in append_event_memory) so the snapshot is queryable immediately. The grep floor is
 unconditional, so RAG fallback works even with the embedding model down (the done-criterion's "RAG
 verified" must assert on memory_search returning the seeded fact via the GREP mode at minimum, so the
 proof does not depend on the optional semantic stack).

================================================================================
4. AUTO-CARRY skills/rules/toggles (how the new session inherits everything)
================================================================================

The carry is over session_toggles (control_state KV) — already the canonical session-settings store.
 - The snapshot captures toggles (section 2). On migration, the orchestration writes them onto the
   post-migration session via session_toggles.set_toggles(new_session_id, snapshot.toggles_dict). Because
   the migration keeps the SAME chat/session identity (section 5), in practice new_session_id == session_id
   and the toggles are already present; the carry is a RE-ASSERT (idempotent upsert) that also covers the
   case where a fresh session row is minted.
 - SKILLS/RULES: the skills toggle holds the pack-general skills subset (a list[str], session_toggles
   TOGGLE_SKILLS). base-cc loads skills from user (~/.claude/skills), managed (.claude/skills), and project
   skills dirs up to home (base-cc/src/skills/loadSkillsDir.ts:640-645 loadSkillsFromSkillsDir). So "carry
   skills/rules" = (a) preserve the skills toggle list across the migration (carried in the snapshot), and
   (b) keep the post-migration base-cc subprocess pointed at the SAME project/skill dirs (cwd + config
   home unchanged). RULES ride the same dirs (.claude/rules loaded with the project). There is NO ELAI
   hardcode: the skills list is data, the dirs are the chat's own project — a non-ELAI chat carries its own
   skills set. PACK-GENERAL by construction (revision generality guardrail :72-93).
 - Net: the carry mechanism is session_toggles.set_toggles(restored toggles) + an unchanged
   cwd/config-home for the re-spawned base-cc turn. No new "skill loader" is built; the toggle + the
   unchanged dirs ARE the carry.

================================================================================
5. MIGRATING-INDICATOR IN THE SAME CHAT (one chat survives, same identity)
================================================================================

SAME IDENTITY: the migration must NOT spawn a visibly-new chat. The chat is keyed by its conductor
session_id (the toggles namespace session:<sid>:*, the mailbox recipient, the spawn parent). Migration
keeps that session_id CONSTANT; only base-cc's internal cc_session_id rolls to a fresh model context (a new
--print subprocess / a /clear-equivalent), with the snapshot's message_history re-seeded as the new
context's preamble. So from the user's seat it is one continuous conversation; under the hood the model
context was swapped. The snapshot.cc_session_id records the old one for the audit trail.

THE INDICATOR (rides the existing wake/event spine, P4-A/P4-E render it):
 - emit a session_migrating event (event_type="session_migrating", payload={session_id, from_model,
   to_model, reason, snapshot_id, phase:"start"}) via append_event + wake.publish(..., project_id,
   recipient=session_id). A matching session_migrated (phase:"done") event is emitted after the snapshot
   round-trips and the new context is seeded.
 - The chat loop / the P4-E dashboard chat pane is ALREADY subscribed to the event stream via slice_api
   GET /events/wait (or /mailbox/wait with recipient=session_id). It receives the session_migrating event
   and renders "migrating..." inline WITHOUT closing the conversation. wake.publish carries recipient so
   only this chat's pane lights up (P4-A recipient filter, wake.wait_for_new). P4-E does the actual render;
   THIS half ships the EVENT + the publish so the signal exists for P4-E to consume (parallel to how P4-C
   shipped the grill GATE for P4-E to render).
 - The conversation does not break because nothing tears down the session_id: the indicator is an
   out-of-band event on the same session, and the next user turn resumes on the re-seeded context.

================================================================================
6. ZERO-INFO-LOSS VERIFICATION (the audit done-criterion proof)
================================================================================

TWO assertions, both BLOCKING (this is the phase done-gate, master plan :200):

 A. STRUCTURED ROUND-TRIP (every captured field restored):
    capture = session_snapshot.capture(session_id) ; restored = session_snapshot.restore(snapshot_id).
    assert restored == capture across EVERY field: message_history (exact ordered equality — the
    load-bearing one), objective, toggles (all six), children (spawn.children_of set), pool_model,
    cc_session_id, token_state. A missing/changed field is a FAIL. This proves the structured path loses
    nothing.

 B. RAG RECALL (the fallback path independently recovers a pre-migration fact):
    seed a distinctive pre-migration fact into the session (a turn / a decision payload), run the
    migration, then post-migration call memory_store.memory_search("<distinctive fact>", project_id=<chat>)
    and assert the fact is in the returned hits via the GREP mode (so the proof holds even with the
    semantic model down). This proves recall survives even if the structured snapshot were incomplete.

 The done-criterion is the CONJUNCTION: one chat (same session_id) migrates, A passes (structured
 zero-loss) AND B passes (RAG fallback recalls), the migrating + migrated events fired on the same
 session, and the post-migration session has the carried toggles/skills. The R10 integration smoke (section
 7) drives this through the REAL slice_api entrypoint end-to-end on a chat that has actual history +
 children + non-default toggles (the HARD input, not an empty session).

================================================================================
7. WIRING (PRODUCTION_HOOK) + TESTS
================================================================================

PRODUCTION_HOOK (the seam the new pub fns must be called from — D-030 / preamble R-8):
 The migration is INVOKED BY CC-1's threshold trigger. CC-1 owns the token counter + the
 ctx_used >= model_pool.get_pool_model(pool,model).migrate_at decision. When that fires it must call THIS
 half's migrate entrypoint. The concrete seam:
  - conductor/session_migrate.py::migrate_session(session_id, db, config, reason, token_state) — the
    orchestration: capture snapshot -> emit session_migrating + wake.publish -> restore/seed new context ->
    carry toggles -> emit session_migrated. This is the fn CC-1's trigger calls.
  - PRODUCTION ENTRYPOINTS that call migrate_session in the SAME changeset (so nothing ships wire-dark):
      (a) a NEW slice_api route POST /chat/migrate (slice_api.do_POST, beside /chat/turn) — the
          cross-process entrypoint CC-1's trigger (or the chat loop) POSTs to. This is the primary hook.
      (b) a NEW conductor.py CLI verb `migrate` (run_migrate, registered in build_parser beside run_spawn
          at conductor.py:586 / the add_parser block) — the same-process production entrypoint, mirrors
          run_spawn exactly (argparse subcommand -> ControlDB -> call -> print result).
   session_snapshot.capture/restore + session_migrate.migrate_session are the new pub fns; the /chat/migrate
   route + the `migrate` CLI verb + (for the indicator) wake.publish are their production callers, wired
   the same changeset. Self-verify before DONE: python .claude/tools/python_wire_check.py --files <changed>
   (NOTE the tracked distro bug, handoff TRACKED ITEM 1: python_wire_check.py RecursionError in-distro;
   prior P2/P3 audits used cross-file-grep + live-execution fallback — the auditor will do likewise).

 NEW TESTS (convention tests/test_p4d_*.py, mirroring test_p4c_interactive.py; pytest, in-distro
 .memory-venv; use tmp db_path like the P4-C tests, never the live control.db):
  - test_p4d_snapshot_roundtrip: capture then restore equals across all fields (assertion A); message_history
    ordered-equality; toggles all six; children set; pool_model window/migrate_at present.
  - test_p4d_rag_fallback: seed fact -> migrate -> memory_search recalls it in GREP mode (assertion B);
    assert it works with the semantic leg forced unavailable (degraded mode) so the proof is grep-floored.
  - test_p4d_carry_toggles_skills: post-migration get_toggles(session_id) equals the pre-migration toggles
    incl the skills list; same session_id (identity preserved, not a new chat).
  - test_p4d_migrating_indicator: migrate emits session_migrating then session_migrated on the SAME
    session_id; a wait_for_new(recipient=session_id) observes the migrating event (the indicator rides the
    hub); the conversation/session_id is unchanged across the pair.
  - test_p4d_zero_loss_blocking: the conjunction gate (A AND B AND indicator AND carry) — the single
    BLOCKING done-criterion assertion the auditor re-runs.
  - test_p4d_generality: a NON-ELAI chat (the demo pack project_id, zero ELAI paths) migrates losslessly —
    snapshot fields are pure data, no ELAI literal reachable; the generality guardrail (revision :90-93)
    applied to the migration path.
  - R10 INTEGRATION SMOKE (auditor-owned, not a unit test): drive POST /chat/migrate over the REAL
    slice_api :8787 on a chat with real message history + >=1 spawned child + non-default toggles +
    always_grill ON, then assert the structured round-trip + a /memory/search recall of a seeded fact +
    the migrating event landed + the carried toggles — the HARD-input end-to-end proof of :200.

VALIDATION WARNINGS:
 - HIGH: no edit_checkpoints/session_dag/transcript/snapshot table exists in this distro (migrations 001-006
   only). The agent brief's assumption of those ELAI-runtime tables does NOT hold; the snapshot is built on
   the event log + a JSON artifact, NOT a reused checkpoint table. Confirmed by reading conductor_db.py +
   migrations/ directly.
 - HIGH: the chat message history is NOT currently persisted as events (the /chat/turn path records the
   spawn/dropin RESULT, not the user/assistant text). The executor MUST add per-turn message capture (a
   small /chat/turn wiring add) or there is nothing to snapshot — this is a prerequisite for zero-loss, not
   an optional extra.
 - python_wire_check.py RecursionError in-distro (handoff TRACKED ITEM 1): the executor's self-verify +
   the auditor's wire check fall back to cross-file-grep + live-execution, per prior P2/P3 practice.
 - The semantic RAG leg depends on nomic-embed + sqlite-vec; it degrades to grep. The zero-loss RAG proof
   (assertion B) MUST assert via the unconditional grep mode so it does not depend on the optional model.
 - CC-1 boundary: do NOT build the token counter or the threshold compare here. Consume CC-1's
   migrate_at decision + token_state via the migrate_session(token_state=...) parameter. model_pool already
   ships context_window + migrate_at per model (model_pool.py _CONTROLLER_MODELS/_FREE_API_MODELS); CC-1
   reads those, this half receives the result.

SOURCES CONSULTED (all in distro /home/user/teikoku unless noted):
 conductor/memory_store.py ; conductor/memory_semantic.py ; conductor/memory_server.py (read head/usage)
 conductor/conductor_db.py (events table, append_event, query_events, triggers, migrations list)
 conductor/conductor_loop.py:780-825 (_compact_epoch precedent) ; conductor/events.py (iter_events)
 conductor/wake.py (publish, wait_for_new, recipient filter, /events/wait) ; conductor/slice_api.py (all
   routes: /chat/turn,/chat/toggles,/chat/sessions,/chat/pools,/events/wait,/memory/search,/spawn,/mailbox)
 conductor/chat_orchestrator.py ; conductor/session_toggles.py ; conductor/model_pool.py (migrate_at table)
 conductor/spawn.py (SpawnDescriptor, children_of) ; conductor/slice_driver.py (base-cc --print json turn,
   ctx_used/ctx_cap recording) ; conductor.py:586-660 (run_spawn/run_headless CLI verb pattern, build_parser)
 conductor/migrations/ (001-006 list — no checkpoint/snapshot/transcript table) ; tests/ listing
 base-cc/src/skills/loadSkillsDir.ts:640-688 (skill dir loading) ; base-cc/src/history.ts (transcript store)
 F: workflow/reasoning/TEIKOKU-P4-REVISION-2026-06-20.md:126-167 (AUTO-MIGRATION + decisions)
 F: workflow/handoff/teikoku-build.md (head, P4 status) ; TEIKOKU-MASTER-PLAN.md:190-205 (4.3 + done :200)
