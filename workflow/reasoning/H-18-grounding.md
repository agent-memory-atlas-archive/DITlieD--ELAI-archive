GROUNDING SWEEP -- phase H-18
Source: docs/thefinalglance/H-plan/H-18_deflated_sharpe_pbo_gate.md
Date: 2026-06-05
FSM: PLANNING | Session: (spec-grounder agent; main session FSM state=PLANNING, phase=H-18)

SPEC REFERENCES
_grounding/H-TRUST-GROUNDING-MAP.md | GROUNDED | file exists; TS-2 (rule_engine.rs:98 CompiledPredicate, eval order), TS-8 (safedelta_runner.rs:454/487, InsecureRateMeasured SSB), NN-2 (deflated_sharpe/PBO/MinBTL ABSENT), TD-1/2/3 (persisted-N-keystone, frozen-evaluator, LLM-advisory-only), TRADING-LANE TIERING, H-18 symbol table -- all sections verified present and on-topic (lines 39-183 of grounding map)
workflow/papers/deflated-sharpe-ratio-2460551-interface.md | GROUNDED | file exists; DSR formula Phi((SR-SR_0)/sigma_SR) :33, sigma_SR :37-39, SR_0 Gumbel/Euler-Mascheroni :46-52, MinBTL :65-68, PBO-CSCV S=16 logit lambda :71-82, Harvey-Liu t>3.0 = DSR>=0.9987 :91-95, threshold discussion :91-95
crates/rules/src/rule_engine.rs:98-330 | GROUNDED | file exists; CompiledPredicate enum :98-137 (MetricGate :99, Composite :124); RuleEngine trait :267-299; eval_order fn :327-334 (MetricGate=0, Composite=3); eval-order test assertion :1426-1427 (eval_order(&grep) < eval_order(&composite))
crates/rules/src/schema.rs:46 | GROUNDED | file exists; line 46: "mutable INTEGER NOT NULL DEFAULT 1" -- the mutable column that H-18 sets to 0 for the LOCKED COMPOSITE seed
crates/rules/src/lifecycle.rs:871 | GROUNDED | file exists; line 865-871 test "active_mutable0_ccr_exempt": mutable=0 bypasses CCR gate in transition_to_active (record.mutable check at :490); production gate at :464 "mutable=false exempt from CCR"
crates/orchestrator/src/benchmarks/safedelta_runner.rs:440-490 | GROUNDED (with caveat) | file exists; cited range :440-490 is test code; production frozen-runner pattern is at :201-259 (run_safedelta + run_safedelta_from_runner + bootstrap_ci); the mirror pattern claim is accurate -- Rust orchestrates frozen body, emits SSB row. Line numbers off (test vs production), pattern itself correct.
crates/cli/src/bench_seccodeplt.rs:290-300 | GROUNDED | file exists; lines 291-309 show InsecureRateMeasuredPayload field-copy test -- the deterministic-fixture-harness pattern H-18 mirrors for its own Tier-2 fixture
H-17_trial_ledger_keystone.md | GROUNDED | file exists; trial_count read API contract described; register_trial choke-point, trial_count_missing fail-closed on N==0 -- all present and on-topic

CLAIM CLASSIFICATION
DSR = Phi((SR_obs - SR_0) / sigma_SR) | GROUNDED | interface map :33 assert abs(dsr - norm.cdf((sr_observed - sr_0) / sigma_sr)) < 1e-9
sigma_SR non-normality correction (skew/kurt) | GROUNDED | interface map :37-39 exact formula with excess kurtosis (Fisher=True) and raw skewness; T-1 in denominator confirmed
SR_0 = sharpe_std * maxZ (Gumbel, Euler-Mascheroni gamma=0.5772) | GROUNDED | interface map :46-52 verbatim formula; Wikipedia + Balaena 2024 cross-confirmed; N=1 edge case (SR_0=0) confirmed
MinBTL = (maxZ / sr_annualized)^2 in years (companion paper SSRN 2326253) | GROUNDED | interface map :65-68; note: MinBTL uses ANNUALIZED SR (different from per-period DSR input) -- implementation must not mix
PBO via CSCV: S=16, C(16,8)=12870 pairs, logit lambda = log(R_bar/(1-R_bar)), PBO=P(lambda<0) | GROUNDED | interface map :71-82; CRAN pbo vignette cited as independent confirmation
Harvey-Liu t>3.0 corresponds to DSR >= 0.9987 (one-tailed p=0.00135) | GROUNDED | interface map :91-95 verbatim; sourced to Harvey Liu & Zhu (2016) RFS 29(1):5-68 NBER w20592
Plan default threshold DSR >= 0.95 (conservative relative to Harvey-Liu) | GROUNDED | plan Task 2 states "Harvey-Liu deflated t>3.0 default"; interface map :94-95 documents ELAI H-18 default as DSR>=0.95 with note that DSR>=0.9987 = strict Harvey-Liu compliance. Plan picks 0.95 as the LOCKED default; must document both in H-18-decisions.md per Task 6
Rust Tier-1 gate is normal-CDF-free (compares scalar from Python output) | GROUNDED | Task 2 says Rust "BLOCKs promotion unless dsr clears threshold" -- the Python sidecar computes DSR (float in [0,1]); Rust compares it to a constant threshold. No normal CDF in Rust. No statrs/libm needed. Confirmed by: no statrs/libm/erf in Cargo.lock or any Cargo.toml
PBO (CSCV O(C(S,S/2)*N*T)) is Tier-2 Python-only | GROUNDED | interface map :103 Complexity: PBO O(C(S,S/2)*N*T); DSR/SR_0/MinBTL O(1). The Rust tier enforces thresholds; the Python tier computes all math including the O(12870) CSCV loop
CompiledPredicate::Composite SHIPPED in rule_engine.rs | GROUNDED | rule_engine.rs:124-127 Composite variant present; MetricGate :99-103 present; eval order MetricGate first (key=0), Composite later (key=3) per eval_order :330
mutable=0 tier SHIPPED (schema.rs:46, lifecycle.rs:871) | GROUNDED | schema.rs:46 column definition confirmed; lifecycle.rs:490 `if record.mutable { ... CCR check ... }` -- mutable=false skips CCR; lifecycle.rs:865-871 test confirms bypass
SecCodePltRunner frozen-runner + InsecureRateMeasured SSB pattern SHIPPED | GROUNDED | ssb.rs:8573-8576 InsecureRateMeasured variant; bench_seccodeplt.rs:291-309 fixture test; run_safedelta production fn :201
numpy in sidecar pyproject.toml | GROUNDED | pyproject.toml line 10: "numpy>=1.24" confirmed present
scipy in sidecar pyproject.toml | MISMATCH | pyproject.toml has NO scipy entry; server.py has NO scipy import. Plan Task 1 and ACCEPTANCE say "numpy/scipy already in the sidecar -- no new dep". scipy IS a new dependency and must be added to pyproject.toml before EXECUTING.
N vs Ne (effective independent trials); ONC correction | VERIFY | interface map :84-89 documents Ne concept via ONC paper (Lopez de Prado & Lewis 2019). Plan uses raw N from H-17 trial_count (conservative; overestimates selection bias per interface map :88). ONC clustering noted in H-17 as BUILD. VERIFY: does H-17 plan expose an Ne read API? If not, using N is correct conservative default.
MinBTL annualized-vs-per-period unit risk | VERIFY | interface map :65-68 states MinBTL uses annualized IS Sharpe; interface map failure mode #1 warns about mixing per-period and annualized. The sidecar must accept T in periods but annualize SR for MinBTL internally. Must be verified in sidecar implementation before audit.
N<5 SR_0 formula degradation (Gumbel EV theory) | VERIFY | interface map :112 failure mode 4: "N<5 SR_0 formula degrades; use PSR". Plan does not specify a guard for N<5. Should add a WARN or floor in the sidecar (not a BLOCK) and note it in H-18-decisions.md.
ELAI_SEED bit-identity across hardware (FP precision) | GROUNDED | plan Open questions section explicitly names this risk and scopes GD-2 to same-env/same-seed; uses threshold band not exact equality. Correctly handled.

ACTIONS REQUIRED
- MISMATCH: scipy is NOT in .elai_cc/python/elai_inference/pyproject.toml (only numpy>=1.24 is present; no scipy). The plan's ACCEPTANCE clause "numpy/scipy already in the sidecar -- no new dep" is FALSE. Before EXECUTING: add "scipy>=1.11" to pyproject.toml dependencies AND re-run anti-slopsquat verification (scipy is a real PyPI package -- verify current version: pypi.org/project/scipy). Update ACCEPTANCE to read "scipy added as new dep, anti-slopsquat verified".
- VERIFY: safedelta_runner.rs line citations in plan (and grounding map) point to test code (:454/:487). The production frozen-runner pattern is at :201-259. Not a blocker (pattern is real and correct) but cite lines should be corrected in H-18-decisions.md to :201 (run_safedelta) and :214 (run_safedelta_from_runner).
- VERIFY: confirm whether H-17 exposes a Ne (effective-N via ONC) read API or only raw trial_count(). If Ne is not in H-17's read API, the sidecar must document explicitly that it uses N (conservative) in H-18-decisions.md. Not a BLOCK -- using N is conservative and documented.
- VERIFY: add a guard in the Python sidecar for N<5 (interface map failure mode 4: Gumbel EV theory degrades; return a WARN not a hard BLOCK so the gate is not artificially strict for new strategies). Document the guard in H-18-decisions.md.
- VERIFY: MinBTL computation must annualize SR internally (use T and per-period SR to compute annualized SR before applying MinBTL formula). The sidecar spec must explicitly state the period unit of T and how annualization happens. Add to Task 1 spec comment and H-18-decisions.md section (a).

VALIDATION WARNINGS
- safedelta_runner.rs:454/487 cited in grounding map and plan as "frozen-runner + measured-row pattern" but that range is inside #[cfg(test)]. Production fn is at :201. Pattern is correct; line numbers should be fixed in H-18-decisions.md.
- rule_engine.rs:1427 cited in H-TRUST-GROUNDING-MAP as "eval-order"; the actual line is a test assertion (assert!(eval_order(&grep) < eval_order(&composite))). The production eval_order fn is at :327-334. No impact on correctness.
- DSR threshold ambiguity: plan Task 2 says "Harvey-Liu deflated t>3.0 default" but sets DSR>=0.95. DSR>=0.95 is NOT t>3.0; Harvey-Liu t>3.0 corresponds to DSR>=0.9987. The plan must pick ONE and state it explicitly. Interface map :94-95 confirms the plan's intent is DSR>=0.95 as a less-strict strategy-level default. This is a design choice, not a formula error, but must be explicit in H-18-decisions.md and the LOCKED toml constant.
- all other sources validated
