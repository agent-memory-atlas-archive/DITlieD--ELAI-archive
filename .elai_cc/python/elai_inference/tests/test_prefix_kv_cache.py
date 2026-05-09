"""Unit tests for PrefixKVCache (Track C-A).

CPU-only. No live sidecar required. Validates LRU semantics, longest-prefix
match, eviction, and auto-disable paths.

Run: cd .elai_cc/python && python -m pytest elai_inference/tests/test_prefix_kv_cache.py -v
"""

import torch

from elai_inference.server import (
    PrefixKVCache,
    init_prefix_kv_cache,
    _KV_MIN_PREFIX_TOKENS,
    _hash_input_ids,
)
import elai_inference.server as server_mod


def _ids(*token_ids):
    return torch.tensor(list(token_ids), dtype=torch.long)


class _FakeCache:
    """Stand-in for DynamicCache in CPU-only tests. PrefixKVCache treats
    the cache as opaque except for `del` on eviction — no methods called."""

    def __init__(self, label: str) -> None:
        self.label = label

    def __repr__(self) -> str:
        return f"_FakeCache({self.label!r})"


def test_lease_returns_none_when_no_entries():
    cache = PrefixKVCache(max_entries=4)
    result, prefix_len, total_len = cache.lease(_ids(1, 2, 3, 4, 5))
    assert result is None
    assert prefix_len == 0
    assert total_len == 5


def test_admit_then_lease_returns_cache():
    cache = PrefixKVCache(max_entries=4)
    long_ids = _ids(*range(_KV_MIN_PREFIX_TOKENS + 10))
    fake = _FakeCache("e1")
    cache.admit(long_ids, fake)

    result, prefix_len, total_len = cache.lease(long_ids)
    assert result is fake
    assert prefix_len == long_ids.shape[0]
    assert total_len == long_ids.shape[0]


def test_lease_picks_longest_match():
    cache = PrefixKVCache(max_entries=4)
    short = _ids(*range(_KV_MIN_PREFIX_TOKENS + 5))
    long = _ids(*range(_KV_MIN_PREFIX_TOKENS + 50))
    cache.admit(short, _FakeCache("short"))
    cache.admit(long, _FakeCache("long"))

    query = _ids(*range(_KV_MIN_PREFIX_TOKENS + 30))
    expected_prefix_len = _KV_MIN_PREFIX_TOKENS + 30
    result, prefix_len, total_len = cache.lease(query)
    assert result is not None
    assert result.label == "long"
    assert prefix_len == expected_prefix_len
    assert total_len == expected_prefix_len


def test_lease_first_mismatch_terminates_match():
    cache = PrefixKVCache(max_entries=4)
    stored = _ids(*range(_KV_MIN_PREFIX_TOKENS + 10))
    cache.admit(stored, _FakeCache("a"))

    diverge = stored.clone()
    diverge[_KV_MIN_PREFIX_TOKENS + 5] = 999
    result, prefix_len, total_len = cache.lease(diverge)
    assert result is not None
    assert prefix_len == _KV_MIN_PREFIX_TOKENS + 5
    assert total_len == diverge.shape[0]


def test_eviction_lru_when_full():
    cache = PrefixKVCache(max_entries=2)
    a = _ids(*range(_KV_MIN_PREFIX_TOKENS + 1))
    b = _ids(*range(100, 100 + _KV_MIN_PREFIX_TOKENS + 1))
    c = _ids(*range(200, 200 + _KV_MIN_PREFIX_TOKENS + 1))

    cache.admit(a, _FakeCache("a"))
    cache.admit(b, _FakeCache("b"))
    cache.admit(c, _FakeCache("c"))

    assert len(cache._lru) == 2
    keys = list(cache._lru.keys())
    assert _hash_input_ids(a) not in keys
    assert _hash_input_ids(b) in keys
    assert _hash_input_ids(c) in keys


def test_lease_bumps_recency():
    cache = PrefixKVCache(max_entries=2)
    a = _ids(*range(_KV_MIN_PREFIX_TOKENS + 1))
    b = _ids(*range(100, 100 + _KV_MIN_PREFIX_TOKENS + 1))
    c = _ids(*range(200, 200 + _KV_MIN_PREFIX_TOKENS + 1))

    cache.admit(a, _FakeCache("a"))
    cache.admit(b, _FakeCache("b"))
    cache.lease(a)
    cache.admit(c, _FakeCache("c"))

    keys = list(cache._lru.keys())
    assert _hash_input_ids(a) in keys
    assert _hash_input_ids(b) not in keys
    assert _hash_input_ids(c) in keys


def test_min_prefix_threshold_misses_short_match():
    cache = PrefixKVCache(max_entries=4)
    short_stored = _ids(*range(_KV_MIN_PREFIX_TOKENS - 1))
    cache.admit(short_stored, _FakeCache("short"))
    result, prefix_len, total_len = cache.lease(short_stored)
    assert result is None
    assert prefix_len == 0


def test_init_disables_when_turboquant_enabled():
    init_prefix_kv_cache(max_entries=4, turboquant_bits=4)
    assert server_mod._prefix_kv_cache is None


def test_init_disables_when_max_entries_zero():
    init_prefix_kv_cache(max_entries=0, turboquant_bits=0)
    assert server_mod._prefix_kv_cache is None


def test_init_enables_with_normal_args():
    init_prefix_kv_cache(max_entries=4, turboquant_bits=0)
    assert server_mod._prefix_kv_cache is not None
    assert server_mod._prefix_kv_cache.max_entries == 4
    init_prefix_kv_cache(max_entries=0, turboquant_bits=0)


def test_clear_drops_all_entries():
    cache = PrefixKVCache(max_entries=4)
    a = _ids(*range(_KV_MIN_PREFIX_TOKENS + 1))
    cache.admit(a, _FakeCache("a"))
    assert len(cache._lru) == 1
    cache.clear()
    assert len(cache._lru) == 0


def test_admit_evicts_stale_lru_entry_with_same_cache_object():
    """Regression: after a lease+mutate cycle, admitting under a new key must
    drop the old key pointing to the SAME (now-mutated) Python cache object.
    Otherwise a future lease on the old prefix would return a cache whose
    get_seq_length() exceeds the matched prefix, corrupting cache_position."""
    cache = PrefixKVCache(max_entries=4)
    turn1_ids = _ids(*range(_KV_MIN_PREFIX_TOKENS + 5))
    shared = _FakeCache("shared")

    cache.admit(turn1_ids, shared)
    leased, _, _ = cache.lease(turn1_ids)
    assert leased is shared

    turn2_ids = _ids(*range(_KV_MIN_PREFIX_TOKENS + 20))
    cache.admit(turn2_ids, shared)

    keys = list(cache._lru.keys())
    assert _hash_input_ids(turn1_ids) not in keys, "stale turn1 key not evicted"
    assert _hash_input_ids(turn2_ids) in keys
    assert len(cache._lru) == 1


def test_admit_replaces_existing_entry_in_place():
    cache = PrefixKVCache(max_entries=4)
    a = _ids(*range(_KV_MIN_PREFIX_TOKENS + 1))
    cache.admit(a, _FakeCache("v1"))
    cache.admit(a, _FakeCache("v2"))
    assert len(cache._lru) == 1
    result, _prefix_len, _total = cache.lease(a)
    assert result is not None
    assert result.label == "v2"
