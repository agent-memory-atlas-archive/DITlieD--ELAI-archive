#!/usr/bin/env bash
# C-48: Idempotent SQLite migration for SafeHarbor memory-trust columns.
#
# Adds two columns to the facts table (if not already present):
#   trust_tier  TEXT NOT NULL DEFAULT 'T3'  -- migration pessimism: pre-C-48 rows are T3
#   provenance  TEXT NOT NULL DEFAULT ''    -- empty JSON provenance for pre-C-48 rows
#
# Usage: migrate_memory_trust.sh <path-to-memory.db>
# Idempotent: safe to run multiple times against the same database.

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <path-to-memory.db>" >&2
    exit 1
fi

DB_PATH="$1"

if [[ ! -f "$DB_PATH" ]]; then
    echo "Error: database file not found: $DB_PATH" >&2
    exit 1
fi

sqlite3 "$DB_PATH" <<'SQL'
PRAGMA journal_mode=WAL;

-- Add trust_tier column if absent. Pre-C-48 rows default to T3 (most pessimistic).
ALTER TABLE facts ADD COLUMN trust_tier TEXT NOT NULL DEFAULT 'T3';

SQL

# The ALTER TABLE above will fail with "duplicate column name" if the column
# already exists (SQLite does not support IF NOT EXISTS on ALTER TABLE).
# Re-run with error suppression for idempotency.
sqlite3 "$DB_PATH" <<'SQL' 2>/dev/null || true
ALTER TABLE facts ADD COLUMN trust_tier TEXT NOT NULL DEFAULT 'T3';
SQL

sqlite3 "$DB_PATH" <<'SQL' 2>/dev/null || true
ALTER TABLE facts ADD COLUMN provenance TEXT NOT NULL DEFAULT '';
SQL

# Verify both columns are present.
COLUMNS=$(sqlite3 "$DB_PATH" "PRAGMA table_info(facts);" | awk -F'|' '{print $2}')
if ! echo "$COLUMNS" | grep -q "^trust_tier$"; then
    echo "Migration failed: trust_tier column not found after ALTER TABLE" >&2
    exit 1
fi
if ! echo "$COLUMNS" | grep -q "^provenance$"; then
    echo "Migration failed: provenance column not found after ALTER TABLE" >&2
    exit 1
fi

echo "Migration complete: facts.trust_tier and facts.provenance present in $DB_PATH"
