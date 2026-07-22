"""S-12 bootstrap-plan scenario: drives `elai bootstrap-plan --output-format
json` and asserts the binary emits the real ELAI framework bootstrap phase
sequence in the correct ORDER (off-agent functional content, not exit-0-only).

Model-free and deterministic: bootstrap-plan dumps the ordered startup
pipeline (CLI entry -> fast-path version -> profiler -> system-prompt ->
chrome-mcp -> daemon-worker -> bridge -> daemon -> background-session ->
template -> environment-runner -> main runtime). The FUNCTIONAL claim under
test is that the plan is the real ordered 12-phase pipeline, not an empty or
shuffled dump.
"""

from __future__ import annotations

import json

from runner import RunResult, register


# The real ordered ELAI bootstrap phase sequence (grounded off-agent on the
# warm binary this session). Asserting BOTH the exact list AND the exact order
# proves the bootstrap plan is the real pipeline (a shuffled or truncated plan
# fails both the equality and the phase-presence checks).
_EXPECTED_PHASES = [
    "CliEntry",
    "FastPathVersion",
    "StartupProfiler",
    "SystemPromptFastPath",
    "ChromeMcpFastPath",
    "DaemonWorkerFastPath",
    "BridgeFastPath",
    "DaemonFastPath",
    "BackgroundSessionFastPath",
    "TemplateFastPath",
    "EnvironmentRunnerFastPath",
    "MainRuntime",
]


@register("s12_bootstrap_plan", ["bootstrap-plan", "--output-format", "json"])
def s12_bootstrap_plan(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    # The off-agent functional content: the real ordered bootstrap plan.
    plan = json.loads(result.stdout)
    assert isinstance(plan, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(plan).__name__}; " \
        f"stdout={result.stdout[:200]!r}"
    assert plan.get("kind") == "bootstrap-plan", \
        f"FUNCTIONAL FAIL: expected kind=bootstrap-plan, got {plan.get('kind')!r}"

    phases = plan.get("phases")
    assert isinstance(phases, list) and len(phases) > 0, \
        f"FUNCTIONAL FAIL: expected a non-empty phases list, got {phases!r}"

    # Exact ORDER + content match (list equality catches reordering, truncation,
    # and a stub dump all at once). A set-only check would miss a reordering of
    # the startup pipeline, so this asserts the ordered list directly.
    assert phases == _EXPECTED_PHASES, \
        f"FUNCTIONAL FAIL: bootstrap phase sequence mismatch. " \
        f"expected order={_EXPECTED_PHASES}, got={phases}"

    # No phase appears more than once (the pipeline is a strictly-ordered set of
    # distinct stages; a duplicate would be a real framework bug).
    assert len(phases) == len(set(phases)), \
        f"FUNCTIONAL FAIL: duplicate bootstrap phase detected in {phases!r}"
