#!/usr/bin/env python3
import subprocess, sys, tempfile
from pathlib import Path
TOOL = Path(__file__).with_name("claims_policy_lint.py")
def _assert_case(text, expected):
    with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
        f.write(text); name=f.name
    try:
        got=subprocess.run([sys.executable, str(TOOL), "--files", name], capture_output=True, text=True)
        assert got.returncode == expected, (got.returncode, got.stdout, got.stderr)
    finally: Path(name).unlink(missing_ok=True)
_assert_case("LLM01 prompt injection is protected [WIRED-DESIGN]\n", 0)
_assert_case("LLM01 prompt injection is fully covered\n", 1)
_assert_case("OWASP certified security\n", 1)
_assert_case("LLM01 design posture is protected [DESIGN]\n", 0)
_assert_case("LLM01 injection delta [BENCH-PENDING]\n", 0)
missing=subprocess.run([sys.executable, str(TOOL), "--files", "/definitely/missing/k13.md"])
assert missing.returncode == 2, missing.returncode
print("PASS: K-13 claims-policy contract")
