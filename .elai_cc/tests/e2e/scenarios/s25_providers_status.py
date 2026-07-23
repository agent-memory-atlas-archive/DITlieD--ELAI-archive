"""s25 providers-status: drives `elai providers status --output-format json` on a
fresh isolated home and asserts the binary returned its real provider registry.

The providers-discovery surface exposes the canonical ELAI provider-kind registry.
It is model-free + deterministic: the registry is static (no provider is called),
byte-identical sha256 across fresh homes. The output is a JSON list of exactly 22
backend objects, each carrying the 6-field contract
(backend/configured/display_name/models/stale/verified_at), with the canonical
backends (sidecar/anthropic_api/claude_code/llama_cpp/ollama/codex_cli/bedrock/
azure_openai) present. A regression that drops a backend, changes the field
contract, or returns a stub/non-JSON fails the assertions, not just exit-0.
"""
from __future__ import annotations

import json

from runner import RunResult, register

EXPECTED_KEYS = {"backend", "configured", "display_name", "models", "stale", "verified_at"}
CANONICAL_BACKENDS = (
    "sidecar",
    "anthropic_api",
    "claude_code",
    "llama_cpp",
    "ollama",
    "codex_cli",
    "bedrock",
    "azure_openai",
)


@register("s25_providers_status", ["providers", "status", "--output-format", "json"])
def s25_providers_status(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    providers = json.loads(result.stdout)
    assert isinstance(providers, list), \
        f"FUNCTIONAL FAIL: expected a JSON list, got {type(providers).__name__}; " \
        f"stdout={result.stdout[:200]!r}"
    assert len(providers) == 22, \
        f"FUNCTIONAL FAIL: expected 22 provider backends, got {len(providers)} " \
        f"(a dropped/added backend fails here)"
    for i, p in enumerate(providers):
        assert isinstance(p, dict), \
            f"FUNCTIONAL FAIL: provider[{i}] is not a dict"
        assert set(p.keys()) == EXPECTED_KEYS, \
            f"FUNCTIONAL FAIL: provider[{i}] keys {set(p.keys())} != {EXPECTED_KEYS}"
        assert isinstance(p["backend"], str) and p["backend"], \
            f"FUNCTIONAL FAIL: provider[{i}] missing backend string"
        assert isinstance(p["models"], list), \
            f"FUNCTIONAL FAIL: provider[{i}] models is not a list"
    backends = {p["backend"] for p in providers}
    for canonical in CANONICAL_BACKENDS:
        assert canonical in backends, \
            f"FUNCTIONAL FAIL: canonical provider backend {canonical!r} absent from the registry"
