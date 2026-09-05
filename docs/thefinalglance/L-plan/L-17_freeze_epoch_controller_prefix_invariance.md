L-17 — FREEZE-EPOCH CONTROLLER (prefix-invariance as an explicit recompute-cause state machine)
==============================================================================================
Track: L-T3 CONTEXT-CORE. Binds CC-2, CC-3.
Nature: BUILD (the concrete implementation of the prefix-invariance + cache-hit hard guard).
Lever: turn "the prefix should stay stable" from an assertion into a state machine whose only legal prefix
  changes carry a named, journaled cause, so a cache-cold event is always explained and auditable.
Wave: W2 | depends L-16.
Composes-with: L-16 ContextProjection (the fold it freezes), I-10 cache_economics.rs (prefix invariance + TTL
  tiering + the cache-hit hard guard), H-50 ContextPolicy (the TTL/late-compaction knobs).
Authored 2026-06-30. Reference: CWD foldFreeze.ts (the frozen view + boundary fingerprint + the exhaustive
  recompute-cause discriminated union + the append-tail-without-recompute optimization).

BUILD
=====
A FreezeState holding: the frozen view, its char/token count, the raw-log length it was computed from, a
boundary fingerprint (role + length + a fast non-cryptographic hash of the boundary message, CWD uses FNV-1a;
ELAI reuses its xxhash/FxHash prefix hash from 3E for one hash family), and reuse/epoch counters. A pure
per-turn decision evaluate_freeze(state, log, budget, policy) -> Reuse | Recompute(cause) over an ENUMERATED
cause set (CC-3): FirstCall, ColdGapTtl (idle gap > policy TTL), TailOverflow (raw tail since the boundary
exceeds the tail cap), BoundaryMismatch (the boundary message hash diverged), HistoryRewound (the log shrank —
impossible under CC-1 in normal flow, so this is a corruption tripwire), PolicyChanged (BackendMode or budget
strategy changed), PressureCeiling (measured input tokens hit the ceiling -> hand to the L-16 BeliefState
projection variant). On Reuse the controller returns the frozen view prefix + the raw tail appended verbatim
(prefix bytes unchanged -> cache stays hot). The append-tail-without-recompute optimization: when only the tail
grew, re-fold ONLY the tail and extend the frozen prefix, never recompute the existing prefix. Every Recompute
journals its cause (CC-3).

COMPOSES-WITH (do NOT re-author)
================================
I-10 cache_economics.rs: this IS the concrete prefix-invariance guard I-10 specifies; the cache-hit hard guard
becomes "the realized cache_read covered the frozen prefix on a Reuse turn, or the miss is explained by a
journaled cause". H-50: the TTL and late-vs-early compaction come from ContextPolicy (cloud long TTL, local
short / no freeze where there is no provider cache). L-16: evaluate_freeze is one of fold's inputs.

INTEGRATION
===========
crates/orchestrator/src/context_projection.rs (FreezeState + evaluate_freeze alongside fold), consuming
cache_economics.rs. SSB: prefer-reuse I-10's CacheEconomicsEvent (TtlPlanned + a CacheMiss/PrefixRecomputed
variant carrying the enumerated cause); decide net 0..+1 at edit (re-grep SCHEMA_VERSION first). frontend_pair:
the context gauge shows the last epoch cause; or justified N/A.

OBSERVABLE_EFFECT (done-gate)
=============================
A two-turn cloud run with no boundary change returns Reuse and the journal shows the prefix bytes unchanged +
turn-2 cache_read covering the frozen prefix. A turn that crosses the TTL returns Recompute(ColdGapTtl) and the
cause is in the journal. A deliberately injected prefix-byte change with NO recompute cause is a hard fail
(CC-3). NEGATIVE: a tail-only growth re-folds only the tail and does NOT move the prefix (append-tail
optimization — proven by the unchanged prefix hash); a cache miss is never reported as a hit (I-10 hard guard,
the dead-evidence path screams).
