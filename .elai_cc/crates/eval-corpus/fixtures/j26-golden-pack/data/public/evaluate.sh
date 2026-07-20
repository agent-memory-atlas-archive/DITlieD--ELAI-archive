#!/usr/bin/env bash
# J-26 golden-pack evaluator (deterministic, no LLM).
#
# Usage: evaluate.sh --gen-dir <dir>
# Writes <dir>/results.json with a known scalar {"fitness": 0.87} and exits 0.
# The contract test asserts run_evaluator reads this scalar and that
# results_to_execution_fitness.fitness == 0.87.

set -euo pipefail

GEN_DIR=""
while [ $# -gt 0 ]; do
    case "$1" in
        --gen-dir)
            GEN_DIR="$2"
            shift 2
            ;;
        *)
            echo "evaluate.sh: unknown arg: $1" >&2
            exit 2
            ;;
    esac
done

if [ -z "$GEN_DIR" ]; then
    echo "evaluate.sh: --gen-dir is required" >&2
    exit 2
fi

mkdir -p "$GEN_DIR"
cat > "$GEN_DIR/results.json" <<'EOF'
{"fitness": 0.87, "pass_rate": 0.9, "tests_total": 3, "tests_passed": 3}
EOF
exit 0
