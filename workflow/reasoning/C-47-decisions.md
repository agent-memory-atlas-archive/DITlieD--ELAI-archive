C-47 — Case-Based Calibration of Tool-Use Telemetry (closure session 2 of s-25120 cluster)

Plan: docs/thefinalglance/C-plan/C-47_case_based_calibration_tool_use.md
Paper: arXiv:2605.15041 (CAST) — workflow/papers/case-based-calibration.md
Schema bump: 121 -> 122 (atomic +3 SsbActionType variants)
TOTAL_VARIANTS bump: 340 -> 343 (atomic)

DECISIONS

D-1. Aggregate case base, NOT per-query nearest-neighbor. The paper extracts
     AGGREGATE signals (complexity profile H in [0,1] + 5-dim failure
     profile) ACROSS the historical case base — it does NOT do per-query
     KNN retrieval. The plan's "CaseRetriever with Jaccard + Levenshtein"
     framing extends the paper, not follows it. C-47 ships the paper's
     aggregate pattern: one AggregatedCase per (tool, call_shape_hash) in
     a per-tool LRU<u64, AggregatedCase> capped at 100. See plan rebase
     point 3 + workflow/papers/case-based-calibration.md
     "Implementation pitfalls" section.

D-2. Persistence: in-memory LRU (lru = "0.18" already in workspace deps);
     NO new sled crate per CLAUDE.md anti-slopsquatting HARD RULE. The
     plan's "sled preferred" framing is WRONG — sled is not in the
     workspace; rusqlite 0.39 IS, but adding a persistent KV layer for
     the case base is not in C-47 scope (case base is in-process
     telemetry; persistence is a follow-on extension). Crash-recovery
     happens via the SSB row stream — the orchestrator can replay
     `tool_call_logged` rows to rebuild a registry on restart.

D-3. SSB path drift reanchored. Plan says
     `MODIFY .elai_cc/crates/orchestrator/src/ssb/schema.rs`. Reality:
     SSB lives at flat `crates/orchestrator/src/ssb.rs` (no ssb/ subdir).
     Same reanchor pattern as C-35 M3 (plan said `mcp/server.rs`,
     ssb.rs / mcp_bridge.rs is canonical). Edits land in ssb.rs at the
     SCHEMA_VERSION const + SsbActionType enum + as_sql_str + from_str +
     SsbPayload + action_type() + test_payload sample + all_test_payloads
     round-trip + DDL CHECK constraint.

D-4. Bridge integration in BOTH dispatchers. Extracted shared
     `consult_calibration` + `record_call` helpers on DefaultMcpBridge.
     Called from BOTH `dispatch_tool_call_with_gate` AND
     `dispatch_tool_call_with_authz` (Allow branch) at the symmetric
     pre-call and post-call points. The contract test asserts both
     call sites exist via include_str! parse.

D-5. Hint injection vehicle. Plan said "inject a hint message in the
     model's next prompt" without naming the seam. ELAI choice: emit a
     `tool_calibration_hint_emitted` SSB row carrying hashed hint text;
     the orchestrator's prompt-assembly path reads recent rows pre-turn
     and prepends the hint as a system-level message before the next
     user turn. Wire-dark friendly — matches the `tool_lazy_resolved` ->
     ToolGateInspector consumer pattern from C-35.

D-6. post_call_audit_ok derivation. Plan rebase point 6: synchronous
     heuristic against `McpCallResult.is_error`. Audit-loop integration
     (feed the auditor's verdict back into the bridge) is a follow-on
     extension explicitly out of scope for C-47. Documented as
     `RUNPOD-PENDING: audit-loop integration` in the plan's Files block.

D-7. LRU eviction determinism. The WIRING_CONTRACT stub_ban requires
     "eviction MUST be deterministic (oldest-by-timestamp, not random)".
     We use the `lru` crate's documented least-recently-USED semantics —
     deterministic by access ordering, not random. The contract test G4
     asserts cap stability across an over-insertion.

D-8. Hint suppression at the emit point. Stub_ban: "hint_suppression
     config MUST gate AT THE EMIT POINT, not after". Enforced by
     `ToolCalibrationRegistry::consult` returning `None` when
     `hint_suppression == true` BEFORE any SSB row is composed.
     Contract G6 asserts this.

D-9. Concurrent session 3 (s-31708). The concurrent-session warning at
     session entry said SCHEMA was 121. Confirmed via grep at execution
     time: still 121. Atomic bump 121 -> 122 fits cleanly. If session 3
     fires first the executor would need to rebase to 122 -> 123; that
     rebase is mechanical (find/replace `122` -> `123` in 3 sites:
     SCHEMA_VERSION const, TOTAL_VARIANTS, the +3 cascade landed
     together).

D-10. Dashboard surface lands as `CaseBaseInspector.svelte` mounted in
      SettingsWindow alongside ToolGateInspector (C-35) + AuthzPanel
      (C-42) + FreezeStatusBar (C-43). Backend at
      `dashboard-v2/src-tauri/src/commands_c47.rs` with two read-only
      commands `tool_calibration_status` + `list_tool_calibration_rows`.
      TS DTOs added to `dashboard-v2/src/lib/api/stub.ts`. The
      orchestrator's bootstrap is expected to wire a live
      `Arc<ToolCalibrationRegistry>` into AppState (CALIBRATION-PENDING
      — follow-on extension; current commands return empty Vec when the
      registry handle is absent, mirroring TOOL-GATE-PENDING semantics).

D-11. CLI surface lands as `elai calibration <status|reset-hints|export>`
      at `crates/cli/src/calibration_cmd.rs` with three sub-actions
      mirroring the C-46 routing pattern. The CLI synthetic fixture is
      deterministic + GPU-free so the surface has a verifiable
      observable effect even before the production task-runner wires
      a live registry.

D-12. Pilot replay (Tasks 7 — 50-session replay) is RUNPOD-PENDING —
      out of scope for the wiring close. The acceptance criterion
      "Pilot replay produces measured reduction in repeat errors >=10%"
      is gated against the live registry IPC seam, not the in-process
      synthetic fixture. Tracked under
      `workflow/sessions/s-25120/case-base-pilot.md` as a deferred
      artifact.

PAPER GROUNDING

Paper interface verified by paper-ingestor at
workflow/papers/case-based-calibration.md (arXiv:2605.15041, 2026-05-24
re-fetch). The invariants enforced in the C-47 contract test (G1..G7)
trace to:
  - G1 — Section 3.2 aggregate-case-base pattern
  - G2 — Section 3.2 5-dim failure profile (name, key, type, constraint,
         value)
  - G3 — Section 3.2 signal-conditioned hint emission
  - G7 — Section 3.2 hardness `H in [0, 1]`

The CAST framework's R_format / R_think / R_tool composite reward (Eq.
Section 3.3) is NOT in scope for C-47 telemetry — that lives in any
future fine-tuning extension. C-47 only ships the telemetry layer that
the paper's case base is built on.

OUT OF SCOPE / FOLLOW-ON EXTENSIONS

  1. Persistent case base (rusqlite-backed `case_base.sqlite3`) — current
     ship is in-process LRU; restart replays via SSB row stream.
  2. Audit-loop integration for post_call_audit_ok — currently
     synchronous `!is_error` heuristic.
  3. Live registry IPC seam to dashboard `CaseBaseInspector` —
     currently CALIBRATION-PENDING (empty Vec).
  4. 50-session pilot replay against real ELAI session logs — currently
     RUNPOD-PENDING.
  5. CLI `--case-base-off` env + flag — wire-dark default per D-10 /
     D-11 precedent; the registry is None by default.

CONCURRENCY NOTE

If session 3 (s-31708) also bumps SCHEMA before this commit, fold via
the cross-session-folding precedent (C-43 + D-8 pattern): rebase 121 ->
122 + N to 122 -> 122 + N + 3. The three new variants land
alphabetically in the golden file at ordinals 0312..0314, contiguous
with the rest of the `tool_*` prefix block.
