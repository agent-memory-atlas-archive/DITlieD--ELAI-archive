#!/usr/bin/env bash
# B-16A Task 14 — Anti-stale pattern lint (REPORT-ONLY for the gradual
# escalation modes; FAIL on the strict patterns when found in production code).
# B-16E update: cast scan split into 4 named categories; HIGH-risk files
# excluded from cast counts (turboquant.rs, sandbox.rs, context_budget.rs).
#
# EXPECTATIONS:
#   - exit 0 when no STRICT pattern violations are found in production code.
#   - exit 1 when any STRICT pattern is found outside excluded paths.
#   - REPORT-ONLY patterns (.unwrap(), narrowing as-casts, use log::) print
#     counts but do not affect exit code in B-16A. These tighten in B-16B/C/D.
#   - --help / -h prints usage and exits 0.
#   - portable: runs on Linux, macOS, and Git-Bash on Windows.
#   - prefers ripgrep (`rg`) when available, otherwise falls back to
#     `grep -RIn`.
#
# Pattern modes (B-16A):
#   STRICT  : 'Box<dyn .*Error'     -> banned in production code
#   STRICT  : 'use log::'           -> banned in production code (use tracing)
#   STRICT  : ort = "...-rc..."     -> banned UNLESS waived in B-16A-decisions.md
#   REPORT  : '\.unwrap\(\)'        -> count only (B-16B will tighten)
#   REPORT  : narrowing casts       -> split by category (B-16E); B-16D will tighten
#     CATEGORY A: int-narrow        (i8/i16/i32/u8/u16/u32 targets, excluding pointer lines)
#     CATEGORY B: f64/f32->usize    (intentional truncation; no TryFrom in std; REPORT-ONLY)
#     CATEGORY C: u128->u64         (as_millis pattern; must be pre-clamped)
#     CATEGORY D: u64->i64          (as_secs pattern; must be pre-clamped)
#
# Excluded from cast scans (HIGH-risk / intentional patterns deferred by B-16E):
#   turboquant.rs  — f64->f32 throughout quantization math; no TryFrom<f64> for f32 in std
#   sandbox.rs     — HRESULT i32->u32 bit-reinterpret for hex display; Win32 HANDLE pointer casts
#   context_budget.rs — ContextSlot/AdaptiveLevel enum-as-usize indexing (compile-time safe)
#
# Excludes for STRICT scans: tests/**, test/**, examples/**, target/**,
# main.rs (entrypoint), build.rs.
#
# Status of this baseline (2026-04-26): expected to be CLEAN for STRICT
# patterns in the elai-knowledge crate after B-16A; other crates may still
# show INFO counts. STRICT scope expands in B-16B+.
# B-16E (2026-04-29): cast counts split by category; HIGH-risk files excluded.

set -e

usage() {
    cat <<'EOF'
Usage: check_anti_stale.sh [--help|-h]

Greps for outdated/forbidden patterns under .elai_cc/crates/.

STRICT (exit 1 on hit):
  - Box<dyn .*Error            (use thiserror typed enums)
  - use log::                  (use tracing instead)
  - ort = "...-rc..."          (use ort 2.0.0 stable when published)

REPORT-ONLY (exit unaffected, count printed):
  - .unwrap()                  (B-16B will tighten)
  - as i*/u*/f*                (narrowing casts; B-16D will tighten)

Excludes: */tests/**, */test/**, */examples/**, target/**, main.rs, build.rs.
EOF
}

case "${1-}" in
    --help|-h) usage; exit 0 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATES_DIR="${SCRIPT_DIR}/../crates"

if [ ! -d "$CRATES_DIR" ]; then
    echo "check_anti_stale: crates dir not found at $CRATES_DIR" >&2
    exit 2
fi

TMP_BOX="$(mktemp -t anti_stale_box.XXXXXX 2>/dev/null || mktemp)"
TMP_LOG="$(mktemp -t anti_stale_log.XXXXXX 2>/dev/null || mktemp)"
TMP_ORT="$(mktemp -t anti_stale_ort.XXXXXX 2>/dev/null || mktemp)"
TMP_UNWRAP="$(mktemp -t anti_stale_unwrap.XXXXXX 2>/dev/null || mktemp)"
TMP_CAST="$(mktemp -t anti_stale_cast.XXXXXX 2>/dev/null || mktemp)"
# B-16E: category split temps
TMP_CAST_INT="$(mktemp -t anti_stale_cast_int.XXXXXX 2>/dev/null || mktemp)"
TMP_CAST_FTOUSZ="$(mktemp -t anti_stale_cast_fusz.XXXXXX 2>/dev/null || mktemp)"
TMP_CAST_MILLIS="$(mktemp -t anti_stale_cast_millis.XXXXXX 2>/dev/null || mktemp)"
TMP_CAST_MILLIS_UNCLAMPED="$(mktemp -t anti_stale_cast_millis_u.XXXXXX 2>/dev/null || mktemp)"
TMP_CAST_SECS="$(mktemp -t anti_stale_cast_secs.XXXXXX 2>/dev/null || mktemp)"
trap 'rm -f "$TMP_BOX" "$TMP_LOG" "$TMP_ORT" "$TMP_UNWRAP" "$TMP_CAST" \
          "$TMP_CAST_INT" "$TMP_CAST_FTOUSZ" "$TMP_CAST_MILLIS" \
          "$TMP_CAST_MILLIS_UNCLAMPED" "$TMP_CAST_SECS"' EXIT

scan() {
    local pattern="$1"
    local out="$2"
    shift 2
    if command -v rg >/dev/null 2>&1; then
        rg --no-heading --line-number --type rust \
            --glob '!**/tests/**' \
            --glob '!**/test/**' \
            --glob '!**/examples/**' \
            --glob '!**/target/**' \
            --glob '!**/main.rs' \
            --glob '!**/build.rs' \
            "$@" \
            "$pattern" "$CRATES_DIR" > "$out" 2>/dev/null || true
    else
        grep -RIn -E "$pattern" "$CRATES_DIR" \
            --include='*.rs' \
            --exclude-dir='tests' \
            --exclude-dir='test' \
            --exclude-dir='examples' \
            --exclude-dir='target' \
            --exclude='main.rs' \
            --exclude='build.rs' > "$out" 2>/dev/null || true
    fi
}

# scan_cast: like scan() but with HIGH-risk file exclusions (turboquant.rs,
# sandbox.rs, context_budget.rs deferred by B-16E).
scan_cast() {
    local pattern="$1"
    local out="$2"
    shift 2
    if command -v rg >/dev/null 2>&1; then
        rg --no-heading --line-number --type rust \
            --glob '!**/tests/**' \
            --glob '!**/test/**' \
            --glob '!**/examples/**' \
            --glob '!**/target/**' \
            --glob '!**/main.rs' \
            --glob '!**/build.rs' \
            --glob '!**/turboquant.rs' \
            --glob '!**/sandbox.rs' \
            --glob '!**/context_budget.rs' \
            "$pattern" "$CRATES_DIR" > "$out" 2>/dev/null || true
    else
        grep -RIn -E "$pattern" "$CRATES_DIR" \
            --include='*.rs' \
            --exclude-dir='tests' \
            --exclude-dir='test' \
            --exclude-dir='examples' \
            --exclude-dir='target' \
            --exclude='main.rs' \
            --exclude='build.rs' \
            --exclude='turboquant.rs' \
            --exclude='sandbox.rs' \
            --exclude='context_budget.rs' > "$out" 2>/dev/null || true
    fi
}

# STRICT 1: Box<dyn ...Error> where Error IS the trait (not a type param of
# Fn/Future). Only alpha/underscore/colon allowed between `dyn` and `Error`,
# anchored by space, `+`, or `>` so `dyn Future<Output = Result<_, FooError>>`
# and `dyn FnMut(...) -> Result<_, BarError>` are NOT matched.
#
# Additional excludes:
#   - rules/src/schema.rs is the DEPRECATED_PATTERN fixture registry; it
#     LISTS the banned strings as data. Same role as spec fixture lines.
#   - Lines starting with /// or //! are doc comments — text only.
scan 'Box<dyn [A-Za-z_:]*Error[ +>]' "$TMP_BOX"
# Filter out doc comments and the rules schema fixture file.
if [ -s "$TMP_BOX" ]; then
    FILTERED="$(mktemp -t anti_stale_box_f.XXXXXX 2>/dev/null || mktemp)"
    grep -v -E '^[^:]*rules/src/schema\.rs:|:\s*///|:\s*//!' "$TMP_BOX" > "$FILTERED" || true
    mv "$FILTERED" "$TMP_BOX"
fi
BOX_COUNT="$(wc -l < "$TMP_BOX" | tr -d ' ')"

# STRICT 2: use log::
scan '^use log::|[^a-zA-Z_]use log::' "$TMP_LOG"
LOG_COUNT="$(wc -l < "$TMP_LOG" | tr -d ' ')"

# STRICT 3: ort -rc in any Cargo.toml under crates/.
# This pattern lives in Cargo.toml, not .rs files — scan separately.
ORT_COUNT=0
if command -v rg >/dev/null 2>&1; then
    rg --no-heading --line-number \
        --glob '**/Cargo.toml' \
        --glob '!**/target/**' \
        '^ort\s*=\s*"[^"]*-rc' \
        "$CRATES_DIR" > "$TMP_ORT" 2>/dev/null || true
else
    grep -RIn -E '^ort\s*=\s*"[^"]*-rc' "$CRATES_DIR" \
        --include='Cargo.toml' \
        --exclude-dir='target' > "$TMP_ORT" 2>/dev/null || true
fi
ORT_COUNT="$(wc -l < "$TMP_ORT" | tr -d ' ')"

# REPORT 1: .unwrap()
scan '\.unwrap\(\)' "$TMP_UNWRAP"
UNWRAP_COUNT="$(wc -l < "$TMP_UNWRAP" | tr -d ' ')"

# REPORT 2: narrowing as-casts — B-16E category split.
# HIGH-risk files (turboquant.rs, sandbox.rs, context_budget.rs) excluded via scan_cast().
#
# CATEGORY A: int-narrow (i8/i16/i32/u8/u16/u32 targets).
# Heuristic — also catches widening (u8->u32 etc.); type-aware exclusion not feasible in bash.
# Excludes lines matching pointer-cast indicators.
scan_cast ' as (i8|i16|i32|u8|u16|u32)([^a-zA-Z0-9_]|$)' "$TMP_CAST_INT"
if [ -s "$TMP_CAST_INT" ]; then
    FILTERED_INT="$(mktemp -t anti_stale_cast_int_f.XXXXXX 2>/dev/null || mktemp)"
    grep -v -E '\*const|\*mut|as \*|HRESULT|FreeSid|AF_INET|AF_UNIX|EPERM' \
        "$TMP_CAST_INT" > "$FILTERED_INT" || true
    mv "$FILTERED_INT" "$TMP_CAST_INT"
fi
CAST_INT_COUNT="$(wc -l < "$TMP_CAST_INT" | tr -d ' ')"

# CATEGORY B: f64/f32->usize (intentional truncation; no TryFrom<f64> for usize; REPORT-ONLY).
# Also catches widening (usize->f64); cannot distinguish without type info.
scan_cast ' as (f32|f64|usize)([^a-zA-Z0-9_]|$)' "$TMP_CAST_FTOUSZ"
CAST_FTOUSZ_COUNT="$(wc -l < "$TMP_CAST_FTOUSZ" | tr -d ' ')"

# CATEGORY C: u128->u64 (as_millis pattern; must be pre-clamped with .min(u128::from(u64::MAX))).
scan_cast '\.as_millis\(\) as u64' "$TMP_CAST_MILLIS"
CAST_MILLIS_COUNT="$(wc -l < "$TMP_CAST_MILLIS" | tr -d ' ')"
# Detect unclamped: lines without a preceding .min( on same line.
grep -v '\.min(' "$TMP_CAST_MILLIS" > "$TMP_CAST_MILLIS_UNCLAMPED" || true
CAST_MILLIS_UNCLAMPED_COUNT="$(wc -l < "$TMP_CAST_MILLIS_UNCLAMPED" | tr -d ' ')"

# CATEGORY D: u64->i64 (as_secs pattern; must be clamped or use try_from).
scan_cast '\.as_secs\(\) as i64' "$TMP_CAST_SECS"
CAST_SECS_COUNT="$(wc -l < "$TMP_CAST_SECS" | tr -d ' ')"

# Legacy combined count (kept for backward compat; reflects totals excl. HIGH-risk files).
scan_cast ' as (i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64)([^a-zA-Z0-9_]|$)' "$TMP_CAST"
CAST_COUNT="$(wc -l < "$TMP_CAST" | tr -d ' ')"

# Output report
echo "check_anti_stale: anti-stale pattern audit (B-16A / B-16E categories)"
echo "  STRICT  Box<dyn .*Error : $BOX_COUNT"
echo "  STRICT  use log::       : $LOG_COUNT"
echo "  STRICT  ort = \"...-rc...\": $ORT_COUNT  (waiver: workflow/reasoning/B-16A-decisions.md DECISION-A; ORT-PENDING: bump to 2.0.0 when crates.io ships stable)"
echo "  REPORT  .unwrap()       : $UNWRAP_COUNT (B-16B Chunks A+B+D shipped; Chunk C closed via LazyLock-static exemption per B-16B D5; raw count includes test/bench/static-exempt sites)"
echo "  REPORT  as-cast total   : $CAST_COUNT (excl. turboquant.rs/sandbox.rs/context_budget.rs)"
echo "    CAT-A int-narrow      : $CAST_INT_COUNT (i8/i16/i32/u8/u16/u32; excl. pointer lines)"
echo "    CAT-B f64/f32/usize   : $CAST_FTOUSZ_COUNT (incl. widening; no TryFrom in std)"
echo "    CAT-C u128->u64 millis: $CAST_MILLIS_COUNT total; $CAST_MILLIS_UNCLAMPED_COUNT unclamped (must add .min(u128::from(u64::MAX)))"
echo "    CAT-D u64->i64 secs   : $CAST_SECS_COUNT (as_secs() as i64; must use try_from or clamp)"
echo "  REPORT  magic-numbers   : enforced by T2 AST lint, NOT this grep script"
echo "    surface: crates/plugins/src/tiered_analysis.rs:1458 check_magic_numbers_rust"
echo "    surface: crates/plugins/src/tiered_analysis.rs:1506 check_magic_numbers_python"
echo "    walker : tiered_analysis.rs:1471 walk_magic_numbers_rust / :1519 walk_magic_numbers_python"
echo "    allow-list: 0 | 1 | 2 | -1 | 0.0 | 1.0 ; ctx-exempt: const/static decls + test files"
echo "    rationale: tree-sitter AST exempts const/static blocks deterministically; bash grep cannot"

# ort -rc is waived in B-16A: any value here is INFO, not failure.
# ort stable is "planned but not yet published" on crates.io as of 2026-04-26.
ORT_FAIL=0
WAIVER="${SCRIPT_DIR}/../../workflow/reasoning/B-16A-decisions.md"
if [ "$ORT_COUNT" -gt 0 ] && [ ! -f "$WAIVER" ]; then
    ORT_FAIL=1
fi

STRICT_FAIL=0
if [ "$BOX_COUNT" -gt 0 ]; then
    STRICT_FAIL=1
    echo ""
    echo "STRICT: Box<dyn Error> hits (first 10):"
    head -n 10 "$TMP_BOX"
fi
if [ "$LOG_COUNT" -gt 0 ]; then
    STRICT_FAIL=1
    echo ""
    echo "STRICT: 'use log::' hits (first 10):"
    head -n 10 "$TMP_LOG"
fi
if [ "$ORT_FAIL" -eq 1 ]; then
    STRICT_FAIL=1
    echo ""
    echo "STRICT: ort -rc pin without waiver doc (first 10):"
    head -n 10 "$TMP_ORT"
fi

if [ "$STRICT_FAIL" -eq 0 ]; then
    echo ""
    echo "check_anti_stale: STRICT clean. REPORT counts above are informational."
    exit 0
fi

echo ""
echo "check_anti_stale: STRICT violations found. Fix or document waiver."
exit 1
