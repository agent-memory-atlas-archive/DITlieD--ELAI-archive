#!/usr/bin/env bash
# B-VERIFY probe 1A: DKG has > 0 nodes after indexing the corpus.
# Emits: PASS|dkg_nodes=<N>  or  FAIL|<reason>
# Exit:  0 = PASS, 1 = FAIL
# Usage: verify_stage_1A.sh <corpus_root>
set -euo pipefail

CORPUS_ROOT="${1:-D:/.ELAI_workflow/docs/.source/nextest-main}"
DKG="${CORPUS_ROOT}/.elai_cc/indexes/dkg.db"

if [ ! -f "$DKG" ]; then
    echo "FAIL|dkg.db not found at $DKG (indexing may not have run)"
    exit 1
fi

COUNT=$(sqlite3 "$DKG" "SELECT COUNT(*) FROM dkg_nodes;" 2>/dev/null) || true

if [ -z "$COUNT" ]; then
    TABLES=$(sqlite3 "$DKG" "SELECT name FROM sqlite_master WHERE type='table';" 2>/dev/null | tr '\n' ',') || TABLES="none"
    echo "FAIL|dkg_nodes table not found; tables=$TABLES"
    exit 1
fi

if [ "$COUNT" -gt 0 ]; then
    echo "PASS|dkg_nodes=$COUNT"
    exit 0
else
    echo "FAIL|dkg_nodes=0"
    exit 1
fi
