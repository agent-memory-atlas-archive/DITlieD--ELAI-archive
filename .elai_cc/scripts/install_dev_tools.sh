#!/usr/bin/env bash
# install_dev_tools.sh — install the CLI tools that are required as dev
# dependencies but cannot be declared in Cargo.toml or pyproject.toml.
#
# Run once per development host. Re-run after major version bumps.
#
# C-28 (Testing-Trust Tier T-1, mutation testing):
#   - cargo-mutants 27.0.0  Rust binary on $PATH (NOT a Cargo dep).
#   - mutmut       3.5.0   Python binary in the sidecar venv.
#
# Anti-stale (CLAUDE.md): versions pinned + verified via crates.io / PyPI
# on 2026-05-19. Bump when a new minor lands AND a green C-28 build proves
# the new output format is still 4-file (caught/missed/timeout/unviable
# for cargo-mutants; caught/survived/timeout/skipped for mutmut).
#
# Spec ref: docs/thefinalglance/C-plan/C-28_testing_trust_t1_mutation.md
# Decision doc: workflow/reasoning/C-28-decisions.md DECISION-D8.
set -euo pipefail

CARGO_MUTANTS_VERSION="27.0.0"
MUTMUT_VERSION="3.5.0"

echo "[install_dev_tools] C-28 mutation testing tools"

# -- cargo-mutants (Rust) ---------------------------------------------------
if command -v cargo >/dev/null 2>&1; then
    if cargo mutants --version 2>/dev/null | grep -q "${CARGO_MUTANTS_VERSION}"; then
        echo "[install_dev_tools] cargo-mutants ${CARGO_MUTANTS_VERSION} already installed"
    else
        echo "[install_dev_tools] installing cargo-mutants ${CARGO_MUTANTS_VERSION} (--locked)"
        cargo install cargo-mutants --version "${CARGO_MUTANTS_VERSION}" --locked
    fi
else
    echo "[install_dev_tools] WARN: cargo not on PATH; skipping cargo-mutants" >&2
fi

# -- mutmut (Python, sidecar venv) ------------------------------------------
SIDECAR_DIR="$(cd "$(dirname "$0")/.." && pwd)/python/elai_inference"
if [[ -d "${SIDECAR_DIR}" ]]; then
    echo "[install_dev_tools] installing mutmut>=${MUTMUT_VERSION} into sidecar dev deps"
    # The sidecar runs remotely (see CLAUDE.md "Remote sidecar (canonical)").
    # Use pip's editable install with the [dev] extra so mutmut lands in the
    # active venv. The user must activate their sidecar venv before invoking.
    pip install --upgrade "mutmut>=${MUTMUT_VERSION}" || {
        echo "[install_dev_tools] WARN: mutmut install failed; activate sidecar venv first" >&2
    }
else
    echo "[install_dev_tools] WARN: sidecar dir ${SIDECAR_DIR} not found; skipping mutmut" >&2
fi

echo "[install_dev_tools] done"
