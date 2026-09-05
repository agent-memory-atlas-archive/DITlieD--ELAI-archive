PROGRESS SUMMARY: Tests: 3/3 passing | Tiers: N/A (plan audit mode, no pipeline) | TestQuality: N/A | Regression: aider-patterns-report.md (mechanism confusion) | BLOCKs: 7 | Score: 2.8/10

Audit type: PLAN AUDIT MODE (design reports and decisions, no code implementation)
Sources verified: LangGraph source at docs/.source/langgraph-main/, aider source at docs/.source/aider-main/
Verification scripts: workflow/audit-scratch/verify_channel_taxonomy.py, verify_bm25_contamination.py, verify_pagerank_claim.py
All three scripts executed and passed.


BLOCK (must fix before proceeding):

BLOCK-1 [Decision B — Channel taxonomy]: UntrackedValue lifecycle misrepresented in source report.
  The report (Pattern 3) states "UntrackedValue: like EphemeralValue but NEVER serialized to checkpoint."
  This is incomplete and misleading. Verified from source:
    EphemeralValue: update([]) branch (line 57-60 in ephemeral_value.py) sets value=MISSING when no writes arrive.
    UntrackedValue: has NO zero-write clearing branch. Value persists until overwritten.
  The lifecycle difference is critical, not cosmetic. If elai_cc uses UntrackedValue expecting automatic
  step-boundary clearing (e.g., for transient model outputs), values will persist across supersteps silently.
  Decision B adopts UntrackedValue for "transient large objects" — if the clearing assumption is baked in,
  this is a silent data corruption vector.
  Fix: before implementing, document explicit clearing semantics. Add test that writes to UntrackedValue
  then runs empty superstep and asserts value still present (not cleared).
  Evidence: workflow/audit-scratch/verify_channel_taxonomy.py — verified against actual source.
  — docs/.source/langgraph-main/.../channels/untracked_value.py, ephemeral_value.py

BLOCK-2 [Decision B — Channel taxonomy]: Report also omits UntrackedValue has guard= parameter.
  UntrackedValue.__init__ accepts guard: bool = True identically to EphemeralValue.
  With guard=True (default), multiple concurrent writes raise InvalidUpdateError.
  Decision B proposes UntrackedValue for "large intermediate objects" where multiple
  agents might write concurrently (e.g., model output pre-postprocessing).
  If guard=True (default), the first concurrent second write will raise, breaking the pipeline.
  Fix: explicitly specify guard=False in the channel taxonomy spec wherever multi-writer is intended.
  — docs/.source/langgraph-main/.../channels/untracked_value.py lines 56-64

BLOCK-3 [Decision G — PageRank DKG boost weights]: Report conflates two distinct mechanisms.
  The report states: "Edge weight boosted 50x if the referencing file is in the current chat."
  This is true for GRAPH EDGE WEIGHTS (repomap.py line 509: use_mul *= 50).
  The report then labels this as "task-aware boost weights for repo map" and Decision G adopts it.
  But Decision G's framing "PageRank over DKG with task-aware boost weights" is ambiguous:
    If implemented via personalization vector only (no edge weight modification):
      The 50x effect is absent. Chat files get uniform 100/N allocation which DECREASES as DKG grows.
      For a 500-node DKG, personalization per chat file = 0.2 — negligible influence.
    If implemented via edge weight modification (the actual aider mechanism):
      The graph structure must be modified at query time — expensive for large DKGs.
  The report never distinguishes between graph-topology signals (edge weights) and
  random-walk restart signals (personalization). These are fundamentally different mechanisms
  with different computational costs and different scaling behaviors.
  Fix: Decision G must explicitly specify whether the implementation is:
    (a) personalization-vector-only (simpler, lower cost, weaker signal at scale),
    (b) edge-weight-modification (stronger signal, requires mutable graph structure),
    (c) both (most faithful to aider, highest complexity).
  An implementation of (a) when (b) was intended will produce measurably wrong repo maps.
  Evidence: workflow/audit-scratch/verify_pagerank_claim.py — verified from repomap.py source.
  — docs/.source/aider-main/.../aider/repomap.py lines 383, 493-514, 519-525

BLOCK-4 [Decision N — RAG namespace fix]: Proposed fix is incomplete and creates false confidence.
  The issue: 5 langgraph .py files and DAIRA/ShortCoder papers are in project=elai_cc namespace.
  Proposed fix: separate project namespaces (separate Qdrant collections per project).
  This fixes the Qdrant vector search path. It does NOT fix the BM25 path.
  Verified from server.py:
    _build_bm25_index() (line 105): builds index over ALL Qdrant documents with no project filter.
    _hybrid_search() (line 232): bm25_scores = _bm25_index.get_scores(tokenized_query) — global index.
    Project filtering only happens at line 250 AFTER BM25 top-K selection.
  Mathematical consequence (verified in verify_bm25_contamination.py):
    IDF('channel') with 5 langgraph contamination docs: 1.07
    IDF('channel') with clean corpus: 3.09
    Cross-project docs consume BM25 top-K slots, then get filtered — elai_cc results starved.
  Fix must include: per-project BM25 index (dict keyed by project) OR rebuild BM25 on each
  project-scoped query (expensive) OR move to Qdrant full-text search with project filter.
  A "separate Qdrant collections" fix without BM25 partitioning leaves the primary contamination
  vector intact and will produce confident-looking but wrong results.
  — C:\Users\user\.claude\rag\server.py lines 105-118, 230-258

BLOCK-5 [Decision A — Deferred node execution]: Adopts defer=True without recognizing the
  Rust translation is non-trivial and the design locks a specific Python execution model.
  LangGraph's defer mechanism relies on apply_writes() calling channel.finish() only when
  updated_channels.isdisjoint(trigger_to_nodes) — i.e., quiescence detection.
  elai_cc's SSB single-writer model does not have a quiescence detection mechanism.
  PART 20's BSP superstep model defines step boundaries at message-passing boundaries,
  not at computation quiescence. These are different termination conditions.
  In BSP, a superstep ends when all agents complete their barrier synchronization,
  not when "no remaining non-deferred work exists."
  Implementing deferred nodes in the BSP model requires either:
    (a) a quiescence detector separate from the BSP barrier (new component, not in design),
    (b) explicit "last superstep" signal from the orchestrator (breaks the deterministic
        BSP invariant that superstep boundary is defined by barrier completion, not by
        externally injected "no more work" signals).
  Decision A was approved based on "eliminates need to enumerate all branch counts at
  graph-build time." This is a real benefit. But the underlying mechanism conflicts with
  the locked BSP+SSB architecture in PART 20. The decision skips the mechanism analysis.
  Fix: before implementing, specify exactly how quiescence detection integrates with
  the SSB single-writer and BSP barrier. Either add quiescence to the design or
  re-evaluate whether deferred execution is achievable in the current architecture.

BLOCK-6 [Decision C — Send/dynamic fan-out]: Adopted without addressing the checkpoint
  namespace collision problem described in Pattern 11 (which is classified HIGH severity).
  Pattern 11 documents that concurrent subgraph executions of the same type require
  numeric disambiguation suffixes in checkpoint_ns.
  elai_cc's PART 24 (Project Isolation Namespace) addresses project-level isolation
  but not task-instance-level isolation within a project.
  If two L2 nodes of the same type each use Send() to spawn L3 subtasks, and both
  spawn the same L3 subgraph type, their checkpoints will collide unless the numeric
  suffix mechanism is implemented.
  Decision C approves Send adoption but the decisions document does not exist
  (workflow/reasoning/phase-*-decisions.md is missing — confirmed by Glob search).
  Without a decisions document, there is no record of which implementation details
  were considered. The namespace collision is a silent data corruption risk.
  Fix: document how Send-spawned task namespaces are disambiguated before implementing.
  The LangGraph scratchpad counter approach (numeric suffix appended to checkpoint_ns)
  needs a Rust equivalent in the orchestrator.

BLOCK-7 [Missing phase decisions document]: workflow/reasoning/phase-*-decisions.md does not exist.
  This is required by CLAUDE.md hard rules: "before EXECUTING, write phase-X-decisions.md."
  The auditor instructions classify this as WARN when missing, but the scale of decisions
  being made here (13 architectural decisions spanning multiple subsystems) combined with
  the absence of ANY decisions document elevates this to BLOCK.
  These 13 decisions modify: channel taxonomy (locked in PART 20), checkpoint architecture,
  the DKG, the context budget allocator, git integration, and the tools/ crate interface.
  Making 13 concurrent architectural changes without a written decision record violates
  the "one change at a time" hard rule and the decisions document requirement.
  Fix: write workflow/reasoning/phase-6-decisions.md before implementing any of A-N.
  Each decision should state: what, key decision rationale, expected test outcome, unknowns.


WARN (should fix, document justification if skipped):

WARN-1 [Decision B — Channel taxonomy]: Decision expands channel taxonomy from 2 to 5 types.
  PART 20.4 testable invariants explicitly state: "Only LastValue and Topic channel types exist."
  Adding UntrackedValue, EphemeralValue, NamedBarrierValue modifies a LOCKED invariant.
  The design document (PART 20) has testable invariants that will fail if new channel types
  are added without updating those invariants.
  Fix: before implementing, update PART 20.4 invariants and note what existing tests
  would need to change. The "Only LastValue and Topic" invariant was deliberately locked
  after the BinaryOp removal (BLOCK 4 in PART 17). Adding new types reopens that discussion.

WARN-2 [Decision F — Node-level cache with TTL]: The report correctly identifies CachePolicy
  but does not address the cache invalidation problem.
  LangGraph's default key_func uses pickle+xxhash on the node input. For elai_cc indicator
  nodes using bar data (OHLCV), the "same input" assumption requires that bar data objects
  produce identical hashes across serialization boundaries. Rust structs with f64 fields
  are subject to NaN and -0.0 representation issues that would produce cache misses
  when mathematically equivalent inputs exist.
  Fix: document the hash function for cache keys. Floating-point input normalization
  (NaN canonical form, -0.0 -> 0.0) is required before hashing indicator inputs.

WARN-3 [Decision G — PageRank for DKG]: The DKG in elai_cc is knowledge-graph edges
  (concept->concept, concept->file, file->file). Aider's PageRank operates on a
  MultiDiGraph where nodes are FILES and edges are SYMBOL REFERENCES.
  These are different graph semantics. File reference graphs are naturally sparse and
  DAG-like. Knowledge graphs can have dense cycles (A->B->C->A for related concepts).
  PageRank on dense cyclic graphs produces more uniform rank distributions — the
  signal-to-noise ratio degrades with graph density.
  Fix: before implementing, measure the edge density of the DKG for a sample project
  and verify that PageRank produces meaningful rank separation (not uniform distribution).

WARN-4 [Decision H — 4-tier edit matching]: Tier 4 (fuzzy SequenceMatcher at threshold 0.8)
  is DISABLED in aider production (report correctly notes "disabled in prod").
  The report includes it in the "4-tier" count. Decision H adopts "4-tier edit matching"
  but the fourth tier is disabled. If elai_cc implements all four tiers and enables
  the fuzzy tier, it will diverge from aider's production behavior and risk incorrect
  edits (SequenceMatcher at 0.8 will accept edits that are "close enough" but wrong).
  Fix: either adopt 3-tier matching (with the fuzzy tier as a compile-time feature flag
  disabled by default) or explicitly justify enabling the fuzzy tier.

WARN-5 [Decision L — Assistant prefill continuation]: The report describes aider's prefill
  as handling "FinishReasonLength" (context exhaustion mid-generation).
  elai_cc's hard rule is "NEVER compact, FINISH the task before running out of context,
  new session if exhausted." This is an explicit architectural commitment from the user.
  Decision L (prefill continuation for long outputs) is in direct tension with this rule.
  Prefill continuation enables indefinite generation by appending partial responses and
  continuing — this IS a form of compaction at the response level.
  Fix: clarify whether Decision L applies only to tool-use crate generation (where
  the output is code/files, not session context) or to the agent's own reasoning context.
  If the latter, this conflicts with the user's explicit "never compact" mandate.

WARN-6 [Decision D — Checkpoint conformance suite]: The LangGraph conformance suite uses
  capability detection via method override checking (DetectedCapabilities.from_instance).
  The report proposes "check if trait method is non-default" for Rust.
  Rust does not support runtime introspection of whether a trait method is the default
  implementation. There is no equivalent of Python's __dict__ method override detection.
  Fix: the Rust conformance suite must use a different detection mechanism:
    (a) explicit capability enum reported by each impl (requires changing CheckpointStore trait),
    (b) marker traits (SupportsDelete, SupportsPrune, etc.),
    (c) test runner that always runs all tests and panics on unimplemented!().
  Option (c) is simplest and most honest. The Python "auto-detect" pattern does not
  translate to Rust without trait changes.

WARN-7 [Decision K — Session-owned commit tracking]: Aider's undo safety check verifies
  "last commit hash is in coder.aider_commit_hashes (session-owned commits only)."
  elai_cc's FSM already has a session handoff (handoff.msgpack) but there is no
  mention of a commit hash registry in the handoff spec (PART 6.5 agent checkpointing).
  If commit hashes are not persisted in the handoff, session-owned commit tracking
  cannot survive session boundaries — the safety check becomes meaningless after restart.
  Fix: add commit_hashes: Vec<String> to the agent checkpoint schema (handoff.msgpack).


INFO (noted, no action required):

INFO-1 [LangGraph report — Pattern 6 Runtime context_schema]: This pattern is directly
  relevant and correctly analyzed. The mapping (broker_config/api_keys -> context,
  portfolio_state -> state) is sound. The separation of immutable-per-run config from
  evolving state is a clean architectural improvement. No implementation risk beyond
  restructuring the FSM config schema.

INFO-2 [LangGraph report — Pattern 8 Durability modes]: Correctly marked LOW severity.
  The async durability optimization (hide checkpoint latency behind computation) is
  a performance optimization, not a correctness issue. Not needed until L3 node throughput
  is measured and shown to be checkpoint-bound. Appropriate deferral.

INFO-3 [LangGraph report — Pattern 12 Trigger-to-nodes index]: Correctly marked LOW severity.
  The O(N) -> O(updated_channels * fanout) optimization is premature at design phase.
  Mark for implementation phase if profiling shows FSM evaluation is a bottleneck.

INFO-4 [Aider report — Pattern I Fence collision detection]: Correctly identified as novel.
  Implementation risk is low (it is a pure string scan before prompt assembly).
  For elai_cc, the equivalent is relevant when generating code that contains backticks
  or code block markers. Low-risk addition to the tools/ crate.

INFO-5 [Aider report — Pattern C Sampling-based token estimation]: Report description is
  accurate. The actual code (repomap.py lines 89-101) uses step = num_lines // 100 with
  extrapolation from sample bytes ratio. The extrapolation uses byte ratio (not token ratio)
  as the scale factor, which introduces error for code with high unicode density. For
  elai_cc's DKG files (primarily Rust/Python source), this error is acceptable.

INFO-6 [RAG architecture]: The single-collection Qdrant design with project payload filter
  is a reasonable starting point. The mcp_list_topics tool already shows project-tagged
  entries. The contamination issue (BLOCK-4) is fixable without a full architectural rewrite.
  The BM25 partitioning can be implemented as dict[str, BM25Okapi] keyed by project,
  with rebuild triggered per mcp_ingest_file call. This is a contained change.

INFO-7 [Decision E — Per-exception retry policy]: Correctly analyzed. The "clear writes
  before retry" invariant (task.writes.clear() in _retry.py) is the most important
  correctness detail. Without this, retry attempts accumulate writes from previous
  attempts and produce corrupted state. The Rust equivalent must explicitly clear
  the task's pending writes before each retry attempt. Include as a testable invariant.

INFO-8 [Aider report — ShortCoder rule mapping discrepancy]: The aider report does not
  reference ShortCoder. But PART 16.2 maps ShortCoder rules to its own internal labeling
  (R1-R10) that differs from the paper's numbering. The DAIRA interface file uses the
  paper's rule numbers. This naming inconsistency will cause confusion when both
  integrations are active. Not a blocker but should be normalized.


DECISION VERDICTS:

A. Deferred node execution: BLOCK-5. Mechanism conflicts with BSP+SSB architecture. Fix required before implementing.
B. Channel taxonomy expansion (5 types): BLOCK-1, BLOCK-2, WARN-1. Misrepresented lifecycle; guard omission; locked invariant violation.
C. Send/dynamic fan-out: BLOCK-6. Namespace collision unresolved. Decisions document missing.
D. Checkpoint conformance suite: WARN-6. Rust capability detection mechanism unspecified.
E. Per-exception retry policy: INFO-7. Sound adoption. Add "clear writes on retry" as testable invariant.
F. Node-level cache with TTL: WARN-2. Hash function for floating-point inputs unspecified.
G. PageRank DKG with boost weights: BLOCK-3, WARN-3. Mechanism conflated; graph density unvalidated.
H. 4-tier edit matching: WARN-4. Tier 4 disabled in production; adoption must be 3-tier or explicitly gated.
I. Sampling-based token estimation: INFO-5. Sound. Minor byte-vs-token extrapolation error acceptable.
J. ChatChunks fixed-slot context ordering: SOUND. Report correctly describes actual code. Low-risk adoption.
K. Session-owned commit tracking: WARN-7. Commit hash persistence in handoff.msgpack not specified.
L. Assistant prefill continuation: WARN-5. Conflicts with "never compact" architectural mandate. Scope must be clarified.
M. File reading tool fix (tree-sitter outline + metadata + start_line/end_line): SOUND. The context-package already identified offset+limit as the primary gap. The tree-sitter outline on large files is a useful addition but requires specifying the outline format (function signatures only? with line numbers?). Low risk. The FileReadResult struct proposed in the context-package is the right interface.
N. RAG namespace separation: BLOCK-4. Proposed fix incomplete — BM25 index not partitioned.
