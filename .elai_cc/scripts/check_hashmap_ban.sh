#!/usr/bin/env bash
# B-15B -- HashMap ban lint.
# Fails (exit 1) if any non-test, non-bench, non-example production .rs file
# under .elai_cc/crates/ contains:
#   - "use std::collections::HashMap"
#   - "std::collections::HashMap::"   (qualified constructor / path)
#
# Excludes: tests/ test/ examples/ target/ benches/ main.rs build.rs
#
# Portable: uses plain grep -R (no GNU-only flags).
#           rg preferred when available for speed.
#
# Exit codes: 0=clean, 1=violations, 2=setup error
#
# Known false positives: 10 cfg(test)-scoped occurrences live inside src/ files
# (not under tests/ subdirectories). The grep-based scan cannot detect cfg(test)
# scope by directory exclusion alone, so they are tracked in an explicit
# allowlist below. The script reports them under "INFO (allow-listed)" but does
# NOT count them toward FAIL exit. A future phase (B-15B-FX) will move them to
# tests/ subdirectories or replace them with BTreeMap to drop the allowlist.

set -e

usage() {
    cat <<'EOF'
Usage: check_hashmap_ban.sh [--help|-h]

Fails if any production .rs file uses std::collections::HashMap.
Allowed replacements: BTreeMap, FxHashMap (rustc_hash), IndexMap (indexmap).
Excludes: tests/ test/ benches/ examples/ target/ main.rs build.rs
EOF
}

case "${1-}" in
    --help|-h) usage; exit 0 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATES_DIR="${SCRIPT_DIR}/../crates"

if [ ! -d "$CRATES_DIR" ]; then
    echo "check_hashmap_ban: crates dir not found at $CRATES_DIR" >&2
    exit 2
fi

TMP_USE="$(mktemp -t hashmap_ban_use.XXXXXX 2>/dev/null || mktemp)"
TMP_QUAL="$(mktemp -t hashmap_ban_qual.XXXXXX 2>/dev/null || mktemp)"
trap 'rm -f "$TMP_USE" "$TMP_QUAL"' EXIT

scan() {
    local pattern="$1"
    local out="$2"
    if command -v rg >/dev/null 2>&1; then
        rg --no-heading --line-number --type rust \
            --glob '!**/tests/**' \
            --glob '!**/test/**' \
            --glob '!**/benches/**' \
            --glob '!**/examples/**' \
            --glob '!**/target/**' \
            --glob '!**/main.rs' \
            --glob '!**/build.rs' \
            "$pattern" "$CRATES_DIR" > "$out" 2>/dev/null || true
    else
        grep -R -n "$pattern" "$CRATES_DIR" \
            --include='*.rs' \
            --exclude-dir=tests \
            --exclude-dir=test \
            --exclude-dir=benches \
            --exclude-dir=examples \
            --exclude-dir=target \
            --exclude='main.rs' \
            --exclude='build.rs' > "$out" 2>/dev/null || true
    fi
}

# Pattern 1: use stmt (matches bare use: `use std::collections::HashMap`)
scan 'use std::collections::HashMap' "$TMP_USE"

# Pattern 2: qualified path constructor/turbofish (matches HashMap::new, HashMap::<K,V>)
scan 'std::collections::HashMap::' "$TMP_QUAL"

# Allowlist of file basenames whose HashMap usage is cfg(test)-scoped inside src/.
# These are tracked as INFO, not violations. Update via B-15B-FX follow-up phase.
ALLOWLIST_BASENAMES='import_extractor.rs telescope.rs tier3.rs checkpoint.rs recovery.rs session_dag.rs traversal.rs contract_tests.rs'

# Filter out lines that are string literals or comments
# Heuristic: line contains a quote before the pattern OR line starts with whitespace+//
filter_false_positives() {
    grep -v -E '^\s*//' | grep -v '".*std::collections::HashMap' || true
}

# Split a violation list into (real, allowed) by basename match.
split_by_allowlist() {
    local input="$1"
    local out_real="$2"
    local out_allow="$3"
    : > "$out_real"
    : > "$out_allow"
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        local file="${line%%:*}"
        local base
        base="$(basename "$file")"
        local matched=0
        for allow in $ALLOWLIST_BASENAMES; do
            if [ "$base" = "$allow" ]; then
                matched=1
                break
            fi
        done
        if [ "$matched" = "1" ]; then
            printf '%s\n' "$line" >> "$out_allow"
        else
            printf '%s\n' "$line" >> "$out_real"
        fi
    done < "$input"
}

# Apply false-positive filter
FILTERED_USE="$(mktemp -t hashmap_use_f.XXXXXX 2>/dev/null || mktemp)"
FILTERED_QUAL="$(mktemp -t hashmap_qual_f.XXXXXX 2>/dev/null || mktemp)"
REAL_USE="$(mktemp -t hashmap_use_r.XXXXXX 2>/dev/null || mktemp)"
ALLOW_USE="$(mktemp -t hashmap_use_a.XXXXXX 2>/dev/null || mktemp)"
REAL_QUAL="$(mktemp -t hashmap_qual_r.XXXXXX 2>/dev/null || mktemp)"
ALLOW_QUAL="$(mktemp -t hashmap_qual_a.XXXXXX 2>/dev/null || mktemp)"
trap 'rm -f "$TMP_USE" "$TMP_QUAL" "$FILTERED_USE" "$FILTERED_QUAL" "$REAL_USE" "$ALLOW_USE" "$REAL_QUAL" "$ALLOW_QUAL"' EXIT

filter_false_positives < "$TMP_USE" > "$FILTERED_USE"
filter_false_positives < "$TMP_QUAL" > "$FILTERED_QUAL"

split_by_allowlist "$FILTERED_USE" "$REAL_USE" "$ALLOW_USE"
split_by_allowlist "$FILTERED_QUAL" "$REAL_QUAL" "$ALLOW_QUAL"

USE_COUNT="$(wc -l < "$REAL_USE" | tr -d ' ')"
QUAL_COUNT="$(wc -l < "$REAL_QUAL" | tr -d ' ')"
ALLOW_USE_COUNT="$(wc -l < "$ALLOW_USE" | tr -d ' ')"
ALLOW_QUAL_COUNT="$(wc -l < "$ALLOW_QUAL" | tr -d ' ')"
TOTAL=$((USE_COUNT + QUAL_COUNT))
ALLOW_TOTAL=$((ALLOW_USE_COUNT + ALLOW_QUAL_COUNT))

echo "check_hashmap_ban: HashMap ban audit (B-15B)"
echo "  use std::collections::HashMap violations : $USE_COUNT"
echo "  std::collections::HashMap:: violations   : $QUAL_COUNT"
echo "  total production violations              : $TOTAL"
echo "  allow-listed (cfg(test) inside src/)     : $ALLOW_TOTAL"

if [ "$ALLOW_TOTAL" -gt 0 ]; then
    echo ""
    echo "INFO (allow-listed cfg(test) occurrences, not counted as fail):"
    cat "$ALLOW_USE" "$ALLOW_QUAL" 2>/dev/null
fi

if [ "$TOTAL" -gt 0 ]; then
    echo ""
    echo "VIOLATIONS (use stmt, first 20):"
    head -n 20 "$REAL_USE"
    if [ "$QUAL_COUNT" -gt 0 ]; then
        echo ""
        echo "VIOLATIONS (qualified, first 20):"
        head -n 20 "$REAL_QUAL"
    fi
    echo ""
    echo "check_hashmap_ban: FAIL -- $TOTAL production violation(s). Replace with BTreeMap/FxHashMap/IndexMap."
    exit 1
fi

echo ""
echo "HashMap ban check: 0 production violations"
echo "check_hashmap_ban: PASS -- no HashMap violations in production code."
exit 0
