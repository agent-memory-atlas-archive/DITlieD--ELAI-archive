#!/usr/bin/env bash
# D-9 — L3 frozen-surfaces pre-commit + CI guard.
#
# Runs the elai CLI subcommand `audit-locked-keys --staged --json` against
# the staged git diff and refuses the commit (exit 1) if any staged path
# overlaps an entry in `.elai_cc/scripts/locked_keys_registry.toml`.
#
# Grounding sources (per CLAUDE.md grounded-by-default rule):
#   * docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md
#   * crates/orchestrator/src/locked_keys_auditor.rs (LockedKeysAuditor)
#   * crates/cli/src/audit_locked_keys.rs (the CLI driver this script
#     shells out to)
#   * arxiv:1908.04734 (Everitt current-RF principle, T1)
#   * arxiv:1906.01820 (Hubinger mesa-optimization, T1)
#
# Invocation convention:
#   * cwd MUST be the workspace root that owns `.elai_cc/`. The script
#     auto-cd's into `.elai_cc/` so the CLI's default registry-path
#     resolution (`<cwd>/scripts/locked_keys_registry.toml`) lands on the
#     canonical file.
#   * Exit 0: audit clean (no violations OR registry empty by design).
#   * Exit 1: at least one violation OR registry unreadable (fail-closed
#     per WIRING_CONTRACT.stub_ban on D-9_l3_frozen_surfaces.md).
#
# Installer integration: this script is invoked by the pre-commit hook
# template in `install_pre_commit_hook.sh` (TODO row in D-9 plan task 7)
# AND by the .github/workflows/locked-keys-check.yml CI job. Either
# entry point shells out the same `elai audit-locked-keys` invocation so
# the same registry semantics apply locally and in CI.
#
# Env vars consumed:
#   ELAI_CARGO_PROFILE   Optional. Defaults to "debug". Set to "release"
#                        in CI to use the cached release binary.
#   ELAI_LOCKED_KEYS_JSON Optional. When set to "1", prints the auditor
#                        JSON body to stderr on FAIL for downstream
#                        machine parsing.
#
# Set -euo pipefail intentionally: any unexpected shell-side failure
# (missing binary, missing registry, syntax error) must NOT silently
# permit a commit. That is the WIRING_CONTRACT.stub_ban contract for D-9.

set -euo pipefail

# Resolve workspace root from this script's location.
# `scripts/locked_keys_check.sh` is two levels under workspace root.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ELAI_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE_ROOT="$(cd "${ELAI_DIR}/.." && pwd)"

if [[ ! -d "${ELAI_DIR}" ]]; then
    echo "locked_keys_check: cannot resolve .elai_cc/ from script dir ${SCRIPT_DIR}" >&2
    exit 1
fi

REGISTRY_PATH="${ELAI_DIR}/scripts/locked_keys_registry.toml"
if [[ ! -f "${REGISTRY_PATH}" ]]; then
    # Fail-closed mirror of LockedKeysAuditError::RegistryUnreadable.
    echo "locked_keys_check: registry missing at ${REGISTRY_PATH}; failing closed" >&2
    exit 1
fi

# Pick the binary. Prefer release if it exists (CI path), fall back to
# `cargo run --quiet` for local dev (no rebuild thrash on no-op commits
# because cargo caches incremental builds in target/debug/).
PROFILE="${ELAI_CARGO_PROFILE:-debug}"
if [[ "${PROFILE}" == "release" ]]; then
    ELAI_BIN="${ELAI_DIR}/target/release/elai"
else
    ELAI_BIN="${ELAI_DIR}/target/debug/elai"
fi

cd "${ELAI_DIR}"

run_audit() {
    if [[ -x "${ELAI_BIN}" ]]; then
        "${ELAI_BIN}" audit-locked-keys --staged --json
    else
        # Fall back to cargo run when the prebuilt binary is absent.
        # `--quiet` suppresses the compiler chatter; the auditor's JSON
        # body remains the only stdout payload.
        cargo run --quiet -p elai-cli --bin elai -- audit-locked-keys --staged --json
    fi
}

# Capture stdout so we can re-emit it on FAIL for the developer.
AUDIT_OUTPUT="$(run_audit 2>&1 || true)"
AUDIT_EXIT="$?"

# The CLI exits 1 on violation / registry-unreadable; 0 on clean.
# We re-derive from the JSON body so the script behaves deterministically
# even if `cargo run` itself injects extra output.
OK_LINE="$(printf '%s\n' "${AUDIT_OUTPUT}" | grep -E '^[[:space:]]*"ok"' | head -n 1 || true)"

if [[ -z "${OK_LINE}" ]]; then
    # No "ok": ... line found — treat as unreadable / unknown failure.
    echo "locked_keys_check: audit output did not contain an ok field; failing closed" >&2
    echo "--- audit stdout/stderr ---" >&2
    printf '%s\n' "${AUDIT_OUTPUT}" >&2
    exit 1
fi

if echo "${OK_LINE}" | grep -q 'true'; then
    # Audit passed.
    if [[ "${ELAI_LOCKED_KEYS_JSON:-0}" == "1" ]]; then
        printf '%s\n' "${AUDIT_OUTPUT}" >&2
    fi
    echo "locked_keys_check: PASS (registry=${REGISTRY_PATH})"
    exit 0
fi

# Audit failed — print the body to stderr so the operator sees the
# violation report inline with the rejected commit.
echo "locked_keys_check: FAIL — staged edits touch an L3-locked surface" >&2
printf '%s\n' "${AUDIT_OUTPUT}" >&2
echo "" >&2
echo "To proceed with an authorised override, document the L3_OVERRIDE in" >&2
echo "docs/L3-FROZEN-SURFACES.md, then re-run with: git commit --no-verify" >&2
exit 1
