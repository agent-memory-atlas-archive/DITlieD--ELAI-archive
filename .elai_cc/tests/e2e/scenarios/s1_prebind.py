"""S-1 prebind scenario: drives `elai prebind --demo-prebind` and asserts on the
CitationAnchorsBound SSB row (off-agent evidence, not stdout).

Two arms:
  (A) normal:  --demo-prebind                     -> Bound row, anchor_count > 0
  (B) freeform: --demo-prebind --allow-freeform-citations -> Blocked row, anchor_count == 0
"""

from __future__ import annotations

from runner import RunResult, register


def _assert_bound_arm(result: RunResult) -> None:
    assert result.exit_code == 0, f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    rows = result.ssb_rows(action_type="citation_anchors_bound")
    assert len(rows) == 1, f"expected exactly 1 CitationAnchorsBound row, got {len(rows)}"
    row = rows[0]
    payload = row["payload"]
    assert "CitationAnchorsBound" in payload, f"payload key mismatch: {list(payload.keys())}"
    inner = payload["CitationAnchorsBound"]
    kind = inner["kind"]
    assert kind == "bound", f"expected kind=bound, got kind={kind}"
    anchor_count = inner["anchor_count"]
    assert anchor_count > 0, f"expected positive anchor_count, got {anchor_count}"
    assert "blake3_set_hash_hex" in inner, "missing blake3_set_hash_hex field"
    assert len(inner["blake3_set_hash_hex"]) == 64, \
        f"blake3 hash should be 64 hex chars, got {len(inner['blake3_set_hash_hex'])}"


def _assert_freeform_arm(result: RunResult) -> None:
    assert result.exit_code == 0, f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    rows = result.ssb_rows(action_type="citation_anchors_bound")
    assert len(rows) == 1, f"expected exactly 1 CitationAnchorsBound row, got {len(rows)}"
    row = rows[0]
    payload = row["payload"]
    inner = payload["CitationAnchorsBound"]
    kind = inner["kind"]
    assert kind == "blocked", \
        f"expected kind=blocked (freeform/broken arm), got kind={kind}"
    anchor_count = inner["anchor_count"]
    assert anchor_count == 0, \
        f"expected anchor_count=0 (blocked arm), got {anchor_count}"


@register("s1_prebind_bound", ["prebind", "--demo-prebind"])
def s1_prebind_bound(result: RunResult) -> None:
    _assert_bound_arm(result)


@register("s1_prebind_freeform", ["prebind", "--demo-prebind", "--allow-freeform-citations"])
def s1_prebind_freeform(result: RunResult) -> None:
    _assert_freeform_arm(result)
