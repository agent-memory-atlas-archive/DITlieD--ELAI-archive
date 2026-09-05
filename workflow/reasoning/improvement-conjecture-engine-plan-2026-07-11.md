# Improvement-Conjecture Engine — plan (2026-07-11)

PROOF_SPINE: none (discipline) — user-acked 2026-07-11 this session (precedents: L-23 S2 leg acked
2026-07-08, L-24 leg D acked 2026-07-11, Q-8/Q-9 acked 2026-07-10). The engine improves plan-authoring
idea quality; it feeds no measured spine number, and its ledger-recheck loop is upstream of everything
but measured by nothing — discipline-class by definition. Per-candidate PSR-5 declarations inside each
run are a separate, mandatory artifact field (see schema).

PLACEMENT (user-acked 2026-07-11): harness tooling, NO letter plan file. This doc is the wargame plan
the executor walks; the deliverables are one new skill, one advisory tool, two skill amendments, and
one feature-map boundary note. Run journals land at workflow/reasoning/conjecture-run-<trigger-slug>-<YYYY-MM-DD>.md.
Skill and .claude/tools files are legally editable in place (the never-amend rule binds plan files only).

## WHAT IT IS

A tool the model uses inside future paper-to-plan / idea-intake runs (and standalone) to generate
improvement ideas WITH ELAI's own machinery instead of freestyle reasoning. Operating claim it makes
true: "generate candidate improvements, go through the best ~10 options, test them, choose one."
INPUT: an improvement trigger — a paper, an external repo, a named weakness or bench gap, or
"improve area X". OUTPUT: a ranked, receipt-carrying pick that flows into the EXISTING paper-to-plan
phase-5 / idea-intake decomposition unchanged. The engine generates and ranks; it NEVER authors plan
files itself.

SIBLING BOUNDARY (binding). L-24's conjecture-memory middle tier and this engine are the same epistemic
shape at two grains — both deliberately from the DeepReason collab. L-24 = RUNTIME memory inside
.elai_cc for the EXPLORE lane (Rust, SSB-journal-grounded, EX-5 lane-walled). The engine = HARNESS-side,
plan-AUTHORING-time conjectures about ELAI ITSELF, living in workflow/reasoning/ run artifacts (the run
artifact IS its journal; no SSB, no Rust store). The engine never writes into the L-24 store or the
verified case graph; L-24 never feeds candidates into plan authoring. A future bridge is a new plan,
never a silent extension.

DGM BOUNDARY (symmetric, binding). The H DGM lane (H-43..H-48) owns CONFIG/CODE mutation behind the
un-gameable-filter theorem. This engine owns DESIGN/PLAN-level improvement ideas; its output goes
through the human/plan-gate pipeline and is never auto-applied. Symmetrically: the DGM lane must not
start proposing plan-level ideas — one owner per grain. (Recorded in the feature map by UNIT 4.)

K-19 SF-1..SF-4 BINDS this work (self-improvement-adjacent: it curates improvement ideas):
SF-1 an executed micro-experiment outranks any judge score, always. SF-2 the judge is never the
generator (cross-family optional). SF-3 never-into-the-auditor: the engine's artifacts are never fed
to the auditor judging the resulting plan. SF-4 losers are never deleted — quarantine via status,
run artifacts are append-only history.

## DECISIONS LOG

Adopted design inputs from workflow/reasoning/improvement-conjecture-engine-support-2026-07-11.md §2,
unmodified — no conclusion overruled:
- FALSIFIER = THE TEST SPEC, one field at two lifecycle points: stage 1 declares it, stage 3 executes
  or estimates it. A candidate whose falsifier cannot be SPECIFIED as a runnable check or a
  paper-grounded estimate forbids nothing: auto-reject (demarcation, the L-22 HG-5 pattern applied to
  improvement ideas).
- LOSERS ARE NOT FALSE: status vocabulary winner / refuted / pending / suspended, never a boolean.
  A candidate killed because its premise or its killer changed is suspended, not refuted.
- EVERY KILL CARRIES ITS KILLER (N-16 S4 edge at harness grain): the ledger entry slug, the judge
  id+version, or the experiment artifact hash. This is what makes loser re-checks MECHANICAL.
- ANTI-LAUNDERING (L-24 ATTACK-3 verbatim): no candidate reaches winner without an executed stage-3
  leg. Untested candidates rank pending and cannot be the pick; an autonomous pick of one is recorded
  PICKED-PENDING — an honest state; silent promotion is not.
- RECORD MOVES, NOT SURVIVORS (L-24 leg D / FD-1 transferred): the run artifact keeps ALL candidates,
  all bins INCLUDING empty bins attempted, all kill reasons with killers, all scores, the pick.
- DIVERSITY IS STRUCTURAL, BANS LIVE AT THE GATE (S3/EX-1, measured placebo + priming): MAP-Elites
  binning + stance rotation + VS tail-weighting are the levers; never a "be different / avoid X"
  directive, and ledger-rejected content is never rendered into a generation prompt.
- JUDGE SEPARATION is the house invariant (D-6 execution/evaluation separation; cited, not re-argued):
  the generator never scores its own candidates; LLM-judge scores are ADVISORY ranking (TD-3) — the
  deterministic stage-2 kills are the only hard rejections; I-24/N-15 order-swap + verbosity-penalty
  audits apply to any LLM judge pass.
Session decisions: placement per PLACEMENT block (user-acked); PROOF_SPINE none (user-acked);
stage-4 pick semantics — default ONE winner, but an invoking pipeline may take the top-k survivors
(paper-to-plan precedent: several orthogonal steals per paper), each individually satisfying the
anti-laundering rule.

## GROUNDING (all in-corpus or verified; feature-map line anchors dated 2026-07-11, stale-prone)

- MAP-Elites binning: E-7 Darwinian loop carries MAP-Elites promotion (feature map ~1671-1677).
- Stance rotation: the anti-spiral skill's 5 personas (Hacker/Researcher/Simplifier/Architect/
  Contrarian; persona agents exist in .claude/agents/).
- Verbalized-Sampling typicality: L-22's VS contract, arXiv:2510.01171 (WebSearch-verified 2026-07-08).
- Falsifier contract + demarcation: L-22 (HG-5); L-24 falsifier-equals-test-spec conclusion.
- Judge separation + bias audits: D-6/D-13 (cross-family), I-24 + N-15 (order-swap, verbosity penalty,
  weight-zeroing on detected bias).
- K-19 SF-1..SF-4 doctrine (feature map ~1817-1835); N-2 corpus-grain composition validator (N-plan).
- Constraining ledger rows re-confirmed this session: "MCTS-first search REJECTED; search is a LADDER
  best-of-N" (this engine's stage 1/3 is a best-of-N ladder — aligned); "Same-model self-critique as a
  gate: BANNED (D-13; TD-3 LLM is advisory)"; "Force-JSON / constrained decoding on reasoning roles:
  BANNED (HD-3)" — so stage-1 candidate cards are free-form markdown validated post-hoc by the tool,
  never grammar-forced generation.
- No REJECTED/ALREADY-TRIED ledger entry conflicts with this engine (Section 4 swept 2026-07-11:
  no prior improvement-ideation-engine proposal, no conflicting doctrine row). B-40 idea-framer +
  critic loop (feature map ~634) is idea-intake's existing front stage — the engine slots AROUND it
  (see UNIT 3), it does not replace the framer. E-8 GEPA self-evolving prompts is prompt optimization,
  a different grain — no overlap.

## THE FOUR STAGES (the canonical protocol UNIT 1 writes into the skill)

STAGE 1 — DIVERGE (~10 typed candidates, structural diversity).
Each candidate is a CARD: id (C1..Cn) / mechanism family / target area / target units (real anchors,
grep-verified in stage 2) / expected measurable effect / cost class (S|M|L) / FALSIFIER (the kill-test
spec: a runnable check or a paper-grounded estimate spec) / bin (mechanism-family x target-area) /
stance used / typicality (0-1, VS self-report) / PROOF_SPINE feed line (PSR-5: `feeds: PS-N <clause>`
or `none (<class>)`).
Mechanism-family starter vocabulary (closed, extensible per run with an explicit declaration in the
artifact): gate/checker, memory/reuse, routing/economics, prompt/context-layout, decoding/inference,
measurement/bench, doctrine/process, integration/seam. Target areas: the subsystem groups / letter
tracks the trigger names.
Structural diversity mechanics: generation runs across >=3 calls with stance/persona rotation; calls
after the first receive ONLY the bin-matrix state (which (family x area) coordinates are filled/empty)
— never prior candidate text, never ledger content — and are directed at EMPTY bins (MAP-Elites
empty-bin forcing: naming a target bin is a structural instruction, not a "be different" directive).
When >=3 candidates cluster in one bin, VS typicality tail-weighting applies: keep the least-typical,
route stage-3 budget toward atypical candidates.
Demarcation: a candidate that cannot specify its falsifier is auto-rejected at the card stage
(forbids nothing).

STAGE 2 — GROUND + FILTER (deterministic, BEFORE any scoring; kills are the only hard rejections).
Run python .claude/tools/conjecture_triage.py over the candidate cards. Checks, each kill recording
its killer:
(a) demarcation: FALSIFIER field present and specified as runnable-or-estimable; missing = refuted,
    killer = triage:demarcation.
(b) ledger duplicate: lexical match against docs/thefinalglance/ELAI-FEATURE-MAP.md SECTION 4
    REJECTED/ALREADY-TRIED + prior conjecture-run artifacts' refuted candidates; hit = refuted,
    killer = ledger:<entry-slug> (the entry's verbatim first line).
(c) anchor verification: every target unit greps to a real symbol/file in .claude/tools/elai.map
    (freshness-checked first, FORK-2); dead anchor = refuted, killer = triage:anchor:<the missing ref>.
(d) pre-N-2 composition triage: cheap lexical pass against the feature map's canonical-subsystem
    entries — does the candidate fork an existing canonical subsystem instead of composing it?
    Flag = refuted, killer = triage:composition:<subsystem>. This is a PRE-N-2 TRIAGE only: N-2 proper
    is the machinery and runs at plan-authoring time on the winner; a second full composition
    validator would itself violate N-2.
(e) PSR-5 presence: the per-candidate PROOF_SPINE feed line exists (declaration check only).
(f) external-claim grounding flag: any claim citing external prior art without a verified source is
    routed to a RECON queue (WebSearch / paper-ingestor per grounding class) — unresolved = the
    candidate cannot leave pending.
The tool is ADVISORY (TD-3): it triages and records; it is never cited as a done-gate and never enters
.claude/gate-manifest.txt. Its kills are "hard" only in the sense that stage 3 does not spend budget
on them — every kill stays in the artifact with its killer and is mechanically reversible when the
killer changes.

STAGE 3 — TEST + SCORE (the executed leg; SF-1 preference order).
Cheaply-testable candidates get a MICRO-EXPERIMENT: execute the stage-1 falsifier (the same object,
second lifecycle point) as a harness-cheap check — a grep, a small script in the session scratchpad,
a token-count measurement; the output is hashed (blake3) into the artifact as the experiment receipt.
Micro-experiments needing an .elai_cc build or live sidecar are OUT of engine scope (FORK-4 downgrades
to estimate). Non-executable candidates get a PAPER-GROUNDED EFFECT ESTIMATE: citation + quoted effect
size, else they stay pending.
The remainder are scored by a JUDGE PASS that did not generate them: one spawned judge agent
(model=sonnet, fresh context, receives cards + stage-2 results + receipts, never the generation
prompts), optionally one cross-family diversity vote via gpt-bridge (grok_chat/gpt_chat, advisory,
never on the critical path). I-24/N-15 audits apply: candidate order randomized between judge calls;
verbosity-length penalty declared in the judge prompt.
Preference order (SF-1, absolute): executed micro-experiment > cited estimate > judge-only score.
A judge-only candidate ranks pending; pending cannot be winner (anti-laundering).

STAGE 4 — CHOOSE.
Top ~3 presented with tradeoffs (mechanism, expected effect + receipt, cost class, composition fit,
falsifier strength). Interactive mode: user picks. Autonomous mode: auto-pick the top-ranked candidate
with an executed leg and record the pick; if no candidate has an executed leg, the pick is recorded
PICKED-PENDING explicitly. The winner flows into the EXISTING paper-to-plan phase-5 / idea-intake
decomposition unchanged. SF-3: the run artifact is never handed to the auditor that later judges the
resulting plan.

## RUN ARTIFACT SCHEMA (workflow/reasoning/conjecture-run-<trigger-slug>-<YYYY-MM-DD>.md)

Sections, all mandatory: TRIGGER (verbatim input + routing fork taken) / BIN MATRIX (all coordinates,
filled + EMPTY-ATTEMPTED) / CANDIDATES (every card, including auto-rejects) / STAGE-2 KILLS (one
KILL-EDGE line each) / STAGE-3 RESULTS (receipts: experiment hashes, citations, judge id+version +
scores) / PICK (top-3 tradeoffs, the pick, PICKED-PENDING if applicable) / RECHECK HOOKS.
KILL-EDGE grammar (greppable, line-start): `KILL-EDGE: <candidate-id> <- <killer-type>:<killer-ref>`
where killer-type ∈ {ledger, judge, experiment, triage, premise} and killer-ref is a STABLE id — a
ledger entry slug, judge id+version, artifact blake3 hash, or the candidate-id of a refuted premise.
Never a line number. Status vocabulary: winner | refuted | pending | suspended. A later session
re-checks losers mechanically: grep conjecture-run-*.md for KILL-EDGE lines pointing at the changed
killer, reinstate those candidates to pending. A premise-killed candidate whose premise later ships
differently is suspended, not refuted.

## UNITS (vertical slices; executor walks in order)

UNIT 1 — engine core: skill + triage tool + toy smoke.
MOVE 1.1: author .claude/skills/improvement-conjecture/SKILL.md — frontmatter (name, description with
trigger phrases: "/conjecture <trigger>", "generate improvement conjectures", "run the conjecture
engine", plus staged invocation from paper-to-plan/idea-intake), the four-stage protocol above
verbatim, the artifact schema, the boundary statements (L-24 sibling, DGM symmetric, SF-1..SF-4),
and the hard rules (no ledger content in generation prompts; no constrained decoding on the
generation role per HD-3; judge never the generator).
EXPECTED_OBSERVATION: the file exists; a fresh session lists the skill; its STAGE 2 section contains
the exact runnable line `python .claude/tools/conjecture_triage.py --run <artifact-path> --json`.
ON_FAIL: skill absent from the session skill list -> frontmatter malformed; diff the frontmatter
against .claude/skills/oracle/SKILL.md (a known-loading sibling) and fix; re-open a session to
re-check.
MOVE 1.2: author .claude/tools/conjecture_triage.py — stdlib-only, --json, checks (a)-(f) above,
plus --validate <artifact> (schema-section + KILL-EDGE grammar check on a run artifact) and
--self-test (embedded fixtures: one forbid-nothing card, one ledger-duplicate card, one dead-anchor
card; expects exactly those three kills, each carrying the right killer-type). Exit 0 = ran (kills
reported in JSON), exit 2 = scan-config error (feature map or elai.map unreachable) — fail-closed,
never read as a clean pass.
EXPECTED_OBSERVATION: `python .claude/tools/conjecture_triage.py --self-test --json` exits 0 and
reports the 3 fixture kills with killer-types triage:demarcation, ledger:<slug>, triage:anchor.
ON_FAIL: exit 2 -> path resolution bug (tool must locate the feature map + elai.map from the repo
root, not CWD); fix resolution, never downgrade a missing input to a pass; wrong kill set ->
fixture/check drift, fix the check not the fixture expectations.
MOVE 1.3: toy smoke — run the skill end-to-end in-session on a toy trigger ("improve the
conjecture_triage.py self-test coverage" or similar), producing a real run artifact.
EXPECTED_OBSERVATION: workflow/reasoning/conjecture-run-<slug>-<date>.md exists with all seven schema
sections; `python .claude/tools/conjecture_triage.py --validate <it> --json` exits 0; at least one
KILL-EDGE line greps.
ON_FAIL: a missing section -> schema drift between SKILL.md and the tool's --validate; fix BOTH in the
same changeset so they agree (the tool is the schema's executable statement).
PRODUCTION_HOOK: conjecture_triage.py's production caller = the skill's STAGE 2 step (the runnable
line in .claude/skills/improvement-conjecture/SKILL.md); the skill's production callers = the UNIT 2 /
UNIT 3 seams + direct user invocation. A tool shipped without the skill line calling it is wire-dark
at harness grain — refused within this unit (vertical slice).

UNIT 2 — paper-to-plan seam.
MOVE 2.1: amend .claude/skills/paper-to-plan/SKILL.md PHASE 4: candidate-steal generation runs THROUGH
the engine — stage 1 diverges candidate improvement conjectures from (paper mechanisms x ELAI target
areas); stage 2 subsumes the existing ledger-check sentence (the check itself is unchanged, now
deterministic via the tool and killer-recorded); the phase-4 steal file and the engine run artifact
are ONE document (the conjecture-run artifact carries the steal verdict vocabulary NEW | EXTENSION |
ALREADY-SHIPPED | ALREADY-REJECTED | REJECT per surviving candidate); stage-4 survivors (top-k, each
with an executed or cited leg) flow into PHASE 5 unchanged.
EXPECTED_OBSERVATION: phase-4 text names the improvement-conjecture skill + the run-artifact path
convention; PHASE 5 text is byte-unchanged; the amendment is <= 15 lines (pointer-grade, the protocol
lives in the skill).
ON_FAIL: the amendment starts restating stages -> stop, replace with a pointer to the skill (redundancy
between files is a bug); phase-5 drift -> revert phase 5, the engine never authors plan files.

UNIT 3 — idea-intake seam.
MOVE 3.1: amend .claude/skills/idea-intake/SKILL.md with an improvement-trigger route: a new Step 1.5
(after Step 1 regen-map, before Step 2 idea-framer), fired ONLY when the intake is an improvement
trigger (named weakness, bench gap, "improve area X") rather than an app/feature brief. The engine
runs stages 1-4; the winner's card is written as the framer's raw_idea input (appended to
idea-source.md with its receipts); Step 2 onward — the B-40-shaped idea-framer + critic loop and the
whole decomposition chain — runs UNCHANGED on the winner. The engine slots around the framer, never
replaces it.
EXPECTED_OBSERVATION: the skill text carries Step 1.5 with the fire-condition and the skill pointer;
Steps 2-14 byte-unchanged; a dry routing read-through of a non-improvement brief ("build me a todo
app") hits Step 2 directly without engine invocation.
ON_FAIL: fire-condition too broad (normal briefs routed into the engine) -> tighten the condition to
the three named trigger shapes; any Step 2+ edit -> revert, the seam is insertion-only.

UNIT 4 — feature-map boundary notes (doc-grain, no code).
MOVE 4.1: in docs/thefinalglance/ELAI-FEATURE-MAP.md: (a) append this run's verdict line at the end of
the Section-4 DEEPREASON-COLLAB block (~4941, re-grep at edit): the engine as a new steal-adjacent
HARNESS-side unit (not one of C1-C4), pointer to this plan; (b) add the symmetric DGM clause to the
K-19 doctrine entry (~1817, re-grep): plan-level improvement ideation is owned by the harness
improvement-conjecture engine; the DGM lane never proposes plan-level ideas — one owner per grain;
(c) header line-4 date append per convention.
EXPECTED_OBSERVATION: all three edits land; grep "improvement-conjecture" in the feature map returns
the two new regions; no other line changed.
ON_FAIL: Read-token-cap trouble near the huge header line -> Grep -n to locate + tight offset Reads +
PowerShell Substring for the header tail (support file §9 gotcha); edit-anchor mismatch -> re-read the
exact on-disk tail line and re-anchor, never anchor on remembered prose.

UNIT 5 — INTEGRATION SMOKE (independent, after UNITS 1-4).
INTEGRATION_SMOKE: an auditor-class agent (NOT the executor of UNITS 1-4) drives the REAL entrypoint —
the skill invoked on a HARD real trigger (a named current bench gap or "improve ELAI's
context-lifecycle economics"), multi-round, long enough to fire the secondary paths. PASS requires ALL
of: >=8 candidates across >=5 bins; >=1 EMPTY-ATTEMPTED bin recorded; >=2 stance rotations visible in
the artifact; >=1 ledger kill with killer-type ledger:<slug>; >=1 executed micro-experiment receipt
(blake3 hash present); a PICK section with top-3 tradeoffs; `conjecture_triage.py --validate` exit 0
on the artifact; THEN the RECHECK DRILL — on a scratch COPY of the feature map, alter the ledger entry
that killed a candidate, run the documented grep over conjecture-run-*.md, observe the killed
candidate found mechanically and reinstated to pending in a drill note (the live feature map is never
edited by the drill). Toy happy-path runs do not satisfy this.
EXPECTED_OBSERVATION: the smoke run artifact satisfies every bullet above; the auditor reports each as
pass|fail|unknown with unknown = FAIL.
ON_FAIL: any bullet unknown/fail -> the failing stage's unit re-opens (schema drift -> UNIT 1; seam
miss -> UNIT 2/3); a passing toy run offered as the smoke -> rejected, re-run on a hard trigger.

## FORKS

FORK-1 trigger routing: paper/repo link observed -> the paper-to-plan seam (UNIT 2 path); named
weakness / bench gap / "improve area X" -> the idea-intake route or standalone /conjecture; ambiguous
trigger -> ask the user one question, never guess.
FORK-2 anchor oracle: `python .claude/tools/elai_map_freshness.py` exit 1 (stale) -> regenerate
elai.map per its header command before stage-2 anchor checks; exit 2 (missing) -> use the
codegraph-elai MCP (codegraph_search/codegraph_callers) as the anchor oracle for this run and record
the substitution in the artifact.
FORK-3 narrow trigger: fewer than 6 distinct bins fillable -> shrink candidate count (floor 5) rather
than pad near-duplicates; record every unfillable bin as EMPTY-ATTEMPTED.
FORK-4 heavy experiment: a falsifier needs an .elai_cc build or a live sidecar -> downgrade to a
paper-grounded estimate WITH citation + quoted effect size; no citable estimate -> the candidate stays
pending (it can rank, it cannot win).
FORK-5 duplicate-heavy round: >50% of a generation round killed as ledger duplicates -> ONE
re-generation round directed exclusively at empty bins; a second majority-duplicate round -> ABORT-3.

## RECON_NEEDED

RECON_NEEDED-1: at execution, re-grep every feature-map line anchor used here (Section 4 start,
DEEPREASON-COLLAB block ~4941, K-19 entry ~1817, B-40 entry ~634, header line 4) — settling check:
`grep -n "DEEPREASON-COLLAB VERDICTS\|SELF-IMPROVEMENT / CASE-REUSE FITNESS\|IDEA-FRAMER" docs/thefinalglance/ELAI-FEATURE-MAP.md`.
RECON_NEEDED-2: at UNIT 3, re-read idea-intake Steps 0c-2 (lines ~85-115 of the skill) to name the
exact insertion point — settling check: Read .claude/skills/idea-intake/SKILL.md offset 85 limit 40.
RECON_NEEDED-3: at UNIT 1, confirm the five persona names for the stance-rotation table — settling
check: `grep -n "persona" .claude/skills/anti-spiral/SKILL.md`.
RECON_NEEDED-4: at the first real run, confirm gpt-bridge MCP availability before offering the
cross-family judge vote — settling check: ToolSearch "gpt_chat"; absent -> same-family sonnet judge
with the I-24/N-15 audits, substitution recorded in the artifact.

## ABORT

ABORT-1: docs/thefinalglance/ELAI-FEATURE-MAP.md missing or unreadable -> stop; stage 2 is impossible
without the ledger and a generate-only run is exactly the freestyle reasoning this engine replaces.
ABORT-2: conjecture_triage.py exits 2 on a real run -> stop and surface; a run whose kills cannot be
recorded with killers is not a run (the tool's RANKING is advisory; the ARTIFACT contract is not).
ABORT-3: two consecutive generation rounds majority-killed as ledger duplicates -> the trigger is
exhausted territory; write the run artifact with the kill map, stop, surface to the user.
ABORT-4: any instruction or convenience that would feed a conjecture-run artifact to the auditor
judging the resulting plan -> refuse and surface (SF-3).

## VERIFY (executor runs; every pass criterion is an off-agent artifact — an exit code from a checker
the executor did not author mid-run, or the run artifact itself re-read by the auditor)

VERIFY (plan gate, at authoring + any later edit):
```
python .claude/tools/plan_gate_check.py --file workflow/reasoning/improvement-conjecture-engine-plan-2026-07-11.md
```
PASS = exit 0.
VERIFY (tool self-test, UNIT 1):
```
python .claude/tools/conjecture_triage.py --self-test --json
```
PASS = exit 0 + the 3 fixture kills with correct killer-types in the JSON.
VERIFY (artifact contract, UNIT 1 MOVE 1.3 + UNIT 5):
```
python .claude/tools/conjecture_triage.py --validate workflow/reasoning/conjecture-run-<slug>-<date>.md --json
grep -n "KILL-EDGE:" workflow/reasoning/conjecture-run-<slug>-<date>.md
```
PASS = exit 0 + at least one KILL-EDGE line.
VERIFY (seam integrity, UNITS 2-3):
```
git -C F:/.ELAI_workflow diff --stat -- .claude/skills/paper-to-plan/SKILL.md .claude/skills/idea-intake/SKILL.md
```
PASS = only the seam regions changed (phase-5 / steps-2..14 untouched), reviewed by the auditor, not
self-asserted.
The UNIT 5 smoke verdict is rendered by the auditor-class agent off-agent per its bullet list;
executor self-report is necessary, never sufficient.

## REDTEAM (attacks run against this plan before authoring was declared done)

ATTACK-1 (succeeded -> patched): estimate-laundering — a candidate wins via a vague "paper-grounded
estimate" citing nothing. PATCH: an estimate requires a citation + quoted effect size or the candidate
stays pending; pending cannot win; recorded in STAGE 3 and FORK-4.
ATTACK-2 (succeeded -> patched): bin-gaming — the generator relabels one mechanism into different bins
to fake coverage. PATCH: stage-2 near-duplicate check is bin-blind (same anchors + same mechanism
verbs = duplicate regardless of declared bin); the judge scores mechanism, not bin labels.
ATTACK-3 (succeeded -> patched): kill-edge rot — killers referenced by feature-map line numbers go
stale and the mechanical re-check silently misses. PATCH: KILL-EDGE grammar bans line numbers; killer
refs are stable slugs / id+version / hashes only; the --validate check enforces the grammar.
ATTACK-4 (succeeded -> patched): priming leak — a later session "helpfully" renders the ledger kills
into the next generation round's prompt as "don't propose these". PATCH: hard rule in SKILL.md
(bans live at the gate); post-first-call generation inputs are the BIN MATRIX ONLY; the artifact
records the generation-call inputs so a leak is auditable.
ATTACK-5 (failed, recorded per P7): pre-justify the resulting plan by attaching the winner's engine
journal to the plan-audit dispatch — fails because SF-3 is stated three times (bindings, stage 4,
ABORT-4) and the auditor's dispatch template carries no such input; the plan the winner becomes is
judged on its own plan-gate + auditor pass, blind to the engine's ranking.

## OPEN LIMITS

- Stage-2 lexical checks are advisory-unsound (regex over prose, not semantics); the sound backstop is
  stage 3's executed leg + N-2 proper at plan-authoring time on the winner. Do not oversell the triage.
- The engine's own effectiveness is unmeasured (PROOF_SPINE none, acked); if a later session wants a
  number, the honest one is a before/after on plan-audit BLOCK rates for engine-picked vs freestyle
  ideas — a new plan, not this one.
- Run artifacts accumulate in workflow/reasoning/; the KILL-EDGE grep scales linearly and is fine to
  ~hundreds of runs; revisit indexing only if that ceiling is ever real.

INTEGRATION_SMOKE: UNIT 5 (see above) — n/a is NOT declared; this plan ships code units.
FORKS: declared above (FORK-1..FORK-5).
