#!/usr/bin/env bash
set -euo pipefail

# Phase 0.10: Determinism proof for entropy quarantine.
# Runs entropy-scoped tests twice with ELAI_SEED=42 and --test-threads=1,
# then diffs stdout. Exits 0 iff diff=0 and no raw timestamps found.
# Scope: contract tests + seeded_determinism integration tests (not full workspace,
# because pre-existing Windows cli failures cause exit 101 unrelated to entropy).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

TMPDIR_PROOF="${TMPDIR:-/tmp}/elai_determinism_proof_$$"
mkdir -p "$TMPDIR_PROOF"
trap 'rm -rf "$TMPDIR_PROOF"' EXIT

RUN1="$TMPDIR_PROOF/run1.txt"
RUN2="$TMPDIR_PROOF/run2.txt"

echo "=== Step 1: CI quarantine check ==="
bash scripts/check_entropy_quarantine.sh
echo "PASS: quarantine check"

echo ""
echo "=== Step 2: Seeded test run 1 (ELAI_SEED=42, --test-threads=1) ==="
ELAI_SEED=42 cargo test --test clock_entropy_contracts --test seeded_determinism -- --test-threads=1 2>/dev/null | tee "$RUN1"
echo "Run 1 captured ($(wc -l < "$RUN1") lines)"

echo ""
echo "=== Step 3: Seeded test run 2 (ELAI_SEED=42, --test-threads=1) ==="
ELAI_SEED=42 cargo test --test clock_entropy_contracts --test seeded_determinism -- --test-threads=1 2>/dev/null | tee "$RUN2"
echo "Run 2 captured ($(wc -l < "$RUN2") lines)"

echo ""
echo "=== Step 4: Diff check ==="
DIFF_COUNT=0
if ! diff -u "$RUN1" "$RUN2" > "$TMPDIR_PROOF/diff.txt" 2>&1; then
    DIFF_COUNT=$(wc -l < "$TMPDIR_PROOF/diff.txt")
    echo "FAIL: $DIFF_COUNT diff lines between run 1 and run 2"
    head -40 "$TMPDIR_PROOF/diff.txt"
    exit 1
fi
echo "PASS: 0 differences between runs"

echo ""
echo "=== Step 5: Timestamp grep (adversarial check) ==="
# ISO-8601 patterns: 2024-01-01T00:00:00, epoch millis (13+ digits)
TIMESTAMP_HITS=0
for f in "$RUN1" "$RUN2"; do
    # ISO-8601 datetime
    if grep -P '\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}' "$f" | grep -v 'TEST_KEEP' > /dev/null 2>&1; then
        echo "FAIL: ISO-8601 timestamp found in $(basename "$f")"
        grep -P '\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}' "$f" | grep -v 'TEST_KEEP' | head -5
        TIMESTAMP_HITS=$((TIMESTAMP_HITS + 1))
    fi
    # Epoch milliseconds (13+ consecutive digits, not in test names or hex)
    if grep -P '(?<![a-fA-Fx\w])\d{13,}(?![a-fA-Fx\w])' "$f" | grep -v 'TEST_KEEP' > /dev/null 2>&1; then
        echo "FAIL: epoch millis pattern found in $(basename "$f")"
        grep -P '(?<![a-fA-Fx\w])\d{13,}(?![a-fA-Fx\w])' "$f" | grep -v 'TEST_KEEP' | head -5
        TIMESTAMP_HITS=$((TIMESTAMP_HITS + 1))
    fi
done

if [ "$TIMESTAMP_HITS" -gt 0 ]; then
    echo "FAIL: $TIMESTAMP_HITS timestamp pattern matches found"
    exit 1
fi
echo "PASS: no raw timestamps in test output"

echo ""
echo "=== RESULT: ALL CHECKS PASSED ==="
echo "determinism_diff_lines=0"
echo "timestamp_violations=0"
exit 0
