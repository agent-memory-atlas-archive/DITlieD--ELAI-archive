#!/usr/bin/env bash
set -euo pipefail

# Entropy quarantine enforcement script.
# Exits 0 if all raw time/entropy access is confined to core/src/clock.rs
# and core/src/entropy.rs (plus TEST_KEEP-marked lines).
# Exits 1 if violations found.

CRATE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC_DIR="$CRATE_ROOT/crates"
VIOLATIONS=0

check_pattern() {
    local pattern="$1"
    local label="$2"
    local hits
    hits=$(grep -rn "$pattern" "$SRC_DIR" --include="*.rs" \
        | grep -v "crates/core/src/clock.rs" \
        | grep -v "crates/core/src/entropy.rs" \
        | grep -v "rust_archived/" \
        | grep -v "TEST_KEEP" \
        || true)
    if [ -n "$hits" ]; then
        echo "VIOLATION ($label):"
        echo "$hits"
        echo ""
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
}

check_pattern "Instant::now()" "raw Instant::now"
check_pattern "SystemTime::now()" "raw SystemTime::now"
check_pattern "thread_rng()" "raw thread_rng"
check_pattern "rand::rngs::OsRng" "raw OsRng usage"
check_pattern "getrandom(" "raw getrandom"

if [ "$VIOLATIONS" -gt 0 ]; then
    echo "FAIL: $VIOLATIONS violation category(ies) found."
    exit 1
fi

echo "PASS: entropy quarantine clean."
exit 0
