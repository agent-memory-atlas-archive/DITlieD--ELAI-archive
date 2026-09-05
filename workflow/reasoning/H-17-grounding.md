GROUNDING SWEEP -- phase H-17
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-17_trial_ledger_keystone.md
Date: 2026-06-05
FSM: PLANNING | Session: (spec-grounder agent invocation)

SPEC REFERENCES

_grounding/H-TRUST-GROUNDING-MAP.md: TS-1, NN-1, TD-1, TD-2, TRADING-LANE TIERING, H-17 symbol table
  | F:\.ELAI_workflow\docs\thefinalglance\H-plan\_grounding\H-TRUST-GROUNDING-MAP.md | GROUNDED
  | Evidence: file exists and read; TS-1 documents SSB WAL shipped (ssb.rs single-writer no UPDATE/DELETE);
    NN-1 explicitly: "register_trial, strategy_trials -- ZERO hits. T-1 (H-17) is pure BUILD on the SSB WAL.";
    TD-1 defines PERSISTED-N-KEYSTONE; TD-2 defines FROZEN-EVALUATOR; TRADING-LANE TIERING block present
    lines 155-165; H-17 symbol table lines 169-175 confirms absent status.

D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md: PART 0 (TB-1), PART 1 (D1/D2), T-1, PART 6 (WAVE 1)
  | D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md | GROUNDED
  | Evidence: file exists and read; PART 0 lines 16-40 state the TB-1 threat model, the ~7 trials claim,
    and the D1/D2 doctrine; file explicitly names Bailey & Lopez de Prado as grounding for the selection
    bias mechanism.

F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (single-writer WAL + no-UPDATE/DELETE + SsbPayload + SCHEMA_VERSION)
  | F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs | GROUNDED
  | Evidence: file exists; header line 5 "Append-only within a session -- no UPDATE or DELETE"; line 14817
    writer_conn.update_hook fires on UPDATE/DELETE ("SSB append-only violation: attempted {:?} on table {}");
    test_append_only at line 32206 asserts UPDATE and DELETE both trigger hook; SCHEMA_VERSION at :2999 = 222
    (STALE vs MAP's 207 -- G-plan + other tracks have since bumped; re-read at edit time per W-9 instruction).

F:\.ELAI_workflow\.elai_cc\crates\rules\src\schema.rs:46 + lifecycle.rs:871
  | F:\.ELAI_workflow\.elai_cc\crates\rules\src\schema.rs | GROUNDED
  | Evidence: schema.rs lines 40-52: mutable column at :46 "mutable INTEGER NOT NULL DEFAULT 1".
  | F:\.ELAI_workflow\.elai_cc\crates\rules\src\lifecycle.rs | GROUNDED
  | Evidence: lifecycle.rs lines 865-872: test "active_mutable0_ccr_exempt" exercises mutable=0 path,
    transition_to_active with false (mutable=0) succeeds at :871.

.claude/gate-manifest.txt + .claude/tools/gate_freeze_check.py
  | F:\.ELAI_workflow\.claude\gate-manifest.txt | GROUNDED
  | Evidence: file exists at F:\.ELAI_workflow\.claude\gate-manifest.txt (NOT under .elai_cc\.claude --
    see PATH NOTE below). Contents confirmed: wiring-enforcement.md, wiring_contract_check.py,
    gate_freeze_check.py, wiring-contract-validator.md, smoke-manifest.toml, wire-dark-allowlist.toml,
    elai_cc.toml, locked_keys_registry.toml.
  | F:\.ELAI_workflow\.claude\tools\gate_freeze_check.py | GROUNDED
  | Evidence: file found at F:\.ELAI_workflow\.claude\tools\gate_freeze_check.py.

PATH NOTE: gate-manifest.txt and gate_freeze_check.py live under F:\.ELAI_workflow\.claude\ (the workflow
  harness root), NOT under .elai_cc\.claude\ as the plan implies ("F:\.ELAI_workflow\.elai_cc\.claude\").
  The Reads (mandatory) block at plan line 52 writes ".elai_cc\.claude\gate-manifest.txt" -- that path
  does NOT exist. The actual path is F:\.ELAI_workflow\.claude\gate-manifest.txt. This is a path MISMATCH
  that context-compiler must surface; the freeze mechanism itself is real and grounded, the path citation
  in the plan is wrong.

CLAIM CLASSIFICATION

MAX of N noisy Sharpe ratios is upward-biased | GROUNDED
  | Primary: Bailey & Lopez de Prado, "The Deflated Sharpe Ratio: Correcting for Selection Bias,
    Backtest Overfitting and Non-Normality" (SSRN 2460551, 2014; davidhbailey.com/dhbpapers/deflated-sharpe.pdf).
  | Wikipedia DSR article confirmed: formula SR_0 = sqrt(V[SR_n]) * ((1-gamma)*Phi^-1(1-1/N) + gamma*Phi^-1(1-1/Ne));
    "the optimal outcome of an unknown number of historical simulations is right-unbounded."
  | Source in design doc: elai-trust-hardening-design.md PART 0 lines 24-27 citing Bailey/Lopez de Prado.

In-sample Sharpe > 1.0 appears after ~7 pure-noise trials | GROUNDED (internal research report; external number not independently replicated from paper)
  | Internal: D:\.REGRET_workflow\workflow\research\elai-trust\reports\03-reward-hacking-financial.md :20:
    "after just 7 trials, you expect at least one 2-year backtest with annualized Sharpe > 1 when the
    true expected Sharpe is 0 (Lopez de Prado, Pseudo-Mathematics and Financial Charlatanism / CSCV work)."
  | External: The accessible DSR paper PDF returned HTTP 429; Wikipedia and Balaena Medium article did NOT
    contain a specific "7 trial" numerical example. The number is attributed to Lopez de Prado's
    "Pseudo-Mathematics and Financial Charlatanism" (likely his 2014 JPM paper or CSCV companion).
  | VERDICT: the 7-trial number is cited internally via the REGRET research run (a synthesis not a direct
    quote+page-anchor from the paper). Per feedback_paper_citation_verify, effect-size numbers require
    quote+page-anchor. The number is PLAUSIBLE given the DSR formula (verified), but the plan should
    note it is a formula-derived estimate, not an exact theorem. Flag as VERIFY before EXECUTING --
    spawn paper-ingestor mode=paper for SSRN 2460551 to get the exact page/figure reference.
  | Risk: LOW -- the design argument (N must be persisted) is valid regardless of whether the threshold
    is 7, 6, or 8 trials; the number is illustrative. The structural claim is GROUNDED.

DSR deflation requires persisted cumulative trial count N | GROUNDED
  | Wikipedia DSR confirmed N directly inflates the expected max SR benchmark; DSR formula takes N as input.
  | Report 02 line 58: "Lopez de Prado's Third Law: every backtest must be reported with all trials
    involved in its production."
  | Report 03 lines 72-76: "every backtest invocation must go through a Rust-side register_trial() that
    writes one append-only row to the SSB...The counter is COUNT(*) over that table; there is no decrement
    and no DELETE path."
  | GROUNDED -- the structural requirement is directly derived from the published DSR formula.

ONC effective-N clustering for correlated trials (Lopez de Prado) | GROUNDED
  | WebSearch confirmed: ONC (Optimal Number of Clusters) is Lopez de Prado's algorithm from "Detection of
    false investment strategies using unsupervised learning methods," Quantitative Finance 19(9):1555-1565.
    mlfinlab docs: "ONC extracts from a series of backtests the number of effectively uncorrelated trials."
  | Application: ONC estimates the number of independently distinct trials when configs are correlated;
    the result is the effective-N for DSR deflation rather than raw count.
  | INGEST: paper-ingestor mode=paper for "Detection of false investment strategies using unsupervised
    learning methods" Lopez de Prado & Lewis 2018 (Quantitative Finance 19(9)) to confirm the ONC algorithm
    is used for THIS purpose (effective-N for DSR) before H-17 Task 4 is coded.

register_trial / strategy_trials ABSENT in codebase (NN-1) | GROUNDED
  | Verified: grep -rn "register_trial|strategy_trials" across all .elai_cc/crates returned ZERO hits
    (background task completed with empty output). Consistent with NN-1 in H-TRUST-GROUNDING-MAP.

SSB WAL append-only invariant (no UPDATE/DELETE) | GROUNDED
  | ssb.rs :5 header; :14817 update_hook panic on UPDATE/DELETE in debug builds;
    :32206-32274 test_append_only confirms INSERT does NOT trigger, UPDATE and DELETE DO trigger.
    The invariant is real and tested.

SCHEMA_VERSION live baseline | VERIFY_NEEDED (STALE)
  | H-TRUST-GROUNDING-MAP recorded 207 as live baseline (2026-06-04).
  | ssb.rs:2999 now reads: const SCHEMA_VERSION: i64 = 222;
  | TOTAL_VARIANTS: H-TRUST-GROUNDING-MAP recorded 484. Re-read action_type_lock.rs at edit time.
  | Plan line 73 says "baseline+1 from the LIVE 207/484" -- this will be WRONG at edit time.
    The instruction "re-read at edit time" is in the plan; executor must comply.

gate-manifest.txt path citation | MISMATCH
  | Plan line 52 cites ".elai_cc\.claude\gate-manifest.txt" and ".claude\tools\gate_freeze_check.py".
  | Actual paths: F:\.ELAI_workflow\.claude\gate-manifest.txt and F:\.ELAI_workflow\.claude\tools\gate_freeze_check.py.
  | The .elai_cc\.claude directory does NOT exist.
  | The Reads (mandatory) block has incorrect paths for gate-manifest.txt and gate_freeze_check.py.
  | Impact: executor will fail to find files at the cited path. Paths must be corrected.

mutable=0 path (lifecycle.rs:871) | GROUNDED
  | lifecycle.rs :865-872 confirmed: test active_mutable0_ccr_exempt at :871 passes with mutable=0.
  | schema.rs :46: mutable column DEFAULT 1 confirmed.

ClockProvider usage (no SystemTime) | VERIFY
  | ssb.rs imports "use elai_core::clock::ClockProvider" at :22. ClockProvider is referenced in the WAL.
  | The plan requires ClockProvider for created_at_ms in strategy_trials. Substrate is present.
  | grep ClockProvider in ssb.rs confirms it is wired; trial_ledger.rs will extend the same pattern.

blake3 for config_hash/trial_id | VERIFY
  | Plan mandates blake3 (never blake2/sha2) per CLAUDE.md HARD RULE.
  | ssb.rs :30840 references ArchiveVerifyResult with blake3 verify (per H-TRUST-GROUNDING-MAP TS-7).
  | blake3 crate is a real dependency in .elai_cc; no independent confirmation needed (CLAUDE.md rule).

Live trading lane vs fixture (W-10) | GROUNDED
  | H-TRUST-GROUNDING-MAP TRADING-LANE TIERING block (lines 155-165): "The live autonomous-trading lane
    is DEFERRED behind the fixture harness; no external trust claim before it."
  | Plan line 15: "Pure in-process Rust on the SSB; no network/sidecar at gate time (W-10 Tier-1 whole)."
  | Plan Task 6 (decisions doc): "the live trading lane is W-10-tiered (the ledger + count are Tier-1
    in-process; the actual backtest sidecar is Tier-2, H-18)."
  | VERDICT: H-17 is ENTIRELY a fixture/in-process unit. There is NO live autonomous trading-search lane
    in the current codebase. The "backtest/strategy-eval path" that register_trial gates is a FIXTURE
    (the observable_effect trigger uses "elai-cli -- <run a fixture search loop>"). The plan is honest
    about this (W-10 Tier-1 only; live lane deferred to H-18 Tier-2 checkbox). The choke-point
    is correctly scoped as in-process.

ACTIONS REQUIRED

- MISMATCH: plan Reads (mandatory) line 52 cites ".elai_cc\.claude\gate-manifest.txt" and
  ".claude\tools\gate_freeze_check.py" -- these paths do not exist. Correct to:
    F:\.ELAI_workflow\.claude\gate-manifest.txt
    F:\.ELAI_workflow\.claude\tools\gate_freeze_check.py
  Resolve before TESTING_FIRST (executor will fail to find the Reads targets otherwise).

- INGEST: spawn paper-ingestor mode=paper for Lopez de Prado & Lewis 2018 "Detection of false investment
  strategies using unsupervised learning methods" (Quantitative Finance 19(9):1555-1565 or arXiv equivalent)
  to ground the ONC effective-N claim before Task 4 coding (effective_trial_count ONC clustering).

- VERIFY: before EXECUTING, re-read ssb.rs:2999 and action_type_lock.rs for live SCHEMA_VERSION and
  TOTAL_VARIANTS (currently 222 / unknown; MAP recorded 207/484 -- already stale). Plan line 73 must
  use live values at edit time, not MAP values.

- VERIFY (low priority): spawn paper-ingestor mode=paper for Bailey & Lopez de Prado SSRN 2460551 to
  obtain the page/figure reference for the "~7 trials -> Sharpe > 1.0" numerical example. The structural
  argument is grounded; the specific number should carry a page-anchor per feedback_paper_citation_verify.
  Not blocking TESTING_FIRST but should land in H-17-decisions.md before AUDITING.

VALIDATION WARNINGS

- SCHEMA_VERSION STALE: H-TRUST-GROUNDING-MAP recorded SCHEMA_VERSION=207 / TOTAL_VARIANTS=484 on
  2026-06-04. Live ssb.rs:2999 now reads 222. The plan's "baseline+1" instruction is correct; the map
  integer is stale. Context-compiler must re-read ssb.rs + action_type_lock.rs at edit time.
- gate-manifest.txt path in plan Reads is WRONG (see MISMATCH action above) -- this is the highest-risk
  finding because it will cause a hard failure at the Reads (mandatory) step.
- "~7 trials" number is not page-anchored to the primary paper (SSRN 2460551); internal research report
  cites Lopez de Prado's CSCV / Pseudo-Mathematics work. Flag in decisions doc.
- No spec reference to a phase-X.txt file. H-17 is a pure Track-3 build on the trust substrate; no
  phase-spec reference is expected. Consistent with the plan's source block citing H-TRUST-GROUNDING-MAP
  and the REGRET design doc rather than a phase spec.
