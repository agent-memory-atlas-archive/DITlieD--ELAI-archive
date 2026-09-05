# Phase 2 decision doc — server.py cluster (D1 + D2 + D3)

Session: s-46448
Phase: Phase 2 (PART7 §5.A-4 + §5.A-5 + §5.A-3)
Date: 2026-05-07
GRC topic: phase2-server-cluster (6 calls / 1200 chars)

## What I will implement

D1 (§5.A-4) — ruff F821 fix at server.py L660. The unresolved name is in a quoted forward-ref type annotation `"DynamicCache | _KVQuantCache"` on `make_cache`. Add `DynamicCache` to the module-level `from transformers import (...)` block at L39-45. The lazy `from transformers import DynamicCache as DC` import at L663 stays — it's the runtime usage; the module-level addition only satisfies static lint and IDE resolvers.

D2 (§5.A-5) — `POST /v1/cache/flush` endpoint. Implementation strategy: modify `PrefixKVCache.clear()` from `-> None` to `-> int` returning the pre-clear `len(_lru)` snapshot, count taken inside the same lock as the clear (TOCTOU-safe). No existing callers of `clear()` (verified via grep on `_prefix_kv_cache.clear|prefix_kv_cache.clear` — 0 hits), so the signature change is safe. Endpoint handler calls `_prefix_kv_cache.clear() if _prefix_kv_cache is not None else 0` and returns `OrjsonResponse({"flushed": <bool>, "n_entries_cleared": <int>})`. Route placement: after the existing `@app.post("/v1/steering/compute")` block, before `/v1/messages`.

D3 (§5.A-3) — backward-compat streaming gate. Already written in TESTING_FIRST. Test gates on `live_sidecar_ready` (NOT `lmfe_required_ready`) because no response_format means LMFE is irrelevant. Test runs whenever the sidecar is reachable.

## Key technical decisions

1. **`clear()` returns int (vs adding `flush_and_count()` method)**: chose direct return-type change because `clear()` has zero existing callers; the new return value is semantically meaningful (number of entries actually cleared); avoids adding a near-duplicate public method. `flush_and_count()` would be additive but redundant.

2. **Encapsulation respected**: the endpoint calls `clear()` not `_lru` directly. The TOCTOU-safe count happens inside the class, behind the same `self._lock` already used by `clear()`. No private-member access from the route handler.

3. **`torch.cuda.empty_cache()` stays in `clear()`**: existing behavior preserved; the cuda-free is part of the "flush" semantics and operators expect VRAM to be reclaimed.

4. **None guard**: `_prefix_kv_cache is None` when TQ is active or `max_entries=0`. Endpoint returns `{"flushed": false, "n_entries_cleared": 0}` with status 200, NOT 503 — operators must be able to call this blindly per `cache_flush_contract.rs::cache_flush_returns_200_with_shape_contract` rationale.

## Expected gate outcomes

- G1 (D1 PRE): ruff F821 DynamicCache → 1 hit before edit (auditor verifies retroactively).
- G2 (D1 POST): ruff F821 DynamicCache → 0 hits after edit.
- G3 (D2 PRE): grep `prefix_cache|_prefix_kv_cache` server.py → ≥1 hit (verified — many hits at L1186, L1201, L1216, L1330, etc.).
- G4 (D2 POST): grep `/v1/cache/flush` server.py → ≥1 hit (route registered).
- G5 (D2 POST): python -m py_compile server.py → exit 0.
- G6 (D3 POST): grep `stream_with_no_response_format_unchanged_sse` research_decompose_constrained.rs → 1 hit.
- G7 (PHASE POST): cargo check -p elai_cc_contracts --tests → clean for the touched files; pre-existing PART6 §1.3 HashMap/BTreeMap failures in unrelated tests are INFO not BLOCK.

## Unknowns to verify empirically

- Whether ruff is installed locally (likely yes; if not, fallback is `python -c "import ast; ast.parse(open('...').read())"` syntax check).
- Whether the new `cache_flush_contract.rs` is auto-discovered by Cargo without a Cargo.toml change — context-compiler confirmed YES (integration tests in `tests/` are auto-discovered).
- Live verification of D2 endpoint behavior — DEFERRED to next session per PART7 §5.B (sidecar offline this session).
- Live verification of D3 streaming gate — DEFERRED to next session per PART7 §5.B.

## Constraints (this session)

NO COMMITS. 1 audit cycle max (covers all 3 deliverables). Sidecar OFFLINE → audit limited to compile/grep/ruff/cargo-check (no live SSE/curl). Project boundary: `.elai_cc/` only.
