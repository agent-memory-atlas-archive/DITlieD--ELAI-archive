B-15C — Decision Doc
=====================
Phase: B-15C (entropy quarantine: SystemTime/Instant/rand/UUID → ClockProvider/EntropySource)
Session: s-27276

WHAT
----
Replace direct entropy/clock calls with injected provider interfaces, in scope:
  - 6 UUID/ENTROPY sites (excluding sandbox.rs HIGH-risk trait change)
  - Chunk 3+4 TIME sites that are LOW or MEDIUM risk (excluding the 3 HIGH-risk files)
  - Tighten check_entropy_quarantine.sh to fail on any new direct call in non-test code.

DEFERRED (B-15C-HIGH follow-up phase, awaiting user attention for higher-risk surface):
  - crates/orchestrator/src/sandbox.rs — AgentSandbox::create() trait signature change (Windows
    + Linux impls + all callers atomic update needed). 4 UUID sites.
  - crates/orchestrator/src/task_runner.rs — 8 TIME + 1 UUID. Free function signature changes
    propagate widely.
  - crates/dashboard/src/commands.rs — 10 TIME. ExploreCacheEntry.expires_at field type change
    propagates through ExploreCache type alias.

KEY DECISIONS
-------------
D1. Use existing core::ClockProvider and core::EntropySource (from prior B-15C planning the
    context-compiler verified they exist).
D2. Strategy per file:
    STRUCT pattern — add `clock: Arc<dyn ClockProvider>` (and / or `entropy: Arc<dyn EntropySource>`)
      field; constructor takes Arc; existing call sites pass an Arc<SystemClock> default for
      now (no semantic change; new tests can inject deterministic providers).
    FUNCTION pattern — add `clock: &dyn ClockProvider` parameter.
    SHIM pattern — only when serde-deserialized state must remain compatible (e.g.
    research_tree.rs ResearchCache).
D3. Lint script tightens to fail (exit 1) on any non-test, non-example, non-bench `.rs` file
    containing the patterns. Allowlist for cfg(test) inside src/ via existing technique
    from check_hashmap_ban.sh.
D4. NO new dependencies. All work uses existing core types.

EXPECTED TEST OUTCOMES
----------------------
- cargo check --workspace PASS.
- cargo test --no-run on touched crates PASS.
- Lint script reports the 3 HIGH-risk files as remaining violations until B-15C-HIGH lands;
  exit code reflects remaining count. Document expected count post-fix.

UNKNOWNS
--------
- Whether some impacted callers are themselves transitively in HIGH-risk files (cascade).
  Mitigation: agent stops at the boundary of HIGH-risk files; logs cascade as deferred.

ROLLBACK
--------
git restore on touched files. No external state.

ACCEPTANCE
----------
A1: cargo check --workspace PASS.
A2: cargo test --no-run on touched crates PASS.
A3: lint script tightened (catches SystemTime, Instant, chrono::Utc/Local, rand::thread_rng,
    OsRng, from_entropy, SeedableRng::from_entropy, Uuid::new_v4).
A4: B-15C-HIGH follow-up logged for sandbox.rs / task_runner.rs / dashboard/commands.rs.
A5: Remaining violation count in lint script output matches the expected residual (the 3
    HIGH-risk files' contribution: ~22-23 sites).
