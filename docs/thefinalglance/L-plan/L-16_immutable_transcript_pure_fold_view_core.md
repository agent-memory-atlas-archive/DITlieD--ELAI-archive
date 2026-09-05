L-16 — IMMUTABLE TRANSCRIPT + PURE FOLD-VIEW CONTEXT CORE
=========================================================
Track: L-T3 CONTEXT-CORE (L-16..L-19). Delta family: CC-1..CC-4 (defined below, registered in L-0_INDEX).
Nature: BUILD [KEYSTONE, net-new core invariant] + binding REVISION to the conversation-runtime compaction
  contract (the mutate-then-recover model is replaced, not extended).
Lever: make HD-2 cache-stability and lossless recovery STRUCTURAL properties of a pure projection, instead of
  behaviours enforced by separate mechanisms that the running loop can violate.
Wave: W1 (the track foundation; L-17/L-18/L-19 all parameterize this projection).
Composes-with: 3E context_budget.rs (the budget parameter), H-50 context_policy.rs (the policy parameter),
  H-49 session_dag.rs + the CAS blake3+zstd store (the log store + snapshot grain), I-11 belief_state.rs
  (a projection variant for cross-tier handoff), K-12 compact.rs:545 (the losslessness gate, now view-level),
  E-25 compaction_ladder.rs (a projection budget strategy, no longer a mutator).
Authored 2026-06-30, main LLM in-band, spec-eval AS-SPECIFIED. New plan file per the house rule; the REVISION
  below is binding on conversation.rs::compact_session + the recover_stubs call-site and is registered in
  L-0_INDEX. Reference design (external prior art, source-read 2026-06-30): Context Warp Drive
  (context-warp-drive-main), FoldSession.prepare / foldFreeze.ts / rollingFold.ts.

ORIGIN — WHY THIS FILE EXISTS
=============================
A design comparison against Context Warp Drive (a shipped, zero-LLM, deterministic message-array folding
library, measured 89.6/93.2% provider cache-read on long agent sessions) found that ELAI's specified context
stack already covers ~90% of CWD feature-for-feature: verbatim-atom conservation is the K-12 critical-atom
gate, ambient page-in is the H-6 read-back, the rebirth seed is the I-11 typed BeliefState, the model budget
table is the 3E allocator, episodic recall is the E-5 memory substrate. CWD has exactly ONE architectural idea
ELAI's spec does not contain even when fully built: it never mutates the raw history. CWD treats the raw
append-only message log as permanent ground truth and the provider-facing array as a PURE deterministic
function of it (FoldSession.prepare returns a view, never a mutation; identical inputs produce byte-identical
output).

ELAI's running model is the opposite: compact_session (conversation.rs) rewrites the message-array prefix into
a System summary block, and recoverability is bolted on by a second system (the DAG snapshot + SSB stub +
blake3 verify in recover_stubs at session_dag.rs:1539, the check_lossless_compaction gate at compact.rs:545,
H-49 edit_checkpoints). That is "destroy then restore". CWD is "never destroy, project". The inversion is
strictly cleaner and it makes several already-specified guarantees fall out for free. This file adopts the
inversion as ELAI's context-core invariant. The three sibling sessions then re-express the freeze controller
(L-17), the losslessness + time-travel corollaries (L-18), and the page-in budget ladder (L-19) as
parameterizations of this one projection.

CC DELTA FAMILY (new blocking rules this track adds)
====================================================
CC-1 RAW IS IMMUTABLE. No context-shaping operation may mutate or delete an entry from the append-only
  transcript log. The log is the single source of truth; compaction, masking, trimming, and handoff all read
  it and never write back into it. (binds L-16, L-18)
CC-2 VIEW IS PURE. The provider-facing message array is a deterministic pure function fold(log, budget, policy,
  freeze) -> view. Identical (log, budget, policy, freeze) inputs MUST produce a byte-identical view. A view
  computation that reads any ambient/mutable state (wall clock, RNG, process-global) outside its declared
  inputs is a CC-2 violation. (binds L-16, L-17)
CC-3 PREFIX MOVES ONLY ON A NAMED CAUSE. The view's cached prefix bytes may change only at a freeze epoch
  carrying one of the enumerated recompute causes (L-17), and the cause MUST be journaled. An unexplained
  prefix-byte change is a hard gate failure (it is the cache-cold event HD-2 exists to forbid). (binds L-17)
CC-4 LOSSLESS BY CONSTRUCTION. Because the log is never destroyed, "lossless" reduces to a view-level
  assertion: the projection preserves the declared critical atoms (K-12). The archive-before-drop gate is
  SUBSUMED by the immutability invariant, not deleted: nothing is dropped from ground truth, so there is
  nothing to fail-close on. (binds L-18)

BUILD
=====
1. THE LOG. Promote the append-only transcript to the canonical store. The SessionDag (session_dag.rs) +
   the H-49 CAS blake3+zstd object store already give an append-only, content-addressed, snapshot-grained
   record; this session declares it the SINGLE SOURCE OF TRUTH for context, not a recovery side-channel. Every
   ConversationMessage is appended; nothing is ever rewritten or removed from it. The CAS dedup keeps the cost
   bounded (identical tool outputs store once).
2. THE PROJECTION. Introduce ContextProjection: a pure function
     fold(log: &TranscriptLog, budget: &ContextBudget, policy: &ContextPolicy, freeze: &FreezeState) -> View
   that computes the provider-facing message array WITHOUT touching the log. The View is what gets sent to the
   model. All existing reducers become STRATEGIES inside fold, selected by policy:
     - the E-25 ladder tiers become budget strategies (which spans to skeletonize, at what granularity),
     - H-6 deterministic masking becomes the trailing-observation strategy (drop trailing tool noise from the
       VIEW, never from the log),
     - the 3E allocator decides per-slot budget the projection must hit,
     - H-50 ContextPolicy picks the whole strategy set by BackendMode (cloud = cache-stable prefix + late
       compaction; local = window-minimization + unload-early).
3. THE REPLACEMENT. conversation.rs::compact_session is re-specified: instead of building a new messages vector
   and assigning session.messages, the runtime holds the immutable log and asks fold() for the current view on
   each turn. "Compaction" is no longer an event that happens to the session; it is the steady-state output of
   the projection at the current budget. recover_stubs at the live call-site becomes "recompute the view with a
   larger budget for the requested span" — no SSB round-trip, no blake3 verify, because the content was never
   gone. (recover_stubs stays for legacy DAG-snapshot reads; the new path does not need it.)
4. STRUCTURAL HD-2. Because fold is pure and the log prefix is immutable, the view's prefix is a pure function
   of an unchanging input and therefore cannot move between turns unless budget/policy/freeze change. HD-2
   ("never bust the byte-stable cache prefix") stops being a doctrine the loop can violate and becomes a type
   property of the projection. The L-17 freeze controller makes the legal prefix-change causes explicit.

COMPOSES-WITH (do NOT re-author any)
====================================
- 3E context_budget.rs ContextBudget / the 10-slot allocator: the budget PARAMETER of fold. Unchanged; fold
  consumes its per-slot targets.
- H-50 context_policy.rs ContextPolicy + BackendMode: the policy PARAMETER. The cloud/local-opposite selection
  is what makes the SAME projection correct on both lanes (cloud keeps the prefix to stay cache-hot; local
  minimizes the window and unloads, where there is no provider cache to preserve).
- H-49 session_dag.rs + CAS store: the immutable LOG. The undo-tree / branch grain is reused at L-18.
- K-12 compact.rs:545 check_lossless_compaction: re-pointed at the VIEW (CC-4) — assert critical atoms present
  in the projection, not "was the dropped message archived".
- I-11 belief_state.rs: a projection VARIANT for cross-tier handoff (a fold whose budget is the typed
  BeliefState shape rather than the full window).
- E-25 compaction_ladder.rs: its tier order becomes a budget STRATEGY inside fold; the ladder stops being a
  separate (and today simulated) mutator.

INTEGRATION
===========
crates/orchestrator/src/context_projection.rs (new): the ContextProjection fold function + the View type +
the TranscriptLog read interface over SessionDag/CAS. Wire conversation.rs to hold the log and call fold() per
turn in place of the compact_session mutation; re-point the live recover_stubs call-site (live_cli.rs) to the
recompute-with-larger-budget path. SSB: prefer-reuse the existing ContextReduction row (a projection at a
tighter budget is a context_reduction); optional +1 ContextProjected (Pattern-2; Kind: ProjectedHotReuse /
ProjectedRecomputed) only if the freeze audit trail (L-17) needs its own variant, decide at edit (re-grep the
live SCHEMA_VERSION + TOTAL_VARIANTS at ssb.rs + action_type_lock.rs first; never hard-code). frontend_pair:
the per-regime context gauge (reuse H-50's) gains a "view: hot-reuse | recomputed" badge; or justified N/A.
Default-ON behind ELAI_CONTEXT_PROJECTION_DISABLE (kill-switch; the mutate path stays available under the flag
for one release as the rollback, then is retired by a later session).

OBSERVABLE_EFFECT (the done-gate; W-1 runtime leg)
==================================================
Drive the real conversation runtime on a long multi-turn session that crosses the compaction budget twice.
PASS: the provider-facing view is byte-identical across turns that did not cross a freeze epoch (CC-2, a real
two-turn cloud run shows turn-2 cache_read covering the unchanged prefix); a span that was skeletonized in turn
N is reconstructable verbatim in turn N+k by recomputing the view at a larger budget, with no SSB read and no
content loss (CC-1 + CC-4); the raw transcript log byte-length is monotonic non-decreasing across the whole
session (CC-1 — nothing was ever removed from ground truth). NEGATIVE: a deliberately injected mutation of the
log (simulating the old rewrite path) is rejected / flagged, never silently accepted (CC-1); a fold() that
reads the wall clock and therefore produces a non-reproducible view fails the byte-identity assert (CC-2); a
prefix-byte change with no journaled recompute cause is a hard fail (CC-3, proven at L-17). Under
ELAI_CONTEXT_PROJECTION_DISABLE the legacy mutate path runs and the byte-identity assert is recorded-not-
enforced (observe-only rollback).

GROUNDING ANCHORS
=================
- CWD (external reference impl, source-read 2026-06-30): FoldSession.prepare (pure-view contract, append-only
  raw history, byte-identical output), foldFreeze.ts (the freeze state), rollingFold.ts (the fold strategies),
  measured 89.6/93.2% cache-read in its README — the evidence the invariant pays off exactly as HD-2 predicts.
- ELAI live anchors (re-grep at edit, W-7): conversation.rs::compact_session (the mutate path being replaced),
  session_dag.rs:1539 recover_stubs (the recovery path being subsumed), compact.rs:545
  check_lossless_compaction (re-pointed at the view per CC-4), context_budget.rs (the budget param),
  context_policy.rs (the policy param), compaction_ladder.rs (the tier strategies).
- Pattern: event-sourcing / CQRS read-model projection (established) — the transcript is the event log, the
  window is a recomputable projection. ELAI-measured byte-identity + cache-read are the done-gate, never an
  external number (ID-5).
