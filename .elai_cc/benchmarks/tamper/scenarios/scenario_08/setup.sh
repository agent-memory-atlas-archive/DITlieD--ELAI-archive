#!/usr/bin/env bash
# scenario_08 — MultiTamper / atomic 2-op (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Apply two tampers in one file:
#      a. Break parent_hash at entry index 1.
#      b. Mutate template_body at entry index 3.
#   3. Run verify_archive(fail_fast=false).
#   4. Assert collect-all surfaces both errors; first-error kind
#      matches the FIRST tamper-op (broken_parent).
set -euo pipefail
echo "scenario_08: MultiTamper — two errors in one file."
