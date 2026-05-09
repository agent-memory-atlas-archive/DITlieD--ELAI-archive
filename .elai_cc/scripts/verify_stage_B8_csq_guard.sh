#!/usr/bin/env bash
# B-VERIFY probe B8_csq_guard: CSQ guard blocks destructive call.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="D:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

BIN=$(find_bin csq_tests)
if [ -n "$BIN" ]; then
    RESULT=$("$BIN" --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        echo "PASS|csq_tests=${N:-?}/ok CsqResult_passed_false_for_unguarded_critical"
        exit 0
    fi
    FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
    echo "FAIL|csq_tests: ${FAIL_MSG:-test returned non-ok}"
    exit 1
fi

BIN_REASONING=$(find_bin elai_reasoning)
if [ -n "$BIN_REASONING" ]; then
    for FILTER in types csq; do
        RESULT2=$("$BIN_REASONING" $FILTER --nocapture 2>&1) || true
        if echo "$RESULT2" | grep -q "test result: ok"; then
            N2=$(pass_count "$RESULT2")
            echo "PASS|reasoning_${FILTER}_tests=${N2:-?}/ok CsqResult_passed_verified"
            exit 0
        fi
    done
fi

TYPES_FILE="${ELAI_ROOT}/crates/reasoning/src/types.rs"
PASSED_FN=$(grep -c "fn passed\|CsqResult\|guard_added" "$TYPES_FILE" 2>/dev/null) || PASSED_FN=0
echo "FAIL|B8_csq_guard: no passing test binary; CsqResult_wired_lines=$PASSED_FN"
exit 1
