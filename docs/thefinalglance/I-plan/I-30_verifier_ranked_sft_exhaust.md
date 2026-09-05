# I-30: verifier-ranked sft exhaust — extract winning trajectories from sealed RRAs into SFT JSONL, rejection-sampling LoRA first

- Track 17 — OPEN-WEIGHT FLYWHEEL | lever ALIGNMENT | grounding STRONG | nature BUILD | effort L | WAVE 5 | depends I-4 | binds ID-6 | IC-7
- Premise: the search compute I-4 paid for produces, for free, verifier-ranked trajectories. the correct
  first distillation step is rejection-sampling SFT on those trajectories — cheap, stable, most of the
  gain, none of the RL instability. GRPO (I-31) comes after, behind blocking guards. dataset rows carry
  provenance hashes to their sealed RRA so every training example is traceable. promotion of any resulting
  LoRA checkpoint passes the ID-6 ledger gates. IC-7 mandates this sequencing.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule).

## Steal/build
the dataset extraction pipeline (sft_exhaust.rs) walks SEALED RRAs — only sealed runs are eligible (an
in-progress or manually-closed RRA is not a training source). for each sealed run, identify the winning
branch (TriLayerVerdict(Promoted) or PhaseComplete with the highest-L2 score that passed L1 HARD VETO).
extract the full chain (root -> verified COMPLETE) as a JSONL record: system prompt, each turn in the
trajectory, the final verified output. record provenance: blake3(RRA_bytes) as the provenance hash; the
JSONL record fails verification if the hash does not match the RRA on disk. negative pairs: for each
winning trajectory, pair it with the highest-L2-scoring sibling that was VETOED by a deterministic L1 gate
(fluent-but-wrong by construction, via I-28's DPO pair extraction if I-28 is shipped; else re-derive from
the search tree). negative pairs are stored in the same JSONL with a label field. IC-7 (SFT before GRPO):
rejection-sampling SFT is the ONLY training run in I-30; no PPO/GRPO/DPO fine-tuning in this session
(DPO is E-55 HARD-AVOID inherited). the LoRA run itself is a RunPod checkbox (W-10 Tier-2): the offline
dataset pipeline and the promotion gate (ID-6) ship now; the actual GPU training run is separately gated.
IC-7 note on LoRA target: the canonical local model is Qwen2.5-Coder (per CLAUDE.md); any MoE retarget is
a RunPod-gated EVALUATION, never an assumed swap. promotion through ID-6 gates: the resulting LoRA checkpoint
promotes ONLY through H-17 persisted trial-N + H-18 deflation gate + H-28 reward-tampering gate + D-6
held-out + D-7 cross-domain canary; the evaluator (checkpoint evaluator harness) lives OUTSIDE the loop's
write boundary (R8). EXCLUDED: DPO fine-tuning (E-55 HARD-AVOID stands, not re-litigated); unsealed RRA
rows as training sources (cannot verify provenance without a sealed RRA); checkpoint promotion without the
full ID-6 ledger chain.

## Existing ground truth
- IS-10: rra.rs build_rra:542, write_rra:693, read_rra:708, verify_rra:724 + attestation.rs (G-2/G-8 shipped surface) — the sealed-RRA source. compose, never re-implement RRA construction.
- IS-16: H-17 trial_ledger.rs, H-18 deflation_gate.rs, H-28 (gate_self_validation.rs or reward-tampering gate — PLAN-asserted; executor re-greps per W-7), D-6 holdout_budget.rs — the ID-6 gate files.
- IS-4: config_archive.rs (the archival precedent; LoRA checkpoint is a config-class artifact per ID-6).
- E-55 SFT-on-rollouts: the plan this session implements the substrate for. PLAN-asserted; executor re-greps per W-7.
- D-6 held-out eval corpus: the held-out set the promotion gate uses. PLAN-asserted; executor re-greps per W-7.
- I-4 search ladder (BLOCKING prerequisite): the trajectories and the sealed RRA source.
- I-28 DPO pairs (consumes if shipped): call `DpoPairExtractor::run` directly from the sealed walker with `DpoProvenance{rra_sealed:true}`. Do not consume live task_runner `DpoPairExtracted` rows until task_runner plumbs seal status.
- IC-7 (mandatory sequencing): SFT before GRPO; LoRA before any GRPO run; no DPO (E-55 HARD-AVOID).
- IS-19: wiring tooling + R8 (the training scripts and the evaluation harness are frozen artifacts outside the training loop's write boundary).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/cli/src/sft_exhaust.rs (~140 LOC — sealed-RRA walker, trajectory extractor, JSONL writer, provenance
  hash, SftExhaustEvent emission)
- `elai dataset export --format sft` CLI subcommand
- ID-6 promotion gate check in the dataset pipeline (block if the resulting artifact is not tied to IS-16 gate chain — emit a lint warning at export time naming the required gate rows)
- dataset card surface is DEFERRED under `owner-ui-rework-hold-2026-08-06` for this walk; the backend `SftExhaustEvent` still ships
- contract test crates/contracts/tests/i30_sft_exhaust_contract.rs
- decision doc workflow/reasoning/I-30-decisions.md

DEFER (Tier-2 / gated — RunPod checkbox):
- LoRA fine-tuning run (GPU required; RunPod; anti-slopsquat any LoRA library before use + user confirmation)
- LoRA checkpoint promotion through IS-16 full ledger chain (requires the training run to have produced a checkpoint)
- MoE retarget evaluation (Qwen3-Coder-Next-class; a separate RunPod evaluation, never an assumed swap)

## Integration
- CREATE: crates/cli/src/sft_exhaust.rs (~140 LOC)
- CREATE: workflow/training/sft/ (offline training scripts — Tier-2, RunPod; not shipped until dep approved)
- MODIFY: crates/cli/src/args.rs (add `dataset export --format sft` subcommand)
- seams from the map: Track 17 anchor — IS-10 RRA + IS-16 ledger gates + E-55 SFT-on-rollouts plan; I-4/I-28 trajectories + pairs
- `frontend_pair`: dataset card drawer is DEFERRED under `owner-ui-rework-hold-2026-08-06` for this walk. The backend `SftExhaustEvent` still ships. `cross_lang_diff.py` is not a Tier-1 UI deliverable.

## WIRING_CONTRACT
produces_symbols:
  - crates/cli/src/sft_exhaust.rs::SftExhaust (sealed-RRA walker + JSONL extractor + ID-6 promotion gate check)
  - workflow/training/sft/<run_id>.jsonl (the SFT JSONL with provenance hashes and negative pairs)

consumes_symbols:
  - IS-10 rra.rs::build_rra / verify_rra (sealed-RRA source — only sealed runs eligible) — GD-1 / IC-7
  - IS-16 gate files (trial_ledger.rs / deflation_gate.rs / holdout_budget.rs / gate_self_validation.rs — ID-6 promotion chain, verified at export time) — ID-6
  - I-4 search ladder trajectories (the winning path extracted from the search tree per sealed RRA) — GD-1
  - I-28 DPO pairs: call `DpoPairExtractor::run` directly at the sealed walker with `DpoProvenance { rra_sealed: true }`. Do not consume live task_runner `DpoPairExtracted` rows until task_runner plumbs seal status — live task_runner currently hardcodes `rra_sealed: false`.

reachable_from:
  - `elai dataset export --format sft` (CLI entrypoint)  [PRODUCTION CLI dataset surface]
  - SftExhaustEvent rows in SSB + the JSONL on disk are the off-agent artifacts; LoRA training run is a separately-gated RunPod checkbox (W-10)

stub_ban:
  - NO unsealed RRA rows as training sources — provenance hash must verify against a sealed RRA (blake3 match mandatory)
  - NO DPO fine-tuning in this session (E-55 HARD-AVOID inherited — not re-litigated; I-31 covers GRPO, LAST)
  - NO LoRA checkpoint promotion without the full IS-16 ledger chain (ID-6 — the export pipeline emits a lint error if the gate files are not referenced)
  - NO assumed MoE model swap — the target stays Qwen2.5-Coder unless a RunPod evaluation concludes otherwise (IC-7)
  - NO fabricated dataset statistics (ID-5 — all counts are derived from actual sealed RRAs)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2

frontend_pair:
- SftExhaustEvent rows feed the future dataset card. Do not add the UI drawer in this Tier-1 walk.

observable_effect:
  trigger: "elai dataset export --format sft --rra-dir <sealed_rra_dir>"
  expect_ssb: "SftExhaustEvent rows: PathExtracted per winning trajectory, NegativePaired per DPO negative, DatasetSealed when the JSONL is finalized; DatasetSealed row carries record_count + provenance_verified_count (must be equal)"
  expect_artifact: "workflow/training/sft/<run_id>.jsonl — every record's provenance_hash verifies against its sealed RRA on disk (a tampered record fails verify_rra and causes a DatasetSealed(provenance_mismatch) row, not a silent pass — R2); negative pairs reference real vetoed branches with TriLayerVerdict(HardVeto) row ids; NEGATIVE: an unsealed RRA is rejected at the walker (PathExtracted row absent for that run, lint error emitted)"

## Grounding
- IS-10 (`rra.rs:542` `build_rra`, `:693` `write_rra`, `:708` `read_rra`, `:724` `verify_rra`, plus `attestation.rs`) is the sealed-RRA source. Compose it. Do not re-implement RRA construction.
- IC-7 (mandatory sequencing: SFT before GRPO; no DPO): I-GROUNDING-MAP binding.
- ID-6 (weights-are-configs): LoRA checkpoint promotes through the IS-16 gate chain. The evaluator stays outside the write boundary (R8). The dataset export only records the gate-chain lint status.
- E-55 HARD-AVOID (DPO): inherited from E-55 plan; not re-litigated in I-30.
- Track 17 anchor table (I-GROUNDING-MAP): I-30 uses IS-10 + IS-16 + E-55 + I-4/I-28.
- ssb module split: use `ssb/{action_type,payloads,schema,store,tests}.rs` and the live action-type lock. Do not treat a monolithic `ssb.rs` or `ssb/action_type_lock.rs` as the source.
- arXiv IDs: rejection-sampling SFT from verifier-ranked trajectories (well-known; WebSearch-confirm the primary citation before headlining any specific arXiv ID for this technique).

## COMPOSES-WITH
- IS-10 rra.rs + attestation.rs: do NOT re-implement RRA construction or verification; read sealed RRAs via verify_rra.
- IS-16 gate files (trial_ledger.rs etc.): do NOT re-implement ledger logic; verify the gate chain at export time.
- I-4 search ladder: do NOT re-implement trajectory extraction; walk the search tree's sealed winning branch.
- `I-28 DPO pairs`: consume the shipped extractor's contract directly. The sealed walker calls `DpoPairExtractor::run` with `DpoProvenance { rra_sealed: true }`. Do not re-derive pairs or consume live task_runner rows until task_runner plumbs seal status.
- I-31 GRPO flywheel: do NOT include any GRPO or PPO logic in I-30; I-31 is LAST; I-30 produces the SFT init checkpoint that I-31 starts from.
- H-43..H-48 (Track 7 spine): do NOT re-implement the convergence filter; I-30 is a dataset pipeline job, not a loop component.

## J-11 AMENDMENT — S-8a DIFFICULTY-BANDED INCLUSION FLOOR (source-cited)
Grounding: DeNovoSWE paper `2606.10728.html` §4.2 (S4.SS2), Table 2 (S4.T2), §5.3 ablation (S5.SS3), read directly per W-3.
DeNovoSWE's only real ablation (§5.3, `S5.SS3.p3.1`) proves that difficulty-banded thresholds beat a fixed
global threshold: +1.2pp on BeyondSWE-Doc2Repo (0.488→0.500). The gain came from the STRICTER easy tier
(`S5.SS3.p3.1`: "the more permissive difficulty-aware configuration, which relaxes the easiest-tier threshold
from 0.90 to 0.80, leads to lower downstream performance ... highlighting the need for stricter thresholds on
low-difficulty instances"). This figure is design rationale, never an ELAI target (HD-1/ID-5).

The banding table (Table 2, `S4.T2`):
| Difficulty Score Range | Inclusion Threshold |
|------------------------|---------------------|
| [0.0, 0.2)             | 0.90                |
| [0.2, 0.4)             | 0.85                |
| [0.4, 0.6)             | 0.80                |
| [0.6, 0.8)             | 0.70                |
| [0.8, 1.0]             | 0.60                |
(`S4.SS2.p1.1`: "Easier instances are subject to stricter thresholds ... harder instances are assigned lower
thresholds to retain informative trajectories that still solve a substantial portion of the repository.")

Amendment for I-30: no global pass/fail gate on SFT exhaust. The difficulty band sets the **inclusion floor**
— a trajectory's score must meet or exceed the band threshold to be included in the SFT JSONL. A PARTIAL on a
hard task (score 0.65, band [0.6,0.8) threshold 0.70 — does NOT meet; score 0.72, same band — DOES meet) is
worth more than a PASS on a trivial one (score 0.95, band [0.0,0.2) threshold 0.90 — meets, but the task
itself is trivial). The partial-over-trivial rule: a trajectory from a harder band that meets its lower
threshold is PREFERRED over a trajectory from an easier band that meets its higher threshold, because the
harder trajectory captures more repository-level reasoning (`S4.SS2.p2.2`: "This relaxation enables the
pipeline to retain informative long-horizon partial successes that still capture substantial repository-level
reasoning and implementation behavior").
The difficulty score d_i comes from the I-25 router feature (S-8b amendment) or I-30's own calibration on
ELAI's rollouts. The structural signal needs a domain substitute (SCIP function count / cross-language
bindings touched / SSB variants added — not "executable Python lines" which is Python-only).

## Wargame / acceptance

EXPECTED_OBSERVATION: The Tier-1 CLI smoke `elai dataset export --format sft --rra-dir <fixture>` emits `PathExtracted` for each eligible winning trajectory and one `DatasetSealed` row. `record_count == provenance_verified_count` on `DatasetSealed`.

EXPECTED_OBSERVATION: The Tier-1 contract cases cover a sealed RRA, an unsealed RRA, a tampered provenance hash, and each J-11 difficulty band. The test writes only records that pass the sealed and provenance checks.

EXPECTED_OBSERVATION: An unsealed RRA is rejected at the walker. It emits no `PathExtracted` row for that RRA and emits a visible rejection or lint result.

EXPECTED_OBSERVATION: A provenance mismatch fails closed. It emits `DatasetSealed(provenance_mismatch)` or the contract-equivalent failure and never writes a silently accepted training record.

EXPECTED_OBSERVATION: Band inclusion uses the Tier-1 fixture or calibration table. Scores at or above each band floor are included, scores below the floor are excluded, and a hard-band qualifying trajectory remains eligible without a global pass/fail gate.

ON_FAIL: If the CLI smoke lacks `PathExtracted` or `DatasetSealed`, or the two counts differ, inspect `dataset_cmd.rs` and the sealed walker seam. Re-run the contract case before changing the event contract.

ON_FAIL: If the contract suite misses a sealed, unsealed, tampered, or band case, add the missing deterministic fixture and repeat the Tier-1 contract command.

ON_FAIL: If an unsealed RRA produces a training row, stop the export path and restore the sealed-walker guard. Do not bypass the guard with a caller-side filter.

ON_FAIL: If a tampered provenance hash is accepted, stop the writer and trace the `read_rra` plus `verify_rra` result. Do not emit a `DatasetSealed` success row.

ON_FAIL: If band inclusion uses a global threshold or accepts a below-floor score, restore the fixture/calibration band table. Do not substitute fabricated difficulty or record counts.

FORKS:
- FORK-1 negative source: DEFAULT = call `DpoPairExtractor::run` directly at the sealed walker with `DpoProvenance { rra_sealed: true }`. Do not consume SSB `DpoPairExtracted` rows because live `task_runner` hardcodes `rra_sealed: false`. Revisit this fork when `task_runner` plumbs seal status.
- FORK-2 difficulty `d_i`: DEFAULT = the fixture or calibration-constant band table in Tier-1. Switch to the I-25 router feature when that feature ships, with a contract fixture that preserves the same inclusion floors.
- FORK-3 module shape: DEFAULT = new `crates/cli/src/dataset_cmd.rs` plus `crates/cli/src/sft_exhaust.rs`, as siblings of `rra_cmd.rs`. Do not extend `rra_cmd.rs` with the dataset export path.

ABORT:
- ABORT if an unsealed RRA becomes a training source.
- ABORT if this unit adds DPO fine-tuning, PPO, or GRPO logic.
- ABORT if a LoRA checkpoint promotes without the full ID-6 ledger chain.
- ABORT if the implementation fabricates record counts, provenance counts, scores, or other dataset statistics.
- ABORT if it adds blake2 or sha2 instead of the repository blake3 contract.
- ABORT if a production path adds `.unwrap()` or `.expect()`.
- ABORT if this Tier-1 walk adds the UI drawer. Keep `frontend_pair` DEFERRED under `owner-ui-rework-hold-2026-08-06`.

VERIFY:
- Run `python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md` and require exit 0.
- Run plain `cargo test -p elai-contracts i30_sft_exhaust`; if the live package is named `elai_cc_contracts`, use that package name instead. Depot resolves plain `cargo`; do not invoke an absolute cargo or rustc path.
- Run `python3 .claude/tools/production_caller_check.py --files <changed-rust-files>` and block any DARK public symbol.
- Run `python3 .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md`.
- Run the `action_type_lock` golden check after adding `SftExhaustEvent`. Record the off-agent verifier result and the serialized integrator result as the certification artifacts.

REDTEAM:
- Unsealed smuggle: place an unsealed RRA in the export directory. The walker must reject it and produce no training record.
- Tampered provenance hash: change a record hash after export. Verification must report a mismatch and block dataset success.
- Fabricated record count: alter the count before `DatasetSealed`. The contract must detect that it differs from the verified count.
- DPO training sneak: provide DPO pairs to a training command in this unit. The Tier-1 surface must expose extraction only and reject training.
- Missing ID-6 lint: export without the required ledger rows. The result must name the missing ID-6 rows and must not claim promotion.

INTEGRATION_SMOKE: Run `elai dataset export --format sft --rra-dir <fixture>`. Assert `PathExtracted` and `DatasetSealed`, then assert `record_count == provenance_verified_count`. Run the matching contract test with the sealed, unsealed, tampered, and difficulty-band fixtures. The off-agent verifier owns the smoke verdict.

RECON: none open. The executor must re-grep the live RRA, task_runner, SSB split, CLI dispatch, ID-6 ledger, and action-type-lock seams before implementation.

End of I-30.
