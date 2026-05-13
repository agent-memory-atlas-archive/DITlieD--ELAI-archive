"""Phase A6 — determinism plumbing: seed propagation contract.

Pins the cross-language wire contract on the Python side:
  1. POST /v1/messages with {"seed": 42, ...} reaches the handler without
     raising (Pydantic-or-dict validation passes).
  2. The seed value propagates as a kwarg into engine.generate(...) — the
     handler MUST forward it so engine.generate can call
     transformers.set_seed / torch.manual_seed immediately before
     model.generate().
  3. Absent `seed` key keeps the pre-A6 behaviour: engine.generate is
     invoked WITHOUT a seed kwarg (or seed=None) — no behavioural drift.
  4. Streaming path (stream=True) MUST also propagate the seed.

CPU-only. No live sidecar required. We monkeypatch the module-level
``_engine`` with a Fake that records every call to ``generate`` /
``generate_streaming`` and short-circuits the downstream model code.

The test directly awaits the async ``handle_messages`` handler with a
stub Request, sidestepping httpx/TestClient dependency injection. Pattern
mirrors test_prefix_kv_cache.py — module-level monkeypatch + asyncio.

Initial expected state: RED. Pre-A6 handler does not read ``seed`` from the
request body and engine.generate does not accept it as a kwarg.

Run: cd .elai_cc/python && python -m pytest elai_inference/tests/test_seed.py -v
"""

from __future__ import annotations

import asyncio
from typing import Any

import elai_inference.server as server_mod
import orjson
import pytest

# ---------------------------------------------------------------------------
# Fakes — minimal stand-ins so handle_messages can run end-to-end without a
# real HF model. Only the surface area the handler touches is implemented.
# ---------------------------------------------------------------------------


class _FakeTokenizer:
    """The handler uses tokenizer in two places before reaching engine.generate:
    `_anthropic_to_gemma_prompt(_engine.tokenizer, ...)` and
    `_extract_cache_marker_token_position(_engine.tokenizer, ...)`. Both call
    ``apply_chat_template`` (server.py:2206, 2319). Returning a short string
    is sufficient; the prompt content is not under test here."""

    eos_token_id = 0

    def apply_chat_template(
        self, _msgs, tokenize=False, add_generation_prompt=True, tools=None,
        return_tensors=None,
    ) -> str:
        # Return a fixed string regardless of input. The handler does not
        # inspect content beyond passing it on to engine.generate.
        return "[stub-formatted-prompt]"

    def encode(self, text, add_special_tokens=False):
        # _extract_cache_marker_token_position uses encode() length to compute
        # marker_boundary. Returning [] keeps marker_boundary == None.
        return []


class _FakeEngine:
    """Records the kwargs engine.generate / generate_streaming receives so the
    test can assert ``seed`` propagated. The recorded list survives across
    multiple calls within one test (cleared in the fixture)."""

    PREFILL_CHUNK_SIZE = 8192

    def __init__(self) -> None:
        self.tokenizer = _FakeTokenizer()
        self.dola_enabled = False
        self.calls: list[dict[str, Any]] = []

    def generate(
        self,
        prompt: str,
        max_tokens: int | None = None,
        temperature: float = 0.0,
        marker_boundary: int | None = None,
        response_format: dict | None = None,
        seed: int | None = None,
    ):
        self.calls.append({
            "fn": "generate",
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "marker_boundary": marker_boundary,
            "response_format": response_format,
            "seed": seed,
        })
        # generate() returns 7-tuple: (text, token_ids, input_token_count,
        # search_trigger, gg_signals, cache_read_tokens, cache_creation_tokens).
        return (
            "stub text",
            [1, 2, 3],
            8,
            None,
            {
                "mean_logprob": None,
                "min_logprob": None,
                "entropy_mean": None,
                "entropy_max": None,
                "generation_novelty": None,
            },
            0,
            0,
        )

    def generate_streaming(
        self,
        prompt: str,
        max_tokens: int | None = None,
        temperature: float = 0.0,
        response_format: dict | None = None,
        seed: int | None = None,
    ):
        self.calls.append({
            "fn": "generate_streaming",
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "response_format": response_format,
            "seed": seed,
        })

        class _EmptyStreamer:
            def __iter__(self):
                return iter([])

        return _EmptyStreamer(), 8


class _StubRequest:
    """Bare-minimum stand-in for fastapi.Request. handle_messages only calls
    ``await request.body()`` — nothing else is touched on the request object."""

    def __init__(self, payload: dict[str, Any]) -> None:
        self._body = orjson.dumps(payload)

    async def body(self) -> bytes:
        return self._body


# ---------------------------------------------------------------------------
# Fixture — swap _engine with a fresh fake per test.
# ---------------------------------------------------------------------------


@pytest.fixture
def fake_engine(monkeypatch):
    eng = _FakeEngine()
    monkeypatch.setattr(server_mod, "_engine", eng)
    return eng


# ---------------------------------------------------------------------------
# GATE 1 — non-streaming /v1/messages with seed propagates to engine.generate.
# ---------------------------------------------------------------------------


def test_seed_propagates_to_engine_generate(fake_engine):
    payload = {
        "model": "qwen",
        "max_tokens": 8,
        "temperature": 0.0,
        "seed": 42,
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "hello"}]}
        ],
        "stream": False,
    }
    req = _StubRequest(payload)
    response = asyncio.run(server_mod.handle_messages(req))

    # Handler must NOT have errored out (would return 503/400 OrjsonResponse).
    assert response.status_code == 200, (
        f"A6 GATE1: handler returned non-200 status {response.status_code} "
        f"with seed=42 — likely root cause: request body parse rejected "
        f"unknown `seed` key, or engine.generate signature does not accept "
        f"seed kwarg. Body: {response.body!r}"
    )

    # Exactly one engine.generate call must have happened.
    assert len(fake_engine.calls) == 1, (
        f"A6 GATE1: expected exactly 1 engine.generate call, got "
        f"{len(fake_engine.calls)}: {fake_engine.calls}"
    )
    call = fake_engine.calls[0]
    assert call["fn"] == "generate", (
        f"A6 GATE1: non-stream path must route to engine.generate, got {call['fn']}"
    )
    assert call["seed"] == 42, (
        f"A6 GATE1: seed=42 did NOT reach engine.generate. Got seed={call['seed']!r}. "
        f"handle_messages must forward `seed` as a kwarg to engine.generate."
    )


# ---------------------------------------------------------------------------
# GATE 2 — absent `seed` key keeps pre-A6 behaviour (engine.generate gets None).
# ---------------------------------------------------------------------------


def test_seed_absent_keeps_pre_a6_behaviour(fake_engine):
    payload = {
        "model": "qwen",
        "max_tokens": 8,
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "hello"}]}
        ],
    }
    req = _StubRequest(payload)
    response = asyncio.run(server_mod.handle_messages(req))

    assert response.status_code == 200, (
        f"A6 GATE2: missing seed key MUST be accepted (Optional/default None). "
        f"Got status {response.status_code}; body: {response.body!r}"
    )
    assert len(fake_engine.calls) == 1
    call = fake_engine.calls[0]
    assert call["seed"] is None, (
        f"A6 GATE2: absent `seed` key must propagate as None to engine.generate; "
        f"got {call['seed']!r}. (A non-None default would silently force "
        f"determinism on pre-A6 callers — unacceptable regression.)"
    )


# ---------------------------------------------------------------------------
# GATE 3 — streaming path also propagates seed.
# ---------------------------------------------------------------------------


def test_seed_propagates_to_streaming(fake_engine):
    payload = {
        "model": "qwen",
        "max_tokens": 8,
        "temperature": 0.7,
        "seed": 99,
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "hello"}]}
        ],
        "stream": True,
    }
    req = _StubRequest(payload)

    async def _drive():
        response = await server_mod.handle_messages(req)
        # StreamingResponse defers the body iterator until it is consumed.
        # Drain it so the async generator runs through engine.generate_streaming
        # at least once. The fake streamer returns no chunks; the generator
        # still completes through the message_start / content_block_start /
        # message_stop emissions and tears down cleanly.
        chunks: list[bytes] = []
        async for chunk in response.body_iterator:
            chunks.append(
                chunk if isinstance(chunk, bytes) else chunk.encode("utf-8")
            )
        return response, b"".join(chunks)

    response, body = asyncio.run(_drive())

    assert response.status_code == 200, (
        f"A6 GATE3: streaming handler returned non-200 status {response.status_code}"
    )

    assert len(fake_engine.calls) == 1, (
        f"A6 GATE3: expected exactly 1 engine.generate_streaming call, got "
        f"{len(fake_engine.calls)}; body={body!r}"
    )
    call = fake_engine.calls[0]
    assert call["fn"] == "generate_streaming", (
        f"A6 GATE3: stream=True must route to engine.generate_streaming, got {call['fn']}"
    )
    assert call["seed"] == 99, (
        f"A6 GATE3: seed=99 did NOT reach generate_streaming; got {call['seed']!r}"
    )


# ---------------------------------------------------------------------------
# GATE 4 — invalid seed types do not crash the handler silently (400, not 500).
# ---------------------------------------------------------------------------


def test_seed_invalid_type_rejected_cleanly(fake_engine):
    """A non-integer seed should be rejected with a clean 4xx, NOT silently
    coerced or 500'd. The exact status code is a project decision (400 or 422);
    we assert 4xx and that engine.generate was NOT invoked."""
    payload = {
        "model": "qwen",
        "max_tokens": 8,
        "seed": "not-an-int",  # bad type
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "hello"}]}
        ],
    }
    req = _StubRequest(payload)
    response = asyncio.run(server_mod.handle_messages(req))

    assert 400 <= response.status_code < 500, (
        f"A6 GATE4: invalid seed type MUST yield 4xx (clean reject), not 5xx. "
        f"Got status {response.status_code}; body: {response.body!r}"
    )
    assert len(fake_engine.calls) == 0, (
        f"A6 GATE4: engine.generate must NOT be invoked for invalid seed; "
        f"got {len(fake_engine.calls)} calls"
    )


# ---------------------------------------------------------------------------
# GATE 5 — seed value exceeding transformers 32-bit ceiling rejected cleanly.
# ---------------------------------------------------------------------------


def test_seed_exceeds_ceiling_rejected_cleanly(fake_engine):
    """transformers.set_seed raises ValueError for seed > 2**32 - 1
    (= 4294967295). Rust-side Option<u64> has no upper bound, so the Python
    handler MUST reject oversized seeds with a clean 4xx before they reach
    engine.generate / transformers.set_seed."""
    payload = {
        "model": "qwen",
        "max_tokens": 8,
        "seed": 2**32,  # exactly one over the ceiling (4294967296)
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "hello"}]}
        ],
    }
    req = _StubRequest(payload)
    response = asyncio.run(server_mod.handle_messages(req))

    assert 400 <= response.status_code < 500, (
        f"A6 GATE5: seed > 2**32 - 1 MUST yield 4xx (clean reject), not 5xx. "
        f"Got status {response.status_code}; body: {response.body!r}"
    )
    assert len(fake_engine.calls) == 0, (
        f"A6 GATE5: engine.generate must NOT be invoked for oversized seed; "
        f"got {len(fake_engine.calls)} calls"
    )
