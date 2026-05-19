#!/usr/bin/env bash
# B-15A Task 10 — Determinism proof harness.
#
# PLATFORM: POSIX-only. Requires bash, mktemp, sed, awk, shasum or sha256sum,
# and process substitution `<(...)`. On Windows use the sibling
# `determinism_proof.ps1` (PowerShell 7+); cmd.exe is unsupported. C-14
# confirmed the test-target name is `bsp_determinism` (NOT `seeded_determinism`,
# which was a typo in the C-14 plan) and the script already uses `--test
# bsp_determinism`, not `--workspace`.
#
# EXPECTATIONS:
#   - runs the orchestrator's `bsp_determinism` integration test twice with a
#     fixed seed, captures stdout to two tmp files, and exits 0 iff the byte
#     hashes match.
#   - exit 1 when the two runs diverge or either run fails.
#   - --help / -h prints usage and exits 0.
#   - DOES NOT use `set -euo pipefail` — the previous incarnation was reported
#     as failing at Step 2 because pipefail aborted on a benign empty pipe
#     (see workflow/papers/.../phase-0.md). We use plain `set -e` and check
#     each step's exit status explicitly so an empty SSB row never causes a
#     spurious abort.
#   - `ELAI_DETERMINISM_SEED` env var (default `0xB15A`) selects the seed.
#   - the script itself is build-external; date/time use is unrestricted.

set -e

usage() {
    cat <<'EOF'
Usage: determinism_proof.sh [--help|-h]

Runs `cargo test -p elai-orchestrator --test bsp_determinism` twice with a
fixed seed, captures stdout, and asserts byte-identical output across runs.

Exits 0 on match, 1 on divergence or test failure.

Environment:
  ELAI_DETERMINISM_SEED  fixed seed for the test (default: 0xB15A)
EOF
}

case "${1-}" in
    --help|-h) usage; exit 0 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_ROOT="${SCRIPT_DIR}/.."
SEED="${ELAI_DETERMINISM_SEED:-0xB15A}"

OUT_A="$(mktemp -t determinism_a.XXXXXX 2>/dev/null || mktemp)"
OUT_B="$(mktemp -t determinism_b.XXXXXX 2>/dev/null || mktemp)"
trap 'rm -f "$OUT_A" "$OUT_B"' EXIT

run_once() {
    local out_file="$1"
    # Step body kept simple — no piping into head/tail/awk that would let
    # a SIGPIPE / empty intermediate kill the run under pipefail. Capture
    # stdout+stderr, then check the exit code explicitly.
    if ! ( cd "$CRATE_ROOT" && \
        ELAI_DETERMINISM_SEED="$SEED" \
        cargo test -p elai-orchestrator --test bsp_determinism -- --nocapture ) > "$out_file" 2>&1; then
        echo "determinism_proof: cargo test failed; output:" >&2
        cat "$out_file" >&2
        return 1
    fi
    return 0
}

echo "determinism_proof: run 1 with seed=$SEED"
if ! run_once "$OUT_A"; then
    exit 1
fi

echo "determinism_proof: run 2 with seed=$SEED"
if ! run_once "$OUT_B"; then
    exit 1
fi

# Strip lines that legitimately differ between runs (timing / paths) so the
# comparison focuses on the deterministic payload. Keep the filter narrow.
filter() {
    # Tolerates empty input (unlike `grep -v` under pipefail).
    sed -E -e '/finished in/d' \
           -e '/running [0-9]+ test/d' \
           -e 's/[0-9]+ms/Nms/g' \
           -e 's/[0-9]+\.[0-9]+s/N.Ns/g' \
        "$1" || true
}

HASH_A=$(filter "$OUT_A" | shasum -a 256 2>/dev/null | awk '{print $1}')
HASH_B=$(filter "$OUT_B" | shasum -a 256 2>/dev/null | awk '{print $1}')

# Fallback to sha256sum if shasum is missing (most Linux distros).
if [ -z "$HASH_A" ] && command -v sha256sum >/dev/null 2>&1; then
    HASH_A=$(filter "$OUT_A" | sha256sum | awk '{print $1}')
    HASH_B=$(filter "$OUT_B" | sha256sum | awk '{print $1}')
fi

if [ -z "$HASH_A" ] || [ -z "$HASH_B" ]; then
    echo "determinism_proof: failed to compute SHA256 (neither shasum nor sha256sum available)" >&2
    exit 1
fi

echo "determinism_proof: hash_a=$HASH_A"
echo "determinism_proof: hash_b=$HASH_B"

if [ "$HASH_A" = "$HASH_B" ]; then
    echo "determinism_proof: PASS (byte-identical filtered output across 2 runs)"
    exit 0
fi

echo "determinism_proof: FAIL — runs diverged. Diff (first 20 lines):" >&2
diff <(filter "$OUT_A") <(filter "$OUT_B") | head -n 20 >&2 || true
exit 1
