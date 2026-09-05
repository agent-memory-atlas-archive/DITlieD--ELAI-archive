# J-2: elai-as-MCP-server control plane — drive a RUNNING elai from outside, security-gated

STATUS: PLANNED (WAVE 4 — deliberately last big build; BUILD; stdio local-only transport;
BLOCKED-until the security-precondition live re-check passes)

Objective: today the only way to drive elai is its own chat/CLI loop — one session, one operator,
gate-shaped like claude code. J-2 exposes elai's primitives as a long-lived MCP SERVER so an
external client (claude code, the dashboard, scripts) can submit tasks, poll status, read FSM
state, fork/rewind sessions, read plan status and RRAs, and answer interventions on a RUNNING
instance — a continuous control surface (U1: "direct mcp control, not per session or gate like
claude code"). Novelty verified (SRC-1 N-2 + pre-read): B-25 ElaiNeuron sketched 4 capabilities
but is DEFERRED on the Empire transport (RT-4); C-39 built the INVERSE direction — CLI as MCP
client (mcp_client.rs/mcp_glue.rs live in the tree, RT-3); I-40 wraps OTHER agents' traffic.
Nobody authored "elai exposes its own control plane". Hard boundary: every verb RIDES an existing
primitive and its existing gates — a verb that bypasses the FSM, the destructive-verb gates, or
the journal is a P0 (GD-1/GD-2). Without the security spine this is a remote-code-execution hole;
with it, it is defensible — hence the blocking PRE_GATE.

Estimated effort: ~2 sessions (~12-16 h). Tier note: pure in-process + stdio; no network leg
(TCP explicitly out of scope this plan).
Pre-requisites (BLOCKING, re-verify LIVE at execution per RT-5 — FINISH 06-07b statuses are
static-only and moving):
  - C-42 #[authz(states,actors)] + DENY-ALL default: live or this plan stalls.
  - E-24 blake3 schema pinning + G-4 rug-pull re-pin: live (G tally was 8 wired / 2 dark —
    confirm G-4 specifically).
  - E-30 argv destructive-verb deny + G-3 post-expansion blast-radius gate: live behind every
    exposed verb that can touch disk/processes.
Parallel-safe with: J-3/J-4/J-5/J-6/J-7. NOT J-1 (shared cli dispatch seam).
Source: SRC-1 N-2. Doctrine: J-0_INDEX.md.

## Context
The control-plane shape is field-convergent (every serious agent system grows one), but ELAI's
version has a property the others lack: the journal IS the state, so an external client gets
exactly the same evidence surface as the interactive operator — no parallel bookkeeping, no
second source of truth (GD-2). The verb list is B-25's capability set minus the Empire transport,
extended with the primitives shipped since: task_submit / task_status (task_runner), fsm_get_state
(FSM), session_fork / session_rewind (the A-1 chain: H-35 rewind + H-49 time-travel + I-52 tool —
whichever is live at execution; the verb degrades to "not available" with a typed error when the
chain is unshipped, never a reimplementation), plan_status (I-39), rra_show (G-2 rra.rs, SHIPPED),
intervention_respond (B-6 intervention_request/response SSB pair). DENY-ALL: a verb without an
authz label does not exist to the client.

## Existing state (B..I coverage — why this is BUILD)
- B-25: 4-capability ElaiNeuron + ToolFilter, DEFERRED on Empire (C-34 external blocker). The
  capability list is reusable; the FactoryLine transport is not wanted here. B-25 stays deferred.
- C-39: --via-mcp CLI->client direction; mcp_client.rs + mcp_glue.rs in tree (RT-3). Its SSB rows
  (cli_mcp_dispatch / _server_started / _server_stopped) are the prefer-reuse candidates.
- I-40: `elai wrap` — ELAI's gates around OTHER agents. The inverse concern; no overlap.
- H-34/C-35: defer-load EXTERNAL MCP schemas INTO elai — the consumption side, untouched.
- Security spine: C-42 / E-24 / G-4 / E-30 / G-3 per RT-5 (statuses MOVING; re-check).

## Reads (mandatory before coding)
1. _grounding/J-HARNESS-GROUNDING-MAP.md (RT-3/RT-4/RT-5, JC-3).
2. SRC-1 N-2 + the B-25 plan file's capability/ToolFilter section (reuse list).
3. crates/cli/src/mcp_client.rs + mcp_glue.rs (the sibling direction's shapes + SSB rows; W-7
   re-grep — these files were mid-edit in the live tree 2026-06-10).
4. The C-42 authz attribute definition + its fuzz corpus (the DENY-ALL mechanism this plan rides).
5. The wire-remediation WORKING METHOD block (serial on shared cli files; sonnet agents).

## Tasks (WAVE 4)
1. PRE_GATE security re-check (~0 LOC, blocking): grep + run the live status of C-42/E-24/G-4/
   E-30/G-3; record verdicts in J-2-decisions.md. ANY miss -> the plan stalls at PLANNED with the
   named blocker (honest, R1).
2. Server skeleton (~200 LOC): crates/cli/src/mcp_server.rs (sibling of mcp_client.rs) — stdio
   transport, tool registry carrying ONLY authz-labelled verbs, DENY-ALL otherwise. NO TCP.
3. Verb implementations (~250 LOC): each verb a thin adapter onto its primitive — task_submit ->
   task_runner entry; fsm_get_state -> FSM read; session_fork/rewind -> the A-1 chain IF live,
   else typed NotAvailable; plan_status -> I-39 derivation; rra_show -> rra.rs read; 
   intervention_respond -> the B-6 SSB pair write path. Every verb passes the SAME FSM/destructive
   gates an interactive call would (no privileged path).
4. SSB: prefer reuse — C-39's cli_mcp_* rows if shipped (verify at edit); else ONE
   +McpControlPlaneEvent (baseline+1 from LIVE 221/532, re-read at edit, 6-part discipline).
   Verbs otherwise emit the EXISTING rows of the primitives they ride (GD-2: one journal).
5. Concurrency safety (~80 LOC): a control-plane call arriving mid-FSM-state respects the state
   machine (a task_submit during EXECUTING queues or refuses per FSM rules — never a second
   concurrent driver of the same session); session-scoped locks reused from the existing
   cross-session machinery.
6. Client smoke: drive the server from an external MCP client fixture (claude-code-shaped):
   submit fixture task -> poll -> read rra. Plus the adversarial arm: unlabelled verb, destructive
   verb without its gate verdict, malformed frame.
7. frontend_pair: control-plane connection + verb rows in an existing sessions/monitor drawer
   (or W-6 justification in the decisions doc).
8. Decision doc: workflow/reasoning/J-2-decisions.md (verb list final, authz labels, transport
   posture, SSB reuse-vs-new verdict, the security re-check evidence).

## WIRING_CONTRACT
produces_symbols:
  - crates/cli/src/mcp_server.rs (the server + verb registry)
  - the authz-labelled verb set (task_submit/task_status/fsm_get_state/session_fork/
    session_rewind/plan_status/rra_show/intervention_respond)
consumes_symbols:
  - task_runner entry (task_submit/status), FSM read API, rra.rs::build/show (G-2),
    I-39 plan-state derivation, B-6 intervention SSB pair, the A-1 rewind chain (conditional)
  - C-42 authz attributes + DENY-ALL; E-24/G-4 pin+re-pin; E-30/G-3 gates (verdicts consumed,
    never re-implemented)
reachable_from:
  - `elai mcp-serve` CLI entrypoint (new arm in the existing CLI dispatch)  [PRODUCTION]
  - an external stdio MCP client connecting to it (the JD-1-style live consumer)
stub_ban:
  - NO verb without an authz label (DENY-ALL is structural, not advisory)
  - NO privileged bypass path around FSM/destructive gates ("the server is trusted" is banned)
  - NO parallel state store (the journal is the only truth; a server-side cache that answers
    status without the journal is a violation)
  - NO TCP listener; NO .unwrap()/.expect() on prod paths
frontend_pair:
  control-plane rows in an existing drawer, or W-6 justification recorded. 
observable_effect:
  trigger: "`elai mcp-serve` + an external MCP client fixture: task_submit(fixture) -> poll
    task_status -> rra_show; then the adversarial arm"
  expect_ssb: "the SAME SSB rows an interactive run of that fixture produces (diffable under
    ELAI_SEED, GD-2) + the server lifecycle rows (reused C-39 family or the one new variant)"
  expect_artifact: "client receives status/rra payloads byte-derivable from the journal.
    NEGATIVE: an unlabelled verb returns DENY-ALL refusal; a destructive verb without its gate
    verdict is BLOCKED; the --skip-authz broken arm exposes the verb and FAILS the contract test
    (R2)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-2_elai_mcp_control_plane.md --run-trigger
rg "mcp_server" .elai_cc/crates/cli/src/                                  # server exists + registered in CLI dispatch
rg -c "DangerouslySkip|skip_authz" .elai_cc/crates/cli/src/mcp_server.rs  # MUST be 0
rg "authz" .elai_cc/crates/cli/src/mcp_server.rs                          # every verb labelled
test -f workflow/reasoning/J-2-decisions.md
```

## ACCEPTANCE
- [ ] PRE_GATE: C-42/E-24/G-4/E-30/G-3 live-verified with evidence in the decisions doc; any miss
      = plan stays PLANNED with the named blocker.
- [ ] every verb rides its primitive + its gates; the interactive-vs-control-plane journal diff is
      empty for the same fixture (GD-2).
- [ ] DENY-ALL default proven by the unlabelled-verb arm; destructive-verb arm BLOCKED.
- [ ] SSB prefer-reuse verdict recorded; if a variant was added, 6-part discipline + baseline
      re-read (W-9) + cross_lang_diff PASS.
- [ ] stdio-only; no TCP code present (grep-proven).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-cli (+orchestrator if touched).
- [ ] no new dependency (the MCP transport rides what mcp_client.rs already uses; anything else
      needs approval).

## Tests to add
- crates/contracts/tests/j2_control_plane_contract.rs (~140 LOC): (a) unlabelled verb ->
  DENY-ALL; (b) task_submit fixture produces journal rows identical to the interactive arm under
  seed; (c) destructive verb without gate verdict -> BLOCK; (d) rewind verb with the A-1 chain
  absent -> typed NotAvailable (no reimplementation, no panic); (e) malformed frame -> typed
  error, server lives; (f) --skip-authz broken arm FAILS.

## Files touched
- CREATE: crates/cli/src/mcp_server.rs (~450 LOC)
- MODIFY: cli args/dispatch (one `mcp-serve` arm)
- MODIFY (conditional): ssb.rs + action_type_lock if the reuse verdict says new variant
- CREATE: workflow/reasoning/J-2-decisions.md

## Citations / Grounding
- SRC-1 N-2 (the verb list + precondition set). RT-3 (C-39 inverse direction live), RT-4 (B-25
  deferred reuse), RT-5 (security spine status, moving — re-verify).
- Hard rules: DENY-ALL/C-42, GD-1/GD-2, W-9, serial-on-shared-cli-files (wire-remediation method).

## Open questions / risks
- C-39's exact shipped surface (its files were mid-edit 2026-06-10): the reuse-vs-new SSB verdict
  and the transport plumbing choice are decided at edit after a fresh read (W-7).
- multi-client behavior (two external clients on one elai): out of scope v1 — single client,
  documented; concurrent drivers refused per task 5.
- auth beyond same-user-stdio: explicitly deferred; any TCP/remote story is a NEW plan with its
  own threat model (GD-5 honesty: this plan's guarantee is local-only).
- the A-1 chain may still be unexecuted at J-2 time: the rewind verbs ship as typed NotAvailable
  until it lands (honest residue, never a stub that lies).

## Next
Closes U1: elai becomes drivable as infrastructure, not just a chat loop. Pairs with I-40 (wrap
others / be driven by others = the two halves of the trust-layer wedge) and gives the dashboard
and future automations one sanctioned, gate-respecting door.

## J-15 AMENDMENT — conductor BSP patterns + MCP-config delegation + transparent-proxy shape (source-cited)
Grounding: determinism triage ledger items 5 + 9
(workflow/reasoning/steal-determinism-repos-2026-06-11.md); microsoft/conductor script-step +
set-step patterns (agent det-repos-S3 verdict); Balchandar/intentusnet WAL + transparent-proxy
shape (agent det-repos-S1 verdict).

### (a) Script-step with exit-code routing + LLM-free set-step as BSP node shapes
If J-2 grows workflow verbs beyond the v1 single-client control-plane surface, two BSP
(phase-3D) node shapes from microsoft/conductor are stealable as EXTENSIONS (not v1 scope):
- **Script-step with exit-code-only routing**: a BSP node whose action is a deterministic
  script (not an LLM call) whose exit code routes the next superstep (0 = continue, non-zero =
  branch to a named fallback). This is the LLM-free deterministic-routing primitive: the
  script runs under Tier-C sandbox, the exit code is the sole routing signal, no LLM in the
  path (TD-3). Composes with the existing BSP barrier->apply-writes-by-SSB-seq dispatch.
- **Set-step (LLM-free derived values)**: a BSP node that computes a derived value from
  prior SSB rows (e.g., `set step_count = count(SessionEvent where session=X)`) and writes
  the result as a new SSB row, without any LLM call. This is the deterministic-aggregate
  primitive: the BSP superstep carries forward a computed value, not a model inference.
Both are BSP node SHAPES, not new FSM states. They extend the existing `JoinSet` +
`spawn_parallel` dispatch with two new node types. Land only if J-2 grows workflow verbs
(the v1 scope is the MCP control-plane surface, not a workflow engine).

### (b) MCP tool-config DELEGATION to the host runtime
J-2's MCP server MUST NOT embed MCP server configuration (tool definitions, resource URIs,
transport settings) inside elai workflow definitions or plan files. MCP tool config is
DELEGATED to the host runtime: the external client (Claude Code, Cursor, etc.) owns its own
tool definitions and passes them via the MCP `tools/list` handshake. elai's MCP server
advertises the verbs it supports (task_submit, fsm_get_state, etc.); it does NOT carry the
client's tool catalog. This is the microsoft/conductor pattern: the conductor delegates tool
config to the host, never embedding it in the workflow definition. Rationale: embedding tool
config in workflow definitions couples the workflow to a specific client's tool surface and
breaks portability; the MCP protocol already handles this via the handshake.

### Transparent-proxy shape note (intentusnet, for the recording leg)
If J-2 adds a recording/replay leg (logging MCP calls for audit), the intentusnet
transparent-proxy shape is the correct architecture: `client -> gateway -> server` with ZERO
changes on either side. The gateway intercepts the MCP traffic, journals it to the SSB, and
forwards it unmodified. The client sees a normal MCP server; the server sees a normal MCP
client. The gateway is a pass-through, not a translator. This is the same shape as the
existing trust-layer wrapper (I-40 `elai wrap`): intercept, journal, forward. Do NOT build a
proxy that translates or transforms MCP messages — that breaks the transparent-contract
invariant and couples the recording leg to the MCP protocol version.

End of J-2.
