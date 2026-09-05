# H-4: Tool Search Tool + advanced-tool-use beta header — load tool defs on demand on the live OAuth path

STATUS: PLANNED (WAVE 1; in-process; BUILD)

Objective: ELAI sends NO advanced-tool-use header and loads its full 16-tool catalog into every request. Anthropic's
Tool Search Tool loads tool DEFINITIONS on demand instead of in the system prefix (~77K -> 8.7K, 85% reduction —
CORRECTED from the doc's ~72K). The blocker (C14): the OAuth-bearer production path (anthropic.rs:695-701) emits ONLY
oauth-2025-04-20 and IGNORES profile betas (with_beta + profile defaults reach the wire only on the API-key else-branch
:703-705), so a beta added the normal way is DEAD on the live OAuth path. H-4 joins advanced-tool-use-2025-11-20 onto
the OAuth beta header (MANDATORY) and moves the catalog behind the Tool Search Tool, gated on catalog size; it also
fixes the 4 C-30 dkg_* tools (catalog-only, no convert/dispatch arm -> model invocation hits unknown=>Err) via the
E-40 3-arm pattern, and corrects the stale len==14 catalog contract (live is 16).

Estimated effort: ~1 session (~5-7 h). Pure in-process; no network at gate time (W-10 Tier-1 whole — the catalog
size + the header presence are statically asserted; the live token reduction is measured by H-9).
Pre-requisites: B-37/E-18 tool catalog SHIPPED (build_tool_catalog production_agent_rpc.rs:266, 16 tools); E-40
  3-arm extension precedent SHIPPED (symbol_dsl_query / symbol_dataflow_impact).
Parallel-safe with: H-1 (coordinate on anthropic.rs beta-header edits), H-5/H-8 (disjoint).
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.4 (LEVER L4); _grounding/H-GROUNDING-MAP.md C14 + C15 + verify:tool-search. Doctrine: H-0_INDEX.md.

## Context

The advanced-tool-use-2025-11-20 beta adds a Tool Search Tool (tool_search_tool_regex_20251119 = re.search, max
200-char patterns; tool_search_tool_bm25_20251119 = NL BM25) that loads tool definitions on demand; deferred tools
carry defer_loading:true and the search tool itself must NEVER be deferred. Anthropic measured 85% tool-def token
reduction (~77K -> 8.7K) and RAISED accuracy on large catalogs (Opus 4.5 79.5%->88.1%). CRITICAL for ELAI (C14 +
verify:tool-search): a custom ANTHROPIC_BASE_URL / gateway DISABLES Anthropic server-side deferral (server-side
tool_search_tool_result + tool_reference expansion is not forwarded), so ELAI CANNOT rely on Anthropic's deferral
when pointed at localhost:8100 — it must implement its own (which it already does via ToolSearch-emulation in
system-reminders). H-4's win is two-fold: (a) the catalog moves behind the search tool (token reduction + a more
stable cache prefix, compounding H-1/H-2 since fewer tool-def bytes change), and (b) the beta is actually emitted on
the live OAuth path. Tool-search is pure overhead for a tiny catalog — gate it on catalog size.

## Existing state (B/C/D/E/F/G coverage)
- build_tool_catalog (production_agent_rpc.rs:266) — SHIPPED, 16 ToolDefinition (vec :754-762); called via handle_infer_request (:835).
- convert_json_input_to_postcard (production_agent_rpc.rs:970) — PARTIAL. Arms for 12 of 16 tools; the 4 C-30 dkg_* tools
  hit unknown=>Err (:1078).
- translate_tool_request (production_agent_rpc.rs:1163) — PARTIAL. Dispatch arms for 12 tools + git stubs + 6 artifact tools
  (not in catalog = agent-invisible); NO arm for the 4 dkg_* tools.
- DKG tools (dkg_callers/dependents/data_flow/subgraph) — WIRE-DARK. Catalog-only (:562-683); no convert/dispatch arm.
- AnthropicRequestProfile::with_beta / default betas (telemetry_types.rs:58/69-72/78/93-104) — PARTIAL. Defaults
  [claude-code-20250219, prompt-caching-scope-2026-01-05]; header_pairs joins them; emitted ONLY on the API-key else-branch.
- AnthropicClient::send_message beta emission (anthropic.rs:684-721) — SHIPPED. OAuth branch (:695-701) sets ONLY
  oauth-2025-04-20 (OAUTH_BETA_HEADER :41); no advanced-tool-use anywhere. with_beta (:384) affects only the API-key path.
- production_agent_rpc_tool_catalog_contract.rs:280 — asserts len==14 (STALE vs 16; would FAIL — fix as part of H-4).

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C14 + C15 + the H-4 symbol table + the WEB VERIFICATION verify:tool-search rows
  (the CORRECTED 77K figure; the second tool-search-tool-2025-10-19 header; the ANTHROPIC_BASE_URL-disables-deferral row).
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.4 (L4) + §12 (Claude Code already does deferred tool loading; ELAI must implement its own).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\production_agent_rpc.rs:260-770,960-1090,1160-1200 (build_tool_catalog + convert + translate + the dkg_* gaps)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\providers\anthropic.rs:380-420,680-721 (with_beta + the OAuth beta emit)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\telemetry_types.rs:55-110 (the profile betas + header_pairs)
- The E-40 precedent: how symbol_dsl_query was added across the 3 arms (grep symbol_dsl_query in production_agent_rpc.rs).
- CLAUDE.md HARD RULES + anti-slopsquatting (the beta identifiers are verified strings, not invented).

## Tasks (WAVE 1 — build)

1. **Join advanced-tool-use-2025-11-20 onto the OAuth beta header (~30 LOC) [MANDATORY].** Edit the OAuth-bearer branch
   (anthropic.rs:696-701) to join OAUTH_BETA_HEADER with the new beta, comma-separated, AND register it in the profile
   defaults (telemetry_types.rs:69-72) so the API-key path carries it too. Without the OAuth-branch edit the beta is
   dead on the live path (C14).

2. **Wire the Tool Search Tool over the catalog, gated on catalog size (~120 LOC).** Add the tool_search_tool_regex_20251119
   tool (never deferred) to the catalog and mark the bulk tool definitions defer_loading:true when the catalog exceeds a
   size threshold; below it, keep them inline (tool-search is pure overhead for a tiny set). Because a custom
   ANTHROPIC_BASE_URL disables server-side deferral, route through ELAI's own deferral implementation when the base URL
   is non-Anthropic (do NOT silently rely on server-side expansion).

3. **Fix the 4 C-30 dkg_* tools via the 3-arm pattern (~90 LOC).** Add the missing convert_json_input_to_postcard arm
   (before unknown=>) and translate_tool_request arm for each, with a TOOL_<X> const + an <X>Request struct in
   elai_ipc_types (field names matching the input_schema verbatim). Mirrors symbol_dsl_query (E-40).

4. **Fix the stale catalog contract (~5 LOC).** production_agent_rpc_tool_catalog_contract.rs:280 asserts len==14;
   bump to the live count (16 + the new search tool + 4 newly-dispatchable dkg tools as applicable).

5. **Decision doc workflow/reasoning/H-4-decisions.md (~90 LOC).** Records: the OAuth-branch beta blocker (C14); the
   77K (not 72K) figure; the ANTHROPIC_BASE_URL-disables-deferral finding (ELAI implements its own); the catalog-size
   gate; the dkg_* 3-arm fix; the contract drift.

## WIRING_CONTRACT
produces_symbols:
  - crates/providers/src/providers/anthropic.rs (the OAuth-branch advanced-tool-use beta join)
  - crates/orchestrator/src/production_agent_rpc.rs (the Tool Search Tool entry + defer_loading gating + the 4 dkg_* convert+dispatch arms)
  - crates/orchestrator/src/production_agent_rpc.rs (catalog-size gate for tool-search activation)
consumes_symbols:
  - crates/orchestrator/src/production_agent_rpc.rs::build_tool_catalog
  - crates/orchestrator/src/production_agent_rpc.rs::convert_json_input_to_postcard
  - crates/orchestrator/src/production_agent_rpc.rs::translate_tool_request
  - crates/providers/src/providers/anthropic.rs::AnthropicClient::with_beta (+ the OAuth beta site :696-701)
  - crates/providers/src/telemetry_types.rs::AnthropicRequestProfile (default betas + header_pairs)
reachable_from:
  - crates/orchestrator/src/production_agent_rpc.rs:835 (handle_infer_request -> build_tool_catalog) -> the request tools list  [PRODUCTION inference]
  - crates/providers/src/providers/anthropic.rs:695-701 (the OAuth header emit) -> the wire  [PRODUCTION OAuth path]
stub_ban:
  - NO advanced-tool-use beta registered WITHOUT the OAuth-branch edit (else it is dead on the live path — C14)
  - NO deferral of the search tool itself (it must always be present)
  - NO reliance on Anthropic server-side deferral when ANTHROPIC_BASE_URL is non-Anthropic (implement ELAI's own)
  - NO tool added with fewer than 3 coordinated arms (catalog + convert + dispatch) — an agent-invisible or convert-failing tool
  - blake3; no .unwrap()/.expect() in prod; anti-slopsquatting (beta identifiers are the verified strings, not invented)
frontend_pair:
  N/A own pair — the tool-def token reduction is surfaced by H-9's cockpit (cross-ref). Justified.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a real inference task on the OAuth path with the full catalog,
    NO env override; and a task that invokes one of the 4 dkg_* tools>"
  expect_ssb: "the per-request tool-definition token count (journal/H-9-recorded) drops materially vs the pre-H-4
    full-catalog baseline when the catalog exceeds the size gate"
  expect_artifact: "the outgoing OAuth-path request's anthropic-beta header CONTAINS advanced-tool-use-2025-11-20
    (the off-agent wire artifact); a dkg_* tool call dispatches successfully (no unknown=>Err); the catalog-size gate
    keeps a tiny tool set inline"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-4_tool_search_beta_header.md --run-trigger
rg "advanced-tool-use-2025-11-20" .elai_cc/crates/providers/src/providers/anthropic.rs .elai_cc/crates/providers/src/telemetry_types.rs  # beta present on the OAuth branch
rg "tool_search_tool_regex_20251119|defer_loading" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs  # tool-search wired
rg "TOOL_DKG_CALLERS|dkg_callers" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs   # the dkg_* convert/dispatch arms exist
rg "len\(\), 1[46]|== 1[46]" .elai_cc/crates/contracts/tests/production_agent_rpc_tool_catalog_contract.rs  # catalog-count contract updated
test -f workflow/reasoning/H-4-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (the beta is on the OAuth wire; tool defs load on demand; a dkg_* tool dispatches).
- [ ] W-2: tool-search active by default above the catalog-size gate; no default-off ENABLE as a terminal state.
- [ ] W-3: the beta + catalog fire from the live OAuth inference path; not test-only.
- [ ] GD-1: the E-40 3-arm precedent is followed; no second tool-dispatch mechanism invented.
- [ ] The stale len==14 catalog contract is fixed (would otherwise FAIL).
- [ ] HD-1: the tool-def token reduction is MEASURED (via H-9), not the 85% literature figure quoted as ELAI's.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-providers -p contracts.
- [ ] No new dependency (the beta identifiers are protocol strings, not packages — anti-slopsquatting N/A but the strings are the verified ones).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; the beta join (produced) is on the live OAuth path (W-3); the dkg_* arms (produced) dispatch (W-8).
- W-2: tool-search default-on above the gate; the search tool never deferred.
- W-3: anthropic.rs:695-701 (OAuth) + production_agent_rpc.rs:835 (catalog); not test.
- W-4: N/A own pair (surfaced by H-9); justified.
- W-5: a runnable OAuth inference + a dkg_* call -> a real wire header + a real dispatch; in-process.
- W-6: the tool-token tile is H-9's; H-4 N/A justified.
- W-7: build_tool_catalog / convert / translate / the OAuth beta site / the dkg_* gaps grep-confirmed 2026-06-03 (C14/C15).
- W-8: the search tool + dkg_* arms (produced) are consumed by the model dispatch + the catalog; the beta join is consumed by the OAuth emit.
- W-9: NO new SSB action_type.
- W-10: Tier-1 in-process (header presence + catalog size are static; the live token cut is measured by H-9, Tier-2-adjacent but not gate-blocking).

## Tests to add
- crates/contracts/tests/h4_tool_search_beta_contract.rs (~150 LOC): (a) the OAuth-path request header contains
  advanced-tool-use-2025-11-20 (was absent); (b) above the catalog-size gate the bulk tools carry defer_loading and the
  search tool does not; (c) below the gate the catalog is inline (no tool-search overhead); (d) each of the 4 dkg_* tools
  has a convert arm AND a dispatch arm (round-trips, no unknown=>Err); (e) the catalog-count contract matches the live count.

## Files touched
- MODIFY: crates/providers/src/providers/anthropic.rs (~30 LOC: OAuth-branch beta join)
- MODIFY: crates/providers/src/telemetry_types.rs (~10 LOC: register the beta in profile defaults)
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs (~200 LOC: tool-search tool + defer_loading gate + 4 dkg_* convert+dispatch arms)
- MODIFY: (elai_ipc_types) TOOL_<X> consts + <X>Request structs for the 4 dkg_* tools (~60 LOC)
- MODIFY: crates/contracts/tests/production_agent_rpc_tool_catalog_contract.rs (~5 LOC: catalog count)
- CREATE: crates/contracts/tests/h4_tool_search_beta_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-4-decisions.md (~90 LOC)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.4 (L4): Tool Search 85% (~77K->8.7K — CORRECTED), Programmatic
  Tool Calling 37%, Opus 4.5 79.5->88.1%; custom-base-URL disables server-side deferral (ELAI implements its own).
- verify:tool-search (live docs 2026-06-03): umbrella beta advanced-tool-use-2025-11-20 (+ the tool-search-scoped
  tool-search-tool-2025-10-19); tool_search_tool_regex_20251119 / _bm25_20251119; the search tool is never deferred;
  max 10,000 tools, returns 3-5 per search, regex max 200 chars; works in Batches at standard pricing.
- ELAI ground truth (C14/C15): OAuth beta site anthropic.rs:695-701 (only oauth-2025-04-20); build_tool_catalog 16 tools;
  the 4 dkg_* tools catalog-only; the len==14 contract drift.
- CLAUDE.md HARD RULES; the E-40 3-arm extension precedent.

## Open questions / risks
- The catalog-size gate threshold is a tuning parameter; too low adds a retrieval hop for a small set (pure overhead),
  too high keeps a bloated prefix. Start conservative; measure the per-request tool-def token count via H-9.
- ELAI's own deferral (ToolSearch-emulation in system-reminders) must produce results compatible with the model's
  tool_reference expectation when ANTHROPIC_BASE_URL is non-Anthropic; verify the sidecar forwards/expands correctly
  (a gateway that drops tool_search_tool_result returns a 400 'Tool reference not found').
- Adding the beta could interact with prompt-caching invalidation (tools->system->messages): a tool-list change busts
  the system+message cache. Coordinate with H-2 so tool-search STABILIZES the tool-def bytes (fewer changes), not the reverse.

## Next
H-9 (cost cockpit) reports the per-request tool-def token count so the reduction is measured. H-2 (cache-stable layout)
benefits: fewer tool-def bytes that change => a more stable cache prefix. H-13 (canvas ops) adds canvas_* tools via the
SAME 3-arm pattern H-4 exercises on the dkg_* tools.

End of H-4.
