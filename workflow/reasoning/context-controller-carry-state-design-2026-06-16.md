Reasoning + design: ContextController + CarryStateContract as a first-class ELAI primitive

PURPOSE: the user elevated P2/P3 of distill-reset-propagation-reasoning-2026-06-16.md
  ("the thing that propagates is the EXTERNAL CONTEXT CONTROLLER + a CARRY-STATE
  CONTRACT, not the distill-reset operator") to a headline primitive "almost as
  important as SSB", and made one correction to the earlier framing: this is NOT a
  local-models-only feature. A controller that owns context across resets lets a
  cloud model with a big window handle a MUCH bigger task too. This file designs the
  primitive concretely, grounds it in what ELAI already has (activation, not
  invention), applies the determinism doctrine, and carries the falsifier forward.
  It is reasoning + a design spec, not a plan file. A later session turns the
  accepted parts into plan files (candidate wave in section 8).

GROUNDING POSTURE: all ELAI internals below are grounded in
  docs/thefinalglance/ELAI-FEATURE-MAP.md (S1.3/S1.5, regenerated 2026-06-11,
  upserted through 2026-06-16) and the J-plan index (J-39/J-40/J-42 entries). The
  doctrine is grounded in docs/strategy/ELAI-DETERMINISM-DOCTRINE.md (D1-D5). The
  context-management literature is grounded in distill-reset-propagation-reasoning-
  2026-06-16.md (F1-F10, deltas REPORTED until the primary arXiv IDs are verified;
  InftyThink 2503.06692 / RULER / lost-in-the-middle / MemGPT 2310.08560 are the
  verified or high-prior anchors). No claim here is headlined on an unmeasured
  number; the whole primitive ships observe-only and falsifiable per section 7.

================================================================
1. THE ONE-SENTENCE THESIS
================================================================
ELAI should own a single harness-resident ContextController that, on a
  deterministic trigger, captures a typed CarryStateContract from a frozen model
  invocation and reseeds the next invocation from that contract instead of from raw
  history -- and this contract is the same object across four grains that today each
  carry state in an incompatible ad-hoc shape. The controller is to STATE what SSB
  is to EVIDENCE: an off-agent, harness-owned substrate the model writes into but
  does not get to trust-without-check.

================================================================
2. THE GAP IS UNIFICATION, NOT A NEW OPERATOR (activation, not invention)
================================================================
ELAI already does "distill -> carry -> continue" at FOUR different grains. Each was
  built separately, each invented its own carry shape, none share a base type. That
  is the defect the primitive fixes.

  GRAIN A -- within-step (one GRC / planning / research step): J-42 (PLANNED).
    Extends GRC reset_topic (engine.rs:556, today clears budget counters only, no
    distillate carry) into checkpoint_and_reset; carry = a free-form distillate.
    Emits ReasoningCheckpointEvent.
  GRAIN B -- session (one chat / task): E-25 compaction ladder (DARK), H-50
    regime-aware ContextPolicy (PARTIAL, context_policy.rs), H-51 cross-session
    sleep-time handoff (PARTIAL, handoff_digest.rs). Carry = a topic-digest /
    compaction tier. SESSION DAG (WIRED, 3-pass lossless trim, stubs recoverable
    from SSB seq) is the lossless backing ledger.
  GRAIN C -- cross-model / cross-tier handoff: I-11 PABU typed beliefstate
    (PLANNED, "replaces raw history on cross-model/tier handoffs, DKG-keyed").
    Carry = a belief-state object. This is ALREADY a carry-state contract -- for
    exactly one grain.
  GRAIN D -- agent / orchestration: the orchestrate + auto-orchestrate skills
    dispatch frozen worker agents whose context is GC'd on return, and the worker
    returns a structured STATUS/FILES TOUCHED/VERIFIED/QUESTION shape; the
    human-grain cross-session analog is handoff-protocol.md
    (SUMMARY/WORK DONE/FILES TOUCHED/USER CHOICES/HOW TO PICK UP/KNOWN LIMITS/
    GROUNDING). Carry = a worker return / a handoff entry.

  Observation that makes the whole design click: grain D is ALREADY "frozen executor
  + external controller carrying a typed state across a context boundary." The
  orchestrate skill is the ContextController, hand-operated by the main LLM, at the
  agent grain. handoff-protocol.md is the CarryStateContract, hand-written, at the
  human/session grain. PABU (I-11) is the CarryStateContract, typed, at the
  cross-model grain. J-42's distillate is the CarryStateContract, free-form, at the
  within-step grain. We have built the same idea four times in four shapes. The
  primitive is: ONE contract type + ONE controller that applies it at every grain,
  with the operator (what compression to run at the boundary) varying per axis/lane.

  So the deliverable is mostly RENAME + UNIFY + thread one type through the existing
  machinery, plus a thin controller that selects trigger+operator+policy. It is not
  a green-field build. This is the determinism-doctrine "activation not invention"
  posture and it lowers risk substantially.

================================================================
3. THE CLOUD REFRAME (the user's central new claim, and why it holds)
================================================================
Earlier framing (J-42, the reasoning file) treated this as a local-small-model
  horizon trick: local windows are tiny (measured 3B NF4+FA2 ~40K, 7B ~25K on 12GB),
  so you must reset to keep going. The user's correction: the controller is NOT
  local-only; it lets a big-window cloud model take on a MUCH bigger task. That is
  correct and it rests on three independent legs, only the first of which is about
  window size.

  LEG 1 -- CAPACITY (horizon > window, true at every window size). A single task can
    exceed even a 1M-token window: a 50-file migration, a 200-turn agentic build, a
    deep-research sweep over hundreds of sources, a multi-day plan walk. A model with
    a finite window can only complete an unbounded-horizon task if something carries
    state across logical resets. The window size sets WHERE the wall is, not WHETHER
    there is one. Bigger windows move the wall; they do not remove it. The controller
    is what lets a finite-window model (any size) act as if its horizon were
    unbounded. This is the MemGPT thesis (2310.08560: paged virtual memory; GPT-4
    DMR 32.1->92.5) applied at the ELAI harness grain.

  LEG 2 -- QUALITY (degradation within the window, independent of capacity). Big
    windows degrade well before they fill: RULER shows 32K-advertised models failing
    at 32K on hard tasks; lost-in-the-middle persists in long contexts; the J-34
    plan exists precisely because the real fill->degradation knee is far below the
    advertised window and must be MEASURED per (model, lane). So even when a task
    FITS, keeping the active window small and clean (sealed goal + 3K typed carry +
    anchors at the edges) holds quality higher than letting 180K of raw history
    accumulate. The controller is a quality lever, not only a capacity lever -- it
    fires at the measured knee, not at the window limit.

  LEG 3 -- COST + CACHE (cloud-specific economics). Cloud is priced per input token
    and ELAI already tracks this (COST COCKPIT H-9, TOKEN LEDGER J-28). Carrying a
    ~3K typed CarryStateContract instead of replaying a ~180K raw history across a
    logical reset is a large $ win on every continued turn. Crucially the cloud
    OPERATOR differs from the local one: cloud keeps a cache-stable prefix (H-50
    already encodes this -- cloud = big cache-stable prefix + late compaction; local
    = window-minimization + unload-early). So on the cloud lane a "reset" is
    curate-in-place / append-new-carry-keep-prefix, NEVER a hard prefix-burning
    reset (a hard reset on cloud destroys the KV cache and costs MORE). The
    controller selects reset-vs-curate by lane. This makes the primitive a cost
    optimizer on cloud and a capacity enabler on local, from one mechanism.

  Net: the controller helps EVERY backend, for different reasons -- capacity at all
  sizes, quality within the window, cost+cache on cloud. Calling it a local feature
  was the framing bug. It is a horizon/quality/cost primitive that is backend-shaped
  by the H-50 policy, not backend-gated.

================================================================
4. THE CARRY-STATE CONTRACT (the schema)
================================================================
One base contract + a typed per-axis payload. This is the SSB Pattern-2 idiom
  (single variant with a discriminator arm, e.g. RouteDecisionEventKind 3-arm) ELAI
  uses everywhere, so it is idiomatic and the action_type golden lock (C-27/E-45)
  already knows how to police it.

  BASE (axis-independent, always present):
    - sealed_goal: blake3(canonical objective bytes). Immutable across all resets in
      the task. Composes I-45 ObjectiveSealed/ReceiptStale -- a goal mutation after a
      checkpoint is a detectably stale carry, not a silent drift. This is the
      lost-in-the-middle / goal-drift anchor; it rides at the FRONT of every reseed.
    - extractive_anchors: verbatim spans + stable IDs (symbol ids from the DKG,
      SSB seq, source fingerprints). HARNESS-EXTRACTED from the model's proposed
      carry and span-containment-checked against the immutable raw ledger (section
      6). Extractive beats abstractive by the F5 evidence (ICML-2024 prompt
      compression +3-5 pts, up to 10x; CompAct up to 47x). These ride at the EDGES of
      the reseed (front+end) to fight lost-in-the-middle.
    - open_obligations: the unsolved-branch list (what is still owed). Composes
      H-54 OBLIGATION-CLOSURE-GATE (sorry/admit/unresolved anywhere disqualifies).
      Carrying open obligations forward is what stops a reset from silently dropping
      a half-finished branch (F8 attractor-drift mitigation: keep unresolved
      branches explicit).
    - uncertainty: typed confidence / abstention fields, NOT prose hedges. Composes
      D-16 RefusalDecision (read, never re-derived). This is the F8 calibration
      mitigation: self-distillation suppresses uncertainty (2603.24472, up to -40%
      OOD), so uncertainty must be a carried FIELD the harness preserves, and
      confidence must be estimated by verifiers not by re-reading the distillate.
    - provenance: which SSB rows / ingest sites / DKG symbols this carry was derived
      from. Composes G-1 taint (Provenanced<String>, 4 ingest sites) + G-2 RRA. A
      carry with no provenance is a fail-closed reject (D3 missing-evidence-is-FAIL).
    - raw_pointer: a SESSION DAG seq range (and SSB seq) that recovers the FULL
      pre-reset trace losslessly. The carry is never the only copy. This is the F8
      error-accumulation mitigation and the H-49 time-travel / RV-1 divergence-guard
      composition: a lossy carry can always be reconciled against the immutable raw
      trace, and a reset is reversible.
    - synopsis: a SHORT abstractive synopsis ON TOP of the above, never alone (F5).
      Free-form, model-authored, advisory. It is the human/LLM-readable gloss; the
      load-bearing state is the typed+extractive fields, never the prose.

  AXIS PAYLOAD (exactly one; the discriminator):
    - Reasoning: partial derivations, the current proof/argument frontier, rejected
      lines (so they are not re-explored -> attractor mitigation). (J-42 grain.)
    - Planning: completed-subtask outputs + downstream inputs + the open-subtask
      frontier (the "fold completed subtask, keep only outputs + anchors" operator).
      (J-39 / phase-4E grain; Context-Folding family.)
    - Research: ranked evidence spans + per-sub-question state + queries already run
      (so they are not re-run). (J-40 grain; ReSum/MEM1/ACON family.)
    - Orchestration: worker-return digests + the task-graph cursor + threaded
      cross-agent contracts (the denylist/schema/whitelist that wiring-enforcement
      R10 says must be a threaded REQUIRED input, not an implicit assumption). This
      is the grain the orchestrate skill runs by hand today; typing it makes worker
      handoffs machine-checkable. (Grain D.)

  Answering the open question Q1 from the reasoning file ("one schema or three
  siblings?"): ONE base + a typed axis discriminator. The slots genuinely differ per
  axis (derivations vs subtask-outputs vs evidence-spans), but the base
  (goal/anchors/obligations/uncertainty/provenance/raw-pointer/synopsis) is provably
  shared -- it is exactly what handoff-protocol.md, PABU, the worker return, and the
  J-42 distillate all independently re-derived. So: not three schemas, not one flat
  schema -- one base with four payload arms. PABU (I-11) becomes the Orchestration/
  cross-model arm's typed payload; it is not a separate thing.

================================================================
5. THE CONTROLLER (what is harness-owned vs model-owned)
================================================================
ContextController is a deterministic harness component (crates/orchestrator). It
  owns four decisions, all of which the determinism doctrine assigns to the harness
  (SIZING / ADMISSION / STATE), and delegates exactly one act to the model
  (GENERATION of the proposed carry).

  D-CONTROLLER-1  WHEN to checkpoint (trigger). Deterministic. The active-context
    fill crosses the J-34 measured per-(model,lane) degradation knee (NOT the
    advertised window, NOT a hardcoded 0.85, NOT a model-decided "I should
    summarize now" -- F6 says model-decided folding causes the pathologies). Until
    J-34 ships the measured curve, fall back to the H-50 fraction + H-42 serial-depth
    budget. The model has zero input into the trigger (mirrors D-10
    select_grounding_mode: the LLM has no input into mode selection).

  D-CONTROLLER-2  WHICH operator + reset-vs-curate (policy). Deterministic, by
    (axis, lane). Axis picks the compression shape (reasoning=summarize-continue;
    planning=fold-completed-subtask; research=search-summarize-resume;
    orchestration=worker-return-digest). Lane picks reset-vs-curate (H-50: local =
    hard reset + reseed for window-minimization; cloud = curate-in-place + keep
    cache prefix). This is a pure function of state, table-driven, no LLM.

  D-CONTROLLER-3  ADMISSION of the proposed carry. Deterministic. The model proposes
    a free-form carry (HD-3: never constrained-decode the reasoning/planner/
    researcher role). The harness then EXTRACTS the typed slots and verbatim anchors
    and VALIDATES: every extractive_anchor must span-contain against the raw ledger
    (a hallucinated anchor is structurally rejected, like H-53 closed-set citation
    binding); provenance must resolve to real SSB/DKG ids; sealed_goal must match
    (else I-45 stale-receipt). A carry that fails admission is REJECTED (fail-closed,
    D3) -> escalate per I-7 cascading escalation (re-propose once, then escalate to a
    bigger lane / surface). Missing carry = FAIL, never SKIP.

  D-CONTROLLER-4  EMIT off-agent evidence. Deterministic. One CarryStateCheckpoint
    SSB row per checkpoint {grain, axis, lane, trigger_reason, carry_tokens,
    raw_tokens, anchors_count, uncertainty_preserved:bool, admission:
    Admitted|Rejected|Escalated, outcome: ResetAndContinued|CuratedInPlace|
    EscalatedToCloud}. This is the D2/D4 leg: the controller's behavior is auditable
    from a row the MODEL did not author, and the falsifier (section 7) reads these
    rows. (J-42 already specs ReasoningCheckpointEvent for its grain; this
    generalizes it -- one action_type with a grain discriminator, not four.)

  MODEL-OWNED (the only thing): GENERATION of the proposed carry text + synopsis. It
    is generative and there is no deterministic oracle for "write a good summary of
    your reasoning so far," so it is legitimately the model's job (D-boundary
    GENERATION). Everything else -- when, which, admit, record -- is the harness's.
    If an LLM is ever used to JUDGE carry quality, it is advisory + cross-family
    (D5), never the admission gate; admission is the mechanical span/provenance check.

================================================================
6. WHY THIS IS "ALMOST AS IMPORTANT AS SSB" (the user's framing, made exact)
================================================================
SSB and CarryState are siblings, and the analogy is structural, not loose:
  - SSB = an append-only, single-writer, off-agent record of what HAPPENED (events,
    verdicts, counts). The model writes into it; gates read from it; the model never
    gets to claim a fact SSB does not contain. It is the EVIDENCE substrate.
  - CarryState = a controller-owned, typed, off-agent record of what we KNOW NOW and
    WHERE WE ARE (state, frontier, obligations), with the raw trace recoverable. The
    model proposes into it; the controller admits it; the next invocation reseeds
    from it. It is the STATE substrate.
  Both exist because the model cannot be trusted to hold the thing itself: SSB
  exists because a model's stdout-claim of "the test passed" is not evidence (F-4
  cant-lie gate); CarryState exists because a model's raw 180K history is not a
  reliable place to keep state across a horizon (RULER/lost-in-the-middle), and its
  own free-text recap is lossy and overconfident (F8). SSB makes evidence
  un-fabricable; CarryState makes state un-lose-able. They are the two halves of
  "the model is a cog": SSB polices what the model CLAIMS, CarryState polices what
  the model REMEMBERS. That is why it ranks with SSB and not below it.

================================================================
7. FALSIFIER + ROLLOUT (carried from P6/EXP-3, now per-grain)
================================================================
Ships OBSERVE-ONLY, default-OFF, promoted per (axis, lane) independently. No grain
  goes default-ON on faith; each must beat its controls on a matched-budget
  ablation. This is the determinism doctrine applied to OUR OWN thesis (P5): the
  unified-policy-across-axes claim is novel and unproven (F10 -- no published system
  shows one context policy across reasoning+planning+research on one small model),
  so it must be instrumented and falsifiable, never headlined.

  THE ABLATION (per axis, >=5 seeds), budget matched on FOUR axes simultaneously:
  generated tokens + sum-of-squared active-context-length (an attention-cost proxy)
  + tool-call count + wall-clock. Four conditions:
    (1) straight-line (no controller) -- baseline.
    (2) controller with FREE-TEXT-only carry (synopsis, no typed/extractive) --
        NEGATIVE CONTROL. F3 (Accordion cold-start) predicts this can REGRESS a
        small model; it exists to prove the typed+extractive fields are what carry
        the gain, not the reset itself.
    (3) controller with the full typed+extractive CarryStateContract -- the design.
    (4) external-retrieval/memory baseline (E-1 A-MEM / RAG) -- the strong control.
  REPORT accuracy AND calibration (Brier / ECE). PROMOTE condition (3) to default-ON
  for that (axis, lane) ONLY IF it beats BOTH (1) straight-line AND (4) retrieval at
  matched budget, AND does not win by becoming more overconfident (worse ECE). If it
  only ties at higher overconfidence, it is rejected for that axis (it became a
  self-distillation death-spiral, F8). The gate reads the CarryStateCheckpoint SSB
  rows (off-agent) -- the model does not score its own controller.

  SEQUENCE BY EVIDENCE (P4): research axis first (strongest training-free evidence:
  ReSum training-free, ContextCurator, MEM1, ACON 26-54% savings; composes the
  already-authored J-40 lane). Planning second (Context-Folding/AgentFold family;
  composes J-39 / 4E). Reasoning last and gated hardest (weakest training-free,
  F3 can regress; this is J-42, correctly the riskiest leg; extractive carry
  mandatory or train the model -- RunPod-gated SFT/InftyThink+ is the only
  strong-reasoning-quality path and stays out of the training-free near-term).
  Orchestration grain is ALREADY in production by hand (orchestrate skill); typing
  its carry is low-risk and can land alongside research.

================================================================
8. CANDIDATE PLAN WAVE FOR A LATER SESSION (gated on arXiv verification per the
   reasoning file section 0, and on a second-model diversity vote)
================================================================
  K-? CONTEXTCONTROLLER + CARRYSTATECONTRACT [keystone] -- the section 4 base type
    (Pattern-2 SSB variant) + the section 5 controller in crates/orchestrator;
    +CarryStateCheckpoint action_type (one, grain-discriminated); subsumes J-42's
    ReasoningCheckpointEvent and I-11 PABU into typed arms; threads the contract
    through SESSION DAG (raw_pointer), H-50 (reset-vs-curate policy), J-34 (trigger
    knee), context_budget 3E (reseed layout: anchors at edges, HD-2 cache-safe
    ordering). Grounding: MemGPT, ContextCurator, AdaCoM, MEM1, ACON.
  K-? RESEARCH-AXIS OPERATOR wired into J-40 (search-summarize-resume). FIRST.
  K-? PLANNING-AXIS OPERATOR wired into J-39 / 4E (fold-completed-subtask). SECOND.
  K-? J-42 REWRITTEN as the reasoning-axis operator over the unified contract
    (extractive carry mandatory; free-text = negative control). LAST, gated hardest.
  K-? ORCHESTRATION-AXIS: type the orchestrate worker-return + handoff-protocol entry
    as the Orchestration payload arm; lands with research (low risk, already manual).
  Cross-cutting: every arm carries the F8 mitigations (immutable raw ledger, typed
    uncertainty, anchors-at-edges, no-new-claims admission) and the section-7
    falsifier as its observe-only step 1.

================================================================
9. OPEN QUESTIONS FOR THE PLAN-AUTHORING SESSION
================================================================
  - Reset-vs-curate boundary on cloud: H-50 says cloud keeps the cache prefix, so a
    cloud "checkpoint" is append-carry-keep-prefix. Confirm the controller never
    issues a prefix-burning hard reset on the cloud lane, and decide whether cloud
    EVER hard-resets (e.g. when even the curated prefix crosses the J-34 knee on a
    1M window -- LEG 1 capacity wall). Likely yes at the true horizon limit.
  - Does the Orchestration arm REPLACE handoff-protocol.md's free-text entry, or sit
    UNDER it (typed fields + the existing prose synopsis on top)? Lean: under it --
    handoff-protocol.md is human-facing prose, the typed arm is the machine half,
    same synopsis-on-top pattern as section 4.
  - I-11 PABU: fold into this keystone as the cross-model payload arm, or keep PABU
    as its own plan and have it IMPLEMENT this contract's cross-model arm? Lean:
    PABU implements the arm (it already specs DKG-keyed typed beliefstate); the
    keystone defines the base type PABU conforms to.
  - Verify the REPORTED-UNVERIFIED arXiv IDs (reasoning file section 0:
    ContextCurator 2604.11462, AdaCoM 2605.30785, MEM1 2506.15841, ReSum 2509.13313,
    ACON, Context-Folding 2510.11967 / AgentFold 2510.24699) BEFORE any becomes a
    load-bearing plan citation. Run a GPT/Grok deep-research diversity pass.
  - J-34 dependency: the trigger knee is only as good as the measured curve. The
    controller's quality leg (section 3 LEG 2) is gated on J-34 shipping real
    fill->degradation curves; until then the trigger is the coarse H-50 fraction.

================================================================
10. BOTTOM LINE
================================================================
The user is right that the controller + carry-state contract is a headline primitive
  and right that it is not local-only. The design is: ONE typed CarryStateContract
  (base + four axis arms, SSB Pattern-2 idiom) carried by ONE harness ContextController
  across the four grains ELAI already manages by hand or by sibling code (within-step
  J-42, session E-25/H-50/H-51, cross-model I-11/PABU, orchestration the orchestrate
  skill + handoff-protocol.md). The controller owns when/which/admit/record
  deterministically; the model only generates the proposed carry; the raw trace stays
  immutable and recoverable. It helps cloud (capacity beyond the window + quality
  within it + cost/cache savings, reset-vs-curate by lane) as much as local, for
  different reasons. It is to STATE what SSB is to EVIDENCE. It ships observe-only,
  default-OFF, and is promoted per axis only when it beats both straight-line and
  retrieval at matched budget without worse calibration -- the determinism doctrine
  turned on our own thesis. The work is unification of existing parts under one
  contract + a thin controller, not a green-field operator.
