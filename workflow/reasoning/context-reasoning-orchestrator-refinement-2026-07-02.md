# Context + Reasoning Orchestration Refinement — gap analysis + implementation plan
Generated 2026-07-02, main LLM in-band, after a full read of phase-index-MASTER + all B..L SUMMARY_INDEX files
plus targeted deep-reads of the context/reasoning waves (J-34/35/36/37/38/42, J-72..76, J-83..86, J-95..98,
J-103..108, K-12/14/15/16/17, L-16..19, H-6/H-30..38/H-42/H-50/H-51, I-5/I-6/I-11/I-14, 3E/3H).
Purpose: the driving question is "if small / variable-context models must do what a flagship does, EVERYTHING
falls on context management — and not just context management, REASONING context management. ELAI must be the
orchestrator; we can never depend on the model to write its own handoff." This doc is the artifact a follow-on
session orchestrates implementation from.

STATUS ASSUMPTION (user-stated): B..I = built (possibly dark / unwired); J..L = NOT started. Therefore:
- ANY behavioral change to a B..I unit ships as a NEW plan file (house rule feedback_new_plan_files_only).
- New sessions below are proposed as NEW J-family files J-109..J-116 (per the standing K-native->K /
  technical->J ruling; K-10 bars new LETTERS). Never amend an existing J/K/L file in place either — the
  J-76 precedent ("new file, never amend J-34/H-50 in place") binds.
- Every referenced B..I symbol is treated as shipped-maybe-dark: the executor re-greps at edit (W-7) and
  ACTIVATES dark substrate rather than rebuilding it (GD-1).

================================================================================
## 1. VERDICT — what the existing corpus already gets right (do NOT re-plan these)
================================================================================
Measured against a first-principles requirement list for "harness owns the context of an arbitrary small
model", the J/K/L design is genuinely close to what I would design from scratch. The follow-on session must
NOT re-author any of the following; the gaps in section 2 compose with them.

R-A KNOW the envelope per model.
  Covered: J-34 (measured per-(model,lane) fill-degradation curve + knee + positional-value curve replacing
  the 0.85 folklore; ModelCapabilityProfile gains calibrated fields; cache-safe fixed-per-model layout),
  K-15 (reasoning-budget knee), J-76 (chunk cap + head/tail placement consuming J-34's curve), I-51
  (models.toml capability catalog), J-9 (per-lane hardware eligibility), E-32 (32K lift + FITM).
  GAPS: token-COUNT truth per backend (sec 2, J-110); envelope for UNKNOWN backends at first use (J-111).

R-B OWN the transcript (never depend on the model's memory of it).
  Covered: L-16..L-19 (immutable transcript + pure fold-view; CC-1..CC-4; freeze-epoch causes; view-level
  losslessness; page-in ladder). This is the single best architectural decision in the corpus: it makes
  HD-2 cache stability STRUCTURAL, makes recover/undo/branch/compaction parameterizations of one fold(),
  and makes model-swap and window-change survivable by re-folding. It must be sequenced FIRST (sec 3).

R-C DECIDE per-turn context composition deterministically.
  Covered piecewise: 3E slot allocator (+floors, breakpoints), H-50 regime split (cloud cache-stable vs
  local window-min), H-6 masking + selective recover_stubs, I-5 pristine injector (contrastive constraint,
  never raw tracebacks), I-6 anchor pane LAST + ack tag, I-14 deterministic compression (LSP slice + AST
  fold + reversible receipts; ciphers banned IC-3), J-73 retrieval abstention, J-103 read-access ladder
  (escalate-on-miss, downgrade-not-deny), J-104 overlay-before-base freshness, J-95 activation wires.
  GAP: no single OWNER/decision-point — the "ContextController/CarryState" design is referenced once
  (J-43 header) as the doctrine's first downstream application but NO session builds it (grep-verified,
  zero hits outside that one line). Sec 2, J-109.

R-D CARRY reasoning state across resets, handoffs, and model swaps — the model proposes, the harness owns.
  Covered: I-11 BeliefState (typed, versioned, honesty-measured) -> J-83 GRC-v3 carrier (per-entry
  {claim, receipt, abstention-bit-from-H-39, DKG provenance}; commit-and-freeze; least-to-most-with-
  receipts), K-12 critical atoms + handoff-faithfulness gate (exact-match, deterministic, never semantic),
  J-42 within-step distill-reset sawtooth (harness disposes when-to-reset; uncertainty-preserving
  admission), H-42/J-84 depth-scaled budgets + 2-axis effort classifier, J-85 overthinking guard,
  J-86 receipt-gated candidate ladder, K-16 RR-1..RR-4 (trace never admissible as confidence; receipt >
  vote > NEVER self-eval; measure-your-own-knee; commit-and-freeze).
  Together J-83..J-86 + J-42 + K-15/K-16 ARE the "reasoning context management orchestrator" the user is
  asking for. The refinement is sequencing (early, not late) + closing the resume grain (J-112) and the
  window-aware decomposition loop (J-113).

R-E VERIFY context integrity at runtime.
  Covered partially: I-6 requires an acknowledgement tag per generation; E-49/J-34 context_rot_probe is
  bench-time; K-12 gates handoffs. GAP: nothing validates the ack deterministically in-session and ACTS on
  failure (J-114).

R-F REDUCE hallucination via the context path.
  Covered: J-38 context-sufficiency gate + mid-task retrieval loop, H-53 citation pre-binding (closed set,
  ghost cites structurally impossible), J-37 4-state staleness adjudication + UNKNOWN-blocks-premise ->
  D-16 abstain, I-13 memory guard, J-105 memory ROI attribution, H-31/H-32 write-time supersession +
  validate-before-use + auditor firewall, L-T1 CGG (typed claims, convergence fuse, do-test, policy gate),
  L-10 measured hallucination-delta. GAP: the hallucination x context-fill COUPLING is never measured
  (J-116) — for small models this is the number that tells the controller where hallucination risk starts.

R-G HETEROGENEOUS free-API / local model reality.
  Covered: B-37 11-backend adapter, J-54 free-api rotation + J-55, J-30 BYOK, I-36 per-(role,backend)
  compiled prompts, I-51 catalog, J-36 frozen-head local KV reuse with per-model eligibility guard.
  GAPS: count truth (J-110), first-use probe (J-111), mid-task swap re-projection (J-115).

Explicitly REJECTED here so it is never re-litigated (consistent with existing doctrine):
- generative/cipher compression as a default (IC-3 ban; LLMLingua-class stays Tier-2 prose-only per I-14);
- chasing bigger windows as the fix (RD-4: window buys access, not depth; the lever is placement + budget);
- any new memory subsystem (activation of crates/memory + J-37/J-105 instead);
- a second index for freshness (J-104 overlay covers it);
- LLM-written handoffs as the load-bearing resume artifact (the whole point of J-112);
- porting Context-Warp-Drive wholesale (L-16 already adopts the one idea that matters).

================================================================================
## 2. THE GAPS — proposed NEW sessions J-109..J-116
================================================================================
All grep-verified absent from B..L before authoring (notably: ZERO hits for tokenizer / token estimator
calibration anywhere in docs/thefinalglance; ONE dangling reference to ContextController; no session-boundary
projection; no runtime ack validation). Every entry follows the house shape. All inherit W-1 run-trigger
done-gates, W-2 no-default-off-graveyard, GD-1 compose-not-restate, GD-2 journal-recomputable, GD-4 FP-bounded
before default-ON, HD-2 cache-prefix safety, TD-3 LLM-never-adjudicates, ID-5/HD-1 ELAI-measured numbers,
CL-1..CL-4, K-16 RR-1..RR-4 where reasoning-adjacent. SSB deltas are ship-order-cumulative off the LIVE
constants (re-grep ssb.rs / action_type_lock.rs at edit, W-9 — every stamp in this doc is presumed stale).

### J-109 — ContextController: one deterministic per-turn context plan + journaled receipt [KEYSTONE]
- lever OWN-THE-CONTEXT | nature BUILD (unification; the J-43-referenced design, unauthored until now) |
  effort L | depends L-16/L-17 (the fold it drives), J-34 (the calibration it consumes), H-50 (the regime
  policy it subsumes as data), J-84 (the reasoning-budget input); composes 3E, H-6, I-5, I-6, J-73, J-76,
  J-103, K-12, J-83.
- Build: ONE deterministic decision point that, per model call, computes a typed ContextPlan:
  {backend, measured_window, calibrated_knee, slot layout (3E order per J-34 positional curve), frozen-head
  boundary (J-36), anchor content (I-6), reasoning-carry slice (J-83 carrier size + J-42 distillate
  admission, budget from J-84/H-42), retrieval slice (J-38/J-73 verdicts), read-ladder rungs in effect
  (J-103), compression tier per payload (I-14), page-in budget band (L-19), recall/recovery pointers}.
  The plan is EXECUTED by L-16's fold(log, budget, policy, freeze) — the controller chooses the parameters,
  the fold produces the bytes; nothing else assembles context. Every plan lands as ONE ContextPlanApplied
  SSB receipt (inputs + chosen parameters + resulting token counts), journal-recomputable (GD-2). The
  scattered mechanisms become strategies the controller selects — they are NOT re-implemented (GD-1 P0).
  The LLM never picks any parameter (TD-3). Per-ROLE context contracts as DATA (max tokens, allowed slots,
  provenance filters incl. the H-31 auditor zero-memory firewall) live here as the controller's config.
- Integration: crates/orchestrator/src/context_controller.rs (new) seated where conversation.rs currently
  assembles turns (post-L-16 rewire); consumes context_calibration.rs (J-34) + context_policy.rs (H-50,
  demoted to a data table) + the fold. +ContextPlanApplied (Pattern-2; PlanComputed / PlanExecuted /
  FloorInfeasible; baseline+1). frontend_pair: ContextStatePanel plan row (decide at edit, W-4).
- observable_effect: a real local turn and a real cloud turn each land ONE ContextPlanApplied whose chosen
  trim/placement/carry values match the J-34 artifact for that model (not folklore constants) and whose
  resulting request bytes are byte-reproducible via fold under ELAI_SEED; the HD-2 prefix assert holds
  across turns. NEGATIVE (R2): with the controller disabled, context is assembled by the legacy path and
  says so loudly; a context assembly that bypasses the controller (no receipt row for a model call) FAILS
  the contract test; an LLM-proposed slot layout is ignored.

### J-110 — token-count truth: per-backend estimator calibration against provider-reported usage
- lever PROVABLE+CHEAPER (budgets are only as real as the counter) | nature BUILD (small) | effort S-M |
  WAVE 0 | depends J-28 (UsageTurnEvent actuals — the ground truth ALREADY captured); composes 3E, I-51,
  E-39, H-9.
- Build: grep-verified gap — no plan anywhere calibrates ELAI's token counting per backend; 3E uses
  token_count_estimate/exact against ONE tokenizer assumption while B-37 serves 11+ backends and free APIs
  whose tokenizers differ. J-110: (1) a per-backend calibration loop — after every completed call, compare
  the journaled PREDICTED prompt tokens vs the provider-REPORTED prompt tokens (J-28 rows); maintain a
  per-(backend, model) correction factor + a safety margin, persisted next to I-51's catalog rows with a
  staleness date; (2) 3E/J-109 consume the corrected counter; (3) a drift alarm: correction drifting >
  a bounded % in-session emits TokenCountDrift and widens the margin (fail-toward-headroom, never
  fail-toward-overflow); (4) local lane: count via the model's own tokenizer where available (sidecar
  exposes it), corrected the same way.
- Integration: crates/orchestrator/src/token_calibration.rs (new, small) + the 3E counting seam + an I-51
  catalog column. Prefer-reuse J-28 rows; +TokenCountCalibrated only if no reusable row (net 0..+1).
  frontend_pair: predicted-vs-actual on the H-9 cockpit (decide at edit).
- observable_effect: after N real calls on a backend, predicted-vs-actual error is measured and shrinks
  under the correction (journal-recomputable); a deliberately mis-tokenized fixture backend produces the
  drift alarm + margin widening BEFORE a context_window_blocked error. NEGATIVE: a request that would
  exceed the corrected window is re-folded at a smaller budget (never sent to fail); zero J-28 rows =
  conservative default margin + a logged unavailability, never silent trust in the estimate.

### J-111 — first-use backend envelope probe (unknown free-API / local models)
- lever REACH+SAFETY | nature BUILD | effort M | WAVE 1 | depends J-110 (counting), J-34 (the full bench it
  triages FOR); composes I-51, B-37, J-54, J-9, G-10.
- Build: I-51's catalog is static data and J-34's bench is heavy; a user pointing ELAI at an arbitrary
  free-API model id or a local GGUF has NO measured envelope. J-111 ships a CHEAP first-use probe, run once
  per new (backend, model) identity and cached in the catalog: advertised window (from /v1/models where
  exposed), a 3-5 point fill spot-check (constraint-recall at stepped fills — the E-49 probe shape, minutes
  not hours), token-count calibration seed (J-110), and a conservative interim knee (min(advertised x
  measured-floor-fraction)). The full J-34 curve remains the authoritative artifact, scheduled when the
  model earns real usage; G-10 identity drift marks probes stale. Until probed, the model runs at a locked
  conservative envelope — degraded-loudly, never assumed-flagship.
- Integration: extend context_calibration.rs (probe tier) + an I-51 catalog write + a first-use hook at the
  B-37 resolve_backend seam. Prefer-reuse ContextCalibrationEvent (+Kind ProbeMeasured); net 0. frontend_pair:
  a probe badge on RuntimePanel rows.
- observable_effect: registering a never-seen model id triggers exactly one probe run whose result lands in
  the catalog + a ProbeMeasured row; subsequent calls use the probed envelope (visible in ContextPlanApplied).
  NEGATIVE: an unprobed model NEVER gets the flagship default envelope (contract test); a probe failure =
  conservative floor + loud log, not a crash and not silence.

### J-112 — deterministic resume: the session boundary as a projection, never a model-authored file
- lever CANT-LIE-TO-YOURSELF (the user's directive verbatim: ELAI writes the handoff, not the model) |
  nature BUILD-on-L-16 (extends H-51's behavior via NEW file — H-51 is B..I, untouched) | effort M |
  WAVE 2 | depends L-16, J-83, K-12; composes H-51, H-6/H-50 selective recovery, H-30 staleness instrument,
  I-11 honesty job, G-2 RRA.
- Build: today's cross-session resume (H-51) authors a digest via compact_session — an LLM summarization —
  gated after the fact by K-12 atoms. J-112 inverts the load-bearing order: the PRIMARY resume artifact is
  DERIVED, not authored — {the J-83 typed carrier (claims + receipts + abstention bits), K-12 critical
  atoms, FSM cursor + DAG tip, open-stub pointers, the J-110-corrected budget} assembled by pure fold over
  the journal at the session boundary (fold(log, resume_budget) — L-16's projection applied at the
  cross-session grain). The LLM-written prose digest survives ONLY as an advisory annotation attached to
  the derived artifact (human-readable color, never load-bearing, never the thing the next session trusts).
  Resume = re-fold + carrier injection; every byte of the load-bearing artifact is journal-recomputable
  (GD-2) and K-12-gated (exact-match atoms). Tiered load semantics (digest-first ~1-3K, detail via
  selective recover_stubs) inherited from H-51 unchanged.
- Integration: crates/orchestrator/src/resume_projection.rs (new) + the /continue + auto-detect seams
  (H-51's); the H-51 digest writer demoted to the annotation lane. Prefer-reuse HandoffLifecycleEvent
  (+Kind DerivedResumeBuilt); net 0..+1. frontend_pair: resume offer card shows "derived (verified)" vs
  "advisory prose" provenance split.
- observable_effect: end a seeded session, resume fresh: the injected resume slice is byte-identical to an
  independent fold over the journal (GD-2), every K-12 must_preserve atom present exact-match, total load
  <= the tier budget; the prose annotation is provably ABSENT from the auditor/critic role assembly (H-31).
  NEGATIVE (R2): a resume where the derived artifact is missing but prose exists REFUSES to load prose as
  state (fails closed to cold start + loud surface); a paraphrased atom does not satisfy the gate (K-12);
  a model-authored edit to the derived artifact is detectable (hash mismatch vs re-fold) and rejected.

### J-113 — window-aware decomposition closure: floor computation + decompose-or-route, never truncate
- lever SMALL-MODEL-VIABILITY (the load falls on decomposition exactly when the window is small) | nature
  EXTENSION (new file; extends J-39's sizer + composes J-84) | effort M | WAVE 2 | depends J-34 + J-110
  (real knee + real counting), J-39; composes H-42/J-84, I-7 escalation, 3G phase-sizing, B-27 D-2 LOC
  constants (superseded by measured values, via new file not amendment).
- Build: J-39 sizes decomposition nodes deterministically but against static tier constants; nothing closes
  the loop "this node's MINIMUM VIABLE CONTEXT (system+rules floor + anchor + carrier + retrieval slice +
  work window, all J-110-counted) exceeds this model's measured knee". J-113: (1) a per-node floor
  computation from the ContextPlan inputs; (2) the closure rule — floor > knee => the orchestrator
  DECOMPOSES FURTHER (re-invoke the J-39 sizer at finer grain) or ROUTES (I-7 escalation to a
  larger-window lane), decided deterministically by cost policy; TRUNCATION IS NEVER THE RESOLUTION (the
  fail-closed arm: a node that cannot be decomposed below the floor and cannot route is surfaced, not
  squeezed); (3) the measured knee replaces the static per-tier LOC constants as the sizer's input where
  a J-34 artifact exists (fallback to constants + logged fallback otherwise).
- Integration: extend the J-39 sizer module via a new floor_check leg + the I-7 route hook; consumes
  context_calibration artifacts. Prefer-reuse J-39's event family (+Kind FloorInfeasible / DecomposedFiner /
  RoutedForWindow); net 0..+1. frontend_pair: plan-view badge on re-decomposed nodes.
- observable_effect: a fixture task on a 8K-window fixture model decomposes into MORE, SMALLER baby steps
  than the same task on a 128K model (journal shows FloorInfeasible -> DecomposedFiner), and every
  dispatched node's floor <= knee; the same task completes on both. NEGATIVE (R2): a --truncate-instead
  broken arm ships a node whose floor > window and FAILS its own step (proves the closure is load-bearing);
  with no J-34 artifact the sizer uses constants + logs the fallback (never a silent pseudo-measurement).

### J-114 — in-session context-health canary + deterministic re-anchor action
- lever CANT-DRIFT | nature EXTENSION (I-6 + K-12 at run grain) | effort S-M | WAVE 3 | depends I-6, L-17;
  composes J-34's constraint-recall probe shape, J-85 (distinct: J-85 watches the ANSWER oscillate, J-114
  watches the CONSTRAINT disappear), anti-spiral (distinct: repetition).
- Build: I-6 already demands a per-generation acknowledgement tag of the active constraint; nothing
  VALIDATES it or acts on failure. J-114: (1) deterministic ack validation — exact/normalized match of the
  emitted tag against the anchor's active constraint (TD-3, string check, no LLM); (2) scheduled canaries
  at cheap boundaries (post-compaction, post-page-in, every N turns on long sessions): a zero-cost check
  that the K-12 critical atoms still appear in the CURRENT view (fold introspection, no model call) + the
  ack check on the next natural generation; (3) the repair action — a failed check triggers re-anchor /
  re-fold at a smaller budget / atom re-surface (L-19 tier-2 recall), journaled; repeated failure escalates
  (I-7) rather than continuing on a drifted context.
- Integration: context_controller.rs (the check + repair live where the plan lives) + the I-6 tag seam.
  +ContextHealthEvent (Pattern-2; AckVerified / AckFailed / AtomMissingFromView / Reanchored; baseline+1)
  or prefer-reuse ContextPlanApplied kinds. frontend_pair: health dot on ContextStatePanel.
- observable_effect: a seeded long session past several compactions shows periodic AckVerified rows; a
  fixture that deliberately drops the anchor (broken-fold arm) produces AckFailed -> Reanchored and the
  NEXT turn's ack passes; atoms provably present in the view at every checkpoint. NEGATIVE (R2): the
  broken arm without the canary ships a constraint-violating output that the canaried arm catches; a
  fuzzy/semantic ack match is banned (exact only, K-12 discipline).

### J-115 — model-swap-safe mid-task re-projection [may merge into J-109 at authoring; kept separate for the swap-specific invariants]
- lever HETEROGENEOUS-REALITY (free-API rotation + BYOK means the model changes mid-task) | nature
  BUILD-on-J-109/L-16 | effort S-M | WAVE 3 | depends J-109, J-83, J-110/J-111; composes J-54 rotation,
  H-7/I-7 routing, I-36 compiled prompts, I-10 cache economics, G-10 drift canary.
- Build: when the backend changes mid-task (rotation, escalation, user /model), the context is NOT carried
  as bytes — it is RE-FOLDED for the new envelope: new ContextPlan (J-109) at the new measured window +
  corrected counter (J-110/J-111), the J-83 carrier re-injected (typed state survives; raw transcript
  re-projected), the I-36 compiled prompt variant for the new (role,backend) swapped in, cache economics
  re-planned (a swap is always a full cache miss — I-10's planner must never count the old prefix), and a
  SwapProjection receipt lands binding (old identity, new identity, carrier hash, re-fold budget). K-12
  atoms gate the swap exactly like a handoff (a swap IS a handoff).
- Integration: the B-37 swap/rotation seam + context_controller.rs. Prefer-reuse ContextPlanApplied /
  BeliefStateEvent (+Kind SwapProjected); net 0..+1. frontend_pair: swap row on RuntimePanel.
- observable_effect: a mid-task rotation fixture (32K local -> 8K free-API) completes with the new lane's
  requests provably under the new envelope, the carrier's committed entries intact (K-12 pass), and one
  SwapProjection receipt; the same swap WITHOUT re-projection (broken arm) overflows or drops atoms and
  FAILS. NEGATIVE: the old backend's cache prefix is never counted as warm post-swap (I-10 assert); a swap
  to an unprobed backend triggers J-111 first.

### J-116 — hallucination x context-fill coupling arm (L-10 x J-34)
- lever PROVABLE (the number that tells the controller where hallucination risk starts per model) | nature
  EXTENSION (bench arm; composes two existing runners, forks neither) | effort S | WAVE 3 | depends J-34 +
  L-10 fixture tiers; composes L-2 ClaimRisk, K-15, I-27.
- Build: run the L-10 hallucination-delta fixture corpus AT STEPPED FILL LEVELS (J-34's padding discipline:
  deterministic real-repo padding, never lorem) per (model, lane): unsupported-claim rate as f(fill%).
  Output: a per-model hallucination-onset point stored beside the J-34 knee in the calibration artifact;
  J-109's controller may then trim EARLIER than the capability knee when the hallucination onset precedes
  it (two curves, one policy input). ELAI-measured only; no paper curve is ever the threshold (ID-5).
- Integration: a fill-stepped arm on hallucination_delta_runner (L-10) reusing J-34's padding + artifact
  store. Prefer-reuse ContextCalibrationEvent / L-10's rows (+Kind FillCoupledArm); net 0. frontend_pair:
  second curve on the J-34 calibration tile.
- observable_effect: the bench emits a recomputable unsupported-claim-vs-fill curve per fixture model; the
  calibration artifact gains hallucination_onset; a ContextPlanApplied on that model shows trim at
  min(knee, onset). NEGATIVE (R2): a flat-padding (lorem) arm FAILS the fixture audit; a missing-onset
  model falls back to the capability knee + a logged absence.

================================================================================
## 3. SEQUENCING for the orchestrating session
================================================================================
Principle: substrate -> measurement -> controller -> economy. Do not build consumers before the fold and
the numbers exist. J-95's own finding binds: MEASURE (J-34) is the #1 leverage; activation wires are cheap.

WAVE 0 (substrate + truth — nothing else starts first):
  L-16, L-17 (immutable transcript + pure fold + freeze-epoch) — the core inversion; everything below
    becomes parameters of it. Building H-6/H-50-class activation BEFORE this doubles work.
  J-110 (token-count truth) — trivial by comparison, unblocks every budget decision.
  J-95 LEG A/B/C (the three dark activation wires: adaptive_reduce, pristine filter, BeliefState condense)
    — cheap, already specced, and their symbols become fold strategies later.

WAVE 1 (measurement + probes):
  J-34 (context calibration bench) on the canonical local model + the main cloud workhorse first.
  K-15 (reasoning-budget knee) alongside.
  J-111 (first-use envelope probe).
  H-30 (staleness instrument) if not already live — it gates the memory-side consumers.

WAVE 2 (the orchestrator itself):
  J-109 ContextController (consumes WAVE 0+1).
  J-112 deterministic resume (the user's hard requirement lands here).
  J-83 -> J-84 -> J-85 -> J-86 (the GRC-v3 reasoning chain, in that order) + J-42 (distill-reset).
  J-113 window-aware decomposition closure.
  L-18 (losslessness/time-travel corollaries) once J-109 owns assembly.

WAVE 3 (economy + robustness):
  J-38 + J-73 (retrieval loop + abstention), J-103 + J-104 (read ladder + overlay), J-76 (placement recipe),
  L-19 (page-in ladder), J-36 (frozen-head local KV), J-114 (health canary), J-115 (swap projection),
  J-116 (hallucination x fill), J-37 (staleness adjudication — HARD dep: the memory store must have a live
  production consumer first, per its own W-3 gate), J-105 (after J-21).

WAVE 4 (verification-facing consumers, unchanged from their own plans):
  L-T1 CGG (L-1..L-10), K-12 flips (if not earlier), K-17 packaging.

Dependency cautions for the orchestrator:
- J-37 refuses to land on a dark store (its own STOP clause). Sequence the crates/memory activation first.
- J-86 activates I-4 substrate; I-4 v1 must exist (it is B..I-built per the status assumption — VERIFY by
  grep, do not assume; if dark, J-86's activation covers it).
- Anything touching conversation.rs is L-16's rewire territory — single owner, no parallel edits.
- H-50/H-51/H-6/I-5/I-6/I-11/3E are B..I: their behavior changes ONLY through the new files above.

================================================================================
## 4. BINDING RULES for the implementing session (inherited, restated once)
================================================================================
- New-plan-files-only for B..I behavior changes; never amend J-34/H-50-class files in place (J-76 precedent).
- Every session: W-1 wiring done-gate (run-trigger, off-agent SSB row), R2 broken-arm-screams, W-2
  no-default-OFF-terminal-state (kill-switch default-ON pattern preferred; allowlist row otherwise).
- GD-1: the COMPOSES-WITH lists above are do-not-reimplement lists. GD-2: every number journal-recomputable
  under ELAI_SEED. GD-4: FP/over-block bounded on >=3 real runs before any default-ON flip.
- HD-2: no change busts the byte-stable cache prefix; volatile content rides the tail (TZ-1 for time).
- HD-3: never constrained-decode or grammar-gate a reasoning role; op-emission roles only (J-106 boundary).
- TD-3: the LLM proposes; deterministic checks + config + the journal dispose. No LLM adjudicates a context
  or reasoning-budget decision anywhere in J-109..J-116.
- K-16 RR-1..RR-4 bind all reasoning-adjacent work; K-19 SF-1..SF-4 bind anything memory/reuse-adjacent;
  K-14 TZ-1..TZ-4 bind anything time-adjacent; CL-1..CL-4 bind everything context.
- H-31 auditor-objectivity firewall: no user memory, no resume prose, no case data into auditor/critic/
  adversary assemblies — J-109's role contracts encode it structurally.
- SCHEMA/TOTAL_VARIANTS: re-grep the live constants at every edit (all stamps drift); bumps are
  ship-order-cumulative; prefer-reuse before new action_types (the net across J-109..J-116 should be ~+4..+6).
- ID-5/HD-1/KD-1: no paper constant is ever a threshold; no claim ships without its measured number or an
  explicit [DESIGN]/[EXTERNAL]/[BENCH-PENDING] tag (K-1 lint applies to any new differentiator wording,
  e.g. "runs flagship tasks on an 8K model" is BANNED until J-113+J-34 produce the supporting numbers).

================================================================================
## 5. One-paragraph summary for the human
================================================================================
The corpus's context + reasoning design (J-34/76 measurement, L-16..19 projection core, J-83..86 + J-42
reasoning governance, J-38/73/103/104 retrieval economy, J-37/H-31/H-32 staleness, L-T1 anti-hallucination)
is already the right architecture for "small models + harness-owned context" — the follow-on session should
ACTIVATE and SEQUENCE it, not redesign it. What was genuinely missing, now specced above: a single
deterministic ContextController with a journaled per-turn plan (J-109), token-count truth per backend
(J-110), a first-use envelope probe for arbitrary models (J-111), a resume artifact that is DERIVED from
the journal rather than written by the model (J-112 — the user's hard requirement), a closure that
decomposes-or-routes instead of truncating when the window is too small (J-113), a runtime context-health
canary (J-114), swap-safe re-projection (J-115), and the hallucination-vs-fill curve the controller trims
against (J-116). Sequencing inverts one thing the letter ordering implies: L-16/L-17 (the newest files)
must be built FIRST, because everything else is a parameter of the fold.

End of context-reasoning-orchestrator-refinement-2026-07-02.md
