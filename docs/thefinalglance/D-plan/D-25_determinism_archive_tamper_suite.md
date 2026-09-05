# D-25: Determinism + archive tamper test suite

Objective: 10 tamper scenarios against D-8 archive verify CLI + C-27
DeterminismDrift detector; each scenario MUST be detected within 5s; clean
repo exits 0; tamper-then-revert recovers.
Estimated effort: 1 session.
Pre-requisites: D-8 (archive verify CLI), C-27 (DeterminismDrift detector).
Parallel-safe with: D-19..D-24.

## Context

D-8 + C-27 are integrity layers; D-25 stress-tests them with concrete tamper
scenarios. Per arxiv:2507.05619 (T1) integrity-checksum recommendation.

10 scenarios:
1. Delete an ArchiveEntry row mid-chain → BrokenParent on subsequent entry.
2. Swap parent_hash field to non-existent hash → BrokenParent.
3. Edit template_body without recomputing self_hash → HashMismatch.
4. Duplicate self_hash across two entries → DuplicateHash.
5. Insert malformed JSON line → MalformedJson.
6. Two `elai /task` runs with same ELAI_SEED produce diverging action_type
   sequences → DeterminismDrift fired.
7. Same as 6 but identical sequences → zero DeterminismDrift rows.
8. Swap blake3 to sha256 on disk (manually rewrite content_hash bytes) →
   HashMismatch.
9. Tamper THEN revert (restore from backup) → verify exits 0 again.
10. Multi-line atomic tamper (lines 3 + 7) → 2 errors reported on full-collect.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-27_testing_trust_t0_wiring_contract.md
- F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-8_crypto_archive_verify.md

## Tasks

1. **Tamper scenario fixtures** at .elai_cc/benchmarks/tamper/scenarios/:
   - 10 scenario directories, each with a setup script + expected verdict.
   - Setup script generates clean archive, applies tamper, runs verify.

2. **TamperRunner** in
   .elai_cc/crates/orchestrator/src/benchmarks/tamper_runner.rs:
   - For each scenario: run setup → run `elai archive verify`; record
     exit code + error types → match against expected.

3. **CLI `elai bench tamper`** runs all 10; exits 0 only if all match
   expected.

4. **Decision doc** workflow/reasoning/D-25-decisions.md (3 decisions):
   - (a) Why 10 scenarios: covers each ArchiveVerifyError variant + at
     least one DeterminismDrift case.
   - (b) Why scenario 9 (tamper-then-revert): verifies idempotency; reverting
     restores clean state.
   - (c) Why scenario 10 (multi-line): tests --fail-fast vs continue mode.

## WIRING_CONTRACT

produces_symbols:
  - TamperRunner + TamperReport
  - 10 scenario setup scripts
  - `elai bench tamper` CLI
consumes_symbols:
  - `elai archive verify` CLI (D-8)
  - DeterminismDrift detector (C-27)
  - SsbWriter
reachable_from:
  - CLI + CI workflow
stub_ban:
  - All 10 scenarios MUST be detected
  - clean repo MUST exit 0 (no false positive)
  - tamper-then-revert MUST recover to clean
observable_effect:
  trigger: `elai bench tamper`
  expect_ssb: 10 BenchmarkStarted + 10 BenchmarkResult rows
  expect_artifact: report at .elai_cc/benchmarks/tamper/reports/

## DETERMINISTIC_CHECK

```bash
ls .elai_cc/benchmarks/tamper/scenarios/ | wc -l
# Expected: 10
cargo check --workspace 2>&1 | tail -3
```

## ACCEPTANCE

- [ ] 10 scenarios present
- [ ] TamperRunner compiles
- [ ] `elai bench tamper` exit 0 only if all 10 match expected
- [ ] cargo check --workspace: zero new errors
- [ ] Cloud smoke test A: all 10 tamper scenarios detected within 5s each
- [ ] Cloud smoke test B: clean repo verify → exit 0
- [ ] Cloud smoke test C: tamper-then-revert → recovers to clean
- [ ] Decision doc workflow/reasoning/D-25-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/benchmarks/tamper_runner.rs
- ADD: .elai_cc/crates/cli/src/bench_tamper.rs
- ADD: .elai_cc/benchmarks/tamper/scenarios/{1..10}/setup.sh + expected.toml
- MODIFY: .elai_cc/crates/cli/src/commands.rs (SlashCommand::BenchTamper)
- ADD: workflow/reasoning/D-25-decisions.md

## Out-of-scope

- Cryptographic-signing tamper (DEFERRED; requires git commit signing
  infrastructure)
- Multi-party signing (DEFERRED)

## Next

Wave 3 complete. Wave 4 (D-26..D-27) dashboard surface for everything.
