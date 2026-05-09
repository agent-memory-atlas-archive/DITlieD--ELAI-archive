#!/usr/bin/env bash
# B-15A Task 11 — Ollama smoke test.
#
# EXPECTATIONS:
#   - if ollama is NOT running on localhost:11434, exits 0 with a "skipping"
#     message. ollama is an optional sidecar; CI without ollama must not
#     fail this script.
#   - if ollama IS running, sends a minimal canary completion via
#     POST /api/generate and asserts the JSON response contains a non-empty
#     `response` field.
#   - exit 0 on skip or success; exit 1 on a positive-but-unhealthy probe.
#   - --help / -h prints usage and exits 0.
#   - portable: depends only on `curl` and (optionally) `jq` / `python3` for
#     JSON inspection. Falls back to a substring check when neither is
#     available.

set -e

usage() {
    cat <<'EOF'
Usage: smoke_test_ollama.sh [--help|-h]

Pings http://localhost:11434/api/tags. If ollama is offline, exits 0 with a
"skipping" message. If online, sends a canary `POST /api/generate` and asserts
a non-empty response field.

Environment:
  OLLAMA_HOST   default http://localhost:11434
  OLLAMA_MODEL  default qwen2.5-coder:3b (canary only, never pulled)
EOF
}

case "${1-}" in
    --help|-h) usage; exit 0 ;;
esac

OLLAMA_HOST="${OLLAMA_HOST:-http://localhost:11434}"
OLLAMA_MODEL="${OLLAMA_MODEL:-qwen2.5-coder:3b}"

if ! command -v curl >/dev/null 2>&1; then
    echo "smoke_test_ollama: curl not found; skipping (script needs curl)."
    exit 0
fi

# Reachability probe.
if ! curl -fsS -m 2 "${OLLAMA_HOST}/api/tags" >/dev/null 2>&1; then
    echo "smoke_test_ollama: ollama not running at ${OLLAMA_HOST}; skipping."
    exit 0
fi

echo "smoke_test_ollama: ollama reachable at ${OLLAMA_HOST}; sending canary."

# Stream=false so we get a single JSON object back rather than an NDJSON stream.
PAYLOAD="{\"model\":\"${OLLAMA_MODEL}\",\"prompt\":\"ping\",\"stream\":false}"
BODY="$(curl -fsS -m 30 -X POST -H 'Content-Type: application/json' \
    -d "$PAYLOAD" "${OLLAMA_HOST}/api/generate" 2>/dev/null || true)"

if [ -z "$BODY" ]; then
    echo "smoke_test_ollama: empty response from ${OLLAMA_HOST}/api/generate" >&2
    exit 1
fi

# Prefer jq, then python3, then a defensive substring check.
RESPONSE=""
if command -v jq >/dev/null 2>&1; then
    RESPONSE="$(printf '%s' "$BODY" | jq -r '.response // empty' 2>/dev/null || true)"
elif command -v python3 >/dev/null 2>&1; then
    RESPONSE="$(printf '%s' "$BODY" | python3 -c 'import json,sys;d=json.load(sys.stdin);print(d.get("response",""))' 2>/dev/null || true)"
else
    case "$BODY" in
        *'"response"'*) RESPONSE="present" ;;
        *) RESPONSE="" ;;
    esac
fi

if [ -z "$RESPONSE" ]; then
    echo "smoke_test_ollama: response field empty or missing in body:" >&2
    echo "$BODY" >&2
    exit 1
fi

echo "smoke_test_ollama: PASS (response field non-empty)."
exit 0
