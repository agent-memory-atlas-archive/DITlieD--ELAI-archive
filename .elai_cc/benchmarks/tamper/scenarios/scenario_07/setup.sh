#!/usr/bin/env bash
# scenario_07 — AppendMalformedTrailing (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Append a trailing garbage line (e.g. echo '{trailing garbage'
#      >> archive.jsonl).
#   3. Run verify_archive(fail_fast=false).
#   4. Assert Outcome::ExpectedError { kind: "malformed_json" }.
set -euo pipefail
echo "scenario_07: AppendMalformedTrailing — garbage at EOF."
