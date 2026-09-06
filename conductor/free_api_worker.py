#!/usr/bin/env python3
"""TEIKOKU P4-B (master-plan 4.1 / D-14 + the carry-forward free-API WORKER PROVIDER).

The free-api worker EXECUTOR: a worker_slot whose harness is "free-api" (or the alias
"anthropic-proxy") runs its TaskSpec by spawning a base-cc HEADLESS turn ROUTED THROUGH the
in-distro litellm free-proxy (the Anthropic Messages API on 127.0.0.1:4000), instead of
falling through to the devin.exe lane. This is the FREE pool's executor.

Why a SEPARATE module (not a worker_adapter.py edit): worker_adapter.py is 983 lines; adding
the command builder + env + result parser there would cross the 1000-line maintainability
boundary (MR-1 BLOCK). So worker_adapter DISPATCHES into this small module (precedent: P3-A
extracted gate_packs.py to keep gate_runner under 1000).

The graft (0.3-proven, env-only, NO base-cc code change):
  - base-cc client.ts honors ANTHROPIC_AUTH_TOKEN as a Bearer and ANTHROPIC_BASE_URL as the
    SDK base-url override. So setting BOTH on the spawned process routes the turn to the proxy.
  - ANTHROPIC_API_KEY is POPPED on the spawned env (the 0.3 hazard: a global API key
    suppresses the routing / plan-OAuth path). The free-api lane carries the bearer the proxy's
    LITELLM_MASTER_KEY expects, never a global key.

The command mirrors slice_driver.run_cc_turn's proven invocation
(`bun dist/cli.js --print --output-format json --strict-mcp-config
--dangerously-skip-permissions --model <m>`), but:
  - --model is a FREE-PROXY model alias (free-pollinations / free-groq / free-fast / ...),
    NEVER a devin model like kimi-k2.7 (kimi is the devin lane; the proxy has no such model).
  - dist/cli.js is passed ABSOLUTE (the provider runs the command with cwd=the task worktree,
    not the base-cc dir), and the TaskSpec prompt is fed via STDIN (input_text), exactly as
    build_codex_command returns (command, prompt).

The result is one JSON line (the `--output-format json` envelope); _parse_cc_json_notes
extracts the assistant turn so TaskResult.sanitized_notes carries it, the SAME shape the
devin/codex branches produce, so scheduler.record_worker_result disposes it unchanged.

Stdlib + in-distro paths only (no new dependency; the worker SPAWNS bun/base-cc).
"""
from __future__ import annotations

import json
import os
import shutil
from pathlib import Path

# The harness names a free-api worker_slot answers to (worker_adapter dispatches here for these).
FREE_API_HARNESSES = ("free-api", "anthropic-proxy")

# Defaults (overridable per slot / via env). The proxy endpoint is the litellm Anthropic shim.
DEFAULT_PROXY_BASE_URL = "http://127.0.0.1:4000"
# The base-cc reconstructed build root (its dist/cli.js is the runnable bundle). Mirrors
# slice_driver.BASE_CC; resolved from $HOME so it is correct in the distro.
BASE_CC_ROOT = Path(os.environ.get("TEIKOKU_BASE_CC", str(Path.home() / "teikoku" / "base-cc")))
# Where the proxy's master_key (the Bearer the proxy expects) is kept, value-read at dispatch,
# NEVER printed. Mirrors start-proxy.sh's KEYFILE.
DEFAULT_MASTER_KEY_FILE = Path(
    os.environ.get(
        "TEIKOKU_PROXY_MASTER_KEY_FILE",
        str(Path.home() / "teikoku" / "teikoku" / "free-proxy" / ".master-key"),
    )
)


def is_free_api_harness(harness: str) -> bool:
    """True when a worker_slot's harness routes to the free-api executor (the dispatch guard
    worker_adapter._build_worker_command checks BEFORE the devin fall-through)."""
    return harness in FREE_API_HARNESSES


def _resolve_bun() -> str:
    """Absolute path to the bun runtime (the provider runs with cwd=worktree, so a bare 'bun'
    relies on PATH; resolve it absolutely when we can, fall back to 'bun' on PATH)."""
    found = shutil.which("bun")
    if found:
        return found
    candidate = Path.home() / ".bun" / "bin" / "bun"
    return str(candidate) if candidate.exists() else "bun"


def _resolve_cli_js() -> str:
    """Absolute path to the base-cc dist/cli.js bundle the headless turn runs."""
    return str(BASE_CC_ROOT / "dist" / "cli.js")


def _resolve_claude_cli() -> str | None:
    """Prefer the official Claude Code CLI when present.

    Measured 2026-08-05: bun+base-cc/dist/cli.js hangs with zero stdout against the
    free Devin bridge (:18811) while connecting both to the bridge and to Anthropic
    telemetry (160.79.104.10:443). The npm `claude` binary completes the same
    glm-5-2 turn in ~2s. Prefer claude for free/devin workers; fall back to base-cc.
    """
    found = shutil.which("claude")
    if found:
        return found
    candidate = Path.home() / ".npm-global" / "bin" / "claude"
    return str(candidate) if candidate.exists() else None


def resolve_master_key(slot_account_id: str | None = None) -> str | None:
    """The Bearer token a free-api turn sends (the proxy's LITELLM_MASTER_KEY). Precedence:
    an already-exported env var (LITELLM_MASTER_KEY) -> the proxy .master-key file. Returns
    None when neither is present (keyless proxy tiers like free-pollinations still work with no
    bearer, so a missing key is not fatal here -- the env builder just omits the header).

    The value is read but NEVER logged/printed (the dispatch prompt forbids emitting secrets)."""
    env_key = os.environ.get("LITELLM_MASTER_KEY")
    if env_key:
        return env_key
    key_file = DEFAULT_MASTER_KEY_FILE
    try:
        if key_file.exists():
            text = key_file.read_text(encoding="utf-8").strip()
            return text or None
    except OSError:
        return None
    return None


def build_free_api_command(
    slot_model: str,
    taskspec_file: Path,
    *,
    bare: bool | None = None,
) -> tuple[list[str], str | None]:
    """Build the base-cc headless command for a free-api worker turn + its stdin prompt.

    slot_model is a FREE-PROXY model alias (free-pollinations / free-groq / free-fast / ...).
    The TaskSpec prompt is fed via STDIN (the returned input_text), mirroring
    build_codex_command, because the provider feeds input_text to the process stdin and
    base-cc --print reads the prompt from stdin (the slice_driver precedent: input=prompt).

    bare=True adds --bare (skip hooks). When bare is None, honor TEIKOKU_WORKER_BARE env.
    Free/devin workers pass bare=True; plan-OAuth verifier must keep bare=False.
    """
    use_bare = bare
    if use_bare is None:
        use_bare = os.environ.get("TEIKOKU_WORKER_BARE", "").strip() in ("1", "true", "yes")
    claude = _resolve_claude_cli()
    if claude:
        # Official Claude Code CLI (working free-bridge path). Prompt on stdin.
        command = [claude]
        if use_bare:
            command.append("--bare")
        command.extend([
            "--print",
            "--output-format",
            "json",
            "--dangerously-skip-permissions",
            "--model",
            slot_model,
        ])
    else:
        # Legacy bun + base-cc bundle (kept as fallback; may hang on free bridge).
        command = [
            _resolve_bun(),
            _resolve_cli_js(),
        ]
        if use_bare:
            command.append("--bare")
        command.extend([
            "--print",
            "--output-format",
            "json",
            "--strict-mcp-config",
            "--dangerously-skip-permissions",
            "--model",
            slot_model,
        ])
    prompt = taskspec_file.read_text(encoding="utf-8")
    return command, prompt


def free_api_env(
    base_env: dict[str, str],
    *,
    proxy_base_url: str | None = None,
    account_id: str | None = None,
) -> dict[str, str]:
    """Augment a worker env for the free-api lane: inject ANTHROPIC_BASE_URL (the proxy) +
    ANTHROPIC_AUTH_TOKEN (the proxy master_key, when present) and POP ANTHROPIC_API_KEY.

    base_env is the already-filtered worker env (worker_adapter._worker_env's keep-allowlist
    output). We mutate a COPY so the caller's dict is untouched. ANTHROPIC_BASE_URL /
    ANTHROPIC_AUTH_TOKEN are NOT in worker_adapter's keep-allowlist, so they are set explicitly
    here (per the 0.3 graft); ANTHROPIC_API_KEY is popped defensively even though the allowlist
    already excludes it (the 0.3 hazard: a global key mis-routes / disables the proxy path)."""
    env = dict(base_env)
    env.pop("ANTHROPIC_API_KEY", None)
    env["ANTHROPIC_BASE_URL"] = proxy_base_url or DEFAULT_PROXY_BASE_URL
    token = resolve_master_key(account_id)
    if token:
        env["ANTHROPIC_AUTH_TOKEN"] = token
    return env


def parse_cc_json_usage(stdout: str) -> dict | None:
    """TEIKOKU W-U3: extract this worker turn's token/cost figures from base-cc's
    `--print --output-format json` envelope, for the per-lane provider_usage ledger.

    base-cc's print envelope carries `usage` ({input_tokens, output_tokens,
    cache_read_input_tokens, ...}) and a top-level `total_cost_usd` (its OWN cost estimate; for a
    free-proxy lane this may be 0 or an estimate -- we record it honestly, never fabricate).
    Returns {tokens_in, tokens_out, cache_read, cost_cents} (whole ints) or None when the envelope
    has no usable usage block (a spawn-only / non-model turn records nothing). Never raises."""
    envelope: dict | None = None
    for line in reversed([ln for ln in stdout.splitlines() if ln.strip()]):
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(obj, dict):
            envelope = obj
            break
    if envelope is None:
        return None
    usage = envelope.get("usage")
    if not isinstance(usage, dict):
        return None

    def _int(value) -> int:
        try:
            return max(0, int(value))
        except (TypeError, ValueError):
            return 0

    tokens_in = _int(usage.get("input_tokens"))
    tokens_out = _int(usage.get("output_tokens"))
    cache_read = _int(usage.get("cache_read_input_tokens"))
    cost_cents = 0
    cost_usd = envelope.get("total_cost_usd")
    if isinstance(cost_usd, (int, float)) and cost_usd > 0:
        cost_cents = int(round(cost_usd * 100))
    if tokens_in == 0 and tokens_out == 0 and cache_read == 0 and cost_cents == 0:
        return None
    return {
        "tokens_in": tokens_in,
        "tokens_out": tokens_out,
        "cache_read": cache_read,
        "cost_cents": cost_cents,
    }


def parse_cc_json_notes(stdout: str, stderr: str, atif_file: Path) -> str | None:
    """Extract the assistant turn from base-cc's `--print --output-format json` envelope and
    persist a small ATIF-shaped record (so dispatch's artifact_refs / atif_artifact resolve the
    same as the devin/codex branches).

    base-cc --print --output-format json emits a single JSON object on the last non-empty
    stdout line (slice_driver: json.loads(stdout.strip().splitlines()[-1])). The assistant text
    is in `.result` (the print-mode result field); we fall back across a few likely keys and to
    the raw last line if the shape is unexpected. Never raises -- a parse failure returns None
    so dispatch falls back to provider stdout/stderr (the existing `or provider_result.stdout`).
    """
    final_text: str | None = None
    envelope: dict | None = None
    last_line = ""
    for line in reversed([ln for ln in stdout.splitlines() if ln.strip()]):
        last_line = line
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(obj, dict):
            envelope = obj
            break
    if envelope is not None:
        for key in ("result", "final_message", "message", "text", "content"):
            value = envelope.get(key)
            if isinstance(value, str) and value.strip():
                final_text = value
                break
        if final_text is None and isinstance(envelope.get("messages"), list):
            for msg in reversed(envelope["messages"]):
                if isinstance(msg, dict) and msg.get("role") == "assistant":
                    content = msg.get("content")
                    if isinstance(content, str) and content.strip():
                        final_text = content
                        break
    if final_text is None and last_line:
        final_text = last_line
    # Persist a minimal record so the ATIF artifact exists (dispatch checks atif_path.exists()).
    try:
        atif_file.parent.mkdir(parents=True, exist_ok=True)
        atif_file.write_text(
            json.dumps(
                {
                    "harness": "free-api",
                    "final_message": final_text or "",
                    "raw_envelope": envelope if isinstance(envelope, dict) else None,
                    "stderr_tail": stderr[-2000:] if stderr else "",
                },
                indent=2,
                sort_keys=True,
            ),
            encoding="utf-8",
        )
    except OSError:
        pass
    return final_text