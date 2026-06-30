#!/usr/bin/env bash
# B-15A Task 9 — Entropy quarantine leak detector (tightened in B-15C).
#
# EXPECTATIONS:
#   - exit 0 when no entropy/clock leak calls are found anywhere under crates/
#     except the canonical modules and any tests/ subdirectory.
#   - exit 1 when leaks are found; prints up to 10 leak lines for triage.
#   - --help / -h prints usage and exits 0.
#   - portable: runs on Linux, macOS, and Git-Bash on Windows.
#   - prefers ripgrep (`rg`) when available, otherwise falls back to `grep -RIn`.
#   - the script itself is not part of the build, so its own use of `date`
#     etc. is unrestricted (see CLAUDE.md anti-stale-training note).
#
# B-15C tightening:
#   - Added patterns: tokio::time::Instant::now, chrono::Utc::now,
#     chrono::Local::now, uuid::Uuid::new_v4, Uuid::new_v4(),
#     rand::thread_rng, OsRng::new, SeedableRng::from_entropy
#   - Added allowlists: entropy.rs (OsEntropy uses rand::rng legitimately),
#     examples/, benches/, pipeline_builder_tests.rs, pipeline_builder_bench.rs
#   - Added TEST_KEEP filter: lines annotated with TEST_KEEP are excluded
#     (cfg(test) timing uses inside src/ files that are intentionally kept)
#
# C-15 tightening:
#   - Added `rand::random` to PATTERN (RNG entry point not previously covered).
#   - Added `entropy-allow:` filter (skip lines annotated with a same-line
#     trailing `// entropy-allow: <reason>` comment after the migration could
#     not be done — e.g., cache TTL Instant comparisons inside #[derive(Deserialize)]
#     configs that can't carry an Arc<dyn ClockProvider>). The manifest
#     convention (top-of-file comment listing all annotated sites) is
#     documented but not script-enforced (see C-15 D9).
#
# C-15 closure (s-25120):
#   - Excluded `**/measure.rs` (benchmark/measurement-intrinsic timing harness;
#     measure.rs IS the timing measurement, not a producer leak — see
#     C-15-decisions.md Option A rationale).
#   - Excluded `**/bench_*.rs` (cli/bench_arc.rs, cli/bench_conformal.rs and
#     any future bench_-prefixed file are benchmark code; the script already
#     excludes `/benches/` subdirs but bench_-prefixed files live in /src/).

set -e

usage() {
    cat <<'EOF'
Usage: check_entropy_quarantine.sh [--help|-h]

Greps for direct entropy/clock calls in .elai_cc/crates/,
excluding canonical modules, tests/ subdirectories, examples/, benches/,
and lines annotated with TEST_KEEP.

Patterns checked:
  SystemTime::now, Instant::now, tokio::time::Instant::now,
  chrono::Utc::now, chrono::Local::now,
  uuid::Uuid::new_v4, Uuid::new_v4(),
  rand::thread_rng, rand::random,
  OsRng::new, SeedableRng::from_entropy

Lines annotated with `// entropy-allow: <reason>` are excluded (C-15).

Exits 0 when clean, 1 when any leak site is found.
EOF
}

case "${1-}" in
    --help|-h) usage; exit 0 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATES_DIR="${SCRIPT_DIR}/../crates"

if [ ! -d "$CRATES_DIR" ]; then
    echo "check_entropy_quarantine: crates dir not found at $CRATES_DIR" >&2
    exit 2
fi

# C-15 extended pattern — covers time AND entropy/UUID violation sites.
# Added in C-15: `rand::random\b` (RNG entry point not previously covered).
PATTERN='SystemTime::now|Instant::now|tokio::time::Instant::now|chrono::Utc::now|chrono::Local::now|uuid::Uuid::new_v4|Uuid::new_v4\(\)|rand::thread_rng|rand::random\b|OsRng::new\b|SeedableRng::from_entropy'

HITS_FILE="$(mktemp -t entropy_quarantine.XXXXXX 2>/dev/null || mktemp)"
trap 'rm -f "$HITS_FILE"' EXIT

# Skip lines whose actual code content (after `file:line:` prefix and leading
# whitespace) starts with `//` — these are comment / doc-comment lines, not
# real entropy leaks. B-15 finish: false-positive elimination.
strip_comments() {
    awk '{
        pos = match($0, /:[0-9]+:/);
        if (pos > 0) {
            content = substr($0, pos + RLENGTH);
            sub(/^[ \t]+/, "", content);
            if (content ~ /^\/\//) next;
        }
        print
    }'
}

if command -v rg >/dev/null 2>&1; then
    rg --no-heading --line-number --type rust \
        --glob '!**/clock.rs' \
        --glob '!**/entropy.rs' \
        --glob '!**/tests/**' \
        --glob '!**/test/**' \
        --glob '!**/examples/**' \
        --glob '!**/benches/**' \
        --glob '!**/.bkp_old/**' \
        --glob '!**/pipeline_builder_tests.rs' \
        --glob '!**/pipeline_builder_bench.rs' \
        --glob '!**/measure.rs' \
        --glob '!**/bench_*.rs' \
        "$PATTERN" "$CRATES_DIR" \
        | grep -v 'TEST_KEEP' \
        | strip_comments \
        | grep -v 'entropy-allow:' \
        > "$HITS_FILE" || true
else
    grep -RIn -E "$PATTERN" "$CRATES_DIR" \
        --include='*.rs' \
        --exclude='clock.rs' \
        --exclude='entropy.rs' \
        --exclude='pipeline_builder_tests.rs' \
        --exclude='pipeline_builder_bench.rs' \
        --exclude='measure.rs' \
        --exclude='bench_*.rs' \
        --exclude-dir='tests' \
        --exclude-dir='test' \
        --exclude-dir='examples' \
        --exclude-dir='benches' \
        --exclude-dir='.bkp_old' \
        | grep -v 'TEST_KEEP' \
        | strip_comments \
        | grep -v 'entropy-allow:' \
        > "$HITS_FILE" || true
fi

LEAK_COUNT="$(wc -l < "$HITS_FILE" | tr -d ' ')"

if [ "$LEAK_COUNT" -eq 0 ]; then
    echo "check_entropy_quarantine: clean — 0 leaks."
    exit 0
fi

echo "check_entropy_quarantine: FOUND $LEAK_COUNT leak site(s). First 10:"
head -n 10 "$HITS_FILE"
exit 1
