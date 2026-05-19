#!/usr/bin/env bash
# C-21 Task 3 — Install git pre-commit hook for LOCKED_KEYS guard.
#
# Installs a pre-commit hook at the workspace-root .git/hooks/pre-commit
# (D:/.ELAI_workflow/.git/hooks/pre-commit). The hook blocks any commit
# that touches a file whose name matches one of the LOCKED_KEYS patterns
# enumerated in self_mod_safety.rs:70 (spec phase-4H.txt:201-204).
#
# LOCKED_KEYS (12 entries):
#   score_weights, gate_definitions, auditor_config, benchmark_harness,
#   test_suite, config_variants_schema, safety_locks, lock_list,
#   cloud_monthly_budget, cloud_opus_allowed, cloud_escalation_threshold,
#   elai_cc.toml (canonical TOML housing all locked keys)
#
# Idempotent: re-running replaces the existing hook with the current
# template body. Does NOT delete any other pre-existing pre-commit hook;
# back up first if you have one.

set -euo pipefail

# Resolve workspace root — script lives at .elai_cc/scripts/, so two levels up.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
HOOK_DIR="${WORKSPACE_ROOT}/.git/hooks"
HOOK_PATH="${HOOK_DIR}/pre-commit"

if [[ ! -d "${WORKSPACE_ROOT}/.git" ]]; then
    echo "ERROR: ${WORKSPACE_ROOT}/.git not found; cannot install pre-commit hook" >&2
    exit 1
fi

mkdir -p "${HOOK_DIR}"

cat > "${HOOK_PATH}" <<'HOOK_EOF'
#!/usr/bin/env bash
# Auto-installed by .elai_cc/scripts/install_pre_commit_hook.sh
# C-21 LOCKED_KEYS guard — refuses commits that modify spec phase-4H.txt:201-204
# locked config files. To bypass intentionally (e.g. spec-approved structural
# change), commit with `git commit --no-verify` and reference the
# StructuralChange approval id in the message.

set -uo pipefail

# 12 LOCKED_KEYS patterns from self_mod_safety.rs:70. Each pattern matches
# any file whose path contains the token; this is intentionally generous so
# nested config files (e.g. cloud_monthly_budget.toml in a subdir) still
# trigger.
LOCKED_PATTERNS=(
    "safety_locks"
    "auditor_config"
    "cloud_monthly_budget"
    "benchmark_harness"
    "test_suite"
    "score_weights"
    "gate_definitions"
    "lock_list"
    "cloud_opus_allowed"
    "cloud_escalation_threshold"
    "config_variants_schema"
)

# Get staged file paths (added/copied/modified/renamed; skips deletions).
STAGED=$(git diff --cached --name-only --diff-filter=ACMR)
if [[ -z "${STAGED}" ]]; then
    exit 0
fi

VIOLATIONS=()
while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    # Check elai_cc.toml directly (canonical container for these keys).
    if [[ "${path}" == *"elai_cc.toml" ]]; then
        VIOLATIONS+=("${path} (canonical locked-keys TOML)")
        continue
    fi
    for pat in "${LOCKED_PATTERNS[@]}"; do
        if [[ "${path}" == *"${pat}"* ]]; then
            VIOLATIONS+=("${path} (matches LOCKED_KEYS pattern: ${pat})")
            break
        fi
    done
done <<< "${STAGED}"

if (( ${#VIOLATIONS[@]} > 0 )); then
    echo "BLOCKED: commit modifies LOCKED_KEYS file (spec phase-4H.txt:201-204)" >&2
    for v in "${VIOLATIONS[@]}"; do
        echo "  - ${v}" >&2
    done
    echo "" >&2
    echo "Spec rule: self-mod agent MAY NEVER modify LOCKED_KEYS, even with" >&2
    echo "human approval. These files are safety rails (cloud spend, gate" >&2
    echo "definitions, benchmark harness, etc.) and modifying them via" >&2
    echo "automation defeats the defence-in-depth design." >&2
    echo "" >&2
    echo "If this is an intentional Layer 3 StructuralChange, bypass with:" >&2
    echo "  git commit --no-verify -m '<msg> (StructuralChange: <approval_id>)'" >&2
    exit 1
fi

exit 0
HOOK_EOF

chmod +x "${HOOK_PATH}"
echo "Installed pre-commit hook at: ${HOOK_PATH}"
echo "LOCKED_KEYS guard active. Bypass: git commit --no-verify"
