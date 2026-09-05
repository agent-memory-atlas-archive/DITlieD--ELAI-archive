F-6 learned-skill library - decision doc (session s-60824, continues s-6344)

What I will implement (decomposed C6a -> C6b -> C6c, each gated on the prior)
C6a (governance, ship first): a SkillTrustTier S1-S4 trust ladder + a learned-skill store + an autonomy barrier that blocks a sub-top-tier skill from autonomous dispatch without a human grant, plus the SkillLibraryEvent SSB beacon + a CLI library surface.
C6b (acquisition): critic-gated skill acquisition that REUSES the existing BestOfN MiniCritic verdict (no new critic), wired into the TemplateCrystallizer post-execute path, with reranker retrieval of learned skills.
C6c (maintenance): a staleness score + a demotion-on-tick cascade.

Key technical decision
SkillTrustTier is a NEW enum LOCAL to crates/orchestrator (S1-S4), kept namespace-distinct from memory::TrustTier (T0-T3) and ReflectionTrustTier (T0-T3); the autonomy gate stays decoupled by reading a pre-computed GateContext.candidate_skill_tier evidence field rather than depending on the skill store; the critic is REUSED (MiniCriticVerdict::Pass), never re-spawned.

Expected test outcome
cargo check --workspace --tests GREEN; a compile_fail doc-test proves no From/Into bridges SkillTrustTier and memory::TrustTier; live `elai skills-mgmt library promote <id> --human-grant` then `library tick` emits skill_library_event rows + `library list` shows the new tier; the autonomy barrier blocks a sub-S4 skill from auto-dispatch.

Unknowns to verify empirically
1. Whether the BestOfN MiniCriticVerdict is in scope at run_crystallizer_post_execute_hook (then pass it) or must be queried from SSB by blank_id/concept (then bound by recency, fail-closed if none).
2. Live all_test_payloads / TOTAL_VARIANTS counts (RE-COUNT; baseline 478->479, SCHEMA 201->202, all_test_payloads 396->399 since +3 Kind arms).
3. Whether extending SkillsPanel.svelte breaks the shipped E-43 telemetry render (fallback: sibling panel in the same drawer).

================================================================================
THREE HIGH BLOCKERS (from the integration-explorer) + resolutions
================================================================================
HIGH-1 ExecutionOutcome (compiled_pipeline.rs:280) lacks agent_id + superstep, and MiniCriticVerdictPayload keys on blank_id (ssb.rs:16064), so the C6b verdict cannot be joined by (agent_id, superstep). RESOLUTION: do NOT thread agent_id/superstep through ExecutionOutcome (that breaks task_runner.rs:3551-area + c25_compiled_pipeline_contract.rs:284,310). Instead acquire_learned_skill consumes the MiniCriticVerdict already in scope at run_crystallizer_post_execute_hook (task_runner.rs:3587, called :7886); if not in scope, query the most-recent MiniCriticVerdict::Pass SSB row by blank_id/concept with a recency bound; if no Pass verdict is found, do NOT acquire (fail-closed acquisition).
HIGH-2 GateContext (autonomous_gate.rs:881) has no Default derive; 8 struct-literal sites (autonomous_gate_contract.rs:57, c10:245, c9:50, phase_4g_extended:48, measure.rs:481+516, autonomous_gate.rs:1288, task_runner.rs:6902). RESOLUTION: add candidate_skill_tier: Option<SkillTrustTier> (None = no candidate = no barrier), add #[derive(Default)] to GateContext + use ..Default::default() at the 8 sites, update the impl Debug at :934. This is the F-10 Evidence-field lesson at 8x scale - run cargo check --workspace --tests (NOT lib-only) to catch every literal.
HIGH-3 autonomous_mode anchor drift (plan:6516 / staged:6573 / live task_runner.rs:6638). RESOLUTION: the barrier does NOT go inline at task_runner; it lives in autonomous_gate::decide() step 7.5 (after the existing step 7), keyed off GateContext.candidate_skill_tier. task_runner derives the tier before the sole decide() call (:6933) and sets the field at the GateContext literal (:6902).

================================================================================
SkillTrustTier + autonomy barrier (C6a)
================================================================================
- SkillTrustTier { S1, S2, S3, S4 } in skill_trust.rs (orchestrator-local; ReflectionTrustTier reflection_buffer.rs:53-55 is the local-enum precedent). P0 type barrier: no From/Into between SkillTrustTier and memory::TrustTier in either direction; a compile_fail doc-test enforces it (trybuild is not a dep, do not add one).
- LearnedSkill { id, description, tier, invocations, failures, acquired_at, last_verified } + SkillStore (BTreeMap, borrow the CompiledPipelineStore shape; HashMap banned). Thread as Arc<AsyncMutex<SkillStore>> on the runtime wiring mirroring the crystallizer at runtime_setup.rs:360.
- promote(now, human_grant): SHARD ladder MECHANISM (invocation-count + failure-rate + min-age); the apex S4 requires human_grant. Threshold constants (TIER_MIN_INVOCATIONS / TIER_MAX_FAILURE_RATE / TIER_MIN_AGE_SECS) are ELAI's OWN design values, NOT labeled SHARD-derived.
- permits_autonomous(tier) = (tier == S4). autonomous_gate::decide() step 7.5: if GateContext.candidate_skill_tier is Some(t) and !permits_autonomous(t) and !skill_autonomy_barrier_disabled() -> AutonomyDecision::RequestHuman. Default-ON; only ELAI_SKILL_AUTONOMY_BARRIER_DISABLE disables (mirror autonomous_gate_disabled_via_env :46). No ELAI_SKILL_LIBRARY_ENABLE default-off flag (W-2).

================================================================================
SSB SkillLibraryEvent (Pattern-2, 3-arm) - mirror the F-4/F-3p1 2-arm cascade
================================================================================
ONE SsbActionType::SkillLibraryEvent(SkillLibraryEventKind) with Kind { Acquired, TrustPromoted, DemotedStale } and SkillLibraryEventPayload { skill_id, description, tier, prior_tier, reason, invocations, failure_rate }. as_sql_str dispatches via kind (Self::SkillLibraryEvent(k) => k.as_sql_str(), idiom ssb.rs:8949) yielding skill_acquired / skill_trust_promoted / skill_demoted_stale. Full lockstep: enum arm + SQL CHECK (3 sql values) + as_sql_str + from_sql_str (3 parse arms) + SsbPayload variant + action_type() + payload struct + test_payload + all_test_payloads (+3, RE-COUNT 396) + ALL_VARIANTS (+1 alpha-insert) + exhaustive_match_guard (+1) + golden (alpha-insert 3 rows + renumber) + SCHEMA_VERSION 201->202 + TOTAL_VARIANTS 478->479 (+1 SsbActionType variant though 3 wire tags). Precedent: SkillEventPhase Pattern-2 (ssb.rs:21030, E-43) + RouteDecisionEventKind. Ssb::new inside rt.block_on.

================================================================================
C6b critic-gated acquisition + C6c staleness demotion
================================================================================
C6b: extend TemplateCrystallizer::check_crystallization (compiled_pipeline.rs:335) so acquisition accrues on the N-consecutive-success gate AND a reused MiniCriticVerdict::Pass; reuse MiniCritic::check / StructuralRetrySequencer::score_candidate (no 4th critic, no duplicate verdict row); retrieve learned skills via knowledge RetrievalResult (description embedding). Emit SkillLibraryEvent(Acquired).
C6c: staleness_score = (now - last_verified) / SKILL_EXPECTED_LIFETIME_SECS (SHARD shape, ELAI constants); demote_if_stale on the scheduler tick demotes one tier (S1 floor) when score > SKILL_STALE_THRESHOLD and emits SkillLibraryEvent(DemotedStale); coordinate (do not collide with) the F-5 decay.rs window - this is a distinct skill scalar.

================================================================================
Dashboard (W-4/W-6) + NON-TARGETS
================================================================================
EXTEND the shipped SkillsPanel.svelte drawer with a Trust-Ladder section (S1-S4 tiers + recent SkillLibraryEvents), fed by a NEW commands_f6::get_skill_library + a types.ts DTO + a stub.ts record_ fn; register the Rust<->TS pair in cross_lang_diff.py. ONE skill surface (the existing drawer) to avoid overlapping skill panels; fallback to a sibling panel mounted in the same drawer only if extension breaks the E-43 telemetry render.
NON-TARGETS (sg GROUNDED, must NOT become acceptance numbers): Voyager 3.3x (Minecraft items, vendor-reported, frozen 2024-04, non-transferable - mechanism only), 70% reuse (ELAI-internal, no baseline, DEFERRED until >=20 acquisitions + concept_class tagging), Springdrift +10.7% (C7, synthetic/unreproduced, must not leak into any F-6 measurable). The Voyager critic-loop + SHARD ladder/decay SHAPES are grounded and reused; only the numbers are forbidden.

OPERATIONAL: local-rag recovered (F-2b memory-writer mcp_memorize succeeded); if it degrades again, memory-writers fall back to direct memory-bank Edits.

================================================================================
SLICE A4 SHIPPED (s-60824) -- C6c staleness wiring + CLI library surface
================================================================================
Staleness tick (task_runner.rs):
- run_skill_staleness_tick(writer, store: &Arc<AsyncMutex<SkillStore>>, now) -- one demote_if_stale sweep; emits one SkillLibraryEvent(DemotedStale) row per DemotedSkill via writer.write_payload (RUNNER_AGENT_ID/RUNNER_SUPERSTEP). Takes the store Arc DIRECTLY (NOT &ResolvedConfig -- ResolvedConfig has no Clone derive + holds trait-object Arcs; adding Clone = high blast-radius. Refactor avoids it).
- run_skill_staleness_tick_loop(writer, store, interval_secs, cancel, max_ticks) -- bounded periodic wrapper mirroring run_refusal_sla_tick (consume-first-tick, select! cancel vs interval). SKILL_STALENESS_TICK_INTERVAL_SECS=3600 (matches F-5 DEFAULT_DECAY_TICK_INTERVAL cadence; ELAI-own, not externally sourced).
- Spawn spliced into run_task_runner ALONGSIDE the refusal-sla tick, gated by `if let Some(skill_store) = resolved.skill_store.as_ref()` -- W-2 (NO ELAI_*_ENABLE flag); a None store simply means no spawn. DISTINCT skill scalar -- never touches the F-5 elai_memory::FactStore decay window (skill rungs S1..S4 vs memory trust scores).
- skill_trust::SkillStore::upsert(LearnedSkill) -> SkillTrustTier added (pub) so the test + CLI can seed a known tier/last_verified for an observable sweep (acquire only seeds S1).
- 1 new orchestrator test: skill_staleness_tick_loop_demotes_and_emits (start_paused, bounded 1 tick: stale S3 -> S2 + exactly one skill_demoted_stale SSB row).

CLI (elai skills-mgmt library <list|promote|show|tick>):
- args.rs SkillsAction +4: LibraryList / LibraryPromote{id,human_grant} / LibraryShow{id} / LibraryTick.
- args_skills.rs: "library" -> parse_skills_library_args nested router (list/promote/show/tick; promote takes <id> [--human-grant]).
- skills_cmd.rs: 4 exhaustive match arms (run() match WOULD fail to compile otherwise) + handlers. Each verb seeds a deterministic demo store (seed_demo_store: fresh climbable S2 + stale S3) -- mirrors routing_cmd "re-run harness so the surface is observable without persistent state" (SkillStore is process-local; a one-shot CLI does not share a live runtime's store). promote climbs one rung + emits TrustPromoted; tick runs demote_if_stale + emits DemotedStale per record. SSB emitted via emit_skill_library_event: Ssb::new INSIDE rt.block_on (bench_seccodeplt pattern; no-reactor panic only at live run). SkillLibraryView serde DTO for --json. +4 unit tests.
- main.rs UNCHANGED -- the existing CliAction::SkillsManagement arm already routes the whole SkillsAction enum to skills_cmd::run, so the new arms dispatch automatically.
- NO SSB schema change in A4 -- A1 already shipped the SkillLibraryEvent variant (live SCHEMA_VERSION=202, all_test_payloads=399). A4 only EMITS the existing variant.

VERIFY: cargo check -p elai-orchestrator --tests and -p elai-cli --tests both GREEN. NOTE: elai-knowledge is currently broken by a CONCURRENT session's uncommitted WIP (crates/knowledge/src/deep.rs + pipeline.rs -- RRF constant-visibility refactor mid-flight: DEEP_CAP_DEFAULT / RRF_K_CONSTANT / RRF_FLOOR_DEFAULT not in scope, RRF_SYMBOL_GRAPH_VEC_WEIGHT private). This blocks a workspace check. Verification was done by git-stashing ONLY those 2 files, running -p elai-cli --tests against committed-good knowledge (GREEN, 42.91s), then git-stash-pop restoring their WIP byte-identical. F-6 A4 touches NONE of those files.
