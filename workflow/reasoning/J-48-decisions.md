J-48 decisions

WHAT
Local multi-model serving tier router (LocalServingRouter) integrated into
RuntimeOrchestrator::try_complete as a pre-step before the fallback chain.
The router decides per dispatch: serve locally, queue locally, hibernate a
small tier then serve the big tier, spill to cloud, or wait.

DECISION
- Reuse the existing RouteDecisionEvent SSB variant (E-31, SCHEMA_VERSION 191)
  with its 3-arm RouteDecisionEventKind discriminator:
    CloudRouteChosen  = SpillToCloud decision
    TierModelSwapped  = HibernateThenServe decision
    LocalRouteKept     = ServeLocal / AdmitQueued / Wait decision
  No new SSB action type, no SCHEMA_VERSION bump. Net 0 SSB schema slots.
- Emission gated by the existing ELAI_ROUTE_DECISION_SSB_ENABLE env flag
  (E-31 gate, already allowlisted per R9). No new ELAI_*_ENABLE flag.
- Cold-start path: empty residency or cold+idle tier returns ServeLocal
  (admit to warm locally), never spills to cloud solely for being cold.
- EXCLUSIVE spill: High tier warm + small tier requested + cloud_available
  returns SpillToCloud with non-empty reason. Without cloud, returns Wait.
- Residency writers: try_complete bumps warm+inflight on ServeLocal and
  HibernateThenServe; decrements inflight via release_tier_inflight after
  the chain walk. High tier warm forces small tiers cold (EXCLUSIVE inv).
- The decide() call reads VramTier::from_env() (ELAI_VRAM_TIER) for the
  requested tier; cloud_available comes from CloudConfig::default().air_gapped
  (false by default).

EXPECTED-TEST
- j48_local_serving_contract.rs: decide() unit pins for all 6 branches
  (ServeLocal, AdmitQueued, HibernateThenServe, SpillToCloud non-empty,
  never spill air-gapped, corrupt inflight Wait).
- W-1 off-agent artifact: a tokio test driving RuntimeOrchestrator::try_complete
  with ELAI_ROUTE_DECISION_SSB_ENABLE=1 + ELAI_VRAM_TIER=high (warm the high
  tier), then ELAI_VRAM_TIER=low (small tier requested while high is warm),
  MUST emit a RouteDecisionEvent SSB row with kind=CloudRouteChosen and
  non-empty reason. Deleting the try_complete RouteDecisionEvent write_payload
  block turns this test RED.

UNKNOWNS
- None remaining for this slice. The spill path, cold-start, EXCLUSIVE inv,
  and SSB emission are all landed in the worktree.
