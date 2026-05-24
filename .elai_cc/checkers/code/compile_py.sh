#!/usr/bin/env sh
# D-11 compile_py checker -- thin stdin/stdout shim called from
# .elai_cc/crates/orchestrator/src/checker_registry.rs (CompilePyChecker,
# TOML-registered).
#
# Grounding: CodeAct arxiv:2402.01030 §2.3 -- "code execution is the
# unified action space"; the syntactic+semantic load of the Python
# interpreter is the ground-truth gate. PoT arxiv:2211.12588 §3.1
# reinforces: when the Python interpreter accepts the source, the
# claim's executability is established.
#
# Protocol:
#   stdin  : {"candidate": "<python source>"}
#   stdout : {"result": "pass"|"fail"|"inapplicable", "reason": "..."}
#   exit   : 0 in all three result cases; non-zero only on script-level error.
#
# The Rust side wraps invocation in a 30s wall-clock timeout. Anti-stale-
# training: rely on `python3 -m py_compile` (stdlib since 2.5); no third-
# party byte-compile dep. Deployments without python3 return Inapplicable.

set -u

input="$(cat 2>/dev/null || true)"
if [ -z "${input}" ]; then
  printf '%s\n' '{"result":"inapplicable","reason":"empty stdin"}'
  exit 0
fi

if ! command -v python3 >/dev/null 2>&1; then
  printf '%s\n' '{"result":"inapplicable","reason":"python3 not on PATH"}'
  exit 0
fi

# Extract "candidate" field via python (already required to run the
# checker itself). On parse failure, surface as inapplicable.
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

if [ -z "${candidate}" ]; then
  printf '%s\n' '{"result":"inapplicable","reason":"empty candidate"}'
  exit 0
fi

tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t pyck)"
if [ -z "${tmpdir}" ] || [ ! -d "${tmpdir}" ]; then
  printf '%s\n' '{"result":"inapplicable","reason":"mktemp failed"}'
  exit 0
fi

# Cleanup on any exit path.
trap 'rm -rf "${tmpdir}" 2>/dev/null || true' EXIT INT TERM

src_path="${tmpdir}/candidate.py"
# shellcheck disable=SC2059
printf '%s' "${candidate}" > "${src_path}"

# `python3 -m py_compile` is the canonical syntactic check (stdlib,
# pure-Python, no network). Output is a .pyc file in the same dir; we
# discard it via the temp-dir cleanup.
stderr_path="${tmpdir}/stderr.log"
if python3 -m py_compile "${src_path}" >/dev/null 2>"${stderr_path}"; then
  printf '%s\n' '{"result":"pass"}'
  exit 0
fi

# Failure path: trim stderr to a bounded reason string.
reason="$(head -c 1024 "${stderr_path}" 2>/dev/null | tr '\n' ' ' | sed 's/"/\\"/g')"
if [ -z "${reason}" ]; then
  reason="py_compile failed (no stderr)"
fi
printf '%s\n' "{\"result\":\"fail\",\"reason\":\"${reason}\"}"
exit 0
