#!/usr/bin/env bash
# B-VERIFY probe 1C: Vector search returns non-empty results (ONNX models required).
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
ONNX_DIR="${ELAI_ROOT}/models/onnx"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

for MODEL in graphcodebert.onnx granite-embed.onnx graphcodebert-tokenizer.json granite-embed-tokenizer.json; do
    if [ ! -f "${ONNX_DIR}/${MODEL}" ]; then
        echo "FAIL|ONNX model missing: $MODEL at $ONNX_DIR"
        exit 1
    fi
done

BIN=$(find_bin knowledge_vector_contract)
if [ -z "$BIN" ]; then
    echo "FAIL|knowledge_vector_contract binary not found in $DEPS"
    exit 1
fi

RESULT=$("$BIN" --nocapture 2>&1) || true

if echo "$RESULT" | grep -q "test result: ok"; then
    N=$(pass_count "$RESULT")
    echo "PASS|vector_contract_tests=${N:-?}/ok ONNX_sessions_loaded"
    exit 0
fi

FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|error\[|panicked" | head -2 | tr '\n' '|')
echo "FAIL|knowledge_vector_contract: ${FAIL_MSG:-test returned non-ok}"
exit 1
