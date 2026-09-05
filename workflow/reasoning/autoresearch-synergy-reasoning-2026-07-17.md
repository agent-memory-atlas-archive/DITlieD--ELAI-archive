# Autoresearch synergy reasoning, 2026-07-17

Trigger: /home/user/Downloads/ELAI_Open_Source_Autoresearch_Synergy_Research_Brief.md (the ELAI x
open-source-autoresearch bidirectional study brief). Inputs read in full: the brief; the four fresh
source summaries (docs/.source/nemo-rl-SUMMARY.md @ a3fee461, nemo-gym-SUMMARY.md @ dd41196f,
evo-SUMMARY.md @ f5e7e9b9, aideml-SUMMARY.md @ 5d66a217 + aide2-announcement.md); ELAI-SPEC-OVERVIEW;
the Q-plan indexes (Q-11..Q-13); the improvement-conjecture SKILL.md v1.1 + the 2026-07-15 AIDE2
conjecture-run artifact; the research-coupled-creativity handoffs (Q-10 REFUTED, P-25 Families A+B
REFUTED 2026-07-17); live seams (research_tree.rs 3108 lines phi/psi convergence no bandit,
novelty.rs 806 lines, web_research_orchestrator.rs 1052 lines, UsageTurnEvent at ssb.rs:1522-1531,
frontier_strategy_bench.py = the B-3 synthetic 45-strategy DSL).

This is a reasoning artifact, not a plan file. Any plan authoring goes through paper-to-plan /
plan-standard separately. No code, no index edits, no commits from this session.

DECISION: WHY THE CONJECTURE ENGINE WAS NOT RE-RUN

The engine already processed the AIDE2 trigger on 2026-07-15: eleven candidates, deterministic
triage, SAFE pick C10, survivors {C10, C6, C1}, and paper-to-plan authored Q-11/Q-12/Q-13 from them.
The brief's "known ELAI conjecture context" section IS that run. Re-running the engine on this brief
would feed it a trigger whose ELAI-improvement content is ~80% ledger duplicates (C1-C11 all recorded),
which is the engine's own ABORT-3 exhausted-territory condition. The genuinely new content from this
study is (a) Evo/NeMo-specific mechanisms not in the AIDE2 dossier and (b) bidirectional
upstream-contribution opportunities, which the engine does not model (its scope is ELAI-targeted
improvements). So the engine's prior run is honored as the authority on the AIDE2 axis, and the new
material is reasoned directly below with source receipts. If any candidate below is later taken to
plan authoring, it should go through the engine seam then (paper-to-plan PHASE 4), where the ledger
check will correctly kill the duplicates and pass the new ones.

SECTION 1 — WHAT THE BRIEF ASKS THAT IS ALREADY COVERED

High confidence (index + run artifact just read):

- C1 bandit lineage allocation + C2 fork-on-stall > Q-12 (ONE selectable experiment behind
  default-OFF ELAI_RESEARCH_BANDIT_ENABLE, deterministic-scheduler control arm, kill criterion,
  equal-budget promotion only).
- C3 operator-specific minimal context (newest-12 + one full extract) > Q-12 (per-operator
  minimal-prompt matrix, journaled prompt-size ratio, freed budget reinvested as iterations).
- C4 failure-history-only-above-15% > folded into Q-12's C3 matrix scope (declared fields per
  operator).
- C6 known-SAME/known-DISTINCT live canaries > Q-11 (fail-closes verdict emission,
  instrument_strength=FAILED).
- C10 fixed-dollar-cap + do-not-win control arms > Q-11 (per-dollar-at-locked-cap columns off
  usage_turn_event, spend_leak flag).
- C11 receipt-bound briefing lines > Q-13.
- C7 noise-band/tie handling > correctly refuted in the engine run; ELAI's paired-delta CI95 gate is
  stronger. The AIDE2 announcement itself corroborates this posture: Weco reports most of its ~95
  rejected proposals died within a 0.02-0.045 noise band, i.e. green-under-noise deltas are coin
  flips (aideml-SUMMARY PART B, vendor-reported).
- C5 novelty-survival re-judge + C9 hidden-probe split > deferred-conditional in Q-11's coda with
  named blockers (temporal-novelty semantics; probe-quality story). The brief's Experiment A/H
  overlap these; the blockers still stand.
- The creativity legs the brief might tempt (inject research artifacts into generation, aligned
  triage+revise loops) are CLOSED: Q-10 validly REFUTED (relational packets lost to every control at
  ~8.8x cost), P-25 Family A and B REFUTED 2026-07-17 with verified archive (padding and shuffled
  controls explained the gains; both blew token caps). Any new creativity mechanism needs a new
  falsifier, same-cost controls, and per-call durable checkpoints. Nothing in this study reopens it.

Bottom line: the brief's foundation layer (canaries, fixed-budget, baselines, receipts) is exactly
the Q-11-first sequencing ELAI already locked on 2026-07-15. The correct next build action on this
whole axis is unchanged: execute Q-11, then Q-12/Q-13. The study below adds refinements and new
candidates, not a re-sequencing.

SECTION 2 — NEW STEAL CANDIDATES FOR ELAI (not in Q-11..Q-13, ranked)

S-1. pareto_per_task specialist retention as a research-frontier policy variant. Source: Evo
frontier_strategies.py:251-377, a working port of GEPA candidate selection (per-task best > tie
collection > set-cover dominance prune > sample weighted by tasks-won), honoring per-task metric
direction. ELAI gap: every ELAI frontier/selection surface is aggregate-scalar (research_tree
info_gain greedy; the B-3 bench's baseline portfolio; Q-12's planned bandit arms are
aggregate-reward). Nothing retains a candidate that wins SOME task while losing the aggregate, which
is the brief's "discard intermediate high-performing checkpoints" anti-pattern at selection grain.
Why it composes: Q-12 already builds a selectable-policy seam in research_tree.rs (policy selector,
deterministic control arm, Q-11 bench as apparatus); pareto-per-task is a third policy in that same
seam, benched under the same equal-budget rule. Cost S-M (reference implementation is ~130 lines of
pure ranking logic; the ELAI side is a policy enum arm + per-task score plumbing). Falsifier: on the
Q-11 fixed-budget bench with per-task scores, pareto retention must preserve at least one
later-winning specialist that greedy/bandit prune, at equal spend, beyond noise; if per-task scores
do not exist on the research lane, the candidate reduces to the bench lane only. Honest caveat:
research-lane tasks may not decompose into stable per-task scores the way Evo benchmarks do; the
bench lane (frontier_strategy_bench extensions) is the safe first consumer.

S-2. Refutation power floor: a REFUTED verdict must carry evidence the check had adequate power.
Source: the NeMo auto-research skill's underpowered-smoke guard ("do not conclude a training idea
failed from an underpowered smoke run"; a tiny run validates plumbing only) (nemo-rl-SUMMARY, Deep
Dive A, keep/discard semantics). ELAI gap: the conjecture/bench protocol distinguishes
INVALID-instrument from REFUTED-mechanism (P-25 did this well), but nothing structurally requires a
refutation to record its power: K, seed count, CI width vs the effect size the claim needs. The P-25
chronology itself surfaced this (a 240-call spend was queued with "K=30 has precedent but no power
calculation in the locked plan"). Proposal shape: a required power block in preregistrations and on
REFUTED closeouts (K, paired-sample count, minimal detectable effect at CI95, and whether the
observed CI width could have detected the preregistered effect); a refutation whose CI is too wide
to have detected the claimed effect is recorded NULL-UNDERPOWERED, not REFUTED. Cost S (protocol +
checker line in the bench tooling, e.g. the analysis engine already computes CI95 widths). Falsifier:
re-run the P-25 analysis leg with the power block; it must classify the actual P-25 families
(which had locked caps and real CIs) as REFUTED still, and classify a synthetic half-K run as
NULL-UNDERPOWERED.

S-3. Verified-flag provenance on the bench/canary catalog. Source: NeMo Gym server_metadata.py:25
verified: bool = False + a pre-commit hook stamping every new environment untrusted until baselined
and human-reviewed (nemo-gym-SUMMARY section 15). ELAI gap: ELAI's gate manifest freezes checkers
(R8) but the growing bench/fixture/canary catalog has no per-artifact reviewed/baselined bit; Q-11's
canary proves the novelty instrument live at run time, this flag records which benches/canaries have
ever been human-baselined at all. Cost S (a field + a stamp discipline; composes with the Q-plan
standing-goals unit which re-checks verified invariants on cadence). Falsifier: trivial (mechanism is
a provenance convention, not an effect claim); the check is that the stamp exists and defaults false.

S-4. Repeats + pass@k/std_err threading in the Q-11 bench, stolen as a shape not a dependency.
Source: NeMo Gym reward_profile.py:310 compute_pass_majority_metrics (num_repeats threaded from
dataset config through to pass@k, pass@1[avg-of-k], majority@k, std_dev/std_err, binary-vs-continuous
handling) (nemo-gym-SUMMARY section 7). ELAI gap: the benches recompute variance stats ad hoc per
instrument (P-25 and B-CREATIVITY each hand-rolled bootstrap plumbing); repeats-as-first-class-config
with one canonical variance helper is cleaner and makes the Q-11 per-dollar columns comparable across
benches. Cost S. This is a design-shape steal (rewrite in ELAI's stack, no dependency on Gym).

S-5. Campaign-grain typed stop conditions with attempted-not-successful counting. Source: the NeMo
auto-research skill (convert user budget to absolute deadline + target_experiment_count +
per_experiment_timeout at campaign start; count attempts including crashes; re-check before AND after
each run; never stop on "feels sufficient") (nemo-rl-SUMMARY Deep Dive A, stop conditions). ELAI
gap: ELAI has spend caps (Q-11) and per-run budgets, but the RESEARCH CAMPAIGN grain (a multi-run
Cosmos/explore effort spanning sessions) has no typed stop-rule object; stop discipline lives in
prose and handoffs. Cost S-M. Composes with the O-plan scheduled TaskSpec substrate and Q standing
goals. Verdict lean: EXTENSION of existing budget machinery, worth carrying to plan authoring only
when a real multi-session research campaign consumer exists.

S-6. Session-memory mismatch-logging on recovery. Source: nemo-rl-session-memory recovery workflow
(verify notes against live git/fs, then append a timeline entry RECORDING the mismatches found)
(nemo-rl-SUMMARY Deep Dive B). ELAI gap: the handoff protocol says re-verify load-bearing claims
against the live tree on resume, but does not require recording where the notes disagreed with
reality; that mismatch record is exactly the data that would let ELAI measure handoff drift over
time. Cost S (one handoff-protocol clause). Harness-side only.

Explicitly NOT stolen, with reasons:

- Evo git-worktree experiment isolation: ELAI already specs copy-on-write workspace isolation for
  parallel branches (I-track) and worktree-pool lifecycle (J-track). The Evo version is a
  confirmation of the design, not a gap. Duplicate-under-different-names.
- Evo pre-phase cheap gates before benchmark spend: ELAI's layered gate doctrine is already
  cheapest-first with fail-closed ordering. Duplicate.
- Evo eval_epoch stale-result fencing: ELAI already carries evaluator-version edges (N-16) that
  collapse what a refuted evaluator vouched for, which is the stronger form. Duplicate.
- NeMo Gym four-server verifier/process split: ELAI 4A process isolation + the I-track supervisor
  (generator emits pure intents, cannot reach gate code) already owns this boundary at design level.
  Duplicate at design grain; the HTTP-contract angle survives only in Section 4 (federation).
- NeMo RL two-tier checkpoint retention (best-K + latest-K): real pattern, but ELAI's archive keeps
  everything append-only already; the delta (best-by-metric pointer beside newest) is cosmetic until
  a concrete consumer exists.
- Anything from the AIDE2 announcement not already in Q-11..Q-13: the announcement is
  vendor-reported, the AIDE_k code is unreleased, and the engine run already extracted the
  mechanisms worth testing. Copying more of it without ablation is the brief's own anti-pattern.

SECTION 3 — IMPROVING THE IMPROVEMENT-CONJECTURE ENGINE ITSELF

The engine's honest ledger: proven grounding (3.5x anchors) + the narrow NOVEL-CREDIBLE selector;
refuted single-shot novelty, ProbeUtility, multi-cycle FDL; and now the adjacent P-25 refutations.
Four concrete improvements, ranked:

E-1. Give the harness-side novelty leg a real instrument + its own canary (the Q-11 discipline
applied to conjecture_triage.py, not just novelty.rs). Evidence of the problem: in the 2026-07-15
AIDE2 run, EVERY candidate's equivalence verdict was UNCERTAIN with instrument_strength=WEAK,
because --novelty without a supplied NLI verdicts file falls back to the deterministic bin-coordinate
leg. Consequence: the NOVEL-CREDIBLE profile (the engine's one supported novelty claim) could not
separate candidates and fell through to the labeled weak fallback; the engine's proven selector is
effectively dead on real runs. Q-11's C6 canary hardens the RUNTIME novelty path (novelty.rs); the
HARNESS triage tool needs the sibling treatment: (a) a documented cheap way to mint the NLI verdicts
file per run (a spawned judge with the existing I-24/N-15 audits produces
{nearest_known_mechanism, SAME|DISTINCT|UNCERTAIN, votes, instrument_version} rows); (b) a
known-SAME + known-DISTINCT canary pair injected into every --novelty invocation, fail-closing to
instrument_strength=FAILED on a miss, mirroring Q-11 exactly. Cost S-M, pure harness Python, no
Rust. Falsifier: on the archived AIDE2 run artifact, the instrumented leg must produce at least one
non-UNCERTAIN verdict on the known-duplicate candidates the ledger later killed lexically (C1 vs the
prior bandit conjecture is a natural known-SAME probe), and the canary must fail closed when handed
a scrambled verdicts file.

E-2. Quantified refutation records (the AIDE2 run's C8, which survived triage but was never picked
or authored anywhere). Current state: KILL-EDGE lines carry killer-type:killer-ref only; a
refutation records WHO killed a candidate but not BY HOW MUCH. The AIDE2 announcement's most
transferable honest finding is that most rejects die within noise; the recheck value of a loser
depends on knowing whether it lost by a mile or by a noise-band whisker. Proposal: an optional
quantitative tail on KILL-EDGE for experiment-killed candidates
(KILL-EDGE: C7 <- experiment:<blake3> delta=-0.031 band=0.045 verdict=within-noise), and the
RECHECK HOOKS grep gains a "reinstate first the within-noise kills" ordering. Composes with S-2
(power floor): a within-noise kill is exactly a NULL-UNDERPOWERED candidate, not a strong refutation.
Cost S (grammar extension + validator arm in conjecture_triage.py; append-only compatible, v1.0/1.1
artifacts unaffected). Falsifier: fold over existing run artifacts must still validate; a synthetic
artifact with the quantified tail must round-trip through --validate and --fold.

E-3. Record the P-25 evidence against the exo-inject operator in the skill text. The skill currently
says empty-region targeting and exogenous injection "survive as ordinary grounding techniques". P-25
Family A then measured target-relevant exogenous artifacts against shuffled and padding controls at
generation grain over improvement-card-shaped output and REFUTED the specificity claim (relevant
artifacts did not reliably beat shuffled or padding; 1.54x token cost). That does not kill exo-inject
as a birth operator (P-25 tested a different lifecycle point: benchmark generation vs engine stage-1
conditioning), but the skill's survival claim is now half-contradicted by adjacent evidence and
should carry the citation so a future session does not re-litigate or over-trust it. Cost: one
sentence in SKILL.md + a line in the feature-map rejection ledger cross-referencing P-25. This is
doctrine hygiene, not a build.

E-4. Selector cost honesty: surface token cost, not just S/M/L class, in the PICK axis table when
receipts exist. The engine's SAFE profile tie-broke five S-class candidates by id-order in the AIDE2
run (declared honestly as arbitrary). Where stage-3 micro-experiments executed, their measured cost
is already hashed into the artifact; folding measured-cost-when-available into the tie-break (before
id-order) makes the SAFE profile marginally less arbitrary at zero new machinery. Cost S. Low
priority; the id-order tie-break is at least deterministic and declared.

Guard rails reaffirmed (no action): do not add a revise/iterate stage to the engine (P-25 Family B
refuted aligned triage+revise against shuffled feedback; iteration helped but alignment was not the
cause and cost 3.1x); do not resurrect ProbeUtility/multi-cycle under any new name; novelty stays a
reported axis, never an asserted discovery capability.

SECTION 4 — BIDIRECTIONAL / UPSTREAM OPPORTUNITIES (ELAI > THEM)

The study confirms ELAI's differentiators are real and absent upstream. Every one of these is
genuinely missing at source level in the named project (receipts in the summaries):

- Fixed total-resource budget accounting: ABSENT in Evo (zero matches for any budget/cost/token
  ledger term package-wide; keep-decision is bare score compare, cli.py:3371), ABSENT in open AIDE
  (backends return token counts, query() discards them), ABSENT in NeMo RL (FLOP/memory/time only,
  no spend ledger). ELAI's usage_turn_event schema + locked-cap per-dollar reporting (Q-11) is the
  portable design. Upstream shapes: an Evo budget plugin (their frontier registry + backend seams
  make this a clean PR); an aideml cost ledger (trivial: stop discarding the counts).
- Noise-aware paired acceptance: Evo keep = point compare vs parent, no CI; open AIDE keeps
  metric > best. ELAI's paired-delta CI95 + tie state is the fix both need; AIDE2's own noise-band
  admission is the sales pitch.
- Semantic hypothesis/failure dedup: Evo's dedup is a substring search (evo discards --like); the
  NVIDIA skills have none (narrative log + TSV). ELAI's novelty classifier + refutation records are
  the richer failed-hypothesis memory the brief asks for, and E-1/E-2 above make them stronger
  before exporting the idea.
- Live verifier/evaluator canaries: ABSENT in NeMo Gym (the verified flag is human-review
  provenance, not a runtime canary; no reward-hacking survival harness), ABSENT in Evo (opt-in grep
  gate only). Q-11's known-answer fail-closed canary is the exact generic mechanism.
- Receipt-bound claims: no project binds a reported metric to artifact-hash + journal-row evidence.
  ELAI's RRA/receipt spine is the differentiator; M-plan's evidence-format-as-open-standard is the
  distribution vehicle if this is ever pursued seriously.
- Quantified refutation records + commit-bound concurrency-safe session memory: the NVIDIA skills'
  keep/discard/crash and session files lack deltas, hash binding, and any concurrency story; ELAI's
  handoff lease + quantified-kill design (E-2) maps directly.

Honest scoping: actually landing upstream PRs is a distribution/marketing activity with real
maintainer-engagement cost, and it feeds no PS number directly (closest is K-track distribution
doctrine). The practical version of "bidirectional" for now: (1) write the mechanisms into ELAI
first (Q-11/Q-12, E-1/E-2, S-1..S-4) so the claims are measured, not aspirational; (2) treat the Evo
budget plugin as the one genuinely cheap upstream candidate if the user ever wants a public
demonstration artifact, because Evo's plugin seams are clean and the gap is embarrassing-obvious.
Per the brief's own rule, no upstream patch before the behavior is confirmed on latest main and a
design issue is opened.

SECTION 5 — FEDERATION ARCHITECTURE ANSWER (brief section 12)

Recommendation: Option B (ELAI-centered federation), executed as Option A-thin in the near term.

Reasoning, two approaches compared:
(a) Option A pure adapters: lowest maintenance, but leaves the brief's foundation layer (budget,
canaries, receipts) unbuilt anywhere; ELAI already owns exactly those mechanisms, so pure adapters
waste ELAI's actual edge.
(b) Option B ELAI-centered: ELAI keeps research reasoning, novelty, conjecture/refutation, memory,
budget, receipts (all live or planned in-tree); Evo-class worktree experiment execution stays
external and optional; NeMo Gym's Responses-API + verify()->reward seam is adopted as the SHARED
CONTRACT SHAPE if/when ELAI needs formal environments, rather than inventing a schema (the brief's
anti-pattern). NeMo RL stays a training executor behind an adapter, relevant only when the I-track
open-weight flywheel (I-30/I-31 GRPO on execution-grounded rewards) unlocks; that is the one real
future consumer, and it is explicitly gated behind D-026 today.
(c) Option C deep fork: rejected; every audited project is early/fast-moving (Gym self-labels
evolving APIs; Evo is 0.7.0), fork burden would be permanent, and ELAI needs none of their internals.

Hardware reality (brief section 10): local is a 12GB RTX 3060 + the remote 3090 sidecar; NeMo RL
training beyond smoke scale requires rented GPUs. Nothing in Sections 2-3 requires any of that:
every candidate above is harness Python or orchestrator Rust benched on the existing free-lane
apparatus. The training layer stays a documented seam, not a build.

What evidence would change this: if the flywheel unlocks and a real ELAI training loop exists, the
NeMo Gym contract adoption moves from deferred to active (its eval-format-is-train-format property
is then directly valuable); if a paying use-case for cross-repo experiment orchestration appears,
Evo's execution kernel becomes worth an adapter instead of a design reference.

SECTION 6 — WHAT NOT TO DO (brief anti-patterns, applied)

- Do not re-open the creativity/injection line off this study; Q-10 + P-25 closeouts bind.
- Do not author a universal experiment schema now; adopt Gym's seam shape when a consumer exists.
- Do not treat AIDE2 numbers (16x compression, +0.042..+0.053, 63>34 hacking) or NVIDIA's
  25%>96.9% social claim as reproduced; the star-count demo's artifacts are absent from both NVIDIA
  repos (checked: no environment, split, ledger, trace, or checkpoint in skills/ or examples/).
  The NVSkills-Eval PASS verdicts are 5-task vendor benchmarks with negative uplift on several
  claude-code dimensions.
- Do not add Q-12's bandit as default routing, ever, without the equal-budget beyond-noise win +
  second benchmark the plan already requires.
- Do not let this study spawn new letter-plan files outside the engine seam; P is the open letter
  and every candidate above goes through paper-to-plan (engine stage-2 will re-check the ledger)
  if/when the user wants them authored.

SECTION 7 — RANKED NEXT ACTIONS

1. Execute Q-11 (already planned, instrument-before-improvement; everything else in this file is
   downstream of a trustworthy novelty instrument + spend-honest bench).
2. E-1 (harness novelty-leg instrument + canary): the engine's one proven selector is currently
   inert on real runs; small, pure-Python, directly composes with Q-11.
3. E-2 + S-2 together (quantified kills + refutation power floor): one protocol wave, both cheap,
   both make every future refutation more reusable.
4. S-1 (pareto_per_task policy variant): fold into Q-12's policy-selector seam when Q-12 executes;
   bench-lane first.
5. S-3/S-4/S-6 (verified flag, repeats/variance helper shape, mismatch-logging): small hygiene
   steals, batch into whichever session touches those surfaces next.
6. E-3 (P-25 citation beside the exo-inject survival claim): one-line doctrine hygiene, do with the
   next skill-file touch.
7. Upstream (Section 4): dormant until the user wants a public demonstration; Evo budget plugin is
   the designated first candidate.

End of reasoning artifact.
