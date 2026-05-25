#!/usr/bin/env bash
# scenario_04 — DuplicateHash (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Append a verbatim copy of the JSONL line at entry index 1.
#      Both copies share the same self_hash.
#   3. Run verify_archive(fail_fast=false).
#   4. Assert Outcome::ExpectedError { kind: "duplicate_hash" }.
set -euo pipefail
echo "scenario_04: DuplicateHash — append a byte-copy of an earlier line."
