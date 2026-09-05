CONJECTURE RUN — aide2-deep-research-novelty — 2026-07-15

ARTIFACT_VERSION: 1.1

RUN TYPE: standalone /conjecture invocation by the user (explicit engine call), exogenous artifact
logged. Generation = ONE direct self-coordinating call (v1.1 live path; no stance fan-out, no
multi-cycle operators — B-3 refuted, mechanics honored).

## TRIGGER

Verbatim input: "use the improvement-conjecture engine on
C:\Users\user\Downloads\AIDE2_recursive_self_improvement_reconstruction_dossier.md ... targeted to
improve elai deep research system.. + the novelty engine etc"
FORK-1 routing: the input is a LOCAL reconstruction dossier (not a paper/repo link) + a named target
area ("deep research system + novelty engine") -> standalone /conjecture route taken, with the
dossier logged as the exogenous artifact. paper-to-plan seam NOT fired (no plan authoring in this
run; the engine never authors plan files).
FORK-2 anchor oracle: `python .claude/tools/elai_map_freshness.py` -> STALE (map 15.0h older than
newest source crates/orchestrator/src/github_pr/mod.rs). Regenerated per header command
(codebase_mapper.py, background, exit 0) BEFORE stage-2 anchor checks. Fresh elai.map used as the
anchor oracle; no codegraph substitution needed.
EXOGENOUS ARTIFACT (logged): AIDE2 reconstruction dossier, 2553 lines, prepared 2026-07-15 over the
Weco AIDE2 announcement (2026-07-14) + public aideml/weco-cli repos. Headline claims
WebSearch-VERIFIED this session (2026-07-15): the announcement exists at
https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement and independently states the
16x prompt reduction, the 63%->34% reward-hack-rate drop, 100 outer steps / 8 days / 7 accepted
improvements. RELIABILITY CEILING: all AIDE_85 effect numbers are VENDOR-REPORTED (technical report
not yet released, no peer review, mechanisms not individually ablated). Every claimed_effect below
inherits that ceiling and is speculative until ELAI-measured.
Ledger source: docs/thefinalglance/ELAI-FEATURE-MAP.md Section 4 read once pre-generation for
grounding (first-call grounding read, precedent run 2026-07-11; single-shot, no later generation
steps). Typed inputs generation received: (1) the verbatim trigger, (2) the bin/region target
matrix below, (3) the logged exogenous dossier. DECLARED.

## BIN MATRIX

Families: gate/checker (F1), memory/reuse (F2), routing/economics (F3), prompt/context-layout (F4),
decoding/inference (F5), measurement/bench (F6), doctrine/process (F7), integration/seam (F8).
Areas (real anchors, grep-verified against fresh elai.map before generation):
  A1 research-tree-search       crates/orchestrator/src/research_tree.rs (ResearchTree, ResearchNode,
                                info_gain, ConvergenceStatus, phi/psi)
  A2 cosmos-research-daemon     crates/knowledge/src/cosmos_worker.rs (CosmosWorker, CosmosStage,
                                briefing compiler; O-1 generalizes)
  A3 novelty-classifier         crates/orchestrator/src/novelty.rs (NoveltyClass, NoveltyClassifiedEmitter,
                                DARK ELAI_NOVELTY_LABEL_ENABLE; D-15, N-10 NoveltyScore consumer)
  A4 research-presets-critic    crates/reasoning/src/chat_reflect.rs + research_presets config (B-40)
  A5 explore-lane-plan          L-23/L-24 (PLAN-asserted; real neighbors: anti_spiral, ssb.rs;
                                conjecture_store.rs greps 0 = not yet shipped)
  A6 harness-novelty-instrument .claude/tools/conjecture_triage.py (--fold/--novelty leg) +
                                workflow/reasoning/frontier-archive-index.json; RUNTIME co-anchors
                                novelty.rs / ssb.rs (elai.map covers .elai_cc only, so harness-file
                                paths cannot satisfy the anchor probe — runtime siblings anchor
                                these cards; the harness leg rides in prose)
  A7 web-research-orchestrator  crates/orchestrator/src/web_research_orchestrator.rs
                                (DARK ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE)
  A8 explore-bench              .claude/tools/frontier_strategy_bench.py (Q-10 Tier-0 instrument,
                                BUILT+AUDITED) + L-10 explore arm (PLAN-asserted); runtime co-anchor
                                usage_turn_event (J-28)

|    | A1 | A2  | A3 | A4 | A5 | A6 | A7 | A8  |
|----|----|-----|----|----|----|----|----|-----|
| F1 |    | C11 |    |    |    | C6 |    |     |
| F2 |    |     |    |    |    | C8 |    |     |
| F3 | C1 |     |    |    |    |    |    | C10 |
| F4 |    |     |    | C4 |    |    | C3 |     |
| F5 | EMPTY-ATTEMPTED (see below) |||||||
| F6 |    |     | C5 |    |    |    |    | C7  |
| F7 | C2 | C9  |    |    |    |    |    |     |
| F8 |    |     |    |    | EMPTY-ATTEMPTED (see below) ||||

EMPTY-ATTEMPTED (recorded, not padded — FORK-3):
- (F5 decoding/inference x A1/A4): the dossier's decoding-adjacent lever is diverse drafting; ELAI's
  L-22 Verbalized-Sampling propose contract (arXiv:2510.01171, planned) already owns
  distribution-call candidate diversity — any candidate here would be a near-duplicate of L-22;
  none emitted.
- (F8 integration/seam x A5): the obvious seam — feed the harness conjecture-archive's novelty
  verdicts into the L-24 runtime store (or vice versa) — is a BANNED boundary (skill L-24 sibling
  wall: "a bridge between them is a new plan, never a silent extension"; Q-10 B-CREATIVITY-0 ladder
  HALTED 2026-07-14). Recorded as boundary-empty, not attempted around.
- (F2 memory/reuse x A4, considered not emitted): Pareto/checkpoint retention of research presets
  (dossier §17.4 non-monotonic checkpoints) — near-duplicate of the H DGM-verification
  worst-case + lineage-diverse promotion scoring + the D-plan variant archive; not emitted.

Distinct filled bins: 11 (F1xA2, F1xA6, F2xA6, F3xA1, F3xA8, F4xA4, F4xA7, F6xA3, F6xA8, F7xA1,
F7xA2). Candidates: 11 (C1-C11). Exceeds floor (>=6 bins, >=5 candidates).

## CANDIDATES

Single-shot generation call id: aide2-ds1-2026-07-15. CARD REVISION NOTE (2026-07-15, same
session): the cards below are revision 2 of the SAME 11 mechanisms emitted by the single-shot call.
Revision 1's field discipline was faulty (prose words inside target_units; paragraph-length
expected_effect fields whose incidental vocabulary lexically collided with unrelated ledger rows)
and triage run 1 killed all 11 on those input errors — see STAGE-2 KILLS, RUN 1, where every run-1
kill is preserved verbatim. Mechanisms, bins, operators, and parents are UNCHANGED; only field
discipline was corrected. The triage tool was not modified.

### CANDIDATE C1
- id: C1
- mechanism_family: routing/economics
- target_area: research-tree-search
- target_units: crates/orchestrator/src/research_tree.rs, ResearchTree, ResearchNode, info_gain
- expected_effect: bandit-allocation of next-iteration spend across sub-question-subtrees
  (arms=draft-lineages, reward=normalized-info_gain), greedy-within-subtree; harness-owned
  arm-selection on the FlashResearch lane.
- cost_class: M
- falsifier: (runnable) `grep -icE "bandit|ucb|arm_" crates/orchestrator/src/research_tree.rs`;
  pass-if 0 production hits (nonzero = premise-false kill: an allocation policy already exists).
  Effect leg FORK-4 (needs a build): paper-grounded estimate — dossier §12 REPORTED accepted
  mechanism; §17.1 quoted transfer "+0.053, p=0.0024" / "+0.042, p=0.0041" vs AIDE_0 (vendor,
  agent-level, mechanism not isolated).
- bin: routing/economics x research-tree-search
- stance: Architect
- typicality: 0.5
- proof_spine: none (research-lane class; needs user ack at plan-authoring)
- external_claims: lineage-bandit accepted in AIDE_85 — WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement (2026-07-15); exact
  bandit formula UNDISCLOSED (dossier §12.5 — never attribute UCB1 to Weco).
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep returns 0; (2) `grep -c reward research_tree.rs` returns 0 (no
  arm statistics exist); (3) breadth_factor/depth_limit caps exist (grep nonzero).
- groundedness: 0.6
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: +0.042..+0.053 vendor-reported agent-level transfer delta (mechanism not
  isolated); more converged research per fixed spend, SPECULATIVE for ELAI

### CANDIDATE C2
- id: C2
- mechanism_family: doctrine/process
- target_area: research-tree-search
- target_units: crates/orchestrator/src/research_tree.rs, ConvergenceStatus, ResearchNode
- expected_effect: plateau-triggered strategy-space re-root: a stalled subtree (last-4 info_gain
  flat by slope+span) spawns a fresh-angle sub-question stream seeded from best-so-far findings,
  as a typed journaled event; today's stall exits are prune-class terminals.
- cost_class: M
- falsifier: (runnable) `grep -icE "re_root|reroot|plateau" crates/orchestrator/src/research_tree.rs`;
  pass-if 0 production hits (nonzero = premise-false kill). Effect leg FORK-4: dossier §12.3-12.4
  REPORTED accepted mechanism (re-root-on-stall, their naming differs); stall criteria are
  dossier-RECOMMENDED (§34), not REPORTED.
- bin: doctrine/process x research-tree-search
- stance: Hacker
- typicality: 0.45
- proof_spine: none (research-lane class; needs user ack at plan-authoring)
- external_claims: re-root-on-stall REPORTED as an AIDE_85 discovered mechanism — dossier §12.3,
  announcement WebSearch-verified https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep 0; (2) no last-4 stall signal exists in research_tree.rs today.
- groundedness: 0.6
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: 0 isolated number (vendor rationale: avoids greedy lock-in AND wasteful restart)

### CANDIDATE C3
- id: C3
- mechanism_family: prompt/context-layout
- target_area: web-research-orchestrator
- target_units: crates/orchestrator/src/web_research_orchestrator.rs, crates/orchestrator/src/research_tree.rs
- expected_effect: per-operator minimal-prompt matrix for FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE:
  newest-12 one-line finding-summaries + exactly-one full-extract per prompt, per-operator declared
  fields (extends the J declared-access-spec pattern); journaled prompt-size ratio; savings
  reinvested as extra tree-iterations.
- cost_class: M
- falsifier: (runnable) `grep -icE "one_full|max_summaries|summary_cap|newest" crates/orchestrator/src/web_research_orchestrator.rs`;
  pass-if 0 (no one-full-extract/summary-cap invariant exists today). Effect leg FORK-4: dossier §13
  quoted "reduced average full-prompt size by approximately 16x ... saved tokens were reinvested
  into more search steps under the fixed budget" (vendor).
- bin: prompt/context-layout x web-research-orchestrator
- stance: Simplifier
- typicality: 0.5
- proof_spine: feeds: PS-4 (research-lane prompt-size reduction lowers the $/verified-task input on
  the hybrid routing bench)
- external_claims: 16x prompt reduction REPORTED — dossier §13, WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement; the full
  operator-to-field matrix is NOT published (§13.2), the mapping here is dossier-INFERRED.
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep 0; (2) research prompts concatenate multi-node history without a
  cap constant today.
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: ~16x avg prompt-size cut on the research lane (vendor number; ELAI already has 3E
  slots so realized delta will be smaller; SPECULATIVE)

### CANDIDATE C4
- id: C4
- mechanism_family: prompt/context-layout
- target_area: research-presets-critic
- target_units: crates/reasoning/src/chat_reflect.rs, research_presets
- expected_effect: health-gated failure-history inclusion: critic-round failure summaries enter
  REFLECT/critic prompts solely-when recent-failure-rate exceeds 15% over a last-20 window
  (named-constant thresholds); healthy runs omit them.
- cost_class: S
- falsifier: (runnable) `grep -icE "bug_rate|failure_rate|recent_fail" crates/reasoning/src/chat_reflect.rs crates/orchestrator/src/research_tree.rs`;
  pass-if 0 (no health-gated inclusion exists today). Effect leg: dossier §13.2 REPORTED
  ("included only when the recent bug rate exceeds 15%"); window size dossier-INFERRED (§32).
- bin: prompt/context-layout x research-presets-critic
- stance: Hacker
- typicality: 0.35
- proof_spine: none (prompt-economy class; needs user ack at plan-authoring)
- external_claims: 15% bug-rate gate REPORTED — dossier §13.2, announcement WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep 0; (2) the B-40 critic loop caps rounds at 3 but carries
  prior-round content unconditionally.
- groundedness: 0.5
- validated_value: 0
- information_gain: 0.3 (advisory)
- claimed_effect: 0 isolated number (vendor rationale: failure-oriented prompt spend scales with run health)

### CANDIDATE C5
- id: C5
- mechanism_family: measurement/bench
- target_area: novelty-classifier
- target_units: crates/orchestrator/src/novelty.rs, NoveltyClass, NoveltyClassifiedEmitter
- expected_effect: novelty-survival rate: DiscoveryCandidate / DISTINCT verdicts are re-judged
  later against the UPDATED archive by an independent judge; a verdict losing at least half its
  claimed distinctness counts as novelty-hacking; the rate lands as a bench column (runtime
  D-15/N-10 surface; harness sibling = the L-10 explore-arm novelty column).
- cost_class: M
- falsifier: (runnable, two legs) (a) `python .claude/tools/conjecture_triage.py --novelty <this
  artifact> --json` with no --verdicts; pass-if emitted verdicts carry equivalence=UNCERTAIN +
  instrument_strength=WEAK and NO survival/re-judge field exists in the schema. (b)
  `grep -icE "survival|re_verify|reverify" crates/orchestrator/src/novelty.rs`; pass-if 0.
- bin: measurement/bench x novelty-classifier
- stance: Researcher
- typicality: 0.4
- proof_spine: feeds: PS-3 (hardens the L-10 explore-arm novelty column the hallucination-delta
  bench family reports)
- external_claims: survival criterion ("less than half of claimed unit-test speedup survives
  end-to-end") + rates 63/42/34/42 REPORTED — dossier §16, WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: reframe (reward-hacking survival -> novelty-claim survival)
- parents: none
- predictions: (1) leg (a) shows UNCERTAIN+WEAK with no survival field; (2) leg (b) 0; (3)
  NoveltyClass has exactly 3 variants and no re-judge path.
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.7 (advisory)
- claimed_effect: hidden/second-look selection cut gaming 63%->34% in the source system (vendor,
  different domain, SPECULATIVE transfer)

### CANDIDATE C6
- id: C6
- mechanism_family: gate/checker
- target_area: harness-novelty-instrument
- target_units: crates/orchestrator/src/novelty.rs, NoveltyClassifiedEmitter
- expected_effect: known-answer canary pairs (one known-SAME, one known-DISTINCT) executed through
  the LIVE novelty path on every invocation — runtime leg: fixture templates through the
  NoveltyClass classify+emit path each bench run; harness leg (prose, elai.map cannot anchor
  harness paths): the same pair through conjecture_triage.py --novelty fold+verdict code; a canary
  miss forces instrument_strength=FAILED and suppresses all verdicts for that run.
- cost_class: S
- falsifier: (runnable) `python .claude/tools/conjecture_triage.py --self-test --json`; pass-if the
  self-test executes NO live-path novelty canary (a present live-path canary = premise-false kill).
- bin: gate/checker x harness-novelty-instrument
- stance: Contrarian
- typicality: 0.3
- proof_spine: none (instrument-discipline class; needs user ack at plan-authoring)
- external_claims: silently-inactive safeguard incident REPORTED (their statistical layer was
  broken in AIDE_85 and "had no impact") — dossier §15, WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier shows self-test lacks live-path novelty canaries; (2) --novelty with
  no judge already degrades honestly to UNCERTAIN+WEAK (existing honest-degrade leg confirmed).
- groundedness: 0.65
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: 0 isolated number (incident-grounded: a discrimination-dead novelty instrument
  fails loud instead of emitting dead verdicts)

### CANDIDATE C7
- id: C7
- mechanism_family: measurement/bench
- target_area: explore-bench
- target_units: crates/orchestrator/src/novelty.rs, usage_turn_event
- expected_effect: paired-repeat noise-band for explore/novelty benches: repeat runs quantify
  run-to-run noise; any delta inside the band renders as TIE in every verdict sentence; extends
  the I-plan QDELTA-floor discipline to the explore/novelty bench family (harness sibling
  frontier_strategy_bench.py named in prose — elai.map cannot anchor harness paths).
- cost_class: S
- falsifier: (runnable) `grep -icE "noise|paired|tie" .claude/tools/frontier_strategy_bench.py`;
  pass-if no noise-band/tie logic exists today (a real noise-band hit = premise-false kill —
  honest risk: the B-CREATIVITY-0 prereg v3 meter hardening may already include it).
- bin: measurement/bench x explore-bench
- stance: Researcher
- typicality: 0.55
- proof_spine: feeds: PS-3 (bench-integrity leg of the explore-arm measurement)
- external_claims: run-to-run noise 0.02-0.045 REPORTED with within-noise deltas treated as coin
  flips — dossier §19, WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep recorded either way; (2) if a noise leg exists it is arm-local,
  not a declared band applied to verdict sentences.
- groundedness: 0.5
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: 0.02-0.045 vendor noise band; within-noise promotions stop (ELAI band TBD)

### CANDIDATE C8
- id: C8
- mechanism_family: memory/reuse
- target_area: harness-novelty-instrument
- target_units: crates/orchestrator/src/ssb.rs
- expected_effect: quantified refutation-records: KILL-EDGE entries (harness) and future
  ConjectureLifecycle criticisms (runtime seam = ssb.rs, L-24 PLAN-asserted) gain an optional
  numeric block {score, delta-vs-incumbent, noise-band}, mirroring the vendor rejected-ideas
  table; a re-proposal near a refuted candidate is then compared against numbers, and a
  within-noise kill is honestly reinstatable.
- cost_class: S
- falsifier: (runnable) `grep -nE "delta|noise" .claude/tools/conjecture_triage.py | grep -ci kill`;
  pass-if 0 (KILL-EDGE grammar carries no numeric field today).
- bin: memory/reuse x harness-novelty-instrument
- stance: Architect
- typicality: 0.35
- proof_spine: none (instrument-discipline class; needs user ack at plan-authoring)
- external_claims: per-rejection numeric table (10 named ideas each with private score + delta +
  noise context) REPORTED — dossier §19, WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier 0; (2) the current recheck hook is killer-ref-change-driven with no
  numeric comparison.
- groundedness: 0.6
- validated_value: 0
- information_gain: 0.3 (advisory)
- claimed_effect: 0 isolated number (table-pattern steal: the anti-repetition boundary becomes quantitative)

### CANDIDATE C9
- id: C9
- mechanism_family: doctrine/process
- target_area: cosmos-research-daemon
- target_units: crates/knowledge/src/cosmos_worker.rs, CosmosWorker, CosmosStage
- expected_effect: hidden-probe split per topic: each Cosmos/O-1 topic keeps a small held-out
  probe-question set that never enters the fetch/summarize path; the compiled briefing is scored
  on probe answer-coverage (harness-computed match first, judge second); topic ranking and
  time-decay consume the HIDDEN score, so a briefing that cannot answer held-out probes decays.
- cost_class: M
- falsifier: (runnable) `grep -icE "probe|held_out|holdout|private_score" crates/knowledge/src/cosmos_worker.rs`;
  pass-if 0 (no hidden-probe scoring exists today). Effect leg FORK-4: dossier §16 (vendor:
  hidden-score selection "naturally removes variants that win through public-score gaming").
- bin: doctrine/process x cosmos-research-daemon
- stance: Researcher
- typicality: 0.45
- proof_spine: none (research-lane class; needs user ack at plan-authoring)
- external_claims: public/private split REPORTED as the core eval design — dossier §8.1,
  WebSearch-verified https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep 0; (2) topic decay today is engagement/time-driven, never
  quality-grounded.
- groundedness: 0.55
- validated_value: 0
- information_gain: 0.7 (advisory)
- claimed_effect: 0 isolated number (vendor rationale: gaming-resistant briefing-quality signal)

### CANDIDATE C10
- id: C10
- mechanism_family: routing/economics
- target_area: explore-bench
- target_units: usage_turn_event, crates/orchestrator/src/novelty.rs
- expected_effect: fixed-dollar-cap novelty reporting + do-not-win control arms: the
  explore/novelty bench reports meaning-cluster coverage and best-verified-outcome per-dollar at a
  locked spend-cap, and carries two named control arms — unbounded best-of-N and
  parallel-widening — whose win flags a spend-leak rather than a promotion (harness sibling
  frontier_strategy_bench.py named in prose).
- cost_class: S
- falsifier: (runnable) `grep -icE "per_dollar|usd|spend_cap" .claude/tools/frontier_strategy_bench.py`;
  pass-if no per-dollar normalization / locked-cap constraint exists today (the Q-10 G3 guard fired
  post-hoc at ~8.8x DIRECT — if it already normalizes per-dollar in-bench, premise-false kill and
  the candidate narrows to the control-arm leg).
- bin: routing/economics x explore-bench
- stance: Architect
- typicality: 0.5
- proof_spine: feeds: PS-4 (spend-normalized selection applied to the explore lane)
- external_claims: fixed-dollar selection + the two named rejected spend-more strategies REPORTED —
  dossier §8.2, WebSearch-verified
  https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep recorded; (2) the G3 guard exists post-hoc, not as an in-bench
  locked-cap constraint with named control arms.
- groundedness: 0.5
- validated_value: 0
- information_gain: 0.5 (advisory)
- claimed_effect: 0 isolated number (vendor design principle: spend-more strategies cannot win the
  novelty bench)

### CANDIDATE C11
- id: C11
- mechanism_family: gate/checker
- target_area: cosmos-research-daemon
- target_units: crates/knowledge/src/cosmos_worker.rs
- expected_effect: receipt-bound briefing lines: every compiled briefing line carries a
  fetched-byte-hash or journal-row-id receipt resolved at compile-time; an unresolvable line is
  dropped and counted (extends the H-52 anchor-resolver + E-48 verified-citation precedents to the
  briefing surface). DUP RISK DECLARED: expected verdict is EXTENSION, or premise-false kill if
  the briefing surface already consumes E-48.
- cost_class: S
- falsifier: (runnable) `grep -icE "receipt|citation|row_id" crates/knowledge/src/cosmos_worker.rs`;
  pass-if 0 (briefing lines are receipt-unbound today; nonzero = premise-false kill).
- bin: gate/checker x cosmos-research-daemon
- stance: Simplifier
- typicality: 0.6
- proof_spine: none (trust class; needs user ack at plan-authoring)
- external_claims: none (the dossier's §31 guard is adjacent; this leg is ELAI-precedent
  composition)
- operator: exo-inject
- frame: trigger-as-given
- parents: none
- predictions: (1) falsifier grep recorded; (2) if 0, the briefing surface is receipt-unbound
  while the chat answer path is receipt-bound (asymmetry confirmed).
- groundedness: 0.5
- validated_value: 0
- information_gain: 0.3 (advisory)
- claimed_effect: 0 isolated number (ELAI-pattern: a fabricated briefing line becomes
  unrepresentable)

BIRTH-EDGE: C1 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C2 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C3 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C4 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C5 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C6 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C7 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C8 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C9 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C10 <- exo-inject:aide2-ds1-2026-07-15
BIRTH-EDGE: C11 <- exo-inject:aide2-ds1-2026-07-15
INJECT: C1 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C2 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C3 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C4 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C5 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C6 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C7 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C8 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C9 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C10 <- dossier:aide2-reconstruction-2026-07-15
INJECT: C11 <- dossier:aide2-reconstruction-2026-07-15

## STAGE-2 KILLS

RUN 1 (2026-07-15, revision-1 cards — preserved history, all reversed by the revision-2 input fix):
All 11 candidates were killed; diagnosis showed INPUT-DISCIPLINE errors, not real duplicates:
prose words inside target_units ("seams", a TOML wildcard, harness paths outside elai.map's
.elai_cc scope) and paragraph-length expected_effect fields whose incidental vocabulary
("model"+"cost", "tokens"+"only", "measured"+"rows") tripped the 2-shared-token lexical matcher
against unrelated ledger rows. Run-1 kill map, verbatim:
  RUN1-KILL: C1 <- ledger:synthetic-local-gpu-cost-model-rejected-for-now (false-positive lexical match)
  RUN1-KILL: C2 <- ledger:deterministic-ocp-lsp-checker-rejected-ocp-future-change (false-positive)
  RUN1-KILL: C3 <- triage:anchor:seams (real input error: prose in target_units) + composition:context-access-spec
  RUN1-KILL: C4 <- ledger:half-stays-rejected-raw-tokens-usd-only (false-positive) + anchor:[research_presets.<class>] (real input error)
  RUN1-KILL: C5 <- ledger:half-stays-rejected-raw-tokens-usd-only (false-positive) + anchor:.claude/tools/conjecture_triage.py (oracle-scope: harness path)
  RUN1-KILL: C6 <- ledger:hard-pretooluse-solid-block-as-a-day-one (false-positive)
  RUN1-KILL: C7 <- ledger:ack-2026-07-10-composes-m-verifier-k (false-positive)
  RUN1-KILL: C8 <- ledger:outdated-narration-of-superseded-memory-rows-rejected-measured (false-positive)
  RUN1-KILL: C9 <- ledger:reads-one-entity-a-deterministic-executor-adjudicates-td (false-positive)
  RUN1-KILL: C10 <- ledger:litellm-proxy-budget-store-as-the-credit-ledger (false-positive)
  RUN1-KILL: C11 <- ledger:executor-self-grading-its-own-output-quality-rejected (false-positive)
The tool was NOT modified. FORK-5 assessed and NOT fired: the run-1 majority-kill was an
input-format artifact, not exhausted territory (no run-1 killer describes the killed mechanism).
TOOL FINDING surfaced for the harness backlog (not fixed in this run): the ledger matcher's
2-shared-token threshold over verbose card prose false-positives heavily; the anchor probe cannot
see harness-side files (elai.map is .elai_cc-scoped by design).

RUN 2 (final, revision-2 cards): candidates=11, refuted=0, flags=0. Artifact --validate: valid=true,
ARTIFACT_VERSION 1.1, 11 BIRTH-EDGE, 11 INJECT, sections_ok. No stage-2 kills stand.

Stage-3 experiment kill (recorded here so status computation sees it; killer-ref = the blake3 of the
receipts file that falsified the premise):

KILL-EDGE: C7 <- experiment:blake3-d73873323019b2decdb79fccf45826e43d7318bbc380b8663d1e9699d564f931

C7 premise ("no noise-band/tie logic exists in the explore bench") is FALSE: frontier_strategy_bench.py
already implements paired non-empty novelty/value deltas with a CI95-lower-bound>0 gate rule
(lines 267/279) — a STRONGER form of within-noise-is-a-tie, landed by the B-CREATIVITY-0 prereg-v3
meter hardening. The runtime L-10 arm inherits that discipline when built; nothing left to propose.

## STAGE-3 RESULTS

MICRO-EXPERIMENTS (executed this session; receipts file stage3-receipts.txt in the session
scratchpad, blake3 = d73873323019b2decdb79fccf45826e43d7318bbc380b8663d1e9699d564f931):
- C1: `grep -icE "bandit|ucb|arm_" research_tree.rs` = 0; `grep -ic reward` = 0;
  `grep -icE "breadth_factor|depth_limit"` = 16. All three predictions MATCHED — no allocation
  policy beyond breadth/depth caps exists. Premise CONFIRMED (executed).
- C2: `grep -icE "re_root|reroot|plateau" research_tree.rs` = 0. Premise CONFIRMED (executed).
- C3: `grep -icE "one_full|max_summaries|summary_cap|newest" web_research_orchestrator.rs` = 0.
  Premise CONFIRMED (executed).
- C4: `grep -icE "bug_rate|failure_rate|recent_fail"` = 0 in both chat_reflect.rs and
  research_tree.rs. Premise CONFIRMED (executed).
- C5: leg (a) --novelty run emitted equivalence=UNCERTAIN + instrument_strength=WEAK for every
  candidate with NO survival field in the verdict schema (receipt: novelty-out.json, scratchpad);
  leg (b) `grep -icE "survival|re_verify|reverify" novelty.rs` = 0. Premise CONFIRMED (executed).
- C6: `conjecture_triage.py --self-test --json` = passed with exactly 3 fixture kills
  (demarcation / ledger / anchor) — NO live-path novelty canary exists. Premise CONFIRMED
  (executed). Prediction 2 also matched: --novelty degrades honestly to UNCERTAIN+WEAK.
- C7: `grep -icE "noise|paired|tie" frontier_strategy_bench.py` = 6; inspection shows a real
  paired-delta CI95 gate. Premise FALSE -> experiment kill above.
- C8: `grep -nE "delta|noise" conjecture_triage.py | grep -ci kill` = 0. Premise CONFIRMED (executed).
- C9: `grep -icE "probe|held_out|holdout|private_score" cosmos_worker.rs` = 0. Premise CONFIRMED
  (executed).
- C10: `grep -icE "per_dollar|usd|spend_cap" frontier_strategy_bench.py` = 0 — the G3 cost guard
  observed in the Q-10 ledger is not a per-dollar in-bench normalization. Premise CONFIRMED (executed).
- C11: `grep -icE "receipt|citation|row_id" cosmos_worker.rs` = 0 — briefing lines are
  receipt-unbound while the chat answer path is receipt-bound. Premise CONFIRMED (executed).

SCOPE NOTE (honest): the executed leg confirms each PREMISE (the gap exists today); every EFFECT
size remains a FORK-4 paper-grounded estimate at vendor-reported reliability (Weco blog 2026-07-14,
WebSearch-verified to exist; technical report unreleased). validated_value stays 0 for all
candidates — no claimed effect was converted (ATTACK-1 honored).

JUDGE PASS: judge-id sonnet-judge-aide2-2026-07-15-v1 (spawned sonnet agent, fresh context,
received cards + receipts only, never generation prompts). SUBSTITUTION RECORDED per
RECON_NEEDED-4: gpt-bridge (cross-family) unavailable this session -> same-family sonnet judge
with the I-24/N-15 audits applied (candidate order randomized: C9,C4,C1,C11,C6,C3,C10,C5,C2,C8;
verbosity-length penalty declared in the judge prompt). Scores (value/credibility, 0-10):
  C1 8/6, C2 7/7, C3 7/7, C4 4/7, C5 6/6, C6 6/8, C8 4/8, C9 7/7, C10 7/6, C11 6/9.
  Judge TOP3 by value: C11, C1, C9.

## PICK

UNFUSED AXIS TABLE (top candidates; equivalence | groundedness | validated | info-gain | cost |
claimed effect | judge v/c):
  C10 UNCERTAIN(WEAK) | 0.50 | 0 | 0.5 | S | vendor design principle, no number      | 7/6
  C11 UNCERTAIN(WEAK) | 0.50 | 0 | 0.3 | S | ELAI-pattern, no number                 | 6/9
  C4  UNCERTAIN(WEAK) | 0.50 | 0 | 0.3 | S | vendor rationale, no number             | 4/7
  C6  UNCERTAIN(WEAK) | 0.65 | 0 | 0.5 | S | incident-grounded, no number            | 6/8
  C8  UNCERTAIN(WEAK) | 0.60 | 0 | 0.3 | S | table-pattern steal, no number          | 4/8
  C1  UNCERTAIN(WEAK) | 0.60 | 0 | 0.5 | M | vendor +0.042..+0.053 agent-level       | 8/6
  C2  UNCERTAIN(WEAK) | 0.60 | 0 | 0.5 | M | vendor rationale, no number             | 7/7
  C3  UNCERTAIN(WEAK) | 0.55 | 0 | 0.5 | M | vendor ~16x prompt cut                  | 7/7
  C5  UNCERTAIN(WEAK) | 0.55 | 0 | 0.7 | M | vendor 63->34 gaming cut (other domain) | 6/6
  C9  UNCERTAIN(WEAK) | 0.55 | 0 | 0.7 | M | vendor rationale, no number             | 7/7

PROFILE (declared): SAFE (engine default; the caller declared no profile). AUTONOMOUS mode — the
profile pick is recorded, INTERACTIVE override available to the user.
SAFE pick: **C10** (fixed-dollar-cap novelty reporting + do-not-win control arms on the explore
bench). Rationale chain: groundedness floor passed -> minimum cost (S) -> validated_value all tied
at 0 -> the tool's deterministic id-order tie-break. TIE STRUCTURE DECLARED HONESTLY: five S-class
candidates (C4, C6, C8, C10, C11) tie at validated_value 0; the id-order tie-break is arbitrary
within that tie, so the alternates below are materially co-equal under SAFE.
NOVEL-CREDIBLE pick (alternate, same axes): C6 (highest groundedness 0.65 among not-refuted; all
equivalence UNCERTAIN so the verdict tier could not separate — the labeled weak fallback broke ties).
Judge value-ordering (advisory, SF-1 lowest tier): C11, C1, C9.

TOP-3 TRADEOFFS:
1. C10 (SAFE winner) — cheapest bench-integrity fix; directly feeds PS-4; risk: narrows if the
   Q-10 cost meter grows per-dollar normalization independently.
2. C6 — the incident-grounded instrument-canary (the dossier's silently-broken-Layer-3 lesson
   applied to ELAI's own novelty leg); highest groundedness in the S tier; protects every FUTURE
   novelty verdict this engine emits; both-profile runner-up and NOVEL-CREDIBLE winner.
3. C1 — the judge's highest-value M-class mechanism (lineage-bandit research allocation, the
   dossier's centerpiece accepted mechanism); needs plan-grade design (reward definition,
   coexistence with breadth/depth caps) before authoring.

DOWNSTREAM (the engine never authors plan files): orthogonal top-k survivors available to
paper-to-plan / idea-intake = {C10, C6, C1} (each independently satisfies anti-laundering: executed
premise receipt + cited estimate; none is judge-only). PSR-2 note: C10 declares feeds PS-4; C6
declares none (instrument-discipline class) and needs explicit user ack if authored; C1 declares
none (research-lane class), same ack requirement. SF-3 REMINDER: this artifact must NEVER be fed to
the auditor that judges any plan authored from these picks.

## RECHECK HOOKS

Mechanical loser re-check: `grep -n "KILL-EDGE:" workflow/reasoning/conjecture-run-*.md`, then
reinstate to pending any candidate whose killer changed (ledger entry removed, anchor now exists,
premise candidate un-refuted, judge version superseded). Premise-false kills recorded here are
reinstatable the moment the named falsifier grep flips in the direction that restores the premise.

## PHASE-5 OUTCOME (2026-07-15)

Plan files authored from the picks (paper-to-plan PHASE 5, Q letter — J CLOSED, Q open):
- Q-11 (docs/thefinalglance/Q-plan/Q-11_novelty_instrument_and_fixed_budget_bench.md): C6 live-path novelty canary + C10 fixed-dollar explore reporting/control arms + pre-change baseline. plan-gate PASS.
- Q-12 (docs/thefinalglance/Q-plan/Q-12_research_lane_context_and_search_policy.md): C3 per-operator minimal-prompt matrix, then C1+C2 lineage-bandit + fork-on-stall as ONE experiment gated on Q-11's bench. plan-gate PASS.
- Q-13 (docs/thefinalglance/Q-plan/Q-13_receipt_bound_briefing.md): C11 receipt-bound Cosmos briefing lines. plan-gate PASS.
C5 and C9 recorded DEFERRED-conditional in Q-11's coda (not build units). No REJECT authored (C7 died in-engine, pre-plan). Q-0_INDEX + Q-SUMMARY_INDEX + ELAI-FEATURE-MAP + ELAI-SPEC-OVERVIEW upserted.
