#!/usr/bin/env bash
# B-29.5: Shallow-clone four pinned open-source codebases for scale benchmarking.
# Idempotent: skips any tier whose directory already exists.
#
# Configure destination via ELAI_SCALE_FIXTURE_ROOT (default: ~/elai_scale_fixtures).
# Each clone pins a specific tag so benchmarks are reproducible across runs.
#
# Usage:
#   bash scripts/fixtures/clone_scale_fixtures.sh
#   ELAI_SCALE_FIXTURE_ROOT=/data/fixtures bash scripts/fixtures/clone_scale_fixtures.sh

set -euo pipefail

ROOT="${ELAI_SCALE_FIXTURE_ROOT:-${HOME}/elai_scale_fixtures}"
mkdir -p "${ROOT}"

# ---------------------------------------------------------------------------
# Tier 1 — Linux kernel v6.7 (~36M LOC, C/asm/Rust/Python)
# ---------------------------------------------------------------------------
if [ -d "${ROOT}/linux" ]; then
    echo "[B-29.5] linux fixture already present — skipping clone"
else
    echo "[B-29.5] Cloning linux v6.7 (depth=1) ..."
    git clone --depth 1 --branch v6.7 \
        https://github.com/torvalds/linux.git \
        "${ROOT}/linux"
    echo "[B-29.5] linux clone complete."
fi

# ---------------------------------------------------------------------------
# Tier 2 — Chromium 121.0.6167.184 (~35M LOC, C++/JS/TS/Python)
# Note: uses depot_tools-free shallow clone via googlesource mirror.
# Chromium's default branch is 'main'; tag 121.0.6167.184 is a release tag.
# If the tag is unavailable on the mirror, fall back to refs/tags/ fetch.
# ---------------------------------------------------------------------------
if [ -d "${ROOT}/chromium" ]; then
    echo "[B-29.5] chromium fixture already present — skipping clone"
else
    echo "[B-29.5] Cloning chromium 121.0.6167.184 (depth=1) ..."
    git clone --depth 1 --branch 121.0.6167.184 \
        https://chromium.googlesource.com/chromium/src.git \
        "${ROOT}/chromium" || {
        echo "[B-29.5] WARNING: chromium googlesource clone failed. Trying GitHub mirror ..."
        git clone --depth 1 --branch 121.0.6167.184 \
            https://github.com/chromium/chromium.git \
            "${ROOT}/chromium"
    }
    echo "[B-29.5] chromium clone complete."
fi

# ---------------------------------------------------------------------------
# Tier 3 — VS Code 1.94.2 (~2.45M LOC, TypeScript/JavaScript)
# Primary SocratiCode parity target.
# ---------------------------------------------------------------------------
if [ -d "${ROOT}/vscode" ]; then
    echo "[B-29.5] vscode fixture already present — skipping clone"
else
    echo "[B-29.5] Cloning vscode 1.94.2 (depth=1) ..."
    git clone --depth 1 --branch 1.94.2 \
        https://github.com/microsoft/vscode.git \
        "${ROOT}/vscode"
    echo "[B-29.5] vscode clone complete."
fi

# ---------------------------------------------------------------------------
# Tier 4 — PostgreSQL REL_17_0 (~1.5M LOC, C/Perl/SQL/Tcl)
# ---------------------------------------------------------------------------
if [ -d "${ROOT}/postgres" ]; then
    echo "[B-29.5] postgres fixture already present — skipping clone"
else
    echo "[B-29.5] Cloning postgres REL_17_0 (depth=1) ..."
    git clone --depth 1 --branch REL_17_0 \
        https://github.com/postgres/postgres.git \
        "${ROOT}/postgres"
    echo "[B-29.5] postgres clone complete."
fi

echo ""
echo "[B-29.5] All fixtures ready under: ${ROOT}"
echo "[B-29.5] Pinned commits:"
for repo in linux chromium vscode postgres; do
    if [ -d "${ROOT}/${repo}/.git" ]; then
        commit=$(git -C "${ROOT}/${repo}" rev-parse HEAD 2>/dev/null || echo "unknown")
        echo "  ${repo}: ${commit}"
    fi
done
