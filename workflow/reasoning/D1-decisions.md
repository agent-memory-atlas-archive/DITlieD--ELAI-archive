D1 decisions: Wire SSB indexing rows on CLI new_prompt path

What: Emit IndexingProgress(start) + IndexingProgress(complete) + DkgReady rows
to the SSB database when init_indexing_service runs a fresh index or a schema
rebuild on the CLI path (new_local / new_prompt with local backend).

Key decision 1: duplicate vs expose
Chose to DUPLICATE the writer logic in indexing.rs rather than expose
emit_indexing_progress / emit_dkg_ready as pub from task_runner.rs.
Reason: task_runner.rs helpers are private async fns tightly coupled to the
runner execution model. Making them pub would widen the module boundary for
a 15-line capability that belongs in indexing.rs. Duplication is 30 lines,
self-contained, and avoids cross-crate coupling issues.

Key decision 2: Arc<Ssb> vs Option<Ssb>
LiveCli.ssb was Option<elai_orchestrator::Ssb>. Ssb is not Clone. The
init_indexing_service function runs a new current_thread tokio runtime (same
pattern as try_precompute_role_steering in sidecar.rs) and needs to hold the
writer across two async write calls. This requires Arc<Ssb>.

Changed LiveCli.ssb from Option<Ssb> to Option<Arc<Ssb>>. This is a partial
implementation of the B-19B/C "promote LiveCli.ssb to Arc<Ssb>" refactor
described at main.rs:3461-3464. Affected sites:
  - LiveCli struct field declaration (line 2143)
  - build_runtime return type (line 6165): wraps Ssb in Arc before returning
  - LiveCli Drop impl (line 4811): Arc::get_mut().map(shutdown)
  - propagate_changes call (line 3552): &**ssb to deref Arc -> Ssb
  - create_handoff call (line 3398): &**ssb_ref to deref Arc -> Ssb

The HookAbortMonitor::spawn already expected Option<Arc<Ssb>> (line 2607)
and was called with None. It remains None (wiring HookAbortMonitor is B-19B scope).

Key decision 3: new tokio runtime for sync-to-async bridge
init_indexing_service is sync (called from sync constructors). SSB writes
are async. Using tokio::runtime::Builder::new_current_thread().build() and
block_on is the established pattern in this codebase (sidecar.rs:149,
HookAbortMonitor::spawn:2612). Applied same pattern here.

Key decision 4: agent_id and superstep for CLI-emitted rows
Used "cli_indexing" as agent_id (distinguishes from task_runner's
"task_runner" rows) and 0 as superstep (same as RUNNER_SUPERSTEP — dashboard
keys on seq, not superstep, per task_runner.rs:117).

Key decision 5: project_id
Used the current working directory's canonical path as project_id, matching
the pattern in indexing.rs and consistent with how dkg_db_path is computed.
The task_runner uses the task.project_id from the dispatched task; the CLI
path has no task object at indexing time, so cwd-derived id is the fallback.

Expected test outcome: after fresh index, sqlite query shows indexing_progress
count >= 2 (start + complete) and dkg_ready count >= 1.

Unknowns at planning time: whether Arc::get_mut succeeds in Drop if ssb
was shared with build_local_knowledge_stack. Analysis: build_local_knowledge_stack
receives ssb.clone(), uses it only during block_on (async writes), then drops
both the clone and the runtime. By the time LiveCli::drop runs, the clone is
long gone. Arc::get_mut will see count=1.
