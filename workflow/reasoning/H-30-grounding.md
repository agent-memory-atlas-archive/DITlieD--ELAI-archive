GROUNDING SWEEP -- phase H-30
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-30_staleness_intent_drift_metric.md
Date: 2026-06-06
FSM: PLANNING | Session: s-8404 lineage

SPEC REFERENCES
H-30 has no "Spec references:" lines. It references code by path+line and two arXiv IDs in its Grounding section.

CODE REFERENCES (all verified by read)
crates/memory/src/fact.rs:138 MemoryFact struct | GROUNDED | read: struct MemoryFact at line 138, ingested_at_ms:162, valid_from_ms:164, valid_to_ms:167, superseded_by:179 -- all confirmed
crates/memory/src/fact.rs:20 FactType enum | GROUNDED | enum FactType at line 20, 8 variants (VoiceStyle..Mistake) confirmed
crates/memory/src/fact.rs:81 per-type thresholds | GROUNDED | retrieval_threshold() at line 81 confirmed with per-variant values
crates/memory/src/store.rs:55 FactStore | GROUNDED | pub struct FactStore at line 55 confirmed
crates/memory/src/store.rs:208 supersede | GROUNDED | pub fn supersede at line 208 confirmed
crates/memory/src/store.rs:303 soft_close | GROUNDED | pub fn soft_close at line 303 confirmed
crates/memory/src/retrieval.rs:46 get_relevant_facts | GROUNDED | pub fn get_relevant_facts at line 46 confirmed
crates/memory/src/retrieval.rs:17 RetrievalFilter | GROUNDED | pub struct RetrievalFilter at line 17 confirmed
crates/memory/src/retrieval.rs:62-76 SafeHarbor exclusion | GROUNDED | safety_critical branch at lines 62-76 confirmed with safety_critical_blocked log
crates/memory/src/retrieval.rs:96-97 evidence_pointers filter | GROUNDED | .filter(|f| !f.evidence_pointers.is_empty()) at line 96-97 confirmed
crates/memory/src/contradiction.rs:64 audit_conflicts | GROUNDED | pub fn audit_conflicts at line 64 confirmed
crates/memory/src/semantic_contradiction.rs:392 adjudicate_semantic | GROUNDED | pub fn adjudicate_semantic at line 392 confirmed
crates/memory/src/semantic_contradiction.rs:115 DeterministicNliJudge | GROUNDED | DeterministicNliJudge struct at line 115 confirmed; "offline lexical, default-ON" confirmed
crates/cli/src/live_cli.rs:2034/:2076 audit_conflicts + adjudicate_semantic | VERIFY_NEEDED | audit_conflicts fires at line 2096, adjudicate_semantic at 2138; plan cites 2034/2076 which are in a different block (staleness quarantine broadcast); the FUNCTIONS do fire as first production callers but at different line numbers than cited
crates/cli/src/live_cli.rs:2006-2008 memory_conflict_detected action_type | VERIFY_NEEDED | lines 2006-2008 are the broadcast_id hasher block, NOT the conflict_detected emit; conflict_kind="semantic_contradiction" emit is at line 2069 comment/2200 block; semantic correct, line numbers off
crates/orchestrator/src/conversation.rs ZERO memory refs | GROUNDED | grep confirmed zero get_relevant_facts / MemoryFact / FactStore / elai_memory references in conversation.rs
crates/orchestrator/src/context_budget.rs:1066 ContextSlot / SLOT_COUNT=8 | GROUNDED | enum ContextSlot at line 1066 confirmed; SLOT_COUNT=8 at line 10; SsbEntries=5 at line 1072; OutputReserve=7 at line 1074 -- discriminant values confirmed
crates/orchestrator/src/determinism.rs:48 ELAI_SEED_ENV | GROUNDED | pub const ELAI_SEED_ENV at line 48 confirmed
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs:65 include_str! corpus | GROUNDED | CORPUS_JSON include_str! at line 65 confirmed
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs:150 SecCodePltReport | GROUNDED | pub struct SecCodePltReport at line 150 confirmed
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs:164 seed | GROUNDED | pub seed: u64 at line 164 confirmed
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs:166 corpus_blake3 | GROUNDED | pub corpus_blake3: String at line 166 confirmed
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs:312 run() | GROUNDED | pub fn run(&self) -> SecCodePltReport at line 312 confirmed
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs:44 InsecureRateMeasured beacon | GROUNDED | SSB emission comment at line 44 confirmed
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs:23 honesty discipline | GROUNDED | "before/after are COMPUTED from evaluate_vibegate, never literals" at line 23 confirmed
crates/cli/src/bench_cmd.rs:139 run_bench_seccodeplt | GROUNDED | pub(crate) fn run_bench_seccodeplt at line 139 confirmed
crates/cli/src/bench_cmd.rs:149 run_bench_safedelta | GROUNDED | pub(crate) fn run_bench_safedelta at line 149 confirmed
crates/orchestrator/src/ssb.rs:2990 SCHEMA_VERSION=220 | MISMATCH | actual SCHEMA_VERSION is 238 (ssb.rs:3259); plan says 220 -- stale by 18 versions
crates/orchestrator/tests/action_type_lock.rs:624 TOTAL_VARIANTS=497 | MISMATCH | actual TOTAL_VARIANTS is 515 (action_type_lock.rs:808); plan says 497 -- stale by 18 variants
crates/orchestrator/src/ssb.rs:15146 write_loop | MISMATCH | write_loop defined at line 16318 (async fn write_loop); line 15146 is TemplateAutoReverted match arm; write_loop tokio::spawn at line 15949; line number wrong but function exists
odysseus memory_extractor.py:32-35 _fingerprint_entries | GROUNDED | _fingerprint_entries at line 32 confirmed; "order-independent, depends only on id+text+category" at line 33 confirmed
odysseus memory_extractor.py:441-443 short-circuit | GROUNDED | fingerprint short-circuit at lines 441-443 confirmed
odysseus memory_extractor.py:573 persist fingerprint | GROUNDED | _save_tidy_state call at line 573 confirmed
odysseus memory_extractor.py:535-540 >50% deletion guard | GROUNDED | before_count >= 8 and after_count < before_count * 0.5 guard at lines 540 confirmed
odysseus memory.py:255 categorize_memory_by_relevance | VERIFY_NEEDED | function exists at src/memory.py:255 (NOT services/memory/memory.py which is an 11-line shim); path in plan is ambiguous but line number correct for src/memory.py
odysseus memory.py:291 get_relevant_memories | GROUNDED | function at src/memory.py:291 confirmed
odysseus memory.py "0.9 identity floor" | VERIFY_NEEDED | 0.9 score exists at src/memory.py:339 INSIDE get_relevant_memories (line 291 function), not inside categorize_memory_by_relevance (line 255 function); plan attributes both to the same block but the 0.9 is in the :291 function; semantically correct, attribution imprecise

CLAIM CLASSIFICATION
arXiv:2605.06527 best-model 55.2% conflict-resolution | INGEST | not in RAG; paper not ingested; load-bearing threat-magnitude claim; ingest to verify numbers
arXiv:2605.06527 most frameworks <10% conflict-resolution | INGEST | same paper; same status
arXiv:2605.06527 recognition 76% / application 39% gap | INGEST | same paper; this gap is the design target for the eval harness -- load-bearing
arXiv:2603.23848 BeliefShift state-X-then-Y belief-revision protocol | INGEST | not in RAG; plan reimplements the protocol shape; verify ID exists and describes this protocol
Graphiti +68%-token trap (narrating [OUTDATED] instead of hard-filtering) | INGEST | attributed to arXiv:2501.13956 in H-31; that paper not ingested in this project's RAG; the +68% number is cited as "measured" -- needs grounding against the Graphiti paper; not blocking the H-30 instrument design (which asserts the design avoids this) but the number itself is ungrounded
Bi-temporal valid-time/transaction-time supersession concept | GROUNDED | crates/memory/src/fact.rs:162-167 shows the three-timestamp triple (ingested_at_ms / valid_from_ms / valid_to_ms) and superseded_by:179; architecture confirmed in code
SCHEMA_VERSION baseline+1 = 221 for StalenessEvalMeasured | MISMATCH | live SCHEMA_VERSION is 238; plan says 220 base + 1 = 221; executor MUST re-read ssb.rs at edit time and use live+1 (currently 239); using 221 would create a gap of 17 missing versions in the append-only log -- P0 data integrity risk
TOTAL_VARIANTS 497->498 for StalenessEvalMeasured | MISMATCH | live TOTAL_VARIANTS is 515; plan says 497+1=498; executor must use live+1 = 516
get_relevant_facts WIRE-DARK (zero prod callers) | GROUNDED | grep confirmed conversation.rs has ZERO memory references; all callers are tests; retrieval.rs:46 confirmed
broken-retrieval arm R2 framing (dead-evidence path screams) | GROUNDED | plan's observable_effect expect_broken matches R2 doctrine exactly: "A run where the broken arm STILL reports 0 is itself a FAIL" -- correctly framed, consistent with wiring-enforcement.md R2
DeterministicNliJudge offline lexical, default-ON | GROUNDED | semantic_contradiction.rs:115 confirmed; the kill-switch ELAI_SEMANTIC_CONTRADICTION_KILL makes it disable-only, so default is ON

ACTIONS REQUIRED
- MISMATCH ssb.rs SCHEMA_VERSION: plan says 220/221 but live is 238; executor MUST grep ssb.rs:SCHEMA_VERSION at edit time and use live+1; using plan's number creates a 17-version gap in the append-only log -- P0 data integrity
- MISMATCH action_type_lock.rs TOTAL_VARIANTS: plan says 497->498 but live is 515; executor uses live+1 = 516; same P0 risk
- MISMATCH ssb.rs write_loop line: plan cites :15146 but write_loop is at :16318; minor -- the function exists, but executor should grep write_loop at edit time rather than jump to the stale line number
- MISMATCH live_cli.rs audit_conflicts line: plan cites :2034/:2076 but actual fire sites are :2096/:2138; same advisory -- functions exist and are prod-wired, line numbers stale
- INGEST arXiv:2605.06527 via paper-ingestor mode=paper; the 55.2% / <10% / 76%recognition/39%application numbers are the threat-magnitude justification for the instrument; if the paper does not exist or the numbers differ, the premise weakens
- INGEST arXiv:2603.23848 BeliefShift via paper-ingestor mode=paper; the fixture protocol mirrors this paper's belief-revision protocol shape; verify the ID and description
- INGEST arXiv:2501.13956 Graphiti/Zep via paper-ingestor mode=paper; the +68%-token narration trap is attributed to this paper in H-31 and inherited by H-30's stub_ban; verify the number exists in the paper before EXECUTING any code that uses it as design rationale
- VERIFY odysseus memory.py path: plan cites "memory.py:255" but the relevant file is docs/.source/odysseus-main/src/memory.py (not services/memory/memory.py which is an 11-line shim); executor should use the full path in any code references or comments

VALIDATION WARNINGS
- no "Spec references:" lines in source -- H-30 references code directly by path+line and arXiv IDs instead; this is intentional (Grounding section), not a gap in coverage, but the spec-grounder notes it per rule 5
- SCHEMA_VERSION / TOTAL_VARIANTS stale by 18 versions: the H-plan index notes live 220/497 but actual code is 238/515 (RAG entry h26-shadow-exec confirms 234/511, with further concurrent additions); the stale baseline in H-30 is a pre-existing issue common to all H-plan files authored in the same batch -- not a defect in H-30 specifically, but executor must treat the plan's numbers as floor estimates only
- arXiv IDs 2605.06527 and 2603.23848 are not in the project RAG; they are cited as "snapshot 2026-06-04" suggesting they were live-checked at authoring but not formally ingested
- all sources validated (code refs) except the three MISMATCHes above and three INGEST items
