# H-47: DGM eval firewall — date-gate + template-regeneration + holdout-budget binding so the loop cannot mine its own eval [TRACK 7]

STATUS: PLANNED (TRACK 7 DGM SELF-IMPROVEMENT VERIFICATION HARDENING; WAVE 7; W-10 tiered: in-process date-gate + template-regen + budget-binding Tier-1; the live large-corpus eval-indistinguishability sweep is a separately-gated checkbox)

Objective: even with H-43's real fitness, H-44's detector and H-45's worst-case scoring, the loop's eval TRUST is
the eval's trust — and a self-improver firing thousands of "did my score go up?" queries IS adaptive analysis that
WILL overfit a static holdout (Thresholdout, 1506.02629), date-leaked items it already saw (LiveCodeBench 2403.07974),
re-used template instances (GSM-Symbolic 2410.05229), and a recognizable eval it can sandbag (eval-awareness
2505.23836 AUC 0.83 / 2510.20487 >80%). ELAI ALREADY PLANS the store + the n-gram/cosine filter + cutoffs +
eval-awareness probes (D-6 + E-54) and the DP holdout budget (H-19). H-47 does NOT re-implement any of them (GD-1
P0). It COMPOSES them into a DGM-loop firewall and adds only the net-new wiring: (1) a DATE-GATE tying D-6's
date-tags to the darwin generation timestamp so the loop only scores on items NEWER than its last config update;
(2) GSM-Symbolic-style TEMPLATE REGENERATION at score-time so the loop never sees the exact instance twice; (3)
BINDING H-19's Thresholdout budget to the loop's adaptive holdout queries. This is GAP-5 made loop-specific (VD-5).

Estimated effort: ~1 session (M, ~6-8 h). W-10 SPLIT: Tier-1 = the in-process date-gate + the template-regenerator +
  the H-19 budget binding + the EvalFirewallEvent SSB row over a FROZEN dated/templated fixture (fail-closed done-
  gate); Tier-2 = the live eval-indistinguishability sweep over a real workload-trace corpus (fixture now, live a gated checkbox).
Pre-requisites: H-43 (the real fitness the firewall protects). COMPOSES D-6 (held-out store + date-tags), E-54
  (13-gram + cosine + cutoffs + eval-awareness), H-19 (Thresholdout DP budget). Reads DG-6, PC-3/PC-4, VD-5.
Parallel-safe with: H-45, H-46 (disjoint files). Sequence AFTER H-43; coordinate with D-6/E-54/H-19 authors on the shared store schema.
Source: ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-5 P1; DOSSIER 6 LiveCodeBench/GSM-Symbolic/GSM1k/
  Thresholdout/eval-awareness; PART C/F step 3); _grounding/H-VERIFICATION-GROUNDING-MAP.md PC-3/PC-4 + VD-5.

## Context

DOSSIER 6's mechanisms: LiveCodeBench (2403.07974) — held-out items carry a release TIMESTAMP; score only items
NEWER than the last update (the date-gate is the firewall). GSM-Symbolic (2410.05229) — regenerate each eval instance
from a TEMPLATE at score time so the loop never sees the exact instance (Apple templates; up to 65% drop when one
irrelevant clause is added). GSM1k (2405.00332) — the held-out store is private + release-gated. Thresholdout /
Reusable Holdout (1506.02629, Dwork et al. Science 2015) — a DP mechanism bounds adaptive-query overfitting to O(1)
INDEPENDENT of query count; a self-improver's thousands of score-queries are exactly adaptive analysis. Eval-awareness
(2505.23836 / 2510.20487) — a strong model detects "this is the eval" and sandbags; eval items must be
indistinguishable from real workload traces. ELAI's D-6/E-54 already build the store + filter + cutoffs + awareness
probes; H-19 builds the DP budget. H-47's net-new is binding these to the DGM loop's generation clock and adding
the template-regen leg — necessary-not-sufficient (a strong improver can still sandbag; VD-5).

## Existing state (B/C/D/E/F/G coverage)
- D-6 held-out store (.elai_cc/held-out/{date}/{task}.toml; date-tags; auto-retire >6mo uniformly-passing; SeededClock C-15) — PLANNED. The store + date-tags H-47 date-GATES (compose, do not rebuild).
- E-54 13-gram + embedding-cosine + per-family shards + cutoffs:{model:YYYY-MM} + test-triplet + eval-awareness probes — PLANNED. The filter + cutoffs + awareness H-47 reads (compose, do not rebuild).
- H-19 reusable-holdout budget (Thresholdout/Ladder DP, HoldoutBudgetEvent QuerySpent/Exhausted/Provisioned, mutable=0) — PLANNED (Track 3). The DP budget H-47 BINDS to the loop's queries (REUSE the variant — GD-1).
- agent_pool.rs:1745 sample_held_out_task — SHIPPED. The held-out sampling surface the date-gate wraps (DG-6).
- darwin_admit_loop generation timestamp / config-update clock — SHIPPED (the loop runs generations). The date-gate's reference clock.
- A DGM-loop date-gate + a score-time template regenerator + the H-19-budget-to-loop binding — ABSENT. The net-new wiring surface.

## Reads (mandatory before coding)
- _grounding/H-VERIFICATION-GROUNDING-MAP.md: DG-6, PC-3, PC-4, VD-5, the SCHEMA + CITATIONS blocks.
- docs/thefinalglance/D-plan/D-6_held_out_task_store.md + docs/thefinalglance/E-plan/E-54_eval_corpus_design.md (the store + filter + cutoffs — compose targets)
- docs/thefinalglance/H-plan/H-19_reusable_holdout_budget.md (the DP budget variant to bind — REUSE)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\agent_pool.rs:1745 (sample_held_out_task — the surface to date-gate)
- workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-5 + DOSSIER 6 + PART C/F)
- LiveCodeBench 2403.07974 + GSM-Symbolic 2410.05229 + Thresholdout 1506.02629 + eval-awareness 2505.23836 PDFs.
- CLAUDE.md HARD RULES: blake3 (item/template hash), no .unwrap()/.expect(); SeededClock (C-15) for the date reference (no wall-clock).

## Tasks (WAVE 7 — Tier-1 date-gate + template-regen + budget-binding + measured row)

1. **Date-gate over sample_held_out_task (~70 LOC, crates/orchestrator/src/eval_firewall.rs).** Wrap the held-out
   sampling so the loop only scores on items whose D-6 date-tag is NEWER than the darwin generation's last config-
   update timestamp (SeededClock C-15, no wall-clock). A pre-update item is REJECTED (PreUpdateItemRejected) — it may
   be memorized. Compose D-6's date-tags; do not re-implement the store.

2. **Score-time template regeneration (~80 LOC).** For templated tasks (GSM-Symbolic style), regenerate a FRESH
   instance from the template at score time (seeded by the generation id), so the loop never scores the exact instance
   twice (TemplateRegenerated). The template set is gate-frozen (R8/TD-2). Compose E-54's test-triplet schema for the
   regenerated instance's tests.

3. **Bind the H-19 Thresholdout budget to the loop's queries (~40 LOC).** Route every DGM score-query through H-19's
   DP budget; after the budget is spent the loop's holdout queries are REFUSED (H-19's Exhausted) until a fresh
   holdout is provisioned via the frozen path. REUSE H-19's HoldoutBudgetEvent variant (GD-1; no new budget mechanism).

4. **Indistinguishability probe (~40 LOC).** Sample the eval-awareness probe (E-54 / D-23) over the loop's eval items
   and emit an IndistinguishabilityProbe measurement (how distinguishable eval items are from real workload traces);
   a high distinguishability is a sandbagging risk surfaced (VD-5), not silently ignored. Compose E-54; do not rebuild the probe.

5. **Off-agent measured row (~30 LOC).** Emit `{kind, items_scored, items_rejected, regen_count, budget_remaining,
   distinguishability, seed}` as an EvalFirewallEvent SSB row (Pattern-2; Kind PreUpdateItemRejected /
   TemplateRegenerated / IndistinguishabilityProbe; baseline+1 from LIVE 224/501, re-read at edit; reconcile e45 pin
   if first Track-7 bump). The budget leg's row REUSES H-19's HoldoutBudgetEvent. Reuse the InsecureRateMeasured pattern (GD-1).

6. **Decision doc workflow/reasoning/H-47-decisions.md (~80 LOC):** the date-gate (LiveCodeBench), template-regen
   (GSM-Symbolic), the Thresholdout adaptive-query argument, the eval-awareness residue; what is COMPOSED (D-6/E-54/
   H-19) vs net-new (the wiring + regen); the honest residue (a strong improver can still sandbag — indistinguishability mitigates, does not cure; VD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/eval_firewall.rs::date_gated_sample (the date-gated held-out sampler)
  - crates/orchestrator/src/eval_firewall.rs::regenerate_instance (the score-time template regenerator)
  - crates/orchestrator/src/ssb.rs::SsbActionType::EvalFirewallEvent (off-agent firewall artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/agent_pool.rs::sample_held_out_task (the held-out surface the date-gate wraps — DG-6)
  - the D-6 held-out store date-tags + E-54 cutoffs/test-triplet (compose — read the store, do not re-implement, GD-1)
  - the H-19 holdout-budget API + HoldoutBudgetEvent (REUSE the DP budget + variant — GD-1)
  - crates/orchestrator/src/execution_fitness.rs (H-43 — the scorer the firewall feeds clean items to)
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs (the measured-row pattern — mirror, GD-1)
reachable_from:
  - the self-mod loop eval step (darwin_admit_loop::run_one_generation -> date_gated_sample/regenerate_instance -> execution_grounded_fitness; queries pass H-19's budget) [PRODUCTION, darwinian kill-switch, allowlisted]
stub_ban:
  - NO scoring on a pre-update (date-leaked) held-out item (the date-gate is mandatory) — FAIL
  - NO scoring the exact templated instance twice when a template exists (template-regen mandatory) — FAIL
  - NO unbounded adaptive holdout querying (the H-19 DP budget is mandatory; after Exhausted -> refuse) — FAIL
  - NO re-implementation of the store / filter / budget (compose D-6/E-54/H-19 — GD-1 P0)
  - the broken `--eval-ignore-dates` arm MUST let a pre-update memorized item inflate the score (R2)
  - blake3 item/template hash; no blake2/sha2; no .unwrap()/.expect(); SeededClock not wall-clock
frontend_pair:
  the firewall stats (items rejected, regen count, budget remaining, distinguishability) in the DGM/trust drawer (composes H-9 / the archive panel; cross_lang_diff PASS).
  W-6 E2E: a pre-update item being REJECTED (and a post-update item scored) is visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && ELAI_DARWINIAN_LOOP_ENABLE=1 cargo run -p elai-cli -- <the self-mod generation subcommand>
    over a FROZEN held-out fixture with a pre-update dated item P, a post-update item Q, and a templated item T, NO other env override"
  expect_ssb: "an EvalFirewallEvent shows P PreUpdateItemRejected, Q scored, T TemplateRegenerated (fresh instance);
    a HoldoutBudgetEvent spends per query and refuses after the budget; recomputable under ELAI_SEED (GD-2)"
  expect_artifact: "the generation's score uses Q + the regenerated T, never P; the broken `--eval-ignore-dates` arm
    scores P and inflates the fitness, proving the date-gate is load-bearing (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-47_dgm_eval_firewall_dategate_templateregen.md --run-trigger
rg "date_gated_sample|regenerate_instance|EvalFirewallEvent" .elai_cc/crates/orchestrator/src --glob '!*test*'
rg "HoldoutBudgetEvent" .elai_cc/crates/orchestrator/src/eval_firewall.rs   # H-19 budget REUSED (GD-1), not re-implemented
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/eval_firewall.rs       # MUST be 0
test -f workflow/reasoning/H-47-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (P rejected, Q scored, T regenerated, budget binds). cargo-green != DONE.
- [ ] W-2: the firewall is the default eval path under the darwinian kill-switch; no new default-off gate.
- [ ] W-3 production entrypoint: the firewall fires from the loop's eval step, not a #[cfg(test)] caller.
- [ ] VD-5: the eval-awareness/sandbagging residue is NAMED + surfaced (indistinguishability probe), not claimed cured.
- [ ] GD-1: D-6 store + E-54 filter + H-19 budget COMPOSED, not re-implemented (P0 if violated).
- [ ] GD-2 reproducible: items-rejected / regen-count / budget-spend re-compute byte-identically under ELAI_SEED.
- [ ] R2: the --eval-ignore-dates broken arm inflates the score with a pre-update item (falsifiable).
- [ ] W-10: Tier-1 date-gate + template-regen + budget-binding is the fail-closed done-gate; the live indistinguishability sweep is a separately-gated checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the date-gate/regen (produced) feed the scorer (W-8) reached by the loop eval step (W-3).
- W-2: default eval path under kill-switch; allowlisted.
- W-3: live loop eval step; not test-only.
- W-4: frontend_pair = the firewall-stats tile; cross_lang_diff PASS.
- W-5: observable_effect is a runnable generation over a frozen dated fixture -> real journal rows; Tier-1 in-process.
- W-6: a pre-update item rejected is visible in the dashboard (dashboard-smoker).
- W-7: sample_held_out_task / the D-6 store / E-54 cutoffs / H-19 budget / execution_fitness / safedelta_runner grep-confirmed (DG-6/PC-3/PC-4); the firewall wiring ABSENT.
- W-8: date_gated_sample + regenerate_instance (produced) consumed by the scorer; the budget gates the queries.
- W-9: +EvalFirewallEvent baseline+1 (Pattern-2); the budget leg reuses H-19's HoldoutBudgetEvent (no 2nd variant); reconcile e45 pin if first Track-7 bump.
- W-10: SPLIT — Tier-1 firewall fail-closed; Tier-2 indistinguishability sweep + checkbox.
- VD-5 + GD-1/2: compose; name the residue; reproducible.

## Tests to add
- crates/contracts/tests/h47_eval_firewall_contract.rs (~160 LOC): (a) a pre-update dated item is REJECTED, a
  post-update item is scored; (b) a templated item yields a FRESH instance each generation (TemplateRegenerated);
  (c) the H-19 budget spends per query and REFUSES after Exhausted; (d) the indistinguishability probe emits a
  measurement; (e) the --eval-ignore-dates arm scores the pre-update item (R2); (f) ELAI_SEED twice -> identical firewall stats.
- crates/contracts/tests/fixtures/eval_firewall/ (frozen P/Q dated items + a template T + a frozen holdout-budget config + an eval-awareness probe slice).

## Files touched
- CREATE: crates/orchestrator/src/eval_firewall.rs (~230 LOC)
- MODIFY: the loop eval step in darwin_admit_loop (route date_gated_sample/regenerate_instance + the H-19 budget, ~40 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (+EvalFirewallEvent) + action_type_lock.rs:645 (+ e45:294 if first Track-7 bump)
- MODIFY: the DGM/trust dashboard drawer (+firewall stats, ~40 LOC) + its types.ts pair
- CREATE: crates/contracts/tests/h47_eval_firewall_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/H-47-decisions.md (~80 LOC)

## Citations / Grounding
- LiveCodeBench 2403.07974 (release-timestamped items; score only items newer than the last update — the date-gate). GSM-Symbolic 2410.05229 (regenerate instances from templates at score time; GSM-NoOp up to 65% drop).
- Thresholdout / Reusable Holdout 1506.02629 (Dwork et al., Science 2015; DP access bounds adaptive-query overfitting to O(1) independent of query count — the loop's score-queries are adaptive analysis). GSM1k 2405.00332 (private release-gated holdout).
- Eval-awareness 2505.23836 (Gemini-2.5-Pro eval-vs-deploy AUC 0.83) + 2510.20487 (Sonnet 4.5 verbalizes "being evaluated" >80%) — eval items must be indistinguishable from real workload (the residue, VD-5).
- ELAI ground truth (DG-6, grep-confirmed 2026-06-05): sample_held_out_task agent_pool.rs:1745; composes D-6/E-54/H-19 (PLANNED).
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(), SeededClock not wall-clock; GD-1 (compose, do not re-implement).

## Open questions / risks
- The date-gate's reference clock is the darwin config-update timestamp; if generations are rapid the "newer-than-last-
  update" window can starve items. Tune the window on the frozen fixture; never use wall-clock (SeededClock C-15).
- Template-regen only covers TEMPLATED tasks; free-form coding tasks rely on the date-gate + the H-19 budget. The
  firewall is a layered mitigation, not a single guarantee (VD-5).
- A strong improver can sandbag the eval if it recognizes it (eval-awareness); the indistinguishability probe SURFACES
  the risk (measured distinguishability) but cannot eliminate it — name it in the decision doc, never advertise immunity (GD-5/VD-5).
- This file MUST coordinate with the D-6/E-54/H-19 authors on the shared store schema; if those ship after H-47, H-47
  is W-10-tiered against a fixture store and the live composition is the separately-gated checkbox.

## Next
- Once D-6/E-54/H-19 ship, the live composition replaces the fixture store. The firewall + H-44's behavioral canaries
  + H-45's worst-case scoring + H-43's execution fitness together form the convergent verification filter the
  META-FINDING requires; E-53's proof gate is the un-gameable ceiling for the narrow CSQ-Critical slice (VD-5).

## J-11 AMENDMENT — S-8c FOUR LEAKAGE BYPASS MODES + TWO-LAYER DOCTRINE (source-cited)
Grounding: DeNovoSWE paper `2606.10728.html` §3.3 (S3.SS3), `S3.I1.i2.p1.1` Multi-channel Leak Purging,
`S3.I1.i3.p1.1` Git history sanitization, `S3.SS3.p3.1` Runtime cheating prevention, read directly per W-3.
DeNovoSWE identifies four distinct leakage bypass modes that an agent with shell access can exploit. H-47's
eval firewall must defend against ALL four, not just the first:

### Mode 1: .git history recovery (`S3.I1.i3.p1.1`)
Agent runs `git reflog` or inspects loose Git objects to reconstruct the original repository history.
**Fix**: destroy and re-initialize the .git directory entirely (D-6's store-hygiene amendment covers the
fixture-build side; H-47 covers the runtime side — the eval container must not carry .git history).

### Mode 2: site-packages / pip cache / /tmp artifacts (`S3.I1.i2.p1.1`)
Agent finds pre-installed package copies, pip wheel caches, or transient compilation artifacts in /tmp that
contain reference source. **Fix**: multi-channel sweep across ALL Python environments — erase site-packages
traces, pip wheel caches, /tmp C/Rust extension wrappers. H-47's eval container must be clean of any
reference-implementation artifacts.

### Mode 3: network retrieval — pip install / pip download / curl / wget (`S3.SS3.p3.1`)
Agent clones the reference repo from public hosting, installs the target package via pip, or fetches source
via curl/wget. **Fix**: command restriction policy inside the Docker container that blocks common
source-recovery operations + rule-based fuzzy-matching filter using the package name to detect suspicious
commands. H-47's eval firewall must include a command allowlist that blocks these vectors.

### Mode 4: LLM-as-judge trace audit (`S3.SS3.p3.1`)
Beyond automated checks, DeNovoSWE audits agent execution traces with an LLM-as-judge to identify
circumvention attempts that handcrafted rules miss. **CAUTION (TD-3)**: ELAI's doctrine is LLM is T3-advisory
never the load-bearing gate. The trace audit is advisory — it SURFACES suspicious patterns but cannot BLOCK
on its own. A deterministic rule must confirm before a block. This is the one place DeNovoSWE's approach
diverges from ELAI's doctrine; H-47 must keep the LLM audit advisory-only.

### Two-layer doctrine
DeNovoSWE's defense is explicitly two-layer (`S3.SS3.p2.1` Pre-cleanup + `S3.SS3.p3.1` Runtime cheating
prevention): (1) **pre-cleanup** removes all local artifacts before the agent starts, (2) **runtime
prevention** blocks network and command-based recovery during execution. H-47's eval firewall must be
two-layer too: fixture-build-time hygiene (Mode 1+2, shared with D-6) + runtime command blocking (Mode 3,
H-47's own scope). Mode 4 is advisory overlay on both layers.

End of H-47.
