#!/usr/bin/env bash
# scenario_03 — ContentHashMismatch (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Mutate template.template_body at entry index 1 (e.g. via jq) WITHOUT
#      recomputing the entry-level self_hash field.
#   3. Run verify_archive(fail_fast=false).
#   4. Assert Outcome::ExpectedError { kind: "hash_mismatch" }.
set -euo pipefail
echo "scenario_03: ContentHashMismatch — mutate body without rehash."
