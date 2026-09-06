#!/usr/bin/env python3
"""Deterministic NDJSON child used by zcode_adapter unit tests."""
from __future__ import annotations

import json
import os
import sys
import time

MODEL = {"providerId": "builtin:zai-coding-plan", "modelId": "GLM-5.2"}
SCENARIO = os.environ.get("FAKE_ZCODE_SCENARIO", "success")
LOG = os.environ.get("FAKE_ZCODE_LOG")


def write(frame):
    sys.stdout.write(json.dumps(frame, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def record(frame):
    if LOG:
        with open(LOG, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(frame, sort_keys=True) + "\n")


record({"startup_env": {
    "CLAUDE_CODE_SESSION_ID": os.environ.get("CLAUDE_CODE_SESSION_ID"),
    "TEIKOKU_AUTONOMOUS": os.environ.get("TEIKOKU_AUTONOMOUS"),
    "ELAI_OPUS_CODEX_EXECUTOR": os.environ.get("ELAI_OPUS_CODEX_EXECUTOR"),
    "claude_secret_present": any(os.environ.get(key) for key in (
        "ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN", "CLAUDE_CODE_OAUTH_TOKEN")),
    "zcode_api_key_present": bool(os.environ.get("ZCODE_API_KEY")),
}})


def snapshot():
    model = dict(MODEL)
    context = 1_000_000
    if SCENARIO == "wrong-model":
        model["modelId"] = "GLM-4.7"
    if SCENARIO == "wrong-context":
        context = 200_000
    return {
        "protocol": {"name": "ZCode Protocol", "version": 1},
        "session": {"sessionId": "fake-session-1"},
        "settings": {"model": {"current": model, "available": [
            {"ref": model, "contextWindow": context}
        ]}},
        "projection": {"contextWindow": context},
    }


for line in sys.stdin:
    frame = json.loads(line)
    record(frame)
    method = frame.get("method")
    request_id = frame.get("id")
    if method == "session/create" or method == "session/setModel":
        write({"id": request_id, "result": snapshot()})
    elif method == "session/subscribe":
        write({"id": request_id, "result": {
            "sessionId": "fake-session-1", "eventSeq": 0, "events": [], "snapshot": snapshot()
        }})
    elif method == "session/send":
        secret = os.environ.get("ZCODE_API_KEY", "")
        if SCENARIO == "secret-error":
            sys.stderr.write(f"provider rejected credential {secret}\n")
            sys.stderr.flush()
            write({"id": request_id, "error": {
                "code": 401, "message": f"provider rejected credential {secret}"
            }})
            continue
        write({"id": request_id, "result": {"sessionId": "fake-session-1", "accepted": True,
                                              "stateRevision": 1}})
        if SCENARIO == "eof":
            raise SystemExit(0)
        if SCENARIO == "malformed":
            sys.stdout.write("not-json\n")
            sys.stdout.flush()
            time.sleep(30)
            continue
        if SCENARIO in {"timeout", "wait-cancel"}:
            continue
        if SCENARIO == "permission":
            write({
                "id": "permission-1",
                "method": "interaction/requestPermission",
                "params": {"requestId": "permission-1", "sessionId": "fake-session-1",
                           "toolCallId": "tool-1", "toolName": "Bash", "reason": "test",
                           "riskLevel": "medium", "input": {"command": "true"},
                           "options": [{"optionId": "allow", "kind": "allow", "name": "Allow",
                                        "response": {"decision": "allow"}}]},
            })
            permission_response = json.loads(sys.stdin.readline())
            record(permission_response)
        if SCENARIO == "secret-output":
            sys.stderr.write(f"provider diagnostic {secret}\n")
            sys.stderr.flush()
        write({"method": "session/event", "params": {
            "type": "turn.completed", "payload": {
                "response": (f"fake controller complete {secret}"
                             if SCENARIO == "secret-output"
                             else "fake controller complete")
            }
        }})
    elif method == "session/stop":
        if SCENARIO != "timeout":
            write({"id": request_id, "result": {}})
    elif method == "session/close":
        write({"id": request_id, "result": {"closed": True}})
        raise SystemExit(0)
    elif "result" in frame or "error" in frame:
        continue
    else:
        write({"id": request_id, "error": {"code": -32601, "message": "unknown"}})
