# J-69: SSB log-integrity hash chain + watermark monotonicity — a tampered or reordered journal fails an O(n) scan before replay

- Track J (NASA-gates wave) | lever PROVABLE | grounding STRONG (raft-interface.md W12 Log-Matching + Leader-Completeness) / CONFIRMATION (I-18/J-22 are the right design) | nature EXTENSION(I-18 + J-22) | effort M | WAVE NASA-GATES-2 | depends none (extends the I-18 chain producer + J-22 writer isolation)
- W12 is INDEPENDENT confirmation that I-18 + J-22 are correct; J-69 is the consumer SCAN-GATE + the watermark rider, NOT a competing chain (NG-2).

- Premise: ELAI's SSB is already a single-writer, append-only, RRA-replayable WAL (= Raft's single-sequencer + Leader-Append-Only + State-Machine-Safety, per raft-interface.md W12). The one NET-NEW gap W12 surfaces: SSB rows have NO prev_hash, so a tampered/reordered/truncated journal is only caught (if at all) during the O(n) RRA replay, not before it. GROUNDED: this is EXACTLY planned I-18 (per-row chain_prev_hash + 5-layer-stack hash + offline blake3-Merkle) + J-22 (writer-isolation contract, sole-commit-point). J-69 does NOT author a competing chain (plan GROUP F is explicit). J-69 ships the two CONSUMER legs I-18/J-22 imply but do not themselves gate: (1) an O(n) chain-SCAN gate that runs at session-open and in RRA replay --verify, catching tamper/reorder BEFORE replay; (2) a watermark-monotonicity check (a new session must start at >= the prior committed watermark — Raft's Leader-Completeness analog). blake3, not sha256 (CLAUDE.md anti-stale).

## Steal/build
1. CHAIN-SCAN GATE: an O(n) scan over the SSB rows verifying prev_hash = blake3(canonical_prev_row_bytes) for every row (the I-18 chain field is the producer; J-69 is the verifier). A broken link = BLOCK at session-open and at `rra ... --verify`, BEFORE the O(n) replay. Reuses I-18's chain field + J-22's sole-commit-point; re-implements neither.
2. WATERMARK-MONOTONICITY GATE: a new session's first row watermark must be >= the prior committed watermark (Leader-Completeness analog); a regression = BLOCK (a rolled-back / forked / replayed-stale journal). Folds in as the I-18 rider the plan names.
3. the canonical-bytes function (the exact serialization the hash covers) is shared with J-68's serializer round-trip harness — J-68 PROVES the canonical form is deterministic, J-69 DEPENDS on it. blake3 keyed per the I-18 algorithm-prefix convention.
EXCLUDED: a NEW chain producer (I-18 ships chain_prev_hash — re-implementing it = GD-1 P0, plan GROUP F); a NEW writer or commit point (J-22 owns sole-commit; J-69 is read-only on the journal); sha256/blake2 (anti-stale, blake3 only); auto-repair of a broken chain (surface + BLOCK, never silently rewrite history — RV-4 honest-irreversibility).

## Existing ground truth (PLAN-asserted unless noted; re-grep at edit, W-7/W-9)
- I-18 UPIP per-row chain_prev_hash + 5-layer stack hash + offline blake3-Merkle default (PLANNED) — the chain PRODUCER J-69 verifies. PLAN-asserted; re-grep.
- J-22 journal-payload schema lock + writer-isolation contract + sole-commit-point (PLANNED) — the single-writer guarantee the scan trusts. PLAN-asserted.
- SSB single-writer append-only stream: ssb.rs:62 (the action_type array), :1503/:1508 (the derived SCHEMA) — grep-confirmed 2026-06-22; the row bytes the chain covers.
- G-2 RRA replay (feature-map; the O(n) replay the O(n) scan runs BEFORE) — `rra ... --verify` is the scan's second entrypoint.
- file_ops.rs:272/300 pre-image capture (SG-6) — the snapshot bytes a chain over file-op rows would cover; PLAN-asserted.
- blake3 (CLAUDE.md anti-stale HARD RULE) — never sha256/blake2 for the chain.

## Integration
- the chain field itself is I-18's deliverable. J-69 adds: a scan_chain fn + a watermark-monotonicity check wired into (a) the session-open path and (b) `rra --verify`. If the chain_prev_hash COLUMN does not yet exist at edit (I-18 unshipped), J-69's first task is the DDL field add — a SCHEMA migration that bumps SCHEMA_VERSION_DDL_EPOCH (ssb.rs:1503), NOT a new action_type (SSB_ACTION_TYPES.len() unchanged); re-read the live epoch at edit (W-9). NO new action_type (the scan emits its verdict through the existing audit/RRA surface, or a single ChainScanVerdict reuse if one exists). 
- frontend_pair: a chain-integrity status line on the existing RRA/review surface (decide at edit, W-4) or a W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/ssb::scan_chain (O(n) prev_hash verification; BLOCK on a broken link)
- crates/orchestrator/src/ssb::check_watermark_monotonicity (new session watermark >= prior committed; Leader-Completeness analog)
- (conditional, only if I-18 unshipped at edit) the chain_prev_hash DDL column + the epoch bump
consumes_symbols:
- I-18 chain_prev_hash + 5-layer-stack hash (the chain PRODUCER; NOT reauthored, GD-1 — plan GROUP F)
- J-22 sole-commit-point + writer-isolation (the single-writer guarantee the scan trusts)
- J-68 canonical-bytes serializer (the deterministic form the hash covers; proven by J-68's round-trip harness)
- G-2 RRA replay (`rra --verify` — the second entrypoint; the scan runs BEFORE the O(n) replay)
reachable_from:
- session-open on a real `elai` run + `elai rra <bundle> --verify` (both drive scan_chain off-agent)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO new chain producer (consume I-18; re-implementing the per-row hash = GD-1 P0, plan GROUP F)
- NO new SSB writer / commit point (J-22 owns it; J-69 is read-only on the journal)
- NO sha256 / blake2 (blake3 only; CLAUDE.md anti-stale)
- NO auto-repair of a broken chain (surface + BLOCK; never rewrite history, RV-4)
- NO "chain absent" treated as pass (a missing chain field at session-open is FAIL — the gate self-tests its evidence path, R2/ND-3)
frontend_pair: chain-integrity status on the RRA/review surface (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "hand-mutate ONE byte of one committed SSB row's payload, then run session-open (and `elai rra <bundle> --verify`) on that journal"
- expect_artifact: "scan_chain reports the broken link at the mutated row's position and BLOCKS before the RRA replay runs; a session whose first watermark is < the prior committed watermark BLOCKS on check_watermark_monotonicity"
- expect_artifact (NEGATIVE, R1/R2): "an untampered journal scans clean and proceeds; a journal with NO chain field FAILS at session-open (never silently passes — the dead evidence path screams); a reordered pair of rows fails the scan (the hash of the wrong predecessor doesn't match), not the replay; the scan pointed at a truncated journal FAILS, never passes a short chain as complete"

## Grounding
- W12 raft-interface.md (Ongaro/Ousterhout 2014: Log-Matching -> the prev_hash chain analog; Leader-Completeness -> the watermark-monotonicity analog; Leader-Append-Only / State-Machine-Safety = ELAI's already-shipped single-writer WAL).
- substrate: I-18 (chain producer, PLANNED) + J-22 (writer isolation, PLANNED) — W12 is independent CONFIRMATION these are the right design (plan GROUP F: "confirms not invents").
- blake3 (CLAUDE.md anti-stale); J-68 proves the canonical-bytes determinism the chain relies on.
- LIVE SSB baseline 268/577 (ssb.rs:62/:1503/:1508, grep 2026-06-22); net 0 new action_type (a DDL field add bumps the epoch only, if I-18 unshipped).

## COMPOSES-WITH
I-18 (the chain producer J-69 verifies — EXTENSION, never a competing chain, NG-2), J-22 (the sole-commit-point the scan trusts), J-68 (the canonical-bytes determinism proof the hash depends on), J-66 (the monitors run on a J-69-chain-verified journal — integrity is their precondition), G-2 RRA (the `--verify` entrypoint + the O(n) replay the O(n) scan precedes), H-49 reversibility (the CAS object store is a separate concern; J-69 verifies the SSB chain, not files). Restates none.

## Open questions / risks
- ORDER OF SHIP vs I-18: J-69 is the consumer; if I-18 has not shipped chain_prev_hash at edit, J-69's first task is the minimal DDL field add (epoch bump, NOT a new action_type) so the scan has something to verify — keep that field add behavior-preserving and re-read the live epoch (W-9). Coordinate with the I-18 author so there is ONE chain producer, not two.
- CANONICAL-BYTES agreement: the scan and the producer MUST hash the identical serialization or every scan fails spuriously — J-68's round-trip harness is the shared proof of that canonical form; pin it before either ships.
- PERFORMANCE: O(n) scan at every session-open on a long journal — bound it (scan from the last verified watermark forward, persisting a verified-watermark, never re-scanning proven history), and LOG the scanned range (no silent partial scan presented as full, R2).

## Next
J-69 makes the journal tamper-evident BEFORE it is read, which is the precondition J-66's temporal monitors and J-67's journaled Insert/Edit both rely on (you cannot trust a verdict computed over an unverified log). It is the W12-confirmed integrity floor under the whole runtime-enforcement half of the wave.

End of J-69.
