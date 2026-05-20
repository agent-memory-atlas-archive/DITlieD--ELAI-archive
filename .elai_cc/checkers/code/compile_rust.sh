#!/usr/bin/env sh
# D-11 compile_rust checker -- thin stdin/stdout shim called from
# .elai_cc/crates/orchestrator/src/checker_registry.rs (CompileRustChecker).
#
# Protocol:
#   stdin  : {"candidate": "<rust source>"}
#   stdout : {"result": "pass"|"fail"|"inapplicable", "reason": "..."}
#   exit   : 0 in all three result cases; non-zero only on script-level error.
#
# Uses a temp dir + `cargo check` as the syntactic + type checker. The Rust
# side wraps invocation in a 30s wall-clock timeout. Anti-stale-training:
# rely on `cargo` on PATH (no hard-coded version); deployments without rust
# return Inapplicable.

set -u

input="$(cat 2>/dev/null || true)"
if [ -z "${input}" ]; then
  printf '%s\n' '{"result":"inapplicable","reason":"empty stdin"}'
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  printf '%s\n' '{"result":"inapplicable","reason":"cargo not on PATH"}'
  exit 0
fi

# Extract "candidate" field via python (already required for the sister
# sympy.py checker). Fall back to a no-jq grep if python is missing.
candidate=""
if command -v python3 >/dev/null 2>&1; then
  candidate="$(printf '%s' "${input}" | python3 -c 'import sys,json
try:
    d=json.load(sys.stdin)
    src=d.get("candidate","")
    if not isinstance(src,str):
        src=""
    sys.stdout.write(src)
except Exception:
    sys.exit(2)
')" || {
    printf '%s\n' '{"result":"inapplicable","reason":"stdin not json"}'
    exit 0
  }
fi

if [ -z "${candidate}" ]; then
  printf '%s\n' '{"result":"inapplicable","reason":"empty candidate"}'
  exit 0
fi

tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t rustck)"
if [ -z "${tmpdir}" ] || [ ! -d "${tmpdir}" ]; then
  printf '%s\n' '{"result":"inapplicable","reason":"mktemp failed"}'
  exit 0
fi

# Cleanup on any exit path.
trap 'rm -rf "${tmpdir}" 2>/dev/null || true' EXIT INT TERM

mkdir -p "${tmpdir}/src"
cat > "${tmpdir}/Cargo.toml" <<'TOML'
[package]
name = "elai_checker_compile_rust"
version = "0.0.0"
edition = "2021"

[lib]
path = "src/lib.rs"
TOML

# shellcheck disable=SC2059
printf '%s' "${candidate}" > "${tmpdir}/src/lib.rs"

# `cargo check` is the syntactic + type check. --offline avoids index hits
# in sandboxed environments; if the candidate pulls in deps the registry
# correctly reports compile failure rather than a slow network fetch.
stderr_path="${tmpdir}/stderr.log"
if (cd "${tmpdir}" && cargo check --offline --message-format=short --color=never \
        >/dev/null 2>"${stderr_path}"); then
  printf '%s\n' '{"result":"pass"}'
  exit 0
fi

# Failure path: trim stderr to a bounded reason string.
reason="$(head -c 1024 "${stderr_path}" 2>/dev/null | tr '\n' ' ' | sed 's/"/\\"/g')"
if [ -z "${reason}" ]; then
  reason="cargo check failed (no stderr)"
fi
printf '%s\n' "{\"result\":\"fail\",\"reason\":\"${reason}\"}"
exit 0
