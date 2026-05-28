"""E-2 Task 9 offline contract test for prefix-KV determinism invariant.

This is the OFFLINE companion to test_prefix_kv_determinism.py (which
RUNPOD-runs against a live model). The live harness verifies byte-equivalent
generation; this harness verifies the CACHE KEY INVARIANT that makes
byte-equivalent generation possible without loading the model:

   determinism invariant:
     for any token-id tensor X
        _hash_input_ids(X) == _hash_input_ids(X)   (pure-of-content hash)
     and
        PrefixKVCache.lease(X) after admit(X, K) returns the SAME object K
        across N invocations on the same cache instance (key-stability +
        LRU recency bookkeeping does not mutate the cache reference).

If both hold, the live byte-equivalent property is by-construction: the
HF DynamicCache passed back is the same Python object on every lease,
so the prefill arithmetic over its tensors is identical. The live harness
on RUNPOD adds the empirical confirmation; this harness locks the code
invariant so any future PrefixKVCache refactor cannot silently violate
determinism.

Run via:
   cd .elai_cc/python && \
     python -m pytest elai_inference/tests/test_prefix_kv_determinism_offline.py -v

Exit 0 = invariants hold. Test failure = determinism contract violated.
CPU-only; no model load required.
"""

from __future__ import annotations

import torch

from elai_inference.server import (
    PrefixKVCache,
    _KV_MIN_PREFIX_TOKENS,
    _hash_input_ids,
)


class _OpaqueCache:
    """Stand-in for HF DynamicCache. PrefixKVCache treats the cache as opaque;
    the determinism contract is that the Python identity is preserved across
    lease() calls until evict()."""

    def __init__(self, label: str) -> None:
        self.label = label

    def __repr__(self) -> str:
        return f"_OpaqueCache({self.label!r})"


def _make_ids(start: int, length: int) -> "torch.LongTensor":
    return torch.tensor(list(range(start, start + length)), dtype=torch.long)


def test_hash_is_pure_of_content_across_repeated_calls():
    """_hash_input_ids(X) is deterministic across repeated invocations on
    the same content. This is the key-stability axiom — without it, two
    identical prefixes would map to different keys and the cache would
    silently miss instead of reusing prior KV state."""
    ids = _make_ids(0, _KV_MIN_PREFIX_TOKENS + 4)
    h_first = _hash_input_ids(ids)
    h_again = _hash_input_ids(ids)
    h_clone = _hash_input_ids(ids.clone())
    assert h_first == h_again
    assert h_first == h_clone


def test_hash_distinguishes_different_content():
    """Distinct token-id sequences must hash to distinct keys. Otherwise
    two semantically different prefixes would collide and the cache would
    return KV state computed against a different prefix (non-deterministic
    generation)."""
    a = _make_ids(0, _KV_MIN_PREFIX_TOKENS + 4)
    b = _make_ids(0, _KV_MIN_PREFIX_TOKENS + 4)
    b[3] = 99999  # diverge mid-prefix
    assert _hash_input_ids(a) != _hash_input_ids(b)


def test_lease_returns_same_object_on_consecutive_calls():
    """Determinism invariant: lease(X) after admit(X, K) returns K on
    every consecutive call. If this fails, generation over the leased
    cache would diverge across runs because the underlying KV tensors
    would not be the same Python object."""
    cache = PrefixKVCache(max_entries=4)
    ids = _make_ids(0, _KV_MIN_PREFIX_TOKENS + 5)
    handle = _OpaqueCache("stable-handle")
    cache.admit(ids, handle)

    leased_first, _, _ = cache.lease(ids)
    leased_second, _, _ = cache.lease(ids)
    leased_third, _, _ = cache.lease(ids)

    assert leased_first is handle
    assert leased_second is handle
    assert leased_third is handle
    assert leased_first is leased_second is leased_third


def test_lease_returns_same_object_after_lru_recency_bump():
    """LRU bookkeeping (move_to_end on lease) MUST NOT change the cache
    handle returned. The recency bump is metadata-only; the object identity
    is the determinism contract."""
    cache = PrefixKVCache(max_entries=4)
    ids_a = _make_ids(0, _KV_MIN_PREFIX_TOKENS + 5)
    ids_b = _make_ids(100, _KV_MIN_PREFIX_TOKENS + 5)
    handle_a = _OpaqueCache("a")
    handle_b = _OpaqueCache("b")
    cache.admit(ids_a, handle_a)
    cache.admit(ids_b, handle_b)

    # Interleaved leases; both must keep returning the same handle.
    for _ in range(5):
        leased_a, _, _ = cache.lease(ids_a)
        leased_b, _, _ = cache.lease(ids_b)
        assert leased_a is handle_a
        assert leased_b is handle_b


def test_admit_replacement_preserves_key_stability():
    """admit(X, K2) on an existing key replaces in-place. The key (which
    drives lease determinism) must be stable across replacement so a
    subsequent lease(X) returns K2 deterministically (not K1, not None)."""
    cache = PrefixKVCache(max_entries=4)
    ids = _make_ids(0, _KV_MIN_PREFIX_TOKENS + 5)
    cache.admit(ids, _OpaqueCache("v1"))
    cache.admit(ids, _OpaqueCache("v2"))
    leased, _, _ = cache.lease(ids)
    assert leased is not None
    assert leased.label == "v2"
    # Re-lease to confirm key stability after replacement.
    leased_again, _, _ = cache.lease(ids)
    assert leased_again is leased


def test_no_match_returns_none_deterministically():
    """When no admitted prefix matches, lease must return None consistently.
    Non-determinism here would surface as a flaky cache-miss that produces
    fresh KV one run and uses some random previous cache the next."""
    cache = PrefixKVCache(max_entries=4)
    a = _make_ids(0, _KV_MIN_PREFIX_TOKENS + 5)
    b = _make_ids(500, _KV_MIN_PREFIX_TOKENS + 5)
    cache.admit(a, _OpaqueCache("a"))
    # b shares no prefix with a (different start range)
    for _ in range(3):
        leased, prefix_len, _ = cache.lease(b)
        assert leased is None
        assert prefix_len == 0
