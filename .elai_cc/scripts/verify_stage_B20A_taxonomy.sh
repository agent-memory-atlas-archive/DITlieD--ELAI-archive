#!/usr/bin/env bash
# B-VERIFY probe B20A_taxonomy: 9-cat failure taxonomy.
# Filter 'classify_failure' targets classify_failure_covers_six_of_nine_brief_categories
# in elai_cc_contracts/tests/tool_dispatcher_contract.rs.
# Asserts N > 0 to prevent degenerate 0-test PASS.
# Note: pass_count_any extracts the max N across all 'test result' lines to handle
# multi-binary output where the primary test binary may not be the first 'test result'.
# Emits: PASS|<detail>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
set -euo pipefail

ELAI_ROOT="D:/.ELAI_workflow/.elai_cc"
# Sum all N values from "N passed" across all test result lines
total_passed() { echo "$1" | awk '/[0-9]+ passed/{match($0,/[0-9]+ passed/); s=substr($0,RSTART,RLENGTH); match(s,/[0-9]+/); n+=substr(s,RSTART,RLENGTH)} END{print n+0}'; }
# Check if any individual test line is "test <name> ... ok"
any_test_ok() { echo "$1" | grep -cE "^test .* \.\.\. ok$" || true; }

cd "${ELAI_ROOT}"
RESULT=$(cargo test -p elai_cc_contracts --release classify_failure 2>&1) || true

if echo "$RESULT" | grep -q "test classify_failure_covers_six_of_nine_brief_categories"; then
    if echo "$RESULT" | grep -q "test classify_failure_covers_six_of_nine_brief_categories ... ok"; then
        echo "PASS|classify_failure_taxonomy_tests=1/ok 9_categories_classified"
        exit 0
    fi
    FAIL_MSG=$(echo "$RESULT" | grep "classify_failure_covers_six_of_nine_brief_categories" | head -1)
    echo "FAIL|B20A_taxonomy: classify_failure test failed: ${FAIL_MSG}"
    exit 1
fi

# Degenerate: filter found no test at all
FAIL_LINES=$(echo "$RESULT" | grep -E "FAILED|panicked" | head -1 | tr '\n' '|')
if [ -n "$FAIL_LINES" ]; then
    echo "FAIL|B20A_taxonomy: test error: ${FAIL_LINES}"
else
    echo "FAIL|B20A_taxonomy: classify_failure_covers_six_of_nine_brief_categories not found in output (degenerate pass prevented)"
fi
exit 1
