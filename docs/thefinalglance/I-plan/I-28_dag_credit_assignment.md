# I-28: dag credit assignment — branch-grain causal credit, DPO pair capture, and deferred chunk demotion

- Track 16 — ROUTING + BUDGETS + MEASUREMENT | lever SELF-MOD | grounding STRONG | nature BUILD | effort M-L | WAVE 5 | depends I-4 | binds IC-n (none direct) | IC-n (none; Shapley DECLINED)
- Premise: I-4 supplies a sealed search-ladder outcome with a winner and ranked siblings. Slice 1 records deterministic branch-grain credit from those sibling score deltas. Slice 2 captures pruned verdicts for a later DPO decision. Slice 3 stays deferred until each branch carries chunk provenance. Shapley/SHARP attribution remains declined.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule).

## Existing ground truth
- I-4 search ladder is the blocking prerequisite. `SearchLadderOutcome` provides the winning branch and ranked siblings. The v1 ladder does not provide per-node exclusion provenance.
- The live production seam is `.elai_cc/crates/orchestrator/src/task_runner.rs` `drive_task`: `dispatch_search_ladder` runs before `maybe_run_escalation_waterfall`.
- `dispatch_search_ladder` currently discards `SearchLadderOutcome`. Slice 1 must return the outcome as an `Option` and pass it to the credit dispatcher.
- I-17 verdicts and the frozen `VerdictRow` contract remain read-only inputs. A hard-veto branch cannot receive an invented L2 score.
- IS-4 archive and H-46 remain consumer context. Slice 1 does not reimplement archive selection or the clade scorer.
- I-30 is the planned consumer of DPO pairs. A JSONL writer must pair with I-30's first reader or carry an explicit wire-dark promotion row.
- IS-19 wiring enforcement and the R8 frozen-gate rules apply to every new public Rust symbol.

## REVISED scope (ratified BUILD-WITH-SEAM)

SHIP NOW (Tier-1):
- **Slice 1 — branch-grain credit.** Add `DagCreditAssigner` and deterministic branch-grain credit. Emit `DagCreditAssigned` SSB rows with `Kind=CreditWalk`. Insert `dispatch_dag_credit` in `task_runner` `drive_task` after `dispatch_search_ladder` and before `maybe_run_escalation_waterfall`. Gate the path with default-OFF `ELAI_DAG_CREDIT_ENABLE`. Add the wire-dark allowlist row with `promotion_plan_id=I-28-promotion`. Add `crates/contracts/tests/i28_dag_credit_contract.rs`.
- **Slice 2 — pruned-verdict capture and DPO pair extraction.** Add `SearchLadderOutcome.pruned` data only after the FORK-1 decision. Emit `DpoPairExtracted` rows and enforce sealed-RRA provenance. Pair the JSONL writer with I-30's first reader, or keep it wire-dark with `promotion_plan_id=I-30`.

DEFERRED (Tier-2 / gated):
- **Slice 3 — chunk demotion.** Do not implement chunk demotion until every branch has a `chunk_id` or equivalent per-branch chunk provenance substrate. Decide the provenance owner when this slice unblocks.
- Node-grain credit waits for v3 PUCT and provenance capture. Shapley/SHARP remains excluded and requires a separate RL-infrastructure decision.

## Production seam
- In `task_runner.rs` `drive_task`, insert `dispatch_dag_credit` immediately after `dispatch_search_ladder` and before `maybe_run_escalation_waterfall`.
- Change `dispatch_search_ladder` from a result-discarding hook to an `Option<SearchLadderOutcome>` producer. Feed that value to `dispatch_dag_credit`; do not read the SSB back to reconstruct it.
- Append the `DagCreditAssigned` action type after `ScopeCeilingAdvisory` in the SSB registry. Follow the existing flat action-type plus payload-kind pattern.
- Re-derive `SCHEMA` from the registry length at edit time. The live tree has 606 registry entries and `SCHEMA_VERSION_DDL_EPOCH = -309`, so the current derived value is 297. Re-read the registry before implementation and do not hardcode a future value.
- Keep all production paths default-off. Do not edit `ssb.rs` or `task_runner.rs` in this plan-refresh unit. The executor must re-grep these seams before code changes.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/dag_credit.rs::DagCreditAssigner (Slice 1 branch-grain credit; Slice 2 pruned-verdict/DPO extension only after FORK-1)
  - `DagCreditAssigned` SSB payload and `CreditWalk` rows (Slice 1)
  - `DpoPairExtracted` SSB rows and the provenance-checked DPO dataset (Slice 2, gated)

consumes_symbols:
  - task_runner.rs `drive_task` and `dispatch_search_ladder` (the production caller seam)
  - I-4 `SearchLadderOutcome` winner and ranked siblings
  - I-17 sealed verdict artifacts. Never mutate the frozen HardVeto `VerdictRow` score or `VETO_SCORE` contract.
  - IS-4 archive and H-46 clade scoring as downstream context. Do not reimplement either subsystem.
  - I-30 first reader for any DPO JSONL writer, or the explicit I-30 wire-dark allowlist row

reachable_from:
  - `elai run <task>` -> `drive_task` -> `dispatch_search_ladder` -> `dispatch_dag_credit`
  - `ELAI_SEARCH_LADDER_ENABLE=1` and `ELAI_DAG_CREDIT_ENABLE=1` -> `DagCreditAssigned` `CreditWalk` rows
  - `ELAI_DAG_CREDIT_ENABLE` unset -> zero credit rows

stub_ban:
  - No LLM-authored credit score. Slice 1 credit is the deterministic winner-score minus sibling-score rule, clamped to the contract range.
  - No L2 score for a vetoed branch. ID-4 is frozen.
  - No unsealed RRA as a DPO source.
  - No Shapley or SHARP enumeration.
  - No Slice 3 chunk demotion without per-branch chunk provenance.
  - No `.unwrap()` or `.expect()` on production Rust paths.
  - No fabricated measurements or stale SSB baseline values.

frontend_pair:
  - No dashboard work in this unit. The SSB rows are the verifiable artifact. A credit heat view is deferred until trajectory volume justifies it.

observable_effect:
  - Slice 1: a seeded ladder outcome produces one `CreditWalk` row per ranked sibling. Each row contains the winner score, sibling score, ranked position, and deterministic credit delta.
  - Slice 2: a sealed winner and selected pruned sibling produce one `DpoPairExtracted` row with verifiable provenance. An unsealed RRA produces zero pairs.
  - Slice 3: no demotion row exists until the chunk-provenance unblock decision.

## Grounding
- `conductor/opus_walk/evidence/I-28/seam-design-proposal.md` §5 verdict, §6 slices, and §7 FORKs.
- `conductor/opus_walk/evidence/I-28/seam-design-ratification.md` ratifies BUILD-WITH-SEAM, Slice 1 first, default-OFF credit, the I-28 wire-dark promotion row, and the Slice 3 provenance gate.
- I-4 search-ladder and task-runner seams were re-read at the current HEAD. The SSB registry was re-counted at 606 entries with derived schema 297.
- ID-4 freezes veto supremacy. ID-5 bans fabricated measurements. IS-19 supplies the production-caller and wire-dark checks.
- No external paper attribution is claimed for this deterministic branch-grain rule.

## Wargame / acceptance

EXPECTED_OBSERVATION: With a seeded `SearchLadderOutcome` and `ELAI_DAG_CREDIT_ENABLE=1`, the contract test yields `CreditWalk` rows. With the environment flag unset, it yields zero rows. The test checks the schema floor and `sql_str` round-trip.

EXPECTED_OBSERVATION: Slice 2, after FORK-1, yields a provenance-checked `DpoPairExtracted` row for a sealed winner and selected pruned sibling. An unsealed RRA yields zero pairs.

EXPECTED_OBSERVATION: Slice 3 remains deferred and emits no `ChunkDemoted` row until per-branch `chunk_id` provenance exists and the unblock decision is recorded.

ON_FAIL: If both environment flags are on and no `CreditWalk` rows appear, fix the dispatcher wiring and repeat the contract test. If rows appear with `ELAI_DAG_CREDIT_ENABLE` unset, abort for an environment-gate leak.

ON_FAIL: If a DPO pair accepts an unsealed RRA or a vetoed branch with an L2 score, stop the extractor and preserve the frozen verdict contract.

ON_FAIL: If Slice 3 has no per-branch chunk provenance, keep chunk demotion deferred and do not add a synthetic chunk identifier.

FORKS:
- FORK-1 DPO-negative key: (a) LEAN — extend `SearchLadderOutcome.pruned` and select by preserved `progress_score`; (b) also carry the gate id on the pruned SSB row; (c) first-vetoed-by-generation-order. Do not use an L2 score for a vetoed branch because ID-4 is frozen.
- FORK-2 credit grain: choose the LEAN branch-grain Slice 1 now. Treat node-grain credit as a new unit after v3 PUCT and provenance capture.
- FORK-3 JSONL emission: pair the writer with I-30's first reader, or use a wire-dark writer with `promotion_plan_id=I-30`.
- FORK-4 chunk provenance owner: decide between the pristine-context builder and retrieval layer when Slice 3 unblocks.

ABORT:
- ABORT if the frozen `VerdictRow` HardVeto score or `VETO_SCORE` contract needs mutation.
- ABORT if any implementation invents Shapley or SHARP attribution.
- ABORT Slice 3 when no per-branch `chunk_id` substrate exists.
- ABORT if the production path emits credit while `ELAI_DAG_CREDIT_ENABLE` is unset.
- ABORT if a new public Rust symbol lacks a real production caller or an approved wire-dark allowlist row.

VERIFY (off-agent; the controller or auditor owns these runs):
- `python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/I-plan/I-28_dag_credit_assignment.md`
- `cargo test -p elai-contracts i28_dag_credit` (or the live `elai_cc_contracts` package name).
- Run `production_caller_check.py` on every changed Rust file. Run `wiring_contract_check.py` against this plan.
- Run the SSB `action_type_lock` golden check after registry regeneration. Confirm the schema is derived from the current registry length.
- Confirm the wire-dark allowlist row uses `promotion_plan_id=I-28-promotion` and the credit environment flag is default-off.

REDTEAM:
- Fabricated-credit attack: submit an LLM-authored score that bypasses the sibling-delta rule. The implementation must reject it because credit equals winner score minus sibling score only.
- Unsealed-RRA attack: submit an unsealed RRA as a DPO source. The extractor must yield zero pairs.
- Veto-score attack: submit an L2 score for a hard-vetoed branch. The implementation must reject it without mutating `VerdictRow` or `VETO_SCORE`.
- Environment-leak attack: run the task-runner path with `ELAI_SEARCH_LADDER_ENABLE=1` and `ELAI_DAG_CREDIT_ENABLE` unset. The SSB must contain zero `DagCreditAssigned` rows.
- Provenance attack: attempt Slice 3 demotion without a per-branch `chunk_id`. The plan must abort the slice.

INTEGRATION_SMOKE: Run `cargo test -p elai-contracts i28_dag_credit_contract` with a seeded ladder outcome and both `ELAI_SEARCH_LADDER_ENABLE=1` and `ELAI_DAG_CREDIT_ENABLE=1`. Assert `CreditWalk` rows, schema-floor validity, and `sql_str` round-trip. Repeat with `ELAI_DAG_CREDIT_ENABLE` unset and assert zero rows through the task-runner path.

RECON: none open — the executor must re-grep the live task-runner, search-ladder, verifier, SSB registry, and wire-dark allowlist seams before implementation.

## Execution boundary
- This file is a plan only. Do not write product Rust in the plan-refresh unit.
- Do not edit `ssb.rs`, `task_runner.rs`, frozen gate files, or evaluator files during plan refresh.
- The off-agent verifier and serialized integrator certify completion. The worker does not claim DONE.

End of I-28.
