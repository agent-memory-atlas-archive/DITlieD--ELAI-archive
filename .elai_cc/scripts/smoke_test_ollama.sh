#!/usr/bin/env bash
set -euo pipefail

# Smoke test: verify ollama connectivity and basic prompt round-trip.
# Returns exit 0 with SKIP=environmental if ollama is unavailable.
# Model can be overridden via OLLAMA_MODEL env var.

OLLAMA_MODEL="${OLLAMA_MODEL:-llama3}"

if ! command -v ollama &>/dev/null; then
    echo "SKIP=environmental (ollama binary not found)"
    exit 0
fi

if ! curl -s --connect-timeout 3 http://localhost:11434/api/tags &>/dev/null; then
    echo "SKIP=environmental (ollama port 11434 unreachable)"
    exit 0
fi

response=$(curl -s -X POST http://localhost:11434/api/generate \
    -d "{\"model\":\"${OLLAMA_MODEL}\",\"prompt\":\"ping\",\"stream\":false}")

if echo "$response" | grep -q '"response"'; then
    echo "OK: ollama round-trip successful (model=${OLLAMA_MODEL})"
    exit 0
elif echo "$response" | grep -q '"error"'; then
    echo "SKIP=environmental (model '${OLLAMA_MODEL}' not available)"
    exit 0
else
    echo "FAIL: ollama responded but missing 'response' field"
    echo "$response"
    exit 1
fi
