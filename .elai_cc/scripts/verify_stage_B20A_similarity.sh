#!/usr/bin/env bash
# B-VERIFY probe B20A_similarity: dispatch_edit emits SimilarityHint when NotMatched.
# Tests live in elai-tools/tests/edit_similarity_hint.rs.
# Filter 'near_match_emits_hint' targets near_match_emits_hint_above_threshold.
# Checks for explicit test-ok line to prevent degenerate 0-test PASS.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"

cd "${ELAI_ROOT}"
RESULT=$(cargo test -p elai-tools --release near_match_emits_hint 2>&1) || true

if echo "$RESULT" | grep -q "test near_match_emits_hint_above_threshold ... ok"; then
    echo "PASS|similarity_hint_tests=1/ok dispatch_edit_err_hint_line_score_nonzero"
    exit 0
fi

if echo "$RESULT" | grep -q "test near_match_emits_hint_above_threshold"; then
    FAIL_MSG=$(echo "$RESULT" | grep "near_match_emits_hint_above_threshold" | head -1)
    echo "FAIL|B20A_similarity: near_match test failed: ${FAIL_MSG}"
    exit 1
fi

FAIL_MSG=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
echo "FAIL|B20A_similarity: near_match_emits_hint filter matched no tests (degenerate pass prevented): ${FAIL_MSG:-no match}"
exit 1
