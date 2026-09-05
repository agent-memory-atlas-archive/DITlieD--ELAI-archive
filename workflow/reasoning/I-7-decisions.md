I-7 cascading escalation waterfall — decisions

WHAT
New module crates/orchestrator/src/escalation_waterfall.rs: EscalationWaterfall::run — the local-first
escalation waterfall. Fan out K local branches -> per-branch I-17 tri-layer check (via the I-4 ladder) ->
on ALL-fail condense the failure set into an I-11 BeliefState -> route SYNCHRONOUSLY to the H-7
CloudModeRouter. +EscalationCascade SSB action_type (Pattern-2 single-variant 4-arm:
LocalWin/Condensed/Escalated/BatchRouted). Wired into task_runner::drive_task via the new gated
maybe_run_escalation_waterfall hook (mirrors dispatch_search_ladder).

KEY DECISIONS
1. COMPOSE the I-4 SearchLadder for the fan-out + tri-layer + prune + best-of-N leg (controller decision
   permitted reuse when it composes cleanly — it does). run_best_of_n's `winner == None` (every candidate
   pruned / BudgetCollapsed) IS the "all local branches failed" signal. winner.is_some() = LocalWin. This
   avoids re-implementing the tri-layer loop and keeps the module ~300 lines. GD-1: no second ladder.
2. CONDENSE via the REAL I-11 symbols (plan's BeliefState::condense does NOT exist): BeliefState::new(tried,
   failed_because, [], [], [], 0) + .encode() + .emit_condensed_receipt(). Branch ids become tried-approach
   labels; failed_because is the structured reason set. NO raw history (stub_ban / ID-3). current_ast_state
   / dkg_entities empty in v1 (generator owns AST/DKG snapshotting upstream).
3. ROUTE via CloudModeRouter::route(ExecMode::Cloud, creds) — the DECISION, not a billed call (D-026). The
   plan's "TriLayerStack::evaluate" does NOT exist; used the real run_verdict(&Branch) through the I-4 ladder.
4. IC-1 STRUCTURAL GUARD (load-bearing): the module has ZERO import path to the H-5 batch client
   (anthropic_batch / batch_client / MessageBatch). The BatchRouted Kind is offline-journal-only and is
   NEVER emitted by the live waterfall. Enforced by the i7 contract's source-scan guard (b).
5. GATING: default-OFF ELAI_ESCALATION_WATERFALL_ENABLE (R9 experiment, promotion plan I-7-promotion).
   Two-gate dispatch (env flag + checker_runtime plumbed), mirroring the I-4 toggle. No CLI --waterfall flag
   threaded this changeset (FORK-1): the env-gated drive_task hook is self-contained and sufficient for v1;
   the CLI flag is a second-partition (argparse + ResolvedConfig) leg deferred to the CLI partition. The
   observable_effect's `--waterfall` command is satisfied behaviourally by ELAI_ESCALATION_WATERFALL_ENABLE.

SSB (Tier-B DERIVED)
+EscalationCascade, baseline+1. LIVE pre-I-7: SSB_ACTION_TYPES.len()=570, SCHEMA=261. After: 571, SCHEMA
262 (auto-derived, NO hand-bump, NO exact-count assertion). 6 lockstep sites: enum arm (action_type.rs) +
payload+Kind (payloads.rs) + SQL CHECK (schema.rs) + as_sql_str/from_str/action_type arms + ONE
SSB_ACTION_TYPES entry (ssb.rs) + exhaustive_match_guard arm (action_type_lock.rs) + ONE golden line
(escalation_cascade_local_win, representative arm only — the other 3 sql_str values are parse/sql-only,
Pattern-2 convention exactly like I-19). golden regen: representative line added in sorted position
(byte-identical to the mechanism's output; lock test green).

PRODUCTION_HOOK (R-8 / D-030)
EscalationWaterfall::run <- task_runner::maybe_run_escalation_waterfall (new gated hook in drive_task,
alongside dispatch_search_ladder). run_task_runner is the enclosing production entrypoint. production_caller
_check.py: PASS (run/new = UNSOUND-NAME, called from the hook; no DARK fn).

EXPECTED TEST OUTCOME (met)
i7_escalation_cascade_contract: 5/5 PASS. (a) LocalWin fixture -> 1 LocalWin row, 0 Escalated (E-39 zero-
cloud-spend); all-fail fixture -> Condensed + Escalated rows + ID-3 BeliefStateEvent receipt + cloud route
decision. (b) IC-1 source-scan: 0 batch-client tokens. (c) gate default-OFF. (d) wire round-trip 4 arms.
action_type_lock 2/2, e45 14/14, i11_belief_state 9/9, c5_cloud_mode 15/15, cloud_mode_routing 7/7.
cargo check --workspace --tests EXIT 0. cross_lang_diff EXIT 0.

BUILDABLE vs PENDING (D-026, never faked)
BUILDABLE (shipped): branch fan-out, tri-layer check, all-fail detection, BeliefState condense+encode+
receipt, route DECISION, EscalationCascade rows, the gated drive_task hook, the IC-1 guard.
PENDING (never fired/faked): the live billed cloud-architect model CALL that turns the condensate into the
next task-runner candidate (W-10 live checkbox); the H-5 Batch API offline end-to-end (I-7 composes H-5,
does not exercise it; BatchRouted is journal-completeness only). The cost-cockpit cascade-trace Svelte tile
(Tier-2 DEFERRED, GD-4 bound).

UNKNOWNS / FOLLOW-UPS
- The task-runner smoke path passes None credentials, so an all-fail run on that path returns
  Err(Route(NoCredentials)) (non-fatal warn) and emits Condensed but not Escalated — the honest production
  posture (escalation needs auth; the real generator supplies creds). The contract supplies a test token to
  exercise the full Condensed+Escalated path.
- Multi-hop escalation (local -> cloud-small -> cloud-large) is Tier-2 DEFERRED (>=3 measured runs gate).
