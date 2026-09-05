H-29 DECISIONS — LOCKED fail-closed drift state machine (T-13), the master deployment-trust interlock
Session: s-8404. Date: 2026-06-06. SCHEMA_VERSION 237 -> 238, TOTAL_VARIANTS 514 -> 515.

WHAT WAS BUILT
A standalone `elai drift-check` CLI driving a four-axis LOCKED warning->drift->halt interlock with a
persistent journal-derived latch that DEFAULTS to HALTED and can only be cleared by a human/frozen
out-of-band path carrying FRESH re-validation evidence. Composes the shipped drift signals (G-10
model_drift, D-7 canary, H-18 deflation) rather than re-detecting them (GD-1 P0). Emits a
DriftInterlock SSB row (Pattern-2 single action-type, inner 4-arm Kind) on every verdict.

THE FOUR DRIFT AXES (closed-form, NO new dependency)
- AXIS 1 PSI (population stability index) on the INPUT feature distribution AND the OUTPUT/signal
  distribution. PSI = sum_bins (actual_pct - expected_pct) * ln(actual_pct / expected_pct). LOCKED
  bands: PSI_WARN_BAND=0.10 (minor shift, surfaces a Warning, NOT latched), PSI_HALT_BAND=0.25 (major
  shift, HALT). The bands are the conventional credit-risk / ML-monitoring values; they are LOCKED
  named Rust constants mirrored into drift_interlock.toml (mutable=0).
- AXIS 2 schema-fingerprint mismatch: blake3 over the live vendor data schema must equal the FROZEN
  baseline fingerprint. A mismatch = the input contract changed (vendor drift) => HALT. blake3 ONLY
  (never blake2/sha2; mirrors model_drift.rs::identity_key).
- AXIS 3 rolling-14d live-vs-backtest deviation: the absolute deviation of the live realized metric
  from the backtest metric over the DEVIATION_WINDOW_DAYS=14 window (ClockProvider-driven, never
  SystemTime::now()). A deviation beyond DEVIATION_HALT_BAND => HALT; a measurement OLDER than the
  window is itself un-evaluable => HALT.
- AXIS 4 90d validation TTL: the last validation must be NEWER than VALIDATION_TTL_DAYS=90
  (ClockProvider-driven). A stale validation => HALT (re-validation required, forcing an H-18
  deflated-Sharpe re-check at promotion).
- COMPOSED upstream signals (GD-1): G-10 model_drift DriftSignal, D-7 canary regression delta, H-18
  deflation verdict feed in as additional halt inputs via DriftEvidence.composed_breaches. ANY
  non-empty entry latches HALTED. These are READ from the shipped gates, NOT re-implemented.

THE ln(0) / EMPTY-BIN GUARD
A zero (degenerate) bin makes ln(actual/expected) undefined (ln(0) or divide-by-zero). compute_psi is
TOTAL and fail-closed: a length mismatch, an empty binning, or a non-finite input returns
PsiResult::Undefined (the caller HALTs). A zero proportion on either side is floored to PSI_BIN_EPSILON
(1e-6) so the term stays FINITE but LARGE — a zero bin is a real structural shift, so the flooring
inflates the PSI term toward the HALT band rather than producing NaN/inf. The verdict is conservative
(bias toward HALT, the SAFE failure). No NaN/inf ever reaches the band comparison.

THE PERSISTENT JOURNAL-DERIVED LATCH (load-bearing)
The HALTED state is DERIVED from the append-only drift_interlock_latch table, NOT a stored mutable bool
a restart could re-init to Running:
  is_halted(strategy) = COUNT('halt' rows) > COUNT('unlock' rows).
Once a 'halt' row is appended the interlock STAYS HALTED across process restarts until a human/frozen
'unlock' row is appended. Running is the ABSENCE of an unresolved halt, so a process restart re-reads
the journal and re-derives HALTED (verified by the latch_persists_across_reopen contract test). This
mirrors the H-19 queries_spent = COUNT(spent rows) discipline exactly. The table is created lazily by
DriftInterlockLedger::open (same db file as the Ssb WAL); the debug update_hook in Ssb::new panics on
any UPDATE/DELETE against drift_interlock_latch (a covert in-place un-latch).

DEFAULT-HALTED FAIL-SAFE (R1/R2 — the opposite of most gates)
Most gates treat missing evidence as "block the action". An interlock that fails OPEN would keep a
drifting strategy trading, so here missing / un-evaluable evidence => HALT (the SAFE block). A missing
schema fingerprint, an empty PSI bin, an un-readable deviation/validation timestamp all latch HALTED.
Running is reachable ONLY on positive evidence of no-drift across ALL four axes AND no pre-existing
unresolved latch. evaluate_drift_interlock short-circuits to HALT if the strategy is already latched,
BEFORE looking at the fresh evidence.

THE NO-AGENT-UNLOCK 3-LEG (mirror H-19/H-17)
1. NO unlock/reset/clear/resume method on the agent-facing DriftInterlockLedger (negative compile-test
   no_unlock_or_reset_method_exists in both drift_interlock.rs and the h29 contract).
2. the debug update_hook DELETE/UPDATE panic extended to the drift_interlock_latch table.
3. drift_interlock.rs + drift_interlock.toml go in .claude/gate-manifest.txt, re-locked by a HUMAN
   out-of-band (gate_freeze_check.py --lock; the executing run cannot re-lock its own gate, R8).
THE HUMAN UNLOCK is the FREE FUNCTION unlock_drift_interlock (NOT a method), so an agent holding only a
ledger handle cannot reach it. It REQUIRES fresh re-validation evidence: validated_at_ms must be
STRICTLY NEWER than the most recent unresolved 'halt' row (the load-bearing semantic guard, mirroring
provision_fresh_holdout's disjointness guard). A stale-evidence replay is REJECTED
(StaleUnlockEvidence) — a privilege check alone is insufficient.

NAMING
The CLI verb is `drift-check` (a new compound verb). NOT `drift` (E-45 cross-language SOURCE drift,
args.rs Drift{action}) and NOT `model-drift` (G-10, args.rs ModelDrift) — both are taken; clobbering
either was avoided exactly as G-10 disambiguated.

GROUNDED CORRECTION — NO LIVE-TRADING RESUME FSM EDGE EXISTS
The plan repeatedly names "the FSM resume/keep-live transition" as the gated edge (Tasks 3, W-3/W-8).
Three independent agents (grounding sweep, integration explorer GAP-1, context-package HIGH warning)
confirmed by grep that NO live-trading resume FSM transition exists in the codebase — only
workflow-durability "resume" hits (recovery.rs, durability/) and Index/Replay "resume" SSB rows; the
fsm.rs FSM is the agent-WORKFLOW FSM (PLANNING/EXECUTING/AUDITING), unrelated to trading. The two
nearest siblings (H-18 deflation-check, H-19 holdout-check) did NOT modify any FSM transition; their
W-3 entrypoint is an always-compiled CLI over a fixture. H-29 followed that pattern exactly: fsm.rs was
NOT touched, no trading-resume transition was fabricated. The W-1/W-3 Tier-1 entrypoint is
`elai drift-check`; the live-trading resume edge is a DEFERRED Tier-2 forward dependency.

DEFERRED TIER-2 (NOT claimed Tier-1)
- The live-trading-resume FSM gating: no such edge exists yet (forward dep). Tier-1 ships the fixture
  CLI; the live resume hook is a separately-gated Tier-2 checkbox.
- The label-fed change-detectors ADWIN / Page-Hinkley (T3-advisory): these operate ONLY after realized
  labels arrive. No ungrounded detector formula was coded (DDM is the already-ingested precedent if
  needed; ADWIN/Page-Hinkley were NOT ingested and are NOT load-bearing for the four deterministic
  axes). The four axes catch INPUT/OUTPUT/schema/deviation drift; the label-fed half catches concept
  drift ONE cycle late.
- The undecidable P(Y|X) concept-drift residue (a regime break BEFORE labels arrive) is provably
  undetectable without ground truth; it routes to the SHIPPED conformal_calibration.rs as a T3
  ADVISORY (TD-5), NEVER a hard halt.

THE HONEST GUARANTEE (TD-5)
"halt within ONE realized-label cycle", NOT "never trade a regime break". P(Y|X) concept drift before
labels is undecidable. The four deterministic axes are the label-FREE half; the residual is named and
routed to human Layer-3, never advertised as a guarantee the mechanism cannot deliver.

CALIBRATION / DEFAULT-ON (reconciled per the MED warning)
The PSI / 14d-deviation bands are calibration-sensitive (too tight halts on noise, too loose trades
through a real break). The plan biases toward sensitivity because a false HALT is the SAFE failure
(stops trading, requires a re-run). Default-ON is shipped over the FIXTURE (Tier-1) now; the live
default-ON is the DEFERRED Tier-2 checkbox gated on bounding the false-halt rate over >=3 real
deployments (GD-4). This is NOT "default-ON live trading shipped".

PURE-vs-SEEDED
The LOCKED bands are load-bearing named Rust constants in drift_interlock.rs (PSI_WARN_BAND,
PSI_HALT_BAND, DEVIATION_WINDOW_DAYS, DEVIATION_HALT_BAND, VALIDATION_TTL_DAYS). crates/rules/seed/
finance/drift_interlock.toml mirrors them as two mutable=0 rules (FIN-DRIFT-001 PSI MetricGate,
FIN-DRIFT-002 schema SchemaRequired) for the rules-engine echo + the vibesec blake3 seed-integrity
baseline; the toml predicate_json blocks are documentation (the seeder inserts its own PREDICATE_JSON
const, exactly like seed_holdout_budget_rules). seed_drift_interlock_rules is called from
runtime_setup.rs alongside the deflation/holdout seeds (the production caller, idempotent).

GD-2 DETERMINISM
The verdict is byte-identically recomputable: the four axes are closed-form (no RNG); the only entropy
is the clock window (14d/90d), driven by an injected ClockProvider (never SystemTime::now()). The latch
state re-computes from the journal (COUNT('halt') > COUNT('unlock')). The evidence_blake3 binds the
verdict to the breached-axis provenance.

THE 14 SSB LOCKSTEP SITES (live 237/514 -> 238/515, mirrored H-19's HoldoutBudgetEvent footprint)
1 SCHEMA_VERSION 237->238 (+ dated comment block). 2 enum SsbActionType::DriftInterlock. 3 as_sql_str
"drift_interlock". 4 from_str arm. 5 SsbPayload::DriftInterlock. 6 action_type() arm. 7 the payload
struct DriftInterlockPayload + inner 4-arm DriftInterlockKind. 8 action_type_lock.rs TOTAL_VARIANTS
514->515. 9 action_type_lock.rs ALL_VARIANTS append. 10 e45:294 "514"->"515". 11 the round-trip
all_test_payloads arm + the count 434->435. 12 the update_hook table-guard (+drift_interlock_latch).
13 CREATE_DRIFT_INTERLOCK_TABLE DDL (table created lazily by the ledger). 14 the SQLite CHECK
constraint action_type IN (... 'drift_interlock'). PLUS the action_type_lock.rs exhaustive-match arm.
The golden file (action_type_lock.golden) is regenerated via the #[ignore]'d regen_golden_file helper.

GATE-MANIFEST (R8 — HUMAN, OUT-OF-BAND, NOT done by this run)
drift_interlock.rs + drift_interlock.toml must be added to .claude/gate-manifest.txt and re-locked via
`python .claude/tools/gate_freeze_check.py --lock` by a HUMAN out-of-band. The executing run CANNOT
re-lock its own gate (R8). This is intentionally NOT done by the H-29 build session.

FILES TOUCHED
CREATE crates/orchestrator/src/drift_interlock.rs (the four-axis interlock + ledger + evaluate/unlock).
CREATE crates/rules/seed/finance/drift_interlock.toml (the LOCKED bands, mutable=0).
CREATE crates/cli/src/drift_check_cmd.rs (the `elai drift-check` W-1 entrypoint).
CREATE crates/contracts/tests/h29_drift_interlock_contract.rs (the 8 contract cases).
MODIFY crates/orchestrator/src/ssb.rs (the 14-site SSB lockstep + DDL + update_hook + CHECK).
MODIFY crates/orchestrator/src/lib.rs (pub mod drift_interlock).
MODIFY crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS + ALL_VARIANTS + match).
MODIFY crates/orchestrator/src/runtime_setup.rs (the seed_drift_interlock_rules production caller).
MODIFY crates/rules/src/project_rule_seeder.rs (seed_drift_interlock_rules).
MODIFY crates/cli/src/args.rs (the DriftCheck CliAction variant + parse fn + subcommand registration).
MODIFY crates/cli/src/main.rs (the dispatch arm + mod drift_check_cmd).
MODIFY crates/contracts/tests/e45_cross_language_drift_contract.rs (:294 "514"->"515").
