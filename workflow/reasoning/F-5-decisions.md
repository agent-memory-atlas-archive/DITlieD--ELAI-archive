F-5 cross-subsystem staleness-quarantine + bi-temporal contradiction cascade - decision doc (session s-60824)

Pre-checks (both passed before any coding)
B-30 prerequisite: SHIPPED. Evidence (ie, greped): dkg/resolution/ multilang submodules (csharp/go_module/jvm/tsconfig, "B-30 Pass C"), language_registry.rs (Pass A), tree_sitter_health/ (abi_probe, registration TIER_A/TIER_B), grammar_tier.rs, SsbActionType::LanguageGrammarStatus (SCHEMA 36), the live dkg insert populates the language column. F-5 is unblocked.
concept_stale consumer audit (the plan's "sharp hazard"): there are ZERO production READ consumers of concept_stale. The only sites are the schema DDL DEFAULT 1 (dkg/mod.rs:36) and the sole INSERT hard-coding literal 1 (dkg/mod.rs:369-370); the other two hits are test-fixture DDL mirrors. So populating concept_stale CANNOT silently change retrieval ranking today (nothing reads it). The real risk is a WRITE/legacy-row hazard: every pre-F-5 row carries the default 1. Recorded in workflow/reasoning/F-5-concept-stale-audit.md.

What I will implement (phase 1 ship-safe first, phase 2 the cascade)
Phase 1: the concept_stale audit doc + a public set_concept_stale mutator on the DkgEngine (GAP - no mutator exists) + a retrieval contract test that asserts an UPDATE concept_stale=2 followed by a RE-QUERY returns 2 (the WRITE round-trip, NOT a constant-filter query - because there is no read filter to test). Quarantine marker value = concept_stale=2 (0 fresh / 1 default-unknown / 2 quarantined) so the cascade only ever writes 2 and never disturbs the legacy 1s.
Phase 2: the cross-subsystem cascade - memory decay emits a StalenessQuarantineBroadcast that propagates to DKG concept annotations + Cosmos artifacts through a bounded fan-out queue.

Key technical decision
The broadcast is emitted at the orchestrator/CLI boundary (the decay_scheduler DecayTickObserver hook / the inline decay tick at live_cli.rs:1846), NOT inside crates/memory/src/decay.rs, because crates/memory has no dependency on crates/knowledge; the CLI (which depends on both) drains a bounded Mutex<VecDeque> drop-oldest queue and calls the knowledge-side apply functions. One flat SSB beacon; default-ON with a single ELAI_STALENESS_CASCADE_KILL kill-switch.

Expected test outcome
cargo check --workspace --tests GREEN; f5_concept_stale_retrieval_contract asserts UPDATE->re-query; the bounded-queue test asserts drop-oldest + dropped_count; a mock-clock cascade test asserts a quarantined fact flips concept_stale=2 on a dependent concept within one tick; live `elai dkg staleness-tick --workspace .` emits a real staleness_quarantine_broadcast row + flips concept_stale=2 + prints dependents_demoted:N.

Unknowns to verify empirically
1. The fact->concept linkage criterion (no fact_id on dkg_nodes; join via the concept TEXT column) - confirm the match is precise enough not to over-demote.
2. Live all_test_payloads / TOTAL_VARIANTS / golden ordinal (RE-COUNT; baseline 479->480, SCHEMA 202->203, all_test_payloads 399->400, golden last row 0478).
3. Whether dkg/mod.rs is being concurrently edited (B-30-contended surface) at the Phase-2 edit window.

================================================================================
THREE STRUCTURAL CORRECTIONS (from the context-refresh - plan assumptions were wrong)
================================================================================
1. grounding-cache-miss measurable is UNVERIFIED: grounding_cache here is a prompt-result SSB stat (commands.rs:5307, returns stub zeros), NOT keyed on DKG concept freshness; no cache reads concept_stale. DROP the "grounding_cache miss on re-query" acceptance assertion; replace with the concept_stale UPDATE + DKG re-query returning the demoted value.
2. dkg_edges has NO valid_to/valid_from column: the bi-temporal edge close cannot UPDATE a temporal field; it INSERTs a non-destructive marker edge (edge_type='staleness_quarantined') - this IS the Graphiti/Mem0 non-destructive-invalidation pattern, just expressed as an insert.
3. No fact_id column on dkg_nodes: the only fact->concept join surface is the concept TEXT column; design the linkage + the Cosmos artifact-match criterion explicitly (concept-name match), documented here, before Phase 2.

================================================================================
SIGNATURE / SEMANTICS DECISIONS (locked before authoring)
================================================================================
- collect_quarantined(&FactStore, now) age-only (immutable borrow, matches the immutable live_cli.rs:1846 decay path); a SEPARATE contradiction sweep if needed (NOT &mut FactStore + audit_conflicts inline).
- QuarantineBroadcastQueue = Mutex<VecDeque<QuarantineRecord>>, QUARANTINE_QUEUE_CAPACITY=256, DROP-OLDEST on overflow, dropped_count surfaced in the SSB payload (NOT tokio mpsc, which drops-newest - wrong semantics; W-2: no silent queue drop).
- Emit seam: the decay_scheduler DecayTickObserver on_tick hook (Arc<dyn Fn(&DecayTickStats)>) OR the inline live_cli.rs:1846 decay tick - memory stays knowledge-free; the CLI/orchestrator bridges memory->knowledge.

================================================================================
SSB StalenessQuarantineBroadcast (single flat beacon)
================================================================================
ONE SsbActionType::StalenessQuarantineBroadcast (no Pattern-2 Kind) - mirror LanguageGrammarStatus (B-30 flat beacon) / MemoryFactSuperseded. Payload { fact_id:String, staleness_ratio:f64, dependents_demoted:u32, cosmos_artifacts_flagged:u32, tick_ms:u64, broadcast_id:String (blake3 prefix of fact_id || tick_ms), dropped_count:u32 }. Lockstep: enum arm + SQL CHECK + as_sql_str ("staleness_quarantine_broadcast") + from_sql_str + SsbPayload variant + action_type() + payload struct + test_payload + all_test_payloads (+1, RE-COUNT 399->400) + ALL_VARIANTS (+1 alpha) + exhaustive_match_guard (+1) + golden (alpha-insert after stagnation_spinning_detected, renumber from the insert point; current last 0478) + SCHEMA_VERSION 202->203 + TOTAL_VARIANTS 479->480. blake3 only. Ssb::new inside rt.block_on.

================================================================================
Integration map + dashboard + grounding
================================================================================
memory: quarantine_broadcast.rs (NEW: collect_quarantined, QuarantineRecord, QuarantineBroadcastQueue) + lib.rs + decay_scheduler.rs (observer wiring). cli: live_cli.rs (emit after the inline decay tick) + dkg_lifecycle.rs (drain queue post-rebuild) + args.rs (dkg staleness-tick subcommand). knowledge: dkg/mod.rs (set_concept_stale mutator + apply_staleness_broadcast: concept_stale=2 via concept TEXT join + marker edge) + cosmos_worker.rs (flag_on_staleness_broadcast -> CosmosStatus::Stale) + knowledge_ssb_sink.rs (emit pattern). dashboard: commands_f5.rs (get_staleness_quarantine) + lib.rs + StalenessQuarantineDrawer.svelte (NEW drawer - NOT a window; s-73316 owns MonitorWindow/IdeaFraming/Plan/Github) + DrawerHost/Drawer/types.ts/stub.ts. Tests: f5_concept_stale_retrieval_contract.rs, memory/tests/{quarantine_broadcast_cascade, quarantine_queue_bounded}, knowledge/tests/{dkg_apply_staleness_broadcast, cosmos_flag_on_broadcast}, orchestrator/tests/ssb_staleness_quarantine_roundtrip.
Grounding (sg): E-5 bi-temporal supersede ships for memory_facts ONLY -> F-5 cross-subsystem propagation is GENUINELY_NEW (the documented novelty). SHARD time-decay quarantine + Mem0 A.U.D.N. + Graphiti non-destructive invalidation are the grounded MECHANISM shapes; F-5 decay/staleness constants (QUARANTINE_QUEUE_CAPACITY etc.) are ELAI own design values, not externally-sourced targets.
HAZARDS: dkg/mod.rs is the B-30-contended surface (focused edit, coordinate the window). Build-isolate the concurrent knowledge RRF breakage (deep.rs/pipeline.rs E0425/E0603) via git-stash; do NOT edit/commit those.
