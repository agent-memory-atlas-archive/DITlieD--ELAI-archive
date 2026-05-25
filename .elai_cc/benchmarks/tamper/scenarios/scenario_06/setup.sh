#!/usr/bin/env bash
# scenario_06 — DeleteEntry mid-chain (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Delete the JSONL line at entry index 2 (sed -i '3d').
#   3. Run verify_archive(fail_fast=false).
#   4. Assert Outcome::ExpectedError { kind: "broken_parent" } —
#      the next entry's parent_hash references a now-missing self_hash.
set -euo pipefail
echo "scenario_06: DeleteEntry — drop a mid-chain row."
