Design Elaboration Decisions (2026-04-05, session 7)

Source: LangGraph source analysis + Aider source analysis + Adversarial audit
Audit: 7 BLOCKs, 7 WARNs, 4 INFOs. All BLOCKs resolved below.

DECISION A: Deferred Node Execution
Status: ADOPTED WITH MODIFICATION
Original: Add defer=True for fan-in aggregation without branch counting.
BLOCK-5: Incompatible with BSP superstep model (BSP uses barrier, LangGraph uses quiescence).
Fix: Add quiescence detection as alternative superstep termination condition.
  BSP barrier: all scheduled tasks complete -> advance superstep (existing)
  Quiescence: no pending writes AND no deferred nodes unsatisfied -> fire deferred nodes
  Deferred nodes execute in a "cleanup superstep" after the main superstep completes.
  This preserves BSP determinism: deferred execution is a post-barrier phase, not a replacement.
Design target: PART 20 addendum.

DECISION B: Channel Taxonomy Expansion (2 -> 4 types)
Status: ADOPTED WITH CORRECTIONS
Original: Add UntrackedValue, EphemeralValue, NamedBarrierValue.
BLOCK-1: UntrackedValue does NOT auto-clear (persists until overwritten). Report was wrong.
BLOCK-2: UntrackedValue guard=True default crashes on concurrent writes.
WARN-1: PART 20.4 invariant "only LastValue and Topic" must be updated.
Fix: 4 channel types (not 5 — NamedBarrierValue folded into deferred node mechanism):
  LastValue: single value, last writer wins (existing)
  Topic: append-only list, accumulate or clear-per-step (existing)
  UntrackedValue: in-memory only, never checkpointed, guard=false for multi-writer.
    Use case: large transient objects (trace reports, intermediate ASTs).
    CRITICAL: does NOT auto-clear. Must be explicitly overwritten or ignored.
  EphemeralValue: cleared to MISSING on steps with no writes. For one-shot signals.
    Use case: "branch taken" signals between agents, notification flags.
Update PART 20.4 invariant to: "Only LastValue, Topic, UntrackedValue, and EphemeralValue channel types exist."

DECISION C: Send / Dynamic Fan-Out
Status: ADOPTED WITH NAMESPACE FIX
Original: Runtime spawning of N parallel sub-tasks via Send objects.
BLOCK-6: Checkpoint namespace collision for concurrent same-type subgraphs.
Fix: Subgraph namespace format: "{parent_ns}|{subgraph_type}:{AtomicU64_counter}"
  Counter is per-subgraph-type, per-session, monotonically increasing.
  Stored in orchestrator state, checkpointed at superstep boundary.
  Rust: std::sync::atomic::AtomicU64 per SubgraphType in a HashMap.
Design target: PART 6.2 addendum + PART 20 addendum.

DECISION D: Checkpoint Conformance Suite
Status: ADOPTED WITH RUST ADAPTATION
Original: Capability-based test harness for CheckpointStore trait.
WARN-6: Python introspection for capability detection has no Rust equivalent.
Fix: Use marker traits for capabilities:
  trait CheckpointBase { put, get_tuple, list }
  trait CheckpointDelete: CheckpointBase { delete_thread }
  trait CheckpointBranch: CheckpointBase { copy_thread }
  trait CheckpointPrune: CheckpointBase { prune }
  Conformance test macros: conformance_test_base!(MyStore), conformance_test_branch!(MyStore)
  Each macro only compiles if the store implements the required marker trait.
Add COPY_THREAD (branch scenario for what-if) and PRUNE (bounded storage) to CheckpointStore.
Design target: PART 6.4 addendum.

DECISION E: Per-Exception Retry Policy
Status: ADOPTED
No BLOCKs. INFO-1 noted: task.writes.clear() before retry is critical invariant.
Add RetryPolicy { retry_on: fn(ErrorType) -> bool, max_attempts: u32, backoff: BackoffStrategy }
Add to anti-spiral: different responses per failure category (from Aider failure taxonomy, 30.4).
  OOM -> reduce scope immediately (not retry)
  TypeErrors -> fix and retry
  Timeouts -> increase timeout once, then reduce scope
  ImportErrors -> check deps and retry
Design target: PART 6.5 addendum.

DECISION F: Node-Level Cache with TTL
Status: ADOPTED WITH FLOAT GUARD
WARN-2: Floating-point cache keys (NaN, -0.0) cause spurious misses.
Fix: Cache key function normalizes floats: NaN -> sentinel, -0.0 -> 0.0, round to 6 decimals.
  CachePolicy { key_fn: fn(Input) -> CacheKey, ttl_seconds: u64 }
  Checked before agent invocation. Hit -> skip agent, return cached output.
  Separate from checkpoint (cache is perf optimization, checkpoint is correctness).
Design target: PART 6.6 addendum.

DECISION G: PageRank Repo Map
Status: ADOPTED WITH MECHANISM SPLIT
BLOCK-3: Two mechanisms conflated. Edge-weight multiplier (50x) != personalization vector.
WARN-3: Dense cyclic DKG may produce uniform ranks.
Fix: Implement BOTH mechanisms separately:
  1. Edge-weight boost: edges from chat-relevant files get weight * 50.
     This is topology modification, applied to the DKG adjacency matrix.
  2. Personalization vector: task-mentioned identifiers get 10x personalization.
     This biases the random walk starting distribution.
  Binary search for token budget fitting (Aider's approach: +-15% tolerance).
  DKG density guard: if graph density > 0.3, fall back to BM25-only ranking (PageRank degrades).
  Special files list (~100 items: Cargo.toml, Dockerfile, etc.) always included.
Design target: PART 4 addendum.

DECISION H: Sampling-Based Token Estimation
Status: ADOPTED
No BLOCKs. Count tokens on every 10th line, multiply by 10. 95%+ accuracy at 1/10 cost.
Use for: context budget estimation, file size preview in Read tool, repo map fitting.
Design target: PART 3 (providers::LlmProvider::token_count_estimate).

DECISION I: 4-Tier Edit Matching
Status: ADOPTED AS 3-TIER (fuzzy disabled)
WARN-4: Aider's tier 4 (fuzzy SequenceMatcher) is disabled in production.
Fix: 3 tiers only:
  Tier 1: exact match
  Tier 2: whitespace-normalized match
  Tier 3: ellipsis-elision expansion ("..." expands to match any lines)
  Tier 4 (fuzzy): NOT adopted. Risk of accepting wrong edits.
  On failure: surface "did you mean" hint with nearest chunk by similarity.
Design target: tools/ crate edit tool.

DECISION J: ChatChunks Fixed-Slot Context Ordering
Status: ADOPTED
No BLOCKs. Define fixed slot order for context window:
  1. system_prompt (GRC structured prompt per role)
  2. rules (active rules for current phase)
  3. phase_spec (current phase specification)
  4. repo_map (PageRank-ranked DKG output)
  5. code_context (BM25 + vector retrieved chunks)
  6. ssb_entries (relevant SSB history)
  7. grc_history (prior reasoning steps)
  8. [output_reserve: 2000 tokens minimum]
Cache breakpoints at slot boundaries (slots 1-3 are stable across turns, cacheable).
Design target: PART 21 addendum.

DECISION K: Session-Owned Commit Tracking
Status: ADOPTED WITH HANDOFF FIX
WARN-7: Commit hashes must persist across session handoff.
Fix: Add commit_hashes: Vec<String> to handoff.msgpack schema.
  Only commits in session's commit_hashes can be undone.
  Checks before undo: dirty files, parent exists, not pushed to remote.
Design target: PART 7.2 addendum.

DECISION L: Assistant Prefill Continuation
Status: ADOPTED WITH SCOPE LIMIT
WARN-5: Conflicts with "never compact" mandate if applied to reasoning context.
Fix: Scope limited to tools/ crate code generation output ONLY.
  When executor generates code and hits output length limit:
    1. Append partial response as assistant turn
    2. Continue generation in new turn
    3. Merge outputs
  NOT applied to: GRC reasoning, agent context, SSB entries.
  This is output continuation, not context compaction.
Design target: PART 3 (providers::LlmProvider) addendum.

DECISION M: File Reading Tool Enhancement
Status: ADOPTED
No BLOCKs. Novel improvement (neither LangGraph nor Aider does this).
  1. Read tool returns metadata header: "File: {path} | {lines} lines | ~{tokens} tokens"
  2. If lines <= threshold (configurable, default 300): return full content
  3. If lines > threshold AND no range specified: return tree-sitter structural outline
     [lines 1-15: imports] [lines 16-42: struct FooBar] etc.
  4. Add start_line/end_line optional parameters for range reads
  Token estimation uses sampling method (Decision H).
Design target: tools/ crate Read tool, claw-code modification.

DECISION N: RAG Namespace Separation
Status: IMPLEMENTED
BLOCK-4: BM25 global index caused cross-project IDF pollution.
Fix (applied):
  1. Per-project BM25 indexes in server.py (_bm25_per_project dict)
  2. Search uses project-scoped BM25 when project filter set
  3. Paper-ingestor agent updated: papers -> project:paper-<slug>, sources -> project:source-<slug>
  4. Existing contamination cleaned: 5 langgraph files + 4 paper entries removed from project:elai_cc
  5. Papers re-ingested under proper namespaces (paper-daira, paper-shortcoder)
