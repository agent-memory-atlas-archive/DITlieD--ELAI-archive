C-39 MCP Bridge CLI Surface — closure decisions
Session: s-25120 (completion-drive re-open of PARTIAL plan)
Date: 2026-05-23
SCHEMA_VERSION: 103 (LOCKED, no bump)
TOTAL_VARIANTS: 306 (LOCKED)
bijection: 245 (LOCKED)

OBJECTIVE

Close the 11 unresolved claims from the prior re-check
(5 PARTIAL + 3 MISSING + 3 NOT_WIRED) without touching SCHEMA_VERSION
or the 3 CliMcp* SSB variants that shipped at v89->v90 in s-68264.

PRE-RESOLVED DECISIONS (binding, from the executor prompt)

D1. Auto-launch design: IN-PROCESS. The plan asks for "co-process MCP
    server" but the OpenHands paper-grounded shape uses fastmcp, which
    is itself in-process from the CLI's perspective. DefaultMcpBridge
    already serves HTTP from default() and is consumed by both CliMcpClient
    and the dashboard. An in-process bridge bound to a free localhost
    port is simpler, requires no new `elai mcp-server` subcommand, and
    achieves the same end-to-end CLI -> HTTP -> tool dispatch surface
    the plan demands. The Started/Stopped SSB rows mark lifecycle even
    though no separate OS process exists. True co-process auto-launch is
    deferred to a follow-up (`--mcp-process-mode=co-process` flag scaffolded
    but not implemented). RATIONALE: KISS + matches OpenHands fastmcp
    pattern; CO-PROCESS-PENDING-FOLLOWUP for the multi-process tier.

D2. --export-ssb design: JSONL written to <path> at process exit. The
    flag lives on CliAction::Run (universal for `elai run`). Format is
    one JSON object per line with {"action_type", "payload", "seq",
    "created_at"} matching the existing ci_mcp_parity_check.py reader.
    Implementation hooks into the SSB broadcast subscription so any
    typed payload written during the run is captured (no DB query
    needed; ssb-broadcast is the canonical observable surface). On
    process exit / SIGINT, the buffered rows are serialized.

D3. mcp_dispatch.rs re-enable: ADD crates/cli/src/lib.rs with pub mod
    re-exports for args, mcp_client, run_subcommand. The integration
    test exists and the 5 C39.x tests were authored TDD-style; the
    only blocker is the missing lib.rs. Keeping bin-only would force
    deletion of an authored test file. RATIONALE: per CLAUDE.md
    "TDD tests passing BEFORE wiring" rule; restore tests, don't
    delete them. The lib target is pure re-exports — zero new
    behavior surface.

D4. 6 spec-drift fixes:
    - phase-4K.txt citation: replaced with mcp_bridge.rs + B-ful-d
      session memory (no MCP content in phase-4K.txt; verified by grep).
    - plugins/src/mcp/server.rs: resolved via C-35 `pub use mcp_bridge
      as server` alias already shipped. NO action this session.
    - commands/task.rs + commands/explore.rs: don't exist. Plan
      updated to cite commands.rs SlashCommand parsing.
    - OpenHands CLI framing: clarified as "matching fastmcp HTTP
      semantics" not "matching OpenHands CLI structure" (which doesn't
      exist).
    - Test IDs 4K.X.*: renamed to C39.1..C39.5 (already done in
      mcp_dispatch.rs file header).
    - docs/cli.md: authored at .elai_cc/docs/cli.md (canonical dir
      per existing mvp-*.md siblings).

D5. Production wiring: dispatch field on RuntimeWiring (Arc<dyn Dispatch>)
    + apply_task_runner_wiring populates it based on wiring.via_mcp.
    The DispatchTrait does NOT replace the existing legacy
    ToolDispatcher (C-13 IpcToolDispatcherAdapter) — it sits alongside
    as an opt-in tier consumed when wiring.via_mcp=true. The legacy
    path stays unchanged when via_mcp=false. RATIONALE: zero
    regression on the default path + clean opt-in surface; full
    Dispatch-replaces-ToolDispatcher consolidation deferred to a
    follow-up C plan.

D6. SSB emit policy:
    - CliMcpDispatch: emitted from McpClientDispatch::dispatch_tool on
      EVERY call (success or error). Carries tool_name + port.
    - CliMcpServerStarted: emitted from the auto-launch path when an
      in-process bridge is bound to a free port for this CLI run.
      Skipped when reusing an external listener (already advertised).
    - CliMcpServerStopped: emitted from the lifecycle Drop guard at
      runner teardown. Symmetric with Started.

ANTI-LAZY DISCIPLINE CHECKLIST

- Zero new TODO / FIXME / unimplemented! / pass-bodies. CO-PROCESS-PENDING
  and CLOSURE-PENDING-EXTERNAL markers are deferral notes (not stubs)
  per 0_SHARED_CONTRACT.md.
- No unwrap()/expect() outside tests/main; all fallible paths surface
  Err with descriptive messages.
- tracing for instrumentation; thiserror for typed errors; tokio::sync
  primitives in async paths.
- distinct-action-type contract preserved: in-process path keeps
  emitting McpToolCalled (legacy); via-mcp path emits CliMcpDispatch.
  ci_mcp_parity_check.py:9 already unions both.

DEFERRED-FOLLOW-UP (registered, not stubs)

- F1. True co-process auto-launch via a future `elai mcp-server`
  subcommand. The in-process path covers the parity-test need;
  multi-process isolation is a hardening tier.
- F2. Dispatch-replaces-ToolDispatcher consolidation. Today's plan
  layers Dispatch alongside the C-13 IpcToolDispatcherAdapter; a future
  C plan can collapse the two trait surfaces.
- F3. Live CI parity job exercise against a real binary build. The
  ci.yml job uses cargo-built CLI but the smoke is RUNPOD-PENDING for
  any path that requires a live sidecar.

FILES MODIFIED (this session)

1. .elai_cc/crates/cli/src/lib.rs                            (NEW: pub-mod re-exports)
2. .elai_cc/crates/cli/src/mcp_client.rs                     (replace get_tool_schema stub)
3. .elai_cc/crates/cli/src/run_subcommand.rs                 (auto-launch + Started/Stopped + --export-ssb sink)
4. .elai_cc/crates/cli/src/args.rs                           (add --export-ssb arg + export_ssb_path field)
5. .elai_cc/crates/cli/src/main.rs                           (pass export_ssb_path through)
6. .elai_cc/crates/cli/src/commands.rs                       (thread via_mcp through Task + Explore variants)
7. .elai_cc/crates/orchestrator/src/dispatch/in_process.rs   (real plugin-registry dispatch + SSB emit)
8. .elai_cc/crates/orchestrator/src/dispatch/mcp.rs          (real bridge call + SSB emit)
9. .elai_cc/crates/orchestrator/src/runtime_setup.rs         (Arc<dyn Dispatch> field + apply_task_runner_wiring branch)
10. .elai_cc/crates/cli/tests/mcp_dispatch.rs                (remove #![cfg(any())] + fix empire field drift)
11. .elai_cc/scripts/ci-mcp-parity.sh                        (real driver invoking python helper)
12. .elai_cc/.github/workflows/ci-mcp-parity.yml             (NEW: parity matrix job)
13. .elai_cc/docs/cli.md                                     (NEW: --via-mcp section)
14. .elai_cc/crates/plugins/src/mcp_bridge.rs                (add http_get_tool_schema)
