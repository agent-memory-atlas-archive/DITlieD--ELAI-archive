#!/usr/bin/env bash
# E-47 -- LoRA ship-gate CI lint.
#
# Greps .elai_cc/elai_cc.toml for the deferral token literal
# `DEFERRED_PER_E47_REPORT` AND `confirm_user_understanding = true`.
# Exit 0 when BOTH are present (default-deferred posture intact);
# exit 1 otherwise.
#
# The deferral token is the load-bearing CI marker that prevents a
# silent drift of the ship-gate semantic. Renaming it (typos like
# DEFERED_PER, deferred_per, etc.) trips the lint. The companion
# .bat sibling mirrors this behavior on Windows runners.
#
# Mirrors .elai_cc/scripts/check_anti_stale.{sh,bat} structural
# conventions: SCRIPT_DIR resolution, --help, ripgrep with grep
# fallback, single-pass counts, explicit exit code.

set -e

usage() {
    cat <<'EOF'
Usage: check_lora_ship_gate_token.sh [--help|-h]

Greps .elai_cc/elai_cc.toml for the E-47 LoRA ship-gate deferral
token and user-understanding confirmation.

Exit 0: BOTH `ship_gate = "DEFERRED_PER_E47_REPORT"` AND
        `confirm_user_understanding = true` present in elai_cc.toml.
Exit 1: either marker absent or renamed.
EOF
}

case "${1-}" in
    --help|-h) usage; exit 0 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOML_PATH="${SCRIPT_DIR}/../elai_cc.toml"

if [ ! -f "$TOML_PATH" ]; then
    echo "check_lora_ship_gate_token: elai_cc.toml not found at $TOML_PATH" >&2
    exit 1
fi

TOKEN_PATTERN='ship_gate = "DEFERRED_PER_E47_REPORT"'
CONFIRM_PATTERN='confirm_user_understanding = true'

scan() {
    local pattern="$1"
    if command -v rg >/dev/null 2>&1; then
        rg --fixed-strings --count-matches "$pattern" "$TOML_PATH" 2>/dev/null || echo 0
    else
        grep -Fc "$pattern" "$TOML_PATH" 2>/dev/null || echo 0
    fi
}

TOKEN_COUNT="$(scan "$TOKEN_PATTERN")"
CONFIRM_COUNT="$(scan "$CONFIRM_PATTERN")"

echo "check_lora_ship_gate_token: E-47 LoRA ship-gate token audit"
echo "  ship_gate token  : $TOKEN_COUNT match(es) -- expected >=1"
echo "  user_confirm flag: $CONFIRM_COUNT match(es) -- expected >=1"

FAIL=0
if [ "$TOKEN_COUNT" -lt 1 ]; then
    echo "FAIL: 'ship_gate = \"DEFERRED_PER_E47_REPORT\"' missing from $TOML_PATH" >&2
    FAIL=1
fi
if [ "$CONFIRM_COUNT" -lt 1 ]; then
    echo "FAIL: 'confirm_user_understanding = true' missing from $TOML_PATH" >&2
    FAIL=1
fi

if [ "$FAIL" -eq 0 ]; then
    echo "check_lora_ship_gate_token: PASS -- default-deferred posture intact."
    exit 0
fi

echo ""
echo "check_lora_ship_gate_token: FAIL -- gate semantic drift detected."
echo "Flip per workflow/decisions/E-47-flip-runbook.md or restore the deferral token."
exit 1
