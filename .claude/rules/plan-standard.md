Plan standard — wargame-grade plans (HARD RULE, fail-closed)

Loaded by: any task that authors, refines, or audits a plan file (any family/letter, memory-bank/plans/, decomposition trees, baby steps, task graphs), and by every orchestrator pre-flight (orchestrate, auto-orchestrate, b-plan-runner, idea-intake step 12, paper-to-plan phase 5). This is the SINGLE planning quality bar; the skills own their flows, this file owns what a plan must BE.

CLOSED FAMILIES (binding on WHERE a new plan file lands, any authoring path). A plan family goes CLOSED once oversized; a CLOSED letter is NEVER a target for a new plan file. J is CLOSED as of 2026-07-10 (the grab-bag, already the largest plan in the tree) — do not author a new J-N file for ANY reason, including EXTENSION of an existing J unit (author the extension in the current open letter and cross-link to the J unit it extends). New plan files target the current OPEN letter (P as of 2026-07-10); if the open letter is itself oversized, open the NEXT letter rather than growing it. This binds every authoring path (paper-to-plan, direct main-LLM authoring, idea-intake trees are exempt — they are session decomposition trees, not letter plans).

WHY THIS EXISTS

Planning was fragmented across 5+ skills, each with a partial quality bar, and plan quality was judged qualitatively ("looks complete"). Meanwhile the measured dominant failure modes were: wire-dark producers (~40-45% pre-gate era), seam defects passing every per-unit gate (Liberty 2026-06-16, 7 defects), and executors improvising when a plan left judgment calls open. Doctrine origin: the wargame format (external fable-wargame-kit, 2026-07: a plan is a SIMULATED COURSE OF ACTION a cheaper executor runs blind) fused with our wiring-enforcement R1-R10, production-caller-gate (D-030), and the grounding sweep. You pay for the strong model's judgment once, at planning time; the executor never needs it.

THE UNIT SHAPE (every unit at every grain: plan phase, B-N step, task-graph node, baby step)

- MOVE: every action states its EXPECTED_OBSERVATION — exactly what the executor should see if the move worked (command output, file state, UI state, event row). An action without an observable is not a move, it is a wish.
- ON_FAIL: every move carries its most likely failure, the CAUSE that failure signals, and the COUNTER-MOVE. One per move minimum; the counter-move is concrete (a command, an edit, a route), never "investigate".
- FORK: every decision point has a deterministic TRIGGER — "if you observe X, take route B". No judgment calls left to the executor. A plan with zero forks declares "FORKS: none" explicitly (absence is declared, never implied — R1 posture).
- RECON_NEEDED: every assumption planning could not settle is marked with the EXACT check that settles it (a command, a grep, a WebSearch, a paper-ingestor dispatch). Maps to the grounding classes: GROUNDED needs nothing; VERIFY/INGEST become RECON_NEEDED entries; an UNGROUNDED claim with no RECON_NEEDED check is a gate FAIL. This is the unknown-unknowns leg: what recon cannot settle is written down as a landmine with a tripline, not silently assumed.
- ABORT: the conditions where the executor STOPS AND FLAGS rather than improvises (composes with tripwires T1-T4 and the bubble-up cap; a worker's ABORT = the R-4 BLOCKED handoff, never self-repair).
- VERIFY: the verification runs the executor performs, WHEN, and what PASS looks like for each — and every pass criterion names an OFF-AGENT artifact (wiring-enforcement R4: an event/SSB row, a coverage hit, a span, a frozen checker's exit code), never the executor's own stdout or self-report.
- WIRING (any unit that ships code): the unit is a VERTICAL SLICE — producer + its real production consumer + the smoke trigger in the SAME unit, with the PRODUCTION_HOOK seam named (file + fn). A producer-only unit is REFUSED at planning time (walking-skeleton PRE_GATE; D-030 leg B). Cross-unit shared resources (schema, catalog, config, denylist) are THREADED CONTRACTS listed as required inputs in every unit that touches them.
- INTEGRATION_SMOKE (any plan with 2+ code units): a dedicated smoke unit after every 2-3 units AND before DONE, driving the REAL entrypoint on a HARD input (strict peer/provider, multi-turn, long enough to fire secondary paths) — R10. Per-unit green is necessary, never sufficient; the seams are where plans die.
- REDTEAM: the plan records its red-team pass — at least one attack that FAILED against the plan (and why it failed) and every PATCH born from an attack that succeeded. A plan that has never been attacked is untested speculation.

THE 10-POINT GATE (the plan-auditor's verdict schema; missing point = FAIL, not WARN)

 1. Every move has an EXPECTED_OBSERVATION.
 2. Every move has ON_FAIL (failure + cause + counter-move).
 3. Every fork has a trigger; zero-fork plans declare FORKS: none.
 4. Every unsettled assumption is RECON_NEEDED with its exact settling check; no UNGROUNDED claim lacks one.
 5. ABORT conditions exist.
 6. VERIFY runs are spelled out with off-agent pass artifacts (R4).
 7. REDTEAM block records the surviving attack + the patches (adversarial-auditor mode=plan is the attacker, never the plan's author).
 8. EXECUTABLE BLIND: a mid-tier executor could run it end to end without asking a single question. Every question it WOULD ask is a missing fork, recon, or abort — name them.
 9. Every code-shipping unit is a vertical slice with a named PRODUCTION_HOOK seam (or WIRING_CONTRACT block); shared resources are threaded contracts.
10. INTEGRATION_SMOKE units present per the 2-3-unit cadence with a HARD-input spec (plans with 0-1 code units declare INTEGRATION_SMOKE: n/a).

Verdict schema per point: pass | fail | unknown, unknown = FAIL (a point the auditor cannot judge is not passing). Point 8 is qualitative and stays with the auditor; points 1-7, 9-10 have a deterministic lexical leg.

THE DETERMINISTIC LEG (off-agent, frozen, fail-closed)

python .claude/tools/plan_gate_check.py --file <plan> [--json]
Lexical block-marker presence + count-consistency checks (EXPECTED_OBSERVATION/ON_FAIL pairing, FORK-or-none, RECON_NEEDED-or-GROUNDED, ABORT, VERIFY with a runnable line, REDTEAM, PRODUCTION_HOOK|WIRING_CONTRACT on code units, INTEGRATION_SMOKE-or-n/a). Exit 0 pass / 1 fail / 2 scan-config error (fail-closed, never read as pass). It is Layer-2 regex-unsound, necessary-not-sufficient — marker presence is not plan quality; the auditor's 10-point verdict is the authority. Checker + this rule are frozen in .claude/gate-manifest.txt (R8): a plan run cannot weaken the bar it is judged by.

ORCHESTRATOR PRE-FLIGHT (binding on orchestrate / auto-orchestrate / b-plan-runner)

Before walking any plan: run plan_gate_check.py on it. FAIL -> the plan is NOT executed; it first gets a wargame-refinement pass (planning-workflow "Plan improvement" protocol, which injects the missing blocks and re-grounds), then re-gates, then executes. This is refine-before-execute, not fail-open: a plan that fails the gate is exactly a plan that will make the executor improvise. Legacy plans (authored before this standard) are not retro-edited in bulk; they hit the same pre-flight when and only when something tries to execute them. The ONLY skip is an explicit user override in the current message (logged as USER_OVERRIDE in the run log).

ROLE BOUNDARY

Authoring: main LLM or decomposer agents per the invoking skill. Red-team + 10-point verdict: an auditor-class agent that did NOT author the plan (in-boundary self-grading is the rubber-stamp failure the whole gate stack exists to prevent). Checker: frozen tool, run by the auditor and the orchestrator pre-flight; an author self-run is a courtesy, never the authority.

BLOCK MARKER VOCABULARY (what the checker greps; ALL-CAPS at line start)

EXPECTED_OBSERVATION | ON_FAIL | FORK / FORKS: none | RECON_NEEDED | ABORT | VERIFY | REDTEAM | PRODUCTION_HOOK / WIRING_CONTRACT | INTEGRATION_SMOKE / INTEGRATION_SMOKE: n/a
The legacy planning-workflow blocks (PRE_GATE, DETERMINISTIC_CHECK, ADVERSARIAL, ACCEPTANCE) stay valid and REQUIRED where that protocol mandates them, but they are NOT checker markers — the checker does not grep them; they feed the AUDITOR's judgment on points 1 and 6 (ADVERSARIAL = code-level attack tests, distinct from REDTEAM which attacks the PLAN). A plan needs BOTH sets when both protocols apply.

SELF-CHECK before calling any plan done
1. Could a mid-tier model run this blind — zero questions? 2. Does every move say what you SEE when it works and what you DO when it fails? 3. Is every unsettled assumption a RECON_NEEDED tripline? 4. Has someone other than the author attacked it, and is the attack on record? 5. Do code units name their seam and does the plan smoke the seams, not just the units?
If any answer is no, it is not a plan — it is a hope.
