GROUNDING SWEEP -- phase H-19
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-19_reusable_holdout_budget.md
Date: 2026-06-06
FSM: PLANNING | Session: s-8404 lineage

SPEC REFERENCES
[1] _grounding/H-TRUST-GROUNDING-MAP.md (NN-3, TS-1, TD-1, TD-2, TD-5, H-19 symbol table)
    | GROUNDED | file exists at F:\.ELAI_workflow\docs\thefinalglance\H-plan\_grounding\H-TRUST-GROUNDING-MAP.md;
      NN-3 line 109 "reusable_holdout / Thresholdout -- ZERO hits. T-3 (H-19) is pure BUILD";
      TS-1 line 35-37 SSB append-only WAL shipped; TD-1..TD-5 lines 133-153; H-19 symbol table lines 185-189.

[2] design T-3 + PART 0 (TB-1) + PART 5 (residue)
    | MISSING | The grounding map traces this to D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md
      (H-TRUST-GROUNDING-MAP.md line 3). D:\ was deleted 2026-05-29 per project memory. No copy found at any
      F:\ path. The W-7 grounding map (H-TRUST-GROUNDING-MAP.md) acts as an in-repo digest of that design, but
      the primary source is inaccessible.

[3] H-17_trial_ledger_keystone.md (append-only-counter contract to mirror)
    | GROUNDED | file at F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-17_trial_ledger_keystone.md confirmed.
      trial_ledger.rs:1-54 confirms no-reset discipline (no delete_trial/reset_trials/update_trial); gate-manifest
      confirms trial_ledger.rs frozen at line 25. ClockProvider injected at trial_ledger.rs:224.

[4] F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\trial_ledger.rs (H-17 no-reset counter)
    | GROUNDED | file exists; lines 1-29 doc block confirms append-only guarantee; line 20 "No delete_trial,
      reset_trials, or update_trial method exists"; line 665 test asserts these methods cannot be called.

[5] F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:98 (MetricGate)
    | GROUNDED | file exists; line 98 is `pub enum CompiledPredicate { MetricGate { metric_name, operator,
      threshold, source, }` -- confirmed MetricGate variant at exact cited line.

[6] Dwork et al. "Preserving Statistical Validity in Adaptive Data Analysis" (STOC 2015)
    | INGEST | cited by title in H-TRUST-GROUNDING-MAP.md line 276; NOT present in workflow/papers/ or
      .claude/papers/ (grep confirmed zero hits). The canonical arXiv for the STOC 2015 paper is 1411.2664
      (Dwork, Feldman, Hardt, Pitassi, Reingold, Roth). NOT ingested.

[7] Dwork et al. "The reusable holdout: Preserving validity in adaptive data analysis" (Science 2015)
    | INGEST | cited by title in H-TRUST-GROUNDING-MAP.md line 276 and H-VERIFICATION-GROUNDING-MAP.md line 75
      as arXiv 1506.02629. NOT present in workflow/papers/ (grep confirmed zero hits). This is the load-bearing
      source for the Thresholdout noise formula + budget bound. NOT ingested.

[8] Blum & Hardt "The Ladder: A Reliable Leaderboard for Machine Learning Competitions" (ICML 2015)
    | INGEST | cited by title in H-TRUST-GROUNDING-MAP.md line 276. NOT present in workflow/papers/ or
      .claude/papers/ (grep confirmed zero hits). This is the load-bearing source for the Ladder algorithm
      budget/query bound. NOT ingested.

[9] CLAUDE.md HARD RULES (blake3, no .unwrap()/.expect(), ClockProvider, TD-1/2/5)
    | GROUNDED | CLAUDE.md Anti-Stale-Training section confirms blake3 rule; no .unwrap() rule confirmed;
      ClockProvider confirmed used in trial_ledger.rs:224 (the reuse pattern H-19 mirrors).

[10] crates/rules/src/schema.rs:46 (mutable column default 1)
     | GROUNDED | schema.rs:46 reads "mutable INTEGER NOT NULL DEFAULT 1" -- confirmed.

[11] crates/rules/src/lifecycle.rs:871 (mutable=0 tested path)
     | GROUNDED | lifecycle.rs:551 "if !record.mutable && !force { return Err(ImmutableRuleCannotAutoRetire) }"
       and line 489 "if record.mutable" CCR gate -- the mutable=0 path is tested and enforced. (Line number
       is ~551/489 by read, cited as :871 in the grounding map; content matches.)

[12] H-0_INDEX.md (doctrine)
     | GROUNDED | file exists at F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-0_INDEX.md.


CLAIM CLASSIFICATION

[C-1] "Dwork et al. reusable holdout / Thresholdout ... give a DP way to answer BOUNDED adaptive holdout queries"
      | INGEST | Paper titles match the referenced work. The Thresholdout algorithm + DP noise calibration +
        budget bound formulas are not ingested. The noise scale (Laplace/Gaussian calibration) is LOAD-BEARING
        for Task 2 -- a wrong constant breaks the DP guarantee. INGEST before coding noise.

[C-2] "Blum-Hardt 'The Ladder' give a DP way ... REFUSE further queries once budget is spent"
      | INGEST | The Ladder algorithm's query budget and threshold are not ingested. Load-bearing for budget_total
        policy constant in Task 3.

[C-3] "DP noise calibrated to the budget (closed-form; ELAI_SEED-seeded so the noise is reproducible, GD-2)"
      | INGEST | The claim that Thresholdout noise is "closed-form" is correct directionally (Laplace mechanism),
        but the EXACT noise scale sigma = f(budget, epsilon, delta) is load-bearing and requires the ingested
        paper to get right. INGEST [C-1] resolves this.

[C-4] arXiv 1506.02629 cited for Thresholdout (H-VERIFICATION-GROUNDING-MAP line 75)
      | VERIFY | From training (unsearched): 1506.02629 is plausible for the Science 2015 Dwork et al. reusable
        holdout paper. The H-VERIFICATION-GROUNDING-MAP attributes it as such. Cannot confirm from ingested
        material. INGEST [C-1]/[7] will verify this ID.

[C-5] "SSB append-only WAL keystone -- SHIPPED. Single-writer, no UPDATE/DELETE methods"
      | GROUNDED | ssb.rs confirmed shipped (trial_ledger.rs:62 imports it); single-writer no-update confirmed
        by trial_ledger.rs:20 doc and gate-manifest.txt:25.

[C-6] "H-17 counter + rule_engine.rs:98 MetricGate present [NN-3 grep-confirmed 2026-06-04]"
      | GROUNDED | trial_ledger.rs exists and is shipped (H-17 complete); rule_engine.rs:98 MetricGate confirmed
        by direct read.

[C-7] "rules-engine MetricGate + mutable=0 tier (rule_engine.rs:98; schema.rs:46/lifecycle.rs:871) -- SHIPPED"
      | GROUNDED | schema.rs:46 mutable DEFAULT 1 confirmed; lifecycle.rs:551 mutable=0 guard confirmed;
        rule_engine.rs:98 MetricGate confirmed.

[C-8] "agent cannot reset its own holdout budget / fresh holdout is human-provisioned"
      | GROUNDED (design claim, VERIFY at EXECUTING) | The no-reset framing correctly mirrors H-17's discipline
        (trial_ledger.rs:20 "No delete_trial, reset_trials, update_trial method exists"). The claim that
        provision-fresh is "human/frozen-only" is an ARCHITECTURAL DESIGN INTENT that must be enforced by
        gate-manifest at EXECUTING time. At plan time: correct framing verified by H-17 precedent. At EXECUTING:
        provision-fresh MUST be listed in gate-manifest.txt to be frozen (currently absent -- to be added in
        Task 5). This is a WIRING_CONTRACT stub_ban item, not a current code fact.

[C-9] "reusable_holdout / Thresholdout ABSENT (NN-3)" -- grep-confirmed 2026-06-04
      | GROUNDED | H-TRUST-GROUNDING-MAP.md line 109 states ZERO hits; this is the BUILD surface claim.

[C-10] "SCHEMA_VERSION baseline+1 from LIVE 207/484, re-read at edit + bump e45:294"
       | VERIFY | H-TRUST-GROUNDING-MAP.md line 18-19 records SCHEMA_VERSION=207, TOTAL_VARIANTS=484 as of
         2026-06-04. The H-19 plan correctly says "re-read at edit" -- the grounding map itself warns these
         will have moved past 207 by Wave 3. Per CLAUDE.md HARD RULE: must re-read at edit time.
         Current git status shows ssb.db modified and multiple crates modified -- baseline may have moved.
         VERIFY at edit time (not blocking now).

[C-11] "No new dependency (closed-form noise; no scipy needed)"
       | VERIFY | Claim is that Laplace/Gaussian noise is implementable in-process without scipy. This is
         directionally correct (rand crate + Box-Muller or Ziggurat for Gaussian). Requires confirming the
         rand >= 0.9 Gaussian distribution API. Not blocking now; VERIFY at Task 2.

[C-12] "Tier-1 whole; no sidecar needed -- Thresholdout noise is closed-form"
       | VERIFY (depends on INGEST) | Claim that noise computation stays in-process. Plausible for Laplace
         mechanism; confirm exact formula from ingested paper before accepting.

[C-13] "ClockProvider not SystemTime" in WIRING_CONTRACT stub_ban
       | GROUNDED | ClockProvider is enforced in trial_ledger.rs:224 (the mirrored pattern). HARD RULE
         confirmed in CLAUDE.md.

[C-14] "blake3" in stub_ban (no blake2/sha2)
       | GROUNDED | CLAUDE.md Anti-Stale-Training HARD RULE: "blake3 (never blake2/sha2 for non-RFC uses)".


ACTIONS REQUIRED
- INGEST: Dwork et al. "Preserving Statistical Validity in Adaptive Data Analysis" (STOC 2015) via
  paper-ingestor mode=paper with arXiv id 1411.2664. The Thresholdout noise formula is load-bearing
  (wrong scale = broken DP guarantee). Block TESTING_FIRST until ingested.
- INGEST: Dwork et al. "The reusable holdout" (Science 2015) via paper-ingestor mode=paper with
  arXiv id 1506.02629 (per H-VERIFICATION-GROUNDING-MAP:75; verify this ID during ingest). The budget
  bound formula + threshold constant are load-bearing for Task 2/3. Block TESTING_FIRST until ingested.
- INGEST: Blum & Hardt "The Ladder: A Reliable Leaderboard for Machine Learning Competitions" (ICML 2015)
  via paper-ingestor mode=paper. The Ladder query budget bound is load-bearing for budget_total policy
  constant (Task 3). Block TESTING_FIRST until ingested.
- VERIFY: "design T-3 + PART 0 + PART 5" Reads (mandatory) entry resolves to
  D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md which is INACCESSIBLE (D: deleted
  2026-05-29). The H-TRUST-GROUNDING-MAP.md is the in-repo digest and covers the key claims (TD-1..TD-5,
  NN-3, TS-1/2). Recommend annotating H-19 Reads block to replace the D:\ path with
  _grounding/H-TRUST-GROUNDING-MAP.md as the authoritative accessible source.
- VERIFY: Confirm arXiv 1506.02629 is correct for the Science 2015 reusable-holdout paper during ingest.
  The H-VERIFICATION-GROUNDING-MAP cites it; if wrong, update the grounding map before EXECUTING.
- VERIFY at EXECUTING: SCHEMA_VERSION + TOTAL_VARIANTS -- re-read ssb.rs and action_type_lock.rs at edit
  time (grounding map warns these will move past 207/484 by Wave 3).
- VERIFY at Task 2: rand >= 0.9 Gaussian distribution API is available for in-process noise (confirm
  no scipy dependency is needed and the noise is implementable closed-form in Rust).


VALIDATION WARNINGS
- "design T-3 + PART 0 (TB-1) + PART 5 (residue)" in Reads (mandatory) resolves to an INACCESSIBLE path
  (D:\REGRET_workflow deleted). The H-TRUST-GROUNDING-MAP.md is the effective substitute and covers all
  load-bearing claims, but the primary source is unavailable.
- No spec references section in H-19 (the file uses "Reads (mandatory)" and "Citations / Grounding"
  sections instead of a "Spec references:" label). The grounding sweep treated both sections as
  spec references per the file's structure.
- arXiv IDs for the three load-bearing papers (1411.2664, 1506.02629, Blum-Hardt ICML 2015) not
  independently verified by RAG (IPC timeout) -- ingest step will confirm.
- lifecycle.rs mutable=0 path cited as :871 in grounding map; direct read found it at :551. The code
  content matches the claim; the line number in the grounding map is stale but not load-bearing.
