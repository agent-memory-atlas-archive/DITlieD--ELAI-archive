D-25 tamper-suite scenarios

10 scenario subdirectories drive D-8 `verify_archive` IN-PROCESS through
the full ArchiveVerifyError taxonomy + idempotency + revert + fail-fast
+ multi-error coverage. Each subdir ships:

- `manifest.json` — machine-readable scenario manifest the runner parses.
- `setup.sh`      — human-readable documentation of the scenario.
                    The runner does NOT execute setup.sh (per D-25
                    DECISION-5; apply_tamper is a typed Rust fn
                    driven from manifest.json).

Coverage matrix:

  scenario_01  PositiveControl          -> CleanPass
  scenario_02  BrokenParent             -> broken_parent
  scenario_03  ContentHashMismatch      -> hash_mismatch
  scenario_04  DuplicateHash            -> duplicate_hash
  scenario_05  MalformedJson            -> malformed_json
  scenario_06  DeleteEntry              -> broken_parent
  scenario_07  AppendMalformedTrailing  -> malformed_json
  scenario_08  MultiTamper              -> {broken_parent, hash_mismatch}
  scenario_09  RevertBackup             -> CleanPass (idempotency)
  scenario_10  FailFastSingleError      -> broken_parent (ff vs collect parity)

Coverage: 4 ArchiveVerifyError variants (HashMismatch, BrokenParent,
DuplicateHash, MalformedJson) + 1 positive control + 1 multi-error
collect-all + 1 revert idempotency + 1 fail-fast parity.

Per-scenario wall budget: 5000ms (plan objective). In-process Rust
harness hits this comfortably — `verify_archive` against the 5-entry
seed chain is microseconds in practice.

Cross-platform: Windows-friendly (the runner is pure Rust; setup.sh
is documentation only).

Add a new scenario:

  1. Create `scenario_NN/manifest.json` with id, kind, tamper_ops,
     expected_outcome, notes.
  2. (optional) Author `scenario_NN/setup.sh` for human readers.
  3. The runner picks it up automatically — directories sorted by
     name + scenarios_blake3 captures both contents AND ordering.
