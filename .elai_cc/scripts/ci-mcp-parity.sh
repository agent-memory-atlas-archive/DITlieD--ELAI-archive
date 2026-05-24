#!/usr/bin/env bash
# C-39 CI parity driver -- compare in-process vs --via-mcp SSB row sequences.
#
# Runs `elai run --export-ssb <path>` once with default dispatch (in-process)
# and once with --via-mcp, then invokes ci_mcp_parity_check.py to assert the
# tool_name sequences match. Exit code is non-zero on divergence (BLOCKING
# per WIRING_CONTRACT.stub_ban).
#
# Usage: bash scripts/ci-mcp-parity.sh
# Optional env:
#   ELAI_BIN -- path to elai binary (default: ./target/debug/elai)
#   PARITY_TMPDIR -- directory for the two JSONL exports (default: mktemp -d)
#
# This script lives in .elai_cc/scripts/; the python helper sibling is
# scripts/ci_mcp_parity_check.py (already shipped, unwired prior to C-39).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ELAI_CC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ELAI_BIN="${ELAI_BIN:-$ELAI_CC_DIR/target/debug/elai}"
PARITY_TMPDIR="${PARITY_TMPDIR:-$(mktemp -d)}"

# C-39: keep two SSB exports separate so the python diff is unambiguous.
INPROC_JSON="$PARITY_TMPDIR/c39-inproc.json"
VIAMCP_JSON="$PARITY_TMPDIR/c39-viamcp.json"

echo "ci-mcp-parity: tmpdir=$PARITY_TMPDIR"
echo "ci-mcp-parity: elai-bin=$ELAI_BIN"

if [ ! -x "$ELAI_BIN" ]; then
    echo "ci-mcp-parity: building elai (release mode skipped for CI speed)..."
    (cd "$ELAI_CC_DIR" && cargo build -p elai-cli --locked)
fi

# Both legs use a temp project_root so they do not pollute the active workspace.
LEG_PROJECT_ROOT="$PARITY_TMPDIR/proj"
mkdir -p "$LEG_PROJECT_ROOT"

# Helper: run elai with the given dispatch flag and a short cancellation
# window. Without an explicit task to dispatch the runner subscribes to
# SSB and idles; we send SIGINT after 5s so the export sink flushes
# whatever lifecycle rows (CliMcpServerStarted etc) accumulated. This is
# enough to validate the SSB row structure is symmetric across both
# dispatch legs even on a cold runner.
run_leg() {
    local label="$1"; shift
    local out_path="$1"; shift
    local extra_args=("$@")
    echo "ci-mcp-parity: leg=$label out=$out_path"
    # Run the runner in the background so we can SIGINT it after a brief
    # idle. The runner produces lifecycle rows + the dispatch-trait
    # registration logging without needing a task submission.
    "$ELAI_BIN" run \
        --project-root "$LEG_PROJECT_ROOT" \
        --export-ssb "$out_path" \
        "${extra_args[@]}" &
    local pid=$!
    sleep 5
    if kill -0 "$pid" 2>/dev/null; then
        kill -INT "$pid" || true
    fi
    # Wait so the export sink flushes; tolerate non-zero exit (SIGINT path
    # returns Ok(()) on clean shutdown but timing can produce code 130).
    wait "$pid" 2>/dev/null || true
}

run_leg "inproc" "$INPROC_JSON"
run_leg "viamcp" "$VIAMCP_JSON" --via-mcp

if [ ! -f "$INPROC_JSON" ] || [ ! -f "$VIAMCP_JSON" ]; then
    echo "ci-mcp-parity: ERROR -- one or both exports missing"
    echo "  inproc:  $INPROC_JSON ($( [ -f "$INPROC_JSON" ] && echo present || echo MISSING))"
    echo "  via-mcp: $VIAMCP_JSON ($( [ -f "$VIAMCP_JSON" ] && echo present || echo MISSING))"
    exit 2
fi

# Delegate the diff to the python helper. Exit code is propagated; the
# helper exits non-zero on tool_name sequence divergence.
python3 "$SCRIPT_DIR/ci_mcp_parity_check.py" "$INPROC_JSON" "$VIAMCP_JSON"
