import json, sys

def extract(path):
    try:
        rows = json.load(open(path))
    except Exception as e:
        print(f"WARNING: {path}: {e}", file=sys.stderr)
        return []
    mcp_actions = ("mcp_tool_called", "cli_mcp_dispatch")
    return [r.get("payload", {}).get("tool_name", "?") for r in rows if r.get("action_type") in mcp_actions]

a = extract(sys.argv[1])
b = extract(sys.argv[2])
if a != b:
    print(f"DIVERGENCE: inproc={a} viamcp={b}")
    sys.exit(1)
print(f"PARITY OK: {len(a)} tool call(s) matched")
