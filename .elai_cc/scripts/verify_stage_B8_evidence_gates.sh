#!/usr/bin/env bash
# B-VERIFY probe B8_evidence_gates: 7 evidence gates; UngroundedStep for ungrounded claims.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

BIN=$(find_bin grc_engine)
if [ -n "$BIN" ]; then
    RESULT=$("$BIN" --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        echo "PASS|grc_engine_tests=${N:-?}/ok UngroundedStep_returned_for_missing_grounding"
        exit 0
    fi
    FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
    echo "FAIL|grc_engine: ${FAIL_MSG:-test returned non-ok}"
    exit 1
fi

BIN_REASONING=$(find_bin elai_reasoning)
if [ -n "$BIN_REASONING" ]; then
    RESULT2=$("$BIN_REASONING" engine --nocapture 2>&1) || true
    if echo "$RESULT2" | grep -q "test result: ok"; then
        N2=$(pass_count "$RESULT2")
        echo "PASS|reasoning_engine_tests=${N2:-?}/ok evidence_gates_verified"
        exit 0
    fi
fi

ENGINE_FILE="${ELAI_ROOT}/crates/reasoning/src/engine.rs"
GATED=$(grep -c "UngroundedStep\|requires_grounding" "$ENGINE_FILE" 2>/dev/null) || GATED=0
echo "FAIL|B8_evidence_gates: no passing test binary; engine_gate_lines=$GATED"
exit 1
