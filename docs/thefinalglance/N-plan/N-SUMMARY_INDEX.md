# N-plan — Summary Index of All 15 sessions (Tier-1..4 interlace program + external-steal track)
Generated: 2026-07-04; external-steal track N-13..N-15 appended 2026-07-05. Mirrors G-SUMMARY_INDEX / M-SUMMARY_INDEX format.

Each entry preserves: the finding W-N it discharges + its impact tier; grounding strength; nature
(ACTIVATION / BUILD / BUILD-on-shipped / DELETE); the fragmented implementations it UNIFIES (with the B..M unit
IDs, the >=2 machines it collapses); the COMPOSES-WITH artifacts CONSUMED (GD-1, never re-implemented); the real
integration point; and the observable_effect done-gate. Doctrine + the ND-1..ND-5 deltas + sequencing live in
N-0_INDEX.md; the WHY (evidence per finding) lives in workflow/reasoning/elai-weakpoints-interlace-2026-07-04.md.

## Premise

N is the INTERLACE letter: it invents almost nothing and CONNECTS everything. The LACUNA program (B..M) reached
capability-saturation; the residual weakness is that letters were authored against SUMMARIES, so same-concern
machinery fragmented (four claim types, ten routers, six intensity deciders, five provenance scales, four
counterfactual engines, five context cores). N declares ONE canonical subsystem per fragmented concern +
mechanizes the anti-fragmentation rule (M-1) so no future letter re-forks. Differentiator vs predecessors:
where G composed F, N composes ALL of B..M, and its N-2 unit lifts wiring-enforcement R6 from code grain to
PLAN-CORPUS grain — the first machine check that plans talk to each other.

==== TIER 1 — CRITICAL ====

### N-1 — measurement spine as ONE subsystem [KEYSTONE][BLOCKING]
Tier 1 | finding W-1+W-11+W-23 | grounding STRONG | nature ACTIVATION | effort XL | WAVE 1
Steal/build: ~30 features across 6 letters terminate their promotion path at "ELAI-measured number" — the
single highest fan-in in the corpus — yet measurement is catalogued as scattered 1.13 entries + K slivers, not a
subsystem. Ten benches (G-6 SAFEDELTA / J-34 context-knee / K-15 reasoning-knee / J-107 longcot / L-10
hallucination-delta / K-18 case-reuse / I-38 QDELTA / I-27 convergence / L-20 supply / J-110 timeliness) all
"reuse the runner" pairwise, all fed by ONE corpus family (J-56/E-54/L-20), all feeding J-84/J-85 knees + L-4/L-7
promotion + K-24/M-5 pricing. N-1 declares the spine: J-56 corpus (+ L-20 supply + J-110 mutators as FEEDERS,
not sibling benches) + ONE runner family (K-15/I-27 shape) + ONE outcome-label fold (generalize J-93-F from
rules-only to the six promotion loops, W-11) + a shipped BOOTSTRAP pack (provenance=BOOTSTRAP defaults for
D-16 tau / J-34 knees / K-15 knees / OQ thresholds, hard-superseded by local measurement, W-23 — the cold-start
fix for a fresh install where every gate is otherwise advisory for weeks).
COMPOSES-WITH: J-56/E-54/L-20 corpus, K-15/I-27 runner, J-93-F label fold, the W-5 ModelCapabilityProfile
(I-51 models.toml) as the ONE calibration sink, B-42's default-artifact-pack shape for the bootstrap. Do NOT
re-implement any bench runner — bind them to the one runner + one profile.
Integration: crates/knowledge (bench/corpus) + crates/orchestrator (ModelCapabilityProfile, label fold); the
CLI bench seams (elai bench *). frontend_pair: a MeasurementPanel tile fed by the spine index (or N/A justified
if G-5 cockpit already surfaces it).
observable_effect: trigger `elai bench <any-family>` writes a corpus row + a calibration curve into the ONE
profile + a +calibration_bootstrap_superseded SSB row when a BOOTSTRAP default is replaced by local measurement;
negative arm: a bench that writes a knee to any store OTHER than the profile must FAIL (ND-4).

==== TIER 2 — HIGH ====

### N-2 — corpus-grain composition validator + M-1 one-registry rule
Tier 2 | finding W-6+M-1+W-33+W-32 | grounding STRONG | nature BUILD | effort L | WAVE 2 | depends N-1
Steal/build: the feature map's EXTENDS/composes/consumes/gated-by edges are hand-maintained prose with zero
machine check; J-SUMMARY_INDEX.md:2 self-declares it drifted from its own letter (detail stopped at J-91). N-2
parses the map's edges as DATA (ND-3 format) + joins against the plan files + elai.map symbols and FAILS on:
(a) dangling edge (a "consumes X" naming no unit that exposes X), (b) orphan producer (a unit with >=1 consumer
in NO letter), (c) duplicate-concept collision (name-similarity + shared-seam detection — the check that would
have auto-caught the four claim types / the E-6+E-29 MAST dup / the ten routers at authoring time). It EMITS the
global unlock-DAG + fan-in ranking (W-32) and derives the summaries/status mechanically (compose J-93-G
map-half-derivation + I-39 journal-status) with a SUMMARY_INDEX-vs-0_INDEX-vs-detail freshness check (the
elai_map_freshness pattern, W-33). Runs at paper-to-plan phase 5 / plan-authoring the way wiring_contract_check
runs at code time. This is the M-1/ND-1 enforcer for every future letter.
COMPOSES-WITH: J-89 L8 plan-DAG type-completeness (per-plan -> lifted to corpus), J-93-G, I-39, the
elai_map_freshness.py + wiring_contract_check.py patterns. Do NOT re-implement the per-plan L8 check — extend it.
Integration: .claude/tools/ (a new corpus_composition_check.py sibling of wiring_contract_check.py) + the
paper-to-plan / planning-workflow authoring seam. frontend_pair: N/A (an authoring-time CI gate, not a runtime
surface); justified.
observable_effect: trigger `python .claude/tools/corpus_composition_check.py --map ELAI-FEATURE-MAP.md` exits
nonzero on a seeded dangling edge / orphan / duplicate-concept + writes a +composition_edge_checked SSB row;
negative arm: a clean corpus with a hand-inserted dangling edge that EXITS 0 is a FAIL (fail-closed, R1).

### N-3 — generalized anti-selection-bias promotion primitives
Tier 2 | finding W-30 | grounding STRONG | nature ACTIVATION | effort M-L | WAVE 3 | depends N-1
Steal/build: H-17 (persisted trial ledger) + H-18 (deflated-significance DSR/PBO/MinBTL, fail-closed on missing
N) + H-19 (reusable-holdout DP budget) were built FINANCE-ONLY (TB-1) — but every promotion loop runs the same
max-of-N-noisy-scores search and is upward-biased: 3F rule SHADOW->ACTIVE, J-57 prompt promotion, J-97 case
reuse fitness, B-23/H-45 configs, I-4/I-25 candidate ladders, L-11 OQ thresholds. Only H-47 partially reuses
H-19. N-3 lifts the three into DOMAIN-GENERIC primitives read by every promotion gate via the ID-6 ledger hook
(I-30/I-31 already route promotions "through H-17 trial-N + H-18 deflation" — proof the generalization is
natural); every reusable eval corpus (J-56/L-20/D-6) gets a Thresholdout budget.
COMPOSES-WITH: H-17/H-18/H-19 (the machinery, verbatim — extend the KEY from finance-candidate to
(candidate-family)), ID-6 ledger hook, N-1's outcome labels (the ground truth the deflation corrects against).
Do NOT re-implement DSR/PBO — re-key it.
Integration: crates/orchestrator promotion sites (rules lifecycle, prompt/case/config archives). frontend_pair:
a deflation column on the existing promotion drawers (or N/A justified).
observable_effect: trigger a real SHADOW->ACTIVE promotion with N<threshold trials -> +promotion_deflated
(DeflationBlocked) SSB row + the promotion is HELD; negative arm: a promotion that ignores trial count and flips
ACTIVE must FAIL the gate.

### N-4 — one Claim/Belief object + receipt memo
Tier 2 | finding W-2+W-4+W-22+W-36 | grounding STRONG | nature BUILD-on-shipped | effort L | WAVE 3
Steal/build: four typed objects all mean "a statement the system may or may not trust, with evidence": L-1
ClaimSet{ClaimKind,span,refs}, J-83 BeliefState entries{claim,receipt,abstain,DKG-provenance}, K-12
critical_atoms{kind,value,must_preserve}, GRC grounding tags "type:reference". N-4 makes them ONE: L-1 typed
claims ARE J-83 entries (L-8 already threads this — finish it); the receipt at the J-83 gate IS the evidence L-3
round-trips; K-12 must_preserve = has-receipt; J-21 re-derives the SAME object against commit+AST. AND the
receipt MEMO (W-4+W-22+W-36): before running an expensive checker/resample, look up claim-identity in the J-96
case graph (the C-30 OracleAnswerCache keying discipline); a hit re-validates cheaply via J-21 identity instead
of re-running; misses mint new nodes. The case graph becomes the RECEIPT CACHE consulted by both the J-83
reasoning gate AND the L-2 CGG — "gets cheaper on YOUR repo" (K-18) applied to reasoning, biggest win on the
checker-poor local 7B lane.
COMPOSES-WITH: L-1, J-83 BeliefState (L-8 carrier), K-12, J-96 case nodes, C-30 OracleAnswerCache, J-21
commit+AST identity, F-5 staleness cascade, L-2..L-6 CGG. Do NOT re-implement the cache — generalize C-30.
Integration: crates/orchestrator ssb::payloads BeliefState + the L-2/J-83 gate call sites. frontend_pair: the
existing belief/claim drawers gain a memo-hit indicator (or N/A justified).
observable_effect: trigger a recurring claim about unchanged code twice -> first pays the checker + mints a J-96
node, second hits +receipt_memo(Hit) and skips the checker; negative arm: a claim on CHANGED code (F-5-invalidated)
that returns a stale Hit must FAIL.

### N-5 — policy / intensity arbitration plane
Tier 2 | finding W-3+W-17+W-35+W-14 | grounding STRONG | nature BUILD | effort L | WAVE 4 | depends N-1
Steal/build: ten deterministic routers (D-10 grounding / J-84 effort / J-55 modality / H-7 role / E-20 local-cloud
/ J-48 serving / J-54 free-pool / L-6 curiosity / J-32 front-door / J-73 should-retrieve) with NO spec for what
happens when they disagree (J-84 escalate vs E-39 cost-block; L-6 retrieve vs J-73 abstain; J-48 cloud-spill vs
J-54 sleep) — resolved implicitly by call order in task_runner (the "dispatch order is prose" failure J-92 flags
for the FSM, at the policy layer). PLUS six verification-intensity deciders on uncoordinated axes (B-41 level /
L-21 model / J-84 effort / C-46 delegation / K-5 gate-risk / B-28 edit-format). N-5 declares a PRECEDENCE spec
(DATA: safety > cost > abstention > escalation > modality > serving-tier) with ONE composite RouteDecision row,
and ONE intensity resolver over (risk x model x level x task-difficulty x delegation) with all six deciders as
axes (B-28 becomes J-55's edit-format leg, W-14). K-4 gate-tax then measures ONE policy.
COMPOSES-WITH: the ten router units + the six intensity units (as axes/rows, verbatim), J-92 FSM-substate
structural-enforcement pattern (lifted to routing order), K-4 gate-tax as the instrument. Do NOT re-implement any
router — arbitrate them.
Integration: crates/orchestrator task_runner routing seam + a frozen precedence table (R8). frontend_pair: a
RouteDecision explain-row on the canvas/RRA (W-7 render).
observable_effect: trigger a task hitting two conflicting routers -> ONE +route_arbitrated row naming both +
winner + precedence-rank; negative arm: a routing decision with no arbitration row when >=2 routers fired must
FAIL (the seam the R10 integration gate exists to catch).

### N-6 — provenance / trust lattice + admission-gate family
Tier 2 | finding W-19+W-29 | grounding STRONG | nature BUILD | effort L | WAVE 4
Steal/build: six independent "how much do we trust this byte's origin" scales — G-1 taint Provenance (4 ingest
sites), C-48 memory TrustTier T0-T3, F-6 SkillTrustTier S1-S4, C-30 oracle source_tier T1-T4, K-23 per-repo
trust, 4D EvidenceLevel — with only ONE pairwise wire (J-93-C taint-survives-memory). A T2 memory row, a G-1
tainted string, and a T2 citation are three unrelated T2s. N-6 declares ONE provenance lattice: a total SEMANTIC
mapping between the scales (NOT one enum — F-6 deliberately kept its tier distinct, so MAP don't merge), carried
on the SAME object ingest(G-1) -> memory(C-48) -> citation(H-53) -> oracle-TTL(C-30) -> skill-autonomy(F-6) ->
cross-repo(K-23). PLUS the ADMISSION family: F-1 VibeGate + J-102 config-ingress + J-109 learned-artifact + the
new K-23/M-2 surfaces share ONE scanner registry (they already share legs — J-102 reuses F-1's secret-grep, J-109
indexes off J-102's manifest — just undeclared as a family).
COMPOSES-WITH: the six trust scales + J-93-C + the three admission gates + F-1's secret-grep leg. Do NOT collapse
F-6's enum into C-48's — declare the mapping.
Integration: crates/security (provenance/taint) + the admission gate call sites; a lattice mapping table. F-6's
auditor-enforced type barrier stays. frontend_pair: a provenance badge on cited/memory surfaces (or N/A justified).
observable_effect: trigger an ingest -> memory-write -> citation chain and assert the trust label MAPS coherently
end-to-end (no silent T-reset at a boundary); negative arm: a tainted string that loses its taint at the
memory/citation/skill boundary must FAIL (four fences with gaps is the thing this closes).

### N-7 — human render as a REQUIRED verdict field
Tier 2 | finding W-7 | grounding MODERATE | nature BUILD | effort M | WAVE 5
Steal/build: the engine is ~200 units deep but the surfaces where a human SEES the trust value are few and late
(K-11 PR-evidence, K-7 brief, G-5 cockpit, M-3 rollup, canvas). Evidence is machine-shaped by default (RRA
bundles, SSB rows, hashes); only K-11 converts it to human-viewable form and only for PRs. The reasoning receipts
(J-83), hallucination verdicts (L-7), measurement numbers (N-1) each produce human-decision-relevant artifacts
with no render but a drawer. N-7 makes "human render" a REQUIRED field of every gate/verdict family the way
+SSB-row is required today: each verdict class declares its K-11-style render (PR comment / chat ACK line /
canvas badge / brief row / M-3 org rollup). ONE render substrate, many verdict producers.
COMPOSES-WITH: K-11 PR render (the template), G-5 cockpit, K-7 brief, M-3 rollup, the SSB verdict rows. Do NOT
re-author K-11 per surface — make it a substrate.
Integration: crates/orchestrator verdict emission + the render substrate; the WIRING_CONTRACT template gains a
render_field requirement (ties N-2). frontend_pair: MANDATORY — this IS the frontend unit; the render targets.
observable_effect: trigger any verdict-emitting gate -> a human-readable render lands at its declared target
(PR/chat/canvas/brief); negative arm: a new verdict class shipped with no declared render fails the N-2 authoring
check (render_field becomes a required contract field).

==== TIER 3 — MEDIUM-HIGH ====

### N-8 — governance-coverage substrate
Tier 3 | finding W-18+W-34+W-31 | grounding STRONG | nature BUILD | effort M | WAVE 5
Steal/build: five instruments of "is our own governance real," none citing more than one other: B-17
gate-liveness ("does the gate fire"), K-22 doctrine-enforcer coverage ("does the rule have an enforcer"), J-46
debt-RATE, J-70 gate-manifest traceability, + the proposed H-27 kill-your-own-mutant-corpus ("does the enforcer
actually CATCH", today finance-only). N-8 is ONE substrate with five views, and it is the MARKETABLE one: "N% of
ELAI's rules are machine-enforced, each enforcer mutation-proven, drift-rate X/quarter" — a K-1-dossier-grade
claim no competitor can utter. Makes kill-your-own-mutant-corpus a gate-manifest LIFECYCLE requirement (a new
gate cannot enter the frozen set without its mutant corpus + measured kill rate).
COMPOSES-WITH: B-17, K-22, J-46, J-70, H-27 (generalized from finance), the gate-manifest (R8 freeze). Do NOT
re-implement any of the five — view them.
Integration: .claude/tools/ governance coverage + the gate-manifest lifecycle. frontend_pair: a governance tile
(the marketable number) on G-5/M-3.
observable_effect: trigger the coverage run -> +governance_coverage_measured with the five Kinds + a new gate
without a mutant corpus is REFUSED entry to the frozen set; negative arm: a decorative rule (clause with no
enforcer) reported as covered must FAIL.

### N-9 — one counterfactual / ablation engine
Tier 3 | finding W-20+W-27 | grounding STRONG | nature BUILD | effort M | WAVE 6 | depends N-1
Steal/build: the SAME mechanism (re-run/re-decide with one factor removed or injected, score the delta off
journal outcomes) is specified EIGHT times: four attribution engines (C-45 role credit / I-28 DAG credit / J-105
memory ROI / J-109 replay fitness) + four ablation harnesses (E-6 iso-budget role / E-10 critic / H-40
verifier-retention / J-75 RRF). N-9 is ONE drop-one/inject-one engine over the SSB with TARGET as a parameter
(role / verifier / ranker / rule / memory / case) + eight thin adapters (ND-5). Natural consumer of N-1's
outcome labels (W-11).
COMPOSES-WITH: the four attribution + four ablation units (as adapters), N-1 labels. Do NOT re-implement the
replay mechanism eight times.
Integration: crates/orchestrator counterfactual engine + the eight call sites. frontend_pair: reuse existing
attribution drawers (or N/A justified).
observable_effect: trigger a role/verifier/ranker ablation through the ONE engine -> +counterfactual_scored with
the right Kind; negative arm: an adapter that carries business logic instead of delegating to the core fails ND-5.

### N-10 — clarity / novelty -> budget wires
Tier 3 | finding W-15+W-24 | grounding STRONG | nature ACTIVATION | effort M | WAVE 6
Steal/build: four passive clarity detectors (B-38 continuous ambiguity / B-40 critic / C-29 intent pairs / J-32
front-door confidence) feed only the J-79 grill OFFER; two novelty detectors (D-15 NoveltyClass on claims / 4F
B-21 novel-step on plans) feed only a human queue. Nothing feeds the MEASURED ambiguity/novelty of THIS task
into how much research/reasoning it gets. N-10 types them once (AmbiguityScore + NoveltyScore, joined) and
declares consumers: research-tree depth (4D), reasoning-effort tier (J-84), decomposition node-sizing (J-39),
grill/clarify offer (J-79). High-ambiguity OR off-map -> deeper research + more clarify, not just a chip.
COMPOSES-WITH: B-38/B-40/C-29/J-32 (ambiguity), D-15/4F/B-21 (novelty), 4D/J-84/J-39/J-79 (consumers). Do NOT
re-detect — wire the existing detectors.
Integration: crates/orchestrator research-tree + effort-tier + node-admission seams. frontend_pair: N/A (internal
resource allocation); justified, unless the depth choice is surfaced on the canvas.
observable_effect: trigger a deliberately vague/novel task -> research depth + effort tier rise vs a clear
baseline task, reading the same AmbiguityScore/NoveltyScore rows; negative arm: a high-ambiguity task that gets
baseline depth must FAIL (the detectors are wired to budgets, not just the offer).

### N-11 — journal-fold substrate
Tier 3 | finding W-10 | grounding STRONG | nature BUILD-on-shipped | effort M-L | WAVE 7
Steal/build: L-16's pure fold(log,budget,policy)->context-view is the RIGHT unifier; five OTHER surfaces
re-derive-from-journal ad hoc (J-31/J-60/J-98/K-23 canvas altitudes, I-39 plan status, K-24 billing balance,
M-3 fleet rollup, K-22 coverage). N-11 extends L-16's fold core beyond context: per-surface fold FUNCTIONS as
data/plugins, ONE J-68 Kani correctness proof covering all five instead of five separate GD-2 audits. Ships the
ND-2 RETIREMENT map for the paths L-16 leaves live (recover_stubs end-of-life; declare H-49 CAS/undo-tree THE
log representation).
COMPOSES-WITH: L-16 fold (EXTEND, do not re-author), J-68 Kani proof, the five re-derive surfaces, H-49 CAS.
Integration: crates/orchestrator fold core + the five surface fold-fns. frontend_pair: canvas altitudes already
consume folds (verify, not re-wire).
observable_effect: trigger two of the five surfaces through the ONE fold core with a pinned seed -> byte-identical
re-derivation + the retirement switch removes the old path; negative arm: a surface still re-deriving off its own
ad-hoc code path must FAIL the conformance check.

==== TIER 4 — MEDIUM ====

### N-12 — consistency + dedupe bundle
Tier 4 | finding W-8+W-9+W-12+W-16+W-21+W-25+W-26+W-28 | grounding STRONG | nature DELETE | effort M | WAVE 8
Steal/build: eight independent same-concern collapses, DISJOINT files (parallel-safe sub-slices):
(a) W-8+W-26 ONE FailureClass enum (failure_taxonomy.py 13-cat + tools 9-cat + E-6 MAST 14-mode + E-29 MAST dup
+ C-10 StagnationPattern) shared by auditor/anti-spiral/MAST/corpus-rows/case-edges;
(b) W-9 ONE AbstainDecision spine (D-16/I-8/D-17/L-7/J-73/J-110) + composition rule (upstream retrieval-abstain
taints downstream answer-without-evidence -> L-2 risk floor);
(c) W-16 ONE ProjectClass registry (B-40's 9-variant, consumed by B-42 cosmos / E-50 archetypes / E-12 manifests
/ L-14 defaults);
(d) W-21 route C-47 sled CaseBase + C-49 markdown ExperienceCorpus writes through the J-109 admission pipeline
(closes an unguarded learned-artifact write path);
(e) W-25 ONE drift-primitive library (D-4/G-10/H-30/F-5/B-22b share a KL/DDM/IQR core, keep distinct policies);
(f) W-12 FSM three-model conformance test (3I DST == J-68 TLA == code transition table);
(g) W-28 ONE LSP daemon pool (E-52 verifier + J-18 retrieval channel + J-52 per-edit block-feed).
COMPOSES-WITH: every unit named above (dedupe to the FIRST-in-flow, adapters for the rest). Do NOT add a new
type for any of these concerns — collapse to the existing canonical one.
Integration: the respective crates; each sub-slice is independently shippable. frontend_pair: N/A for most (a
FailureClass drawer if surfaced); justified per sub-slice.
observable_effect: per sub-slice — trigger the one surviving implementation from >=2 former call sites -> one
canonical row/type; negative arm: a second parallel implementation of a collapsed concern re-appearing fails the
N-2 duplicate-concept check.

==== EXTERNAL-STEAL TRACK — four-source-steal 2026-07-05 ====
Routed to N (not J — full) because the highest-value verdict is a textbook N unification. Source: the PHASE-4
steal analysis workflow/reasoning/four-source-steal-2026-07-05.md over programasweights (repo) + arXiv:2607.02440
EvoPolicyGym + elai-any-model-research-gaps.md + the Ford transcript. Only genuine NEW/EXTENSION verdicts land as
units; REJECTED (PAW compiled-LoRA tier, EvoPolicyGym n=1 stats + missing-memorization-defense, gaps-doc P3-P5
already-planned-deeper, Ford checklist as a build item) go to the feature-map ledger; prefix-KV (S1-1) is an
ALREADY-recorded rider (J-95/H-50); zip-slip (S1-2) folds into N-6's admission family; the ADR-evidence-table
discipline (S1-3), the K-1/K-7 packaging (S4-1/S3-4/S1-6) and the EvoPolicyGym bench (S2-4) are feature-map
dossier/K-2 notes.

### N-13 — OutputBoundary + structured-output capability matrix
External-steal | verdicts S3-1+S3-2 (gaps-doc P1/P2) | grounding STRONG (corpus read) / doc-side [EXTERNAL] |
  nature BUILD/UNIFY | effort L | WAVE 9
Steal/build: the derived gaps doc named the ONE hole the corpus has only in fragments — no single named invariant
"every machine-actionable model output crosses ONE provider-agnostic boundary", and no per-type registry {schema,
parser, repair policy, retry budget, fail-closed action, journaled invalid-rate, backend enforcement mode}. The
machinery exists in >=8 places (C-26 DecodingMode / E-35 XGrammar+INSTRUCTOR / I-34 CRANE+grammar-pin+post-parse-
revalidate / J-53 tool-input-repair / J-51 adherence-floor / H-13 canvas-grammars / B-24 VLM-JSON / L-1 ClaimSet /
I-33 CEFL-schemas) with no unifier — the exact N-2 duplicate-concept class. N-13 declares ONE OutputBoundary
registry (existing mechanisms become parameterizations, NO new engine, GD-1) + structured-output capability FIELDS
on the ONE catalog (B-37/I-51) + a strongest-available-enforcement resolver (provider strict > native tool > local
CFG > regex > parse-validate-retry-fail-closed; the model never picks, TD-3). Inherits I-34 grammar-pinning +
H-13 reject-never-repair + J-53/I-46 frozen-repair-table as a binding security rider.
COMPOSES-WITH: the >=8 constrained-output machines (parameterized, NEVER re-implemented), B-37/I-51 catalog, N-1
spine (invalid-rate sink), N-2 authoring check, D-16 refusal. Do NOT mint a validation engine or a second catalog.
Integration: crates/orchestrator/src/output_boundary.rs + the ONE catalog + the four machine-actionable emission
seams. frontend_pair: a boundary-verdict badge on claim/decision/tool drawers (or N/A if G-5 surfaces the tile).
observable_effect: emit a tool directive / ClaimSet / decision card through the real path on a backend lacking
strict schema -> one +output_boundary_verdict row with the RESOLVED enforcement_mode; per-(type,backend) invalid-
rate lands in N-1; negative arm: a new output type with no registry row FAILS the N-2 authoring check, a second
bespoke validation path FAILS the duplicate-concept check.

### N-14 — self-improvement edit-loop instruments (EvoPolicyGym)
External-steal | verdicts S2-1+S2-2+S2-3+S2-5 | grounding STRONG (arXiv:2607.02440 verified 2026-07-05; n=1 caveat
  carried) | nature BUILD/ACTIVATION | effort M-L | WAVE 9
Steal/build: EvoPolicyGym's deterministic edit-type classifier is finer than ELAI's stagnation stack — anti-spiral
C-10 detects Spinning/Oscillation/NoDrift/DiminishingReturns off content hashes + plateaus but cannot say "churning
STRUCTURE without traction" vs "tuning constants inside the WRONG structure" (the paper's two failure modes; leaders
convert synthesis edits 41-48%, weak models churn structure 3-10% while competitive on tuning 25-38%). N-14 ships
four ADVISORY (TD-3) instruments: (1) stripped-AST EditKind classifier {SynthesisEdit=new topology after
constants+identifiers normalized / ParametricEdit=same topology} joined onto every score-bearing attempt (upgrades
C-10 persona choice from blind rotation to evidence-driven, feeds I-4/I-22/J-10); (2) synthesis-vs-tuning TASK-DEMAND
as an N-5/H-7/L-21/J-84 routing axis (protects the local-7B lane from owning structure discovery — the measured
weak-model synthesis collapse); (3) two-stage hidden SELECT-then-REPORT on H-45/H-47 (select on hidden validation,
report on disjoint held-out); (4) improvement-event-timing on I-27. ELAI-measured thresholds only (ID-5).
COMPOSES-WITH: C-10/B-22b, I-4/I-22/J-10, I-27 (EXTEND the curve), H-45/H-47/D-6/H-19, N-5/H-7/L-21/J-84, N-1 sink;
tree-sitter in crates/knowledge. Do NOT re-implement the detector, the curve, or the promotion machinery.
Integration: crates/orchestrator/src/edit_kind.rs + the anti-spiral / I-27 / promotion / routing seams. frontend_pair:
an edit-kind chip on the anti-spiral/receipt drawer (or N/A internal-advisory).
observable_effect: run an attempt with a constant-only then a restructuring edit under ELAI_SEED -> Parametric then
Synthesis; repeated no-hit synthesis churn shifts the persona to Architect/Simplifier; negative arm: a whitespace/
rename-only diff flipping topology FAILS the normalization test; a promotion reporting the SELECTION-slice number
FAILS the two-stage gate.

### N-15 — anti-affirmation stability measurement
External-steal | verdicts S4-2+S4-3 | grounding STRONG (arXiv:2510.01395 verified 2026-07-05; HBR framing [EXTERNAL])
  | nature BUILD/ACTIVATION | effort M | WAVE 9
Steal/build: ELAI's sycophancy defense achieves structural irrelevance at the decision boundary (TD-3, I-17 L1-veto,
I-33 externalized candidate set, H-31 judge firewall, D-14 inverted-framing, E-4 chat drift) but the ONE residue is
advisory prose a human reads directly — detection-covered, not structurally covered. N-15 closes it: (1) advisory
framing-invariance — permute presentation/option order across the N advisory samples (B-41 review / J-45 cross-family
/ L-11 OCP-LSP / I-41 lenses); an advisory whose verdict flips under presentation-only permutation is logged
framing-unstable and its weight ZEROED for the run (the I-24 scorer-neutralization pattern applied to framing);
(2) two D-23 fixture classes — option-order inversion (HBR: same facts reordered, decision must not flip) +
assertion-pull ("I think X" on false X, adoption rate with/without the D-14 probe, arXiv:2510.01395). Advisory-only
(the harm is already bounded — this makes it measured + disqualifying), ELAI-measured (ID-5), GD-4 FP-bound before
default-ON.
COMPOSES-WITH: I-33/I-24 (pattern EXTENDED to advisory lanes), B-41/J-45/L-11/I-41, D-23/E-54/D-14 (supply fixtures,
do NOT re-author the runner), N-1 sink, N-8 governance view. 
Integration: crates/orchestrator/src/framing_invariance.rs + the D-23 corpus + the advisory emission seams.
frontend_pair: a framing-stability tile on G-5/N-8 (or N/A if the sycophancy-bench tile surfaces it).
observable_effect: run an advisory on a presentation-order-sensitive input, then `elai bench sycophancy` with the two
fixtures -> a flip emits +framing_stability_measured(FramingUnstable) + weight zeroed; assertion-pull emits adoption
rates with/without the probe; negative arm: a presentation-STABLE advisory logged framing-unstable FAILS (FP), and a
framing check default-ON with no measured FP bound FAILS GD-4.

### N-16 — governance closure: evaluator-refutation verdict-collapse + a lambda exogenous-grounding floor + adjudicator-ritual metrics
External-steal (DeepReason 2026-07-08) | verdicts S4+S6+S7 | grounding STRONG (steal-deepreason-2026-07-08.md; ELAI
  anchors PLAN-asserted, re-grep W-7) | nature EXTENSION(N-8) + BUILD | effort M | WAVE 9
Steal/build: DeepReason's spec §11 treats ADJUDICATOR capture as first-class — criticism ritualizes while statuses
stay formally correct, and verdicts keep standing after the standard that issued them is refuted. That is the residue
N-8's five liveness/coverage views do NOT catch; N-16 adds three views over the SAME "what governance is decorative"
question (ND-1 EXTENSION, not a new framework): (S4) every SSB verdict row carries an evaluator-id + version-hash
(additive field), and a registered evaluator-refutation event (a failed H-27 mutant corpus / an I-24/N-15 bias
detection / a frozen-tool bugfix) triggers an F-5 staleness cascade quarantining exactly the verdicts that evaluator
issued into an advisory re-audit queue, NEVER deleted (his rubric-closure taken WITHOUT the Dung argument-graph
adjudicator, REJECTED §4.3); (S6) a lambda exogenous-grounding floor — the windowed fraction of accepted verdicts
whose support bottoms out in a program/off-agent/retrieval/human anchor vs a model rubric — that fires a retrieval/
escalation BRAKE below the floor and is NEVER a PASS signal (detects STALLED/closed-loop dynamics, not wrong-but-
stable consensus, his honest §17 residue); (S7) adjudicator-ritual metrics — auditor finding-class entropy (probing
new surfaces vs re-litigating the same nodes) + the fraction of shipped units whose declared VERIFY leg never ran
(criticism debt) + the rate at which gates themselves get challenged/amended. Advisory-only (TD-3), measured through
N-1 (ND-4), ELAI-measured thresholds (ID-5).
COMPOSES-WITH: N-8 (the substrate EXTENDED with three views), F-5 (quarantine cascade — reuse, never re-author),
H-27 + I-24 + N-15 (the refutation events), gate-manifest + gate_freeze_check.py (R8), N-1 sink, D-17 (the escalation
brake), R4 off-agent doctrine (the exogenous anchor S6 counts). Do NOT re-implement the five N-8 views, F-5, or the
detectors.
Integration: crates/orchestrator (the verdict-row field + the F-5 cascade hook + the lambda/ritual derivations) +
.claude/tools (the governance-coverage checker gains three views) + the N-1 sink. frontend_pair: the lambda gauge +
quarantine-queue count + ritual-entropy band on the N-8 / G-5 / M-3 governance tile.
observable_effect: register an evaluator-refutation event then run the governance-coverage checker -> the refuted
evaluator's past verdicts land in the advisory re-audit queue (+evaluator_closure_swept), lambda under the floor fires
a brake (+grounding_ratio_measured BelowFloorBraked), a shipped unit with an un-run VERIFY leg is counted as debt;
negative arm: a quarantined verdict silently deleted FAILS (F-5 append-only), a lambda/ritual number used as a PASS
gate FAILS (TD-3), a governance number asserted without an N-1 measurement FAILS (ND-4).

## Track cross-cutting (what makes N different)
Every other letter ADDED capability; N SUBTRACTS fragmentation. Its done-gates are unusual: the negative arm is
almost always "a SECOND implementation of the unified concern must FAIL" (the anti-fork assertion), and N-2 is
the only unit in the corpus whose observable_effect runs at PLAN-AUTHORING time, not code-run time. N is also the
letter that closes the loop the user feared: after N-2 ships, "plans don't talk to each other" is a machine-caught
BLOCK, not a hand-trusted narrative.

End of N-SUMMARY_INDEX.md.
