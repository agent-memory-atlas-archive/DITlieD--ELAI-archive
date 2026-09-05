# G-4: Live MCP rug-pull / line-jump re-pin gate

STATUS: PLANNED (WAVE 1; cheapest high-grounding move; no deps — extends the shipped schema-pin)

Objective: ELAI's MCP schema pin (plugins/src/mcp_schema_pin.rs: `pin_or_verify` + `hash_tool_list` +
`PinStatus` + the `mcp_schema_pin_violation` SSB row) is TRUST-ON-FIRST-USE — it pins a server's tool schema
at FIRST connect and never re-checks. A malicious or compromised MCP server can therefore SWAP a tool's
description or schema AFTER approval (a "rug pull" / "line-jumping" attack: the description is where injected
instructions hide). G-4 makes the pin LIVE: on every `notifications/tools/list_changed`, recompute the blake3
schema-pin via the existing `hash_tool_list` and BLOCK fail-closed on mismatch, requiring explicit operator
re-approval, emitting a new `mcp_rug_pull_blocked` SSB row carrying the before/after tool-DESCRIPTION diff.
Estimated effort: ~1 session (~5-8 h). Pure in-process / fail-closed; no network beyond the MCP connection ELAI
already holds; no RunPod (W-10 Tier-1 whole). No new dependency.

Pre-requisites: mcp_schema_pin.rs SHIPPED (grep-confirmed 2026-06-02: PinStatus:21, SchemaPinError:36,
PinRecord:49, ToolDef:57, hash_tool_list:69, pin_or_verify:134, update_pin:177; emits mcp_schema_pin_violation
per the module header :7). E-24 MCP version-pin SHIPPED (the version pin G-4 sits beside). E-43 skill-import
whole-directory scan SHIPPED (the publish/install-time sibling). The MCP plugin's tools-list handler is in
crates/plugins/src/mcp/* — confirm the exact `notifications/tools/list_changed` handler at authoring (W-7).
Parallel-safe with: G-1 (disjoint files), G-2. NOT parallel-safe with any plan editing mcp_schema_pin.rs or the
MCP plugin handler in the same window.
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R4. Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R4: ELAI's schema pin is TOFU. The MCP attack surface includes post-approval rug
pulls (a server changes a tool after the user trusted it) and line-jumping (malicious instructions injected via
a tool DESCRIPTION the model reads). GROUNDING (verifier-CONFIRMED): OWASP MCP03:2025 names "schema changes
accepted at runtime without operator confirmation" as a risk; Trail of Bits' line-jumping write-up prescribes
"alert on description changes"; Invariant Labs documented a WhatsApp MCP "sleeper" rug-pull. (CORRECTION carried
from G-0 landmines: mcp-scan is Invariant Labs, NOT Snyk; its dist-channel is UNVERIFIED — do NOT wire a
mcp-scan subprocess without anti-slopsquatting verification. Do NOT cite "CVE-2026-30623 in Anthropic's MCP SDK"
— that CVE is LiteLLM; say "the stdio command-injection family; Anthropic declined a protocol fix".)

ELAI ground truth (grep-confirmed 2026-06-02): mcp_schema_pin.rs already computes the blake3 schema hash
(`hash_tool_list`, :69), holds a `PinRecord` (:49), and verifies on connect (`pin_or_verify`, :134 ->
`PinStatus::Violation` -> the `mcp_schema_pin_violation` row). It can `update_pin` (:177). The machinery is
SHIPPED; the gap is purely the TRIGGER: it fires at first connect, not on `tools/list_changed`. G-4 adds the
live re-pin trigger + the before/after description diff + a distinct `mcp_rug_pull_blocked` action_type (so a
runtime rug-pull is distinguishable from a first-connect mismatch). This is the cheapest high-grounding G-N: it
extends a shipped module, has no external dependency, and closes a CONFIRMED attack class no first-connect scan
catches.

## Existing state (B/C/D/E/F coverage)
- mcp_schema_pin.rs (crates/plugins) — SHIPPED. hash_tool_list:69 (blake3 schema hash), pin_or_verify:134
  (TOFU verify), update_pin:177, PinStatus:21 (incl Violation), PinRecord:49, ToolDef:57, SchemaPinError:36,
  emits mcp_schema_pin_violation. G-4 EXTENDS this — re-uses hash_tool_list + the PinRecord; adds the live
  re-pin path. (GD-1 COMPOSES-WITH — re-implements none of it.)
- the MCP plugin handler (crates/plugins/src/mcp/*) — the connection loop that receives JSON-RPC notifications;
  the `notifications/tools/list_changed` handler is where G-4 hooks the re-pin. Confirm the exact symbol at
  authoring (W-7) — it is the live production MCP path.
- E-24 (MCP version pin) — the version pin beside the schema pin. E-43 (skill whole-directory scan) — the
  publish/install sibling. G-4 is the RUNTIME third bracket (first-connect pin -> install scan -> runtime re-pin).
- SSB (ssb.rs) — the action_type golden + Pattern-2 precedent for the new mcp_rug_pull_blocked row.

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R4 + §5 (the mcp-scan / CVE landmines)
- F:\.ELAI_workflow\.elai_cc\crates\plugins\src\mcp_schema_pin.rs (full — hash_tool_list, pin_or_verify,
  update_pin, PinStatus, PinRecord, ToolDef, the mcp_schema_pin_violation emit)
- F:\.ELAI_workflow\.elai_cc\crates\plugins\src\mcp\* (locate the tools/list_changed notification handler +
  the connection loop — the re-pin hook site, W-7)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (the action_type golden + Pattern-2 pattern)
- CLAUDE.md HARD RULES: blake3 (hash_tool_list already uses it), no .unwrap()/.expect() in prod, anti-slopsquatting.

## Tasks (WAVE 1)

1. **Add the live re-pin path (~90 LOC delta in mcp_schema_pin.rs).** New `pub fn reverify_on_list_changed(
   workspace_root, server_url, new_tools: &[ToolDef], prior: &PinRecord) -> RugPullVerdict` that recomputes
   `hash_tool_list(new_tools)` and compares to `prior` — returning `RugPullVerdict::{ Unchanged, Changed {
   description_diffs: Vec<ToolDescDiff>, schema_blake3_before, schema_blake3_after } }`. `ToolDescDiff { tool_name,
   before, after }` is computed per-tool so a line-jump (description swap with an unchanged name) is surfaced
   explicitly. Fail-closed: a Changed verdict on a previously-approved server BLOCKS the changed tool(s) until
   operator re-approval (reuse the existing operator-confirm path); `update_pin` is called ONLY after explicit
   re-approval, never automatically. blake3 via the existing hash_tool_list. NO .unwrap()/.expect().

2. **Wire reverify_on_list_changed into the MCP notification handler (~50 LOC delta in plugins/src/mcp/*).** On a
   `notifications/tools/list_changed` from a server with an existing PinRecord, call reverify_on_list_changed;
   on `Changed`, emit `mcp_rug_pull_blocked` + block the changed tools (the model cannot call a tool whose
   description changed since approval) until the operator re-approves. Default-ON (W-2): kill-switch
   ELAI_MCP_REPIN_DISABLE only; the observable_effect runs with NO override. NO LLM-bypass: the block is a pure
   hash mismatch; no prompt can wave it through.

3. **Add the `mcp_rug_pull_blocked` SSB action_type (~60 LOC in ssb.rs, W-9).** `SsbActionType::McpRugPullBlocked`
   + `McpRugPullBlockedPayload { server_url, schema_blake3_before, schema_blake3_after, changed_tools: Vec<String>,
   description_diff_count: u32 }`. sql_str "mcp_rug_pull_blocked" + parse + CHECK + round-trip golden +
   action_type_lock. Distinct from the existing mcp_schema_pin_violation (first-connect) so the dashboard can
   tell a runtime rug-pull from a first-connect mismatch. SCHEMA bump named baseline+1.

4. **Frontend pair: the before/after description diff (~160 LOC svelte + ~90 LOC tauri) (W-4).** Extend the MCP/
   security drawer (or a new McpRugPullDrawer.svelte) to render the mcp_rug_pull_blocked rows with the before/
   after tool-description diff (the human SEES exactly what the server tried to swap). Tauri `get_mcp_rug_pulls`.
   cross_lang_diff.py PASS.

5. **Decision doc workflow/reasoning/G-4-decisions.md (~80 LOC).** Records: (a) TOFU -> live re-pin gap; (b) the
   description-diff is the line-jump signal (Trail of Bits); (c) re-approval is operator-gated, update_pin never
   auto-fires; (d) the carried landmines (mcp-scan = Invariant Labs not Snyk + dist-channel unverified; no
   LiteLLM-CVE-as-Anthropic claim); (e) the three-bracket model (E-43 install scan / first-connect pin / G-4 runtime).

## WIRING_CONTRACT
produces_symbols:
  - crates/plugins/src/mcp_schema_pin.rs::reverify_on_list_changed
  - crates/plugins/src/mcp_schema_pin.rs::RugPullVerdict
  - crates/plugins/src/mcp_schema_pin.rs::ToolDescDiff
  - crates/orchestrator/src/ssb.rs::SsbActionType::McpRugPullBlocked
  - crates/orchestrator/src/ssb.rs::McpRugPullBlockedPayload
  - crates/dashboard-v2/src-tauri/src/commands_g4.rs::get_mcp_rug_pulls
  - crates/dashboard-v2/src/lib/drawers/McpRugPullDrawer.svelte
consumes_symbols:
  - crates/plugins/src/mcp_schema_pin.rs::hash_tool_list
  - crates/plugins/src/mcp_schema_pin.rs::pin_or_verify
  - crates/plugins/src/mcp_schema_pin.rs::update_pin
  - crates/plugins/src/mcp_schema_pin.rs::PinRecord
  - crates/plugins/src/mcp_schema_pin.rs::ToolDef
  - crates/plugins/src/mcp_schema_pin.rs::PinStatus
  - crates/orchestrator/src/mcp_rugpull_gate.rs::run_stdio_repin_gate
  - crates/orchestrator/src/mcp_rugpull_ssb_adapter.rs::OrchestratorRugPullSsbEmitter
# G-4-corrected 2026-06-03 (see workflow/reasoning/G-4-decisions.md DD1): the HTTP push handler does not exist;
# the gate runs on the stdio discovery funnel (mcp_stdio.rs:796 discover_tools_for_server -> run_rug_pull_gate_for_server).
reachable_from:
  - crates/orchestrator/src/mcp_stdio.rs::McpServerManager::discover_tools_for_server (:796, live stdio MCP
    tool-discovery funnel) -> run_rug_pull_gate_for_server -> mcp_rugpull_gate.rs::run_stdio_repin_gate ->
    OrchestratorRugPullSsbEmitter::emit -> mcp_rug_pull_blocked + tool blocked  [PRODUCTION stdio MCP path]
  - crates/dashboard-v2/src-tauri/src/commands_g4.rs::get_mcp_rug_pulls -> McpRugPullDrawer.svelte  [dashboard]
stub_ban:
  - "TODO: rug pull later" / "// re-pin later" literals absent
  - update_pin MUST NOT be called automatically on a Changed verdict — re-approval is operator-gated
    (auto-update would re-pin the malicious schema; a silent auto-update is a BLOCK)
  - NO default-off ENABLE flag (only ELAI_MCP_REPIN_DISABLE kill-switch; default ON, W-2)
  - NO LLM-bypass: the block is a pure hash-mismatch; no prompt/tool-input can wave it through
  - blake3 only (hash_tool_list); no blake2/sha2
  - NO mcp-scan subprocess wired without anti-slopsquatting verification (dist-channel UNVERIFIED — G-0 landmine)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/McpRugPullDrawer.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g4.rs::get_mcp_rug_pulls
observable_effect:
  # G-4-corrected 2026-06-03 (see workflow/reasoning/G-4-decisions.md DD1): the HTTP push handler does not exist;
  # the gate runs on the stdio discovery funnel. The live trigger is a reconnect-time re-fetch through
  # discover_tools_for_server (mcp_stdio.rs:796), not a server-pushed notifications/tools/list_changed.
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <connect a fixture stdio MCP server, approve it (first
    discover_tools_for_server pins the list), then restart/reconnect the fixture so it serves a swapped tool
    DESCRIPTION on the next discover_tools_for_server; run with NO env override (default ON, ELAI_MCP_REPIN_DISABLE
    unset)>"
  expect_ssb: "an mcp_rug_pull_blocked row with schema_blake3_before != after + the changed tool name +
    description_diff_count >= 1 in the live ssb.db (emitted by OrchestratorRugPullSsbEmitter::emit from inside
    run_stdio_repin_gate)"
  expect_artifact: "the swapped tool is BLOCKED (the model cannot call it) until operator re-approval; the
    McpRugPullDrawer renders the before/after description diff"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-4_mcp_rugpull_repin.md --run-trigger
rg "pub fn reverify_on_list_changed|pub enum RugPullVerdict|pub struct ToolDescDiff" .elai_cc/crates/plugins/src/mcp_schema_pin.rs
# G-4-corrected 2026-06-03 (G-4-decisions.md DD1): the list_changed handler does not exist; gate runs on the stdio funnel.
rg "run_rug_pull_gate_for_server|run_stdio_repin_gate" .elai_cc/crates/orchestrator/src/mcp_stdio.rs   # wired into discover_tools_for_server
rg "McpRugPullBlocked|mcp_rug_pull_blocked" .elai_cc/crates/orchestrator/src/ssb.rs
rg "mcp_rug_pull_blocked" .elai_cc/crates/orchestrator/tests/action_type_lock.golden
rg -c "ELAI_MCP_REPIN_ENABLE" .elai_cc/crates/plugins/src/mcp_schema_pin.rs   # MUST be 0 (kill-switch DISABLE only)
rg -c "blake2|sha2" .elai_cc/crates/plugins/src/mcp_schema_pin.rs             # MUST be 0
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/McpRugPullDrawer.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-4-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS; the observable_effect emitted
  mcp_rug_pull_blocked AND blocked the swapped tool. cargo-green != DONE.
- [ ] W-2 default-ON: a description swap on a previously-approved server is blocked with NO env override;
  ELAI_MCP_REPIN_DISABLE=1 disables it; no ENABLE flag.
- [ ] W-3 production entrypoint: the re-pin fires from the live MCP tools/list_changed handler (not test-only).
- [ ] W-6 dashboard E2E: McpRugPullDrawer renders the before/after description diff via dashboard-smoker.
- [ ] update_pin is NOT called automatically on Changed — a contract test asserts a swapped schema requires
  operator re-approval before the pin updates (auto-update would re-pin the malicious schema).
- [ ] A line-jump fixture (tool NAME unchanged, DESCRIPTION swapped) is caught (description_diff_count >= 1) —
  not just a schema-hash change.
- [ ] mcp_rug_pull_blocked round-trips + action_type_lock golden; distinct from mcp_schema_pin_violation; SCHEMA
  bump named baseline+1.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-plugins -p elai-orchestrator -p contracts.
- [ ] No new dependency (mcp_schema_pin.rs + blake3 in-tree; no mcp-scan subprocess without verification).

## WIRING DONE-GATE (clause by clause)
# G-4-corrected 2026-06-03 (see workflow/reasoning/G-4-decisions.md DD1): the HTTP push handler does not exist;
# the gate runs on the stdio discovery funnel (discover_tools_for_server -> run_rug_pull_gate_for_server ->
# run_stdio_repin_gate). W-1/W-3/W-7/W-8 below are repointed from the fictional list_changed handler to it.
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE 1; run_stdio_repin_gate (produced) is consumed by
  the stdio discovery funnel (W-8) reached by the live discover_tools_for_server path (W-3).
- W-2: only ELAI_MCP_REPIN_DISABLE; default ON; observable_effect runs with no override.
- W-3: the live stdio MCP discovery funnel discover_tools_for_server (mcp_stdio.rs:796) -> run_rug_pull_gate_for_server
  -> run_stdio_repin_gate; not test-only.
- W-4: mcp_rug_pull_blocked is human-surfaceable -> McpRugPullDrawer + get_mcp_rug_pulls; cross_lang_diff PASS.
- W-5: observable_effect is a runnable connect+swap fixture -> a live ssb.db row + a blocked tool. Pure
  in-process (the MCP connection ELAI already holds); no RunPod fixture needed.
- W-6: ACCEPTANCE verifies the diff THROUGH McpRugPullDrawer via dashboard-smoker.
- W-7: mcp_schema_pin.rs symbols grep-confirmed 2026-06-02 (hash_tool_list:69, pin_or_verify:134, update_pin:177,
  PinRecord:49, ToolDef:57, PinStatus:21). G-4-corrected 2026-06-03: the named tools/list_changed handler does not
  exist (grep across crates = 0); the gate is hooked into the stdio discovery funnel discover_tools_for_server
  (mcp_stdio.rs:787 -> :796 run_rug_pull_gate_for_server -> mcp_rugpull_gate.rs:141 run_stdio_repin_gate),
  validator re-greps the stdio funnel.
- W-8: run_stdio_repin_gate (produced) called by run_rug_pull_gate_for_server inside the stdio discovery funnel
  (mcp_stdio.rs:796); McpRugPullBlockedPayload emitted by OrchestratorRugPullSsbEmitter::emit + consumed by the
  Tauri command (Task 4). No produced symbol caller-less.
- W-9: ONE McpRugPullBlocked variant through the golden lock (distinct from mcp_schema_pin_violation); bump baseline+1.
- W-10: entirely Tier-1 in-process (hash recompute + comparison on the MCP connection ELAI already holds) — no
  new network, no subprocess, no RunPod at gate time; genuinely fail-closed (a mismatch blocks).
- GD-1: COMPOSES-WITH the shipped mcp_schema_pin.rs (re-uses hash_tool_list/PinRecord) + E-24/E-43; re-implements
  none. GD-4: the FP risk is a benign legitimate tool-update mis-read as a rug-pull — mitigated by operator
  re-approval (not a hard reject) + measuring the legitimate-update rate before default-on.

## Tests to add
- crates/contracts/tests/g4_mcp_rugpull_contract.rs (~150 LOC): (a) an unchanged tool list -> Unchanged, no row;
  (b) a swapped tool description (name unchanged) -> Changed{ description_diff_count>=1 } + mcp_rug_pull_blocked +
  the tool blocked; (c) update_pin is NOT auto-called on Changed (requires explicit re-approval); (d) a benign
  legitimate update behind operator re-approval updates the pin cleanly; (e) the McpRugPullBlocked variant round-trips.
- crates/contracts/tests/fixtures/mcp_rugpull/ (a first-connect tool list + a post-approval swapped-description list).

## Files touched
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g4.rs (~90 LOC: get_mcp_rug_pulls)
- CREATE: crates/dashboard-v2/src/lib/drawers/McpRugPullDrawer.svelte (~160 LOC)
- CREATE: crates/contracts/tests/g4_mcp_rugpull_contract.rs (~150 LOC)
- CREATE: crates/contracts/tests/fixtures/mcp_rugpull/*.json (2 fixtures)
- CREATE: workflow/reasoning/G-4-decisions.md (~80 LOC)
- MODIFY: crates/plugins/src/mcp_schema_pin.rs (~90 LOC: reverify_on_list_changed + RugPullVerdict + ToolDescDiff)
- MODIFY: crates/plugins/src/mcp/* (~50 LOC: call reverify_on_list_changed in the tools/list_changed handler)
- MODIFY: crates/orchestrator/src/ssb.rs (~60 LOC: McpRugPullBlocked variant + payload + sql_str + parse + CHECK + golden)
- MODIFY: crates/orchestrator/tests/action_type_lock.golden (~1 row)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + DrawerHost.svelte (~6 LOC: DrawerKey 'mcp-rug-pull')

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R4 (the gap + grounding; verifier-CONFIRMED).
  OWASP MCP03:2025 (runtime schema change w/o operator confirm) CONFIRMED; Trail of Bits line-jumping "alert on
  description changes" CONFIRMED; Invariant Labs WhatsApp sleeper rug-pull CONFIRMED.
- LANDMINES (G-0, binding): mcp-scan is Invariant Labs NOT Snyk + dist-channel UNVERIFIED (no subprocess without
  anti-slopsquatting verification); do NOT cite "CVE-2026-30623 in Anthropic's MCP SDK" (that CVE is LiteLLM).
- ELAI ground truth (grep-confirmed 2026-06-02): mcp_schema_pin.rs hash_tool_list:69, pin_or_verify:134 (TOFU),
  update_pin:177, PinRecord:49, ToolDef:57, PinStatus:21, SchemaPinError:36, emits mcp_schema_pin_violation (:7).
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, Pattern-2 SSB, anti-slopsquatting, SCHEMA baseline+N.

## Open questions / risks
- FP risk (GD-4): a LEGITIMATE tool-description update (a server's real new version) is mis-read as a rug-pull.
  MITIGATION: the verdict is operator-re-approval (not a hard reject) — the human sees the diff and approves a
  real update; measure the legitimate-update rate before default-on so re-approval prompts are not noise.
- The tools/list_changed handler must be the actual production MCP notification path — confirm at authoring
  (W-7); a re-pin wired only into a test harness is wire-dark.
- Scope: G-4 covers the SCHEMA/description swap. The stdio command-injection family (Anthropic declined a protocol
  fix) is a DIFFERENT MCP risk class — out of G-4 scope; note it in the decision doc, do not conflate.

## Next
G-4 is the runtime third bracket of MCP trust (E-43 install scan -> first-connect pin -> G-4 runtime re-pin).
The mcp_rug_pull_blocked rows feed G-2's RRA (a rug-pull block is part of the reproducible run record) and the
G-5 cockpit (a reviewer sees that a tool was swapped + blocked). G-1's taint propagation treats a post-rug-pull
tool output as Untrusted, composing the data-plane defense with the schema-integrity gate.

End of G-4.
