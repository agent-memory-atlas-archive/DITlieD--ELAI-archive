H-15 decisions — spawn_view reply channel + taint-gated deixis + snapshot/delta
Session: s-10612 | Date: 2026-06-04 | Executor: main LLM

RESOLVED DESIGN DECISIONS (do not re-litigate)

D-1: CanvasViewKind arm set
  Choice: ViewSpawned / UiAction / StateCheckpoint
  Rationale: H-0_INDEX §18 is the authoritative discriminator list for H-15.
    CANVAS-WORKBENCH-DESIGN.md §18 listed "ViewSpawned / OpApplied / UiAction" in
    an earlier pass; OpApplied was SUPERSEDED when H-13 shipped as a separate
    CanvasOpApplied variant (frozen, wiring-enforcement R8). StateCheckpoint is the
    snapshot/delta anchor for per-tab reconciliation (H15-3 slice) — a distinct
    Kind that CANVAS §18 did not anticipate because H15-3 post-dates that section.
    CanvasViewEvent does NOT subsume CanvasOpApplied; the three transports are
    independent and each carries its own 9-tuple lockstep footprint.
    Shipped variants CanvasActivityEvent (H-11) and CanvasOpApplied (H-13) are
    frozen. CanvasViewEvent is the third and final canvas SSB transport for the
    H-plan wave-2 canvas.

D-2: taint gate for CanvasUiAction::Select
  Choice: reuse capability_flow::taint_text (synchronous regex fast-path) PLUS
    the live G-1 chain (safety::taint_flow::TaintFlowCtx::lookup_for_argv) when
    the tainted_context thread-local is populated (stronger gate, same production
    call site as production_agent_rpc.rs:179 enforce_taint_sink_gate).
  Rationale: taint_text detects injection patterns in raw text (regex-based,
    synchronous, no state). TaintFlowCtx catches verbatim-substring propagation
    from a registered ingest source (state-based, more precise). Both are
    necessary-not-sufficient alone. Combining them gives defence-in-depth at the
    Select handoff without re-implementing either primitive (GD-1).
    GD-5 honest framing: the Select gate is necessary-not-sufficient until the
    full data-plane taint propagation is wired to register ALL web-sourced ingest
    at input time. Current coverage: regex patterns + any content registered via
    TaintFlowCtx.register() before the Select fires.
  Emit on Reject: emit_taint_flow (taint_flow_ssb_adapter.rs) fires on BLOCK
    (allow_tool_with_origin returns Reject). On FLAG-only (taint_text detects but
    TaintFlowCtx finds no registered provenance): attach provenance flag to
    context without blocking, emit UiAction canvas_view_event row with
    taint_flagged=true in op_data.

D-3: G-2 RRA composition
  Choice: no new RRA API. spawn_view StateCheckpoint rows emit into the shared
    Arc<Ssb> stream. build_rra seals all SSB rows (including ViewSpawned,
    UiAction, StateCheckpoint) into the RunArtifact at run end via emit_run_artifact
    (run_subcommand.rs:457). GuaranteeLevel::ReproducibleActions (honest — GD-5).
  Rationale: the compose-not-reimplement posture (GD-1). A "view produced by run X
    reproduces from the journal" claim is satisfied by the existing RRA bundle
    containing the ViewSpawned + seed_ops rows. No new guarantee tier needed.

D-4: spawn_view default-ON gating (GD-4)
  Choice: spawn_view is gated behind a routing-confidence bar. Default is
    text-unless-confidence-clears (suppress-canvas hatch). Not default-ON until
    over-block rate measured on >=3 real runs. The ELAI_CANVAS_SPAWN_ENABLE
    env flag controls the autonomy gate; absence = text-only mode (conservative).
  Rationale: Mosca arXiv:2103.01701 counter-evidence; CANVAS §20.1/§20.5.
    arXiv:2603.26942 is motivating-not-proven (n~10, single domain) — kept out of
    done-gate per CANVAS §20.2 and GD-5.

D-5: snapshot/delta mechanism
  Choice: per-tab sequence numbers tracked in canvasViewState.svelte.ts store.
    On first mount: poll all recent canvas_view_event rows, record last_seen_seq
    from the SSB row seq field, set checkpoint from most recent StateCheckpoint row.
    Incremental: poll rows where seq > last_seen_seq. Periodic StateCheckpoint
    emission: every CANVAS_CHECKPOINT_INTERVAL_OPS canvas_view_event rows (named
    constant = 50) OR on tab focus-gained, whichever fires first.
    Drift detection: if last_seen_seq jumps backward (reconnect) or exceeds
    MAX_DELTA_WITHOUT_CHECKPOINT rows (named constant = 100), request a fresh
    ViewSpawned snapshot from the backend.
  Rationale: Monk 2003 — pointing breaks if two parties see different display
    versions. SSB seq is a monotonic integer suitable as the rebuild key.
    No separate snapshot file (journal-backed, aligned with G-2 RRA replay).

D-6: arXiv:2603.26942 framing
  NOT in done-gate. Cited as motivating-not-proven (n~10, single domain, positive
  result was human code-injection not UI). CANVAS §20.2 is authoritative.

D-7: spawn_view 3-arm tool count
  H-13 shipped 7 canvas tools; catalog count was 23.
  H-15 adds 1 new tool (canvas_spawn_view): catalog count becomes 24.
  h13_canvas_ops_contract asserts 23 — H-15 does NOT change that assertion;
  h15_spawn_view_contract asserts the count is now 24 (in its own assertion).

D-8: CanvasUiAction::ToolCall gate
  ToolCall variant passes the F-1/F-2 write-time gate + explicit user consent.
  No auto-execute. Defined in canvas_ui_action.rs; the consent check is a
  runtime flag check (not wired to the live IPC gate in H-15 — that wiring is
  F-1/F-2's footprint). The field is present and documented; activation is
  F-1/F-2's job.

D-9: frontend drain separation
  canvas_view_event rows draining in canvasViewState.svelte.ts is a SEPARATE
  $effect from canvas_op_applied draining in canvasDispatcher.svelte.ts /
  CanvasPanel.svelte. Mixing them is a wire-dark R6 violation
  (gotcha OP_APPLICATION_TRANSPORT_IS_H13_DRAIN_PENDINGOPS).

EXPECTED TEST OUTCOME
  h15_spawn_view_contract.rs: (a) 3 CanvasViewKind rows round-trip in real SQLite
    DB; (b) Select taint gate REJECTS tainted content and emits taint_flow row,
    ALLOWS clean content; (c) snapshot/delta: seq-ordered StateCheckpoint +
    deltas reconstruct view state; (d) spawn_view in all 3 arms + catalog count 24.
  cargo check --workspace: GREEN.
  svelte-check: GREEN.

UNKNOWNS
  1. tainted_context() thread-local population timing: in the contract test context
     there is no live task_runner to pre-register ingest. The contract test uses
     taint_text() for the regex gate (which IS available without thread-local state)
     and registers content manually via TaintFlowCtx::register for the stronger
     gate test. This covers the contract; the live gate is the auditor's HD-4 job.
  2. GD-4 FP rate measurement: deferred per H-0_INDEX. No spawn_view default-ON
     until measured on >=3 runs.
