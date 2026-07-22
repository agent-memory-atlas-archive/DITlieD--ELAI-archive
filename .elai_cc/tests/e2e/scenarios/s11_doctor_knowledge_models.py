"""S-11 doctor knowledge-models scenario: drives `elai doctor knowledge-models
--output-format json` on a fresh isolated home and asserts the doctor actually
probed each knowledge-reranker model file and reported a real per-entry status
(off-agent functional content, not exit-0-only — R4).

Model-free: doctor knowledge-models probes ONNX/SPLADE knowledge-reranker model
FILES on disk (existence/size/blake3) — no inference, no network. On a fresh
home no model files are staged, so the probe correctly reports every model
ABSENT and EXITS 1 (missing models = failure is the designed behavior, not a
bug). The FUNCTIONAL claim under test is that the probe enumerated its real
model set with a genuine per-entry probe result, not that the models are
present or that the command exited 0.
"""

from __future__ import annotations

import json

from runner import RunResult, register


# The real knowledge-model probe label set (grounded off-agent on the warm
# binary this session). Asserting the exact set proves the doctor actually
# probed its real model registry rather than emitting an empty/no-op report.
_EXPECTED_LABELS = {
    "cross-encoder-l12",
    "cross-encoder-l6",
    "coderankembed",
    "coderankembed-tok",
    "graphcodebert",
    "graphcodebert-tok",
    "granite-embed",
    "granite-embed-tok",
    "cross-encoder-tok",
    "splade-model",
    "splade-tok",
}


@register(
    "s11_doctor_knowledge_models",
    ["doctor", "knowledge-models", "--output-format", "json"],
)
def s11_doctor_knowledge_models(result: RunResult) -> None:
    # project-root defaults to cwd (the fresh isolated home the runner created),
    # so the probe scans an empty models dir -> every model reported absent.
    # On a fresh home the designed behavior is EXIT 1 (missing models = failure),
    # NOT a bug — the probe correctly detects absent models. We assert the
    # functional content (the probe ran and reported each model) and accept
    # exit 1 as the documented missing-models outcome.
    assert result.exit_code == 1, \
        f"expected exit 1 (missing-models failure on fresh home), " \
        f"got exit {result.exit_code}; stderr={result.stderr[:200]}"

    # The off-agent functional content: the real per-model probe report. Clean
    # JSON to stdout (the failing-entries line goes to stderr, not stdout).
    report = json.loads(result.stdout)
    assert isinstance(report, dict) and "entries" in report, \
        f"FUNCTIONAL FAIL: expected a JSON object with 'entries', " \
        f"got {type(report).__name__}; stdout={result.stdout[:200]!r}"

    entries = report["entries"]
    assert isinstance(entries, list) and len(entries) > 0, \
        f"FUNCTIONAL FAIL: expected a non-empty entries list, got {entries!r}"

    labels = set()
    for entry in entries:
        assert isinstance(entry, dict), f"entry not a dict: {entry!r}"
        label = entry.get("label")
        assert isinstance(label, str), f"entry label not a str: {entry!r}"
        labels.add(label)
        # Every entry carries the real probe fields.
        assert "present" in entry and isinstance(entry["present"], bool), \
            f"FUNCTIONAL FAIL: {label!r} missing boolean 'present': {entry!r}"
        assert "relative_path" in entry and isinstance(entry["relative_path"], str), \
            f"FUNCTIONAL FAIL: {label!r} missing 'relative_path': {entry!r}"
        assert "size_bytes" in entry and isinstance(entry["size_bytes"], int), \
            f"FUNCTIONAL FAIL: {label!r} missing int 'size_bytes': {entry!r}"
        assert "blake3" in entry, \
            f"FUNCTIONAL FAIL: {label!r} missing 'blake3' probe field: {entry!r}"

    # Every expected model label was actually probed (proves the doctor ran its
    # real model registry, not an empty stub). Exact-set match catches both a
    # truncated registry and a silent no-op.
    assert labels == _EXPECTED_LABELS, \
        f"FUNCTIONAL FAIL: knowledge-model probe label set mismatch. " \
        f"missing={_EXPECTED_LABELS - labels}, " \
        f"unexpected={labels - _EXPECTED_LABELS}"

    # On a fresh home NO model files are staged -> every model must be reported
    # ABSENT (present=false). This is the REAL functional finding: the probe
    # correctly detects the empty models dir, recorded not hidden (R10: a green
    # suite over a broken probe would be worse than no test).
    present_models = [e["label"] for e in entries if e["present"]]
    assert present_models == [], \
        f"FUNCTIONAL FAIL: fresh home should have NO staged models, but these " \
        f"were reported present: {present_models!r} (probe is not isolated)"
