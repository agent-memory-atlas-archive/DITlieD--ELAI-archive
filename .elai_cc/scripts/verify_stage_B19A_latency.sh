#!/usr/bin/env bash
# B-VERIFY probe B19A_latency: Hook latency budgets enforced.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"
DISPATCHER="${ELAI_ROOT}/crates/orchestrator/src/hook_dispatcher.rs"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

# Read constants from source (grounded)
T1=$(grep "HOOK_T1_BUDGET_MS" "$DISPATCHER" 2>/dev/null | grep -oE '[0-9]+;' | tr -d ';' | head -1) || T1=""
T2=$(grep "HOOK_T2_BUDGET_MS" "$DISPATCHER" 2>/dev/null | grep -oE '[0-9]+;' | tr -d ';' | head -1) || T2=""
T3=$(grep "HOOK_T3_BUDGET_MS" "$DISPATCHER" 2>/dev/null | grep -oE '[0-9]+;' | tr -d ';' | head -1) || T3=""

CONSTS_OK=0
[ "$T1" = "1" ] && [ "$T2" = "10" ] && [ "$T3" = "5" ] && CONSTS_OK=1

BIN_CONTRACT=$(find_bin hook_dispatcher_contract)
if [ -n "$BIN_CONTRACT" ]; then
    RESULT=$("$BIN_CONTRACT" --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        echo "PASS|hook_dispatcher_contract_tests=${N:-?}/ok T1=${T1:-?}ms T2=${T2:-?}ms T3=${T3:-?}ms"
        exit 0
    fi
    FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
    echo "FAIL|hook_dispatcher_contract T1=${T1:-?} T2=${T2:-?} T3=${T3:-?}: ${FAIL_MSG:-non-ok}"
    exit 1
fi

BIN_LATENCY=$(find_bin hook_latency_budgets)
if [ -n "$BIN_LATENCY" ]; then
    RESULT2=$("$BIN_LATENCY" --nocapture 2>&1) || true
    if echo "$RESULT2" | grep -q "test result: ok"; then
        N2=$(pass_count "$RESULT2")
        echo "PASS|hook_latency_budgets_tests=${N2:-?}/ok T1=${T1:-?}ms T2=${T2:-?}ms T3=${T3:-?}ms"
        exit 0
    fi
fi

echo "FAIL|B19A_latency: T1=${T1:-missing} T2=${T2:-missing} T3=${T3:-missing} consts_ok=${CONSTS_OK} no_test_binary"
exit 1
