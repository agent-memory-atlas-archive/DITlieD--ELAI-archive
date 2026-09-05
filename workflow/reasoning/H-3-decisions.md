H-3 DECISION DOC — output-token stack (diff/FIM/streaming early-abort, the 5x-billed lever)
session s-66424 | 2026-06-04 | orchestrate H-plan WAVE 1 | composes-with B-28 (focus tunnel), B-20 (try_edit_tiers)

WHAT
Output bills 5x input and caching never touches it. Three output-token levers were dark: FocusTunnel DIFF
never emits in prod (double-blocked), FIM double-gated dark, streaming early-abort dark on ALL paths
(streaming_validator hardcoded None). H-3 activates the cleanly-tractable levers and registers the gate.

GROUNDED FINDINGS (3 explorers, live-verified; lines drifted in the shared G/H tree)
- StreamingEarlyTerminated SSB variant ALREADY EXISTS (B-28 E-4) — ACTIVATE the emitter, NO bump. LIVE SCHEMA
  210 / TOTAL_VARIANTS 487 (G run advanced it; uncommitted in the shared tree).
- The early-abort HOOK is ALREADY wired (runtime_client.rs:602-625): on_token -> EarlyTerm -> MessageStop +
  break. Dark only because streaming_validator is None (runtime_builder.rs:306) and no SSB row is written
  (only tracing::warn). RuntimeClient has NO Ssb handle (GAP) — needs a new streaming_ssb field.
- DIFF double-dark: BLOCKER-1 = mode.rs:184 Rule 1 forces Structured on ALL cloud (Api->Cloud) traffic;
  BLOCKER-2 = task_runner.rs:7878 passes editor_fs_override=None -> UnsupportedMode -> Structured fallback.
- BLOCKER-2 needs a production FileReadWrite (RealFs) adapter — NONE EXISTS (only test stubs StubFs/InMemoryFs);
  plus multi-site buffer_editor threading from the InProcessToolDispatcher (runtime_builder.rs:688) through
  TaskRunnerConfig -> ResolvedConfig. This is net-new + multi-site + risky under cargo-check-only.
- try_edit_tiers (B-20, dispatcher.rs:288) is the shipped 3-tier matcher — REUSE via replay_hunks (GD-1).

KEY DECISIONS
D1 (streaming activation — the clean half): add `streaming_ssb: Option<Arc<Ssb>>` to RuntimeClient (populated
   from the ssb_arc at runtime_builder.rs:177-210). Construct Some(StreamingValidator::new(
   StreamingValidatorConfig::default())) at runtime_builder.rs:306 GATED on ELAI_FOCUS_TUNNEL_EXECUTE==1 (else
   None — default-OFF: the early-abort over-abort rate is unbounded until >=3 live runs, GD-4). At the early-abort
   site, write the StreamingEarlyTerminated SSB row via `.await` (NOT block_on — we are inside the outer block_on
   async; a nested block_on panics). agent_id="cli:stream", superstep 0. Default config is telemetry-poor
   (Generic, empty step_id) — the per-BabyStepSpec config is the focus_tunnel EXECUTE-path follow-on.
D2 (DIFF SELECTION — lift BLOCKER-1): narrow mode.rs:184 Rule 1 so cloud EditExisting falls through to the
   FIM/DIFF rules (cloud CreateFile + non-edit still force Structured). With the flag OFF the focus-tunnel
   EXECUTE branch (task_runner.rs:7799) is skipped so this is a no-op; with the flag ON cloud EditExisting
   SELECTS diff (the GenerationModeSelected SSB row shows mode=diff/exec=cloud — the off-agent artifact).
D3 (DIFF REPLAY — BLOCKER-2 DEFERRED, documented): the editor+fs injection that makes DIFF actually replay
   (instead of UnsupportedMode->Structured fallback) needs a NET-NEW production FileReadWrite adapter + the
   buffer_editor slot threaded from runtime_builder.rs:688's InProcessToolDispatcher through TaskRunnerConfig ->
   ResolvedConfig -> task_runner.rs:7878. Deferred to a focused follow-on (high net-new + multi-site risk under
   cargo-check-only). After H-3: cloud diff is SELECTABLE (flag-on) but REPLAY falls back to Structured
   gracefully — honest partial; the streaming half is the delivered output-token saving.
D4 (FIM): cloud Anthropic models do NOT support raw FIM sentinels (supports_fim stays FALSE for Sonnet/Opus) —
   FIM is local-only; on cloud, DIFF is the correct mode. No FIM change for the cloud path (correct, not a gap).
D5 (R9 allowlist): ELAI_FOCUS_TUNNEL_EXECUTE is a PRE-EXISTING unregistered default-OFF flag (a pre-existing
   W-2/R9 gap, not introduced by H-3). H-3 RELIES on it, so it is registered in wire-dark-allowlist.toml
   (type=experiment, owner=maintainer, expiry 2026-09-01, promotion_plan_id=H-3, smoke_test). NOTE: the allowlist is
   gate-frozen (R8); this legitimate addition will trip gate_freeze_check.py until RE-LOCKED out-of-band:
   `python .claude/tools/gate_freeze_check.py --lock`. Flagged for the user — this is a sanctioned flag
   registration, NOT gate-gaming.
D6 (no schema bump): StreamingEarlyTerminated already exists; SCHEMA untouched.

DONE-GATE (Tier-1; live = deferred Tier-2)
crates/contracts/tests/h3_output_stack_contract.rs (source-wiring): streaming_ssb field present; the validator
construction gated on ELAI_FOCUS_TUNNEL_EXECUTE; the StreamingEarlyTerminated SSB emit present at the abort site;
Rule 1 narrowed for cloud EditExisting; try_edit_tiers NOT re-implemented. cargo check -p elai-cli -p
elai-orchestrator -p elai_cc_contracts (SCOPED — the G-2 run_subcommand.rs:453 error breaks --workspace).
Test EXECUTION + the live early-abort/diff run = DEFERRED Tier-2 (sidecar-gated + GD-4 over-abort/diff-fallback
bound on >=3 runs before the flag flips default-ON).

RISK
R1 streaming_ssb threading: `ssb` (Option<Arc<Ssb>>) at runtime_builder.rs:177-210 cloned into the field; if
   moved before the struct literal, use .clone() at the literal (verify).
R2 nested block_on (GAP-4): the SSB emit at the abort site uses `.await` directly (inside the existing block_on
   async), never self.runtime.block_on.
R3 StreamingEarlyTerminatedPayload field types (token counts u32 vs u64) — used events.len() converted; auditor
   confirms the exact width.
R4 Rule 1 relax breadth: ONLY EditExisting falls through; CreateFile + non-edit cloud stay Structured (GAP-3).

DEFERRED (Tier-2 / follow-on): DIFF replay (RealFs + buffer_editor injection); per-BabyStepSpec validator config;
the GD-4 flag flip to default-ON; the live early-abort/diff measurement. NO commit (accumulate to run end).
