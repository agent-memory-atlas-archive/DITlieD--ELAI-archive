H-6 DECISION DOC — deterministic observation masking + memory read-back (ACTIVATION)
session s-66424 | 2026-06-04 | orchestrate H-plan WAVE 1 | composes-with E-25 ladder, F-5 memory, B-9 session_dag

WHAT
Three real-but-dark symbols: adaptive_reduce (zero prod callers), recover_stubs (real blake3 SSB recovery, NOT
called — prod substitutes a hardcoded "[stubbed]" placeholder), get_relevant_facts (read-dark). The E-25
compaction ladder is built + production-wired (task_runner.rs:9361) but env-gated default-OFF.

GROUNDED FINDINGS (3 explorers, live-verified; lines drifted in the shared G/H tree)
- LIVE SCHEMA 210/487 (G run advanced it). H-6 adds NO action_type — ContextReduction (ssb.rs:10223) reuse only.
- adaptive_reduce (context_budget.rs:1525, async) emits ContextReduction per level; ZERO prod callers (8 sites all
  in context_budget_contract.rs). The E-25 ladder (compaction_ladder.rs) "rides" the same 3-level model but emits
  a DIFFERENT artifact (ClaudeCodeArchitectureEvent CompactionLadderTierApplied), NOT ContextReduction — the two
  are distinct mechanisms, not one calling the other.
- compaction_ladder_enabled() (compaction_ladder.rs:50) reads ELAI_E25_COMPACTION_LADDER_ENABLE, default false.
  The ladder is ALREADY production-wired at task_runner.rs:9361 — only the env gate is dark. The flag is NOT in
  wire-dark-allowlist.toml (R9 build-break unless promoted-default-ON or registered).
- recover_stubs (session_dag.rs:1507): `(&self, snap_id, ssb: &dyn SsbReader) -> Result<Vec<TrimmedMessage>>` —
  loads the snapshot, recovers each stubbed msg's raw bytes from SSB (blake3-verified), un-stubs in place. The
  prod placeholder is at live_cli.rs:1252 (create_handoff/snapshot branch, INSIDE `if let Some(ssb_ref) =
  self.ssb.as_ref()` :1239 — ssb_ref:&Arc<Ssb> in scope) AND :1276 (the trim_context live-trim branch, a
  DIFFERENT case — no snapshot). HD-2: masking drops TRAILING observations only; trailing-slot reductions leave
  the blake3 breakpoint prefix valid (the cache-preserving edge over LLM re-summarization).
- get_relevant_facts (retrieval.rs:46): read-dark; call site is run_turn (live_cli.rs:~1526). GAPS: no held
  FactStore handle / project_id resolution + the reflect early-return bypasses the injection point — net-new.

KEY DECISIONS
D1 (un-gate the E-25 ladder — the headline): flip compaction_ladder_enabled() to DEFAULT-ON, with
   ELAI_E25_COMPACTION_LADDER_ENABLE="0" as the KILL-SWITCH (env unset => ON; "0" => OFF). This makes it a
   default-ON kill-switch flag (W-2-compliant; R9 "default-OFF flag needs allowlist" no longer applies — AVOIDS a
   second frozen-allowlist re-lock). The ladder is the long-session overflow manager (reduction at PreCompact is
   strictly better than hard truncation), cheapest-first + deterministic. GD-4: the over-reduction rate is a
   Tier-2 measurement; latent test-regression (a test that hits PreCompact without setting the env now gets the
   ladder) is UNVERIFIABLE under cargo-check-only — flagged for the Tier-2 cargo-test pass.
D2 (activate recover_stubs — real recovery): replace the live_cli.rs:1252 load_snapshot+placeholder with
   recover_stubs(&snap.snap_id, ssb_ref.as_ref()). Zip the recovered Vec against a load_snapshot original-flags
   read so ONLY the originally-stubbed indices are overwritten (preserves non-stubbed messages' blocks); set the
   recovered bytes as Text (same flattening the placeholder used, but REAL content). On a recovery Err, fall back
   to the placeholder VISIBLY (eprintln + placeholder) — never a silent gap (R2/W-3).
D3 (DEFER, documented):
   - T1 adaptive_reduce DIRECT caller (to emit ContextReduction rows as the "default manager"): it OVERLAPS the
     now-on ladder (both reduce at PreCompact); adding a redundant direct call risks double-reduction. Clarifying
     adaptive_reduce-vs-ladder ownership + routing the ladder's T1 tier through adaptive_reduce (so ContextReduction
     rows land) is a focused follow-on. The ladder default-ON delivers the masking via CompactionLadderTierApplied.
   - T4 get_relevant_facts: needs FactStore-handle + project_id resolution + the reflect early-return fix
     (net-new). Deferred.
   - the :1276 trim_context (live-trim) placeholder: a different case from the snapshot branch (no snap to recover
     from); left as-is.
D4 (no schema bump): ContextReduction reuse; SCHEMA untouched.

DONE-GATE (Tier-1; live = deferred Tier-2)
crates/contracts/tests/h6_masking_readback_contract.rs (source-wiring): compaction_ladder_enabled defaults ON
(kill-switch on "0"); live_cli.rs calls recover_stubs (no longer load_snapshot+placeholder on the snapshot
branch). cargo check -p elai-orchestrator -p elai-cli -p elai_cc_contracts (SCOPED). Test EXECUTION (no-regression
on the ladder default-flip) + live long-session masking/recovery + the GD-4 over-reduction bound = DEFERRED Tier-2.

RISK
R1 ladder default-flip latent test regression (cargo-check-only invisible) — flagged for Tier-2 cargo test.
R2 recover_stubs zip: TrimmedMessage.content is Vec<u8>; String::from_utf8_lossy is total (no panic). The Err
   path keeps the placeholder visibly.
R3 GD-4 over-reduction at PreCompact — measured Tier-2; the ladder is cheapest-first + only fires on overflow.

DEFERRED: T1 adaptive_reduce-direct/ContextReduction-rows, T4 get_relevant_facts, the :1276 placeholder, the
Tier-2 test+over-reduction measurement, the default-ON promotion verification. NO commit (accumulate to run end).
