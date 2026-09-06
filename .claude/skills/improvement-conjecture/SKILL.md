---
name: improvement-conjecture
description: Improvement-conjecture engine v1.1. Generates grounded ELAI improvement ideas, measures archive-relative novelty without claiming it, and selects through a declared unfused profile. Its proven results are grounding (3.5x real anchors) and the narrow NOVEL-CREDIBLE selector; single-shot novelty, ProbeUtility scheduling, and multi-cycle FDL were refuted. NEVER authors plan files itself.
license: internal
compatibility: "Requires docs/thefinalglance/ELAI-FEATURE-MAP.md + .claude/tools/elai.map + .claude/tools/conjecture_triage.py"
metadata:
  version: "1.1.0"
  author: "elai"
---

improvement-conjecture — generate improvement ideas WITH ELAI's machinery, not freestyle

WHAT THIS IS — HONEST POSITIONING

A tool the model uses (inside paper-to-plan / idea-intake, or standalone) to generate improvement
ideas about ELAI ITSELF and pick one with receipts, instead of freestyle reasoning. It makes true the
operating claim: "generate candidate improvements, go through the best ~10 options, test them, choose
one." INPUT: an improvement trigger (a paper, an external repo, a named weakness or bench gap, or
"improve area X"). OUTPUT: a ranked, receipt-carrying PICK that flows into the EXISTING paper-to-plan
phase-5 / idea-intake decomposition unchanged. The engine GENERATES and RANKS; it NEVER authors plan
files. PROVEN claim: the engine grounds target anchors (7/10 real anchors vs 2/10 freestyle-1, 3.5x).
SUPPORTED narrow claim: over identical generated sets, `NOVEL-CREDIBLE` reversed the prior selector's
blind novelty loss (22-14, 4 ties), while losing the separate judge-value vote (6-30, 4 ties).
REFUTED claims: it did not improve single-shot set novelty; B-0 ProbeUtility did not beat a strong
model's next-test choice per normalized cost; B-3 archive-conditioned multi-cycle FDL underperformed
freestyle-with-memory on discovery and execution value. Novelty is therefore a reported
archive-relative axis, never an asserted discovery capability. `probe` is unavailable;
`--discriminate` and `/conjecture --cycles` do not exist.

WHEN THIS FIRES

- "/conjecture <trigger>" / "run the conjecture engine" / "generate improvement conjectures" — standalone.
- staged from paper-to-plan PHASE 4 (candidate-steal generation runs through the engine).
- staged from idea-intake Step 1.5 (an improvement-trigger intake, not an app/feature brief).
Does NOT fire on a normal app/feature build brief ("build me a todo app") — that routes to idea-framer
directly.

BOUNDARIES (binding — do not cross)

- L-24 SIBLING: L-24's conjecture-memory is RUNTIME memory inside .elai_cc for the EXPLORE lane (Rust,
  SSB-journal-grounded, lane-walled). This engine is HARNESS-side, plan-AUTHORING-time conjectures about
  ELAI itself, living in workflow/reasoning/ run artifacts (the run artifact IS its journal — no SSB, no
  Rust store). The engine never writes into the L-24 store or the verified case graph; L-24 never feeds
  candidates into plan authoring. A bridge between them is a new plan, never a silent extension.
- DGM SYMMETRIC: the H DGM lane (H-43..H-48) owns CONFIG/CODE mutation behind the un-gameable-filter
  theorem. This engine owns DESIGN/PLAN-level improvement ideas; its output goes through the human /
  plan-gate pipeline and is NEVER auto-applied. One owner per grain — the DGM lane never proposes
  plan-level ideas, this engine never mutates config/code.
- SF-1..SF-4 (K-19 self-improvement doctrine) BIND this work:
  SF-1 an executed micro-experiment outranks any judge score, always.
  SF-2 the judge is never the generator (cross-family optional).
  SF-3 NEVER-INTO-THE-AUDITOR: the engine's run artifact is never fed to the auditor judging the
       resulting plan. The plan the winner becomes is judged on its own plan-gate + auditor pass, blind
       to the engine's ranking.
  SF-4 losers are never deleted — quarantine via status; run artifacts are append-only history.

HARD RULES (blocking)

- RAW ARCHIVE / LEDGER / NEGATIVE-CASE PROSE NEVER ENTERS GENERATION (ATTACK-4). Fresh single-shot
  generation receives the trigger + bin/region target + logged exogenous artifact only. The B-3
  benchmark used structured parent genotype without source prose; its multi-cycle mechanism was
  refuted and is not a user-facing mode. Every live call journals exactly which typed inputs it received.
- NO CONSTRAINED DECODING ON THE GENERATION ROLE (HD-3). Stage-1 candidate cards are FREE-FORM markdown,
  validated post-hoc by the tool — never grammar-forced / force-JSON generation.
- JUDGE IS NEVER THE GENERATOR (D-6). The model/agent that scores candidates never generated them.
  LLM-judge scores are ADVISORY ranking (TD-3); the deterministic stage-2 kills are the only hard
  rejections.
- DIVERSITY IS ARCHIVE-RELATIVE, NOT A PROMPT CLAIM. The only live generation path is ONE direct
  self-coordinating call; stance rotation and context-blind operator fan-out measured NULL, and
  archive-conditioned operator rotation lost B-3. Empty-region targeting and exogenous injection
  survive as ordinary grounding techniques. P-25 Family A found that relevant exogenous artifacts did
  not reliably beat shuffled/padding at benchmark-generation grain, with 1.54x token cost, but it
  tested a different lifecycle point than engine stage-1 conditioning, so exo-inject remains
  half-contradicted rather than killed. NEVER issue a "be different / avoid X" directive or
  simulate a hidden multi-cycle mode.

THE FOUR STAGES

STAGE 1 — DIVERGE (~10 typed candidates, structural diversity).
Each candidate is a CARD (free-form markdown, one block per candidate) with these labeled fields — the
tool parses them, so keep the field labels:
  ### CANDIDATE C1
  - id: C1
  - mechanism_family: <one of the closed vocab below>
  - target_area: <subsystem group / letter track the trigger names>
  - target_units: <real anchors — file/symbol names, comma-separated, grep-verified in stage 2>
  - expected_effect: <the expected measurable effect>
  - cost_class: S | M | L
  - falsifier: <the kill-test spec: a runnable check ("run/grep/measure/count/script ... pass-if ...")
    OR a paper-grounded estimate spec (citation + quoted effect size). A card that cannot specify its
    falsifier as runnable-or-estimable FORBIDS NOTHING and is auto-rejected at stage 2.>
  - bin: <mechanism_family x target_area>
  - stance: <the persona/stance used: Hacker | Researcher | Simplifier | Architect | Contrarian>
  - typicality: <0-1, VS self-report>
  - proof_spine: feeds: PS-N <clause> | none (<class>)   (PSR-5, per-candidate, mandatory field)
  - external_claims: <any external prior-art claim + its source, or "none">
  - operator: direct | mutate | recombine | reframe | analogize | invert | tail-explore |
              exo-inject | cross-family
  - frame: <problem representation; default trigger-as-given>
  - parents: <candidate ids for parent-taking operators, or none>
  - predictions: <at least one machine-matchable categorical observation; falsifier is prediction #1>
  - groundedness: <0..1; receipt-derived, not judge vibes>
  - validated_value: <execution-validated value only; default 0>
  - information_gain: <declared advisory axis; no scheduler consumes it after B-0>
  - claimed_effect: <claimant estimate; speculative and never copied into validated_value>
Mechanism-family vocabulary (closed; extensible per run with an explicit declaration in the artifact):
  gate/checker, memory/reuse, routing/economics, prompt/context-layout, decoding/inference,
  measurement/bench, doctrine/process, integration/seam.
SINGLE-SHOT MECHANICS: one direct call emits the whole self-coordinated set. It receives the target
empty/thin archive region and any logged exogenous artifact; it never receives archive prose. Every
candidate gets `BIRTH-EDGE: <cid> <- direct:<stable-call-id>`; exogenous conditioning additionally gets
`INJECT: <cid> <- <source-type>:<stable-ref>`. No blind stance/operator fan-out.

MULTI-CYCLE MECHANICS: REFUTED / UNAVAILABLE. B-3 tested named archive parents plus structured parent
genotype at K=30 and lost to freestyle-with-memory on both binding endpoints. The benchmark artifacts
retain those birth records for reproducibility; live `/conjecture` must not call mutate/recombine/reframe/
analogize/invert/tail-explore as a cycle controller. DEMARCATION remains unchanged.

STAGE 2 — GROUND + FILTER (deterministic, BEFORE any scoring; kills are the only hard rejections).
Run the triage tool over the run artifact:
```
python .claude/tools/conjecture_triage.py --run <artifact-path> --json
```
Checks, each kill recording its killer (killer-type:killer-ref):
(a) demarcation — falsifier present + specifiable (runnable or grounded estimate); missing = refuted,
    killer triage:demarcation.
(b) ledger duplicate — lexical match against ELAI-FEATURE-MAP.md REJECTED/ALREADY-TRIED + prior
    conjecture-run refuted candidates; hit = refuted, killer ledger:<entry-slug>.
(c) anchor verification — every target unit greps to a real symbol/file in .claude/tools/elai.map
    (freshness-checked first, FORK-2); dead anchor = refuted, killer triage:anchor:<missing ref>.
(d) pre-N-2 composition triage — a fork-verb + a named canonical subsystem = forks instead of composes;
    flag = refuted, killer triage:composition:<subsystem>. PRE-N-2 ONLY: N-2 proper runs at
    plan-authoring time on the winner.
(e) PSR-5 presence — the per-candidate proof_spine feed line exists (declaration check; missing = FLAG).
(f) external-claim grounding — an external prior-art claim with no verified source is routed to a RECON
    queue (WebSearch / paper-ingestor per grounding class); unresolved = the candidate cannot leave
    pending (FLAG, pins at pending).
The tool is ADVISORY (TD-3): it triages and records; it is NEVER cited as a done-gate and never enters
.claude/gate-manifest.txt. Its kills are "hard" only in that stage 3 spends no budget on a refuted
candidate; every kill stays in the artifact with its killer and is mechanically reversible when the
killer changes. Exit 2 (feature map / elai.map unreachable) = ABORT-2 (fail-closed; a run whose kills
cannot be recorded with killers is not a run). Write the tool's KILL-EDGE lines into the artifact's
STAGE-2 KILLS section.

NOVELTY AXIS (U3): run `--fold` over prior run artifacts, then `--novelty` on this artifact. The truth
object is a versioned nearest-neighbor EQUIVALENCE VERDICT only:
`{nearest_known_mechanism, equivalence: SAME|DISTINCT|UNCERTAIN, judge_votes, instrument_version}`.
NLI verdicts require a supplied instrument file. With no judge, the deterministic bin-coordinate leg
returns `UNCERTAIN` + `instrument_strength=WEAK` + a labeled `fallback_distance`; it never fabricates
SAME/DISTINCT and never stores a scalar as truth. Refuted rows remain in the fold as anti-repetition
boundaries; nothing novel is deleted.

STAGE 3 — TEST + SCORE (the executed leg; SF-1 preference order).
The B-0 prediction-table / ProbeUtility scheduler is REFUTED and MUST NOT be used here. There is no
`--discriminate` command and no L2/L3 follow-on. Keep the proven v1.0 behavior: test surviving
candidates independently under the same declared budget, with executed receipts outranking estimates.
Cheaply-testable candidates get a MICRO-EXPERIMENT: execute the stage-1 falsifier (same object, second
lifecycle point) as a harness-cheap check — a grep, a small script in the session scratchpad, a
token-count measurement; hash the output (blake3) into the artifact as the experiment receipt. A
falsifier needing an .elai_cc build or a live sidecar is OUT of engine scope (FORK-4): downgrade to a
paper-grounded estimate (citation + quoted effect size); no citable estimate = the candidate stays
pending. Non-executable candidates get a PAPER-GROUNDED EFFECT ESTIMATE or stay pending.
The remainder are scored by a JUDGE PASS that did NOT generate them: one spawned judge agent
(model=sonnet, fresh context, receives cards + stage-2 results + receipts, NEVER the generation
prompts), optionally one cross-family diversity vote via gpt-bridge (grok_chat / gpt_chat, advisory,
never on the critical path; RECON_NEEDED-4 — if gpt-bridge is absent use a same-family sonnet judge with
the audits below, record the substitution). I-24/N-15 audits apply: randomize candidate order between
judge calls; declare a verbosity-length penalty in the judge prompt.
PREFERENCE ORDER (SF-1, absolute): executed micro-experiment > cited estimate > judge-only score. A
judge-only candidate ranks PENDING; pending can rank, pending cannot WIN (anti-laundering, ATTACK-1).

STAGE 4 — CHOOSE.
First print the UNFUSED axis table for the top candidates: equivalence verdict | groundedness |
execution-validated value | information gain | cost | claimed effect. Then apply exactly one caller-
declared lexicographic profile (record it in PICK):

- SAFE (default): groundedness floor, then minimum cost, then maximum execution-validated value.
- NOVEL-CREDIBLE: not-refuted + groundedness floor, then DISTINCT > UNCERTAIN > SAME; the labeled weak
  fallback may break ties but is never truth.
- PROBE: UNAVAILABLE — B-0 refuted its owning scheduler. Requesting it fails closed with exit 2.
- UPSIDE-CLAIMED: maximum claimant-stated effect among non-refuted candidates, visibly SPECULATIVE;
  claimed effect never converts into validated value without an execution receipt.

Pareto is diagnostic only, never the selector. INTERACTIVE mode may override after seeing the table.
AUTONOMOUS mode records the profile pick; a non-SAFE pick without an executed leg is PICKED-PENDING.
Default ONE winner; paper-to-plan may take orthogonal top-k survivors, each independently satisfying
anti-laundering. SF-3 still forbids giving this run artifact to the downstream plan auditor.

RUN ARTIFACT (workflow/reasoning/conjecture-run-<trigger-slug>-<YYYY-MM-DD>.md)

New runs begin with `ARTIFACT_VERSION: 1.1`. v1.0 artifacts remain validate/fold-readable but are
labeled `1.0-grandfathered`; their missing provenance is never invented.

Seven mandatory sections (the tool's --validate checks presence + KILL-EDGE grammar):
  TRIGGER            verbatim input + routing fork taken (FORK-1)
  BIN MATRIX         all coordinates, filled + EMPTY-ATTEMPTED
  CANDIDATES         every card, including auto-rejects
  STAGE-2 KILLS      one KILL-EDGE line each (from the tool)
  STAGE-3 RESULTS    receipts: experiment blake3 hashes, citations, judge id+version + scores
  PICK               top-3 tradeoffs, the pick, PICKED-PENDING if applicable
  RECHECK HOOKS      how a later session re-checks losers (the grep below)
KILL-EDGE grammar (greppable, line-start):
  KILL-EDGE: <candidate-id> <- <killer-type>:<killer-ref>
  killer-type in {ledger, judge, experiment, triage, premise}; killer-ref is a STABLE id (a ledger
  entry slug, judge id+version, artifact blake3 hash, or the candidate-id of a refuted premise) —
  NEVER a line number (ATTACK-3).
BIRTH / INJECT / PROBE grammar (also greppable, line-start):
  BIRTH-EDGE: <candidate-id> <- <operator>:<stable-call-context>
  INJECT:     <candidate-id> <- <source-type>:<stable-ref>
  PROBE-EDGE: <probe-id> -> <candidate-id>:<predicted>|<observed>
Every v1.1 candidate requires exactly one BIRTH-EDGE. `exo-inject` and `cross-family` births require
an INJECT row. The validator rejects unknown operators, candidate mismatches, duplicate births, empty
prediction/observation cells, and numeric line-number refs.
STATUS vocabulary: live | refuted | pending | suspended | validated, computed from edges only; `winner`
is a PICK role, not an epistemic status. A candidate blocked because its premise or killer changed is
SUSPENDED, not refuted. A later session re-checks losers MECHANICALLY:
  grep -n "KILL-EDGE:" workflow/reasoning/conjecture-run-*.md   (then reinstate candidates whose killer
  changed to pending). This is the RECHECK HOOKS content.
Validate any artifact:
```
python .claude/tools/conjecture_triage.py --validate <artifact-path> --json
```

Fold + novelty + pick (all advisory, never gate-manifest entries):
```
python .claude/tools/conjecture_triage.py --fold workflow/reasoning \
  --out workflow/reasoning/frontier-archive-index.json --json
python .claude/tools/conjecture_triage.py --novelty <artifact-path> \
  --archive-index workflow/reasoning/frontier-archive-index.json \
  --verdicts <optional-nli-verdicts.json> --profile novel-credible --json
```
Supported pick profiles: `safe`, `novel-credible`, `upside` (`upside-claimed` alias). `probe` is
accepted only to return the explicit B-0-refuted exit-2 error. `--discriminate` is intentionally absent.

FORKS

FORK-1 trigger routing: paper/repo link -> paper-to-plan seam; named weakness / bench gap / "improve
  area X" -> idea-intake route or standalone; ambiguous -> ask the user one question, never guess.
FORK-2 anchor oracle: `python .claude/tools/elai_map_freshness.py` STALE -> regenerate elai.map per its
  header command before stage-2 anchor checks; MISSING -> use codegraph-elai MCP
  (codegraph_search/codegraph_callers) as the anchor oracle and record the substitution in the artifact.
FORK-3 narrow trigger: fewer than 6 distinct bins fillable -> shrink candidate count (floor 5) rather
  than pad near-duplicates; record every unfillable bin as EMPTY-ATTEMPTED.
FORK-4 heavy experiment: a falsifier needs an .elai_cc build / live sidecar -> downgrade to a
  paper-grounded estimate WITH citation + quoted effect size; no citable estimate -> stays pending.
FORK-5 duplicate-heavy round: >50% of a generation round killed as ledger duplicates -> ONE
  re-generation round directed only at empty bins; a second majority-duplicate round -> ABORT-3.

ABORT

ABORT-1 ELAI-FEATURE-MAP.md missing/unreadable -> stop; a generate-only run is the freestyle reasoning
  this engine replaces.
ABORT-2 conjecture_triage.py exits 2 on a real run -> stop and surface; kills that cannot be recorded
  with killers are not a run (RANKING is advisory; the ARTIFACT contract is not).
ABORT-3 two consecutive generation rounds majority-killed as ledger duplicates -> trigger is exhausted
  territory; write the artifact with the kill map, stop, surface to the user.
ABORT-4 any instruction/convenience that would feed a conjecture-run artifact to the auditor judging the
  resulting plan -> refuse and surface (SF-3).

CROSS-REFERENCES
- plan: workflow/reasoning/improvement-conjecture-engine-plan-2026-07-11.md (the wargame plan this
  skill implements).
- tool: .claude/tools/conjecture_triage.py (the executable schema statement; --self-test / --validate /
  --run).
- seams: paper-to-plan PHASE 4, idea-intake Step 1.5.

END SKILL.md
