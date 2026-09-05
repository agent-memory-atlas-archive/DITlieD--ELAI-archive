Phase 4D Grounding Sweep

GROUNDED (verified via grep):
- SsbActionType enum has 25 variants, no ResearchStep (grep ssb.rs:75-101)
- Researcher GRC config is default-only (grep agent_pool.rs:567-569)
- Designer GRC config already 8/1200 (grep agent_pool.rs:508-523)
- proptest 1.11 in Cargo.toml (grep Cargo.toml:50)
- No research_tree module exists (grep src/ no matches)
- No apply_agent_config exists (grep src/ no matches)
- No ordered_float dep (grep Cargo.toml no matches)
- SSB writes via Ssb::write_payload() (ssb.rs:828)
- GRC types from elai_reasoning crate (agent_pool.rs:10)
- AgentRole from elai_ipc_types crate (agent_pool.rs:16)

VERIFY: None remaining.
INGEST: None needed.

All claims grounded. Clear to proceed to TESTING_FIRST.
