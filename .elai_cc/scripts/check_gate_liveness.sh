#!/usr/bin/env bash
# B-17 Task 1 helper — Gate liveness regression runner.
#
# Runs `cargo test -p elai-orchestrator --test gate_liveness` and reports the
# pass / fail / ignored counts. Exits 0 on clean pass, 1 on any failure, 2 on
# missing manifest / test binary.
#
# Mirrors B-15A scripts/check_anti_stale.sh layout: bash, set -e (NOT
# -euo pipefail per workflow/CLAUDE.md "phase-0 spec"), SCRIPT_DIR + CRATES_DIR
# resolution, no EXIT trap.
#
# `#[ignore]` tests for B-2 / B-4 / B-6 wiring are EXPECTED to remain ignored
# until the corresponding wiring lands; they appear in the "ignored" count and
# do NOT contribute to STRICT_FAIL.

set -e

usage() {
    cat <<'EOF'
Usage: check_gate_liveness.sh [--help|-h]

Runs the B-17 gate-liveness regression suite scoped to elai-orchestrator.

Exits 0 on clean pass, 1 on any failure (one or more wired gate stopped
firing), 2 on environment error (missing manifest, etc.).

Wiring-blocked stubs marked `#[ignore]` are reported but do not affect exit.
EOF
}

case "${1-}" in
    --help|-h) usage; exit 0 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATES_DIR="${SCRIPT_DIR}/../crates"
MANIFEST="${CRATES_DIR}/orchestrator/Cargo.toml"

if [ ! -f "$MANIFEST" ]; then
    echo "check_gate_liveness: manifest not found at $MANIFEST" >&2
    exit 2
fi

OUT="$(mktemp -t gate_liveness_out.XXXXXX 2>/dev/null || mktemp)"

set +e
cargo test --manifest-path "$MANIFEST" --test gate_liveness -- --nocapture > "$OUT" 2>&1
RC=$?
set -e

# Surface the run output verbatim so per-test pass/fail attribution is visible.
cat "$OUT"

# Parse the canonical line:
#   test result: ok. N passed; M failed; K ignored; ...
SUMMARY_LINE="$(grep -E '^test result:' "$OUT" | tail -n 1 || true)"

if [ -z "$SUMMARY_LINE" ]; then
    echo ""
    echo "check_gate_liveness: could not find 'test result:' line in cargo output" >&2
    rm -f "$OUT"
    exit 1
fi

PASSED="$(echo "$SUMMARY_LINE" | sed -n 's/.* \([0-9][0-9]*\) passed.*/\1/p')"
FAILED="$(echo "$SUMMARY_LINE" | sed -n 's/.* \([0-9][0-9]*\) failed.*/\1/p')"
IGNORED="$(echo "$SUMMARY_LINE" | sed -n 's/.* \([0-9][0-9]*\) ignored.*/\1/p')"
PASSED="${PASSED:-0}"
FAILED="${FAILED:-0}"
IGNORED="${IGNORED:-0}"

echo ""
echo "check_gate_liveness: B-17 gate-liveness regression suite"
echo "  passed : $PASSED"
echo "  failed : $FAILED"
echo "  ignored: $IGNORED  (wiring-blocked stubs for B-2/B-4/B-6)"

rm -f "$OUT"

if [ "$RC" -ne 0 ]; then
    echo ""
    echo "check_gate_liveness: cargo test exited $RC — at least one wired gate stopped firing." >&2
    exit 1
fi

if [ "$FAILED" -ne 0 ]; then
    echo ""
    echo "check_gate_liveness: $FAILED failed test(s) — gate liveness regression." >&2
    exit 1
fi

echo ""
echo "check_gate_liveness: clean. All wired gates fired; $IGNORED stub(s) pending wiring."
exit 0
