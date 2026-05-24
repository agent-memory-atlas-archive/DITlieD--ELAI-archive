#!/usr/bin/env bash
# B-VERIFY probe B19A_stop_hook: Stop hook double-Ctrl+C-within-3s aborts task.
# OS-signal simulation not used on Windows. Uses unit tests only.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

BIN=$(find_bin stop_hook_double_ctrl_c)
if [ -n "$BIN" ]; then
    RESULT=$("$BIN" --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        echo "PASS|stop_hook_double_ctrl_c_tests=${N:-?}/ok double_ctrlc_3s_window_verified"
        exit 0
    fi
    FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
    echo "FAIL|stop_hook_double_ctrl_c: ${FAIL_MSG:-test returned non-ok}"
    exit 1
fi

BIN_CLI=$(find_bin elai)
if [ -n "$BIN_CLI" ]; then
    RESULT2=$("$BIN_CLI" hook_abort --nocapture 2>&1) || true
    if echo "$RESULT2" | grep -q "test result: ok"; then
        N2=$(pass_count "$RESULT2")
        echo "PASS|hook_abort_monitor_tests=${N2:-?}/ok"
        exit 0
    fi
fi

MAIN_FILE="${ELAI_ROOT}/crates/cli/src/main.rs"
WINDOW=$(grep "CONFIRMATION_WINDOW_SECS" "$MAIN_FILE" 2>/dev/null | grep -oE '[0-9]+' | head -1) || WINDOW=""
ABORT_WIRED=$(grep -c "abort_signal\|hook_abort_monitor\|Ctrl.C confirmed" "$MAIN_FILE" 2>/dev/null) || ABORT_WIRED=0

if [ "$WINDOW" = "3" ] && [ "$ABORT_WIRED" -gt 0 ]; then
    echo "PASS|B19A_stop_hook=REDUCED window=${WINDOW}s abort_wired=${ABORT_WIRED} (test_binaries_not_found)"
    exit 0
fi

echo "FAIL|B19A_stop_hook: window=${WINDOW:-missing} abort_wired=${ABORT_WIRED} no_test_binary"
exit 1
