H-18 DECISIONS
Phase: H-18 Deflated-Sharpe/PBO/MinBTL gate | Session: s-10612 | Date: 2026-06-05

WHAT: Build the #1 net-new trust gate (T-2): a LOCKED COMPOSITE rules-engine gate
that BLOCKS strategy promotion unless Deflated Sharpe Ratio (DSR) >= 0.95 AND
Probability of Backtest Overfitting (PBO) < 0.5 AND T >= MinTRL, with N read from
the H-17 trial ledger and fail-closed on TrialCountMissing or absent PBO.

KEY DECISIONS

D-1: THRESHOLD: DSR >= 0.95 (LOCKED, mutable=0)
    Source: Bailey & Lopez de Prado (2014) SSRN 2460551 + ingested interface map.
    DSR >= 0.95 is the ELAI strategy-level default (conservative).
    Harvey, Liu & Zhu (2016) RFS 29(1):5-68 NBER w20592: t-stat > 3.0 corresponds
    to DSR >= 0.9987 (one-tailed p=0.00135). This is the strict factor-discovery
    threshold. ELAI uses 0.95 as a strategy-level default (less strict, appropriate
    for retail-scale backtests). Harvey-Liu is the stricter alternative; document
    both but LOCK the ELAI default at 0.95.

D-2: THRESHOLD: PBO < 0.5 (LOCKED, mutable=0)
    Source: Bailey, Borwein, Lopez de Prado, Zhu (2015) SSRN 2326253.
    PBO < 0.5 means fewer than half of IS-best selections underperform OOS.
    PBO >= 0.5 means the strategy is likely overfit (randomly guessing IS-best
    would produce the same or better OOS performance). LOCKED.

D-3: FAIL-CLOSED on TrialCountMissing (TD-1)
    The gate BLOCKS immediately on TrialCountMissing (zero trial rows for the scope).
    A reported metric with zero matching trial rows is evidence of an unregistered
    evaluation path. This is a hard FAIL per R1 (missing evidence = FAIL, not SKIP).
    Implementation: match TrialCountResult enum directly; NEVER call
    n_treating_missing_as_zero_UNSAFE from gate code.

D-4: FAIL-CLOSED on absent PBO (R1/R2)
    PBO requires the full T x N return matrix (CSCV). When the Tier-2 sidecar is not
    invoked (pbo=None), the gate BLOCKS immediately. Missing PBO evidence is a hard
    FAIL per R1/R2 (wiring-enforcement.md: "missing evidence is a hard FAIL, never
    SKIP"). The Rust gate receives PBO as a scalar; None = absent = Block.

D-5: TIER SPLIT (W-10)
    Tier-1 (this session, in-process Rust): computes DSR + SR_0 + sigma_sr + MinTRL
    in pure Rust using hand-rolled erf (Abramowitz & Stegun 7.1.26) and Acklam's
    rational approximation for Phi_inv. No new crate (statrs/libm absent from
    Cargo.lock; zero-dep is better for the fail-closed gate substrate).
    Tier-2 (frozen, NOT invoked): deflation_sidecar.py computes PBO via CSCV using
    scipy.stats.norm + numpy CSCV implementation. Registers stub-12 (H18.TIER2).
    The Tier-2 sidecar is frozen in gate-manifest; the live darwin_loop promote hook
    is DEFERRED pending return-matrix store + sidecar HTTP endpoint.

D-6: NORMAL CDF — hand-rolled erf (no new dep)
    The DSR formula needs Phi(x) = 0.5*(1+erf(x/sqrt(2))). Implemented via the
    Abramowitz & Stegun 7.1.26 polynomial approximation for erf(x), maximum absolute
    error < 1.5e-7. Phi_inv uses Acklam (2003) rational approximation, accuracy
    better than 1.15e-9 for p in (0,1). No new Rust crate required. The Tier-2
    Python sidecar uses scipy.stats.norm.cdf/ppf which are more accurate.

D-7: DSR UNIT CONSISTENCY (interface-map failure mode #1)
    sr_observed MUST be per-period (NOT annualized) when passed to sigma_sr.
    T is in the same period units as sr_observed.
    MinTRL output is in the same period units as T (divide by 252 for years).
    MinBTL (companion paper) uses annualized SR and produces years directly; H-18
    uses MinTRL (DSR paper Section 4) for per-period consistency with the Tier-1 gate.

D-8: EXCESS KURTOSIS (interface-map failure mode #2)
    The sigma_sr formula requires EXCESS kurtosis (Fisher=True).
    scipy.stats.kurtosis(x, fisher=True) is correct (default).
    scipy.stats.kurtosis(x, fisher=False) - 3 is WRONG (adds 3 then subtracts 3;
    precision implications; do NOT use this pattern).
    In the Rust Tier-1 gate, excess_kurt is passed directly as a parameter.

D-9: CONSERVATIVE N (N vs Ne, interface-map failure mode #7)
    H-17 trial_count() returns raw N (conservative; overestimates trial independence).
    ONC effective-N (Ne <= N) requires return-series correlation matrix (not in
    strategy_trials). Using raw N is the safe conservative direction: more N =
    HIGHER SR_0 = LOWER DSR = STRICTER gate. Using Ne < N would LOOSEN the gate;
    this requires the ONC return-matrix stub (stub-11) to resolve.

D-10: LOCKED SEED RULE (mutable=0, TD-2)
    Thresholds (DSR_THRESHOLD=0.95, PBO_THRESHOLD=0.5, MINBTL_CONFIDENCE=0.95) are
    named constants in deflation_gate.rs. The FIN-DEFL-001 rule seed is inserted
    with mutable=0 by seed_deflation_rules() in project_rule_seeder.rs. mutable=0
    bypasses CCR (counter-causal-reasoning) auto-retirement (lifecycle.rs:490).
    The rule can only be modified by a human out-of-band change with a schema bump.

D-11: DETERMINISTIC VERDICT (TD-3)
    evaluate_deflation() takes no LLM input parameter. Verdict is pure arithmetic.
    The LLM/auditor reads the verdict as data; it does NOT decide the verdict. The
    gate is the numeric computation, not the LLM's opinion of the computation.

D-12: GD-2 REPRODUCIBILITY SCOPE
    DSR/PBO bit-equality is scoped to same-env/same-seed (floating-point hardware
    differences across machines are physics — TD-5). The verdict (Block/Allow) MUST
    be stable even if the last DSR digit differs. Implementation uses a threshold
    band (dsr < 0.95 not dsr == something), so hardware FP drift does not flip verdicts.

D-13: GD-5 HONEST RESIDUE
    DSR >= 0.95 AND PBO < 0.5 AND T >= MinTRL is NECESSARY but NOT SUFFICIENT for
    a genuine edge. The gate eliminates statistically invalid promotions; it does NOT
    prove the strategy has alpha in live trading. Final capital allocation remains
    human Layer-3 (TD-5). No fabricated guarantee (GD-5): the dashboard tile and
    decisions doc must state this explicitly.

D-14: W-3 PRODUCTION ENTRYPOINT
    promote_after_bench_completed is behind cfg(feature="darwinian-evolve") with no
    live non-test caller. H-18 creates a NEW `elai deflation-check` CLI subcommand
    (crates/cli/src/deflation_check_cmd.rs) as the W-3 production entrypoint. The
    command registers fixture strategies, calls evaluate_deflation, and emits a real
    DeflationGateEvaluated SSB row. The darwin_loop integration is Tier-2 stub-12.

D-15: DOMAIN = "trading" (NOT "finance")
    "finance" is not in DOMAIN_KEYWORDS (rule_engine.rs:206-214). "trading" is
    detected by path keywords ["trading","backtest","signal","pipeline"]. The
    deflation_promotion.toml seed uses domain="trading". The rule fires on promotion
    requests regardless of domain because evaluate_deflation is called directly
    (not via file-path domain matching).

D-16: safedelta_runner.rs LINE CITATION CORRECTION
    The grounding map cited safedelta_runner.rs:454/487 as "frozen-runner + measured-
    row pattern". The actual production fn is at :201 (run_safedelta) and :214
    (run_safedelta_from_runner). The test code at :454/:487 is #[cfg(test)]. Pattern
    is correct; line numbers were a plan-authoring artifact.

EXPECTED TEST OUTCOME
  cargo test -p elai_cc_contracts --test h18_deflation_gate_contract: 8 tests PASS
  cargo test -p elai-orchestrator --test action_type_lock: PASS (TOTAL_VARIANTS 501)
  cargo check --workspace: GREEN

UNKNOWNS
  - N < 5 behavior: Gumbel EV degrades (interface-map FM-4); gate emits WARN not BLOCK
    (conservative: lower N = less deflation = looser gate = safe direction).
  - Cross-hardware FP drift on DSR last digit: scoped by GD-2 (threshold band, D-12).
  - Live trading-strategy return matrix storage: required for live PBO; deferred (stub-12).

REFERENCES
  Bailey & Lopez de Prado (2014) JPM 40(5):94-107, SSRN 2460551 (primary).
  Bailey, Borwein, Lopez de Prado, Zhu (2015) SSRN 2326253 (PBO, MinBTL).
  Harvey, Liu & Zhu (2016) RFS 29(1):5-68, NBER w20592 (t>3.0 = DSR>=0.9987).
  Abramowitz & Stegun (1964) Handbook of Mathematical Functions, Section 7.1.26.
  Acklam, P.J. (2003) Rational approx for the inverse normal CDF.
  workflow/papers/deflated-sharpe-ratio-2460551-interface.md (ingested, HIGH).
