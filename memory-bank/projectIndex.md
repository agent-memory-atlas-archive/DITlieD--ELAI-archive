Format: projectIndex
Subsystem paths one per line. Path | purpose | size category (small/medium/large)

.elai_cc/crates/orchestrator/src/perception/ | perception gateway modules | medium
.elai_cc/crates/orchestrator/src/production_agent_rpc.rs | production inference RPC seam | large
.claude/skills/see/SKILL.md | /see skill manifest for image preprocessing | small
.claude/skills/see/scripts/preprocess.py | image preprocessing harness script | small
.claude/skills/see/eval/positive.jsonl | positive eval cases for /see skill | small
.claude/skills/see/eval/negative.jsonl | negative eval cases for /see skill | small
.elai_cc/crates/contracts/tests/fsm_phase_transition_contract.rs | B-2 FSM PhaseTransition contract tests (3 assertions: action_type lock, idle_to_planning, consumer-path latest_fsm_state guard) | small
.elai_cc/crates/contracts/tests/fsm_phase_transition_smoke_ssb.rs | B-2 SSB smoke test writing real phase_transition row to project-root sessions/ssb.db | small
.elai_cc/crates/contracts/tests/context_budget_smoke_ssb.rs | B-10 SSB smoke test writing real context_budget_computed row to project-root sessions/ssb.db | small
.elai_cc/crates/providers/src/vram_tier.rs | VRAM tier detection + detect_vram_tier_or_default helper (B-10) | small
antenna/channel_fit.py | deterministic keyword guard for channel routing (fit_for/filter_bundle) | small
antenna/feed_chat.py | chat bundle processing and follow-up recency gating | medium
antenna/discord_gateway.py | Discord gateway with reply recency and reaction auth gating | large
antenna/reactions_store.py | reaction scoring store including honey feedback | small
antenna/tests/test_channel_fit.py | channel_fit keyword guard unit tests | small
antenna/tests/test_feed_chat.py | feed_chat follow-up recency tests | medium
antenna/tests/test_discord_gateway.py | discord_gateway reply recency and reaction auth tests | large
antenna/tests/test_reactions_store.py | reactions_store honey scoring tests | small
