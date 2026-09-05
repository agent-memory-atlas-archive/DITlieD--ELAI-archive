Phase: Tier 3 complete — Research + Anti-Staleness drawers
Session: s-456340
Date: 2026-04-24

What I will implement:
  Research drawer with 4 new tauri commands (get_research_tree reads real
  SSB WebResearchStep/ResearchStep rows; get_research_budget and
  get_source_tier_distribution are graceful-empty stubs per D3-3 precedent;
  get_searxng_status probes the existing sidecar /search endpoint with a 2s
  timeout). ResearchWindow.svelte hosts ResearchDrawer; WindowRegistry LIVE
  gains RESEARCH (8 -> 9); layoutState RESEARCHING routing flips primary
  ACTIVITY -> RESEARCH. Anti-Staleness drawer ships as a pure graceful-empty
  stub (zero new commands; ghost list I3-4 forbids the 3 originally-planned
  commands) mounted inside SettingsWindow alongside PI/ProviderVram.

Key technical decision:
  Zero new SsbActionType variants (stays at 73). Zero new sidecar
  endpoints. Zero new deps. Research-budget and source-tier distribution
  stay stubs until a writer-side row type lands; Anti-Staleness stays
  permanent graceful-empty per DEF-T3-6 candidate-SSB-row-types.

Expected test outcome:
  tier3-drawers-backend.mjs 11 -> 15 checks green (4 new assertions).
  tier2-windows-backend.mjs 6/6 green after FROZEN 85 -> 89.
  cargo check -p elai-dashboard clean.
  dashboard_tier3_drawers_contract.rs unchanged (symbol-grep commands.rs
  + lib.rs for the 4 new names should pass automatically).

Unknowns to verify empirically:
  - WebResearchStep payload field names (agent_id/node_id/action/phi/psi/
    iteration/sources_fetched/sources_used) — confirmed from ssb.rs:762.
  - reqwest timeout API shape for the 2s probe — use Client builder
    timeout(Duration::from_secs(2)).
  - svelte-check clean for both new drawers — verify via audit.
