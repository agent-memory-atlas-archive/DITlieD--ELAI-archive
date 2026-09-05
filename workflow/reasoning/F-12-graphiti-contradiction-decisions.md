F-12 — semantic contradiction pre-filter (deterministic local NLI judge)
Session s-6312, orchestrate Phase D. Decisions log.

SUMMARY

F-12 adds a trigger-only SECOND pass to ELAI's deterministic contradiction
adjudicator. The existing audit_conflicts (crates/memory/src/contradiction.rs:64)
only flags a contradiction when two currently-valid rows share a byte-identical
(project_id, scope, key) and their value JSON differs. It cannot detect two facts
that are semantically opposed across NON-matching keys. F-12 closes that recall
gap with a LOCAL judge — no cloud LLM — wired into the live memory tick, reusing
the existing SSB MemoryConflictDetected variant with conflict_kind=
"semantic_contradiction" (no new action_type, no SCHEMA bump).

TIER-1 / TIER-2 SPLIT

Tier-1 (ships now, fully OFFLINE, no model, DEFAULT-ON):
  crates/memory/src/semantic_contradiction.rs
  - NliJudge trait (classify(premise, hypothesis) -> Result<NliVerdict>): the ONLY
    seam a model plugs into. Synchronous, local, no async, no network client.
  - NliLabel {Entailment, Neutral, Contradiction} + NliVerdict {label, confidence}.
  - DeterministicNliJudge: a LOCAL lexical opposition heuristic (NOT a neural NLI
    model). Lowercase content-token sets; an explicit opposition/switch marker
    (not/instead/migrated/dropped/off/"in favour of"/...) co-occurring with a
    shared content token -> Contradiction (confidence scales with marker count +
    shared-token overlap, capped at 0.99, base 0.80 so a CLEAR opposition over a
    shared subject clears the 0.85 threshold while a weak one does not);
    high-Jaccard no-negation -> Entailment; else Neutral. Pure integer/string
    math: bit-stable across replays (no FP reduction order).
  - ContradictionCandidate, collect_cross_key_candidates (localize-first; re-pairs
    currently-valid rows across DIFFERENT keys within the SAME (project_id, scope)
    via list_current — the same query audit_conflicts uses; bounded so the judge
    sees ~2 rows per call, never the full cross-product).
  - adjudicate_semantic (the resolver), SemanticConflictOutcome / SemanticAction
    {InvalidatedOlder, SurfaceToUser}.
  - pair_fingerprint (blake3, 16-char hex prefix, mirrors live_cli.rs F-5 pattern).
  - SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD = 0.85 (named const, no magic
    number).
  The DeterministicNliJudge is the DEFAULT-ON enforcement so there is NO W-2
  default-OFF graveyard: the pass runs offline today, gated only on the
  disable-only kill-switch ELAI_SEMANTIC_CONTRADICTION_KILL.

Tier-2 (deferred, model-gated, dark-of-the-model):
  crates/memory/src/nli_onnx.rs (behind the NEW default-OFF cargo feature
  `nli-onnx`). OnnxNliJudge impl of NliJudge over the already-vetted ort/ONNX
  runtime. Housing decision: kept in crates/memory behind the feature (NOT moved
  to crates/knowledge). Grounding: ort + tokenizers are direct version pins in
  crates/knowledge/Cargo.toml:61-62 (ort = "=2.0.0-rc.12", tokenizers = "0.22"),
  already Cargo.lock-resolved; adding them as OPTIONAL deps to crates/memory with
  the identical pins activates the existing lockfile entries — no new external
  crate (anti-slopsquatting satisfied; both verified in-tree). Housing in memory
  avoids a NEW knowledge->memory inter-crate edge and co-locates OnnxNliJudge with
  the NliJudge trait. cargo check -p elai-memory --features nli-onnx compiles
  (ort 2.0.0-rc.12 + tokenizers 0.22.2 pulled from the existing lock). The default
  Tier-1 build compiles WITHOUT ort.
  OnnxNliJudge mirrors crates/knowledge/src/reranker.rs build_session EXACTLY:
  Session::builder().with_optimization_level(GraphOptimizationLevel::All)
  .with_intra_threads(1).with_deterministic_compute(true).commit_from_file(path),
  models/onnx path via elai_core::paths::models_dir_path(project_root).join("onnx").
  Difference from the reranker: the reranker extracts a single [batch,1] logit;
  the NLI cross-encoder emits [batch,3] (contradiction/entailment/neutral), so
  classify does a 3-class argmax + a numerically-stable softmax-max for the
  confidence margin. Construction FAILS LOUDLY when the model file is absent (no
  silent stub fall-through). Model staging (cross-encoder/nli-deberta-v3-{xsmall|
  small|base} exported to ONNX) is the ops-gated Tier-2 sub-phase, NOT a code dep.

THE NO-CLOUD-JUDGE DECISION (decisive instrument choice)

A cloud LLM judge is REJECTED by construction. Grounding: arXiv:2506.09501
("Understanding and Mitigating Numerical Sources of Nondeterminism in LLM
Inference") — temp=0 + fixed seed does NOT yield bit-identical cloud output:
continuous batching + GPU/TP-variant kernel reduction order breaks FP
associativity. An auto-close decision routed through the cloud lane could flip
between identical ELAI_SEED replays, directly contradicting ELAI's
deterministic-trust-spine value prop. The adjudication path therefore makes ZERO
network/cloud/anthropic/sidecar calls; no_cloud_judge_contract.rs asserts the two
F-12 modules contain no cloud_mode / route_with_spend / CloudModeRouter /
anthropic / reqwest / http references. (Note: arXiv:2506.09501 is cited for its
content only; the "NeurIPS 2025" venue label is unverified and is NOT asserted
here — the grounding sweep flagged the venue as UNGROUNDED.)

DETERMINISM FRAMING (honest)

  Tier-1 DeterministicNliJudge: bit-stable by construction (pure integer/string
  lexical math; no floating-point reduction order). A unit test asserts identical
  label + confidence across repeated calls.

  Tier-2 OnnxNliJudge: "practically stable argmax, NOT bit-identical." The same
  GPU FP batch-variance class (arXiv:2506.09501) applies to its logits. The
  deterministic-compute flags + single intra thread make replay robustly stable,
  not formally seed-invariant. The decision doc + any panel copy must NOT claim
  formal seed-invariance for the cross-encoder.

SOFT-CLOSE-OLDER-ROW SEMANTICS (Graphiti invalid_at = valid_at)

On a Contradiction at or above the threshold, adjudicate_semantic soft-closes the
OLDER row via store.soft_close(&older.fact_id, newer.valid_from_ms) — the
invalidation event-time is the INVALIDATING fact's valid_from_ms, NOT now_ms
(mirrors Graphiti edge_operations resolve_edge_contradictions invalid_at =
resolved_edge.valid_at). The newer row survives; the close is the existing
non-destructive soft_close (valid_to_ms UPDATE, never a row DELETE). A
soft_close NotFound (concurrent close) is treated as a no-op surfacing, never an
error that aborts the sweep (mirrors audit_conflicts:130). The smoke test asserts
the older row's valid_to_ms == newer.valid_from_ms.

CONFIDENCE-GATED WARN-THEN-ESCALATE POSTURE (FP guard, killer-risk 2)

SurfaceToUser is the SAFE DEFAULT: an InvalidatedOlder auto-close fires ONLY at
confidence >= SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD (0.85). Below threshold,
Neutral, Entailment, and any judge Err all return SurfaceToUser — both rows
survive. There is NO branch that auto-closes below threshold. A judge failure
NEVER auto-closes (fail-SAFE degrade). semantic_contradiction_threshold_contract.rs
asserts: below-threshold Contradiction -> SurfaceToUser; Entailment -> no close;
judge Err -> SurfaceToUser; the threshold constant is conservative (>= 0.80).

SSB REUSE (W-9, no new variant, no SCHEMA bump)

conflict_kind is a FREE-FORM String on the existing MemoryConflictDetectedPayload
(ssb.rs:16735; docstring 16730-16733: "future kinds can land without a schema
bump because the field is a free-form string"). F-12 emits conflict_kind=
"semantic_contradiction" through the EXISTING SsbActionType::MemoryConflictDetected
(enum at ssb.rs:12658, sql_str "memory_conflict_detected" at :8689). NO new
SsbActionType, NO sql_str/parse/golden change, SCHEMA_VERSION stays 207.
semantic_contradiction_ssb_reuse_contract.rs round-trips the payload through rmp,
asserts action_type() resolves to MemoryConflictDetected, and negative-greps
ssb.rs to assert no SemanticContradictionDetected variant snuck in. (Plan-cited
ssb line numbers had drifted ~+208 lines; the structure was confirmed at the real
lines by grep, not the literal plan anchors.)

audit_conflicts FIRST-CALLER NOTE (the wire-dark fix)

audit_conflicts (contradiction.rs:64) had ZERO production callers — its only
callers were three #[cfg(test)] tests. F-12 supplies its FIRST production caller:
the live memory tick (crates/cli/src/live_cli.rs dispatch_chat_memory_reflect)
runs audit_conflicts FIRST (auto_supersede_by_tier=false, keeping Surface-to-user
the default) so the byte-identical / tier-dominant cases resolve at zero model
cost, THEN runs collect_cross_key_candidates + adjudicate_semantic on the
cross-key residue. This closes a pre-existing wire-dark gap as a deliberate side
effect (bounded: ~one extra call per scanned fact_type; it does not widen the
storage/trigger contract).

LIVE-TICK WIRING (the &mut facts fix)

crates/cli/src/live_cli.rs:1786 `let facts = FactStore::open(...)` was an IMMUTABLE
binding; audit_conflicts / adjudicate_semantic / soft_close all need &mut FactStore.
Changed to `let mut facts`. The earlier &facts reads (decay_tick :1846,
collect_quarantined :1887, reflect :1956) reborrow immutably and still compile; the
&mut borrow is sequenced AFTER the F-5 quarantine block closes (~:1949), inside the
Ok(stats) arm of the decay_tick match, so the borrows never overlap. The semantic
pass mirrors F-5's kill-switch-default-ON (ELAI_SEMANTIC_CONTRADICTION_KILL, a
disable-only env added to dkg_lifecycle.rs alongside staleness_cascade_killed) and
the fire-and-forget SSB emit pattern (detached current-thread tokio runtime).
Scanned fact types: Preference, Opinion, DomainKnowledge (the user-modeling classes
most prone to paraphrase drift; bounded set keeps the funnel cheap).

FRONTEND PAIR

commands_e5.rs::get_semantic_contradictions reads the SSB VIEWER (NOT the
FactStore) for memory_conflict_detected rows filtered to conflict_kind=
"semantic_contradiction" (mirrors commands_c42 gateway_authz_recent_denies viewer
read; empty envelope when no viewer attached; defensive per-row decode tolerating
both the externally-tagged {"MemoryConflictDetected":{...}} and a flat payload
shape). Registered in lib.rs invoke_handler. stub.ts get_semantic_contradictions +
types.ts SemanticContradiction/SemanticContradictionsResponse + MemoryFactsDrawer
"Semantic Contradictions" section complete the Rust<->TS pair.

DEFERS (explicit, no scope-creep)

  - The DKG cascade exact-text join (knowledge/src/dkg/mod.rs concept==fact_id
    exact-equality) is a SEPARATE limitation needing embedding/alias resolution —
    NOT this F-N.
  - F-5 age-based quarantine STAYS as-is (Graphiti has no time-decay; that remains
    ELAI's edge — not a gap to close).
  - The mem0_dispatcher write path is NOT changed (the sweep is post-hoc over
    currently-valid rows, not per-ADD).
  - Tier-2 model staging (the NLI ONNX file + tokenizer) is ops-gated; until
    staged, the production tick runs the DeterministicNliJudge (Tier-1), so the
    pass is never dark.

GROUNDING STATUS

  mechanism (invalid_at = valid_at)      : MIRRORED from Graphiti (not imported)
  Tier-1 determinism                     : FORMAL (pure integer/string math)
  Tier-2 determinism                     : PRACTICAL, not bit-identical
                                           (arXiv:2506.09501 FP batch variance)
  no-cloud-judge                         : ENFORCED by construction + contract test
  NLI ONNX model                         : NET-NEW data artifact (runtime reused)
  recall-uplift                          : DESIGN HYPOTHESIS until measured pair-
                                           level precision on >= 20 real candidates
  SSB reuse / no schema bump             : CONFIRMED (conflict_kind free-form;
                                           SCHEMA stays 207)
  audit_conflicts first prod caller      : CONFIRMED (was wire-dark; now wired)
