CONTEXT MANAGEMENT / COMPRESSION LOSSLESSNESS — REASONING
Date 2026-06-25. Mode: spec-eval AS-SPECIFIED (judge design as if built; status is a
separate labelled note). GRC: mcp_think/local-rag offline this session, reasoning in-band.

QUESTION (verbatim intent)
Is ELAI context management / compression lossless? How does ELAI move context between
(small) models so the next clean-context model keeps the IMPORTANT info (where the bug is,
which resources matter) and drops the BLOAT (search commands, wrong turns, failed probes)?
"lossless on what matters, removing bloat."

================================================================================
PART 1 — WHAT THE FEATURE MAP SAYS (read 100%, 2501 lines)
================================================================================

The honest one-line answer: ELAI's context management is NOT lossless in the literal
"the model sees everything" sense, and it is deliberately not. It is a TWO-LAYER design:
a LOSSLESS RECORD (the journal) under a LOSSY-BY-DESIGN VIEW (what any one model's window
holds). The doctrine name for this is RV-2 (H Track 8): "lossless store / lossy view —
trim is a projection, never the record." This is exactly the "lossless on what matters,
remove bloat" frame the user is asking for — ELAI already holds that frame as doctrine.

LAYER 1 — THE LOSSLESS RECORD (WIRED, this is the strong, shipped part)
- SSB SHARED STATE BOARD [WIRED] crates/orchestrator/src/ssb.rs: single-writer SQLite WAL,
  APPEND-ONLY, ~80K inserts/sec, broadcast-after-commit. Nothing is ever deleted or
  rewritten. This append-only journal IS the lossless substrate. Every other context
  artifact is a projection over it.
- GD-2 doctrine: every emitted number is recomputable from the SSB journal under ELAI_SEED.
  Losslessness is enforced as a house rule, not a hope.
- SESSION DAG [WIRED] (phase-3H, B-9): DAG snapshot versioning + "3-pass LOSSLESS TRIM
  (stubs recoverable from SSB seq)" + branch/merge max 3 + 85% handoff trigger. The trim
  that shrinks a window is reversible: trimmed spans become stubs that recover_stubs can
  re-inflate from the journal sequence. So even the in-window trim is lossless against the
  record.
- REVERSIBLE EDITS / TIME-TRAVEL [PARTIAL] (H-49): CAS blake3+zstd object store wrapping
  edit_checkpoints, undo-tree over SessionDag, "lossless paged transcript", pre-image
  already captured at file_ops.rs:272/300. RV-1 snapshot-before-restore.

LAYER 2 — THE VIEW / COMPRESSION (the model's actual window; lossy by design)
ELAI's compression PHILOSOPHY is the key finding, and it is contrarian vs the rest of the
industry. Two hard bans define it:
- HD-2 (and Section-4 ledger): "LLM re-summarization compaction on the cache prefix
  REJECTED — busts cache; deterministic MASKING only." ELAI refuses to let a model
  summarize the transcript to shrink it.
- IC-3: "Ciphers / abbreviation-substitution context compression BANNED; deterministic
  reversible transforms only." I-14 ContextTransformReceipt makes every transform
  reversible.
Why this matters for the user's question: the obvious way to "keep the important, drop the
bloat" is to have a small model summarize. ELAI BANS that, because (a) it busts the KV
cache prefix (cost), and (b) a small-model summary is itself a hallucination/lossy step
with no guarantee it kept "what matters." ELAI's answer instead is DETERMINISTIC SELECTION
and STRUCTURAL DISTILLATION, not generative summarization.

The shipped/partial view-layer machinery:
- CONTEXT BUDGET ALLOCATOR [WIRED] phase-3E / B-10: 8 fixed slots (SystemPrompt / Rules /
  PhaseSpec / RepoMap / CodeContext / SsbEntries / GrcHistory / OutputReserve), per-slot
  floors + steal, 2 cache breakpoints (xxhash prefix), 3-level adaptive reduction <50ms.
  This is the deterministic budgeter — it decides WHAT goes in the window by slot priority,
  not by asking a model.
- DETERMINISTIC OBSERVATION MASKING + READ-BACK [PARTIAL] (H-6): "masking drops TRAILING
  observations, never rewrites the cache prefix"; wire recover_stubs + get_relevant_facts
  read-backs. This is the live compaction lever: drop stale tool-output tails, keep the
  prefix cache-stable, recover on demand.
- INFINITE-CONTEXT CHAT [WIRED] (E-1): A-MEM + MemGPT-style digest, NoteStore + ppr symbols
  in production paths. The shipped long-conversation manager.
- COMPACTION LADDER [DARK ELAI_E25_* x9] (E-25): 6-tier compaction ladder + edit_checkpoints
  (not wired to hook_dispatcher). Built, gated off.
- REGIME-AWARE CONTEXT POLICY [PARTIAL, uncommitted] (H-50): cloud vs local get OPPOSITE
  policies — cloud = big cache-stable prefix + late compaction; local = window-minimization
  + unload-early. Fixes the bug that local Qwen ran a cloud-shaped policy (flat-100K
  auto-compact never fires <32K; 64K gate ignores ELAI_LOCAL_MAX_CONTEXT). DIRECTLY relevant
  to "between small models": the small-model lane needs aggressive minimization, and H-50 is
  the policy split that gives it that.
- EMPIRICAL CONTEXT CALIBRATION [PLANNED] (J-34) + CONTEXT-ROT CEILING/PLACEMENT (J-76):
  measure per-(model,lane) fill->verified-outcome degradation curves; head/tail placement
  (never middle, "lost in the middle"); chunk-count cap. Replaces folklore thresholds
  (hardcoded CLOUD_TRIM_FRACTION=0.85) with a MEASURED knee. This is how ELAI would learn,
  per small model, how much context that model can actually use before quality rots — the
  empirical basis for "lossless on what matters FOR THIS MODEL."

LAYER 3 — CROSS-MODEL / CROSS-AGENT HANDOFF (the literal "next clean model gets clean
context" mechanism the user describes). THIS IS THE CRUX, AND IT IS MOSTLY PLANNED.
- PRISTINE CONTEXT INJECTOR [PLANNED(I-5)]: "hard-exclude failed-sibling tracebacks,
  contrastive constraint only." This is EXACTLY the user's ask: when a worker burned context
  searching and went down a wrong path, the next worker does NOT inherit the dead-end
  transcript — it inherits only a contrastive constraint ("not X, because Y"). Drop the
  bloat, keep the lesson. PLANNED, not built.
- PABU TYPED BELIEFSTATE HANDOFF [PLANNED(I-11)]: "replaces raw history on cross-model/tier
  handoffs; DKG-keyed." Instead of shipping the raw transcript across a model boundary, ELAI
  ships a typed belief state (what is known, keyed to DKG symbols). This is the
  distillation-on-handoff the user wants for small models. PLANNED. Consumed by I-7
  CASCADING ESCALATION (condense-on-escalate).
- CONTEXT-ACCESS SPEC [PLANNED(J-78)]: each dispatch gets a DECLARED, journaled spec of
  which prior SSB rows / sibling outputs / memory it MAY read; the orchestrator computes it
  deterministically (TD-3, the LLM does not choose). I-5 hard-exclude becomes a declared
  case. This is the formal "the next model only sees the curated slice."
- DETERMINISTIC CONTEXT COMPRESSION [PLANNED(I-14)]: LSP slicing + tree-sitter folding;
  every transform lands a reversible ContextTransformReceipt. The deterministic
  "keep-the-relevant-code-spans, fold the rest" compressor.
- SCOPED CONTEXT (orchestrate / auto-orchestrate dispatch rule, WIRED as doctrine): a worker
  receives ONLY the task + relevant paths + the orchestrator-summarized slice of prior
  results + its handoff pointer — "never the full run-state, never sibling agents' raw
  output, never failed siblings' tracebacks." The orchestration model already enforces
  "clean context per worker" at dispatch time. This IS shipped (it's how the agents run);
  I-5/I-11/J-78 are the in-runtime, journaled, typed formalization of it.
- MODEL-HANDOFF UX [DARK] (E-3): compress_handoff_payload (test-only caller) + show ROLE
  hide MODEL. The compression-on-handoff primitive exists but is dark.
- CROSS-SESSION SLEEP-TIME HANDOFF [PARTIAL, uncommitted] (H-51): grounded topic-digest
  auto-authored FROM THE JOURNAL, tiered load ~20-27K -> ~1-3K tokens. The across-session
  analog: a fresh session loads a 1-3K distilled digest instead of a 20-27K transcript.
- MIMOCODE CONTEXT+MEMORY MECHANICS [PLANNED(J-25)]: typed 11-section digest template +
  section-aware budgeted read + tool-pair boundary guard (un-orphan a tool call/result pair
  before ANY trim boundary) + 4-level pressure ladder + newest-wins writer coalescing. The
  tool-pair guard is a real correctness detail: trimming must not split a tool_use from its
  tool_result.

SUPPORTING MEMORY SUBSTRATE (what "important info" persists into)
- USER MEMORY SUBSTRATE crates/memory [PARTIAL] (E-5): Mem0 ADD/UPDATE/DELETE/NOOP + Zep
  bi-temporal + MINJA poisoning defense.
- TIER-B PREFERENCE MEMORY (H-31): retrieval HARD-FILTERS superseded rows (never
  "[OUTDATED]" narration — the measured Graphiti +68%-token trap). AUDITOR-OBJECTIVITY
  FIREWALL: auditor/critic/adversary get ZERO memory injection. Relevant: ELAI's stance is
  that even keeping superseded info as "[OUTDATED]" in-window is BLOAT and is rejected.
- TIER-A CODE-GROUNDED FACT MEMORY (H-32): validate-citation-before-use on DKG, 28d TTL.
- The retrieval stack (DKG, BM25, vector, PageRank, CatRAG/A-RAG, cross-encoder reranker;
  all WIRED) is how a fresh-context model RE-FETCHES the important resources on demand
  instead of carrying them in-window — "55-87% token-savings target" (A-RAG). The design
  bet: don't carry the codebase in context, carry a pointer + re-retrieve precisely.

================================================================================
PART 2 — IS IT LOSSLESS? (precise verdict, AS-SPECIFIED)
================================================================================
- THE RECORD: provably lossless. Append-only SSB journal; trim is recoverable
  (recover_stubs from SSB seq); GD-2 recomputable-from-journal is a house rule. Verdict:
  lossless and shipped (WIRED).
- THE VIEW: lossy by design, but the loss is CONTROLLED and (per doctrine) REVERSIBLE.
  Deterministic masking drops recoverable trailing observations; I-14 transforms carry
  reversible receipts. ELAI deliberately refuses generative (lossy, cache-busting)
  summarization. Verdict: "lossy projection over a lossless record," which is the correct
  design — but the projection is NOT YET PROVEN to be "lossless on what matters." There is
  no shipped measured guarantee that the kept slice preserves every decision-relevant fact.
  J-34 (measure the degradation knee) is the closest, and it is PLANNED.
- THE BLOAT-REMOVAL the user specifically describes (drop search commands / wrong turns,
  keep where-the-bug-is): the precise mechanism is PRISTINE CONTEXT INJECTOR (I-5,
  hard-exclude failed-sibling tracebacks + contrastive constraint) + PABU beliefstate
  (I-11) + CONTEXT-ACCESS SPEC (J-78). All three are PLANNED, not WIRED. What IS shipped is
  the coarser version: SCOPED CONTEXT at dispatch (workers never get sibling raw output) +
  deterministic masking of trailing tool noise + retrieval re-fetch instead of carry.

So: the ARCHITECTURE is sound and exactly aimed at the user's goal (lossless record, lossy
controlled view, deterministic-not-generative compression, typed cross-model handoff). The
specific "clean hand-off of just-the-findings between small models" is designed but the
load-bearing pieces (I-5, I-11, J-78, I-14) are on the PLANNED list, and the live compaction
(H-6/H-50/E-25) is PARTIAL/DARK/uncommitted. The honesty note from the Section-4 ledger
itself says this: "ELAI context-mgmt DESIGN is at/beyond [the external sources], but
ACTIVATION is the gap — H-50/H-51 PARTIAL uncommitted, E-25 DARK x9, J-34 PLANNED. Live
cloud trim still on the folklore 0.85."

================================================================================
PART 3 — OPEN GAPS / WHAT WOULD MOVE THE NEEDLE (pre-research; refine after papers)
================================================================================
G1. No measured proof the lossy view is "lossless on what matters." Need J-34 + a
    decision-relevant-fact preservation metric (did trimming drop a fact the model later
    needed?).
G2. The cross-model distillation (I-5 / I-11 / J-78) is PLANNED. This is the single most
    direct answer to the user's question and is unbuilt.
G3. Local-lane (small-model) caching is a NO-OP today: sidecar PrefixKVCache (server.py:1819)
    is bypassed on the streaming path (runtime_client.rs:547). So for small models the
    "keep a stable prefix" lever currently does nothing — window-minimization (H-50 local
    policy) is the only lever that works for them right now.
G4. Generative summarization is banned (HD-2/IC-3) — correct for cache + determinism, but it
    means ELAI cannot do the "small model writes a 200-token summary of what it found" move
    at all; the distillation must be STRUCTURAL (typed belief state, symbol pointers, AST
    folding). This is a strength (no hallucinated summaries) but raises the bar on building
    I-11/I-14 well.

PART 4 — EXTRA DETAIL FROM PLAN-SUMMARY SEARCH (grep over *-SUMMARY*.md)
The summary indexes add nothing the feature map missed (the feature map IS the synthesis,
and is newer). One useful phrasing: I-5 PRISTINE CONTEXT INJECTOR is described as the
"first quantified VIVIDNESS TAX" — i.e. failed-path / wrong-turn content is not just token
bloat, it actively biases the next model (vivid failures over-anchor attention), so excluding
it is a quality win, not only a cost win. This directly matches the user's "removing bloat of
search commands, finding wrong stuff." Also confirmed: ALL-PLANS-MASTER-SUMMARY.md is a
single-file digest of the 10 indexes and is explicitly SUPERSEDED by ELAI-FEATURE-MAP.md for
context-load. H-50 detail: local lever = window-MINIMIZATION not caching (HF sidecar has no
cross-request KV reuse), confirming small-model handling is minimize-the-window, not
keep-a-prefix.
[PART 5 reserved: external paper/repo grounding (2305.14788, 2307.06945, LoPace,
  honey-for-devs, code-context-engine, LLMLingua) — appended after ingest.]
PART 6 — DEEP-RESEARCH-REPORT(4).md FINDINGS ("Language and Context Compression for
LLM Systems"). NOTE: this is a GPT/Gemini deep-research output = tier-2 / WEAK grounding
(one vote; never a GRC grounding source). Treated as corroboration + a source list; specific
arXiv IDs to be verified by the paper agents + WebSearch before entering any plan.

THE REPORT'S BLUNT ANSWER (directly answers the user): As of June 2026 there is NO
general-purpose, WinRAR-like lossless compressor you can apply to arbitrary prompts and get
fewer BILLED tokens through normal text APIs. APIs bill on tokens produced by the tokenizer
from the text you send, not on file entropy. gzip/zstd/Brotli/Base64 only help STORAGE and
TRANSPORT; Base64-of-gzip-in-prompt usually makes tokenization WORSE. Pigeonhole limit: no
lossless scheme makes ALL inputs shorter. Tokenizers (BPE/SentencePiece) are themselves
~4x dictionary compressors and that is already priced in — you do not get a second free layer.

THE REPORT'S RECOMMENDED ARCHITECTURE — and it is ELAI's design, almost feature-for-feature:
A "HYBRID STATE LEDGER", not a universal compressor:
  (1) STRUCTURED SCHEMA HANDOFF with exact fields (goal/constraints/safety, repo+commit+
      artifacts with sha256+span, done/next/blocked, evidence with verbatim+confidence,
      edits as patch-refs, critical_atoms with must_preserve:true). "Yes, strongly."
  (2) RETRIEVAL + POINTERS + DELTAS — keep canonical data OUTSIDE the prompt, send
      references (path+line span, commit SHA, chunk id, test name, stack-trace fingerprint)
      + deltas only. "Closest practical LOSSLESS strategy." "Yes, strongly."
  (3) COMMITMENT-PRESERVING SEMANTIC ATOMS (Context Codec / CCL): typed, source-grounded
      atoms with evidence spans + Critical Atom Recall / Weighted Atom Recall / Commitment
      Density / round-trip recoverability METRICS. "Yes, for advanced systems."
  Hard prompt compression (LLMLingua family) = LOSSY, "not for critical facts" (drops exact
  IDs/constraints). Soft/latent (Gist/ICAE/AutoCompressors) + latent inter-agent (LatentMAS)
  = need model control / hidden-state access = "only if self-hosted" / "no for most teams."

THREE TIERS OF LOSSLESS the report insists on separating (the precise vocabulary for the
user's question): BYTE-IDENTICAL (classical + dictionary schemes) / SEMANTIC (Context Codec,
SemanticZip — meanings & commitments preserved, wording differs) / TASK-EQUIVALENT (hard &
soft compression — model still solves the task, details irrecoverable). Rule: "only call a
method lossless if the original is recoverable exactly OR the schema contains every field you
operationally care about. Semantic similarity alone is NOT enough."

HOW EACH MAPS ONTO ELAI (validation, not new):
- "keep canonical data outside the prompt + references" = ELAI SSB append-only journal (the
  durable store) + the WIRED retrieval stack (DKG/BM25/vector/PageRank/cross-encoder, A-RAG
  55-87% token-save). ELAI already does the report's #2, the strongest-rated approach.
- "structured schema handoff with critical_atoms must_preserve" = ELAI PABU typed
  BeliefState handoff (I-11) + the report's Context Codec atoms. ELAI design matches; PLANNED.
- "wrong/unfiltered prior experience gives negative benefit" (SWE-ContextBench) = ELAI I-5
  Pristine Context Injector (exclude failed-sibling tracebacks) + the "vividness tax". PLANNED.
- "deterministic, boring, structured beats exotic codes; the more exotic the code the more
  decoder fragility" + "Base64-gzip-in-prompt fails" + "semantic similarity hides loss" =
  ELAI's HD-2/IC-3 bans (no LLM-resummarization of cache prefix, no ciphers, deterministic
  reversible only). The report independently arrives at ELAI's exact prohibitions.
- "delta-only updates" + "evidence+source pointers (file+line, test id, commit SHA, stack
  fingerprint)" = ELAI RRA (G-2) + Finding Anchoring (H-52, verbatim snippet never a line
  number) + Citation Pre-binding (H-53, closed-set [N]) + Session DAG delta snapshots.
- code-specific pruning (LongCodeZip 5.6x) = ELAI I-14 deterministic context compression
  (LSP slicing + tree-sitter folding) but ELAI's is DETERMINISTIC+REVERSIBLE (stronger guarantee).
- dictionary encoding for repetitive logs (Amazon 2026, up to 80%, exact-match >0.99 on
  Claude 3.7) = the ONE black-box near-lossless token reducer; ELAI fit is NARROW (repetitive
  build logs / telemetry tool-output, NOT code) — a possible minor lane, not a general answer.

THE ONE GENUINELY-NEW THING THE REPORT SURFACES (closes my Gap G1): the Context Codec
MEASUREMENT layer — CRITICAL ATOM RECALL + COMMITMENT DENSITY + ROUND-TRIP RECOVERABILITY as
a deterministic gate on the handoff projection. ELAI measures CONTEXT-FILL degradation (J-34)
but has no "did the projection preserve the must-preserve atoms?" gate on cross-model/trim
handoffs. Adding a deterministic HANDOFF-FAITHFULNESS GATE (the kept slice must contain every
critical_atom flagged must_preserve, verified by exact string/symbol presence against the SSB
record, missing=FAIL per wiring R1) is what lets ELAI claim "lossless on what matters" with a
PROOF instead of a hope. This is additive, deterministic (TD-3), and rides existing seams
(I-11 belief-state schema + the journal as ground truth + G-2 RRA verify).
PART 7 — CODE-GROUNDED REALITY (codegraph + elai.map; CORRECTS the feature map's stale
PLANNED/PARTIAL labels — and the correction is strongly in ELAI's favour). Verified 2026-06-25.

THE LOSSLESS RECORD IS REAL AND CRYPTOGRAPHICALLY VERIFIED (this is the headline finding):
- session_dag.rs:1539 recover_stubs(): trimmed messages become STUBS carrying {ssb_seq,
  summary_hash}. Recovery re-fetches the payload from the append-only SSB by seq and asserts
  blake3(entry.payload) == stub.summary_hash; mismatch -> StubRecoveryFailed. This is
  BYTE-IDENTICAL, HASH-VERIFIED lossless recovery against the journal — the strongest possible
  losslessness guarantee, and it is WIRED (1 production caller in live_cli.rs, plus
  recover_single_stub). So "is the trim lossless?" -> YES, provably, against the SSB record.
- compact.rs:545 check_lossless_compaction() (H-49 RV-2): a fail-closed precondition that
  counts to-be-dropped messages and how many are unrecoverable (no SSB stub); is_safe =
  unrecoverable==0. IMPORTANT HONEST GAP: it WARNS, it does NOT BLOCK ("we emit a warning but
  do not break it"). So the losslessness GATE is observe-only today; the recovery MECHANISM is
  sound, but a compaction that would drop un-archived messages is allowed to proceed with only
  a tracing::warn. Turning this from WARN to fail-closed-BLOCK (wiring R1 missing=FAIL) is a
  small, high-value hardening.
- session_dag.rs create_handoff()/load_handoff(): 3-pass trim + tagged snapshot + branch-tip
  load. WIRED. Undo-tree HEAD + CAS file_manifest (H-49) present.

THE COMPRESSION / BLOAT-REMOVAL PIECES ARE FURTHER ALONG THAN THE MAP SAYS:
- I-5 PRISTINE CONTEXT INJECTOR: pristine_path_filter.rs EXISTS (not just planned) — imports
  context_budget + regime_policy + ContextPrunedKind/ContextPrunedPayload SSB; contract
  i5_pristine_context_contract.rs asserts branch1_traceback_emits_context_pruned_and_constraint
  AND pristine_rebuild_zero_traceback_bytes (proves the rebuilt context contains ZERO bytes of
  the failed-path traceback while emitting a ContextPruned event + a contrastive constraint
  phrase). This is EXACTLY the user's "drop the search/wrong-turn bloat, keep the lesson" — and
  it has code + a passing-shape contract. Map said PLANNED; reality = BUILT (wiring audit
  pending, so PARTIAL at worst).
- I-11 PABU TYPED BELIEFSTATE: BeliefState::new is wired at escalation_waterfall.rs:239 (the
  local->cloud escalation handoff), with belief_state_event SSB (BeliefStateEventKind::Condensed)
  and i11_belief_state_contract.rs. So the typed-beliefstate-instead-of-raw-history handoff is
  BUILT on the escalation path. Map said PLANNED.
- H-50 REGIME-AWARE CONTEXT POLICY: context_policy.rs EXISTS (effective_context_window /
  regime_policy / trim_threshold / LOCAL_TRIM_FRACTION / CLOUD_TRIM_FRACTION / LOCAL_DEFAULT_WINDOW
  / ELAI_LOCAL_MAX_CONTEXT_VAR) and is IMPORTED INTO live_cli.rs:50 with the comment "H-50:
  regime-aware window truth (replaces max_tokens_for_model in prepare_turn_runtime)". So H-50 is
  WIRED into the live turn path, not "PARTIAL uncommitted". The small-model lane now gets its own
  window-minimization policy. cache_economics.rs also consumes ContextPolicy.
- H-51 handoff_digest.rs EXISTS (TopicHandle exported). H-6 adaptive_reduce wired at
  context_budget.rs:1579 (3-level reduction, contract-tested, fires on 20% underestimate).

NET CODE-GROUNDED VERDICT: the two-layer design (lossless hash-verified record + lossy
controlled view + deterministic bloat-exclusion + typed cross-model belief-state) is not a
plan — it is substantially BUILT and wired into live_cli + the escalation path. The map's
PLANNED labels for I-5/I-11/H-50 are stale (the map itself warns "statuses drift"; spec-eval
says don't judge design on shipped-status anyway). What is genuinely MISSING is the MEASURED
PROOF that the lossy view is "lossless on what matters": there is no handoff-faithfulness gate
asserting the must-preserve atoms survived, and check_lossless_compaction WARNs instead of
BLOCKING. Those two are the real, small, high-leverage gaps.
PART 8 — RECOMMENDATION (what we should do). Grounded in: code read this session (PART 7),
the deep-research report (PART 6, tier-2 corroboration), the feature map + ledger (PARTS 1-4).
External paper specifics (PART 5) pending the two ingest agents; recommendation is robust to
them and will be annotated if they change anything.

DIRECT ANSWER TO THE QUESTION
1. Is ELAI's context management lossless? It is a TWO-LAYER design and the honest answer is
   "lossless where it must be, lossy-by-design where it should be":
   - The RECORD is provably, cryptographically lossless: trim->stub->SSB-recover with a
     blake3 integrity check (recover_stubs, WIRED). Byte-identical recovery or hard error.
   - The VIEW (the model's window) is a lossy but REVERSIBLE projection. ELAI deliberately
     REFUSES the naive lossy compressors (LLM re-summarization, ciphers, gzip/Base64-in-prompt)
     — and the independent deep-research report confirms each of those is either lossy on
     facts or doesn't even reduce billed tokens. So ELAI's "lossless on what matters" is the
     correct frame and is largely realized.
2. How does it move context between (small) models keeping the signal, dropping the bloat?
   NOT by compressing language. By five mechanisms, most of which are BUILT:
   (a) SCOPED DISPATCH — a worker never inherits sibling raw output / failed tracebacks
       (orchestrate/auto-orchestrate dispatch rule, shipped doctrine).
   (b) I-5 PRISTINE CONTEXT INJECTOR — hard-excludes failed-path tracebacks and injects a
       short CONTRASTIVE CONSTRAINT instead ("don't go down X, it was a dead end"); contract
       proves zero traceback bytes survive the rebuild. BUILT (pristine_path_filter.rs).
   (c) I-11 PABU TYPED BELIEFSTATE — the next model gets a typed belief state (what's known,
       keyed to DKG symbols), not the raw transcript; wired on the escalation handoff. BUILT.
   (d) RETRIEVAL RE-FETCH — don't carry the codebase; carry pointers + re-retrieve precisely
       (DKG/BM25/vector/PageRank/reranker, WIRED). The report rates this the strongest
       practical lossless strategy.
   (e) H-50 REGIME POLICY — the small local lane gets aggressive window-minimization (it can't
       reuse a KV prefix anyway); wired into live_cli. BUILT.
   So "model A searched, found the bug at file X:line Y; model B starts clean with just that
   + the constraint not to repeat A's dead end" is precisely I-5 + I-11 + retrieval pointers
   over the lossless SSB journal — and that path largely exists in code today.

THE TWO REAL GAPS (small, high-leverage; everything else is activation/finishing)
G-A. check_lossless_compaction WARNS, does not BLOCK. A compaction that would drop
     un-archived messages proceeds with a tracing::warn. The record is only lossless if the
     archival actually happened first.
G-B. There is no MEASURED PROOF the lossy VIEW preserved the decision-relevant facts. ELAI
     measures context-fill degradation (J-34, planned) but has no "did the must-preserve
     atoms survive this handoff/trim?" gate.

RECOMMENDED ACTIONS (ranked by leverage/cost; mostly ACTIVATION, one small NEW gate)
R1 [tiny, highest leverage] Make check_lossless_compaction FAIL-CLOSED: refuse the trim (or
   force the SSB capture-hook to run first) when unrecoverable_count > 0, per wiring R1
   (missing evidence = FAIL). Flips the lossless-record guarantee from advisory to enforced.
   Rides existing code in compact.rs + the capture hook. No new plan.
R2 [small NEW extension — the one genuinely-new idea, surfaced by the report's Context Codec]
   HANDOFF-FAITHFULNESS GATE: extend the I-11 BeliefState schema with
   critical_atoms{kind,value,must_preserve} (file path, symbol, test name, line span, the bug
   location, a safety constraint). After any cross-model handoff / pristine-rebuild / trim,
   deterministically assert EVERY must_preserve atom is present in the projected context by
   exact string/symbol match against the SSB record; missing = FAIL. This is the deterministic
   "Critical Atom Recall" gate (TD-3, LLM never adjudicates). It is what lets ELAI claim
   "lossless on what matters" with a PROOF, not a hope. Composes I-11 + the SSB record + G-2
   RRA verify + J-78 context-access spec; +1 SSB kind or reuse BeliefStateEvent. Authors as a
   new unit in the current plan family (new file, per the never-amend rule).
R3 [finish + measure] Land J-34 EMPIRICAL CONTEXT CALIBRATION so the trim threshold is the
   MEASURED per-(model,lane) degradation knee, not the folklore CLOUD_TRIM_FRACTION=0.85 —
   most important for the small local lane the user is asking about. Pair with J-72 in-domain
   retriever adaptation so the re-fetch (the lossless alternative to carrying context) stays
   accurate per-domain. Both PLANNED; this is the measurement spine under R2.
R4 [verify it's wired end-to-end] I-5 / I-11 / H-50 have code + contracts but no FINISH-grade
   wiring audit. Run production_caller_check.py over them + an INTEGRATION smoke (wiring R10)
   driving a real local->cloud handoff on a HARD multi-turn input, asserting the pristine
   filter actually drops the traceback and the belief-state actually carries the bug location
   in production — not only in the unit contract.
R5 [narrow, opportunistic, low priority] Dictionary-encoding (Amazon 2026, the one black-box
   near-lossless token reducer) for REPETITIVE tool-output only (build logs / test spew /
   telemetry), NEVER for code. Clearly-scoped minor lane; measure before shipping.

DO-NOT (anti-recommendations, all grounded)
- No LLM-summarization compaction of the cache prefix (HD-2 ban; report confirms lossy +
  cache-busting). The distillation must stay STRUCTURAL (typed belief-state, symbol pointers,
  AST folding), never generative.
- No gzip/Base64/ciphers-in-prompt (IC-3 ban; report: doesn't reduce billed tokens, worsens
  tokenization). Tokenizer already does ~4x dictionary compression; there is no free second layer.
- No latent/soft compression (Gist/ICAE/AutoCompressors/LatentMAS) on the cloud wedge — needs
  hidden-state/decoder access ELAI structurally lacks (same reason RecursiveMAS was rejected).
  It is a self-hosted-only frontier; revisit only for a fully-owned local stack.
- honey-for-devs: hold a verdict until the ingest agent reports whether its token reduction is
  lossless-on-code-quality or a quality tradeoff (the user's explicit red line: reduce tokens,
  NEVER reduce code quality). If it is a lossy-on-quality compressor, REJECT for code paths.

BOTTOM LINE: ELAI's design is right and the external SOTA report independently re-derives it
almost feature-for-feature (hybrid state ledger = structured handoff + retrieval pointers +
commitment atoms; deterministic-not-generative). The record-side losslessness is
cryptographically real and wired. The work to do is not invention — it is (R1) enforce the
lossless gate, (R2) add the deterministic faithfulness proof, (R3) measure the per-model knee,
(R4) prove the built pieces fire in production. One small new gate (R2); the rest is activation.

PART 5 — EXTERNAL PAPER/REPO GROUNDING (sonnet paper-ingestor agents; interface maps under
workflow/papers/). REPO HALF returned 2026-06-25; PAPER HALF (RECOMP/ICAE/LLMLingua) pending.

REPOS (agent 2, DONE; interface maps: lopace-interface.md, honey-for-devs-interface.md,
code-context-engine-interface.md; all MIT, clones in docs/.source/):

LoPace (commit 6f94369, arXiv 2602.13266): a STORAGE compressor (Zstd / BPE-pack / Hybrid),
lossless byte-identical, deterministic (no LLM). BUT output is BINARY — cannot re-enter a
prompt. Confirms the deep-research report exactly: LoPace reduces DISK/WIRE bytes, NOT billed
tokens. ELAI fit: NOT a token reducer; only useful if ELAI ever caches prompts to disk
between sessions (the SSB journal already is that store). VERDICT: not applicable to the
context-window problem; do not adopt as a token lever.

honey-for-devs (commit 9682ea3) — THE USER'S RED-LINE REPO ("reduce tokens, NEVER reduce code
quality"). VERDICT: its prose stripper (precompress.js) is SAFE on code correctness, and this
is BENCH-VERIFIED: 10 coding tasks padded with filler, compressed, unit-test pass rate
100%->100%, judge delta -0.25 (noise). The reason is the same protected-span idea ELAI
already uses: fenced/inline code, URLs, paths, filenames, digit-runs pass BYTE-FOR-BYTE; only
greetings/please/thanks/hedging/dupes are dropped. TWO HONEST CAVEATS: (1) real-world saving
is marginal — on 266 real human prompts the MEDIAN saving was 0.0% (219/266 cut nothing);
developers are already terse, the 16.5% headline came from an artificially verbose corpus.
(2) its LOG compressor (logcompress.js, collapses >=3 repeated lines to "line (xN)") IS LOSSY
on volatile values (timestamps/UUIDs/hex inside collapsed runs are irrecoverable from context).
Its ESO columnar wire format competes with ELAI's SSB serialization; its CCR (keep
head/change-point/tail rows, offload the rest to a hash-keyed cache, retrieve by hash) is
architecturally the SAME pattern as ELAI's retrieval + recover_stubs. ELAI fit: the
protected-span-mask-before-prose-edit pattern is a clean, deterministic steal; but on ELAI's
already-terse internal prompts the payoff is small, and the log collapser must only run where
the dropped volatile values are also archived to the journal (else it violates losslessness).

code-context-engine (v0.4.24, commit 144332b): a full code-index + context pipeline. Best
deterministic steals: grammar.py (priority-ordered tokenizer — fence>inline>URL>datetime>
version>path>identifier>prose; drops only prose fillers, structured tokens verbatim; pure
stdlib, DEFAULT level "lite") — a MORE COMPREHENSIVE protected-span set than honey-for-devs
(adds identifier/version/datetime classes), directly adoptable and consistent with ELAI's
deterministic-reversible rule. extractive.py (centroid-cosine sentence SELECTION, real
sentences only, no synthesis, no numpy) — adoptable ONLY as a retrieval INDEX over a retained
lossless journal (which ELAI has). DO-NOT-ADOPT: its compression/compressor.py uses a local
Ollama LLM (phi3:mini) to summarize code chunks = LOSSY + generative — violates ELAI's HD-2/
IC-3 deterministic-reversible-only rule for active context (same class as the banned
LLM-resummarization). Quality gate (<40% identifier retention -> fallback) is a nice idea but
the right ELAI analog is the R2 critical-atom faithfulness gate, deterministic not heuristic.

REPO-HALF NET: nothing here overturns the recommendation; it REINFORCES it. The deterministic
protected-span tokenizer (grammar.py-style) is the one concrete, safe steal — it is a cleaner
implementation of the masking I-5/H-6 already do, and could harden the pristine filter's
prose-stripping. Every LLM-driven or binary-blob approach is correctly out of bounds for
ELAI. The honey-for-devs bench is useful EXTERNAL evidence for R2: it shows the ONLY reliable
way to compress safely is to mark must-preserve spans and prove they survive — exactly the
critical-atom gate. The agent left RAG-indexing undone (local-rag MCP was offline this run);
interface maps exist as files.

PAPERS (agent 1, DONE; interface maps: autocompressor-2305.14788-interface.md,
icae-2307.06945-interface.md, llmlingua-source-interface.md):

CITATION CORRECTION (house rule, WebSearch-verified): arXiv:2305.14788 is "Adapting Language
Models to Compress Contexts" (AutoCompressor, Chevalier et al., EMNLP 2023) — NOT RECOMP. The
earlier dispatch label "RECOMP" was wrong; RECOMP is arXiv:2310.04408 and was NOT ingested.
ICAE = 2307.06945 (correct). LLMLingua family = 2310.05736 / LongLLMLingua 2310.06839 /
LLMLingua-2 2403.12968 (all verified).

AutoCompressor (2305.14788): LOSSY soft-prompt compression, ~40:1 (2048 tokens -> 50 summary
vectors), REQUIRES an LLM forward pass, output is MODEL-SPECIFIC float vectors. ICAE
(2307.06945): LOSSY, 4x (512->128 memory slots), EM=0.6 at that ratio (40% irrecoverable),
LoRA-adapted encoder, output bound to the encoder's embedding space. BOTH are
cross-model-INCOMPATIBLE — the compressed form cannot cross a model boundary, which is fatal
for the user's exact "between small models" scenario, and they violate all three ELAI rules
(need an LM pass; float tensors can't live in the text journal; not a typed belief-state).
This is the same disqualification ELAI already applied to LatentMAS/RecursiveMAS, now
confirmed for the soft-prompt family.

LLMLingua family: LOSSY on task quality (a measured tradeoff, drops tokens incl. exact
identifiers), needs a small-LM scoring pass (violates ELAI rule a, deterministic-only), BUT
outputs plain cross-model text (satisfies b+c). LLMLingua-2 (2403.12968) is the most
ELAI-adjacent variant: a CLASSIFICATION encoder (not generative), deterministic given fixed
model+seed=42, 3-6x faster — but still an LM pass, still lossy on facts. Only conceivable
ELAI use: an OFFLINE, non-critical-text preprocessing cache, NEVER inline, NEVER on code facts
/ identifiers / the must-preserve atoms. In practice the deterministic protected-span
tokenizer (grammar.py, repo half) is strictly safer than LLMLingua for ELAI and should be
preferred; LLMLingua stays evaluate-only at most.

PAPER-HALF NET: CONFIRMS the DO-NOT list in PART 8. Soft/latent compression (AutoCompressor,
ICAE) is correctly out — it cannot cross the small-model boundary the user asked about and is
model-bound + lossy. LLMLingua is lossy-on-identifiers and needs a model — at best an offline
non-critical lane, dominated by the deterministic grammar.py steal. Nothing in the paper half
changes the recommendation; it strengthens the case that ELAI's deterministic-structural path
(protected-span masking + typed belief-state + retrieval pointers + hash-verified journal) is
the right one, and that the faithfulness gate (R2) — not a learned compressor — is the missing
piece. RAG-indexing of the interface maps left undone (local-rag MCP offline); maps on disk.

END OF REASONING (PARTS 1-8 complete; all agents returned).

OUTCOME (2026-06-26 authoring session): R1 (fail-close check_lossless_compaction) + R2 (the
handoff-faithfulness Critical-Atom-Recall gate, the one genuinely-new idea) authored as K-12
(context-losslessness enforcement + handoff-faithfulness gate) — K-native per the user's
placement ruling. The deterministic protected-span tokenizer (PART 5 grammar.py steal) + the
narrow dictionary-log lane (R5) went to the technical half as J-82 leg (b)/(c). R3 (J-34/J-72)
+ R4 (FINISH wiring audit of I-5/I-11/H-50) stay activation/measurement, not new files. MAP
FIXES applied to ELAI-FEATURE-MAP: I-5/I-11 PLANNED->PARTIAL + H-50 "uncommitted"->WIRED
(code-grounded PART 7). K-0/K-SUMMARY updated; Section-4 verdict block (AutoCompressor/ICAE/
LoPace/LLMLingua rejects) recorded.
