# J-50: recovery-trajectory sft exhaust — mine failed-then-recovered chains into the SFT dataset, not winning-paths-only (EXTENSION of I-30)

STATUS: PLANNED (WAVE GUAVA-1; Tier-1 offline pipeline now, Tier-2 LoRA run RunPod-gated; nature EXTENSION of I-30 — adds a record class, re-implements nothing)

- Track J (Guava intake) | lever ALIGNMENT+PROVABLE | grounding STRONG (Guava arXiv:2606.18363 §3.2 + App A.4, measured; I-30 surface PLAN-asserted, executor re-greps W-7) | nature EXTENSION(I-30) | effort M | WAVE GUAVA-1 | depends I-30 (sft_exhaust.rs surface) | binds IC-7, ID-6
- Premise: I-30 extracts, per sealed RRA, the WINNING branch (root -> verified COMPLETE) plus NEGATIVE pairs (the highest-L2 sibling VETOED by an L1 gate — fluent-but-wrong, never recovers). Both are single-verdict signals: do-it-right, and don't-do-this. Neither teaches RECOVERY. Guava (arXiv:2606.18363, "An Effective and Universal Harness for Embodied Manipulation", 2026-06-16) measured that 743/1934 = 38.4% of its distillation trajectories are failed-THEN-recovered (a perturbation injected into a success trace, generation continued from the broken state, same success filtering), and that success-only SFT collapses on long-horizon tasks (Finding 3: shell-game 6.7%, place-all-red 0.0% under SFT-only; SFT+RL lifts them to 60.0% / 93.3% — but the SFT floor on long-horizon is what recovery data raises). ELAI's sealed RRAs already CONTAIN recovery chains for free: a search tree where a branch hit an error/veto turn and a corrective continuation reached TriLayerVerdict(Promoted). J-50 extracts that recovery SUBSEQUENCE as a distinct labeled positive record at a target fraction, so the distilled local model learns to self-correct, not only to be right first try.

## Steal/build

Extend I-30's crates/cli/src/sft_exhaust.rs with a recovery-trajectory extractor (do NOT re-implement the RRA walker, the JSONL writer, the provenance hash, or the winning-branch extraction — all are I-30's, consumed). For each sealed RRA: walk the search tree for chains that (a) contain >=1 turn that produced a deterministic error or an L1 HARD-VETO or a failed gate, AND (b) continue through corrective turns to a verified COMPLETE (TriLayerVerdict(Promoted) / PhaseComplete passing L1). Extract the full chain INCLUDING the error turn and the correction turns as a JSONL record with label="recovery" (distinct from I-30's label-less winning record and its label="negative" vetoed pairs). Carry the same blake3(RRA_bytes) provenance hash — a recovery record fails verification if the hash does not match the sealed RRA on disk (R2). Enforce a target mix via a named const RECOVERY_TARGET_FRACTION (default 0.35, in [0.30, 0.40] per Guava's 38.4%): the exporter reports actual recovery fraction and emits a WARN when the sealed-RRA corpus cannot supply the target (honest residue — UNDER-target is reported, NEVER back-filled with synthetic or relabeled non-recovery chains, ID-5). The LoRA run that consumes the enriched JSONL stays I-30's Tier-2 RunPod checkbox.

HARD boundaries (negative space): NOT a conflict with I-5 PRISTINE CONTEXT INJECTOR — I-5 EXCLUDES failed-sibling tracebacks from INFERENCE context (don't contaminate live reasoning); J-50 INCLUDES failed-then-recovered chains in TRAINING data (teach recovery offline). Different stage, stated so in-file (GD-5). NOT a new training run — J-50 is dataset extraction only; SFT stays the single I-30 run (IC-7: SFT before GRPO; no DPO E-55 HARD-AVOID; GRPO is I-31, LAST). NOT a relabel of I-30's negatives — a vetoed sibling that never reaches COMPLETE stays label="negative"; only a chain that errs AND recovers to a verified COMPLETE is label="recovery".

## Existing ground truth (PLAN-asserted from I-30 / FEATURE-MAP; executor re-greps per W-7)

- I-30 crates/cli/src/sft_exhaust.rs::SftExhaust (sealed-RRA walker + JSONL extractor + provenance hash + SftExhaustEvent emission) — the surface J-50 extends. PLANNED (I-30 WAVE 5). depends I-30.
- I-30 SftExhaustEvent (Pattern-2; Kind: PathExtracted / NegativePaired / DatasetSealed) — J-50 adds Kind::RecoveryExtracted (reuse the action_type, +1 Kind arm; no new action_type).
- IS-10 rra.rs::build_rra / verify_rra + attestation.rs (G-2/G-8 SHIPPED) — the sealed-RRA source; compose, never re-implement (GD-1).
- I-4 EVIDENCE-RANKED SEARCH LADDER (search/ladder.rs) — the search tree the recovery chain is walked from; the same tree I-30 walks for the winning branch.
- IC-7 sequencing (SFT before GRPO, no DPO) — inherited; J-50 changes only the DATASET, not the training order.
- ID-6 promotion chain (H-17 trial_ledger.rs / H-18 deflation_gate.rs / H-28 / D-6 holdout_budget.rs) — unchanged; the recovery-enriched checkpoint promotes through the same gates.
- E-55 SFT-on-rollouts — the plan whose dataset substrate I-30+J-50 feed.

## REVISED scope (W-10 tiered)

SHIP NOW (Tier-1):
- recovery-trajectory extractor in crates/cli/src/sft_exhaust.rs (~90 LOC: error/veto-turn detection on the tree, corrective-continuation-to-COMPLETE walk, label="recovery" JSONL record, RECOVERY_TARGET_FRACTION accounting + under-target WARN, SftExhaustEvent::RecoveryExtracted emission)
- `elai dataset export --format sft` (I-30's subcommand) now also emits recovery records; a `--recovery-fraction <f>` override on the named-const default
- dataset card (I-30's drawer) gains recovery_count + actual_recovery_fraction + under_target flag
- contract test crates/contracts/tests/j50_recovery_exhaust_contract.rs
- decision doc workflow/reasoning/J-50-decisions.md

DEFER (Tier-2 / RunPod): the LoRA run on the recovery-enriched JSONL (I-30's existing Tier-2 checkbox; anti-slopsquat any LoRA lib + user confirmation before the GPU run).

## Integration

- MODIFY: crates/cli/src/sft_exhaust.rs (~90 LOC — the recovery extractor + the RecoveryExtracted arm; I-30 owns the file)
- MODIFY: crates/orchestrator/src/ssb.rs + crates/orchestrator/tests/action_type_lock.rs (the +1 Kind arm on SftExhaustEvent — full 9-tuple lockstep; SCHEMA_VERSION is COMPUTED `-309 + SSB_ACTION_TYPES.len()` at ssb.rs:1500, auto-follows; re-grep BOTH at edit, W-9)
- MODIFY: the I-30 dataset-card Svelte drawer + its #[tauri::command] (recovery counts); cross_lang_diff.py covers Tauri -> Svelte
- frontend_pair: the I-30 dataset card drawer gains recovery_count + actual_recovery_fraction + under_target badge. Justified: training-data composition (how much recovery signal) must be operator-visible before a LoRA run, exactly as I-30's W-6 surface gates data quality.

## WIRING_CONTRACT

produces_symbols:
  - crates/cli/src/sft_exhaust.rs::SftExhaust::extract_recovery_trajectories (error/veto-turn -> corrective-continuation-to-verified-COMPLETE walker; label="recovery" JSONL records; RECOVERY_TARGET_FRACTION accounting)
  - SftExhaustEvent::RecoveryExtracted Kind arm (per recovery chain extracted)

consumes_symbols:
  - I-30 crates/cli/src/sft_exhaust.rs::SftExhaust (the walker/writer/provenance surface — extend, never re-implement) — GD-1 / depends I-30
  - IS-10 rra.rs::verify_rra (provenance verify on every recovery record — blake3 match mandatory) — GD-1
  - I-4 search/ladder.rs tree (the chains walked for recovery subsequences) — GD-1
  - SftExhaustEvent (I-30 action_type; +1 Kind arm RecoveryExtracted) — reuse, net 0 new action_types

reachable_from:
  - `elai dataset export --format sft` (I-30 CLI entrypoint; J-50 adds recovery records to the same run)  [PRODUCTION CLI dataset surface]
  - SftExhaustEvent::RecoveryExtracted rows in SSB + the label="recovery" JSONL records on disk are the off-agent artifacts; a #[cfg(test)] caller never satisfies this (R7)

stub_ban:
  - NO synthetic or relabeled recovery records — a record is label="recovery" ONLY if its chain contains a real error/veto turn AND reaches a verified COMPLETE in the sealed RRA (ID-5)
  - NO back-filling to hit RECOVERY_TARGET_FRACTION — under-target is a WARN + a logged under_target flag, never a fabricated record (R2/ID-5)
  - NO re-implementation of I-30's walker, JSONL writer, or provenance hash (GD-1)
  - NO relabeling I-30 negatives as recovery (a vetoed-never-COMPLETE sibling stays label="negative")
  - NO DPO/GRPO/PPO logic (IC-7; E-55 HARD-AVOID DPO; I-31 owns GRPO, LAST)
  - NO unsealed RRA as a source; NO .unwrap()/.expect() on prod paths; NO blake2/sha2

frontend_pair:
  the I-30 dataset-card drawer gains recovery_count + actual_recovery_fraction + under_target badge, fed by SftExhaustEvent::RecoveryExtracted rows; cross_lang_diff.py covers Tauri -> Svelte. Justified: recovery-signal proportion is the W-6 data-quality surface the operator checks before a LoRA run.

observable_effect:
  trigger: "elai dataset export --format sft --rra-dir <sealed_rra_dir_with_a_known_recovered_run>"
  expect_ssb: "SftExhaustEvent::RecoveryExtracted rows, one per recovery chain, each carrying the source RRA provenance hash + the error-turn index + the COMPLETE-turn index; the DatasetSealed row carries recovery_count + actual_recovery_fraction (both derived from real records, ID-5)"
  expect_artifact: "workflow/training/sft/<run_id>.jsonl contains label=\"recovery\" records whose provenance_hash verifies against the sealed RRA (a tampered recovery record fails verify_rra -> DatasetSealed(provenance_mismatch), never a silent pass, R2); NEGATIVE: a corpus with ZERO recovered runs produces ZERO recovery records, an under_target WARN, and a DatasetSealed row with actual_recovery_fraction=0.0 — NOT a fabricated record (R2/ID-5)"

## Grounding

- Guava arXiv:2606.18363 §3.2 + Appendix A.4 (recovery-trajectory injection; 743/1934 = 38.4%) + Finding 3 (success-only SFT collapses on long-horizon: shell-game 6.7%, place-all-red 0.0%). VERIFIED at authoring: title + watermark "arXiv:2606.18363v1 [cs.RO] 16 Jun 2026" (workflow/papers/guava-interface.md). Scope caveat (GD-5/HD-1): Guava's numbers are EMBODIED-MANIPULATION on a 4B VLA in sim — the recovery-data PRINCIPLE transfers (teach self-correction with failed-then-recovered traces); the specific 38.4% is a directional anchor for RECOVERY_TARGET_FRACTION, NOT a claimed ELAI number.
- I-30 surface (sft_exhaust.rs, SftExhaustEvent, IC-7, ID-6) — PLAN-asserted from the I-30 file; executor re-greps per W-7.
- IS-10 rra.rs/attestation.rs (sealed-RRA source, SHIPPED per FEATURE-MAP) ; I-4 search ladder.
- SSB: reuse SftExhaustEvent; +1 Kind arm RecoveryExtracted. SCHEMA_VERSION COMPUTED (`-309 + SSB_ACTION_TYPES.len()`, ssb.rs:1500) -> baseline 268 / TOTAL_VARIANTS 577 as of LIVE re-grep 2026-06-16 (J-0_INDEX constants block); NEVER hard-code, re-grep BOTH at edit (W-9/R-c).
- CLAUDE.md hard rules: blake3 only, no .unwrap in prod, scoped -p tests, ID-5 no fabricated stats, W-2 (no new ELAI_*_ENABLE — J-50 adds none).

## COMPOSES-WITH

- I-30 sft_exhaust.rs: EXTEND the walker with a recovery extractor; do NOT re-implement extraction, JSONL, or provenance.
- IS-10 rra.rs verify_rra: verify every recovery record's provenance; do NOT re-implement RRA verification.
- I-4 search ladder: walk the existing tree; do NOT re-implement trajectory extraction.
- I-31 GRPO flywheel: J-50 enriches the SFT init only; do NOT add GRPO/PPO; I-31 is LAST.
- I-5 PRISTINE CONTEXT INJECTOR: orthogonal (inference-context exclusion vs training-data inclusion); do NOT couple them.

## Riders (recorded, not in-scope)

- Targeted-RL-on-bottleneck-tasks (Guava Finding 3: GRPO applied ONLY to the 2 hard long-horizon tasks, full-suite RL too expensive) = a design RIDER on I-31 GRPO FLYWHEEL (gate GRPO on a per-task bottleneck score, not the whole suite); recorded here, authored nowhere until I-31 is active (I-31 is deferred-LAST behind H-43/H-44/H-28 + I-24).
- Publishing a recovery-vs-success ablation (does recovery data raise the SFT long-horizon floor on ELAI's own tasks) = a SAFEDELTA-framed measurement once the Tier-2 LoRA run exists; recorded.

End of J-50.
