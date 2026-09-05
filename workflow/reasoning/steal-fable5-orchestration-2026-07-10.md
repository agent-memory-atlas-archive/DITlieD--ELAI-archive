STEAL ANALYSIS — Fable 5 orchestration patterns + @Av1dlive nine-loops agentic OS
date: 2026-07-10. session posture: spec-eval (ELAI judged AS SPECIFIED; shipped-vs-planned never feeds a verdict).
source: workflow/reasoning/fable5-orchestration-sources-2026-07-10.md (SVG guide verbatim; Anthropic benchmark
numbers are TIER-3 third-party press citing Anthropic — 92%-quality-at-63%-cost SWE-bench Pro advisor,
96%-at-46% BrowseComp orchestrator-workers. [EXTERNAL] tags mandatory until confirmed against the primary
anthropic.com post. No arXiv IDs in this corpus.)
elai grounding: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md (read full); F-SUMMARY_INDEX.md:111-124;
H-SUMMARY_INDEX.md:121-150; I-SUMMARY_INDEX.md:141-153, 467-481; ELAI-FEATURE-MAP.md Section 4 ledger
(lines 3532-3724) checked — no verdict below re-proposes a ledger row.

DEDUP NOTE (ledger collisions checked)
- Track-20 decline "B9 cron/monitor/background-job tools (supervisor owns scheduling)" declined worker-facing
  cron TOOLS; steals 2/4/6 below are supervisor-owned triage/enrollment policy, not agent tools. No conflict.
- "code-review-as-a-tool REJECT (generator-side reviewer blurs auditor boundary)": steal 3 (advisor) is
  forward-looking guidance INTO the executor, advisory-never-the-gate (TD-3); the auditor boundary is untouched.
- "dual cross-model plan review = D-13 + gpt-bridge already": quorum (steal 2) is a WAKE gate, not plan review.

================================================================
ACTIONABLE STEALS (4)
================================================================

STEAL 1 — STANDING GOALS: "finishing IS the enrollment" (loop 05). Verdict: NEW (as a composition unit).
What: when a unit of work completes, the completion itself writes a standing goal — a deterministic predicate
(shell command / frozen checker, timeout-bounded) re-verified on a schedule forever after. exit 1 flips the
goal to VIOLATED, a sentinel names suspects (merges since last-pass) but fixes nothing, and the violation
re-enters the NORMAL work pipeline. Flaky predicate -> quarantine, never delete. "A goal you only verify once
is an assumption with a timestamp."
ELAI today (spec): every done-gate is verified ONCE at DONE. Continuous re-verification exists only
domain-scoped: F-5 staleness quarantine (data staleness), G model-drift canary (the exogenous model), C
anti-staleness automation (docs). There is NO general primitive that converts a done-gate's VERIFY artifact
into a persistent scheduled invariant. This is the exact rot class the wiring doctrine names ("a once-passing
check can't silently rot" — H context, but never mechanized corpus-wide).
Shape in ELAI terms: goal ledger = SSB rows (typed GoalEnrolled / GoalVerified / GoalViolated action_types),
schedule = O-plan scheduled TaskSpec substrate (typed cron/event -> inbox), predicate = the unit's existing
off-agent VERIFY artifact re-run (frozen checker exit code, R4-compliant), violation -> F-5-style quarantine +
re-open through the front-door router. Two designs considered: (a) new goal-ledger subsystem composing O
TaskSpecs; (b) add a cadence field to smoke-manifest/wiring-contract entries. Lean (a): (b) couples goal
lifecycle to the frozen gate fileset (R8 makes those read-only to runs, enrollment writes would violate the
freeze), while (a) keeps enrollment as journal data. Side effects: predicate-run cost per tick (bound it:
timeout 60s each, quiet-tick budget); flaky-predicate noise (quarantine tier absorbs).
PROOF_SPINE: feeds PS-6 posture indirectly (wire-dark regression would surface as a violated reachability
goal); primarily a trust-spine durability unit.

STEAL 2 — QUORUM WAKE-GATE: cheap cross-family voters before the expensive model wakes (loop 06).
Verdict: EXTENSION(O research daemon + 4G AutonomousGate + I-25 router).
What: on a scheduled/autonomous tick, N cheap cross-family models independently vote "actionable?" (blind to
each other's answers); the expensive conductor wakes only on majority yes. Quiet tick ≈ $0.03 vs waking the
flagship every tick. Uninstall trigger is MEASURED: "when the dispatch ledger shows wake-ups that ended in
action: stop" — the gate retires itself when its precision proves the filter redundant.
ELAI today (spec): D-13 cross-family adversary + D multi-verifier consensus exist for VERIFICATION; B REFLECT
per-message triage exists for chat. Nothing gates the WAKE of the expensive lane on scheduled ticks — 4G
advances on deterministic evidence but the evaluator model still runs. I-25 routes per-task AFTER wake.
Shape in ELAI terms: a triage stage in the O daemon / scheduled TaskSpec path: J free-API rotation pool
supplies the voters (cross-family, cost ~0), votes journaled as typed rows, conductor dispatch conditional on
quorum; wake-precision (wakes-ending-in-action / wakes) is a recomputable journal metric with the
install/uninstall threshold declared up front. Alternative considered: single cheap triager (loop 01 shape) —
cheaper but one family's blind spot becomes the system's; the cross-family blindness of 3 voters is the D-13
insight applied to attention. Voters are advisory-never-the-gate (TD-3): they gate COST, not correctness.
PROOF_SPINE: feeds PS-4 (routing economics $/verified-task — quiet-tick cost is a direct term).

STEAL 3 — ADVISOR LANE: cheap executor every turn, expensive judgment on demand (loop 03 / Anthropic
pattern A). Verdict: EXTENSION(H-7 + I-7 + I-25 + 4M).
What: the executor stays the cheap model for EVERY turn; the flagship is exposed as a budgeted tool the loop
calls roughly once per task when judgment is needed, advice flows back, executor continues. [EXTERNAL, tier-3]
~92% of flagship solo quality at ~63% cost on SWE-bench Pro.
ELAI today (spec): H-7 architect/editor/verifier = flagship reasons AT PLAN TIME then the editor executes
(up-front split); I-7 cascading escalation = cloud consulted ON FAILURE with a condensed BeliefState; I-25
routes BEFORE dispatch. The mid-loop, non-failure, executor-initiated "ask for judgment without switching
executors" slot is genuinely absent — it is the inverse of I-13 voluntary-abort (ask instead of give up).
Shape in ELAI terms: an advisor tool on the weak/local lane: call carries the I-11 BeliefState condensed
payload (<=2K tokens, same discipline as I-7), answer is journaled + advisory-never-the-gate (TD-3), budget
~1 call/task enforced deterministically (the H-9 spend ledger), over-ask rate measured like H-7's
over-escalation rate. Composes the K-2 routing-economics bench: add an advisor ARM next to the existing
local/cloud/hybrid arms so PS-4 gets the advisor $/verified-task number; the Anthropic figures become the
[EXTERNAL] baseline the bench is compared against, never claimed as ELAI numbers (HD-1/ID-5 discipline).
RECON_NEEDED (plan time): confirm the 92/63 + 96/46 figures against the primary anthropic.com post before any
plan file cites them even as [EXTERNAL].
PROOF_SPINE: feeds PS-4 directly.

STEAL 4 — NEVER ITERATE ON A SWAPPED MODEL (loop 01, exit code 2 "reroute"). Verdict: EXTENSION(4G
SUBSTITUTE_MODEL + 4M fallback chains + J free-API rotation) — doctrine rule + small enforcement.
What: when the runtime swaps the model mid-unit (fallback chain, rotation, substitution), the loop must NOT
continue iterating the new model on the old model's partial state — the tick exits with a distinct code and
the unit restarts fresh under the new model.
Why it's right: iteration state (partial diffs, self-consistency with earlier turns, implicit style
commitments) is model-conditioned; a swapped model inheriting it produces neither model's behavior and
poisons attribution (which model earned/lost trust-ledger credit — F-6, I-25 training rows — becomes
unanswerable). ELAI's ledger-trained router and per-skill trust tiers REQUIRE clean per-model attribution.
ELAI today (spec): 4G has SUBSTITUTE_MODEL as a fallback action and 4M has per-role fallback chains; the spec
does not state whether substitution restarts the unit or continues in-place. RECON_NEEDED: grep the 4G/4M
phase files + escalation_policy.rs for substitution semantics before authoring — if restart-fresh is already
the behavior, this downgrades to ALREADY (doctrine sentence only).
Shape in ELAI terms: substitution invalidates iteration state; restart from unit start with the condensed
BeliefState handoff (I-11) as the ONLY carried context; journal a ModelSwapRestart row so PS-4 cost
accounting sees the restart tax honestly.
PROOF_SPINE: protects PS-4/PS-5 attribution integrity (a mis-attributed outcome corrupts both benches).

================================================================
ALREADY-SHIPPED / MINOR (5)
================================================================

TRUST LEDGER, per-skill autonomy (loop 04): ALREADY-SHIPPED(F-6). SHARD T1-T4 trust ladder with promotion by
invocation-count + failure-rate + min-days, top tier human-gated, staleness demotion, and the tier GATES
AUTONOMY (S2 blocked from autonomous dispatch without human gate) — F-SUMMARY_INDEX.md:111-124 is the
watch->queue->auto ladder near 1:1, with a stronger provenance split (SkillTrustTier distinct from memory
TrustTier). Delta worth ONE line at F-6 execution time, not a unit: the loop's demotion is instant on ANY
fail with tier recomputed on every ledger write; F-6 says failure-rate + staleness — consider adding the
any-fail-demotes-immediately clause to the promotion predicate.

ORCHESTRATOR-WORKERS (loop 02 / Anthropic pattern B): ALREADY-SHIPPED(3C/3D BSP supersteps + typed channels,
4M per-role backend assignment, H cache economics for per-subagent cache). ELAI's version is stronger:
barrier-synchronized, journal-ordered, deterministic. The only net-new residue is the bench arm folded into
steal 3.

COMPOST, weekly failures become laws (loop 08): ALREADY-SHIPPED(3F rules engine self-discovery of recurring
violations with PROPOSED->SHADOW->ACTIVE promotion gates; 4H self-modification from measured outcomes; human
sign = interactive gate). Delta note folded into steal 1: compost's best idea ELAI lacks is proposing a
MISSING STANDING GOAL from the week's failure exhaust — that lands as a proposer feeding steal 1's ledger,
if/when steal 1 is authored. The <=3-proposals-per-week budget is congruent with existing gate discipline.

SPARRING, builder/breaker opposed daily (loop 07): EXTENSION but LOW priority. C mutation testing + D wave-3
adversarial probes + I tri-layer adversary already cover adversarial pressure; the net-new residue is only
the SCHEDULE (a diff-triggered breaker TaskSpec authoring <=1 failing test/day from merged diffs into a
dedicated test dir, builder fixes code never test, 2 disputes -> human). Cheap to author later as one O-plan
TaskSpec preset + D-13 dispatch; not worth a standalone unit now.

RATCHET, one number monotonic or self-reverting (loop 09): mostly ALREADY (F-10 per-iteration
security-monotonicity, I QDELTA + statistical floor, R8 frozen-config = "never edit the lint config"
structurally). Net-new residue is the WELD: when a burn-down reaches its floor, the floor auto-enrolls as a
standing goal — a rider ON steal 1, dependent on it, not separable.

CROSS-CUTTING PRINCIPLES, no steal needed: "nothing grades its own homework" + deterministic last gate =
ELAI's off-agent evidence + TD-3 + frozen checkers (ELAI is strictly stronger: hash-bound, journaled,
replayable vs tsv files + shell). "Everything is a ledger row" = SSB. Verifier sees spec+diff in fresh
context = I pristine-context injector + H-7 verifier-separate-context. Per-subagent prompt cache = H/I cache
economics.

================================================================
RECOMMENDED SEQUENCE (if authored)
================================================================
1. Steal 1 (standing goals) — the only genuinely new primitive; ratchet-weld + compost-proposer ride on it.
2. Steal 3 (advisor lane + K-2 advisor bench arm) — direct PS-4 feed, external baseline exists.
3. Steal 2 (quorum wake-gate) — PS-4 feed, cheap, but only matters once O-daemon scheduled ticks exist.
4. Steal 4 (swap-restart doctrine) — RECON first; possibly a one-line doctrine amendment, not a unit.
Target family: next open letter per index at authoring time (O/P are the last letters in the spec overview;
steals 1/2 naturally compose O's TaskSpec substrate — plausibly O-extensions rather than a new letter).

OUTCOME: analysis only this session (user asked for summary before authoring). No plan files created yet.

OUTCOME UPDATE 2026-07-10b (paper-to-plan Phase 5, main-LLM in-band): the four steals authored into NEW LETTER Q, Track ORCHESTRATION-ECONOMICS. STEAL 1 -> Q-1 (standing goals, KEYSTONE, feeds PS-6; ratchet-weld + compost-proposer registered as Tier-2 ride-alongs on the goal ledger). STEAL 2 -> Q-2 (quorum wake-gate, feeds PS-4). STEAL 3 -> Q-3 (advisor lane + K-2 advisor arm, feeds PS-4; the 92/63 + 96/46 figures WebSearch-confirmed 2026-07-10 as tier-3 press citing Anthropic, tagged [EXTERNAL], RECON-4 = confirm vs the primary anthropic.com post before the K-2 report cites them). STEAL 4 -> Q-4 (swap-restart; RECON on autonomous_gate.rs done 2026-07-10 CONFIRMED the gap: SubstituteModel swaps active_model + emits ModelSwap but discards no iteration state and records no restart tax; the task_runner consumer restart-vs-continue behavior stays Q-4 RECON-1 — if restart-fresh already holds, Q-4 collapses to a doctrine line + the ModelSwapRestart row). All four PASS plan_gate_check. Q-0_INDEX + Q-SUMMARY authored; ELAI-FEATURE-MAP + ELAI-SPEC-OVERVIEW upserted. No commit (docs gitignored).
