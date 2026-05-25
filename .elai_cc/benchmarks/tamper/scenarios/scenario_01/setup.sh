#!/usr/bin/env bash
# scenario_01 — positive control (documentation only).
#
# The runner does NOT execute this script (per D-25 DECISION-5).
# It is shipped for human readers who want a parallel record of
# the scenario semantics outside the JSON manifest.
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. (no tamper)
#   3. Run `elai archive verify --fail-fast=false`.
#   4. Assert exit 0 (no errors).
set -euo pipefail
echo "scenario_01: positive control — verify a clean 5-entry archive."
echo "Runner behavior: apply zero tamper ops, expect Outcome::CleanPass."
