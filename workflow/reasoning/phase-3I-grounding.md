Phase 3I Grounding Log
Date: 2026-04-06
Session: s-1921824

SPEC VERIFICATION

  READ docs/specs/crates/orchestrator.txt "DST section" | SPEC_GAP | No DST section exists in orchestrator.txt. DST design is in external-analysis.txt L268-305. Corrected READ directive to reference BSP (section 4) and channels (section 5) instead.
  READ docs/specs/crates/core.txt "Section 4 (ClockProvider, EntropySource)" | VERIFIED with correction | Section 4 is KEY TYPES (L53). Entropy traits are unnumbered subsection at L67-92 between sections 4-5. Corrected to "KEY TYPES section + Deterministic entropy traits subsection".
  READ docs/specs/decisions/external-analysis.txt "DST entries" | GROUNDED | Lines 268-305 cover DST adopted/deferred/not-adopted. Lines 447-461 cover source references. Both exist and match phase description.

CLAIM GROUNDING

  "madsim is drop-in tokio replacement" | VERIFIED: FALSE | Requires cfg(madsim) flag + Cargo.toml dep swaps. v0.2.34, Oct 2025. Source: crates.io, github README.
  "madsim supports 30% packet loss, 500ms jitter, asymmetric disconnects" | VERIFIED | net::Config.packet_loss_rate (f64), send_latency (Range<Duration>), NetSim::clog_link(src,dst). Also buggify_with_prob().
  "madsim seed-based deterministic scheduling" | VERIFIED | Runtime::with_seed_and_config(seed,config). Also Runtime::check_determinism(seed, config, bare_fn_pointer) -- STATIC function, not instance method. No closure captures. Source: madsim/src/sim/runtime/mod.rs.
  "madsim works on Windows" | UNGROUNDED | No platform-specific code, but no Windows CI confirmed. Spike test added as blocking gate (step 3I.1).
  "proptest stateful PBT with auto-shrinking" | VERIFIED | proptest-state-machine v0.8.0 (same repo, released 2026-03-24). ReferenceStateMachine + StateMachineTest traits. 3-phase shrinking. StateMachineTest::apply() returns SUT by value (ownership model). Source: docs.rs/proptest-state-machine.
  "FsmReferenceModel as F(State,Event)->State" | GROUNDED | external-analysis.txt L277.
  "10,000 random operation sequences" | GROUNDED | external-analysis.txt L278.
  "100K ops/sec reference model speed" | UNGROUNDED | No benchmark cited. Pure in-memory assumption. Will measure empirically.
  "Digital sibling methodology" | GROUNDED | external-analysis.txt L275-276, ICSE 2024/2025 + TigerBeetle VOPR.
  "UntrackedValue never checkpointed" | GROUNDED | orchestrator.txt L116-117, L134.
  "EphemeralValue clears recorded in SSB" | GROUNDED | orchestrator.txt L121-122.
  "Channel ordering follows SSB seq" | GROUNDED | orchestrator.txt L115.

IMPROVEMENTS MADE (beyond grounding)

  1. Added RISKS section (3 risks: madsim Windows, cfg flag complexity, proptest-state-machine dependency).
  2. Added madsim Windows spike test as blocking step 3I.1 (was missing entirely).
  3. Restructured SKELETON to use proptest-state-machine traits (ReferenceStateMachine, StateMachineTest) instead of hand-rolled approach.
  4. Added 6 PRE_GATE blocks, 6 DETERMINISTIC_CHECK blocks, 8 ADVERSARIAL tests, 9 ACCEPTANCE criteria.
  5. Fixed "madsim network mock" -> "madsim deterministic runtime" (madsim is a full runtime, not just a mock).
  6. Added madsim check_determinism() API usage in BSP tests.
  7. Specified proptest-regressions/ directory for shrunk test storage + git integration.
  8. Added burst-pattern adversarial test for IPC (uniform jitter misses bursty failures).
  9. Added oracle complexity constraint: reference model LOC < 30% of real FSM LOC.
  10. Added vacuous-truth adversarial test: inject always-true preconditions, verify invariants still catch failures.
  11. Split into 9 numbered steps (was 6 unnumbered sections) for sub-phase resume support.
  12. Added conditional skip for step 3I.6 if madsim unavailable on Windows.

ADVERSARIAL AUDIT (post-improvement)

  Auditor session: a035d2cedcd999e89
  5 BLOCKs found and fixed:
    B1: check_determinism is static fn, not instance method. Fixed API usage pattern throughout.
    B2: StateMachineTest::apply returns SUT by value. Fixed signature description.
    B3: FsmEvent missing BSP-internal events. Added SuperstepBoundary (composite) + AgentCleanup.
    B4: Step 3I.5 PRE_GATE missing entropy quarantine verification. Added grep for raw time/entropy.
    B5: TDD ordering contradiction (3I.4-3I.7 both pass and red). Restructured as TEST-WRITE (red) + IMPLEMENT (green).
  7 WARNs addressed:
    W1: proptest-regressions lazy creation. Added synthetic failure exercise.
    W2: wc -l not valid on Windows. Replaced with python pathlib cross-platform check.
    W3: PROPTEST_CASES=10000 timeout. Added 10 min hard timeout.
    W4: RUSTFLAGS recompile documented. Added R4 risk + compile time note in 3I.6/3I.9.
    W5: send_latency syntax. Fixed to explicit Duration::from_millis() construction.
    W6: Missing AgentCleanup event. Added to FsmEvent enum.
    W7: Verify step separation (3I.2 check vs 3I.3 test). Kept as-is (compile-check is fast gate, test is full verification -- acceptable two-step).
  3 INFOs noted:
    I1: EXIT CRITERIA HITL is walking skeleton milestone, not phase-specific. Added clarifying note.
    I2: Grounding log correction notation adequate. No change needed.
    I3: ollama PRE_GATE in 3I.1 unrelated to madsim. Removed.
