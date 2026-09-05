# I-22: progress-shaped rewards

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever ANTI-GAMING | grounding STRONG | nature BUILD | effort M | WAVE 4 | depends I-4 | ID-4
- Premise: absolute-score rewards on trajectories inflate when a branch can earn points by producing plausible-looking
  output without actually advancing state. i-22 replaces absolute scoring with DELTA progress (phi(h_t) - phi(h_{t-1})
  where phi is computed from deterministic observables only: tests passing, compile errors falling, requirements
  covered). first-error localization truncates the branch at the failing step and salvages the valid prefix. reward
  decay on no-progress loops provides the search-native analog of the anti-spiral skill. all three depend on i-4's
  search ladder existing (the PUCT value function consumes the shaped reward). ID-4 binds: phi components are
  deterministic observables enumerable from the journal; an LLM-opinion phi component is structurally absent.

## Steal/build
DELTA PROGRESS SCORING: phi(h_t) is a weighted sum of deterministic observables at step t: test-pass count,
compile-error count (inverted), requirements-covered count (from the i-37 EARS surface if available, else wiring
contract fields). the shaped reward for step t is phi(h_t) - phi(h_{t-1}). a branch that holds position earns
zero reward; a branch that improves earns proportionally; a branch that regresses earns negative reward. the weights
are ELAI-measured on the fixture corpus (ID-5 — no paper weight vector is wired as a gate).
ID-4 COMPLIANCE: every phi component must be a deterministic observable enumerable from the journal. the ProgressRewardScored
SSB row carries the component breakdown so the review is reproducible under ELAI_SEED (GD-2). any weight assigned
to an LLM opinion, a self-report, or a non-deterministic signal is a BLOCK.
FIRST-ERROR TRUNCATION: on a branch failing at step k (the first TriLayerStack HardVeto), the branch is truncated
to step k-1 (the last passing state) and that prefix is preserved as a fork node in the H-49 undo-tree. the
generator is then reinvested in the preserved prefix rather than the failing branch root. this is the "salvage the
valid prefix" mechanic — cheaper than discarding the whole branch.
REWARD DECAY ON NO-PROGRESS LOOPS: if phi(h_t) = phi(h_{t-1}) for D consecutive steps (the no-progress window,
a MEASURED parameter on ELAI's fixture set, not a paper constant), the reward for the next step is multiplied by a
decay factor (<1). accumulated decay breaks isomorphic-state spirals in the search tree. the decay schedule is
linear-step until ELAI-measured behavior warrants a different curve.
ANTI-SPIRAL COMPLEMENT: this is the search-native sibling of the anti-spiral CLI skill — it does NOT replace the
skill (the skill fires on the main session; i-22 fires on per-branch search trajectories). the two compose without
interaction.
EXCLUDED: any phi component involving an LLM opinion or self-report (ID-4); any reward function that can be gamed
by producing plausible-looking output without detectable state change; paper-derived weight vectors as hard gates.

## Existing ground truth
- D-2 PpmClient seam (L2 scorer this shapes) — SHIPPED; i-22 SHAPES the L2 input, not the PPM output (GD-1 do NOT re-implement PPM)
- H-49 undo-tree (first-error truncation fork node) — SHIPPED per H-plan Track 8; i-22 creates fork nodes via the existing H-49 API (GD-1)
- i-4 search ladder — depends (must exist for the PUCT value function to consume shaped rewards; i-22 is WAVE 4, after i-4)
- anti-spiral skill (CLI side) — SHIPPED; i-22 is the search-native sibling, explicitly does NOT replace it
- IS-7: conformal_calibration.rs — the calibration surface; the uncertainty fork (high epistemic uncertainty -> exploration bias) is handled by i-23; i-22 shapes the reward, i-23 shapes the prior; they compose at the PUCT value function
- TriLayerStack (i-17) — the first-error truncation trigger is a HardVeto from i-17's L1; i-22 listens to i-17 output
- "PLAN-asserted; executor re-greps per W-7": D-2 PpmClient exact file:line; H-49 fork-node API; the PUCT value function seam in i-4

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- search/score.rs in the i-4 module: delta_progress scorer computing phi(h_t) - phi(h_{t-1}) from SSB observables
- phi components: test-pass count, compile-error count (inverted), requirements-covered count (i-37 field or wiring contract fallback)
- first-error truncation: on HardVeto at step k, fork H-49 at step k-1 and reinvest in the prefix
- no-progress decay: decay multiplier applied after D consecutive zero-delta steps (D measured from fixture corpus)
- ELAI-measured weight baseline: first-deploy measurement writes metrics/progress_reward_weights.toml (ID-5 compliance)
- ProgressRewardScored SSB action_type (Kind: Delta / FirstErrorTruncated / DecayApplied)
DEFER (Tier-2 / gated):
- requirements-covered phi component from i-37 EARS: scaffolded as a zero-contribution stub until i-37 ships, then wired; the stub is REGISTERED per stub-tracking protocol with will_resolve_in=i-37
- non-linear decay schedules (exponential, adaptive): deferred until ELAI-measured evidence warrants; linear ships first

## Integration
new module: crates/orchestrator/src/search/score.rs (the shaped reward scorer, consumed by the i-4 PUCT value
function and by the ladder ranker as a branch quality signal). modify: i-4 search/ladder.rs (wire in score.rs as
the value function at the PUCT expansion seam — PLAN-asserted; executor confirms the exact API at EXECUTING, W-7).
modify: the H-49 undo-tree fork API (add a FirstErrorTruncation fork event type if not already present —
PLAN-asserted; executor re-greps at EXECUTING).
SSB action_type: +ProgressRewardScored (Kind: Delta / FirstErrorTruncated / DecayApplied) — baseline+1.
frontend_pair: progress sparkline per branch on the branch board (i-4 surface). the sparkline shows phi over steps
with decay regions highlighted. cross_lang_diff PASS required (W-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/search/score.rs::DeltaProgressScorer (phi computation + shaped reward + decay)
  - crates/orchestrator/src/search/score.rs::FirstErrorTruncator (fork-H-49-at-k-1 logic)
  - ProgressRewardScored SSB action_type with Delta / FirstErrorTruncated / DecayApplied discriminators
  - metrics/progress_reward_weights.toml (ID-5 baseline weight measurement artifact)
consumes_symbols:
  - D-2 PpmClient (why: L2 scorer that receives the shaped reward as input; GD-1 do NOT re-implement PPM)
  - H-49 undo-tree fork API (why: FirstErrorTruncation creates a fork node; GD-1 do NOT re-implement the undo-tree)
  - i-17 TriLayerStack::run_verdict (why: HardVeto at step k triggers truncation; GD-1)
  - i-4 search/ladder.rs PUCT value function (why: the shaped reward is the value; compose, do not inline)
  - i-37 EARS requirement objects (why: requirements-covered phi component — REGISTERED STUB until i-37 ships; stub_ban exception: registered stub with will_resolve_in=i-37)
reachable_from:
  - crates/orchestrator/src/search/ladder.rs PUCT expansion loop (the value function call — the production entrypoint that triggers scoring per branch step)  [PRODUCTION search ladder PUCT]
  (contracts/ callers never satisfy R7 — production reachability is via the PUCT expansion; executor re-traces at EXECUTING)
stub_ban:
  - NO LLM-opinion phi component: every element of phi must be a deterministic observable from the journal (ID-4); an LLM self-assessment score in phi is a BLOCK
  - NO absolute-score reward replacing the delta (the entire point is the delta; an absolute-score fallback path defeats the gaming resistance)
  - NO paper-derived weight vector as a hard gate (ID-5): weights come from metrics/progress_reward_weights.toml populated by the first-deploy measurement
  - NO unbounded no-progress window D: the window parameter must come from the measurement fixture output, not a hard-coded number in source
  - NO .unwrap()/.expect() on production paths; NO fabricated phi measurements; NO blake2/sha2
frontend_pair:
  progress sparkline per branch on the i-4 branch board. sparkline shows phi(h_t) values over steps, with decay
  regions in a different color. Tauri IPC: ProgressRewardScored SSB rows (Delta kind) carry per-step phi values;
  the sparkline renders from those rows via the existing SSB subscription. cross_lang_diff PASS required (W-4).
  Justified — the branch board is the natural surface; no new drawer.
observable_effect:
  trigger: "elai run --search --task-fixture fixtures/loop_task.toml --emit-ssb (fixture designed to produce an isomorphic-state loop)"
  expect_ssb: "ProgressRewardScored(Delta) rows for each step with phi component breakdown (test_pass, compile_err_inv, req_covered), all deterministic observables. ProgressRewardScored(DecayApplied) after D consecutive zero-delta steps on the loop fixture. ProgressRewardScored(FirstErrorTruncated) when a branch fails at step k, with the H-49 fork node id at step k-1 visible in the journal."
  expect_artifact: "a branch that loops on isomorphic states sees reward decay and the search backtracks to a sibling (journal shows sibling expansion triggered by DecayApplied threshold). phi components are all derivable from the journal without any model call (ID-4 compliance). an --llm-phi broken arm (adds a model self-assessment component to phi) is caught by the contract test asserting phi components are fully deterministic — FAILS (R2/ID-4)."

## Grounding
- D-2 PpmClient seam: PLAN-asserted; executor re-greps at EXECUTING (W-7)
- H-49 undo-tree: SHIPPED per H-plan Track 8 (RV-1..RV-4 + H-49 plan file); fork API PLAN-asserted
- ID-4 (evidence-ranked-search, deterministic-veto-supreme): the spine of the phi-component discipline
- ID-5 (local-baseline-before-paper-threshold): the weight vector and no-progress window D must be measured, not borrowed from literature
- i-4 dependency: i-22 is WAVE 4, after i-4; the PUCT value function is the consume seam
- anti-spiral skill: SHIPPED; explicitly a different surface (CLI vs per-branch search) — does not interact
- Consolidated Part IV + review 1.1 "delta progress scoring + first-error truncation + decay"
- arXiv: reward shaping / potential-based shaping (Ng et al. 1999) is the mathematical basis for delta-progress rewards; no arXiv ID required for this classical result
- SSB: +ProgressRewardScored (Kind: Delta/FirstErrorTruncated/DecayApplied) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
D-2 PpmClient (the L2 scorer that receives the shaped reward — do NOT re-implement PPM or its scoring logic); H-49
undo-tree (FirstErrorTruncation fork — do NOT re-implement the undo-tree); i-4 search ladder PUCT (the consumer
of the value function — do NOT inline scoring into the ladder); i-17 TriLayerStack (the HardVeto trigger — do NOT
duplicate the gate logic); anti-spiral skill (the CLI-side cousin — no interaction, no replacement). the delta
scorer, truncator, and decay logic are net-new in search/score.rs.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule) — effort M.

End of I-22.
