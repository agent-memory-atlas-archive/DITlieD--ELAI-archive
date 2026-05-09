#!/usr/bin/env bash
# stage_onnx_models.sh — Linux/macOS wrapper for stage_onnx_models.py
# Usage: ./stage_onnx_models.sh [--force] [--only graphcodebert,granite,...] [args]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

export ELAI_PROJECT_ROOT="${ELAI_PROJECT_ROOT:-$PROJECT_ROOT}"

# Prefer staging venv if it exists
if [ -f "$PROJECT_ROOT/.venv-staging/bin/python" ]; then
    PYTHON="$PROJECT_ROOT/.venv-staging/bin/python"
elif command -v python3 &>/dev/null; then
    PYTHON="python3"
else
    PYTHON="python"
fi

exec "$PYTHON" "$SCRIPT_DIR/stage_onnx_models.py" "$@"
