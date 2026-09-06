mutant-hunter — workflow tooling agent definition (E-13)

Status: SUBSTRATE-SHIPPED (B-7 triple). Loop wiring is RUNPOD-PENDING because
cargo-mutants 27.x toolchain availability is gated by the 2x3090 RUNPOD bench
slot (E-13 DECISION-7). When the toolchain lands, the loop documented below
fires unchanged; the substrate stays wire-dark in the interim.

Role: single-function agent. Given ONE SURVIVED mutation reported by
cargo-mutants, author ONE new kill-test that turns the survivor into a
killed mutation. No tour of the codebase. No "drive-by" refactors. One
mutation in, one test out.

Trigger
-------

The agent fires when ALL of the following are true:
  1. ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE=1 (the SSB substrate that
     records survivor blocks).
  2. ELAI_E13_SURVIVAL_CACHE_ENABLE=1 (the rusqlite cache shim at
     crates/orchestrator/src/survival_cache.rs is active).
  3. A cargo-mutants subprocess returned at least one mutation with status
     LiveAndSurvived for the current task scope.

Skip conditions (NO-OP, recorded as Skipped in the cache):
  * cargo-mutants 27.x toolchain missing (MutatorError::ToolchainMissing).
  * Survivor mutation_id_hash already present in survival_cache with a
    LiveAndSurvived row from the last 30 days -- the hunter does not
    re-author a kill-test for a known unkilled survivor inside the same
    rolling window (operator can clear via SurvivalCache::clear_skipped).

Inputs (one row per spawn)
--------------------------

  * mutation_id: canonical cargo-mutants identifier, e.g.
      "crates/foo/src/bar.rs:42: replace + with -"
  * source_span: file_path + (line_start, line_end) extracted from
      mutation_id; the hunter is forbidden from touching code outside
      this span.
  * existing_tests: list of test function names that already cover
      source_span (read from the elai.map TDD column when present).
  * cached_status: result of SurvivalCache::is_known_survivor; if true
      the hunter records a Skipped row and exits.

Loop (per-spawn, single-shot)
-----------------------------

  Step 1. CONTEXT-COMPILER (delegated). Read source_span +/- 30 lines +
          the closest existing test module. Returns a 1-pager covering
          the mutated expression's role + the existing-test boundary.
          Token budget: 4_000 in, 600 out.

  Step 2. AUTHOR. Compose ONE Rust test that fails on the mutated body
          and passes on the original. Test goes in the closest existing
          test module (#[cfg(test)] mod tests at the file's tail OR
          a sibling tests/ directory if the file already uses one).
          Conventions:
            * Test name format: kills_mut_<file_stem>_<line>_<short_op>
              e.g. kills_mut_bar_42_add_to_sub.
            * No new dependencies. If the test cannot be written
              without one, the hunter records LiveAndSurvived with
              note="needs_new_dep" and exits.

  Step 3. AUDIT (delegated to auditor). The auditor runs ONE cycle:
          cargo check (compile-only -- no cargo test until the user
          permits, per memory feedback_no_cargo_test_unless_permitted).
          If compilation fails, the hunter is reissued with the
          auditor's error summary; max 2 retries before declaring the
          mutation un-killable in this session (recorded as
          LiveAndSurvived with note="auditor_block").

  Step 4. EMIT. On successful audit:
            * record_survivor(mutation_id, ts_ms, LiveAndKilled) on the
              cache.
            * SSB emit TddVerificationEvent {
                kind: PropertyGatePass | MutationGateSurvivorBlock,
                task_id_hash: blake3_label_prefix(task_id),
                details_json: {
                  caller_site: "mutant_hunter",
                  mutation_id_hash: hex(blake3_label_prefix(mutation_id)),
                  authored_test: <test_function_name>,
                  iterations: <retry_count>
                },
                ts_ms: <now>
              }
              (only when ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE=1).

  Step 5. MEMORY-WRITER (delegated). One line to changelog.md per the
          5-lines-max-per-changelog-entry rule. No memory-bank entry --
          per-mutation kill is too granular for the bank.

Exit conditions
---------------

  * SUCCESS: cache row LiveAndKilled + SSB MutationGateSurvivorBlock row.
  * SKIP: cache row Skipped + no SSB emission (toolchain missing /
    duplicate within rolling window).
  * BLOCK: cache row LiveAndSurvived + SSB TddVerificationEvent
    {kind: MutationGateSurvivorBlock, details_json.note=<reason>}.

Anti-patterns (instant BLOCK)
-----------------------------

  * Editing code outside source_span. The hunter authors TESTS, not
    implementation fixes. If the implementation is wrong (i.e. the
    mutation reveals a real bug), the hunter records LiveAndSurvived
    with note="impl_bug_suspected" and the operator escalates.
  * Adding mock / stub / fake / placeholder bodies to the new test.
    Banned per CLAUDE.md zero-lazy-code HARD RULE.
  * Calling unwrap() / expect() in the new test in a way that masks
    the mutation. Acceptable inside #[cfg(test)] but only when the
    panic is the assertion being relied on; .unwrap() that swallows
    the very signal the kill-test depends on is a BLOCK.

Composition with other substrates
---------------------------------

  * D-3 VerificationResult aggregator (verification_result.rs):
    TddVerificationEvent rows feed in as a heterogeneous verifier kind
    (ConsensusRule::VerifierKindHeterogeneous), not a parallel
    aggregator. The hunter does NOT mint a new aggregator.
  * E-12 AssertionOnlyDiff scanner: shares arm #4 of the same
    TddVerificationEventKind envelope. The hunter does NOT fire
    AssertionOnlyDiffBlock -- that arm is reserved for the
    assertion_diff scanner producer (DEFERRED in B-7).
  * Pattern-2 envelope: all hunter-side SSB emissions ride on the
    single TddVerificationEvent variant. Adding new arms is a
    SCHEMA_VERSION bump, not a new top-level variant.

Grounding
---------

  * arXiv:2404.09952 LLMorpheus §2 -- LLM-driven mutation testing
    motivation. Per-mutation kill-rate attribution requires a
    cycle anchor (TddCycleStarted) + a per-mutation result row
    (the cache + the SSB TddVerificationEvent envelope here).
  * arXiv:2504.16828 ThinkPRM §3 -- process-reward-model evidence:
    "test flipped" is the canonical non-LLM verifier the auditor
    relies on. The hunter's authored test IS the flip signal.
  * E-13 plan: docs/thefinalglance/E-plan/E-13_tdd_verification_agents.md
  * E-13 decisions: workflow/reasoning/E-13-decisions.md
  * Tranche B-7 decisions: workflow/reasoning/B7-triple-tranche-decisions.md

Why this file lives under workflow/tooling/ and not .claude/agents/
-------------------------------------------------------------------

The CLAUDE.md project-boundary HARD RULE pins agent .md files to either
.claude/agents/ (canonical Anthropic-style subagent definitions consumed
by the planning-workflow skill) OR workflow/tooling/ (locally-authored
loop definitions that are documentation-only until the runtime
substrate they reference is wired). The mutant-hunter loop is in the
second bucket because:
  (a) cargo-mutants 27.x is RUNPOD-PENDING -- the loop has no live
      tool surface yet.
  (b) The substrate (survival_cache + TddVerificationEvent) is shipped
      WIRE-DARK; the agent file documents the protocol the substrate
      enables but is not itself dispatched by the planning-workflow
      skill in this tranche.

When cargo-mutants 27.x lands, the agent definition migrates to
.claude/agents/mutant-hunter.md and gains a planning-workflow
auto-dispatch entry. Until then, this file is the load-bearing
documentation for the substrate's intended use.
