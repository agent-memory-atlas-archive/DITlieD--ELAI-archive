# I-30 verifier-ranked SFT exhaust — seam-design proposal (DOC-ONLY)

session: opuswalk-I-30-PLAN-1786169934-50304c  date: 2026-08-08
actor: synthesizer-elai on posthog-fable-5  role: PLAN  writes: this doc only
base: worktree HEAD a2744e5 (branch conductor/opuswalk-I-30-PLAN-1786169934-50304c-2c639134,
3727 tracked files under .elai_cc/; same HEAD as /home/user/teikoku/elai main). Every seam below
verified against this tree with grep/sed this session, not from plan memory.
plan: docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md (146 ln, read in full)
NOTE: /home/user/teikoku/conductor/opus_walk/evidence/I-30/ is permission-denied to this worker
(touch probe failed). The complete work product therefore lives in THIS file, the one allowed
write. Section 6 carries the L2/L3 digest a scoped-mount consumer needs.

## 1. Queue reason + gate evidence restatement

Queue reason (TaskSpec verbatim): "genuine unshipped BUILD (verifier-ranked SFT exhaust
(SftExhaust absent)) but plan_gate_check exit 1 (missing EXPECTED_OBSERVATION/FORK/ABORT/VERIFY,
legacy plan shape); refine-first per plan-standard + K-27 forbids executing a plan_gate-failing
plan; owner-prioritization gated ML-infra work, parked not lost." gate_json_ref: plan_gate_check exit 1.

Re-run this session (off plan file at HEAD):
`python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md --json`
=> verdict FAIL, exit 1. Per point:

- P1 EXPECTED_OBSERVATION present — fail (count 0)
- P2 ON_FAIL pairs every move — pass
- P3 FORK trigger or FORKS: none — fail
- P4 RECON_NEEDED / GROUNDING coverage — pass (GROUNDING header at plan ln 97)
- P5 ABORT conditions exist — fail
- P6 VERIFY runs with off-agent pass artifact — fail ("no VERIFY/VERIFICATION marker found")
- P7 REDTEAM block records an attack — fail
- P9 vertical slice names PRODUCTION_HOOK/WIRING_CONTRACT — pass (WIRING_CONTRACT at plan ln 66)
- P10 INTEGRATION_SMOKE per unit cadence — fail (9 unit headings, 0 smoke markers)

The queue premise is CONFIRMED, both halves: the build is genuinely unshipped (section 2b) and
the plan genuinely fails the gate (above). This is NOT an H-27/H-29-class stale-recon case; I
checked that failure mode explicitly (git log --all --grep='I-30' is empty; tree-wide symbol
grep is empty).

## 2. Real code seams (verified file:line at a2744e5)

Paths are worktree-relative under `.elai_cc/` unless noted.

### 2a. Sealed RRA source (compose, never re-implement — IS-10)

- `crates/orchestrator/src/rra.rs:542` `pub fn build_rra` and `:555`
  `pub fn build_rra_with_sampling_seed` — construction. PLAN DRIFT: plan cites build_rra:384;
  real line is 542.
- `rra.rs:693` `pub fn write_rra`, `:708` `pub fn read_rra`, `:724` `pub fn verify_rra`
  (returns `ReplayVerdict`) — the walker's read+verify surface. `.rra` is the on-disk bundle
  (rra.rs:3).
- `rra.rs:264` I-18 5-layer blake3 stack hash; `:792-803` sealed stack + objective hash checks
  inside verification — provenance-hash verification ALREADY exists; I-30's blake3(RRA_bytes)
  record check composes on read_rra bytes + verify_rra, zero new crypto.
- Re-export seam: `crates/orchestrator/src/lib.rs:2195` (`verify_rra, write_rra, ...` already
  public to the cli crate).
- CLI precedent that reads `.rra` bundles today: `crates/cli/src/rra_cmd.rs:25` imports
  `export_fixture, read_rra, verify_artifact_tiered, ReplayVerdict, RunArtifact`; handler
  `run_export_fixture` at rra_cmd.rs:115; dispatch `CliAction::Rra` at
  `crates/cli/src/main.rs:800`.
- `crates/orchestrator/src/attestation.rs` exists (G-2/G-8 surface), as the plan asserts.

### 2b. SftExhaust absence (the unshipped BUILD, confirmed)

- `grep -rniE 'SftExhaust|sft_exhaust'` over *.rs/*.py/*.toml, whole worktree: 0 hits.
- `git ls-files` has no `crates/cli/src/sft_exhaust.rs`; no `workflow/training/sft/`.
- `git log --oneline --all --grep='I-30'`: no commits.
- SSB registry has no SftExhaust action type (`crates/orchestrator/src/ssb/action_type.rs`,
  grep 0 hits). Genuine unshipped BUILD.

### 2c. CLI dataset export surface (the production hook target)

- `crates/cli/src/args.rs:75` `pub(crate) enum CliAction` — NO Dataset variant today. Every
  current `dataset` token is a bench/farm flag, not a subcommand: `--dataset PATH` at
  args.rs:839-891 (bench prmbench/mode-routing/conformal/arc) and :2009-2014 + :5247-5382
  (farm-adv-rm run).
- The sibling pattern to clone is the `rra` subcommand family: `pub(crate) enum RraAction`
  args.rs:2220 (`ExportFixture { bundle, adapter }` :2235), hand-rolled parse at
  args.rs:5748-5775, dispatch at main.rs:800-801 into `rra_cmd::run_rra`, handler module
  `crates/cli/src/rra_cmd.rs`. `elai dataset export --format sft` lands as: new
  `CliAction::Dataset { action, .. }` variant near args.rs:75 enum + parse arm next to the
  "rra" arm (:5748 region) + dispatch arm next to main.rs:800 + new `crates/cli/src/dataset_cmd.rs`
  (sibling of rra_cmd.rs / holdout_check_cmd.rs / reward_tamper_cmd.rs) that drives
  `sft_exhaust::SftExhaust`. This is an EXISTING production entrypoint pattern, not greenfield:
  main.rs CliAction dispatch is the always-compiled W-3 surface (same seam class H-29 used for
  `elai drift-check`, args.rs:1849 DriftCheck / main.rs:1381).

### 2d. I-28 DPO pair extractor (negative-pair source — PRESENT, shipped Slice 2)

- SHIPPED at commit 4c61cca ("Add pruned-branch capture and sealed-RRA DPO pair extraction for
  I-28 Slice 2"), in tree at base:
  - `crates/orchestrator/src/dag_credit.rs:48` `pub struct DpoPairExtractor`; `:52` `pub fn run`
    (takes `&SearchLadderOutcome` + `&DpoProvenance`, returns `DagCreditAssignedPayload` rows,
    kind `DagCreditAssignedKind::DpoPairExtracted` :70). Fail-closed: `:56` returns empty when
    `rra_sealed == false`.
  - `DpoProvenance { rra_sealed: bool }` dag_credit.rs:43-45.
  - Pruned-branch source: `crates/orchestrator/src/search/ladder.rs:91` `pub struct PrunedBranch`,
    `:100` `pub struct SearchLadderOutcome` (winner + pruned), Promoted emit :261,
    `TriLayerVerdictKind::HardVeto` mapping :418.
  - Contract test `crates/contracts/tests/i28_dag_credit_contract.rs:111-132`.
- LOAD-BEARING GAP for I-30: the production wire `task_runner.rs:14035-14037` calls the
  extractor with hardcoded `DpoProvenance { rra_sealed: false }` — so the LIVE loop never emits
  DpoPairExtracted rows yet; seal status is not plumbed at PhaseComplete time. Consequence:
  I-30 must NOT plan to consume DpoPairExtracted SSB rows as its negative source. The correct
  compose per plan ln 20 ("via I-28's DPO pair extraction if I-28 is shipped") is to call
  `DpoPairExtractor::run` DIRECTLY from the sft_exhaust walker with
  `DpoProvenance { rra_sealed: true }` — the walker only ever visits sealed RRAs, so the
  provenance precondition is satisfied by construction at the call site. No re-derive branch
  needed; no task_runner edit needed for Tier-1.
- Both upstream gates are default-OFF env gates: `dag_credit_enabled` dag_credit.rs:6
  (ELAI_DAG_CREDIT_ENABLE) and `search_ladder_enabled` ladder.rs:401
  (ELAI_SEARCH_LADDER_ENABLE, wire-dark per I-4-promotion). Sealed ladder-bearing RRAs exist
  only from gate-on runs; the Tier-1 contract test must therefore drive fixtures, and the
  export CLI must handle "0 eligible RRAs" as a clean, observable outcome, not an error.

### 2e. ID-6 gate files (all four present, `crates/orchestrator/src/`)

- H-17 `trial_ledger.rs`: `TrialLedger::open:190`, `trial_count:332`, `effective_trial_count:384`.
- H-18 `deflation_gate.rs`: `compute_dsr_tier1:214`.
- D-6 `holdout_budget.rs`: `classify_holdout_query:319`, `min_holdout_size:377`,
  `HoldoutBudgetLedger::open:433` (opens on the SAME SQLite file as Ssb — the export-time gate
  check can open these ledgers read-only beside the SSB it already reads).
- `gate_self_validation.rs`: `GateId::name:110`, `is_pass:199`.
- Plan's "H-28 (gate_self_validation.rs OR reward-tampering gate — PLAN-asserted)" resolves to
  BOTH existing: H-28 surface is `crates/cli/src/reward_tamper_cmd.rs` + contract test
  `crates/contracts/tests/h28_reward_tampering_contract.rs`; gate_self_validation.rs is a
  separate real file. The Tier-1 export-time check is a LINT (plan ln 49: "emit a lint warning
  ... naming the required gate rows"), not a promotion execution — promotion itself is Tier-2.

### 2f. SSB event seam + stale plan refs (drift to fix at refine)

- New `SftExhaustEvent` lands via: `crates/orchestrator/src/ssb/payloads.rs:2652`
  `pub enum SsbPayload` (+ payload struct beside `DagCreditAssignedPayload:21826`, kind enum
  precedent `DagCreditAssignedKind:21817`, mapping arm :4855) and
  `crates/orchestrator/src/ssb/action_type.rs` (variant :4892 region, string map :5863 region,
  parse :6772 region).
- The action-type LOCK is `crates/orchestrator/tests/action_type_lock.rs` +
  `action_type_lock.golden`. Adding a variant requires the golden relock — that is the
  sanctioned add-a-variant flow, but the executor's TaskSpec must LIST the golden in its
  allowed write set explicitly, or the unit trips the frozen-gate rule. Flag for the refiner.
- PLAN DRIFT (stale grounding refs, plan ln 104): "ssb.rs:2981 / action_type_lock.rs:601" —
  ssb.rs is now 1745 lines total and action_type_lock.rs no longer exists under src/ (the ssb
  module split into ssb/{action_type,payloads,schema,store,tests}.rs). Also build_rra:384 -> :542
  (2a). The W-9 re-read-at-edit rule already anticipates this; the refine pass should update
  the pins.

## 3. plan_gate_check gaps and whether refine-first unblocks without owner

Missing REQUIRED blocks: EXPECTED_OBSERVATION (P1), FORK/FORKS:none (P3), ABORT (P5),
VERIFY+runnable line (P6), REDTEAM (P7), INTEGRATION_SMOKE-per-unit (P10).

The semantic CONTENT for most already exists in legacy shape, so the refine is mechanical
re-blocking, not new design:
- P1: plan ln 92-95 `observable_effect:` (trigger/expect_ssb/expect_artifact + a NEGATIVE case)
  is EXPECTED_OBSERVATION content under a lowercase non-standard header; the checker regex
  requires line-start `EXPECTED_OBSERVATION`.
- P5: plan ln 29-31 EXCLUDED + ln 81-87 stub_ban are ABORT conditions in substance
  (unsealed-RRA rows, DPO fine-tuning, promotion without ledger chain, fabricated stats).
- P6: plan ln 51 names `crates/contracts/tests/i30_sft_exhaust_contract.rs`; VERIFY needs the
  runnable line (`cargo test -p contracts i30_sft_exhaust` — Depot-shim plain `cargo`, per
  build law) + the off-agent pass artifact convention.
- P7: the observable_effect NEGATIVE row (tampered record fails verify_rra =>
  DatasetSealed(provenance_mismatch)) is a REDTEAM attack seed; needs the block + one more
  attack (e.g. unsealed RRA smuggled into --rra-dir; gate-lint bypass by exporting without
  ID-6 rows).
- P3: genuinely new text. Real forks to record: (i) negative source = direct
  DpoPairExtractor::run vs SSB-row consumption (2d resolves: direct call; SSB rows do not
  exist in production yet); (ii) difficulty score d_i source = I-25 router feature vs own
  calibration (plan ln 142-144, open); (iii) dataset_cmd.rs new module vs extending rra_cmd.rs.
- P10: 9 unit headings need per-unit smoke or `INTEGRATION_SMOKE: n/a` for doc-only units; the
  real smoke is the CLI trigger at plan ln 93 run against a fixture RRA dir.

Verdict on refine-first: YES — a doc-only refine (re-block + re-pin lines per 2f + record the
three forks) turns exit 1 into exit 0 with no owner decision, because no fork requires new
owner policy: fork (i) is resolved by code evidence above, forks (ii)/(iii) can be carried AS
recorded FORK blocks with a default and a trigger, which is exactly what P3 wants. What
refine-first does NOT clear: the owner-prioritization PARK on ML-infra work — that is a
scheduling gate above K-27, stays with the owner, and is orthogonal to gate-greening the plan.
Tier-2 (RunPod LoRA run, W-10 checkbox) also stays owner-gated regardless.

## 4. Verdict

**BUILD-WITH-SEAM** (after the K-27 refine lands; do not execute against the plan as-is).

Production seam (existing hooks only, per 2c):
- `crates/cli/src/args.rs:75` `CliAction` + parse arm beside args.rs:5748 (`rra` family
  pattern) — new `dataset export --format sft --rra-dir <dir>` subcommand.
- `crates/cli/src/main.rs:800` dispatch region — arm to new `crates/cli/src/dataset_cmd.rs`
  (rra_cmd.rs sibling), which drives the new `crates/cli/src/sft_exhaust.rs::SftExhaust`.
- Walker composes `read_rra:708` + `verify_rra:724` (rra.rs) for sealed-only eligibility +
  provenance hashes; negative pairs via direct `DpoPairExtractor::run` (dag_credit.rs:52) with
  `DpoProvenance { rra_sealed: true }` at the sealed-walker call site; ID-6 export-time lint
  reads the four gate files' ledgers (2e); `SftExhaustEvent` via ssb/payloads.rs:2652 +
  ssb/action_type.rs + golden relock (allowed-write-set flag, 2f).

Slice plan (Tier-1 offline only; Tier-2 RunPod stays behind the W-10 owner checkbox):
1. REFINE (doc-only, K-27 unblock): re-block the plan per section 3; re-pin drifted refs per
   2a/2f; record forks (ii)/(iii) with defaults. Gate check must exit 0 before any EXECUTE.
2. Slice 1 (BUILD): sft_exhaust.rs walker (sealed-only via verify_rra, winning-branch extract,
   JSONL + blake3 provenance, difficulty-band inclusion floor per J-11 table) + CLI subcommand
   + SftExhaustEvent + contract test fixture with sealed + unsealed + tampered RRAs. Smoke:
   `elai dataset export --format sft --rra-dir <fixture>` emits PathExtracted/DatasetSealed
   with record_count == provenance_verified_count; unsealed RRA rejected with lint, exit clean.
3. Slice 2 (BUILD): negative pairs via DpoPairExtractor at the walker call site
   (NegativePaired rows) + ID-6 gate-chain lint naming required gate rows + dataset card
   frontend_pair (drawer counts off SftExhaustEvent; cross_lang_diff.py covers Tauri->Svelte).
DEFER (unchanged): LoRA run, checkpoint promotion through the full IS-16 chain, MoE retarget
evaluation — all Tier-2/RunPod, owner-gated.

## 5. No-greenfield attestation

Every seam above cites an existing production hook verified at a2744e5: the CliAction dispatch
surface (main.rs:800 family), the sealed-RRA read/verify API (rra.rs:542/693/708/724), the
shipped I-28 extractor (dag_credit.rs:48), the four ID-6 ledgers (2e), and the SSB
payload/action-type registry (2f). The only NEW files are the ones the plan itself mandates
(sft_exhaust.rs, dataset_cmd.rs, contract test, decision doc), each reachable from the named
existing entrypoint in the same changeset (D-030/R-8).

## 6. L2/L3 digest for a scoped-mount consumer (evidence-dir substitute)

Binding plan facts a worktree-scoped executor cannot re-read from the repo root:
- Unit: I-30, Track 17 OPEN-WEIGHT FLYWHEEL, WAVE 5, depends I-4, binds ID-6 + IC-7; SFT
  before GRPO; DPO fine-tuning HARD-AVOID (E-55) — extraction of DPO-shaped PAIRS is in scope,
  training on them is not. Target model stays Qwen2.5-Coder; MoE swap only via RunPod eval.
- Contract: produces `crates/cli/src/sft_exhaust.rs::SftExhaust` +
  `workflow/training/sft/<run_id>.jsonl`; consumes rra.rs build/verify (IS-10), ID-6 gate
  files (IS-16), I-4 ladder trajectories, I-28 pairs; reachable from
  `elai dataset export --format sft`. Stub bans: no unsealed RRA sources (blake3 must match),
  no DPO training, no promotion without full ledger chain, no fabricated stats, no
  .unwrap()/.expect() on production paths, no blake2/sha2.
- Acceptance observation: SftExhaustEvent rows PathExtracted / NegativePaired / DatasetSealed;
  DatasetSealed carries record_count == provenance_verified_count; tampered record =>
  DatasetSealed(provenance_mismatch), never a silent pass; unsealed RRA => no PathExtracted +
  lint error.
- J-11 S-8a amendment: difficulty-banded inclusion floor, no global gate. Bands
  [0,.2)=.90 / [.2,.4)=.85 / [.4,.6)=.80 / [.6,.8)=.70 / [.8,1]=.60; harder-band pass
  preferred over easier-band pass; d_i from I-25 router feature or own calibration; structural
  signal needs a domain substitute (SCIP fn count / cross-language bindings / SSB variants).
- I-SUMMARY_INDEX: I-30 row at ln 554-555; I-28 credit feeds I-30 (ln 519); I-4 ladder is the
  trajectory producer for I-22/I-28/I-30 (ln 93).

END. This worker does not certify DONE; the off-agent verifier and opus controller own that.
