#!/usr/bin/env bash
# scenario_02 — BrokenParent (documentation only; runner is pure Rust).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Rewrite the parent_hash field at entry index 2 to a 32-byte all-zero
#      hash (no entry earlier in the chain has self_hash == 0).
#   3. Run verify_archive(fail_fast=false).
#   4. Assert Outcome::ExpectedError { kind: "broken_parent" }.
set -euo pipefail
echo "scenario_02: BrokenParent — dangling parent_hash on entry 2."
