H-22 trading metamorphic null-test COMPOSITE — decisions (s-8404, 2026-06-05)

WHAT
A new f64-domain Tier-1 fail-closed gate (crates/orchestrator/src/trading_null_gate.rs) that asks:
does a strategy's edge survive a block-permutation null? A genuine edge sits in the tail of the
null distribution (small p); an overfit/spurious edge sits inside the null band (large p) — it is
indistinguishable from a permuted version of itself, i.e. it "survived" the null => BLOCK. Ships the
+TradingNullTest SSB action_type (Pattern-2, 4-arm Kind), a LOCKED mutable=0 seed (FIN-NULL-001),
the `elai null-test` CLI done-gate, a real frozen Tier-2 permutation sidecar, and a contract test.

KEY DECISION 1 — NEW f64 gate, NOT the shipped byte-closure runner (HIGH-1 resolution)
The shipped MetamorphicTestRunner (metamorphic_testing.rs) operates on `Fn(&[u8]) -> Vec<u8>` BYTE
closures (Identity/Inverse/PermutationInvariance); it cannot carry an f64 returns series. Shoehorning
f64 time-series through run_all would not compile cleanly. RESOLUTION (per the integration explorer +
context package): build a NEW f64-domain gate that REUSES the MrViolated SSB row for per-relation MR
failures and the parse_relation naming convention, but implements the evaluation in new f64 code
(GD-1: reuse the framework, do not re-implement the runner). The plan's "add 7 enum arms to the
RelationLibrary" was structurally unsound as written.

KEY DECISION 2 — p-value estimator + K-floor (LOCKED constants)
p = (C + 1) / (K + 1), C = #{null_stat >= observed} (Ojala & Garriga 2010, JMLR 11:1833-1863; sklearn
permutation_test_score form). The +1s are load-bearing: the un-permuted data is itself one element of
the permutation group, so p is never exactly 0 and the estimator is conservative (Phipson & Smyth
2010, PMC6405018). Resolution bound: smallest p = 1/(K+1); to resolve p < alpha need K >=
ceil(1/alpha) - 1. ALPHA=0.05 (LOCKED, the ONLY tunable, TD-2) => K-floor=19. Production default
K_MIN_PERMUTATIONS=999 (p_min=0.001 << alpha). An under-powered null (K < floor) is NOT evidence of
significance => fail-closed Block.

KEY DECISION 2b — the statistic is signal-return ALIGNMENT, not a bare returns Sharpe (build-time fix)
First implementation used Sharpe(returns) as the observed statistic. That is WRONG: a plain Sharpe
(mean/std) is ORDER-INVARIANT — block-permuting returns reorders the same multiset, so mean/std (hence
Sharpe) is identical for every permutation => observed == every null value => p ≈ 1 => the gate could
never distinguish genuine from overfit (the genuine fixture returned p=0.452, EdgeSurvivedNull). The
permutation null only works on an ALIGNMENT-dependent statistic (Ojala & Garriga Test-1: the predictor
is held fixed, the outcome is permuted). FIX: the input carries `signals` (the strategy's per-bar
position, the FIXED predictor) AND `returns` (the market outcome). The observed statistic is the Sharpe
of the strategy P&L stream pnl[t] = signal[t]*return[t]; the null block-permutes the RETURNS against the
fixed signal. A genuine edge (signal predicts the sign of the bar's return) loses its alignment under
permutation and collapses; a static-bias edge (constant-long on a zero-mean market) keeps its
(near-zero) Sharpe under any reorder and survives. Demos/fixtures use a sign-alternating market with
VARYING magnitude (i%3+1) so the perfectly-aligned P&L has positive VARIANCE (a constant-magnitude
market would make the aligned P&L a zero-variance constant -> degenerate NaN). Verified live: overfit
p=1.0 (BLOCK), genuine p=0.001 (ALLOW), Rust and the Python sidecar agree.

KEY DECISION 3 — BLOCK permutation, not iid shuffle
Naive iid label-shuffle breaks return autocorrelation and makes the test anticonservative (too many
false positives — Ojala & Garriga block note). The gate block-permutes contiguous chunks
(DEFAULT_BLOCK_SIZE=5) and only randomizes block ORDER, preserving within-block autocorrelation. The
shuffle is a Fisher-Yates over block indices under a seeded splitmix64 RNG (no new crate); the same
ELAI_SEED gives the same null (GD-2 same-env). Series length is preserved (a trailing short block is
kept whole).

KEY DECISION 4 — multiple-testing: N deflates the BONFERRONI leg (which leg N deflates)
N is read from H-17 via TrialLedger::trial_count(&objective, &data_window). TrialCountMissing => Block
(TD-1), matched on the enum directly; NEVER n_treating_missing_as_zero_UNSAFE. The significance bar is
the Bonferroni selection correction p < alpha / N (family-wise error inflates ~ N*alpha when the
strategy was selected from N searched trials). DECISION: in THIS gate N deflates the PERMUTATION
p-value threshold (alpha/N), which is the multiple-testing-correct form for a permutation test (Ojala
& Garriga "relationship to multiple testing"). The H-18 deflation gate separately deflates N via the
Bailey-Lopez de Prado Deflated Sharpe; the two legs are complementary (H-22 = permutation-null,
H-18 = DSR-vs-null), not redundant.

KEY DECISION 5 — SEED leg is genuinely consumed (not wire-dark)
trading_nulls.toml (mutable=0, domain="trading" NOT "finance" — "finance" is not in DOMAIN_KEYWORDS;
FIN-NULL-001) + seed_trading_nulls_rules in project_rule_seeder.rs + a production caller in
runtime_setup.rs (after the H-18 :353 / H-20 :374 blocks). The gate VERDICT is pure Rust LOCKED
constants (ALPHA, K_MIN_PERMUTATIONS, DEFAULT_BLOCK_SIZE), mirroring deflation_gate — it does NOT query
a rules.db predicate at runtime (that would be a new untested path). The seed feeds the vibesec blake3
seed-integrity baseline (the real consumer): a tampered seed fails the integrity check closed.

KEY DECISION 6 — DEFER CostMonotoneConcave (honest residue, grounded)
The sqrt-law of market impact (Toth et al. 2011, arXiv:1105.1694 — NOTE the plan's "arXiv:1004.0425"
is WRONG; that is a quantum-walk paper) holds at INSTITUTIONAL scale only. Retail trades (< ~0.1% ADV)
are in the LINEAR impact regime (Bucci et al. 2019, arXiv:1811.05230). Locking a concave cost
invariant for a retail-scale gate is not supported by the literature. CostMonotoneConcave is documented
as a SIZE-GATED future relation (concave above ~0.1% ADV, linear below); NOT shipped locked this run.
Metamorphic-testing reference: Segura et al. 2016 "A Survey on Metamorphic Testing" (IEEE TSE
42(9):805-824), NOT the unverifiable "Chen et al. taxonomy".

FAIL-CLOSED guards (R1/R2, each a hard Block with an emitted SSB row, guarded at the TOP of the gate)
  1. returns series missing/too short (< MIN_SERIES_LEN=8)         => block_missing_series
  2. observed statistic NaN/degenerate (zero-variance Sharpe)      => block_nan_statistic
  3. K < k_floor(alpha) (under-powered null)                       => block_insufficient_permutations
  4. TrialCountMissing from H-17                                   => block_trial_count_missing (TD-1)
  5. empty null OR any NaN in the null distribution                => block_missing_null / block_nan_statistic
"Cannot prove the edge survives the null" => Block, never a silent pass.

SSB LOCKSTEP (+TradingNullTest, Pattern-2 single-variant + inner 4-arm Kind) — 226/503 -> 227/504
14 sites, all updated in lockstep (mirroring how H-21 LookaheadDifferential was added):
  1. SCHEMA_VERSION 226 -> 227 (ssb.rs, + bump comment block)
  2. SsbActionType::TradingNullTest enum variant (ssb.rs)
  3. as_sql_str arm => "trading_null_test" (ssb.rs)
  4. from_str arm "trading_null_test" => Ok(...) (ssb.rs)
  5. SQLite action_type CHECK constraint 'trading_null_test' (ssb.rs ~4895 — THE 14TH SITE; a miss
     compiles but fails the INSERT at runtime)
  6. SsbPayload::TradingNullTest(TradingNullTestPayload) variant (ssb.rs)
  7. SsbPayload::action_type() arm (ssb.rs)
  8. TradingNullTestKind enum (EdgeCollapsedToNull/EdgeSurvivedNull/PermutationNullFail/Unprovable) +
     TradingNullTestPayload struct (ssb.rs substrate block)
  9. typed-payload test array entry (ssb.rs all_test_payloads)
  10. typed-payload count 423 -> 424 (ssb.rs)
  11. action_type_lock.rs ALL_VARIANTS array entry
  12. action_type_lock.rs TOTAL_VARIANTS 503 -> 504
  13. action_type_lock.rs exhaustive match leg
  14. e45_cross_language_drift_contract.rs:294 "503" -> "504" (+ message :295)
Inner Kind: EdgeCollapsedToNull (passed/Allow), EdgeSurvivedNull (blocked), PermutationNullFail
(blocked, reserved for an explicit permutation-significance miss), Unprovable (blocked, fail-closed).

TIER-2 SIDECAR + cross-lang path taken
permutation_null_sidecar.py is a REAL frozen numpy module (block_permute + build_null_distribution +
permutation_p_value + compute_null_band + a _spot_check), mirroring deflation_sidecar.py's posture
(runnable, frozen, NOT invoked live this run; cloud-only override). CROSS-LANG: deflation_sidecar.py
is NOT registered in cross-lang-pairs.toml (only the 6 wire-protocol pairs are); H-21's
lookahead_differential.rs uses the in-file `// cross-lang-skip:` header. cross-lang-pairs.toml is 1:1
LOCKED with the PairKind enum + golden (the known H-20 gotcha) — registering a new PairKind would force
a locked-enum + golden multi-file change. DECISION: took the cross-lang-skip path — put
`// cross-lang-skip: rust-py-sidecar` on trading_null_gate.rs (line 1) and DID NOT do the locked
PairKind-enum change. The sidecar pair registration is a follow-on (same treatment as the H-18/H-21
sidecars). The sidecar IS frozen in gate-manifest.txt (TD-2/R8: re-lock OUT-OF-BAND).

CLI demo cases (W-1/W-3 done-gate)
`elai null-test --demo-overfit`  => a zero-mean alternating series whose edge SURVIVES the null => BLOCK.
`elai null-test --demo-genuine`  => a strong positive-drift series significant vs the null => ALLOW.
Both register one distinct H-17 trial (N=1 => threshold alpha/N = alpha) on the same ssb.db, open a
real Ssb, and emit a real TradingNullTest row. Registered in args.rs (variant + dispatch +
parse_null_test_args), main.rs (mod + dispatch arm), and orchestrator lib.rs (pub mod).

CSQ (counterfactual, financial logic per CLAUDE.md)
If the K-floor guard were wrong (e.g. off-by-one accepting K below ceil(1/alpha)-1), an under-powered
null would produce a coarse p-value that could spuriously clear alpha/N and ALLOW an overfit strategy.
Guard: k_floor(alpha) returns ceil(1/alpha)-1, the K<floor branch Blocks BEFORE the null is built, and
a unit test pins k_floor(0.05)==19. If trial_count were read via the _UNSAFE accessor, a missing N
would silently become 0 and alpha/0 = +inf would ALLOW everything; guard: the enum is matched directly
and TrialCountMissing Blocks.

RISKS / OPEN
- Threshold calibration is the design's named main risk: alpha + the null-band must catch a real leak
  without flagging a genuine-but-noisy edge. GD-4 (bound the FP rate on >=3 real strategies before
  default-ON) is a Tier-2 follow-up; no real strategies this run — do NOT claim a validated FP rate.
- The permutation test CANNOT prove out-of-sample robustness, absence of leakage, or causality (Ojala &
  Garriga honest residue). It is NECESSARY, not sufficient (TD-5); final allocation stays human Layer-3.
- The relations are mutation-tested by H-27 (TD-4, future plan); not implemented here.

TESTS
crates/contracts/tests/h22_trading_null_contract.rs (8 cases over a real Ssb db) + unit tests in
trading_null_gate.rs (p-value estimator, K-floor, block-permutation determinism, degenerate Sharpe).
SCHEMA assertions use >= (BRITTLE_GLOBAL_SCHEMA_PIN gotcha).
