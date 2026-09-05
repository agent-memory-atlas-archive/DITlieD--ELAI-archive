# H-23: Same-seed re-run hash-equality METRIC_GATE (T-7) — the reproducibility centerpiece

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER D; in-process BUILD on the SHIPPED RRA/determinism substrate)

Objective: ELAI_SEED asserts INTENT (the run was configured deterministically) but nothing RE-RUNS the experiment
and compares the DECLARED METRIC OUTPUTS. A hidden unseeded RNG (np/torch/random, DataLoader shuffle, a missing
determinism env var) silently breaks reproducibility while every existing gate stays green. H-23 re-invokes the
experiment's REAL entrypoint under the recorded pinned env + ELAI_SEED, blake3-hashes the DECLARED METRIC OUTPUTS
(not stdout), and asserts equality vs run #1 — composing the SHIPPED RRA (rra.rs RunArtifact + --emit-rra + replay)
and DriftComparator (determinism.rs first-divergence-index). Plus a T1 seed-coverage AST gate (FAIL on any unseeded
np/torch/random / DataLoader-shuffle / missing determinism env var) and a SCHEMA_REQUIRED env-pin manifest
(lockfile hash, image @sha256, cuda/driver/torch versions). SCOPE (TD-5): same-env/same-seed equality ONLY —
cross-hardware GPU/FP bitwise is physics; record the delta, never assert it.

Estimated effort: ~1 session (M, ~6-8 h). In-process re-run + diff (Tier-1); the re-run executes the real entrypoint.
Pre-requisites: RRA SHIPPED (rra.rs:200 RunArtifact, RunArtifactEmittedPayload ssb.rs:20060, --emit-rra args.rs:2615,
  replay verify run_subcommand.rs:584, replay_resume ssb.rs:23502); DriftComparator SHIPPED (determinism.rs:317,
  system_fingerprint_pin :175); the rules-engine AST_PATTERN + SchemaRequired SHIPPED.
Parallel-safe with: H-24 (sim-real parity — both compose RRA/replay), H-25 (result manifest — shares the digest discipline).
Source: design T-7 + PART 5 (residue); _grounding/H-TRUST-GROUNDING-MAP.md TS-5 + NN-7 + TD-5. Doctrine: H-0_INDEX.md.

## Context

The RRA (G-2, SHIPPED per TS-5 — the G-SUMMARY's "ABSENT" is STALE) already bundles the SSB action stream + ELAI_SEED
+ HardwareClass + system_fingerprint pin + an evidence manifest, and `elai replay --verify` re-runs the action
stream and reports PASS or a DIVERGENCE at the first mismatching event. H-23 adds the missing leg: re-RUN the real
experiment entrypoint and assert the DECLARED-METRIC-OUTPUT blake3 digests are equal across two runs under the same
env+seed. A hidden unseeded RNG makes run#2's metric digest differ from run#1 -> FAIL; a clean experiment hashes
equal twice. The seed-coverage AST gate catches the cause statically (unseeded np/torch/random); the env-pin manifest
(SchemaRequired) records the environment so "same env" is verifiable. The honest scope (TD-5, the design's PART 5
residue): cross-hardware FP bitwise equality is unprovable — H-23 is scoped to same-env/same-seed; cross-hardware
deltas are recorded, never asserted (Thinking Machines batch-invariant kernels are the future bitwise path, not assumed).

## Existing state (B/C/D/E/F/G coverage)
- RunArtifact + emit/replay (crates/orchestrator/src/rra.rs:200; ssb.rs:20060 RunArtifactEmittedPayload;
  ssb_viewer.rs:5248 RunArtifactRow; cli/args.rs:2615 emit_rra; cli/run_subcommand.rs:584 replay verify reading
  artifact.system_fingerprint_pin; ssb.rs:23502 replay_resume) — SHIPPED (G-2). H-23 adds metric-OUTPUT digest
  equality on top (GD-1; do NOT re-implement RRA).
- DriftComparator + DeterminismConfig (determinism.rs:317; system_fingerprint_pin from_env :175; DeterminismDriftPayload
  :403) — SHIPPED (C-27). The blake3-prefix + first-divergence pattern H-23 reuses.
- rules-engine AST_PATTERN + SchemaRequired (predicate.rs:102) — SHIPPED. The seed-coverage gate + env-pin manifest.
- metric-OUTPUT digest re-run-equality — ABSENT (NN-7). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-5, NN-7, TD-5, the H-23 symbol table.
- design T-7 + PART 5; H-24_sim_to_real_parity.md (shares the replay-equality discipline); H-25 (shares the digest binding).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\rra.rs:180-260 (RunArtifact bundle + the evidence manifest)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\run_subcommand.rs:560-600 (the replay --verify path reading system_fingerprint_pin)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\determinism.rs:160-410 (DeterminismConfig from_env + DriftComparator + first-divergence)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\predicate.rs:100-120 (AstPattern + SchemaRequired)
- CLAUDE.md HARD RULES: blake3 (the metric-output digest), no .unwrap()/.expect(); TD-5 (cross-hardware FP residue).

## Tasks (WAVE 3 — in-process BUILD on RRA/determinism)

1. **DECLARED-METRIC-OUTPUT digest (~50 LOC).** Extend the RRA evidence manifest with a blake3 digest over the
   DECLARED metric outputs (the experiment declares which artifacts are its metrics — not stdout, R4). The digest is
   computed at run-end and stored in the RunArtifact.

2. **Re-run hash-equality METRIC_GATE (~80 LOC).** A gate that re-invokes the experiment's REAL entrypoint under the
   recorded env + ELAI_SEED (via the SHIPPED replay path), recomputes the metric-output digest, and asserts equality
   vs run #1 using DriftComparator's first-divergence reporting. A digest mismatch = FAIL with the first diverging
   metric named. Missing a re-run = FAIL (W-1/R1).

3. **Seed-coverage AST gate (~40 LOC).** An AST_PATTERN (T1) failing on any unseeded np.random/torch/random call,
   DataLoader(shuffle=True) without a generator+seed, or a missing determinism env var (e.g. PYTHONHASHSEED,
   CUBLAS_WORKSPACE_CONFIG). BLOCK at TESTING_FIRST. This catches the CAUSE statically before the expensive re-run.

4. **SCHEMA_REQUIRED env-pin manifest (~30 LOC).** A SchemaRequired predicate forcing the experiment to declare a
   pinned-env manifest (lockfile hash, image @sha256, cuda/driver/torch versions). "Same env" is verifiable from this;
   missing = BLOCK.

5. **Off-agent artifact (~20 LOC).** Emit `rerun_hash_equality` (Pattern-2 with Kind: Equal / Divergent; baseline+1
   from LIVE 207/484, re-read + bump e45:294) carrying the two digests + the first-divergence index. The two output
   digests are the off-agent artifact (the re-run is orchestrator-driven, not agent-authored).

6. **Decision doc workflow/reasoning/H-23-decisions.md (~80 LOC).** Records: (a) digest of DECLARED OUTPUTS not
   stdout (R4); (b) the same-env/same-seed SCOPE + the cross-hardware FP residue (TD-5 — recorded, never asserted);
   (c) the seed-coverage AST gate catches the cause before the re-run; (d) composes RRA/DriftComparator, restates neither.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/rerun_equality.rs::verify_rerun_digest (the re-run + digest-equality gate)
  - crates/rules/seed/repro/seed_coverage.toml (the AST_PATTERN seed-coverage gate, mutable=0)
  - crates/rules/seed/repro/env_pin_manifest.toml (the SchemaRequired env-pin gate)
  - crates/orchestrator/src/ssb.rs::SsbActionType::RerunHashEquality (Pattern-2 2-Kind; baseline+1)
  - the RunArtifact metric-output digest field (extension of the SHIPPED RRA manifest)
consumes_symbols:
  - crates/orchestrator/src/rra.rs::RunArtifact + the replay path (run_subcommand.rs:584) — REUSE, GD-1
  - crates/orchestrator/src/determinism.rs::DriftComparator (first-divergence reporting — REUSE)
  - crates/rules/src/predicate.rs::{AstPattern, SchemaRequired} (REUSE)
reachable_from:
  - the experiment/backtest run-end + a verify subcommand (elai replay --verify --metric-digest) [PRODUCTION path]
  - verify_rerun_digest -> the replay re-run -> RerunHashEquality SSB row [off-agent artifact]
stub_ban:
  - NO PASS without both digests (a single run cannot self-certify reproducibility — missing re-run = FAIL, W-1/R1)
  - NO digest over stdout (the DECLARED metric outputs only — R4)
  - NO cross-hardware bitwise assertion (same-env/same-seed scope; cross-hw recorded not asserted — TD-5)
  - NO re-implementation of RRA or DriftComparator (GD-1 P0); blake3 only; no .unwrap()/.expect() in prod
frontend_pair:
  the two digests + Equal/Divergent badge in the RRA viewer (G-2's panel). cross_lang_diff PASS. W-6: a divergent re-run visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- replay --verify --metric-digest <on a fixture experiment with a HIDDEN unseeded RNG, NO env override>"
  expect_ssb: "a RerunHashEquality(Divergent) row lands carrying the two metric-output digests + the first-divergence index, recomputable from the journal (GD-2 same-env)"
  expect_artifact: "the experiment with a hidden unseeded RNG FAILS (run#2's metric digest != run#1's); a clean seeded experiment hashes Equal twice; the seed-coverage AST gate BLOCKs the unseeded call at TESTING_FIRST"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-23_same_seed_rerun_hash_equality.md --run-trigger
rg "verify_rerun_digest|RunArtifact|DriftComparator" .elai_cc/crates/orchestrator/src --glob '!*test*'   # the gate composes RRA + DriftComparator
rg -n "seed_coverage|env_pin" .elai_cc/crates/rules/seed/repro/                                          # the AST + SchemaRequired seeds present
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/rerun_equality.rs                                   # MUST be 0
test -f workflow/reasoning/H-23-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a hidden-unseeded-RNG experiment FAILS digest-equality; a clean one hashes Equal twice). cargo-green != DONE.
- [ ] W-2 default-ON: the gate runs with no env override; kill-switch only.
- [ ] W-3 production entrypoint: verify_rerun_digest fires from the run-end/verify path, not a #[cfg(test)] caller.
- [ ] TD-5: same-env/same-seed SCOPE documented; cross-hardware FP recorded not asserted.
- [ ] GD-1: RRA + DriftComparator REUSED (grep shows one each). GD-2: digests re-compute under ELAI_SEED (same-env).
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; verify_rerun_digest (produced) consumed on the run-end/verify path (W-8) reached by the experiment (W-3).
- W-2: no override; kill-switch only. W-3: live run-end/verify path; not test-only.
- W-4: frontend_pair = RRA viewer digest badge; cross_lang_diff PASS. W-5: runnable replay -> real row; Tier-1 in-process re-run.
- W-6: divergent re-run visible in dashboard. W-7: rra.rs:200 + determinism.rs:317 + predicate.rs grep-confirmed (TS-5); digest-equality absent (NN-7).
- W-8: verify_rerun_digest (produced) consumed on verify; RRA + DriftComparator reused.
- W-9: +RerunHashEquality Pattern-2 baseline+1; bump e45:294.
- W-10: Tier-1 in-process (the re-run executes the real entrypoint; no external service). TD-5: same-env scope honest.

## Tests to add
- crates/contracts/tests/h23_rerun_equality_contract.rs (~150 LOC): (a) hidden unseeded RNG -> Divergent with the
  first-divergence metric; (b) clean seeded experiment -> Equal twice; (c) missing re-run -> FAIL (fail-closed);
  (d) seed-coverage AST gate blocks np.random without seed; (e) missing env-pin manifest -> Block; (f) the digest is
  over declared outputs not stdout; (g) same-env ELAI_SEED twice -> Equal (cross-hardware out of scope).
- crates/contracts/tests/fixtures/repro/ (a hidden-unseeded-RNG experiment + a clean seeded one).

## Files touched
- CREATE: crates/orchestrator/src/rerun_equality.rs (~130 LOC: verify_rerun_digest + the metric-output digest)
- MODIFY: crates/orchestrator/src/rra.rs (~20 LOC: add the declared-metric-output digest field to the manifest)
- CREATE: crates/rules/seed/repro/seed_coverage.toml + env_pin_manifest.toml (the AST + SchemaRequired gates, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+RerunHashEquality Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: crates/cli/src/run_subcommand.rs (~20 LOC: wire --metric-digest into the replay --verify path)
- CREATE: crates/contracts/tests/h23_rerun_equality_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-23-decisions.md (~80 LOC)

## Citations / Grounding
- design T-7 + PART 5; ML reproducibility crisis; Thinking Machines batch-invariant kernels 2025 (bitwise inference,
  the future cross-hardware path — documented, not assumed). WebSearch the determinism env-var set before the AST gate.
- ELAI ground truth (TS-5/NN-7, grep-confirmed 2026-06-04): rra.rs:200 RunArtifact + --emit-rra args.rs:2615 + replay
  run_subcommand.rs:584; determinism.rs:317 DriftComparator; metric-OUTPUT digest re-run-equality ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-5 (cross-hardware FP is physics — record, never assert).

## Open questions / risks
- The re-run is expensive (a full experiment re-execution) — route it pre-promotion / on-demand (elai replay --verify),
  not per-edit. The cheap seed-coverage AST gate is the per-edit early-catch; the re-run is the sound late proof.
- "Declared metric outputs" requires the experiment to declare its metrics — a sandbagging experiment could declare a
  trivial output. The result manifest (H-25) binds the REPORTED number to its provenance, closing that gap; H-23 +
  H-25 ship together for the reproducibility claim to be complete.
- Same-env verification depends on the env-pin manifest being accurate — a lying manifest defeats "same env"; the
  manifest hash is itself in the RRA evidence bundle (tamper-evident), and H-28 reward-tampering guards the harness.

## Next
H-24 (sim-to-real parity) asserts research-path == live-path + replay == frozen golden, composing the same replay
substrate. H-25 (result manifest) binds the reported number to its provenance. Together CLUSTER D makes "the number
is reproducible" a deterministic gate, the believe-the-number control that pairs with G-6 SAFEDELTA.

End of H-23.
