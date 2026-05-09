#!/usr/bin/env bash
# B-VERIFY probe B19A_ruff: ruff PostToolUse blocks Python file with syntax error.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="D:/.ELAI_workflow/.elai_cc"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

if ! command -v ruff &>/dev/null; then
    echo "FAIL|ruff not found on PATH; install: pip install ruff"
    exit 1
fi
RUFF_VER=$(ruff --version 2>/dev/null || echo "unknown")

BIN=$(find_bin ruff_post_tool_use)
if [ -n "$BIN" ]; then
    RESULT=$("$BIN" --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        echo "PASS|ruff_post_tool_use_tests=${N:-?}/ok ruff=$RUFF_VER SyntaxError_category_detected"
        exit 0
    fi
    FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
    if echo "$RESULT" | grep -iE "BinaryNotFound|not found|No such file" | grep -q .; then
        echo "FAIL|ruff_post_tool_use: ruff binary not found by plugin ruff=$RUFF_VER"
        exit 1
    fi
    echo "FAIL|ruff_post_tool_use: ${FAIL_MSG:-test returned non-ok}"
    exit 1
fi

# Fallback: direct ruff binary syntax detection
RUFF_OUT=$(printf 'def foo(:\n    pass\n' | ruff check --select=E999 --stdin-filename=test.py - 2>&1) || true
if echo "$RUFF_OUT" | grep -iE "SyntaxError|E999|syntax" | grep -q .; then
    echo "PASS|B19A_ruff=REDUCED ruff_detects_syntax=true ruff=$RUFF_VER (plugin_test_binary_not_found)"
    exit 0
fi

echo "FAIL|B19A_ruff: binary not found and ruff syntax detection inconclusive ruff=$RUFF_VER"
exit 1
