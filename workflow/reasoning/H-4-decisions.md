H-4 DECISION DOC — Tool Search Tool + advanced-tool-use beta header
session s-66424 | 2026-06-04 | orchestrate H-plan WAVE 1 | composes-with B-37/E-18 tool catalog, E-40 3-arm

WHAT
ELAI sends NO advanced-tool-use header and the OAuth-bearer production path (anthropic.rs:695-701) emits ONLY
oauth-2025-04-20 + IGNORES profile betas (C14) — so any beta added the normal way is DEAD on the live OAuth path.
The headline (move the 16-tool catalog behind the Tool Search Tool for an ~85% tool-def token cut) is heavy
net-new; H-4 ships the MANDATORY enabler + documents the heavy follow-ons.

GROUNDING (spec-grounder, dual-confirmed via live Anthropic docs 2026-06-04 — all 6 claims GROUNDED, EXECUTING cleared)
- Umbrella beta = `advanced-tool-use-2025-11-20` (verbatim, anthropic.com/engineering/advanced-tool-use).
- Tool Search Tool type = `tool_search_tool_regex_20251119` (regex variant; BM25 sibling exists). Search tool
  must NEVER be deferred. A custom ANTHROPIC_BASE_URL DISABLES server-side deferral (client self-defers).
- The ~77K->8.7K (85%) reduction is a [WEAK]/borrowed figure (Anthropic's own sources disagree: 72K/55k/77K) —
  NEVER asserted as ELAI's; H-9 measures the real per-request delta (HD-1).

KEY DECISIONS
D1 (T1 — the deliverable, MANDATORY): join `advanced-tool-use-2025-11-20` onto the OAuth header. New
   `pub const ADVANCED_TOOL_USE_BETA` in telemetry_types.rs (one source of truth) + added to the API-key default
   betas vec; anthropic.rs:698 OAuth branch now sends `format!("{OAUTH_BETA_HEADER},{ADVANCED_TOOL_USE_BETA}")`
   (the OAuth branch does NOT loop profile betas — C14, so this join is mandatory or the beta is dead). Verifiable
   in elai-providers (NOT G-blocked). This lights the prerequisite for ALL advanced-tool-use (incl. tool-search).
D2 (DEFERRED — heavy net-new, documented Tier-2 follow-ons):
   - T2 tool-search catalog DEFERRAL (the ~85% reduction headline): ToolDefinition (types.rs:202) has NO
     defer_loading field and is a custom-function shape (no server-tool type) — so T2 needs (a) a net-new
     `defer_loading: Option<bool>` on ToolDefinition touching all 16+ catalog construction literals, (b) a
     server-tool representation for tool_search_tool_regex_20251119 (a new ToolDefinition variant / type field),
     (c) a base-URL gateway-detection predicate (none exists), (d) ELAI's own deferral when base-URL is
     non-Anthropic. Heavy + behaviorally unverifiable under cargo-check-only — deferred.
   - T3 dkg_* 3-arm fix: the catalog input_schema (project_root/target/max_depth) MISMATCHES the existing
     ipc-types structs (DkgCallersRequest{symbol}, DkgDataFlowRequest{variable,direction},
     DkgSubgraphRequest{entry_point,depth}). Reconciling (reshape schema OR structs + DkgQuerier call sites) is a
     real conflict, risky blind. Deferred.
   - T4 catalog contract len: production_agent_rpc_tool_catalog_contract.rs:280 asserts len==14 but the live
     catalog is 16 (E-40 added symbol_dsl_query + symbol_dataflow_impact without updating it) — a PRE-EXISTING
     failure (not H-4's regression), in the currently-G-blocked elai_cc_contracts crate (unverifiable now). The
     fix is: bump 14->16 + add TOOL_SYMBOL_DSL_QUERY + TOOL_SYMBOL_DATAFLOW_IMPACT to the expected set. Deferred to
     the Tier-2 pass (when the contracts crate compiles again, post-G's Path fix).
D3 (no schema bump): H-4 adds no action_type. SCHEMA untouched (live 211/488; do NOT edit — G owns the churn).

DONE-GATE (Tier-1)
crates/contracts/tests/h4_tool_search_beta_contract.rs (source-wiring): anthropic.rs OAuth path joins
advanced-tool-use-2025-11-20; telemetry_types has the const + default. cargo check -p elai-providers clean
(NOT G-blocked). The contract crate itself is G-blocked (production_agent_rpc.rs:249 Path) — the h4 contract
SOURCE is correct + will verify once G unblocks. Test exec + live wire-capture + the T2 reduction = Tier-2.

RISK
R1 anthropic.rs:594 has a SECOND OAuth header site (a separate request builder, context-compiler LOW). H-4 joins
   the MAIN inference path (:698, the wiring-contract reachable_from). The :594 site is flagged for a verify
   follow-on (if it is a live inference path it needs the same join).
R2 the format! String value on .header() — reqwest accepts String as HeaderValue; OAUTH_BETA_HEADER is a &str const.

DEFERRED: T2 (tool-search deferral / the reduction), T3 (dkg 3-arm), T4 (catalog contract len), the :594 OAuth
site verify, live wire-capture. NO commit (accumulate to run end).
