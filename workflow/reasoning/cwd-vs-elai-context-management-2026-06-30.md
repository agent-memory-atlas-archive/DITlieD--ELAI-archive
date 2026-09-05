CONTEXT WARP DRIVE vs ELAI CONTEXT MANAGEMENT — detailed comparison
Date: 2026-06-30
Mode: spec-eval (AS-SPECIFIED; design verdict separate from build-readiness)
ELAI side grounded ONLY from the plan-summary index files (no code read for this doc):
  docs/specs/guide/steps/phase-index-MASTER.txt (3E, 3H, 3J)
  docs/thefinalglance/ALL-PLANS-MASTER-SUMMARY.md
  docs/thefinalglance/{E,H,I,J,K}-plan/*-SUMMARY_INDEX.md
CWD side grounded from the context-warp-drive-main source/README exploration (the comparator).


0. THE ONE-LINE ANSWER
----------------------
Not a single winner — a scope split. CWD is a narrow, fully-built, MEASURED, zero-LLM in-session
message-array folding ENGINE optimized for one objective: keep a cloud provider's prompt cache hot
across a long function-calling session. ELAI is a context LIFECYCLE PLATFORM (in-session +
cross-session + cross-model/tier + reversibility + memory + measurement) that is broader and, on
losslessness and handoff, stronger by design — but its in-session cache-preserving leg is partly
designed-not-running, and unlike CWD it has no measured cache number yet.

For "keep one cloud session cache-hot, deterministically, today" -> CWD's design is tighter and it is
the only one of the two that ships+measures it.
For "every task always has the right context across small AND cloud models, no bloat, recoverable,
across sessions and model swaps" -> ELAI's design is a strict superset; CWD answers ~a third of it.

CWD is therefore best read as a reference implementation / steal target for ELAI's in-session
cache-preserving leg (the H-6 / H-50 / I-10 / K-12 cluster), explicitly cloud-lane-only — not a rival.


1. WHAT EACH SYSTEM IS
----------------------
CONTEXT WARP DRIVE (CWD)
  A single-purpose TypeScript library (zero runtime deps; better-sqlite3 optional). It folds the
  provider-shaped message array of a function-calling agent so a long session stays in-window WITHOUT
  any LLM summarization call, deterministically, with byte-identical output so the provider prompt
  cache stays warm. Six composed layers: Rolling Fold (page-out to tool skeletons + budgeted assistant
  text), Coordinate Closet (verbatim identifier conservation), Fold Freeze (byte-identical cache-hot
  prefix reuse between epochs), Fold Recall (ambient page-in cards on re-touch), Hard-Epoch Raw Rebirth
  Seed (deterministic single-message continuity seed at a pressure ceiling), Episodic Recall (durable
  cross-session SQLite, path-keyed). Hard scope limit (its own README): function-calling APIs ONLY —
  it folds the message array you control. Measured in its README: 89.6% / 93.2% cache-read on long
  production agents. Maturity: shipped, unpublished (install-from-source).

ELAI CONTEXT MANAGEMENT (the 1.5 CONTEXT-LIFECYCLE stack)
  A multi-subsystem context lifecycle, authored across base phase 3 and plan families E/H/I/J/K. Not
  one engine — a cooperating set: a fixed-slot budget allocator, a DAG-snapshot session handoff with
  cryptographically-recoverable stubs, deterministic observation masking, a regime-aware policy that
  runs OPPOSITE strategies for cloud vs local, reversibility/time-travel, typed cross-model handoff,
  user-memory substrate, and a planned measurement layer that replaces folklore thresholds with a
  measured per-model degradation knee. Maturity: mixed — allocator + handoff + recovery + typed
  handoff built/wired; masking, ladder, regime policy, and the measurement bench range
  partial->planned (see readiness note).


2. MECHANISM-BY-MECHANISM MAP (CWD layer -> ELAI equivalent, with plan IDs from the summaries)
----------------------------------------------------------------------------------------------
CWD Fold Freeze (byte-identical cache-hot prefix; epoch only on TTL/tail-overflow/boundary-change)
  -> ELAI 3H "system / rules / phase-spec never stubbed (cache prefix preservation)"
     (phase-index-MASTER 3H) + HD-2 doctrine "never bust the byte-stable cache prefix; masking drops
     trailing observations, never rewrites the prefix" (H-SUMMARY 477; ALL-PLANS HD-2)
     + I-10 "prefix invariance + TTL tiering + cache-hit hard guard" Track-14 keystone, with a
     cache-economics planner choosing TTL from journal inter-call gaps (I-SUMMARY I-10)
     + H-50 cloud policy = "big cache-stable prefix + late compaction" (H-SUMMARY H-50).
  Verdict: SAME PRINCIPLE on both sides. CWD ships+measures it as one engine; ELAI spreads it across
  doctrine (HD-2) + a cache-economics layer (I-10) + a regime policy (H-50), with the actively-running
  compaction path not yet the cache-safe one (readiness note).

CWD Coordinate Closet (auto-extract + conserve verbatim IDs/paths/values inside the fold)
  -> ELAI K-12 "handoff-faithfulness gate: critical_atoms{kind,value,must_preserve} + exact
     string/symbol presence-assert after any handoff/trim, missing=FAIL" (K-SUMMARY K-12)
     + I-14 deterministic context compression with reversible receipts, ciphers banned (feature-map 1.5).
  Verdict: SAME IDEA (conserve exact values across compaction, never semantic similarity). CWD has it
  running as a regex extractor; ELAI's is a stronger FAIL-CLOSED gate but is PLANNED(K-12).

CWD Rolling Fold (graduated page-out: full text -> essence -> tool skeleton, newest-first budget)
  -> ELAI E-25 6-tier compaction ladder (BudgetReduction/Snip/Microcompact/ContextCollapse/
     AutoCompact/NarrativeSummary, cheapest-first) (E-SUMMARY E-25; ALL-PLANS E-25)
     + H-6 deterministic observation masking (make adaptive_reduce the default long-session manager,
     un-gate the E-25 ladder) (H-SUMMARY H-6).
  Verdict: SAME SHAPE (graduated cheapest-first reduction). CWD's fold actually transforms content;
  ELAI's ladder is the design analog but its activation is gated (readiness note).

CWD Fold Recall (ambient page-in: re-touch a folded path -> recall card rides the cache tail)
  -> ELAI H-6 memory read-back + selective recover_stubs ("/continue rehydrates real content via
     recover_stubs, not the placeholder") (H-SUMMARY H-6) + F-5 memory read + I-5 pristine context
     injector (rebuild branch context from a compressed contrastive constraint, measured A/B)
     (I-SUMMARY I-5).
  Verdict: SAME IDEA (fold away, page back when relevant). ELAI's recover_stubs is integrity-verified
  (stronger); CWD's recall is append-only cards keyed on tool re-touch (simpler, shipped).

CWD Raw Rebirth Seed (deterministic single-message continuity seed at pressure ceiling)
  -> ELAI I-11 PABU typed BeliefState handoff (tried / failed-because / current-AST-state /
     constraints, DKG-keyed, measured vs full-history replay) (I-SUMMARY I-11) + 3H DAG session
     handoff (trigger at 85% token usage) (phase-index-MASTER 3H) + H-51 cross-session sleep-time
     digest (H-SUMMARY H-51).
  Verdict: ELAI strictly richer. CWD's seed is one deterministic text blob for a single in-process
  session; ELAI's is a typed, versioned, DKG-keyed object usable across model/tier/session boundaries.

CWD Episodic Recall (durable cross-session SQLite, path-keyed)
  -> ELAI E-1 infinite-context chat (A-MEM + MemGPT digest) (E-SUMMARY E-1) + E-5 user modeling &
     memory (Mem0 ADD/UPDATE/DELETE/NOOP + crates/memory) (E-SUMMARY E-5) + the H-31/H-32 memory tiers.
  Verdict: SAME CATEGORY; ELAI's is a governed bi-temporal memory substrate, CWD's is recency-ordered
  path-keyed recall.

CWD model-aware budgets + Anthropic cache breakpoints (1h prefix / 5m tail TTL)
  -> ELAI 3E context budget allocator: 8 fixed slots (now 10 per feature-map: +Memory +Evidence),
     floors + steal, 2 cache breakpoints (xxhash prefix), 3-level adaptive reduction <50ms
     (phase-index-MASTER 3E) + ModelCapabilityProfile + I-10 TTL tiering.
  Verdict: ELAI's is a multi-class allocator (System/Rules/PhaseSpec/RepoMap/CodeContext/SsbEntries/
     GrcHistory/OutputReserve/Memory/Evidence). CWD only budgets assistant text WITHIN a fold.

CWD has NO analog for:
  -> ELAI H-49 reversibility / time-travel: undo/redo/branching file time-travel + lossless paged
     session history, CAS blake3+zstd store, undo-tree HEAD over SessionDag, snapshot-restore (works on
     the cloud lane) (H-SUMMARY H-49; ALL-PLANS H-49).
  -> ELAI H-50 regime split: cloud and local need OPPOSITE policies; CWD is single-policy cloud-only
     (H-SUMMARY H-50).
  -> ELAI J-34 empirical context calibration: measure the per-(model,lane) fill->verified-outcome
     degradation KNEE + positional-value curve, replace folklore CLOUD_TRIM_FRACTION=0.85 (J-SUMMARY
     J-34; K-SUMMARY K-17). CWD has nothing here (but see "where CWD is more honest" below).
  -> ELAI cryptographically-verified lossless recovery: 3H stubs recoverable from SSB seq +
     K-12 fail-closed losslessness gate. CWD's fold is intentionally lossy.


3. AXIS-BY-AXIS VERDICT
-----------------------
Axis                                                  Winner   Grounding
Keep a cloud provider prompt cache hot in-session     CWD      CWD freezes a byte-identical prefix and
                                                               measures 89.6/93.2% cache-read; ELAI's
                                                               HD-2/I-10/H-50 design matches it but the
                                                               cache-safe path is not the running one yet.
Graduated in-session compaction that ships AND runs   CWD      CWD's fold runs+measured; ELAI E-25 ladder
                                                               + H-6 masking are the design analog, gated.
Verbatim critical-value conservation across compaction TIE     CWD Coordinate Closet (shipped) vs ELAI
                                                               K-12 critical_atoms gate (stronger, planned).
Provable / recoverable losslessness                   ELAI     3H SSB-seq stub recovery + K-12 fail-closed;
                                                               CWD fold is lossy by design.
Cross-model / cross-tier handoff                      ELAI     I-11 typed DKG-keyed BeliefState; CWD has
                                                               only a single-process text seed.
Cross-session handoff (curated digest)                ELAI     H-51 sleep-time topic-digest + auto-detect;
                                                               CWD episodic recall is path-keyed lookups.
Local models (no provider KV cache to preserve)       ELAI     H-50 window-minimization + unload-early;
                                                               CWD's whole premise (cloud prefix cache)
                                                               does not apply locally.
Reversibility / undo / branch / time-travel           ELAI     H-49; CWD has none.
Multi-class context budgeting                         ELAI     3E 10-slot allocator; CWD budgets only
                                                               assistant text inside a fold.
Honesty of the cache claim it makes                   CWD      CWD measures cache-read directly; ELAI's
                                                               thresholds are still folklore until J-34
                                                               measures the knee (K-17 BENCH-PENDING).
Maturity (does the context system run end-to-end now) CWD      CWD is shipped+measured; ELAI's leg is a
                                                               mix of wired + gated + planned.
Breadth / completeness of the context lifecycle       ELAI     ELAI spans 6 categories CWD never enters.


4. WHERE CWD IS GENUINELY BETTER (legitimate design credit, not just maturity)
------------------------------------------------------------------------------
4.1 The byte-identical-freeze discipline is its native, measured property. CWD's entire architecture is
    organized so identical inputs produce byte-identical folds, which is exactly what makes provider
    cache hits possible. ELAI states the same principle (HD-2) and plans the machinery (I-10, H-50), but
    holds it as doctrine+plan rather than as the single invariant the running loop is built around.
4.2 Coordinate Closet conserves exact identifiers automatically INSIDE the fold, every fold, today.
    ELAI's equivalent (K-12 critical-atom gate) is the stronger idea (FAIL-CLOSED, must_preserve) but
    is PLANNED.
4.3 It is honest about its number. CWD leads with a measured cache-read rate. ELAI's own K-native wave
    (K-17) flags that until J-34 measures the per-(model,lane) knee, the whole stack optimizes against
    GUESSED thresholds (CLOUD_TRIM_FRACTION=0.85, the slot order). On the measurement-discipline axis,
    CWD is ahead of ELAI on precisely the thing ELAI admits it hasn't closed.
4.4 Zero-LLM, zero-dependency, deterministic, drop-in. As an engine it is small and verifiable; ELAI's
    equivalent capability is distributed across many crates and plan families.


5. WHERE ELAI IS GENUINELY BETTER (by design, AS-SPECIFIED)
----------------------------------------------------------
5.1 Recoverable losslessness. 3H stubs are recoverable from an SSB seq, and K-12 turns "lossless on
    what matters" into a fail-closed gate. CWD's fold is lossy; its only durable backstop is the
    path-keyed SQLite episode store. ELAI can restore any dropped message; CWD cannot.
5.2 Regime awareness. H-50's central insight — cloud and local need OPPOSITE policies (cache-stable +
    late compaction vs window-min + unload-early) — is an axis CWD does not have. CWD is a cloud
    prefix-cache optimizer; it offers nothing for the local lane, which is the half of "small + cloud
    models" ELAI already owns.
5.3 Cross-boundary handoff. I-11 typed BeliefState (DKG-keyed, measured vs full-history replay) +
    H-51 cross-session digest address model->model, tier->tier, and session->session handoff. CWD's
    rebirth seed is single-process only.
5.4 Reversibility / time-travel. H-49 (undo/redo/branch, CAS blake3+zstd, snapshot-restore on the cloud
    lane). CWD has no concept of it.
5.5 Multi-class budgeting. 3E arbitrates 10 competing context classes under one ceiling with floors and
    steal. CWD budgets a single class (assistant text within a fold).
5.6 Measurement as a planned first-class layer. J-34 / K-17 make "the threshold is the measured knee,
    not folklore" an explicit deliverable. Once built, this exceeds CWD's single-metric honesty by
    measuring degradation-vs-outcome per model+lane, not just cache-read.


6. THE SYNTHESIS — COMPLEMENTARY, NOT COMPETITIVE
-------------------------------------------------
The cleanest framing: CWD is what ELAI's in-session cache-preserving leg looks like when it is
finished, deterministic, and measured. Each CWD layer maps to a specific ELAI plan that is either
doctrine-only or planned:
  CWD Fold Freeze       == ELAI HD-2 + I-10 + H-50 cloud policy   (design present, running path not it)
  CWD Coordinate Closet == ELAI K-12 critical-atom gate           (planned)
  CWD Rolling Fold      == ELAI E-25 ladder + H-6 masking         (gated/partial)
  CWD Fold Recall       == ELAI H-6 read-back + recover_stubs      (gated, but recovery is stronger)
  CWD Rebirth Seed      == ELAI I-11 BeliefState + 3H + H-51       (ELAI richer, partly built)
  CWD Episodic Recall   == ELAI E-1 + E-5 memory                   (ELAI richer)

They also compose at the losslessness level: CWD's cache-safe lossy fold + ELAI's blake3 SSB recovery
would give BOTH a hot cache AND provable recovery — a combination neither system has alone today.

Bounded steal opportunity (if pursued): port CWD's Fold-Freeze + Coordinate-Closet discipline into
ELAI's cloud-lane overflow path — i.e. the H-6 / H-50 / I-10 / K-12 cluster — SCOPED TO THE CLOUD LANE
ONLY (CWD's premise does not transfer to the local HF sidecar, which ELAI's H-50 window-minimization
already handles). That converts ELAI's "design present, not the running path" status on the
cache-preserving leg into a shipped+measured one, using CWD as the reference implementation.


7. IMPLEMENTATION-READINESS NOTE (sequencing only — NOT part of the design verdict; status is
   index/feature-map-derived, confirm against live code)
-----------------------------------------------------------------------------------------------------
- CWD: shipped + measured, unpublished (install-from-source). FC-APIs only.
- ELAI built/wired: 3E allocator (10 slots), 3H session handoff + recover_stubs, I-11 BeliefState,
  H-49 reversibility primitives, H-50 context_policy wired into the live CLI (per feature-map the
  "PARTIAL/uncommitted" labels are flagged STALE).
- ELAI gated/partial: E-25 ladder default-on but the per-tier transform is a savings estimate not a
  content transform; H-6 deterministic masking + adaptive_reduce reportedly has zero production callers;
  K-12 losslessness gate currently WARN not FAIL-CLOSED (K-12 is the flip).
- ELAI planned: J-34 measurement bench (the per-(model,lane) knee) ABSENT — the single missing LAYER
  per the K-17 honesty note; K-17 packages the differentiator as DESIGN + BENCH-PENDING.
So the parts of ELAI that map to CWD's strengths are exactly the gated/planned ones; that is WHY CWD is
a clean reference-impl steal rather than a redundant rebuild. This is a timing gap on one leg, not a
design gap on the platform.


8. BOTTOM LINE
--------------
Neither is simply "better" — they are built for different objectives and mostly different layers.
CWD wins the narrow, real, shipped objective: deterministic, cache-preserving, in-session compaction
for cloud function-calling agents, with an honest measured number. ELAI wins on breadth and on the
hard guarantees CWD never attempts: provable recovery, regime-aware cloud/local policies, cross-model
and cross-session handoff, reversibility, and multi-class budgeting — as a design that is in places
ahead of CWD, with the caveat that its in-session cache-preserving leg is partly designed-not-running
and its degradation thresholds are still folklore until J-34 measures them. The highest-value move is
not to pick a winner but to use CWD as the reference implementation for ELAI's cloud-lane
cache-preserving leg, while keeping ELAI's recovery, regime, handoff, and measurement layers — which
CWD has no answer for.
