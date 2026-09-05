# Q-3: advisor lane — the cheap executor asks the flagship for judgment mid-loop without switching executors, plus the K-2 advisor bench arm

STATUS: PLANNED (WAVE 2; EXTENSION(H-7 + I-7 + I-11 BeliefState + I-25 router + 4M + H-9 spend ledger + K-2 bench); BUILD-on-shipped — the mid-loop advisor tool + the routing-economics bench arm)

- Track: ORCHESTRATION-ECONOMICS | lever: cheap-executor-every-turn-expensive-judgment-on-demand | source: steal-fable5-orchestration STEAL 3 (loop 03 / Anthropic pattern A) | grounding: MODERATE (index-derived + [EXTERNAL] baseline WebSearch-confirmed 2026-07-10) | nature: EXTENSION | effort: M-L | WAVE 2
- PROOF_SPINE: feeds PS-4 (adds an ADVISOR arm to the K-2 $/verified-task bench alongside local/cloud/hybrid; the advisor $/verified-task number is a direct PS-4 term).

- Premise (steal-fable5-orchestration STEAL 3): the executor stays the CHEAP model for EVERY turn; the flagship is a BUDGETED TOOL the loop calls ~once/task when judgment is needed, advice flows back, the SAME executor continues. This is genuinely absent in ELAI: H-7 splits reasoning UP-FRONT (flagship at plan time, editor executes); I-7 escalates ON FAILURE with a condensed BeliefState; I-25 routes BEFORE dispatch. The mid-loop, non-failure, executor-INITIATED "ask for judgment without switching executors" slot — the inverse of I-13 voluntary-abort (ask instead of give up) — has no home. [EXTERNAL, tier-3, WebSearch-confirmed 2026-07-10 across the-decoder/XiaoHu/developersdigest citing Anthropic; NOT the primary anthropic.com post]: Sonnet-executor + Fable-advisor ~= 92% of Fable-solo quality at ~63% cost on SWE-bench Pro; a cookbook 20-fact verify ran $1.61/194s split vs $4.00/608s solo. These are the [EXTERNAL] BASELINE the K-2 arm is compared against, NEVER claimed as ELAI numbers (HD-1/ID-5).

## Steal/build
An advisor tool on the weak/local lane: the executor calls it with the I-11 BeliefState condensed payload (<=2K tokens, same discipline as I-7), the flagship answers, the answer is journaled (`AdvisorConsulted{task_id, belief_hash, advice_hash}`) and is advisory-never-the-gate (TD-3 — advice informs the executor, it is never an off-agent verdict), budget ~1 call/task enforced deterministically via the H-9 spend ledger, and the over-ask rate is measured exactly like H-7's over-escalation rate. The K-2 side: add an ADVISOR arm next to the existing local/cloud/hybrid arms so PS-4 gets the advisor $/verified-task number; the [EXTERNAL] Anthropic figures become the tagged baseline the arm is compared against.
EXCLUDED: switching the executor on an advisor call (the whole point is SAME executor continues — a switch is I-7 escalation, a different mechanism, and it poisons per-model attribution, see Q-4); advice that gates/blocks the run (TD-3 — the advisor is a consultant, not an auditor; if advice could block, it would be an in-boundary self-review); an unbounded advisor budget (the H-9 spend-ledger cap is what keeps the economics honest and measurable).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- H-7 architect/editor/verifier split — flagship-at-plan-time; Q-3 is the mid-loop complement, not a replacement.
- I-7 cascading escalation with a condensed BeliefState + I-11 BeliefState (<=2K) — the payload shape Q-3 reuses verbatim.
- I-25 ledger-trained router — routes before dispatch; the advisor is a within-dispatch tool.
- 4M RuntimeAdapter — exposes the flagship as a callable backend the advisor tool targets.
- H-9 spend ledger — the deterministic budget enforcer (1 call/task).
- K-2 $/verified-task bench (local/cloud/hybrid arms) — the bench Q-3 adds an arm to; K-4 gate-tax is the sibling cost instrument.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the advisor tool on the weak/local lane (BeliefState payload -> flagship -> journaled advice), the H-9-enforced 1-call/task budget, `AdvisorConsulted` journaling, and the K-2 ADVISOR arm recomputing advisor $/verified-task against the [EXTERNAL] baseline.
- DEFER (Tier-2): the over-ask-rate governor (auto-tighten the budget when the measured over-ask rate exceeds H-7's over-escalation threshold) — land after the arm has a real corpus.

## Integration
- crates/orchestrator: the advisor tool exposed to the weak-lane loop (a bounded call, not a lane switch); the BeliefState condenser reuse; the H-9 budget check; the `AdvisorConsulted` emit.
- crates/cli: the K-2 bench `--arm advisor` recompute.
- frontend_pair: N/A with justification — advisor consults surface as SSB rows + the K-2 bench report; no new panel. Justified.

## MOVES (wargame)
M1 add the advisor tool: the weak-lane executor calls it with the I-11 BeliefState (<=2K), the 4M-exposed flagship answers, one `AdvisorConsulted{task_id, belief_hash, advice_hash}` row is journaled; the executor does NOT change.
EXPECTED_OBSERVATION: an advisor call emits one `AdvisorConsulted` row, the executor model_id BEFORE and AFTER the call is identical, the BeliefState payload is <=2K tokens.
ON_FAIL: the executor switches models on an advisor call -> attribution poison + wrong mechanism -> keep the executor fixed; the advisor is a tool return, not a dispatch; assert executor-id invariance across the call.
M2 enforce the ~1-call/task budget via the H-9 spend ledger; a second advisor call in the same task is refused (or counted against a declared over-budget policy) and the over-ask is journaled.
EXPECTED_OBSERVATION: the first advisor call in a task proceeds; a second is refused/flagged with an `AdvisorConsulted{over_budget:true}` (or equivalent) row; the budget is read from the H-9 ledger, not a local counter.
ON_FAIL: the budget is a local per-loop int -> not deterministically enforced across resumes -> read/write the H-9 spend ledger; assert the budget survives a resume.
M3 add the K-2 ADVISOR arm: `elai bench routing-economics --arm advisor` recomputes advisor $/verified-task from the journal and reports it beside local/cloud/hybrid, with the [EXTERNAL] Anthropic 92/63 figure tagged as the baseline (never as an ELAI number).
EXPECTED_OBSERVATION: the bench emits an advisor arm number recomputed from `AdvisorConsulted` + verified-task rows; the report tags the Anthropic baseline [EXTERNAL] and does not sum it with ELAI's number.
ON_FAIL: the [EXTERNAL] baseline is printed as an ELAI result -> claims-policy violation (HD-1/KD-1) -> tag [EXTERNAL] in the report + the claims-lint; the arm reports ELAI's own recomputed number only.

## FORK
FORK: if 4M does not expose the flagship as a synchronous callable the weak lane can await mid-loop (only as a full lane switch) -> ship Tier-1 as a bounded sub-call that returns advice WITHOUT changing the loop's active executor record (a tool-return shape), and record the seam; if even that is structurally a switch, this is a material fork -> STOP + handoff (do not ship an advisor that is secretly an executor swap).

## RECON_NEEDED
RECON_NEEDED-1: the weak-lane loop seam where a mid-loop tool call returns to the SAME executor -> settle via `grep -rn "executor\|loop\|tool.call\|advisor" .elai_cc/crates/orchestrator/src` + codegraph_explore on the dispatch loop.
RECON_NEEDED-2: the I-11 BeliefState condenser API (how to produce the <=2K payload) -> settle via `grep -rn "BeliefState\|belief\|condense" .elai_cc/crates`.
RECON_NEEDED-3: the H-9 spend-ledger check/charge API + the K-2 bench arm registration -> settle via `grep -rn "spend.ledger\|budget\|bench.*routing\|verified.task" .elai_cc/crates/orchestrator/src .elai_cc/crates/cli/src`.
RECON_NEEDED-4 (claim honesty): re-confirm the 92/63 + 96/46 figures against the PRIMARY anthropic.com post before the K-2 report cites them even as [EXTERNAL] (WebSearch 2026-07-10 confirmed the tier-3 press; the primary post is the settling source).

## ABORT
ABORT: if the advisor's advice would be wired as an off-agent VERDICT that can PASS/BLOCK a run (not just inform the executor) -> STOP + handoff. The advisor is advisory-never-the-gate (TD-3); an advisor that grades the run is an in-boundary self-review and is the vetoed design.

## WIRING_CONTRACT
produces_symbols:
  - the weak-lane advisor tool (BeliefState payload -> flagship -> journaled advice, same executor)
  - `AdvisorConsulted` SSB row (baseline+1; re-read at edit W-9)
  - the K-2 `--arm advisor` recompute + the [EXTERNAL]-tagged baseline comparison
consumes_symbols:
  - I-11 BeliefState condenser + I-7 payload discipline; 4M RuntimeAdapter (flagship backend); H-9 spend ledger (budget); K-2 routing-economics bench; H-7 over-escalation-rate idiom
reachable_from:
  - the weak-lane loop invokes the advisor mid-task [PRODUCTION executor loop]; `elai bench routing-economics --arm advisor` [PRODUCTION bench CLI]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO executor switch on an advisor call (same executor continues; TD attribution)
  - NO advisor verdict that gates/blocks a run (advisory-never-the-gate; TD-3)
  - NO local budget counter (H-9 spend ledger, resume-durable)
  - NO [EXTERNAL] Anthropic figure printed as an ELAI number or summed with one (HD-1/KD-1)
observable_effect:
  trigger: "run a weak-lane coding task that hits one judgment call, then `elai bench routing-economics --arm advisor`"
  expect_ssb: "one AdvisorConsulted row with executor-id invariant across the call; a second call in-task is over_budget-flagged"
  expect_artifact: "a journal-recomputed advisor $/verified-task in the bench report with the Anthropic baseline tagged [EXTERNAL]; NEGATIVE: an advisor call that swaps the executor, or advice printed as a PASS verdict, must FAIL"

PRODUCTION_HOOK: producer the advisor tool -> consumer the weak-lane loop seam (RECON-1) wired same-changeset; producer the K-2 arm -> consumer the routing-economics bench CLI (RECON-3); the `AdvisorConsulted` row + the bench report are the off-agent artifacts.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-3_advisor_lane.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
`python .claude/tools/claims_policy_lint.py --files <the K-2 report copy> --json`   # [EXTERNAL] baseline tagged, not asserted as ELAI's
pass = an `AdvisorConsulted` row landed off-agent from a real weak-lane task with an invariant executor id, the H-9 budget held across a resume, and the K-2 advisor arm reported a journal-recomputed number with the Anthropic baseline tagged [EXTERNAL].

INTEGRATION_SMOKE: an independent agent runs a real multi-turn weak-lane coding task that hits >=1 genuine judgment call (a HARD input), asserts exactly one advisor consult with the executor unchanged, then runs the K-2 advisor arm end-to-end — a full task + bench recompute, not a stubbed consult (R10).

## REDTEAM
REDTEAM: attack that FAILED — "let the advisor quietly become the auditor so the cheap lane rubber-stamps itself": FAILS because TD-3 + the ABORT + stub_ban scope advice to the executor's INPUT; the off-agent verdict stays with the frozen auditor gates, which the advisor cannot emit. Attack that SUCCEEDED and its patch — "inflate the win by reporting the [EXTERNAL] 92/63 as ELAI's measured result": patched by M3 + the claims_policy_lint gate that BLOCKS an untagged external number; the arm reports only ELAI's own journal-recomputed $/verified-task.

## Grounding
- steal-fable5-orchestration-2026-07-10.md STEAL 3 (loop 03 / Anthropic pattern A) — the mid-loop advisor slot, the I-11 payload reuse, the K-2 arm.
- [EXTERNAL] Anthropic advisor/orchestrator figures WebSearch-confirmed 2026-07-10 (the-decoder.com, best.xiaohu.ai, developersdigest.tech, all citing Anthropic; RECON-4 = confirm against the primary anthropic.com post before the report cites them).
- H-7/I-7/I-11/I-25/4M/H-9/K-2 — feature-map-confirmed at authoring, executor re-greps (W-7).

## COMPOSES-WITH
I-11 BeliefState + I-7 payload discipline, 4M RuntimeAdapter, H-9 spend ledger, K-2 routing-economics bench, H-7 over-escalation idiom. Re-implements NONE (GD-1). Pairs with Q-2 (both are cheap-model-in-the-loop cost levers feeding PS-4) and Q-4 (Q-4 protects the per-model attribution the advisor's same-executor invariant depends on).

End of Q-3.
