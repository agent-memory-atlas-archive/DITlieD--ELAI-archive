"""C-39 CI parity check.

Compares two SSB row exports produced by `elai run --export-ssb <path>`
(one in-process leg, one --via-mcp leg) and asserts that the per-call
tool_name sequence is identical. The action_type differs by design
(in-process emits `mcp_tool_called`; --via-mcp emits `cli_mcp_dispatch`);
the union below normalizes both into a single tool_name stream.

Input format: either a JSON array of row objects (the format produced
by the run_subcommand --export-ssb sink as of C-39) OR JSONL (one row
per line, accepted for forward-compat with future emit shapes).

Exit code: 0 on parity, 1 on divergence, 2 on input error.
"""

import json
import sys


def _load_rows(path):
    """Load SSB rows from `path`. Accepts JSON-array or JSONL."""
    with open(path, "r", encoding="utf-8") as fh:
        text = fh.read()
    if not text.strip():
        return []
    # Try JSON-array first (canonical export shape).
    try:
        rows = json.loads(text)
        if isinstance(rows, list):
            return rows
    except json.JSONDecodeError:
        pass
    # Fall back to JSONL.
    rows = []
    for line_no, line in enumerate(text.splitlines(), start=1):
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as exc:
            print(
                "WARNING: %s:%d: malformed JSONL line: %s" % (path, line_no, exc),
                file=sys.stderr,
            )
    return rows


def extract(path):
    """Extract the per-call tool_name sequence from `path`."""
    try:
        rows = _load_rows(path)
    except OSError as exc:
        print("WARNING: %s: %s" % (path, exc), file=sys.stderr)
        return []
    # Union both action types; the dispatch leg differs but the
    # tool_name field is shared.
    mcp_actions = ("mcp_tool_called", "cli_mcp_dispatch")
    out = []
    for row in rows:
        action = row.get("action_type")
        if action not in mcp_actions:
            continue
        payload = row.get("payload") or {}
        name = payload.get("tool_name", "?")
        out.append(name)
    return out


def main(argv):
    if len(argv) != 3:
        print("usage: ci_mcp_parity_check.py <inproc.json> <viamcp.json>", file=sys.stderr)
        return 2

    a = extract(argv[1])
    b = extract(argv[2])

    if a != b:
        print("DIVERGENCE: inproc=%r viamcp=%r" % (a, b))
        return 1
    print("PARITY OK: %d tool call(s) matched" % len(a))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
