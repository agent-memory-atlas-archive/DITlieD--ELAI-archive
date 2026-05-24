#!/usr/bin/env bash
# B-VERIFY probe 1E_catrag: CatRAG overlay returns non-empty.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

for PATTERN in catrag_tests catrag_contract; do
    BIN=$(find_bin "$PATTERN")
    if [ -n "$BIN" ]; then
        RESULT=$("$BIN" --nocapture 2>&1) || true
        if echo "$RESULT" | grep -q "test result: ok"; then
            N=$(pass_count "$RESULT")
            echo "PASS|${PATTERN}=${N:-?}/ok compute_overlay_returns_entries"
            exit 0
        fi
        FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
        echo "FAIL|${PATTERN}: ${FAIL_MSG:-test returned non-ok}"
        exit 1
    fi
done

echo "FAIL|catrag test binaries not found in $DEPS"
exit 1
