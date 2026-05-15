#!/usr/bin/env bash
#
# B-31.5 Tier-B helper. Polls the project's metadata.db every 5s; SIGKILLs
# the elai indexer (best-effort by PID file) once the indexed-batch
# progress percentage reaches $2.
#
# Usage:
#   kill_at_progress.sh <fixture_root> <kill_pct> [<pid_file>]
#
# fixture_root is the project root (the parent of .elai_cc/.elai_cc/indexes).
# kill_pct is an integer 1..=99. pid_file defaults to <fixture_root>/.elai.pid.
set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "usage: kill_at_progress.sh <fixture_root> <kill_pct> [<pid_file>]" >&2
    exit 2
fi

FIXTURE="$1"
KILL_PCT="$2"
PID_FILE="${3:-$FIXTURE/.elai.pid}"

if ! [[ "$KILL_PCT" =~ ^[0-9]+$ ]] || (( KILL_PCT < 1 || KILL_PCT > 99 )); then
    echo "error: kill_pct must be integer in 1..=99, got '$KILL_PCT'" >&2
    exit 2
fi

# DECISION-2: METADATA_DB_FILENAME constant value is metadata.db; the path
# layout is <project_root>/.elai_cc/indexes/metadata.db (NOT nested under
# another .elai_cc).
METADATA_DB="$FIXTURE/.elai_cc/indexes/metadata.db"

# Wait for the DB to appear.
WAIT_LIMIT=120
WAITED=0
while [[ ! -f "$METADATA_DB" ]]; do
    if (( WAITED >= WAIT_LIMIT )); then
        echo "error: metadata DB never appeared at $METADATA_DB (waited ${WAIT_LIMIT}s)" >&2
        exit 1
    fi
    sleep 5
    WAITED=$(( WAITED + 5 ))
done

# Poll every 5s.
while true; do
    if ! command -v sqlite3 >/dev/null 2>&1; then
        echo "error: sqlite3 command not found in PATH" >&2
        exit 3
    fi
    # SELECT batches_processed, batches_total from the first row.
    ROW=$(sqlite3 "$METADATA_DB" \
        "SELECT batches_processed || ' ' || COALESCE(batches_total, 0) FROM project_metadata LIMIT 1;" 2>/dev/null || true)
    if [[ -z "$ROW" ]]; then
        sleep 5
        continue
    fi
    BATCHES_PROC=$(echo "$ROW" | awk '{print $1}')
    BATCHES_TOTAL=$(echo "$ROW" | awk '{print $2}')
    if [[ "$BATCHES_TOTAL" == "0" ]]; then
        sleep 5
        continue
    fi
    PCT=$(( BATCHES_PROC * 100 / BATCHES_TOTAL ))
    if (( PCT >= KILL_PCT )); then
        if [[ -f "$PID_FILE" ]]; then
            ELAI_PID=$(cat "$PID_FILE")
            echo "kill_at_progress: pct=$PCT >= $KILL_PCT, killing pid=$ELAI_PID" >&2
            kill -KILL "$ELAI_PID" 2>/dev/null || true
        else
            echo "kill_at_progress: pct=$PCT >= $KILL_PCT but no pid file at $PID_FILE" >&2
        fi
        # Record the kill point for the Tier-B metrics aggregator.
        echo "killed_at_batch=$BATCHES_PROC"
        echo "killed_at_pct=$PCT"
        exit 0
    fi
    sleep 5
done
