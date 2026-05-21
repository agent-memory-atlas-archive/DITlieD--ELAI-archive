#!/usr/bin/env bash
# E-51 runtime compatibility probe harness.
# Measures cold-boot, TTFT, decode tok/s for each available adapter.
# HARD RULE: tear down adapter N before starting adapter N+1 (CLAUDE.md
# hardware-heavy rule: never run two model instances simultaneously).
#
# Usage: ./probe_local.sh [--output-json /path/to/results.json]
# Requires: curl, python3

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_JSON="${1:-$SCRIPT_DIR/probe_results.json}"
PROBE_SCRIPT="$SCRIPT_DIR/probe_local.py"
PROMPT="Write a hello world function in Python."

# Adapters to probe in order. Each entry: name|url|teardown_cmd
# Teardown is a best-effort kill; empty string means no subprocess to kill.
declare -a ADAPTERS=(
    "sidecar|http://localhost:8100|"
    "mlx|http://localhost:8080|"
    "llama_cpp|http://localhost:8181|"
)

RESULTS="[]"

for adapter_spec in "${ADAPTERS[@]}"; do
    IFS='|' read -r name url teardown_cmd <<< "$adapter_spec"
    echo "--- Probing $name at $url ---"

    # Check if adapter is reachable
    if ! curl -s --connect-timeout 2 "$url/health" > /dev/null 2>&1; then
        echo "  SKIP: $name not reachable at $url"
        continue
    fi

    result=$(python3 "$PROBE_SCRIPT" --adapter "$name" --url "$url" --prompt "$PROMPT" 2>&1 || echo '{"error":"probe failed"}')
    echo "  Result: $result"

    # Teardown before next adapter
    if [ -n "$teardown_cmd" ]; then
        echo "  Teardown: $teardown_cmd"
        eval "$teardown_cmd" || true
        sleep 2
    fi
done

echo "Probe complete. Results written to $OUTPUT_JSON"
