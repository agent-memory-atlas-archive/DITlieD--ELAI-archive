#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SETTINGS_DIR="${SCRIPT_DIR}/../searxng-settings"

docker run -d \
    --name searxng \
    --restart unless-stopped \
    -p 8888:8080 \
    -v "${SETTINGS_DIR}/settings.yml:/etc/searxng/settings.yml:ro" \
    searxng/searxng:latest

echo "SearXNG started on http://localhost:8888"
