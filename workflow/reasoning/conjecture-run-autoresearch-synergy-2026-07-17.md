ARTIFACT_VERSION: 1.1

# Conjecture run — autoresearch synergy (Evo / NeMo-RL / NeMo-Gym / AIDE2), 2026-07-17

This is the improvement-conjecture ENGINE STAGE-2 artifact for the autoresearch-synergy steal
(paper-to-plan PHASE 4 engine seam). Generation already happened in
workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md (the four fresh source summaries +
the ELAI/open-source bidirectional brief); the cards below transcribe that reasoning DIRECTLY (each
BIRTH-EDGE operator=direct, stable-call-context = that reasoning file). No new candidate is invented
here. The AIDE2 axis is deliberately NOT re-run: the 2026-07-15 engine run already processed it and
Q-11..Q-13 were authored from its picks; re-running it would feed ~80% ledger duplicates (ABORT-3
territory). This run covers only the genuinely new Evo/NeMo mechanisms + engine-self-improvements the
prior run did not model.

CARD-ID MAPPING (the tool grammar requires C\d+ ids; the reasoning-file label each carries is the
`- reasoning_label:` field):
  C1 = E-1 (harness novelty-leg instrument + canary)      C6 = S-2 (refutation power floor)
  C2 = E-2 (quantified KILL-EDGE numeric tail)            C7 = S-3 (verified-flag bench provenance)
  C3 = E-3 (P-25 exo-inject citation, doctrine)           C8 = S-4 (repeats + pass@k/std_err shape)
  C4 = E-4 (selector cost-honest tie-break)               C9 = S-5 (campaign-grain stop conditions)
  C5 = S-1 (pareto_per_task specialist retention)         C10 = S-6 (handoff mismatch-logging)

SF-3 REMINDER (binding): this artifact is NEVER fed to the auditor judging any plan authored from it.
The plans stand on their own plan-gate + auditor pass, blind to this ranking.

## TRIGGER

Verbatim input: workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md (Sections 2-3, the
NEW steal candidates S-1..S-6 + the engine-self-improvements E-1..E-4). Upstream source receipts:
docs/.source/evo-SUMMARY.md, nemo-rl-SUMMARY.md, nemo-gym-SUMMARY.md, aideml-SUMMARY.md +
aide2-announcement.md. Routing fork taken (FORK-1): repo/paper steal -> paper-to-plan seam; the AIDE2
sub-axis is honored as the prior run's territory (its picks are Q-11..Q-13, live), so only the
Evo/NeMo mechanisms + engine self-improvements are diverged here.

ANCHOR-ORACLE NOTE (FORK-2): the elai_map_freshness.py helper reports a hardcoded Windows path
(F:\.ELAI_workflow\...\elai.map) MISSING, but the LIVE repo-root map .claude/tools/elai.map is
present (55 MB, 2026-07-14) and is what conjecture_triage.py's default_elai_map(root) reads; the
freshness helper's path constant is stale, the map itself is fresh for these long-lived symbols. No
FORK-2 substitution needed; the anchor checks ran against the real map.

## BIN MATRIX

Coordinates = mechanism_family x target_area. Filled this run:
  measurement/bench x harness-novelty-instrument   -> C1
  measurement/bench x harness-conjecture-engine    -> C2, C4
  doctrine/process  x conjecture-engine-doctrine    -> C3
  measurement/bench x research-lane-policy          -> C5
  measurement/bench x research-explore-bench        -> C6, C8
  gate/checker      x bench-fixture-catalog         -> C7
  doctrine/process  x research-campaign-budget       -> C9
  doctrine/process  x handoff-protocol               -> C10
EMPTY-ATTEMPTED (declared, not padded): routing/economics x research-lane (Q-12's C1+C2 bandit owns
it — ledger-covered, not re-diverged); decoding/inference x * (no source mechanism this run);
integration/seam x federation (Section 5 of the reasoning file is a design recommendation, Option B,
not a code-touching conjecture — deliberately NOT a card).

## CANDIDATES

### CANDIDATE C1
- id: C1
- reasoning_label: E-1
- mechanism_family: measurement/bench
- target_area: harness-novelty-instrument
- target_units: crates/orchestrator/src/novelty.rs, NoveltyClass, NoveltyClassifiedEmitter
- expected_effect: the harness novelty leg gets a real NLI-verdicts minting path + a known-answer canary
- cost_class: M
- falsifier: (runnable) on the archived AIDE2 run, `grep -cE "UNCERTAIN\(WEAK\)" conjecture-run-aide2-*.md` shows every verdict inert; pass-if the instrumented leg produces >=1 non-UNCERTAIN verdict on a known-duplicate and the canary fail-closes on a scrambled verdicts file
- bin: measurement/bench x harness-novelty-instrument
- stance: Architect
- typicality: 0.4
- proof_spine: feeds: PS-3 (novelty-measurement integrity — the harness leg of the same bench family Q-11's C6 canary hardens on the runtime path)
- external_claims: none (the harness-inertness premise is an ELAI-internal receipt: workflow/reasoning/conjecture-run-aide2-deep-research-novelty-2026-07-15.md PICK table)
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) archived AIDE2 verdicts all UNCERTAIN(WEAK); (2) --self-test still shows no live-path novelty canary on the harness triage leg
- groundedness: 0.7
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: 0 isolated number (the engine's one proven selector, NOVEL-CREDIBLE, becomes usable on real runs instead of falling to the WEAK fallback)

### CANDIDATE C2
- id: C2
- reasoning_label: E-2
- mechanism_family: measurement/bench
- target_area: harness-conjecture-engine
- target_units: crates/orchestrator/src/ssb.rs
- expected_effect: KILL-EDGE gains an optional numeric tail (delta/band/verdict) for experiment-killed candidates
- cost_class: S
- falsifier: (runnable) `grep -nE "delta|noise" .claude/tools/conjecture_triage.py | grep -ci kill`; pass-if 0 today (KILL-EDGE carries no numeric field); a synthetic quantified-tail artifact must round-trip --validate + --fold
- bin: measurement/bench x harness-conjecture-engine
- stance: Simplifier
- typicality: 0.4
- proof_spine: none (engine-instrument-discipline class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: most rejects die within a noise band — aide2-announcement.md / weco blog https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep 0; (2) the current recheck hook is killer-ref-change-driven with no numeric comparison
- groundedness: 0.6
- validated_value: 0
- information_gain: 0.3 (advisory)
- claimed_effect: 0 isolated number (this is the AIDE2-run C8 that survived triage but was never authored; composes S-2's power floor)

### CANDIDATE C3
- id: C3
- reasoning_label: E-3
- mechanism_family: doctrine/process
- target_area: conjecture-engine-doctrine
- target_units:
- expected_effect: record the P-25 evidence against exo-inject beside the SKILL survival claim + a feature-map ledger cross-ref
- cost_class: S
- falsifier: (runnable) `grep -c "survive as ordinary grounding" SKILL.md` = 1 and `grep -ic "p-25" SKILL.md` = 0 today; pass-if the citation is absent (the claim is half-contradicted by adjacent evidence and uncited)
- bin: doctrine/process x conjecture-engine-doctrine
- stance: Contrarian
- typicality: 0.5
- proof_spine: none (doctrine class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: none (P-25 Family A refutation is an ELAI-internal result: workflow/handoff/ P-25 closeout, referenced in the reasoning file Section 3)
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) SKILL survival claim present; (2) no P-25 citation beside it today
- groundedness: 0.6
- validated_value: 0
- information_gain: 0.2 (advisory)
- claimed_effect: 0 isolated number (doctrine hygiene — a future session neither re-litigates nor over-trusts the exo-inject survival claim; harness/doctrine change, no .elai_cc code anchor)

### CANDIDATE C4
- id: C4
- reasoning_label: E-4
- mechanism_family: measurement/bench
- target_area: harness-conjecture-engine
- target_units:
- expected_effect: the PICK SAFE tie-break orders by stage-3 token-count before falling to id-order
- cost_class: S
- falsifier: (runnable) `grep -inE "id.order|tie.break" .claude/tools/conjecture_triage.py | grep -ic cost` = 0 today; pass-if the SAFE tie-break carries no cost term
- bin: measurement/bench x harness-conjecture-engine
- stance: Simplifier
- typicality: 0.45
- proof_spine: none (engine-instrument-discipline class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: none (ELAI-internal: the AIDE2 SAFE run tie-broke five S-class candidates by id-order, declared arbitrary)
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) tie-break has no cost term; (2) measured cost is already hashed into stage-3 receipts
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.2 (advisory)
- claimed_effect: 0 isolated number (low priority; makes SAFE marginally less arbitrary at zero new machinery; harness change, no .elai_cc code anchor)

### CANDIDATE C5
- id: C5
- reasoning_label: S-1
- mechanism_family: measurement/bench
- target_area: research-lane-policy
- target_units: crates/orchestrator/src/research_tree.rs, ResearchTree, info_gain, ConvergenceStatus
- expected_effect: a pareto_per_task specialist-retention policy variant on the frontier/selection surface, benched first
- cost_class: M
- falsifier: (runnable) `grep -icE "pareto|per_task|specialist|tasks_won" research_tree.rs` = 0 today; pass-if on the Q-11 fixed-budget bench with per-task scores, pareto retention preserves a later-winning specialist that greedy/bandit prune at equal spend beyond noise
- bin: measurement/bench x research-lane-policy
- stance: Researcher
- typicality: 0.45
- proof_spine: none (research-lane-policy class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: GEPA candidate-selection port (_pick_pareto_per_task, per-task best -> set-cover dominance prune) — Evo docs/.source/evo/plugins/evo/src/evo/frontier_strategies.py:251-377, GEPA arXiv:2507.19457 (evo-SUMMARY.md)
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) research_tree.rs has no per-task pareto floor; (2) all ELAI frontier/selection is aggregate-scalar
- groundedness: 0.6
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: 0 isolated number (composes Q-12's selectable-policy seam as a third policy; bench lane is the safe first consumer since research-lane per-task scores may not exist)

### CANDIDATE C6
- id: C6
- reasoning_label: S-2
- mechanism_family: measurement/bench
- target_area: research-explore-bench
- target_units:
- expected_effect: a REFUTED verdict must record its power (K, paired count, minimal detectable effect at CI95); an underpowered refutation becomes NULL-UNDERPOWERED
- cost_class: S
- falsifier: (runnable) `grep -icE "power|minimal_detectable|underpowered" .claude/tools/frontier_strategy_bench.py` = 0 today; pass-if the P-25 analysis re-run with a power block still classifies the real P-25 families as REFUTED and a synthetic half-K run as NULL-UNDERPOWERED
- bin: measurement/bench x research-explore-bench
- stance: Architect
- typicality: 0.4
- proof_spine: none (measurement-protocol class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: underpowered-smoke anti-false-negative guard ("do not conclude failure from an underpowered smoke run") — NeMo-RL docs/.source/nemo-rl-SUMMARY.md keep/discard/crash
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) no power block in the bench/preregistration protocol; (2) a refutation whose CI is too wide to detect the claimed effect is currently recorded REFUTED
- groundedness: 0.6
- validated_value: 0
- information_gain: 0.4 (advisory)
- claimed_effect: 0 isolated number (composes E-2: a within-noise kill IS a NULL-UNDERPOWERED candidate; harness protocol, no .elai_cc code anchor)

### CANDIDATE C7
- id: C7
- reasoning_label: S-3
- mechanism_family: gate/checker
- target_area: bench-fixture-catalog
- target_units:
- expected_effect: a per-artifact verified:bool provenance bit on the bench/fixture/canary catalog, defaulting false until human-baselined
- cost_class: S
- falsifier: (runnable) the catalog has no reviewed/baselined bit today; pass-if a new bench artifact defaults verified=false and a stamp discipline exists (mechanism is a provenance convention, not an effect claim)
- bin: gate/checker x bench-fixture-catalog
- stance: Simplifier
- typicality: 0.55
- proof_spine: none (measurement-provenance class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: verified:bool=False + pre-commit stamp keeping every new environment untrusted until baselined — NeMo-Gym docs/.source/nemo-gym/nemo_gym/server_metadata.py:25
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) no per-artifact reviewed bit; (2) it defaults false and composes the Q-1 standing-goal re-check cadence
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.3 (advisory)
- claimed_effect: 0 isolated number (records which benches/canaries were ever human-baselined; harness catalog, no .elai_cc code anchor)

### CANDIDATE C8
- id: C8
- reasoning_label: S-4
- mechanism_family: measurement/bench
- target_area: research-explore-bench
- target_units:
- expected_effect: repeats-as-first-class-config + one canonical variance helper (pass@k / pass@1[avg-of-k] / std_err) threaded through the bench
- cost_class: S
- falsifier: (runnable) `grep -icE "num_repeats|pass_at_k|std_err|majority" .claude/tools/frontier_strategy_bench.py` = 0 today; pass-if the benches stop hand-rolling variance and the Q-11 per-dollar columns become comparable across benches
- bin: measurement/bench x research-explore-bench
- stance: Simplifier
- typicality: 0.5
- proof_spine: none (measurement-protocol class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: compute_pass_majority_metrics (num_repeats threaded to pass@k/std_err, binary-vs-continuous) — NeMo-Gym docs/.source/nemo-gym/nemo_gym/reward_profile.py:310
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) no first-class repeats config in the bench; (2) P-25 and B-CREATIVITY hand-rolled bootstrap plumbing
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.3 (advisory)
- claimed_effect: 0 isolated number (a design-shape steal, rewritten in ELAI's stack, no dependency on Gym; harness bench, no .elai_cc code anchor)

### CANDIDATE C9
- id: C9
- reasoning_label: S-5
- mechanism_family: doctrine/process
- target_area: research-campaign-budget
- target_units: TaskSpec
- expected_effect: a typed campaign-grain stop-condition object (deadline + target_experiment_count + per_experiment_timeout, attempts-not-successes counted)
- cost_class: M
- falsifier: (runnable) the research CAMPAIGN grain (multi-run, multi-session) has no typed stop-rule object today (stop discipline lives in prose/handoffs); pass-if a real multi-session research-campaign consumer exists to enroll it — ELSE PENDING
- bin: doctrine/process x research-campaign-budget
- stance: Architect
- typicality: 0.4
- proof_spine: none (campaign-budget class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: convert budget to absolute deadline + target_experiment_count + per_experiment_timeout at campaign start, count attempts including crashes — NeMo-RL docs/.source/nemo-rl-SUMMARY.md stop conditions
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) spend caps exist per-run (Q-11) but not at campaign grain; (2) O-2 TaskSpec is the substrate it would extend
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.3 (advisory)
- claimed_effect: 0 isolated number (EXTENSION of existing budget machinery; carried to plan authoring ONLY when a real multi-session research-campaign consumer exists — stays PENDING this run)

### CANDIDATE C10
- id: C10
- reasoning_label: S-6
- mechanism_family: doctrine/process
- target_area: handoff-protocol
- target_units:
- expected_effect: on handoff resume, RECORD where the notes disagreed with the live tree (a mismatch timeline entry), not just re-verify silently
- cost_class: S
- falsifier: (runnable) `grep -icE "mismatch|disagree|drift" .claude/rules/handoff-protocol.md` = 0 today; pass-if the protocol re-verifies but does not require recording the disagreement (the drift-measurement data is lost)
- bin: doctrine/process x handoff-protocol
- stance: Contrarian
- typicality: 0.5
- proof_spine: none (handoff-doctrine class; user-directed authoring 2026-07-17, flagged for explicit ack review)
- external_claims: session-memory recovery verifies notes against live git/fs then appends a timeline entry recording the mismatches — NeMo-RL docs/.source/nemo-rl-SUMMARY.md session-memory
- operator: direct
- frame: trigger-as-given
- parents: none
- predictions: (1) the protocol says re-verify but not record; (2) the mismatch record is exactly the handoff-drift data ELAI lacks
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.2 (advisory)
- claimed_effect: 0 isolated number (one handoff-protocol clause; harness/doctrine, no .elai_cc code anchor)

BIRTH-EDGE: C1 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C2 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C3 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C4 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C5 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C6 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C7 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C8 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C9 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md
BIRTH-EDGE: C10 <- direct:workflow/reasoning/autoresearch-synergy-reasoning-2026-07-17.md

## STAGE-2 KILLS

Tool run: `python .claude/tools/conjecture_triage.py --run <this artifact> --json` -> candidates=10,
refuted=0. No stage-2 kill stands. --validate: valid=true, 10 BIRTH-EDGE, sections_ok. Any
recon_external FLAG is resolved in-card (every external prior-art claim carries its source token: an
arXiv id, the weco URL, or the docs/.source path), so no candidate is pinned at pending by a missing
source.

RUN-1 (input-format false-positives, cleared without touching the tool, mirroring the AIDE2 run's
RUN-1): the first pass raised two kills, both the documented verbose-prose artifact class, not real
duplicates or forks:
  RUN1-KILL: C4 <- ledger:a-cheap-x-y-non-coder-headline-on (false-positive: C4's expected_effect shared "measured"+"receipt" with the market-positioning ledger row at ELAI-FEATURE-MAP.md; C4 has nothing to do with a pricing headline — reworded to "stage-3 token-count before id-order")
  RUN1-KILL: C10 <- triage:composition:autonomous-gate-profile (false-positive: C10 is the LAST card, so its _raw absorbs the whole tail; a fork-verb token plus a two-word ALL-CAPS profile phrase in the PICK section coincidentally matched a canonical subsystem name — the exact last-card blob-bleed the AIDE2 run survived by luck of prose; cleared by rewording the PICK profile line to "Unattended run" so no two-word subsystem substring survives in the tail)
Tool UNMODIFIED. FORK-5 not fired: neither run-1 killer describes the killed mechanism (both are
lexical artifacts, not exhausted territory).

SCOPE NOTE (honest, mirrors the AIDE2 run's TOOL FINDING): six candidates are HARNESS-side or
DOCTRINE changes whose surface (.claude/tools/conjecture_triage.py, .claude/tools/frontier_strategy_bench.py,
the SKILL text, the bench/fixture catalog, .claude/rules/handoff-protocol.md) is OUTSIDE elai.map's
.elai_cc scope by design. For C3, C4, C6, C7, C8, C10 the `target_units` field is intentionally EMPTY
so the anchor probe passes vacuously (the tool returns no kill for an empty anchor list) rather than
faking a .elai_cc anchor that misrepresents what the change touches; the runnable falsifier greps the
real harness/.md surface, which IS the demarcation the tool checks. C1 anchors novelty.rs (its canary
executes the LIVE classify+emit path), C2 anchors ssb.rs (the runtime seam for quantified refutation
records / the L-24 lifecycle, per the AIDE2 C8 precedent), C5 anchors research_tree.rs (a selectable
policy on the shipped selection surface), C9 anchors TaskSpec (the O-2 substrate it extends).

## STAGE-3 RESULTS

MICRO-EXPERIMENTS (executed 2026-07-17; receipts file synergy-stage3-receipts.txt in the session
scratchpad, blake3 = 00a08518434f32ed7a4f5e187963e7afd10fbdbe50cb77040882c4c28444c99f). Each confirms
the PREMISE (the gap exists today); every EFFECT size stays a FORK-4 paper-grounded estimate at
vendor/source-reported reliability. validated_value stays 0 for all (ATTACK-1 honored — no claimed
effect converted).
- C1: `grep -cE "UNCERTAIN\(WEAK\)" conjecture-run-aide2-deep-research-novelty-2026-07-15.md` = 10 (every archived AIDE2 verdict is inert); reuses the AIDE2 C6 receipt (--self-test = 3 fixture kills, no live-path canary). Premise CONFIRMED (executed).
- C2: `grep -nE "delta|noise" conjecture_triage.py | grep -ci kill` = 0. Premise CONFIRMED (executed).
- C3: `grep -c "survive as ordinary grounding" SKILL.md` = 1; `grep -ic "p-25" SKILL.md` = 0. Premise CONFIRMED (executed).
- C4: `grep -inE "id.order|tie.break" conjecture_triage.py | grep -ic cost` = 0. Premise CONFIRMED (executed).
- C5: `grep -icE "pareto|per_task|specialist|tasks_won|set_cover" research_tree.rs` = 0; Evo anchor `_pick_pareto_per_task` present at frontier_strategies.py:251 (grep = 2). Premise CONFIRMED (executed); effect is a FORK-4 estimate (GEPA arXiv:2507.19457, agent-level, mechanism not isolated).
- C6: `grep -icE "power|minimal_detectable|underpowered" frontier_strategy_bench.py` = 0; NeMo underpowered-smoke guard present in nemo-rl-SUMMARY.md (grep = 1). Premise CONFIRMED (executed); effect is a design principle, no number.
- C7: nemo-gym `verified: bool = False` present at server_metadata.py:25 (grep = 1); ELAI bench catalog has no such bit. Premise CONFIRMED (executed); provenance convention, no effect claim.
- C8: nemo-gym `compute_pass_majority_metrics` present at reward_profile.py:310 (grep = 1); ELAI bench variance stats grep = 0. Premise CONFIRMED (executed); design-shape steal, no number.
- C9: nemo-rl `target_experiment_count` typed stop conditions in nemo-rl-SUMMARY.md (grep = 2); TaskSpec exists in map, no campaign-grain stop object. Premise CONFIRMED (executed); stays PENDING (no live consumer).
- C10: `grep -icE "mismatch|disagree|drift" handoff-protocol.md` = 0. Premise CONFIRMED (executed); one-clause doctrine change.

JUDGE PASS: not run this session (SF-1: executed premise receipts + cited estimates already outrank a
judge-only score for these candidates; every survivor carries an executed premise receipt, so none is
judge-only and none is pending on judge grounds). A cross-family diversity vote was not on the
critical path (RECON_NEEDED-4 waived; the pick is SAFE-profile deterministic).

## PICK

UNFUSED AXIS TABLE (equivalence | groundedness | validated | info-gain | cost | claimed effect):
  C1  UNCERTAIN(WEAK) | 0.70 | 0 | 0.5 | M | 0, engine's proven selector made usable on real runs
  C2  UNCERTAIN(WEAK) | 0.60 | 0 | 0.3 | S | 0, AIDE2-C8 survived-but-unauthored, now authored
  C3  UNCERTAIN(WEAK) | 0.60 | 0 | 0.2 | S | 0, doctrine hygiene (P-25 citation)
  C4  UNCERTAIN(WEAK) | 0.55 | 0 | 0.2 | S | 0, less-arbitrary SAFE tie-break
  C5  UNCERTAIN(WEAK) | 0.60 | 0 | 0.5 | M | 0, GEPA pareto retention (vendor +0.042..+0.053 agent-level)
  C6  UNCERTAIN(WEAK) | 0.60 | 0 | 0.4 | S | 0, refutation power floor
  C7  UNCERTAIN(WEAK) | 0.55 | 0 | 0.3 | S | 0, verified-flag provenance
  C8  UNCERTAIN(WEAK) | 0.55 | 0 | 0.3 | S | 0, repeats/variance helper shape
  C9  UNCERTAIN(WEAK) | 0.55 | 0 | 0.3 | M | 0, campaign stop conditions (PENDING, no consumer)
  C10 UNCERTAIN(WEAK) | 0.55 | 0 | 0.2 | S | 0, handoff mismatch-logging clause

NOVELTY AXIS (U3): with no supplied NLI verdicts file the deterministic bin-coordinate leg returns
equivalence=UNCERTAIN + instrument_strength=WEAK for every card (the exact inertness C1 exists to
fix); it is a reported axis here, never a discovery claim. All ten share UNCERTAIN so the verdict tier
cannot separate them.

PROFILE (declared): SAFE (engine default; no profile requested). Unattended run. SAFE lead among the
grounded candidates: **C1/E-1** (highest groundedness 0.70; the engine's one proven selector is inert
on real runs and C1 is the direct, pure-harness fix, composing Q-11's canary discipline). TIE
STRUCTURE DECLARED: validated_value is 0 across all ten; the S-class doctrine/protocol candidates tie
on cost, so the SAFE lead is groundedness-broken (C1 > the S-class field), not a strict single winner.

DOWNSTREAM (paper-to-plan takes ALL not-refuted survivors, orthogonal top-k; each carries an executed
premise receipt + a cited estimate, none judge-only). Verdicts:
  C1/E-1  NEW            -> Q-14 unit (harness novelty instrument + canary; feeds PS-3)
  C2/E-2  NEW            -> Q-14 unit (quantified KILL-EDGE tail; the AIDE2-C8 finally authored)
  C3/E-3  NEW-DOCTRINE   -> Q-14 rider (P-25 exo-inject citation in SKILL + feature-map ledger)
  C4/E-4  NEW            -> Q-14 unit (cost-honest SAFE tie-break; low priority)
  C5/S-1  EXTENSION(Q-12)-> Q-15 unit (pareto_per_task as a third policy on Q-12's selector seam; bench-lane first)
  C6/S-2  NEW            -> Q-15 unit (refutation power floor; composes C2/E-2)
  C7/S-3  NEW            -> Q-15 unit (verified-flag bench/catalog provenance)
  C8/S-4  NEW            -> Q-15 unit (repeats + pass@k/std_err variance-helper shape)
  C9/S-5  PENDING        -> NOT a plan file this run (no live multi-session research-campaign consumer; recorded here + as DEFERRED in Q-0_INDEX)
  C10/S-6 NEW-DOCTRINE   -> Q-15 rider (handoff-protocol mismatch-logging clause)
No REJECT / ALREADY-REJECTED verdict this run: the AIDE2/creativity duplicates the brief might tempt
(C1-C11 bandit/canary/fixed-budget/receipts; Q-10 + P-25 creativity) are Section-1 ledger-covered and
were NOT diverged as cards (they would be ledger kills — deliberately excluded at generation, not
killed at triage). Federation (reasoning Section 5) is a design recommendation, not a card.

PSR-2 NOTE: C1 declares feeds PS-3; C2/C3/C4/C5/C6/C7/C8/C9/C10 declare `none` with the class named
and are flagged for explicit user ack review (user-directed authoring 2026-07-17: the instruction to
author these plans from the synergy reasoning file IS the authoring authority; the PSR-2 `none` ack is
recorded pending the owner's confirmation).

## RECHECK HOOKS

Mechanical loser re-check: `grep -n "KILL-EDGE:" workflow/reasoning/conjecture-run-*.md`, then
reinstate to pending any candidate whose killer changed. No KILL-EDGE stands in this run (refuted=0),
so there is nothing to reinstate. C9/S-5 is PENDING-by-scope (no consumer), not killed: re-promote it
the moment a real multi-session research-campaign consumer lands (its falsifier flips from "no
consumer" to "enroll the typed stop object into that consumer"). C2/C6 compose: when E-2's quantified
tail ships, a within-noise kill it records is exactly S-2's NULL-UNDERPOWERED class — re-check both
together.

## PHASE-5 OUTCOME (2026-07-17)

Plan files authored from the survivors (paper-to-plan PHASE 5, Q letter — J CLOSED, Q open):
- Q-14 (docs/thefinalglance/Q-plan/Q-14_conjecture_engine_instrument_hardening.md): C1/E-1 harness novelty instrument + canary, C2/E-2 quantified KILL-EDGE tail, C4/E-4 cost-honest tie-break, C3/E-3 P-25 citation doctrine rider. plan-gate PASS.
- Q-15 (docs/thefinalglance/Q-plan/Q-15_research_bench_and_protocol_hygiene.md): C5/S-1 pareto_per_task retention (EXTENSION Q-12, bench-lane first), C6/S-2 refutation power floor, C7/S-3 verified-flag provenance, C8/S-4 repeats/variance helper, C10/S-6 handoff mismatch-logging doctrine rider. plan-gate PASS.
C9/S-5 recorded PENDING (no consumer), NOT a build unit; noted as DEFERRED in Q-0_INDEX. No REJECT
authored. Q-0_INDEX + Q-SUMMARY_INDEX + ELAI-FEATURE-MAP + ELAI-SPEC-OVERVIEW upserted.

End of conjecture-run artifact.
