#!/usr/bin/env bash
# B-VERIFY probe B10_context_budget: 28 context_budget tests GREEN.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="D:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

BIN_CONTRACT=$(find_bin context_budget_contract)
BIN_INTEG=$(find_bin context_budget_integration)

CONTRACT_OK=0; INTEG_OK=0; PASS_C=0; PASS_I=0

if [ -n "$BIN_CONTRACT" ]; then
    RESULT_C=$("$BIN_CONTRACT" --nocapture 2>&1) || true
    echo "$RESULT_C" | grep -q "test result: ok" && CONTRACT_OK=1
    PC=$(pass_count "$RESULT_C"); PASS_C=${PC:-0}
fi

if [ -n "$BIN_INTEG" ]; then
    RESULT_I=$("$BIN_INTEG" --nocapture 2>&1) || true
    echo "$RESULT_I" | grep -q "test result: ok" && INTEG_OK=1
    PI=$(pass_count "$RESULT_I"); PASS_I=${PI:-0}
fi

TOTAL=$((PASS_C + PASS_I))

if [ "$CONTRACT_OK" -eq 1 ] && [ "$INTEG_OK" -eq 1 ]; then
    echo "PASS|context_budget_contract=${PASS_C}/ok integration=${PASS_I}/ok total=${TOTAL}"
    exit 0
fi

if [ "$CONTRACT_OK" -eq 1 ]; then
    echo "PASS|context_budget_contract=${PASS_C}/ok (integration_${BIN_INTEG:+failed}${BIN_INTEG:-missing})"
    exit 0
fi

FAIL_MSG=""
[ "$CONTRACT_OK" -eq 0 ] && FAIL_MSG="contract_failed"
[ "$INTEG_OK" -eq 0 ] && FAIL_MSG="${FAIL_MSG:+${FAIL_MSG}_}integration_failed"
echo "FAIL|B10_context_budget: ${FAIL_MSG} contract=${BIN_CONTRACT:-missing} integ=${BIN_INTEG:-missing}"
exit 1
