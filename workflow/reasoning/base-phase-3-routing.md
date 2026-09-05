BASE PHASE 3 — ROUTING + REGISTERED DEFERRALS
date: 2026-06-05
session: base-phase-3 PART 1 (3L/3C/3D/3E/3I high-value live wiring)
scope: items from base-phase-3-FINISH.md that are ROUTED to another plan or
       REGISTERED-DEFERRED (precondition genuinely absent). Items marked DONE
       in the session report are NOT repeated here.

------------------------------------------------------------------------------
ROUTED
------------------------------------------------------------------------------

3C.allowed_paths  ->  B-18 (B-18C BLOCK)
  Item: wire AgentHandle.allowed_paths into the tool-dispatch gate (tools/src/lib.rs
        is_hidden_path or a new per-agent gate fn) so per-agent path restrictions
        are enforced at runtime.
  Decision: NOT implemented here. The base-phase-3 reconciliation and the
        FINISH punch list both assign this to B-18's punch list (B-18C is the
        BLOCK that owns the tool-dispatch path gate). Implementing it in
        base-phase-3 would duplicate / conflict with B-18C.
  Owner: B-18 (B-18C).
  Status at this session: AgentHandle.allowed_paths field exists and is carried
        through recovery (recovery.rs recovered_handle preserves it), but it is
        NOT consulted by any tool-dispatch gate. That wiring stays with B-18.

------------------------------------------------------------------------------
REGISTERED DEFERRALS (precondition genuinely absent — honest comment left in code,
no silent TODO, no bare stub)
------------------------------------------------------------------------------

3D.apply_writes  (bsp/engine.rs apply_writes slot, ~line 314-316)
  Investigated: YES. The ChannelWrite<T> type EXISTS (bsp/channels.rs:233) but
        the agent-execution path produces ZERO of them. AgentPool::spawn_parallel
        (agent_pool.rs:224) returns Vec<AgentHandle>, not channel writes; there
        is no agent->engine ChannelWrite protocol yet (in-process pool returns
        only handles; out-of-process agents arrive in Phase 4A via IPC).
  Path taken: DEFERRED. Filling the slot with a real "sort by ssb_seq + apply"
        would require fabricating a data source — completed_handles carry no
        writes to sort. The existing comment at engine.rs:314-316 is already an
        HONEST deferral (it states exactly why the slot is empty and what future
        phases will do), not a silent TODO. Left as-is; no stub introduced.
  Precondition for promotion: an agent->engine ChannelWrite return protocol
        (spawn_parallel, or the 4A IPC AgentResponse path, must surface
        Vec<ChannelWrite<T>> from completed agents).
  Promotion target: Phase 4A (out-of-process agent pool) OR the 3F/3J agent-
        integration work that first makes agents emit channel writes. When that
        lands, fill the slot: collect channel writes from completed_handles,
        sort by ssb_seq ascending, apply each via InMemoryChannelRegistry::write.
  Note: the SIBLING item 3D.recover_from_checkpoint was the OPPOSITE case — its
        precondition (BspCheckpoint.channel_state carries the per-channel bytes,
        and InMemoryChannelRegistry can restore them) DID exist, so it was FILLED
        this session (restore_state + recover_from_checkpoint now rehydrate
        durable channels + a real test). See session report.

3D.recover_from_checkpoint — PRODUCTION CALLER deferral (partial)
  Done this session: the restoration LOGIC is now real (channels.rs restore_state
        + engine.rs recover_from_checkpoint loop rehydrates Topic/LastValue/
        uncleared-Ephemeral; unit test recover_from_checkpoint_restores_durable_
        channel_values proves a value is read back).
  Deferred remainder: BspEngine::run() does NOT yet CALL recover_from_checkpoint
        on startup — the engine only WRITES checkpoints (save_checkpoint), it has
        no resume-from-latest entrypoint. So the restoration is currently
        reachable only from the test, not from a production crash-resume.
  Precondition for promotion: a resume entrypoint in run() (or a TaskRunner-level
        resume) that, on (re)start with an existing checkpoints/superstep_N.msgpack,
        loads the latest BspCheckpoint and calls recover_from_checkpoint before
        the first superstep.
  Promotion target: the 3I/DST crash-resume integration (the FINISH note ties
        channel-state restoration to "when DST/3I integration starts"), or a
        dedicated BSP resume step. Until then the logic is wired + tested but
        the run()-side trigger is the open edge.

3E.tokencounter  (ContextBudget.validate_exact — TokenCounter abstraction)
  Item: expose a TokenCounter abstraction on ContextBudget.validate_exact to
        satisfy the spec token exact-validation supplement.
  Path taken: DEFERRED (low priority per FINISH punch list "do the smaller
        correct thing"). Introducing a TokenCounter trait + threading a concrete
        tokenizer through ContextBudget is a multi-call-site refactor (the budget
        currently uses the chars/4 estimate convention shared with
        TelescopeContext::estimated_tokens). Doing it half-way (a trait with no
        real tokenizer impl) would be a lazy stub. The chars/4 estimate is the
        documented convention everywhere else in the budget path, so it is
        internally consistent today.
  Precondition for promotion: a concrete tokenizer surface already plumbed into
        the orchestrator (e.g. the sidecar tokenizer count, or a tiktoken-style
        local counter) that validate_exact can call without a new dependency.
  Promotion target: the context-assembler / real-tokenizer work (same gate that
        3G's "real-tokenizer test" deferred item names).

------------------------------------------------------------------------------
OUT-OF-SCOPE SPEC AMBITION (declared, not implemented — zero code evidence)
------------------------------------------------------------------------------

3I.vcr — VCR cassette / TraceOps / SQLite VFS
  Finding: ZERO code evidence in the orchestrator crate. grep across
        crates/orchestrator/src for "VCR", "cassette", "TraceOps", "SQLite VFS"
        / custom VFS returns nothing implemented. These appear in the phase-3I
        spec as DST ambition (record/replay cassettes, a trace-ops layer, a
        custom SQLite VFS for deterministic I/O) but were never built, and no
        B-G plan remediates them.
  Decision: DECLARED OUT-OF-SCOPE SPEC AMBITION. Not a wiring gap (nothing to
        wire), not a stub (no code exists). The madsim-based DST harness (E-44)
        is the realised DST mechanism; VCR/TraceOps/SQLite-VFS were superseded by
        that approach. If deterministic record/replay is wanted later it is a new
        plan, not a base-phase-3 fill.
  Promotion target: a dedicated DST-record/replay plan (new), only if the
        madsim harness proves insufficient.

------------------------------------------------------------------------------
3I.madsim_metric — handled in code (not a routing item)
  metrics/phase_3I.toml madsim_compile_time_sec (was 83.0, flagged fabricated)
  removed + replaced with madsim_compile_time_sec_status = "UNMEASURED ...".
  A real re-measure needs RUSTFLAGS="--cfg madsim" cargo check with the E-44
  madsim dev-dep harness wired — out of scope this pass. Re-add ONLY with a
  reproduced measurement (cmd + wall-clock + date).

==============================================================================
PART 2 (session base-phase-3 PART 2 — 3B / 3F / 3H / 3J)
date: 2026-06-05
scope: peripheral + registrations + security. 3L/3C/3D/3E/3I handled in PART 1;
       3A/3G FOUNDATION_SOLID (untouched). Items DONE in code are summarized
       here; ROUTED / DEFERRED / measurement-BLOCKED items carry their gate.
==============================================================================

------------------------------------------------------------------------------
DONE THIS SESSION (code/doc landed — recorded for traceability)
------------------------------------------------------------------------------

3B.deadpool — DONE (decision: ACCEPT deadpool-sqlite as the r2d2 substitute).
  The phase-3B spec named r2d2+r2d2_sqlite; the SSB read pool uses
  deadpool-sqlite instead (ssb.rs:15155-15163, SsbPool::builder + deadpool::
  Runtime::Tokio1). This is CORRECT-BY-MODERN-PRACTICE, not a deviation to fix:
  CLAUDE.md HARD RULE Anti-Stale-Training mandates "deadpool/bb8 over r2d2".
  deadpool-sqlite gives async-native pooling (the SSB read path is async; r2d2
  is sync/blocking and would force spawn_blocking around every read). No
  correctness gap: both are connection pools over the same rusqlite Connection;
  deadpool adds async acquire + Tokio1 runtime parity. The writer side is a
  single dedicated connection (not pooled) either way. FORMAL ACCEPTANCE: the
  r2d2 spec line is superseded by the deadpool mandate; do NOT port to r2d2.
  (No code change — the existing deadpool wiring already satisfies the intent.)

3B.bench — DONE (harness); NUMBER BLOCKED_MEASUREMENT.
  Added crates/orchestrator/benches/ssb_throughput.rs + [[bench]] registration
  in orchestrator/Cargo.toml. Measures the END-TO-END durable single-producer
  SSB write path (write_payload -> redaction -> rmp_serde -> single-writer mpsc
  -> INSERT -> oneshot seq), awaiting each seq. Convention: harness=false +
  std::time::Instant (orchestrator D2/DRIFT-2: NO criterion dep — adding it
  would violate CLAUDE.md no-new-deps + the documented decision; the C-19
  plugins bench uses the same no-harness pattern). The 50K inserts/sec claim is
  BLOCKED_MEASUREMENT: the harness EXISTS but the number is only valid once run
  on a named host. Run: `cargo bench -p elai-orchestrator --bench ssb_throughput`
  then record throughput_inserts_per_sec + host/os/date. NEVER transcribe a
  fabricated number into the spec.

3F.shadow — DONE (doc). Documented the SHADOW-not-ACTIVE seed decision as a
  doc-comment on project_rule_seeder.rs SEEDED_STATUS const: auto-derived rules
  are MEDIUM trust and must EARN enforcement (5 TP + non-None CCR -> ACTIVE; 3 FP
  -> retire); security/finance seeds ship SHADOW too but their LOCKED thresholds
  are enforced by dedicated Rust gates that don't read the rules.db row, so the
  SHADOW row status doesn't weaken them.

3F.crosswalk — DONE. Cross-walked spec 3F.8 (phase-3F.txt:309-313) exact names
  against data/rules/*.toml. FINDING: the 3 post-tool completeness GREP rules
  ("web_fetch with <img>", "web_fetch with <table>", "file_read with
  TODO/FIXME") were ABSENT from BOTH data/rules/web.toml AND engineering.toml
  AND the rules crate (confirmed via grep for "not extracted"/"TODO/FIXME" —
  zero hits). They are ENGINEERING-domain Tier-1 PostToolUse rules per spec, NOT
  web-domain — and web.toml is contract-pinned at EXACTLY 27 rules
  (rules_web_contract.rs EXPECTED_RULE_COUNT=27, 12T1+11T2+4T3), so adding them
  to web.toml would BREAK that contract. Correct home = engineering.toml. ADDED
  ENG-013 (img/WARN), ENG-014 (table/WARN), ENG-015 (TODO|FIXME/INFO), all
  domain=engineering tier=1 scope=null phase_mask=all, with a 3F.8 crosswalk
  note + spec-name alias comment. engineering.toml has no count-pinning contract
  (only the web-vs-eng collision check, which ENG-0NN ids pass).

3H.msgpack_blob — DONE (choice: REGISTER, not wire). See DEFERRED section below.

3H.0.7b — DONE (doc). The JSONL-to-DAG migration is COMPLETE for the primary
  path: the live REPL persists via SessionDag::save_snapshot (session_dag.rs,
  SQLite DAG store), and the legacy session.rs JSONL writer is explicitly
  demoted — session.rs:419-424 doc states the JSONL store is NOT authoritative
  and "New callers MUST go through SessionDag::save_snapshot instead." The
  session.rs msgpack migration (Phase-0 GAP-P0-4) is FORMALLY DEFERRED here,
  consistent with the base-phase-0 deferral: session.rs is the legacy/secondary
  path being phased out, so re-encoding its on-disk format is wasted work — the
  DAG store (already msgpack-BLOB-capable via encode_blob) is the forward store.
  Promotion: only if a consumer still reads the legacy session.rs JSONL/json
  format directly AND needs msgpack there; none does today.

3J.9 — DONE (FAKE-SUCCESS REMOVED + registered). plugins/src/mcp_bridge.rs
  DefaultMcpBridge::connect stdio path previously returned Ok(McpConnection{
  transport: Stdio }) WITHOUT spawning a process or doing a handshake — a fake
  success. connect() NOW returns Err(McpError::ProtocolError("stdio MCP
  transport is not implemented; use the HTTP JSON-RPC transport ...")). EXACT
  new behavior: the stdio arm is honest-unimplemented; HTTP JSON-RPC
  (http_connect/http_list_tools/http_call_tool) is the supported transport.
  Non-breaking: the sole caller McpConnectionManager::connect_with_retry
  (mcp_lifecycle.rs:140-165) already treats Err as a retry/backoff-then-propagate
  failure; mcp_lifecycle tests assert only not-connected / heartbeat-timeout /
  graceful-degradation (no fake-success assertion). NOTE: the REAL stdio
  transport already exists in the ORCHESTRATOR crate (mcp_stdio.rs:
  spawn_mcp_stdio_process + McpServerManager::discover_tools_for_server, real
  tokio::process::Command, proven by spawns_stdio_process_and_round_trips_io) —
  the plugins DefaultMcpBridge is vestigial scaffolding, so the fake removal is
  pure safety with no functional loss. REGISTERED wire-dark-allowlist row
  ELAI_MCP_STDIO_TRANSPORT (type=experiment, promotion_plan_id=3J-stdio-followup,
  smoke_test=the real orchestrator stdio round-trip test).

3J.WIRE.RULES — DONE (security finding: NO GAP). Investigated whether SHADOW
  mode for seed rules creates a real evaluation BYPASS. VERDICT: NO BYPASS.
  Evidence chain:
    - lifecycle::fetch_all_active_rules (lifecycle.rs:317-341) is the SOLE
      production query feeding the rule engine; its SQL is
      `SELECT ... FROM rules WHERE status = 'ACTIVE' ORDER BY tier ASC`.
    - SHADOW rows are NEVER loaded into the evaluation set. A SHADOW rule is
      therefore never in a position to block AND is never EXPECTED to block —
      there is no "should-block-but-silently-doesn't" path.
    - RuleEngine::get_active_rules (rule_engine.rs:433-457) filters only by
      phase_mask + domain; it assumes the caller pre-status-filtered (which
      fetch_all_active_rules does). The status gate is at the DB boundary,
      fail-closed by construction.
    - The doc-SQL contract at rule_engine.rs:283 names status='ACTIVE' as the
      intended semantics; the production query honors it.
  Full rationale also lives as a doc-comment on project_rule_seeder.rs
  SEEDED_STATUS (3J.WIRE.RULES SECURITY FINDING block).

------------------------------------------------------------------------------
REGISTERED DEFERRAL (3H.msgpack_blob — REGISTER chosen over wire)
------------------------------------------------------------------------------

3H.msgpack_blob — CHOICE: REGISTER (wire-dark-allowlist), NOT wire. WHY:
  Investigated whether the columns are "meant to be live now". The msgpack
  SERIALIZATION is ALREADY real + wired — session_dag.rs:309-341 encode_blob/
  decode_blob (rmp_serde + JSON-legacy fallback), save_snapshot (line 553) and
  the production READER extract_code_change_paths (line 1453, used by
  merge_branch conflict detection) all round-trip the BLOBs. The columns are NOT
  inert at the storage layer (the FINISH "inert dead code" note is partly stale
  — B-9 T4 wired the codec + a real reader). What is None is the SOURCE DATA at
  the only production SAVE site, the live REPL (live_cli.rs:1474-1476): the chat
  loop has no structured findings/code_changes/tests_written — those are
  AGENT-produced (executor/auditor FSM), and the live REPL is not running that
  FSM. Populating them here would mean FABRICATING a data source (same
  anti-pattern as the 3D.apply_writes PART-1 deferral). So: REGISTERED deferral.
  Allowlist row: ELAI_SESSION_DAG_SNAPSHOT_BLOBS (type=experiment, owner=
  phase-3H, expiry 2026-09-01, promotion_plan_id=3H-agent-snapshot-followup,
  smoke_test=session_dag.rs::tests::merge_branch_conflict_detection — the
  existing test that populates code_changes Some(_) and reads it back via
  decode_blob). Promotion = the agent-integration snapshot writer fills the 3
  Snapshot fields via encode_blob.
  R8 NOTE: this edit to wire-dark-allowlist.toml (a gate-manifest.txt-frozen
  file) trips gate_freeze_check.py; re-lock OUT-OF-BAND with
  `python .claude/tools/gate_freeze_check.py --lock` after human review.

------------------------------------------------------------------------------
ROUTED (owned by another plan)
------------------------------------------------------------------------------

3F.bench  ->  C-19 (bench exists, numbers unrecorded).
  `cargo bench -p elai-plugins latency` -> metrics/phase_4C.toml. The bench file
  crates/plugins/benches/tiered_analysis_bench.rs EXISTS (harness=false, the
  T1/T2/T3 tiered-analysis latency groups); metrics/phase_4C.toml has the test
  counts but no recorded latency numbers. This is a C-19 punch-list item (record
  the measured latencies). NOT run here (scoped-test-run rule; main model never
  runs benches). Owner: C-19.

3H.banner  ->  C-23 (dashboard work consolidated in the C-plan).
  SSB banner renderer for branch_forked / branch_merged / branch_closed events
  in SessionsDrawer is a C-23 open action. Dashboard rendering is consolidated
  in the C-plan unit; building it in base-phase-3 would duplicate/conflict.
  Owner: C-23. NOT built here.

3J.2  ->  DEFER (RunPod/vLLM-deployment-gated).
  vLLM ModelPool swap: load_model/unload_model HTTP wrappers + a
  [providers.backend] CLI config consumer. PRECONDITION genuinely absent: there
  is no live vLLM server to target — the ModelPool swap HTTP calls need a running
  vLLM deployment (RunPod / a GPU box with vLLM up). Not tracked by any B-G punch
  list. DEFER until RunPod/vLLM deployment is live OR open an H-plan task.
  Promotion gate: a reachable vLLM HTTP endpoint + the [providers.backend]
  config surface.

3J.6  ->  DEFER (same gate as 3J.2).
  Background preloading runtime path (tokio::spawn dispatch + VRAM enforcement)
  is linked to the 3J.2 ModelPool swap (you preload INTO the pool). Same
  RunPod/vLLM precondition. DEFER with 3J.2; promote together.

3J.13  ->  patch-N / H-plan item (usearch --all-features CI compile failure).
  `usearch --all-features` fails to compile in CI (native C++ build-env issue —
  the usearch 2.x crate needs a C++ toolchain the CI image lacks). This is a
  BUILD-ENV / dependency-feature issue, not a base-phase-3 wiring gap. Track as a
  patch-N (CI image gains the C++ toolchain, OR pin usearch features to exclude
  the failing one) or fold into an H-plan CI-hardening task. NOTE: usearch-hnsw
  is already a registered wire-dark experiment (wire-dark-allowlist.toml:
  usearch-hnsw, owner phase-1C/B-16) — UsearchIndex is cfg-gated with zero
  production callers, so the --all-features failure does not affect default
  builds. Owner: patch-N / H-plan CI.

3K  ->  B-4 (MRR monotonicity re-run, BLOCKED_MEASUREMENT).
  Re-run benchmark_30_queries_writes_toml with cross-encoder-l12.onnx loaded and
  verify DEEP MRR crosses STANDARD MRR, or document the gap. Owned by the B-4
  punch list; BLOCKED_MEASUREMENT (needs the bench run with the ONNX model).
  NOT duplicated here.
