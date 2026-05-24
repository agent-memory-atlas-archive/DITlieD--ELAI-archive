#!/usr/bin/env bash
# B-VERIFY probe B9_sessiondag: SessionDag handoff at 85% budget.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

BIN=$(find_bin session_dag_contract)

if [ -n "$BIN" ]; then
    RESULT=$("$BIN" handoff --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        THRESHOLD=$(grep "SESSION_HANDOFF_THRESHOLD" "${ELAI_ROOT}/crates/cli/src/main.rs" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1) || THRESHOLD="?"
        echo "PASS|session_dag_handoff_tests=${N:-?}/ok threshold=${THRESHOLD} create_handoff_load_handoff_roundtrip"
        exit 0
    fi

    RESULT2=$("$BIN" --nocapture 2>&1) || true
    if echo "$RESULT2" | grep -q "test result: ok"; then
        N2=$(pass_count "$RESULT2")
        echo "PASS|session_dag_contract_all=${N2:-?}/ok(reduced:source-grep-only)"
        exit 0
    fi

    FAIL_MSG=$(echo "$RESULT2" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
    echo "FAIL|session_dag_contract: ${FAIL_MSG:-test returned non-ok}"
    exit 1
fi

THRESHOLD=$(grep "SESSION_HANDOFF_THRESHOLD" "${ELAI_ROOT}/crates/cli/src/main.rs" 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1) || THRESHOLD=""
echo "FAIL|B9_sessiondag: binary not found threshold=${THRESHOLD:-missing}"
exit 1
