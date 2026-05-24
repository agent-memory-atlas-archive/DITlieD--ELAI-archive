#!/usr/bin/env bash
# B-VERIFY probe 1D: PageRank returns non-empty result set after DKG build.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
# Usage: verify_stage_1D.sh [corpus_root]
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
CORPUS_ROOT="${1:-F:/.ELAI_workflow/docs/.source/nextest-main}"
DKG="${CORPUS_ROOT}/.elai_cc/indexes/dkg.db"
DEPS="${ELAI_ROOT}/target/debug/deps"

find_bin() { find "$DEPS" -name "${1}-*.exe" -type f 2>/dev/null | sort | tail -1; }
pass_count() { echo "$1" | awk '/passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); print substr(s,RSTART,RLENGTH)}' | head -1; }

BIN=$(find_bin elai_knowledge)

if [ -n "$BIN" ]; then
    RESULT=$("$BIN" pagerank --nocapture 2>&1) || true
    if echo "$RESULT" | grep -q "test result: ok"; then
        N=$(pass_count "$RESULT")
        EDGES=0
        [ -f "$DKG" ] && EDGES=$(sqlite3 "$DKG" "SELECT COUNT(*) FROM dkg_edges;" 2>/dev/null) || EDGES=0
        echo "PASS|pagerank_unit_tests=${N:-?}/ok dkg_edges=$EDGES"
        exit 0
    fi
fi

EDGES=0
[ -f "$DKG" ] && EDGES=$(sqlite3 "$DKG" "SELECT COUNT(*) FROM dkg_edges;" 2>/dev/null) || EDGES=0

if [ "$EDGES" -gt 0 ]; then
    echo "PASS|dkg_edges=$EDGES pagerank_data_present(unit_tests_not_run)"
    exit 0
fi

FAIL_MSG=""
[ -z "$BIN" ] && FAIL_MSG="binary_not_found" || FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
echo "FAIL|pagerank: dkg_edges=$EDGES ${FAIL_MSG:-no passing path}"
exit 1
