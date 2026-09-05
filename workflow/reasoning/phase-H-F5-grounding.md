GROUNDING SWEEP -- phase H-F5
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-5_staleness_quarantine_cascade.md
Date: 2026-06-01
FSM: EXECUTING | Session: s-6344

SPEC REFERENCES
-- F-5 cites synthesis VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md as primary spec.
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md sections 1,3,4C5,5 | GROUNDED | file exists; C5 at line 335-361 contains staleness formula (flag>1.0, quarantine>2.0), handle_stale_memory cascade, cross-subsystem scope claim, concept_stale always-1 hazard, Phase-1-then-Phase-2 sequencing -- all on-topic.
synthesis section 1 (correction: ELAI NOT missing bi-temporal) | GROUNDED | verified at synthesis lines 71-73: "1. GROUNDING DISCIPLINE - CORRECTIONS..."; C5 line 349 explicitly: "correction: ELAI is NOT missing bi-temporal".
arXiv:2501.13956 Graphiti §2.2.3 (bi-temporal contradiction invalidation) | GROUNDED | synthesis C5 line 344: "Graphiti arXiv:2501.13956 §2.2.3 (confirmed in edge_operations.py)" -- on-topic; bi-temporal edge-close pattern used in Task 5.
ELAI ground truth (code-read, W-7 confirmed in elai.map + source) | GROUNDED | all symbols confirmed below in CLAIM CLASSIFICATION.
DETAILED_SUMMARY_01:119 (concept_stale always-1) | GROUNDED | synthesis C5 line 348: "DKG has concept_stale that is ALWAYS 1 and never populated (DETAILED_SUMMARY_01:119)"; dkg/mod.rs:36 confirms DEFAULT 1; mod.rs:370 INSERT always passes literal 1.

CLAIM CLASSIFICATION
decay_tick at decay.rs:88 emits DecayTickStats | GROUNDED | grep confirmed: decay.rs:88 "pub fn decay_tick(store: &FactStore, now_ms: u128) -> Result<DecayTickStats, MemoryError>"
spawn_decay_scheduler at decay_scheduler.rs:101 with on_tick:Option<DecayTickObserver> at param pos 4 | GROUNDED | read confirmed: spawn_decay_scheduler at line 101, on_tick parameter at line 105
DecayTickObserver type alias at decay_scheduler.rs:88 | GROUNDED | read confirmed: "pub type DecayTickObserver = Arc<dyn Fn(&DecayTickStats) + Send + Sync + 'static>" at line 88
audit_conflicts at contradiction.rs:64 (plan claims line 72) | VERIFY | grep confirmed fn at line 64, not 72; stale line ref, function exists and matches purpose
live_cli.rs in-line decay_tick call at line 1789 (plan claim) | MISMATCH | actual call is at line 1796; plan claims 1789; verified via read lines 1785-1796; 7-line drift
ensure_dkg_or_rebuild wired at live_cli.rs:1722-1732 (plan claim) | MISMATCH | ensure_dkg_or_rebuild is NOT present in live_cli.rs; it is called from knowledge_stack.rs:316; the plan's "wired at live_cli.rs:1722-1732" claim is wrong
ensure_dkg_or_rebuild fn at dkg_lifecycle.rs:12 (C-38, plan claim) | VERIFY | fn is at line 127; line 12 is a module-level comment referencing the fn name; plan refers to both as "C-38, line 12" which is the comment not the fn
MemoryFactSuperseded variant at ssb.rs:5320 (plan claim) | VERIFY | actual line is 5348 (28-line drift); variant and payload shape match Pattern-2 description
MemoryFactSupersededPayload at ssb.rs:15891 (plan claim) | VERIFY | actual struct definition at line 15958; fields (old_fact_id, new_fact_id, valid_to_ms) confirmed -- shape correct, line stale
SCHEMA_VERSION baseline 191 for bump (plan claim) | MISMATCH | current SCHEMA_VERSION is 196 (ssb.rs:2742 "const SCHEMA_VERSION: i64 = 196;"); plan's "baseline 191 per CLAUDE.md" is stale; bump must be 196->197, not 191->192
TOTAL_VARIANTS 470->471 for golden lock (plan claim) | MISMATCH | golden lock comments show TOTAL_VARIANTS already at 473; plan's "470->471" is stale; actual bump must be 473->474
concept_stale INSERT always literal 1 in dkg/mod.rs:369-370 | GROUNDED | read confirmed: INSERT at line 368-370 passes literal "1" for concept_stale column; DEFAULT 1 at schema line 36
concept_stale SELECTs in dkg_adapter.rs (plan claims lines 63,140,441,451,462) | VERIFY | grep of dkg_adapter.rs shows NO concept_stale column reads in those SELECTs; the listed lines are SELECT queries but they select id/name NOT concept_stale; plan reads are over-claimed -- the always-1 hazard audit must extend to whether ranking functions downstream use concept_stale implicitly via a WHERE filter or ORDER BY
CosmosStatus::Stale variant reachable from status_for | GROUNDED | cosmos_worker.rs:134-145 confirms CosmosStatus enum with Fresh/Stale/Decayed; status_for at line 729 returns Stale; B-42 confirmed shipped via RAG
KnowledgeSsbSink trait + emit_dkg_rebuild_started at knowledge_ssb_sink.rs:191 | GROUNDED | read confirmed: trait at line 169, emit_dkg_rebuild_started at line 191; exact match
B-30 DKG hardening SHIPPED (prereq) | GROUNDED | RAG confirms: "B-30 Multilang DKG ALL FOUR PASSES SHIPPED 2026-05-15/16 s-21092"
E-5 bi-temporal FactStore SHIPPED (prereq) | GROUNDED | RAG confirms: "E-5 SHIPPED s-62376 2026-05-20 vertical slice long-term user-modeling layer"
B-42 Cosmos artifact tier SHIPPED (prereq) | GROUNDED | RAG confirms E-48 session notes B-42 as shipped substrate; CosmosStatus confirmed in source
concept_stale=2 for quarantine (plan sets quarantine -> concept_stale=2) | MISMATCH | synthesis C5 line 360 says "concept_stale=1 on all DKG concepts citing it" as the measurable post-quarantine state; plan Tasks 3/5 use concept_stale=2 for quarantined, concept_stale=1 for non-stale; synthesis says =1 post-quarantine which contradicts plan -- the plan's 3-tier schema (0=fresh,1=default/maybe-stale,2=quarantine) is an elaboration not explicitly in the synthesis; confirm this design choice in F-5-decisions.md before Task 5 lands
staleness_ratio >2.0 quarantine threshold | GROUNDED | synthesis C5 line 339: "flag >1.0; quarantine >2.0" -- matches plan exactly
bounded fan-out queue with drop-oldest + dropped_count | GROUNDED | synthesis C5 line 357-358: "Phase 2: implement the cascade with a BOUNDED fan-out queue (explicit channel capacity) to prevent scheduler-tick pile-up" -- on-topic
blake3 for broadcast_id | GROUNDED | CLAUDE.md HARD RULE Anti-Stale-Training: "blake3 (never blake2/sha2 for non-RFC uses)"
QUARANTINE_QUEUE_CAPACITY=256 (named constant) | UNGROUNDED | no citation in synthesis or spec; design choice -- document rationale in F-5-decisions.md
spawn_decay_scheduler currently wire-dark (only a comment references it) | GROUNDED | grep of live_cli.rs:1790 comment "spawn_decay_scheduler is wired into the orchestrator task_runner separately"; source read confirms no active call site in production paths

ACTIONS REQUIRED
- MISMATCH live_cli.rs decay_tick line drift (1789 vs 1796): update reachable_from annotation in F-5 WIRING_CONTRACT to line 1796; no functional change
- MISMATCH ensure_dkg_or_rebuild not wired at live_cli.rs:1722-1732 -- it is wired at crates/cli/src/knowledge_stack.rs:316; correct the W-7 grounding note in F-5 plan + F-5-decisions.md before TESTING_FIRST
- MISMATCH SCHEMA_VERSION baseline stale: plan says "baseline 191 per CLAUDE.md"; actual current is 196; bump must be 196->197; update Task 3 + ACCEPTANCE item + WIRING DONE-GATE text before coding the SSB variant
- MISMATCH TOTAL_VARIANTS stale: plan says "470->471"; golden lock shows 473 already; bump must be 473->474; update Task 3 + action_type_lock context before TESTING_FIRST
- MISMATCH concept_stale=2 quarantine value vs synthesis =1: synthesis measurable says "concept_stale=1 on all DKG concepts citing it"; plan uses concept_stale=2 for quarantine; if the 3-tier schema (0/1/2) is the authoritative design choice, record it explicitly in F-5-decisions.md with rationale before Task 5 lands -- if it is a copy error, correct to align with synthesis
- VERIFY dkg_adapter.rs concept_stale consumer audit: plan claims SELECTs at lines 63/140/441/451/462 read concept_stale; grep shows they do NOT include concept_stale in the SELECT column list; Phase-1 Task 1 audit must extend to WHERE clauses and any downstream ranking that may implicitly filter on concept_stale via an outer JOIN or application-level filter, not just column presence in SELECT list
- VERIFY stale line numbers (non-blocking, informational): audit_conflicts at 64 not 72; ensure_dkg_or_rebuild at 127 not 12; MemoryFactSuperseded at 5348 not 5320; MemoryFactSupersededPayload at 15958 not 15891 -- update W-7 grounding footnote in F-5 plan before TESTING_FIRST
- UNGROUNDED QUARANTINE_QUEUE_CAPACITY=256 -- no spec citation; document capacity choice rationale in F-5-decisions.md (why 256, what churn rate it supports, what the drop frequency is under artifact-churn load)

VALIDATION WARNINGS
- SCHEMA_VERSION in CLAUDE.md (191) is stale vs source (196); the plan inherits the stale value -- this is a known session-mid drift per CLAUDE.md "SCHEMA_VERSION is now ~196-197 mid-session" annotation; treat 196 as the ground truth
- concept_stale consumers in dkg_adapter.rs appear to NOT read the concept_stale column directly; the Phase-1 audit scope must include implicit filtering (WHERE clauses, ORM-level mapping, outer callers) not just column-in-SELECT presence
- ensure_dkg_or_rebuild production caller is knowledge_stack.rs not live_cli.rs; the plan's W-3 claim that DKG consumer is reachable from live_cli.rs:1722 is unverified; knowledge_stack.rs is reachable from CLI, confirm the full call chain in Phase-2 wiring
