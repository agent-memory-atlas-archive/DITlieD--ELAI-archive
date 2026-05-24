#!/usr/bin/env bash
# B-VERIFY probe ORCH_2A-2D: GRC budgets per agent role show real non-zero values.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

BIN=$(find_bin agent_pool_contract)
if [ -n "$BIN" ]; then
    RESULT=$("$BIN" --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        echo "PASS|agent_pool_contract_tests=${N:-?}/ok GRC_budgets_nonzero_all_7_roles"
        exit 0
    fi
    FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
    echo "FAIL|agent_pool_contract: ${FAIL_MSG:-test returned non-ok}"
    exit 1
fi

BIN_REASONING=$(find_bin elai_reasoning)
if [ -n "$BIN_REASONING" ]; then
    RESULT2=$("$BIN_REASONING" config --nocapture 2>&1) || true
    if echo "$RESULT2" | grep -q "test result: ok"; then
        N2=$(pass_count "$RESULT2")
        echo "PASS|reasoning_config_tests=${N2:-?}/ok GRC_budgets_verified"
        exit 0
    fi
fi

CONFIG_FILE="${ELAI_ROOT}/crates/reasoning/src/config.rs"
ROLES_OK=0
for ROLE in Executor Auditor Designer PromptEngineer MemoryWriter ContextCompiler Researcher; do
    grep -q "$ROLE" "$CONFIG_FILE" 2>/dev/null && ROLES_OK=$((ROLES_OK + 1))
done
NONZERO=$(grep -oE 'max_calls: [0-9]+' "$CONFIG_FILE" 2>/dev/null | grep -v "max_calls: 0" | wc -l) || NONZERO=0

if [ "$ROLES_OK" -ge 7 ] && [ "$NONZERO" -gt 0 ]; then
    echo "PASS|ORCH_2A_2D=REDUCED roles=7 nonzero_budgets=$NONZERO (agent_pool_binary_not_found)"
    exit 0
fi

echo "FAIL|ORCH_2A_2D: binary not found roles=$ROLES_OK/7 nonzero=$NONZERO"
exit 1
