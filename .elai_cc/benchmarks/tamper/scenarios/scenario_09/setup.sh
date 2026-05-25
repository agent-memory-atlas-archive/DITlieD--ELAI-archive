#!/usr/bin/env bash
# scenario_09 — RevertBackup (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Snapshot archive.jsonl -> archive.jsonl.bak.
#   3. Apply MutateContentHash on entry 1.
#   4. Restore from archive.jsonl.bak.
#   5. Run verify_archive(fail_fast=false).
#   6. Assert Outcome::CleanPass — the revert undoes the tamper.
#
# This satisfies plan ACCEPTANCE C: tamper-then-revert recovers to clean.
set -euo pipefail
echo "scenario_09: RevertBackup — tamper then restore from snapshot."
