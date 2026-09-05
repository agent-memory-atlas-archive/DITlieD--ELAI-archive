# I-3: SSB throughput + atomic checkpoint saves — harden the write path for search-era rates

- Track 12 — FAULT-ISOLATION + ENFORCEMENT-PLANE | lever PERF | grounding STRONG | nature EXTENSION | effort S | WAVE 1
- Premise: the SSB is the single append-only audit log every other I-plan session emits rows to; under
  search-era write rates (I-4 fans to K candidates, each emitting multiple rows) the current pragma
  configuration and checkpoint write sites have not been hardened against concurrent writers or mid-write
  crashes. Phase 3B specifies pragmas; Phase 3C specifies atomic checkpoint writes; some sites may already
  be atomic — the real gap is closing the rest and measuring insert throughput with a real benchmark
  (no paper number claimed; ID-5 baseline-before-paper-threshold applies).

## Steal/build
Harden the single-writer SQLite path for high-frequency appends: confirm/extend the Phase 3B pragma block
(WAL mode, busy_timeout, cache_size, wal_autocheckpoint) and close any checkpoint write sites not yet using
the .tmp+fsync+rename atomic-write pattern. Every checkpoint write is: write to <path>.tmp, fsync, rename
atomically — the POSIX rename guarantee means a reader sees either the old complete file or the new complete
file, never a torn one. Measure actual insert throughput into metrics/phase_I3.toml from a real seeded run
(not a synthetic microbenchmark; the real SSB write loop under fixture load). io_uring is a Linux-only
opt-in experiment: allowlisted in wire-dark-allowlist.toml, behind ELAI_IO_URING_ENABLE, never the default
path (W-2 compliant). No paper throughput number is carried as a target; the TOML records the measured
number and that number becomes the ID-5 baseline for any future claim.
Declined: switching from SQLite to a different storage engine (scope explosion; append-only SQLite +WAL is
the established substrate per IS-1); async batched writes to the SSB (breaks the append-only audit guarantee
and the action_type_lock golden-lock discipline).

## Existing ground truth
- IS-1: ssb.rs golden-locked via action_type_lock.rs (append-only discipline ssb.rs:5). SCHEMA_VERSION = 221,
  TOTAL_VARIANTS = 532, anchors at ssb.rs:2981 and action_type_lock.rs:601. Live at grep time 2026-06-10.
- Phase 3B SSB pragma spec: PLAN-asserted (referenced by I-0 and the grounding map); executor re-greps
  the pragma block in ssb.rs at EXECUTING (W-7).
- Phase 3C checkpoint atomic-write spec: PLAN-asserted; some sites may already be atomic; executor audits
  all checkpoint write sites in orchestrator/ and session_dag.rs per W-7.
- H-44 durability: PLANNED (PLAN-asserted; executor re-greps per W-7) — the durability concern H-44
  addresses composes with the atomic-write pattern here; do not re-implement H-44's scope.
- ssb.rs write loop and checkpoint write sites: live source (PLAN-asserted; executor reads the write-loop
  section and all checkpoint call sites at EXECUTING).
- No io_uring integration: absent (confirmed by IS-19 tooling list; io_uring not in the crate graph).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- Audit and extend pragma block in ssb.rs (WAL mode, busy_timeout, cache_size, wal_autocheckpoint) to match
  Phase 3B spec exactly; cargo check confirms no regression
- Close all checkpoint write sites in crates/orchestrator/ and session_dag.rs with .tmp+fsync+rename pattern
- Measure and record insert throughput in metrics/phase_I3.toml from a seeded fixture run (ELAI_SEED)
  recomputable under GD-2
- Kill-9 crash fixture: assert torn-file is structurally impossible (old or new, never partial)
- NO new action_type: reuse existing SSB infrastructure (I-0 table entry: "NO new action_type — reuse SSB
  append"; no SSB schema delta for I-3)

DEFER (Tier-2 / gated):
- io_uring Linux-only opt-in: behind ELAI_IO_URING_ENABLE, allowlisted in wire-dark-allowlist.toml with
  promotion plan; never the default; separate gate before enabling
- Concurrent multi-writer fanout hardening: sequenced after I-4 search-ladder ships and actual contention
  is measured; decide at full-form expansion

## Integration
- MODIFY: crates/orchestrator/src/ssb.rs — pragma block extension + write-loop review (~30 LOC delta)
- MODIFY: crates/orchestrator/src/session_dag.rs — checkpoint write sites converted to .tmp+fsync+rename
  where not already atomic (~20-40 LOC delta; PLAN-asserted; executor confirms exact sites per W-7)
- MODIFY: crates/orchestrator/src/ (any other checkpoint write sites found at EXECUTING) — same pattern
- CREATE: metrics/phase_I3.toml — measured throughput record from seeded run
- CREATE: workflow/reasoning/I-3-decisions.md
- frontend_pair: N/A — throughput lands in metrics/phase_I3.toml; no live dashboard surface added in this
  session. W-6 justified: the SSB is internal infrastructure; the metrics TOML is the observable artifact
  and is consumed by future cockpit tiles (I-27) without a new panel. Justified.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/ssb.rs pragma block (extended/confirmed against Phase 3B spec)
  - .tmp+fsync+rename atomic write pattern at all checkpoint sites in orchestrator/ + session_dag.rs
  - metrics/phase_I3.toml (measured insert throughput, recomputable under ELAI_SEED)
  - NO new SSB action_type variant for I-3 — reuse existing append infrastructure (I-0 table)
consumes_symbols:
  - IS-1 ssb.rs append-only discipline ssb.rs:5 (why: the pragma extension and write-loop harden the same
    file; append-only golden-lock must not regress)
  - Phase 3B SSB pragma spec (why: confirms and closes the spec; do NOT re-specify pragmas from scratch)
  - Phase 3C checkpoint atomic-write spec (why: closes remaining sites; do NOT re-implement spec from scratch)
  - H-44 durability (why: compose the durability concern; do NOT re-implement H-44's checkpoint scope)
  - action_type_lock.rs:601 ALL_VARIANTS (why: any ssb.rs edit re-reads this to confirm no new variant
    introduced; re-read at edit per W-9)
reachable_from:
  - crates/orchestrator/src/ssb.rs write loop (called by every SSB-emitting session in production) —
    the pragma block and atomic writes are exercised by any real elai run that emits SSB rows  [PRODUCTION SSB write path]
  - crates/orchestrator/src/session_dag.rs checkpoint call -> atomic write  [PRODUCTION checkpoint]
  (a #[cfg(test)] / contracts/ caller NEVER satisfies this — R7)
stub_ban:
  - NO synthetic microbenchmark numbers cited as throughput (real seeded run, ELAI_SEED — ID-5)
  - NO new action_type variant for I-3 (pragma/write hardening has no schema delta)
  - NO io_uring on the default path (Linux-only opt-in, wire-dark-allowlist.toml row required)
  - NO .unwrap()/.expect() on production paths in ssb.rs or session_dag.rs
  - NO blake2/sha2 (blake3 only per CLAUDE.md; ssb write keying stays blake3)
  - NO abandonment of the append-only golden-lock discipline (ssb.rs:5 invariant must hold)
  - NO fabricated file:line numbers beyond what the executor re-greps per W-7
frontend_pair:
  N/A — throughput lands in metrics/phase_I3.toml; no new dashboard surface. The pragma and atomic-write
  changes are internal write-path hardening; the observable artifact is the TOML record + the kill-9 fixture
  result. W-6 justified: no live signal to surface until I-27 cockpit tile consumes the metrics. Justified.
observable_effect:
  trigger: "elai run --task <ssb-write-fixture> --seed <ELAI_SEED>; kill -9 <pid> mid-checkpoint on
    the crash fixture; python -c \"import tomllib; d=tomllib.load(open('metrics/phase_I3.toml','rb')); assert d['insert_throughput_per_sec'] > 0\""
  expect_ssb: "existing SSB rows emit normally on the fixture run (no regression to existing append behavior);
    NO new I-3-specific variant emitted (I-0 table: reuse only)"
  expect_artifact: "metrics/phase_I3.toml written with a measured insert_throughput_per_sec value recomputable
    under ELAI_SEED (GD-2); kill-9 crash fixture leaves either the old complete checkpoint or the new complete
    checkpoint, NEVER a torn file (verified by comparing file length + blake3 hash against pre-crash snapshot);
    the negative arm — writing without .tmp+fsync+rename — produces a torn file on the crash fixture and FAILS (R2)"

## Grounding
- IS-1: ssb.rs:2981 (SCHEMA_VERSION=221) + action_type_lock.rs:601 (TOTAL_VARIANTS=532) — live grep-confirmed.
  Phase 3B pragma spec + Phase 3C atomic-write spec: PLAN-asserted; executor re-greps at EXECUTING (W-7).
  H-44 durability: PLAN-asserted; executor re-greps per W-7.
- Consolidated draft Part I §SSB hardening section. Infrastructure dossier (atomic write pattern context).
  All arXiv IDs from source dossiers: WebSearch-confirm before headline (map citation caveat).
  No paper throughput number is a target (ID-5 baseline-before-paper-threshold applies; the TOML record
  IS the baseline).
- SSB: NO new action_type variant for I-3 — baseline unchanged at LIVE 221/532 as of 2026-06-10
  (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- IS-1 ssb.rs append-only discipline: the write loop being hardened. Do NOT change the append-only model.
- Phase 3B SSB pragma spec: confirmed and extended. Do NOT re-specify pragmas from scratch.
- Phase 3C checkpoint atomic-write spec: remaining sites closed. Do NOT re-implement from scratch.
- H-44 durability: checkpoint durability concern. Do NOT re-implement H-44's scope.
- I-1 (Supervisor): the Supervisor dispatches tool calls that emit SSB rows; I-3's write-path hardening
  is a prerequisite for I-1's DirectiveExecuted rows to land durably. Wave sequencing: I-3 is WAVE 1,
  I-1 is WAVE 2.
- I-4 search ladder: relies on high-throughput SSB appends when fanning to K candidates; I-3 is the
  write-path foundation. I-3 WAVE 1 precedes I-4 WAVE 4.

End of I-3.
