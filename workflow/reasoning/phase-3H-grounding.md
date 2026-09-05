Phase 3H Grounding Results
Date: 2026-04-06
Session: s-476332

SPEC VERIFICATION
  docs/specs/crates/orchestrator.txt Section 10 | EXISTS | L281-325 (SessionDag, trimming, branch, handoff)
  docs/specs/architecture-overview.txt Section 5 | EXISTS | L190-194 (SessionDag cross-cutting mention)
  docs/specs/crates/orchestrator.txt Section 3 | EXISTS | L45-73 (SSB schema for stub recovery)
  docs/specs/crates/orchestrator.txt Section 6 | EXISTS | L137-152 (context budget slots)
  workflow/papers/cmv-interface.md | EXISTS | CMV paper interface map with invariants and failure modes

CLAIM GROUNDING
  "snapshots schema columns" | GROUNDED | orchestrator.txt L289-292 (exact column list)
  "snapshot_context schema" | DIVERGES | orchestrator.txt L293-294 specifies BLOB design; phase proposes per-message rows. See SPEC AMENDMENTS #1.
  "session_dag.db per session" | GROUNDED | orchestrator.txt L287
  "3 streaming passes" | GROUNDED | orchestrator.txt L300-304, CMV paper complexity O(3*N)
  "stub_threshold default 500" | GROUNDED | orchestrator.txt L301, CMV hyperparameters range [100,1000]
  "structurally lossless" | GROUNDED | orchestrator.txt L630 invariant
  "every stub references valid SSB seq" | GROUNDED | orchestrator.txt L631 invariant
  "max 3 active branches" | GROUNDED | orchestrator.txt L314, L632
  "pick winning branch" | GROUNDED | orchestrator.txt L312
  "SSB append-only" | GROUNDED | orchestrator.txt L47
  "branch tag format" | GROUNDED | orchestrator.txt L310 (tag="branch:{name}")
  "handoff tag" | GROUNDED | orchestrator.txt L317, L633
  "load branch tip not trunk" | GROUNDED | orchestrator.txt L319
  "no summarization" | GROUNDED | orchestrator.txt L320
  "commit_hashes per session" | GROUNDED | orchestrator.txt L324
  "prompt cache prefix Slots 1-3" | GROUNDED | orchestrator.txt L635, context budget L143
  "EntropyProvider for UUID" | GROUNDED | implementation-guide.txt L46 (entropy quarantined)
  "SQLite FK off by default" | VERIFIED | SQLite documentation (PRAGMA foreign_keys defaults OFF)
  "SQLite batch insert ~50K/sec" | VERIFIED | SQLite documentation (within single transaction)
  "Kahn's algorithm O(V+E)" | VERIFIED | standard graph algorithm (used in phase 3G)
  "blake3 for hashing stubs" | UNGROUNDED | orchestrator.txt does not specify hash algorithm. Assumed: blake3 is fast (3.5 GB/s), available in Rust ecosystem. Alternative: sha256.
  "file conflict detection on merge" | UNGROUNDED | orchestrator.txt does not specify merge conflict detection. Assumed: two branches modifying same file is ambiguous. Safe default.
  "SSB pruning" | VERIFIED | grep for "prune" in orchestrator.txt returns 0 hits. Not currently specified. Flagged as future risk for stub recovery.
  "WAL not needed for session_dag" | VERIFIED | single writer per session (orchestrator.txt L287), no concurrent readers needed unlike SSB.

IMPROVEMENTS MADE (beyond grounding/gating)
  1. snapshot_context schema expanded: added seq (ordering), is_stubbed, stub_ssb_seq columns.
     Rationale: original schema had only role + trimmed_messages BLOB. Per-message rows enable
     selective stub recovery and ordered reconstruction. BLOB approach would require deserializing
     entire context to recover one stub.
  2. Stub struct defined: original_type, original_tokens, summary_hash, ssb_seq.
     Rationale: orchestrator.txt L302 says "type, size, summary hash" but no struct. Making it
     explicit ensures implementer doesn't invent ad-hoc format.
  3. MergeResult enum with Conflict variant for file overlap detection.
     Rationale: orchestrator.txt says "pick winning branch" but doesn't address the case where
     branches modify the same file. Silent winner selection risks data loss.
  4. TrimConfig struct with explicit target_tokens derivation from context budget.
     Rationale: "trimmed context fits within model context window" (L634) but no formula given.
     target = model_window - allocated_slots - output_reserve is the only consistent derivation.
  5. Branch naming validation (alphanumeric + hyphens, max 64 chars).
     Rationale: tag="branch:{name}" goes into TEXT column. Unrestricted names could inject
     SQL-like patterns or break tag parsing.
  6. stub_threshold minimum enforced at 100 (from CMV hyperparameter range).
     Rationale: prevents degenerate case where everything gets stubbed.

UNGROUNDED ASSUMPTIONS (2)
  1. blake3 hash algorithm for stub integrity. Not specified in spec. Low risk — any fast hash works.
     Note: sha256 via sha2 crate may be preferred (no new dependency). Or skip hash entirely
     and use SSB seq as sole integrity anchor.
  2. File conflict detection on branch merge. Not specified in spec. Medium risk — may over-engineer
     if branches never touch same files in practice. Marked optional in phase file pending spec clarification.

ADVERSARIAL AUDIT (2026-04-06, session s-476332)
  Auditor found: 3 BLOCK, 6 WARN, 4 INFO.
  All 3 BLOCKs resolved:
    BLOCK 1: snapshot_context schema false "matches exactly" claim -> fixed. Now explicitly
      documents divergence and requires spec amendment before implementation.
    BLOCK 2: branch_abandon action_type not in SSB CHECK -> fixed. Now uses 'checkpoint'
      action_type or requires spec amendment. Documented in SPEC AMENDMENTS #2.
    BLOCK 3: SSB cleanup ambiguity -> fixed. Documented as SPEC AMENDMENTS #3 with
      default assumption (logical exclusion, not physical deletion).
  WARNs addressed:
    - CMV failure mode #1 (compaction boundary): added as R-3H-6 + test in 3H.6
    - MergeResult::Conflict: marked optional in phase, depends on spec resolution
    - allocated_slots_tokens coupling: added as R-3H-8 + PRE_GATE check in 3H.3
    - Per-message schema docs: ACCEPTANCE block updated to not claim "matches exactly"
    - Session file corruption: added as R-3H-7 + integrity_check in 3H.1
    - Merge reparenting: clarified in 3H.4 TASK (create new snapshot, don't mutate parent_id)
  INFOs noted (no action):
    - blake3 dependency concern (already flagged UNGROUNDED)
    - All 6 improvements confirmed beneficial by auditor
    - "Only winning branch SSB entries kept" spec ambiguity documented
    - EntropyProvider seed injection for tests (verify in Phase 3A)
