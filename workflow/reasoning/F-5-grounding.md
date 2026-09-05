GROUNDING SWEEP -- phase F-5
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-5_staleness_quarantine_cascade.md
Date: 2026-06-02
FSM: PLANNING | Session: s-60824

SPEC REFERENCES
(F-5 is an F-plan synthesis-driven plan; it cites synthesis/prior-art/source files, not docs/specs/guide/steps/phase-*.txt. Each cited Reads entry verified below.)
VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md §4 C5 + §1 + §3 + §5 | GROUNDED | read in full; §4 C5 (line 335-361) carries mechanism+ELAI-ground-truth+REVISED-SCOPE+always-1-hazard+measurable; §1.x corrections present (no C5 correction, but §1 read); §3 SHARD=blueprint (line 146-164); §5 sequencing "after B-30; concept_stale audit FIRST" (line 459)
00-COMPARISON-shard-vs-priorart.md | GROUNDED | §2 Pillar1 line 34: SHARD staleness=(now-last_verified)/expected_lifetime, flag>1.0, quarantine>2.0; §3 line 82 "unique against all rivals (invalidate on contradiction, never on age)"; handle_stale_memory cascade real in composition.py (line 73 + provenance line 98)
crates/memory/src/decay.rs:40-120 | GROUNDED | decay_tick DEF at decay.rs:88 (exact); DecayTickStats struct decay.rs:69; decayed_trust decay.rs:45
crates/memory/src/decay_scheduler.rs:35-150 | GROUNDED | spawn_decay_scheduler DEF at :101 (exact); DecayTickObserver type alias :88; on_tick observer hook :105/:126 (exact)
crates/memory/src/contradiction.rs:60-145 | GROUNDED | audit_conflicts DEF at :64 (plan cites :72 -- :72 is the function body line `let rows = store.list_current`; fn header is :64; minor 8-line drift, function real and is the bi-temporal conflict closer)
crates/knowledge/src/dkg/mod.rs:25-76 + 360-380 | GROUNDED | `concept_stale INTEGER DEFAULT 1` at dkg/mod.rs:36 (exact); only writer = INSERT at :369 passing the default (exact) -- the always-1 hazard confirmed at source
crates/knowledge/src/cosmos_worker.rs:90-145 + 800-820 | GROUNDED | CosmosStatus enum :134, Stale variant :135/:144; status_for DEF :729 (plan cites :122/:811 -- :122 is doc near enum, :811 is a status_for call; the Stale variant + status_for both real, reachable)
crates/cli/src/dkg_lifecycle.rs:1-120 | GROUNDED | ensure_dkg_or_rebuild DEF :127 (+ _with_clock :149); plan cites :12 for the symbol -- the fn header is :127, :12 is module-top; symbol real
crates/knowledge/src/knowledge_ssb_sink.rs:135-220 | GROUNDED | KnowledgeSsbSink trait :169; emit_dkg_rebuild_started :191 (exact)
crates/orchestrator/src/ssb.rs:5319-5321 + 15885-15898 | VERIFY_NEEDED | MemoryFactSuperseded variant at ssb.rs:5471 (plan cites :5320), payload MemoryFactSupersededPayload at :16229 (plan cites :15891). Variant + payload REAL and are the correct E-5 Pattern-2 reference shape, but the line numbers in the plan have drifted ~150/~340 lines. Implementer must re-locate by symbol, not line.
crates/cli/src/live_cli.rs:1770-1810 | VERIFY_NEEDED | in-line decay_tick call is at live_cli.rs:1846 (plan cites :1789). The W-3 live path EXISTS (post-reflect `elai_memory::decay_tick(&facts, now_ms)` :1846, emits MemoryEvolutionRun phase="decay_tick" :1858). Line drifted ~57 lines; relocate by symbol.
arXiv:2501.13956 Graphiti §2.2.3 | GROUNDED | paper RESOLVES (03-zep-graphiti.md:32); bi-temporal 4-timestamp + non-destructive invalidation verbatim (dossier §3.2/§3.3 lines 68-83, confirmed in edge_operations.py per comparison line 100)
SHARD composition.py handle_stale_memory | GROUNDED (as BLUEPRINT, reliability MEDIUM) | fetched real 2026-06-01 (comparison §4 line 98); borrow the cascade SHAPE not the code (synthesis §3)

CLAIM CLASSIFICATION
E-5 ships bi-temporal for memory_facts only | GROUNDED | synthesis §4 C5 line 347-349 + comparison; ELAI is NOT missing bi-temporal -- binding correction
cross-subsystem propagation is F-5 NEW scope | GROUNDED | synthesis §4 C5 NOVELTY=GENUINELY_NEW (line 336); §5 ranks C5 T3 (line 459); the gap is SCOPE not mechanism
concept_stale is ALWAYS 1 / never populated | GROUNDED | source: dkg/mod.rs:36 DEFAULT 1, only writer :369 passes default; synthesis line 348 + DETAILED_SUMMARY_01:119
SHARD time-decay staleness + quarantine is unique vs all rivals | GROUNDED | comparison §3 line 82; Letta/Mem0/Graphiti invalidate on contradiction not age (comparison §2 line 34)
SHARD staleness = (now-last_verified)/expected_lifetime, flag>1.0, quarantine>2.0 | GROUNDED (as SHARD's constants, NOT a port) | comparison §2 line 34; these are SHARD's thresholds
ELAI reuses existing decay math -- NO new staleness formula (plan Task 4) | UNGROUNDED-as-stated / MISMATCH | decay.rs has EXPONENTIAL trust-decay exp(-rate*days) clamped 0-1 (decay.rs:45-64), NOT a (now-last_verified)/expected_lifetime ratio. There is NO staleness_ratio in decay.rs today. The plan's staleness_ratio>2.0 quarantine cut is a SHARD-derived DESIGN CHOICE, not reuse of existing math. Plan's own Citations block (line 220) flags this correctly; the Task-4 phrasing "reusing the existing decay math ... NO new staleness formula" overstates it.
staleness_ratio > 2.0 = quarantine threshold | DO-NOT-TARGET / design constant | SHARD's constant (comparison line 34). Must be confirmed as ELAI own design value in F-5-decisions.md (plan line 220 acknowledges). NOT a measured ELAI guarantee.
Mem0 A.U.D.N. (ADD/UPDATE/DELETE/NOOP) | GROUNDED | 02-mem0.md:63-64/179/235 (grounded [C3]); top-k-retrieve-then-LLM-classify
Graphiti bi-temporal LLM-judged non-destructive invalidation | GROUNDED | 03-zep-graphiti.md §3.3 line 82-83 verbatim; confirmed edge_operations.py (comparison line 100); arXiv:2501.13956 §2.2.3
audit_conflicts is the existing bi-temporal conflict closer (reuse, do not re-detect) | GROUNDED | contradiction.rs:64 (auto_supersede_by_tier path); reuse satisfies synthesis correction (no 2nd detector)
SCHEMA_VERSION baseline+1 (baseline 191); TOTAL_VARIANTS 470 -> 471 | MISMATCH (STALE) | action_type_lock.golden head is now SCHEMA_VERSION 201 / TOTAL_VARIANTS ~478 (F-2b), NOT 191/470. F-10 already took 199->200 / 476->477 THIS session (s-60824); F-2b took 201. CLAUDE.md context (191/464) is itself stale vs the golden file. F-5 must bump from the CURRENT golden head at implement time, alphabetical insert, NOT 470->471.
broadcast_id = blake3 prefix of fact_id||tick_ms (blake3 only) | GROUNDED | CLAUDE.md HARD RULE blake3 never blake2/sha2; consistent
QUARANTINE_QUEUE_CAPACITY = 256 named const; tokio::sync::mpsc bounded | GROUNDED (design value) | CLAUDE.md tokio::sync in async + named-constant rule; 256 is an ELAI tuning start-value to confirm in decisions doc (plan line 226 acknowledges)
dropped_count surfaces drop-oldest (no silent loss) | GROUNDED | CSQ/counterfactual guard per CLAUDE.md; internally consistent
CosmosStatus has a reachable Stale state (reuse, no new variant) | GROUNDED | cosmos_worker.rs:135 Stale, :144 "stale", reachable from status_for :729
ensure_dkg_or_rebuild is the C-38 production DKG entrypoint | GROUNDED | dkg_lifecycle.rs:127; wired path per plan; symbol real
"grounding-cache miss on re-query" measurable assumes a cache keyed on concept freshness | UNGROUNDED (assumption) | plan line 220 + 224 flags it: confirm in Phase-1 audit (Task 1) that the cache key reads concept_stale BEFORE relying on this measurable. No cache-key read verified this sweep.
no new external dependency | GROUNDED | Cosmos/DKG/queue all shipped crates; tokio already in tree; anti-slopsquatting satisfied
F-5 fully deterministic / not RunPod-gated (mock-clock, in-process) | GROUNDED | SQLite UPDATE + in-mem mpsc; no network/subprocess/model on gate path (plan W-10)

ACTIONS REQUIRED
- MISMATCH (elevate, resolve before TESTING_FIRST): plan's "reusing the existing decay math ... NO new staleness formula" (Task 4) contradicts decay.rs:45-64 which is exponential trust-decay, not a (now-last_verified)/expected_lifetime ratio. Either (a) derive staleness_ratio from the existing trust/last_observed fields as an EXPLICIT new mapping, or (b) reword Task 4 to "introduce a staleness_ratio mapping over the existing decay inputs". Confirm the >2.0 cut as ELAI own design value in F-5-decisions.md (it is SHARD's constant, NOT a ported guarantee).
- MISMATCH (elevate, resolve before TESTING_FIRST): SCHEMA_VERSION / TOTAL_VARIANTS in the plan (191 baseline / 470->471) are STALE. action_type_lock.golden head = SCHEMA_VERSION 201 / TOTAL_VARIANTS ~478 (F-2b; F-10 took 199->200/476->477 in this same s-60824 session). F-5 must bump from the live golden head with an alphabetical insert of staleness_quarantine_broadcast, NOT the plan's 470->471.
- DO-NOT-TARGET: staleness>2.0 quarantine and >1.0 flag are SHARD's constants; QUARANTINE_QUEUE_CAPACITY=256 is an ELAI tuning start-value. None are measured ELAI guarantees -- never publish or treat as a benchmark target; record as design choices in F-5-decisions.md.
- VERIFY before EXECUTING: re-locate MemoryFactSuperseded (ssb.rs:5471 variant / 16229 payload), live decay_tick (live_cli.rs:1846), audit_conflicts (contradiction.rs:64), status_for (cosmos_worker.rs:729), ensure_dkg_or_rebuild (dkg_lifecycle.rs:127) BY SYMBOL -- every plan line number has drifted (50-340 lines). Symbols all confirmed real this sweep; only the line anchors are stale.
- VERIFY in Phase-1 audit (Task 1): confirm the grounding cache key actually reads concept_stale before relying on the "grounding-cache miss on re-query" measurable; if no such cache key exists, drop that half of the measurable.
- No paper-ingest required: arXiv:2501.13956 Graphiti, Mem0 A.U.D.N., SHARD handle_stale_memory are all GROUNDED via primary-source-verified prior-art dossiers (workflow/research/shard-prior-art/) read this sweep. SHARD composition.py is reliability MEDIUM (fetched repo) -- borrow SHAPE only, never as ground truth.

VALIDATION WARNINGS
- No docs/specs/guide/steps/phase-*.txt references in source -- expected: F-5 is a synthesis-driven F-plan, traceable to VIBESEC-SHARD-STEAL-AND-BUILD §4 C5 + the shard-prior-art comparison, not to the numbered guide-steps spec.
- All cited ELAI source symbols confirmed real; ALL cited line numbers are stale (drift 8-340 lines) -- re-anchor by symbol at implement time.
- SCHEMA_VERSION/TOTAL_VARIANTS baseline in both the plan AND CLAUDE.md context is stale vs action_type_lock.golden head (201/~478) -- the golden file is the single source of truth for the bump.
- E-5/B-30/B-42 prerequisite framing consistent with synthesis §5; B-30 dependency on dkg_nodes is a real sequencing gate (plan Open-questions line 225) -- not a grounding defect, a sequencing risk.
