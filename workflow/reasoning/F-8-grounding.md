GROUNDING SWEEP -- phase F-8
Source: docs/thefinalglance/F-plan/F-8_scf_symbolic_conflict_selfmod.md
Synthesis source: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md (sec 4 C8 / sec 1.6 / sec 3 Pillar 4 / sec 5 TIER 3 / sec 7)
SCF prior-art source: workflow/research/shard-prior-art/04-semantic-consensus.md + 00-COMPARISON Pillar 2/4
Date: 2026-06-02
FSM: PLANNING | Session: s-60824

SPEC REFERENCES
[F-8 cites the synthesis + shard prior-art + elai.map, not docs/specs/guide/steps/phase-*.txt -- this is an F-plan, source-grounded to the synthesis, not a guide-phase plan. No phase-{X}.txt refs expected.]
VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md sec 4 C8 / sec 1.6 / sec 5 / sec 7 | GROUNDED | read in full; C8 mechanism + REVISED SCOPE (self-mod-only, WARN-then-escalate, inter-agent DEFER) + sec 1.6 27.9% correction all present verbatim
workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md Pillar 2+4 | GROUNDED | Pillar 4 net (line 69): "DGM archive search UNDER SHARD's guards plus a real semantic pre-execution review -- which neither ships" = the F-8 Pillar-4 synthesis target verbatim; Pillar 2 confirms SCF IntentNode + 3 conflict types + Graphiti LLM-judged alternative
workflow/research/shard-prior-art/04-semantic-consensus.md | GROUNDED | canonical SCF source: arXiv:2604.16339; repo curiosityexplorer/semantic-consensus 0 stars / empty tests/ / single 1h push / no license / no CI; 3 SYMBOLIC conflict methods confirmed from source (cde.py)
elai.map integration points (self_mod_safety/types/layer1a, tripwire, config_archive, ssb) | GROUNDED | DEFs confirmed: apply_proposal (self_mod_layer1a.rs:336 trait / :831 impl), ParameterKey enum (self_mod_safety.rs:94+), ConfigArchive (config_archive.rs:148), observe_post_consensus_tripwires (used tripwire_observable_smoke.rs:114), apply_proposal_contract.rs exists

CLAIM CLASSIFICATION
SCF precision = 27.9% | GROUNDED | arXiv:2604.16339 Table 2 P/R/F1 = 27.9/65.2/39.0 (04-semantic-consensus.md:144,168). DO-NOT-TARGET: it is the precision column from a SIMULATION, not an ELAI metric
27.9% is a SIMULATION-ONLY limitation (not a design choice) | GROUNDED | synthesis sec 1.6 explicitly CORRECTS the source's "deliberately low-precision" framing (04-semantic-consensus.md:129) to OVERSTATED; runs are n=600 single-seed sim, "no actual AutoGen/CrewAI/LangGraph or live LLM" (:96,:128)
27.9% justifies WARN-then-escalate NOT hard-BLOCK on self-mod path | GROUNDED | synthesis C8 REVISED SCOPE + sec 7 DEFER list; one-promotion-per-session bounds FP cost; F-8 plan Task 2 + ACCEPTANCE "WARN-then-escalate, not hard-BLOCK"
SCF 3-conflict-type method (contradictory/resource-contention/causal-violation), LLM-free symbolic | GROUNDED | 04-semantic-consensus.md:42-46 from cde.py source: _detect_contradictory (post-string compare), _detect_resource_contention (precond vs postcond + numeric), _detect_causal_violation (dependency postcond fails dependent precond); SentenceTransformer is OPTIONAL/commented-out, core is rule-based
IntentNode pre/postconditions are Dict[str,str] set/string comparison (not embeddings) | GROUNDED | 04-semantic-consensus.md:37,40 from sig.py source
Pillar-4 synthesis target = DGM archive search under guards + real semantic pre-execution review, neither ships today | GROUNDED | 00-COMPARISON Pillar 4 net line 69 verbatim; DGM improvement.py is a 404 / spec-only (00-COMPARISON:88)
DEFER inter-agent BSP half: 27.9% FP in cap=3 parallel loop = noise | GROUNDED (as reasoning from a grounded number) | synthesis sec 7 + C8 REVISED SCOPE; the noise-degradation is a design inference from the (grounded, sim-only) 27.9% precision, not an independently measured ELAI number -- inference flagged, not a target
inter-agent half collides with C-50 ContractSpec precondition/postcondition namespace | VERIFY -> GROUNDED | F-8 + synthesis claim C-50 is PLANNED/not-shipped (zero elai.map matches); F-8 avoids it by naming SymbolicIntent/IntentCondition/asserts/requires; collision risk is forward-looking (C-50 not yet in tree), so the avoidance is sound by construction
Graphiti ships LLM-judged contradiction detection (higher-maturity cloud-lane alternative for inter-agent half) | GROUNDED | 00-COMPARISON:24,49 "Zep/Graphiti ... shipped, LLM-judged, non-destructive semantic contradiction invalidation"; 27k stars Apache-2.0; arXiv:2501.13956 cited in synthesis C5
self-mod stack (D-3 consensus / D-4 tripwire / E-7 Two-Gate) SHIPPED | GROUNDED | elai.map: tripwire.rs (TripwireKind IqrOutlier/Perplexity/Divergence), observe_post_consensus_tripwires reachable in tests, apply_proposal_contract.rs present, config_archive.rs DEFs present
SHARD semantic detection self-undercut (E2 7/10 recall < syntactic 9/10) | GROUNDED | 00-COMPARISON:48 verbatim; reinforces "semantic conflict detection is a DESIGN HYPOTHESIS, not a proven uplift"
F-8 "catches X% of bad promotions" would be UNGROUNDED until measured | GROUNDED (the plan already labels it) | F-8 Open-questions: any accuracy claim UNGROUNDED until measured on >=20 ELAI self-mod proposals -- correctly pre-flagged
blake3 fingerprint (never blake2/sha2) | GROUNDED | CLAUDE.md HARD RULE; F-8 Task 1 + Citations comply
arXiv:2604.16339 (SCF) exists / verified | GROUNDED | 04-semantic-consensus.md:12 "VERIFIED, resolves to a real CS.AI submission, submitted 2026-03-13"; not from training
arXiv:2501.13956 (Graphiti) cited for the alternative | INGEST (low priority) | cited in synthesis C5 as confirmed in edge_operations.py; if F-8-decisions wants to cite Graphiti specifics, confirm the bi-temporal-invalidation claim via paper-ingestor before quoting any number. F-8 itself makes NO Graphiti number claim, so non-blocking for F-8

NUMBERS THAT MUST NOT BECOME TARGETS (explicit DO-NOT-TARGET list)
- 27.9% (SCF precision, Table 2) -- sim-only, single-seed n=600, no live LLM; it is a NON-target / known-limitation, never an ELAI accuracy goal or a hard-BLOCK trigger threshold
- 65.2% / 39.0 (SCF recall / F1, Table 2) -- same sim-only provenance; do not cite as expected ELAI behaviour
- 100% vs 25.1% (SCF workflow completion vs Judge-Agent baseline) -- SIMULATION harness, UNREPRODUCED on real agents; never an ELAI uplift claim
- 74.3% conflict-reduction / 145ms overhead -- README-only, NOT in paper body, UNRECONCILED; do not cite at all
- SHARD E2 7/10 semantic, 9/10 syntactic -- 20 hand-built pairs; evidence that semantic detection is a hypothesis, not a target
- cap=3 / one-promotion-per-session -- ELAI-internal config facts (NOT external claims), fine to state; the FP-noise inference drawn from them is reasoning, not a measured ELAI number

RELIABILITY NOTE (SCF source)
SCF reliability = LOW. arXiv:2604.16339 + repo curiosityexplorer/semantic-consensus: 0 stars / 0 forks / single ~1h push 2026-03-10 / empty tests/ (only __init__.py) / NO license on repo (all-rights-reserved -> NOT reusable code) / paper CC BY-NC-ND 4.0 (no derivatives) / 0 citations / no peer-reviewed venue located / no CI / no framework adapter / no live LLM in evals. Per CLAUDE.md source-tiers this is a simulation scaffold, NOT a library: F-8 PORTS the mechanism (re-implements the 3 rules in Rust), it does NOT depend on or adapt SCF code (correct -- the NoDerivatives + no-license posture forbids reuse anyway). The mechanism is GROUNDED (read from cde.py/sig.py source); the precision/recall/F1 numbers are LOW-reliability sim-only and are correctly treated as NON-targets throughout F-8.

ACTIONS REQUIRED
- (optional, non-blocking) ingest Graphiti paper arXiv:2501.13956 via paper-ingestor mode=paper BEFORE the future inter-agent F-N cites any Graphiti contradiction-detection number; F-8 itself makes no Graphiti number claim so this does not block F-8 TESTING_FIRST
- carry the DO-NOT-TARGET list above into F-8-decisions.md GROUNDING STATUS table (Task 7d/7e): mechanism=PORTED, precision/recall/F1=UNVERIFIED-AS-ELAI-TARGET sim-only; the plan already mandates this -- verify the decision doc lands it
- at audit: confirm no fixture or test asserts "27.9%" or any SCF sim number as an expected ELAI value (it must appear only as a documented non-target in prose)

NET: all external-behaviour claims in F-8 are GROUNDED to the synthesis + the SCF prior-art source + elai.map. The 27.9% precision is correctly classified as a SIMULATION-ONLY known-limitation (NON-target) justifying WARN-then-escalate over hard-BLOCK; the inter-agent BSP half is correctly DEFERRED (FP-noise inference + C-50 collision), with Graphiti LLM-judged detection as the grounded higher-maturity cloud-lane alternative. One optional, non-blocking INGEST (Graphiti) is staged for the future inter-agent F-N only.

VALIDATION WARNINGS
- F-8 cites no docs/specs/guide/steps/phase-{X}.txt -- EXPECTED: F-plans are source-grounded to the VIBESEC synthesis, not the guide spec; not a traceability defect
- the source SCF file (04-semantic-consensus.md:129) uses the "deliberately low-precision" framing that synthesis sec 1.6 explicitly corrected; F-8 carries the CORRECTED framing -- watch that no downstream edit re-imports the source's original "deliberately" wording (synthesis sec 1.6 / F-8 sec 1.6 reference both forbid it)
- all other sources validated
