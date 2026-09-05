H-17 decisions -- persisted untamperable trial ledger (T-1, selection-bias keystone)
Session: s-10612 | Date: 2026-06-05 | SCHEMA 222->223 / TOTAL_VARIANTS 499->500
FSM: EXECUTING

WHAT
====
H-17 ships strategy_trials (a new SQLite table on the SSB db) + TrialLedger::register_trial
(the single write choke-point) + a read API (trial_count / effective_trial_count) +
TrialRegistered (Pattern-2 SSB off-agent artifact).

WHY N IS THE KEYSTONE (TB-1)
=============================
An autonomous system that optimises a financial metric ships selection-inflated
Sharpe estimates EVEN IF every agent is perfectly honest. The reason: a search loop
is an unbounded multiple-testing machine. The maximum of N noisy Sharpe ratios is
right-unbounded (Bailey & Lopez de Prado, DSR SSRN 2460551, 2014). Without a
persisted cumulative trial count N, the DSR denominator (H-18) has no honest input,
and every reported Sharpe is gameable by simply running more searches. The "7 trials"
numerical estimate is attributed to Lopez de Prado's Pseudo-Mathematics / CSCV work;
the exact figure is formula-derived (see grounding doc H-17-grounding.md line 66-72)
and is used illustratively -- the structural claim (N must be persisted) is
independently grounded by the published DSR formula itself.

KEY DECISION: ONE CHOKE-POINT, APPEND-ONLY
===========================================
register_trial is the ONLY write path to strategy_trials. There is no
delete_trial, reset_trials, or update_trial method on TrialLedger (TD-1,
by construction). The debug-build update_hook in Ssb::new() is extended to
cover "strategy_trials" alongside "ssb" and "ssb_files" so any rogue
UPDATE/DELETE attempt panics immediately in dev/test builds.

The table's UNIQUE constraint on trial_id ensures idempotent re-registration:
the same (config_hash, data_window, seed) tuple does NOT double-count N.
A DIFFERENT config DOES increment N. This is the invariant the H-18 gate depends on.

KEY DECISION: RAW N vs EFFECTIVE N (ONC)
=========================================
Raw trial_count() returns COUNT(*) over strategy_trials for (objective, data_window).
This is monotone, recomputable from the SSB journal, and never agent-supplied (TD-1).

The ONC effective-N algorithm (Lopez de Prado & Lewis, Quantitative Finance 19(9)
1555-1565, 2019, DOI 10.1080/14697688.2019.1622311) requires an N×N return-series
correlation matrix. strategy_trials stores only scalar metrics (config_hash + seed);
the full return series is not persisted here. Conservative default: Ne = raw N.

Conservative posture rationale: fewer effective trials = LESS deflation = LOOSER gate.
Using raw N therefore over-estimates trial independence (safe upper bound for the
selection-bias penalty). The ONC refinement can only REDUCE Ne, which would make the
gate STRICTER. The stub is registered in trial_ledger.rs (TODO[H17.ONC-STUB]) to be
resolved when the return-matrix store ships alongside H-18's backtest sidecar.

KEY DECISION: SESSION-SCOPE vs PROJECT-SCOPE
=============================================
Ssb is session-scoped (one db per (project_id, session_id)). strategy_trials must
accumulate across sessions. Resolution: TrialLedger opens its own rusqlite::Connection
to the same db_path used by Ssb. SQLite WAL mode supports multiple readers + one writer.
TrialLedger takes BEGIN IMMEDIATE for inserts on its own connection; the two tables
(ssb + strategy_trials) do not contend. This is GD-1 compose: reuse the shipped WAL
infrastructure, no new persistence engine.

KEY DECISION: CHOKE-POINT PRODUCTION ENTRYPOINT (W-3)
======================================================
There is no live financial backtest/strategy-promotion path in the current codebase
(confirmed NN-1 via elai.map grep 2026-06-05; see H-17 exploration). W-10 Tier-1:
the observable_effect entrypoint is the `elai trial-search` CLI subcommand
(crates/cli/src/trial_search_cmd.rs), which runs a fixture search loop registering
>=3 distinct configs and producing >=3 TrialRegistered SSB rows. This IS the real
production entrypoint (the binary runs, calls register_trial, rows land in SSB).

The live autonomous-trading-search integration (Tier-2) is a stub registered in
trial_ledger.rs (TODO[H17.ONC-STUB] mentions the H-18 / backtest sidecar dependency).
H-18 will be the first consumer: it reads trial_count() and fails closed on
TrialCountMissing (zero rows = HARD FAIL, never SKIP).

KEY DECISION: TRIAL_COUNT_MISSING vs REAL-ZERO
===============================================
trial_count() returns TrialCountResult::TrialCountMissing when zero rows exist for
(objective, data_window). This is DISTINCT from returning 0u64 -- the TrialCountMissing
variant is the TD-1 fail-closed sentinel. H-18 matches on this variant and emits a
HARD FAIL for any config with zero matching trial rows. "No rows" means "unregistered
evaluation path" -- a forged metric, not a legitimate result with low N.

TD-2 FREEZE
===========
trial_ledger.rs is added to .claude/gate-manifest.txt (the TD-2 frozen set). Any run
that edits trial_ledger.rs is blocked by gate_freeze_check.py (R8 -- frozen gate).
The --lock re-baseline must be run OUT-OF-BAND by a human after this session (the
executing plan run cannot re-lock its own gate per R8).

Additionally: adding trial_ledger.rs to scripts/locked_keys_registry.toml (the
LockedKeysAuditor pre-commit hook) is a follow-up action for belt-and-suspenders
enforcement at the git-commit level. This is noted here but NOT done in this session
to keep the scope clean (gate-manifest covers the R8 requirement; locked_keys adds the
pre-commit layer).

SCHEMA LOCKSTEP (W-9)
======================
12 sites updated atomically in ssb.rs + action_type_lock.rs + e45_cross_language_drift_contract.rs:
  SITE 1:  enum arm SsbActionType::TrialRegistered added after CanvasViewEvent
  SITE 2:  SQL CHECK list adds 'trial_registered' after 'canvas_view_event'
  SITE 3:  as_sql_str arm Self::TrialRegistered => "trial_registered"
  SITE 4:  from_str arm "trial_registered" => Ok(SsbActionType::TrialRegistered)
  SITE 5:  SsbPayload::TrialRegistered(TrialRegisteredPayload) variant added
  SITE 6:  action_type() arm Self::TrialRegistered(_) => SsbActionType::TrialRegistered
  SITE 7:  all_test_payloads: added TrialRegistered entry, count bumped 419->420
  SITE 8:  action_type_lock.rs TOTAL_VARIANTS: 499->500
  SITE 9:  action_type_lock.rs ALL_VARIANTS array: TrialRegistered appended
  SITE 10: action_type_lock.rs exhaustive_match_guard: TrialRegistered => () arm added
  SITE 11: e45 contract:294 TOTAL_VARIANTS pin: 499->500
  SCHEMA: ssb.rs SCHEMA_VERSION: 222->223
  strategy_trials table: CREATE_STRATEGY_TRIALS_TABLE const + run_migration call
  update_hook: extended to cover "strategy_trials" (debug_assertions)

EXPECTED TEST OUTCOME
=====================
cargo check -p elai-orchestrator -p elai-cli: GREEN (no new deps introduced)
cargo test -p elai_cc_contracts h17_trial_ledger_contract: PASS (7 test functions)
  - register_trial_appends_row_and_emits_ssb: PASS
  - reregistering_same_config_does_not_double_count: PASS
  - different_configs_increment_trial_count_monotonically: PASS
  - trial_count_missing_for_unseen_scope: PASS
  - effective_count_le_raw_count: PASS
  - append_only_update_hook_fires_on_strategy_trials: PASS (debug-only)
  - fixture_trial_search_registers_three_distinct_configs: PASS
  - schema_version_is_at_least_223: PASS
  - trial_registered_in_all_variants_via_round_trip: PASS

UNKNOWNS / OPEN QUESTIONS
==========================
1. ONC effective-N (return-matrix absent): conservative raw N is the default.
   Resolve in H-18 when backtest sidecar + return-matrix store ships.
2. LockedKeysAuditor pre-commit hook: adding trial_ledger.rs to locked_keys_registry.toml
   for belt-and-suspenders protection at commit time. Deferred (out-of-band human step).
3. gate_freeze_check.py --lock re-baseline: REQUIRED after this session (R8).
   Command: cd F:\.ELAI_workflow && python .claude/tools/gate_freeze_check.py --lock

LIVE TRADING LANE TIERING (W-10)
=================================
Tier-1 (shipped here): in-process table + count. The `elai trial-search` CLI fixture
proves the choke-point fires from a real binary entrypoint.
Tier-2 (deferred, H-18): actual backtest sidecar + DSR/PBO gate. register_trial will be
wired into the live eval path when H-18 ships the promotion gate + backtest bridge.
The live autonomous-trading-search integration is separately gated (not wired until H-18).

GROUNDING
=========
- Bailey & Lopez de Prado, Deflated Sharpe Ratio (SSRN 2460551, 2014): N as DSR input
- Lopez de Prado & Lewis, ONC effective-N (Quantitative Finance 19(9), 2019): Ne <= N
- ELAI source: ssb.rs:2999 (SCHEMA_VERSION 222 pre-H-17), action_type_lock.rs:632 (499)
- NN-1: register_trial/strategy_trials/TrialRegistered ABSENT pre-H-17 (grep-confirmed)
- TS-1: SSB append-only WAL (ssb.rs:5 + :14817) grounded, single-writer confirmed
- workflow/papers/onc-lopezdeprado-lewis-2019-interface.md (ONC algorithm, peer-reviewed)
- workflow/papers/deflated-sharpe-ratio-2460551-interface.md (DSR, context for H-18)
