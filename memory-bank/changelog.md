Format: changelog
Max 5 lines per entry. [date] [filepath] [title] [what] [why] [change] [impact] [lesson]

2026-07-10 F:\.ELAI_workflow\antenna\channel_fit.py ANTENNA channel-fit routing module
- Added deterministic keyword guard module with fit_for() and filter_bundle() helpers
- Route incoming bundles to the right channel model only when keywords match; reduce cross-channel replies
- New file; integrates with feed_chat.process_bundle via channel-model keyword matching
- Phase 1 routing gate; no-op when no channel match

2026-07-10 F:\.ELAI_workflow\antenna\feed_chat.py ANTENNA follow-up recency gating
- Wired channel_fit guard into process_bundle and added FOLLOWUP_MAX_PRIMARY_AGE_HOURS (default 6h)
- Prevent stale primary messages from triggering follow-up replies
- process_bundle now checks channel_fit fit_for/filter_bundle before fan-out; follow-ups gated by primary message age
- Follow-ups suppressed after 6h; mention/reply-path still available

2026-07-10 F:\.ELAI_workflow\antenna\discord_gateway.py ANTENNA reply recency + reaction auth gating
- Added REPLY_MAX_MESSAGE_AGE_MINUTES (default 30m) for non-mention replies; gated on_raw_reaction_add to bot-authored messages only
- Avoid replying to ancient messages; stop recording reactions on non-bot messages
- @mentions and role-pings still always answer; non-mention replies require message age under 30m; reactions only recorded on bot-authored messages
- Cleaner reply boundaries; reaction score noise reduced

2026-07-10 F:\.ELAI_workflow\antenna\reactions_store.py ANTENNA honey reaction scoring
- Added 🍯 to POSITIVE_REACTIONS with weight 2.0 and get_message_score()
- Use the honey emoji as a strong positive feedback signal
- New reaction entry and score helper; no schema migration required
- Honey reactions now contribute 2x positive score

2026-07-10 F:\.ELAI_workflow\antenna\tests\test_channel_fit.py ANTENNA channel_fit tests
- Added unit tests for fit_for and filter_bundle deterministic keyword guard
- Lock phase 1 routing behavior under test
- New test file; covers match/no-match and bundle filtering paths
- Changed-module tests pass 94/94

2026-07-10 F:\.ELAI_workflow\antenna\tests\test_feed_chat.py ANTENNA feed_chat recency tests
- Added tests for follow-up recency gating (FOLLOWUP_MAX_PRIMARY_AGE_HOURS)
- Verify stale follow-up suppression
- Tests for recent/stale primary ages; channel_fit wiring assertions
- Changed-module tests pass 94/94

2026-07-10 F:\.ELAI_workflow\antenna\tests\test_discord_gateway.py ANTENNA discord_gateway recency/auth tests
- Added tests for non-mention reply recency and bot-author reaction gating
- Verify reply age gate and reaction auth gate
- Tests cover mention bypass, age-block, and reaction on bot/non-bot messages
- Changed-module tests pass 94/94

2026-07-10 F:\.ELAI_workflow\antenna\tests\test_reactions_store.py ANTENNA reactions_store honey tests
- Added tests for 🍯 reaction scoring and get_message_score
- Lock honey feedback scoring under test
- Tests cover positive reaction weights and honey 2.0 multiplier
- Changed-module tests pass 94/94

2026-07-03 F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-15_phase0_debt_burndown.md B-15 phase0 debt burndown plan rebaselined
- Rebaselined stale plan numbers (main.rs 555->1632 LOC tripwire FIRED, HashMap 5->21, entropy 82 files->1 leak, compact/oauth/prompt LOC) + fixed WIRING_CONTRACT (produces sidecar.rs->remote_sidecar.rs, trigger cargo test, expect_ssb phase_transition)
- Plan-stated numbers had drifted from tree reality across sessions; WIRING_CONTRACT block failed wiring_contract_check.py
- Plan markdown only; no .rs/.py/.sh modified; relabeled stale Files-touched section as prior batch G reference
- wiring_contract_check.py verdict PASS (6/6 produces, 6/6 reachable, matches=30); S6 audit PASSED 10/10 + 1/1 trigger; commit 5de5362 pushed
- PLAN-NUMBER-DRIFT: rebaseline plan LOC/violation counts with terminal commands before acting on plan-stated numbers

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\perception\scaffold.rs J-1.T2 scaffold gate wired
- Added prompt_carries_image() helper and ScaffoldInput::from_prompt() constructor; wired gate into ProductionAgentRpcServer::handle_infer_request
- Perception scaffold must detect image-bearing prompts and prepare scaffold input
- Removed WIRE-DARK[J-1] marker from scaffold fn; logged-only gate at T2 (no prompt/MessageRequest mutation); operates on post-assembly volatile tail
- 5 WIRE-DARK rows remain for T3/T5 wiring

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\perception\mod.rs J-1.T2 perception module exports
- Exported prompt_carries_image and ScaffoldInput for production caller
- Production seam needs public helpers
- Added pub re-exports/definitions so handle_infer_request can construct scaffold input
- Enables T2 wiring in production_agent_rpc.rs

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\production_agent_rpc.rs J-1.T2 production seam wiring
- Called perception scaffold gate from handle_infer_request after effective_max_tokens, before MessageRequest construction
- Production hook for perception gateway
- Integrated scaffold input construction and log-only gate invocation
- HD-2 safe: no slot content mutated

2026-06-30 F:\.ELAI_workflow\.elai_cc\wire-dark-allowlist.toml J-1.T2 allowlist update
- Removed 1 WIRE-DARK row for scaffold fn
- Scaffold is now wired production caller
- production_caller_check: 0 dark, 4 wired, 11 warn
- 5 WIRE-DARK rows remain

2026-06-30 F:\.ELAI_workflow\.claude\skills\see\SKILL.md J-1.T6 /see skill manifest
- Created skill definition for image preprocessing
- Harness skill for perception gateway preprocessing
- Validated by agentskills_validator.py; trigger-rich description; no angle brackets; sweet-spot edges labeled EXTERNAL SRC-3
- Mirrors Rust PreprocessSpec sweet-spot table

2026-06-30 F:\.ELAI_workflow\.claude\skills\see\scripts\preprocess.py J-1.T6 /see preprocess script
- Created ~160 LOC stdlib+pillow image preprocessor
- Preprocess images per model sweet-spots
- Pillow absent -> DegradedPassthrough (copy unchanged + stderr warning, exit 0, JC-4); sweet-spot table labeled EXTERNAL SRC-3; orjson fallback NameError fixed
- Evaluates to pass on skill eval harness

2026-06-30 F:\.ELAI_workflow\.claude\skills\see\eval\positive.jsonl J-1.T6 /see positive eval
- Created 6 positive eval cases
- Skill eval contract requires >=4 positives (JC-5)
- Exercises preprocess success paths and normal inputs
- agentskills_validator.py pass

2026-06-30 F:\.ELAI_workflow\.claude\skills\see\eval\negative.jsonl J-1.T6 /see negative eval
- Created 6 negative eval cases naming owning skills
- Skill eval contract requires >=4 negatives (JC-5)
- Negatives name spec-eval, incremental-eval, orchestrate, paper-to-plan
- Evaluates failure modes and boundaries

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb\action_type.rs J-1.T4 SSB action type variant
- Added PerceptionScaffoldEvent(PerceptionScaffoldEventKind) SSB action type variant
- Pattern-2 single-variant needs lockstep action_type mapping
- Updated enum, as_sql_str, from_str, and lockstep sites
- 578 SSB action types; lockstep sites consistent

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb\payloads.rs J-1.T4 SSB payload shape
- Added PerceptionScaffoldEventKind enum with 4 arms + Payload struct
- SSB payload needs kind and payload shape
- Added SsbPayload::PerceptionScaffoldEvent arm and action_type match
- Enables scaffold/router event emission

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs J-1.T4 SSB registry entry
- Registered PerceptionScaffoldEvent in SSB_ACTION_TYPES
- SSB registry must know all action types
- Bumped count 577->578; updated SCHEMA 269
- Action-type lock + schema version advance

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb\schema.rs J-1.T4 SQL CHECK update
- Added SQL CHECK constraint for PerceptionScaffoldEvent
- Database schema must validate new action type
- SQL CHECK includes new variant string
- Schema 269 enforces action type domain

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.rs J-1.T4 lockstep test guard
- Updated exhaustive_match_guard test for 578 variants
- Lockstep invariant must cover new action type
- Added match arm/guard; 2 pass/0 fail/1 ignore
- action_type_lock stays green

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.golden J-1.T4 regenerated golden
- Regenerated action_type_lock.golden for 578 variants
- Golden file must match lockstep enum
- Added PerceptionScaffoldEvent line
- Lock test compares against updated canonical list

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\production_agent_rpc.rs J-1.T4 production seam SSB emission
- Wired SSB emission at scaffold and router gates
- Production seam must emit PerceptionScaffoldEvent events
- Added emit_perception_scaffold_event helper (ScaffoldEmitted/RoutedToSpecialist/DegradedPassthrough)
- Non-fatal SSB-guarded emission; ts_ms via Ssb::now_ms_via_clock

2026-06-30 F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\j1_perception_gateway_contract.rs J-1.T4 contract test re-enabled
- Un-ignored contract test case (d) scaffold_event_lands_with_correct_kind
- Contract must assert SSB variant mapping and payload
- 6 assertions covering Kind as_str, payload construction, action_type mapping, as_sql_str, from_str round-trip, DegradedPassthrough+RoutedToSpecialist mapping
- contract 23 pass/0 fail/2 ignore
