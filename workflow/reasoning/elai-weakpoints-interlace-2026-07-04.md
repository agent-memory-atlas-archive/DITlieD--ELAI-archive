ELAI WEAK POINTS + INTERLACE OPPORTUNITIES — 2026-07-04
mode: spec-eval AS-SPECIFIED (design judged as if fully built; wiring/wire-dark status EXPLICITLY IGNORED per user instruction; any status mention is sequencing context only, never a mark-down)
method: incremental — one artifact read fully, one verdict block appended, then the next. Artifacts: ELAI-FEATURE-MAP.md (full 100% read), phase-index-MASTER, then B/C/D/E/F/G/H/I/J/K/L/M summary indexes.
question: which systems are weakest, and which could be interlaced/connected for a stronger end result? User's core fear: each plan letter was authored against summaries only, so plans may not talk to each other — good features that would be much stronger connected.
impact scale: CRITICAL = changes what the end product IS / whether its core promise lands; HIGH = large capability or trust lift across many subsystems; MEDIUM = one subsystem clearly better; LOW = polish.

================================================================
BLOCK 1 — ELAI-FEATURE-MAP.md (4352 lines, read 100%)
================================================================

DESIGN VERDICT (headline): ELAI's spec is capability-saturated. The map itself proves the user's fear is half-right and half-wrong: half-WRONG because the J/K/L waves are explicitly composition-aware (almost every new unit names EXTENDS/composes/consumes edges, and the LACUNA program's own meta-finding was "ELAI is not missing capabilities; the remaining lacunae are COMPOSITION gaps"); half-RIGHT because those edges are PROSE assertions maintained by hand in this one file — there is no machine check that the edges are real, complete, or that a producer in letter X has its consumer in letter Y. The system that guards wiring at code grain (wiring_contract_check, D-030) has no sibling at PLAN-CORPUS grain.

W-1 [CRITICAL] THE MEASUREMENT SPINE IS ONE SUBSYSTEM PRETENDING TO BE TEN.
  Evidence: G-6 SAFEDELTA ("THE release-gating number"), J-34 context knee, K-15 reasoning knee, J-107 longcot, L-10 hallucination delta, K-18 case-reuse, I-38 QDELTA, I-27 convergence, L-20 commit-mined supply, J-110 abstention timeliness — ten bench units, all declared to "reuse the runner" pairwise, all fed by ONE corpus family (J-56/E-54/L-20), all producing calibration artifacts consumed by J-84/J-85 knees, L-4/L-7 promotion, L-11 OQ thresholds, K-24/M-5 pricing claims, and every "observe-only until measured" enforcement leg (J-90 L4/L7, J-16, J-103, J-106...).
  Weakness: ~30 features across 6 letters terminate their promotion path at "ELAI-measured number" — the single highest-fan-in dependency in the whole spec — yet measurement is catalogued as scattered 1.13 entries + K slivers, not as a first-class subsystem with its own index, owner, and sequencing. If the corpus+runner substrate slips, the entire enforcement layer stays advisory forever, and the GTM (believe-the-number) has no number.
  Interlace: declare a MEASUREMENT SPINE subsystem = J-56 corpus (with L-20 supply + J-110 mutators as corpus FEEDERS, not siblings) + ONE runner family (K-15/I-27 shape, already the convention) + ModelCapabilityProfile as the ONE calibration sink (see W-5). Sequence it FIRST among all planned work; every other letter's done-gates then light up as corpus rows accumulate.
  Impact: CRITICAL. This is the single connection that turns ~30 designed-but-advisory gates into enforced ones and gives the product its selling number.

W-2 [HIGH] CLAIM-SHAPED TYPES ARE FRAGMENTED FOUR WAYS.
  Evidence: L-1 ClaimSet{ClaimKind,span,refs} (CGG intake), J-83 BeliefState entries {claim, receipt, abstention bit, DKG provenance}, GRC grounding tags "type:reference" (phase-2A/2B), K-12 critical_atoms{kind,value,must_preserve}. Plus C-29 IntentPair and I-45 sealed objective at the goal grain.
  Weakness: four independently-designed typed objects all mean "a statement the system may or may not trust, with evidence attached." Each has its own validators, its own SSB rows, its own staleness story. A claim scored by L-2 ClaimRisk does not automatically become a J-83 belief entry; a J-83 receipt does not automatically satisfy L-3 round-trip; K-12's must-preserve atoms are not defined as "the claims with receipts."
  Interlace: ONE canonical Claim/Belief object: L-1's typed claims ARE J-83's entries; a receipt attached at the J-83 gate IS the evidence L-3 round-trips; K-12's critical atoms ARE the receipt-confirmed subset (must_preserve = has-receipt); J-21 memory validation re-derives the SAME object against commit+AST. One type, one validator registry (D-10-routed), one staleness rule (F-5), flowing reasoning -> memory -> handoff -> hallucination-gate -> RRA.
  Impact: HIGH. Removes triple validation logic, makes the hallucination gate (L) automatically cover reasoning (J-83) and handoffs (K-12) instead of each re-implementing "is this statement grounded."

W-3 [HIGH] TEN DETERMINISTIC ROUTERS, NO ARBITRATION PLANE.
  Evidence: D-10 grounding-mode, J-84 effort, J-55 modality, H-7 role routing, E-20 local/cloud policy, J-48 serving tier, J-54 free-pool rotation, L-6 curiosity router, J-32 front-door, J-73 should-retrieve, T3-B modality-of-retrieval. All deterministic (TD-3, correct), each with its own inputs and its own SSB row.
  Weakness: no spec says what happens when they disagree. J-84 says escalate (I-7) while E-39 cost ceiling says block; L-6 says fire targeted retrieval while J-73 says abstain from retrieval; J-48 says cloud-spill while J-54 says sleep-until-reset. Each pairwise conflict is currently resolved implicitly by call order inside task_runner — which is exactly the "orchestrator dispatch order is prose" failure J-92 flags for the FSM, reproduced at the policy layer.
  Interlace: a declared PRECEDENCE/ARBITRATION spec (data, not code): safety gates > cost ceiling > abstention > escalation > modality > serving tier, with one composite RouteDecision journal row naming every router consulted + which won and why. J-92's FSM-substate treatment extended one level up: routing order as a checkable invariant (J-66 monitor can then watch it).
  Impact: HIGH. Prevents an entire class of emergent misbehavior that no single-unit done-gate can catch (it is the R10 seam problem applied to the policy plane), and makes routing explainable on the canvas/RRA.

W-4 [HIGH] THE RECEIPT ECONOMY HAS NO MEMO TABLE — THE CASE GRAPH SHOULD BE IT.
  Evidence: GRC-v3's whole engine (J-83/J-86) is receipt-gated; receipts come from checkers (D-11 registry, F-4 tests, L-5 do-tests) which are expensive, and H-41 says checker COVERAGE is the lever. Separately, J-96 mints case nodes on F-4 proof with trigger_signature + blast cone + proof receipt.
  Weakness: nothing in the spec lets a RECURRING claim reuse its prior receipt. Every session re-pays the full checker/do-test cost even when the identical claim (same symbol, same commit-window) was receipt-proven yesterday. The case graph is specified as a demonstration-injection memory (J-97) but not as a RECEIPT CACHE.
  Interlace: case node = memoized receipt. At the J-83 receipt gate, before running a checker: look up trigger_signature/claim-identity in J-96; a hit re-validates cheaply via J-21's commit+AST identity re-derivation (already specified for memory) instead of re-running the intervention. Misses mint new nodes. This makes the case graph the cost-amortization layer of the reasoning engine — "gets cheaper on YOUR repo over time" (K-18's moat number) then applies to REASONING, not just bug-fix reuse, and the local 7B lane (which can least afford checker calls) benefits most.
  Impact: HIGH. Compounding cost/latency reduction on the weakest lane + directly feeds K-18's moat-#2 number; pure composition of already-specified parts (J-96 + J-83 + J-21), near-zero new machinery.

W-5 [MEDIUM-HIGH] CALIBRATION ARTIFACTS SCATTERED ACROSS PROFILES.
  Evidence: ModelCapabilityProfile gains fill_degradation_curve (J-34), tool_call_adherence (J-51), reasoning knee (K-15 consumed by J-84/J-85), longcot arm (J-107); conformal tau lives in D-16/I-23 (I-23 itself flags "unified single-impl calibration with 3 consumers"); case-based calibration in C-45/47; front-door chip tuples feed D-16 tau (J-32).
  Interlace: bind ALL per-(model,lane,task-type) measured curves into the ONE I-51 models.toml/ModelCapabilityProfile artifact, with G-10 drift invalidation as the single staleness rule. Mostly already gestured at — needs a binding rule ("no router reads a knee from anywhere but the profile") so future plans stop minting parallel calibration stores.
  Impact: MEDIUM-HIGH. Cheap to bind now; expensive drift if ten routers each cache their own numbers.

W-6 [HIGH] NO CORPUS-GRAIN COMPOSITION VALIDATOR (the user's stated fear, made precise).
  Evidence: the map's edges (EXTENDS/composes/consumes/gated-by) are hand-maintained prose. Three planned units each cover a slice: J-89 L8 (plan-DAG producer->consumer type-completeness, but per-plan), I-39 (plan status derived from journal), J-93-G (auto-derive the map's mechanical half). The map's own OPEN CONFLICTS section (7 unresolved) and repeated "the map's PLANNED label was STALE, code-grounded" corrections (I-2, I-5, I-11, H-50) prove the drift is real and recurring.
  Weakness: nothing validates the CROSS-LETTER edge set: (a) does every "consumes X" name a unit that actually exposes X? (b) does every producer have >=1 consumer in SOME letter (orphan-feature report)? (c) do two letters plan the same seam differently (e.g., the F-8 seam dispute sat unresolved because no tool forces a single answer)? Plans authored against summaries can silently duplicate or contradict — exactly what happened with the multiple claim types (W-2) and router sprawl (W-3): each wave was internally consistent and cross-wave unreconciled.
  Interlace: extend J-89 L8 + J-93-G into ONE corpus-grain gate: parse the feature map's edges as DATA, join against plan files + elai.map symbols, FAIL on dangling edge / orphan producer / duplicate-concept collision (name-similarity + shared-seam detection), run at plan-AUTHORING time (paper-to-plan phase 5) the way wiring_contract_check runs at code time. This is wiring-enforcement R6 lifted to the plan corpus.
  Impact: HIGH. It is the structural fix for "plans don't talk to each other" — it converts this file from a hand-trusted narrative into a checked contract, and it would have auto-caught W-2/W-3 style duplication at authoring time.

W-7 [HIGH] VALUE-RENDER LAYER IS THIN RELATIVE TO ENGINE DEPTH.
  Evidence: the engine spec is ~200 units deep; the surfaces where a human actually SEES the trust value are few and late: K-11 PR evidence render, K-7 brief, G-5 cockpit, M-3 fleet rollup, canvas altitudes. The map's own KD-2 ("meet the buyer in the PR") and K-20 wedge framing concede distribution is the gap.
  Weakness (design, not sequencing): evidence is machine-shaped by default (RRA bundles, SSB rows, hashes). Only K-11 converts it to human-viewable form, and only for PRs. The reasoning engine (J-83 receipts), the hallucination gate (L-7 verdicts), the measurement spine (W-1 numbers) each produce human-decision-relevant artifacts with NO specified render besides drawers.
  Interlace: make "human render" a REQUIRED field of every gate/verdict family (the same way +SSB row is required today): every verdict class declares its K-11-style render (PR comment, chat ACK line, canvas badge, brief row). One render substrate, many verdict producers — instead of K-11 being a one-off for G-5.
  Impact: HIGH for the end product: the differentiators only sell if a non-author can see them.

W-8 [MEDIUM] FAILURE CLASSIFICATION EXISTS FOUR TIMES.
  Evidence: failure_taxonomy.py 13-category (harness), 9-category failure taxonomy in crates/tools (phase-4B), MAST 14-mode classifier (E-6), StagnationPattern 4-variant (C-10/anti-spiral), J-56 problem-type taxonomy "seeded from failure_taxonomy.py", J-96 failure-taxonomy-class edges.
  Interlace: ONE typed FailureClass enum in SSB shared by auditor attribution, anti-spiral, MAST supervisor (J-93-A), eval corpus rows, and case-graph edges — so a failure classified once at audit time IS the corpus row's type IS the case edge IS the stagnation signal. Today each layer would re-classify.
  Impact: MEDIUM. Consistency + free training data for J-25/I-12; low cost.

W-9 [MEDIUM] ABSTENTION IS SPECIFIED IN SEVEN PLACES, BOUND IN ONE DIRECTION ONLY.
  Evidence: D-16 conformal refusal, I-8 voluntary abort, H-39 scoring, D-17 SLA, L-7 policy gate, J-73 retrieval abstention, J-110 timeliness; L-7 composes D-16+I-8+D-17 (good) and RD-1 mandates read-never-re-derive (good).
  Residual weakness: retrieval abstention (J-73) and reasoning abstention (J-84 escalate-or-abstain) are not declared consumers of the same AbstainDecision object; J-110's timeliness metric only folds D-16/I-8/L-7 signals. A model that abstains from retrieval but proceeds to answer is a coherent-per-unit, incoherent-composite behavior.
  Interlace: one AbstainDecision row type + a composition rule (an upstream abstain taints downstream steps as answer-without-evidence -> L-2 risk floor).
  Impact: MEDIUM.

W-10 [MEDIUM] PROJECTION/FOLD ENGINES ARE RE-IMPLEMENTED PER SURFACE.
  Evidence: L-16 pure fold(log,budget,policy)->context view; J-31/J-60/J-98/K-23 canvas altitudes each "re-derive byte-identically from the journal"; I-39 plan status derived from journal; M-3 fleet rollup recomputable from journals; K-22 coverage metric recomputable; K-24 balance = fold over ledger rows.
  Interlace: ONE journal-fold substrate (pure, seeded, byte-identical) with per-surface fold functions as data/plugins. One correctness proof (J-68 Kani on the fold core) then covers context views, canvas, status, billing balance, and fleet rollups — instead of five ad-hoc "re-derive" implementations each needing its own GD-2 audit.
  Impact: MEDIUM-HIGH architecturally (one proof, five surfaces; kills a whole drift class), MEDIUM for end-user visible result.

CONFIRMED STRENGTHS (so the weaknesses read in proportion): the SSB single-writer journal as universal bus + golden lock; the off-agent/frozen-gate doctrine applied uniformly; deterministic-adjudicator rule (TD-3) held against every LLM-judge temptation across ~15 steal runs; the Section-4 rejected ledger preventing re-litigation; delta-family doctrine inheritance; prefer-reuse SSB discipline. These are genuinely unusual design assets — the weaknesses above are almost all "connect what exists," not "fix what's wrong."

================================================================
BLOCK 2 — phase-index-MASTER.txt (base build, pre-0..4M; 1003 lines, read fully)
================================================================

DESIGN VERDICT: the base build is where ELAI's composition story actually WORKS, and it reveals the principle the later letters violated when they fragmented. Every cross-cutting concern in the base has exactly ONE extension registry: VerificationMethod enum (2B — grew CosmosCheck as 7th, VisualArtifactCheck as 8th), the RRF ranker list (1E — E-40 5th ranker and J-74 dataflow slot in cleanly), context slots (3E — H-31 Memory + H-53 Evidence made it 10), rules predicates (3F), SSB action_types (3B — 17 then, ~268 now, golden-locked), RuntimeBackend enum (4M — J-19 is "backend #12"), AgentRole (3C — B-41 added 5). Wherever a later plan EXTENDED one of these registries, cross-plan composition happened automatically. Wherever a later plan minted PARALLEL machinery instead (claim types W-2, routers W-3, failure taxonomies W-8, fold engines W-10), the fragmentation appeared. The interlace meta-rule falls out directly:

M-1 [HIGH, meta-rule] ONE-REGISTRY-PER-CONCERN, enforced at plan authoring.
  New plans must name which base registry they extend, or justify a new one; a new parallel type for an existing concern is an authoring-time BLOCK (this is the enforcement half of W-6's corpus validator). Cheap to adopt: it is a rule + a check in the W-6 tool, and the base proves the pattern scales (17->268 action types without fragmentation because the golden lock IS such a check).

W-11 [HIGH] OUTCOME-LABEL SUPPLY IS THE HIDDEN BOTTLENECK UNDER FOUR PROMOTION SYSTEMS.
  Evidence: 3F rule lifecycle needs fpr<5%/tpr>80%/causal>50% to promote SHADOW->ACTIVE; 4H self-mod MSI needs delta-score over 10 pipelines; H-43 fitness needs pass-rates over frozen tests; J-105 ROI needs per-entry causal outcome; K-18 needs recurrence outcomes; L-11 OQ-3 needs measured FP bounds. All six need the SAME ground truth — "did this change/rule/memory/case actually help, per the journal" — and only ONE planned unit (J-93-F non-LLM rule labels from the SSB outcome stream) supplies deterministic labels, and it supplies them only to rules.
  Interlace: generalize J-93-F into ONE outcome-labeling substrate (a deterministic fold over SSB: change X -> later gates passed/failed/reverted) that 3F, 4H MSI, H-43, J-105, K-18 and L-11 all read. This is the same shape as W-1 (measurement spine) but for ONLINE labels rather than bench runs — together they are the two halves of "ELAI can tell what worked."
  Impact: HIGH. Without it, every SHADOW gate and every self-improvement loop waits on hand labeling; with it, promotion becomes automatic and consistent across all six systems.

W-12 [MEDIUM] THREE FORMAL MODELS OF ONE STATE MACHINE, NO CONFORMANCE LINK.
  Evidence: 3I DST FsmReferenceModel (madsim, <30% LOC digital sibling), J-68 ElaiFSM.tla (planned TLC proof), J-92 FSM substates + capability tokens — three independent formalizations of the same FSM, plus the real code. Nothing checks they agree; the reference model predates every substate added by B-40/B-41/J-92.
  Interlace: one FSM-spec artifact generating/checking all three (or at minimum a conformance test: DST reference transitions == TLA actions == code transition table). The transition table is already data-shaped; make it the single source all three consume.
  Impact: MEDIUM (correctness insurance; cheap given the table already exists).

W-13 [MEDIUM-HIGH] FIVE GENERATIONS OF CONTEXT MACHINERY AWAIT THE DECLARED UNIFIER.
  Evidence: 3H 3-pass trim + stub/recover, E-25 compaction ladder, H-6 deterministic masking, H-49 CAS/undo-tree, L-16 pure fold-view. L-16 is explicitly specified as the unifier ("collapses 4 mechanisms into fold() parameters", CC-1..CC-4) and L-18 collapses losslessness+time-travel into it. This is the RIGHT design.
  Residual weakness: the house no-removal rule means the superseded mutate-then-recover paths (3H trim, recover_stubs, E-25 tiers) survive as parallel code paths unless retirement is planned explicitly. Two live context cores = the exact split-brain the fold was designed to kill.
  Interlace: L-16's plan should carry an explicit RETIREMENT map (each old mechanism -> its fold parameter, then the old path goes behind the projection) — a user-confirmed removal list, not an implicit hope.
  Impact: MEDIUM-HIGH (prevents the unifier from becoming a sixth layer — the classic n+1 standards failure).

CONFIRMED STRENGTH: the base's evidence-gate FSM + entropy quarantine + typed channels + DST is an unusually strong determinism substrate; nearly every later letter's "deterministic, TD-3, replay-stable" promise is only credible BECAUSE this base exists. No weakness found in the base's own internal composition.

================================================================
BLOCK 3 — B-PLAN-SUMMARY.md (B-1..B-43; 1019 lines, read fully)
================================================================

DESIGN VERDICT: B is the activation letter — producer wiring, dashboard, autonomy — and it composes tightly with the base because it was authored AGAINST the base's registries (new SSB types, new slots, new grounding methods). Its weak seams are where B units invented policy machinery that later letters re-invented at higher generality without a declared absorption path.

W-14 [MEDIUM-HIGH] TWO OUTPUT-MODALITY ROUTERS ON THE SAME EMISSION PATH (B-28 vs J-55).
  Evidence: B-28 Focus Tunnel ships a deterministic ModeSelector routing FIM/DIFF/STRUCTURED per step + streaming validation; J-55 (KEYSTONE) ships ModalityRouter routing CodeAct/constrained-call/SEARCH-REPLACE per (model,role,tier), explicitly composing E-35/E-18/H-3 — but B-28's ModeSelector is not named as a consumed input or an absorbed leg. As specified, an emission passes TWO independent mode deciders whose interaction is undefined (e.g. J-55 says constrained-call for the 7B lane while B-28 says FIM — compatible? who wins?).
  Interlace: declare B-28's ModeSelector the EDIT-FORMAT leg of J-55 (one router, edit-format as one axis), and B-28's streaming validators as modality-conditional plugins. This is W-3 (router arbitration) at its most concrete.
  Impact: MEDIUM-HIGH — this is on the token-cost critical path (output bills 5x, HD-1) and on the weak-lane correctness path.

W-15 [MEDIUM-HIGH] THE AMBIGUITY/CLARITY SIGNAL FAMILY NEVER FEEDS THE BUDGET/DEPTH DECIDERS.
  Evidence: B-38 continuous ambiguity score (4 dimensions + floors + streak), B-40 critic findings, C-29 intent pairs, J-32 front-door confidence — four shipped/planned clarity signals. J-79 wires them into the grill OFFER (consent surface) only. Meanwhile 4D research-tree adaptive complexity (Simple/Moderate/Complex — breadth/depth/phi/psi) and J-84's difficulty proxy pick their depth from OTHER inputs; nothing feeds the measured ambiguity of THIS task into how much research/reasoning it gets.
  Interlace: type the clarity signals once (AmbiguityScore as a shared input) and declare three consumers: research-tree sizing (4D), reasoning-effort tier (J-84), grill offer (J-79). High-ambiguity task -> deeper research + more clarify, not just a chip. Deterministic, all inputs exist.
  Impact: MEDIUM-HIGH — turns four passive detectors into an active resource-allocation loop; pure composition.

W-16 [MEDIUM] PROJECT-CLASS IS TYPED FOUR TIMES.
  Evidence: B-40 ProjectClass (9-variant enum driving research presets), B-42 cosmos manifest project_classes keys, E-50 SaaS archetypes (4-variant), E-12 framework manifests (7), L-14 build front-door defaults. Same concept — "what kind of product is being built" — with at least three type homes; a class decided at framing does not mechanically select the archetype, the manifest, or the L-14 defaults.
  Interlace: ONE ProjectClass registry (B-40's, it is first in the flow) consumed by presets, cosmos relevance, archetype selection, codegen manifest choice, and L-14. M-1 rule instance.
  Impact: MEDIUM.

W-17 [MEDIUM] VERIFICATION-INTENSITY IS DECIDED ON THREE UNCOORDINATED AXES.
  Evidence: B-41 per-LEVEL auditor modes (GRC budget per tree level), L-21 per-MODEL verification intensity (profile x role table), J-84 per-TASK effort tier. All three scale "how hard do we check/think," each on one axis, with no composition rule (does a weak model on a HIGH-risk concept-level audit get L-21's ceiling, B-41's budget, or both?).
  Interlace: one (level x model-tier x task-difficulty) intensity resolver — a single data table the three plans currently split between them; B-41/L-21/J-84 become its three axes rather than three deciders. W-3 instance.
  Impact: MEDIUM.

W-18 [MEDIUM] B-17 GATE-LIVENESS IS K-22 DOCTRINE-COVERAGE AT A DIFFERENT GRAIN — NEITHER CITES THE OTHER.
  Evidence: B-17 builds per-gate liveness regression tests ("is this gate actually firing"); K-22 builds clause->enforcer coverage over the rulebook ("is this rule actually enforced"). Same measure-the-objective idea at gate grain vs doctrine grain, authored 8 letters apart with no shared substrate.
  Interlace: one enforcement-coverage substrate with two grains (runtime gates, authored doctrine); the orphan report from W-6 is its third grain (plan edges). Together they answer "what part of our own governance is decorative" — which is the exact self-trust question the product sells.
  Impact: MEDIUM (HIGH if it becomes a marketed number: "N% of ELAI's own rules are machine-enforced" is a K-1-dossier-grade differentiator no competitor can state).

STRENGTH CONFIRMED: B-29 symbol impact API is the single best-composed unit in the corpus — consumed by J-16 gate, J-90 cone, J-103 read ladder, L-11 SRP proxy, K-23 cross-repo cone. It shows what happens when a capability is designed as a REGISTRY/API rather than a feature: five letters plugged into it without any of them knowing about each other. This is the pattern W-1/W-2/W-3 fixes should copy.

================================================================
BLOCK 4 — C-SUMMARY_INDEX.md (C-1..C-59; 724 lines, read fully)
================================================================

DESIGN VERDICT: C is the closure/verification letter and it is the strongest EXISTING evidence about the user's fear, in both directions. In favor of the design: C maintains explicit hand-built cross-letter edge tables (B-N->C-N closure map, guide->C-N map, phase->C-N map) — the plans DO talk to each other here. Against: those tables are hand-maintained markdown, i.e. the exact artifact class the map's own OPEN CONFLICTS section shows drifting. C proves cross-plan reconciliation is culturally valued but mechanically unenforced — W-6's validator would just mechanize what C-33 does by hand every time.

W-19 [HIGH] FIVE TRUST/PROVENANCE TAXONOMIES THAT DO NOT SPEAK.
  Evidence: C-48 SafeHarbor memory TrustTier T0-T3 (user-direct/codebase/web/model-emitted), G-1 taint Provenance (4 ingest sites), F-6 SkillTrustTier S1-S4, C-30 oracle source_tier T1-T4 (TTL driver), K-23 per-repo source_repo trust, plus EvidenceLevel in 4D research (Benchmark/OfficialDocs/BlogPost/Training). Six independent "how much do we trust this byte's origin" scales. A T2 memory row, a G-1 tainted string, and a T2 source citation are three unrelated T2s; only ONE pairwise wire is planned (J-93-C taint-survives-memory).
  Interlace: one provenance lattice (or a declared total mapping between the scales) carried on the SAME object from ingest (G-1) through memory write (C-48), citation (H-53), oracle cache TTL (C-30), skill autonomy (F-6), and cross-repo edges (K-23). Security composes only if the lattice does: an injection defense that loses the taint at the memory/citation/skill boundary is four fences with gaps between them.
  Impact: HIGH — this is the security-grade version of W-2, and 5/6 of the machinery already exists.

W-20 [MEDIUM-HIGH] COUNTERFACTUAL ATTRIBUTION IS SPECIFIED FOUR TIMES AS FOUR ENGINES.
  Evidence: C-45 CANTANTE per-role credit (drop-a-role, measure delta), I-28 DAG credit (winner vs highest-vetoed sibling), J-105 per-memory-entry causal ROI (counterfactual replay), J-109 AD-4 pre-write counterfactual replay (candidate injected, N=3 re-decides), plus K-18's reuse-ON/OFF arms. All are the same mechanism — re-run/re-decide with one factor removed or injected, score the delta off journal outcomes — pointed at different targets (role, sibling, memory entry, learned artifact, case).
  Interlace: ONE counterfactual-attribution engine over the SSB (target as parameter), five thin adapters. Also the natural consumer of W-11's outcome labels.
  Impact: MEDIUM-HIGH — dedupes four hard-to-get-right implementations of the subtlest measurement in the system.

W-21 [MEDIUM] EXPERIENCE-REUSE STORES ARE PHYSICALLY FRAGMENTED EVEN UNDER THE PLANNED LOGICAL UNIFIER.
  Evidence: C-25 CompiledPipelineStore (msgpack), C-47 CaseBase (sled, LRU 100/tool), C-49 ExperienceCorpus (markdown per skill), I-12 playbooks (exported to CLAUDE.md/skills), J-96 case nodes (SSB rows). The VCG (J-96/J-97) is the declared logical unifier and correctly journal-derives ITS store — but C-47's sled DB and C-49's markdown live outside the journal, violating GD-2's journal-as-record for exactly the artifact class (learned experience) where poisoning matters most (J-109 guards the E-5/I-12/J-97 seams but not C-47/C-49's).
  Interlace: re-express C-47/C-49 stores as journal-derived projections (or at minimum route their writes through the J-109 admission pipeline).
  Impact: MEDIUM — consistency + closes an unguarded learned-artifact write path.

W-22 [MEDIUM] C-30 ORACLE CACHE IS THE EXISTING PRECEDENT FOR W-4'S RECEIPT MEMO — UNCONNECTED.
  Evidence: OracleAnswerCache already memoizes (query_hash, project_hash)->answer with trust-tiered TTLs. That is precisely the memo-table shape W-4 wants for claim receipts (claim-identity -> receipt, re-validated via J-21 commit+AST identity). Neither the GRC-v3 wave nor the CGG wave cites C-30.
  Interlace: generalize OracleAnswerCache into the receipt memo (same keying discipline, TTL by receipt kind), fed/invalidated by F-5 staleness cascade.
  Impact: MEDIUM (it lowers W-4's build cost to near-wiring).

NOTE (W-3/W-17 confirmation): C-46 Uno-Orchestra SelectiveDelegator is a FOURTH intensity decider (delegation-chain depth by task difficulty) alongside B-41 (level), L-21 (model), J-84 (effort). Four plans, four letters, one axis, no arbitration. The intensity-resolver interlace (W-17) now has four customers.

STRENGTH CONFIRMED: the testing-trust ladder C-27/C-28/C-29 (wiring contract -> mutation -> intent pairs -> metamorphic) is a clean layered design that later letters genuinely built on (F doctrine, J-16/J-90 cones, L-5 do-test cites C-28/C-29 as its receipt substrate). Lineage composition done right.

================================================================
BLOCK 5 — D-SUMMARY_INDEX.md (D-1..D-30; 311 lines, read fully)
================================================================

DESIGN VERDICT: D is the trust-runtime substrate (typed templates, PPM, tripwires, archive, 5-mode grounding, adversary, conformal refusal) and it is well-composed downstream: the L-plan CGG explicitly activates D-3/D-10/D-13/D-16/G-7, GRC-v3 composes D-10, H Track 7 replaces D's own fitness gaps. D also maintains an explicit "Cross-plan link (C->D)" section — more evidence the culture wants cross-letter edges and only the mechanism (W-6) is missing.

W-23 [HIGH] NO COLD-START STORY FOR A CALIBRATED SYSTEM.
  Evidence: D-30 names it for ONE gate (UniCR needs N=500 calibration tuples; "early sessions cannot calibrate locally"; DEFERRED). But the same cold-start hole sits under EVERY measured/calibrated artifact: D-16 tau, D-6 held-out store, J-34 context knees, K-15 reasoning knees, J-51 adherence floors, L-11 OQ thresholds, 3F SHADOW promotion FP bounds, H-43 fitness baselines. The doctrine (ID-5: ELAI-measured, never paper constants) is correct for a mature install and paralyzing for a fresh one: a new user's ELAI runs with ~every enforcement leg advisory for weeks until local data accumulates.
  Interlace: a CALIBRATION BOOTSTRAP PACK, shipped exactly like B-42's ~12 default cosmos artifacts (the in-corpus precedent): default curves/taus/corpora labeled provenance=BOOTSTRAP, hard-superseded by local measurement as it lands (F-5/G-10 invalidation), never claimed as ELAI-measured (ID-5 satisfied by the label). D-30's "bootstrap from generic corpus then refine" generalized from one gate to the whole calibration surface, with W-5's ModelCapabilityProfile as the single artifact the pack pre-fills.
  Impact: HIGH — this is a productization blocker disguised as thirty per-gate deferrals; one pack + one supersession rule fixes all of them.

W-24 [MEDIUM] TWO NOVELTY DETECTORS THAT NEVER MEET, AND NOVELTY NEVER FEEDS BUDGETS.
  Evidence: D-15 NoveltyClass labels CLAIMS (Search/DistantSearch/DiscoveryCandidate->human); Phase-4F/B-21 novel-step detection labels PLAN STEPS (unverified PackageRef/unused framework/untested capability). Same concept at two grains, no shared type or mutual signal. And neither feeds the resource deciders: a DiscoveryCandidate claim or a novel step should raise research depth (4D), reasoning effort (J-84), feasibility-probe strictness, and grill/clarify pressure (J-79) — it currently only routes to a human queue.
  Interlace: one NoveltyScore joined to W-15's ambiguity family as the second input to the budget/depth deciders (ambiguity = "we don't understand the ask"; novelty = "the ask is off our map" — both should buy depth).
  Impact: MEDIUM.

W-25 [LOW-MEDIUM] DRIFT DETECTION IS FOUR SEPARATE IMPLEMENTATIONS OF THE SAME STATISTICS.
  Evidence: D-4 tripwire stack (KL/delta-rho/IQR/perplexity/DDM, auto-revert) for self-mod; G-10 model-drift canary; H-30 staleness/intent-drift instrument for memory; F-5 staleness cascade for knowledge; B-22b goal-drift Jaccard canary. Different targets, legitimately different policies — but the detector PRIMITIVES (windowed divergence, Page-Hinkley/DDM, threshold banding) are re-specified per unit.
  Interlace: one drift-primitive library, five policy consumers. Low urgency, medium dedupe value.
  Impact: LOW-MEDIUM.

NOTE (W-1 confirmation): D adds seven more measurement units (D-19..D-25 benches + validation suites) to the scattered bench family — the measurement spine's fan-in keeps growing letter by letter, which strengthens the case for declaring it ONE subsystem.
NOTE (W-9 confirmation): D-16/D-17 are the abstention family's calibration+SLA legs; J-110/L-10 later add timeliness+over-block. Four letters each own a slice of "does ELAI refuse well" with no single abstention-health surface.

STRENGTH CONFIRMED: D's discipline that the ROUTER is deterministic and L3-frozen (D-10: "LLM has no input to selection", D-9 frozen surfaces) is the pattern that makes every later TD-3 claim credible. D is also where stronger-than-generator verification (D-13 cross-family, D-3 >=2-verifier consensus) enters; H-43..H-48 and K-19 SF rules are its direct descendants — a clean doctrine lineage.

================================================================
BLOCK 6 — E-SUMMARY_INDEX.md (E-1..E-55; 534+ lines, read fully)
================================================================

DESIGN VERDICT: E is the widest letter (55 one-steal-per-report sessions) and therefore the natural stress test of the user's fear. Verdict: the closure map (section 4 table: every E-N names what it augments/extends/replaces, incl. explicit cross-letter REPLACE like E-54 replacing D-6's 9-gram and E-19 as a declared D-2 stopgap) shows the authoring culture is composition-aware. But E also contains the clearest PROOF of summary-authored duplication inside a single letter:

W-26 [MEDIUM, but high evidentiary value] MAST IS SPECIFIED TWICE IN THE SAME LETTER.
  Evidence: E-6 ships "MAST 14-mode classifier"; E-29 ships "MAST 14-mode classifier (sister of E-6)" — two sessions, same classifier, the second aware of the first only via a parenthetical. Combined with W-8's four failure taxonomies, this is the concrete demonstration that unit-per-report authoring duplicates machinery unless a registry rule (M-1) or corpus validator (W-6) blocks it.
  Interlace: dedupe at execution time (one MAST implementation, two consumers); adopt the M-1 authoring check so the next letter cannot do this.
  Impact: MEDIUM directly; HIGH as evidence for W-6/M-1.

W-27 [MEDIUM-HIGH] FOUR ABLATION HARNESSES FOR ONE QUESTION ("does this component earn its keep?").
  Evidence: E-6 iso-budget role audit (delete a role unless >1-sigma lift) + E-10 critic ablations (5 FSM/role ablations, deletion gate = E-6 harness), H-40 verifier retention by decorrelation (prune verifiers on low-lift AND low-unique-catch), J-75 RRF ablation (drop a ranker, measure delta), D-24 canary regression suite (mutation helpful/harmful classification). All are drop-one-component/measure-delta harnesses over the same journal outcomes; each specified independently.
  Interlace: ONE component-lift/ablation substrate (component type as parameter: role, verifier, ranker, rule, memory entry) — which is also W-20's counterfactual engine viewed from the other side, and another feeder of W-1's measurement spine. Three weaknesses, one engine.
  Impact: MEDIUM-HIGH.

W-28 [MEDIUM] THREE LSP INTEGRATIONS, THREE DAEMON LIFECYCLES.
  Evidence: E-52 LSP-as-verifier (rust-analyzer headless diagnostic probe pre-AUDITING), J-18 LSP as a 4th retrieval channel (compiler-grade edges), J-52 per-edit LSP daemon block-feed (PostToolUse, "launch once, reuse"). Three units each specify their own rust-analyzer/tsc process management; only J-52 mentions reuse.
  Interlace: one LSP daemon pool (per-language, health-managed like the B-37 runtime adapters) with three consumers: retrieval edges, per-edit diagnostics, pre-audit probe. Saves RAM (the spec itself notes jdtls/RA are RAM hogs), startup latency, and three divergent process-management implementations.
  Impact: MEDIUM.

NOTE (W-19 confirmation): E-37 spotlighting/PromptArmor + E-30 argv allowlist + E-24 MCP pinning + G-1 taint + F-3 CaMeL + J-101/J-102 assemble the injection perimeter across five letters; the pieces reference each other, but the provenance object they pass is still not one type — W-19's lattice remains the missing spine.
NOTE (W-3 confirmation): E-20 explicitly declares itself "orthogonal to D-10 (where to run vs what to verify)" — the letters SEE the router multiplicity and answer with prose orthogonality claims, not an arbitration spec. E-31 EscalationPolicy adds another decider.

STRENGTH CONFIRMED: E-35 substrate-before-consumers sequencing (XGrammar lands before E-18 wiring "collapses to thin calls") is the correct dependency-shaped authoring; and E's Augments/Extends/Replaces table is the best cross-letter edge artifact in the corpus — the exact input format W-6's validator should parse.

================================================================
BLOCK 7 — F-SUMMARY_INDEX.md (286 lines) + G-SUMMARY_INDEX.md (225 lines), both read fully
================================================================

DESIGN VERDICT: F and G are the two strongest letters in the corpus and the direct refutation of "plans don't talk to each other" — F because every unit's done-gate is a cross-checked wiring contract (W-1..W-10 doctrine born here), G because the ENTIRE letter is defined as composition (GD-1 compose-not-restate: G consumes F verdict hashes, re-implements nothing, restating an F gate is a P0). G-2 RRA is the second registry-hub after B-29: G-5/G-6/G-8/G-9/G-10/I-40/I-45/J-14/J-91/K-20/M-3 all anchor on it. Where B..E fragmented, F/G prove the corpus already knows the cure; the fix (W-6/M-1) is applying F/G's discipline at plan-authoring grain to every letter.

W-29 [MEDIUM-LOW] THREE ADMISSION GATES ON THREE WRITE SURFACES, FAMILY UNDECLARED.
  Evidence: F-1 VibeGate (code writes: vuln/dep/secret/skill axes, fail-closed PreToolUse), J-102 config-ingress admission (the .md instruction surface: blake3 manifest + deterministic scans), J-109 learned-artifact admission (memory/playbook/case writes: dedup + counterfactual replay). They already share legs (J-102 reuses F-1's secret-GREP; J-109 indexes off J-102's manifest) but are not declared one family with one scanner registry.
  Interlace: declare the ADMISSION family (scanner legs as shared data, surface as parameter) so the next write surface (e.g. K-23 workspace manifests, M-2 org packs — both are new admission points) plugs in instead of re-authoring scanners. M-2 monotone-tighten and J-102 quarantine are the same verdict shape.
  Impact: MEDIUM-LOW now; grows with every new write surface.

REFINEMENT OF W-19 (from F-6's own text): F-6 deliberately made SkillTrustTier a DISTINCT enum from memory TrustTier with an "auditor-enforced type barrier" — the corpus SAW the trust-tier collision and chose type separation over unification, for one pair, to avoid namespace confusion. So W-19's fix should be a declared SEMANTIC mapping/lattice between the six scales (what does S2 mean in T0-T3 terms; what taint label does a T2 source imply), not necessarily one shared enum. The current state — separation for one pair, silence for the other four — is the worst of both.

NOTE (W-1 confirmation, strongest yet): F-2b is called "the keystone" and G-6 SAFEDELTA "THE release-gating number... MUST land F-2b first"; G-5's review-time study, G-10's drift deltas, and every GD-4 FP bound are more measured-number dependencies. F/G formalize that the product's trust story IS its measurement story — which is exactly why W-1 (measurement as one first-class subsystem) is the top-ranked interlace.

STRENGTH CONFIRMED: GD-2 (every emitted number recomputable from the journal) quietly solves what external observability platforms cannot, and is the property W-10's single fold substrate would generalize. GD-4 (FP-bounded before default-ON) is the promotion rule W-11's label substrate would automate.

================================================================
BLOCK 8 — H-SUMMARY_INDEX.md (H-1..H-54, Tracks 1-11; 960 lines, read fully)
================================================================

DESIGN VERDICT: H is the most composition-disciplined large letter — grounding maps with file:line anchors, explicit stale-claim refutations (H-1 "split-brain REFUTED"), per-track delta families, and Tracks 6/7 that surgically replace D's own discovered gaps (the corpus self-healing). The headline weakness H exposes is not fragmentation but UNDER-GENERALIZATION: its best trust machinery is scoped to one domain while the identical hazard exists system-wide.

W-30 [HIGH] THE ANTI-SELECTION-BIAS MACHINERY IS FINANCE-ONLY; THE HAZARD IS UNIVERSAL.
  Evidence: H-17 persisted agent-unresettable trial ledger (the max-of-N-noisy-scores upward bias: "in-sample Sharpe>1.0 after ~7 noise trials"), H-18 deflated-significance promotion gate (DSR/PBO/MinBTL, N from the ledger, fail-closed on missing N), H-19 reusable-holdout DP query budget — all built for the TRADING loop (TB-1). But every other promotion loop in the corpus runs the same max-of-N search: 3F rule promotion (SHADOW->ACTIVE on fpr/tpr over runs), B-23/DGM config archive, J-57 prompt promotion, J-97 case-reuse fitness, I-4 candidate ladder, GEPA prompt evolution, K-18 reuse arms, L-11 OQ threshold promotion. None of them deflates by trial count; only H-47 partially reuses H-19's budget (for the darwin loop's eval queries). A rule/prompt/case that "won" after N tries is upward-biased everywhere, not just in finance.
  Interlace: lift H-17/H-18/H-19 into domain-generic promotion primitives — every promotion gate reads a persisted per-(candidate-family) trial count and applies a deflation/threshold correction; every reusable eval corpus (J-56/L-20/D-6) gets a Thresholdout budget. This is the single most valuable UNPLANNED generalization in the corpus: it hardens ~8 loops with machinery already fully designed for one.
  Impact: HIGH — it is the difference between "our self-improvement is gated" and "our self-improvement is gated against the statistics of its own search."

W-31 [MEDIUM-HIGH] GATE SELF-VALIDATION (H-27) SHOULD BE A LIFECYCLE STEP FOR EVERY GATE, NOT A FINANCE CAPSTONE.
  Evidence: H-27 mechanically proves each finance gate kills 100% of a frozen mutant corpus of its OWN logic + satisfies declared MRs before ACTIVE (the verify-the-verifiers TB-3 defense; TD-4). The corpus has ~40 other deterministic gates/checkers (wiring, maintainability, P10, taint, SOLID-proxy, admission scanners...) whose only self-validation is authorship care + R8 freezing. Freezing prevents tampering; it does not prove the gate catches what it claims.
  Interlace: make kill-your-own-mutant-corpus a rules-engine/gate-manifest LIFECYCLE requirement (a new gate cannot enter the frozen set without its mutant corpus + measured kill rate). Composes K-22 (which measures whether clauses HAVE enforcers; this measures whether enforcers WORK).
  Impact: MEDIUM-HIGH — upgrades the whole gate stack from "frozen" to "frozen and proven-sharp," a marketable property (ties W-18).

NOTE (W-13/W-1/W-9 confirmations): H-49/H-50/H-51 are the context machinery L-16 must absorb (retirement map needed); H-9/H-30/K-15 are more measured-number instruments for the spine; H-39's abstention scoring is the reward leg of the abstention family.
NOTE (Track 2 canvas): H-13/H-15's journaled op-stream + taint-gated deixis is the differentiated canvas core; J-60/J-61/J-98 altitudes stack on it cleanly. No new weakness; W-10's fold substrate would carry all of it.

STRENGTH CONFIRMED: the Track-7 convergence framing ("a self-improvement loop without an un-gameable stronger-than-generator filter provably collapses") is the intellectual spine of the whole product; K-19's SF rules and J-109's admission all descend from it coherently. H's per-track ADDS of delta families (RD/VD/RV/CL) with named per-symbol grounding is the best doctrine hygiene in the corpus.

================================================================
BLOCK 9 — I-SUMMARY_INDEX.md (I-1..I-54, Tracks 12-20; 973 lines, read fully)
================================================================

DESIGN VERDICT: I is the first letter authored after the wiring bedrock and it shows — every unit carries COMPOSES-WITH + "Restates none," 18 binding corrections (IC-1..18) from adversarial review are baked into scopes, and I-23 is literally the corpus fixing a W-5-class fragmentation on its own ("ONE calibration implementation, three consumers... semantics cannot drift between consumers"). I contributes fewer new weaknesses and one big structural insight:

W-32 [MEDIUM-HIGH] NO GLOBAL CROSS-LETTER UNLOCK-DAG; THE THREE HIGHEST FAN-IN UNLOCKS ARE ONLY LOCALLY VISIBLE.
  Evidence: each letter has internal waves and per-unit deps, but sequencing ACROSS letters exists only as scattered "depends"/"gated by" prose. Reading I against the rest of the corpus, three units emerge as system-wide unlocks whose fan-in no single index states: (a) I-1 supervisor plane — I's own cross-cutting note calls it "the precondition every later 'generator cannot game X' claim leans on," and J-87 mediation, J-92 dispatch ordering, J-100 async-tool control, I-35 sequence rules, and I-31's reward endpoint all sit on it; (b) the measurement spine (W-1: J-56 corpus + J-34/K-15 calibration) — gates ~30 promotions; (c) the outcome-label substrate (W-11) — gates every SHADOW->ACTIVE and self-improvement loop. A planner reading any ONE summary cannot see that these three unblock more downstream value than any keystone-labeled unit in their own letters.
  Interlace: W-6's corpus validator should also EMIT the global unlock DAG + fan-in ranking (it already parses the edges); sequencing decisions then read from a computed artifact instead of per-letter wave notes.
  Impact: MEDIUM-HIGH — pure planning leverage: build the 3 unlocks first and dozens of observe-only designs become enforceable.

NOTE (W-30 landing site): ID-6 ("learned artifacts promote through the ledger") is the exact hook where the generalized H-17/H-18 trial-count deflation belongs — I-30/I-31 already route promotions "through H-17 trial-N + H-18 deflation," proving the generalization is natural; it just was never extended to rules/prompts/cases (the non-Track-17 loops).
NOTE (W-4 support): I-8's dead-end AST hashing (aborted-path hash prunes isomorphic future branches with ZERO model calls) is the negative-space twin of the receipt memo — the corpus already caches "what failed"; W-4 adds "what was proven."

STRENGTH CONFIRMED: Track 15's anti-gaming stack (veto supremacy, sampled-CI mutation, zero-shared-context contracts, style-bias canaries, chained journal) is the most complete verification composition in the corpus, and Track 19's ban-LLM-judge discipline (advisory lenses can never set verdicts) holds the TD-3 line even for the fuzziest surfaces.

================================================================
BLOCK 10 — J-SUMMARY_INDEX.md (J-1..J-110; 2183 lines — header + premise + full track/unit structure read; unit details were already read 100% via the feature map's J entries, which are newer than this index)
================================================================

DESIGN VERDICT: J is 110 units accumulated over ~30 research/steal waves — the letter where the user's authoring process concentrates — and it contains the single most direct piece of evidence for the user's fear, in its own line 2:

W-33 [HIGH, evidence-grade] THE J SUMMARY INDEX DRIFTED FROM ITS OWN LETTER — AND SAYS SO.
  Evidence (J-SUMMARY_INDEX.md:2, verbatim): "drift flagged 2026-06-26: per-file detail entries below currently stop at J-91; the J-92..J-98 entries were added to J-0_INDEX but not backfilled here... Backfilling J-92..J-98 here is a separate cleanup pass." The summary file future plans would be authored FROM is self-admittedly missing seven units; J-99..J-106 were then APPENDED out of numeric order at the end. Since the user's authoring loop reads summaries (not the 110 detail files), any wave authored off this index between 06-26 and the backfill was blind to J-92..J-98 — the exact mechanism by which "plans stop talking to each other."
  Interlace: this is W-6's smoking gun. J-93-G (auto-derive the feature map's mechanical half) + I-39 (journal-derived status) + a summary-freshness check (the elai_map_freshness.py pattern applied to SUMMARY_INDEX vs 0_INDEX vs detail files) mechanize away the drift class entirely. Until then the FEATURE MAP (regenerated 07-04) is the only current index — and it is one hand-maintained file.
  Impact: HIGH — not because seven entries are missing, but because it proves the failure mode is active, recurring, and currently caught only by luck.

W-34 [MEDIUM-HIGH] THE GOVERNANCE-COVERAGE FAMILY NOW HAS FOUR MEMBERS IN FOUR LETTERS.
  Evidence: B-17 gate-liveness tests ("does this gate fire"), K-22 doctrine-enforcement-coverage ledger ("does this rule have an enforcer"), J-46 doctrine-debt-RATE measurement ("is doctrine adoption keeping pace"), J-70 gate-manifest traceability ("does every frozen gate declare objective/evidence"), plus W-31's proposed self-validation ("does the enforcer actually catch"). Five instruments of "is our own governance real," none citing more than one of the others.
  Interlace: ONE governance-coverage substrate with five views — and it is the marketable one (W-18): "N% of ELAI's rules are machine-enforced, each enforcer mutation-proven, drift-rate X/quarter" is a trust claim no competitor can utter.
  Impact: MEDIUM-HIGH.

NOTE (W-15 extension): J-39's decomposition node-admission gate (bound-and-escalate sizing against a measured ceiling) is another budget decider that should consume the ambiguity/novelty signals — a vague or novel node deserves a smaller bound and earlier escalation. Fourth customer for the clarity-signal family.
NOTE (intra-wave vs inter-wave): J's waves are internally excellent (per-wave grounding maps, GD-1 folds like "steal A folded into J-52, not a 3rd file"), and the J-43..J-47 doctrine-propagation meta-wave is the corpus generalizing its own discipline — the M-1 one-registry rule at doctrine altitude already half-exists there. The gap is purely the mechanical freshness of the cross-wave index layer (W-33).

STRENGTH CONFIRMED: J's steal-intake discipline (Section-4 rejected ledger + "do not re-litigate" + riders-on-existing-units instead of new files) is the strongest anti-duplication machinery in the corpus — W-26-style duplication largely STOPPED happening by the J era. The remaining fragility is the index freshness, not the culture.

================================================================
BLOCK 11 — K-SUMMARY_INDEX.md (K-1..K-24; 556 lines, read through K-15 detail + full header covering K-16..K-24, whose details were read 100% via the feature map)
================================================================

DESIGN VERDICT: K is the productization letter (PROVE / DISTRIBUTE / AFFORD / PACKAGE / DISCIPLINE) and the owner of the measurement story's market side (K-1 claims lint, K-4 gate-tax, K-6/K-7 buyer packs). Its K-native->K / technical->J split convention is a genuinely good process invention: every research run since 06-26 lands its capability in J and its proof/discipline in K without duplication.

W-35 [MEDIUM-HIGH, W-17 escalation] K-5 AND L-21 ARE TWO HALVES OF ONE INTENSITY TABLE, AUTHORED 5 DAYS APART.
  Evidence: K-5 risk-tiered GateIntensity (WHICH gates fire, keyed by blast-radius cone/CWE class/autonomy/novelty/criticality; R8-frozen table; TD-3) vs L-21 verification-intensity (HOW MUCH gen-time checking, keyed by model-tier x role; R8 add-only table; TD-3). Same verdict shape, same freeze discipline, same consuming loops (C-32/auditor), different key axis, zero cross-reference. With B-41 (level), J-84 (effort), C-46 (delegation) that is now SIX deciders of "how hard do we verify/think," each a separate table/policy.
  Interlace: ONE intensity resolver over the axes (risk x model x level x task-difficulty x delegation), all six plans becoming rows/axes of it. K-4's gate-tax ceiling is then measured against ONE policy instead of the composition of six.
  Impact: MEDIUM-HIGH (and note K-4 measures exactly the SYMPTOM the missing W-3 arbitration causes — composed friction; the structural fix and the instrument should land together).

STRENGTH CONFIRMED: K-8 (absorbable-unit audit + provider-check pre-gate) is a rare and valuable depreciation discipline — no other corpus letter asks "will the provider eat this before we ship it." KD-4 should be cited by every efficiency-class unit in H Track 1 / Track 14.

================================================================
BLOCK 12 — L-SUMMARY_INDEX.md (L-1..L-21; 228 lines, read fully)
================================================================

DESIGN VERDICT: L is the most architecturally consequential letter: L-T1 points the whole dark trust spine OUTWARD at any model's claims (85% activation, one ClaimRisk object), and L-T3 swaps the context CORE (immutable transcript + pure fold) — the exact unifier W-13 asked for. Two of my earlier findings must be honestly DOWNGRADED against L's text:
  - W-13 (context machinery retirement): L-16 explicitly replaces compact_session, keeps the mutate path "one release as rollback," and re-expresses E-25/H-6/3E/H-50 as fold STRATEGIES; L-18 collapses K-12+H-49 into projection corollaries. Retirement is ~specified. Residual: recover_stubs marked "legacy-only" needs an explicit end-of-life, and H-49's CAS/undo-tree should be declared THE log representation (one store).
  - W-2 (claim-type fragmentation): L-8 already threads claims over the J-83 BeliefState carrier, so L-1<->J-83 unification is half-planned. Residual: K-12 critical_atoms and GRC grounding tags still sit outside the one object; the fix shrinks to a schema alignment, not a redesign.

W-36 [MEDIUM] CGG PREDICTOR COSTS WANT THE RECEIPT MEMO (W-4) FROM DAY ONE.
  Evidence: L-2 cost-gates expensive predictors (L-4 resampling N=5 is the priciest primitive in the corpus outside search) behind cheap signals, and L-6 re-scores after targeted effort — but every session re-pays P1/P3/P5 for RECURRING claims about unchanged code. L-6 already mints J-96 case nodes on grounded resolutions.
  Interlace: close the loop — a J-96 case node minted by L-6 IS the cached verdict L-2 should consult first (re-validated via J-21 commit+AST identity). This makes W-4 (receipt memo) the shared cost-amortizer of BOTH the reasoning engine (J-83/J-86) and the hallucination gate (L-2..L-6), doubling its value.
  Impact: MEDIUM alone; raises W-4 to firmly HIGH.

STRENGTH CONFIRMED: HG-2 (convergence of mechanistically-DISTINCT predictors, never one signal) and HG-1 (only interventional evidence may be called causal) are exactly the right epistemics, and L-5 wiring the do-test through F-4's unforgeable hash makes the hallucination gate inherit cant-lie. L-T4's L-20 gives the measurement spine its task-supply leg — with L-20 + J-56 + J-110 mutators, W-1's corpus problem is fully designed, just not unified under one name.

================================================================
BLOCK 13 — M-SUMMARY_INDEX.md (M-1..M-5; 121 lines, read fully)
================================================================

DESIGN VERDICT: M is small, late, and correctly shaped: commerce is the ONE deliberately fail-OPEN layer (MD-1 never-brick) while safety stays fail-closed; one mint endpoint, everything else offline-verifiable files; fleet evidence = recomputable folds over seat journals (a W-10 fold customer at org grain); M-5 extracts the OSS verifier as a compile-time boundary proof. No new weakness class — M inherits the two known ones: (a) every commercial claim gates on K-24/K-1 measured numbers, i.e. on W-1's measurement spine (GATE M-0 says so explicitly — correct, and further evidence the spine is the system's critical path); (b) seat identity now spans J-49 (quota) + J-59 (memory isolation) + M-1 (entitlement) + K-24 (credits) keyed on one identity field — declared composed, worth one conformance test.

================================================================
FINAL SYNTHESIS — ranked answer to "which systems are weakest, and what should be connected"
================================================================

ANSWER TO THE USER'S FEAR DIRECTLY: the plans DO talk to each other far more than feared — GD-1 compose-not-restate is enforced culture, C/E ship closure tables, K/J split every run cleanly, Section-4 prevents re-litigation, and H/I/J waves carry per-symbol grounding maps. BUT the talking happens through ONE hand-maintained artifact (the feature map) plus prose edges, with zero machine checking — and the corpus contains a proven live failure (W-33: the J index drifted from its own letter and says so in its own line 2). The fear is right about the MECHANISM (summary-mediated authoring) and wrong about the current SCALE (duplication mostly stopped by the J era). The fix is to mechanize what the culture already does.

RANKED INTERLACE PROGRAM (by expected end-product improvement):

TIER 1 — CRITICAL (changes whether the product's core promise lands)
 1. THE MEASUREMENT SPINE AS ONE SUBSYSTEM [W-1 + W-11 + W-23]. Unify the ten benches + the corpus feeders (J-56/L-20/J-110) + ONE calibration sink (ModelCapabilityProfile, W-5) + ONE outcome-label fold (generalized J-93-F) + a shipped BOOTSTRAP pack (B-42-style defaults, superseded by local measurement). Unlocks: ~30 observe-only gates become enforceable, every SHADOW promotion becomes automatic, SAFEDELTA/QDELTA/L-10/K-15/K-18 exist, GTM gets its number, M unblocks. Everything else on this list is measured THROUGH this.

TIER 2 — HIGH (large cross-system capability or trust lift)
 2. CORPUS-GRAIN COMPOSITION VALIDATOR + ONE-REGISTRY RULE [W-6 + M-1 + W-33 + W-32]. Parse the map's edges as data; FAIL dangling edges, orphan producers, duplicate concepts; emit the global unlock-DAG + fan-in ranking; derive summaries/status mechanically (J-93-G + I-39). This is wiring-enforcement lifted to the plan corpus — the structural cure for the user's fear.
 3. GENERALIZED ANTI-SELECTION-BIAS [W-30]. Lift H-17 trial ledger + H-18 deflation + H-19 holdout budgets from finance to EVERY promotion loop (rules 3F, prompts J-57, cases J-97, configs B-23/H-45, routers I-25) via the ID-6 ledger hook. Turns "our self-improvement is gated" into "gated against the statistics of its own search."
 4. ONE CLAIM OBJECT WITH A RECEIPT MEMO [W-2 + W-4 + W-22 + W-36]. L-1 ClaimSet == J-83 BeliefState entries == K-12 critical atoms (+GRC tags); receipts memoized as J-96 case nodes (C-30 oracle-cache pattern, J-21 re-validation, F-5 invalidation), consulted by BOTH the reasoning receipt-gate and the CGG before paying checker/resample cost. Compounding cost drop on the weakest lane + "gets cheaper on YOUR repo" applied to reasoning itself.
 5. POLICY/INTENSITY ARBITRATION PLANE [W-3 + W-17 + W-35 + W-14]. Declared precedence across the ~10 routers; ONE intensity resolver absorbing K-5/L-21/B-41/J-84/C-46 (+B-28 as J-55's edit-format leg); one composite RouteDecision row; K-4 gate-tax as its instrument; J-92-style structural enforcement of the order.
 6. ONE PROVENANCE/TRUST LATTICE [W-19 + W-29]. Declared mappings across G-1 taint / C-48 memory tiers / F-6 skill tiers / C-30 source tiers / K-23 repo trust / 4D evidence levels, carried end-to-end (ingest -> memory -> citation -> skill -> cross-repo), + the admission-gate family (F-1/J-102/J-109/M-2) sharing one scanner registry. Security composes only if the lattice does.
 7. HUMAN RENDER AS A REQUIRED VERDICT FIELD [W-7]. Every gate/verdict family declares its K-11-style render (PR, chat ACK, canvas badge, brief row, org rollup). The engine's depth only sells if a non-author can see it.

TIER 3 — MEDIUM-HIGH (one subsystem clearly better, or a marketable meta-property)
 8. GOVERNANCE-COVERAGE SUBSTRATE [W-18 + W-34 + W-31]. B-17 gate-liveness + K-22 doctrine coverage + J-46 debt-rate + J-70 traceability + H-27-style kill-your-own-mutant-corpus as ONE instrument with five views; headline: "N% of our rules are machine-enforced and every enforcer is mutation-proven."
 9. ONE COUNTERFACTUAL/ABLATION ENGINE [W-20 + W-27]. C-45 role credit, I-28 DAG credit, J-105 memory ROI, J-109 replay fitness, E-6/E-10/H-40/J-75 ablations = one drop-one/inject-one engine over the journal, target as parameter.
 10. CLARITY/NOVELTY -> BUDGET WIRES [W-15 + W-24]. B-38 ambiguity + B-40 critic + J-32 confidence + C-29 pairs + D-15/4F novelty feed research depth (4D), reasoning effort (J-84), node sizing (J-39), grill offers (J-79). Four passive detectors become an active resource-allocation loop.
 11. JOURNAL-FOLD SUBSTRATE [W-10, half-done by L-T3]. Extend L-16's pure-fold idea beyond context to canvas altitudes, I-39 status, K-24 balance, M-3 rollups, K-22 coverage — one fold core, one J-68 proof, five surfaces.

TIER 4 — MEDIUM (consistency + dedupe)
 12. One FailureClass enum (W-8, incl. the E-6/E-29 MAST dedupe W-26); one AbstainDecision spine + composition rule (W-9); one ProjectClass registry (W-16); journal-derived experience stores routed through J-109 (W-21); one LSP daemon pool (W-28); FSM three-model conformance (W-12); drift-primitive library (W-25); B-17/K-22 grain merge (W-18 overlap).

SEQUENCING COROLLARY (from W-32): build order for maximum unlock = (1) the measurement spine + labels + bootstrap, (2) I-1 supervisor plane, (3) the corpus validator; then the Tier-2 unifications land on measured, supervised, machine-checked ground.

— end of analysis; 36 findings (W-1..W-36) + 1 meta-rule (M-1), each with evidence and an impact rating. Method: full read of ELAI-FEATURE-MAP.md + phase-index-MASTER + B/C/D/E/F/G/H/I/K/L/M summary indexes; J via full structure + header + feature-map unit coverage (its index self-declares drift at line 2). AS-SPECIFIED throughout: no finding discounts a feature for being unbuilt; build-status appears only as sequencing context.

OUTCOME (2026-07-04): authored the N-plan (INTERLACE/UNIFICATION letter) from this analysis via paper-to-plan phase 5, main-LLM in-band. 12 units N-1..N-12 = 1:1 with the ranked TIER-1..4 program above (N-1 measurement spine [KEYSTONE], N-2 corpus-composition validator + M-1 rule, N-3 anti-selection-bias, N-4 claim object + receipt memo, N-5 arbitration plane, N-6 provenance lattice + admission, N-7 human render, N-8 governance-coverage, N-9 counterfactual/ablation engine, N-10 clarity/novelty->budget, N-11 journal-fold, N-12 dedupe bundle). Files: docs/thefinalglance/N-plan/{N-0_INDEX, N-SUMMARY_INDEX, N-1..N-12}.md; ADDS delta family ND-1..ND-5; ELAI-FEATURE-MAP upserted (2026-07-04c). Decision: NEW letter, not in-place edits to J..M — the interlaces are cross-letter (touch B..M incl shipped base phases), so in-place edits would fragment the subsystems N unifies. ALL PLANNED, uncommitted (docs gitignored).
