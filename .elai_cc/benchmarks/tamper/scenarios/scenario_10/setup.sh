#!/usr/bin/env bash
# scenario_10 — FailFastSingleError (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Apply one tamper (BreakParentLink at entry 2).
#   3. Run verify_archive(fail_fast=true) — assert exactly 1 error.
#   4. Run verify_archive(fail_fast=false) — assert >= 1 error.
#   5. Both arms classify the first error as "broken_parent".
set -euo pipefail
echo "scenario_10: FailFastSingleError — verify ff vs collect-all parity."
