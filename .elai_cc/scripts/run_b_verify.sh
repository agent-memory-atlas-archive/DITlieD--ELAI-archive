#!/usr/bin/env bash
# B-VERIFY aggregator — runs all 18 probe scripts and writes B-VERIFY-report.md.
# Usage: bash run_b_verify.sh [--skip-sidecar] [--skip-elai-run] [--skip-build]
# Outputs:
#   workflow/sessions/s-27276/B-VERIFY-report.md
#   workflow/sessions/s-27276/b_verify_run.log
# Exit: 0 if all 18 PASS, 1 if any FAIL (non-zero = rework needed)
set -uo pipefail

# ── Paths ───────────────────────────────────────────────────────────────────
ELAI_ROOT="F:/.ELAI_workflow/.elai_cc"
WORKFLOW_ROOT="F:/.ELAI_workflow"
SESSION_DIR="${WORKFLOW_ROOT}/workflow/sessions/s-27276"
SCRIPTS_DIR="${ELAI_ROOT}/scripts"
CORPUS="${WORKFLOW_ROOT}/docs/.source/nextest-main"
SIDECAR_LOG="${SESSION_DIR}/sidecar.log"
ELAI_LOG="${SESSION_DIR}/elai_run.log"
RUN_LOG="${SESSION_DIR}/b_verify_run.log"
REPORT="${SESSION_DIR}/B-VERIFY-report.md"
ELAI_BIN="${ELAI_ROOT}/target/release/elai.exe"
SIDECAR_MODEL="${ELAI_ROOT}/models/Qwen2.5-Coder-7B-Instruct-abliterated"
SIDECAR_MODEL_3B="${ELAI_ROOT}/models/Qwen2.5-Coder-3B-Instruct-abliterated"
ONNX_DIR="${ELAI_ROOT}/models/onnx"

# ── Flags ────────────────────────────────────────────────────────────────────
SKIP_SIDECAR=0
SKIP_ELAI_RUN=0
SKIP_BUILD=0
for ARG in "$@"; do
    case "$ARG" in
        --skip-sidecar)   SKIP_SIDECAR=1 ;;
        --skip-elai-run)  SKIP_ELAI_RUN=1 ;;
        --skip-build)     SKIP_BUILD=1 ;;
    esac
done

# ── Helpers ───────────────────────────────────────────────────────────────────
log() { echo "[$(date '+%H:%M:%S')] $*" | tee -a "$RUN_LOG"; }
ts_now() { date '+%Y-%m-%dT%H:%M:%S'; }

mkdir -p "$SESSION_DIR"
: > "$RUN_LOG"

STARTED_AT=$(ts_now)
log "B-VERIFY aggregator started at $STARTED_AT"

# ── Step 0: Sanity gates ──────────────────────────────────────────────────────
log "=== Step 0: Sanity gates ==="

SANITY_OK=1
for F in \
    "${ONNX_DIR}/graphcodebert.onnx" \
    "${ONNX_DIR}/graphcodebert-tokenizer.json" \
    "${ONNX_DIR}/granite-embed.onnx" \
    "${ONNX_DIR}/granite-embed-tokenizer.json" \
    "${ONNX_DIR}/cross-encoder-l12.onnx" \
    "${ONNX_DIR}/tokenizer.json" \
    "${ELAI_ROOT}/models/splade/model.onnx" \
    "${ELAI_ROOT}/models/splade/tokenizer.json" \
    "${ELAI_ROOT}/models/Qwen2.5-Coder-7B-Instruct-abliterated/config.json"; do
    if [ -f "$F" ]; then
        log "  OK: $F"
    else
        log "  MISSING: $F"
        SANITY_OK=0
    fi
done

if [ "$SANITY_OK" -eq 0 ]; then
    log "FATAL: sanity gate failed -- required model files missing. Aborting."
    exit 1
fi
log "Sanity gates: PASS"

# ── Step 1: Cargo build --release ─────────────────────────────────────────────
BUILD_STATUS="SKIP"
BUILD_DURATION=0
if [ "$SKIP_BUILD" -eq 0 ]; then
    log "=== Step 1: cargo build --release -p elai-cli ==="
    BUILD_START=$(date +%s)
    cd "$ELAI_ROOT"
    if cargo build --release -p elai-cli >> "$RUN_LOG" 2>&1; then
        BUILD_END=$(date +%s)
        BUILD_DURATION=$((BUILD_END - BUILD_START))
        BUILD_STATUS="PASS"
        log "cargo build: PASS (${BUILD_DURATION}s)"
    else
        BUILD_END=$(date +%s)
        BUILD_DURATION=$((BUILD_END - BUILD_START))
        BUILD_STATUS="FAIL"
        log "cargo build: FAIL -- aborting (binary may be stale)"
        # Don't abort if binary already exists (previous build)
        if [ ! -f "$ELAI_BIN" ]; then
            log "FATAL: elai.exe not found and build failed."
            exit 1
        fi
        log "WARN: using existing stale binary at $ELAI_BIN"
    fi
else
    log "Step 1: SKIPPED (--skip-build)"
    [ -f "$ELAI_BIN" ] && BUILD_STATUS="SKIP(binary_exists)" || BUILD_STATUS="SKIP(binary_MISSING)"
fi

# ── Step 2: Symlink ONNX models under corpus root ─────────────────────────────
# elai.exe resolves models via <CWD>/.elai_cc/models/onnx/
# Corpus CWD = CORPUS; we need CORPUS/.elai_cc/models -> ELAI_ROOT/models
CORPUS_MODELS="${CORPUS}/.elai_cc/models"
if [ ! -d "$CORPUS_MODELS" ]; then
    log "Creating ONNX model symlink: $CORPUS_MODELS -> ${ELAI_ROOT}/models"
    mkdir -p "${CORPUS}/.elai_cc"
    # On Windows (Git Bash), ln -s may not work; use junction/mklink via cmd
    if ln -s "${ELAI_ROOT}/models" "$CORPUS_MODELS" 2>/dev/null; then
        log "Symlink created"
    else
        # Fallback: create a junction via PowerShell
        powershell.exe -NoProfile -Command "New-Item -ItemType Junction -Path '${CORPUS}/.elai_cc/models' -Target '${ELAI_ROOT}/models'" >> "$RUN_LOG" 2>&1 || true
        if [ -d "$CORPUS_MODELS" ]; then
            log "Junction created via PowerShell"
        else
            log "WARN: could not create models link; elai.exe will degrade to BM25-only"
        fi
    fi
else
    log "ONNX model link/dir already present at $CORPUS_MODELS"
fi

# ── Step 3: Set ELAI_SSB_PATH ─────────────────────────────────────────────────
mkdir -p "${CORPUS}/.elai/sessions"
export ELAI_SSB_PATH="${CORPUS}/.elai/sessions/ssb.db"
log "ELAI_SSB_PATH=$ELAI_SSB_PATH"

# ── Step 4: Start sidecar ──────────────────────────────────────────────────────
SIDECAR_PID=""
SIDECAR_STATUS="SKIP"
SIDECAR_DURATION=0

if [ "$SKIP_SIDECAR" -eq 0 ]; then
    log "=== Step 3: Start sidecar ==="
    cd "$ELAI_ROOT"

    # Try 7B first; fall back to 3B on OOM
    MODEL_PATH="$SIDECAR_MODEL"
    if [ ! -d "$MODEL_PATH" ] && [ -d "$SIDECAR_MODEL_3B" ]; then
        MODEL_PATH="$SIDECAR_MODEL_3B"
        log "WARN: 7B model not found; using 3B fallback: $MODEL_PATH"
    fi

    python -m elai_inference.server \
        --model "$MODEL_PATH" \
        --port 8100 \
        --no-dola \
        --turboquant-bits 8 \
        --no-steering \
        --load-in-4bit \
        --log-level INFO \
        > "$SIDECAR_LOG" 2>&1 &
    SIDECAR_PID=$!
    log "Sidecar PID=$SIDECAR_PID; waiting for 'Application startup complete.' (up to 90s)"

    SIDECAR_START=$(date +%s)
    READY=0
    for i in $(seq 1 90); do
        sleep 1
        if grep -q "Application startup complete\." "$SIDECAR_LOG" 2>/dev/null; then
            READY=1
            break
        fi
        if ! kill -0 "$SIDECAR_PID" 2>/dev/null; then
            log "WARN: sidecar process died before ready (exit)"
            break
        fi
    done

    SIDECAR_END=$(date +%s)
    SIDECAR_DURATION=$((SIDECAR_END - SIDECAR_START))

    if [ "$READY" -eq 1 ]; then
        SIDECAR_STATUS="PASS"
        log "Sidecar ready in ${SIDECAR_DURATION}s"
    else
        SIDECAR_STATUS="FAIL"
        log "WARN: sidecar did not signal ready in ${SIDECAR_DURATION}s (may still be loading)"
        log "Sidecar log tail:"
        tail -10 "$SIDECAR_LOG" 2>/dev/null | while IFS= read -r line; do log "  $line"; done
    fi
else
    log "Step 3: SKIPPED (--skip-sidecar)"
fi

# ── Step 5: Run elai.exe one-shot indexing pass ───────────────────────────────
ELAI_EXIT="-"
ELAI_DURATION=0
SSB_ROWS=0
TRACE_LINES=0

if [ "$SKIP_ELAI_RUN" -eq 0 ]; then
    log "=== Step 4: elai.exe indexing pass on corpus ==="
    cd "$CORPUS"
    ELAI_START=$(date +%s)
    RUST_LOG="elai_knowledge=debug,elai_orchestrator=debug" \
        "$ELAI_BIN" -p "Search this codebase for the test runner entry point and summarize how nextest discovers and groups tests." \
        --output-format text \
        > "$ELAI_LOG" 2>&1
    ELAI_EXIT=$?
    ELAI_END=$(date +%s)
    ELAI_DURATION=$((ELAI_END - ELAI_START))
    log "elai.exe exit=$ELAI_EXIT duration=${ELAI_DURATION}s"

    # Count SSB rows
    if [ -f "$ELAI_SSB_PATH" ]; then
        SSB_ROWS=$(sqlite3 "$ELAI_SSB_PATH" "SELECT COUNT(*) FROM ssb;" 2>/dev/null) || SSB_ROWS=0
        log "SSB rows: $SSB_ROWS"
        # Archive for probes
        cp "$ELAI_SSB_PATH" "${SESSION_DIR}/b-verify-ssb.db" 2>/dev/null || true
    else
        log "WARN: SSB db not found at $ELAI_SSB_PATH"
    fi

    # Count tracing log lines
    if [ -f "$ELAI_LOG" ]; then
        TRACE_LINES=$(wc -l < "$ELAI_LOG") || TRACE_LINES=0
        log "Tracing log lines: $TRACE_LINES"
    fi
else
    log "Step 4: SKIPPED (--skip-elai-run)"
fi

# ── Step 6: Run all 18 probes ─────────────────────────────────────────────────
log "=== Step 5: Running 18 probes ==="

declare -A PROBE_RESULTS
declare -A PROBE_DURATIONS

run_probe() {
    local ID="$1"
    local SCRIPT="${SCRIPTS_DIR}/verify_stage_${ID}.sh"
    local ARGS="${2:-}"

    if [ ! -f "$SCRIPT" ]; then
        PROBE_RESULTS["$ID"]="FAIL|script_not_found:$SCRIPT"
        PROBE_DURATIONS["$ID"]=0
        log "  [$ID] FAIL (script not found)"
        return
    fi

    local P_START=$(date +%s)
    local OUTPUT
    OUTPUT=$(timeout 300 bash "$SCRIPT" $ARGS 2>/dev/null) || true
    local P_END=$(date +%s)
    local DURATION=$((P_END - P_START))

    # Extract the last PASS|... or FAIL|... line
    local RESULT
    RESULT=$(echo "$OUTPUT" | grep -E '^(PASS|FAIL)\|' | tail -1)
    if [ -z "$RESULT" ]; then
        RESULT="FAIL|no_pass_fail_output:$(echo "$OUTPUT" | tail -1 | cut -c1-60)"
    fi

    PROBE_RESULTS["$ID"]="$RESULT"
    PROBE_DURATIONS["$ID"]=$DURATION
    log "  [$ID] ${RESULT} (${DURATION}s)"
}

# Run probes in order with appropriate args
run_probe "1A" "$CORPUS"
run_probe "1B" "$CORPUS"
run_probe "1C"
run_probe "1D" "$CORPUS"
run_probe "1E_catrag"
run_probe "1E_arag"
run_probe "1G"
run_probe "1I"
run_probe "ORCH_2A_2D"
run_probe "B8_evidence_gates"
run_probe "B8_csq_guard"
run_probe "B9_sessiondag"
run_probe "B10_context_budget"
run_probe "B19A_stop_hook"
run_probe "B19A_ruff"
run_probe "B19A_latency"
run_probe "B20A_taxonomy"
run_probe "B20A_similarity"

# ── Step 7: Count results ──────────────────────────────────────────────────────
PASS_COUNT=0
FAIL_COUNT=0
FAIL_LIST=""

for ID in 1A 1B 1C 1D 1E_catrag 1E_arag 1G 1I ORCH_2A_2D B8_evidence_gates B8_csq_guard B9_sessiondag B10_context_budget B19A_stop_hook B19A_ruff B19A_latency B20A_taxonomy B20A_similarity; do
    R="${PROBE_RESULTS[$ID]:-FAIL|not_run}"
    if [[ "$R" == PASS* ]]; then
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        FAIL_LIST="${FAIL_LIST} $ID"
    fi
done

FINISHED_AT=$(ts_now)
log "=== Aggregation complete: $PASS_COUNT/18 PASS, $FAIL_COUNT FAIL ==="

# ── Step 8: Stop sidecar ──────────────────────────────────────────────────────
if [ -n "$SIDECAR_PID" ] && kill -0 "$SIDECAR_PID" 2>/dev/null; then
    log "Stopping sidecar PID=$SIDECAR_PID"
    kill "$SIDECAR_PID" 2>/dev/null || true
    sleep 2
    kill -9 "$SIDECAR_PID" 2>/dev/null || true
fi

# ── Step 9: Write B-VERIFY-report.md ─────────────────────────────────────────
cat > "$REPORT" << REPORT_EOF
B-VERIFY REPORT — session s-27276
===================================
STARTED:  $STARTED_AT
FINISHED: $FINISHED_AT

SANITY GATES
------------
ONNX models: ${ONNX_DIR}/graphcodebert.onnx         OK
ONNX models: ${ONNX_DIR}/granite-embed.onnx          OK
ONNX models: ${ONNX_DIR}/cross-encoder-l12.onnx      OK
ONNX models: ${ONNX_DIR}/tokenizer.json              OK
SPLADE:      ${ELAI_ROOT}/models/splade/model.onnx   OK
7B model:    ${ELAI_ROOT}/models/Qwen2.5-Coder-7B-Instruct-abliterated/  OK

SIDECAR
-------
PID:    ${SIDECAR_PID:-n/a}
Status: $SIDECAR_STATUS (${SIDECAR_DURATION}s)
Log:    $SIDECAR_LOG

ELAI RUN
--------
Command: elai.exe -p "Search this codebase for the test runner entry point..."
Corpus:  $CORPUS
Exit:    $ELAI_EXIT
Duration: ${ELAI_DURATION}s
SSB rows: $SSB_ROWS
Trace log lines: $TRACE_LINES
SSB archive: ${SESSION_DIR}/b-verify-ssb.db

PROBE RESULTS (18 checks)
--------------------------
ID                    | Result                                        | Duration
--------------------- | --------------------------------------------- | --------
REPORT_EOF

for ID in 1A 1B 1C 1D 1E_catrag 1E_arag 1G 1I ORCH_2A_2D B8_evidence_gates B8_csq_guard B9_sessiondag B10_context_budget B19A_stop_hook B19A_ruff B19A_latency B20A_taxonomy B20A_similarity; do
    R="${PROBE_RESULTS[$ID]:-FAIL|not_run}"
    DUR="${PROBE_DURATIONS[$ID]:-0}"
    printf "%-21s | %-45s | %ss\n" "$ID" "$R" "$DUR" >> "$REPORT"
done

cat >> "$REPORT" << SUMMARY_EOF

SUMMARY
-------
$PASS_COUNT/18 PASS
FAIL_LIST:${FAIL_LIST:- none}

BUILD
-----
Status:   $BUILD_STATUS
Duration: ${BUILD_DURATION}s
Binary:   $ELAI_BIN

VERDICT
-------
SUMMARY_EOF

if [ "$FAIL_COUNT" -eq 0 ]; then
    echo "READY-FOR-AUDIT" >> "$REPORT"
else
    echo "NEEDS-REWORK" >> "$REPORT"
    echo "REWORK: ${FAIL_COUNT} probe(s) failed:${FAIL_LIST}" >> "$REPORT"
fi

log "Report written: $REPORT"

# Return exit code based on probe results
[ "$FAIL_COUNT" -eq 0 ] && exit 0 || exit 1
