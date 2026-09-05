H-24 sim-to-real parity gate — decision record (session s-8404, 2026-06-05)

WHAT
A sim-to-real parity gate proving the research/backtest path is the SAME path that
runs live (structural parity) and that a seeded replay reproduces a frozen golden
event log (replay parity). Built as a hybrid across the rules engine (Part A,
seeded) + a pure-Rust diff gate (Part C). The numeric live-vs-backtest
reconciliation (Part B) is DEFERRED Tier-2.

KEY DECISIONS

1. A+C are deterministic pure-build; B is honestly DEFERRED (GD-5).
   Part A (mode-divergent-branch AST ban + single-entrypoint manifest) and Part C
   (replay==golden blake3 EXACT equality) are in-process, fixture-testable WITHOUT
   a live feed. Part B (numeric Sharpe/return/maxDD reconciliation against a live
   paper-trade run) needs a live feed (cloud-only override) and a LOCKED
   reconciliation threshold; it is NOT built here. Shipping B as Tier-1 would
   require a live run we do not have — GD-5 forbids advertising it as live.

2. The mode-divergent-branch AST ban is SCOPED, NOT blanket (LEAN-grounded).
   Grounding: workflow/papers/lean-sim-real-parity-interface.md SECTION 2 (verified
   from QuantConnect LEAN, github.com/QuantConnect/Lean + the QC reconciliation
   docs, 2026-06-05). LeanMode/LiveMode (a.k.a. AlgorithmMode) is a LEGITIMATE
   property whose official doc-comment intent is "notification emails, displaying
   runtime statistics". A blanket ban on `if self.LiveMode:` would
   false-positive-block every legitimate notification/logging conditional. The
   DANGEROUS antipattern is a mode-branch guarding EXECUTION/SIGNAL/ORDER logic
   (e.g. `if self.LiveMode: self.MarketOrder(...)`), which makes the backtest run a
   different code path than live. So FIN-SIMREAL-001's AstPattern carries a
   non-null `scope` field
   ("execution_path:order,signal,fill,execution,strategy,on_data,ondata,initialize")
   that narrows the ban to the order/signal execution path.

   RESIDUAL (honest): the scope is a NAME-based approximation of "reachable from
   order-emission". LEAN's precise scoping intent ("scope to the call-graph
   reachable from order-emission functions") is not expressible in a single
   tree-sitter query + a name allowlist. A notification helper named with an
   "order"/"signal" substring could be in scope (false positive); a mis-named
   execution module could be out of scope (false negative). The scope is bounded
   to the narrowest defensible execution-path pattern. The runtime replay==golden
   gate (Part C) is the SOUND backstop (R5: the AST leg is
   necessary-not-sufficient).

3. The single-entrypoint-with-adapters pattern (LEAN adapter-swap).
   FIN-SIMREAL-002 is a SchemaRequired manifest forcing the strategy to declare ONE
   shared execution entrypoint + its swappable adapters (data_source_adapter,
   fill_model_adapter, clock_adapter). Grounding (LEAN, verified): the strategy
   implements ONE IAlgorithm with Initialize()+OnData(); the ENGINE swaps the
   IBrokerage / ISynchronizer adapter (BacktestingBrokerage vs the live broker;
   Synchronizer vs LiveSynchronizer). Strategy code is NOT the adapter; the engine
   is. Missing manifest => BLOCK: without a declared single entrypoint the engine
   cannot guarantee live and backtest run the same code path.

4. Part C is EXACT blake3 equality, NOT a tolerance band.
   replay==golden is byte-exact hash equality (blake3 either matches or it does
   not). There is NO FpTolerance constant for C — that LOCKED-threshold shape only
   appears in the DEFERRED Part B numeric reconciliation. verify_replay_golden
   COMPOSES (GD-1, restates none): the replay_resume substrate (ssb.rs:24030, the
   live-line; the plan's :23502/:23963 were stale) produces the replay artifact in
   the Tier-2 driver; DriftComparator::compare (determinism.rs) names the first
   divergent event for free (routed through a drop sink so it does not emit a
   duplicate DeterminismDrift row); the RRA 16-hex blake3 convention is reused.

5. The inner SimRealParityKind is 4-arm, not 3 (the plan's Kind list omitted PASS).
   The plan's WIRING_CONTRACT listed a 3-arm Kind (ModeDivergentBranch /
   ReplayGoldenMismatch / SingleEntrypointMissing) — all BLOCK arms. But the plan's
   own test (e) "clean single-entrypoint + matching replay -> Allow" REQUIRES a
   non-block row to emit on success (R4: an SSB row on EVERY decision incl. PASS).
   Added a 4th `Parity` PASS arm so a clean replay emits a Parity row. This was a
   plan gap (flagged in the context package VALIDATION WARNINGS, HIGH).

6. FAIL-CLOSED (R1/R2): missing replay / missing-or-empty golden / event-set
   mismatch / per-event digest mismatch all => Block with an emitted SimRealParity
   row, never a silent pass. blake3(b"") is a fixed constant, so an empty golden
   would hash equal to an empty replay and fail-OPEN — the empty/trivial guard
   (hazard B) rejects before any Parity verdict.

HONEST RESIDUE (TD-5 / the Part-B completion)
Structural parity (A: single entrypoint + no mode-divergent execution branch) +
replay parity (C: byte-reproducible replay) are NECESSARY but NOT SUFFICIENT for
behavioral parity. They eliminate the structural failure mode (different code in
two modes) but do NOT close the residual sim-real gap. Grounding (LEAN, official,
SECTION 3+4 of the interface map):
  - MARKET IMPACT: "LEAN does not currently model market impact, thus fill prices
    can be better during a backtest than in live trading."
  - PARTIAL FILLS: backtest fill models assume COMPLETE fills
    (fill.FillQuantity = order.Quantity); live orders partially fill.
  - LATENCY: backtest submits + fills in the same time slice; live is async
    (10ms..seconds), can span slices.
  - TICK SLICE TIMING: backtest 1ms vs live up to 70ms.
Without Part B, "parity" means only "same code ran + replay reproduced", NOT "same
results". Part B (numeric equity-curve / per-fill / position-drift reconciliation
against a live paper-trade run) is the deferred completion that measures the
residual gap AFTER structural parity is confirmed.

UNGROUNDED CONJECTURE (do NOT assert as a gate)
A min-live-sample requirement (e.g. N>=30 completed round-trips) for the Part-B
numeric reconciliation is NOT documented by QuantConnect. The interface map labels
N>=30 as statistical best practice, explicitly "ungrounded conjecture". Part B,
when built, should carry a min-real-observation fail-closed guard (a parity "pass"
on near-zero live activity is statistically meaningless — analogous to the H-22
K_MIN_PERMUTATIONS under-power BLOCK), but the EXACT N must be grounded against a
primary source before it is locked. For Tier-1 (A+C) the degenerate-real-sample
guard is N/A — A/C read no live trade counts, so "near-zero real activity trivially
matches" cannot occur (there is no real-sample input). Stated explicitly to avoid a
false "we guarded it" claim.

SSB / LOCKSTEP
+1 variant SimRealParity (Pattern-2 single-variant + inner 4-arm SimRealParityKind).
SCHEMA_VERSION 228 -> 229; TOTAL_VARIANTS 505 -> 506; e45:294 505 -> 506. 15
mechanical edits across 4 files (ssb.rs x10 incl the SQLite action_type CHECK
constraint, action_type_lock.rs x3, action_type_lock.golden x1, e45 x1), mirroring
the H-23 RerunHashEquality additions exactly. The plan's body baseline "207/484"
was 21 versions stale; live baseline was 228/505 (confirmed in code at edit time).

FILES
- CREATE crates/orchestrator/src/sim_real_parity.rs (Part C: verify_replay_golden,
  ~720 LOC incl. unit tests).
- CREATE crates/rules/seed/repro/sim_real_parity.toml (Part A: 2 LOCKED rows,
  mutable=0, scoped AST ban + single-entrypoint SchemaRequired).
- MODIFY crates/rules/src/project_rule_seeder.rs (+seed_sim_real_parity_rules).
- MODIFY crates/orchestrator/src/runtime_setup.rs (+production caller).
- MODIFY crates/orchestrator/src/lib.rs (+pub mod sim_real_parity).
- MODIFY crates/orchestrator/src/ssb.rs (+SimRealParity Pattern-2, 10 sites).
- MODIFY crates/orchestrator/tests/action_type_lock.rs (+3 sites, 505->506).
- MODIFY crates/orchestrator/tests/action_type_lock.golden (+1, sorted ordinal 0395).
- MODIFY crates/contracts/tests/e45_cross_language_drift_contract.rs (505->506 + var).
- MODIFY crates/cli/src/args.rs (+VerifyGolden / GoldenDemoMode + --golden parse).
- MODIFY crates/cli/src/rra_cmd.rs (+run_verify_golden, --golden handler).
- CREATE crates/contracts/tests/h24_sim_real_parity_contract.rs (7 cases).

CLI ENTRYPOINT
`elai replay --verify --golden <parity|gap>` — ADDITIVE sub-flag, sibling to H-23's
`--metric-digest`. Does NOT regress the offline `.rra` verifier (Verify { path }) or
the H-23 re-run gate (VerifyRerun). --golden parity => replay==golden => Parity PASS
(exit 0); --golden gap => a divergent fill event => ReplayGoldenMismatch BLOCK
(exit 1), first divergent event named. Fixtures are deterministic in-memory event
logs (mirrors the H-23 fixture pattern) so the gate is testable without a live feed.
