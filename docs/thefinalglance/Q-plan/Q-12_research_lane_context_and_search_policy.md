# Q-12: research-lane operator context compression + lineage-bandit search policy — cut per-operator prompt bloat, then allocate the freed budget with a bandit that must beat the deterministic scheduler

STATUS: PLANNED (WAVE AIDE2-2; GATED on Q-11's fixed-budget bench + baseline being green; two units in one file — UNIT 1 C3 operator-specific minimal-prompt matrix, UNIT 2 C1+C2 lineage-bandit allocation + plateau fork-on-stall as ONE search-policy experiment. No deep integration in this wave: the bandit ships as a measured experiment behind a flag, never default routing.)

Authoring provenance: paper-to-plan PHASE 5 over the AIDE² deep-research steal (conjecture engine run 2026-07-15; picks C3 then C1+C2, user-sequenced 2026-07-15 — "C2 alone could generate unnecessary branches; C1 alone could keep feeding exhausted arms; together they form a coherent policy"). Auditors do NOT read the engine run artifact — anchors below are re-greppable in-tree (W-7).

- Track: AIDE2-RESEARCH-INSTRUMENT-ECONOMICS | lever: minimal-context-then-bandit-allocation-under-fixed-budget | source: AIDE² reconstruction dossier §12 (draft-lineages-as-bandit-arms + fork-on-stall), §13 (operator-specific minimal context, 16x reduction, savings reinvested), §33-34 (plateau signal + fork criteria) | grounding: STRONG for the gap (premises grep-confirmed in-tree), [EXTERNAL] for every vendor number | nature: BUILD-on-shipped (C3 hardens the web orchestrator prompt path; C1+C2 add a selectable policy to the shipped research tree) | effort: M-L | WAVE AIDE2-2
- PROOF_SPINE: feeds PS-4 (C3 per-operator prompt-size reduction lowers the $/verified-task INPUT on the research lane; the freed budget is journaled and reinvested). The C1+C2 search-policy sub-unit is research-lane class (`none`) on its own — user-acked 2026-07-15 (in-session sequencing ruling authorizing this build IS the ack).

- Premise: ELAI's research lane concatenates node history into operator prompts with no per-operator field discipline and no summary/one-full-extract cap (`grep -icE "one_full|max_summaries|summary_cap|newest"` in `web_research_orchestrator.rs` = 0), and its `ResearchTree` allocates next-iteration spend with breadth/depth caps only — no arm statistics, no reward (`grep -icE "bandit|ucb|arm_"` = 0, `grep -ic reward` = 0, breadth_factor/depth_limit = 16 hits). AIDE² reports both levers as accepted discovered mechanisms: operator-specific minimal context cut average full-prompt size ~16x with savings reinvested as more search steps under a fixed budget [EXTERNAL https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement]; draft-lineages-as-bandit-arms with greedy-within-lineage and fork-on-stall gave a paired transfer of +0.042..+0.053 vs the baseline (agent-level, mechanism not isolated) [EXTERNAL, same URL]. The funnel: C3 is the cheap prompt-economy win; C1+C2 spends the freed budget smarter, but ONLY if a bandit beats a deterministic scheduler on Q-11's fixed-budget bench — "a poorly calibrated bandit can perform worse than a simple deterministic scheduler," so the unit ships its own control arm and kill criterion.

## Steal/build

UNIT 1 — C3 per-operator minimal-prompt matrix. For each research operator (FORMULATE / SEARCH / EXTRACT / REFLECT / CONVERGE) assemble a MINIMAL prompt: newest-12 one-line finding-summaries + exactly-ONE full-extract, with per-operator declared fields (extends the J declared-access-spec pattern, GD-1). Journal a prompt-size RATIO (before/after) per operator; the freed tokens are reinvested as extra tree iterations under the same budget. Ships default-ON behind an `ELAI_RESEARCH_MINIMAL_PROMPT_DISABLE` kill-switch (W-2). Realized delta will be SMALLER than the vendor 16x because ELAI already has 3E context slots — the claim is the journaled ratio, not the vendor number.

UNIT 2 — C1+C2 lineage-bandit search policy (ONE unit). Treat each draft-lineage (an initial-draft subtree or a fork root) as a bandit ARM; the outer selector chooses a lineage exploration-aware, then parent selection WITHIN the lineage stays greedy (best valid node). When the best lineage PLATEAUS (last-4 `info_gain` flat by slope+span, dossier §33), fork: copy global-best findings, root a NEW lineage with a fresh-angle sub-question stream, give it a finite exploration prior, retire arms that repeatedly fail, cap active lineages — a restart in STRATEGY space, not code-quality space (dossier §34). Ships as a SELECTABLE policy behind a default-OFF `ELAI_RESEARCH_BANDIT_ENABLE` (type=experiment) with a mandatory DETERMINISTIC-SCHEDULER control arm and a KILL criterion: if the bandit does not beat the deterministic scheduler beyond noise on Q-11's fixed-budget bench, it does not promote.

EXCLUDED: claiming a specific bandit formula is Weco's (§12.5 — the formula is undisclosed; the reconstruction uses a declared UCB1-or-Thompson choice, cited as ELAI's own); any default-routing change in this wave (experiment flag only); a bandit with unbounded new-arm bonuses (finite prior / mandatory initial pull, §34); passing private/hidden scores into any operator prompt (§13.4); deep integration before a second-benchmark transfer.

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)

- `ResearchTree`, `ResearchNode`, `info_gain`, `ConvergenceStatus` in `.elai_cc/crates/orchestrator/src/research_tree.rs`; breadth_factor/depth_limit present (16 hits), no bandit/reward/arm statistics, no re_root/plateau (`grep -icE "re_root|reroot|plateau"` = 0).
- `.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs`, DARK `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE`; no one-full-extract/summary-cap invariant today.
- Q-11 (this batch): the fixed-budget explore bench + the `unbounded_best_of_n`/`parallel_widening` control arms + the frozen pre-change baseline — Q-12's measurement apparatus and attribution t0. Q-12 does NOT re-implement it (GD-1).
- B-40 / Phase 4D `ResearchReport`/`SubQuestionResult` (the finding rows the newest-12 summaries compress) in `research_tree.rs`.

## REVISED scope (W-10 tiered)

- SHIP FIRST (C3, cheap): the per-operator field matrix + newest-12 + one-full-extract cap + the journaled prompt-size ratio + the `ELAI_RESEARCH_MINIMAL_PROMPT_DISABLE` kill-switch. Reinvest freed budget as iterations. Measured on Q-11's baseline.
- SHIP SECOND (C1+C2, gated): the lineage-bandit + plateau fork-on-stall as a selectable policy behind `ELAI_RESEARCH_BANDIT_ENABLE` (experiment) + the deterministic-scheduler control arm, benched against the control on Q-11's fixed-budget bench. Promotes to default only on a beyond-noise win on that bench AND a second benchmark (funnel: one controlled bench -> transfer -> only then deep integration). This wave stops at the first-bench result.

## Integration

- `crates/orchestrator/src/web_research_orchestrator.rs`: the per-operator prompt-assembly matrix + the prompt-size-ratio journal (C3).
- `crates/orchestrator/src/research_tree.rs`: the lineage/arm statistics, the exploration-aware selector, the plateau signal off `info_gain`, the fork-on-stall procedure, the deterministic-scheduler control arm, all behind the policy selector (C1+C2).
- `.elai_cc/wire-dark-allowlist.toml`: a row for `ELAI_RESEARCH_BANDIT_ENABLE` (type=experiment, owner, expiry_date, promotion_plan_id = a future Q/L promotion unit) — R9 (default-OFF enable needs a promotion plan).
- `.claude/tools/frontier_strategy_bench.py`: the Q-11 fixed-budget bench gains the bandit-vs-deterministic arm pair (reuse, not fork).
- `crates/orchestrator/src/ssb.rs`: reuse `usage_turn_event` for the reinvested-budget accounting + the prompt-size ratio FIELD (no new action_type if schema-legal; else at most +1 — RECON at edit, W-9).
- frontend_pair: N/A with justification — research-lane prompt assembly + a selectable search policy behind an experiment flag; no dashboard surface in this wave. Justified.

## MOVES (wargame)

M1 build the C3 per-operator minimal-prompt matrix and drive the web orchestrator with `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1` on a real multi-node research run.
EXPECTED_OBSERVATION: each operator prompt carries only its declared fields — newest-12 one-line summaries + exactly one full-extract; the journaled prompt-size ratio shows a measured reduction vs the Q-11 baseline; the kill-switch off restores the pre-change prompt.
ON_FAIL: prompts still concatenate multi-node history (the cap is bypassed on one operator path) -> the matrix missed an assembly site; grep the operator prompt builders and cap each; a ratio of ~1.0 means no compression happened, not that it is fine.
M2 reinvest the freed budget as extra tree iterations under the same fixed budget and confirm iteration count rises without total cost rising.
EXPECTED_OBSERVATION: iterations-per-budget increases vs baseline while `usage_turn_event` total cost stays within the fixed cap.
ON_FAIL: cost rises with iterations (the saving was illusory / double-counted) -> reconcile the token accounting against the ledger; do not claim reinvestment the ledger does not show.
M3 build the C1+C2 lineage-bandit + plateau fork-on-stall as a selectable policy behind `ELAI_RESEARCH_BANDIT_ENABLE`, with the deterministic-scheduler control arm, and run BOTH arms on Q-11's fixed-budget bench.
EXPECTED_OBSERVATION: the bench reports the bandit arm and the deterministic-scheduler arm at equal fixed budget; lineage arm statistics and reward are populated; a plateaued lineage forks a new strategy-space arm (a journaled event), not a prune-terminal.
ON_FAIL: the bandit spends more than the control (it wins by budget not policy) -> Q-11's per-dollar/`spend_leak` flag fires and the win is void; re-cap the bandit at the control's budget before comparing.
M4 apply the kill criterion on the M3 result.
EXPECTED_OBSERVATION: if the bandit beats the deterministic scheduler beyond Q-11's noise band at equal budget, it is a candidate for a SECOND benchmark; if it ties or loses, the experiment flag stays default-OFF and the decision doc records the null — the deterministic scheduler is retained.
ON_FAIL: a within-noise delta is read as a win -> re-apply the CI95 within-noise=TIE gate from `frontier_strategy_bench.py`; a coin-flip promotes nothing (§19 noise ~0.02-0.045 [EXTERNAL]).

## FORK

FORK: if the `info_gain` series is too sparse for a last-4 slope+span plateau signal (short subtrees terminate before 4 scores), take route B — define the plateau over the lineage's last-4 IMPROVEMENT deltas (parent-relative) instead of absolute scores, per dossier §33 normalized-delta note, and record the choice in the decision doc; do not fabricate scores to fill the window.
FORK: if `ResearchTree` allocation is structurally single-lineage (no subtree/lineage concept to attach an arm to), take route B — introduce the lineage id as a `ResearchNode` field threading to the draft/fork root, and treat C1+C2 as a larger unit; if that threading is structural it is a material fork -> STOP and handoff, do not ship a half-wired bandit.

## RECON_NEEDED

RECON_NEEDED-1: before C3, re-grep `web_research_orchestrator.rs` + its operator prompt builders for a FUNCTIONAL-equivalent summary/context cap already present (the engine's premise grep proved the NAMED tokens absent, not that no equivalent exists — inspect via `mcp codegraph-elai callees <operator prompt builder>` + `.claude/tools/elai.map`); a live cap collapses C3 to per-operator field declaration only.
RECON_NEEDED-2: before C1+C2, re-grep `research_tree.rs` for any existing lineage/arm/reward concept or an allocation policy beyond breadth/depth caps (same functional-equivalent caveat); confirm `info_gain` is per-node and readable for the plateau signal.
RECON_NEEDED-3: re-read the live `SCHEMA_VERSION`/`UsageTurnEvent` sites in `ssb.rs` + `action_type_lock.rs` at edit to choose reuse-field vs +1 for the prompt-size ratio (W-9); confirm the `wire-dark-allowlist.toml` schema fields (type/owner/expiry_date/promotion_plan_id) before writing the `ELAI_RESEARCH_BANDIT_ENABLE` row.

## ABORT

ABORT: any proposal that (a) makes the bandit default routing in this wave, (b) lets the bandit win on more spend rather than better allocation, (c) attributes a specific bandit formula to Weco, (d) passes hidden/private scores into an operator prompt, (e) gives new arms unbounded exploration bonuses, or (f) ships the bandit without the deterministic-scheduler control arm and the kill criterion -> STOP and flag. Each is an explicit dossier caveat or a funnel-discipline breach.

## WIRING_CONTRACT
produces_symbols:
  - web_research_orchestrator.rs — the per-operator minimal-prompt matrix + the prompt-size-ratio journal (C3)
  - research_tree.rs — lineage/arm statistics, exploration-aware lineage selector, plateau signal off info_gain, fork-on-stall, deterministic-scheduler control arm (C1+C2), behind the policy selector
  - a wire-dark-allowlist.toml row for ELAI_RESEARCH_BANDIT_ENABLE (type=experiment)
consumes_symbols:
  - research_tree.rs::ResearchTree / ResearchNode / info_gain / ConvergenceStatus (extend; do NOT re-implement, GD-1)
  - web_research_orchestrator.rs operator prompt builders (harden, not replace)
  - Q-11 frontier_strategy_bench.py fixed-budget bench + control arms (reuse as the comparison apparatus)
  - ssb.rs::usage_turn_event (read for the reinvested-budget + ratio accounting)
reachable_from:
  - `elai <research entrypoint>` / `ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1` drives C3's prompt path [PRODUCTION research orchestrator]
  - `python .claude/tools/frontier_strategy_bench.py` with `ELAI_RESEARCH_BANDIT_ENABLE=1` selecting the bandit arm vs the deterministic control [PRODUCTION explore-bench CLI]
  - the `research_tree.rs` allocation site selecting the policy (bandit vs deterministic) is the production caller of the bandit fn
  - a #[cfg(test)] caller NEVER satisfies reachability (R7)
stub_ban:
  - NO default-routing change this wave; the bandit is a default-OFF experiment flag with a promotion plan
  - NO bandit win on higher spend — equal fixed budget, Q-11 spend_leak enforced
  - NO attributing a bandit formula to Weco (undisclosed §12.5); the UCB1/Thompson choice is declared as ELAI's
  - NO private/hidden score in any operator prompt; NO unbounded new-arm bonus (finite prior, §34)
  - NO ratio claimed that the usage_turn_event ledger does not show; NO blake2/sha2
  - NO producer-only bandit fn — the allocation site + the bench arm are wired same changeset (R6)
frontend_pair:
  N/A with justification — research-lane prompt + selectable search policy behind an experiment flag; no dashboard surface this wave. Justified.
observable_effect:
  trigger: "run a real multi-node research run (ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE=1) then `python .claude/tools/frontier_strategy_bench.py` for the bandit and deterministic arms at equal budget"
  expect_ssb: "the prompt-size ratio and reinvested-iteration accounting recompute from usage_turn_event rows (GD-2); the bandit arm's per-dollar figures show no spend_leak vs the deterministic control"
  expect_artifact: "a bench report with the bandit-vs-deterministic arm pair at equal fixed budget + a journaled prompt-size ratio per operator. NEGATIVE: a bandit win with spend_leak=true, a within-noise delta read as a win, an operator prompt still carrying full node history, or a bandit shipped as default routing must FAIL"

PRODUCTION_HOOK: producer C3 minimal-prompt matrix -> consumer the operator prompt-assembly path in `web_research_orchestrator.rs`, wired same changeset; producer C1+C2 bandit policy -> consumer the `research_tree.rs` allocation/policy-selection site AND the `frontier_strategy_bench.py` bandit arm, wired same changeset behind `ELAI_RESEARCH_BANDIT_ENABLE` with its allowlist row. No producer-only fn satisfies the hook.

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-12_research_lane_context_and_search_policy.md --run-trigger
rg -c "summary_cap|one_full|newest" .elai_cc/crates/orchestrator/src/web_research_orchestrator.rs   # MUST be >0
rg -c "bandit|lineage|plateau|reward" .elai_cc/crates/orchestrator/src/research_tree.rs             # MUST be >0
rg -c "ELAI_RESEARCH_BANDIT_ENABLE" .elai_cc/wire-dark-allowlist.toml                                # MUST be >0 (R9)
test -f workflow/reasoning/Q-12-decisions.md
```

## VERIFY

VERIFY: before declaring Q-12 done, the auditor runs off-agent —
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-12_research_lane_context_and_search_policy.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`
`python .claude/tools/frontier_strategy_bench.py` for the bandit and deterministic arms, confirming equal budget + no spend_leak + the CI95 within-noise=TIE verdict on the delta
plus a re-check that `ELAI_RESEARCH_BANDIT_ENABLE` has an allowlist row (R9) and defaults OFF. Pass = wire-checker clean (or WIRE-DARK[Q-12] rows), the C3 ratio journal-recomputable (GD-2), and the bandit either winning beyond noise at equal budget OR staying default-OFF with a recorded null. Cargo-green alone is necessary-not-sufficient.

INTEGRATION_SMOKE: an INDEPENDENT agent drives a HARD real research task (a multi-sub-question run long enough to trigger a plateau + a fork-on-stall) end-to-end: with C3 on, the operator prompts are minimal and the freed budget buys extra iterations at the same cost; with the bandit arm on, a plateaued lineage forks a new strategy-space arm and the bandit is compared to the deterministic scheduler at equal budget with the spend_leak guard live. A single-sub-question toy run does not fire the plateau path and does not satisfy R10.

## REDTEAM

REDTEAM: attack that SUCCEEDED against an earlier framing — "ship C2 (fork-on-stall) alone and generate a burst of unnecessary new branches" and separately "ship C1 (bandit) alone and keep feeding an exhausted arm": patched by fusing them into ONE policy — the bandit chooses the lineage, greedy refines within it, a four-step plateau triggers the fork, and arm retirement + a finite new-arm prior bound the branching (§34); neither failure mode survives the coupled policy. Attack that FAILED — "make the bandit look better by giving it a bigger budget": FAILS because Q-11's fixed-budget bench caps both arms equally and flags a spend-more win as `spend_leak`. Attack that FAILED — "promote on a few-thousandths delta": FAILS because the CI95 within-noise gate treats it as a TIE (§19 noise ~0.02-0.045 [EXTERNAL]). Attack that FAILED — "quietly make the bandit the default so it ships even on a null": FAILS because it is a default-OFF experiment flag with an allowlist promotion plan and the kill criterion retains the deterministic scheduler on a null.

## Grounding

- AIDE² reconstruction dossier §12 (draft-lineages-as-bandit-arms, greedy-within-lineage, fork-on-stall as a strategy-space restart; §12.5 the exact formula is undisclosed — never attribute UCB1 to Weco), §13 (operator-specific minimal context; ~16x average full-prompt reduction; savings reinvested under the fixed budget), §17.1 (paired transfer +0.053 / +0.042 vs baseline, agent-level, mechanism not isolated), §33-34 (plateau signal off last-4 scores; fork criteria + finite new-arm prior + arm retirement). ALL vendor-reported, unreleased report, no ablation: [EXTERNAL https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement]. Ground the DESIGN, never an ELAI number.
- ELAI ground truth: `research_tree.rs` (ResearchTree/ResearchNode/info_gain/ConvergenceStatus, breadth/depth caps only, no bandit/reward/plateau), `web_research_orchestrator.rs` (DARK, no summary cap). Re-greppable in-tree (W-7).
- CLAUDE.md hard rules: no .unwrap/.expect on prod; scoped tests; GD-2 journal-recomputable; W-2 kill-switch (C3) vs R9 allowlisted experiment flag (C1+C2); [EXTERNAL] tags never asserted or summed (HD-1/KD-1).

## COMPOSES-WITH

Q-11 fixed-budget explore bench + control arms + baseline (the measurement apparatus and t0), B-40/Phase 4D ResearchReport/SubQuestionResult, the J declared-access-spec pattern (per-operator fields), `frontier_strategy_bench.py`, J-28 usage_turn_event. Re-implements NONE (GD-1). The only new objects are the per-operator field matrix and the lineage-bandit policy; both compose the shipped research tree + web orchestrator.

## Open questions / risks

- Realized C3 reduction is bounded by ELAI's existing 3E context slots — expect well under the vendor 16x; the claim is the journaled ratio, not the number (GD-5 honest residue).
- A poorly-calibrated bandit can lose to the deterministic scheduler under a strict budget (§19 lesson) — the control arm + kill criterion make that a recorded null, not a shipped regression.
- The reward definition (absolute vs marginal info gain), uncertainty representation, cheap-vs-expensive branch comparison, delayed-reward credit, arm retirement, and coexistence with the shipped breadth/depth caps are decided at authoring against the live `research_tree.rs` (RECON-2); a mis-set choice is caught by the kill criterion, not shipped.

## Next

Closes the research-lane prompt-economy gap (C3, cheap, PS-4) and lands the lineage-bandit as a measured experiment gated on Q-11's honest bench (C1+C2). A beyond-noise win at equal budget authors its own second-benchmark + promotion unit; a null closes the search-policy line honestly with the deterministic scheduler retained. Pairs with Q-11 (its apparatus) and Q-13 (the trust sibling).

End of Q-12.
