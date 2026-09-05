# H-46: HGM clade-metaproductivity parent selection — expand the productive lineage, not the highest immediate score [TRACK 7]

STATUS: PLANNED (TRACK 7 DGM SELF-IMPROVEMENT VERIFICATION HARDENING; WAVE 7; W-10 tiered: in-process clade aggregator + measured SSB row Tier-1; the multi-generation lineage measurement is a separately-gated checkbox)

Objective: config_archive.rs selects the next parent to self-modify by the node's IMMEDIATE score —
`sigmoid_perf_score(v.score) * exploration_bonus(children_count)` (config_archive.rs:49/63, DGM section 2.3) at
select_parent (:274). HGM (2510.21614, WebSearch-confirmed this session) identifies the Metaproductivity-Performance
Mismatch: an agent's immediate score has a SURPRISINGLY WEAK correlation with its long-term potential to generate
better descendants — a high-scoring node can be an evolutionary dead-end while a mediocre one is the ancestor of a
highly successful lineage. HGM's fix, Clade-Metaproductivity (CMP), selects the node to expand by the AGGREGATED
measured success of its ENTIRE descendant lineage (clade), and beats DGM on SWE-Verified (56.7 vs 53.3) and Polyglot
(30.5 vs 27.1) at ~2.38x LESS compute (517 vs 1,231 CPU-hrs). H-46 adds a CMP selection key to config_archive's
parent selection — biasing expansion toward genuinely productive lineages — composing the SHIPPED archive (do not
rewrite it; GD-1) and reading the measured per-node deltas H-43's execution-grounded fitness supplies.

Estimated effort: ~1 session (M, ~6-8 h). W-10 SPLIT: Tier-1 = the in-process clade-metaproductivity aggregator over
  the archive's recorded descendant deltas + the CladeSelected SSB row + the select_parent wire (fail-closed done-
  gate); Tier-2 = the multi-generation lineage measurement proving CMP beats immediate-score selection on ELAI's lane (fixture now, live a gated checkbox).
Pre-requisites: H-43 (the execution-grounded per-node delta CMP aggregates — a clade over byte-hashes is meaningless).
  Composes config_archive (the SHIPPED archive + sigmoid selection), D-5 (BAI promotion). Reads DG-4/DG-1, PC-2, VD-1/VD-4.
Parallel-safe with: H-45, H-47 (disjoint files). Sequence AFTER H-43 (needs the real per-node signal).
Source: ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (DOSSIER 1 HGM; PART C "Selection upgrade
  (config_archive.rs): HGM clade-metaproductivity 2510.21614"; PART F step 5); _grounding/H-VERIFICATION-GROUNDING-MAP.md DG-4.

## Context

DGM (the baseline ELAI runs) expands the highest-scoring archive node. HGM (2510.21614, metauto-ai/HGM, ICLR 2026
oral, authors incl. Schmidhuber/Zhuge) shows that is the wrong key: the Metaproductivity-Performance Mismatch means
immediate score weakly predicts descendant productivity. CMP aggregates the benchmark performances of a node's
descendants as the indicator of its self-improvement POTENTIAL (inspired by Huxley's clades), and the HGM-optimized
agent reaches human-level on SWE-bench Lite at ~2.38x less compute than DGM. ELAI's config_archive ALREADY records a
parent/child structure (insert_variant, children_count, ancestors-of-active protected) and is otherwise solid — so
CMP is a SELECTION-KEY change, not an archive redesign (GD-1). The per-node delta CMP aggregates is H-43's
execution-grounded fitness (VD-1: a clade over byte-hashes is noise); the verifier producing those deltas stays
cross-family / execution-grounded (VD-4). A new node with no descendants falls back to the immediate sigmoid score
(ImmediateScoreFallback Kind) — CMP only overrides once a lineage exists.

## Existing state (B/C/D/E/F/G coverage)
- config_archive.rs:49 sigmoid_perf_score (lambda=10, alpha_0=0.5) + :63 sigmoid*exploration_bonus + :274 select_parent / :283 select_parent_with<E:EntropySource> + :198 insert_variant — SHIPPED (DG-4). The selection-key EXTEND target.
- config_archive children_count / parent linkage + ancestors-of-active protection — SHIPPED. The clade-tree CMP walks (compose, do not rebuild).
- H-43 execution_grounded_fitness — PLANNED (Track 7). The measured per-node delta CMP aggregates (hard pre-req).
- D-5 / D-29 BAI promotion — PLANNED. H-48's target; CMP changes selection, the bandit changes arm-sampling — orthogonal.
- Clade-metaproductivity aggregator — ABSENT (the net-new mechanism). The BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-VERIFICATION-GROUNDING-MAP.md: DG-4, DG-1, PC-2, VD-1, VD-4, the SCHEMA + CITATIONS blocks.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\config_archive.rs:40-90 (sigmoid + exploration) + :190-300 (insert_variant + select_parent + select_parent_with)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\execution_fitness.rs (H-43 — the per-node delta source)
- workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (DOSSIER 1 HGM + PART C/F)
- HGM 2510.21614 PDF + github.com/metauto-ai/HGM (the CMP definition; WebSearch-confirmed this session — uses GPT-5-mini naming, 2026 ecosystem, expected).
- CLAUDE.md HARD RULES: no .unwrap()/.expect(); deterministic-under-seed selection.

## Tasks (WAVE 7 — Tier-1 clade aggregator + select_parent wire + measured row)

1. **Clade-metaproductivity aggregator (~110 LOC, crates/orchestrator/src/clade_metaproductivity.rs).** Given the
   archive tree + per-node execution-grounded deltas (H-43), compute for each candidate parent the CMP = the
   aggregated measured success of its descendant clade (HGM's lineage-aggregate; deterministic, no wall-clock).
   Expose {cmp, clade_size, immediate_score}.

2. **Selection-key change in select_parent (~40 LOC).** select_parent / select_parent_with weight by CMP (with the
   existing exploration_bonus retained for unexplored nodes), NOT by sigmoid_perf_score(immediate) alone. KEEP the
   sigmoid + exploration machinery for the no-descendants fallback (GD-1; do not rewrite the archive election).

3. **No-lineage fallback (~20 LOC).** A node with clade_size == 0 (no measured descendants yet) selects on the
   immediate sigmoid score (ImmediateScoreFallback Kind) — CMP overrides only once a lineage exists. This prevents a
   cold-start from starving brand-new nodes.

4. **Off-agent measured row (~30 LOC).** Emit `{node_id, cmp, clade_size, immediate_score, key_used, seed}` as a
   CladeSelected SSB row (Pattern-2; Kind CladeMetaproductivity / ImmediateScoreFallback; baseline+1 from LIVE
   224/501, re-read at edit; reconcile e45 pin if first Track-7 bump). Reuse the InsecureRateMeasured pattern (GD-1).

5. **Decision doc workflow/reasoning/H-46-decisions.md (~70 LOC):** the Metaproductivity-Performance Mismatch; CMP
   vs immediate-score; the ~2.38x compute result (report as the PAPER's number, not ELAI's — HD-1); the fallback
   rationale; the honest residue (CMP is a measured proxy for potential, still bounded by the fitness it aggregates — VD-1; and by the verifier gap — VD-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/clade_metaproductivity.rs::clade_metaproductivity (the CMP aggregator)
  - crates/orchestrator/src/clade_metaproductivity.rs::CladeScore (the {cmp, clade_size, immediate_score} struct)
  - crates/orchestrator/src/ssb.rs::SsbActionType::CladeSelected (off-agent selection artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/config_archive.rs::select_parent (the selection key now reads CMP — EXTEND, do not rewrite the archive, GD-1)
  - crates/orchestrator/src/config_archive.rs::sigmoid_perf_score (retained for the no-lineage fallback)
  - crates/orchestrator/src/execution_fitness.rs::execution_grounded_fitness (H-43 — the per-node delta CMP aggregates)
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs (the measured-row pattern — mirror, GD-1)
reachable_from:
  - the self-mod loop selection step (darwin_admit_loop::run_one_generation -> config_archive::select_parent -> clade_metaproductivity -> CladeSelected SSB) [PRODUCTION, darwinian kill-switch, allowlisted]
stub_ban:
  - NO selection on immediate score alone where a measured clade exists (the Metaproductivity-Performance Mismatch) — FAIL
  - NO CMP over byte-hash deltas (VD-1; the clade aggregates execution-grounded deltas from H-43) — FAIL
  - the broken `--select-immediate-score` arm MUST expand a dead-end node the clade key avoids (R2)
  - no .unwrap()/.expect() in prod; deterministic under ELAI_SEED
frontend_pair:
  the per-node CMP vs immediate-score + the selected lineage in the DGM/archive drawer (composes the archive panel; cross_lang_diff PASS).
  W-6 E2E: the loop selecting a mediocre-immediate but high-CMP ancestor (over a high-immediate dead-end) is visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && ELAI_DARWINIAN_LOOP_ENABLE=1 cargo run -p elai-cli -- <the self-mod generation subcommand>
    over a FROZEN archive fixture with node A (high immediate score, all descendants regress) and node B (mediocre immediate, descendants improve), NO other env override"
  expect_ssb: "a CladeSelected row shows cmp(B) > cmp(A) despite immediate(A) > immediate(B); the loop selects B
    (CladeMetaproductivity); a fresh leaf selects on the fallback; recomputable under ELAI_SEED (GD-2)"
  expect_artifact: "select_parent returns B; the broken `--select-immediate-score` arm returns A (the dead-end), proving CMP is load-bearing (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-46_clade_metaproductivity_selection.md --run-trigger
rg "clade_metaproductivity|CladeSelected" .elai_cc/crates/orchestrator/src --glob '!*test*'
rg "select_parent" .elai_cc/crates/orchestrator/src/config_archive.rs   # still present; key extended, archive not rewritten
test -f workflow/reasoning/H-46-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (B selected over the A dead-end; CladeSelected row landed). cargo-green != DONE.
- [ ] W-2: CMP selection is the default under the darwinian kill-switch; no new default-off gate.
- [ ] W-3 production entrypoint: CMP fires from config_archive::select_parent on the live loop, not a #[cfg(test)] caller.
- [ ] VD-1: the clade aggregates execution-grounded deltas (H-43), never byte-hashes.
- [ ] HGM: a high-immediate dead-end loses to a high-CMP mediocre ancestor; a no-lineage leaf uses the fallback.
- [ ] GD-1: config_archive (select_parent + sigmoid + the tree) REUSED (no archive rewrite; P0 if violated).
- [ ] GD-2 reproducible: the CMP aggregate re-computes byte-identically under ELAI_SEED over the frozen archive.
- [ ] R2: the --select-immediate-score broken arm expands the dead-end (falsifiable).
- [ ] W-10: Tier-1 aggregator is the fail-closed done-gate; the multi-generation lineage measurement is a separately-gated checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the aggregator (produced) feeds select_parent (W-8) reached by run_one_generation (W-3).
- W-2: default selection under kill-switch; allowlisted.
- W-3: live selection step; not test-only.
- W-4: frontend_pair = the CMP/lineage tile; cross_lang_diff PASS.
- W-5: observable_effect is a runnable generation over a frozen archive fixture -> real journal rows; Tier-1 in-process.
- W-6: a mediocre-immediate high-CMP ancestor being selected is visible in the dashboard (dashboard-smoker).
- W-7: select_parent / sigmoid_perf_score / execution_fitness / safedelta_runner grep-confirmed (DG-4/DG-1/DG-7); the CMP aggregator ABSENT.
- W-8: clade_metaproductivity (produced) consumed by select_parent's key.
- W-9: +CladeSelected baseline+1 (Pattern-2); reconcile the e45 pin if first Track-7 bump.
- W-10: SPLIT — Tier-1 aggregator fail-closed; Tier-2 lineage measurement + checkbox.
- VD-1/VD-4 + GD-1/2: execution-grounded clade; compose the archive; reproducible.

## Tests to add
- crates/contracts/tests/h46_clade_metaproductivity_contract.rs (~150 LOC): (a) cmp(B) > cmp(A) when A's clade
  regresses and B's improves despite immediate(A)>immediate(B); (b) select_parent returns the high-CMP node;
  (c) a clade_size==0 leaf selects on the immediate sigmoid fallback (ImmediateScoreFallback); (d) the
  --select-immediate-score arm returns the dead-end (R2); (e) ELAI_SEED twice -> identical selection.
- crates/contracts/tests/fixtures/clade/ (a frozen archive tree with A/B lineages + their execution-grounded deltas).

## Files touched
- CREATE: crates/orchestrator/src/clade_metaproductivity.rs (~110 LOC)
- MODIFY: crates/orchestrator/src/config_archive.rs (select_parent key -> CMP with sigmoid fallback, ~60 LOC; do NOT rewrite the archive)
- MODIFY: crates/orchestrator/src/ssb.rs (+CladeSelected) + action_type_lock.rs:645 (+ e45:294 if first Track-7 bump)
- MODIFY: the DGM/archive dashboard drawer (+CMP/lineage, ~40 LOC) + its types.ts pair
- CREATE: crates/contracts/tests/h46_clade_metaproductivity_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-46-decisions.md (~70 LOC)

## Citations / Grounding
- HGM 2510.21614 "Huxley-Godel Machine" (metauto-ai/HGM, ICLR 2026 oral; Clade-Metaproductivity; the Metaproductivity-
  Performance Mismatch; SWE-Verified-60 HGM 56.7 vs DGM 53.3 vs SICA 50.0; Polyglot 30.5 vs 27.1 vs 25.4; ~2.38x less
  compute 517 vs 1,231 CPU-hrs). WebSearch-confirmed this session (title + repo + CMP mechanism match).
- DGM 2505.22954 (the baseline ELAI runs — expand-highest-score; HGM is its successor). Solver-Verifier Gap 2507.00075 (CMP is bounded by the fitness it aggregates).
- ELAI ground truth (DG-4/DG-1, grep-confirmed 2026-06-05): config_archive sigmoid_perf_score:49 / select_parent:274; execution_fitness (H-43).
- CLAUDE.md HARD RULES: no .unwrap()/.expect(); HD-1 (report the paper's compute number as the paper's, not ELAI's).

## Open questions / risks
- CMP needs a measured lineage to override immediate score; early in a run most nodes are leaves and the fallback
  dominates — the CMP benefit only appears once lineages accumulate. That is expected (HGM's gain is multi-generation);
  the Tier-2 lineage measurement is where the ELAI-lane benefit is proven, not the Tier-1 fixture.
- HGM's numbers are on cloud flagships + 70B-class; ELAI's abliterated 7B local lane is unproven for CMP, and CMP
  multiplies generations (compute). Report the ELAI-measured delta on >=3 runs (GD-4) before claiming any lift; never
  quote HGM's 2.38x as ELAI's (HD-1/GD-5).
- CMP aggregation rule (sum vs mean vs decayed) is a design choice; pick HGM's documented aggregate, freeze it, and
  ablate against immediate-score selection on the Tier-2 lineage measurement.

## Next
- H-48 wires the BAI bandit (arm-sampling) on H-43's signal — orthogonal to H-46's parent-selection; together they
  are the regret-bounded selection the dossier's GAP-4 names. D-5/D-29 are the bandit substrate. ExIt (2509.04575)
  curriculum (generate guard tasks from partial intermediate states) composes with H-48.

End of H-46.
