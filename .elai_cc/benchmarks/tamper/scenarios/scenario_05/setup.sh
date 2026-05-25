#!/usr/bin/env bash
# scenario_05 — MalformedJson (documentation only).
#
# Equivalent runner behavior:
#   1. Seed a sandboxed archive.jsonl with 5 deterministic entries.
#   2. Replace the JSONL line at entry index 2 with a string that
#      serde_json::from_str will reject (e.g. "{not valid json").
#   3. Run verify_archive(fail_fast=false).
#   4. Assert Outcome::ExpectedError { kind: "malformed_json" }.
set -euo pipefail
echo "scenario_05: MalformedJson — corrupt a JSONL line."
