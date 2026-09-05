# N-plan exhaustive feature discovery

## Scope and evidence discipline

- Primary source read completely: `docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md`, 363/363 lines.
- Orientation-only source: `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`, especially lines 419-431. It corroborates N as the interlace/anti-fragmentation layer but is not used to add details absent from the N summary.
- This pass intentionally did **not** read the N detail files. They are enumerated in the deeper-source manifest for later agents.
- Unless marked **Inferred** or **Ambiguous**, every inventory item below is explicit in the summary. No implementation completion is inferred from plan language.
- Source anchors use `N-SUMMARY_INDEX.md:Lx-Ly`.

## Quantitative coverage

| Measure | Coverage |
|---|---:|
| Primary-source lines read | 363/363 (100%) |
| N unit headings captured | 16/16: N-1 through N-16 (100%) |
| Tier-1 units | 1/1 |
| Tier-2 units | 6/6 |
| Tier-3 units | 4/4 |
| Tier-4 units | 1/1 |
| External-steal units | 4/4 |
| Units with metadata, composition inputs, integration seam, frontend disposition, positive observable, and negative arm captured | 16/16 |
| N detail/original files located | 17/17 expected (`N-0_INDEX.md` plus N-1..N-16) |
| Explicitly labeled N-12 sub-slices captured | 7/7, despite the prose saying eight |

The title says “All 15 sessions” and the generation note says N-13..N-15 were appended, but the file contains N-16 as well. This discovery treats the actual N-1..N-16 headings as authoritative and flags the header as stale (`L1-L2`, `L326-L354`).

## Program-level purpose and anti-fragmentation doctrine

Source: `N-SUMMARY_INDEX.md:L4-L18,L356-L361`.

- N is the **INTERLACE** letter: it primarily connects and consolidates capabilities from B..M rather than inventing new capability.
- The identified systemic failure is summary-driven plan authorship causing same-concern forks: four claim types, ten routers, six intensity deciders, multiple provenance scales, four counterfactual engines, and multiple context cores.
- N requires one canonical subsystem per fragmented concern.
- N mechanizes the M-1 anti-fragmentation rule so later letters cannot silently fork a concern again.
- N-2 lifts wiring enforcement from code grain to the complete plan-corpus grain.
- Every entry is intended to preserve the discharged W finding(s), impact tier, grounding strength, nature, unified implementations, consumed artifacts, real integration point, and observable-effect done gate.
- The normal N negative arm is an anti-fork assertion: a second implementation of a unified concern must fail.
- N-2 is exceptional because its done gate executes at plan-authoring time, not runtime.
- After N-2, plans failing to compose is intended to become a machine-caught block rather than a prose-only concern.
- Definitions of ND-1..ND-5 and sequencing are delegated to `N-0_INDEX.md`; per-finding evidence is delegated to `workflow/reasoning/elai-weakpoints-interlace-2026-07-04.md` (`L7-L8`).

## Exhaustive atomic feature inventory

### N-1 — measurement spine as ONE subsystem [KEYSTONE][BLOCKING]

Metadata: Tier 1; findings W-1, W-11, W-23; STRONG grounding; ACTIVATION; XL; Wave 1 (`L22-L23`).

- N1-E01 — Declare measurement as one named subsystem rather than scattered benches and “K slivers” (`L24-L26`).
- N1-E02 — Bind all ten named bench families to the shared spine: G-6 SAFEDELTA, J-34 context-knee, K-15 reasoning-knee, J-107 longcot, L-10 hallucination-delta, K-18 case-reuse, I-38 QDELTA, I-27 convergence, L-20 supply, and J-110 timeliness (`L26-L28`).
- N1-E03 — Use one corpus family rooted in J-56/E-54/L-20 (`L28-L30`).
- N1-E04 — Treat L-20 supply as a feeder, not a sibling bench (`L29-L30`).
- N1-E05 — Treat J-110 mutators as feeders, not sibling benches (`L29-L30`).
- N1-E06 — Use one runner family with the K-15/I-27 shape (`L30`).
- N1-E07 — Generalize the J-93-F rules-only outcome-label fold to the six promotion loops (`L30-L31`).
- N1-E08 — Feed J-84/J-85 knee selection from the shared measurements (`L28-L29`).
- N1-E09 — Feed L-4/L-7 promotion decisions from the shared measurements (`L28-L29`).
- N1-E10 — Feed K-24/M-5 pricing from the shared measurements (`L28-L29`).
- N1-E11 — Ship a default measurement bootstrap pack for fresh-install cold start (`L31-L33`).
- N1-E12 — Mark bootstrap defaults with `provenance=BOOTSTRAP` (`L31-L32`).
- N1-E13 — Include bootstrap values for D-16 tau, J-34 knees, K-15 knees, and OQ thresholds (`L32-L33`).
- N1-E14 — Hard-supersede bootstrap values when local measurements become available (`L32-L33`).
- N1-E15 — Use the W-5 `ModelCapabilityProfile` / I-51 `models.toml` as the **only** calibration sink (`L34-L36`).
- N1-E16 — Reuse B-42’s default-artifact-pack shape for the bootstrap rather than creating another pack mechanism (`L34-L36`).
- N1-E17 — Expose the family through the existing CLI seam `elai bench <any-family>` / `elai bench *` (`L37-L40`).
- N1-E18 — A bench run writes both a corpus row and a calibration curve into the one profile (`L40-L41`).
- N1-E19 — Replacing a bootstrap default with local measurement appends `+calibration_bootstrap_superseded` to the SSB (`L40-L42`).
- N1-E20 — A bench writing a knee to any store other than the profile must fail under ND-4 (`L41-L42`).
- N1-E21 — Integrate corpus/bench work in `crates/knowledge` and profile/label-fold work in `crates/orchestrator` (`L37-L38`).
- N1-E22 — Provide a `MeasurementPanel` tile sourced from the spine index, unless G-5 already surfaces it and the N/A decision is justified (`L38-L39`).
- N1-E23 — Do not implement another bench runner; existing benches bind to the one runner and profile (`L34-L36`).

Ambiguous/needs detail-file resolution:

- N1-A01 — The “~30 features across 6 letters” are not individually listed in the summary (`L24-L25`).
- N1-A02 — The six promotion loops generalized by J-93-F are not enumerated as a closed set (`L30-L31`).
- N1-A03 — OQ threshold ownership is not identified by unit in this section (`L32`).
- N1-A04 — Whether the `MeasurementPanel` is mandatory depends on G-5 coverage (`L38-L39`).

### N-2 — corpus-grain composition validator + M-1 one-registry rule

Metadata: Tier 2; findings W-6, M-1, W-33, W-32; STRONG; BUILD; L; Wave 2; depends on N-1 (`L46-L47`).

- N2-E01 — Parse feature-map `EXTENDS`, `composes`, `consumes`, and `gated-by` edges as data in the ND-3 format (`L48-L50`).
- N2-E02 — Join parsed edges against plan files (`L50`).
- N2-E03 — Join parsed edges against `elai.map` symbols (`L50`).
- N2-E04 — Fail on a dangling edge whose named consumed unit does not expose the named artifact (`L50-L52`).
- N2-E05 — Fail on an orphan producer (`L51-L52`).
- N2-E06 — Detect duplicate-concept collisions using name similarity (`L52-L53`).
- N2-E07 — Detect duplicate-concept collisions using shared-seam detection (`L52-L53`).
- N2-E08 — The duplicate check is intended to catch forks such as the four claim types, E-6/E-29 MAST duplication, and ten routers during authoring (`L52-L53`).
- N2-E09 — Emit the global unlock DAG (`L53-L54`).
- N2-E10 — Emit global fan-in rankings (`L53-L54`).
- N2-E11 — Mechanically derive summaries/status by composing J-93-G’s map-half derivation and I-39 journal status (`L54-L55`).
- N2-E12 — Check freshness across `SUMMARY_INDEX`, `0_INDEX`, and detail plans using the `elai_map_freshness` pattern (`L55-L56`).
- N2-E13 — Run at paper-to-plan phase 5 / planning authoring time as a CI gate (`L56-L62`).
- N2-E14 — Extend J-89 L8 per-plan DAG type-completeness to corpus scope rather than reimplementing it (`L58-L59`).
- N2-E15 — Add `.claude/tools/corpus_composition_check.py` beside `wiring_contract_check.py` (`L60-L61`).
- N2-E16 — Invoke it as `python .claude/tools/corpus_composition_check.py --map ELAI-FEATURE-MAP.md` (`L63-L64`).
- N2-E17 — Seeded dangling-edge, orphan, or duplicate-concept cases must produce a nonzero exit (`L63-L65`).
- N2-E18 — A check appends `+composition_edge_checked` to the SSB (`L63-L64`).
- N2-E19 — A hand-inserted dangling edge returning exit 0 is a fail-closed R1 failure (`L64-L65`).
- N2-E20 — The M-1/ND-1 rule applies to every future letter, not only N (`L56-L57`).
- N2-E21 — No runtime frontend is required because this is an authoring-time CI surface (`L60-L62`).

Ambiguous/needs detail-file resolution:

- N2-A01 — “orphan producer (a unit with >=1 consumer in NO letter)” is grammatically ambiguous; likely it means a producer with no consumer in any letter, but that interpretation is not promoted to explicit fact here (`L51-L52`).
- N2-A02 — Similarity thresholds, collision adjudication, edge schema, output filenames, and SSB payload fields are not specified in the summary.
- N2-A03 — J’s documented drift stops at J-91, but the exact repair/migration behavior is not stated (`L48-L49`).

### N-3 — generalized anti-selection-bias promotion primitives

Metadata: Tier 2; W-30; STRONG; ACTIVATION; M-L; Wave 3; depends N-1 (`L67-L68`).

- N3-E01 — Generalize H-17’s persisted trial ledger from finance-only use to all promotion domains (`L69-L75`).
- N3-E02 — Re-key H-17 from a finance-candidate key to `(candidate-family)` (`L76-L78`).
- N3-E03 — Generalize H-18’s deflated-significance machinery, including DSR/PBO/MinBTL, without reimplementing it (`L69-L78`).
- N3-E04 — H-18 remains fail-closed when trial count N is missing (`L69-L70`).
- N3-E05 — Generalize H-19’s reusable-holdout differential-privacy budget (`L69-L75`).
- N3-E06 — Route every promotion gate through these primitives using the ID-6 ledger hook (`L73-L77`).
- N3-E07 — Cover 3F rule lifecycle `SHADOW -> ACTIVE` (`L71-L72`).
- N3-E08 — Cover J-57 prompt promotion (`L71-L72`).
- N3-E09 — Cover J-97 case-reuse fitness promotion (`L71-L72`).
- N3-E10 — Cover B-23/H-45 config promotion (`L72`).
- N3-E11 — Cover I-4/I-25 candidate ladders (`L72`).
- N3-E12 — Cover L-11 OQ threshold promotion (`L72`).
- N3-E13 — Give each reusable J-56/L-20/D-6 evaluation corpus a Thresholdout budget (`L75`).
- N3-E14 — Use N-1 outcome labels as the ground truth against which deflation corrects (`L76-L77`).
- N3-E15 — Integrate at `crates/orchestrator` promotion sites for rules, prompts, cases, and config archives (`L79-L80`).
- N3-E16 — Add a deflation column to existing promotion drawers, or explicitly justify N/A (`L79-L80`).
- N3-E17 — A real promotion with N below threshold is held and emits `+promotion_deflated(DeflationBlocked)` (`L81-L83`).
- N3-E18 — Any promotion that ignores trial count and flips ACTIVE must fail (`L82-L83`).

Ambiguous/needs detail-file resolution:

- N3-A01 — The N threshold, privacy accounting parameters, and what constitutes each `candidate-family` are not stated.
- N3-A02 — H-47 partially reuses H-19, but required migration of that partial path is not specified (`L72-L73`).

### N-4 — one Claim/Belief object + receipt memo

Metadata: Tier 2; W-2, W-4, W-22, W-36; STRONG; BUILD-on-shipped; L; Wave 3 (`L85-L86`).

- N4-E01 — Unify L-1 `ClaimSet{ClaimKind, span, refs}`, J-83 `BeliefState` entries, K-12 `critical_atoms`, and GRC `type:reference` grounding tags as one claim/belief object (`L87-L91`).
- N4-E02 — Make L-1 typed claims be J-83 entries, completing the carrier already threaded by L-8 (`L89-L91`).
- N4-E03 — Treat the J-83 gate receipt as the same evidence that L-3 round-trips (`L90-L91`).
- N4-E04 — Map K-12 `must_preserve` to `has-receipt` (`L91`).
- N4-E05 — Have J-21 re-derive the same object against commit and AST identity (`L91`).
- N4-E06 — Before an expensive checker/resample, look up claim identity in the J-96 case graph (`L92-L94`).
- N4-E07 — Apply C-30 `OracleAnswerCache` keying discipline to the receipt memo (`L92-L93`).
- N4-E08 — On a memo hit, cheaply revalidate identity through J-21 instead of rerunning the expensive checker (`L93-L94`).
- N4-E09 — On a memo miss, run the check and mint a new J-96 node (`L93-L95`).
- N4-E10 — Use the J-96 case graph as the receipt cache (`L94-L95`).
- N4-E11 — Have both the J-83 reasoning gate and L-2 CGG consult that cache (`L94-L95`).
- N4-E12 — Invalidate memo eligibility through F-5’s staleness cascade when code changes (`L97-L103`).
- N4-E13 — Generalize C-30 rather than implementing another cache (`L97-L98`).
- N4-E14 — Integrate the unified object in `crates/orchestrator` `ssb::payloads::BeliefState` and L-2/J-83 gate call sites (`L99-L100`).
- N4-E15 — Add a memo-hit indicator to existing belief/claim drawers, or justify N/A (`L99-L100`).
- N4-E16 — First occurrence of a claim on unchanged code pays the checker and mints a J-96 node (`L101-L102`).
- N4-E17 — Second occurrence on unchanged code emits `+receipt_memo(Hit)` and skips the checker (`L101-L102`).
- N4-E18 — A changed-code claim returning a stale memo hit must fail (`L102-L103`).
- N4-E19 — The design is specifically intended to reduce reasoning verification cost on a user’s repository and especially on the checker-poor local 7B lane (`L94-L96`).

Ambiguous/needs detail-file resolution:

- N4-A01 — The final unified schema and exact mapping for `abstain` and DKG provenance are not fully enumerated.
- N4-A02 — Claim identity composition, cache TTL/eviction, checker eligibility, and “cheap” revalidation cost bounds are unspecified.

### N-5 — policy / intensity arbitration plane

Metadata: Tier 2; W-3, W-17, W-35, W-14; STRONG; BUILD; L; Wave 4; depends N-1 (`L105-L106`).

- N5-E01 — Arbitrate ten existing routers without reimplementing them: D-10 grounding, J-84 effort, J-55 modality, H-7 role, E-20 local/cloud, J-48 serving, J-54 free-pool, L-6 curiosity, J-32 front-door, and J-73 should-retrieve (`L107-L118`).
- N5-E02 — Resolve the J-84 escalation versus E-39 cost-block conflict explicitly (`L109-L110`).
- N5-E03 — Resolve the L-6 retrieve versus J-73 abstain conflict explicitly (`L109-L110`).
- N5-E04 — Resolve the J-48 cloud-spill versus J-54 sleep conflict explicitly (`L109-L110`).
- N5-E05 — Replace implicit `task_runner` call-order policy with an explicit precedence specification (`L110-L115`).
- N5-E06 — Store precedence as data in this order: `safety > cost > abstention > escalation > modality > serving-tier` (`L112-L114`).
- N5-E07 — Emit one composite `RouteDecision` row for an arbitration (`L113-L115`).
- N5-E08 — Unify six verification-intensity deciders as axes: B-41 level, L-21 model, J-84 effort, C-46 delegation, K-5 gate risk, and B-28 edit format (`L111-L115`).
- N5-E09 — Resolve intensity over `risk × model × level × task-difficulty × delegation` (`L113-L115`).
- N5-E10 — Make B-28 the edit-format leg of J-55 (`L114-L115`).
- N5-E11 — Reuse J-92’s FSM-substate structural-enforcement pattern for routing order (`L116-L118`).
- N5-E12 — Let K-4 gate-tax measure the one combined policy (`L115-L118`).
- N5-E13 — Freeze the precedence table under R8 (`L119-L120`).
- N5-E14 — Integrate at the `crates/orchestrator` `task_runner` routing seam (`L119`).
- N5-E15 — Render a `RouteDecision` explanation row on the canvas/RRA (`L119-L120`).
- N5-E16 — When two or more routers fire, emit one `+route_arbitrated` row naming both/all fired routers, the winner, and its precedence rank (`L121-L123`).
- N5-E17 — A route produced with no arbitration row while at least two routers fired must fail the R10 integration gate (`L121-L123`).

Ambiguous/needs detail-file resolution:

- N5-A01 — The source calls out E-39 in a conflict although E-39 is not in the stated ten-router list (`L107-L110`).
- N5-A02 — Tie-breaking within a precedence tier, policy-versioning, and how task difficulty is measured are not specified.
- N5-A03 — The declared intensity resolver tuple omits edit-format in its parenthesized axes even though B-28 is explicitly incorporated (`L113-L115`).

### N-6 — provenance / trust lattice + admission-gate family

Metadata: Tier 2; W-19, W-29; STRONG; BUILD; L; Wave 4 (`L125-L126`).

- N6-E01 — Define a single semantic provenance/trust lattice mapping G-1 taint `Provenance`, C-48 `TrustTier T0-T3`, F-6 `SkillTrustTier S1-S4`, C-30 oracle `source_tier T1-T4`, K-23 per-repo trust, and 4D `EvidenceLevel` (`L127-L136`).
- N6-E02 — Make the mapping total across all six scales (`L130-L132`).
- N6-E03 — Preserve separate enums where intentional; specifically do not merge F-6’s enum into C-48’s (`L130-L137`).
- N6-E04 — Carry mapped provenance on the same object through ingest at G-1 (`L132`).
- N6-E05 — Preserve/map it through memory at C-48 (`L132`).
- N6-E06 — Preserve/map it through citation at H-53 (`L132`).
- N6-E07 — Preserve/map it into C-30 oracle TTL decisions (`L132`).
- N6-E08 — Preserve/map it into F-6 skill autonomy (`L132-L133`).
- N6-E09 — Preserve/map it into K-23 cross-repository trust (`L132-L133`).
- N6-E10 — Retain the existing J-93-C taint-survives-memory wire as part of the lattice (`L129-L136`).
- N6-E11 — Create one admission-scanner registry shared by F-1 `VibeGate`, J-102 config ingress, J-109 learned-artifact admission, and new K-23/M-2 surfaces (`L133-L136`).
- N6-E12 — Reuse F-1’s secret-grep scanner leg in that family (`L133-L136`).
- N6-E13 — Preserve J-109’s indexing relationship to J-102’s manifest (`L134-L135`).
- N6-E14 — Keep F-6’s auditor-enforced type barrier (`L138-L139`).
- N6-E15 — Integrate lattice/taint in `crates/security` and wire the admission-gate call sites to a lattice mapping table (`L138-L139`).
- N6-E16 — Add a provenance badge on cited and memory surfaces, or justify N/A (`L138-L139`).
- N6-E17 — Test an ingest → memory write → citation chain for coherent end-to-end trust mapping with no silent tier reset (`L140-L142`).
- N6-E18 — Any tainted string losing taint at a memory, citation, or skill boundary must fail (`L141-L142`).

Ambiguous/needs detail-file resolution:

- N6-A01 — The premise says “five provenance scales,” while N-6 enumerates six (`L14,L127-L130`).
- N6-A02 — Exact lattice ordering/mapping cells, treatment of incomparable meanings, and whether K-23/M-2 are gates or only surfaces require detail.
- N6-A03 — The summary says “three admission gates” while naming F-1, J-102, J-109 plus K-23/M-2 surfaces (`L133-L136`).

### N-7 — human render as a REQUIRED verdict field

Metadata: Tier 2; W-7; MODERATE; BUILD; M; Wave 5 (`L144-L145`).

- N7-E01 — Make human render a required field for every gate/verdict family, analogous to the existing required SSB row (`L146-L152`).
- N7-E02 — Require each verdict class to declare its render target/form (`L150-L152`).
- N7-E03 — Support PR comments as render targets (`L151-L152`).
- N7-E04 — Support chat acknowledgment lines as render targets (`L151-L152`).
- N7-E05 — Support canvas badges as render targets (`L151-L152`).
- N7-E06 — Support brief rows as render targets (`L151-L152`).
- N7-E07 — Support M-3 organization rollups as render targets (`L151-L152`).
- N7-E08 — Generalize K-11 PR evidence rendering into one render substrate rather than reauthoring it for each surface (`L153-L154`).
- N7-E09 — Feed the substrate from many verdict producers, including J-83 reasoning receipts, L-7 hallucination verdicts, and N-1 measurements (`L148-L152`).
- N7-E10 — Integrate at `crates/orchestrator` verdict emission and the render substrate (`L155-L156`).
- N7-E11 — Add a required `render_field` to the `WIRING_CONTRACT` template, tied to N-2 (`L155-L156`).
- N7-E12 — The frontend pairing is mandatory because rendering is the feature itself (`L155-L156`).
- N7-E13 — Every verdict-emitting gate must deliver a human-readable render to its declared PR/chat/canvas/brief target (`L157-L158`).
- N7-E14 — N-2 must reject a newly shipped verdict class with no declared render (`L158-L159`).

Ambiguous/needs detail-file resolution:

- N7-A01 — The mapping from each existing verdict class to exactly one or multiple render targets is not enumerated.
- N7-A02 — Accessibility, localization, acknowledgement semantics, and delivery-failure behavior are absent.

### N-8 — governance-coverage substrate

Metadata: Tier 3; W-18, W-34, W-31; STRONG; BUILD; M; Wave 5 (`L163-L164`).

- N8-E01 — Unify five governance instruments as views of one substrate rather than reimplementing them (`L165-L173`).
- N8-E02 — View B-17 gate liveness: whether a gate fires (`L165-L166`).
- N8-E03 — View K-22 doctrine-enforcer coverage: whether a rule has an enforcer (`L166-L167`).
- N8-E04 — View J-46 governance-debt rate (`L166-L168`).
- N8-E05 — View J-70 gate-manifest traceability (`L167-L168`).
- N8-E06 — Generalize H-27’s finance-only kill-your-own-mutant corpus: whether an enforcer actually catches violations (`L167-L173`).
- N8-E07 — Produce a combined marketable governance claim: percentage of rules machine-enforced, mutation proof per enforcer, and drift rate per quarter (`L168-L170`).
- N8-E08 — Make mutant-corpus coverage a gate-manifest lifecycle requirement (`L170-L172`).
- N8-E09 — Refuse a new gate entry to the R8-frozen set unless it has a mutant corpus (`L170-L177`).
- N8-E10 — Also require a measured mutant kill rate before frozen-set entry (`L170-L177`).
- N8-E11 — Integrate governance coverage in `.claude/tools/` and the gate-manifest lifecycle (`L174-L175`).
- N8-E12 — Surface a governance tile on G-5/M-3 (`L174-L175`).
- N8-E13 — A coverage run emits `+governance_coverage_measured` with five Kinds (`L176-L177`).
- N8-E14 — A decorative rule with no enforcer must not be reported as covered; doing so fails (`L177-L178`).

Ambiguous/needs detail-file resolution:

- N8-A01 — The five `Kind` enum names and formula for the composite marketable number are not provided.
- N8-A02 — Required kill-rate threshold and quarter/window definition are unspecified.

### N-9 — one counterfactual / ablation engine

Metadata: Tier 3; W-20, W-27; STRONG; BUILD; M; Wave 6; depends N-1 (`L180-L181`).

- N9-E01 — Build one SSB-backed engine that reruns/redecides with one factor removed or injected and scores the delta from journal outcomes (`L182-L186`).
- N9-E02 — Parameterize target kind as role, verifier, ranker, rule, memory, or case (`L185-L186`).
- N9-E03 — Adapt C-45 role credit to the engine (`L183-L188`).
- N9-E04 — Adapt I-28 DAG credit to the engine (`L183-L188`).
- N9-E05 — Adapt J-105 memory ROI to the engine (`L183-L188`).
- N9-E06 — Adapt J-109 replay fitness to the engine (`L183-L188`).
- N9-E07 — Adapt E-6 iso-budget role ablation to the engine (`L184-L188`).
- N9-E08 — Adapt E-10 critic ablation to the engine (`L184-L188`).
- N9-E09 — Adapt H-40 verifier-retention ablation to the engine (`L184-L188`).
- N9-E10 — Adapt J-75 RRF ablation to the engine (`L184-L188`).
- N9-E11 — Keep all eight adapters thin under ND-5; business logic belongs in the core (`L185-L193`).
- N9-E12 — Consume N-1 outcome labels (`L186-L189`).
- N9-E13 — Integrate one counterfactual engine in `crates/orchestrator` and replace the eight call-site mechanisms (`L190-L191`).
- N9-E14 — Reuse existing attribution drawers or justify no frontend (`L190-L191`).
- N9-E15 — A role/verifier/ranker ablation emits `+counterfactual_scored` with the appropriate Kind (`L192-L193`).
- N9-E16 — An adapter containing business logic rather than delegating to the core fails ND-5 (`L193`).

Ambiguous/needs detail-file resolution:

- N9-A01 — Replay isolation, seed control, delta metric, injected-factor semantics, and allowable side effects are not stated.
- N9-A02 — The target enum lists six targets but the eight source adapters are not mapped one-to-one in the summary.

### N-10 — clarity / novelty -> budget wires

Metadata: Tier 3; W-15, W-24; STRONG; ACTIVATION; M; Wave 6 (`L195-L196`).

- N10-E01 — Unify B-38 continuous ambiguity, B-40 critic, C-29 intent pairs, and J-32 front-door confidence into one typed ambiguity signal (`L197-L204`).
- N10-E02 — Unify D-15 claim `NoveltyClass` and the 4F/B-21 novel-plan-step signal into one typed novelty signal (`L198-L204`).
- N10-E03 — Materialize `AmbiguityScore` (`L200`).
- N10-E04 — Materialize `NoveltyScore` (`L200`).
- N10-E05 — Join the two scores for the current task (`L200`).
- N10-E06 — Feed the shared scores into 4D research-tree depth (`L200-L203`).
- N10-E07 — Feed the shared scores into J-84 reasoning-effort tier (`L201-L203`).
- N10-E08 — Feed the shared scores into J-39 decomposition node sizing (`L201-L203`).
- N10-E09 — Feed the shared scores into J-79 grill/clarify offers (`L201-L203`).
- N10-E10 — High ambiguity or an off-map task increases research depth (`L202`).
- N10-E11 — High ambiguity or an off-map task increases clarification behavior, rather than merely showing a chip (`L202`).
- N10-E12 — Wire existing detectors; do not implement another detector (`L203-L204`).
- N10-E13 — Integrate at `crates/orchestrator` research-tree, effort-tier, and node-admission seams (`L205-L206`).
- N10-E14 — The default frontend disposition is N/A because this is internal allocation; canvas may surface depth if desired (`L205-L206`).
- N10-E15 — A deliberately vague/novel task must receive greater research depth and effort than a clear baseline while both consumers read the same score rows (`L207-L209`).
- N10-E16 — A high-ambiguity task receiving only baseline depth must fail (`L208-L209`).

Ambiguous/needs detail-file resolution:

- N10-A01 — Score ranges, normalization/join rule, cutoffs, exact budget increments, and the relationship between “off-map” and `NoveltyScore` are unspecified.
- N10-A02 — “more clarify” is explicit but whether it means stronger offer, mandatory query, or repeated grill is not stated.

### N-11 — journal-fold substrate

Metadata: Tier 3; W-10; STRONG; BUILD-on-shipped; M-L; Wave 7 (`L211-L212`).

- N11-E01 — Extend L-16’s pure `fold(log, budget, policy) -> context-view` core rather than reauthoring it (`L213-L219`).
- N11-E02 — Represent per-surface fold functions as data/plugins (`L215-L216`).
- N11-E03 — Move J-31/J-60/J-98/K-23 canvas-altitude derivations through the shared fold core (`L213-L220`).
- N11-E04 — Move I-39 plan-status derivation through the shared fold core (`L214-L220`).
- N11-E05 — Move K-24 billing-balance derivation through the shared fold core (`L214-L220`).
- N11-E06 — Move M-3 fleet-rollup derivation through the shared fold core (`L214-L220`).
- N11-E07 — Move K-22 coverage derivation through the shared fold core (`L214-L220`).
- N11-E08 — Use one J-68 Kani correctness proof for all five surface fold functions instead of five GD-2 audits (`L215-L219`).
- N11-E09 — Ship the ND-2 retirement map for old paths left live by L-16 (`L216-L218`).
- N11-E10 — End the `recover_stubs` path (`L217-L218`).
- N11-E11 — Declare H-49 CAS/undo-tree the canonical log representation (`L217-L219`).
- N11-E12 — Integrate the fold core and five fold functions in `crates/orchestrator` (`L220`).
- N11-E13 — Verify existing canvas altitudes consume the folds; do not blindly rewire them (`L220-L221`).
- N11-E14 — With a pinned seed, two of the five surfaces must rederive byte-identical output through the one core (`L222-L224`).
- N11-E15 — The retirement switch removes the old path (`L222-L224`).
- N11-E16 — Any surface retaining an ad-hoc journal-rederivation path fails conformance (`L223-L224`).

Ambiguous/needs detail-file resolution:

- N11-A01 — The summary calls these “five OTHER surfaces”; canvas altitude is one surface with four contributing unit IDs, which should be confirmed in detail.
- N11-A02 — Plugin schema, budgets/policies per surface, proof property, retirement timing, and recovery migration are absent.

### N-12 — consistency + dedupe bundle

Metadata: Tier 4; W-8, W-9, W-12, W-16, W-21, W-25, W-26, W-28; STRONG; DELETE; M; Wave 8 (`L228-L229`).

Cross-slice behaviors:

- N12-E01 — Treat the labeled consistency collapses as disjoint-file, parallel-safe, independently shippable sub-slices (`L230,L244-L245`).
- N12-E02 — For each concern, deduplicate to the first-in-flow implementation and use adapters at other call sites (`L242-L243`).
- N12-E03 — Do not add another type for a collapsed concern (`L242-L243`).
- N12-E04 — Trigger the surviving implementation from at least two former call sites and observe one canonical row/type (`L246-L247`).
- N12-E05 — Have N-2 reject any reappearing parallel implementation (`L247-L248`).
- N12-E06 — Default frontend disposition is N/A per sub-slice, except a surfaced `FailureClass` may have a drawer; each N/A needs justification (`L244-L245`).

Sub-slice (a), W-8+W-26 — failure taxonomy:

- N12-E07 — Create one `FailureClass` enum shared across the auditor, anti-spiral system, MAST, corpus rows, and case edges (`L231-L232`).
- N12-E08 — Collapse `failure_taxonomy.py`’s 13 categories into the canonical enum (`L231`).
- N12-E09 — Collapse the tools taxonomy’s 9 categories into it (`L231`).
- N12-E10 — Collapse E-6 MAST’s 14 modes into it (`L231-L232`).
- N12-E11 — Remove/collapse the E-29 MAST duplicate (`L231-L232`).
- N12-E12 — Map C-10 `StagnationPattern` into it (`L232`).

Sub-slice (b), W-9 — abstention:

- N12-E13 — Create one `AbstainDecision` spine spanning D-16, I-8, D-17, L-7, J-73, and J-110 (`L233-L234`).
- N12-E14 — Propagate an upstream retrieval abstention as taint on a downstream answer lacking evidence (`L233-L234`).
- N12-E15 — Raise that downstream path to the L-2 risk floor (`L233-L234`).

Sub-slice (c), W-16 — project classification:

- N12-E16 — Make B-40’s nine-variant `ProjectClass` the one project-class registry (`L235-L236`).
- N12-E17 — Have B-42 cosmos consume the registry (`L235`).
- N12-E18 — Have E-50 archetypes consume the registry (`L235`).
- N12-E19 — Have E-12 manifests consume the registry (`L235-L236`).
- N12-E20 — Have L-14 defaults consume the registry (`L235-L236`).

Sub-slice (d), W-21 — learned-artifact admission:

- N12-E21 — Route C-47 sled `CaseBase` writes through the J-109 learned-artifact admission pipeline (`L237-L238`).
- N12-E22 — Route C-49 Markdown `ExperienceCorpus` writes through the J-109 admission pipeline (`L237-L238`).
- N12-E23 — Close the previously unguarded learned-artifact write path (`L237-L238`).

Sub-slice (e), W-25 — drift primitives:

- N12-E24 — Provide one shared KL/DDM/IQR drift-primitive library for D-4, G-10, H-30, F-5, and B-22b (`L239`).
- N12-E25 — Preserve each consumer’s distinct policy while sharing only the primitives (`L239`).

Sub-slice (f), W-12 — FSM conformance:

- N12-E26 — Add a three-model conformance test requiring 3I DST, J-68 TLA, and the code transition table to agree (`L240`).

Sub-slice (g), W-28 — LSP lifecycle:

- N12-E27 — Create one LSP daemon pool shared by E-52 verifier, J-18 retrieval channel, and J-52 per-edit block feed (`L241`).

Ambiguous/needs detail-file resolution:

- N12-A01 — The source says “eight independent same-concern collapses” but labels only seven, (a) through (g). There are eight W findings because (a) combines W-8 and W-26; whether a sub-slice is missing is unresolved (`L229-L241`).
- N12-A02 — “First-in-flow” canonical owners, migration/deletion lists, enum mappings, and adapter boundaries are not enumerated.
- N12-A03 — Failure taxonomy category reconciliation and whether `StagnationPattern` remains a policy-specific sub-type are not stated.
- N12-A04 — LSP pool process isolation, language/workspace key, lifetime, crash recovery, and resource limits are absent.

### External-steal track disposition ledger

Source: `N-SUMMARY_INDEX.md:L250-L258`.

- NX-E01 — Route the four-source steal to N because its highest-value verdict is unification rather than another J unit.
- NX-E02 — Use `workflow/reasoning/four-source-steal-2026-07-05.md` as the phase-4 steal analysis over the programasweights repository, arXiv:2607.02440 EvoPolicyGym, `elai-any-model-research-gaps.md`, and a Ford transcript.
- NX-E03 — Promote only genuine NEW/EXTENSION verdicts into N units.
- NX-E04 — Reject PAW’s compiled-LoRA tier as an N build item and record it in the feature-map ledger.
- NX-E05 — Reject EvoPolicyGym n=1 statistics and missing-memorization-defense as N build items; carry the n=1 caveat into N-14.
- NX-E06 — Reject gaps-document P3-P5 because ELAI already plans them more deeply; record them in the feature-map ledger.
- NX-E07 — Reject the Ford checklist as a build item; record it in the feature-map ledger.
- NX-E08 — Treat prefix-KV S1-1 as already recorded by J-95/H-50.
- NX-E09 — Fold zip-slip S1-2 into N-6’s admission family.
- NX-E10 — Keep ADR evidence-table discipline S1-3 as a feature-map/dossier note rather than an N unit.
- NX-E11 — Keep K-1/K-7 packaging verdicts S4-1/S3-4/S1-6 as feature-map/dossier/K-2 notes.
- NX-E12 — Keep the EvoPolicyGym bench S2-4 as a feature-map/dossier/K-2 note rather than a new N unit.

### N-13 — OutputBoundary + structured-output capability matrix

Metadata: external-steal verdicts S3-1/S3-2 (gaps P1/P2); STRONG corpus-read grounding with doc-side external evidence; BUILD/UNIFY; L; Wave 9 (`L260-L262`).

- N13-E01 — Establish the invariant that every machine-actionable model output crosses one provider-agnostic `OutputBoundary` (`L263-L269`).
- N13-E02 — Create one per-output-type registry row with `schema` (`L264-L265`).
- N13-E03 — Include `parser` in each row (`L264-L265`).
- N13-E04 — Include `repair policy` in each row (`L264-L265`).
- N13-E05 — Include `retry budget` in each row (`L264-L265`).
- N13-E06 — Include `fail-closed action` in each row (`L264-L265`).
- N13-E07 — Include journaled invalid rate in each row (`L264-L265`).
- N13-E08 — Include backend enforcement mode in each row (`L265`).
- N13-E09 — Parameterize, rather than reimplement, C-26 `DecodingMode`, E-35 XGrammar/INSTRUCTOR, I-34 CRANE/grammar pin/post-parse revalidation, J-53 tool-input repair, J-51 adherence floor, H-13 canvas grammars, B-24 VLM JSON, L-1 `ClaimSet`, and I-33 CEFL schemas (`L266-L274`).
- N13-E10 — Add structured-output capability fields to the one B-37/I-51 model catalog (`L269-L274`).
- N13-E11 — Resolve the strongest available enforcement mode in order: provider strict > native tool > local CFG > regex > parse/validate/retry/fail-closed (`L270-L272`).
- N13-E12 — The model cannot choose or weaken enforcement mode (TD-3) (`L270-L272`).
- N13-E13 — Inherit I-34 grammar pinning as a binding security requirement (`L271-L272`).
- N13-E14 — Inherit H-13 reject-never-repair as a binding security requirement (`L271-L272`).
- N13-E15 — Inherit J-53/I-46’s frozen repair table as a binding security requirement (`L271-L272`).
- N13-E16 — Do not create another validation engine or model catalog (`L273-L274`).
- N13-E17 — Send per `(type, backend)` invalid rates to N-1 (`L273-L279`).
- N13-E18 — Let D-16 refusal handle fail-closed output (`L273-L274`).
- N13-E19 — Implement the registry seam at `crates/orchestrator/src/output_boundary.rs`, the one catalog, and four machine-actionable emission seams (`L275-L276`).
- N13-E20 — Add a boundary-verdict badge to claim/decision/tool drawers, or justify N/A if G-5 surfaces the tile (`L275-L276`).
- N13-E21 — Tool directives, `ClaimSet`s, and decision cards on backends lacking strict schema still cross the boundary and emit one `+output_boundary_verdict` containing resolved `enforcement_mode` (`L277-L279`).
- N13-E22 — N-2 rejects a new output type lacking a registry row (`L279-L280`).
- N13-E23 — N-2’s duplicate-concept check rejects a second bespoke validation path (`L279-L280`).

Ambiguous/needs detail-file resolution:

- N13-A01 — The source says machinery exists in “>=8 places” but enumerates nine unit references; this is not contradictory but the machine count is intentionally non-exact (`L266-L268`).
- N13-A02 — Four emission seams are claimed but only three example output classes (tool directive, ClaimSet, decision card) are named (`L275-L278`).
- N13-A03 — Registry storage format, schema language, retry budgets, invalid-rate window, strict-provider capability detection, and exact refusal payload are unspecified.
- N13-A04 — “reject-never-repair” and a registry `repair policy` coexist; the permitted distinction requires detail-file clarification (`L264-L272`).

### N-14 — self-improvement edit-loop instruments (EvoPolicyGym)

Metadata: external verdicts S2-1/S2-2/S2-3/S2-5; STRONG external grounding; arXiv:2607.02440 verified 2026-07-05; n=1 caveat retained; BUILD/ACTIVATION; M-L; Wave 9 (`L282-L284`).

- N14-E01 — Add a deterministic edit-type classifier finer than C-10’s stagnation categories (`L285-L289`).
- N14-E02 — Classify `SynthesisEdit` when normalized stripped-AST topology changes (`L289-L290`).
- N14-E03 — Classify `ParametricEdit` when normalized stripped-AST topology stays the same (`L289-L290`).
- N14-E04 — Normalize constants before topology comparison (`L289-L290`).
- N14-E05 — Normalize identifiers before topology comparison (`L289-L290`).
- N14-E06 — Join `EditKind` onto every score-bearing attempt (`L289-L291`).
- N14-E07 — Use edit kind to upgrade C-10 persona choice from blind rotation to evidence-driven selection (`L290-L291`).
- N14-E08 — Feed edit kind to I-4, I-22, and J-10 (`L291,L295`).
- N14-E09 — Add synthesis-versus-tuning task demand as an N-5/H-7/L-21/J-84 routing axis (`L291-L295`).
- N14-E10 — Use that axis to avoid assigning local-7B the ownership of structure discovery when measured synthesis ability is weak (`L292-L293`).
- N14-E11 — Add hidden two-stage SELECT-then-REPORT to H-45/H-47 (`L293-L294`).
- N14-E12 — Select candidates using hidden validation data (`L293-L294`).
- N14-E13 — Report performance using a disjoint held-out slice, never the selection slice (`L293-L294,L301-L302`).
- N14-E14 — Add improvement-event timing to I-27’s existing curve (`L294-L296`).
- N14-E15 — Keep all four instruments advisory under TD-3 (`L288-L294`).
- N14-E16 — Use ELAI-measured thresholds only under ID-5 (`L294`).
- N14-E17 — Reuse C-10/B-22b detection, I-27 curve, H-45/H-47/D-6/H-19 promotion machinery, N-5 routing, and N-1 measurement; do not reimplement them (`L295-L296`).
- N14-E18 — Use tree-sitter in `crates/knowledge` for AST structure (`L295-L296`).
- N14-E19 — Integrate `crates/orchestrator/src/edit_kind.rs` with anti-spiral, I-27, promotion, and routing seams (`L297-L298`).
- N14-E20 — Add an edit-kind chip to the anti-spiral/receipt drawer, or justify internal-advisory N/A (`L297-L298`).
- N14-E21 — Under `ELAI_SEED`, a constant-only edit followed by a restructuring edit must classify Parametric then Synthesis (`L299-L300`).
- N14-E22 — Repeated synthesis churn with no improvement shifts persona to Architect/Simplifier (`L300`).
- N14-E23 — Whitespace-only or rename-only diffs must not flip normalized topology; such a flip fails (`L300-L301`).
- N14-E24 — Reporting selection-slice performance as promotion performance fails the two-stage gate (`L301-L302`).

Evidence/caveat preserved, not treated as a local threshold:

- N14-C01 — The source reports external paper observations that leaders convert synthesis edits 41-48%, while weak models churn structure 3-10% yet score 25-38% on tuning; the n=1 caveat means these numbers are not promoted into hard ELAI thresholds (`L283-L288`).

Ambiguous/needs detail-file resolution:

- N14-A01 — Supported languages, tree-sitter fallback for parse errors, topology representation, score-bearing-attempt schema, churn window, and persona-switch threshold are unspecified.
- N14-A02 — “Hidden” storage/access controls and held-out budget integration with H-19 need detail.

### N-15 — anti-affirmation stability measurement

Metadata: external verdicts S4-2/S4-3; STRONG; arXiv:2510.01395 verified 2026-07-05; HBR framing external; BUILD/ACTIVATION; M; Wave 9 (`L304-L306`).

- N15-E01 — Limit the new mechanism to advisory prose, because I-17 L1 veto, I-33 externalized candidates, H-31 judge firewall, D-14 inverted framing, E-4 chat drift, and TD-3 already make sycophancy structurally irrelevant at the decision boundary (`L307-L309`).
- N15-E02 — Permute presentation order across N advisory samples (`L309-L311`).
- N15-E03 — Permute option order across N advisory samples (`L309-L311`).
- N15-E04 — Apply the sampling to B-41 review, J-45 cross-family, L-11 OCP-LSP, and I-41 lens advisories (`L310-L317`).
- N15-E05 — If a verdict flips under a presentation-only permutation, log it as framing-unstable (`L311-L312`).
- N15-E06 — Zero a framing-unstable advisory’s weight for the current run (`L311-L312`).
- N15-E07 — Extend I-24 scorer neutralization to framing in advisory lanes (`L312,L317`).
- N15-E08 — Add a D-23 option-order-inversion fixture class where the same reordered facts must not flip the decision (`L313-L314`).
- N15-E09 — Add a D-23 assertion-pull fixture class using a false user assertion such as “I think X” (`L313-L314`).
- N15-E10 — Measure assertion adoption rate without the D-14 probe (`L314`).
- N15-E11 — Measure assertion adoption rate with the D-14 probe (`L314`).
- N15-E12 — Keep the check advisory-only and disqualifying rather than granting it structural decision power (`L314-L316`).
- N15-E13 — Measure through N-1 and use only ELAI-measured thresholds under ID-5 (`L315-L318`).
- N15-E14 — Establish a GD-4 false-positive bound before enabling the check by default (`L315-L316`).
- N15-E15 — Reuse D-23/E-54/D-14 fixtures and the existing runner rather than reauthoring it (`L317-L318`).
- N15-E16 — Integrate `crates/orchestrator/src/framing_invariance.rs`, the D-23 corpus, and advisory emission seams (`L319-L320`).
- N15-E17 — Surface a framing-stability tile on G-5/N-8, or justify N/A if an existing sycophancy-bench tile provides it (`L319-L320`).
- N15-E18 — `elai bench sycophancy` runs the two fixture classes (`L321-L323`).
- N15-E19 — A flip emits `+framing_stability_measured(FramingUnstable)` and zeroes weight (`L321-L323`).
- N15-E20 — Assertion-pull emits adoption rates with and without the D-14 probe (`L322-L323`).
- N15-E21 — A presentation-stable advisory incorrectly logged unstable fails the false-positive arm (`L323-L324`).
- N15-E22 — Default-on framing checks without a measured false-positive bound fail GD-4 (`L323-L324`).

Ambiguous/needs detail-file resolution:

- N15-A01 — Sample count N, permutation scheme, verdict-equivalence definition, stability threshold, weight scope beyond “for the run,” and false-positive bound are unspecified.
- N15-A02 — The exact HBR source is not named in the summary.

### N-16 — governance closure: evaluator-refutation verdict-collapse + a lambda exogenous-grounding floor + adjudicator-ritual metrics

Metadata: DeepReason external steal dated 2026-07-08; verdicts S4/S6/S7; STRONG; ELAI anchors are PLAN-asserted pending W-7 re-grep; EXTENSION(N-8)+BUILD; M; Wave 9 (`L326-L328`).

- N16-E01 — Extend N-8 with three more views of decorative governance; do not build a separate framework (`L329-L332`).
- N16-E02 — Add `evaluator-id` to every SSB verdict row (`L332-L333`).
- N16-E03 — Add evaluator `version-hash` to every SSB verdict row (`L332-L333`).
- N16-E04 — Register evaluator-refutation events (`L333-L335`).
- N16-E05 — Treat a failed H-27 mutant corpus as an evaluator-refutation source (`L333-L334`).
- N16-E06 — Treat an I-24/N-15 bias detection as an evaluator-refutation source (`L333-L334`).
- N16-E07 — Treat a frozen-tool bug fix as an evaluator-refutation source (`L333-L334`).
- N16-E08 — On refutation, invoke the existing F-5 staleness cascade (`L334-L345`).
- N16-E09 — Quarantine exactly the past verdicts issued by the refuted evaluator (`L334-L335`).
- N16-E10 — Place quarantined verdicts into an advisory re-audit queue (`L334-L335`).
- N16-E11 — Never delete quarantined verdicts; preserve F-5 append-only history (`L335,L350-L354`).
- N16-E12 — Explicitly reject the Dung argument-graph adjudicator from the borrowed rubric-closure idea (`L335-L336`).
- N16-E13 — Compute lambda as the windowed fraction of accepted verdicts whose support bottoms out in an exogenous anchor rather than a model rubric (`L336-L339`).
- N16-E14 — Count program anchors as exogenous (`L336-L338`).
- N16-E15 — Count off-agent anchors as exogenous (`L336-L338`).
- N16-E16 — Count retrieval anchors as exogenous (`L336-L338`).
- N16-E17 — Count human anchors as exogenous (`L336-L338`).
- N16-E18 — Below the lambda floor, fire a retrieval/escalation brake (`L337-L339`).
- N16-E19 — Never use lambda as a PASS signal (`L338-L339`).
- N16-E20 — Interpret lambda as a detector of stalled/closed-loop dynamics, not wrong-but-stable consensus (`L338-L339`).
- N16-E21 — Measure auditor finding-class entropy to distinguish probing new surfaces from relitigating the same nodes (`L339-L340`).
- N16-E22 — Measure criticism debt as the fraction of shipped units whose declared VERIFY leg never ran (`L340-L341`).
- N16-E23 — Measure the rate at which gates are challenged or amended (`L340-L341`).
- N16-E24 — Keep all three new views advisory-only under TD-3 (`L341-L342`).
- N16-E25 — Measure all governance values through N-1 under ND-4 (`L341-L345`).
- N16-E26 — Use ELAI-measured thresholds under ID-5 (`L341-L342`).
- N16-E27 — Reuse N-8’s existing five views, F-5, H-27/I-24/N-15 detectors, the gate manifest/freeze checker, D-17 escalation, and R4 off-agent doctrine (`L343-L346`).
- N16-E28 — Add the verdict field, F-5 cascade hook, and lambda/ritual derivations in `crates/orchestrator` (`L347-L348`).
- N16-E29 — Extend the `.claude/tools` governance-coverage checker with the three views (`L347-L349`).
- N16-E30 — Add a lambda gauge, quarantine-queue count, and ritual-entropy band to the N-8/G-5/M-3 governance tile (`L348-L349`).
- N16-E31 — A refutation followed by a governance run emits `+evaluator_closure_swept` and queues the evaluator’s historical verdicts (`L350-L352`).
- N16-E32 — Lambda below its floor emits `+grounding_ratio_measured BelowFloorBraked` and activates the brake (`L351-L352`).
- N16-E33 — An unrun VERIFY leg on a shipped unit increments criticism debt (`L352`).
- N16-E34 — Silently deleting a quarantined verdict fails the append-only invariant (`L353-L354`).
- N16-E35 — Using lambda or ritual metrics as a PASS gate fails TD-3 (`L353-L354`).
- N16-E36 — Asserting a governance number without N-1 measurement fails ND-4 (`L353-L354`).

Ambiguous/needs detail-file resolution:

- N16-A01 — Window length, lambda numerator/denominator edge cases, floor, brake duration, re-audit release criteria, evaluator identity migration, and entropy formula are unspecified.
- N16-A02 — “frozen-tool bugfix” event registration and version-hash algorithm are unspecified.
- N16-A03 — “PLAN-asserted, re-grep W-7” explicitly lowers confidence in the local anchor verification even though the steal’s grounding is labeled STRONG (`L327-L328`).

## Explicit workflows and interactions

### Measurement-to-decision loop

Source: `L24-L42,L67-L83,L180-L193,L260-L302,L304-L354`.

1. J-56/E-54/L-20 and the named feeders provide evaluation material to N-1.
2. A family uses the one K-15/I-27-shaped runner via `elai bench *`.
3. The run appends a corpus row, updates the one I-51 `ModelCapabilityProfile`, and supplies outcome labels.
4. Local calibration hard-supersedes matching BOOTSTRAP defaults and records the supersession.
5. N-3 reads the outcome labels and trial ledger to deflate promotion results and apply reusable-holdout budgets.
6. N-9 uses the same outcomes for drop-one/inject-one counterfactual scoring.
7. N-13 writes per-type/backend invalid rates into the same spine.
8. N-14 supplies edit kind and improvement timing; N-15 supplies framing/adoption measurements; N-16 supplies governance measurements.

### Plan-authoring anti-fork loop

Source: `L46-L65,L155-L159,L246-L280,L356-L361`.

1. Plan authors encode map edges in ND-3 data form.
2. N-2 joins map edges, plan units, and `elai.map` symbols.
3. It checks dangling consumption, orphan production, duplicate concepts, freshness, and extended type completeness.
4. It emits an unlock DAG/fan-in rank and mechanically derives summary/status material.
5. It also enforces required render fields (N-7), output-boundary registry rows (N-13), and the no-second-implementation rule used by N-12/N-13.
6. Any seeded invalid case must fail closed before the plan ships.

### Claim, receipt, provenance, and output path

Explicit parts are sourced from `L85-L103,L125-L159,L260-L280`; the cross-unit ordering below is partly **Inferred** where noted.

1. **Explicit:** N-4 represents typed claims, beliefs, critical atoms, receipts, and grounding tags as one object.
2. **Explicit:** A repeated claim checks J-96 receipt cache identity; unchanged claims can reuse a receipt, changed claims are invalidated by F-5.
3. **Explicit:** N-6 maps trust semantics while the object travels ingest → memory → citation → oracle TTL → skill autonomy → cross-repo.
4. **Inferred interaction:** A machine-generated ClaimSet likely crosses N-13’s `OutputBoundary` before N-4/J-83 uses it, because ClaimSet is a named boundary type, but the summary does not state the exact temporal order.
5. **Explicit:** Every resulting gate/verdict declares an N-7 human render destination.

### Routing and resource-allocation loop

Source: `L105-L123,L195-L209,L282-L302,L336-L354`.

1. N-10 joins ambiguity and novelty and sends the shared score to research depth, reasoning effort, node sizing, and clarification.
2. N-14 adds synthesis-versus-tuning task demand as another policy/intensity axis.
3. Existing routers and intensity deciders fire independently.
4. N-5 resolves conflict by frozen precedence and emits one explained `RouteDecision`.
5. N-16 can activate a retrieval/escalation brake when exogenous grounding falls below its floor; it cannot create a pass.

### Governance lifecycle

Source: `L144-L178,L326-L354`.

1. A gate/verdict class must declare both its SSB representation and human render.
2. A new gate must have a mutant corpus and measured kill rate before entering the frozen manifest.
3. Governance coverage reports liveness, enforcer coverage, debt rate, traceability, and mutation kill performance.
4. Each verdict also records evaluator identity/version.
5. An evaluator refutation invokes F-5 to quarantine exactly its verdicts into advisory re-audit without deletion.
6. The extended governance run also reports exogenous-grounding ratio, finding-class entropy, unrun verification debt, and gate-challenge rate.
7. These closure metrics are advisory/braking signals, never pass signals.

### Journal derivation and retirement

Source: `L211-L224`.

1. H-49 CAS/undo-tree is declared the one log representation.
2. L-16’s pure fold core accepts surface-specific fold functions as data/plugins.
3. Canvas altitude, plan status, billing balance, fleet rollup, and coverage derive through it.
4. One Kani proof covers the shared fold machinery.
5. Pinned-seed conformance requires byte-identical rederivation.
6. ND-2 retirement ends `recover_stubs` and removes ad-hoc derivation paths.

## Global constraints and invariants

- **Canonicality:** one subsystem/registry/core per concern; adapters parameterize or delegate but do not carry duplicate core logic (`L12-L18,L34-L36,L242-L248,L356-L361`).
- **Consume, do not recreate (GD-1):** N repeatedly extends shipped/planned artifacts rather than replacing their engines.
- **Fail closed (R1):** authoring checks, missing trial counts, missing boundary rows, policy arbitration omissions, stale receipt hits, taint loss, and unsupported output enforcement all have negative arms.
- **Data over prose:** map edges use ND-3 data, precedence is a frozen data table, OutputBoundary is a registry, and fold functions are data/plugins.
- **Measurement authority (ND-4/ID-5):** one `ModelCapabilityProfile` and N-1 spine own calibration; N-14/N-15/N-16 thresholds must be locally measured.
- **Bootstrap provenance:** cold-start values are explicitly BOOTSTRAP and cannot remain authoritative after local data exists.
- **Append-only evidence:** F-5 quarantine never deletes historical verdicts.
- **Structural separation (TD-3):** the model does not choose enforcement; advisory instruments cannot become pass gates.
- **Security preservation:** taint must survive all mapped boundaries; F-6’s type barrier remains; grammar pins, reject-never-repair, frozen repair tables, and admission scanners remain binding.
- **Human visibility:** every verdict family must declare a render, with N-7 frontend pairing mandatory.
- **Thin adapters (ND-5):** counterfactual and other unified call-site adapters contain no business logic.
- **Retirement (ND-2):** a unification is incomplete while an ad-hoc predecessor path remains live.
- **Determinism:** N-11 uses a pinned seed and byte-identical outputs; N-14 has an `ELAI_SEED` classification test.
- **False-positive control (GD-4):** N-15 cannot default on without a measured FP bound.

## Interfaces, state, configuration, observability, and operations

### CLI and authoring commands

- `elai bench <any-family>` / `elai bench *` invokes N-1 families.
- `python .claude/tools/corpus_composition_check.py --map ELAI-FEATURE-MAP.md` invokes N-2.
- `elai bench sycophancy` invokes N-15 fixtures.
- No public network API, protocol, authentication flow, or remote service API is specified in this summary.

### Canonical state/data structures

- I-51 `ModelCapabilityProfile` / `models.toml`: only calibration sink.
- Corpus row + calibration curve + outcome-label fold: measurement output.
- `Claim/Belief` object: unified statement/evidence carrier.
- J-96 case graph: receipt cache keyed with C-30 discipline.
- Composite `RouteDecision`: fired routers, winner, precedence rank.
- Semantic trust lattice mapping table: maps six intentionally distinct trust scales.
- Admission scanner registry: shared ingress scanner family.
- Gate/verdict `render_field`: required authoring contract data.
- Governance substrate: five original views plus N-16’s three extension views.
- Counterfactual target Kind: role/verifier/ranker/rule/memory/case.
- Joined `AmbiguityScore` + `NoveltyScore` rows.
- H-49 CAS/undo-tree log plus per-surface fold functions.
- `FailureClass`, `AbstainDecision`, `ProjectClass`, shared drift primitives, FSM conformance models, and LSP daemon pool.
- `OutputBoundary` registry rows with schema/parser/repair/retry/fail-closed/invalid-rate/enforcement fields.
- `EditKind` with `SynthesisEdit` and `ParametricEdit`.
- SSB verdict evaluator identity/version and advisory re-audit queue.

### Configuration and policy

- BOOTSTRAP calibration pack with provenance and hard local supersession.
- Thresholdout budget on each reusable evaluation corpus.
- Frozen routing precedence: safety > cost > abstention > escalation > modality > serving-tier.
- Per-output-type retry and repair/fail-closed policy plus backend enforcement capability.
- ELAI-measured thresholds only for edit-loop, framing, and governance instruments.
- GD-4 measured FP bound gates N-15 default-on status.
- Exact config syntax/default values are deferred to detail plans.

### Named observability artifacts

| Unit | Artifact / observable |
|---|---|
| N-1 | corpus row, calibration curve, `+calibration_bootstrap_superseded` |
| N-2 | unlock DAG, fan-in ranking, derived summaries/status, `+composition_edge_checked`, process exit status |
| N-3 | `+promotion_deflated(DeflationBlocked)` and held lifecycle state |
| N-4 | J-96 receipt node, `+receipt_memo(Hit)`, checker skip |
| N-5 | composite `RouteDecision`, `+route_arbitrated` |
| N-6 | carried trust label and optional provenance badge; no named SSB row is given |
| N-7 | human render at PR/chat/canvas/brief/rollup target |
| N-8 | `+governance_coverage_measured` with five Kinds, governance tile |
| N-9 | `+counterfactual_scored` with target Kind |
| N-10 | joined ambiguity/novelty rows and observable budget delta |
| N-11 | byte-identical fold output and retirement-switch effect; no named SSB row is given |
| N-12 | one canonical row/type per sub-slice; no universal row name is given |
| N-13 | `+output_boundary_verdict`, resolved enforcement mode, per-type/backend invalid rate |
| N-14 | `EditKind`, improvement-event timing, persona shift; no named SSB row is given |
| N-15 | `+framing_stability_measured(FramingUnstable)`, zeroed weight, adoption rates |
| N-16 | `+evaluator_closure_swept`, `+grounding_ratio_measured BelowFloorBraked`, criticism debt and ritual metrics |

### Error, fallback, and recovery behavior

- Fresh-install measurement fallback uses BOOTSTRAP defaults until hard-superseded locally.
- Missing promotion N is fail-closed; below-threshold N holds promotion.
- Conflicting routers fall back to the frozen precedence plane, not call order.
- Changed claim identity invalidates receipt reuse through F-5.
- Trust-label reset or taint loss at a boundary is a hard failure.
- Output enforcement degrades only through the fixed strongest-available order and ultimately parse/validate/retry/fail-closed; D-16 provides refusal.
- Low exogenous-grounding ratio triggers retrieval/escalation braking, never passing.
- Refuted-evaluator verdicts are quarantined for advisory re-audit, never removed.
- No backup/restore, retention duration, privacy-redaction, or disaster-recovery policy is explicit beyond these behaviors.

### UI surfaces

- N-1 `MeasurementPanel` or justified reuse of G-5.
- N-3 deflation column in promotion drawers or justified N/A.
- N-4 memo-hit indicator in belief/claim drawers or justified N/A.
- N-5 canvas/RRA route-explanation row.
- N-6 provenance badge on citation/memory views or justified N/A.
- N-7 mandatory PR/chat/canvas/brief/rollup render substrate.
- N-8 governance tile on G-5/M-3.
- N-9 existing attribution drawers or justified N/A.
- N-10 normally internal; optionally show chosen depth on canvas.
- N-11 verify existing canvas fold consumption.
- N-12 normally N/A; optional `FailureClass` drawer.
- N-13 boundary-verdict badge or G-5 tile reuse.
- N-14 edit-kind chip or justified internal N/A.
- N-15 G-5/N-8 framing-stability tile or existing sycophancy tile reuse.
- N-16 lambda gauge, quarantine count, and ritual-entropy band on governance tile.

## Dependencies, sequencing, status, and deferred work

### Wave/dependency matrix

| Unit | Wave | Explicit dependency | Nature / stated baseline | Summary-level status |
|---|---:|---|---|---|
| N-1 | 1 | none stated | ACTIVATION, keystone/blocking | Planned; activates existing pieces and adds bootstrap/fold integration |
| N-2 | 2 | N-1 | BUILD | Planned |
| N-3 | 3 | N-1 | ACTIVATION | Planned generalization of finance-shipped/planned machinery |
| N-4 | 3 | none stated | BUILD-on-shipped | Planned extension/unification |
| N-5 | 4 | N-1 | BUILD | Planned |
| N-6 | 4 | none stated | BUILD | Planned |
| N-7 | 5 | none stated; ties N-2 | BUILD | Planned |
| N-8 | 5 | none stated | BUILD | Planned |
| N-9 | 6 | N-1 | BUILD | Planned |
| N-10 | 6 | none stated | ACTIVATION | Planned |
| N-11 | 7 | none stated | BUILD-on-shipped | Planned extension + retirement |
| N-12 | 8 | N-2 operationally enforces anti-refork, though not in metadata | DELETE | Planned dedupe/retirement bundle |
| N-13 | 9 | consumes N-1 and N-2 | BUILD/UNIFY | Planned external-steal addition |
| N-14 | 9 | consumes N-1 and N-5 | BUILD/ACTIVATION | Planned advisory instruments |
| N-15 | 9 | consumes N-1 and N-8 | BUILD/ACTIVATION | Planned advisory measurement |
| N-16 | 9 | extends N-8; consumes N-1 | EXTENSION(N-8)+BUILD | Planned DeepReason addition |

“Planned” here means the N summary describes required work; it is not a claim that dependent B..M artifacts or current repository code are unimplemented. Only an implementation audit can establish runtime status.

### Reuse/dependency inventory by unit

- N-1 consumes G-6, J-34, K-15, J-107, L-10, K-18, I-38, I-27, L-20, J-110, J-56, E-54, J-93-F, W-5/I-51, B-42, J-84/J-85, L-4/L-7, K-24/M-5, and optionally G-5.
- N-2 consumes/extends M-1, ND-1/ND-3, J-89 L8, J-93-G, I-39, `elai_map_freshness.py`, `wiring_contract_check.py`, paper-to-plan phase 5, R1, and the feature map.
- N-3 consumes H-17/H-18/H-19, H-47, I-30/I-31, ID-6, N-1, J-56/L-20/D-6, 3F, J-57, J-97, B-23/H-45, I-4/I-25, and L-11.
- N-4 consumes L-1/L-2..L-6/L-8, J-83/J-96/J-21, K-12/K-18, GRC tags, C-30, and F-5.
- N-5 arbitrates D-10, J-84, J-55, H-7, E-20, J-48, J-54, L-6, J-32, J-73, plus B-41/L-21/C-46/K-5/B-28; it consumes J-92, K-4, R8, W-7, and R10; E-39 is named in a conflict.
- N-6 maps G-1, C-48, F-6, C-30, K-23, 4D, J-93-C, and H-53; it families F-1, J-102, J-109, K-23/M-2.
- N-7 consumes K-11, G-5, K-7, M-3, SSB verdict rows, J-83, L-7, N-1, WIRING_CONTRACT, and N-2.
- N-8 consumes B-17, K-22, J-46, J-70, H-27, gate-manifest/R8, G-5, M-3, and K-1 dossier framing.
- N-9 adapts C-45, I-28, J-105, J-109, E-6, E-10, H-40, J-75 and consumes N-1/ND-5.
- N-10 wires B-38, B-40, C-29, J-32, D-15, 4F/B-21 into 4D, J-84, J-39, J-79.
- N-11 extends L-16 and consumes J-31/J-60/J-98/K-23, I-39, K-24, M-3, K-22, J-68, H-49, ND-2, and GD-2.
- N-12 consumes all units explicitly named in its seven sub-slices and depends on N-2 for ongoing anti-fork enforcement.
- N-13 consumes C-26, E-35, I-34, J-53, J-51, H-13, B-24, L-1, I-33, B-37/I-51, N-1/N-2, D-16, I-46, GD-1, and TD-3.
- N-14 consumes C-10/B-22b, I-4/I-22/J-10, I-27, H-45/H-47/D-6/H-19, N-5/H-7/L-21/J-84, N-1, tree-sitter, TD-3, and ID-5.
- N-15 consumes TD-3, I-17, I-33/I-24, H-31, D-14/D-23, E-4/E-54, B-41, J-45, L-11, I-41, N-1/N-8, ID-5, and GD-4.
- N-16 extends N-8 and consumes F-5, H-27, I-24, N-15, gate-manifest, `gate_freeze_check.py`, R8, N-1, D-17, R4, ND-1/ND-4, ID-5, and TD-3.

### Explicitly rejected, already-covered, note-only, or future/deferred items

- Rejected from build: PAW compiled-LoRA tier.
- Rejected as evidence/build basis: EvoPolicyGym n=1 statistics and missing memorization defense; n=1 caveat retained.
- Rejected as new work: gaps P3-P5 because already planned deeper.
- Rejected as a build unit: Ford checklist.
- Already recorded: prefix-KV S1-1 via J-95/H-50.
- Folded into an existing N unit: zip-slip S1-2 into N-6.
- Note/dossier only: ADR evidence-table S1-3; packaging S4-1/S3-4/S1-6; EvoPolicyGym bench S2-4.
- Rejected from N-16: Dung argument-graph adjudicator.
- Deferred to detail/original plans: all thresholds, schemas, exact mappings, migration mechanics, formulae, and unresolved ambiguities listed per unit.

## Inferred interactions (not promoted to requirements)

- N-I01 — N-1 is the likely foundation for every “ELAI-measured” N threshold even when metadata does not list `depends N-1`; this is explicit for N-13..N-16 consumption but not a formal dependency for every earlier unit.
- N-I02 — N-2 likely becomes the enforcement mechanism for N-7, N-12, and N-13 authoring invariants; those ties are explicit, but whether it enforces every other N canonicality rule is only implied by the global anti-fork doctrine.
- N-I03 — N-4’s unified Claim/Belief object, N-6’s provenance carrier, N-13’s ClaimSet boundary, and N-7’s render substrate likely form a single end-to-end object lifecycle; individual pairwise roles are explicit, exact execution order is not.
- N-I04 — N-8 plus N-16 yields eight governance views (five + three); the summary calls N-16 an extension over the same question but never names a formal eight-view schema.
- N-I05 — N-11’s shared journal folds likely supply N-8/N-16 governance tiles and other N observability, but only the explicitly named five surface folds are requirements.
- N-I06 — N-12’s learned-artifact admission sub-slice likely uses N-6’s unified admission registry because both name J-109; the summary does not explicitly wire N-12(d) to N-6.

## Open ambiguities and missing specification candidates

Cross-cutting unresolved points that deeper agents should answer:

1. Header count: 15 sessions vs 16 unit headings.
2. N-12 count: eight independent collapses vs seven labeled sub-slices.
3. N-13 count: four emission seams vs three named examples.
4. Exact ND-1..ND-5 rules, sequencing, and retirement/adaptor requirements in `N-0_INDEX.md`.
5. Complete W-finding evidence and whether all W references remain current.
6. Exact schemas for map edges, Claim/Belief, RouteDecision, trust mapping, governance Kinds, fold plugins, OutputBoundary, EditKind, and evaluator identity.
7. Every threshold, budget, window, tie-breaker, and false-positive bound.
8. Lifecycle/migration of old duplicated types, stores, runners, caches, journal paths, validators, and daemons.
9. UI target mapping and fallback behavior when a declared human render cannot be delivered.
10. Privacy/retention/access policy for claim receipts, eval corpora, model profiles, evaluator histories, and re-audit queues; the summary is silent.
11. Concrete tests beyond each observable-effect positive/negative arm and the named Kani/FSM/normalization checks.
12. Implementation status versus plan status; this source is a specification summary, not a completion ledger.

## Deeper-source manifest

### N originals located but intentionally unread in this pass

All paths exist on disk:

1. `docs/thefinalglance/N-plan/N-0_INDEX.md` — doctrine, ND-1..ND-5 deltas, sequencing.
2. `docs/thefinalglance/N-plan/N-1_measurement_spine.md`
3. `docs/thefinalglance/N-plan/N-2_corpus_composition_validator.md`
4. `docs/thefinalglance/N-plan/N-3_anti_selection_bias_promotion.md`
5. `docs/thefinalglance/N-plan/N-4_claim_object_receipt_memo.md`
6. `docs/thefinalglance/N-plan/N-5_policy_intensity_arbitration.md`
7. `docs/thefinalglance/N-plan/N-6_provenance_lattice_admission.md`
8. `docs/thefinalglance/N-plan/N-7_human_render_verdict_field.md`
9. `docs/thefinalglance/N-plan/N-8_governance_coverage_substrate.md`
10. `docs/thefinalglance/N-plan/N-9_counterfactual_ablation_engine.md`
11. `docs/thefinalglance/N-plan/N-10_clarity_novelty_budget_wires.md`
12. `docs/thefinalglance/N-plan/N-11_journal_fold_substrate.md`
13. `docs/thefinalglance/N-plan/N-12_consistency_dedupe_bundle.md`
14. `docs/thefinalglance/N-plan/N-13_output_boundary_and_structured_output_capability_matrix.md`
15. `docs/thefinalglance/N-plan/N-14_self_improvement_edit_loop_instruments.md`
16. `docs/thefinalglance/N-plan/N-15_anti_affirmation_stability_measurement.md`
17. `docs/thefinalglance/N-plan/N-16_governance_closure_grounding_floor_ritual.md`

### Referenced local evidence and tooling located

- `workflow/reasoning/elai-weakpoints-interlace-2026-07-04.md` — WHY/evidence per W finding.
- `workflow/reasoning/four-source-steal-2026-07-05.md` — phase-4 steal analysis for N-13..N-15 dispositions.
- `workflow/reasoning/steal-deepreason-2026-07-08.md` — N-16 DeepReason steal evidence.
- `workflow/handoff/deepreason-steal-and-collab.md` — related DeepReason handoff candidate; not explicitly cited by the N summary, so relevance requires confirmation.
- `workflow/papers/programasweights-summary.md` — local programasweights summary candidate; the source cites the repository itself, so this may be a proxy rather than the original.
- `docs/thefinalglance/ELAI-FEATURE-MAP.md` — N-2 input and external-steal disposition ledger target.
- `docs/thefinalglance/J-plan/J-SUMMARY_INDEX.md` — drift example cited by N-2.
- `.claude/tools/elai_map_freshness.py` — N-2 pattern.
- `.claude/tools/wiring_contract_check.py` — N-2 sibling/pattern.
- `.claude/tools/gate_freeze_check.py` — N-16 composition input.

### Referenced source candidates not located by exact/local filename scan

- `elai-any-model-research-gaps.md` — cited at `L253`; no exact local file path was found.
- “the Ford transcript” — cited at `L253`; no uniquely identifiable local transcript filename was found. Unrelated source-tree files containing “ford” were excluded.
- The programasweights repository — cited at `L252`; no local directory named for it was found, though `workflow/papers/programasweights-summary.md` exists.
- arXiv:2607.02440 / EvoPolicyGym full paper — cited at `L252,L283`; no local filename containing the identifier/title was found.
- arXiv:2510.01395 full paper — cited at `L305,L314`; no local filename containing the identifier was found.
- The exact HBR framing source — cited only generically at `L305,L313`; title/path is absent.
- DeepReason specification §11/§17 original — the local steal analysis exists, but the summary does not identify a standalone original spec path.

### Highest-priority deeper reads

1. `N-0_INDEX.md` for ND-1..ND-5 and the authoritative sequencing/retirement contract.
2. Each N-1..N-16 detail file, one agent per unit, to close the per-unit ambiguities above and enumerate test/config/schema minutiae.
3. `elai-weakpoints-interlace-2026-07-04.md` to validate every W finding and discover rejected alternatives.
4. `four-source-steal-2026-07-05.md` and `steal-deepreason-2026-07-08.md` for source-side caveats and exact external dispositions.
5. `ELAI-FEATURE-MAP.md` plus the cited B..M units to confirm canonical-owner direction and eliminate accidental double-counting.

