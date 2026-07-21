"""J-36 streaming prefix-KV lease contract (offline, CPU-only, no GPU).

Pins the streaming-path lease wiring:
  1. handle_messages stream=True threads marker_boundary into
     _stream_messages_response, which threads it into generate_streaming.
  2. generate_streaming, with a fake PrefixKVCache that returns a hit,
     uses cropped gen_input_ids / past_kv (assert call kwargs).
  3. SSE message_start surfaces cache_read/creation tokens (not hardcoded 0).

No live GPU required. Pattern mirrors test_seed.py (module-level monkeypatch
+ asyncio drive of handle_messages) and test_prefix_kv_cache.py (_FakeCache).

Run: cd .elai_cc/python && python -m pytest elai_inference/tests/test_j36_streaming_prefix_lease.py -v
"""

from __future__ import annotations

import asyncio
import types
from typing import Any, Optional

import torch

import elai_inference.server as server_mod
import orjson
import pytest


# ---------------------------------------------------------------------------
# Fakes
# ---------------------------------------------------------------------------


class _FakeTokenizer:
    eos_token_id = 0

    def apply_chat_template(
        self, _msgs, tokenize=False, add_generation_prompt=True,
        tools=None, return_tensors=None,
    ) -> str:
        return "[stub-formatted-prompt]"

    def encode(self, text, add_special_tokens=False):
        return []


class _FakeCache:
    """Stand-in for a leased DynamicCache. Records crop calls so the test
    can assert the proven crop-to-_prefix_len-1 invariant was reused."""

    def __init__(self, label: str, seq_len: int = 100) -> None:
        self.label = label
        self._seq_len = seq_len
        self.crop_calls: list[int] = []

    def crop(self, new_len: int) -> None:
        self.crop_calls.append(new_len)
        self._seq_len = new_len

    def get_seq_length(self) -> int:
        return self._seq_len


class _FakePrefixKVCache:
    """Fake PrefixKVCache.lease returns a hit with a configurable prefix_len.
    Records the lease call kwargs so the test can assert marker_boundary
    was threaded."""

    def __init__(self, prefix_len: int = 20, cache_obj: Optional[_FakeCache] = None) -> None:
        self._prefix_len = prefix_len
        self._cache = cache_obj or _FakeCache("leased", seq_len=prefix_len + 10)
        self.lease_calls: list[dict[str, Any]] = []

    def lease(self, input_ids, marker_boundary=None):
        self.lease_calls.append({
            "input_ids_shape": tuple(input_ids.shape),
            "marker_boundary": marker_boundary,
        })
        return self._cache, self._prefix_len, self._prefix_len + 50

    def admit(self, input_ids, cache):
        pass


class _FakeStreamer:
    """Empty streamer — generate_streaming's _run thread calls model.generate
    which we bypass; the streamer just needs to be iterable."""
    def __iter__(self):
        return iter([])


class _FakeEngine:
    """Fake engine that records generate_streaming kwargs and returns a
    _FakeStreamer + cache counters. Used by the handle_messages-level test."""

    PREFILL_CHUNK_SIZE = 8192

    def __init__(self) -> None:
        self.tokenizer = _FakeTokenizer()
        self.dola_enabled = False
        self.calls: list[dict[str, Any]] = []

    def generate_streaming(
        self,
        prompt: str,
        max_tokens: int | None = None,
        temperature: float = 0.0,
        response_format: dict | None = None,
        seed: int | None = None,
        marker_boundary: int | None = None,
    ):
        self.calls.append({
            "fn": "generate_streaming",
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "response_format": response_format,
            "seed": seed,
            "marker_boundary": marker_boundary,
        })
        streamer = _FakeStreamer()
        streamer.cache_read_tokens = 20
        streamer.cache_creation_tokens = 0
        return streamer, 8


class _StubRequest:
    def __init__(self, payload: dict[str, Any]) -> None:
        self._body = orjson.dumps(payload)

    async def body(self) -> bytes:
        return self._body


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def fake_engine(monkeypatch):
    eng = _FakeEngine()
    monkeypatch.setattr(server_mod, "_engine", eng)
    return eng


# ---------------------------------------------------------------------------
# GATE 1 — handle_messages stream=True threads marker_boundary into
# generate_streaming via _stream_messages_response.
# ---------------------------------------------------------------------------


def test_marker_boundary_threads_into_streaming(fake_engine):
    payload = {
        "model": "qwen",
        "max_tokens": 8,
        "temperature": 0.0,
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "hello"}]},
        ],
        "stream": True,
    }
    req = _StubRequest(payload)

    async def _drive():
        response = await server_mod.handle_messages(req)
        chunks: list[bytes] = []
        async for chunk in response.body_iterator:
            chunks.append(
                chunk if isinstance(chunk, bytes) else chunk.encode("utf-8")
            )
        return response, b"".join(chunks)

    response, body = asyncio.run(_drive())

    assert response.status_code == 200, (
        f"J-36 GATE1: streaming handler returned non-200: {response.status_code}"
    )
    assert len(fake_engine.calls) == 1, (
        f"J-36 GATE1: expected 1 generate_streaming call, got {len(fake_engine.calls)}"
    )
    call = fake_engine.calls[0]
    assert call["fn"] == "generate_streaming"
    assert "marker_boundary" in call, (
        "J-36 GATE1: generate_streaming call missing marker_boundary kwarg"
    )


# ---------------------------------------------------------------------------
# GATE 2 — generate_streaming with a fake PrefixKVCache hit uses cropped
# gen_input_ids and past_kv (the proven crop-to-_prefix_len-1 invariant).
# This test monkeypatches _prefix_kv_cache and the model.generate call.
# ---------------------------------------------------------------------------


class _FakeModel:
    """Fake model.generate that records kwargs and does nothing."""
    def __init__(self) -> None:
        self.generate_calls: list[dict[str, Any]] = []
        self.device = torch.device("cpu")

    def generate(self, **kwargs):
        self.generate_calls.append(kwargs)
        return types.SimpleNamespace(sequences=torch.zeros(1, 1, dtype=torch.long))


def test_streaming_lease_hit_crops_and_feeds_past_kv(monkeypatch):
    """With a fake PrefixKVCache returning a hit, generate_streaming must:
    - call lease with marker_boundary
    - crop the cache to _prefix_len - 1
    - pass past_key_values + cropped gen_input_ids to model.generate
    """
    fake_cache_obj = _FakeCache("leased", seq_len=100)
    fake_pkv = _FakePrefixKVCache(prefix_len=20, cache_obj=fake_cache_obj)
    monkeypatch.setattr(server_mod, "_prefix_kv_cache", fake_pkv)

    fake_model = _FakeModel()
    fake_tokenizer = types.SimpleNamespace(
        eos_token_id=0,
        _FakeTokenizer_cls=_FakeTokenizer,
    )

    # Build a minimal fake engine instance
    eng = server_mod.InferenceEngine.__new__(server_mod.InferenceEngine)
    eng.model = fake_model
    eng.tokenizer = _FakeTokenizer()
    eng.max_new_tokens = 16
    eng.dola_enabled = False
    eng.kv_compressor = types.SimpleNamespace(enabled=False)
    eng.streaming_llm_sinks = 0
    eng.streaming_llm_window = 0
    eng._sdpa_pin_enabled = False
    eng._dola_processor = None
    eng._gg_collector = types.SimpleNamespace(reset=lambda: None)
    eng._novelty_tracker = types.SimpleNamespace(
        compute=lambda p, t, m, tok: 0.0,
    )
    eng.lock = __import__("threading").Lock()
    eng._format_prompt = lambda prompt: prompt
    eng._build_constrained_prefix_fn = lambda rf: None
    eng._apply_streaming_llm_if_active = lambda mask, context_position: mask
    eng._make_cache = lambda: None
    eng._chunked_prefill = lambda ids, mask: None

    # Monkeypatch the tokenizer to return a fixed tensor
    _fake_input_ids = torch.tensor([[1, 2, 3, 4, 5, 6, 7, 8, 9, 10] * 3], dtype=torch.long)
    _fake_attention_mask = torch.ones_like(_fake_input_ids)

    class _FakeTokenizerBatch:
        def __init__(self, input_ids, attention_mask):
            self.input_ids = input_ids
            self.attention_mask = attention_mask

        def to(self, device):
            return self

    class _FakeTokenizerWithCall:
        eos_token_id = 0

        def __call__(self, text, return_tensors="pt"):
            return _FakeTokenizerBatch(_fake_input_ids, _fake_attention_mask)

        def apply_chat_template(self, *a, **kw):
            return "[stub]"

        def encode(self, text, add_special_tokens=False):
            return []

        def decode(self, ids, skip_special_tokens=False):
            return ""

    eng.tokenizer = _FakeTokenizerWithCall()

    # We need to prevent the _run thread from actually running model.generate
    # in a background thread (race conditions). Instead, capture generation_kwargs
    # by intercepting model.generate directly.
    captured_kwargs: list[dict[str, Any]] = []

    def _fake_generate(**kwargs):
        captured_kwargs.append(kwargs)
        return types.SimpleNamespace(sequences=torch.zeros(1, 1, dtype=torch.long))

    eng.model.generate = _fake_generate

    # Call generate_streaming
    streamer, input_count = eng.generate_streaming(
        "test prompt", max_tokens=8, temperature=0.0,
        marker_boundary=15,
    )
    cache_read = getattr(streamer, "cache_read_tokens", 0)
    cache_creation = getattr(streamer, "cache_creation_tokens", 0)

    # Wait for the background thread to execute (it calls model.generate)
    import time as _time
    _time.sleep(0.2)

    # Assert lease was called with marker_boundary
    assert len(fake_pkv.lease_calls) == 1, (
        f"J-36 GATE2: expected 1 lease call, got {len(fake_pkv.lease_calls)}"
    )
    assert fake_pkv.lease_calls[0]["marker_boundary"] == 15, (
        f"J-36 GATE2: lease not called with marker_boundary=15, "
        f"got {fake_pkv.lease_calls[0]['marker_boundary']}"
    )

    # Assert cache was cropped to _prefix_len - 1 = 19
    assert len(fake_cache_obj.crop_calls) == 1, (
        f"J-36 GATE2: expected 1 crop call, got {len(fake_cache_obj.crop_calls)}"
    )
    assert fake_cache_obj.crop_calls[0] == 19, (
        f"J-36 GATE2: crop should be _prefix_len-1=19, "
        f"got {fake_cache_obj.crop_calls[0]}"
    )

    # Assert cache_read_tokens returned
    assert cache_read == 20, (
        f"J-36 GATE2: cache_read_tokens should be 20 (prefix_len), got {cache_read}"
    )

    # Assert model.generate received past_key_values + cropped input_ids
    assert len(captured_kwargs) == 1, (
        f"J-36 GATE2: expected 1 model.generate call, got {len(captured_kwargs)}"
    )
    gen_kwargs = captured_kwargs[0]
    assert "past_key_values" in gen_kwargs, (
        "J-36 GATE2: model.generate must receive past_key_values on cache hit"
    )
    assert gen_kwargs["past_key_values"] is fake_cache_obj, (
        "J-36 GATE2: past_key_values must be the leased (cropped) cache"
    )
    # gen_input_ids should be inputs[:, _prefix_len-1:] = inputs[:, 19:]
    expected_gen_len = _fake_input_ids.shape[1] - 19
    actual_gen_len = gen_kwargs["input_ids"].shape[1]
    assert actual_gen_len == expected_gen_len, (
        f"J-36 GATE2: gen_input_ids length should be {expected_gen_len} "
        f"(total - _prefix_len + 1), got {actual_gen_len}"
    )


# ---------------------------------------------------------------------------
# GATE 3 — SSE message_start surfaces cache_read/creation tokens (not 0).
# ---------------------------------------------------------------------------


def test_sse_message_start_has_cache_tokens(fake_engine):
    """The fake engine returns cache_read=20, cache_creation=0. The SSE
    message_start event must surface these instead of hardcoded 0."""
    payload = {
        "model": "qwen",
        "max_tokens": 8,
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": "hello"}]},
        ],
        "stream": True,
    }
    req = _StubRequest(payload)

    async def _drive():
        response = await server_mod.handle_messages(req)
        chunks: list[bytes] = []
        async for chunk in response.body_iterator:
            chunks.append(
                chunk if isinstance(chunk, bytes) else chunk.encode("utf-8")
            )
        return response, b"".join(chunks)

    response, body = asyncio.run(_drive())

    assert response.status_code == 200

    body_str = body.decode("utf-8")
    # Find the message_start event and parse its data
    assert "event: message_start" in body_str, (
        "J-36 GATE3: message_start event not found in SSE stream"
    )
    # Extract the JSON data line after message_start
    lines = body_str.split("\n")
    for i, line in enumerate(lines):
        if line.strip() == "event: message_start":
            data_line = lines[i + 1]
            assert data_line.startswith("data: "), (
                f"J-36 GATE3: expected data: line after message_start, got: {data_line!r}"
            )
            data = orjson.loads(data_line[len("data: "):])
            usage = data["message"]["usage"]
            assert usage["cache_read_input_tokens"] == 20, (
                f"J-36 GATE3: cache_read_input_tokens should be 20, "
                f"got {usage['cache_read_input_tokens']}"
            )
            assert "cache_creation_input_tokens" in usage, (
                "J-36 GATE3: cache_creation_input_tokens missing from usage"
            )
            break
    else:
        pytest.fail("J-36 GATE3: message_start event not found in SSE stream")
