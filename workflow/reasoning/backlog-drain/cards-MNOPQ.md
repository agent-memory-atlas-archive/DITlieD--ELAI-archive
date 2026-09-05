# Backlog-drain state + supersession map — letters M, N, O, P, Q

Analyst scope: M, N, O, P, Q (the NEWEST letters in the corpus).
Grounding basis: `.claude/tools/elai.map` (fresh, mtime 2026-07-14 08:31; confirmed current because
K-shipped symbols land in it — RoutingEconomicsMeasured=7 refs / safedelta=6 / insecure_rate_measured=3),
the per-letter 0_INDEX + per-unit plan files, and the on-disk benchmark/session artifacts.

TOP-LINE GROUNDED FINDING (read before the cards):
ALL of M/N/O/P/Q is UNEXECUTED code. Every signature symbol these letters would introduce is ABSENT from
a fresh map (EntitlementEvent 0, OrgPolicyEvent 0, FleetRegistry 0, AppManifest 0, GoalEnrolled 0,
WakeQuorum 0, AdvisorConsulted 0, ModelSwapRestart 0, OutputBoundary 0, composition_edge_checked 0,
fleet_health_tick 0, operator_action 0, CredentialVault 0, IngressAdapter 0, NoveltyCanary 0,
cobrowse_session 0, CeremonyProfile 0, privacy_grant 0, vault_write 0, seat_entitlement 0, standing_goal 0,
quorum_wake 0, advisor_lane 0, brain_vault 0, provenance_lattice 0, output_boundary 0). Every 0_INDEX
SESSION LOG is empty ("empty at generation"). M is explicitly GATED behind GATE M-0 (K's A-1..A-4 + K-3 +
K-24 L1..L4). N/O/P/Q are authored-now / execute-after-the-proof-spine (N-1 + PS-1..PS-6) by their own
sequencing rules. So the drain-system risk these letters pose is NOT "an old plan reverting newer shipped
work" — it is the INVERSE: these newest letters are almost entirely PROSPECTIVE, and the older letters
(B..L) are the ones actively shipping. The two exceptions with real executed state are harness-only
benchmark work:
  - Q-10 (B-CREATIVITY-0): RAN → VALID REFUTED 2026-07-14. Downstream permanently forbidden.
  - P-25 (B-EXO-0): IN-PROGRESS harness prep (V4 object migration), no scored run, no shipped code.

State legend used below:
  dark        = planned, zero production code shipped (signature symbols absent from fresh map; empty session log)
  partial     = some harness/prep artifacts on disk, but the unit's production deliverable is not shipped
  refuted     = the unit RAN and closed dead (verdict on record); code deliverable never authored, downstream forbidden
  wired       = production symbol DEF + non-test production caller present in map (NONE in M-Q)
  uncertain   = could not ground cleanly

CRITICAL NUANCE ON SUPERSESSION IN THIS CLUSTER:
N-plan is a UNIFICATION letter BY DESIGN (ND-1 "unify-not-fork"): its explicit job is to REPLACE >=2
fragmented implementations of one concern with ONE canonical object + thin adapters, and to ship an ND-2
RETIREMENT MAP that puts each superseded older-letter path behind an `ELAI_*_LEGACY` flag for one release
then a USER-CONFIRMED removal. So nearly every N unit carries supersession edges back into B..L. These are
PROSPECTIVE (N is unshipped) but they are the load-bearing "respect / do-not-re-fragment" edges for the
drain: an implementer draining B..L must NOT ship an (n+1)th parallel implementation of a concern an N unit
is slated to unify, and must NOT later revert an N unification back to the scattered originals. O/P/Q are
COMPOSE-not-restate letters (they EXTEND shipped substrate, rarely replace). The single genuine
"replaces-the-older-approach" edge inside this cluster is P-25 → Q-10 (a refuted mechanism's successor).

================================================================================
SUPERSESSION_EDGES  (M/N/O/P/Q  →  older B..L, with evidence)
================================================================================
Format: NEWER_ID  SUPERSEDES/UNIFIES/EXTENDS/REPLACES  OLDER_ID[...]  — evidence (file / quoted phrase).
"UNIFIES" = ND-1 replace-N-fragments-with-one + ND-2 retire-old-behind-LEGACY (prospective, unshipped).
"EXTENDS" = builds-on-shipped, old path survives unchanged. "REPLACES" = a genuine redesign/kill edge.

--- N-plan (the unification letter — every edge below is an ND-1/ND-2 supersession) ---
N-1  UNIFIES  G-6, J-34, K-15, J-107, L-10, K-18, I-38, I-27, L-20, J-110  — N-0:130-136 "Unify the ten
     scattered benches ... into one MEASUREMENT SPINE"; J-56 corpus + L-20 supply + J-110 mutators demoted
     to FEEDERS ("as FEEDERS not siblings"). The K-15/I-27 runner shape becomes THE one runner family.
N-1  hard-SUPERSEDES (calibration)  D-16 tau, J-34 knees, K-15 knees, OQ thresholds  — N-SUMMARY_INDEX:32
     "D-16 tau / J-34 knees / K-15 knees / OQ thresholds, hard-superseded by local measurement, W-23";
     +calibration_bootstrap_superseded row (Kind Prefilled/LocallySuperseded/DriftInvalidated).
N-1  generalizes  J-93-F  — from rules-only outcome-labels to all six promotion loops (N-1 detail line 8).
N-2  UNIFIES/composes  J-93-G, I-39  — N-0:139-145 "derive the summaries/status mechanically (compose
     J-93-G + I-39)"; lifts wiring-enforcement R6 to the plan corpus. Builds the ND-1/ND-3 checker itself.
N-3  generalizes (scope-supersede)  H-17, H-18, H-19  — N-0:146 "Lift H-17 trial ledger + H-18
     deflated-significance + H-19 reusable-holdout budget from finance-only to DOMAIN-GENERIC".
N-4  UNIFIES (one Claim/Belief object)  L-1 ClaimSet, J-83 BeliefState, K-12 critical_atoms  — N-0:153-159
     "ONE canonical Claim/Belief type: L-1 ClaimSet == J-83 BeliefState entries == K-12 critical_atoms";
     receipts memoized as J-96 case nodes. "L-8 already threads L-1 over J-83 (half-planned) — N-4 finishes it."
N-5  UNIFIES (~10 routers)  D-10, J-84, J-55, H-7, E-20, J-48, J-54, L-6, J-32, J-73  — N-0:160-166 "A
     declared PRECEDENCE spec (DATA not code) across the ~10 deterministic routers".
N-5  UNIFIES (6 intensity deciders)  B-41, L-21, J-84, C-46, K-5, B-28  — same block "ONE intensity
     resolver absorbing the SIX verification-intensity deciders ... B-28 ModeSelector as J-55's edit-format leg".
N-6  UNIFIES/maps (6 trust scales)  G-1 taint, C-48 memory T0-T3, F-6 skill S1-S4, C-30 source T1-T4,
     K-23 repo trust, 4D EvidenceLevel  — N-0:167-173. NOTE it is a declared SEMANTIC MAPPING, not a shared
     enum (F-6 deliberately kept SkillTrustTier distinct; landmine N-0:347).
N-6  UNIFIES (admission family)  F-1 VibeGate, J-102 config-ingress, J-109 learned-artifact, K-23, M-2  —
     "share ONE scanner registry".
N-7  generalizes  K-11  — N-0:174-178 "Make 'human render' a REQUIRED field ... instead of K-11 being a
     one-off for G-5."
N-8  UNIFIES  B-17, K-22, J-46, J-70, H-27  — N-0:181-186 "ONE instrument with five views"; H-27
     kill-your-own-mutant lifecycle generalized from finance to EVERY frozen gate.
N-9  UNIFIES (4 counterfactual engines)  C-45, I-28, J-105, J-109  AND (4 ablation harnesses)  E-6, E-10,
     H-40, J-75  — N-0:187-191 "ONE drop-one/inject-one engine ... replacing the four counterfactual
     engines ... AND the four ablation harnesses".
N-10 types+wires (activation)  B-38, B-40, C-29, J-32 (AmbiguityScore) + D-15, 4F, B-21 (NoveltyScore)  —
     consumers 4D, J-84, J-39, J-79 (N-0:192-196). ADDS consumers, no producer.
N-11 EXTENDS + retires  L-16 fold core  — N-0:197-202 / N-11 detail: extend L-16's pure fold beyond context
     to canvas altitudes (J-31/J-60/J-98/K-23), I-39 plan status, K-24 billing, M-3 rollup, K-22 coverage;
     ND-2 retirement map for recover_stubs (EOL) + 3H-trim + E-25-tier; declare H-49 CAS/undo-tree THE log rep.
N-12 UNIFIES/DELETES (8 disjoint collapses)  — one FailureClass enum (W-8 + E-6/E-29 MAST dedupe);
     one AbstainDecision (W-9); one ProjectClass registry (B-40's, consumed by presets/cosmos/archetype/L-14);
     route C-47/C-49 experience stores through J-109; one drift-primitive lib (D-4/G-10/H-30/F-5/B-22b);
     FSM 3-model conformance (3I DST/J-68 TLA/code); one LSP daemon pool (E-52/J-18/J-52). (N-0:205-212)
N-13 UNIFIES/retires (>=8 constrained-output impls)  C-26, E-35, I-34, J-53, J-51, H-13, B-24, L-1, I-33  —
     N-0:220-224 → ONE OutputBoundary registry behind `ELAI_OUTPUT_BOUNDARY_LEGACY` (N-13:74 "old scattered
     paths survive one release, then ND-2 retires them"). Inherits I-34 grammar-pinning as security rider.
N-14 REPLACES (behind legacy flag)  anti-spiral C-10 blind persona rotation  — N-14:84 "default ON behind
     `ELAI_EDIT_KIND_LEGACY` (blind persona rotation survives one release as fallback)". Feeds N-1/N-3/N-5.
N-15 EXTENDS  I-24 (bias-detection pattern), D-23 fixtures  — permute-presentation framing-invariance; the
     I-24 pattern extended to framing (N-0:230-234). ELAI_FRAMING_INVARIANCE_LEGACY.
N-16 EXTENDS  N-8  (self-edge, same-letter) — N-0:236-247 "Three views on the SAME N-8 substrate (ND-1
     EXTENSION, not a new framework)". Adds evaluator-id+version-hash FIELD to existing verdict rows;
     triggers an F-5 staleness cascade (composes F-5).

--- M-plan (commercial wrapper — SEQUENCES existing units, GD-1 "never re-authors") ---
M-1  builds-on / gates  K-24 (billing rail), J-49 (multi-seat admission), B-43 (installer)  — M-0:43-46
     "MA-1 K-24 BCC billing ... the metering rail M-1's entitlement gates". Section-A pointers, not supersede.
M-3  builds-on / rolls-up  J-59 (per-seat memory), K-3/K-11 (PR evidence), K-6/K-7/K-13 (compliance export)  —
     M-0:44-48. Aggregates, does not replace.
M-4  EXTENDS  B-43, B-45 (installer)  — M-0:46 "M-4 builds the updater they defer". Signed release channel.
M-5  EXTENDS/wraps  E-23-class packaging, G-8 signing, G-9 redaction, K-1 claims-lint  — verifier OSS extraction.
M-6  EXTENDS  M-5a  — the evidence-format open standard (EEF v1) after M-5a extraction (same-letter edge).

--- O-plan (research machine — COMPOSE/EXTEND, no supersede) ---
O-1  EXTENDS (generalizes manifest)  B-42 Cosmos, C-16  — O-1:28 "replace the designer-brief manifest
     struct with the typed topic schema (closed kind enum), keeping the existing loader signature so
     CosmosStage is unchanged." This is a SWAP of the topic source, NOT a CosmosStage rewrite (O-0:108).
     Threads G-1/F-3/G-2 trust spine through fetched bytes.
O-2  EXTENDS  3F (rules-engine), C-9 (task_runner), K-14 (SSB temporal-monitor)  — typed TaskSpec scheduler.
O-3  EXTENDS  E-5 (memory), D-5 (privacy), teikoku brain_store  — MOUNTS a vault, never converts memory_facts.
O-4  EXTENDS  H-12, J-31 (FSM dispatch), J-60 (canvas)  — canvas WORKFLOW mode; NO canvas runtime.

--- P-plan (personal-software fleet — COMPOSE/EXTEND, plus the ONE genuine REPLACE edge) ---
P-1  EXTENDS  O-2, G-10, F-9, F-5, L-14, N-7, M-4  — lifts the spine grain changeset→app (fleet registry).
P-2  EXTENDS  P-1, O-4, N-6, G-1/G-2  — typed ports over the canvas world bus.
P-3  EXTENDS  P-1, P-2, M-5/M-6, L-12, G-9  — persona/mechanism split + friend-export bundle.
P-4  EXTENDS  O-3, B-35, F-5, N-6, D-5, P-6  — living external brains (Obsidian/Notion), GATED behind RECON.
P-5  EXTENDS  K-5, K-4, N-7, P-1  — graduated-ceremony toy/personal profile; observe-only STILL journals (never skips).
P-6  EXTENDS (dimension on)  N-6 lattice  — P-0:333 "P-2/P-6 ride N-6's ONE lattice; a second provenance/
     privacy scale is an authoring FAIL." Out-flow privacy mirror of taint.
P-7  EXTENDS  N-3, H-17/H-18, M-2, E-5  — per-user OperatingProfile (asymmetric, own-run-FP-gated).
P-8  EXTENDS  O-3, K-23, I-2, M-2/M-3, 3H  — two-seat SSB-stream merge kernel (schema-invasive: seat_id/agent_id cols).
P-9  EXTENDS (applies M-6 own-the-format to capability)  M-6, B-37, L-12/N-6 admission  — P-0:20-23 "M-6
     own-the-FORMAT applied to CAPABILITY". Connector-pack admission format.
P-10 EXTENDS  G-1, O-2 (trigger enum stays fixed), M  — IngressAdapter family + CredentialVault.
P-11 EXTENDS  O-4, O-2, H-13, L-14  — workflow-from-goal → proposed TaskSpec graph, human-confirmed.
P-12 NEW (honest gap)  — desktop/GUI operator lane; cua vendored substrate + OmniParser Tier-3 fallback. Not a supersede.
P-13 EXTENDS  E-14, J-1, J-8, J-77, G-1  — perception provenance + Set-of-Marks id-indirection. Counts NEVER from a VLM (J-1 law unchanged).
P-14 EXTENDS  E-14, O-2, G-2  — explore-then-crystallize record→replay lane. FEEDS PS-5.
P-15 EXTENDS  J-96, K-18, K-19  — verified case-graph hardening (gate the SAVE behind J-96 mint-on-proof; do NOT copy agent-s ungated save).
P-16 EXTENDS  P-9, P-10, L-12, N-6, P-12  — gated install catalog.
P-17 EXTENDS  J-28, I-50, Q-4, B-37, K-2  — provider failover + honest routing economics. FEEDS PS-4.
P-18 EXTENDS + CORRECTS  O-2, O-4, P-11, 3A  — durable TaskSpec exec + anti-TOCTOU approval drift-binding.
     Explicitly REJECTS the openclaw lobster resume-token model as UNVERIFIED-external (P-0:352, C1); builds a
     REAL subprocess-checkpoint model instead. (External-approach rejection, not an ELAI-unit supersede.)
P-19 NEW-area  — config/ops lifecycle + `elai doctor` + verified backups (ELAI's thinnest area). Does the
     OPPOSITE of openclaw's full-env-copy (env allowlist); adopts openclaw's cwd path-jail verbatim.
P-20 EXTENDS  H-50, H-51, H user-memory, L, H-MCP-defer  — context/memory lifecycle hardening.
P-21 EXTENDS  Q-6, G-9, M, wiring-R10  — gate-credibility hardening (verify-the-verifier canaries). FEEDS PS-6.
P-22 EXTENDS  3C, G-1, J-control-plane, P-10, M, 4C  — control-plane/channel hardening.
P-23 EXTENDS  H-13/H-15, E-14, P-13, P-14, P-6, P-10  — journaled co-browse lane; native Tauri multiwebview EXCLUDED (unstable).
P-24 EXTENDS  H-14 (the deferred RN/Expo slice), E-14, P-12/13/14/16, P-23  — mobile as a typed DIMENSION, never a parallel subsystem.
P-25 REPLACES (successor to a refuted mechanism)  Q-10 B-CREATIVITY-0  — P-0:367-386 "the user-directed
     SUCCESSOR falsifier to the Q-10 B-CREATIVITY-0 REFUTED closeout"; exogenous-injection replaces the dead
     relational-packet coupling. INTERNAL supersede: P-25 V4 supersedes its own V3 object (P-25:4 "V4 repairs
     that binding and supersedes V3 as the object to audit"). Q-10's pad/meter bytes remain PHYSICALLY
     UNCHANGED (P-25 imports none of them) — a clean-successor, not an in-place rewrite.

--- Q-plan (economics/taste/distribution — COMPOSE/EXTEND, plus one self-retiring gate) ---
Q-1  EXTENDS  O-2 (TaskSpec), F-5 (quarantine), I-25, 3F  — standing goals; the ONE genuinely-new primitive. FEEDS PS-6.
Q-2  EXTENDS  O-1, O-2, 4G, I-25, J-pool, D-13  — quorum wake-gate; SELF-RETIRES when measured wake-precision
     crosses threshold (Q-2:8 "the gate retires ITSELF"). Self-supersede-by-measurement, not an older-unit edge.
Q-3  EXTENDS  H-7, I-7, I-11, I-25, 4M, H-9, K-2  — advisor lane + K-2 advisor arm. FEEDS PS-4.
Q-4  EXTENDS/guards  4G, 4M, I-11, F-6, I-25  — swap-restart attribution (code-confirmed gap in autonomous_gate.rs).
Q-5  EXTENDS  G-2, J-28 (adds output_token_hash FIELD to the usage row), blake3  — local per-turn token-hash journal.
Q-6  EXTENDS  H-selfval, 3I-DST, gate-manifest  — evidence-corruption corpus + per-gate kill-rate. Frozen auditor-owned.
Q-7  EXTENDS  D-6/E-54, local-lane, L-gate  — weak-lane code-style bench. FEEDS PS-4.
Q-8  EXTENDS  M-verifier, K-4  — size/dep ceiling on the M standalone verifier (PENDING a real M build).
Q-9  EXTENDS  4M vendored CLIs, cross_lang_diff, ts-rs  — distribution hygiene (fork markers + one surface registry).
Q-10 (see card — REFUTED; its downstream is dead; P-25 is its successor).
Q-11 EXTENDS  novelty.rs, conjecture_triage, frontier_bench, J-28, RoutingEconomicsMeasured(schema 274)  —
     novelty-instrument canary + fixed-budget bench. FEEDS PS-4/PS-3/PS-6. Ships FIRST in AIDE2 wave.
Q-12 EXTENDS  web_research_orch, research_tree, Q-11 bench, B-40  — minimal-prompt matrix + lineage-bandit
     behind default-OFF `ELAI_RESEARCH_BANDIT_ENABLE`. Gated on Q-11 bench green.
Q-13 EXTENDS  E-48, H-52, cosmos_worker, briefing.rs  — receipt-bound briefing lines (fabricated line unrepresentable).

CROSS-LETTER SUPERSESSION EDGE COUNT INTO B..L:
Distinct N→(B..L) unification/scope-supersede edges: ~55 older-unit targets across N-1..N-15 (the bulk of the
cluster's supersession weight — all PROSPECTIVE / retirement-map-gated / unshipped).
Genuine REPLACE edges: 1 within-cluster (P-25 → Q-10, a refuted-mechanism successor) + 2 external-approach
rejections (P-18 rejects openclaw lobster-resume; P-19 inverts openclaw env-copy) that touch no ELAI unit.
M/O/P(non-25)/Q edges into B..L are EXTEND/compose (build-on-shipped), not reversions.

================================================================================
CARDS — grouped by letter
================================================================================

---------- M-plan (6 units; commercial shipping; GATE M-0 blocks all execution) ----------

id: M-1  seat entitlement + purchase rail  [KEYSTONE]
does: Offline-verifiable per-seat entitlement (mint/renew/revoke) + purchase rail; gates on K-24 metering.
  Failure/expiry degrades to FREE tier (MD-1 never-brick, the one deliberate fail-OPEN inversion of R1).
state: dark — EntitlementEvent absent from fresh map (0 refs); empty M-0 session log; execution GATED behind
  GATE M-0 (K A-1..A-4 + K-3 done + K-24 L1..L4 live).
supersedes[]: none  superseded_by[]: none
residual: entire unit + +EntitlementEvent SSB row (Pattern-2, baseline+1).
key_files[]: (unbuilt) — sequences K-24 billing rail (MA-1). respect[]: K-24, J-49, MD-1 fail-open floor.

id: M-2  org policy pack + distribution + drift detection
does: File/git-distributable org policy pack every seat obeys and cannot weaken; drift detection. NO SSO/SAML/SCIM.
state: dark — OrgPolicyEvent 0 refs; empty session log; GATE M-0.
supersedes[]: none  superseded_by[]: (prospective) N-6 admission family will absorb the M-2 scanner surface.
residual: entire unit + +OrgPolicyEvent row.
key_files[]: (unbuilt). respect[]: MD-3 minimal-cloud (no org-console), the K-0:531 non-goal + ENTERPRISE RIDER.

id: M-3  fleet evidence rollup (`elai org report`)
does: Aggregates per-seat compliance artifacts (K-6/K-7/K-13) into an org rollup of NUMBERS + HASHES, never
  raw transcripts (G-9 precondition). Answers the org-side buyer.
state: dark — empty session log; GATE M-0; depends M-1, MA-3 (J-59), MA-6.
supersedes[]: none  superseded_by[]: (prospective) N-11 journal-fold substrate names M-3 rollup as a fold surface.
residual: entire unit + optional +OrgReportEvent (prefer-reuse a K-6/G-8 row first).
key_files[]: (unbuilt). respect[]: MD-4 seat-privacy (aggregate hashes only), G-9 redaction, J-59 isolation.

id: M-4  signed release channel + auto-updater + support bundle
does: Signed release channel + updater the B-43/B-45 installers defer + support bundle.
state: dark — empty session log; GATE M-0; depends MA-4.
supersedes[]: none  superseded_by[]: none
residual: entire unit + optional +ReleaseChannelEvent (prefer-reuse).
key_files[]: (unbuilt). respect[]: B-43/B-45 installer, G-8 signing.

id: M-5  commercial packaging + verifier OSS + trust continuity
does: EULA/DPA/pricing under K-1 claims-lint (b), continuity statement (c), verifier-OSS crate extraction (a),
  tiers.toml (d). The GTM open-source-verifier decision executed.
state: dark — empty session log; GATE M-0. Zero-SSB (E-23-class + one crate extraction).
supersedes[]: none  superseded_by[]: Q-8 will lock a size/dep ceiling on M-5a's extracted verifier.
residual: entire unit. key_files[]: (unbuilt). respect[]: K-1 claims-lint on all money/pricing copy (MD-5); GTM agencies-not-enterprise.

id: M-6  evidence format as a versioned open standard (EEF v1)
does: The evidence emission format frozen as a versioned open standard, after M-5a extraction + stable emit paths.
state: dark — empty session log; GATE M-0; depends M-5a.
supersedes[]: none (extends M-5a)  superseded_by[]: none; P-9 APPLIES M-6's own-the-format doctrine to capability.
residual: entire unit. key_files[]: (unbuilt). respect[]: M-5a extraction, the stable emit paths it standardizes.

---------- N-plan (16 units; the UNIFICATION letter; execute after N-1 spine + PS-1..PS-6) ----------
(Every N unit's supersedes[] is a PROSPECTIVE ND-1/ND-2 unification — see SUPERSESSION_EDGES for full lists.
 State for ALL 16 is dark: no signature symbol in the fresh map, empty N-0 session log.)

id: N-1  measurement spine  [KEYSTONE][BLOCKING]
does: ONE measurement spine unifying the ten scattered benches + corpus feeders + one runner family + one
  outcome-label fold (generalized J-93-F) + a BOOTSTRAP calibration pack. Highest fan-in in the whole corpus
  (~30 promotions terminate here). Nothing downstream is enforceable until it lands.
state: dark — measurement_spine 0 refs; unify targets exist as fragments (ModelCapabilityProfile 257 refs
  [I-51], insecure_rate_measured 3 refs) but no spine over them; empty session log.
supersedes[]: UNIFIES G-6/J-34/K-15/J-107/L-10/K-18/I-38/I-27/L-20/J-110; hard-supersedes D-16 tau + J-34/K-15
  knees + OQ thresholds on the calibration leg; generalizes J-93-F.  superseded_by[]: none
residual: whole spine + BOOTSTRAP pack + +calibration_bootstrap_superseded row (baseline+1).
key_files[]: (unbuilt over crates/orchestrator bench surfaces). respect[]: the existing bench action_types
  (UNIFIES, does not replace them); ModelCapabilityProfile as the ONE calibration sink (ND-4).

id: N-2  corpus-grain composition validator + M-1 one-registry rule
does: Parse the feature-map edges as DATA; FAIL on dangling edge / orphan producer / duplicate-concept; emit
  the global unlock-DAG + fan-in ranking; freshness check on SUMMARY-vs-0_INDEX-vs-detail. wiring R6 lifted to
  the plan corpus. Builds the ND-1/ND-3 checker itself.
state: dark — composition_edge_checked 0 refs; empty session log; depends N-1.
supersedes[]: composes J-93-G + I-39.  superseded_by[]: none
residual: whole validator + +composition_edge_checked row.
key_files[]: (unbuilt). respect[]: the emitted fan-in ranking ORDERS N-3..N-12 — do not reorder past it.

id: N-3  generalized anti-selection-bias promotion primitives
does: Lift H-17 trial ledger + H-18 deflated-significance + H-19 reusable-holdout from finance-only to
  domain-generic promotion primitives via the ID-6 ledger hook; every promotion gate reads a persisted
  per-family trial count + deflation.
state: dark — empty session log; depends N-1 labels.
supersedes[]: scope-generalizes H-17/H-18/H-19.  superseded_by[]: none; P-7 consumes N-3.
residual: whole unit + +promotion_deflated row. key_files[]: (unbuilt). respect[]: reads the N-1 spine, never a private store (ND-4).

id: N-4  one Claim/Belief object + receipt memo
does: ONE canonical Claim/Belief type unifying L-1 ClaimSet / J-83 BeliefState / K-12 critical_atoms + GRC
  tags; receipts memoized as J-96 case nodes, consulted by the J-83 receipt-gate AND the L-2 CGG.
state: dark — empty session log; L-8 already threads L-1 over J-83 (half-planned), N-4 finishes it.
supersedes[]: UNIFIES L-1 + J-83 + K-12.  superseded_by[]: none; N-16 adds evaluator-id fields to verdict rows.
residual: schema alignment (per N-0:355 "a schema alignment, not a redesign") + receipt memo + +receipt_memo row.
key_files[]: (unbuilt). respect[]: L-8's existing L-1-over-J-83 threading (extend, don't re-author, GD-1).

id: N-5  policy / intensity arbitration plane
does: A declared PRECEDENCE spec (DATA) across ~10 routers + ONE intensity resolver absorbing 6 verification-
  intensity deciders; ONE composite RouteDecision journal row. K-4 gate-tax then measures ONE policy.
state: dark — empty session log; depends N-1; leans on I-1 supervisor plane (carry as DEFERRED if I-1 unshipped).
supersedes[]: UNIFIES D-10/J-84/J-55/H-7/E-20/J-48/J-54/L-6/J-32/J-73 (routers) + B-41/L-21/J-84/C-46/K-5/B-28 (intensity).
superseded_by[]: none  residual: whole plane + +route_arbitrated row. key_files[]: (unbuilt). respect[]: I-1 (do not re-implement, GD-1).

id: N-6  provenance / trust lattice + admission-gate family
does: ONE declared SEMANTIC MAPPING between six trust scales (NOT one shared enum) + ONE admission scanner
  registry (F-1/J-102/J-109/K-23/M-2).
state: dark — provenance_lattice 0 refs; empty session log.
supersedes[]: maps G-1/C-48/F-6/C-30/K-23/4D; unifies the admission family.  superseded_by[]: none; P-2/P-6 ride this ONE lattice.
residual: whole lattice + optional +admission_scanned row. key_files[]: (unbuilt). respect[]: F-6 CHOSE type
  separation for SkillTrustTier — MAP the six scales, do NOT merge the enum (landmine N-0:347).

id: N-7  human render as a REQUIRED verdict field
does: Make "human render" a required field of every gate/verdict family (PR comment / chat ACK / canvas badge
  / brief row / M-3 rollup). ONE render substrate, many verdict producers.
state: dark — empty session log. supersedes[]: generalizes K-11 from a G-5 one-off.  superseded_by[]: none; M-3/P-1/P-5 consume it.
residual: whole substrate + +verdict_rendered row. key_files[]: (unbuilt). respect[]: K-11's existing G-5 render shape.

id: N-8  governance-coverage substrate
does: ONE instrument, five views (B-17 gate-liveness / K-22 doctrine-enforcer coverage / J-46 debt-rate /
  J-70 gate-manifest traceability / H-27 mutant-kill generalized to every frozen gate).
state: dark — empty session log. supersedes[]: UNIFIES B-17/K-22/J-46/J-70/H-27.  superseded_by[]: EXTENDED by N-16.
residual: whole substrate + +governance_coverage_measured row. key_files[]: (unbuilt). respect[]: the K-1 dossier ties to this number.

id: N-9  one counterfactual / ablation engine
does: ONE drop-one/inject-one engine over the SSB + five thin adapters, replacing 4 counterfactual engines +
  4 ablation harnesses. Natural consumer of N-1 outcome labels.
state: dark — empty session log; depends N-1. supersedes[]: UNIFIES C-45/I-28/J-105/J-109 + E-6/E-10/H-40/J-75.
superseded_by[]: none  residual: whole engine + +counterfactual_scored row. key_files[]: (unbuilt). respect[]: adapter-thinness (ND-5) — logic in the ONE core.

id: N-10  clarity / novelty → budget wires  (ACTIVATION)
does: Type AmbiguityScore + NoveltyScore ONCE; declare consumers (research depth 4D, effort tier J-84, node
  sizing J-39, grill offer J-79). Four passive detectors → an active resource-allocation loop.
state: dark — empty session log. supersedes[]: types B-38/B-40/C-29/J-32 + D-15/4F/B-21 (adds CONSUMERS, no new producer).
superseded_by[]: none  residual: the typing + consumer wires, baseline+0 (reuse shipped rows). key_files[]: (unbuilt). respect[]: reuse the shipped ambiguity/novelty rows.

id: N-11  journal-fold substrate
does: Extend L-16's pure fold(log,budget,policy)→view beyond context to five re-derive surfaces (canvas
  altitudes, I-39 status, K-24 billing, M-3 rollup, K-22 coverage); ONE J-68 Kani proof covers all five.
state: dark — empty session log. supersedes[]: EXTENDS L-16; ND-2 retires recover_stubs (EOL) + 3H-trim + E-25-tier;
  declares H-49 CAS/undo-tree THE log rep.  superseded_by[]: none
residual: fold core adopted by 2-3 surfaces (Tier-1) + Kani proof + the recover_stubs/H-49 retirement map (USER-CONFIRMED).
key_files[]: (unbuilt, extends L-16 fold core). respect[]: do NOT re-author the fold (GD-1) — N-11 EXTENDS L-16.

id: N-12  consistency + dedupe bundle  (DELETE)
does: Eight disjoint same-concern collapses (FailureClass enum, AbstainDecision spine, ProjectClass registry,
  experience-store routing, drift-primitive lib, FSM 3-model conformance, LSP daemon pool).
state: dark — empty session log; lowest risk, sequenced last (WAVE 8).
supersedes[]: UNIFIES/DELETES across W-8/W-9/W-12/W-16/W-21/W-25/W-26/W-28 targets (E-6/E-29, B-40, C-47/C-49,
  D-4/G-10/H-30/F-5/B-22b, 3I/J-68, E-52/J-18/J-52).  superseded_by[]: none
residual: the eight collapses (baseline+0, removes duplicate emitters). key_files[]: (unbuilt). respect[]: the surviving canonical type keeps its row.

id: N-13  OutputBoundary + structured-output capability matrix
does: Collapse >=8 scattered constrained-output impls to ONE OutputBoundary registry + a capability matrix on
  the ONE catalog (B-37/I-51) + a strongest-available-enforcement resolver. Inherits I-34 grammar-pinning.
state: dark — OutputBoundary 0 refs; empty session log.
supersedes[]: UNIFIES C-26/E-35/I-34/J-53/J-51/H-13/B-24/L-1/I-33 behind ELAI_OUTPUT_BOUNDARY_LEGACY.  superseded_by[]: none
residual: whole registry + +output_boundary_verdict row. key_files[]: (unbuilt). respect[]: I-34 grammar-pinning as a binding security rider.

id: N-14  self-improvement edit-loop instruments (EvoPolicyGym)
does: Four ADVISORY instruments (EditKind classifier, synthesis-vs-tuning routing axis, two-stage SELECT-then-
  REPORT, improvement-event-timing) feeding N-1/N-3/N-5. Upgrades anti-spiral persona choice from blind rotation.
state: dark — empty session log; ADVISORY (TD-3), low risk.
supersedes[]: REPLACES anti-spiral C-10 blind persona rotation (behind ELAI_EDIT_KIND_LEGACY).  superseded_by[]: none
residual: four instruments + +edit_kind_classified row (prefer-reuse). key_files[]: (unbuilt). respect[]: TD-3 advisory-never-the-gate; ELAI-measured thresholds only.

id: N-15  anti-affirmation stability measurement
does: Advisory framing-invariance (permute presentation order across B-41/J-45/L-11/I-41 samples, zero weight
  on a flip) + two D-23 fixture classes. The I-24 pattern extended to framing. GD-4 FP-bound before default-ON.
state: dark — empty session log; ADVISORY.
supersedes[]: EXTENDS I-24 bias-detection pattern + D-23 fixtures.  superseded_by[]: none
residual: measurement + +framing_stability_measured row (prefer-reuse). key_files[]: (unbuilt). respect[]: advisory-only, GD-4 FP-bound.

id: N-16  governance closure (evaluator-refutation + lambda floor + ritual)  EXTENSION(N-8)
does: Three views on the N-8 substrate: evaluator-id+version-hash on every verdict row + F-5 staleness cascade
  on evaluator refutation (quarantine, never delete); a lambda exogenous-grounding floor with a BRAKE; adjudicator-
  ritual metrics. Rejects the Dung argument-graph, takes only the closure consequence.
state: dark — empty session log; ADVISORY; depends N-8 + N-1.
supersedes[]: EXTENDS N-8 (ND-1 extension, not a new framework); composes F-5.  superseded_by[]: none
residual: three views + +evaluator_closure_swept + +grounding_ratio_measured rows (S7 prefer-reuse). key_files[]: (unbuilt). respect[]: never delete verdicts (advisory re-audit queue only).

---------- O-plan (4 units; personal research machine; PROOF_SPINE none; execute after WAVE-1) ----------
(All 4 dark: no signature symbol; empty O-0 session log. Substrate they compose IS shipped — cosmos_worker
 791 refs, research_tree TaskSpec struct present, ModelCapabilityProfile 257 refs.)

id: O-1  research daemon (generalize Cosmos)  [KEYSTONE]
does: Generalize the Cosmos manifest to a user-editable typed topic set (closed kind enum) + thread G-1/F-3/G-2
  trust spine through every fetched byte; ships steal-scan + monitor kinds. SWAP of the topic source, not a CosmosStage rewrite.
state: dark — the O-1 topic-schema/classify surface is unbuilt (no daemon symbols in map); the B-42 Cosmos
  substrate it swaps into exists (cosmos_worker 791 refs).
supersedes[]: EXTENDS B-42/C-16 (manifest generalization).  superseded_by[]: none; Q-1/Q-2/Q-11 compose on it.
residual: typed-topic schema + enum-classify + trust threading + O-1 classify SSB row.
key_files[]: crates (knowledge) cosmos_worker (extend, keep loader signature). respect[]: CosmosStage stays
  unchanged (O-0:108); the G-1/F-3/G-2 threading is the ELAI-over-teikoku delta.

id: O-2  scheduled TaskSpec substrate
does: Typed TaskSpec in SQLite, cron/ssb/manual triggers, journaled fires, dashboard intervention-card inbox.
  Reuses task_runner interval + SSB temporal-monitor.
state: dark — the ONLY TaskSpec DEF in the map is `research_tree.rs:1960 pub struct TaskSpec` (the research-tree
  task spec, a DIFFERENT concept); the O-2 scheduler substrate is unbuilt; empty session log.
supersedes[]: EXTENDS 3F/C-9/K-14.  superseded_by[]: none; O-4/P-11/P-18/Q-1/Q-2 all depend on this store.
residual: whole scheduler + +task_spec_fired row. key_files[]: (unbuilt, orchestrator/rules). respect[]: the
  wall-clock trigger stays SUPERVISOR-PLANE, outside the K-14 replay set (O-2 ABORT).

id: O-3  brain vault contract + mounting
does: MOUNT a portable markdown brain vault (not convert) with a NEW namespace-claims contract so ELAI +
  teikoku share one vault; memory_facts stays the machine grain; guarded upsert.
state: dark — vault_write/BrainVault 0 refs; empty session log.
supersedes[]: EXTENDS E-5/D-5/teikoku brain_store.  superseded_by[]: none; P-4/P-8 depend on O-3.
residual: vault contract + namespace-claims hard-refusal + +vault_write/+claim_violation rows.
key_files[]: (unbuilt). respect[]: the vault is MOUNTED, never converted into memory_facts (O-3 ABORT + stub_ban).

id: O-4  canvas WORKFLOW mode
does: Editor + monitor over TaskSpecs on the canvas, enum-constrained palette, wire-dark made visual, dispatch
  through the FSM (J-31). NO canvas runtime.
state: dark — empty session log; depends O-2 store queryable (else ships against a read-only fixture per its FORK).
supersedes[]: EXTENDS H-12/J-31/J-60.  superseded_by[]: none; P-2/P-11 build on the canvas mode.
residual: whole mode (reuses O-2's fire row). key_files[]: dashboard-v2 + orchestrator FSM dispatch. respect[]:
  NO runtime — dispatch is the FSM (O-4 ABORT + stub_ban).

---------- P-plan (25 units; personal-software fleet; execute after the proof spine, EXCEPT P-25) ----------
(P-1..P-24 all dark: no signature symbol; empty P-0 session log. P-25 is partial harness prep — see its card.)

id: P-1  app-fleet layer (health-tick + repair)  [KEYSTONE]
does: AppManifest + FleetRegistry + O-2 health-tick TaskSpecs + G-10 per-app drift canaries + gated auto-repair.
  Lifts the spine grain changeset→app. Every other P unit binds to its manifest.
state: dark — AppManifest/FleetRegistry/fleet_health_tick 0 refs; empty session log.
supersedes[]: EXTENDS O-2/G-10/F-9/F-5/L-14/N-7/M-4.  superseded_by[]: none; P-2/P-3 depend on P-1.
residual: whole layer + +fleet_health_tick row. key_files[]: (unbuilt). respect[]: gated auto-repair is a
  CHANGESET (same wiring/claim gates), never a silent patch (P-1 ABORT).

id: P-2  inter-tool ports + world-grain bus
does: PortSchema on the manifest + provenance/taint-carrying tool→tool edges on the O-4 canvas + wire-dark-made-visual.
state: dark — empty session log; depends P-1, O-4.
supersedes[]: EXTENDS P-1/O-4/N-6/G-1/G-2.  superseded_by[]: none
residual: PortSchema registry + +tool_port_edge row. key_files[]: (unbuilt). respect[]: rides N-6's ONE lattice
  (a second provenance scale is an authoring FAIL, A-1/N-2).

id: P-3  persona/mechanism split + friend-export
does: Factor every app into mechanism-vs-persona-data + `elai app export/import` bundle re-verified on the recipient seat.
state: dark — empty session log; depends P-1, P-2.
supersedes[]: EXTENDS P-1/P-2/M-5/M-6/L-12/G-9.  superseded_by[]: none
residual: export bundle + +app_export/+app_import rows. key_files[]: (unbuilt). respect[]: re-verification on the recipient seat is the point.

id: P-4  living external brains
does: LivingStoreContract per store class (Obsidian/Notion/SQLite) + change feeds + F-5/N-6-tagged ingest +
  O-3-claim-gated writes. MOUNTS live, not static.
state: dark — empty session log; GATED behind a BLOCKING Obsidian-API RECON (the digest flags it UNGROUNDED).
supersedes[]: EXTENDS O-3/B-35/F-5/N-6/D-5/P-6.  superseded_by[]: none
residual: whole unit + +vault_ingest/+claim_violation rows. key_files[]: (unbuilt). respect[]: author NO code
  against a training-asserted store API — the RECON is BLOCKING (P-4 ABORT).

id: P-5  operator surface + graduated ceremony
does: A toy/personal CeremonyProfile (gates observe-only, one-line receipts, nothing skipped) + non-coder render.
  Scales the trust spine DOWN.
state: dark — CeremonyProfile 0 refs; empty session log; WAVE 1 (cheap).
supersedes[]: EXTENDS K-5/K-4/N-7/P-1.  superseded_by[]: none
residual: ceremony profile + +ceremony_profile_set row. key_files[]: (unbuilt). respect[]: observe-only STILL
  RUNS + JOURNALS the gate — reduces ceremony, never skips evidence (P-5 ABORT, W-2/R1).

id: P-6  life-data privacy lattice
does: Which TOOL may read which NAMESPACE — a domain DIMENSION on N-6's one lattice + per-tool read grants, default-deny.
state: dark — privacy_grant 0 refs; empty session log; depends N-6, O-3.
supersedes[]: EXTENDS N-6 (dimension on the ONE lattice) / G-1/G-9/E-20/O-3/H-49.  superseded_by[]: none
residual: the privacy dimension + +privacy_grant/+privacy_read_refusal rows. key_files[]: (unbuilt). respect[]: rides N-6's ONE lattice (A-1).

id: P-7  per-user operating profile
does: Adapt autonomy/strictness/verbosity per domain; loosen ONLY on measured own-run FP evidence (asymmetric,
  so adaptation cannot become sycophancy). Three archetypes = three defaults on ONE engine.
state: dark — OperatingProfile 0 refs; empty session log; depends N-3 landed.
supersedes[]: EXTENDS N-3/H-17/H-18/M-2/E-5.  superseded_by[]: none
residual: whole profile engine + +operating_profile_change row. key_files[]: (unbuilt). respect[]: user affect
  NEVER loosens a gate (A-4 / P-7 ABORT); loosen only on measured own-run FP.

id: P-8  small-team collaboration grain
does: The 2-10 human shared world; the hard kernel is a defined lossless deterministic two-seat SSB-stream merge
  + per-seat/agent provenance. Schema-invasive (seat_id/agent_id columns on the SSB row).
state: dark — empty session log; depends O-3, K-23; design-spike-first.
supersedes[]: EXTENDS O-3/K-23/I-2/M-2/M-3/3H.  superseded_by[]: none
residual: merge kernel + +seat_journal_merge + seat_id/agent_id columns. key_files[]: (unbuilt). respect[]:
  lossless + deterministic + local-first; NO last-writer-wins, NO org SaaS control plane (P-8 ABORT).

id: P-9  connector-pack admission format  [2nd-wave KEYSTONE]
does: A signed versioned declarative connector/stage-pack (ONE generic http_call kind parameterized by DATA) +
  a code lane (B-37), mounted through the EXISTING admission family. Converts the closed vocabulary into a moat.
state: dark — empty session log; PROOF_SPINE none (owner-acked); executes after the spine.
supersedes[]: EXTENDS/applies M-6 own-the-format to capability; rides L-12/skillspector/J-102/N-6 admission.  superseded_by[]: none; P-10/P-16 depend on P-9.
residual: whole format + +install_catalog_admit-class rows. key_files[]: (unbuilt). respect[]: the L-12/N-6 admission family (fail-closed).

id: P-10  ingress adapters + credential vault
does: An IngressAdapter family normalizing four external shapes to a taint-tagged (G-1) journaled SsbEvent (so
  the O-2 trigger enum does NOT grow) + a first-class CredentialVault (secrets never journaled/exported).
state: dark — IngressAdapter/CredentialVault 0 refs; empty session log.
supersedes[]: EXTENDS G-1/O-2/M.  superseded_by[]: none; P-9's http_call needs P-10's auth plane.
residual: adapter family + vault. key_files[]: (unbuilt). respect[]: the O-2 trigger enum stays fixed; secrets never journaled.

id: P-11  elai workflow from-goal
does: intent → a CLOSED-enum-constrained proposed TaskSpec graph on the O-4 canvas, human-confirmed before it
  runs, with an L-14 RRA receipt. The n8n-user-converting demo.
state: dark — empty session log; depends O-4/O-2/H-13.
supersedes[]: EXTENDS O-4/O-2/H-13/L-14.  superseded_by[]: none; P-18 hardens P-11 runs.
residual: whole unit. key_files[]: (unbuilt). respect[]: closed-enum-constrained + human-confirmed-before-run.

id: P-12  desktop/GUI operator lane  (NEW — the honest gap)
does: cua sandbox vendored substrate + OmniParser Tier-3 pixel fallback + frameId/display fail-closed binding +
  per-action bounded screen-recording as off-agent evidence + crosshair/zoom crops, all under ELAI gates.
state: dark — operator_action 0 refs; empty session log; PROOF_SPINE none.
supersedes[]: none (fills a genuine gap; comparators are VENDORED substrate + operator lanes, never rivals).  superseded_by[]: none; P-16/P-24 use P-12 as a backend.
residual: whole lane + +operator_action/+operator_frame_bind rows. key_files[]: (unbuilt). respect[]: cua's own
  in-loop safety is a stubbed TODO — ELAI brings the fail-closed gate; operator-action stage is a taint source (G-1), never load-bearing infra.

id: P-13  perception provenance + Set-of-Marks id-indirection  (EXTENSION)
does: VLM picks an integer id off a source-tagged element list, never coords (OmniParser's 70.5→93.8% grounding
  lift); each element carries a source tag (axtree_exact|ocr_text|vlm_caption|pixel_yolo); Tier-3 pixel fires
  only on empty/incomplete AXTree.
state: dark — empty session log; EXTENDS J-1 PerceptionScaffold with a source/id field (net 0..+1, prefer-reuse).
supersedes[]: EXTENDS E-14/J-1/J-8/J-77/G-1.  superseded_by[]: none; P-14/P-23/P-24 depend on P-13.
residual: id-indirection + source tags. key_files[]: (unbuilt, extends J-1). respect[]: counts NEVER from a VLM
  (J-1 law unchanged); axtree_exact is the only authoritative source; a per-crop caption never stands in for element identity.

id: P-14  explore-then-crystallize record→replay lane  (EXTENSION)
does: An agentic exploration run emits a deterministic Playwright/pyautogui script AS an O-2 TaskSpec stage with
  a G-2 RRA receipt binding the script to the minting run; replay = the cheap execution mode.
state: dark — crystallized_stage_minted 0 refs; empty session log; depends P-13. FEEDS PS-5.
supersedes[]: EXTENDS E-14/O-2/G-2.  superseded_by[]: none; P-15/P-24 build on the crystallize lane.
residual: whole lane + +crystallized_stage_minted row. key_files[]: (unbuilt). respect[]: cite cua replay_env
  as the record→re-score prior art, NOT openclaw jsonl-replay (input-through-a-mock, C4); a script with no minting-run receipt is refused.

id: P-15  verified case-graph hardening  (EXTENSION)
does: Port agent-s S2 KnowledgeBase retrieve/distill/fuse, but GATE the save behind J-96's mint-on-proof filter;
  add a bBoN behavior-narrative judge OVER JOURNAL ROWS as an evidence-ranked tie-breaker.
state: dark — empty session log; EXTENDS J-96 case rows with a reuse-hit field (net 0..+1). FEEDS PS-5.
supersedes[]: EXTENDS J-96/K-18/K-19.  superseded_by[]: none
residual: the gated save + bBoN judge. key_files[]: (unbuilt, extends J-96). respect[]: do NOT copy agent-s's
  ungated KnowledgeBase commit; gate the save behind J-96 mint-on-proof; the bBoN judge runs over JOURNAL ROWS, never screenshots.

id: P-16  gated install catalog  (EXTENSION)
does: Named {tool,source,mirror,alias,license_tag} rows + an installPolicy behind the EXISTING admission family;
  "download blender on demand" = a P-9 headless code-pack wrapping the app's CLI, P-12 GUI stage is last resort.
state: dark — install_catalog 0 refs; empty session log; depends P-9, P-10.
supersedes[]: EXTENDS P-9/P-10/L-12/N-6/P-12.  superseded_by[]: none
residual: catalog + installPolicy + +install_catalog_admit row. key_files[]: (unbuilt). respect[]: the sanctioned
  form is a P-9 HEADLESS code-pack; P-12 GUI driving is the LAST resort, never the mechanism; OmniParser icon weights are AGPL.

id: P-17  provider failover + honest routing economics  (EXTENSION)
does: Failure-class cooldown taxonomy (rate/billing/auth lanes) + session-stickiness-for-cache-warmth +
  persist-before-retry + strict-explicit-selection + never-merge-billed-truth-with-estimate as typed fields.
state: dark — empty session log; EXTENDS the J-28 UsageTurnEvent with typed billed/estimate fields. FEEDS PS-4.
supersedes[]: EXTENDS J-28/I-50/Q-4/B-37/K-2.  superseded_by[]: none
residual: failover semantics + +provider_failover_event row. key_files[]: (unbuilt). respect[]: never merge billed truth with estimate (S4).

id: P-18  durable TaskSpec exec + approval drift-binding  (EXTENSION)
does: Revision-counter optimistic concurrency + sticky-cancel + retry-ladders-per-failure-class + watchdog for
  the run store + approval drift-binding anti-TOCTOU (every approval binds a hashed run-plan + operand fingerprints; drift=deny).
state: dark — empty session log; depends P-11.
supersedes[]: EXTENDS O-2/O-4/P-11/3A; REJECTS openclaw's lobster resume-token as UNVERIFIED-external (C1),
  builds a REAL subprocess-checkpoint model instead.  superseded_by[]: none
residual: durable-run store + +taskspec_run_revision/+approval_drift_reject rows. key_files[]: (unbuilt).
respect[]: the VERIFIED mechanism is the revision-counter (S6); carry the resume-token only as a RECON idea, never "openclaw shipped this".

id: P-19  config/ops lifecycle + `elai doctor`  (NEW-area, ELAI's thinnest)
does: Fail-closed-default-object config + last-known-good/rejected-write/shrink-guard/atomic writes + doctor
  detect/repair/lint with stable exit codes + VACUUM-INTO verified SSB backups + restart continuation queue + compaction split-rule.
state: dark — empty session log; PROOF_SPINE none (ops).
supersedes[]: NEW area (EXTENDS the M + 3H surfaces).  superseded_by[]: none
residual: whole lifecycle + +config_write_rejected/+doctor_finding rows. key_files[]: (unbuilt). respect[]: do
  the OPPOSITE of openclaw's full-env-copy (env allowlist, C2); adopt their cwd path-jail verbatim (correct).

id: P-20  context/memory lifecycle hardening  (EXTENSION)
does: Pre-compaction memory-flush as an ordered journaled step + versioned on-demand skill/rules index +
  Dreaming promotion-gate prior art + per-provider cache-control mapping table.
state: dark — memory_flush 0 refs; empty session log.
supersedes[]: EXTENDS H-50/H-51/H user-memory/L familiarity/H-MCP-defer.  superseded_by[]: none
residual: flush step + index + +memory_flush_event row. key_files[]: (unbuilt). respect[]: "no compaction destroys context never offered to durable memory" (S3).

id: P-21  evidence / gate-credibility hardening  (EXTENSION)
does: Verify-the-verifier seeded negative-control canaries that hard-block the whole gate when a canary goes
  undetected + missing-artifact=FAIL manifest + skipBackfill cross-lane dependency + STRUCTURAL redaction + posture labels.
state: dark — empty session log; EXTENDS Q-6 corpus verdict with posture-label + verifier-canary fields. FEEDS PS-6.
supersedes[]: EXTENDS Q-6/G-9/M/wiring-R10.  superseded_by[]: none
residual: canaries + manifest + structural redaction. key_files[]: (unbuilt, extends Q-6). respect[]: copy
  openclaw's unknown=FAIL posture ONLY (C3), NOT the default that lets a classified product-bug pass; keep audit ledger (metadata) SEPARATE from the debug bundle (content) by type (V2/V3).

id: P-22  control-plane / channel hardening  (EXTENSION)
does: Child-output-as-fenced-evidence + depth-scoped tool-strip defaults + 9-step deterministic channel-binding
  precedence + pairing codes + dmScope + challenge-signed pairing + unknown-event fail-closed gating + idempotency keys + hook tier ordering.
state: dark — empty session log.
supersedes[]: EXTENDS 3C/G-1/J-control-plane/P-10/M/4C.  superseded_by[]: none
residual: whole hardening + +subagent_return_fenced/+idempotency_replayed rows. key_files[]: (unbuilt). respect[]:
  do NOT port openclaw's advisory regex scanner (signature-match with no teeth is what the gate doctrine rejects, V5); a subagent return is a taint source (S10).

id: P-23  journaled co-browse lane  (EXTENSION)
does: A canvas browse node over the E-14 engine — Tier-1 read-only extraction + Tier-2 CDP-screencast co-browse
  where the pane is DUMB GLASS and ONE InputProxy journals every actor-tagged input BEFORE dispatch; typed session persistence; outbound text gated by the P-6 privacy lattice.
state: dark — cobrowse_session 0 refs; empty session log; depends P-13.
supersedes[]: EXTENDS H-13/H-15/E-14/P-13/P-14/P-6/P-10; closes eval ENTRY 14 (the Claude Code in-app-browser gap).  superseded_by[]: none
residual: whole lane + +cobrowse_session/+cobrowse_input rows. key_files[]: (unbuilt). respect[]: native Tauri
  multiwebview EXCLUDED as mechanism (unstable flag); no unjournaled input path can compile; browse content G-1-tainted.

id: P-24  mobile design/test lane  (EXTENSION)
does: The deferred H-14 RN/Expo slice as 3 rungs: DeviceProfile matrix on E-14/H-14/P-23; Expo-web click-to-edit
  through the H-14 proxy; native emulator as a P-12 backend + mobile_axtree P-13 tag + P-14 Maestro-flow crystallize (actuation-only). Typed render_source per row, no rung laundering.
state: dark — DeviceProfile 0 refs; empty session log; depends H-14, P-12, P-13, P-14, P-16.
supersedes[]: EXTENDS H-14/E-14/P-12/P-13/P-14/P-16/P-23.  superseded_by[]: none
residual: three rungs + +emulator_session (+device_profile/render_source fields). key_files[]: (unbuilt).
respect[]: a lower-rung pass NEVER discharges a higher-rung gate (render_source is typed on every row); Maestro is actuation-only.

id: P-25  exogenous-injection + evidence-coupled-revision creativity falsifier (B-EXO-0)  [FIFTH WAVE]
does: SIX arms in TWO independently-preregistered gate families (FAMILY A injection: model-free target-relevant
  artifact retrieval into the champion call vs DIRECT/EXO_SHUFFLED/PAD_MATCHED, 1.3x cap; FAMILY B grounding-as-
  exit: TRIAGE_REVISE vs SHUFFLED_REVISE, 2.5x cap) on a frozen K=30 open-world fixture corpus. Harness-only,
  ~280 calls; NOT spine-gated — executable on user go.
state: partial — the SUCCESSOR to Q-10 REFUTED. Harness prep IN PROGRESS: session s-80604 exists (recon2/4/5,
  context-package-M1.md, exploration/); prep artifacts on disk (workflow/reasoning/exo-injection-*-2026-07-1{4,5}.md,
  exo-injection-fixture-pool-2026-07-14.json). BUT the full V4 exo_ tool suite is NOT yet on disk (only Q-10-era
  conjecture_triage.py / research_creativity_analysis.py / research_creativity_artifact_check.py present; the
  exo_records/exo_trust/exo_injection_ab/exo_creativity_analysis V4 files are unbuilt). No scored arm run. STATUS-
  HISTORY (P-25:4): V4 supersedes V3 as the object to audit after an M1 value-closure defect.
supersedes[]: REPLACES Q-10 B-CREATIVITY-0 (refuted relational-packet coupling) as the successor creativity
  falsifier; INTERNAL: V4 supersedes V3.  superseded_by[]: none
residual: full V4 M0..M5 build (13-module boundary, 14 DO_NOT_REUSE seams, the exo_ suite + independent
  exo_creativity_artifact_check.py verifier), then the scored six-arm run. A SUPPORTED family authors its own
  separate adoption unit; BOTH families NULL closes the creativity line entirely.
key_files[]: .claude/tools/exo_*.py (to build), .claude/tools/conjecture_triage_core.py, workflow/sessions/s-80604/,
  workflow/reasoning/exo-injection-*.md, exo-injection-fixture-pool-2026-07-14.json.
respect[]: Q-10's runner/legacy-checker/75-test module/pad-meter bytes remain PHYSICALLY UNCHANGED — P-25
  imports NONE of them (clean-successor, moved=193/copied=0). Do not resurrect the refuted Q-10 relational mechanism.

---------- Q-plan (13 units; economics/taste/distribution/creativity; most feed the proof spine) ----------
(Q-1..Q-9, Q-11..Q-13 all dark: no signature symbol; empty Q-0 session log. Q-10 is REFUTED with a real artifact.)

id: Q-1  standing goals  [KEYSTONE]
does: A done-gate's off-agent VERIFY artifact becomes a persistent scheduled invariant (O-2 TaskSpec + SSB
  ledger + F-5 quarantine); a violation re-enters the front door. The one genuinely-NEW primitive.
state: dark — standing_goal/GoalEnrolled 0 refs; empty session log. FEEDS PS-6.
supersedes[]: EXTENDS O-2/SSB/F-5/I-25/3F.  superseded_by[]: none
residual: whole primitive + GoalEnrolled/GoalVerified/GoalViolated rows (+3). key_files[]: (unbuilt). respect[]:
  the sentinel NAMES suspects, never FIXES (self-healing = the un-auditable path, Q-1 stub_ban); enrollment is journal DATA, never a write into the frozen gate-manifest (R8).

id: Q-2  quorum wake-gate
does: N cheap cross-family voters gate the expensive lane's WAKE on a scheduled tick (D-13-for-attention);
  advisory-never-the-gate; the gate SELF-RETIRES when measured wake-precision crosses threshold.
state: dark — quorum_wake 0 refs; empty session log; depends O-2 ticks. FEEDS PS-4.
supersedes[]: EXTENDS O-1/O-2/4G/I-25/J-pool/D-13.  superseded_by[]: self (measured self-uninstall, Tier-2 deferred).
residual: wake-gate + WakeQuorumVoted/WakeGated rows (+2); DEFER the self-uninstall. key_files[]: (unbuilt).
respect[]: voters gate COST/INFORM only, never a PASS/BLOCK verdict (TD-3, Q-2 ABORT).

id: Q-3  advisor lane + K-2 advisor arm
does: A mid-loop advisor tool on the weak lane (cheap executor every turn, flagship judgment on demand, ~1
  call/task via the H-9 ledger) + the K-2 advisor $/verified-task arm.
state: dark — advisor_lane/AdvisorConsulted 0 refs; empty session log. FEEDS PS-4; [EXTERNAL] Anthropic 92/63 baseline.
supersedes[]: EXTENDS H-7/I-7/I-11/I-25/4M/H-9/K-2.  superseded_by[]: none
residual: advisor lane + AdvisorConsulted row (+1). key_files[]: (unbuilt). respect[]: advisor INFORMS only, never a verdict (TD-3, Q-3 ABORT); [EXTERNAL] 92/63 never summed with an ELAI number.

id: Q-4  swap-restart attribution
does: A mid-unit SUBSTITUTE_MODEL restarts the unit fresh (iteration state is model-conditioned), keeping only
  the BeliefState; journals a ModelSwapRestart tax row. Code-confirmed gap in autonomous_gate.rs.
state: dark — ModelSwapRestart 0 refs; empty session log. Guards PS-4/PS-5 attribution.
supersedes[]: EXTENDS 4G/4M/J/I-11/F-6/I-25.  superseded_by[]: none; P-17 composes on Q-4.
residual: restart behavior + ModelSwapRestart row (+1) — MAY collapse to doctrine+row if RECON-1 shows
  restart-fresh already holds. key_files[]: (unbuilt, autonomous_gate.rs consumer). respect[]: RECON-1 the task_runner consumer FIRST.

id: Q-5  local per-turn token-hash journal
does: blake3(token_id_seq) per local turn on the J-28 usage row; replay --verify localizes the first divergent
  turn; scopes the RRA claim honestly. LOCAL-lane only.
state: dark — empty session log; EXTENDS J-28 with an output_token_hash FIELD (no new action_type if schema-legal).
supersedes[]: EXTENDS G-2/J-28.  superseded_by[]: none; Q-11 reuses the token-hash surface.
residual: the hash field + replay-verify localization. key_files[]: (unbuilt, extends J-28 row). respect[]:
  LOCAL-lane only (cloud is a reproducible action-stream); it LOCALIZES divergence, does not claim hw-independent determinism.

id: Q-6  gate-manifest evidence-corruption corpus
does: ONE evidence-type-keyed corpus GENERATOR + a measured per-gate kill-rate; turns wiring R1/R2 into a
  tested number. Frozen auditor-owned artifact.
state: dark — empty session log. Feeds the K compliance/appsec brief.
supersedes[]: EXTENDS H-selfval/3I-DST/R1/R2/gate-manifest.  superseded_by[]: none; P-21 extends Q-6's corpus verdict.
residual: corpus generator + runner + GateKillRateMeasured row (+1). key_files[]: (unbuilt, .claude/tools frozen).
respect[]: one GENERATOR not hand-written per-gate corpora (A4); frozen auditor artifact, never run by the plan run it judges (R8).

id: Q-7  weak-lane code-style bench
does: Measure whether a code-STYLE policy raises weak-lane task success; cheapest 2-variant falsifier first;
  promote to the L gate only on a measured beyond-noise win.
state: dark — empty session log. FEEDS PS-4.
supersedes[]: EXTENDS D-6/E-54/local-lane/C-compl/L-gate.  superseded_by[]: none
residual: bench + WeakLaneStyleBench row (+1). key_files[]: (unbuilt). respect[]: a NULL result promotes nothing and closes honestly (Carmack's tweet is a hypothesis, not evidence).

id: Q-8  verifier size/dep ceiling
does: A locked crate-count + binary-size ceiling on the M standalone verifier (the auditor must stay smaller
  than the audited). PENDING a real M verifier build sets the baseline.
state: dark — empty session log; PROOF_SPINE none (discipline, delegated ack); depends a real M-5a build.
supersedes[]: EXTENDS M-verifier/K-4.  superseded_by[]: none
residual: ceiling check + VerifierSizeMeasured row (+1). key_files[]: (unbuilt). respect[]: no fabricated size number (KD-1); ceiling PENDING until M sets the baseline.

id: Q-9  distribution hygiene (2 units)
does: UNIT A elai_change markers + CI quarantine on the 4M VENDORED CLIs; UNIT B one surface/command registry
  drift check extending cross_lang_diff.
state: dark — empty session log; PROOF_SPINE none (distribution); sequenced AFTER the spine.
supersedes[]: EXTENDS 4M-vendored/cross_lang_diff/ts-rs.  superseded_by[]: none
residual: both units (build/CI checks; artifact = a frozen-checker exit code, no new action_type). key_files[]:
  (unbuilt). respect[]: fork-hygiene applies to the 4M VENDORED CLIs, NOT the diverged Phase-0 fork; ONE surface registry (A-1/N-2).

id: Q-10  research-conditioned creativity
does: A SOURCE_ONLY relational ResearchInspirationPacket coupling research to the creativity harness; a locked
  three-gate executable benchmark ladder (B0→B1→B2).
state: REFUTED — RAN, VALID REFUTED run 3 (b0v3) 2026-07-14. Benchmark artifact ON DISK
  (workflow/benchmarks/research-creativity-b0v3-2026-07-14/ with manifest.json, executions/, report.md).
  report.md: "## Joint verdict: REFUTED ... Every P1 delta is NEGATIVE ... every P2 delta is ~0" at ~8.8x DIRECT
  cost. Gate ladder HALTS: no B1, no skill/Rust/runtime consumer, no paper-to-plan/idea-intake adoption.
supersedes[]: none  superseded_by[]: P-25 B-EXO-0 (the user-directed successor falsifier with a different mechanism).
residual: NONE for code — every downstream adoption stage is permanently FORBIDDEN by the refutation. The
  one-way-packet DESIGN is not refuted, but this specific coupling does not pay on the synthetic falsifier.
key_files[]: workflow/benchmarks/research-creativity-b0v3-2026-07-14/report.md,
  docs/thefinalglance/Q-plan/_grounding/Q-RESEARCH-CREATIVITY-GROUNDING-MAP.md.
respect[]: the drain MUST treat Q-10's downstream as dead — do NOT let any later unit resurrect the relational-
  packet-to-creativity coupling, FDL, controller cycles, ProbeUtility, or an archive-to-L bridge on this evidence.

id: Q-11  novelty-instrument canary + fixed-budget bench  [AIDE2, ships FIRST]
does: A live-path known-answer novelty canary that fail-closes verdict emission on a discrimination-dead
  classifier (C6) + fixed-dollar-at-locked-cap explore reporting with two do-not-win control arms (C10) + a
  frozen pre-change baseline. Instrument-before-search.
state: dark — NoveltyCanary/instrument_strength 0 refs; empty session log. FEEDS PS-4/PS-3/PS-6; Q-12 gates on it.
supersedes[]: EXTENDS novelty.rs/conjecture_triage/frontier_bench/J-28/RoutingEconomicsMeasured(schema 274).  superseded_by[]: none
residual: canary + fixed-budget reporting + baseline + instrument_strength FIELD (or +NoveltyCanaryChecked).
key_files[]: (unbuilt, extends novelty.rs). respect[]: RoutingEconomicsMeasured is at schema 274 (K-2 shipped — 7 refs in map); re-read live schema at edit.

id: Q-12  research-lane context + search policy  [AIDE2, gated on Q-11]
does: C3 per-operator minimal-prompt matrix + journaled prompt-size ratio, THEN C1+C2 lineage-bandit + plateau
  fork-on-stall as ONE selectable experiment behind default-OFF ELAI_RESEARCH_BANDIT_ENABLE with a mandatory
  deterministic-scheduler control arm + kill criterion. No default-routing change.
state: dark — empty session log; gated on Q-11 bench green. FEEDS PS-4.
supersedes[]: EXTENDS web_research_orch/research_tree/Q-11-bench/B-40.  superseded_by[]: none
residual: prompt matrix + bandit experiment + a wire-dark-allowlist.toml row for ELAI_RESEARCH_BANDIT_ENABLE (type=experiment, R9).
key_files[]: (unbuilt, extends web_research_orch + research_tree.rs). respect[]: the bandit stays a default-OFF
  allowlisted experiment; promotes only on a beyond-noise equal-budget win on Q-11's bench + a second benchmark.

id: Q-13  receipt-bound briefing lines  [AIDE2]
does: Every compiled Cosmos briefing line resolves a fetched-byte-hash or journal-row-id receipt at compile time
  or is dropped and counted (C11); reuses E-48/H-52. A fabricated briefing line becomes unrepresentable.
state: dark — empty session log; PROOF_SPINE none (trust); rides alongside Q-12.
supersedes[]: EXTENDS E-48/H-52/cosmos_worker/briefing.rs.  superseded_by[]: none
residual: the receipt resolution + a dropped_receipt_lines FIELD (no new action_type if schema-legal).
key_files[]: (unbuilt, extends briefing.rs/cosmos_worker). respect[]: reuse E-48/H-52; the dropped-line count is the observable.

================================================================================
DRAIN-SYSTEM NOTES (implications for the backlog-drain sequencer)
================================================================================
1. M/N/O/P/Q are the FRONTIER, not shipped work at risk of reversion. The reversion-risk framing in the brief
   applies to B..L (the units actively shipping via the opus/glm walk); M-Q are the consumers-in-waiting.
2. The load-bearing constraint from THIS cluster: N is a giant UNIFICATION front. Draining B..L must NOT ship
   an (n+1)th parallel implementation of any concern an N unit is slated to unify (see SUPERSESSION_EDGES —
   benches→N-1, routers→N-5, provenance scales→N-6, counterfactual engines→N-9, constrained-output→N-13,
   claim types→N-4, fold surfaces→N-11, failure/abstain/drift→N-12). Every such B..L unit shipped BEFORE its
   N unifier ADDS a retirement-map entry N later has to absorb — cheap now, but track it so N is not surprised.
3. HARD SEQUENCING GATES that bind the drain: GATE M-0 blocks ALL of M until K A-1..A-4 + K-3 + K-24 L1..L4;
   N/O/P/Q execute AFTER N-1 (the measurement spine) + PS-1..PS-6. N-1 is the single highest-fan-in unit in
   the whole corpus (~30 promotions terminate there) — it is the true unlock. N-2 then EMITS the fan-in ranking
   that ORDERS N-3..N-12; do not hand-order past it.
4. Q-10 is a permanent DEAD-END: its downstream is forbidden by refutation. P-25 is its successor and is the
   only M-Q unit with a live executable path today (not spine-gated, ~280 free-lane calls, on user go). If the
   drain reaches a "creativity" item, route to P-25, never re-open Q-10's mechanism.
5. Live schema baseline for any bump: SCHEMA_VERSION derives at ssb.rs:1559 (DDL_EPOCH -309 + SSB_ACTION_TYPES.len);
   RoutingEconomicsMeasured landed at schema 274 (K-2, 7 refs in the fresh map). Re-read at edit; NEVER hard-code.

End of cards-MNOPQ.md.
