#!/usr/bin/env python3
"""Headless ZCode app-server adapter for one GLM controller turn.

The adapter owns one child process and one fresh protocol session.  It never
resumes a provider session.  At spawn time only, it validates the authenticated
desktop provider and passes its API key to the official ZCode child in memory.
The credential is never copied, serialized, or included in diagnostics.
"""
from __future__ import annotations

import collections
import json
import os
import queue
import re
import signal
import subprocess
import threading
import time
from pathlib import Path
from typing import Any


ZCODE_COMMAND = (
    "/opt/ZCode/zcode",
    "/opt/ZCode/resources/glm/zcode.cjs",
    "app-server",
    "--cwd",
    "/home/researcher/teikoku",
)
ZCODE_ENV = {
    "ELECTRON_RUN_AS_NODE": "1",
    "ZCODE_APP_VERSION": "3.3.6",
    "ZCODE_ENV": "production",
}
DESKTOP_PROVIDER_CONFIG = Path.home() / ".zcode" / "v2" / "config.json"
WORKSPACE_PATH = "/home/researcher/teikoku"
MODEL_REF = {"providerId": "builtin:zai-coding-plan", "modelId": "GLM-5.2"}
EXPECTED_BASE_URL = "https://api.z.ai/api/anthropic"
EXPECTED_CONTEXT_WINDOW = 1_000_000
DELIVERY_KIND = "desktop-continuous"
DEFAULT_TIMEOUT_S = 3 * 60 * 60
_READ_POLL_S = 0.2
_CLOSE_TIMEOUT_S = 2.0
_STDERR_TAIL_LINES = 80
_CLAUDE_SECRET_ENV_KEYS = frozenset({
    "ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN", "CLAUDE_CODE_OAUTH_TOKEN",
})
_ZCODE_API_KEY_ENV = "ZCODE_API_KEY"
_INHERITED_SECRET_ENV_KEYS = _CLAUDE_SECRET_ENV_KEYS | {_ZCODE_API_KEY_ENV}


class ZCodeProtocolError(RuntimeError):
    """The child violated the NDJSON protocol or reported unsafe model state."""


class ZCodeTimeoutError(TimeoutError):
    """The controller turn exceeded its wall-clock deadline."""


class ZCodeCancelled(RuntimeError):
    """The GLM stop sentinel cancelled the active controller turn."""


class ZCodeProviderConfigError(ZCodeProtocolError):
    """The authenticated desktop provider failed the spawn-time preflight."""


_SECRET_PATTERNS = (
    re.compile(r"(?i)(authorization\s*[:=]\s*(?:bearer\s+)?)[^\s,;]+"),
    re.compile(r"(?i)((?:api[-_]?key|access[-_]?token|refresh[-_]?token)\s*[:=]\s*)[^\s,;]+"),
)


def _redact(value: object, secret: str | None = None) -> str:
    text = str(value)
    if secret:
        text = text.replace(secret, "<redacted>")
    for pattern in _SECRET_PATTERNS:
        text = pattern.sub(r"\1<redacted>", text)
    return text


def _sanitize(value: Any, secret: str | None) -> Any:
    """Return a protocol/result-safe copy with the runtime secret removed."""
    if isinstance(value, dict):
        return {
            _redact(key, secret) if isinstance(key, str) else key: _sanitize(item, secret)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [_sanitize(item, secret) for item in value]
    if isinstance(value, tuple):
        return tuple(_sanitize(item, secret) for item in value)
    if isinstance(value, str):
        return _redact(value, secret)
    return value


def _desktop_provider_api_key(config_path: Path) -> str:
    """Validate the exact ZCode Coding Plan provider and return its key in memory."""
    try:
        raw = config_path.read_text(encoding="utf-8")
    except OSError:
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: configuration is unreadable"
        ) from None
    try:
        config = json.loads(raw)
    except (TypeError, ValueError):
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: configuration is not valid JSON"
        ) from None
    providers = config.get("provider") if isinstance(config, dict) else None
    provider_id = MODEL_REF["providerId"]
    provider = providers.get(provider_id) if isinstance(providers, dict) else None
    if not isinstance(provider, dict):
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: required provider is absent"
        )
    if provider.get("enabled") is not True:
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: required provider is disabled"
        )
    options = provider.get("options") if isinstance(provider.get("options"), dict) else None
    if options is None or options.get("baseURL") != EXPECTED_BASE_URL:
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: provider base URL mismatch"
        )
    models = provider.get("models") if isinstance(provider.get("models"), dict) else None
    model = models.get(MODEL_REF["modelId"]) if isinstance(models, dict) else None
    if not isinstance(model, dict):
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: required model is absent"
        )
    limit = model.get("limit") if isinstance(model.get("limit"), dict) else None
    context = limit.get("context") if isinstance(limit, dict) else None
    if type(context) is not int or context != EXPECTED_CONTEXT_WINDOW:
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: model context mismatch"
        )
    api_key = options.get("apiKey")
    if not isinstance(api_key, str) or not api_key.strip():
        raise ZCodeProviderConfigError(
            "ZCode desktop provider preflight failed: API key is absent"
        )
    return api_key.strip()


def _workspace() -> dict[str, str]:
    return {"workspacePath": WORKSPACE_PATH, "workspaceKey": WORKSPACE_PATH}


def _event_type(params: object) -> str | None:
    if not isinstance(params, dict):
        return None
    direct = params.get("type")
    if isinstance(direct, str):
        return direct
    event = params.get("event")
    if isinstance(event, dict):
        nested = event.get("type")
        if isinstance(nested, str):
            return nested
    return None


def _event_payload(params: object) -> dict[str, Any]:
    if not isinstance(params, dict):
        return {}
    event = params.get("event")
    envelope = event if isinstance(event, dict) else params
    payload = envelope.get("payload")
    return payload if isinstance(payload, dict) else {}


class ZCodeAppServer:
    """Run one fresh GLM-5.2 ZCode session over stdin/stdout NDJSON."""

    def __init__(
        self,
        *,
        command: tuple[str, ...] = ZCODE_COMMAND,
        timeout_s: float = DEFAULT_TIMEOUT_S,
        permission_policy: str = "allow",
        extra_env: dict[str, str] | None = None,
        desktop_config_path: Path | str = DESKTOP_PROVIDER_CONFIG,
    ) -> None:
        if permission_policy not in {"allow", "deny"}:
            raise ValueError("permission_policy must be 'allow' or 'deny'")
        if extra_env and _ZCODE_API_KEY_ENV in extra_env:
            raise ValueError("extra_env must not provide ZCODE_API_KEY")
        self.command = tuple(command)
        self.timeout_s = float(timeout_s)
        self.permission_policy = permission_policy
        self.extra_env = dict(extra_env or {})
        self.desktop_config_path = Path(desktop_config_path)
        self.proc: subprocess.Popen[str] | None = None
        self.session_id: str | None = None
        self.events: list[dict[str, Any]] = []
        self.diagnostics: list[str] = []
        self._responses: dict[object, dict[str, Any]] = {}
        self._stdout_queue: queue.Queue[object] = queue.Queue()
        self._stderr_tail: collections.deque[str] = collections.deque(maxlen=_STDERR_TAIL_LINES)
        self._request_seq = 0
        self._reader_threads: list[threading.Thread] = []
        self._runtime_api_key: str | None = None

    def _spawn(self) -> None:
        # Read the desktop credential only now, keep it in memory, and give it only to the official
        # child. Never inherit a stale key or Claude credential from the interactive parent shell.
        api_key = _desktop_provider_api_key(self.desktop_config_path)
        self._runtime_api_key = api_key
        env = {key: value for key, value in os.environ.items()
               if key not in _INHERITED_SECRET_ENV_KEYS}
        env.update(ZCODE_ENV)
        env.update(self.extra_env)
        for key in _INHERITED_SECRET_ENV_KEYS:
            env.pop(key, None)
        env[_ZCODE_API_KEY_ENV] = api_key
        self.proc = subprocess.Popen(
            list(self.command),
            cwd=WORKSPACE_PATH,
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
            bufsize=1,
            start_new_session=True,
        )
        stdout_thread = threading.Thread(target=self._read_stdout, name="zcode-stdout", daemon=True)
        stderr_thread = threading.Thread(target=self._read_stderr, name="zcode-stderr", daemon=True)
        stdout_thread.start()
        stderr_thread.start()
        self._reader_threads = [stdout_thread, stderr_thread]

    def _read_stdout(self) -> None:
        assert self.proc is not None and self.proc.stdout is not None
        try:
            for raw in self.proc.stdout:
                self._stdout_queue.put(raw.rstrip("\r\n"))
        finally:
            self._stdout_queue.put(None)

    def _read_stderr(self) -> None:
        assert self.proc is not None and self.proc.stderr is not None
        for raw in self.proc.stderr:
            self._stderr_tail.append(_redact(raw.rstrip("\r\n"), self._runtime_api_key))

    def _write(self, frame: dict[str, Any]) -> None:
        if self.proc is None or self.proc.stdin is None or self.proc.poll() is not None:
            raise ZCodeProtocolError("ZCode app-server is not writable")
        try:
            self.proc.stdin.write(json.dumps(frame, separators=(",", ":")) + "\n")
            self.proc.stdin.flush()
        except (BrokenPipeError, OSError) as exc:
            raise ZCodeProtocolError(
                f"ZCode app-server write failed: {_redact(exc, self._runtime_api_key)}"
            ) from exc

    def _next_id(self) -> int:
        self._request_seq += 1
        return self._request_seq

    def _handle_server_request(self, frame: dict[str, Any]) -> None:
        request_id = frame.get("id")
        method = frame.get("method")
        params = frame.get("params") if isinstance(frame.get("params"), dict) else {}
        if method == "interaction/requestPermission":
            decision = "allow" if self.permission_policy == "allow" else "deny"
            reason = f"headless GLM orchestrator permission policy: {self.permission_policy}"
            result = {"decision": decision, "reason": reason}
            self.diagnostics.append(
                "permission "
                f"tool={_redact(params.get('toolName', '<unknown>'))} "
                f"risk={_redact(params.get('riskLevel', '<unknown>'))} decision={decision}"
            )
            self._write({"id": request_id, "result": result})
            return
        if method == "interaction/requestUserInput":
            self._write({
                "id": request_id,
                "result": {"action": "decline", "reason": "headless controller has no user input"},
            })
            return
        if method == "interaction/requestProviderRuntimeHeaders":
            self._write({
                "id": request_id,
                "result": {
                    "headersApplied": False,
                    "errorMessage": "headless controller does not supply provider headers",
                },
            })
            return
        self._write({
            "id": request_id,
            "error": {"code": -32601, "message": f"unsupported client method: {_redact(method)}"},
        })

    def _read_frame(self, deadline: float, stop_path: Path | None) -> dict[str, Any]:
        while True:
            if stop_path is not None and stop_path.is_file():
                raise ZCodeCancelled("GLM stop sentinel present")
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise ZCodeTimeoutError(f"ZCode controller turn timed out after {self.timeout_s:g}s")
            try:
                raw = self._stdout_queue.get(timeout=min(_READ_POLL_S, remaining))
            except queue.Empty:
                if self.proc is not None and self.proc.poll() is not None:
                    raw = None
                else:
                    continue
            if raw is None:
                rc = self.proc.poll() if self.proc is not None else None
                stderr = " | ".join(self._stderr_tail)
                detail = f"; stderr={stderr[-1000:]}" if stderr else ""
                raise ZCodeProtocolError(f"ZCode app-server stdout closed early (rc={rc}){detail}")
            try:
                frame = json.loads(str(raw))
            except ValueError as exc:
                safe_raw = _redact(raw, self._runtime_api_key)
                raise ZCodeProtocolError(
                    f"malformed ZCode NDJSON output: {safe_raw[:300]}"
                ) from exc
            if not isinstance(frame, dict):
                raise ZCodeProtocolError("malformed ZCode NDJSON output: object required")
            return _sanitize(frame, self._runtime_api_key)

    def _route_frame(self, frame: dict[str, Any]) -> None:
        if "method" in frame and "id" in frame:
            self._handle_server_request(frame)
            return
        if "method" in frame:
            params = frame.get("params")
            if frame.get("method") == "session/event" and isinstance(params, dict):
                self.events.append(params)
            return
        if "id" in frame:
            self._responses[frame.get("id")] = frame
            return
        raise ZCodeProtocolError("malformed ZCode NDJSON frame: no method or id")

    def _request(
        self,
        method: str,
        params: dict[str, Any],
        deadline: float,
        stop_path: Path | None,
    ) -> Any:
        request_id = self._next_id()
        self._write({"id": request_id, "method": method, "params": params})
        while request_id not in self._responses:
            self._route_frame(self._read_frame(deadline, stop_path))
        response = self._responses.pop(request_id)
        if "error" in response:
            error = response.get("error") if isinstance(response.get("error"), dict) else {}
            raise ZCodeProtocolError(
                "ZCode "
                f"{method} failed code={error.get('code')}: "
                f"{_redact(error.get('message', 'error'), self._runtime_api_key)}"
            )
        if "result" not in response:
            raise ZCodeProtocolError(f"ZCode {method} response omitted result")
        return response["result"]

    @staticmethod
    def _validate_snapshot(snapshot: object) -> None:
        if not isinstance(snapshot, dict):
            raise ZCodeProtocolError("ZCode session snapshot is not an object")
        settings = snapshot.get("settings") if isinstance(snapshot.get("settings"), dict) else {}
        model = settings.get("model") if isinstance(settings.get("model"), dict) else {}
        current = model.get("current") if isinstance(model.get("current"), dict) else None
        if current is None:
            raise ZCodeProtocolError("ZCode session did not report a current modelRef")
        reported_ref = {"providerId": current.get("providerId"), "modelId": current.get("modelId")}
        if current.get("variant") is not None:
            reported_ref["variant"] = current.get("variant")
        if reported_ref != MODEL_REF:
            raise ZCodeProtocolError(
                f"ZCode modelRef mismatch: expected={MODEL_REF!r} reported={reported_ref!r}"
            )
        contexts: list[int] = []
        projection = snapshot.get("projection") if isinstance(snapshot.get("projection"), dict) else {}
        projected = projection.get("contextWindow")
        if isinstance(projected, int) and projected > 0:
            contexts.append(projected)
        available = model.get("available") if isinstance(model.get("available"), list) else []
        for item in available:
            if not isinstance(item, dict) or item.get("ref") != current:
                continue
            context_window = item.get("contextWindow")
            if isinstance(context_window, int) and context_window > 0:
                contexts.append(context_window)
        if not contexts:
            raise ZCodeProtocolError("ZCode session did not report a positive context window")
        if any(value != EXPECTED_CONTEXT_WINDOW for value in contexts):
            raise ZCodeProtocolError(
                "ZCode context-window mismatch: "
                f"expected={EXPECTED_CONTEXT_WINDOW} reported={sorted(set(contexts))}"
            )

    def _request_stop(self, deadline: float) -> None:
        if self.session_id is None or self.proc is None or self.proc.poll() is not None:
            return
        try:
            self._request("session/stop", {"sessionId": self.session_id}, deadline, None)
        except (ZCodeProtocolError, ZCodeTimeoutError):
            return

    def _terminate_process_group(self) -> None:
        proc = self.proc
        if proc is None:
            return
        if proc.poll() is None:
            try:
                os.killpg(proc.pid, signal.SIGTERM)
            except (ProcessLookupError, PermissionError, OSError):
                try:
                    proc.terminate()
                except OSError:
                    pass
            try:
                proc.wait(timeout=_CLOSE_TIMEOUT_S)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(proc.pid, signal.SIGKILL)
                except (ProcessLookupError, PermissionError, OSError):
                    try:
                        proc.kill()
                    except OSError:
                        pass
                try:
                    proc.wait(timeout=_CLOSE_TIMEOUT_S)
                except subprocess.TimeoutExpired:
                    pass
        for stream in (proc.stdin, proc.stdout, proc.stderr):
            try:
                if stream is not None:
                    stream.close()
            except OSError:
                pass
        for thread in self._reader_threads:
            if thread is not threading.current_thread():
                thread.join(timeout=_CLOSE_TIMEOUT_S)

    def _wait_for_terminal(self, deadline: float, stop_path: Path | None) -> dict[str, Any]:
        checked = 0
        while True:
            while checked < len(self.events):
                event = self.events[checked]
                checked += 1
                kind = _event_type(event)
                if kind == "turn.completed":
                    return {"kind": kind, "payload": _event_payload(event)}
                if kind == "turn.failed":
                    return {"kind": kind, "payload": _event_payload(event)}
            self._route_frame(self._read_frame(deadline, stop_path))

    def run_turn(self, prompt: str, controller_session_id: str, stop_path: Path | None = None) -> dict:
        """Run a fresh session and return the supervisor-compatible result dictionary."""
        started = time.monotonic()
        deadline = started + self.timeout_s
        terminal: dict[str, Any] = {}
        cancelled = False
        timed_out = False
        error = ""
        rc: int | None = None
        try:
            self._spawn()
            created = self._request(
                "session/create",
                {
                    "workspace": _workspace(),
                    "mode": "build",
                    "model": dict(MODEL_REF),
                    "persistence": "immediate",
                },
                deadline,
                stop_path,
            )
            self._validate_snapshot(created)
            self.session_id = created.get("session", {}).get("sessionId")
            if not isinstance(self.session_id, str) or not self.session_id.strip():
                raise ZCodeProtocolError("ZCode session/create omitted sessionId")
            pinned = self._request(
                "session/setModel",
                {"sessionId": self.session_id, "model": dict(MODEL_REF)},
                deadline,
                stop_path,
            )
            self._validate_snapshot(pinned)
            subscribed = self._request(
                "session/subscribe",
                {
                    "sessionId": self.session_id,
                    "deliveryKind": DELIVERY_KIND,
                    "afterSeq": 0,
                    "includeSnapshot": True,
                },
                deadline,
                stop_path,
            )
            snapshot = subscribed.get("snapshot") if isinstance(subscribed, dict) else None
            self._validate_snapshot(snapshot)
            backlog = subscribed.get("events") if isinstance(subscribed, dict) else None
            if isinstance(backlog, list):
                self.events.extend(item for item in backlog if isinstance(item, dict))
            accepted = self._request(
                "session/send",
                {
                    "sessionId": self.session_id,
                    "inputId": controller_session_id,
                    "queryId": controller_session_id,
                    "content": prompt,
                },
                deadline,
                stop_path,
            )
            if not isinstance(accepted, dict) or accepted.get("accepted") is not True:
                raise ZCodeProtocolError("ZCode session/send was not accepted")
            terminal = self._wait_for_terminal(deadline, stop_path)
            if terminal.get("kind") == "turn.completed":
                rc = 0
            else:
                rc = 1
                error = _redact(
                    terminal.get("payload", {}).get("error", "ZCode turn failed"),
                    self._runtime_api_key,
                )
        except ZCodeCancelled as exc:
            cancelled = True
            rc = 130
            error = str(exc)
            self._request_stop(min(time.monotonic() + _CLOSE_TIMEOUT_S, deadline))
        except ZCodeTimeoutError as exc:
            timed_out = True
            error = str(exc)
            self._request_stop(time.monotonic() + _CLOSE_TIMEOUT_S)
        except (ZCodeProtocolError, OSError, ValueError) as exc:
            rc = 1
            error = _redact(exc, self._runtime_api_key)
        finally:
            if self.session_id is not None and not cancelled and not timed_out:
                try:
                    self._request(
                        "session/close",
                        {"sessionId": self.session_id, "expectedPersistence": "immediate"},
                        time.monotonic() + _CLOSE_TIMEOUT_S,
                        None,
                    )
                except (ZCodeProtocolError, ZCodeTimeoutError):
                    pass
            child_pid = self.proc.pid if self.proc is not None else None
            self._terminate_process_group()
            self._runtime_api_key = None
        payload = terminal.get("payload") if isinstance(terminal.get("payload"), dict) else {}
        response = payload.get("response") if isinstance(payload.get("response"), str) else ""
        stderr_tail = "\n".join(self._stderr_tail)[-2000:]
        diagnostic = error or stderr_tail
        return {
            "rc": rc,
            "timed_out": timed_out,
            "cancelled": cancelled,
            "stdout_tail": response[-2000:],
            "stderr_tail": _redact(diagnostic)[-2000:],
            "events": list(self.events),
            "session_id": self.session_id,
            "model_ref": dict(MODEL_REF),
            "context_window": EXPECTED_CONTEXT_WINDOW,
            "child_pid": child_pid,
            "duration_s": time.monotonic() - started,
            "diagnostics": list(self.diagnostics),
        }


def run_zcode_turn(
    prompt: str,
    controller_session_id: str,
    *,
    timeout_s: float = DEFAULT_TIMEOUT_S,
    stop_path: Path | None = None,
    extra_env: dict[str, str] | None = None,
) -> dict:
    """Production entrypoint used by ``glm_supervisor._spawn_glm_turn``."""
    return ZCodeAppServer(timeout_s=timeout_s, extra_env=extra_env).run_turn(
        prompt, controller_session_id, stop_path)
